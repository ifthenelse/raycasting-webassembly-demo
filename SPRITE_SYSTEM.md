# Sprite System Documentation

## Overview

The raycasting engine now supports textured sprites with automatic orientation-based frame selection, similar to classic games like Duke Nukem 3D and Doom.

## How It Works

### Sprite Atlas Format

Sprites are loaded from **horizontal atlas images** where each frame represents a different viewing angle. The recommended format is:

```
[Frame 0: Front] [Frame 1: 3/4 Right] [Frame 2: Right] [Frame 3: 3/4 Back-Right]
[Frame 4: Back] [Frame 5: 3/4 Back-Left] [Frame 6: Left] [Frame 7: 3/4 Left]
```

- **8 frames** provide full 360° rotation coverage
- Each frame should be the same width (1/8 of the total atlas width for 8 frames)
- Frames are arranged horizontally in a single row
- The atlas should be provided as a PNG with RGBA format

### Sprite Orientation

Each sprite has an **orientation vector** that defines which direction it "faces":

- For wall-mounted lamps, this is perpendicular to the wall, pointing into the walkable area
- The system automatically selects the correct frame based on the viewing angle

### Creating Sprite Atlases

You have two options:

#### Option 1: Single Atlas File (Recommended)

Create one image with all frames side-by-side:

- File: `lamp-wall.png` (512×64 px = 8 frames of 64×64 px each)
- Easier to manage
- More efficient loading
- Currently implemented

#### Option 2: Individual Frame Files

Split the frames into separate files with naming convention:

- `lamp-wall-front.png`, `lamp-wall-3q-right.png`, etc.
- More flexible for editing
- Requires loading multiple files

**Recommendation**: Use Option 1 (single atlas) as it's more efficient and matches the classic raycasting approach.

## API Reference

### JavaScript Functions

#### Loading Sprite Textures

```javascript
// In App.svelte
const spriteTex = await loadTexture('../../assets/lamp-wall.png');
const spriteTexPtr = pkg.alloc_sprite_texture(spriteTex.width, spriteTex.height);
// ... copy texture data ...
pkg.set_sprite_texture_size(spriteTex.width, spriteTex.height);
```

### WASM Functions

#### `add_sprite(x, y, sprite_type, orientation_x, orientation_y, num_frames, height)`

Add a sprite to the world.

**Parameters:**

- `x`: X position in world coordinates (float)
- `y`: Y position in world coordinates (float)
- `sprite_type`: Which row in sprite atlas (0 for single-row atlases)
- `orientation_x`: X component of orientation vector (will be normalized)
- `orientation_y`: Y component of orientation vector (will be normalized)
- `num_frames`: Number of directional frames (typically 8)
- `height`: Sprite height in world units (e.g., 0.8 for wall lamps)

**Example:**

```javascript
// Add a lamp on the north wall, facing south (into room)
pkg.add_sprite(5.5, 3.1, 0, 0.0, 1.0, 8, 0.8);

// Add a lamp on the east wall, facing west
pkg.add_sprite(8.1, 5.5, 0, -1.0, 0.0, 8, 0.8);
```

#### `clear_sprites()`

Remove all sprites from the world.

#### `get_sprite_count()`

Returns the number of sprites currently in the world.

## Automatic Sprite Placement

The engine automatically places wall lamps during map generation:

- Scans walls for adjacent open spaces
- Randomly places lamps on ~20% of suitable wall cells
- Correctly orients lamps perpendicular to walls
- Ensures lamps face into walkable areas

## Frame Selection Algorithm

The system calculates which frame to display based on:

1. **Sprite orientation**: The direction the sprite "faces"
2. **Viewing angle**: The angle from sprite to camera
3. **Frame mapping**: Divides 360° into N equal segments (e.g., 45° for 8 frames)

```
       Frame 0 (Front)
           ↓
    7 ← ● → 1
    6 ← ● → 2
        ●
        ↑
    Frame 4 (Back)
```

## Transparency

The sprite renderer uses a luminance threshold to determine transparency:

- Pixels with RGB values all below 10 are considered transparent
- For better transparency, use alpha channel in sprite atlases
- The renderer can be modified to check the alpha channel if needed

## Performance Considerations

- Sprites are sorted by distance (far to near) for correct depth ordering
- Each sprite is only rendered if in front of the camera (transform_y > 0)
- Column-by-column rendering with Z-buffer checking
- Bilinear filtering for smooth scaling

## Creating Your Own Sprites

### Guidelines for Sprite Atlases

1. **Resolution**: 64×64 px per frame is typical (512×64 for 8-frame atlas)
2. **Background**: Use transparent or pure black (RGB 0,0,0) background
3. **Lighting**: Pre-baked lighting works well; engine applies distance shading
4. **Style**: Pixel art or pre-rendered 3D both work
5. **Symmetry**: Can reuse frames (e.g., Left = mirrored Right) to save space

### Example Atlas Layout (8 frames)

```
[  0° ] [ 45° ] [ 90° ] [135° ] [180° ] [225° ] [270° ] [315° ]
Front   3Q-R    Right   3Q-BR   Back    3Q-BL   Left    3Q-L
```

## Current Implementation

The system currently uses:

- `lamp-wall.png`: Wall-mounted lamp atlas (assumed 8 frames)
- Automatic placement during map generation
- Single sprite type (type 0)

### Next Steps

To add more sprite types:

1. Create additional sprite atlas images
2. Load them with unique sprite_type values
3. Modify `place_wall_lamps()` or add new placement functions
4. Support for freestanding sprites (not wall-mounted)
5. Animated sprites (cycling through frames over time)

## Troubleshooting

**Sprites not appearing?**

- Check sprite texture is loaded correctly
- Verify sprite positions are in valid map cells
- Ensure `num_frames` matches atlas layout
- Check console for WASM errors

**Wrong frame showing?**

- Verify orientation vector is correct
- Check that num_frames parameter matches atlas
- Ensure sprite atlas has frames in correct order

**Sprites appear blocky?**

- Increase sprite atlas resolution
- Check bilinear filtering is enabled
- Verify texture upload completed successfully

**Transparency issues?**

- Use pure black (RGB 0,0,0) for transparent areas
- Or modify renderer to check alpha channel
- Ensure sprite atlas has RGBA format
