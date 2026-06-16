# Dropdown Menu Integration Guide

This guide explains how to use `DropdownMenu` for completion/selection popups in txv applications. Three patterns exist depending on the context.

## Architecture

```
DropdownMenu<D: DropdownSource>  — the popup widget (renders bordered list)
SidekickManager                  — postprocess view that hosts/positions the popup
SidekickRequest                  — message payload: view + rect + emitter_id
CM_SIDEKICK_SHOW / CM_SIDEKICK_HIDE — commands to show/hide the popup
```

The popup is always managed by `SidekickManager` (a postprocess view at the top level). The caller emits `CM_SIDEKICK_SHOW` with a `SidekickRequest` containing the dropdown and positioning info. The sidekick manager positions it relative to the emitter view using `origin_of` (recursive tree traversal).

## Pattern 1: InputLine with built-in completion

InputLine has built-in completion support. No manual dropdown management needed.

```rust
let input = InputLine::new()
    .with_completer(Box::new(MyCompleter::new()))
    .with_command(CM_MY_DONE);
```

The InputLine handles Tab internally:
- Calls `completer.complete()` to get items
- If 1 match: auto-completes inline
- If multiple: creates a `DropdownMenu`, emits `CM_SIDEKICK_SHOW`
- On accept: fills text, emits `CM_SIDEKICK_HIDE`

### Completer trait

```rust
impl txv_core::complete::Completer for MyCompleter {
    fn complete(&self, text: &str, cursor: usize, cb: &mut dyn FnMut(Completion) -> Result<bool, ()>) -> usize {
        // Return items via callback. Return value = prefix length.
        for item in self.find_matches(text, cursor) {
            if cb(Completion::new(item.text, item.display))? == false {
                break; // caller says enough
            }
        }
        prefix_len
    }
}
```

### Positioning

InputLine passes `rect = Rect::new(0, 0, w, h)` (no cursor offset). The sidekick manager detects `(0,0)` and places the popup above the status bar: `y = screen_h - h - 2`.

## Pattern 2: Status bar ModalKey + InputLine

For key-triggered prompts (like Ctrl+P file finder):

```rust
let input = InputLine::new()
    .with_completer(Box::new(FileFinderCompleter::new(root)))
    .with_command(CM_FILE_OPEN);

let finder = ModalKey::new("", "file: ")
    .trigger_key(KeyEvent::new(KeyCode::Char('p'), KeyMod::CTRL))
    .terminal_command(CM_FILE_OPEN)
    .add_child(Box::new(input));

bar.add(StatusSlot::new(Box::new(finder)).priority(5).stretch(1));
```

Key points:
- `ModalKey` shows/hides on trigger key press
- `.stretch(1)` gives the input room to expand when active
- Completion works the same as Pattern 1 (InputLine handles it)
- `terminal_command` hides the ModalKey after the command fires

## Pattern 3: Custom widget (EditorView completion)

For LSP completion in the editor, the delegate manages the dropdown manually:

### 1. Implement DropdownSource

```rust
pub struct LspCompletionSource {
    items: Vec<CompletionItem>,
}

impl DropdownSource for LspCompletionSource {
    fn len(&self) -> usize { self.items.len() }

    fn label(&self, idx: usize) -> &str {
        self.items.get(idx).map(|i| &i.label as &str).unwrap_or("")
    }

    fn secondary(&self, idx: usize) -> &str {
        self.items.get(idx).and_then(|i| i.detail.as_deref()).unwrap_or("")
    }
}
```

### 2. Show the dropdown

Emit `CM_SIDEKICK_SHOW` with cursor position in `rect.x/y`:

