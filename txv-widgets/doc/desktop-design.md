# TiledWorkspace Widget — Design Document (v2)

## Architecture Redesign

### Widget Hierarchy

```
TabBar          — horizontal tab strip with powerline separators
                  Handles M-0 dropdown, M-digit switching
                  Fill style: configurable (transparent for kairn)

TabPanel        — TabBar + N stacked child Views (shows one at a time)
                  Self-contained tabbed container, works standalone

SplitPanel      — 1..N children in proportional split
                  Generic (children can be any View)
                  Direction switchable at runtime (H↔V)
                  Focus cycling, resize

TiledWorkspace  — N panels (TabPanel or SplitPanel) with layout engine
                  Configurable keybindings, command-based API
```

### Color::Transparent

`txv-core` adds `Color::Transparent` sentinel. `Buffer::blit()` skips cells
where both fg and bg are Transparent. This lets TabBar "float" over the
parent's chrome line — tabs are painted on top, gaps show the parent's
horizontal rule.

## TabBar Visual Design

### Powerline Separators

```
active ₁tab2₂tab3₃tab4 ▾…5──────
```

- Active→inactive: `` (U+E0B0) with fg=active_bg, bg=next_bg
- Inactive→inactive: if bg differs → ``, if same → `` (U+E0B1, fg=dim)
- Last tab→fill: `` with fg=last_bg, bg=fill_bg

### Tab Palette (lookup table, not computed)

```rust
pub struct TabBarPalette {
    pub active: PaletteStyle,       // focused tab
    pub active_unfocused: PaletteStyle, // active tab when panel not focused
    pub tabs: [PaletteStyle; 10],   // positional: tabs[0]=nearest, tabs[9]=furthest
}
```

Defaults: gray gradient (truecolor) or flat (low-color).
User can define rainbow, alternating, anything.

Powerline glyph logic: compare `tabs[n].bg` vs `tabs[n+1].bg` →
solid arrow if different, thin separator if same.

### Number Labels

- Subscript (₁₂₃₄₅₆₇₈₉) when unicode tier supports it
- `1:` `2:` fallback when not
- Active/top tab: no number label (it's always position 0)

### Foreground

- Active: bright/white
- Inactive: cyan or light grey, fading with position
- Numbers: dim or same as tab fg

### Overflow / Scrolling

**LRU tabs (editor):** Active is always leftmost (position 0). Overflow
on right only → `▾…N` badge (count + dropdown arrow).

**Fixed tabs:** If active doesn't fit:
- Scroll so active is visible
- Left indicator: `…N` (N hidden left)
- Right indicator: `▾…N` (N hidden right + dropdown)

```
…2 ₃Active ₄Next ₅Next2 ▾…3
```

**Dropdown always shows all tabs** regardless of scroll position.

### Fill

TabBar fill character and style are configurable:
- `transparent` (kairn): parent's chrome line shows through
- `─` + dim: standalone use, draws its own separator
- space + bg: flat background fill

## TabPanel

TabBar + child stack. Owns:
- `TabBar` (drawn at row 0)
- `Vec<Box<dyn View>>` children (one visible at a time, below TabBar)
- Dirty state per tab
- LRU ordering (optional, configurable)

Handles:
- Tab switching (from TabBar commands)
- Child bounds management (content_rect = bounds minus 1 row)
- Tick dispatch to all children (background refresh)

## SplitPanel

Generic split container:
- `Vec<Box<dyn View>>` children
- `Vec<f32>` proportions
- `SplitDir` (Horizontal or Vertical), switchable at runtime
- Focused child index
- `cycle_focus()`, `grow_focused()`, `shrink_focused()`

No tab awareness — that's the caller's concern. When used with TabPanels,
the app provides `move_tab_to_next()` as a helper or via commands.

## TiledWorkspace

Same as v1 but uses TabPanel/SplitPanel instead of TabGroup/ToolsPanel.

## Command-Based API

Same as v1. All keyboard actions have corresponding CM_* commands.
External integrations emit commands, never call methods directly.

### Command ID Ranges

- `1..99` (CM_CORE_MAX) — txv-core
- `100..149` (CM_WORKSPACE_BASE..MAX) — workspace commands
- `150..999` — reserved for other txv-widgets
- `1000+` (CM_TXV_MAX+1) — application-specific

## Dual-Mode Key Handling

- `handle_keys = true`: widget handles keys internally (standalone)
- `handle_keys = false`: only responds to commands (app/status bar owns keys)
- `default_bindings()` exports keymap for status bar registration
