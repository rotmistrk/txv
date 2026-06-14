# TXV

A Turbo Vision-inspired TUI framework in Rust. Retained-mode view tree with
three-phase event dispatch, dirty tracking, and diff-flush rendering.

## Crates

| Crate | Purpose |
|-------|---------|
| `txv-core` | Pure logic. View trait, GroupState, Buffer, Event, geometry. Zero I/O. |
| `txv-render` | Terminal backend (crossterm). TermBuf, VTE parsing, diff flush. |
| `txv-widgets` | Concrete views: TabPanel, PtyTerminal, TextArea, TreeView, StatusBar, etc. |
| `txv-edit` | Vi-style text editor with syntax highlighting, :commands, search. |
| `txv-gallery` | Widget gallery demo + integration test harness (56+ scenario tests). |

## Quick Start

```toml
[dependencies]
txv-core = { git = "https://github.com/rotmistrk/txv.git" }
txv-render = { git = "https://github.com/rotmistrk/txv.git" }
txv-widgets = { git = "https://github.com/rotmistrk/txv.git" }
```

## Gallery Demo

Run the interactive widget gallery:

```sh
cargo run -p txv-gallery
```

Navigate with ↑↓, switch panels with Tab or Ctrl+Shift+Arrows,
resize with Alt+Shift+Arrows. The gallery demonstrates all widgets
with live instances and setup code.

Run the integration tests:

```sh
cargo test -p txv-gallery
```

## Creating a View

```rust
use txv_core::prelude::*;

struct MyView {
    state: ViewState,
}

impl View for MyView {
    delegate_view_state!(state, override { draw, handle });

    fn draw(&mut self) {
        let b = self.state.bounds();
        self.state.buffer_mut().print(0, 0, "Hello, TXV!", Style::default());
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        if let Event::Key(k) = event {
            if k.code() == KeyCode::Char('q') {
                self.state.put_command(CM_QUIT, None);
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
- **Command-based communication** — views emit commands via EventSink, never call each other
- **Dirty tracking** — only changed regions are flushed to the terminal
- **Render pipeline** — `render()` calls `draw()` then blits children; never call `draw()` or `blit_child()` directly
- **Inline images** — views place images in Buffer via `place_image()`; terminal renders using iTerm2/Kitty protocols

## Widgets

TabPanel, SplitPanel, SplitPane, TiledWorkspace, PtyTerminal, TextArea,
TreeView, ListView, Table, StatusBar, InputLine, InputDialog, FuzzySelect,
Menu, ScrollView, ProgressBar, FileTree, DropdownMenu, ModalKey, ImageView.

## Documentation

- [Architecture](docs/ARCHITECTURE.md) — View trait, GroupState, dispatch, patterns
- [User Guide](docs/user-guide.md) — API reference, event model, layout, render lifecycle
- [Developer Guide](docs/developer-guide.md) — Crate structure, testing, contributing

## License

MIT
