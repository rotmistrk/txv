# Skill: TXV Framework

## Purpose
Guide for building TUI widgets and applications using the txv framework.

## Core Concepts

### Two Building Blocks
- **Leaf View**: embed `ViewState`, use `delegate_view_state!(state, override { draw, handle })`
- **Group (container)**: embed `GroupState`, use `delegate_group_state!(group, override { set_bounds, draw, handle })`

If your widget owns `Box<dyn View>` children, it MUST use GroupState.

### Render Lifecycle (strict rules)
1. `render()` is called by the parent (or event loop)
2. It renders children first, then calls `draw()`, then blits children on top
3. **NEVER** call `draw()` directly
4. **NEVER** call `blit_child()` directly — it's `#[doc(hidden)]`
5. **NEVER** call `set_child_bounds()` inside `draw()` — use `set_bounds()` or `handle()`
6. A debug_assert fires if `set_child_bounds` is called during render

### Three-Phase Event Dispatch
```
Event → Group.dispatch()
  Phase 1: Preprocess children (options().preprocess() == true)
  Phase 2: Focused child (or modal child)
  Phase 3: Postprocess children (options().postprocess() == true)
```
If any phase returns `HandleResult::Consumed`, dispatch stops.

### Commands (decoupled communication)
Views never call each other. Emit commands via:
```rust
self.state.put_command(CM_MY_ACTION, Some(Box::new(data)));
```
Commands are drained from the EventSink after each event and re-delivered to root.

## Patterns

### Creating a Leaf View
```rust
struct MyView { state: ViewState }
impl View for MyView {
    delegate_view_state!(state, override { draw, handle });
    fn draw(&mut self) {
        self.state.buffer_mut().print(0, 0, "hello", Style::default());
    }
    fn handle(&mut self, event: &Event) -> HandleResult {
        HandleResult::Ignored
    }
}
```

### Creating a Container
```rust
struct MyPanel { group: GroupState }
impl View for MyPanel {
    delegate_group_state!(group, override { set_bounds, draw, handle });
    fn set_bounds(&mut self, r: Rect) {
        self.group.set_bounds(r);
        self.group.set_child_bounds(0, Rect::new(0, 0, r.w(), 1));
        self.group.set_child_bounds(1, Rect::new(0, 1, r.w(), r.h() - 1));
    }
    fn draw(&mut self) {
        self.group.buffer_mut().fill(' ', Style::default());
    }
    fn handle(&mut self, event: &Event) -> HandleResult {
        if matches!(event, Event::Tick) {
            for i in 0..self.group.child_count() {
                if let Some(c) = self.group.child_mut(i) { c.handle(event); }
            }
            return HandleResult::Ignored;
        }
        self.group.dispatch(event)
    }
}
```

### Placing Inline Images
```rust
fn draw(&mut self) {
    let buf = self.state.buffer_mut();
    let transparent = Style::new(Color::Transparent, Color::Transparent);
    buf.fill(' ', transparent);
    buf.place_image(Rect::new(0, 0, buf.width(), buf.height()), data.clone(), ImageTransform::Fit);
}
```

### Testing with MockBackend
```rust
#[test]
fn my_test() {
    let mut view = MyView::new();
    let mut backend = MockBackend::new(80, 24);
    view.set_bounds(Rect::new(0, 0, 80, 24));
    run_cycles(&mut view, &mut backend, 1);
    assert!(backend.contains("expected text"));
}
```

## Anti-patterns
- ❌ Manual `children: Vec<Box<dyn View>>` — use GroupState
- ❌ Manual focus tracking — use `group.switch_focus()`
- ❌ Manual event routing — use dispatch with preprocess/postprocess
- ❌ Calling `draw()` or `blit_child()` outside render
- ❌ Layout (set_child_bounds) inside draw
- ❌ Accessing fields on method return values — use accessor methods

## Code Standards
- Max 240 code lines per file (split by responsibility)
- Max nesting depth 3 (extract helpers)
- Max 7 parameters per function
- No `?` in middle of method chains (break into separate let)
- Use `use` imports, not fully qualified paths
- Zero clippy warnings
