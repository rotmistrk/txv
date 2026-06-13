# Bug: Scroll-to-cursor doesn't account for wrapped lines

## Symptoms
When viewing a file with long lines (e.g. JSON with long strings) and `wrap` is enabled, the cursor can go below the visible viewport. Moving down past a sequence of wrapped lines causes the cursor to disappear below the screen.

## Root Cause
The scroll/viewport logic in txv-edit uses **buffer line numbers** for scroll calculations (viewport_scroll, ensure_cursor_visible, etc.). When wrap is enabled, a single buffer line can occupy multiple **visual rows**. The scroll math doesn't account for this — it assumes 1 buffer line = 1 visual row.

## Where to Fix
- `txv-edit/src/view/draw/viewport.rs` or equivalent — wherever `viewport_scroll` is compared against `cursor_line`
- `txv-edit/src/editor/mod.rs` — `viewport_scroll`, `viewport_height`, `set_viewport_scroll`
- The `ensure_cursor_visible` / scroll logic needs to count visual rows (sum of wrapped line heights) not buffer lines

## Expected Behavior
When cursor is on buffer line N, the viewport should scroll so that the visual row containing line N (accounting for all wrapped lines above it) is within the visible area with at least `scrolloff` margin.

## Reproduction
1. Open a file with lines > terminal width (e.g. a JSON file with long strings)
2. Enable wrap (`:set wrap`)
3. Move cursor down past several long wrapped lines
4. Observe: cursor disappears below the viewport

## Complexity
LOE: 5 — requires changing the core scroll calculation to use visual row counting, which affects viewport_scroll, ensure_cursor_visible, PageUp/PageDown, and potentially mouse scrolling.
