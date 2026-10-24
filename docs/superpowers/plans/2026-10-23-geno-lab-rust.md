# Geno LAB native Rust implementation plan

> **For agentic workers:** Use superpowers:executing-plans to implement this plan task by task.

**Goal:** Port every function in `pc/geno/geno_lab_mode.c` to `melee_modes` and verify each against native C.

**Architecture:** Native `melee_types` records, generated GC metadata, the existing TU hook convention, and the shared MTU oracle infrastructure. The oracle includes the original C TU, compiles with shipping contraction and gwtool lowering, and compares logical fields, returns, callback identities, and dependency call traces.

**Tech Stack:** Rust, Python/libclang, native Windows clang, existing Cargo workspace.

**Spec:** User shard-993-993 request; `RUST_PORT.md`; `docs/rust-port/inventory.md` section 4; `docs/rust-port/group-modes.md`.

## Global constraints

- Work only in this worktree on `port/shard-993-993`; never push, merge, or remove the worktree.
- Generate types with `melee_types`; native little-endian values retain GC offset metadata.
- All cargo commands set `CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_PROFILE_TEST_DEBUG=0`, `CARGO_INCREMENTAL=0`.
- Build the workspace and test `melee_modes`; do not test the entire workspace.
- No disc-derived data. Every commit ends with the requested Codex co-author line.

## Review focus

- Six-slot scene exits and invalid destinations: exercise all menu routes and rejected requests.
- Cursor ownership and callback identity: stale/replaced fighters, invalid slots, sub-fighters, missing lists.
- Hitlag/burst/hold sequencing: phase boundaries, victim clearing, pulse saturation, eight-frame rehit.
- Floating point: signed zero, NaN/range rejection, speed clamps, fused distance with discriminating inputs.
- Full oracle coverage: include static helpers and both C test-entry functions, not only public API smoke tests.

## Task 1: Types and failing oracle

- [ ] Generate `FlyCursor` from its C declaration using `rust/melee_types/gen/gen_types.py`.
- [ ] Add `rust/melee_modes/gen/cref/geno_lab_oracle.py`, generated C/Rust walkers, and `tests/tu_geno_lab_mode.rs` using MTU conventions.
- [ ] Compile native C with shipping contraction, generate synthetic fixtures, and run a failing Rust oracle before implementation.
- [ ] Commit generated type support independently after checking layout/build.

## Task 2: Native TU implementation

- [ ] Implement all C-defined functions in `src/tu/geno_lab_mode.rs`, and register once in `src/tu/mod.rs`.
- [ ] Reuse available dependency ports as defaults; unported calls remain named TODOs in `mod ext` with oracle overrides.
- [ ] Run every function's oracle, correct mismatches, and mutation-check sensitive paths.
- [ ] Commit the complete port and oracle.

## Task 3: Completion checks

- [ ] Run `cargo build --workspace` and `cargo test -p melee_modes` with the three required environment variables.
- [ ] Review coverage and the final diff; record verification limits and remaining stubs.
- [ ] Commit the report; report functions ported/oracle-verified/smoke-only and blockers.

## Execution ledger

Ruling: Execute directly under the user's explicit port-and-commit instruction; no additional plan approval is needed. Existing worktree is clean and on the requested branch.
