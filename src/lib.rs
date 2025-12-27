use wasm_bindgen::prelude::*;

// Raycaster core (CPU) compiled to WebAssembly and exposed to JS via `wasm-bindgen`.
//
// Integration notes for frontend developers:
// - The shared framebuffer is `FRAMEBUFFER` (RGBA, WIDTH x HEIGHT). JS obtains a
//   pointer via `get_framebuffer_ptr()` and builds a `Uint8ClampedArray`/`ImageData`
//   over wasm memory to blit to a `<canvas>` with `putImageData()`.
// - Textures are uploaded once from JS into WASM memory. JS calls `alloc_*` to
//   get a pointer, then copies `ImageData.data` into the returned pointer. After
//   copying, call `set_*_texture_size(w,h)` so the renderer uses the right dims.
// - Call `init()` once after module instantiation to set initial state and render
//   the first frame. On each animation frame call `step(forward, turn, strafe, jump)`
//   which updates player physics and calls `render_scene()` internally.

const WIDTH: usize = 320;
const HEIGHT: usize = 200;
const FB_SIZE: usize = WIDTH * HEIGHT * 4;

// Runtime-generated map: 1 = wall, 0 = empty
static mut MAP_W: usize = 8;
static mut MAP_H: usize = 8;
static mut WORLD_MAP_VEC: Option<Vec<i32>> = None;
static mut CEILING_MAP_VEC: Option<Vec<i32>> = None;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = Date, js_name = now)]
    fn date_now() -> f64;
}

// Small LCG for deterministic pseudo-randomness seeded with Date.now()
fn lcg_next(seed: &mut u32) -> u32 {
    *seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
    *seed
}

fn generate_map(w: usize, h: usize, mut seed: u32) {
    if w == 0 || h == 0 { return; }
    let mut v = vec![0i32; w * h];
    let mut ceiling_v = vec![0i32; w * h];

    // outer walls
    for x in 0..w {
        v[x] = 1;
        v[(h - 1) * w + x] = 1;
    }
    for y in 0..h {
        v[y * w] = 1;
        v[y * w + (w - 1)] = 1;
    }

    // random interior walls (sparse)
    for y in 1..(h - 1) {
        for x in 1..(w - 1) {
            let r = (lcg_next(&mut seed) % 100) as u8;
            if r < 18 { // ~18% walls
                v[y * w + x] = 1;
            }
        }
    }

    // Add random ceilings in open areas (creates enclosed rooms)
    // Only add ceilings in cells that are not walls themselves
    for y in 2..(h - 2) {
        for x in 2..(w - 2) {
            if v[y * w + x] == 0 {
                let r = (lcg_next(&mut seed) % 100) as u8;
                // ~12% chance of ceiling in empty cells
                if r < 12 {
                    ceiling_v[y * w + x] = 1;
                }
            }
        }
    }

    unsafe {
        MAP_W = w;
        MAP_H = h;
        WORLD_MAP_VEC = Some(v);
        CEILING_MAP_VEC = Some(ceiling_v);
    }
}

static mut FRAMEBUFFER: [u8; FB_SIZE] = [0; FB_SIZE];

// Legacy fixed-size fallback (small). We prefer dynamic vectors allocated at runtime.
const TEXTURE_MAX_SIZE: usize = 256 * 256 * 4; // 256x256 RGBA fallback

static mut WALL_TEX_BUF: [u8; TEXTURE_MAX_SIZE] = [0; TEXTURE_MAX_SIZE];
static mut WALL_TEX_W: usize = 64;
static mut WALL_TEX_H: usize = 64;
static mut FLOOR_TEX_BUF: [u8; TEXTURE_MAX_SIZE] = [0; TEXTURE_MAX_SIZE];
static mut FLOOR_TEX_W: usize = 64;
static mut FLOOR_TEX_H: usize = 64;
static mut SKY_TEX_BUF: [u8; TEXTURE_MAX_SIZE] = [0; TEXTURE_MAX_SIZE];
static mut SKY_TEX_W: usize = 0;
static mut SKY_TEX_H: usize = 0;
static mut CEILING_TEX_BUF: [u8; TEXTURE_MAX_SIZE] = [0; TEXTURE_MAX_SIZE];
static mut CEILING_TEX_W: usize = 64;
static mut CEILING_TEX_H: usize = 64;

// Dynamically allocated texture storages (used when JS requests larger textures)
static mut WALL_TEX_VEC: Option<Vec<u8>> = None;
static mut FLOOR_TEX_VEC: Option<Vec<u8>> = None;
static mut SKY_TEX_VEC: Option<Vec<u8>> = None;
static mut CEILING_TEX_VEC: Option<Vec<u8>> = None;

