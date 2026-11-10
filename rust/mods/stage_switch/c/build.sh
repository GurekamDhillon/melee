#!/bin/bash
# The C twin of the stage_switch mod: OUT/stage_switch.wasm (PERIOD=N overrides the 600 logic frames per stage; ROTATION_EXT=1 adds Kongo 64, YI64 and PS).
#   bash rust/mods/stage_switch/c/build.sh [OUT]   (default build/mods-c)
set -eu
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; out="${1:-$here/../../../../build/mods-c}"; mkdir -p "$out"
clang --target=wasm32 -O2 -ffp-contract=off -nostdlib -ffreestanding -fno-builtin -Wall -Wextra -Werror \
  -Wl,--no-entry -Wl,--export-dynamic -Wl,--initial-memory=131072 -Wl,--max-memory=131072 -Wl,--export-table \
  -DPERIOD="${PERIOD:-600}" ${ROTATION_EXT:+-DROTATION_EXT} -o "$out/stage_switch.wasm" "$here/stage_switch.c"
echo "stage_switch (C): $out/stage_switch.wasm"
