#!/usr/bin/env bash
# The gw_rollback.c oracle: cuts rb_hash, rb_neutral_raw, rb_gen and rb_arrival out of
# pc/platform/gw_rollback.c at build time (no copy edited by hand), compiles them the way shipping
# shims are built (clang --target=i686-pc-windows-msvc -O2) against a minimal `rb`, and writes
# tests/data/rb_oracle.txt: one line per case, compared by tests/session_oracle.rs.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../../.." && pwd)"
out="$root/rust/target/rb-oracle"
CLANG="${CLANG:-clang}"
command -v "$CLANG" >/dev/null 2>&1 || CLANG="/c/Program Files/LLVM/bin/clang.exe"
mkdir -p "$out"
src="$root/pc/platform/gw_rollback.c"
cut() { # the definition starting at the line matching $1, through its closing brace at column 0
  awk -v pat="$1" 'index($0, pat) == 1 && !/;$/ {on=1} on {print} on && /^}/ {exit}' "$src"
}
{
  echo '#include <stdint.h>'
  echo '#include <string.h>'
  echo '#include "rb_oracle_pre.h"'
  cut 'static unsigned rb_hash(unsigned a, unsigned b, unsigned c) {'
  cut 'static void rb_neutral_raw(GwRbInput *o) {'
  cut 'static void rb_gen(int slot, int frame, GwRbInput *o) {'
  cut 'static long rb_arrival(int slot, int frame) {'
} > "$out/rb_cut.c"
sed -n '/^typedef struct GwRbInput {/,/^} GwRbInput;/p' "$root/pc/platform/gw_rollback.h" > "$out/gwrbinput.h"
"$CLANG" --target=i686-pc-windows-msvc -O2 -w -D_CRT_SECURE_NO_WARNINGS -I"$out" -I"$here" \
  "$here/rb_oracle.c" -o "$out/rb_oracle.exe"
"$out/rb_oracle.exe" | tr -d '\r' > "$here/../tests/data/rb_oracle.txt"
echo "wrote tests/data/rb_oracle.txt ($(wc -l < "$here/../tests/data/rb_oracle.txt") lines)"
