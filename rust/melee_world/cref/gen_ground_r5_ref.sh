#!/usr/bin/env bash
# Build the C reference for melee_world's night-3 round-5 ports (gr::ground: the map-gobj,
# x280 and x694 tables, Ground_MexParamStkind + Ground_801C2AE8, and the grRCruise / grBigBlue
# / grIceMt scroll getters). Regenerates tests/data/ground_r5_ref.txt.
#
# Bodies cut verbatim from the game C into ground_r5_body.inc, compiled in ground_r5_tu.c with
# the game's flags (clang, contraction ON), then gwtool's lowering redone by lower.py. Only the
# driver (ground_r5_ref.c) is compiled normally.
#
# Usage (Git Bash, from anywhere): bash rust/melee_world/cref/gen_ground_r5_ref.sh
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../../.." && pwd)"
out="${TMPDIR:-${TEMP:-/tmp}}/melee_world_cref_ground_r5"
mkdir -p "$out"
cut() { # file name
  awk -v n="$2" 'index($0, n"(") && $0 !~ /;/ && $0 ~ /^[A-Za-z]/ {p=1} p{print} p&&/^}/{exit}' "$root/src/melee/$1"
}
{
  grep '^static ssize_t const Gr_CObj_Max' "$root/src/melee/gr/ground.c"
  for f in Ground_MexParamStkind Ground_801C2AD8 Ground_801C2AE8 Ground_GetMapGObj Ground_801C2BBC \
           Ground_801C2BD4 Ground_801C2C8C Ground_801C2CF4 Ground_801C2D0C \
           Ground_801C20D0 Ground_801C205C Ground_801C2090 Ground_801C498C Ground_801C1D38; do cut gr/ground.c "$f"; done
  cut gr/grrcruise.c grRCruise_80201918
  cut gr/grbigblue.c grBigBlue_801EF7D8
  cut gr/gricemt.c grIceMt_801FA728
} > "$out/ground_r5_body.inc"
n=$(grep -c '^}' "$out/ground_r5_body.inc"); [ "$n" = 17 ] || { echo "cut $n of 17" >&2; exit 1; }
cd "$root"
CL="${CLANG:-/c/Program Files/LLVM/bin/clang.exe}"
F=(--target=x86_64-pc-windows-msvc -std=c99 -nostdinc -fno-builtin -DLINT -DTARGET_PC -fno-short-enums
   -fsigned-char -mlong-double-64 -fno-strict-aliasing -fwrapv -fcommon -fgnu89-inline -ftrivial-auto-var-init=zero -O2
   -Xclang -disable-llvm-passes -S -emit-llvm -w -Isrc -Isrc/melee -Ipc -Ipc/gameworld -isystem src/MSL
   -isystem libs/dolphin/include -isystem libs/dolphin/src -include src/MSL/math_ppc.h
   "-D_Static_assert(a,b)=" -Dsize_t=gc_size_t -I"$out")
"$CL" "${F[@]}" "$here/ground_r5_tu.c" -o "$out/tu.ll"
echo "fmuladd: $(grep -c 'fmuladd' "$out/tu.ll" || true)"
python "$root/rust/melee_runtime/oracle/lower.py" "$out/tu.ll" "$out/tu.lo.ll"
"$CL" -O2 -fno-builtin -ffp-contract=off -w -c "$out/tu.lo.ll" -o "$out/tu.o"
"$CL" -O2 -fno-builtin -w "$here/ground_r5_ref.c" "$out/tu.o" -o "$out/ground_r5_ref.exe"
"$out/ground_r5_ref.exe" | tr -d '\r' > "$here/../tests/data/ground_r5_ref.txt"
wc -l "$here/../tests/data/ground_r5_ref.txt"
