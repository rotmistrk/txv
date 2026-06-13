---
name: implement-feature
description: Test-first implementation cycle for txv. Use when implementing a feature or fix. Writes failing test, implements, verifies with lint per file, builds, runs tests, commits.
---

# Implement Feature Skill

## When to Use
When implementing a new feature or fixing a bug in txv.

## Procedure

1. **Estimate** — Set LOE on the task. If LOE > 3, split into subtasks first.

2. **Test First** — Write a test that demonstrates the desired behavior. The test MUST fail initially.
   - Unit tests in the relevant crate's tests module
   - Integration tests in txv-gallery/tests/

3. **Implement** — Write minimal code to make the test pass.
   - After modifying EACH file: run `check_file` on it, fix violations immediately
   - Max 240 code lines per file
   - No unwrap/expect in production code

4. **Verify** — Run the full cycle:
   - `cargo fmt`
   - `cargo build`
   - `cargo clippy --workspace -- -D warnings`
   - `cargo test --workspace`

5. **Commit** — Stage, commit with descriptive message, push.

## Workspace Structure
- txv-core: View trait, GroupState, Buffer, event system, program loop
- txv-widgets: Reusable widgets (TabPanel, TreeView, InputLine, DropdownMenu, etc.)
- txv-edit: Editor component (EditorView, vi keymap, syntax highlighting)
- txv-gallery: Examples and integration tests

## Render Pipeline Rules (absolute)
1. draw() ONLY called from render() — never directly
2. blit_child() ONLY called from render()
3. set_child_bounds() ONLY from handle() — NEVER from draw()
4. render() called from parent group's render() or app event loop
5. mark_redrawn() only from render() (via default impl or delegate macro)