// Player state
static mut POS_X: f32 = 2.5;
static mut POS_Y: f32 = 2.5;
static mut DIR_X: f32 = -1.0;
static mut DIR_Y: f32 = 0.0;
static mut PLANE_X: f32 = 0.0;
static mut PLANE_Y: f32 = 0.66;
static mut JUMP_OFFSET: f32 = 0.0;
static mut JUMP_VY: f32 = 0.0;
// camera height factor (0.0 top, 1.0 bottom). default 0.6
static mut CAMERA_HEIGHT_FACTOR: f32 = 0.5;

fn map_at(x: i32, y: i32) -> i32 {
    unsafe {
        let w = MAP_W;
        let h = MAP_H;
        if x < 0 || y < 0 || x >= w as i32 || y >= h as i32 {
            1
        } else if let Some(ref v) = WORLD_MAP_VEC {
            v[(y as usize) * w + (x as usize)]
        } else {
            1
        }
    }
}

fn has_ceiling(x: i32, y: i32) -> bool {
    unsafe {
        let w = MAP_W;
        let h = MAP_H;
        if x < 0 || y < 0 || x >= w as i32 || y >= h as i32 {
            false
        } else if let Some(ref v) = CEILING_MAP_VEC {
            v[(y as usize) * w + (x as usize)] != 0
        } else {
            false
        }
    }
}

fn put_pixel(x: i32, y: i32, r: u8, g: u8, b: u8) {
    if x < 0 || y < 0 || x >= WIDTH as i32 || y >= HEIGHT as i32 {
        return;
    }
    let idx = ((y as usize) * WIDTH + (x as usize)) * 4;
    unsafe {
        FRAMEBUFFER[idx] = r;
        FRAMEBUFFER[idx + 1] = g;
        FRAMEBUFFER[idx + 2] = b;
        FRAMEBUFFER[idx + 3] = 255;
    }
}

fn sample_bilinear(tex: &[u8], tex_w: usize, tex_h: usize, u: f32, v: f32) -> (u8, u8, u8) {
    if tex.is_empty() || tex_w == 0 || tex_h == 0 {
        return (100, 100, 100);
    }
    let u = u.fract().abs();
    let v = v.fract().abs();
    let fx = u * (tex_w as f32 - 1.0).max(0.0);
    let fy = v * (tex_h as f32 - 1.0).max(0.0);
    let x0 = (fx.floor() as usize).min(tex_w.saturating_sub(1));
    let y0 = (fy.floor() as usize).min(tex_h.saturating_sub(1));
    let dx = (fx - fx.floor()).max(0.0).min(1.0);
    let dy = (fy - fy.floor()).max(0.0).min(1.0);
    let x1 = (x0 + 1).min(tex_w.saturating_sub(1));
    let y1 = (y0 + 1).min(tex_h.saturating_sub(1));

    let i00 = (y0 * tex_w + x0) * 4;
    let i10 = (y0 * tex_w + x1) * 4;
    let i01 = (y1 * tex_w + x0) * 4;
    let i11 = (y1 * tex_w + x1) * 4;

    // Safely access texture data with bounds checking
    if i11 + 2 >= tex.len() {
        return (100, 100, 100);
    }

    let s00r = tex[i00] as f32;
    let s00g = tex[i00 + 1] as f32;
    let s00b = tex[i00 + 2] as f32;
    let s10r = tex[i10] as f32;
    let s10g = tex[i10 + 1] as f32;
    let s10b = tex[i10 + 2] as f32;
    let s01r = tex[i01] as f32;
    let s01g = tex[i01 + 1] as f32;
    let s01b = tex[i01 + 2] as f32;
    let s11r = tex[i11] as f32;
    let s11g = tex[i11 + 1] as f32;
    let s11b = tex[i11 + 2] as f32;

    let r = s00r * (1.0 - dx) * (1.0 - dy) + s10r * dx * (1.0 - dy) + s01r * (1.0 - dx) * dy + s11r * dx * dy;
    let g = s00g * (1.0 - dx) * (1.0 - dy) + s10g * dx * (1.0 - dy) + s01g * (1.0 - dx) * dy + s11g * dx * dy;
    let b = s00b * (1.0 - dx) * (1.0 - dy) + s10b * dx * (1.0 - dy) + s01b * (1.0 - dx) * dy + s11b * dx * dy;

    ((r.min(255.0) as u8), (g.min(255.0) as u8), (b.min(255.0) as u8))
}

fn apply_shade(r: u8, g: u8, b: u8, shade: f32) -> (u8, u8, u8) {
    let s = shade.max(0.0).min(1.0);
    ((((r as f32) * s) as u8), (((g as f32) * s) as u8), (((b as f32) * s) as u8))
}

