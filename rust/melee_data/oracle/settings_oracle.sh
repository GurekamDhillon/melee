#!/usr/bin/env bash
# Build the gw_settings.c oracle (table/env/load/save/summary cut out of the shipping file) and run
# settings_script.txt through it -> ../tests/data/settings_oracle.txt
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../../.." && pwd)"
out="$root/rust/target/settings-oracle"
CLANG="${CLANG:-clang}"
command -v "$CLANG" >/dev/null 2>&1 || CLANG="/c/Program Files/LLVM/bin/clang.exe"
mkdir -p "$out"
src="$root/pc/platform/gw_settings.c"
{
  awk '/^#define ST_MAX/{p=1} /^\/\* Widescreen/{p=0} p' "$src"
  awk '/^\/\* Every saved key/{p=1} /^\/\* Typing into/{p=0} p' "$src"
} > "$out/settings_cut.inc"
python "$here/settings_script.py" > "$here/../tests/data/settings_script.txt"
"$CLANG" -O1 -w -D_CRT_SECURE_NO_WARNINGS -I"$out" "$here/settings_oracle.c" -o "$out/settings_oracle.exe"
"$out/settings_oracle.exe" "$here/../tests/data/settings_script.txt" "$out/oracle.cfg" | tr -d '\r' > "$here/../tests/data/settings_oracle.txt"
echo "wrote tests/data/settings_oracle.txt ($(wc -l < "$here/../tests/data/settings_oracle.txt") lines)"
