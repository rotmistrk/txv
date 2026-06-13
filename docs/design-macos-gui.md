# macOS GUI Backend — Design

## Overview

A native macOS GUI backend for txv that renders the same `Buffer` cell grid
in a window using a monospace font. Supports mouse interaction, inline images,
Cmd+/- font scaling, and full clipboard integration.

## Architecture

```
┌─────────────────────────────────────────────┐
│  txv-core (View, Buffer, Event, Backend)    │
├─────────────────────────────────────────────┤
│  txv-macos (new crate)                      │
│    MacBackend: impl Backend                 │
│    - Window management (winit or AppKit)    │
│    - Glyph rendering (monospace atlas)      │
│    - Image blitting (texture from ImageData)│
│    - Event translation (NSEvent → Event)    │
│    - Cmd+/- font size adjustment            │
└─────────────────────────────────────────────┘
```

## Rendering Pipeline

Each frame (on flush):
1. Walk `Buffer` cells, render glyphs into a texture/surface
   - Background: fill cell rect with bg color
   - If bg is Transparent and image underneath → leave clear
   - Foreground: render glyph with fg color (or skip if Transparent)
2. Render images: for each `ImagePlacement`, blit into the backing store
   at pixel position = `rect.x * cell_w, rect.y * cell_h`
   clipped to visible (non-opaque-bg) cells
3. Composite: images render first, text on top (natural z-order from
   opaque bg cells covering image)
4. Draw cursor (blinking block/beam)

## Font Handling

- Default: system monospace font (SF Mono / Menlo)
- Cell size = font metrics (advance width × line height)
- Cmd+`=` increases font size, Cmd+`-` decreases
- On font size change: recalculate grid dimensions (cols = window_w / cell_w),
  emit `Event::Resize(new_cols, new_rows)` to root view

## Event Translation

| macOS Event          | txv Event                           |
|---------------------|-------------------------------------|
| Key down            | Event::Key (with Cmd→Ctrl mapping?) |
| Mouse click         | Event::Mouse(Press) at cell coords  |
| Mouse move/drag     | Event::Mouse(Move/Drag)            |
| Scroll wheel        | Event::Mouse(ScrollUp/Down)         |
| Window resize       | Event::Resize(cols, rows)           |
| Paste (Cmd+V)       | Event::Paste(text)                  |

### Coordinate Mapping
```
cell_col = pixel_x / cell_width
cell_row = pixel_y / cell_height
```

## Mouse Support (shared with terminal)

Mouse events require infrastructure in txv-core:
- `GroupState::dispatch` must hit-test mouse coords against child bounds
- Translate coords to child-relative before forwarding
- Click on unfocused child → focus + raise
- See separate mouse support design (not in this doc)

## Image Rendering in GUI

- `ImageData` (RGBA buffer) → GPU texture
- Cache textures by `Arc` pointer identity (avoid re-upload)
- Blit texture into window at pixel-exact position
- Transform (Fit/Fill/Stretch) applied via source/dest rect math
- Visible region = cells with Transparent bg within image rect

## Cmd+/- Font Scaling

- Track current font size (default: 14pt)
- Cmd+`=`/Cmd+`+` → size += 1
- Cmd+`-` → size -= 1 (min 8pt)
- Cmd+`0` → reset to default
- On change: recompute cell metrics, resize grid, emit Resize event

## Framework Options

| Option | Pros | Cons |
|--------|------|------|
| winit + wgpu | Cross-platform, GPU-accelerated | More boilerplate |
| winit + softbuffer | Simple, CPU rendering | Slower for large grids |
| raw AppKit (objc2) | Most native feel, direct Metal | macOS only, more code |

Recommendation: Start with **winit + wgpu** for portability, with option to
add a raw AppKit layer later for native menus/dialogs.

## Crate Structure

```
txv-macos/
  Cargo.toml          # deps: winit, wgpu, cosmic-text, txv-core
  src/
    lib.rs            # MacBackend, public API
    window.rs         # Window creation, event loop
    renderer.rs       # Cell grid → GPU texture
    glyph_atlas.rs    # Font rasterization, glyph cache
    image_cache.rs    # ImageData → GPU texture cache
    event.rs          # winit Event → txv Event translation
```