fn render_scene() {
    // pixels per meter: scale physical jump height to screen offset
    let jump_px_scale: f32 = 50.0;
    // Camera center determined by CAMERA_HEIGHT_FACTOR (0.0..1.0)
    let center: f32 = (HEIGHT as f32) * unsafe { CAMERA_HEIGHT_FACTOR } + unsafe { JUMP_OFFSET * jump_px_scale };

    // store per-column wall bottom (draw_end) so floor rendering can be clipped to walls
    // store per-column wall top (draw_start) so ceiling rendering can be clipped to walls
    let mut draw_ends: Vec<i32> = vec![HEIGHT as i32; WIDTH];
    let mut draw_starts: Vec<i32> = vec![0; WIDTH];

    let (pos_x, pos_y, dir_x, dir_y, plane_x, plane_y) = unsafe {
        (POS_X, POS_Y, DIR_X, DIR_Y, PLANE_X, PLANE_Y)
    };

    // Fill sky + floor base. For sky pixels we sample the sky texture (if provided)
    // using the ray direction to compute a horizontal wrap (longitude) and the
    // vertical position as latitude.
    for y in 0..HEIGHT as i32 {
        let sky = (y as f32) < center;
        for x in 0..WIDTH as i32 {
            if sky {
                // compute ray for this column
                let camera_x = 2.0 * x as f32 / WIDTH as f32 - 1.0;
                let ray_dir_x = dir_x + plane_x * camera_x;
                let ray_dir_y = dir_y + plane_y * camera_x;

                // convert direction to [0,1] horizontal coordinate
                let angle = ray_dir_y.atan2(ray_dir_x);
                let mut u = angle / (2.0 * std::f32::consts::PI) + 0.5;
                if !u.is_finite() { u = 0.0; }
                // vertical mapping: map screen Y to texture V (0..1)
                // flip so image top corresponds to top of sky area
                let mut v = (y as f32 / center).max(0.0).min(1.0);
                if !v.is_finite() { v = 0.0; }

                unsafe {
                    let sky_slice: &[u8] = if let Some(ref v) = SKY_TEX_VEC { &v[..] } else { &SKY_TEX_BUF[..] };
                    if SKY_TEX_W > 0 && SKY_TEX_H > 0 && !sky_slice.is_empty() {
                        let (r, g, b) = sample_bilinear(sky_slice, SKY_TEX_W, SKY_TEX_H, u, v);
                        put_pixel(x, y, r, g, b);
                    } else {
                        put_pixel(x, y, 80, 120, 200);
                    }
                }
            } else {
                put_pixel(x, y, 60, 60, 60);
            }
        }
    }

    // Render walls
    for x in 0..WIDTH as i32 {
        let camera_x = 2.0 * x as f32 / WIDTH as f32 - 1.0;
        let ray_dir_x = dir_x + plane_x * camera_x;
        let ray_dir_y = dir_y + plane_y * camera_x;

        let mut map_x = pos_x as i32;
        let mut map_y = pos_y as i32;

        let delta_dist_x = if ray_dir_x == 0.0 { f32::INFINITY } else { (1.0 / ray_dir_x).abs() };
        let delta_dist_y = if ray_dir_y == 0.0 { f32::INFINITY } else { (1.0 / ray_dir_y).abs() };

        let (step_x, mut side_dist_x) = if ray_dir_x < 0.0 {
            (-1, (pos_x - map_x as f32) * delta_dist_x)
        } else {
            (1, (map_x as f32 + 1.0 - pos_x) * delta_dist_x)
        };

        let (step_y, mut side_dist_y) = if ray_dir_y < 0.0 {
            (-1, (pos_y - map_y as f32) * delta_dist_y)
        } else {
            (1, (map_y as f32 + 1.0 - pos_y) * delta_dist_y)
        };

        let mut hit = false;
        let mut side = 0;

        while !hit {
            if side_dist_x < side_dist_y {
                side_dist_x += delta_dist_x;
                map_x += step_x;
                side = 0;
            } else {
                side_dist_y += delta_dist_y;
                map_y += step_y;
                side = 1;
            }
            if map_at(map_x, map_y) > 0 {
                hit = true;
            }
        }

        let perp_wall_dist = if side == 0 {
            (map_x as f32 - pos_x + (1 - step_x) as f32 / 2.0) / ray_dir_x
        } else {
            (map_y as f32 - pos_y + (1 - step_y) as f32 / 2.0) / ray_dir_y
        };

        let line_height = (HEIGHT as f32 / perp_wall_dist) as i32;

        let mut draw_start = -line_height / 2 + center as i32;
        if draw_start < 0 {
            draw_start = 0;
        }
        let mut draw_end = line_height / 2 + center as i32;
        if draw_end >= HEIGHT as i32 {
            draw_end = HEIGHT as i32 - 1;
        }

        // Compute wall X (horizontal texture coordinate)
        // This is the exact coordinate where the ray hits the wall
        let wall_x = if side == 0 {
            // Hit vertical wall - use Y coordinate of intersection
            pos_y + perp_wall_dist * ray_dir_y
        } else {
            // Hit horizontal wall - use X coordinate of intersection
            pos_x + perp_wall_dist * ray_dir_x
        };
        let wall_x = wall_x - wall_x.floor();

        // Use wall_x directly without flipping - the fractional position
        // already gives us the correct texture coordinate
        let tex_u = wall_x;

        let shade = (255.0 / (1.0 + perp_wall_dist * 0.3)).min(255.0);
        let shade = if side == 1 { shade * 0.7 } else { shade };
        let shade = shade / 255.0;

        unsafe {
            for y in draw_start..=draw_end {
                // Calculate the exact position in the wall texture
                // Map from screen space to texture space accounting for clipping
                let d = y - center as i32; // distance from horizon to current pixel
                let tex_y_pos = (d - (-line_height / 2)) as f32 / line_height as f32;
                let tex_v = tex_y_pos.max(0.0).min(1.0);
                
                let wall_slice: &[u8] = if let Some(ref v) = WALL_TEX_VEC { &v[..] } else { &WALL_TEX_BUF[..] };
                let (r, g, b) = sample_bilinear(wall_slice, WALL_TEX_W, WALL_TEX_H, tex_u, tex_v);
                let (r, g, b) = apply_shade(r, g, b, shade);
                put_pixel(x, y, r, g, b);
            }
        }
        // record wall top and bottom for this column to prevent floor/ceiling overdrawing
        if x >= 0 && (x as usize) < WIDTH {
            draw_ends[x as usize] = draw_end;
            draw_starts[x as usize] = draw_start;
        }
    }

    // Render floor with perspective-correct casting
    let ray_dir_x0 = dir_x - plane_x;
    let ray_dir_y0 = dir_y - plane_y;
    let ray_dir_x1 = dir_x + plane_x;
    let ray_dir_y1 = dir_y + plane_y;

    let start_y = (center as i32 + 1).max(0);
    for y in start_y..HEIGHT as i32 {
        let p = (y as f32) - center;
        if p == 0.0 {
            continue;
        }
        let row_distance = (0.5 * HEIGHT as f32) / p;

        let floor_step_x = row_distance * (ray_dir_x1 - ray_dir_x0) / WIDTH as f32;
        let floor_step_y = row_distance * (ray_dir_y1 - ray_dir_y0) / WIDTH as f32;

        let mut floor_x = pos_x + row_distance * ray_dir_x0;
        let mut floor_y = pos_y + row_distance * ray_dir_y0;

        let shade = (255.0 / (1.0 + row_distance * 0.15)).min(255.0) / 255.0;

        unsafe {
            for x in 0..WIDTH as i32 {
                let fx = floor_x - floor_x.floor();
                let fy = floor_y - floor_y.floor();
                // skip floor pixels that are behind a wall drawn for this column
                if (y as i32) <= draw_ends[x as usize] {
                    floor_x += floor_step_x;
                    floor_y += floor_step_y;
                    continue;
                }
                let floor_slice: &[u8] = if let Some(ref v) = FLOOR_TEX_VEC { &v[..] } else { &FLOOR_TEX_BUF[..] };
                let (r, g, b) = sample_bilinear(floor_slice, FLOOR_TEX_W, FLOOR_TEX_H, fx, fy);
                let (r, g, b) = apply_shade(r, g, b, shade);
                put_pixel(x, y, r, g, b);
                floor_x += floor_step_x;
                floor_y += floor_step_y;
            }
        }
    }

    // Render ceiling with perspective-correct casting
    // Mirrors floor rendering but for rows above the camera horizon
    let end_y = (center as i32).max(0);
    for y in (0..end_y).rev() {
        let p = center - (y as f32);
        if p <= 0.0 {
            continue;
        }
        let row_distance = (0.5 * HEIGHT as f32) / p;

        let ceiling_step_x = row_distance * (ray_dir_x1 - ray_dir_x0) / WIDTH as f32;
        let ceiling_step_y = row_distance * (ray_dir_y1 - ray_dir_y0) / WIDTH as f32;

        let mut ceiling_x = pos_x + row_distance * ray_dir_x0;
        let mut ceiling_y = pos_y + row_distance * ray_dir_y0;

        let shade = (255.0 / (1.0 + row_distance * 0.15)).min(255.0) / 255.0;

        unsafe {
            for x in 0..WIDTH as i32 {
                let cx = ceiling_x - ceiling_x.floor();
                let cy = ceiling_y - ceiling_y.floor();
                // Check if current position has a ceiling above it
                let ceil_x = ceiling_x as i32;
                let ceil_y = ceiling_y as i32;
                // Only draw ceiling if:
                // 1. The current map cell has a ceiling
                // 2. The current row is not blocked by a wall (above the wall's top)
                if has_ceiling(ceil_x, ceil_y) && (y as i32) < draw_starts[x as usize] {
                    let ceiling_slice: &[u8] = if let Some(ref v) = CEILING_TEX_VEC { &v[..] } else { &CEILING_TEX_BUF[..] };
                    if CEILING_TEX_W > 0 && CEILING_TEX_H > 0 && !ceiling_slice.is_empty() {
                        let (r, g, b) = sample_bilinear(ceiling_slice, CEILING_TEX_W, CEILING_TEX_H, cx, cy);
                        let (r, g, b) = apply_shade(r, g, b, shade);
                        put_pixel(x, y, r, g, b);
                    }
                }
                ceiling_x += ceiling_step_x;
                ceiling_y += ceiling_step_y;
            }
        }
    }
}

