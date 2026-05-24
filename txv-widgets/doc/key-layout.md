# TiledWorkspace Key Layout

## Panel Navigation

| Key | Command | Action |
|-----|---------|--------|
| `Ctrl+Shift+Left` | `CM_TW_FOCUS_LEFT` | Focus panel to the left |
| `Ctrl+Shift+Right` | `CM_TW_FOCUS_RIGHT` | Focus panel to the right |
| `Ctrl+Shift+Up` | `CM_TW_FOCUS_UP` | Focus panel above |
| `Ctrl+Shift+Down` | `CM_TW_FOCUS_DOWN` | Focus panel below |

## Panel Visibility & Zoom

| Key | Command | Action |
|-----|---------|--------|
| `M-,` | `CM_TW_TOGGLE_TREE` | Toggle tree (left) panel |
| `M-.` | `CM_TW_TOGGLE_TOOLS` | Toggle tools (right/bottom) panel |
| `M-/` | `CM_TW_ZOOM` | Toggle zoom on focused panel |
| `M-\` | `CM_TW_LAYOUT_CYCLE` | Cycle layout mode (wide/narrow/auto) |

## Panel Resize

| Key | Command | Action |
|-----|---------|--------|
| `Alt+Shift+Right` | `CM_TW_GROW_H` | Grow focused panel horizontally |
| `Alt+Shift+Left` | `CM_TW_SHRINK_H` | Shrink focused panel horizontally |
| `Alt+Shift+Down` | `CM_TW_GROW_V` | Grow focused panel vertically |
| `Alt+Shift+Up` | `CM_TW_SHRINK_V` | Shrink focused panel vertically |

Resize model: Right/Down = grow (push boundary outward from origin 0,0).
Left/Up = shrink (reverse). Left is always the undo of Right.

macOS Option equivalents: `≠` (grow H), `–` (shrink H), `±` (grow V), `—` (shrink V).

## Tabs

| Key | Command | Action |
|-----|---------|--------|
| `M-1..M-9` | `CM_TW_ACTIVATE_TAB` | Activate tab by index (1-based) |
| `M-0` | `CM_TW_TAB_DROPDOWN` | Open tab dropdown with filter |
| `Alt+Up` | `CM_TW_TAB_DROPDOWN_UP` | Dropdown selection up |
| `Alt+Down` | `CM_TW_TAB_DROPDOWN_DOWN` | Dropdown selection down |
| `M-;` | `CM_TW_TAB_NEXT` | Next tab |
| `M-'` | `CM_TW_TAB_PREV` | Previous tab |
| `M-w` | `CM_TW_TAB_CLOSE` | Close active tab |

## Subpanel (SplitPanel within a panel)

| Key | Command | Action |
|-----|---------|--------|
| `Ctrl+W` | `CM_TW_CYCLE_SUBPANEL` | Cycle focus between subpanels |
| `Ctrl+Alt+W` | `CM_TW_MOVE_TAB_SUBPANEL` | Move tab to other subpanel |
| `Alt+=` | `CM_TW_GROW_SUBPANEL` | Grow focused subpanel |
| `Alt+-` | `CM_TW_SHRINK_SUBPANEL` | Shrink focused subpanel |

## Design Principles

- **Alt+punctuation** = workspace-level actions (toggle, zoom, tabs, layout)
- **Ctrl+Shift+Arrow** = panel focus navigation
- **Alt+Shift+Arrow** = panel resize
- **Ctrl+W** = subpanel (vim-like window commands)
- **Alt+digit** = direct tab access
- Resize Left/Right are inverses (undo each other)
- `Ctrl+Tab` deliberately avoided (terminals capture it)
- `handle_keys=false` mode: keys go through StatusBar → commands
- `handle_keys=true` mode: widget handles keys directly (standalone)
