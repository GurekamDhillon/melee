#!/bin/bash
export MELEE_WINDOW_HIDDEN=${MELEE_WINDOW_HIDDEN:-1}  # agents' game windows are invisible (MELEE_WINDOW_HIDDEN=0 shows them)
# Envoy endless loops (envoy-v3-rules.md "Endless loops"): a strong scripted run from the camp door that clears the
# boss, goes on through later loops and finally loses, then (SAVE_TEST=1) a save / continue across a loop boundary.
#
#   GW_ISO_VANILLA=... EXE=... MODS=... OUT=... CARD=... bash tools/rust-port/envoy/loop_run.sh
# Run A plays FRAMES (default 900000, turbo, headless) with DRIVES (the strong row) and Petey as the boss; it must
# reach loop 3 and end in RUN OVER. With SAVE_TEST=1, run B plays until the run is in loop 2 (STOP_FRAMES) and stops;
# run C boots again on the same save and must continue in loop 2 (the camp's Continue).
set -u
here=$(cd "$(dirname "$0")" && pwd)
repo=$(cd "$here/../../.." && pwd)
exe=${EXE:-$repo/build/cargo/release/melee.exe}
out=${OUT:?OUT}
mods=${MODS:?MODS}
card=${CARD:?CARD}
frames=${FRAMES:-900000}
drives=${DRIVES:-ember:3,nova:3,bolt:3,*4000|grand_finale}
[ -n "${GW_ISO_VANILLA:-}" ] || { echo "loop_run: GW_ISO_VANILLA is not set"; exit 2; }
mkdir -p "$out"
unset MELEE_ENVOY_ARM MELEE_ENVOY_START_FIGHT MELEE_ENVOY_COOP
play() { # name save frames title
  rm -rf "$out/$1.card"; cp -r "$card" "$out/$1.card"; mkdir -p "$2"
  (cd "$out" && env MELEE_VOLUME=3 MELEE_WINDOW_TITLE="v3g agent: loop_run $1 ($4)" MELEE_WASM_MODS="$mods" \
    MELEE_ENVOY_DIR="$2" MELEE_CARD_PATH="$out/$1.card" MELEE_PAD_SCRIPT="$here/camp_pad.txt" MELEE_PAD_CHANNELS=03 \
    MELEE_ENVOY_TEST_P1CPU=1 MELEE_PAD_NO_PAUSE=1 MELEE_ENVOY_BOSS=petey MELEE_ENVOY_TEST_DRIVES="$drives" MELEE_TURBO=1 \
    MELEE_CRASH_STACK=200 MELEE_RECLAIM_LEAK=1 MELEE_HANG_DUMP=60 \
    timeout 5400 "$exe" play --iso "$GW_ISO_VANILLA" --headless --no-audio --max-frames "$3" >"$out/$1.log" 2>&1)
  echo "     $1: exit $?"
}
fail=0
ok() { echo "ok   $*"; }
bad() { echo "FAIL $*"; fail=1; }
if [ "${RUN_A:-1}" = 1 ]; then
  rm -rf "$out/a.save"
  play a "$out/a.save" "$frames" "a strong run through the loops"
  echo "     a: $(grep -c 'envoy: fight won' "$out/a.log") fights won, loops: $(grep -o 'envoy: loop now [0-9]*' "$out/a.log" | tr '\n' ' ')"
  grep -q "envoy: loop now 2" "$out/a.log" && ok "the boss fell and the run went on into loop 2" || bad "no loop 2"
  grep -q "envoy: loop now 3" "$out/a.log" && ok "loop 3 reached" || bad "no loop 3"
  grep -q "envoy: RUN OVER" "$out/a.log" && ok "the run ended in RUN OVER ($(grep -o 'RUN OVER at fight [0-9]*' "$out/a.log" | head -1))" || bad "no RUN OVER inside FRAMES"
  [ -f "$out/a.save/best.bin" ] && ok "best loop saved: $(od -An -i "$out/a.save/best.bin" | tr -d ' ')" || bad "no best.bin"
  grep -q "crash\|panicked" "$out/a.log" && bad "a crash or panic in run a"
fi
if [ "${SAVE_TEST:-0}" = 1 ]; then
  rm -rf "$out/b.save"
  play b "$out/b.save" "${STOP_FRAMES:-120000}" "save in loop 2"
  grep -q "envoy: loop now 2" "$out/b.log" && ok "run b reached loop 2" || bad "run b never reached loop 2 (raise STOP_FRAMES)"
  [ -f "$out/b.save/run.bin" ] && ok "run b left a saved run" || bad "run b left no saved run"
  play c "$out/b.save" 20000 "continue in loop 2"
  if grep -q "envoy: armed, run continued at loop\*10 + fight.* 2[0-9]" "$out/c.log"; then
    ok "run c continued in loop 2 ($(grep -o 'run continued at loop\*10 + fight.*' "$out/c.log" | head -1))"
  else bad "run c did not continue in loop 2"; fi
  grep -q "saved run loaded (loop 2" "$out/c.log" && ok "the host loaded the loop-2 record" || bad "the host did not load a loop-2 record"
fi
if [ "${ROAD_TEST:-0}" = 1 ]; then
  # the forked road: a new run takes the Steep road (DRIVES has "steep"), stops inside its session, and the next
  # boot continues on it
  rm -rf "$out/r.save"
  play r "$out/r.save" "${STOP_FRAMES:-6000}" "steep road, save mid-session"
  grep -q "envoy: road taken.* 2" "$out/r.log" && ok "run r took the Steep road" || bad "run r did not take the Steep road"
  play s "$out/r.save" 6000 "continue on the steep road"
  grep -q "envoy: continued on road.* 2" "$out/s.log" && ok "run s continued on the Steep road" || bad "run s did not continue on the Steep road"
fi
[ $fail = 0 ] && echo "LOOP RUN OK" || echo "LOOP RUN FAILED (logs: $out)"
exit $fail
