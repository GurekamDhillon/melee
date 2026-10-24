#!/bin/bash
# Regenerate tests/data/vectors_r10.txt (Git Bash, LLVM at C:\Program Files\LLVM, Windows Python with
# libclang==18.1.1). The cut game functions are compiled as the shipping build compiles them
# (contraction ON) and gwtool's IR rewrites are applied (melee_runtime/oracle/lower.py); only
# the dispatcher main.c is compiled normally.
set -e
HERE="$(cd "$(dirname "$0")" && pwd)"
WT="$(cd "$HERE/../../.." && pwd)"
OUT="${OUT:-$WT/rust/target/g6r10-oracle}"
CL="${CLANG:-/c/Program Files/LLVM/bin/clang.exe}"
GALE="${GALE_INCLUDE:-$WT/../../melee/build/GALE01/include}"
cd "$WT"
python "$HERE/gen_oracle_r10.py" "$OUT"
F=(--target=x86_64-pc-windows-gnu -mno-ms-bitfields -std=c99 -nostdinc -fno-builtin -DLINT -DTARGET_PC -fno-short-enums
   -fsigned-char -mlong-double-64 -fno-strict-aliasing -fwrapv -fcommon -fgnu89-inline
   -ftrivial-auto-var-init=zero -ffp-contract=on -O2 -Xclang -disable-llvm-passes -S -emit-llvm -w
   -I. -Isrc -Ipc -Ipc/gameworld -isystem src/MSL -isystem libs/dolphin/include -isystem libs/dolphin/src
   -isystem "$GALE" -include src/MSL/math_ppc.h -include "$WT/rust/target/native_inc/melee/if/types.h" -include "$WT/rust/target/native_inc/melee/ty/types.h" "-D_Static_assert(a,b)=" -Dsize_t=gc_size_t)
LL=()
for f in "$OUT"/o_*.c "$WT/pc/gameworld/gekko_fp.c"; do
  n=$(basename "$f" .c)
  "$CL" "${F[@]}" "$f" -o "$OUT/$n.ll"
  python "$WT/rust/melee_runtime/oracle/lower.py" "$OUT/$n.ll" "$OUT/$n.lo.ll"
  LL+=("$OUT/$n.lo.ll")
done
"$CL" -O1 -w "$OUT/main.c" "${LL[@]}" -o "$OUT/oracle.exe"
mkdir -p "$HERE/../tests/data"
"$OUT/oracle.exe" < "$OUT/inputs.txt" > "$HERE/../tests/data/vectors_r10.txt"
cp "$OUT/sigs.txt" "$HERE/../tests/data/sigs_r10.txt"
wc -l "$HERE/../tests/data/vectors_r10.txt"
