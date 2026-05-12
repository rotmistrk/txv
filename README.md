# TXV

A Turbo Vision-inspired TUI framework in Rust. Retained-mode view tree with
three-phase event dispatch, dirty tracking, and diff-flush rendering.

## Crates

| Crate | Purpose |
|-------|---------|
| `txv-core` | Pure logic. View trait, GroupState, EventQueue, Surface, geometry. Zero I/O. |
| `txv-render` | Terminal backend (crossterm). TermBuf, VTE parsing, diff flush. |
| `txv-widgets` | Concrete views: TabGroup, PtyTerminal, TextArea, TreeView, StatusBar, etc. |

## Quick Start

```toml
[dependencies]
txv-core = { git = "https://github.com/rotmistrk/txv.git" }
txv-render = { git = "https://github.com/rotmistrk/txv.git" }
txv-widgets = { git = "https://github.com/rotmistrk/txv.git" }
```

```rust
use txv_core::prelude::*;
use txv_render::CrosstermBackend;

struct MyView {
    state: ViewState,
}

impl View for MyView {
    delegate_view_state!(state);

    fn draw(&self, surface: &mut Surface) {
        let b = self.bounds();
        surface.write_str(b.x, b.y, "Hello, TXV!", Style::default());
    }

    fn handle(&mut self, event: &Event, queue: &mut EventQueue) -> HandleResult {
        if let Event::Key(k) = event {
            if k.code == KeyCode::Char('q') {
                queue.put_command(CM_QUIT, None);
                return HandleResult::Consumed;
            }
        }
        HandleResult::Ignored
    }
}
```

## Design Principles

- **Composition over inheritance** — ViewState/GroupState are embedded, not inherited
- **Delegation macros** eliminate boilerplate while keeping views focused
- **Three-phase dispatch** — preprocess → focused → postprocess
- **Command-based communication** — views emit commands via EventQueue, never call each other
- **Dirty tracking** — only changed regions are flushed to the terminal

## Widgets

TabGroup, SplitPane, PtyTerminal, TextArea, TreeView, ListView, Table,
StatusBar, InputLine, InputDialog, FuzzySelect, Menu, Overlay, ScrollView,
ProgressBar, FileTree, InlineEditor.

## Documentation

- [User Guide](docs/user-guide.md) — View trait, widgets, event model, layout
- [Developer Guide](docs/developer-guide.md) — Architecture, testing, contributing

## License

MIT
