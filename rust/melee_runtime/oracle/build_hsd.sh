#!/bin/bash
# Regenerate tests/data/oracle_hsd.txt (baselib mtx.c, quatlib.c, spline.c) from the C sources.
# Same flags and gwtool lowering as build.sh (contraction ON, lower.py), Git Bash + LLVM.
set -e
HERE="$(cd "$(dirname "$0")" && pwd)"
OUT="${OUT:-$HERE/../../target/g1-oracle-hsd}"
mkdir -p "$OUT"
cd "$HERE/../../.."
CL="${CLANG:-/c/Program Files/LLVM/bin/clang.exe}"
F=(--target=x86_64-pc-windows-msvc -std=c99 -nostdinc -fno-builtin -DLINT -DTARGET_PC -fno-short-enums
   -fsigned-char -mlong-double-64 -fno-strict-aliasing -fwrapv -fcommon -fgnu89-inline -ftrivial-auto-var-init=zero -O2
   -Xclang -disable-llvm-passes -S -emit-llvm -w -Isrc -Ipc -Ipc/gameworld -isystem src/MSL
   -isystem libs/dolphin/include -isystem libs/dolphin/src -include src/MSL/math_ppc.h
   "-D_Static_assert(a,b)=" -Dsize_t=gc_size_t)
LL=()
for f in pc/gameworld/mtx_pc.c pc/gameworld/gekko_fp.c src/melee/lb/lbtrigf.c \
         src/sysdolphin/baselib/mtx.c src/sysdolphin/baselib/quatlib.c src/sysdolphin/baselib/spline.c \
         "$HERE/wrap.c" "$HERE/stubs_hsd.c"; do
  n=$(basename "$f" .c)
  "$CL" "${F[@]}" "$f" -o "$OUT/$n.ll"
  python "$HERE/lower.py" "$OUT/$n.ll" "$OUT/$n.lo.ll"
  LL+=("$OUT/$n.lo.ll")
done
"$CL" -O2 -fno-builtin -ffp-contract=off -w "$HERE/driver_hsd.c" "${LL[@]}" -o "$OUT/oracle_hsd.exe"
"$OUT/oracle_hsd.exe" "$HERE/../tests/data/oracle_hsd.txt"
wc -l "$HERE/../tests/data/oracle_hsd.txt"