#[wasm_bindgen]
pub fn get_width() -> u32 {
    WIDTH as u32
}

#[wasm_bindgen]
pub fn get_height() -> u32 {
    HEIGHT as u32
}

#[wasm_bindgen]
pub fn get_framebuffer_ptr() -> *const u8 {
    #[allow(static_mut_refs)]
    unsafe { FRAMEBUFFER.as_ptr() }
}

#[wasm_bindgen]
pub fn set_camera_height_factor(f: f32) {
    unsafe {
        CAMERA_HEIGHT_FACTOR = f.max(0.0).min(1.0);
    }
}

#[wasm_bindgen]
pub fn get_camera_height_factor() -> f32 {
    unsafe { CAMERA_HEIGHT_FACTOR }
}

#[wasm_bindgen]
pub fn get_rot_speed() -> f32 {
    // rotation speed used in `step` (radians per frame)
    0.03
}

#[wasm_bindgen]
pub fn rotate_by(angle: f32) {
    let a = angle;
    unsafe {
        let old_dir_x = DIR_X;
        DIR_X = DIR_X * a.cos() - DIR_Y * a.sin();
        DIR_Y = old_dir_x * a.sin() + DIR_Y * a.cos();

        let old_plane_x = PLANE_X;
        PLANE_X = PLANE_X * a.cos() - PLANE_Y * a.sin();
        PLANE_Y = old_plane_x * a.sin() + PLANE_Y * a.cos();
    }
}

