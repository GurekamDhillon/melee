#!/bin/bash
# Regenerate tests/data/native_cobj_eye.txt from cobj_eye_ref.c over the real pc/gameworld/mtx_pc.c + gekko_fp.c
# (shipping-build contraction + lower.py, as melee_runtime/oracle/build_cobj.sh). Git Bash + LLVM. No disc data.
set -e
HERE="$(cd "$(dirname "$0")" && pwd)"
OUT="${OUT:-$HERE/../../../target/items-oracle-cobj-eye}"
mkdir -p "$OUT"
cd "$HERE/../../../.."
CL="${CLANG:-/c/Program Files/LLVM/bin/clang.exe}"
F=(--target=x86_64-pc-windows-msvc -std=c99 -nostdinc -fno-builtin -DLINT -DTARGET_PC -fno-short-enums
   -fsigned-char -mlong-double-64 -fno-strict-aliasing -fwrapv -fcommon -fgnu89-inline -ftrivial-auto-var-init=zero -O2
   -Xclang -disable-llvm-passes -S -emit-llvm -w -Isrc -Ipc -Ipc/gameworld -isystem src/MSL
   -isystem libs/dolphin/include -isystem libs/dolphin/src -include src/MSL/math_ppc.h
   "-D_Static_assert(a,b)=" -Dsize_t=gc_size_t)
LL=()
for f in pc/gameworld/{mtx_pc,gekko_fp}.c; do
  n=$(basename "$f" .c)
  "$CL" "${F[@]}" "$f" -o "$OUT/$n.ll"
  python rust/melee_runtime/oracle/lower.py "$OUT/$n.ll" "$OUT/$n.lo.ll"
  LL+=("$OUT/$n.lo.ll")
done
"$CL" -O2 -fno-builtin -ffp-contract=off -w rust/melee_items/gen/native/cobj_eye_ref.c "${LL[@]}" -o "$OUT/cobj_eye_ref.exe"
"$OUT/cobj_eye_ref.exe" > rust/melee_items/tests/data/native_cobj_eye.txt
wc -l rust/melee_items/tests/data/native_cobj_eye.txt
