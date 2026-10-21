#!/bin/bash
# Regenerate tests/data/oracle_vectors.txt from the C sources (Git Bash, LLVM at C:\Program Files\LLVM).
# The game TUs are compiled the way the real build compiles them (clang, contraction ON), and
# gwtool's fmuladd lowering is applied to the IR text (lower.py), so the oracle computes what the
# C build computes. Only the driver is compiled normally.
set -e
HERE="$(cd "$(dirname "$0")" && pwd)"
OUT="${OUT:-$HERE/../../target/g1-oracle}"
mkdir -p "$OUT"
cd "$HERE/../../.."
CL="${CLANG:-/c/Program Files/LLVM/bin/clang.exe}"
F=(--target=x86_64-pc-windows-msvc -std=c99 -nostdinc -fno-builtin -DLINT -DTARGET_PC -fno-short-enums
   -fsigned-char -mlong-double-64 -fno-strict-aliasing -fwrapv -fcommon -fgnu89-inline -ftrivial-auto-var-init=zero -O2
   -Xclang -disable-llvm-passes -S -emit-llvm -w -Isrc -Ipc -Ipc/gameworld -isystem src/MSL
   -isystem libs/dolphin/include -isystem libs/dolphin/src -include src/MSL/math_ppc.h
   "-D_Static_assert(a,b)=" -Dsize_t=gc_size_t)
LL=()
for f in pc/gameworld/mtx_pc.c pc/gameworld/gekko_fp.c libs/dolphin/src/dolphin/mtx/mtx44.c \
         src/sysdolphin/baselib/random.c src/melee/lb/lbtrigf.c src/melee/lb/lbvector.c "$HERE/wrap.c"; do
  n=$(basename "$f" .c)
  "$CL" "${F[@]}" "$f" -o "$OUT/$n.ll"
  python "$HERE/lower.py" "$OUT/$n.ll" "$OUT/$n.lo.ll"
  LL+=("$OUT/$n.lo.ll")
done
"$CL" -O2 -fno-builtin -ffp-contract=off -w "$HERE/driver.c" "${LL[@]}" -o "$OUT/oracle.exe"
"$OUT/oracle.exe" "$HERE/../tests/data/oracle_vectors.txt"
wc -l "$HERE/../tests/data/oracle_vectors.txt"
