#!/usr/bin/env bash
# Rebuild tests/c_oracle/expected.rs from oracle.c (Git Bash; needs LLVM clang + MSVC CRT).
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
wt="$(cd "$here/../../../.." && pwd)"
clang="${CLANG:-/c/Program Files/LLVM/bin/clang.exe}"
tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT
cd "$wt"
"$clang" --target=x86_64-pc-windows-msvc -std=c99 -DLINT -DTARGET_PC -fno-short-enums \
  -fsigned-char -fwrapv -ffp-contract=off -fno-fast-math -O1 -w '-D_Static_assert(a,b)=' \
  -Dsize_t=gc_size_t -Isrc -Ipc -Ipc/gameworld -Ilibs/dolphin/include -Ilibs/dolphin/src -I"${GALE_INCLUDE:-$wt/../../melee/build/GALE01/include}" \
  "$here/oracle.c" -o "$tmp/oracle.exe"
"$tmp/oracle.exe" > "$here/expected.rs"
echo "wrote $here/expected.rs"
