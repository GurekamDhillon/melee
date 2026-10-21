#!/usr/bin/env bash
# Build the C reference for melee_world::gr::grlib; regenerates tests/data/grlib_ref.txt.
#
# PointInsideColl is cut verbatim from src/melee/gr/grlib.c into grlib_body.inc and compiled
# (grlib_tu.c, stand-in CollData) with the game's flags, contraction ON like
# _build/masstest/pipe_win.sh, then gwtool's fmuladd lowering redone by lower.py (the
# melee_runtime oracle recipe). Only the driver (grlib_ref.c) is compiled normally.
#
# Usage (Git Bash, from anywhere): bash rust/melee_world/cref/gen_grlib_ref.sh
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../../.." && pwd)"
out="${TMPDIR:-${TEMP:-/tmp}}/melee_world_cref_grlib"
mkdir -p "$out"
awk '/^static inline bool PointInsideColl\(/{p=1} p{print} p&&/^}$/{exit}' "$root/src/melee/gr/grlib.c" > "$out/grlib_body.inc"
grep -q "PointInsideColl" "$out/grlib_body.inc" || { echo "PointInsideColl not cut" >&2; exit 1; }
cd "$root"
CL="${CLANG:-/c/Program Files/LLVM/bin/clang.exe}"
F=(--target=x86_64-pc-windows-msvc -std=c99 -nostdinc -fno-builtin -DLINT -DTARGET_PC -fno-short-enums
   -fsigned-char -mlong-double-64 -fno-strict-aliasing -fwrapv -fcommon -fgnu89-inline -ftrivial-auto-var-init=zero -O2
   -Xclang -disable-llvm-passes -S -emit-llvm -w -I"$out")
"$CL" "${F[@]}" "$here/grlib_tu.c" -o "$out/grlib_tu.ll"
grep -c "fmuladd" "$out/grlib_tu.ll" || true
python "$root/rust/melee_runtime/oracle/lower.py" "$out/grlib_tu.ll" "$out/grlib_tu.lo.ll"
"$CL" -O2 -fno-builtin -ffp-contract=off -w "$here/grlib_ref.c" "$out/grlib_tu.lo.ll" -o "$out/grlib_ref.exe"
"$out/grlib_ref.exe" > "$here/../tests/data/grlib_ref.txt"
wc -l "$here/../tests/data/grlib_ref.txt"
