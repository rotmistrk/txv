# TXV User Guide

## Core Concepts

### View Trait

Every UI component implements `View`:

```rust
pub trait View: Send {
    fn draw(&self, surface: &mut Surface);
    fn handle(&mut self, event: &Event, queue: &mut EventQueue) -> HandleResult;
    fn bounds(&self) -> Rect;
    fn set_bounds(&mut self, rect: Rect);
    fn select(&mut self) {}
    fn unselect(&mut self) {}
    fn needs_redraw(&self) -> bool;
    fn mark_redrawn(&mut self);
    fn title(&self) -> &str { "" }
    fn options(&self) -> ViewOptions;
}
```

### ViewState

Embed `ViewState` in your view and delegate boilerplate:

```rust
pub struct MyView {
    state: ViewState,
    content: String,
}

impl MyView {
    pub fn new(content: &str) -> Self {
        Self {
            state: ViewState::new(ViewOptions { focusable: true, ..Default::default() }),
            content: content.to_owned(),
        }
    }
}

impl View for MyView {
    delegate_view_state!(state);

    fn draw(&self, surface: &mut Surface) { /* ... */ }
    fn handle(&mut self, event: &Event, queue: &mut EventQueue) -> HandleResult {
        HandleResult::Ignored
    }
}
```

ViewState manages: bounds, dirty flag, focused state. Access via methods:
- `bounds()`, `is_dirty()`, `is_focused()`
- `mark_dirty()`, `mark_redrawn()`, `set_bounds()`, `set_focused()`

### Override Specific Delegated Methods

```rust
impl View for MyView {
    delegate_view_state!(state, override { title, needs_redraw });

    fn title(&self) -> &str { &self.content }
    fn needs_redraw(&self) -> bool { true }
    // draw and handle still required
}
```

## Events

```rust
pub enum Event {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Tick,
    Resize(u16, u16),
    Command(CommandId, Option<Box<dyn Any + Send>>),
}
```

### Key Events

```rust
pub struct KeyEvent {
    pub code: KeyCode,
    pub modifiers: KeyMod,
}
```

KeyCode includes: `Char(char)`, `Enter`, `Esc`, `Tab`, `Backspace`, `Delete`,
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
    delegate_group_state!(group);

    fn draw(&self, surface: &mut Surface) {
        for child in self.group.children_iter() {
            child.draw(surface);
        }
    }

    fn handle(&mut self, event: &Event, queue: &mut EventQueue) -> HandleResult {
        self.group.dispatch(event, queue)
    }
}
```

### Three-Phase Dispatch

`group.dispatch()` routes events in three phases:

1. **Preprocess** — children with `options().preprocess = true` see events first
   (e.g., StatusBar intercepts certain keys)
2. **Focused** — the focused child (or modal child) handles the event
3. **Postprocess** — children with `options().postprocess = true` see unhandled events

### GroupState API

```rust
// Child access
group.child(index) -> Option<&dyn View>
group.child_mut(index) -> Option<&mut Box<dyn View>>
group.focused_child_mut() -> Option<&mut Box<dyn View>>
group.children_iter() -> impl Iterator<Item = &dyn View>
group.children_iter_mut() -> impl Iterator<Item = &mut Box<dyn View>>

// Focus management
group.focused_index() -> usize
group.set_focused_index(index)
group.switch_focus(new_index)  // unselects old, selects new, marks dirty

// Mutation
group.insert(child: Box<dyn View>)
group.remove(index) -> Box<dyn View>
group.set_child_bounds(index, rect)
group.child_count() -> usize
group.is_empty() -> bool
```

## Commands — Decoupled Communication

Views communicate via commands, never calling each other directly:

```rust
fn handle(&mut self, event: &Event, queue: &mut EventQueue) -> HandleResult {
    if let Event::Key(k) = event {
        if k.code == KeyCode::Char('o') && k.modifiers.ctrl {
            queue.put_command(CM_OPEN_FILE, Some(Box::new(path)));
            return HandleResult::Consumed;
        }
    }
    HandleResult::Ignored
}
```

Commands bubble up to the `Program` handler where they're dispatched to actions.

## Program — The Event Loop

```rust
let mut program = Program::new(status_bar, desktop);
program.run(&mut backend, |ctx| {
    match ctx.command {
        CM_OPEN_FILE => { /* handle */ }
        CM_QUIT => { /* handled automatically */ }
        _ => {}
    }
});
```

Program handles: event polling, Tick (50ms), draw cycle (dirty-only), command routing.

## Surface — Drawing

```rust
fn draw(&self, surface: &mut Surface) {
    let b = self.bounds();
    surface.write_str(b.x, b.y, "text", Style::default());
    surface.fill(b, Cell::new(' ', style));
    surface.hline(b.x, b.y + 1, b.w, '─', style);
}
```

Surface is a 2D cell buffer. Only cells that changed since last flush are written
to the terminal (diff flush via `txv-render`).

## Layout

Parent views set bounds on children via `set_bounds(rect)`. There is no automatic
layout engine — parents compute child rectangles explicitly. This gives full control:

```rust
fn set_bounds(&mut self, r: Rect) {
    self.group.set_bounds(r);
    // Left panel: 30 columns
    self.group.set_child_bounds(0, Rect::new(r.x, r.y, 30, r.h));
    // Right panel: remainder
    self.group.set_child_bounds(1, Rect::new(r.x + 30, r.y, r.w - 30, r.h));
}
```

## Widgets Reference

### TabGroup
Tabbed container — multiple views, one active at a time. Tab bar drawn at top.

### PtyTerminal
Full terminal emulator (VTE + PTY). Scrollback buffer, OSC title tracking.

### TextArea
Multi-line text display with syntax highlighting and line numbers.

### TreeView
Hierarchical tree with expand/collapse. Implement `TreeData` trait for your model.

### ListView
Scrollable list. Implement `ListData` trait.

### SplitPane
Two children side-by-side (horizontal or vertical) with configurable ratio.

### SplitPanel
N children arranged horizontally or vertically with proportional sizing and
divider lines. Supports dynamic resize, focus cycling, and a `chrome_row` mode
where row 0 is reserved for parent chrome (divider starts at row 1).

### TabPanel
Tabbed container: a TabBar on top with stacked child views below. Only the
active tab is drawn and receives events. Propagates `needs_redraw` from children.

### TiledWorkspace
Multi-panel workspace with configurable wide/narrow layouts defined as
`SplitNode` trees. Features:
- Automatic layout switching at a width threshold
- Ring-based panel navigation (Ctrl+Shift+Arrow)
- Directional panel resize (Alt+Shift+Arrow moves the adjacent border)
- Panel zoom, tab management, and subpanel focus cycling
- Chrome drawing with horizontal tier lines and vertical gap connectors

### StatusBar
Bottom bar with pluggable items (clock, mode indicator, cursor position, messages).

### InputLine / InputDialog
Single-line text input, optionally wrapped in a modal dialog.

### FuzzySelect
Fuzzy-filtered list selection (like Ctrl-P file picker).

### Menu
Popup menu with keyboard navigation.

### Table
Column-based data display with headers.

### Overlay
Renders a child view on top of existing content (for popups/modals).