```rust
fn show_popup(&mut self, items: Vec<CompletionItem>, editor: &Editor) {
    let source = LspCompletionSource::new(items);
    let menu = DropdownMenu::new(source)
        .with_numbers(NumberMode::None)
        .with_filter(FilterMode::None)
        .with_open_side(OpenSide::None)
        .with_cursor(self.selected);

    let h = items.len().min(8) as u16 + 2; // content + border
    let w = compute_width(&items);

    // Cursor position relative to this view's origin:
    let gw = compute_gutter_width(editor, self);
    let scroll = editor.viewport_scroll();
    let cx = gw + editor.cursor_col().saturating_sub(editor.h_scroll()) as u16;
    let cy = visual_cursor_row(editor, gw); // accounts for wrap + sticky

    // rect x,y = cursor offset within emitter view
    let rect = Rect::new(cx, cy, w, h);
    let data = SidekickRequest::new(Box::new(menu), rect, self.view_id);
    self.emit(CM_SIDEKICK_SHOW, Some(Box::new(data)));
}
```

### 3. Handle navigation keys

The dropdown is passive (not focused). The editor intercepts keys:

```rust
fn handle_completion_key(&mut self, key: &KeyEvent, editor: &mut Editor) -> Option<HandleResult> {
    if !self.completion_visible { return None; }
    match key.code() {
        KeyCode::Down => { self.selected = (self.selected + 1) % self.items.len(); self.show_popup(...); }
        KeyCode::Up   => { self.selected = (self.selected + len - 1) % len; self.show_popup(...); }
        KeyCode::Tab | KeyCode::Right => { self.accept_selected(editor); }
        KeyCode::Esc  => { self.hide(); }
        KeyCode::Enter => { self.hide(); } // let Enter pass through
        _ => { self.hide(); } // any other key dismisses
    }
    Some(HandleResult::Consumed)
}
```

Each navigation recreates the dropdown with updated `.with_cursor(selected)`.

### 4. Hide the dropdown

```rust
fn hide(&mut self) {
    self.completion_visible = false;
    self.items.clear();
    self.selected = 0;
    self.emit(CM_SIDEKICK_HIDE, None);
}
```

### 5. Positioning details

The `SidekickManager` receives `CM_SIDEKICK_SHOW` and:
1. Extracts `rect.x, rect.y` as cursor offset within the emitter
2. Calls `origin_of(emitter_id)` on the parent group to find the emitter's absolute position
3. Computes final position: `emitter_origin + (cx, cy + 2)`
4. Clamps to screen bounds (right edge, bottom → mirror above)
5. Emits `CM_REPOSITION` to place itself at the computed position

### Critical requirement: group_state()

For `origin_of` to find your view, every group-based view in the ancestry chain MUST implement `group_state()`. If any wrapper view uses `delegate_view!` without overriding `group_state`, the traversal breaks.

For wrapper views around a group-based inner view:
```rust
impl View for MyWrapper {
    delegate_view!(inner, override { group_state, ... });

    fn group_state(&self) -> Option<&GroupState> {
        self.inner.group_state() // delegate to inner group
    }
}
```

## DropdownMenu configuration

```rust
DropdownMenu::new(source)
    .with_numbers(NumberMode::None)      // None | All | SkipFirst
    .with_filter(FilterMode::Prefix)     // None | Prefix | Substring | Subsequence
    .with_open_side(OpenSide::None)      // None | Top | Bottom (border gap)
    .with_cursor(index)                  // pre-select item
    .with_border_style(style)            // custom border color
    .with_max_visible(12)                // max rows shown
```

## Commands reference

| Command | Direction | Data | Purpose |
|---------|-----------|------|---------|
| CM_SIDEKICK_SHOW | emit upward | `Box<SidekickRequest>` | Show popup |
| CM_SIDEKICK_HIDE | emit upward | None | Hide popup |
| CM_DROPDOWN_DONE | from dropdown | `Box<usize>` (index) | Item selected via Enter |
| CM_DROPDOWN_CANCELLED | from dropdown | None | Esc pressed in dropdown |
| CM_DROPDOWN_CHANGED | from dropdown | `Box<usize>` (index) | Cursor moved |
