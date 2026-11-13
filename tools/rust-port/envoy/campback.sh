#!/bin/bash
# Envoy camp: B backs out of the camp to the main menu, never to the title (flaw "B in camp drops to the title").
# Boots with no run armed, opens the camp from the main menu's Envoy row (campback_pad.txt), presses B once, and checks
# the scene sequence in the log: the main menu must be the last scene, with no title scene after the camp opened.
#   GW_ISO_VANILLA=... EXE=... MODS=... CARD=... OUT=... bash tools/rust-port/envoy/campback.sh
set -u
here=$(cd "$(dirname "$0")" && pwd)
repo=$(cd "$here/../../.." && pwd)
. "$here/exe_check.sh"; exe_pick campback; exe_banner campback
mods=${MODS:?MODS}; card=${CARD:?CARD}; out=${OUT:?OUT}
[ -n "${GW_ISO_VANILLA:-}" ] || { echo "campback: GW_ISO_VANILLA is not set"; exit 2; }
rm -rf "$out"; mkdir -p "$out/save"; cp -r "$card" "$out/card"
unset MELEE_ENVOY_ARM MELEE_ENVOY_START_FIGHT MELEE_ENVOY_COOP MELEE_ENVOY_TEST_DRIVES
env MELEE_VOLUME=3 MELEE_FULLSCREEN=off MELEE_WINDOW_TITLE="${TITLE:-envoy campback: B in the camp}" MELEE_WASM_MODS="$mods" \
  MELEE_ENVOY_DIR="$out/save" MELEE_CARD_PATH="$out/card" MELEE_PAD_SCRIPT="$(pad_for_disc "${PAD:-$here/campback_pad.txt}" "$out")" MELEE_PAD_CHANNELS=0 \
  MELEE_PAD_IGNORE_ADAPTER=1 MELEE_RECLAIM_LEAK=1 ${EXTRA_ENV:-} \
  timeout 600 "$exe" play --iso "$GW_ISO_VANILLA" --headless --no-audio --max-frames "${FRAMES:-1500}" >"$out/run.log" 2>&1
seq=$(grep -aoE "play: tick [0-9]+: scene [a-z_]+" "$out/run.log" | sed -E 's/play: tick ([0-9]+): scene /\1:/' | tr '\n' ' ')
echo "     scenes: $seq"
last=$(echo "$seq" | awk '{print $NF}' | cut -d: -f2)
titles=$(echo "$seq" | grep -oE "[0-9]+:title" | wc -l)
fail=0
grep -aq "envoy: camp opened" "$out/run.log" && echo "ok   the camp opened" || { echo "FAIL the camp never opened (or an exe without the camp log)"; fail=1; }
grep -aq "envoy: camp closed" "$out/run.log" && echo "ok   B closed the camp" || { echo "FAIL the camp did not close on B"; fail=1; }
[ "$titles" -le 1 ] && echo "ok   no title scene after the main menu" || { echo "FAIL the title came back ($titles title scenes)"; fail=1; }
[ "$last" = menu ] && echo "ok   the game ends on the main menu" || { echo "FAIL the last scene is '$last', not the main menu"; fail=1; }
[ $fail = 0 ] && echo "CAMPBACK OK" || echo "CAMPBACK FAILED (see $out/run.log)"
exit $fail
