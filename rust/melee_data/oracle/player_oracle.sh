#!/usr/bin/env bash
# Build the player oracle (the shipping gw_replay.c + gw_slippi_pad.c linked unchanged, plus the
# container half of gw_script_rec.inc cut out at build time, against stubs) and run every case ->
# tests/data/player_oracle.txt. Stage 1 runs the s/r/G cases (G = whole-match .gdlp recordings, whose
# files are kept in tests/data/gdlp); stage 2 makes damaged copies of those and plays them (g cases).
# Needs oracle/replay_oracle.sh run first (it makes the real-replay list and the synthetic corpus).
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../../.." && pwd)"
out="$root/rust/target/player-oracle"
CLANG="${CLANG:-clang}"
command -v "$CLANG" >/dev/null 2>&1 || CLANG="/c/Program Files/LLVM/bin/clang.exe"
mkdir -p "$out"
awk '/^\/\* ---- recording ---/{p=1} !p' "$root/pc/platform/gw_script_rec.inc" > "$out/script_rec_cut.inc"
"$CLANG" -O1 -w -D_CRT_SECURE_NO_WARNINGS -I"$out" -I"$root/pc/platform" "$here/player_oracle.c" "$root/pc/platform/gw_slippi_pad.c" -o "$out/player_oracle.exe"
data="$here/../tests/data"
REAL="${REAL_SLP_DIR:-$root/../../Gang-Steals}"
synth="$(cd "$data/slp" && pwd -W)"
realw="$(cd "$REAL" && pwd -W)"
mkdir -p "$data/gdlp"
rm -f "$data"/gdlp/*.gdlp
gd="$(cd "$data/gdlp" && pwd -W)"
python "$here/player_cases.py" "$data/slp" "$root/rust/target/replay-oracle/real_list.txt" | tr -d '\r' > "$data/player_cases.txt"
: > "$data/player_oracle.txt"

run_cases() {   # $1 = cases file
  local b name flags base k=0
  while IFS='|' read -r b name flags; do
    case "$b" in r) base="$realw";; g) base="$gd";; *) base="$synth";; esac
    rm -rf "$out/case"; mkdir -p "$out/case"
    echo "== $b|$name|$flags" >> "$data/player_oracle.txt"
    (cd "$out/case" && "$out/player_oracle.exe" "$base/$name" . "$flags") | tr -d '\r' | sed "s|$base|<S>|g" >> "$data/player_oracle.txt" || true
    if [ "$b" = G ]; then
      k=$((k + 1))
      if [ -f "$out/case/m.gdlp" ]; then cp "$out/case/m.gdlp" "$data/gdlp/g$k.gdlp";
      elif [ -f "$out/case/geno-lab_lab/recordings/match_1700000000_1.gdlp" ]; then cp "$out/case/geno-lab_lab/recordings/match_1700000000_1.gdlp" "$data/gdlp/g$k.gdlp"; fi
    fi
  done < "$1"
}
run_cases "$data/player_cases.txt"
python "$here/gdlp_variants.py" "$data/gdlp" | tr -d '\r' > "$out/stage2.txt"
cat "$out/stage2.txt" >> "$data/player_cases.txt"
run_cases "$out/stage2.txt"
echo "ran $(wc -l < "$data/player_cases.txt") cases -> $(wc -l < "$data/player_oracle.txt") lines"
