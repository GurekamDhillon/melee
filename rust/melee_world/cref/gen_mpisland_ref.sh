#!/usr/bin/env bash
# Build the C reference for melee_world::mp::{island, stitch}; regenerates tests/data/mpisland_ref.txt.
#
# Same method as gen_mplib_joint_ref.sh: bodies cut verbatim out of mplib.c and mpisland.c,
# built i686 (4-byte pointers), SSE, contraction ON like the shipping build
# (_build/masstest/pipe_win.sh), then gwtool's fmuladd lowering redone by
# melee_runtime/oracle/lower.py.
#
# Usage (Git Bash, from anywhere): bash rust/melee_world/cref/gen_mpisland_ref.sh
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../../.." && pwd)"
out="${TMPDIR:-${TEMP:-/tmp}}/melee_world_cref_island"
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
sed -e 's/sqrt_tmp - 4)/sqrt_tmp)/' -e 's/sqrt_tmp - 5)/sqrt_tmp)/' "$out/mplib_joint_body.inc.raw" > "$out/mplib_joint_body.inc"
[ "$(grep -c 'sqrtf_store(x_f2 + y_f0, sqrt_tmp)' "$out/mplib_joint_body.inc")" = 2 ] || { echo "sqrt_tmp patch failed" >&2; exit 1; }
{
  awk '/^bool mpCheckFloor\(/{p=1} /^bool mpLineIntersectionV\(/{p=0} p' "$src"
  awk '/^bool mpCheckLeftWall\(/{p=1} /^static inline int mpLineGetNextCheckInline/{p=0} p' "$src"
  awk '/^bool mpCheckedBounding\(/{p=1} /^static char lbl_803BF526/{p=0} p' "$src"
} > "$out/mplib_check_body.inc"
awk '/^static inline void mpLib_GetJointVtxRange\(/{p=1} /^bool mpCheckedBounding\(/{p=0} p' "$src" > "$out/mplib_stitch_body.inc"
awk '/^\/\* 458E88 \*\/ struct mpIsland_80458E88_t/{p=1} p' "$root/src/melee/mp/mpisland.c" | sed 's/^\/\* 458E88 \*\/ struct/static struct/' > "$out/mpisland_body.inc"
awk '{print} /^#include "mplib_lines_body.inc"$/{exit}' "$here/mplib_geom_ref.c" > "$out/check_prelude.inc"
for f in mplib_joint_body mplib_geom_body mplib_lines_body mplib_check_body mplib_stitch_body mpisland_body; do
  [ -s "$out/$f.inc" ] || { echo "$f marker extraction failed" >&2; exit 1; }
done

clang="${CLANG:-/c/Program Files/LLVM/bin/clang.exe}"
"$clang" --target=i686-pc-windows-msvc -O2 -Xclang -disable-llvm-passes -ffp-contract=on -ftrivial-auto-var-init=zero -S -emit-llvm -w -I"$out" -I"$root/pc/gameworld" "$here/mpisland_ref.c" -o "$out/mpisland_ref.ll"
python "${LOWER_PY:-$root/rust/melee_runtime/oracle/lower.py}" "$out/mpisland_ref.ll" "$out/mpisland_ref.lo.ll"
"$clang" --target=i686-pc-windows-msvc -O1 -msse2 -mfpmath=sse -w "$out/mpisland_ref.lo.ll" "$root/pc/gameworld/gekko_fp.c" -o "$out/mpisland_ref.exe"
"$out/mpisland_ref.exe" > "$here/../tests/data/mpisland_ref.txt"
wc -l "$here/../tests/data/mpisland_ref.txt"
