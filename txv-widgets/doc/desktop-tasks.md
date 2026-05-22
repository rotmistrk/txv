# TiledWorkspace Widget — Implementation Tasks

## Phase 1: TabGroup Enhancements

### 1.1 Horizontal Tab Bar Chrome
- Render all tabs as `│Tab1│Tab2│Tab3│` with glyph separators
- Active tab uses focused style, others use dim
- Overflow indicator (badge + count) when tabs exceed width
- Dirty indicator (•) support per tab

### 1.2 Searchable Tab Dropdown
- Add filter string to dropdown state
- Accept character input to narrow tab list (fuzzy match)
- Show matching tabs with highlighted match portions
- Backspace removes filter chars, Esc closes
- Enter selects highlighted tab

### 1.3 Tab Dirty State
- Add `dirty: Vec<bool>` to TabGroup
- API: `set_dirty(index, bool)`, `is_dirty(index) -> bool`
- Chrome renders • indicator for dirty tabs

## Phase 2: DesktopKeymap

### 2.1 Keymap Struct
- Define `DesktopKeymap` with fields for each action
- Each field is a `KeyEvent` (code + modifiers)
- `Default` impl provides the agreed bindings
- Matching helper: `fn matches(&self, event: &KeyEvent, action: Action) -> bool`

## Phase 3: Desktop Widget Core

### 3.1 TiledWorkspace Struct
- N children based on PanelConfig list provided at construction
- Uses GroupState with N children
- Stores: panel sizes, visibility flags, zoom state, layout thresholds
- Keymap field
- PanelConfig per panel (closeable, hideable, splittable, position)

### 3.2 Layout Engine
- `compute_layout()` based on bounds width and visibility flags
- Wide mode: tree | main | tools (left to right)
- Narrow mode: tree | main (top) / tools (bottom)
- Hysteresis for auto-switching
- Zoom: focused panel gets full bounds

### 3.3 Panel Visibility Toggle
- `M-,` toggles tree visibility (hideable panels only)
- `M-.` toggles tools visibility
- Focus management on hide/show
- Size restoration on show

### 3.4 Panel Focus Navigation
- `C-S-Arrow` moves focus between visible panels
- Spatial: respects actual panel positions in current layout
- Wraps or stops at edges (configurable)

### 3.5 Panel Resize
- `M-S-Arrow` adjusts panel borders
- Minimum size enforcement
- Direction-aware: left/right adjusts vertical borders, up/down adjusts horizontal

### 3.6 Zoom
- `M-/` toggles zoom on focused panel
- Zoomed panel gets full TiledWorkspace bounds
- `C-S-Arrow` while zoomed cycles which panel is zoomed

## Phase 4: Tools Panel (ToolsPanel widget)

### 4.1 ToolsPanel Struct
- Contains 1..N TabGroups in a split arrangement
- Split direction follows position (right→vertical splits, bottom→horizontal)
- Manages focused subpanel index

### 4.2 Subpanel Focus
- `C-w` cycles focus between subpanels
- Visual indicator of which subpanel is focused

### 4.3 Tab Move Between Subpanels
- `C-M-w` moves active tab from focused subpanel to next
- Tab is removed from source TabGroup, inserted in target

### 4.4 Subpanel Resize
- `M-=` grows focused subpanel
- `M--` shrinks focused subpanel
- Respects minimum sizes

### 4.5 Split/Unsplit
- Command to split tools panel (add subpanel)
- Auto-unsplit when subpanel becomes empty

## Phase 5: Integration & Tests

### 5.1 Integration Tests
- Layout computation tests (wide/narrow/zoom)
- Panel toggle + focus management
- Tab dropdown with filter
- Subpanel focus cycling
- Tab move between subpanels
- Resize bounds checking

### 5.2 Documentation
- Update user guide with Desktop widget usage
- Document keymap customization

## Dependencies

- Phase 1 has no dependencies (enhances existing TabGroup)
- Phase 2 has no dependencies (standalone struct)
- Phase 3 depends on Phase 1 + 2
- Phase 4 depends on Phase 2
- Phase 5 depends on Phase 3 + 4
