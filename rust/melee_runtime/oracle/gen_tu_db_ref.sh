#!/usr/bin/env bash
# C reference for melee_runtime::tu::db* (tests/tu_db_pure.rs). Same recipe as
# melee_world/cref/gen_camera_ref.sh: i686, contraction ON, gwtool fmuladd lowering via lower.py,
# tanf from pc/gameworld/gekko_fp.c.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../../.." && pwd)"
out="${TMPDIR:-${TEMP:-/tmp}}/melee_runtime_tu_db"
mkdir -p "$out"
clang="${CLANG:-/c/Program Files/LLVM/bin/clang.exe}"
"$clang" --target=i686-pc-windows-msvc -O2 -Xclang -disable-llvm-passes -ffp-contract=on -ftrivial-auto-var-init=zero -fwrapv -S -emit-llvm -w "$here/tu_db_ref.c" -o "$out/r.ll"
python "$here/lower.py" "$out/r.ll" "$out/r.lo.ll"
"$clang" --target=i686-pc-windows-msvc -O1 -msse2 -mfpmath=sse -w "$out/r.lo.ll" "$root/pc/gameworld/gekko_fp.c" -o "$out/r.exe"
"$out/r.exe" | tr -d '\r' > "$here/../tests/data/tu_db_ref.txt"
wc -l "$here/../tests/data/tu_db_ref.txt"