#[wasm_bindgen]
pub fn get_wall_texture_ptr() -> *mut u8 {
    #[allow(static_mut_refs)]
    unsafe {
        if let Some(ref mut v) = WALL_TEX_VEC {
            v.as_mut_ptr()
        } else {
            WALL_TEX_BUF.as_mut_ptr()
        }
    }
}

#[wasm_bindgen]
pub fn set_wall_texture_size(w: usize, h: usize) {
    unsafe {
        WALL_TEX_W = w;
        WALL_TEX_H = h;
    }
}

#[wasm_bindgen]
pub fn get_floor_texture_ptr() -> *mut u8 {
    #[allow(static_mut_refs)]
    unsafe {
        if let Some(ref mut v) = FLOOR_TEX_VEC {
            v.as_mut_ptr()
        } else {
            FLOOR_TEX_BUF.as_mut_ptr()
        }
    }
}

#[wasm_bindgen]
pub fn alloc_wall_texture(w: usize, h: usize) -> *mut u8 {
    let bytes = match w.checked_mul(h).and_then(|p| p.checked_mul(4)) {
        Some(b) => b,
        None => return std::ptr::null_mut(),
    };
    unsafe {
        // allocate and zero-initialize
        WALL_TEX_VEC = Some(vec![0u8; bytes]);
        WALL_TEX_W = w;
        WALL_TEX_H = h;
        WALL_TEX_VEC.as_mut().unwrap().as_mut_ptr()
    }
}

#[wasm_bindgen]
pub fn alloc_floor_texture(w: usize, h: usize) -> *mut u8 {
    let bytes = match w.checked_mul(h).and_then(|p| p.checked_mul(4)) {
        Some(b) => b,
        None => return std::ptr::null_mut(),
    };
    unsafe {
        FLOOR_TEX_VEC = Some(vec![0u8; bytes]);
        FLOOR_TEX_W = w;
        FLOOR_TEX_H = h;
        FLOOR_TEX_VEC.as_mut().unwrap().as_mut_ptr()
    }
}

#[wasm_bindgen]
pub fn set_floor_texture_size(w: usize, h: usize) {
    unsafe {
        FLOOR_TEX_W = w;
        FLOOR_TEX_H = h;
    }
}

#[wasm_bindgen]
pub fn get_sky_texture_ptr() -> *mut u8 {
    #[allow(static_mut_refs)]
    unsafe {
        if let Some(ref mut v) = SKY_TEX_VEC {
            v.as_mut_ptr()
        } else {
            SKY_TEX_BUF.as_mut_ptr()
        }
    }
}

#[wasm_bindgen]
pub fn alloc_sky_texture(w: usize, h: usize) -> *mut u8 {
    let bytes = match w.checked_mul(h).and_then(|p| p.checked_mul(4)) {
        Some(b) => b,
        None => return std::ptr::null_mut(),
    };
    unsafe {
        SKY_TEX_VEC = Some(vec![0u8; bytes]);
        SKY_TEX_W = w;
        SKY_TEX_H = h;
        SKY_TEX_VEC.as_mut().unwrap().as_mut_ptr()
    }
}

