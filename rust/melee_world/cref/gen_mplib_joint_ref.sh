#!/usr/bin/env bash
# Build the C reference for melee_world::mp::joint and regenerate tests/data/mplib_joint_ref.txt.
#
# Same method as gen_mplib_check_ref.sh: the bodies are cut verbatim out of src/melee/mp/mplib.c,
# built i686 (4-byte pointers), SSE, contraction ON like the shipping build
# (_build/masstest/pipe_win.sh), then gwtool's fmuladd lowering redone by
# melee_runtime/oracle/lower.py.
#
# Usage (Git Bash, from anywhere): bash rust/melee_world/cref/gen_mplib_joint_ref.sh
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../../.." && pwd)"
out="${TMPDIR:-${TEMP:-/tmp}}/melee_world_cref_joint"
mkdir -p "$out"
src="$root/src/melee/mp/mplib.c"

{
  awk '/^int mpLineGetNext\(int line_id\)$/{p=1} /^\/\/\/ remap point/{p=0} p' "$src"
  awk '/^static inline int mpLineGetNextCheckInline/{p=1} /^\/\/\/ what even is this lol/{p=0} p' "$src"
} > "$out/mplib_lines_body.inc"
{
  awk '/^\/\/\/ remap point p from line a to line b$/{p=1} /^bool mpCheckFloor\(/{p=0} p' "$src"
  awk '/^bool mpLineIntersectionV\(/{p=1} /^bool mpCheckLeftWall\(/{p=0} p' "$src"
  awk '/^void mpVtxGetPos\(/{p=1} /^#if defined\(TARGET_PC\)/{p=0} p' "$src"
} > "$out/mplib_geom_body.inc"
{
  awk '/^void mpJointHide\(/{p=1} /^void mpLib_80055E9C\(/{p=0} p' "$src"
  awk '/^void mpJointUpdateBounding\(/{p=1} /^void mpVtxGetPos\(/{p=0} p' "$src"
  awk '/^static inline float sqrtf_store/{p=1} /^void mpJointSetCb1\(/{p=0} p' "$src"
} > "$out/mplib_joint_body.inc.raw"
# mpLib_80056C54 passes `sqrt_tmp - 4` / `sqrt_tmp - 5` to sqrtf_store, an out-of-bounds volatile
# store (a decomp stack-matching trick). In the shipping build (ppc32 -O2, full pipeline) that
# store is UB and is deleted: the optimized IR of mpLib_80056C54 has no volatile store and no
# sqrt_tmp alloca. This oracle is built with -disable-llvm-passes, which would keep the store and
# clobber a neighbouring local, so it is pointed at sqrt_tmp itself (in bounds) instead.
sed -e 's/sqrt_tmp - 4)/sqrt_tmp)/' -e 's/sqrt_tmp - 5)/sqrt_tmp)/' "$out/mplib_joint_body.inc.raw" > "$out/mplib_joint_body.inc"
[ "$(grep -c 'sqrtf_store(x_f2 + y_f0, sqrt_tmp)' "$out/mplib_joint_body.inc")" = 2 ] || { echo "sqrt_tmp patch failed" >&2; exit 1; }
awk '{print} /^#include "mplib_lines_body.inc"$/{exit}' "$here/mplib_geom_ref.c" > "$out/check_prelude.inc"
for f in mplib_joint_body mplib_geom_body mplib_lines_body; do
  [ -s "$out/$f.inc" ] || { echo "$f marker extraction failed" >&2; exit 1; }
done

clang="${CLANG:-/c/Program Files/LLVM/bin/clang.exe}"
"$clang" --target=i686-pc-windows-msvc -O2 -Xclang -disable-llvm-passes -ffp-contract=on   -ftrivial-auto-var-init=zero -S -emit-llvm -w -I"$out" -I"$root/pc/gameworld"   "$here/mplib_joint_ref.c" -o "$out/mplib_joint_ref.ll"
python "${LOWER_PY:-$root/rust/melee_runtime/oracle/lower.py}" "$out/mplib_joint_ref.ll" "$out/mplib_joint_ref.lo.ll"
"$clang" --target=i686-pc-windows-msvc -O1 -msse2 -mfpmath=sse -w   "$out/mplib_joint_ref.lo.ll" "$root/pc/gameworld/gekko_fp.c" -o "$out/mplib_joint_ref.exe"
"$out/mplib_joint_ref.exe" > "$here/../tests/data/mplib_joint_ref.txt"
wc -l "$here/../tests/data/mplib_joint_ref.txt"
