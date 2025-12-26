# Raycasting WebAssembly Demo

Tiny CPU raycaster written in Rust and compiled to WebAssembly via `wasm-bindgen`.
The web frontend uploads textures into WASM memory and blits a shared RGBA
framebuffer to a `<canvas>` each frame. A debug overlay shows a top-down map
and player position; the world map can be regenerated at runtime.

Prerequisites

- Rust toolchain with the `wasm32-unknown-unknown` target installed:
  `rustup target add wasm32-unknown-unknown`
- `wasm-pack` (https://rustwasm.github.io/wasm-pack/installer/)
- Node.js and npm

Install

```bash
npm install
```

Build & scripts

First install dependencies:

```bash
npm install
```

Available npm scripts (run from project root):

- `npm run build:wasm` — compile the Rust crate to WebAssembly and emit `pkg/` (uses `wasm-pack`).
- `npm run build:web` — bundle the frontend with Vite into `dist/`.
- `npm run build` — run `build:wasm` then `build:web` (production build).
- `npm run dev` — start the guarded dev server (checks port and runs Vite). Use this normally for local development.
- `npm run dev:internal` — run `vite` directly (bypass the guard script).
- `npm run preview` — serve the `dist/` output for a production preview (`vite preview`).
- `npm run test` — run the lightweight JS test harness in `tools/run_tests.js`.
- `npm run format` — format the codebase with Prettier (JS/Svelte/JSON/CSS/HTML/MD).

Notes:

- On a fresh checkout, or after changing Rust dependencies, run `npm run build:wasm` first so `pkg/` exists for the frontend imports. `npm run build` will do both steps.
- Use `npm run dev` for iterative development; the guard script prevents accidentally starting a second dev server on the same port.
- To preview the production bundle after `npm run build`, run `npm run preview` and open the URL shown by Vite.

Project layout

- `src/lib.rs` — Rust crate (raycaster core + wasm-bindgen exports)
- `pkg/` — generated wasm package (output of `wasm-pack`)
- `src/web/` — frontend Svelte app (entry: `src/web/main.js`, component: `src/web/App.svelte`)
- `assets/` — images used by the demo (wall/floor/skybox)

Usage / Controls

- Move forward/back: `ArrowUp` / `ArrowDown` or `W` / `S`
- Turn left/right: `ArrowLeft` / `ArrowRight` or `A` / `D`
- Strafe left/right: `,` / `.`
- Jump: `Space`
- Regenerate map: press `M` or click the "Regenerate Map" button
- Debug: call `printMap()` in the browser console to dump the top-down map

Developer notes

- WASM exports (see `src/lib.rs`): `get_framebuffer_ptr()`, texture allocators,
  `init()`, `step(...)`, `get_map_ptr()`, `regenerate_map()` and helpers used by
  the frontend to access and update the framebuffer.
- Frontend: `src/web/App.svelte` uploads textures, builds `ImageData` views into
  the wasm memory and calls `putImageData()` each animation frame.

Refactor notes

The frontend was moved from `www/src/` into `src/web/` for a flatter repo
structure. `index.html` now references `./src/web/main.js` and asset paths were
adjusted accordingly.

If you run into Svelte runtime/compiler issues, try removing the Vite cache and
restarting the dev server:

```bash
rm -rf node_modules/.vite
npm run dev
```

Contributing

PRs and issues welcome — keep changes focused and run the build/tests before
submitting.

License

This demo has no specified license in `Cargo.toml`.
