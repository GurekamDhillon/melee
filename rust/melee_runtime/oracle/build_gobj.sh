#!/bin/bash
# Regenerate tests/data/oracle_gobj.txt (baselib GObj system: gobj.c, gobjplink.c, gobjgxlink.c,
# gobjproc.c, gobjobject.c, gobjuserdata.c, gobjinit.c) from the C sources. Same flags and gwtool
# lowering as build.sh (shipping-build contraction ON, lower.py; these TUs have no float code).
# Generated inputs only, no disc data.
set -e
HERE="$(cd "$(dirname "$0")" && pwd)"
OUT="${OUT:-$HERE/../../target/g1-oracle-gobj}"
mkdir -p "$OUT"
cd "$HERE/../../.."
CL="${CLANG:-/c/Program Files/LLVM/bin/clang.exe}"
F=(--target=x86_64-pc-windows-msvc -std=c99 -nostdinc -fno-builtin -DLINT -DTARGET_PC -fno-short-enums
   -fsigned-char -mlong-double-64 -fno-strict-aliasing -fwrapv -fcommon -fgnu89-inline -ftrivial-auto-var-init=zero -O2
   -Xclang -disable-llvm-passes -S -emit-llvm -w -Isrc -Ipc -Ipc/gameworld -isystem src/MSL
   -isystem libs/dolphin/include -isystem libs/dolphin/src -include src/MSL/math_ppc.h
   "-D_Static_assert(a,b)=" -Dsize_t=gc_size_t)
LL=()
for n in gobj gobjplink gobjgxlink gobjproc gobjobject gobjuserdata gobjinit; do
  "$CL" "${F[@]}" "src/sysdolphin/baselib/$n.c" -o "$OUT/$n.ll"
  python "$HERE/lower.py" "$OUT/$n.ll" "$OUT/$n.lo.ll"
  LL+=("$OUT/$n.lo.ll")
done
"$CL" -O2 -fno-builtin -w "$HERE/driver_gobj.c" "$HERE/stubs_gobj.c" "${LL[@]}" -o "$OUT/oracle_gobj.exe"
"$OUT/oracle_gobj.exe" "$HERE/../tests/data/oracle_gobj.txt"
wc -lc "$HERE/../tests/data/oracle_gobj.txt"
