#!/usr/bin/env bash
# Build the C reference for melee_world::cm::camera; regenerates tests/data/camera_ref.txt.
#
# Same method as gen_mplib_joint_ref.sh: function bodies cut verbatim out of
# src/melee/cm/camera.c (and the struct definitions out of cm/types.h), built i686, SSE,
# contraction ON like the shipping build (_build/masstest/pipe_win.sh), then gwtool's fmuladd
# lowering redone by melee_runtime/oracle/lower.py. tanf comes from pc/gameworld/gekko_fp.c.
#
# Usage (Git Bash, from anywhere): bash rust/melee_world/cref/gen_camera_ref.sh
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../../.." && pwd)"
out="${TMPDIR:-${TEMP:-/tmp}}/melee_world_cref_camera"
mkdir -p "$out"
src="$root/src/melee/cm/camera.c"
types="$root/src/melee/cm/types.h"

awk '/^struct CmSubject \{$/{p=1} /^struct Camera_x2D0 \{$/{p=0} p' "$types" > "$out/cm_types.inc"
awk '/^struct CameraUnkGlobals \{$/{p=1} p&&/^\};$/{print; p=0} p' "$types" >> "$out/cm_types.inc"
{
  awk '/^extern int GenoFly_Any\(void\);$/{p=1} p{print} /^#define Stage_GetCamBoundsBottomOffset\(\) cm_fly_bottom\(\)$/{exit}' "$src"
  awk '/^u32 Camera_80029124\(/{p=1} /^\/\/\/ @note doesnt check all stages/{p=0} p' "$src"
  awk '/^bool Camera_8002E158\(/{p=1} /^bool Camera_8002E234\(/{p=0} p' "$src"
  awk '/^void Camera_800300F0\(/{p=1} /^Vec3\* Camera_8003019C\(/{p=0} p' "$src"
  awk '/^void Camera_80030730\(/{p=1} /^static inline void project_ground_x\(/{p=0} p' "$src"
  awk '/^void Camera_80030DE4\(/{p=1} /^static inline s64 inline_cam_gx_b0\(/{p=0} p' "$src"
} > "$out/camera_body.inc"
for f in cm_types camera_body; do
  [ -s "$out/$f.inc" ] || { echo "$f marker extraction failed" >&2; exit 1; }
done
grep -q '^void Camera_ApplyQuake' "$out/camera_body.inc" || { echo "ApplyQuake not cut" >&2; exit 1; }
grep -q '^void Camera_800311DC' "$out/camera_body.inc" || { echo "800311DC not cut" >&2; exit 1; }

clang="${CLANG:-/c/Program Files/LLVM/bin/clang.exe}"
"$clang" --target=i686-pc-windows-msvc -O2 -Xclang -disable-llvm-passes -ffp-contract=on -ftrivial-auto-var-init=zero -fwrapv -S -emit-llvm -w -I"$out" "$here/camera_ref.c" -o "$out/camera_ref.ll"
python "${LOWER_PY:-$root/rust/melee_runtime/oracle/lower.py}" "$out/camera_ref.ll" "$out/camera_ref.lo.ll"
"$clang" --target=i686-pc-windows-msvc -O1 -msse2 -mfpmath=sse -w "$out/camera_ref.lo.ll" "$root/pc/gameworld/gekko_fp.c" -o "$out/camera_ref.exe"
"$out/camera_ref.exe" > "$here/../tests/data/camera_ref.txt"
wc -l "$here/../tests/data/camera_ref.txt"
