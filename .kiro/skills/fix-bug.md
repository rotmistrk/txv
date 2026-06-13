---
name: fix-bug
description: Bug fix cycle for txv. Reproduce with test, diagnose, fix, verify no regressions.
---

# Fix Bug Skill

## When to Use
When a bug is reported or discovered.

## Procedure

1. **Reproduce** — Write a test that demonstrates the bug (test MUST fail).

2. **Diagnose** — Read the relevant code path. Identify root cause.

3. **Fix** — Apply minimal change. After each file modification: `check_file`, fix violations.

4. **Verify** — Run:
   - `cargo fmt`
   - `cargo build`
   - `cargo test --workspace`

5. **Commit** — Descriptive message referencing the bug symptoms.

## Anti-patterns
- Do NOT fix the test to match broken behavior
- Do NOT add `#[ignore]` to tests
- If the fix would break other tests, diagnose deeper