#[wasm_bindgen]
pub fn set_sky_texture_size(w: usize, h: usize) {
    unsafe {
        SKY_TEX_W = w;
        SKY_TEX_H = h;
    }
}

#[wasm_bindgen]
pub fn get_ceiling_texture_ptr() -> *mut u8 {
    #[allow(static_mut_refs)]
    unsafe {
        if let Some(ref mut v) = CEILING_TEX_VEC {
            v.as_mut_ptr()
        } else {
            CEILING_TEX_BUF.as_mut_ptr()
        }
    }
}

#[wasm_bindgen]
pub fn alloc_ceiling_texture(w: usize, h: usize) -> *mut u8 {
    let bytes = match w.checked_mul(h).and_then(|p| p.checked_mul(4)) {
        Some(b) => b,
        None => return std::ptr::null_mut(),
    };
    unsafe {
        CEILING_TEX_VEC = Some(vec![0u8; bytes]);
        CEILING_TEX_W = w;
        CEILING_TEX_H = h;
        CEILING_TEX_VEC.as_mut().unwrap().as_mut_ptr()
    }
}

#[wasm_bindgen]
pub fn set_ceiling_texture_size(w: usize, h: usize) {
    unsafe {
        CEILING_TEX_W = w;
        CEILING_TEX_H = h;
    }
}

#[wasm_bindgen]
pub fn init() {
    // Generate a new random map on each init (seeded from Date.now())
    let mut seed = date_now() as u32;
    let map_w = 8 + (lcg_next(&mut seed) % 13) as usize; // 8..20
    let map_h = 8 + (lcg_next(&mut seed) % 9) as usize;  // 8..16
    generate_map(map_w, map_h, seed);

    unsafe {
        POS_X = 2.5;
        POS_Y = 2.5;
        DIR_X = -1.0;
        DIR_Y = 0.0;
        PLANE_X = 0.0;
        PLANE_Y = 0.66;

        // Ensure player starts in an empty cell; if not, scan for one
        let mut sx = POS_X as i32;
        let mut sy = POS_Y as i32;
        if map_at(sx, sy) != 0 {
            let mw = MAP_W as i32;
            let mh = MAP_H as i32;
            'outer: for y in 1..(mh - 1) {
                for x in 1..(mw - 1) {
                    if map_at(x, y) == 0 {
                        sx = x;
                        sy = y;
                        break 'outer;
                    }
                }
            }
            POS_X = sx as f32 + 0.5;
            POS_Y = sy as f32 + 0.5;
        }
    }

    render_scene();
}

