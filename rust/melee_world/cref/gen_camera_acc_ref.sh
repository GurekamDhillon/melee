#!/usr/bin/env bash
# Build the C reference for the melee_world::cm::camera accessors, Camera_8002FE38/8002EEC8/
# 8002BC78 and the PC cm_script helpers; regenerates tests/data/camera_acc_ref.txt.
#
# Bodies are cut verbatim from src/melee/cm/camera.c into camera_acc_body.inc and compiled in
# camera_acc_tu.c with the game flags together with lbvector.c, lbtrigf.c, gekko_fp.c, mtx44.c and
# mtx_pc.c (the gen_ground_ref.sh recipe: contraction ON like _build/masstest/pipe_win.sh, then
# the gwtool fmuladd lowering redone by lower.py). Only the driver is compiled normally.
#
# Usage (Git Bash, from anywhere): bash rust/melee_world/cref/gen_camera_acc_ref.sh
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../../.." && pwd)"
out="${TMPDIR:-${TEMP:-/tmp}}/melee_world_cref_camera_acc"
mkdir -p "$out"
src="$root/src/melee/cm/camera.c"
cut() {
  awk -v n="$1" '!p && $0 ~ ("^[A-Za-z].*[ *]" n "[(]") && $0 !~ /;[ \t]*$/ {p=1} p{print} p&&/^}$/{exit}' "$src"
}
FNS="Camera_8002F260 Camera_SetModeToStandard Camera_SetModeToFixed Camera_800300F0 Camera_8003010C
Camera_80030130 Camera_80030154 Camera_80030178 Camera_80030730 Camera_SetBackgroundColor
Camera_GetBackgroundColor Camera_GetTransformPosition Camera_GetTransformInterest Camera_80030DE4
Camera_80030DF8 Camera_80030E10 Camera_80031060 Camera_80031074 Camera_8003108C Camera_800310A0
Camera_800310E8 Camera_80031144 Camera_800311CC Camera_800311DC Camera_StageSlotClipGet
Camera_80030A60 Camera_80030A78 Camera_80030A8C Camera_SetStageVisible Camera_80030AC4
Camera_80030AE0 Camera_80030AF8 Camera_80030B0C Camera_80030B24 Camera_80030B38 Camera_80030B50
Camera_80030B64 Camera_80030B7C Camera_80030B90 Camera_80030BA8 Camera_8002FE38 Camera_8002EEC8
OrthonormalizeBasis Camera_8002BC78 cm_script_float cm_script_bits cm_script_ease cm_script_blend"
{
  awk '/^typedef struct CmScriptPose \{$/{p=1} p{print} p&&/^\} CmScriptPose;$/{exit}' "$src"
  grep -E '^static f32 cm_script_lerp[(]' "$src"
  for f in $FNS; do cut "$f"; done
} > "$out/camera_acc_body.inc"
for f in $FNS cm_script_lerp; do
  grep -qE "[ *]$f[(]" "$out/camera_acc_body.inc" || { echo "$f not cut" >&2; exit 1; }
done
grep -q "CmScriptPose;" "$out/camera_acc_body.inc" || { echo "CmScriptPose not cut" >&2; exit 1; }
cd "$root"
CL="${CLANG:-/c/Program Files/LLVM/bin/clang.exe}"
F=(--target=x86_64-pc-windows-msvc -std=c99 -nostdinc -fno-builtin -DLINT -DTARGET_PC -fno-short-enums
   -fsigned-char -mlong-double-64 -fno-strict-aliasing -fwrapv -fcommon -fgnu89-inline -ftrivial-auto-var-init=zero -O2
   -Xclang -disable-llvm-passes -S -emit-llvm -w -Isrc -Isrc/melee -Ipc -Ipc/gameworld -isystem src/MSL
   -isystem libs/dolphin/include -isystem libs/dolphin/src -include src/MSL/math_ppc.h
   "-D_Static_assert(a,b)=" -Dsize_t=gc_size_t -I"$out" -I"$here")
LL=()
for f in pc/gameworld/mtx_pc.c pc/gameworld/gekko_fp.c libs/dolphin/src/dolphin/mtx/mtx44.c src/melee/lb/lbtrigf.c src/melee/lb/lbvector.c \
         rust/melee_runtime/oracle/wrap.c "$here/camera_acc_tu.c"; do
  n=$(basename "$f" .c)
  "$CL" "${F[@]}" "$f" -o "$out/$n.ll"
  python "$root/rust/melee_runtime/oracle/lower.py" "$out/$n.ll" "$out/$n.lo.ll"
  LL+=("$out/$n.lo.ll")
done
"$CL" -O2 -fno-builtin -ffp-contract=off -w -I"$here" "$here/camera_acc_ref.c" "${LL[@]}" -o "$out/camera_acc_ref.exe"
"$out/camera_acc_ref.exe" > "$here/../tests/data/camera_acc_ref.txt"
wc -l "$here/../tests/data/camera_acc_ref.txt"
