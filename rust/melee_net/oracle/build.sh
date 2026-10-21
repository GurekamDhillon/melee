#!/usr/bin/env bash
# Build and run the gw_net oracle (oracle.c), writing tests/data/net_oracle.txt.
# gw_net.c (+ gw_net.h, gw_matchbuild.h) is copied verbatim and compiled the way the shipping
# build compiles native shims (tools/port/portlib.sh: clang --target=i686-pc-windows-msvc -O2,
# clang's default -ffp-contract=on; i686 has no FMA, so the frame-advantage EMA a + (b - a) * k is
# a separate f32 multiply and add).
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../../.." && pwd)"
out="$root/rust/target/net-oracle"
CLANG="${CLANG:-clang}"
command -v "$CLANG" >/dev/null 2>&1 || CLANG="/c/Program Files/LLVM/bin/clang.exe"
mkdir -p "$out"
cp "$root/pc/platform/gw_net.c" "$root/pc/platform/gw_net.h" "$root/pc/platform/gw_matchbuild.h" "$out/"
"$CLANG" --target=i686-pc-windows-msvc -O2 -w -D_CRT_SECURE_NO_WARNINGS -I"$out" \
  "$here/oracle.c" -o "$out/net_oracle.exe" -lws2_32
"$out/net_oracle.exe" | tr -d '\r' > "$here/../tests/data/net_oracle.txt"
echo "wrote tests/data/net_oracle.txt ($(wc -l < "$here/../tests/data/net_oracle.txt") lines)"
# The real C peer for tests/interop.rs (Rust <-> C over UDP on 127.0.0.1).
"$CLANG" --target=i686-pc-windows-msvc -O2 -w -D_CRT_SECURE_NO_WARNINGS -I"$out" \
  "$here/net_peer.c" -o "$out/net_peer.exe" -lws2_32
echo "built $out/net_peer.exe"
