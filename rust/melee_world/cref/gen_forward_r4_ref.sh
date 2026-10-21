#!/usr/bin/env bash
# Build the C reference for melee_world's night-3 round-4 ports: the single-call gr forwarders
# (gr/forward.rs, regenerated here by gen_forward_r4.py). Regenerates
# tests/data/forward_r4_ref.txt (one call trace per function per map_id).
#
# Bodies and the gr/inlines.h inlines are cut verbatim from the game C, compiled in
# forward_r4_tu.c with the game's flags (clang, contraction ON), then gwtool's lowering redone
# by lower.py. Only the driver/recorders are compiled normally.
#
# Usage (Git Bash, from anywhere): bash rust/melee_world/cref/gen_forward_r4_ref.sh
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../../.." && pwd)"
out="${TMPDIR:-${TEMP:-/tmp}}/melee_world_cref_forward_r4"
mkdir -p "$out"
python "$here/gen_forward_r4.py" "$root" "$out"
cut() { # file name
  awk -v n="$2" 'index($0, n"(") && $0 !~ /;/ && $0 ~ /^[A-Za-z]/ {p=1} p{print} p&&/^}/{exit}' "$root/src/melee/$1"
}
{
  cut gr/inlines.h Ground_StartMapAnim
  cut gr/inlines.h Ground_InitMapCollAndAnim
  cut gr/inlines.h Ground_UpdateWindAndMapColl
} > "$out/forward_r4_inl.inc"
n=$(grep -c '^}' "$out/forward_r4_inl.inc"); [ "$n" = 3 ] || { echo "cut $n of 3" >&2; exit 1; }
cp "$here/forward_r4_harness.h" "$out/"
cd "$root"
CL="${CLANG:-/c/Program Files/LLVM/bin/clang.exe}"
F=(--target=x86_64-pc-windows-msvc -std=c99 -nostdinc -fno-builtin -DLINT -DTARGET_PC -fno-short-enums
   -fsigned-char -mlong-double-64 -fno-strict-aliasing -fwrapv -fcommon -fgnu89-inline -ftrivial-auto-var-init=zero -O2
   -Xclang -disable-llvm-passes -S -emit-llvm -w -Isrc -Isrc/melee -Ipc -Ipc/gameworld -isystem src/MSL
   -isystem libs/dolphin/include -isystem libs/dolphin/src -include src/MSL/math_ppc.h
   "-D_Static_assert(a,b)=" -Dsize_t=gc_size_t -I"$out")
"$CL" "${F[@]}" "$here/forward_r4_tu.c" -o "$out/tu.ll"
grep -c 'fmuladd' "$out/tu.ll" || true
python "$root/rust/melee_runtime/oracle/lower.py" "$out/tu.ll" "$out/tu.lo.ll"
"$CL" -O2 -fno-builtin -ffp-contract=off -w -c "$out/tu.lo.ll" -o "$out/tu.o"
"$CL" -O2 -fno-builtin -w "$out/forward_r4_driver.c" "$out/tu.o" -o "$out/forward_r4_ref.exe"
"$out/forward_r4_ref.exe" | tr -d '\r' > "$here/../tests/data/forward_r4_ref.txt"
wc -l "$here/../tests/data/forward_r4_ref.txt"
