#!/usr/bin/env bash
# Build and run the PAD oracle (oracle.cpp), writing tests/data/pad_oracle.txt.
# Needs clang++ (C:\Program Files\LLVM). Compiled -O2 with clang's default -ffp-contract=on, as the
# shipping build compiles (there are no float sites in these functions anyway).
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../../.." && pwd)"
out="$root/rust/target/pad-oracle"
mkdir -p "$out"
pad="$root/extern/aurora/lib/dolphin/pad/pad.cpp"
{
  echo "// cut from extern/aurora/lib/dolphin/pad/pad.cpp by build.sh"
  sed -n '/^struct PADCLampRegion {/,/^};/p' "$pad"
  sed -n '/^constexpr PADCLampRegion ClampRegion{/,/^};/p' "$pad"
  sed -n '/^void ClampTrigger(/,/^void PADGetVidPid(/p' "$pad" | sed '$d'
} > "$out/aurora_clamp.inc"
clang -O2 -w -I"$root/pc/platform" -I"$here" -c "$here/oracle_decode.c" -o "$out/oracle_decode.o"
clang++ -std=c++20 -O2 -w -I"$here" -I"$out" "$here/oracle.cpp" "$out/oracle_decode.o" -o "$out/pad_oracle.exe"
"$out/pad_oracle.exe" > "$here/../tests/data/pad_oracle.txt"
echo "wrote tests/data/pad_oracle.txt"
