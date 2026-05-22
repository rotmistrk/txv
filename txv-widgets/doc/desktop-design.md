# TiledWorkspace Widget — Design Document

## Overview

`TiledWorkspace` is a reusable IDE-style layout widget in `txv-widgets`. It
manages N panels with automatic layout adaptation, panel visibility toggling,
zoom, and fully configurable keybindings.

The application defines panel configuration at construction. The widget handles
layout, focus, and keybindings generically — it doesn't know about "files" or
"editors", only about panels with tabs.

## Command-Based API

External integrations (scripting, MCP, TCL, ex-commands) interact with
TiledWorkspace by emitting command events into the EventQueue — NOT by
calling methods directly. This keeps the architecture decoupled and
consistent with TXV's event-driven design.

See `tiled_workspace::commands` for available command IDs and payload types.

### Command ID Ranges

- `1..99` (CM_CORE_MAX) — txv-core (quit, close, focus, etc.)
- `100..149` (CM_WORKSPACE_BASE..MAX) — workspace commands
- `150..999` — reserved for other txv-widgets
- `1000+` (CM_TXV_MAX+1) — application-specific

## Panels

Each panel is defined by a `PanelConfig`:

```rust
PanelConfig {
    name: String,        // for debugging/persistence
    closeable: bool,     // user can close tabs (editor) vs app-managed (tree)
    hideable: bool,      // user can toggle visibility
    splittable: bool,    // supports internal subpanel splits (ToolsPanel)
    position: PanelPosition,  // Left, Center, Right, Bottom
}
```

Splittable panels use `ToolsPanel` (inserted as a View). Non-splittable
panels use `TabGroup` directly.

## Layout

### Split Trees

The developer provides two layout trees (wide and narrow):

```rust
let wide = SplitNode::h(vec![
    (0.2, SplitNode::leaf(0)),   // Tree
    (0.5, SplitNode::leaf(1)),   // Main
    (0.3, SplitNode::leaf(2)),   // Tools
]);
let narrow = SplitNode::h(vec![
    (0.25, SplitNode::leaf(0)),  // Tree
    (0.75, SplitNode::v(vec![
        (0.6, SplitNode::leaf(1)),   // Main
        (0.4, SplitNode::leaf(2)),   // Tools
    ])),
]);
```

### Layout Modes

- **Auto** — switches based on `wide_threshold`
- **Wide** — forced wide layout regardless of terminal width
- **Narrow** — forced narrow layout

Cycle with `M-;` or `CM_LAYOUT_CYCLE`.

### Proportional Resize

- Sizes are proportions (f32) within parent split
- Window resize scales all panels proportionally
- `M-S-Arrow` adjusts boundary between adjacent panels
- Hidden panels are excluded; remaining panels redistribute space

### State Persistence

```rust
let state = workspace.save_state();   // WorkspaceState
workspace.restore_state(&state);      // restore proportions + hidden
```

Developer handles serialization/I/O.

## Keyboard Bindings

### Dual-Mode Key Handling

- `handle_keys = true` (default): widget handles keys internally
- `handle_keys = false`: widget only responds to command events;
  app/status bar owns key dispatch

```rust
workspace.set_handle_keys(false);
let bindings = workspace.default_bindings(); // Vec<KeyBinding>
// Register with status bar
```

### Default Bindings

| Action | Key | Command |
|--------|-----|---------|
| Toggle tree | `M-,` | CM_TOGGLE_PANEL |
| Toggle tools | `M-.` | CM_TOGGLE_PANEL |
| Zoom focused | `M-/` | CM_ZOOM |
| Cycle layout | `M-;` | CM_LAYOUT_CYCLE |
| Panel focus | `C-S-Arrow` | CM_FOCUS_DIRECTION |
| Panel resize | `M-S-Arrow` | CM_RESIZE_PANEL |
| Tab by index | `M-1..9` | CM_ACTIVATE_TAB |
| Tab dropdown | `M-0` | CM_TAB_DROPDOWN |
| Subpanel focus | `C-w` | CM_CYCLE_SUBPANEL |
| Move tab subpanel | `C-M-w` | CM_MOVE_TAB_SUBPANEL |
| Grow subpanel | `M-=` | CM_GROW_SUBPANEL |
| Shrink subpanel | `M--` | CM_SHRINK_SUBPANEL |

All bindings are fully configurable via `WorkspaceKeymap`.

## ToolsPanel (Subpanel Splitting)

Panels marked `splittable: true` can use `ToolsPanel` instead of `TabGroup`.

- Contains 1..N TabGroups in a split arrangement
- Split direction follows panel position (right→horizontal, bottom→vertical)
- **Split-on-move**: `C-M-w` creates a split if only one subpanel exists
- **Auto-unsplit**: empty subpanels are removed automatically
- **Proportional**: subpanel sizes are proportions, adjusted with `M-=`/`M--`

## Zoom

- `M-/` toggles zoom on focused panel (full workspace bounds)
- `C-S-Arrow` while zoomed cycles which panel is zoomed
- Other panels retain state but are hidden

## Tab Chrome

Each TabGroup renders a horizontal tab bar:

```
│Shell│Build •│Output│…2
```

- Glyph separators between tabs
- Active tab highlighted
- Dirty indicator (•)
- Overflow badge (…N)
- Searchable dropdown (`M-0`): type to fuzzy-filter, Enter selects

## Architecture

```
TiledWorkspace (View, GroupState)
├── Panel 0: TabGroup (or ToolsPanel if splittable)
├── Panel 1: TabGroup
└── Panel 2: ToolsPanel
    ├── Subpanel 0: TabGroup
    └── Subpanel 1: TabGroup (if split)
```

### Files

```
tiled_workspace/
├── mod.rs          — TiledWorkspace struct, panel access, state
├── types.rs        — PanelConfig, SplitNode, LayoutMode, WorkspaceState
├── keymap.rs       — WorkspaceKeymap with Default
├── commands.rs     — CM_* command IDs
├── layout.rs       — proportional layout engine
├── handle_cmd.rs   — command event dispatcher
├── view_impl.rs    — View trait, key dispatch, spatial focus
└── tests.rs        — 10 integration tests

tools_panel.rs      — ToolsPanel (subpanel splitting)
tools_panel_tests.rs — 6 unit tests
```
