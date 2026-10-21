#!/usr/bin/env bash
# Build the C reference for melee_world::gr::stage; regenerates tests/data/stage_ref.txt.
#
# Same method as gen_camera_ref.sh: bodies cut verbatim out of src/melee/gr/stage.c (+ its
# stage_stage_slot_retail.inc), pc/gameworld/script_arena.inc (ScriptGame_ArenaBound) and
# src/melee/lb/lbvector.c (sin/cos/Rotate/ApplyEulerRotation), built i686 SSE with contraction ON
# like the shipping build, then gwtool's fmuladd lowering redone by melee_runtime/oracle/lower.py.
#
# Usage (Git Bash, from anywhere): bash rust/melee_world/cref/gen_stage_ref.sh
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../../.." && pwd)"
out="${TMPDIR:-${TEMP:-/tmp}}/melee_world_cref_stage"
mkdir -p "$out"
st="$root/src/melee/gr/stage.c"

awk '/^f32 Stage_GetCamBoundsLeftOffset\(/{p=1} /^void Stage_80224E38\(/{p=0} p' "$st" \
  | grep -v '^#undef ARENA_BOUND' > "$out/stage_body.inc"
{
  awk '/^struct StageIdMapEntry stage_id_map\[ST_MEX_EXT_MAX\] = \{$/{p=1} p&&/^#else$/{next} p&&/^struct StageIdMapEntry stage_id_map\[\] = \{$/{next} p&&/^#endif$/{next} p{print} p&&/^\};$/{exit}' "$st"
  awk '/^void Stage_MexInitIds\(void\)$/{p=1} p{print} p&&/^}$/{exit}' "$st"
  awk '/^GrKind Stage_8022519C\(/{p=1} p{print} p&&/^}$/{exit}' "$st"
} > "$out/stage_map.inc"
cp "$root/src/melee/gr/stage_stage_slot_retail.inc" "$out/slot.inc"
awk '/^float ScriptGame_ArenaBound\(/{p=1} p{print} p&&/^}$/{exit}' "$root/pc/gameworld/script_arena.inc" > "$out/arena.inc"
{
  awk '/^static float lbvector_sin\(/{p=1} /^\/\/\/ 8000D8F4|^void lbVector_RotateAboutUnitAxis/{p=0} p' "$root/src/melee/lb/lbvector.c"
  awk '/^void lbVector_Rotate\(/{p=1} p{print} p&&/^}$/{exit}' "$root/src/melee/lb/lbvector.c"
  awk '/^Vec3\* lbVector_ApplyEulerRotation\(/{p=1} p{print} p&&/^}$/{exit}' "$root/src/melee/lb/lbvector.c"
} > "$out/lbvector.inc"
# GrKind enum from gr/forward.h
awk '/^typedef enum GrKind/{p=1} p{print} p&&/\} GrKind;/{exit}' "$root/src/melee/gr/forward.h" > "$out/grkind.inc"
for f in stage_body stage_map slot arena lbvector grkind; do
  [ -s "$out/$f.inc" ] || { echo "$f extraction failed" >&2; exit 1; }
done
grep -q 'Stage_80224DC8' "$out/stage_body.inc" || { echo "80224DC8 not cut" >&2; exit 1; }
grep -q 'Gr_Kind_Battle, 0, 0 },       { Gr_Kind_PStadium' "$out/stage_map.inc" || { echo "map not cut" >&2; exit 1; }

clang="${CLANG:-/c/Program Files/LLVM/bin/clang.exe}"
"$clang" --target=i686-pc-windows-msvc -O2 -Xclang -disable-llvm-passes -ffp-contract=on -ftrivial-auto-var-init=zero -fwrapv -S -emit-llvm -w -I"$out" "$here/stage_ref.c" -o "$out/stage_ref.ll"
python "${LOWER_PY:-$root/rust/melee_runtime/oracle/lower.py}" "$out/stage_ref.ll" "$out/stage_ref.lo.ll"
"$clang" --target=i686-pc-windows-msvc -O1 -msse2 -mfpmath=sse -w "$out/stage_ref.lo.ll" -o "$out/stage_ref.exe"
"$out/stage_ref.exe" > "$here/../tests/data/stage_ref.txt"
wc -l "$here/../tests/data/stage_ref.txt"