#[wasm_bindgen]
pub fn step(forward: f32, turn: f32, strafe: f32, jump: f32, dt: f32) {
    // Helper: test if position is collision-free by checking a circle around it
    fn is_position_clear(x: f32, y: f32, radius: f32) -> bool {
        // Diamond-shaped collision detection using Manhattan distance.
        // This creates a hitbox rotated 45° that's more forgiving at corners
        // than a circle, allowing smooth sliding through diagonal corner gaps.
        // A point (px, py) collides with wall at (wx, wy) if:
        //   |px - closest_x| + |py - closest_y| < radius
        
        let min_grid_x = ((x - radius).floor() as i32);
        let max_grid_x = ((x + radius).ceil() as i32);
        let min_grid_y = ((y - radius).floor() as i32);
        let max_grid_y = ((y + radius).ceil() as i32);
        
        for grid_y in min_grid_y..=max_grid_y {
            for grid_x in min_grid_x..=max_grid_x {
                if map_at(grid_x, grid_y) != 0 {
                    // Found a wall cell at [grid_x, grid_x+1) x [grid_y, grid_y+1)
                    
                    // Find closest point on this wall cell to player position
                    let closest_x = x.max(grid_x as f32).min((grid_x + 1) as f32);
                    let closest_y = y.max(grid_y as f32).min((grid_y + 1) as f32);
                    
                    // Use Manhattan distance (diamond shape) for collision
                    let dx = (x - closest_x).abs();
                    let dy = (y - closest_y).abs();
                    let manhattan_distance = dx + dy;
                    
                    if manhattan_distance < radius {
                        return false; // Collision detected
                    }
                }
            }
        }
        true
    }
    
    // Detect collision and return the surface normal of the wall being hit.
    // Returns None if no collision, Some((nx, ny)) with the wall normal if collision.
    // For Manhattan offset: diagonal edges have normals like (±1/√2, ±1/√2)
    // For straight walls: normals are (±1, 0) or (0, ±1)
    fn detect_collision_normal(x: f32, y: f32, radius: f32) -> Option<(f32, f32)> {
        let min_grid_x = ((x - radius).floor() as i32);
        let max_grid_x = ((x + radius).ceil() as i32);
        let min_grid_y = ((y - radius).floor() as i32);
        let max_grid_y = ((y + radius).ceil() as i32);
        
        let mut closest_dist = f32::INFINITY;
        let mut best_normal: Option<(f32, f32)> = None;
        
        for grid_y in min_grid_y..=max_grid_y {
            for grid_x in min_grid_x..=max_grid_x {
                if map_at(grid_x, grid_y) != 0 {
                    // Wall cell bounds
                    let wx0 = grid_x as f32;
                    let wx1 = (grid_x + 1) as f32;
                    let wy0 = grid_y as f32;
                    let wy1 = (grid_y + 1) as f32;
                    
                    // Closest point on cell
                    let closest_x = x.max(wx0).min(wx1);
                    let closest_y = y.max(wy0).min(wy1);
                    
                    let dx = x - closest_x;
                    let dy = y - closest_y;
                    let manhattan_dist = dx.abs() + dy.abs();
                    
                    if manhattan_dist < radius && manhattan_dist < closest_dist {
                        closest_dist = manhattan_dist;
                        
                        // Determine which surface: axis-aligned wall or diagonal corner
                        let on_left = (closest_x - wx0).abs() < 0.001;
                        let on_right = (closest_x - wx1).abs() < 0.001;
                        let on_bottom = (closest_y - wy0).abs() < 0.001;
                        let on_top = (closest_y - wy1).abs() < 0.001;
                        
                        // Check if at corner (diagonal edge of Manhattan offset)
                        let at_corner = (on_left || on_right) && (on_bottom || on_top);
                        
                        if at_corner {
                            // Diagonal edge - normal points away from corner
                            // Normalize the diagonal normal
                            let inv_sqrt2 = 0.70710678118; // 1/sqrt(2)
                            let nx = dx.signum() * inv_sqrt2;
                            let ny = dy.signum() * inv_sqrt2;
                            best_normal = Some((nx, ny));
                        } else {
                            // Straight edge - axis-aligned normal
                            if dx.abs() > dy.abs() {
                                best_normal = Some((dx.signum(), 0.0));
                            } else {
                                best_normal = Some((0.0, dy.signum()));
                            }
                        }
                    }
                }
            }
        }
        
        best_normal
    }
    
    // Physics-based wall sliding: project movement vector onto wall tangent.
    // When hitting a wall, decompose the force into parallel (tangent) and
    // perpendicular (normal) components. Only the tangent component produces movement.
    fn slide_move(start_x: f32, start_y: f32, dx: f32, dy: f32, radius: f32) -> (f32, f32) {
        let target_x = start_x + dx;
        let target_y = start_y + dy;
        
        // 1. Try full movement - no collision
        if is_position_clear(target_x, target_y, radius) {
            return (target_x, target_y);
        }
        
        // 2. Collision detected - find the wall normal at target position
        if let Some((nx, ny)) = detect_collision_normal(target_x, target_y, radius) {
            // Project movement vector onto wall tangent (perpendicular to normal)
            // Tangent vector: rotate normal by 90° => tangent = (-ny, nx)
            let tx = -ny;
            let ty = nx;
            
            // Project (dx, dy) onto tangent: dot product with tangent vector
            let projection = dx * tx + dy * ty;
            
            // Movement along wall = projection * tangent_direction
            let slide_dx = projection * tx;
            let slide_dy = projection * ty;
            
            let slide_x = start_x + slide_dx;
            let slide_y = start_y + slide_dy;
            
            // Verify the slide position is valid
            if is_position_clear(slide_x, slide_y, radius) {
                return (slide_x, slide_y);
            }
        }
        
        // 3. Fallback: try axis-aligned sliding (for edge cases)
        let abs_dx = dx.abs();
        let abs_dy = dy.abs();
        
        if abs_dx > abs_dy {
            if is_position_clear(target_x, start_y, radius) {
                return (target_x, start_y);
            }
            if is_position_clear(start_x, target_y, radius) {
                return (start_x, target_y);
            }
        } else {
            if is_position_clear(start_x, target_y, radius) {
                return (start_x, target_y);
            }
            if is_position_clear(target_x, start_y, radius) {
                return (target_x, start_y);
            }
        }
        
        // No movement possible
        (start_x, start_y)
    }
    
    // Movement speeds are in world units per second, but code previously
    // used per-frame constants. We'll treat them as per-second and multiply
    // by dt so motion is framerate-independent.
    let move_speed = 2.5; // meters per second (forward/back)
    let rot_speed = 2.0; // radians per second
    // collision buffer in meters: player stops this far before wall
    // Reduced from 0.2 to 0.15 to allow tighter corner navigation
    let collide_radius: f32 = 0.15;
    // More realistic jump physics (m/s and m/s^2). Increase gravity
    // to make the arc quicker while keeping a reasonable apex.
    let jump_initial_v: f32 = 4.0; // m/s
    let gravity: f32 = 18.0; // m/s^2 (stronger for snappier arc)

    unsafe {
        if forward.abs() > 0.01 {
            let dx = DIR_X * move_speed * forward * dt;
            let dy = DIR_Y * move_speed * forward * dt;
            let (new_x, new_y) = slide_move(POS_X, POS_Y, dx, dy, collide_radius);
            POS_X = new_x;
            POS_Y = new_y;
        }

        if strafe.abs() > 0.01 {
            let dx = PLANE_X * move_speed * strafe * dt;
            let dy = PLANE_Y * move_speed * strafe * dt;
            let (new_x, new_y) = slide_move(POS_X, POS_Y, dx, dy, collide_radius);
            POS_X = new_x;
            POS_Y = new_y;
        }

        if turn.abs() > 0.01 {
            let angle = rot_speed * turn * dt;
            let old_dir_x = DIR_X;
            DIR_X = DIR_X * angle.cos() - DIR_Y * angle.sin();
            DIR_Y = old_dir_x * angle.sin() + DIR_Y * angle.cos();

            let old_plane_x = PLANE_X;
            PLANE_X = PLANE_X * angle.cos() - PLANE_Y * angle.sin();
            PLANE_Y = old_plane_x * angle.sin() + PLANE_Y * angle.cos();
        }

        if jump > 0.5 && JUMP_OFFSET == 0.0 && JUMP_VY == 0.0 {
            JUMP_VY = jump_initial_v;
        }

        if JUMP_VY != 0.0 || JUMP_OFFSET != 0.0 {
            // integrate using dt (simple Euler integration)
            JUMP_OFFSET += JUMP_VY * dt;
            JUMP_VY -= gravity * dt;
            if JUMP_OFFSET <= 0.0 {
                JUMP_OFFSET = 0.0;
                JUMP_VY = 0.0;
            }
        }
    }

    render_scene();
}

