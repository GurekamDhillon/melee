#!/bin/bash
# Regenerate tests/data/oracle_anim.txt (baselib fobj.c, util.c, crypt.c; spline.c for
# splGetHelmite) from the C sources. Same flags and gwtool lowering as build.sh (shipping-build
# contraction ON, lower.py), Git Bash + LLVM. Generated inputs only, no disc data.
set -e
HERE="$(cd "$(dirname "$0")" && pwd)"
OUT="${OUT:-$HERE/../../target/g1-oracle-anim}"
mkdir -p "$OUT"
cd "$HERE/../../.."
CL="${CLANG:-/c/Program Files/LLVM/bin/clang.exe}"
F=(--target=x86_64-pc-windows-msvc -std=c99 -nostdinc -fno-builtin -DLINT -DTARGET_PC -fno-short-enums
   -fsigned-char -mlong-double-64 -fno-strict-aliasing -fwrapv -fcommon -fgnu89-inline -ftrivial-auto-var-init=zero -O2
   -Xclang -disable-llvm-passes -S -emit-llvm -w -Isrc -Ipc -Ipc/gameworld -isystem src/MSL
   -isystem libs/dolphin/include -isystem libs/dolphin/src -include src/MSL/math_ppc.h
   "-D_Static_assert(a,b)=" -Dsize_t=gc_size_t)
LL=()
for f in src/sysdolphin/baselib/fobj.c src/sysdolphin/baselib/util.c src/sysdolphin/baselib/crypt.c \
         src/sysdolphin/baselib/spline.c pc/gameworld/mtx_pc.c pc/gameworld/gekko_fp.c src/melee/lb/lbtrigf.c \
         "$HERE/wrap.c"; do
  n=$(basename "$f" .c)
  "$CL" "${F[@]}" "$f" -o "$OUT/$n.ll"
  python "$HERE/lower.py" "$OUT/$n.ll" "$OUT/$n.lo.ll"
  LL+=("$OUT/$n.lo.ll")
done
"$CL" -O2 -fno-builtin -ffp-contract=off -w "$HERE/driver_anim.c" "$HERE/stubs_anim.c" "${LL[@]}" -o "$OUT/oracle_anim.exe"
"$OUT/oracle_anim.exe" "$HERE/../tests/data/oracle_anim.txt"
wc -l "$HERE/../tests/data/oracle_anim.txt"
