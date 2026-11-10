## What changes for the person using Velapor

**Before:**

**After:**

## How

## Checks

- [ ] `cargo fmt --all`, `cargo clippy --workspace -- -D warnings` and `cargo test --workspace` pass
- [ ] Velapor still only reads: nothing in `core/` writes, changes or deletes anything (`core/tests/read_only.rs` passes)
- [ ] A new rule has a test in `core/`
