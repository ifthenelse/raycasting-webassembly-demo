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

    // Allocate buffers in WASM for exact sizes
    const wallTexPtr = pkg.alloc_wall_texture(wallTex.width, wallTex.height);
    const floorTexPtr = pkg.alloc_floor_texture(floorTex.width, floorTex.height);

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
