#!/usr/bin/env bash
# Build the player oracle (the shipping gw_replay.c + gw_slippi_pad.c, linked unchanged against stubs)
# and run every case of tests/data/player_cases.txt -> tests/data/player_oracle.txt.
# Needs oracle/replay_oracle.sh run first (it makes the real-replay list and the synthetic corpus).
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../../.." && pwd)"
out="$root/rust/target/player-oracle"
CLANG="${CLANG:-clang}"
command -v "$CLANG" >/dev/null 2>&1 || CLANG="/c/Program Files/LLVM/bin/clang.exe"
mkdir -p "$out"
"$CLANG" -O1 -w -D_CRT_SECURE_NO_WARNINGS -I"$root/pc/platform" "$here/player_oracle.c" "$root/pc/platform/gw_slippi_pad.c" -o "$out/player_oracle.exe"
data="$here/../tests/data"
python "$here/player_cases.py" "$data/slp" "$root/rust/target/replay-oracle/real_list.txt" | tr -d '\r' > "$data/player_cases.txt"
REAL="${REAL_SLP_DIR:-$root/../../Gang-Steals}"
synth="$(cd "$data/slp" && pwd -W)"
realw="$(cd "$REAL" && pwd -W)"
: > "$data/player_oracle.txt"
n=0
while IFS='|' read -r b name flags; do
  if [ "$b" = r ]; then base="$realw"; else base="$synth"; fi
  rm -rf "$out/case"; mkdir -p "$out/case"
  echo "== $b|$name|$flags" >> "$data/player_oracle.txt"
  (cd "$out/case" && "$out/player_oracle.exe" "$base/$name" . "$flags") | tr -d '\r' | sed "s|$base|<S>|g" >> "$data/player_oracle.txt" || true
  n=$((n + 1))
done < "$data/player_cases.txt"
echo "ran $n cases -> $(wc -l < "$data/player_oracle.txt") lines"
