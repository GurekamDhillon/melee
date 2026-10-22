#!/bin/bash
# Build the C side of the shard-497-504 state oracle and write tests/data/s497_<tu>_exp.txt.
#
#   bash rust/melee_modes/oracle/s497/build.sh [tu...]       (Git Bash, from anywhere in the worktree)
#
# Needs LLVM at C:\Program Files\LLVM (CLANG overrides), Windows Python with libclang==18.1.1, and
# the GALE01 include dir (GALE_INCLUDE overrides). The game's own C file is compiled as the shipping
# build compiles it (clang default -ffp-contract=on, -O2 with the LLVM passes off, TARGET_PC) and
# gwtool's IR rewrites (fmuladd lowering, saturating float->int, oversize shifts) are applied by
# melee_runtime/oracle/lower.py; only rt.c (the scenario interpreter) is compiled normally.
set -eo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
WT="$(cd "$HERE/../../../.." && pwd)"
OUT="${OUT:-$WT/rust/target/s497}"
CL="${CLANG:-/c/Program Files/LLVM/bin/clang.exe}"
GALE="${GALE_INCLUDE:-$WT/../../melee/build/GALE01/include}"
cd "$WT"
TUS=("$@")
[ ${#TUS[@]} -eq 0 ] && TUS=(gmtiny gmsupersudden gmsinglebutton gmslomo gmstamina gmscdata gmscene gmstaffroll)

F=(--target=x86_64-pc-windows-gnu -mno-ms-bitfields -std=c99 -nostdinc -fno-builtin -DLINT -DTARGET_PC -fno-short-enums
   -fsigned-char -mlong-double-64 -fno-strict-aliasing -fwrapv -fcommon -fgnu89-inline
   -ftrivial-auto-var-init=zero -ffp-contract=on -O2 -Xclang -disable-llvm-passes -S -emit-llvm -w
   -I. -Isrc -Ipc -Ipc/gameworld -Isrc/melee/gm -I"$HERE" -isystem src/MSL -isystem libs/dolphin/include -isystem libs/dolphin/src
   -isystem "$GALE" -include src/MSL/math_ppc.h "-D_Static_assert(a,b)=" -Dsize_t=gc_size_t)

mkdir -p "$OUT/common"
# shared game C the harnesses link: the Gekko paired-single / tanf / frsqrte emulation and the matrix routines
for f in pc/gameworld/gekko_fp pc/gameworld/mtx_pc; do
  n=$(basename "$f")
  if [ ! -f "$OUT/common/$n.lo.ll" ] || [ "$f.c" -nt "$OUT/common/$n.lo.ll" ]; then
    "$CL" "${F[@]}" "$f.c" -o "$OUT/common/$n.ll"
    python "$WT/rust/melee_runtime/oracle/lower.py" "$OUT/common/$n.ll" "$OUT/common/$n.lo.ll"
  fi
done

for tu in "${TUS[@]}"; do
  python "$HERE/gen.py" oracle "$tu"
  d="$OUT/$tu"
  "$CL" "${F[@]}" "$d/tu_part.c" -o "$d/tu_part.ll"
  python "$WT/rust/melee_runtime/oracle/lower.py" "$d/tu_part.ll" "$d/tu_part.lo.ll"
  "$CL" -O1 -w -ffp-contract=off "$HERE/rt.c" "$d/tu_part.lo.ll" "$OUT/common/gekko_fp.lo.ll" "$OUT/common/mtx_pc.lo.ll" -o "$d/oracle.exe"
  "$d/oracle.exe" "$WT/rust/melee_modes/tests/data/s497_${tu}_scen.txt" | tr -d '\r' > "$WT/rust/melee_modes/tests/data/s497_${tu}_exp.txt"
  wc -l "$WT/rust/melee_modes/tests/data/s497_${tu}_exp.txt"
done
