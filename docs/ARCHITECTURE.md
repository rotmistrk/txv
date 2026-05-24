# TXV Architecture — View, Group, and Dispatch

## The Two Building Blocks

Every TXV widget is one of:

| Type | Embed | Use when |
|------|-------|----------|
| **Leaf View** | `ViewState` | Widget has NO child views (editor, terminal, tree) |
| **Group** | `GroupState` | Widget OWNS one or more child views |

There is no third option. If your widget contains `Box<dyn View>` or
`Vec<Box<dyn View>>`, it MUST use `GroupState`.

## Decision Tree

```
Does your widget own child views?
├── NO  → ViewState + delegate_view_state!
└── YES → GroupState + delegate_group_state!
          └── Override: set_bounds (layout), draw (chrome/blit), handle (tick broadcast)
```

## GroupState Provides (DO NOT REIMPLEMENT)

- `children: Vec<Box<dyn View>>` — child storage
- `focused: usize` — focus tracking with `switch_focus()` (auto select/unselect)
- `dispatch(&Event)` — three-phase event routing
- `set_sink()` propagation to all children
- `any_dirty()` — aggregated needs_redraw
- `mark_redrawn()` propagation
- `cursor()` forwarding from focused child
- `insert()` / `remove()` — child management with sink propagation

## Three-Phase Dispatch

```
Event arrives at Group
  │
  ├─ Phase 1: PREPROCESS
  │  Children with options().preprocess == true see event first.
  │  If any returns Consumed → stop.
  │  Use case: status bar translates keys→commands, tab bar intercepts tab keys.
  │
  ├─ Phase 2: FOCUSED CHILD (or modal child)
  │  The focused child handles the event.
  │  If it returns Consumed → stop.
  │  Use case: the main content view processes the event.
  │
  └─ Phase 3: POSTPROCESS
     Children with options().postprocess == true see event last.
     Use case: scroll sync between split panes, logging, side effects.
```

### When to Use Each Phase

| Phase | Role | Example |
|-------|------|---------|
| Preprocess | Intercept keys before content | TabBar in TabPanel, StatusBar in Program |
| Focused | Normal content handling | Editor, terminal, tree |
| Postprocess | React after content handled | Scroll sync, linked views, observers |

## The Transparent Content Trick

Groups often need to draw decorations (dividers, borders, chrome) that sit
BEHIND their children. The pattern:

1. Group draws decorations into its own buffer (dividers, separators)
2. Children draw into their own buffers
3. Group blits children ON TOP of its buffer
4. Children use `Color::Transparent` for cells that should show the parent's decoration

```rust
// In child's draw():
let transparent = Style {
    fg: Color::Transparent,
    bg: Color::Transparent,
    ..Style::default()
};
self.buffer_mut().put(col, row, ' ', transparent);
// Parent's divider/chrome shows through these cells
```

This is how:
- SplitPanel draws dividers between children
- TabPanel's tab bar row shows the parent's horizontal separator
- TiledWorkspace draws panel borders

## Tick Broadcast

`Tick` events are state-update signals, not user input. Groups MUST broadcast
ticks to ALL children (not just focused). Override `handle()`:

```rust
fn handle(&mut self, event: &Event) -> HandleResult {
    if matches!(event, Event::Tick) {
        for i in 0..self.group.child_count() {
            if let Some(child) = self.group.child_mut(i) {
                child.handle(event);
            }
        }
        return HandleResult::Ignored;
    }
    self.group.dispatch(event)
}
```

## Antipatterns — NEVER DO THESE

### ❌ Manual children Vec in a View

```rust
// WRONG — reimplements GroupState by hand
struct MyPanel {
    state: ViewState,           // ← should be GroupState
    children: Vec<Box<dyn View>>,  // ← GroupState owns this
    focused: usize,            // ← GroupState tracks this
}
```

If you find yourself writing `children: Vec<Box<dyn View>>` with a `focused`
index, you are reimplementing GroupState. Stop. Use GroupState.

### ❌ Manual set_sink propagation

```rust
// WRONG — GroupState::set_sink() does this automatically
fn set_sink(&mut self, sink: EventSink) {
    self.state.set_sink(sink.clone());
    for child in &mut self.children {
        child.set_sink(sink.clone());
    }
}
```

### ❌ Manual focused-child dispatch

```rust
// WRONG — GroupState::dispatch() does this with three-phase support
fn handle(&mut self, event: &Event) -> HandleResult {
    if let Some(child) = self.children.get_mut(self.focused) {
        return child.handle(event);
    }
    HandleResult::Ignored
}
```

### ❌ Custom "bar-first" dispatch instead of preprocess

```rust
// WRONG — the bar should be a preprocess child
fn handle(&mut self, event: &Event) -> HandleResult {
    let result = self.bar.handle(event);
    if result == HandleResult::Consumed { return result; }
    self.children[self.active].handle(event)
}
```

The bar is a preprocess view. Make it a child with `preprocess: true`.

### ❌ Wrapping a Group to add behavior

```rust
// WRONG — adding a wrapper view around a Group to intercept events
struct EditorSplit {
    split: SplitPanel,  // ← should be a postprocess child INSIDE the group
    linked_scroll: bool,
}
```

If you need behavior after the focused child handles an event, add a
postprocess child to the existing group. Don't wrap the group in another view.

## Correct Patterns

### Split container (SplitPanel)

```rust
struct SplitPanel {
    group: GroupState,      // owns children
    proportions: Vec<f32>,  // layout metadata
    direction: SplitDir,    // layout metadata
}
// delegate_group_state!(group, override { set_bounds, draw, handle })
// set_bounds: compute child rects from proportions
// draw: dividers + blit children
// handle: tick broadcast + group.dispatch()
```

### Tabbed container (TabPanel)

```rust
struct TabPanel {
    group: GroupState,  // child 0 = bar (preprocess), children 1..N = tabs
}
// Bar is child 0 with preprocess: true
// Active tab is the focused child (index 1..N)
// Tick broadcast to ALL children (background tabs need updates)
```

### Adding linked scroll to a split

```rust
// ScrollSyncView is a postprocess child inside the SplitPanel's group
struct ScrollSyncView {
    state: ViewState,  // options: { postprocess: true }
}
impl View for ScrollSyncView {
    fn handle(&mut self, event: &Event) -> HandleResult {
        // After focused editor handles scroll, sync sibling
        HandleResult::Ignored  // never consume — just observe
    }
}
```

## Summary

| Need | Solution |
|------|----------|
| Leaf widget (no children) | `ViewState` + `delegate_view_state!` |
| Container with children | `GroupState` + `delegate_group_state!` |
| Intercept before content | Preprocess child |
| React after content | Postprocess child |
| Draw behind children | Transparent cells in children |
| Broadcast to all children | Override `handle()` for Tick |
