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
# the disc tests (melee_core/test_support/disc.rs) fail without GW_ISO_VANILLA unless MELEE_SKIP_DISC_TESTS=1
if [ -z "${GW_ISO_VANILLA:-}" ]; then
  if [ -n "${MELEE_SKIP_DISC_TESTS:-}" ]; then echo "oracle_tests: WARNING no disc - the disc tests are SKIPPED, not passed" >&2
  else echo "oracle_tests: no disc - the disc tests will FAIL (set GW_ISO_VANILLA, or MELEE_SKIP_DISC_TESTS=1 to skip them)" >&2; fi
fi
exec cargo test --release "$@"
