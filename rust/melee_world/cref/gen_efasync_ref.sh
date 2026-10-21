#!/usr/bin/env bash
# C reference for melee_world::tu::efasync (tests/tu_efasync.rs). Same recipe as
# melee_runtime/oracle/gen_tu_db_ref.sh: i686, contraction ON, gwtool fmuladd lowering via lower.py.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
lower="$here/../../melee_runtime/oracle/lower.py"
out="${TMPDIR:-${TEMP:-/tmp}}/melee_world_efasync"
mkdir -p "$out" "$here/../tests/data"
clang="${CLANG:-/c/Program Files/LLVM/bin/clang.exe}"
"$clang" --target=i686-pc-windows-msvc -O2 -Xclang -disable-llvm-passes -ffp-contract=on -ftrivial-auto-var-init=zero -fwrapv -S -emit-llvm -w "$here/efasync_ref.c" -o "$out/r.ll"
python "$lower" "$out/r.ll" "$out/r.lo.ll"
"$clang" --target=i686-pc-windows-msvc -O1 -msse2 -mfpmath=sse -w "$out/r.lo.ll" -o "$out/r.exe"
"$out/r.exe" | tr -d '\r' > "$here/../tests/data/tu_efasync_ref.txt"
wc -l "$here/../tests/data/tu_efasync_ref.txt"
