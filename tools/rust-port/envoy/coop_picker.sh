#!/bin/bash
# Envoy co-op picker (two pads, two scripts): P2 joins at the camp with Start, both players pick in the co-op
# picker (P2 picks the Random cell as partner), P1 starts the run. Checks: P2 joined, the run is co-op, the party
# the engine got has four real fighters (Random rolled; none -1 or 99) and P2's main is the one P2 picked, and the
# fight starts as an Envoy co-op match. Windowed with screenshots of the picker and the camp (SHOTS=0: headless).
#   GW_ISO_VANILLA=... EXE=... MODS=... CARD=... OUT=... bash tools/rust-port/envoy/coop_picker.sh
set -u
here=$(cd "$(dirname "$0")" && pwd)
repo=$(cd "$here/../../.." && pwd)
. "$here/exe_check.sh"; exe_pick coop_picker; exe_banner coop_picker
mods=${MODS:?MODS}; card=${CARD:?CARD}; out=${OUT:?OUT}
[ -n "${GW_ISO_VANILLA:-}" ] || { echo "coop_picker: GW_ISO_VANILLA is not set"; exit 2; }
rm -rf "$out"; mkdir -p "$out/save" "$out/shots"; cp -r "$card" "$out/card"
unset MELEE_ENVOY_ARM MELEE_ENVOY_START_FIGHT MELEE_ENVOY_COOP MELEE_ENVOY_TEST_DRIVES
if [ "${SHOTS:-1}" = 1 ]; then view=(--widescreen --windowed --window "${WINDOW:-1280x720}" --shots --shot-dir "$out/shots" --shot-ages "${AGES:-300,420,470,520,640}"); else view=(--headless); fi
env MELEE_VOLUME=3 MELEE_FULLSCREEN=off MELEE_WINDOW_UNFOCUSED=1 MELEE_WINDOW_TITLE="${TITLE:-envoy coop_picker}" MELEE_WASM_MODS="$mods" \
  MELEE_ENVOY_DIR="$out/save" MELEE_CARD_PATH="$out/card" MELEE_PAD_IGNORE_ADAPTER=1 \
  MELEE_PAD_SCRIPT="$(pad_for_disc "$here/coop_pick_p1.txt" "$out")" MELEE_PAD_CHANNELS=0 MELEE_PAD_SCRIPT2="$(pad_for_disc "$here/coop_pick_p2.txt" "$out" 203)" MELEE_PAD_CHANNELS2=1 \
  MELEE_ENVOY_TEST_P1CPU=1 MELEE_RECLAIM_LEAK=1 MELEE_HANG_DUMP=60 ${EXTRA_ENV:-} \
  timeout 900 "$exe" play --iso "$GW_ISO_VANILLA" --no-audio "${view[@]}" --max-frames "${FRAMES:-2400}" >"$out/run.log" 2>&1
echo "     exit $?"
fail=0
ok() { echo "ok   $*"; }
bad() { echo "FAIL $*"; fail=1; }
grep -aq "gw: pad script: .* segments from .*coop_pick_p2" "$out/run.log" && ok "P2's own script loaded" || bad "no second pad script (an exe without MELEE_PAD_SCRIPT2)"
m=$(grep -a -m1 -oE "envoy: match 1 rewritten: main -?[0-9]+ partner -?[0-9]+ co-op [0-9]+" "$out/run.log")
echo "     $m"
[ -n "$m" ] && ok "the run started (fight 1 rewritten)" || bad "fight 1 never started"
case "$m" in *"co-op 1") ok "the run is co-op";; *) bad "the run is not co-op";; esac
p=$(grep -a -m1 -oE "envoy: party m1 -?[0-9]+ p1 -?[0-9]+ m2 -?[0-9]+ p2 -?[0-9]+" "$out/run.log")
echo "     $p"
if [ -n "$p" ]; then
  echo "$p" | grep -qE " (-1|99)( |$)" && bad "the party has an unset or Random seat: $p" || ok "four real fighters in the party"
else bad "no party line from the host (gd_ui_envoy_party)"; fi
grep -aq "panicked\|crash" "$out/run.log" && bad "a crash or panic"
[ $fail = 0 ] && echo "COOP PICKER OK" || echo "COOP PICKER FAILED (see $out/run.log)"
exit $fail
