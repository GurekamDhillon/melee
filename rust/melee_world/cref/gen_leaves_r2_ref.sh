#!/usr/bin/env bash
# Build the C reference for melee_world's night-3 round-2 leaves (gr/leaves.rs, hud/leaves.rs);
# regenerates tests/data/leaves_r2_ref.txt.
#
# The bodies are cut verbatim from the game C into leaves_r2_body.inc (`static`/`inline`
# dropped), compiled in leaves_r2_tu.c with the game's flags (clang, contraction ON), then
# gwtool's fmuladd / float->int lowering redone by lower.py. Only the driver
# (leaves_r2_ref.c) is compiled normally.
#
# Usage (Git Bash, from anywhere): bash rust/melee_world/cref/gen_leaves_r2_ref.sh
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../../.." && pwd)"
out="${TMPDIR:-${TEMP:-/tmp}}/melee_world_cref_leaves_r2"
mkdir -p "$out"
cut() { # file name
  awk -v n="$2" 'index($0, n"(") && $0 !~ /;/ && $0 ~ /^[A-Za-z]/ {p=1} p{print} p&&/^}/{exit}' "$root/src/melee/$1" \
    | sed -E '1s/^static (inline )?//'
}
{
  cut gr/grgreens.c grGreens_80213AB4
  cut gr/groldpupupu.c grOldPupupu_8021128C
  cut gr/grpstadium.c grStadium_ScaleColor
  cut gr/grmutecity.c grMc_DistanceSquared
  cut gr/grzebes.c grZebes_Subtract
  cut gr/grinishie1.c fabsf_inline
  cut gr/grkongo.c grKongo_801D8270
  cut gr/groldkongo.c grOldKongo_802105AC
  cut if/textdraw.c DevText_StrLen
  cut if/textdraw.c DevText_NumToStr
  cut if/textlib.c DevText_Clamp
  cut if/ifstock.c ifStock_802F89F8_count
  cut if/ifstock.c ifStock_802F89F8_divisor
  cut if/ifstock.c ifStock_802F89F8_digit
  cut if/ifnametag.c inlineA1
  cut if/ifstatus.c foo
} > "$out/leaves_r2_body.inc"
n=$(grep -c '^}' "$out/leaves_r2_body.inc"); [ "$n" = 16 ] || { echo "cut $n of 16" >&2; exit 1; }
cd "$root"
CL="${CLANG:-/c/Program Files/LLVM/bin/clang.exe}"
F=(--target=x86_64-pc-windows-msvc -std=c99 -nostdinc -fno-builtin -DLINT -DTARGET_PC -fno-short-enums
   -fsigned-char -mlong-double-64 -fno-strict-aliasing -fwrapv -fcommon -fgnu89-inline -ftrivial-auto-var-init=zero -O2
   -Xclang -disable-llvm-passes -S -emit-llvm -w -Isrc -Isrc/melee -Ipc -Ipc/gameworld -isystem src/MSL
   -isystem libs/dolphin/include -isystem libs/dolphin/src -include src/MSL/math_ppc.h
   "-D_Static_assert(a,b)=" -Dsize_t=gc_size_t -I"$out")
"$CL" "${F[@]}" "$here/leaves_r2_tu.c" -o "$out/tu.ll"
grep -c 'fmuladd' "$out/tu.ll" || true
python "$root/rust/melee_runtime/oracle/lower.py" "$out/tu.ll" "$out/tu.lo.ll"
"$CL" -O2 -fno-builtin -ffp-contract=off -w "$here/leaves_r2_ref.c" "$out/tu.lo.ll" -o "$out/leaves_r2_ref.exe"
"$out/leaves_r2_ref.exe" > "$here/../tests/data/leaves_r2_ref.txt"
wc -l "$here/../tests/data/leaves_r2_ref.txt"
