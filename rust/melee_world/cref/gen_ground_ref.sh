#!/usr/bin/env bash
# Build the C reference for melee_world::gr::ground; regenerates tests/data/ground_ref.txt.
#
# Ground_801C4B50 / 5630 / 5694 / 5AEC are cut verbatim from src/melee/gr/ground.c into
# ground_tu.c, which is compiled together with lbvector.c, lbtrigf.c, gekko_fp.c, mtx_pc.c and mtx44.c
# with the game's flags (as melee_runtime/oracle/build.sh does: clang, contraction ON like
# _build/masstest/pipe_win.sh), then gwtool's fmuladd lowering redone by lower.py. Only the
# driver (ground_ref.c) is compiled normally.
#
# Usage (Git Bash, from anywhere): bash rust/melee_world/cref/gen_ground_ref.sh
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../../.." && pwd)"
out="${TMPDIR:-${TEMP:-/tmp}}/melee_world_cref_ground"
mkdir -p "$out"
gr="$root/src/melee/gr/ground.c"
{
  awk '/^void Ground_801C4B50\(/{p=1} p{print} p&&/^}$/{exit}' "$gr"
  awk '/^void Ground_801C5630\(/{p=1} p{print} p&&/^}$/{exit}' "$gr"
  awk '/^void Ground_801C5694\(/{p=1} p{print} p&&/^}$/{exit}' "$gr"
  awk '/^void Ground_801C5AEC\(/{p=1} p{print} p&&/^}$/{exit}' "$gr"
} > "$out/ground_body.inc"
for fn in 801C4B50 801C5630 801C5694 801C5AEC; do
  grep -q "^void Ground_$fn(" "$out/ground_body.inc" || { echo "$fn not cut" >&2; exit 1; }
done
cd "$root"
CL="${CLANG:-/c/Program Files/LLVM/bin/clang.exe}"
F=(--target=x86_64-pc-windows-msvc -std=c99 -nostdinc -fno-builtin -DLINT -DTARGET_PC -fno-short-enums
   -fsigned-char -mlong-double-64 -fno-strict-aliasing -fwrapv -fcommon -fgnu89-inline -ftrivial-auto-var-init=zero -O2
   -Xclang -disable-llvm-passes -S -emit-llvm -w -Isrc -Isrc/melee -Ipc -Ipc/gameworld -isystem src/MSL
   -isystem libs/dolphin/include -isystem libs/dolphin/src -include src/MSL/math_ppc.h
   "-D_Static_assert(a,b)=" -Dsize_t=gc_size_t -I"$out")
LL=()
for f in pc/gameworld/mtx_pc.c pc/gameworld/gekko_fp.c libs/dolphin/src/dolphin/mtx/mtx44.c src/melee/lb/lbtrigf.c src/melee/lb/lbvector.c \
         rust/melee_runtime/oracle/wrap.c "$here/ground_tu.c"; do
  n=$(basename "$f" .c)
  "$CL" "${F[@]}" "$f" -o "$out/$n.ll"
  python "$root/rust/melee_runtime/oracle/lower.py" "$out/$n.ll" "$out/$n.lo.ll"
  LL+=("$out/$n.lo.ll")
done
"$CL" -O2 -fno-builtin -ffp-contract=off -w "$here/ground_ref.c" "${LL[@]}" -o "$out/ground_ref.exe"
"$out/ground_ref.exe" > "$here/../tests/data/ground_ref.txt"
wc -l "$here/../tests/data/ground_ref.txt"