#[wasm_bindgen]
pub fn get_pos_x() -> f32 {
    unsafe { POS_X }
}

#[wasm_bindgen]
pub fn get_pos_y() -> f32 {
    unsafe { POS_Y }
}

#[wasm_bindgen]
pub fn get_dir_x() -> f32 {
    unsafe { DIR_X }
}

#[wasm_bindgen]
pub fn get_dir_y() -> f32 {
    unsafe { DIR_Y }
}

#[wasm_bindgen]
pub fn get_plane_x() -> f32 {
    unsafe { PLANE_X }
}

#[wasm_bindgen]
pub fn get_plane_y() -> f32 {
    unsafe { PLANE_Y }
}

#[wasm_bindgen]
pub fn get_map_w() -> u32 {
    unsafe { MAP_W as u32 }
}

#[wasm_bindgen]
pub fn get_map_h() -> u32 {
    unsafe { MAP_H as u32 }
}

#[wasm_bindgen]
pub fn get_map_ptr() -> *const i32 {
    #[allow(static_mut_refs)]
    unsafe {
        if let Some(ref v) = WORLD_MAP_VEC {
            v.as_ptr()
        } else {
            std::ptr::null()
        }
    }
}

#[wasm_bindgen]
pub fn regenerate_map(seed: u32) {
    // seed==0 -> use Date.now()
    let mut s = if seed == 0 { date_now() as u32 } else { seed };
    let map_w = 8 + (lcg_next(&mut s) % 13) as usize; // 8..20
    let map_h = 8 + (lcg_next(&mut s) % 9) as usize;  // 8..16
    generate_map(map_w, map_h, s);

    unsafe {
        // ensure player not inside wall
        let mut sx = POS_X as i32;
        let mut sy = POS_Y as i32;
        if map_at(sx, sy) != 0 {
            let mw = MAP_W as i32;
            let mh = MAP_H as i32;
            'outer3: for y in 1..(mh - 1) {
                for x in 1..(mw - 1) {
                    if map_at(x, y) == 0 {
                        sx = x;
                        sy = y;
                        break 'outer3;
                    }
                }
            }
            POS_X = sx as f32 + 0.5;
            POS_Y = sy as f32 + 0.5;
        }
    }

    render_scene();
}

#[wasm_bindgen]
pub fn regenerate_map_with_size(w: u32, h: u32, seed: u32) {
    let mut s = if seed == 0 { date_now() as u32 } else { seed };
    let map_w = if w == 0 { 8 + (lcg_next(&mut s) % 13) as usize } else { w as usize };
    let map_h = if h == 0 { 8 + (lcg_next(&mut s) % 9) as usize } else { h as usize };
    generate_map(map_w, map_h, s);

    unsafe {
        // ensure player not inside wall
        let mut sx = POS_X as i32;
        let mut sy = POS_Y as i32;
        if map_at(sx, sy) != 0 {
            let mw = MAP_W as i32;
            let mh = MAP_H as i32;
            'outer4: for y in 1..(mh - 1) {
                for x in 1..(mw - 1) {
                    if map_at(x, y) == 0 {
                        sx = x;
                        sy = y;
                        break 'outer4;
                    }
                }
            }
            POS_X = sx as f32 + 0.5;
            POS_Y = sy as f32 + 0.5;
        }
    }

    render_scene();
}
