#!/bin/bash
# Builds the probe and its desync variant: OUT/hashword_probe.wasm and OUT/skew/hashword_probe.wasm (-DSKEW=$SKEW, default 600).
#   bash rust/mods/hashword_probe/build.sh [OUT]   (default build/mods-hashword)
set -eu
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; repo="$here/../../.."
out="${1:-$repo/build/mods-hashword}"; mkdir -p "$out/skew"
cc() { clang --target=wasm32 -O2 -ffp-contract=off -nostdlib -ffreestanding -fno-builtin -Wall -Wextra -Werror \
  -Wl,--no-entry -Wl,--export-dynamic -Wl,--initial-memory=131072 -Wl,--max-memory=131072 -Wl,--export-table "$@"; }
cc -o "$out/hashword_probe.wasm" "$here/hashword_probe.c"
cc -DSKEW="${SKEW:-600}" -o "$out/skew/hashword_probe.wasm" "$here/hashword_probe.c"
echo "hashword_probe: $out/hashword_probe.wasm, $out/skew/hashword_probe.wasm (skew from frame ${SKEW:-600})"
