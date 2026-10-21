#!/usr/bin/env bash
# Build the C reference for melee_world's night-3 round-6 ports (hud::textlib: the DevText
# buffer/state functions of if/textlib.c, textlib_1.c `adjust`; gr::leaves grstadium_mul_vec).
# Regenerates tests/data/textlib_r6_ref.txt.
#
# Bodies cut verbatim into textlib_r6_body.inc, compiled in textlib_r6_tu.c with the game's
# flags (clang, contraction ON), then gwtool's lowering redone by lower.py. Only the driver
# (textlib_r6_ref.c) is compiled normally.
#
# Usage (Git Bash, from anywhere): bash rust/melee_world/cref/gen_textlib_r6_ref.sh
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../../.." && pwd)"
out="${TMPDIR:-${TEMP:-/tmp}}/melee_world_cref_textlib_r6"
mkdir -p "$out"
cut() { # file name
  awk -v n="$2" 'index($0, n"(") && $0 !~ /;/ && $0 ~ /^[A-Za-z]/ {p=1} p{print} p&&/^}/{exit}' "$root/src/melee/$1" \
    | sed -E '1s/^static (inline )?//'
}
{
  cut if/textdraw.c DevText_NumToStr
  for f in DevText_EraseFirstLine DevText_Clamp DevText_SetCursorXY DevText_SetCursorX DevText_HideCursor \
           DevText_80302AC0 DevText_ShowBackground DevText_HideBackground DevText_ShowText DevText_HideText \
           DevText_SetScale DevText_SetXY DevText_StoreColorIndex DevText_SetTextColor DevText_SetBGColor \
           DevText_Erase DevText_AdvanceLine; do cut if/textlib.c "$f"; done
  awk '/^typedef struct DevTextGlyph/ {p=1} p{print} p&&/^}/{exit}' "$root/src/melee/if/textlib.c"
  cut if/textlib.c DevText_Print
  cut if/textlib.c DevText_PrintInt
  cut if/textlib_1.c adjust
  cut gr/grpstadium.c mul_vec
} > "$out/textlib_r6_body.inc"
n=$(grep -c '^}' "$out/textlib_r6_body.inc"); [ "$n" = 23 ] || { echo "cut $n of 23" >&2; exit 1; }
cd "$root"
CL="${CLANG:-/c/Program Files/LLVM/bin/clang.exe}"
F=(--target=x86_64-pc-windows-msvc -std=c99 -nostdinc -fno-builtin -DLINT -DTARGET_PC -fno-short-enums
   -fsigned-char -mlong-double-64 -fno-strict-aliasing -fwrapv -fcommon -fgnu89-inline -ftrivial-auto-var-init=zero -O2
   -Xclang -disable-llvm-passes -S -emit-llvm -w -Isrc -Isrc/melee -Ipc -Ipc/gameworld -isystem src/MSL
   -isystem libs/dolphin/include -isystem libs/dolphin/src -include src/MSL/math_ppc.h
   "-D_Static_assert(a,b)=" -Dsize_t=gc_size_t -I"$out")
"$CL" "${F[@]}" "$here/textlib_r6_tu.c" -o "$out/tu.ll"
echo "fmuladd: $(grep -c 'fmuladd' "$out/tu.ll" || true)"
python "$root/rust/melee_runtime/oracle/lower.py" "$out/tu.ll" "$out/tu.lo.ll"
"$CL" -O2 -fno-builtin -ffp-contract=off -w -c "$out/tu.lo.ll" -o "$out/tu.o"
"$CL" -O2 -fno-builtin -w "$here/textlib_r6_ref.c" "$out/tu.o" -o "$out/textlib_r6_ref.exe"
"$out/textlib_r6_ref.exe" | tr -d '\r' > "$here/../tests/data/textlib_r6_ref.txt"
wc -l "$here/../tests/data/textlib_r6_ref.txt"
