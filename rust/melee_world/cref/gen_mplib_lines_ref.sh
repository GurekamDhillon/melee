#!/usr/bin/env bash
# Build the C reference for melee_world::mp::lines and regenerate tests/data/mplib_lines_ref.txt.
#
# The function bodies are cut out of src/melee/mp/mplib.c verbatim (between marker lines), so
# the reference is the real C, compiled natively with clang. It is built for i686 because the
# C does pointer arithmetic that assumes 4-byte pointers ((int) groundCollLine + offset, and
# flags_base[id * 2] over CollLine). SSE math, contraction ON like the shipping build
# (_build/masstest/pipe_win.sh): emitted as IR, then gwtool's fmuladd lowering is redone by
# melee_runtime/oracle/lower.py (f32 fmuladd -> f64 multiply-add, one f32 rounding).
#
# Usage (Git Bash, from anywhere): bash rust/melee_world/cref/gen_mplib_lines_ref.sh
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../../.." && pwd)"
out="${TMPDIR:-${TEMP:-/tmp}}/melee_world_cref"
mkdir -p "$out"
src="$root/src/melee/mp/mplib.c"

{
  awk '/^int mpLineGetNext\(int line_id\)$/{p=1} /^\/\/\/ remap point/{p=0} p' "$src"
  awk '/^static inline int mpLineGetNextCheckInline/{p=1} /^\/\/\/ what even is this lol/{p=0} p' "$src"
} > "$out/mplib_lines_body.inc"
[ -s "$out/mplib_lines_body.inc" ] || { echo "marker extraction failed" >&2; exit 1; }

clang="${CLANG:-/c/Program Files/LLVM/bin/clang.exe}"
"$clang" --target=i686-pc-windows-msvc -O2 -Xclang -disable-llvm-passes -ffp-contract=on   -ftrivial-auto-var-init=zero -S -emit-llvm -w -I"$out" -I"$root/pc/gameworld"   "$here/mplib_lines_ref.c" -o "$out/mplib_lines_ref.ll"
python "$root/rust/melee_runtime/oracle/lower.py" "$out/mplib_lines_ref.ll" "$out/mplib_lines_ref.lo.ll"
"$clang" --target=i686-pc-windows-msvc -O1 -msse2 -mfpmath=sse -w   "$out/mplib_lines_ref.lo.ll" "$root/pc/gameworld/gekko_fp.c" -o "$out/mplib_lines_ref.exe"
"$out/mplib_lines_ref.exe" > "$here/../tests/data/mplib_lines_ref.txt"
wc -l "$here/../tests/data/mplib_lines_ref.txt"
