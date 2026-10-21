#!/usr/bin/env bash
# Regenerate src/types_gen.rs from src/melee/ft/kinds/*/types.h (run from Git Bash, any cwd).
#   ppc32 record layouts  -> GameCube offsets (no TARGET_PC: the original game layout)
#   native x86_64 AST     -> native field set and C types (TARGET_PC, as the PC build sees it)
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
wt="$(cd "$here/../../.." && pwd)"
CLANG="${CLANG:-/c/Program Files/LLVM/bin/clang}"
tmp="${TMPDIR:-/tmp}/melee_chara_gen"; mkdir -p "$tmp"
cd "$wt"
for f in src/melee/ft/kinds/*/types.h; do echo "#include <${f#src/}>"; done > "$tmp/all_types.c"
common=(-std=c99 -nostdinc -fno-builtin -DLINT -fno-short-enums -fsigned-char -Isrc -Ipc
        -isystem src/MSL -isystem libs/dolphin/include -fsyntax-only -w)
"$CLANG" --target=powerpc-none-eabi "${common[@]}" -Xclang -fdump-record-layouts-complete \
  "$tmp/all_types.c" > "$tmp/gc_layout.txt"
"$CLANG" --target=x86_64-pc-windows-msvc "${common[@]}" -DTARGET_PC "-D_Static_assert(a,b)=" \
  -Dsize_t=gc_size_t -Ipc/gameworld -isystem tools/rust-port/include -Xclang -ast-dump=json \
  "$tmp/all_types.c" > "$tmp/ast.json"
python "$here/gen_types.py" "$tmp/ast.json" "$tmp/gc_layout.txt" "$here/../src/types_gen.rs"
