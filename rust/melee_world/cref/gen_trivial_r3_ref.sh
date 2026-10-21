#!/usr/bin/env bash
# Build the C reference for melee_world's night-3 round-3 ports: the 568 constant-return /
# empty gr callbacks (gr/trivial.rs, regenerated here by gen_trivial_r3.py) and the three
# efLib list walkers (ef/eflib.rs). Regenerates tests/data/trivial_r3_ref.txt and
# tests/data/eflib_r3_ref.txt.
#
# Bodies are cut verbatim from the game C (static/inline dropped), compiled in
# trivial_r3_tu.c with the game's flags (clang, contraction ON), then gwtool's lowering redone
# by lower.py. Only the drivers are compiled normally.
#
# Usage (Git Bash, from anywhere): bash rust/melee_world/cref/gen_trivial_r3_ref.sh
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../../.." && pwd)"
out="${TMPDIR:-${TEMP:-/tmp}}/melee_world_cref_trivial_r3"
mkdir -p "$out"
python "$here/gen_trivial_r3.py" "$root" "$out"
cut() { # file name
  awk -v n="$2" 'index($0, n"(") && $0 !~ /;/ && $0 ~ /^[A-Za-z]/ {p=1} p{print} p&&/^}/{exit}' "$root/src/melee/$1" \
    | sed -E '1s/^static (inline )?//'
}
{
  cut ef/eflib.c efLib_SetFlags
  cut ef/eflib.c efLib_PauseAll
  cut ef/eflib.c efLib_ResumeAll
} > "$out/eflib_r3_body.inc"
n=$(grep -c '^}' "$out/eflib_r3_body.inc"); [ "$n" = 3 ] || { echo "cut $n of 3" >&2; exit 1; }
cd "$root"
CL="${CLANG:-/c/Program Files/LLVM/bin/clang.exe}"
F=(--target=x86_64-pc-windows-msvc -std=c99 -nostdinc -fno-builtin -DLINT -DTARGET_PC -fno-short-enums
   -fsigned-char -mlong-double-64 -fno-strict-aliasing -fwrapv -fcommon -fgnu89-inline -ftrivial-auto-var-init=zero -O2
   -Xclang -disable-llvm-passes -S -emit-llvm -w -Isrc -Isrc/melee -Ipc -Ipc/gameworld -isystem src/MSL
   -isystem libs/dolphin/include -isystem libs/dolphin/src -include src/MSL/math_ppc.h
   "-D_Static_assert(a,b)=" -Dsize_t=gc_size_t -I"$out")
"$CL" "${F[@]}" "$here/trivial_r3_tu.c" -o "$out/tu.ll"
grep -c 'fmuladd' "$out/tu.ll" || true
python "$root/rust/melee_runtime/oracle/lower.py" "$out/tu.ll" "$out/tu.lo.ll"
"$CL" -O2 -fno-builtin -ffp-contract=off -w -c "$out/tu.lo.ll" -o "$out/tu.o"
"$CL" -O2 -fno-builtin -w "$out/trivial_r3_driver.c" "$out/tu.o" -o "$out/trivial_r3_ref.exe"
"$CL" -O2 -fno-builtin -w "$here/eflib_r3_ref.c" "$out/tu.o" -o "$out/eflib_r3_ref.exe"
"$out/trivial_r3_ref.exe" > "$here/../tests/data/trivial_r3_ref.txt"
"$out/eflib_r3_ref.exe" > "$here/../tests/data/eflib_r3_ref.txt"
wc -l "$here/../tests/data/trivial_r3_ref.txt" "$here/../tests/data/eflib_r3_ref.txt"
