#!/bin/bash
# Build goku.wasm (geno 12 fighter code): freestanding C, one fixed 64 KiB page, no libc, no start function.
#   bash rust/mods/geno_goku/build.sh [out.wasm]   (default: build/mods/geno_goku.wasm)
set -eu
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
out="${1:-$here/../../../build/mods/geno_goku.wasm}"
mkdir -p "$(dirname "$out")"
clang --target=wasm32 -O2 -nostdlib -ffreestanding -fno-builtin -Wall -Wextra -Werror \
  -Wl,--no-entry -Wl,--initial-memory=65536 -Wl,--max-memory=65536 -Wl,-z,stack-size=8192 \
  -Wl,--allow-undefined -o "$out" "$here/goku.c"
echo "built $out ($(wc -c <"$out") bytes)"
