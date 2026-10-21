#!/usr/bin/env bash
# Regenerate src/gen/layout.rs from the group-5 headers (Git Bash, from anywhere).
# Needs clang (LLVM at C:\Program Files\LLVM) and python. Reads headers only; no disc data.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
wt="$(cd "$here/../../.." && pwd)"
clang="${CLANG:-/c/Program Files/LLVM/bin/clang.exe}"
gale="${GALE_INCLUDE:-$wt/../../melee/build/GALE01/include}"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
cd "$wt"
for f in $(find src/melee/it src/melee/ty src/melee/vi -name '*.h' ! -name '*.static.h' | sort); do
  echo "#include \"${f#src/}\""
done > "$tmp/probe.c"
flags=(--target=powerpc-none-eabi -std=c99 -nostdinc -fno-builtin -DLINT -DTARGET_PC
  -fno-short-enums -fsigned-char '-D_Static_assert(a,b)=' -Isrc -Ipc -Ipc/gameworld
  -isystem src/MSL -isystem libs/dolphin/include -isystem libs/dolphin/src -isystem "$gale"
  -include src/MSL/math_ppc.h -fsyntax-only -fno-color-diagnostics -w)
"$clang" "${flags[@]}" -Xclang -fdump-record-layouts-complete "$tmp/probe.c" > "$tmp/lay.txt"
"$clang" "${flags[@]}" -Xclang -ast-dump "$tmp/probe.c" | grep -E '^\|-(TypedefDecl|RecordDecl|EnumDecl)' > "$tmp/decls.txt"
python "$here/gen_layout.py" "$tmp/lay.txt" "$tmp/decls.txt" "$wt/rust/melee_items/src/gen/layout.rs"
