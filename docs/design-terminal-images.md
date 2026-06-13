# Terminal Image Rendering — Design

## Overview

Render `Buffer` image placements in terminal emulators that support inline
images (iTerm2, Kitty, WezTerm, Ghostty). Detect protocol at startup,
emit only visible (non-covered) image regions during flush.

## Protocol Detection

At backend startup, probe terminal capabilities:
1. Check `$TERM_PROGRAM` / `$TERM` for known support
2. Optionally send Kitty graphics query (`\e_Gi=31,s=1,v=1,a=q...`) and
   check response
3. Store detected protocol in `CrosstermBackend`

```rust
enum ImageProtocol {
    None,         // no image support
    Iterm2,       // iTerm2 inline image (base64 PNG in OSC 1337)
    Kitty,        // Kitty graphics protocol (chunked RGBA)
}
```

## Visible Region Computation

For each `ImagePlacement` in the buffer:
1. Start with the full image rect
2. Subtract cells that have **opaque background** (non-Transparent bg)
3. Remaining cells = visible image area

Optimization: compute rectangular "runs" of visible cells per row to
minimize protocol overhead (one image command per contiguous visible run).

## Rendering Strategy

### iTerm2 Protocol
- Encode visible sub-image region as PNG
- Emit: `\e]1337;File=inline=1;width=Ncells;height=Nrows;...:BASE64\a`
- Position cursor before emitting (image draws at cursor position)

### Kitty Protocol
- Send RGBA pixel data in chunks
- Supports placement at specific cell positions
- Supports partial image display (crop source region)
- More efficient for updates (can reference previously uploaded images by ID)

## Diff-Awareness

Images are re-emitted only when:
- The image placement changed (different data or rect)
- Cells covering the image changed (opaque→transparent or vice versa)
- Terminal was resized (full redraw)

Store previous image state in `CrosstermBackend` alongside `previous` buffer
for diffing.

## Cell Size Detection

To map cells to pixels, we need the cell size in pixels:
- Kitty: query via `\e[16t` (reports cell size)
- iTerm2: query via `\e[14t` (window pixel size) ÷ terminal size
- Fallback: assume standard (8×16 or detected from font metrics)

## Limitations

- Terminals without image support simply skip image rendering (text only)
- Scrolling regions with images need special handling (may flash)
- Some terminals limit concurrent displayed images
