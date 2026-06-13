# Image Support in Buffer — Design

## Overview

Add an image overlay layer to `Buffer` so views can place RGBA images
behind text cells. Text with transparent bg/fg reveals the image beneath.
Terminal backends render visible (uncovered) image regions using
iTerm2/Kitty inline image protocols.

## Data Model

```
Buffer {
    cells: Vec<Cell>,          // existing text grid
    images: Vec<ImagePlacement>,  // NEW: ordered back-to-front
}

struct ImagePlacement {
    rect: Rect,                // cell-based bounding rectangle
    data: Arc<ImageData>,      // shared decoded bitmap
    transform: ImageTransform, // fit/fill/stretch within rect
}

struct ImageData {
    width: u32,
    height: u32,
    pixels: Vec<u8>,           // RGBA, row-major
}

enum ImageTransform {
    Fit,       // scale to fit, letterbox
    Fill,      // scale to fill, crop
    Stretch,   // distort to fill rect exactly
}
```

## Compositing Rules

During rendering (flush), for each cell:
1. If cell has opaque bg → cell covers image entirely (skip image for this cell)
2. If cell bg is `Transparent` → image shows through as background
3. If cell fg is `Transparent` → image shows through character area (rare)
4. If cell is completely transparent (both fg/bg) → pure image region

The renderer determines **visible image regions** = image rect minus all
opaque-bg cells. Only those pixel areas are emitted to the terminal.

## Buffer API

```rust
impl Buffer {
    /// Place an image behind text in the given cell rect.
    pub fn place_image(&mut self, rect: Rect, data: Arc<ImageData>, transform: ImageTransform);

    /// Remove all images (called on resize/clear).
    pub fn clear_images(&mut self);

    /// Iterate image placements for rendering.
    pub fn images(&self) -> &[ImagePlacement];
}
```

## Terminal Rendering (Phase 2)

See `docs/design-terminal-images.md`.

## Future: Sub-cell Positioning

`ImageTransform` can be extended with offset/scale fields for pixel-accurate
positioning within the bounding rect. This is a non-breaking addition.
