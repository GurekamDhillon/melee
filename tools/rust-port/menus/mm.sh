#!/bin/bash
# mm.sh SCRIPT TICKS TAG: C vs Rust shots of a navigation script run from the title (timeline ticks).
set -e
D="$(cd "$(dirname "$0")" && pwd)"; WS="/e/Projects/Melee Workspace"; B="${SHOT_BASE:-$WS/_build/agents/plat-menus}"
S="$D/$1"; T=$2; TAG=$3
set -a; . "$WS/.env"; set +a
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0
if [ -z "$SKIP_C" ]; then python "$D/gen_lua.py" "$S" $T "$B/$TAG.lua"; bash "$D/run_c.sh" "$B/$TAG.lua" "$B/c_$TAG" >/dev/null; fi
rm -rf "$B/rs_$TAG"
cd "$D/../../../rust"
MAXF=$(( $(echo $T | tr ',' '\n' | sort -n | tail -1) + 5 ))
MELEE_PAD_SCRIPT="$S" MELEE_PAD_IGNORE_ADAPTER=1 cargo run -q -p melee -- play --skip-intro --no-audio --shot-dir "$B/rs_$TAG" --shot-ticks $T --max-frames $MAXF 2>&1 | grep "^play:.*->\|error\|panick" | head
for f in "$B"/rs_$TAG/play_tl_*.png; do n=$(basename "$f" .png); n=${n#play_tl_}; cp "$f" "$B/rs_$TAG/play_${TAG}_$n.png"; done
python "$D/cmp.py" "$B/c_$TAG" "$B/rs_$TAG" $TAG
