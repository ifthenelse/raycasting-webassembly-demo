# Quick Start: Adding Your Sprite Atlas

## TL;DR

1. Create a sprite atlas PNG (8 frames horizontally)
2. Save it as `assets/lamp-wall.png` (one row per sprite type)
3. Run `npm run build && npm run dev`
4. Sprites will appear on walls

## What You Need to Provide

### Sprite Atlas Image (recommended single-row)

- A single horizontal strip with exactly the directional frames for one sprite type.
- Typical layout (left→right):
  [Front] [3/4-Right] [Right] [3/4-Back-R] [Back] [3/4-Back-L] [Left] [3/4-Front-L]
  (0°, 45°, 90°, 135°, 180°, 225°, 270°, 315°)
- One row = one sprite type. If you need multiple types, use multiple rows (one row per type) or separate atlas files.

### Sizes & format

- Per-frame sizes: 64×64, 64×128, 128×128, etc. (choose height to match sprite proportions)
- Example default: 8 × 64×64 → atlas 512×64
- Format: PNG with alpha (preferred). Do not rely on black-as-transparent.
- Keep consistent canvas size and crop for all frames in the row.

### Consistency rules (important)

- Only include the object itself in the frames. Remove unrelated elements (e.g., spare light bulbs, UI bits, decorative panels).
- All frames in the same row must show the same object from different viewing angles. Do not replace a glass side with a metal panel in just one frame.
- Keep lighting, scale, and cropping consistent across frames.
- If an object is symmetric, you may mirror frames (e.g., Left = mirrored Right) rather than drawing them twice.
- Avoid multiple distinct objects on the same row — that breaks frame indexing.

## Detailed Frame Specifications

**CRITICAL:** Each frame must show the object rotated by the exact angle specified. Off-by-angle errors break the illusion.

- **Frame 0 (0° / Front)**: Object facing camera directly. All front-facing surfaces fully visible. Mounting hardware (brackets, screws) centered and facing user.
- **Frame 1 (45° / 3/4 Right)**: Object rotated 45° clockwise from front. Right front surfaces visible, left surfaces mostly hidden. Mounting bracket on the left (further away).
- **Frame 2 (90° / Right)**: Pure side view — object rotated 90°. Only the right profile visible. Mounting bracket on left edge. Completely symmetrical with Frame 6.
- **Frame 3 (135° / 3/4 Back-Right)**: Object rotated 135°. Right back surfaces visible, front almost completely hidden. Mounting bracket on left side. Mirror of Frame 5.
- **Frame 4 (180° / Back)**: Object facing away from camera. Back surfaces fully visible. Mounting hardware (if visible) on back side only — never show duplicate mounting on both sides.
- **Frame 5 (225° / 3/4 Back-Left)**: Object rotated 225°. Left back surfaces visible, front almost hidden. Mounting bracket on right side. This is Frame 3 mirrored horizontally.
- **Frame 6 (270° / Left)**: Pure side view from left. Left profile fully visible. Mounting bracket on right edge. This is Frame 2 mirrored horizontally.
- **Frame 7 (315° / 3/4 Front-Left)**: Object rotated 315°. Left front surfaces visible, right surfaces mostly hidden. Mounting bracket on right side. This is Frame 1 mirrored horizontally.

## Common atlas mistakes to avoid

- **Wrong rotation angle**: Frame shows 45° when it should show 90° (or any angle mismatch). Verify each frame is rotated by the exact amount.
- **Duplicate mounting hardware**: Showing two mounting brackets facing different directions in the same frame (e.g., one facing user, one facing 45° away). All hardware must face the same direction relative to the object.
- **Inconsistent geometry**: Replacing a glass panel with a metal panel in a single frame. All sides must show the same material in corresponding angles.
- **Non-mirrored symmetry frames**: Frames 2 & 6 should be perfect horizontal mirrors. Frames 3 & 5, and Frames 1 & 7 should also be mirrors. If they differ, one is wrong.
- **Including separate elements**: Spare light bulbs, UI bits, or decorative panels that aren't part of the object itself.
- **Mixed object types**: Using multiple distinct objects in one row (breaks frame indexing).
- **Different frame sizes or offsets**: All frames must have identical canvas size and consistent object positioning.
- **Using opaque PNGs**: Use PNG with real alpha channel, not black-as-transparent hacks.

## Validation Checklist Before Submitting

Before including an atlas in your project, verify:

1. **Rotation angles**: Hold each frame and visually confirm it matches the expected angle (0°, 45°, 90°, etc.).
2. **Mounting consistency**: All mounting hardware faces the same direction. Frame 0 bracket faces user → Frame 2 bracket faces left → Frame 4 bracket faces away.
3. **Mirror validation**: Rotate Frame 2 horizontally and compare to Frame 6 — they should be identical. Same for Frames 1 & 7, and Frames 3 & 5.
4. **Object consistency**: Zoom in on visible surfaces. Glass panels, metal sides, details should all be present in the correct angle.
5. **Canvas alignment**: All frames same size, object centered consistently.
6. **No duplicates**: Object appears once per frame, not multiple orientations superimposed.

## File placement

Save your atlas (one row per sprite) as:

```
assets/lamp-wall.png
```

