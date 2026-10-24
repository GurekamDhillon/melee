#!/bin/bash
# Synthetic helper vectors only; no game assets. Reuses the shipping arithmetic lowering.
set -eu
HERE="$(cd "$(dirname "$0")" && pwd)"
OUT="$HERE/../../target/g1-oracle-shard-leaf"
mkdir -p "$OUT"
cd "$HERE/../../.."
CL="${CLANG:-/c/Program Files/LLVM/bin/clang.exe}"
"$CL" --target=x86_64-pc-windows-msvc -std=c99 -nostdinc -fno-builtin -DLINT -DTARGET_PC \
  -fno-short-enums -fsigned-char -mlong-double-64 -fno-strict-aliasing -fwrapv -fcommon \
  -fgnu89-inline -ftrivial-auto-var-init=zero -ffp-contract=on -O2 -Xclang -disable-llvm-passes \
  -S -emit-llvm -w -Isrc -Ipc -Ipc/gameworld -isystem src/MSL -isystem libs/dolphin/include \
  -isystem libs/dolphin/src -include src/MSL/math_ppc.h '-D_Static_assert(a,b)=' -Dsize_t=gc_size_t \
  "$HERE/helper_shard_leaf.c" -o "$OUT/leaf.ll"
python "$HERE/lower.py" "$OUT/leaf.ll" "$OUT/leaf.lo.ll"
"$CL" -O2 -fno-builtin -ffp-contract=off -w "$HERE/driver_shard_leaf.c" "$OUT/leaf.lo.ll" -o "$OUT/oracle.exe"
"$OUT/oracle.exe" "$HERE/../tests/data/oracle_shard_leaf.txt"
wc -l "$HERE/../tests/data/oracle_shard_leaf.txt"
