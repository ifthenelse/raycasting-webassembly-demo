<script>
  import { onMount, onDestroy } from "svelte";
  import * as wasmMod from "../../pkg/wasm_raycaster.js";
  import DebugPanel from "./components/DebugPanel.svelte";
  import { showStats, showSidebar, sidebarWidth, lastSeed } from "./stores.js";

  function startDrag(e) {
    e.preventDefault();
    const startX = e.clientX;
    let startW;
    const unsub = sidebarWidth.subscribe((v) => (startW = v));
    unsub();
    function onMove(ev) {
      const dx = startX - ev.clientX; // dragging left increases width
      const newW = Math.max(220, Math.min(900, startW + dx));
      sidebarWidth.set(newW);
    }
    function onUp() {
      window.removeEventListener("mousemove", onMove);
      window.removeEventListener("mouseup", onUp);
    }
    window.addEventListener("mousemove", onMove);
    window.addEventListener("mouseup", onUp);
  }

  let canvas;
  let mapCanvas;

  let pkg;
  let wasmExports;
  let memory;
  let fbPtr;
  let fb;
  let img;

  let forward = 0;
  let turn = 0;
  let strafe = 0;
  let jump = 0;

  let raf = null;
  let lastFrameTime = performance.now();
  let unbind = null;

  // map overlay state
  let mapView = null;
  let mapPtr = 0;
  let mapW = 0;
  let mapH = 0;

  function ensureViews(w, h) {
    if (!memory) return;
    if (!fb || fb.buffer !== memory.buffer) {
      fb = new Uint8ClampedArray(memory.buffer, fbPtr, w * h * 4);
      img = new ImageData(fb, w, h);
    }
  }

  function updateMapView() {
    try {
      const mw = pkg.get_map_w();
      const mh = pkg.get_map_h();
      const ptr = pkg.get_map_ptr();
      if (!mw || !mh || !ptr) {
        mapView = null;
        return;
      }
      if (
        mapPtr !== ptr ||
        mapW !== mw ||
        mapH !== mh ||
        (mapView && mapView.buffer !== memory.buffer)
      ) {
        mapPtr = ptr;
        mapW = mw;
        mapH = mh;
        mapView = new Int32Array(memory.buffer, mapPtr, mapW * mapH);
      }
    } catch (e) {
      console.warn("updateMapView failed", e);
      mapView = null;
    }
  }

  function drawMapOverlay() {
    updateMapView();
    const ctx = mapCanvas.getContext("2d");
    ctx.clearRect(0, 0, mapCanvas.width, mapCanvas.height);
    if (!mapView) {
      ctx.fillStyle = "#fff";
      ctx.font = "12px monospace";
      ctx.fillText("no map", 8, 16);
      return;
    }
    const cellW = mapCanvas.width / mapW;
    const cellH = mapCanvas.height / mapH;
    for (let y = 0; y < mapH; y++) {
      for (let x = 0; x < mapW; x++) {
        const v = mapView[y * mapW + x];
        ctx.fillStyle = v === 1 ? "#888" : "#111";
        ctx.fillRect(x * cellW, y * cellH, Math.ceil(cellW), Math.ceil(cellH));
      }
    }
    try {
      const px = pkg.get_pos_x();
      const py = pkg.get_pos_y();
      const dx = pkg.get_dir_x();
      const dy = pkg.get_dir_y();
      const cx = px * cellW;
      const cy = py * cellH;
      const size = Math.max(4, Math.min(cellW, cellH) * 0.6);
      ctx.save();
      ctx.translate(cx, cy);
      const angle = Math.atan2(dy, dx);
      ctx.rotate(angle);
      ctx.fillStyle = "#0f0";
      ctx.beginPath();
      ctx.moveTo(size * 0.6, 0);
      ctx.lineTo(-size * 0.4, -size * 0.4);
      ctx.lineTo(-size * 0.4, size * 0.4);
      ctx.closePath();
      ctx.fill();
      ctx.restore();
    } catch (e) {}
  }

  function createWasmBuffer(ptr, width, height, name) {
    const bytes = width * height * 4;
    if (typeof ptr !== "number" || !Number.isFinite(ptr)) {
      throw new Error(`${name} ptr is not a number: ${ptr}`);
    }
    const buf = memory.buffer;
    const available = Math.max(0, buf.byteLength - ptr);
    const len = Math.min(bytes, available);
    return new Uint8Array(buf, ptr, len);
  }

  function bindKeys() {
    const down = (e) => {
      if (e.code === "ArrowUp" || e.code === "KeyW") forward = 1;
      if (e.code === "ArrowDown" || e.code === "KeyS") forward = -1;
      if (e.code === "ArrowLeft" || e.code === "KeyA") turn = 1;
      if (e.code === "ArrowRight" || e.code === "KeyD") turn = -1;
      if (e.code === "Comma") strafe = -1;
      if (e.code === "Period") strafe = 1;
      if (e.code === "Space") jump = 1;
      if (e.code === "KeyM") {
        try {
          pkg.regenerate_map(0);
          setTimeout(() => drawMapOverlay(), 10);
        } catch (e) {}
      }
    };
    const up = (e) => {
      if (e.code === "ArrowUp" || e.code === "KeyW") {
        if (forward > 0) forward = 0;
      }
      if (e.code === "ArrowDown" || e.code === "KeyS") {
        if (forward < 0) forward = 0;
      }
      if (e.code === "ArrowLeft" || e.code === "KeyA") {
        if (turn > 0) turn = 0;
      }
      if (e.code === "ArrowRight" || e.code === "KeyD") {
        if (turn < 0) turn = 0;
      }
      if (e.code === "Comma") {
        if (strafe < 0) strafe = 0;
      }
      if (e.code === "Period") {
        if (strafe > 0) strafe = 0;
      }
      if (e.code === "Space") {
        if (jump > 0) jump = 0;
      }
    };
    window.addEventListener("keydown", down);
    window.addEventListener("keyup", up);
    return () => {
      window.removeEventListener("keydown", down);
      window.removeEventListener("keyup", up);
    };
  }

  onMount(async () => {
    pkg = wasmMod;
    wasmExports = await pkg.default();
    memory = wasmExports.memory;
    window.wasm = pkg;
    pkg.init();

    const w = pkg.get_width();
    const h = pkg.get_height();

    canvas.width = w;
    canvas.height = h;
    canvas.style.width = `${w * 3}px`;
    canvas.style.height = `${h * 3}px`;

    const ctx = canvas.getContext("2d");

    // textures
    async function loadTexture(src) {
      const img = new Image();
      img.src = src;
      await img.decode();
      const c = document.createElement("canvas");
      c.width = img.width;
      c.height = img.height;
      const cx = c.getContext("2d");
      cx.drawImage(img, 0, 0);
      const imgData = cx.getImageData(0, 0, c.width, c.height);
      return { width: img.width, height: img.height, data: imgData.data };
    }

    const [wallTex, floorTex] = await Promise.all([
      loadTexture("../../assets/wall.png"),
      loadTexture("../../assets/floor.png"),
    ]);
    const skyTex = await loadTexture("../../assets/skybox.png");

    // allocate in wasm
    const wallTexPtr = pkg.alloc_wall_texture(wallTex.width, wallTex.height);
    const floorTexPtr = pkg.alloc_floor_texture(
      floorTex.width,
      floorTex.height
    );
    const skyTexPtr = pkg.alloc_sky_texture(skyTex.width, skyTex.height);

    const wallMemory = createWasmBuffer(
      wallTexPtr,
      wallTex.width,
      wallTex.height,
      "wall"
    );
    const wallSrc = new Uint8Array(
      wallTex.data.buffer,
      wallTex.data.byteOffset || 0,
      wallTex.data.byteLength || wallTex.data.length
    );
    wallMemory.set(
      wallSrc.subarray(0, Math.min(wallSrc.length, wallMemory.length))
    );
    pkg.set_wall_texture_size(wallTex.width, wallTex.height);

    const floorMemory = createWasmBuffer(
      floorTexPtr,
      floorTex.width,
      floorTex.height,
      "floor"
    );
    const floorSrc = new Uint8Array(
      floorTex.data.buffer,
      floorTex.data.byteOffset || 0,
      floorTex.data.byteLength || floorTex.data.length
    );
    floorMemory.set(
      floorSrc.subarray(0, Math.min(floorSrc.length, floorMemory.length))
    );
    pkg.set_floor_texture_size(floorTex.width, floorTex.height);

    const skyMemory = createWasmBuffer(
      skyTexPtr,
      skyTex.width,
      skyTex.height,
      "sky"
    );
    const skySrc = new Uint8Array(
      skyTex.data.buffer,
      skyTex.data.byteOffset || 0,
      skyTex.data.byteLength || skyTex.data.length
    );
    skyMemory.set(
      skySrc.subarray(0, Math.min(skySrc.length, skyMemory.length))
    );
    pkg.set_sky_texture_size(skyTex.width, skyTex.height);

    fbPtr = pkg.get_framebuffer_ptr();
    ensureViews(w, h);

    unbind = bindKeys();

    // record the initial seed used by wasm.init() as a best-effort
    try {
      lastSeed.set(Date.now());
    } catch (e) {}

    // expose printMap
    window.printMap = function printMap() {
      try {
        updateMapView();
        if (!mapView) {
          console.log("no map available");
          return;
        }
        const rows = [];
        for (let y = 0; y < mapH; y++) {
          let row = "";
          for (let x = 0; x < mapW; x++)
            row += mapView[y * mapW + x] === 1 ? "#" : ".";
          rows.push(row);
        }
        try {
          const px = pkg.get_pos_x();
          const py = pkg.get_pos_y();
          console.log(`player @ ${px.toFixed(2)}, ${py.toFixed(2)}`);
        } catch (e) {}
        console.log(rows.join("\n"));
      } catch (e) {
        console.error("printMap failed", e);
      }
    };

    // animation frame
    let perfFrameCount = 0,
      perfAccumMs = 0,
      perfMs = 0,
      perfFps = 0;

    function frame() {
      const now = performance.now();
      let dt = (now - lastFrameTime) / 1000;
      if (!isFinite(dt) || dt <= 0) dt = 1 / 60;
      if (dt > 0.1) dt = 0.1;
      lastFrameTime = now;

      const t0 = performance.now();
      pkg.step(forward, turn, strafe, jump, dt);
      const t1 = performance.now();
      const dur = t1 - t0;

      perfAccumMs += dur;
      perfFrameCount++;
      if (perfFrameCount >= 30) {
        perfMs = perfAccumMs / perfFrameCount;
        perfFps = 1000 / (perfMs || 1);
        perfAccumMs = 0;
        perfFrameCount = 0;
      }

      ensureViews(w, h);
      ctx.putImageData(img, 0, 0);
      drawMapOverlay();

      // small overlay (toggleable)
      if ($showStats) {
        ctx.save();
        ctx.fillStyle = "rgba(0,0,0,0.45)";
        ctx.fillRect(4, 4, 120, 20);
        ctx.fillStyle = "#0f0";
        ctx.font = "12px monospace";
        const txt = `${perfFps.toFixed(1)} fps ${perfMs.toFixed(1)} ms`;
        ctx.fillText(txt, 8, 18);
        ctx.restore();
      }

      raf = requestAnimationFrame(frame);
    }

    raf = requestAnimationFrame(frame);
  });

  onDestroy(() => {
    if (raf) cancelAnimationFrame(raf);
    if (typeof unbind === "function") unbind();
  });
