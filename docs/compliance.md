# TXV Compliance Checklist

Rules every View implementation must follow. Use this as a review checklist
when writing or auditing txv-based code.

## Render Pipeline

| Rule | Rationale |
|------|-----------|
| `draw()` is called ONLY from `render()` | The macro-generated `render()` orchestrates child renders → self draw → blit. Manual `draw()` calls break ordering. |
| `blit` is called ONLY from `render()` | `blit_child()` / `blit_all_children()` are `#[doc(hidden)]` render-phase internals. |
| `render()` is called ONLY from a parent's `render()` or the event loop | The call chain is: event loop → root.render() → child.render() recursively. Never call `render()` from `handle()` or `draw()`. |
| `draw()` draws ONLY self (own buffer) | A view's `draw()` fills its own buffer at local coordinates (0,0). It must NOT write to child buffers, call child methods that mutate display state, or blit children. |
| Views draw at (0,0) in their own buffer | Absolute positioning is the framework's job via `set_bounds()`. |

## Size and Layout

| Rule | Rationale |
|------|-----------|
| Size changes happen ONLY in `set_bounds()` or `handle()` | `set_bounds()` propagates from parent. A view may call `set_child_bounds()` from its own `set_bounds()` override or from `handle()` (e.g., layout change on key). |
| NEVER call `set_child_bounds()` from `draw()` | Draw is read-only rendering. Layout mutations in draw cause infinite loops or stale state. |

## Event Handling

| Rule | Rationale |
|------|-----------|
| `handle()` returns `HandleResult` | `Consumed` = event was handled, `Ignored` = pass to next handler. |
| Groups use `self.group.dispatch(event)` for routing | Three-phase dispatch (preprocess → focused → postprocess). Don't reimplement. |
| Tick events broadcast to ALL children | Tick is not user input. Override `handle()` to iterate all children for Tick, then use `dispatch()` for other events. |
| Views never call each other directly | Communicate via commands: `state.put_command(id, data)`. Commands bubble up to the Program-level handler. |

## Encapsulation

| Rule | Rationale |
|------|-----------|
| `ViewState`/`GroupState` fields are private or `pub(crate)` | External access through accessors only. |
| Use `delegate_view_state!` or `delegate_group_state!` | Macro handles render pipeline, dirty tracking, and trait forwarding. |
| Leaf views embed `ViewState`, containers embed `GroupState` | No third option. If you own `Box<dyn View>`, you must use `GroupState`. |

## Code Reuse

| Rule | Rationale |
|------|-----------|
| Use txv-widgets primitives before building custom | `InputLine`, `TreeView`, `TabPanel`, `SplitPanel`, `TextArea`, `PtyTerminal`, `TiledWorkspace` cover most needs. |
| Compose, don't inherit | Add behavior with pre/post-process children or wrapper Groups, not subclassing. |
| Shared behavior → extract to txv-widgets | If two apps need it, it belongs in the widget crate. |

## State Management

| Rule | Rationale |
|------|-----------|
| Dirty tracking is automatic | Writing to `buffer_mut()` or calling `mark_dirty()` triggers redraw. Don't mark dirty unnecessarily (especially on Tick). |
| Focus propagation via `select()` / `unselect()` | GroupState calls these automatically via `switch_focus()`. Don't propagate focus in `draw()`. |
| `as_any_mut()` for cross-view access | Parent views downcast children via `as_any_mut()` when needed (e.g., status bar reading editor state). |

## Quick Self-Check

Before submitting a View implementation, verify:

1. ☐ `draw()` only writes to `self.state.buffer_mut()` (or `self.group.buffer_mut()`)
2. ☐ No `render()` calls anywhere in your code
3. ☐ No `blit` calls anywhere in your code
4. ☐ `set_child_bounds()` only in `set_bounds()` or `handle()`
5. ☐ Tick broadcasts to all children (if Group)
6. ☐ Focus set in `select()`/`unselect()`, not in `draw()`
7. ☐ Inter-view communication via commands only
8. ☐ No manual children Vec (use GroupState)
9. ☐ `delegate_*!` macro used
10. ☐ `mark_dirty()` not called on every tick
