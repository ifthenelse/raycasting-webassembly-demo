# Sprite Atlas Format Guide

## CRITICAL: Exact Angle Diagram

Before creating or validating any sprite atlas, understand the exact rotation angles:

```
               Frame 0 (0°)
              FRONT VIEW
                  ↑
                  │
    Frame 7   Frame 1
    (315°)    (45°)
       ╲        │        ╱
        ╲       │       ╱
         ╲      │      ╱
          ╲     │     ╱
           ╲    │    ╱
Frame 6 ────────●──────── Frame 2
(270°)       (90°)
           ╱    │    ╲
          ╱     │     ╲
         ╱      │      ╲
        ╱       │       ╲
       ╱        │        ╲
    Frame 5   Frame 3
    (225°)    (135°)
                  ↓
               Frame 4 (180°)
              BACK VIEW

Each angle represents a 45° rotation clockwise from the previous.
Frames at opposite angles (differing by 180°) show opposite views.
Frames mirrored horizontally should have mirrored angles (±180°, ±90°, etc.).
```

## Expected Atlas Layout

Your sprite atlas should be a **horizontal strip** containing all viewing angles:

```
┌──────────┬──────────┬──────────┬──────────┬──────────┬──────────┬──────────┬──────────┐
│ Frame 0  │ Frame 1  │ Frame 2  │ Frame 3  │ Frame 4  │ Frame 5  │ Frame 6  │ Frame 7  │
│  Front   │  3/4 R   │  Right   │ 3/4 BR   │   Back   │ 3/4 BL   │  Left    │ 3/4 FL   │
│   0°     │   45°    │   90°    │   135°   │   180°   │   225°   │   270°   │   315°   │
└──────────┴──────────┴──────────┴──────────┴──────────┴──────────┴──────────┴──────────┘
 64×64 px   64×64 px   64×64 px   64×64 px   64×64 px   64×64 px   64×64 px   64×64 px

Total size: 512 pixels wide × 64 pixels tall
```

**Key angle checkpoints for validation:**

- Frame 0 (0°) and Frame 4 (180°): opposite views (upside-down)
- Frame 2 (90°) and Frame 6 (270°): opposite side views
- Frame 2 (90°) rotated 180° ≈ Frame 6 (270°) when flipped

## Viewing Angles Explained

Imagine you're walking around the lamp in a circle, clockwise:

```
                    Frame 0: FRONT VIEW
                          ↑
                         🔦 ← Lamp facing you

      Frame 7            |            Frame 1
     3/4 Front-L         |          3/4 Front-R
         ↖              🔦              ↗
                         |
                         |
Frame 6: LEFT ← -------- 🔦 -------- → Frame 2: RIGHT
                         |
                         |
         ↙              🔦              ↘
      Frame 5            |            Frame 3
     3/4 Back-L          |          3/4 Back-R
                         |
                         ↓
                    Frame 4: BACK VIEW
                         🔦
```

## Frame Details

### Frame 0: Front (0°) — Object Facing Camera

- Rotated 0°: facing directly at camera
- All front-facing surfaces visible
- Mounting hardware (brackets, screws) centered and facing user
- Show detailed/brightened view
- For lamps: Both glass panes visible, bracket centered

### Frame 1: 3/4 Right (45°) — 45° Clockwise

- Rotated exactly 45° clockwise from Frame 0
- Right front surfaces partially visible, left surfaces hidden
- Mounting bracket on the left side (recedes away from camera)
- Transitional view showing right-front
- For lamps: Right glass pane prominent, bracket on left edge

### Frame 2: Right (90°) — Pure Side View

- Rotated 90° clockwise from Frame 0
- Perfectly perpendicular to camera — pure side profile
- Only right-side surfaces visible
- Mounting bracket visible on left edge
- **Exactly horizontal mirror of Frame 6**
- For lamps: One glass pane side view, bracket on left

### Frame 3: 3/4 Back-Right (135°) — 135° Clockwise

- Rotated 135° clockwise from Frame 0
- Right back surfaces visible, front almost completely hidden
- Mounting bracket on left side
- Transitional view showing right-back
- **Exactly horizontal mirror of Frame 5**
- For lamps: Back-right corner visible, bracket receding left

### Frame 4: Back (180°) — Object Facing Away

- Rotated 180° — directly away from camera
- Back surfaces fully visible
- **CRITICAL**: Show mounting hardware on back side ONLY. Never show duplicate brackets facing different directions.
- Exactly opposite of Frame 0
- For lamps: Back view of lantern, bracket centered on back

### Frame 5: 3/4 Back-Left (225°) — 225° Clockwise

- Rotated 225° clockwise from Frame 0
- Left back surfaces visible, front almost completely hidden
- Mounting bracket on right side
- Transitional view showing left-back
- **This is Frame 3 mirrored horizontally**
- For lamps: Back-left corner visible, bracket receding right

### Frame 6: Left (270°) — Pure Side View (Left)

- Rotated 270° clockwise (or 90° counter-clockwise) from Frame 0
- Perfectly perpendicular to camera — pure side profile from opposite side
- Only left-side surfaces visible
- Mounting bracket visible on right edge
- **Exactly horizontal mirror of Frame 2**
- For lamps: One glass pane side view, bracket on right

### Frame 7: 3/4 Front-Left (315°) — 315° Clockwise

- Rotated 315° clockwise (or 45° counter-clockwise) from Frame 0
- Left front surfaces partially visible, right surfaces hidden
- Mounting bracket on right side (recedes away)
- Transitional view showing left-front
- **This is Frame 1 mirrored horizontally**
- For lamps: Left glass pane prominent, bracket on right edge

## Frame Validation Rules

