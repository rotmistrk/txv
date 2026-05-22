# TiledWorkspace v2 — Redesign Tasks

## Phase 0: Core — Color::Transparent

- [ ] Add `Color::Transparent` variant to `txv-core::cell::Color`
- [ ] Update `Buffer::blit()` to skip cells where fg+bg are both Transparent
- [ ] Update terminal backend to never emit Transparent (treat as no-op)
- [ ] Tests for transparent blit behavior

## Phase 1: TabBar (new widget)

- [ ] Create `TabBar` struct: labels, active index, dirty flags, dropdown state
- [ ] Implement powerline rendering with `TabBarPalette` lookup
- [ ] Number labels: subscript (₁₂₃) or `N:` fallback based on glyph tier
- [ ] Overflow: scroll to active, left `…N` indicator, right `▾…N` badge
- [ ] Searchable dropdown (fuzzy filter, same as current)
- [ ] Configurable fill (transparent / `─` / space)
- [ ] M-0 opens dropdown, M-1..9 activates tab (when handle_keys=true)
- [ ] Emits CM_ACTIVATE_TAB / CM_TAB_DROPDOWN commands
- [ ] Tests: rendering, overflow, scroll, dropdown filter

## Phase 2: TabPanel (replaces TabGroup)

- [ ] Create `TabPanel` struct: TabBar + Vec<Box<dyn View>> children
- [ ] Content rect = bounds minus 1 row (TabBar height)
- [ ] Tab switching: set_active, insert_tab, remove_tab, close_tab
- [ ] Dirty state, LRU ordering (optional)
- [ ] Tick dispatch to all children
- [ ] View impl: draw TabBar + blit active child
- [ ] Works standalone (no TiledWorkspace dependency)
- [ ] Tests: tab management, draw, focus

## Phase 3: SplitPanel (replaces ToolsPanel)

- [ ] Create `SplitPanel` struct: Vec<Box<dyn View>> + proportions + direction
- [ ] `set_direction()` switchable at runtime
- [ ] `cycle_focus()`, `grow_focused()`, `shrink_focused()`
- [ ] View impl: layout children, draw, dispatch to focused
- [ ] Tests: split, resize, direction switch

## Phase 4: TiledWorkspace update

- [ ] Replace TabGroup usage with TabPanel
- [ ] Replace ToolsPanel usage with SplitPanel
- [ ] Update handle_cmd.rs for new types
- [ ] Split-on-move helper (creates SplitPanel from TabPanel when needed)
- [ ] Update all existing tests

## Phase 5: Cleanup

- [ ] Delete old TabGroup (tab_group.rs, tab_group_view.rs, tab_group_dropdown.rs)
- [ ] Delete old TabBar (tab_bar.rs)
- [ ] Delete old ToolsPanel (tools_panel.rs)
- [ ] Update lib.rs exports
- [ ] Update design docs final

## Dependencies

- Phase 0 has no dependencies (txv-core only)
- Phase 1 depends on Phase 0 (transparent fill)
- Phase 2 depends on Phase 1 (uses TabBar)
- Phase 3 has no dependencies (generic container)
- Phase 4 depends on Phase 2 + 3
- Phase 5 depends on Phase 4
