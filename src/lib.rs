use wasm_bindgen::prelude::*;

const WIDTH: usize = 320;
const HEIGHT: usize = 200;
const FB_SIZE: usize = WIDTH * HEIGHT * 4;

// Simple 8x8 map: 1 = wall, 0 = empty
const MAP_W: usize = 8;
const MAP_H: usize = 8;
const WORLD_MAP: [i32; MAP_W * MAP_H] = [
    1,1,1,1,1,1,1,1,
    1,0,0,0,0,0,0,1,
    1,0,0,0,0,0,0,1,
    1,0,0,1,1,0,0,1,
    1,0,0,1,1,0,0,1,
    1,0,0,0,0,0,0,1,
    1,0,0,0,0,0,0,1,
    1,1,1,1,1,1,1,1,
];

static mut FRAMEBUFFER: [u8; FB_SIZE] = [0; FB_SIZE];

// Legacy fixed-size fallback (small). We prefer dynamic vectors allocated at runtime.
const TEXTURE_MAX_SIZE: usize = 256 * 256 * 4; // 256x256 RGBA fallback

static mut WALL_TEX_BUF: [u8; TEXTURE_MAX_SIZE] = [0; TEXTURE_MAX_SIZE];
static mut WALL_TEX_W: usize = 64;
static mut WALL_TEX_H: usize = 64;
static mut FLOOR_TEX_BUF: [u8; TEXTURE_MAX_SIZE] = [0; TEXTURE_MAX_SIZE];
static mut FLOOR_TEX_W: usize = 64;
static mut FLOOR_TEX_H: usize = 64;

// Dynamically allocated texture storages (used when JS requests larger textures)
static mut WALL_TEX_VEC: Option<Vec<u8>> = None;
static mut FLOOR_TEX_VEC: Option<Vec<u8>> = None;

// Player state
static mut POS_X: f32 = 2.5;
static mut POS_Y: f32 = 2.5;
static mut DIR_X: f32 = -1.0;
static mut DIR_Y: f32 = 0.0;
static mut PLANE_X: f32 = 0.0;
static mut PLANE_Y: f32 = 0.66;
static mut JUMP_OFFSET: f32 = 0.0;
static mut JUMP_VY: f32 = 0.0;

fn map_at(x: i32, y: i32) -> i32 {
    if x < 0 || y < 0 || x >= MAP_W as i32 || y >= MAP_H as i32 {
        1
    } else {
        WORLD_MAP[(y as usize) * MAP_W + (x as usize)]
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
    let jump_px_scale: f32 = 30.0;
    let center: f32 = (HEIGHT as f32) / 2.0 + unsafe { JUMP_OFFSET * jump_px_scale };

    // Fill sky + floor base
    for y in 0..HEIGHT as i32 {
        let sky = (y as f32) < center;
        for x in 0..WIDTH as i32 {
            if sky {
                put_pixel(x, y, 80, 120, 200);
            } else {
                put_pixel(x, y, 60, 60, 60);
            }
        }
    }

    let (pos_x, pos_y, dir_x, dir_y, plane_x, plane_y) = unsafe {
        (POS_X, POS_Y, DIR_X, DIR_Y, PLANE_X, PLANE_Y)
    };

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
        let wall_x = if side == 0 {
            pos_y + perp_wall_dist * ray_dir_y
        } else {
            pos_x + perp_wall_dist * ray_dir_x
        };
        let wall_x = wall_x - wall_x.floor();

        let shade = (255.0 / (1.0 + perp_wall_dist * 0.3)).min(255.0);
        let shade = if side == 1 { shade * 0.7 } else { shade };
        let shade = shade / 255.0;

        unsafe {
            for y in draw_start..=draw_end {
                let tex_v = (y - draw_start) as f32 / ((draw_end - draw_start) as f32 + 1.0);
                let wall_slice: &[u8] = if let Some(ref v) = WALL_TEX_VEC { &v[..] } else { &WALL_TEX_BUF[..] };
                let (r, g, b) = sample_bilinear(wall_slice, WALL_TEX_W, WALL_TEX_H, wall_x, tex_v);
                let (r, g, b) = apply_shade(r, g, b, shade);
                put_pixel(x, y, r, g, b);
            }
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
                let floor_slice: &[u8] = if let Some(ref v) = FLOOR_TEX_VEC { &v[..] } else { &FLOOR_TEX_BUF[..] };
                let (r, g, b) = sample_bilinear(floor_slice, FLOOR_TEX_W, FLOOR_TEX_H, fx, fy);
                let (r, g, b) = apply_shade(r, g, b, shade);
                put_pixel(x, y, r, g, b);
                floor_x += floor_step_x;
                floor_y += floor_step_y;
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
pub fn init() {
    unsafe {
        POS_X = 2.5;
        POS_Y = 2.5;
        DIR_X = -1.0;
        DIR_Y = 0.0;
        PLANE_X = 0.0;
        PLANE_Y = 0.66;
    }
    render_scene();
}

#[wasm_bindgen]
pub fn step(forward: f32, turn: f32, strafe: f32, jump: f32) {
    let move_speed = 0.05;
    let rot_speed = 0.03;
    let jump_initial_v: f32 = 0.9;
    let gravity: f32 = 0.05;

    unsafe {
        if forward.abs() > 0.01 {
            let nx = POS_X + DIR_X * move_speed * forward;
            let ny = POS_Y + DIR_Y * move_speed * forward;

            if map_at(nx as i32, ny as i32) == 0 {
                POS_X = nx;
                POS_Y = ny;
            } else {
                if map_at(nx as i32, POS_Y as i32) == 0 {
                    POS_X = nx;
                }
                if map_at(POS_X as i32, ny as i32) == 0 {
                    POS_Y = ny;
                }
            }
        }

        if strafe.abs() > 0.01 {
            let nx = POS_X + PLANE_X * move_speed * strafe;
            let ny = POS_Y + PLANE_Y * move_speed * strafe;

            if map_at(nx as i32, ny as i32) == 0 {
                POS_X = nx;
                POS_Y = ny;
            } else {
                if map_at(nx as i32, POS_Y as i32) == 0 {
                    POS_X = nx;
                }
                if map_at(POS_X as i32, ny as i32) == 0 {
                    POS_Y = ny;
                }
            }
        }

        if turn.abs() > 0.01 {
            let angle = rot_speed * turn;
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
            JUMP_OFFSET += JUMP_VY;
            JUMP_VY -= gravity;
            if JUMP_OFFSET <= 0.0 {
                JUMP_OFFSET = 0.0;
                JUMP_VY = 0.0;
            }
        }
    }

    render_scene();
}