</script>

<div id="container">
  <div id="stage">
    <canvas bind:this={canvas} id="screen"></canvas>
  </div>

  {#if $showSidebar}
    <div
      id="sidebarWrapper"
      style="display:flex; flex-direction:column; gap:8px;"
    >
      <aside
        id="sidebar"
        style="width: {$sidebarWidth}px; transition: width 160ms ease; position:relative;"
      >
        <button
          id="sidebarHandle"
          aria-label="Resize sidebar"
          on:mousedown={startDrag}
          title="Drag to resize"
        ></button>
        <DebugPanel {pkg} />
      </aside>
      <canvas bind:this={mapCanvas} id="mapOverlay" width="160" height="160"
      ></canvas>
    </div>
  {:else}
    <button
      id="openSidebar"
      on:click={() => showSidebar.set(true)}
      title="Open debug panel"
      style="position:fixed; right:12px; top:12px; z-index:40; padding:6px 8px;"
      >Open Panel</button
    >
  {/if}
</div>

<style>
  :global(body) {
    margin: 0;
    background: #111;
  }
  #container {
    display: flex;
    gap: 16px;
    align-items: flex-start;
    justify-content: center;
    height: 100vh;
    padding: 12px;
    box-sizing: border-box;
  }
  #stage {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
    flex: 1 1 auto;
  }
  #stage #screen {
    image-rendering: pixelated;
    border: 2px solid #444;
    display: block;
    max-width: calc(100vw - 380px);
    height: auto;
  }
  #mapOverlay {
    width: 160px;
    height: 160px;
    border: 1px solid #333;
    background: #000;
  }
  #sidebarHandle {
    position: absolute;
    left: -8px;
    top: 0;
    bottom: 0;
    width: 12px;
    background: transparent;
    border: 0;
    cursor: ew-resize;
  }
  #sidebar {
    width: 500px;
    max-height: calc(100vh - 24px);
    overflow: auto;
  }
</style>
