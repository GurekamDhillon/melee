#!/bin/bash
# Envoy sessions (v3g): one windowed scripted run from the camp door with in-engine screenshots of every screen
# (the camp, the session banner, the HUD with drops, the batched picks, the shop, the road). Look at them.
#   GW_ISO_VANILLA=... EXE=... MODS=... CARD=... OUT=... bash tools/rust-port/envoy/session_shots.sh
# FRAMES (default 60000), AGES (scene ages to shoot, default "20,45,150,260,420,900,1800"), DRIVES (the build).
set -u
here=$(cd "$(dirname "$0")" && pwd)
exe=${EXE:?EXE}; mods=${MODS:?MODS}; card=${CARD:?CARD}; out=${OUT:?OUT}
[ -n "${GW_ISO_VANILLA:-}" ] || { echo "session_shots: GW_ISO_VANILLA is not set"; exit 2; }
rm -rf "$out"; mkdir -p "$out/shots" "$out/save"; cp -r "$card" "$out/card"
unset MELEE_ENVOY_ARM MELEE_ENVOY_START_FIGHT MELEE_ENVOY_COOP
(cd "$out" && env MELEE_VOLUME=3 MELEE_WINDOW_TITLE="${TITLE:-v3g agent: session_shots, windowed run with screenshots (port/v3g)}" \
  MELEE_WASM_MODS="$mods" MELEE_ENVOY_DIR="$out/save" MELEE_CARD_PATH="$out/card" MELEE_PAD_SCRIPT="$here/camp_pad.txt" \
  MELEE_PAD_CHANNELS=03 MELEE_ENVOY_TEST_P1CPU=1 MELEE_ENVOY_BOSS=petey MELEE_TURBO=1 MELEE_PAD_NO_PAUSE=1 \
  MELEE_ENVOY_TEST_DRIVES="${DRIVES:-ember:3,nova:3,bolt:3,*8000|grand_finale}" \
  MELEE_CRASH_STACK=200 MELEE_RECLAIM_LEAK=1 MELEE_HANG_DUMP=60 ${EXTRA_ENV:-} \
  timeout 3600 "$exe" play --iso "$GW_ISO_VANILLA" --max-frames "${FRAMES:-60000}" --shots --shot-dir "$out/shots" \
  --shot-ages "${AGES:-20,45,150,260,420,900,1800}" >"$out/run.log" 2>&1)
echo "exit $?; $(ls "$out/shots" | wc -l) screenshots in $out/shots"
grep -aoE "envoy: (fight won, next [0-9]|loop now [0-9]|drive offer, fight [0-9]+|shop open, fight [0-9]+|RUN OVER.*)" "$out/run.log" | uniq -c | head -40
