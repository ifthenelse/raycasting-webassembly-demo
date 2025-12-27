# Sprite Texture System Implementation Summary

## What Was Implemented

I've successfully added a complete sprite texture system to your raycasting engine with automatic viewing angle selection, similar to Duke Nukem 3D.

## Key Features

### 1. **Sprite Atlas Support**

- Horizontal sprite atlases where frames are arranged side-by-side
- Each frame represents a different viewing angle (typically 8 frames for 360° coverage)
- Automatic frame selection based on camera angle
- Bilinear texture filtering for smooth rendering

### 2. **Orientation-Based Rendering**

- Each sprite has an orientation vector defining which direction it "faces"
- The engine calculates the relative viewing angle
- Automatically selects the correct frame from the atlas
- Frame 0 = front view, progressing clockwise around the sprite

### 3. **Depth Sorting & Transparency**

- Painter's algorithm: sprites sorted far-to-near
- Transparency support (pixels with low RGB values are skipped)
- Proper depth testing against walls
- Distance-based shading for realism

### 4. **Automatic Wall Lamp Placement**

- Scans generated maps for walls
- Places lamps on ~20% of suitable wall cells
- Automatically orients lamps perpendicular to walls
- Ensures lamps face into walkable areas

## Files Modified

### Rust (src/lib.rs)

1. **New Data Structures:**
   - `Sprite` struct with position, orientation, type, and rendering parameters
   - Static storage for sprite texture and sprite list

2. **New Functions:**
   - `alloc_sprite_texture()` - Allocate sprite texture memory
   - `set_sprite_texture_size()` - Set sprite texture dimensions
   - `get_sprite_texture_ptr()` - Get texture pointer for JS
   - `add_sprite()` - Add a sprite to the world
   - `clear_sprites()` - Remove all sprites
   - `get_sprite_count()` - Get sprite count
   - `render_sprites()` - Main sprite rendering logic
   - `place_wall_lamps()` - Automatic lamp placement

3. **Modified Functions:**
   - `render_scene()` - Now calls sprite renderer after floor/ceiling
   - `init()` - Initializes sprites and places lamps
   - `regenerate_map()` - Places lamps in new map
   - `regenerate_map_with_size()` - Places lamps in new map

### JavaScript (src/web/App.svelte)

1. **Texture Loading:**
   - Added loading of `lamp-wall.png`
   - Allocated sprite texture in WASM memory
   - Uploaded texture data to WASM

## How to Use

### Recommended Atlas Format

**Single atlas file (RECOMMENDED):**

```
File: lamp-wall.png
Layout: [Front(0°)] [3/4-R(45°)] [Right(90°)] [3/4-BR(135°)] [Back(180°)] [3/4-BL(225°)] [Left(270°)] [3/4-FL(315°)]
Size: 512×64 px (8 frames @ 64×64 px each)
```

**Important:** Each frame must show the object rotated by the exact angle specified. Off-by-angle errors break the visual illusion.

This is the format I've implemented. You can provide a single horizontal atlas where:

- Each frame is the same width (width / num_frames)
- Frames show the sprite from different angles at exact 45° intervals
- Frame 0 = front (0°), facing viewer directly
- Frame 2 = right (90°), pure side view
- Frame 4 = back (180°), facing away
- Frame 6 = left (270°), opposite side view
- Frames progress clockwise and symmetrically:
  - Frames 1 & 7 are mirrors (45° and 315°)
  - Frames 2 & 6 are mirrors (90° and 270°)
  - Frames 3 & 5 are mirrors (135° and 225°)

**CRITICAL:** Never include AI-generated atlases without manual validation. Common errors:

- Frames rotated to wrong angles (e.g., frame shows 45° when it should show 90°)
- Duplicate mounting hardware (bracket visible from two directions simultaneously)
- Inconsistent material geometry (glass panel replaced with metal in one frame)
- Unmirrored symmetry frames

See QUICKSTART_SPRITES.md and SPRITE_ATLAS_FORMAT.md for detailed validation checklists before using any sprite atlas.

### Adding Sprites Manually

