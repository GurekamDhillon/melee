#!/bin/bash
export MELEE_WINDOW_HIDDEN=${MELEE_WINDOW_HIDDEN:-1}  # agents' game windows are invisible (MELEE_WINDOW_HIDDEN=0 shows them)
# pair.sh SCRIPT TICKS TAG: C vs Rust shots of a `melee play` pad script run from the title at the same timeline ticks.
# C lane: $C_LANE (a dir holding a working melee-pc.exe + dlls), Rust exe: $RS_EXE (default build/cargo/fast/melee.exe).
# Shots land in $SHOT_BASE/{c,rs}_TAG; no OS input, windows unfocused/off screen, volume 3.
set -e
D="$(cd "$(dirname "$0")" && pwd)"; WS="/e/Projects/Melee Workspace"; REPO="$(cd "$D/../../.." && pwd)"
B="${SHOT_BASE:-$WS/_build/agents/menus3/shots}"; S="$D/$1"; [ -f "$1" ] && S="$1"; T=$2; TAG=$3
EXE="${RS_EXE:-$REPO/build/cargo/fast/melee.exe}"
set -a; . "$WS/.env"; set +a
mkdir -p "$B"
if [ -z "$SKIP_C" ]; then python "$D/gen_lua.py" "$S" $T "$B/$TAG.lua"; MELEE_WINDOW_TITLE="menus3:c:$TAG" bash "$D/run_c.sh" "$B/$TAG.lua" "$B/c_$TAG" >/dev/null; fi
if [ -z "$SKIP_RS" ]; then
rm -rf "$B/rs_$TAG"
MAXF=$(( $(echo $T | tr ',-' '\n\n' | sort -n | tail -1) + 5 ))
MELEE_RETAIL_SHOT_TICKS="$T" MELEE_PAD_SCRIPT="$S" MELEE_PAD_IGNORE_ADAPTER=1 MELEE_WINDOW_UNFOCUSED=1 MELEE_VOLUME=3 MELEE_CARD=0 MELEE_WINDOW_TITLE="menus3:rs:$TAG" \
  nice "$EXE" play --iso "$GW_ISO_VANILLA" --skip-intro --shot-dir "$B/rs_$TAG" --shot-ticks $T --max-frames $MAXF > "$B/rs_$TAG.log" 2>&1 || true
grep "^play:.*->\|error\|panick" "$B/rs_$TAG.log" | head
for f in "$B"/rs_$TAG/*tl_*.png; do n=$(basename "$f" .png); n=${n##*tl_}; n=$(printf %04d $((10#$n + ${RS_OFS:-0}))); cp "$f" "$B/rs_$TAG/play_${TAG}_$n.png"; done
fi
python "$D/cmp.py" "$B/c_$TAG" "$B/rs_$TAG" $TAG
