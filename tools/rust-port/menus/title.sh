#!/bin/bash
# title.sh: Rust title scene shots vs the C build's, same ages (scene age = ticks since the first title frame).
set -e
D="$(cd "$(dirname "$0")" && pwd)"; WS="/e/Projects/Melee Workspace"; B="${SHOT_BASE:-$WS/_build/agents/plat-menus}"
AGES=${AGES:-1,2,3,5,10,30,60,120,200}
set -a; . "$WS/.env"; set +a
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0
if [ -z "$SKIP_C" ]; then python "$D/gen_lua.py" "$D/idle.txt" $AGES "$B/title.lua"; bash "$D/run_c.sh" "$B/title.lua" "$B/c_title" >/dev/null; fi
rm -rf "$B/rs_title"
cd "$D/../../../rust"
MELEE_PAD_SCRIPT="$D/idle.txt" MELEE_PAD_IGNORE_ADAPTER=1 cargo run -q -p melee -- play --skip-intro --no-audio --shot-dir "$B/rs_title" --shot-ages $AGES --max-frames 210 2>&1 | grep "^play:" | tail -3
python "$D/cmp.py" "$B/c_title" "$B/rs_title" title