```javascript
// Example: Add a lamp at position (5.5, 3.0)
// Facing north (orientation vector: 0, -1)
// Using 8 frames, height 0.8 units
pkg.add_sprite(5.5, 3.0, 0, 0.0, -1.0, 8, 0.8);
```

### Orientation Examples

```javascript
// North-facing (points up)
orientation: (0, -1);

// South-facing (points down)
orientation: (0, 1);

// East-facing (points right)
orientation: (1, 0);

// West-facing (points left)
orientation: (-1, 0);

// For wall-mounted sprites, orient perpendicular to wall
// and pointing into the walkable space
```

## Viewing Angle Calculation

The system determines which frame to show based on:

```
Player's viewing angle relative to sprite's orientation:

    Front (0°)
        ↑
        |
Left ←--+--→ Right
        |
        ↓
    Back (180°)

8 frames divide 360° into 45° segments:
- 0°:   Frame 0 (Front)
- 45°:  Frame 1 (3/4 Right)
- 90°:  Frame 2 (Right)
- 135°: Frame 3 (3/4 Back-Right)
- 180°: Frame 4 (Back)
- 225°: Frame 5 (3/4 Back-Left)
- 270°: Frame 6 (Left)
- 315°: Frame 7 (3/4 Front-Left)
```

## Creating Sprite Atlases

### Option 1: Single Atlas (What I Implemented)

Provide one image like `lamp-wall.png` with all frames horizontally:

- **Pros:** Simple, efficient, one file to manage
- **Cons:** Must edit all frames together
- **Best for:** Final production assets

### Option 2: Separate Frames

Provide individual images with naming convention:

- `lamp-wall-000.png`, `lamp-wall-045.png`, etc.
- Or: `lamp-wall-front.png`, `lamp-wall-3q-right.png`, etc.
- **Pros:** Easy to edit individual frames
- **Cons:** More files, requires loading logic
- **Best for:** Development and iteration

**I recommend Option 1** (single atlas) as it's what I've implemented and matches classic raycasting engines.

## Next Steps

### To Test With Your Own Sprite:

1. **Create or obtain a sprite atlas** with 8 horizontal frames (512×64 px recommended)
2. **Save it as** `assets/lamp-wall.png`
3. **Build and run:** `npm run build && npm run dev`
4. **You should see:** Wall lamps automatically placed on walls throughout the map

### To Add Different Sprite Types:

1. Load additional sprite atlases in App.svelte
2. Use different `sprite_type` values (0, 1, 2, etc.)
3. Modify texture sampling in `render_sprites()` to support multi-row atlases
4. Create placement functions for different sprite types

### Advanced Features (Not Yet Implemented):

- **Animated sprites:** Cycle through frames over time
- **Freestanding sprites:** Objects not attached to walls
- **Multiple sprite atlases:** Different textures for different types
- **Alpha channel transparency:** Better transparency than luminance threshold
- **Sprite scaling:** Variable sizes per sprite instance
- **Billboarding:** Always face camera (different from current system)

## Technical Details

### Sprite Rendering Pipeline

1. **Sort sprites** by distance (far to near)
2. **For each sprite:**
   - Calculate screen position via camera transformation
   - Determine viewing angle and select frame
   - Calculate screen size based on distance
   - Render column by column with transparency
   - Apply distance-based shading

### Performance

- Sprites are culled if behind camera
- Bilinear filtering provides quality
- Sorting is O(n log n) but n is typically small
- Column-based rendering integrates with existing renderer

### Transparency Handling

Currently using luminance threshold:

```rust
if r > 10 || g > 10 || b > 10 {
    // Draw pixel
}
```

For better results, modify to check alpha channel in RGBA textures.

## Conclusion

You now have a fully functional sprite system! The **recommended workflow** is:

1. **Prepare your sprite atlas** as a single horizontal image with 8 frames
2. **Place it** in `assets/lamp-wall.png`
3. **Build:** `npm run build`
4. **Run:** `npm run dev`
5. **Enjoy** automatic wall lamps with correct viewing angles!

The system automatically handles:

- ✅ Loading sprite textures
- ✅ Placing sprites in the world
- ✅ Selecting correct viewing angle
- ✅ Rendering with depth and transparency
- ✅ Distance-based shading

Let me know if you'd like help creating sprite atlases or adding additional features!
