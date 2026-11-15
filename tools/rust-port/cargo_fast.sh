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
# Does not touch the release profile; `--release` / `--profile release|perf` pass through unchanged. RELEASE=1: release instead of fast.
# One cargo target dir per worktree (notes/buildspeed.md): a CARGO_TARGET_DIR shared by worktrees makes lanes overwrite each
# other's rlibs (cargo hashes workspace crates by workspace-relative path, so two trees get the same file names).
# A target dir outside this worktree is ignored (build/cargo here, from .cargo/config.toml) unless ALLOW_SHARED_TARGET=1.
_repo_for_target="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
if [ -n "${CARGO_TARGET_DIR:-}" ] && [ -z "${ALLOW_SHARED_TARGET:-}" ]; then
  case "$(cd "$CARGO_TARGET_DIR" 2>/dev/null && pwd || echo "$CARGO_TARGET_DIR")/" in
    "$_repo_for_target"/*) ;;
    *) echo "$(basename "$0"): ignoring CARGO_TARGET_DIR=$CARGO_TARGET_DIR (outside this worktree; ALLOW_SHARED_TARGET=1 keeps it)" >&2; unset CARGO_TARGET_DIR ;;
  esac
fi
ws="${GW_WORKSPACE:-E:/Projects/Melee Workspace}"
if [ -z "${NO_SCCACHE:-}" ]; then
  export SCCACHE_DIR="${SCCACHE_DIR:-$ws/_build/sccache}"
  export SCCACHE_CACHE_SIZE="${SCCACHE_CACHE_SIZE:-30G}"
  export RUSTC_WRAPPER="${RUSTC_WRAPPER:-sccache}"
  command -v "$RUSTC_WRAPPER" >/dev/null 2>&1 || { echo "cargo_fast: $RUSTC_WRAPPER not found (cargo install sccache --locked); building without it" >&2; unset RUSTC_WRAPPER; }
fi
[ -n "${FAST_NO_INCR:-}" ] && export CARGO_INCREMENTAL=0
export CARGO_PROFILE_DEV_DEBUG="${CARGO_PROFILE_DEV_DEBUG:-0}" CARGO_PROFILE_TEST_DEBUG="${CARGO_PROFILE_TEST_DEBUG:-0}"
# Default profile (notes/buildspeed.md): `build` and `run` without --release/--profile get `--profile fast`; RELEASE=1 gets
# `--release` instead. Candidate, packaging and perf/frame-time builds do not come through here: they run plain
# `cargo build --release` (package.sh, zero_hitch_gate.sh, the exe handed to the owner) and must say so.
sub="${1:-}"
case "$sub" in
  build|run)
    case " $* " in
      *" --release "*|*" --profile "*|*" --profile="*|*" -r "*) ;;
      *) if [ -n "${RELEASE:-}" ]; then set -- "$sub" --release "${@:2}"; else set -- "$sub" --profile fast "${@:2}"; fi ;;
    esac ;;
esac
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$here/../../rust" && exec cargo "$@"
