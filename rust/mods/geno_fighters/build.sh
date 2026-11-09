#!/bin/bash
# Build the geno 12 fighter modules: each <name>.c here -> <name>.wasm beside it (the committed modules) or into OUT,
# and the game-half suite's fixtures tests/<name>.c -> tests/<name>.wasm.
# Freestanding C, one fixed 64 KiB page, no libc, no start function.   bash rust/mods/geno_fighters/build.sh [OUT]
set -eu
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
out="${1:-$here}"
mkdir -p "$out"
for c in "$here"/*.c "$here"/tests/*.c; do
  n=$(basename "$c" .c); d="$out"; [ "$(dirname "$c")" = "$here/tests" ] && d="$out/tests"; mkdir -p "$d"
  clang --target=wasm32 -O2 -nostdlib -ffreestanding -fno-builtin -Wall -Wextra -Werror \
    -Wl,--no-entry -Wl,--initial-memory=65536 -Wl,--max-memory=65536 -Wl,-z,stack-size=8192 \
    -Wl,--allow-undefined -o "$d/$n.wasm" "$c"
  echo "built $d/$n.wasm ($(wc -c <"$d/$n.wasm") bytes)"
done
