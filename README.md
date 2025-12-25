# Raycasting WebAssembly Demo

This project is a tiny CPU raycaster implemented in Rust and compiled to WebAssembly
via `wasm-bindgen`. The web frontend uploads textures into WASM memory and blits a
shared framebuffer to a `<canvas>` each frame.

Quick start

Prerequisites

- Rust toolchain with `wasm32-unknown-unknown` target installed (rustup target add wasm32-unknown-unknown)
- `wasm-pack` installed (https://rustwasm.github.io/wasm-pack/installer/)
- Node.js + npm

1. Install JS dependencies

```bash
npm install
```

2. Build the WebAssembly package

```bash
npm run build:wasm
```

3. Start the frontend (development)

This project uses a small Svelte app inside `www/` served by Vite during development.

```bash
npm run dev
```

Open the URL printed by Vite (usually http://localhost:5173).

4. Production build & preview

```bash
# build wasm + web app
npm run build

# preview the built site (vite preview)
npm run preview
```

Notes

- The WASM package is produced into `pkg/` by `wasm-pack` and imported by the Svelte app.
- You must run `npm run build:wasm` before starting the dev server if `pkg/wasm_raycaster.js` is not present.
- Press `M` or click the "Regenerate Map" button to generate a new random world map at runtime.
- In the browser console you can call `printMap()` to dump the current top-down map (walls as `#`, empty as `.`) and the player position.

Controls / Hotkeys

- Move forward/back: `ArrowUp` / `ArrowDown` or `W` / `S`
- Turn left/right: `ArrowLeft` / `ArrowRight` or `A` / `D`
- Strafe left/right: `,` (Comma) / `.` (Period)
- Jump: `Space`

Developer notes

- The WASM exports used by the frontend are in `src/lib.rs`. Key functions:

  - `get_framebuffer_ptr()` — pointer to RGBA framebuffer bytes
  - `alloc_wall_texture(w,h)` / `alloc_floor_texture(w,h)` — allocate texture
  - `set_*_texture_size(w,h)` — notify actual texture dimensions
  - `init()` / `step(forward, turn, strafe, jump)` — lifecycle & per-frame update

-- The JS glue is now implemented inside the Svelte app at `www/src/App.svelte`. It uploads textures once, builds a `ImageData`
view into wasm memory and calls `putImageData()` each frame. A small overlay shows
FPS and ms/frame for profiling, plus a top-down debug map and controls.
