#!/bin/bash
# Build the in-repo wasm mods (rust/mods/*) to build/mods/*.wasm (mod-c-v1 limits: fixed 1 MiB memory, 16 KiB stack,
# exported function table). Needs `rustup target add wasm32-unknown-unknown`.
#   build_mods.sh [mod ...]   only the named mods (e.g. `build_mods.sh lab quick_swap`); no argument = all. A change under
#   rust/mods needs only this, not the exe: the host loads build/mods/*.wasm at run time. Mods are built 4 at a time (MODS_JOBS).
#   Run it through `buildlock.sh run --light <owner> bash tools/rust-port/modhost/build_mods.sh ...` (a wasm mod is small).
set -eu
repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
out="$repo/build/mods"; mkdir -p "$out"
build_one() {
  m="$1"; n=$(basename "$m")
  grep -q cdylib "$m/Cargo.toml" || return 0  # a helper library (gd_ui_enc), not a mod
  RUSTFLAGS="-C target-feature=-simd128 -C link-arg=--initial-memory=1048576 -C link-arg=--max-memory=1048576 -C link-arg=-zstack-size=16384 -C link-arg=--export-table"     cargo build --quiet --release --target wasm32-unknown-unknown --manifest-path "$m/Cargo.toml" --target-dir "$repo/build/cargo-mods/$n"
  cp "$repo/build/cargo-mods/$n/wasm32-unknown-unknown/release/$n.wasm" "$out/$n.wasm"
  echo "built $out/$n.wasm"
}
export -f build_one; export repo out
if [ $# -gt 0 ]; then list=(); for n in "$@"; do list+=("$repo/rust/mods/$n"); done; else list=(); for m in "$repo"/rust/mods/*/; do list+=("${m%/}"); done; fi
for m in "${list[@]}"; do printf '%s\0' "$m"; done | xargs -0 -n1 -P "${MODS_JOBS:-4}" bash -c 'build_one "$0"'
