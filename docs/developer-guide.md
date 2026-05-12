# TXV Developer Guide

## Architecture

```
txv-core        Pure logic. Zero I/O. Defines View trait, Group dispatch, geometry.
                No terminal dependency — testable without a TTY.

txv-render      Terminal I/O (crossterm). Implements Backend trait.
                TermBuf: VTE terminal emulator for PTY output.
                Diff flush: only changed cells written to terminal.

txv-widgets     Concrete Views built on txv-core.
                Each widget is a self-contained View implementation.
```

### Dependency Graph

```
txv-widgets → txv-core
txv-widgets → txv-render (for TermBuf/PtyTerminal)
txv-render  → txv-core (for Backend trait, Surface, Event)
```

## Key Design Decisions

### Retained Mode (not Immediate Mode)

Unlike ratatui (redraw everything every frame), TXV views own their state and
only redraw when dirty. The framework tracks `needs_redraw()` and the backend
performs diff-flush (only changed cells go to the terminal).

### Composition + Delegation Macros

Views compose ViewState/GroupState as fields, then delegate via macros:
- `delegate_view_state!(field)` — View trait boilerplate
- `delegate_group_state!(field)` — Group dispatch + View delegation
- `delegate_window_state!(field)` — Window (border + title)
- `delegate_dialog_state!(field)` — Dialog (modal + buttons)

### Three-Phase Event Dispatch

GroupState dispatches events in order:
1. Preprocess children (e.g., StatusBar intercepts keys)
2. Focused/modal child
3. Postprocess children

This eliminates complex event routing logic in parent views.

### Command-Based Communication

Views never call each other. They emit commands via `EventQueue::put_command()`.
Commands bubble up to the Program handler. This fully decouples all views.

## File Organization

```
txv-core/src/
  view.rs           View trait, ViewState, EventQueue, HandleResult
  geometry.rs       Rect, Point
  surface.rs        2D cell buffer for drawing
  cell.rs           Cell, Style, Color
  event.rs          Event enum, KeyEvent, MouseEvent
  group/
    mod.rs          GroupState struct + accessor API
    dispatch.rs     Three-phase dispatch + delegate_group_state! macro
  program.rs        Program event loop
  window.rs         WindowState (border + title)
  dialog.rs         DialogState (modal + buttons)
  commands.rs       Standard command IDs (CM_QUIT, CM_CLOSE, etc.)
  run/              MockBackend for testing

txv-render/src/
  backend.rs        CrosstermBackend (enter/leave alt screen, poll, flush)
  termbuf/          VTE terminal emulator (for PtyTerminal)
  event_translate.rs  crossterm → txv event translation
  color.rs          Color mapping

txv-widgets/src/
  tab_group.rs      Tabbed container
  pty_terminal.rs   Full terminal emulator widget
  text_area.rs      Multi-line text display
  tree_view.rs      Hierarchical tree
  list_view.rs      Scrollable list
  split_pane.rs     Two-panel split
  status_bar.rs     Bottom status bar
  input_line.rs     Single-line input
  fuzzy_select.rs   Fuzzy picker
  menu.rs           Popup menu
  table.rs          Column table
  ...
```

## Testing

### MockBackend

txv-core provides `MockBackend` for headless testing:

```rust
use txv_core::run::mock::MockBackend;

#[test]
fn test_view_renders() {
    let mut backend = MockBackend::new(80, 24);
    // inject events, run program, check surface content
    assert!(backend.content_contains("expected text"));
}
```

### Testing Principles

- **Deterministic** — no timing, no shared state, no flakiness
- **Independent** — each test runs in isolation, safe for parallel execution
- **Use `content_contains()`** — avoids false positives from status bar clock
- **One file per feature** — test files cover a single scenario/concern

### Running Tests

```bash
cargo test --workspace
cargo test --workspace --no-fail-fast  # don't stop on first failure
```

## Code Standards

### Formatting

- `rustfmt.toml`: `max_width = 120`, `single_line_if_else_max_width = 0`
- Run: `cargo fmt --all`

### Linting

- Zero clippy warnings: `cargo clippy --workspace -- -D warnings`
- No `unwrap()`, `expect()`, `panic!()` in runtime code (tests are OK)

### File Size

- **240 code lines maximum per file** (blank/comment lines don't count)
- When exceeded: split conceptually into files with clear single responsibilities
- Never reduce code quality to fit the limit

### Encapsulation

- ViewState fields are private — use accessor methods
- GroupState.children/focused are `pub(crate)` — external code uses the API:
  `child()`, `child_mut()`, `focused_index()`, `switch_focus()`, etc.

## Contributing

### Adding a Widget

1. Create `txv-widgets/src/my_widget.rs`
2. Embed `ViewState` (or `GroupState` if it owns children)
3. Implement `View` trait, using `delegate_view_state!` for boilerplate
4. Re-export from `txv-widgets/src/lib.rs`
5. Add tests
6. Ensure `cargo clippy --workspace -- -D warnings` passes

### Modifying Core

Changes to txv-core affect all consumers. Ensure:
- All workspace tests pass
- Delegation macros still expand correctly
- No breaking API changes without version bump

### Commit Style

- Imperative mood: "Add widget" not "Added widget"
- Body explains WHY, not just what
- Keep commits focused — one logical change per commit
