# TiledWorkspace Widget — Design Document

## Overview

`TiledWorkspace` is a reusable IDE-style layout widget for `txv-widgets`. It
manages three panels (tree, main, tools) with automatic layout adaptation,
panel visibility toggling, zoom, and configurable keybindings.

The application defines the panel configuration at construction:
- Which panels exist and their roles
- Whether each panel's tabs are **closeable** (user can close tabs, e.g. editor
  buffers) or **fixed** (tabs are managed by the app, e.g. file tree, git tree)
- What types of views each panel accepts

## Panels

| Panel | Role | Position | Visibility |
|-------|------|----------|------------|
| **Tree** | File tree, project navigation | Left, full height | Toggleable (`M-,`) |
| **Main** | Editor, primary content | Center, fills remaining space | Always visible |
| **Tools** | Terminals, output, search results | Right (wide) or Bottom (narrow) | Toggleable (`M-.`) |

### Tools Panel Internal Structure

The tools panel can **split** into multiple subpanels, each with its own
TabGroup. This mirrors vim's window splitting within a single panel.

- When tools is on the **right**: splits are **horizontal** (stacked vertically)
- When tools is on the **bottom**: splits are **vertical** (side by side)
- Each subpanel has its own TabGroup with independent tab navigation
- Subpanel count: 1 (default) to N (user-initiated splits)

## Layout Modes

Layout adapts automatically based on terminal width:

```
Wide (≥ threshold_wide):          Narrow (< threshold_narrow):
┌──────┬──────────┬───────┐      ┌──────┬──────────────────┐
│ Tree │   Main   │ Tools │      │ Tree │      Main        │
│      │          │       │      │      ├──────────────────┤
│      │          │       │      │      │      Tools       │
└──────┴──────────┴───────┘      └──────┴──────────────────┘
```

Hysteresis prevents flapping at boundary widths.

## Keyboard Bindings (Defaults)

All bindings are configurable. Defaults:

### Panel Level (Desktop scope)

| Action | Default Key | Description |
|--------|-------------|-------------|
| Toggle tree | `M-,` | Show/hide tree panel |
| Toggle tools | `M-.` | Show/hide tools panel |
| Zoom focused | `M-/` | Toggle zoom on focused panel |
| Panel focus | `C-S-Arrow` | Move focus between visible panels |
| Panel resize | `M-S-Arrow` | Grow/shrink panel in arrow direction |

### Tab Level (within focused TabGroup)

| Action | Default Key | Description |
|--------|-------------|-------------|
| Tab by index | `M-1`..`M-9` | Switch to tab N in focused TabGroup |
| Tab dropdown | `M-0` | Open searchable tab list |

### Subpanel Level (within Tools panel)

| Action | Default Key | Description |
|--------|-------------|-------------|
| Subpanel focus | `C-w` | Cycle focus between subpanels |
| Move tab | `C-M-w` | Move active tab to next subpanel |
| Grow subpanel | `M-=` | Grow focused subpanel |
| Shrink subpanel | `M--` | Shrink focused subpanel |

## Architecture

```
TiledWorkspace (View)
├── Tree: TabGroup (fixed — app manages tabs)
├── Main: TabGroup (closeable — user manages tabs)
└── Tools: ToolsPanel
    ├── Subpanel 0: TabGroup (closeable or fixed, per app config)
    ├── Subpanel 1: TabGroup (if split)
    └── ...
```

### Panel Configuration

Each panel is defined by a `PanelConfig`:

```rust
pub struct PanelConfig {
    /// Panel role identifier (for keybinding dispatch).
    pub role: PanelRole,
    /// Whether users can close tabs (true) or tabs are app-managed (false).
    pub closeable: bool,
    /// Preferred position (Left, Center, Right/Bottom).
    pub position: PanelPosition,
    /// Whether the panel can be hidden by the user.
    pub hideable: bool,
    /// Whether the panel supports internal splitting.
    pub splittable: bool,
}
```

The application constructs `TiledWorkspace` with a list of panel configs.
The widget handles layout, focus, and keybindings generically — it doesn't
know about "files" or "editors", only about panels with tabs.

TiledWorkspace uses `GroupState` with N children (one per panel). The tools
panel is itself a container that manages its subpanels.

### Key Dispatch

1. Desktop intercepts panel-level keys (`M-,`, `M-.`, `M-/`, `C-S-Arrow`, `M-S-Arrow`)
   in **preprocess** phase.
2. Tab-level keys (`M-0`..`M-9`) are handled by the focused TabGroup.
3. Subpanel keys (`C-w`, `C-M-w`, `M--`, `M-=`) are handled by the Tools
   panel when it has focus.

### Configurable Keybindings

Bindings are stored in a `DesktopKeymap` struct passed at construction.
A `Default` impl provides the bindings above. Applications can override
any binding.

## Tab Bar / Tab Chrome

Each TabGroup renders a horizontal tab bar as its chrome row:

- All tabs shown as `│Tab1│Tab2│Tab3│` with separators
- Active tab highlighted with focused style
- Overflow indicator when tabs don't fit
- Dirty indicator (•) per tab
- Badge support (activity indicators for terminals)

### Searchable Tab Dropdown (`M-0`)

- Opens as overlay within the TabGroup
- Shows all tabs with index numbers
- Typing filters the list (fuzzy match on title)
- Enter selects, Esc cancels
- Arrow keys navigate filtered results

## Zoom Behavior

- `M-/` toggles zoom on the focused panel
- Zoomed panel takes the entire Desktop bounds
- Other panels are hidden but retain state
- Focus cycling (`C-S-Arrow`) while zoomed switches which panel is zoomed
- Pressing `M-/` again restores normal layout

## Panel Visibility

- Hidden panels retain their TabGroups and state
- Toggling a panel back restores previous size
- When tools hides, focus moves to main
- When tree hides, focus stays or moves to main

## Resize

- `M-S-Arrow` adjusts the border between panels
- Minimum panel width/height enforced (configurable)
- Resize is persisted in panel size fields (not recomputed from ratio)