**Before submitting your sprite atlas, verify these conditions:**

1. **Each frame shows the correct rotation angle** (0°, 45°, 90°, 135°, 180°, 225°, 270°, 315°)
   - Not approximate or off-by-angle — the viewer will notice misaligned rotations
   - Use a protractor, grid, or 3D model to confirm angles
   - Compare against the examples and expected angles precisely

2. **Mounting hardware is consistent throughout**
   - Bracket/screws always face outward relative to the object
   - Never show duplicate hardware facing different directions in the same frame
   - Position progresses smoothly: centered (Frame 0) → left edge (Frame 2) → back (Frame 4) → right edge (Frame 6)

3. **Mirror pairs are true horizontal flips**
   - Frame 6 = Frame 2 flipped horizontally (should be identical after flip)
   - Frame 5 = Frame 3 flipped horizontally
   - Frame 7 = Frame 1 flipped horizontally
   - If a pair doesn't match after flipping, one frame is wrong

4. **All visible surfaces show consistent geometry**
   - Glass panels, metal details, seams — all present and aligned
   - No missing or extra features in any frame
   - Lighting/shading consistent with rotation

5. **Canvas and positioning are uniform**
   - All frames identical size
   - Object centered consistently
   - No offset or cropping differences between frames

### Frame 6: Left (270°)

- Pure side view (left)
- Mirror of Frame 2
- One glass pane visible

### Frame 7: 3/4 Front-Left (315°)

- Rotated 315° clockwise (or -45°)
- Slight angle to the left
- Transitional view

## Symmetry Optimization

You can reuse frames to reduce atlas size:

- Frame 6 (Left) can be a horizontal flip of Frame 2 (Right)
- Frame 7 (3/4 FL) can be a flip of Frame 1 (3/4 FR)
- Frame 5 (3/4 BL) can be a flip of Frame 3 (3/4 BR)

This reduces unique frames from 8 to 5!

## Technical Specs

### Recommended Dimensions

- **Individual frame**: 64×64 pixels (minimum 32×32, maximum 128×128)
- **Full atlas (8 frames)**: 512×64 pixels
- **Full atlas (16 frames)**: 1024×64 pixels (for smoother rotation)

### File Format

- **PNG** with RGBA support (for transparency)
- **Transparent background** or pure black (RGB 0,0,0)
- **Pre-lit** sprites work best (engine adds distance fog)

### Color Depth

- 32-bit RGBA recommended
- 24-bit RGB acceptable (uses luminance for transparency)
- 8-bit paletted acceptable (convert to RGBA on load)

## Creating Sprites

### Method 1: Pixel Art

1. Draw the front view in your pixel art editor
2. Copy and rotate/modify for other angles
3. Export as PNG strip

### Method 2: 3D Rendering

1. Model the lamp in Blender/Maya
2. Set up camera at 8 positions around the model
3. Render each view
4. Stitch renders into horizontal atlas

### Method 3: Photo-Based

1. Photograph real object from 8 angles
2. Remove background
3. Resize and align
4. Combine into atlas

## Example Code to Generate Test Atlas

```python
# Python script to create a simple test atlas
from PIL import Image, ImageDraw

# Create 512×64 image
atlas = Image.new('RGBA', (512, 64), (0, 0, 0, 0))
draw = ImageDraw.Draw(atlas)

# Draw 8 simple lamp frames
for i in range(8):
    x = i * 64
    # Draw a simple colored rectangle for each frame
    color = (255, 200, 100, 255)  # Warm lamp color
    draw.rectangle([x+16, 16, x+48, 48], fill=color)
    # Add frame number
    draw.text((x+28, 28), str(i), fill=(0, 0, 0, 255))

atlas.save('lamp-wall.png')
```

## Orientation in Map Coordinates

Remember: When placing sprites, orientation must point **perpendicular to wall** and **into the room**.

```
Wall cells (represented by █):

    █ █ █ █ █ █ █ █ █
    █     🔦←       █   Lamp orientation: (-1, 0) [West]
    █               █
    █               █
    █   🔦          █   Lamp orientation: (0, -1) [North]
    █   ↑           █
    █ █ █ █ █ █ █ █ █
```

## Testing Your Atlas

1. Save atlas as `assets/lamp-wall.png`
2. Build: `npm run build`
3. Start dev server: `npm run dev`
4. Check browser console for loading errors
5. Walk around walls to see lamps from different angles

## Common Issues

### Lamps Always Show Same Frame

- Check orientation vector is correct
- Verify num_frames parameter (should be 8)
- Ensure frames are in clockwise order

### Lamps Appear Stretched

- Check atlas dimensions match width:height ratio
- Verify all frames are same size
- Adjust sprite height parameter

### Transparency Not Working

- Ensure background is pure black RGB(0,0,0)
- Or modify renderer to check alpha channel
- Check PNG has alpha channel

### Lamps Floating or Embedded

- Adjust sprite Y position (closer to wall edge)
- Modify sprite height parameter
- Check collision detection

## Advanced: Multi-Row Atlases

For multiple sprite types in one texture:

```
Row 0: Lamp frames    [🔦][🔦][🔦][🔦][🔦][🔦][🔦][🔦]
Row 1: Torch frames   [🔥][🔥][🔥][🔥][🔥][🔥][🔥][🔥]
Row 2: Barrel frames  [🛢️][🛢️][🛢️][🛢️][🛢️][🛢️][🛢️][🛢️]
```

Total size: 512×192 pixels (3 rows × 64 px each)

Use sprite_type parameter to select row:

- sprite_type = 0 → Row 0 (lamps)
- sprite_type = 1 → Row 1 (torches)
- sprite_type = 2 → Row 2 (barrels)
