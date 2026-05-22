# TiledWorkspace Widget — Implementation Status

## Completed

### Phase 1: TabGroup Enhancements ✓
- Horizontal tab bar chrome with glyph separators (│Tab1│Tab2│)
- Active tab highlighting, dirty indicator (•), overflow badge (…N)
- Searchable tab dropdown with fuzzy filter
- `set_dirty()` / `is_tab_dirty()` API

### Phase 2: WorkspaceKeymap ✓
- Fully configurable `WorkspaceKeymap` struct
- Default bindings for all actions
- `matches()` helper for key comparison

### Phase 3: TiledWorkspace Core ✓
- `PanelConfig` (closeable, hideable, splittable, position)
- `SplitNode` split trees for wide/narrow layouts
- Proportional layout engine with hidden panel redistribution
- Panel visibility toggle, zoom, spatial focus navigation
- Proportional resize via split tree adjustment
- `LayoutMode` (Auto/Wide/Narrow) with `M-;` cycle
- `WorkspaceState` save/restore for persistence
- Command-based API (CM_* events) for scripting/MCP
- Dual-mode key handling (internal or app-owned)
- `default_bindings()` export for status bar registration
- `CM_CORE_MAX` / `CM_TXV_MAX` range separation

### Phase 4: ToolsPanel ✓
- Split-on-move (`C-M-w` creates split if single subpanel)
- Focus cycling (`C-w`) between subpanels
- Proportional resize (`M-=`/`M--`) between subpanels
- Auto-unsplit when subpanel becomes empty
- Configurable split direction (horizontal/vertical)
- Wired into TiledWorkspace via `with_tools_panel()` downcast

## Test Coverage

- 10 TiledWorkspace tests (layout, toggle, zoom, commands, keys, state)
- 6 ToolsPanel tests (split, unsplit, cycle, resize, edge cases)
- 5 cursor integration tests
- 1 palette integration test (consolidated to avoid race)

## Future Work

- PtyTerminal cursor exposure
- Status bar integration helpers
- Mouse support for tab bar / panel resize
- Tab drag-and-drop between panels
