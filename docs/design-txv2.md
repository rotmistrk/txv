# txv2 — TUI Framework Redesign

## Principles

1. Framework owns the window tree — windows are pure behavior
2. StyleId everywhere — theming is free, cells are small
3. Drawable trait hides buffer/blit — windows draw at (0,0)
4. GlyphId for all chrome — backend resolves to chars or vectors
5. Three-phase dispatch lives in EventLoop, not in windows
6. No macros, no delegation — just implement the Window trait

## Core Traits

### Window
- on_key, on_cmd, on_focus, on_blur, on_resize, on_tick
- draw(&mut dyn Drawable)
- options() -> WindowOptions (focusable, preprocess, postprocess, modal)

### Drawable
- put, print, fill (text + StyleId)
- glyph, hline, vline (chrome + GlyphId + StyleId)
- width, height, clip -> ClipArea (zero-alloc sub-region)
- place_image

## Core Structs

### WindowData (framework-owned)
- id, parent, children, buffer, position, size, options, dirty, visible, focused

### Buffer (implements Drawable)
- cells: Vec<Cell> where Cell = { ch: char, style_id: u8, width: u8 }
- images: Vec<ImagePlacement>

### ClipArea (implements Drawable)
- borrows parent Drawable, offsets coordinates, clips bounds

### Palette / GlyphSet
- StyleId indexes into Palette (resolved at flush time)
- GlyphId indexes into GlyphSet (resolved at draw time)

### EventLoop
- Owns WindowTree (SlotMap<WindowId, (WindowData, Box<dyn Window>)>)
- Cycle: poll -> dispatch(3-phase) -> geometry -> render(dirty subtrees)

## Dispatch (framework, not windows)
1. Preprocess children
2. Focused (or modal) window
3. Postprocess children

## Render (framework, not windows)
1. Walk dirty subtrees
2. Render children first (recursive)
3. Call window.draw(&mut buffer)
4. Blit children on top (framework does this)

## Import from txv
- PieceTable + undo (as-is)
- Editor motions/commands/keymap (adapt)
- Crossterm event translation (as-is)
- VTE TermBuf (as-is)
- Image protocol + PNG encoder (as-is)
- Diff-flush (rewrite for new cell)
- All widgets (rewrite, simpler)
- Delete: GroupState, ViewState, delegation macros, EventSink
