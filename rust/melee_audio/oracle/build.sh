#!/usr/bin/env bash
# Build and run the AX oracle (oracle.c), writing tests/data/ax_oracle.txt.
# shim_ax.c is copied verbatim and compiled the way the shipping build compiles native shims
# (tools/port/portlib.sh: clang --target=i686-pc-windows-msvc -O2, clang's default
# -ffp-contract=on; i686 has no FMA, so a*b+c in the AXFX reverb stays a separate mul and add).
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../../.." && pwd)"
out="$root/rust/target/ax-oracle"
mkdir -p "$out"
cp "$root/pc/platform/shim_ax.c" "$out/shim_ax.c"
clang --target=i686-pc-windows-msvc -O2 -w -DTARGET_PC -I"$here/fake" -I"$out" \
  "$here/oracle.c" -o "$out/ax_oracle.exe"
"$out/ax_oracle.exe" | tr -d '\r' > "$here/../tests/data/ax_oracle.txt"
echo "wrote tests/data/ax_oracle.txt ($(wc -l < "$here/../tests/data/ax_oracle.txt") lines)"
