#!/bin/bash
# Build and run the C side of one TU state oracle (Git Bash, LLVM at C:\Program Files\LLVM, Windows
# Python with libclang==18.1.1):   rust/melee_modes/oracle/tuo/build.sh <tu stem>   (e.g. gmomake15)
# Writes rust/melee_modes/tests/data/tuo_<stem>.txt (+ .cfg) and tests/tuo_gen/<stem>_gen.rs.
set -e
HERE="$(cd "$(dirname "$0")" && pwd)"
WT="$(cd "$HERE/../../../.." && pwd)"
STEM="$1"
OUT="${OUT:-$WT/rust/target/tuo/$STEM}"
CL="${CLANG:-/c/Program Files/LLVM/bin/clang.exe}"
GALE="${GALE_INCLUDE:-$WT/../../melee/build/GALE01/include}"
cd "$WT"
python "$HERE/gen_tuo.py" "$HERE/specs/$STEM.py" "$OUT"
F=(--target=x86_64-pc-windows-gnu -mno-ms-bitfields -std=c99 -nostdinc -fno-builtin -DLINT -DTARGET_PC -fno-short-enums
   -fsigned-char -mlong-double-64 -fno-strict-aliasing -fwrapv -fcommon -fgnu89-inline
   -ftrivial-auto-var-init=zero -ffp-contract=on -O2 -Xclang -disable-llvm-passes -S -emit-llvm -w
   -I. -Isrc -Ipc -Ipc/gameworld -isystem src/MSL -isystem libs/dolphin/include -isystem libs/dolphin/src
   -isystem "$GALE" -include src/MSL/math_ppc.h "-D_Static_assert(a,b)=" -Dsize_t=gc_size_t)
"$CL" "${F[@]}" "$OUT/unit.c" -o "$OUT/unit.ll"
python "$HERE/guest_patch.py" "$OUT"
python "$WT/rust/melee_runtime/oracle/lower.py" "$OUT/unit.ll" "$OUT/unit.lo.ll"
"$CL" -O1 -w "$OUT/unit.lo.ll" "$OUT/stubs.c" "$HERE/tuo_rt.c" -o "$OUT/oracle.exe"
mkdir -p "$HERE/../../tests/data" "$HERE/../../tests/tuo_gen"
cp "$OUT/tuo.cfg" "$HERE/../../tests/data/tuo_$STEM.cfg"
cp "$OUT/${STEM}_gen.rs" "$HERE/../../tests/tuo_gen/${STEM}_gen.rs"
"$OUT/oracle.exe" "$OUT/tuo.cfg" > "$HERE/../../tests/data/tuo_$STEM.txt"
wc -l "$HERE/../../tests/data/tuo_$STEM.txt"
