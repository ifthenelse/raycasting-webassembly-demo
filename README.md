# Raycasting WebAssembly Demo

This project is a tiny CPU raycaster implemented in Rust and compiled to WebAssembly
via `wasm-bindgen`. The web frontend uploads textures into WASM memory and blits a
shared framebuffer to a `<canvas>` each frame.

Quick start

1. Build the WASM package (requires `wasm-pack`):

```bash
wasm-pack build --target web --release
```

2. Serve the `www/` folder (any static server). The project is already importing the "serve" Node module:

```bash
npm run start
```

1. Open the served page (usually http://localhost:3000).

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

- The JS glue is `www/main.js`. It uploads textures once, builds a `ImageData`
  view into wasm memory and calls `putImageData()` each frame. A small overlay shows
  FPS and ms/frame for profiling.
