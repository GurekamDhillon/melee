#!/bin/bash
# Regenerate tests/data/oracle_fog.txt (the logic of baselib fog.c through helper_fog.c, over the real aobj.c,
# fobj.c, id.c, list.c, spline.c, mtx_pc.c and gekko_fp.c) from the C sources. Same flags and gwtool lowering as build.sh
# (shipping-build contraction ON, lower.py), Git Bash + LLVM. Generated inputs only, no disc data.
set -e
HERE="$(cd "$(dirname "$0")" && pwd)"
OUT="${OUT:-$HERE/../../target/g1-oracle-fog}"
mkdir -p "$OUT"
cd "$HERE/../../.."
CL="${CLANG:-/c/Program Files/LLVM/bin/clang.exe}"
F=(--target=x86_64-pc-windows-msvc -std=c99 -nostdinc -fno-builtin -DLINT -DTARGET_PC -fno-short-enums
   -fsigned-char -mlong-double-64 -fno-strict-aliasing -fwrapv -fcommon -fgnu89-inline -ftrivial-auto-var-init=zero -O2
   -Xclang -disable-llvm-passes -S -emit-llvm -w -Isrc -Ipc -Ipc/gameworld -isystem src/MSL
   -isystem libs/dolphin/include -isystem libs/dolphin/src -include src/MSL/math_ppc.h
   "-D_Static_assert(a,b)=" -Dsize_t=gc_size_t)
LL=()
for f in src/sysdolphin/baselib/{aobj,fobj,id,list,spline}.c pc/gameworld/{mtx_pc,gekko_fp}.c "$HERE/helper_fog.c"; do
  n=$(basename "$f" .c)
  "$CL" "${F[@]}" "$f" -o "$OUT/$n.ll"
  python "$HERE/lower.py" "$OUT/$n.ll" "$OUT/$n.lo.ll"
  LL+=("$OUT/$n.lo.ll")
done
"$CL" -O2 -fno-builtin -ffp-contract=off -w "$HERE/driver_fog.c" "$HERE/stubs_fog.c" "${LL[@]}" -o "$OUT/oracle_fog.exe"
"$OUT/oracle_fog.exe" "$HERE/../tests/data/oracle_fog.txt"
wc -lc "$HERE/../tests/data/oracle_fog.txt"
