#!/bin/bash
set -e
HERE="$(cd "$(dirname "$0")" && pwd)"
OUT="$HERE/../../target/g1-oracle-lobj-complete"
mkdir -p "$OUT"
cd "$HERE/../../.."
CL="${CLANG:-/c/Program Files/LLVM/bin/clang.exe}"
F=(--target=x86_64-pc-windows-msvc -std=c99 -nostdinc -fno-builtin -DLINT -DTARGET_PC -fno-short-enums -fsigned-char -mlong-double-64 -fno-strict-aliasing -fwrapv -fcommon -fgnu89-inline -ftrivial-auto-var-init=zero -ffp-contract=on -O2 -Xclang -disable-llvm-passes -S -emit-llvm -w -Isrc -Ipc -Ipc/gameworld -isystem src/MSL -isystem libs/dolphin/include -isystem libs/dolphin/src -include src/MSL/math_ppc.h '-D_Static_assert(a,b)=' -Dsize_t=gc_size_t)
LL=()
for f in "$HERE/helper_lobj_complete.c" pc/gameworld/{mtx_pc,gekko_fp}.c; do
 n=$(basename "$f" .c)
 "$CL" "${F[@]}" "$f" -o "$OUT/$n.ll"
 python "$HERE/lower.py" "$OUT/$n.ll" "$OUT/$n.lo.ll"
 LL+=("$OUT/$n.lo.ll")
done
"$CL" -O2 -ffp-contract=off -w "$HERE/driver_lobj_complete.c" "${LL[@]}" -o "$OUT/oracle.exe"
"$OUT/oracle.exe" "$HERE/../tests/data/oracle_lobj_complete.txt"
