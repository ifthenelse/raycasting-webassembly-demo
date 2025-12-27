# Raycasting WebAssembly Demo

Tiny CPU raycaster written in Rust and compiled to WebAssembly via `wasm-bindgen`.
The web frontend uploads textures into WASM memory and blits a shared RGBA
framebuffer to a `<canvas>` each frame. A debug overlay shows a top-down map
and player position; the world map can be regenerated at runtime.

## Features

- **Textured raycasting** with walls, floors, ceilings, and skybox
- **Sprite system** with orientation-based frame selection (Duke Nukem 3D style)
- **Automatic sprite placement** (wall lamps positioned on generated maps)
- **Physics-based movement** with wall sliding and jumping
- **Procedural map generation** with random walls and ceiling sections
- **Real-time rendering** at 320×200 with bilinear texture filtering

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
- `assets/` — images used by the demo (wall/floor/skybox/sprites)

## Sprite System

The engine includes a complete sprite rendering system with automatic orientation-based
frame selection. See the detailed documentation:

- **[SPRITE_SYSTEM.md](SPRITE_SYSTEM.md)** — API reference and technical details
- **[SPRITE_IMPLEMENTATION.md](SPRITE_IMPLEMENTATION.md)** — Implementation summary
- **[SPRITE_ATLAS_FORMAT.md](SPRITE_ATLAS_FORMAT.md)** — How to create sprite atlases

Sprites are rendered with:

- 8-directional viewing angles (like Duke Nukem 3D)
- Distance sorting for correct depth
- Transparency support
- Automatic placement on generated maps

## Texture & Sprite Sizes (recommended)

Practical defaults and recommended ranges for assets used by the demo:

- Sprites (per frame):
  - Typical: 64×64 (retro/fast), 128×128 (good detail), 256×256 (high quality).
  - For vertically tall sprites (lamps, humans) consider 64×128 or 128×256 (width×height).
  - Atlas layout: frames arranged horizontally; one row per sprite type. Example: 8 frames × 128×128 → atlas = 1024×128.
  - Use RGBA PNG with transparency (preferred). If alpha isn't available, keep transparent pixels pure black if you rely on luminance heuristics.

- Wall / Floor / Ceiling textures:
  - Small / retro: 64×64 (matches engine defaults).
  - Good detail: 128×128 or 256×256 (recommended for modern displays).
  - High detail: 512×512 (only if you accept higher memory & CPU cost).
  - Prefer square textures and power-of-two sizes when possible.

- Skybox / panoramic sky:
  - Minimum equirectangular/panoramic: 1024×512.
  - Recommended: 2048×1024 for higher fidelity.
  - High-quality: 4096×2048 if bandwidth/memory allow.
  - If using cubemap faces, prefer square faces (512×512 or 1024×1024 per face).

Notes:

- Choose sprite frame size to match the expected on-screen scale; larger frames reduce pixelation when you approach sprites.
- Keep sprite frame height consistent with the `height` world-unit parameter you pass to `add_sprite()`.
- Use RGBA and check the alpha channel in textures for best transparency handling (the current demo also supports a luminance-based transparency fallback).

## Recent changes (last day)

The demo was extended with a sprite rendering system and related tooling. Summary of developer-facing changes made recently:

- Added a sprite system into the renderer (`src/lib.rs`):
  - `Sprite` struct and static sprite storage.
  - WASM bindings for sprite texture upload: `alloc_sprite_texture()`, `set_sprite_texture_size()`, `get_sprite_texture_ptr()`.
  - Sprite management API: `add_sprite()`, `clear_sprites()`, `get_sprite_count()`.
  - Sprite rendering: viewing-angle frame selection (8 directions), distance sorting (painter's algorithm), bilinear sampling, distance shading and basic transparency handling.
  - Automatic wall-lamp placement during map generation (helper `place_wall_lamps()`).

- Frontend changes (`src/web/App.svelte`):
  - Loads `assets/lamp-wall.png` sprite atlas and uploads it to WASM memory alongside existing textures.
  - Uses `pkg.alloc_sprite_texture()` and `pkg.set_sprite_texture_size()` for allocation.

- Documentation and helper files added:
  - `SPRITE_SYSTEM.md` — API reference and implementation notes.
  - `SPRITE_IMPLEMENTATION.md` — implementation summary.
  - `SPRITE_ATLAS_FORMAT.md` — atlas layout and creation guide.
  - `QUICKSTART_SPRITES.md` — quick start and testing instructions.

These additions are ready to use — place your sprite atlas (e.g. `assets/lamp-wall.png`) and rebuild. See the docs above for atlas layout and recommended sizes.

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
