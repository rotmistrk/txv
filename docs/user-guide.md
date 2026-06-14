# TXV User Guide

## Core Concepts

### View Trait

Every UI component implements `View`:

```rust
pub trait View: Send {
    fn draw(&mut self);
    fn handle(&mut self, event: &Event) -> HandleResult;
    fn bounds(&self) -> Rect;
    fn set_bounds(&mut self, rect: Rect);
    fn select(&mut self) {}
    fn unselect(&mut self) {}
    fn needs_redraw(&self) -> bool;
    fn mark_redrawn(&mut self);
    fn render(&mut self) -> bool;   // calls draw + mark_redrawn if dirty
    fn title(&self) -> &str { "" }
    fn options(&self) -> ViewOptions;
    fn set_sink(&mut self, sink: EventSink);
    fn cursor(&self) -> Option<CursorRequest> { None }
}
```

### ViewState

Embed `ViewState` in your view and delegate boilerplate:

```rust
pub struct MyView {
    state: ViewState,
}

impl View for MyView {
    delegate_view_state!(state, override { draw, handle });

    fn draw(&mut self) {
        self.state.buffer_mut().print(0, 0, "content", Style::default());
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        HandleResult::Ignored
    }
}
```

ViewState manages: bounds, dirty flag, own buffer, EventSink.
- `bounds()`, `is_dirty()`, `mark_dirty()`
- `buffer()`, `buffer_mut()` — the cell grid this view draws into
- `put_command(id, data)` — emit a command via the EventSink
- `set_bounds(rect)` — resizes the buffer automatically

### Render Lifecycle

```
Parent calls child.render()
  → if needs_redraw():
      1. Render all visible children (recursive)
      2. Call self.draw() — write to own buffer
      3. Blit children's buffers on top (automatic via macro)
      4. mark_redrawn()
```

**Rules:**
- `draw()` is ONLY called by `render()` — never directly
- `blit_child()` is ONLY called by `render()` — via the macro
- `set_child_bounds()` is ONLY called from `handle()` or `set_bounds()` — NEVER from `draw()`
- Views draw into their OWN buffer at coordinates (0,0) — parent handles positioning

### Buffer — The Drawing Target

Each view owns a `Buffer` (cell grid). Views draw into it during `draw()`:

```rust
fn draw(&mut self) {
    let buf = self.group.buffer_mut();
    buf.fill(' ', Style::default());
    buf.print(0, 0, "Hello", style);
    buf.hline(0, 1, 20, '─', dim_style);
    buf.put(5, 3, '●', highlight_style);
}
```

Buffer methods: `fill()`, `print()`, `put()`, `hline()`, `vline()`, `cell()`, `cell_mut()`.

### Inline Images

Views can place images behind text using the Buffer image API:

```rust
use std::sync::Arc;

fn draw(&mut self) {
    let buf = self.state.buffer_mut();
    // Fill with transparent so image shows through
    let transparent = Style::new(Color::Transparent, Color::Transparent);
    buf.fill(' ', transparent);
    // Place image covering entire bounds
    let rect = Rect::new(0, 0, buf.width(), buf.height());
    buf.place_image(rect, self.image_data.clone(), ImageTransform::Fit);
}
```

Images are rendered by the terminal backend using iTerm2 or Kitty protocols.
Text with opaque background draws on top of images. The rendering pipeline:
- Images propagate through blit (coordinates offset by parent origin)
- Images cleared at start of each render cycle (no accumulation)
- Terminal emits PNG-encoded image at the absolute screen position

## Events

```rust
pub enum Event {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Paste(String),
    Tick,
    Resize(u16, u16),
    Command { id: CommandId, data: Option<Box<dyn Any + Send>>, broadcast: bool },
}
```

### Key Events

```rust
pub struct KeyEvent {
    code: KeyCode,
    modifiers: KeyMod,
}
```

Access via `key.code()`, `key.modifiers()`. Modifiers: `.ctrl()`, `.alt()`, `.shift()`.

KeyCode: `Char(char)`, `Enter`, `Esc`, `Tab`, `BackTab`, `Backspace`, `Delete`,
`Left`, `Right`, `Up`, `Down`, `Home`, `End`, `PageUp`, `PageDown`, `F(u8)`.

### HandleResult

```rust
pub enum HandleResult {
    Consumed,  // event was handled, stop dispatch
    Ignored,   // pass to next handler
}
```

## Groups — Owning Children

Views that contain other views embed `GroupState`:

```rust
pub struct MyPanel {
    group: GroupState,
}

impl View for MyPanel {
    delegate_group_state!(group, override { set_bounds, draw, handle });

    fn set_bounds(&mut self, r: Rect) {
        self.group.set_bounds(r);
        // Layout children
        self.group.set_child_bounds(0, Rect::new(0, 0, r.w(), 1));
        self.group.set_child_bounds(1, Rect::new(0, 1, r.w(), r.h() - 1));
    }

    fn draw(&mut self) {
        // Draw chrome/background. Children blit on top automatically.
        self.group.buffer_mut().fill(' ', Style::default());
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        if matches!(event, Event::Tick) {
            // Broadcast ticks to ALL children
            for i in 0..self.group.child_count() {
                if let Some(child) = self.group.child_mut(i) {
                    child.handle(event);
                }
            }
            return HandleResult::Ignored;
        }
        self.group.dispatch(event)
    }
}
```

### Three-Phase Dispatch

`group.dispatch()` routes events in three phases:

1. **Preprocess** — children with `options().preprocess() == true` see events first
2. **Focused** — the focused child (or modal child) handles the event
3. **Postprocess** — children with `options().postprocess() == true` see unhandled events

### GroupState API

```rust
// Child access
group.child(index) -> Option<&dyn View>
group.child_mut(index) -> Option<&mut Box<dyn View>>
group.child_count() -> usize

// Focus management
group.focused_index() -> usize
group.set_focused_index(index)
group.switch_focus(new_index)  // unselects old, selects new, marks dirty

// Layout
group.set_child_bounds(index, rect)
group.set_child_visible(index, bool)

// Mutation
group.insert(child: Box<dyn View>)
group.remove(index) -> Box<dyn View>

// Drawing
group.buffer_mut() -> &mut Buffer
group.mark_dirty()
```

## Commands — Decoupled Communication

Views communicate via commands through the EventSink, never calling each other:

```rust
fn handle(&mut self, event: &Event) -> HandleResult {
    if let Event::Key(k) = event {
        if k.code() == KeyCode::Char('o') && k.modifiers().ctrl() {
            self.state.put_command(CM_OPEN_FILE, Some(Box::new(path)));
            return HandleResult::Consumed;
        }
    }
    HandleResult::Ignored
}
```

Commands are drained from the sink after each event is processed and delivered
back to the root view as `Event::Command { ... }`.

## Layout

Parent views set bounds on children via `set_child_bounds(index, rect)`.
There is no automatic layout engine — parents compute child rectangles
explicitly in their `set_bounds()` override:

```rust
fn set_bounds(&mut self, r: Rect) {
    self.group.set_bounds(r);
    self.group.set_child_bounds(0, Rect::new(0, 0, 30, r.h()));
    self.group.set_child_bounds(1, Rect::new(30, 0, r.w() - 30, r.h()));
}
```

### StatusBar Layout

StatusBar is special — it uses a priority-based slot system:
- Each slot has a `priority` (higher = more important) and optional `stretch`
- Layout allocates space to highest-priority items first
- Stretch items fill remaining space proportionally
- Slots below the minimum width are hidden

StatusBar children are laid out by the bar itself during its `draw()` — this
is the ONE exception to "don't layout in draw" because StatusBar items resize
dynamically based on available space (the bar's own width can change frame to frame).

## Widgets Reference

### TabPanel
Tabbed container. TabBar (preprocess child) at top, content views below.
Only active tab is visible. Supports Static/LRU/Single tab bar modes.
Alt-digit switches tabs (including macOS Option-digit chars).

### TiledWorkspace
Multi-panel workspace with wide/narrow layouts. Each panel is a TabPanel
inside a SplitPanel. Supports directional focus, resize, zoom, tab management.

### SplitPanel
N children arranged with proportional sizing and divider lines.

### PtyTerminal
Full terminal emulator (VTE + PTY). Scrollback, OSC title, mouse passthrough.

### TextArea
Multi-line read-only text with optional line numbers and syntax highlighting.

### Editor (txv-edit)
Vi-style editor: normal/insert/visual modes, motions, :commands, search,
undo/redo, syntax highlighting, wrap, scrolloff.

### TreeView / TreeTableView
Hierarchical tree with expand/collapse. Implement `TreeData` for your model.

### ListView
Scrollable list with selection. Implement `ListData`.

### DropdownMenu
Filtered popup list with numbered selection and badges.

### ModalKey
Status bar item that captures input on trigger key. Used for Go-to, search, etc.

### ImageView
Displays an inline image filling its bounds. Set image data via `set_image(Arc<ImageData>)`.
Works in terminals that support iTerm2 or Kitty graphics protocols.
Requires `tmux set -g allow-passthrough on` when running inside tmux.

### InputLine
Single-line text input with readline bindings and tab completion.

### StatusBar
Horizontal bar with priority-sorted pluggable items.
