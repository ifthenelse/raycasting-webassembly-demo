import * as wasmMod from "../pkg/wasm_raycaster.js";

// main.js: JS/WASM glue
// - Instantiates the wasm module and exposes `pkg` (wasm-bindgen wrapper)
// - Uploads textures into WASM memory once at startup
// - Obtains the framebuffer pointer and builds a shared `ImageData`
// - Calls `pkg.step()` every frame, then blits the framebuffer to the canvas

let forward = 0;
let turn = 0;
let strafe = 0;
let jump = 0;

window.addEventListener("keydown", (e) => {
    if (e.code === "ArrowUp" || e.code === "KeyW") forward = 1;
    if (e.code === "ArrowDown" || e.code === "KeyS") forward = -1;
    if (e.code === "ArrowLeft" || e.code === "KeyA") turn = 1;
    if (e.code === "ArrowRight" || e.code === "KeyD") turn = -1;
    if (e.code === "Comma") strafe = -1;
    if (e.code === "Period") strafe = 1;
    if (e.code === "Space") jump = 1;
});

window.addEventListener("keyup", (e) => {
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
});

(async () => {
    // initialize the wasm module: `wasmExports` is the raw wasm exports (contains `memory`),
    // `pkg` is the JS wrapper with convenience functions. Expose the wrapper on `window.wasm` for tests.
    const pkg = wasmMod;
    const wasmExports = await pkg.default();
    window.wasm = pkg;
    pkg.init();

    // allow external scripts (tests) to request precise rotations via CustomEvent
    window.addEventListener('rotate_by', (ev) => {
        try {
            const angle = ev && ev.detail ? ev.detail : 0.0;
            if (typeof wasm.rotate_by === 'function') wasm.rotate_by(angle);
        } catch (e) {
            console.error('rotate_by handler error', e);
        }
    });

    const w = pkg.get_width();
    const h = pkg.get_height();

    const canvas = document.getElementById("screen");
    canvas.width = w;
    canvas.height = h;
    canvas.style.width = w * 3 + "px";
    canvas.style.height = h * 3 + "px";

    const ctx = canvas.getContext("2d");

    // --- Map overlay: top-down debug view ---
    const mapCanvas = document.createElement("canvas");
    mapCanvas.id = "mapOverlay";
    mapCanvas.style.position = "absolute";
    mapCanvas.style.left = "8px";
    mapCanvas.style.top = "8px";
    mapCanvas.style.width = "160px";
    mapCanvas.style.height = "160px";
    mapCanvas.style.border = "1px solid rgba(255,255,255,0.2)";
    mapCanvas.style.background = "rgba(0,0,0,0.25)";
    mapCanvas.width = 160;
    mapCanvas.height = 160;
    document.body.appendChild(mapCanvas);
    const mapCtx = mapCanvas.getContext("2d");

    // visible button to regenerate map
    const regenBtn = document.createElement('button');
    regenBtn.textContent = 'Regenerate Map';
    regenBtn.style.position = 'absolute';
    regenBtn.style.left = '8px';
    regenBtn.style.top = '176px';
    regenBtn.style.zIndex = '1000';
    regenBtn.style.padding = '6px 8px';
    regenBtn.style.font = '12px monospace';
    regenBtn.addEventListener('click', () => {
        try {
            pkg.regenerate_map(0);
            setTimeout(() => drawMapOverlay(), 10);
        } catch (err) {
            console.error('regenerate_map failed', err);
        }
    });
    document.body.appendChild(regenBtn);

    let mapView = null;
    let mapPtr = 0;
    let mapW = 0;
    let mapH = 0;

    function updateMapView() {
        try {
            const mw = pkg.get_map_w();
            const mh = pkg.get_map_h();
            const ptr = pkg.get_map_ptr();
            if (!mw || !mh || !ptr) {
                mapView = null;
                return;
            }
            // only recreate if sizes/ptr changed or memory grew
            if (mapPtr !== ptr || mapW !== mw || mapH !== mh || (mapView && mapView.buffer !== memory.buffer)) {
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
        mapCtx.clearRect(0, 0, mapCanvas.width, mapCanvas.height);
        if (!mapView) {
            mapCtx.fillStyle = "#fff";
            mapCtx.font = "12px monospace";
            mapCtx.fillText("no map", 8, 16);
            return;
        }
        const cellW = mapCanvas.width / mapW;
        const cellH = mapCanvas.height / mapH;
        for (let y = 0; y < mapH; y++) {
            for (let x = 0; x < mapW; x++) {
                const v = mapView[y * mapW + x];
                if (v === 1) mapCtx.fillStyle = "#888"; else mapCtx.fillStyle = "#111";
                mapCtx.fillRect(x * cellW, y * cellH, Math.ceil(cellW), Math.ceil(cellH));
            }
        }
        // draw player
        try {
            const px = pkg.get_pos_x();
            const py = pkg.get_pos_y();
            mapCtx.fillStyle = "#0f0";
            mapCtx.fillRect((px - 0.25) * cellW, (py - 0.25) * cellH, Math.ceil(cellW * 0.5), Math.ceil(cellH * 0.5));
        } catch (e) { }
    }

    // Regenerate map helper UI: press 'M' to regenerate
    window.addEventListener('keydown', (e) => {
        if (e.code === 'KeyM') {
            try {
                pkg.regenerate_map(0);
                // small timeout to allow wasm memory/view updates
                setTimeout(() => drawMapOverlay(), 10);
            } catch (err) {
                console.error('regenerate_map failed', err);
            }
        }
    });

    // Expose a console helper to pretty-print the current map
    window.printMap = function printMap() {
        try {
            updateMapView();
            if (!mapView) {
                console.log('no map available');
                return;
            }
            const rows = [];
            for (let y = 0; y < mapH; y++) {
                let row = '';
                for (let x = 0; x < mapW; x++) {
                    row += mapView[y * mapW + x] === 1 ? '#' : '.';
                }
                rows.push(row);
            }
            try {
                const px = pkg.get_pos_x();
                const py = pkg.get_pos_y();
                console.log(`player @ ${px.toFixed(2)}, ${py.toFixed(2)}`);
            } catch (e) { }
            console.log(rows.join('\n'));
        } catch (e) {
            console.error('printMap failed', e);
        }
    };

    const memory = wasmExports.memory;
    const fbPtr = pkg.get_framebuffer_ptr();

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
        loadTexture("../assets/wall.png"),
        loadTexture("../assets/floor.png"),
    ]);

    const skyTex = await loadTexture("../assets/skybox.png");

    // Allocate buffers in WASM for exact sizes
    const wallTexPtr = pkg.alloc_wall_texture(wallTex.width, wallTex.height);
    const floorTexPtr = pkg.alloc_floor_texture(floorTex.width, floorTex.height);
    const skyTexPtr = pkg.alloc_sky_texture(skyTex.width, skyTex.height);

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

    const wallMemory = createWasmBuffer(wallTexPtr, wallTex.width, wallTex.height, "wall");
    const wallSrc = new Uint8Array(wallTex.data.buffer, wallTex.data.byteOffset || 0, wallTex.data.byteLength || wallTex.data.length);
    wallMemory.set(wallSrc.subarray(0, Math.min(wallSrc.length, wallMemory.length)));
    pkg.set_wall_texture_size(wallTex.width, wallTex.height);

    const floorMemory = createWasmBuffer(floorTexPtr, floorTex.width, floorTex.height, "floor");
    const floorSrc = new Uint8Array(floorTex.data.buffer, floorTex.data.byteOffset || 0, floorTex.data.byteLength || floorTex.data.length);
    floorMemory.set(floorSrc.subarray(0, Math.min(floorSrc.length, floorMemory.length)));
    pkg.set_floor_texture_size(floorTex.width, floorTex.height);

    const skyMemory = createWasmBuffer(skyTexPtr, skyTex.width, skyTex.height, "sky");
    const skySrc = new Uint8Array(skyTex.data.buffer, skyTex.data.byteOffset || 0, skyTex.data.byteLength || skyTex.data.length);
    skyMemory.set(skySrc.subarray(0, Math.min(skySrc.length, skyMemory.length)));
    pkg.set_sky_texture_size(skyTex.width, skyTex.height);

    let fb = new Uint8ClampedArray(memory.buffer, fbPtr, w * h * 4);
    let img = new ImageData(fb, w, h);
    // Recreate typed views if wasm memory grows
    function ensureViews() {
        if (fb.buffer !== memory.buffer) {
            fb = new Uint8ClampedArray(memory.buffer, fbPtr, w * h * 4);
            img = new ImageData(fb, w, h);
        }
    }

    // Simple performance measurement & overlay
    let perfFrameCount = 0;
    let perfAccumMs = 0;
    let perfMs = 0;
    let perfFps = 0;

    // track wall-clock between frames so physics uses real dt
    let lastFrameTime = performance.now();

    function frame() {
        const now = performance.now();
        let dt = (now - lastFrameTime) / 1000; // seconds
        // clamp dt to avoid physics blowups on tab-switch / pause
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

        ensureViews();
        ctx.putImageData(img, 0, 0);

        // update and draw map overlay
        try { drawMapOverlay(); } catch (e) { /* non-fatal */ }

        // Draw small overlay showing fps / ms/frame
        ctx.save();
        ctx.fillStyle = "rgba(0,0,0,0.45)";
        ctx.fillRect(4, 4, 120, 20);
        ctx.fillStyle = "#0f0";
        ctx.font = "12px monospace";
        const txt = `${perfFps.toFixed(1)} fps ${perfMs.toFixed(1)} ms`;
        ctx.fillText(txt, 8, 18);
        ctx.restore();

        requestAnimationFrame(frame);
    }

    frame();
})();

export { };
