#!/bin/bash
# Build the in-repo wasm mods (rust/mods/*) to build/mods/*.wasm (mod-c-v1 limits: fixed 1 MiB memory, 16 KiB stack,
# exported function table). Needs `rustup target add wasm32-unknown-unknown`.
set -eu
repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
out="$repo/build/mods"; mkdir -p "$out"
for m in "$repo"/rust/mods/*/; do
  n=$(basename "$m")
  RUSTFLAGS="-C target-feature=-simd128 -C link-arg=--initial-memory=1048576 -C link-arg=--max-memory=1048576 -C link-arg=-zstack-size=16384 -C link-arg=--export-table" \
    cargo build --quiet --release --target wasm32-unknown-unknown --manifest-path "$m/Cargo.toml" --target-dir "$repo/build/cargo-mods"
  cp "$repo/build/cargo-mods/wasm32-unknown-unknown/release/$n.wasm" "$out/$n.wasm"
  echo "built $out/$n.wasm"
done
