#!/usr/bin/env bash
# Rebuild tests/c_oracle/expected.rs from oracle.c (Git Bash; needs LLVM clang + MSVC CRT).
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
wt="$(cd "$here/../../../.." && pwd)"
clang="${CLANG:-/c/Program Files/LLVM/bin/clang.exe}"
tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT
cd "$wt"
# Contraction ON + gwtool's lowering, as the shipping build (_build/masstest/pipe_win.sh) does.
"$clang" --target=x86_64-pc-windows-msvc -std=c99 -DLINT -DTARGET_PC -fno-short-enums   -fsigned-char -fwrapv -ffp-contract=on -ftrivial-auto-var-init=zero -O2 -Xclang -disable-llvm-passes   -S -emit-llvm -w '-D_Static_assert(a,b)='   -Dsize_t=gc_size_t -Isrc -Ipc -Ipc/gameworld -Ilibs/dolphin/include -Ilibs/dolphin/src -I"${GALE_INCLUDE:-$wt/../../melee/build/GALE01/include}"   "$here/oracle.c" -o "$tmp/oracle.ll"
python "$wt/rust/melee_runtime/oracle/lower.py" "$tmp/oracle.ll" "$tmp/oracle.lo.ll"
"$clang" --target=x86_64-pc-windows-msvc -O1 -w "$tmp/oracle.lo.ll" -o "$tmp/oracle.exe"
"$tmp/oracle.exe" > "$here/expected.rs"
echo "wrote $here/expected.rs"
