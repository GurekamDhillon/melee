#!/usr/bin/env bash
# Build and run the shim oracle (shim_oracle.cpp), writing tests/data/shim_oracle.txt.
#
# The functions under test are cut verbatim out of their C sources at build time, so the oracle
# cannot drift from them:
#   pc/platform/shim_gx.c          gw_xplat_GXProject (with its FP_CONTRACT pragmas), gw_GXInitFogAdjTable
#   extern/aurora/.../GXTexture.cpp GXGetTexBufferSize
#   extern/aurora/.../GXLighting.cpp GXInitLightSpot, GXInitLightDistAttn
# compiled the way the shipping build compiles native shims (tools/port/portlib.sh: clang
# --target=i686-pc-windows-msvc -O2, default -ffp-contract=on; i686 has no FMA). The two Aurora
# functions ship compiled by MSVC x86 instead; they are render-only (light coefficients, a size).
set -euo pipefail
command -v clang >/dev/null || export PATH="$PATH:/c/Program Files/LLVM/bin"
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../../.." && pwd)"
out="$root/rust/target/gx-shim-oracle"
mkdir -p "$out"
cut_fn() { # file, first-line regex: print from the match to the first line that is exactly "}"
  awk -v re="$2" 'f==0 && $0 ~ re {f=1} f==1 {print} f==1 && $0 == "}" {exit}' "$1"
}
shim="$root/pc/platform/shim_gx.c"
aur="$root/extern/aurora/lib/dolphin/gx"
{
  echo '#pragma STDC FP_CONTRACT OFF'
  cut_fn "$shim" '^void gw_xplat_GXProject[(]'
  echo '#pragma STDC FP_CONTRACT ON'
  cut_fn "$shim" '^void gw_GXInitFogAdjTable[(]'
} > "$out/shim_cut.inc"
{
  cut_fn "$aur/GXTexture.cpp" '^u32 GXGetTexBufferSize[(]'
  cut_fn "$aur/GXLighting.cpp" '^void GXInitLightSpot[(]'
  cut_fn "$aur/GXLighting.cpp" '^void GXInitLightDistAttn[(]'
} > "$out/aurora_cut.inc"
grep -q 'gw_xplat_GXProject' "$out/shim_cut.inc" && grep -q 'gw_GXInitFogAdjTable' "$out/shim_cut.inc"
grep -q 'GXInitLightDistAttn' "$out/aurora_cut.inc"
clang --target=i686-pc-windows-msvc -O2 -w -I"$out" -c "$here/shim_oracle_c.c" -o "$out/shim_oracle_c.o"
clang++ --target=i686-pc-windows-msvc -O2 -w -std=c++17 -I"$out" "$here/shim_oracle.cpp" "$out/shim_oracle_c.o" -o "$out/shim_oracle.exe"
"$out/shim_oracle.exe" | tr -d '\r' > "$here/../tests/data/shim_oracle.txt"
echo "wrote tests/data/shim_oracle.txt ($(wc -l < "$here/../tests/data/shim_oracle.txt") lines)"
