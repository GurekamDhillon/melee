#!/usr/bin/env bash
# run_modes_check.sh: the same scripted run headless and in a hidden real window must simulate the same game.
# Runs the Envoy camp route (envoy/camp_pad.txt, P1 CPU) both ways with MELEE_TICK_LOG (per tick: engine frame,
# scene, the pads the sim got, the logic seed) and MELEE_HASH_LOG (per match frame NHV1) and fails on the first
# differing line. notes/run-modes.md.
#   EXE=<melee.exe> MODS=<mods dir> CARD_MASTER=<test card dir> OUT=<dir> [TICKS=6000] run_modes_check.sh
# Needs GW_ISO_VANILLA (.env). Hidden windows, MELEE_VOLUME=3, a hard timeout per run.
set -u
: "${EXE:?}" "${MODS:?}" "${CARD_MASTER:?}" "${OUT:?}" "${GW_ISO_VANILLA:?}"
TICKS=${TICKS:-6000}
here=$(cd "$(dirname "$0")" && pwd)
run() { # <name> <mode args...>
  local o="$OUT/$1"; shift
  rm -rf "$o"; mkdir -p "$o/save" "$o/card/USA/Card A"; cp "$CARD_MASTER/"* "$o/card/USA/Card A/"
  (cd "$o" && env MELEE_WINDOW_HIDDEN=1 MELEE_VOLUME=3 MELEE_FULLSCREEN=off MELEE_WINDOW_UNFOCUSED=1 MELEE_PAD_IGNORE_ADAPTER=1 \
    MELEE_WINDOW_TITLE="run_modes_check: $(basename "$o")" MELEE_WASM_MODS="$MODS" MELEE_ENVOY_DIR="$o/save" MELEE_CARD_PATH="$o/card" \
    MELEE_PAD_SCRIPT="$here/envoy/camp_pad.txt" MELEE_PAD_CHANNELS=03 MELEE_ENVOY_TEST_P1CPU=1 MELEE_ENVOY_BOSS=petey \
    MELEE_TURBO=1 MELEE_PAD_NO_PAUSE=1 MELEE_ENVOY_TEST_DRIVES=noshop MELEE_TICK_LOG="$o/tick.log" MELEE_HASH_LOG="$o/hash.log" \
    timeout "${RUN_TIMEOUT:-900}" "$EXE" play --iso "$GW_ISO_VANILLA" "$@" --max-frames "$TICKS" >"$o/run.log" 2>&1)
}
run headless --headless --no-audio
run window --widescreen --windowed --window 1280x720
fail=0
for f in tick hash; do
  a="$OUT/headless/$f.log"; b="$OUT/window/$f.log"
  n=$(wc -l < "$a"); m=$(wc -l < "$b")
  d=$(cmp "$a" "$b" 2>&1 | head -1)
  if [ -z "$d" ]; then echo "$f.log: equal ($n lines)";
  elif [ "${d#*EOF}" != "$d" ]; then echo "$f.log: equal over the common $((n<m?n:m)) lines (lengths $n / $m)";
  else
    l=$(echo "$d" | sed -n 's/.*line \([0-9]*\).*/\1/p'); echo "$f.log: DIFFER at line $l"
    sed -n "${l}p" "$a" | sed 's/^/  headless: /'; sed -n "${l}p" "$b" | sed 's/^/  window:   /'; fail=1
  fi
done
[ "$(wc -l < "$OUT/headless/tick.log")" -ge 1000 ] || { echo "too few ticks logged"; fail=1; }
[ $fail = 0 ] && echo "RUN MODES OK" || echo "RUN MODES FAIL"
exit $fail
