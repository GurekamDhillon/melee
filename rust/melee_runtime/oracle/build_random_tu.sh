#!/bin/bash
# C random.c, including its private selector and statics; synthetic state only.
set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
OUT="$HERE/../../target/g1-oracle-random-tu"
mkdir -p "$OUT"
cd "$HERE/../../.."
CL="${CLANG:-/c/Program Files/LLVM/bin/clang.exe}"
F=(--target=x86_64-pc-windows-msvc -std=c99 -nostdinc -fno-builtin -DLINT -DTARGET_PC
   -fno-short-enums -fsigned-char -mlong-double-64 -fno-strict-aliasing -fwrapv -fcommon
   -fgnu89-inline -ftrivial-auto-var-init=zero -O2 -Xclang -disable-llvm-passes
   -S -emit-llvm -w -Isrc -Ipc -Ipc/gameworld -isystem src/MSL
   -isystem libs/dolphin/include -isystem libs/dolphin/src
   "-D_Static_assert(a,b)=" -Dsize_t=gc_size_t)
"$CL" "${F[@]}" "$HERE/driver_random_tu.c" -o "$OUT/random.ll"
python "$HERE/lower.py" "$OUT/random.ll" "$OUT/random.lo.ll"
"$CL" -O2 -fno-builtin -ffp-contract=off "$OUT/random.lo.ll" -o "$OUT/random.exe"
"$OUT/random.exe" > "$HERE/../tests/data/oracle_random_tu.txt"
wc -l "$HERE/../tests/data/oracle_random_tu.txt"
