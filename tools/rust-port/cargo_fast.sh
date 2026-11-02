#!/bin/bash
# The agents' fast cargo wrapper (notes/fastbuild.md): cargo + sccache for the dependencies + incremental for the workspace.
#   bash tools/rust-port/cargo_fast.sh build -p melee [--profile fast]     (any cargo arguments; run from anywhere)
# Env:
#   RUSTC_WRAPPER=sccache, SCCACHE_DIR=E:/Projects/Melee Workspace/_build/sccache, SCCACHE_CACHE_SIZE=30G (all overridable).
#   NO_SCCACHE=1   plain cargo (no wrapper).
#   FAST_NO_INCR=1 CARGO_INCREMENTAL=0: workspace crates are then cached by sccache too (a clean build in a new worktree is
#                  warm for everything), but every edit recompiles the whole crate. Default: incremental on, so only the
#                  third-party dependencies go through sccache.
# Trade-off: sccache does not cache a crate rustc compiles with -C incremental (it reports "incremental compilation" as
# non-cacheable), and cargo only makes workspace/path crates incremental. So the dependencies (wgpu, naga, lua-src, ...) come out of
# sccache across worktrees and target dirs, and the workspace crates come out of each worktree's incremental dir. Never delete a
# worktree's build/cargo/fast/incremental if you want the one-file edit loop to stay fast.
# Does not touch the release profile; `--release` / `--profile release|perf` pass through unchanged.
ws="${GW_WORKSPACE:-E:/Projects/Melee Workspace}"
if [ -z "${NO_SCCACHE:-}" ]; then
  export SCCACHE_DIR="${SCCACHE_DIR:-$ws/_build/sccache}"
  export SCCACHE_CACHE_SIZE="${SCCACHE_CACHE_SIZE:-30G}"
  export RUSTC_WRAPPER="${RUSTC_WRAPPER:-sccache}"
  command -v "$RUSTC_WRAPPER" >/dev/null 2>&1 || { echo "cargo_fast: $RUSTC_WRAPPER not found (cargo install sccache --locked); building without it" >&2; unset RUSTC_WRAPPER; }
fi
[ -n "${FAST_NO_INCR:-}" ] && export CARGO_INCREMENTAL=0
export CARGO_PROFILE_DEV_DEBUG="${CARGO_PROFILE_DEV_DEBUG:-0}" CARGO_PROFILE_TEST_DEBUG="${CARGO_PROFILE_TEST_DEBUG:-0}"
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$here/../../rust" && exec cargo "$@"
