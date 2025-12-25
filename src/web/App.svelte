<script>
  import { onMount, onDestroy } from "svelte";
  import * as wasmMod from "../../pkg/wasm_raycaster.js";

  let canvas;
  let mapCanvas;
  let regenBtn;

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
      ctx.fillStyle = "#0f0";
      ctx.fillRect(
        (px - 0.25) * cellW,
        (py - 0.25) * cellH,
        Math.ceil(cellW * 0.5),
        Math.ceil(cellH * 0.5)
      );
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

    // regen button handler
    regenBtn.addEventListener("click", () => {
      try {
        pkg.regenerate_map(0);
        setTimeout(() => drawMapOverlay(), 10);
      } catch (e) {
        console.error(e);
      }
    });

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

      // small overlay
      ctx.save();
      ctx.fillStyle = "rgba(0,0,0,0.45)";
      ctx.fillRect(4, 4, 120, 20);
      ctx.fillStyle = "#0f0";
      ctx.font = "12px monospace";
      const txt = `${perfFps.toFixed(1)} fps ${perfMs.toFixed(1)} ms`;
      ctx.fillText(txt, 8, 18);
      ctx.restore();

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
  <canvas bind:this={canvas} id="screen"></canvas>
  <div style="position: absolute; left: 8px; top: 8px;">
    <canvas bind:this={mapCanvas} id="mapOverlay" width="160" height="160"
    ></canvas>
    <div style="margin-top:6px; text-align:left;">
      <button bind:this={regenBtn} style="padding:6px 8px; font:12px monospace;"
        >Regenerate Map</button
      >
    </div>
  </div>
</div>

<style>
  :global(body) {
    margin: 0;
    background: #111;
  }
  #container {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100vh;
  }
  canvas {
    image-rendering: pixelated;
    border: 2px solid #444;
  }
  #mapOverlay {
    width: 160px;
    height: 160px;
  }
</style>
