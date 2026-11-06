#!/bin/bash
# SPDX-License-Identifier: MIT
# The opt-in coverage build of `melee` (docs/rust-port/notes/coverage-testing.md):
#   bash tools/rust-port/covtest/build_cov.sh            -> $COV_TARGET/cov/melee.exe
# LLVM source-based coverage with branch counters in the game crates only (rustc_cov.rs), profile `cov` (= fast).
# Env: COV_TARGET (default _build/cargo-agents/covtest), the target dir; never the shared or a normal one.
set -eu
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
root="$(cd "$here/../../.." && pwd)"
ws="${GW_WORKSPACE:-E:/Projects/Melee Workspace}"
export CARGO_TARGET_DIR="${COV_TARGET:-$ws/_build/cargo-agents/covtest}"
mkdir -p "$CARGO_TARGET_DIR/covwrap"
wrap="$CARGO_TARGET_DIR/covwrap/rustc_cov.exe"
if [ ! -x "$wrap" ] || [ "$here/rustc_cov.rs" -nt "$wrap" ]; then
  rustc -O --edition 2021 -o "$wrap" "$here/rustc_cov.rs"
fi
export RUSTC_WORKSPACE_WRAPPER="$wrap"
unset RUSTC_WRAPPER
cd "$root/rust"
# DISC=1: also the discrimination bits (feature disc), profile covdisc (its own exe: covdisc/melee.exe)
if [ -n "${DISC:-}" ]; then cargo build -p melee --profile covdisc --features disc "$@"; echo "cov build: $CARGO_TARGET_DIR/covdisc/melee.exe"; exit 0; fi
cargo build -p melee --profile cov "$@"
echo "cov build: $CARGO_TARGET_DIR/cov/melee.exe"
