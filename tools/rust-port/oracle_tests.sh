#!/usr/bin/env bash
# Run the Rust workspace tests the way the C-oracle compares need them (docs/rust-port/notes/fn-identity.md):
# function merging off, so body-identical C callbacks keep distinct addresses (rustc's MergeFunctions folds them,
# and a state-table / callback hash then names two C functions with one id). -Z needs RUSTC_BOOTSTRAP on stable.
# Its own target dir: the flag changes codegen, and the shipping build/cargo stays as it is.
#   bash tools/rust-port/oracle_tests.sh [cargo test args...]    e.g. -p melee_items --no-fail-fast
set -euo pipefail
repo="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$repo/rust"
export RUSTC_BOOTSTRAP=1
export RUSTFLAGS="${RUSTFLAGS:+$RUSTFLAGS }-Zmerge-functions=disabled"
export CARGO_TARGET_DIR="${ORACLE_TARGET_DIR:-$repo/build/cargo-oracle}"
# one test at a time inside each binary: the harnesses keep their state in `static mut`s (the log, the arena, the
# fn registry), and libtest's parallel threads race on it (tu_ftkirby segfaulted that way)
export RUST_TEST_THREADS="${RUST_TEST_THREADS:-1}"
exec cargo test --release "$@"
