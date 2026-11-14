#!/usr/bin/env bash
# pipe_seed.sh: fill the pipeline warm list (melee_gx render.rs `warmlist`, gpucache/pipelines-<layout>-<size>.bin) with
# every fighter an Envoy run can spawn (envoy_sim CPU_POOL), every VS stage and the Envoy effects, so a fresh install or a new build
# compiles them all at start-up (off the frame path, before the camp) instead of on the frame a wave brings a new
# fighter in (39-43 ms "GPU pipelines" frames before; notes/zero-hitch.md "Pipeline warm list").
#   EXE=<melee.exe> [MODS=<envoy mods dir>] [CARD_MASTER=<test card>] [MELEE_GPUCACHE=<dir>] pipe_seed.sh [out dir]
# Hidden windows, MELEE_VOLUME=3, turbo, a hard timeout per run. The list is local GPU state, never committed.
set -u
: "${EXE:?}" "${GW_ISO_VANILLA:?}"
here=$(cd "$(dirname "$0")/.." && pwd)
out=${1:-$(dirname "$EXE")/pipe_seed}; mkdir -p "$out"
# every VS stage (an Envoy stage shift or a gauntlet room can land on any), one per match
stages=("FINAL DESTINATION" "BATTLEFIELD" "YOSHI'S STORY" "FOUNTAIN OF DREAMS" "POKEMON STADIUM" "DREAM LAND N64"
        "PEACH'S CASTLE" "KONGO JUNGLE" "BRINSTAR" "CORNERIA" "ONETT" "MUTE CITY" "RAINBOW CRUISE" "JUNGLE JAPES"
        "GREAT BAY" "HYRULE TEMPLE" "BRINSTAR DEPTHS" "YOSHI'S ISLAND" "GREEN GREENS" "FOURSIDE" "MUSHROOM KINGDOM"
        "MUSHROOM KINGDOM II" "VENOM" "POKE FLOATS" "BIG BLUE" "ICICLE MOUNTAIN" "FLAT ZONE" "YOSHI'S ISLAND N64"
        "KONGO JUNGLE N64")
# envoy_sim CPU_POOL (the foes a wave can bring) and the two starters, as --match names, two per match
pairs=("falcon dk" "fox gw" "kirby bowser" "link luigi" "mario marth" "mewtwo ness" "peach pikachu" "jigglypuff samus"
       "yoshi falco" "younglink drmario" "roy pichu" "zelda sheik")
i=0
while [ $i -lt ${#stages[@]} ]; do
  p=${pairs[$((i % ${#pairs[@]}))]}; st=${stages[$i]}; i=$((i + 1))
  set -- $p
  echo "-- $1 + $2 on $st"
  env MELEE_WINDOW_HIDDEN=1 MELEE_VOLUME=3 MELEE_FULLSCREEN=off MELEE_TURBO=1 MELEE_WINDOW_TITLE="pipe_seed: $1 $2" \
    timeout "${RUN_TIMEOUT:-300}" "$EXE" play --iso "$GW_ISO_VANILLA" --widescreen --windowed --window 1280x720 \
    --match "$1/cpu9,$2/cpu9,$st" --max-frames "${FRAMES:-1500}" >"$out/$i-$1-$2.log" 2>&1
done
if [ -n "${MODS:-}" ] && [ -n "${CARD_MASTER:-}" ]; then
  # one Envoy run through the effects (drives, tags, waves, stage shift, combo breaks)
  o="$out/envoy"; rm -rf "$o"; mkdir -p "$o/save" "$o/card/USA/Card A"; cp "$CARD_MASTER/"* "$o/card/USA/Card A/"
  echo "-- envoy"
  (cd "$o" && env MELEE_WINDOW_HIDDEN=1 MELEE_VOLUME=3 MELEE_FULLSCREEN=off MELEE_TURBO=1 MELEE_WINDOW_TITLE="pipe_seed: envoy" \
    MELEE_WASM_MODS="$MODS" MELEE_ENVOY_DIR="$o/save" MELEE_CARD_PATH="$o/card" MELEE_PAD_SCRIPT="$here/envoy/camp_pad.txt" \
    MELEE_PAD_CHANNELS=03 MELEE_ENVOY_TEST_P1CPU=1 MELEE_ENVOY_BOSS=petey MELEE_PAD_NO_PAUSE=1 \
    "MELEE_ENVOY_TEST_DRIVES=ember,bolt,tagauto,wave2,shift,dropall/ember,bolt" \
    timeout "${RUN_TIMEOUT:-600}" "$EXE" play --iso "$GW_ISO_VANILLA" --widescreen --windowed --window 1280x720 \
    --max-frames "${ENVOY_FRAMES:-20000}" >"$o/run.log" 2>&1)
fi
d=${MELEE_GPUCACHE:-$(dirname "$EXE")/gpucache}
ls -la "$d" 2>/dev/null | grep pipelines- || echo "no warm list written in $d"
