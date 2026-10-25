#!/usr/bin/env bash
# Build the mods oracle (the shipping gw_mods.c linked unchanged + shim_dvd.c's overlay scan cut out
# at build time, against stubs) and run the random scenarios of mods_scenarios.py through it
# -> tests/data/mods_scenarios.txt, tests/data/mods_oracle.txt
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../../.." && pwd)"
out="$root/rust/target/mods-oracle"
CLANG="${CLANG:-clang}"
command -v "$CLANG" >/dev/null 2>&1 || CLANG="/c/Program Files/LLVM/bin/clang.exe"
mkdir -p "$out"
awk '/^#define GW_MOD_OFFSET_FLAG/{p=1} /^\/\* Mount the enabled mods/{p=0} p' "$root/pc/platform/shim_dvd.c" > "$out/overlay_cut.inc"
"$CLANG" -O1 -w -D_CRT_SECURE_NO_WARNINGS -I"$out" -I"$root/pc/platform" "$here/mods_oracle.c" -o "$out/mods_oracle.exe"
data="$here/../tests/data"
python "$here/mods_scenarios.py" "${MODS_SCENARIOS:-60}" | tr -d '\r' > "$data/mods_scenarios.txt"
: > "$data/mods_oracle.txt"
rm -rf "$out/sc"; mkdir -p "$out/sc"
rootw="$(cd "$out/sc" && pwd -W)"
name=""
flush() {
  if [ -n "$name" ]; then
    echo "== $name" >> "$data/mods_oracle.txt"
    rm -rf "$out/sc/root"; mkdir -p "$out/sc/root"
    ("$out/mods_oracle.exe" "$out/sc/script.txt" "$rootw/root" | tr -d '\r' | sed "s|$rootw/root|<R>|g") >> "$data/mods_oracle.txt" || true
  fi
}
while IFS= read -r line; do
  case "$line" in
    "=== "*) flush; name="${line#=== }"; : > "$out/sc/script.txt";;
    *) echo "$line" >> "$out/sc/script.txt";;
  esac
done < "$data/mods_scenarios.txt"
flush
echo "mods oracle: $(grep -c '^== ' "$data/mods_oracle.txt") scenarios, $(wc -l < "$data/mods_oracle.txt") lines"
