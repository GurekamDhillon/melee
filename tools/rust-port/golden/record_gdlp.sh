#!/bin/bash
# Record a whole-match .gdlp with the C build (MELEE_GDLP_RECORD), then play it back with MELEE_GDLP: oracle for Rust .gdlp playback.
# replay player (`melee golden --slp`). No window is shown (MELEE_WINDOW_X=30000 MELEE_WINDOW_Y=30000, turbo) and no OS
# input is sent: the match is driven by pad_fixed.lua through MELEE_PAD_SCRIPT.
#
#   export GW_BUILD_ROOT=<private build root holding melee-pc.exe>
#   bash tools/rust-port/golden/record_slp.sh <name> <scene> [minutes]
# Writes $GW_BUILD_ROOT/slp/<name>.gdlp (the recording), <name>.rec.csv (neutral hash of the
# recording run) and <name>.play.csv (neutral hash of MELEE_SLP playback with).
set -u
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ws="${GW_WORKSPACE:-E:/Projects/Melee Workspace}"
name="$1"; scene="$2"
out="$GW_BUILD_ROOT/slp"
mkdir -p "$out"
set -a; source "$ws/.env"; set +a
export SDL_WINDOW_ACTIVATE_WHEN_SHOWN=0 SDL_HINT_FORCE_RAISEWINDOW=0 MELEE_UNATTENDED=1 MELEE_WINDOW_X=30000 MELEE_WINDOW_Y=30000 MELEE_SCRIPTS=0 MELEE_TEST_SEED=12345 MELEE_TURBO=1
padw="$(cd "$here" && pwd -W)"
outw="$(cd "$out" && pwd -W)"
rm -f "$out/$name.gdlp" "$out/$name.rec.csv" "$out/$name.play.csv"
env MELEE_PAD_SCRIPT="$padw/pad_fixed.lua" "MELEE_SCENE=$scene" MELEE_GDLP_RECORD="$outw/$name.gdlp" \
    MELEE_NEUTRAL_HASHLOG="$outw/$name.rec.csv" \
    bash "$ws/tools/port/run.sh" --max-seconds 300 "slp-$name-rec-$$" --iso "$GW_ISO_VANILLA" > "$out/$name.rec.out" 2>&1
echo "record rc=$? slp=$(stat -c %s "$out/$name.gdlp" 2>/dev/null) rows=$(($(wc -l < "$out/$name.rec.csv" 2>/dev/null || echo 1) - 1))"
env MELEE_GDLP="$outw/$name.gdlp" MELEE_NEUTRAL_HASHLOG="$outw/$name.play.csv" \
    bash "$ws/tools/port/run.sh" --max-seconds 300 "slp-$name-play-$$" --iso "$GW_ISO_VANILLA" > "$out/$name.play.out" 2>&1
echo "play rc=$? rows=$(($(wc -l < "$out/$name.play.csv" 2>/dev/null || echo 1) - 1))"