## For AI Generation Tools (CRITICAL)

If you use AI image generators to create sprites, use this **exact prompt template** to minimize angle/rotation errors:

```
Create an 8-frame sprite atlas (512×64 px) of a [OBJECT] mounted on a wall.
Each frame is 64×64 px showing the object from a different angle in the EXACT order below.
Use a transparent background (PNG with alpha channel).

CRITICAL REQUIREMENTS - FOLLOW EXACTLY:
- All 8 frames MUST appear in a single horizontal row, in this EXACT LEFT-TO-RIGHT order:

  Position 1 (Frame 0): Object facing camera directly (0° rotation)
    - All front-facing surfaces fully visible
    - Mounting bracket centered, facing user
    - Brightest/most detailed view

  Position 2 (Frame 1): Object rotated 45° CLOCKWISE from front
    - Right-front surfaces visible, left surfaces hidden
    - Mounting bracket visible on left edge (receding away)
    - Transitional view

  Position 3 (Frame 2): Object rotated 90° CLOCKWISE (pure SIDE view from RIGHT)
    - Only right profile visible
    - Pure side profile perpendicular to camera
    - Mounting bracket on left edge
    - MUST be perfect HORIZONTAL MIRROR of Position 7

  Position 4 (Frame 3): Object rotated 135° CLOCKWISE from front
    - Right-back surfaces visible, front mostly hidden
    - Mounting bracket on left side
    - MUST be perfect HORIZONTAL MIRROR of Position 6

  Position 5 (Frame 4): Object rotated 180° (BACK view, facing AWAY from camera)
    - All back surfaces fully visible
    - CRITICAL: Show mounting hardware ONLY on back side
    - NEVER show duplicate brackets - if visible on front, it's wrong
    - Opposite of Position 1

  Position 6 (Frame 5): Object rotated 225° CLOCKWISE from front
    - Left-back surfaces visible, front mostly hidden
    - Mounting bracket on right side
    - MUST be a perfect HORIZONTAL MIRROR of Position 4

  Position 7 (Frame 6): Object rotated 270° CLOCKWISE (pure SIDE view from LEFT)
    - Only left profile visible
    - Pure side profile perpendicular to camera (opposite from Position 3)
    - Mounting bracket on right edge
    - MUST be perfect HORIZONTAL MIRROR of Position 3

  Position 8 (Frame 7): Object rotated 315° CLOCKWISE from front
    - Left-front surfaces visible, right surfaces hidden
    - Mounting bracket on right side (receding away)
    - MUST be perfect HORIZONTAL MIRROR of Position 2

MIRROR VALIDATION (REQUIRED):
- Position 2 flipped horizontally = Position 7 (identical after flip)
- Position 1 flipped horizontally = Position 8 (identical after flip)
- Position 3 flipped horizontally = Position 6 (identical after flip)
- If mirrors don't match, one is rotated wrong - REJECT AND REGENERATE

GEOMETRY RULES:
- All frames show same object from different angles
- Glass panes/panels always glass, metal always metal - NO SWAPS
- All mounting hardware faces OUTWARD relative to object
- No duplicate brackets visible from two directions
- All frames same canvas size, object centered consistently

NO EXTRA ELEMENTS:
- Only the [OBJECT] itself - no light bulbs, wires, decorative plates, or separate parts
```

## Validation Checklist for Any Atlas

Before using an atlas:

1. **Frame count**: Exactly 8 frames in one horizontal row ✓
2. **Angle verification** (use a protractor or grid overlay):
   - Frame 0 (left-most): object front-facing ✓
   - Frame 2 (1/4 way): object 90° rotated (pure side view) ✓
   - Frame 4 (center): object 180° rotated (back view, upside-down) ✓
   - Frame 6 (3/4 way): object 270° rotated (opposite side view) ✓
3. **Hardware path** (brackets/screws):
   - Frame 0: bracket centered ✓
   - Frame 2: bracket left edge ✓
   - Frame 4: bracket back only (never front) ✓
   - Frame 6: bracket right edge ✓
4. **Mirror pairs** (flip images horizontally and compare):
   - Frame 2 ↔ Frame 6: identical after flip ✓
   - Frame 1 ↔ Frame 7: identical after flip ✓
   - Frame 3 ↔ Frame 5: identical after flip ✓
5. **Geometry consistency** (zoom in and verify):
   - Glass panels always glass, metal always metal ✓
   - No missing features in any frame ✓
   - No swapped materials or duplicate hardware ✓
6. **Canvas**: All frames same size, object centered consistently ✓
7. **Transparency**: Real PNG alpha channel (not fake black) ✓

If ANY check fails, the atlas will render incorrectly in-game. Fix or regenerate.

## Build & test

```bash
npm run build
npm run dev
```

Open the URL shown (typically http://localhost:5173).

## Troubleshooting

- No sprites? Check `ls -lh assets/lamp-wall.png` and browser console.
- Wrong frames? Verify frame order and consistency.
- Transparency issues? Use PNG with real alpha channel.

## Next steps

- Add more sprite rows (one row per type) or separate atlas files for additional types.
- Implement animated sprites by stacking animation frames vertically (one animation per row) — ask if you want an example loader change.

See SPRITE_SYSTEM.md for advanced usage.
