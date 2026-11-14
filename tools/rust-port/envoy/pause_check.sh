#!/bin/bash
export MELEE_WINDOW_HIDDEN=${MELEE_WINDOW_HIDDEN:-1}  # agents' game windows are invisible (MELEE_WINDOW_HIDDEN=0 shows them)
# The Envoy pause overlay (envoy_ui pause.rs), driven by a pad script (pause_pad.txt), in an armed Envoy fight 1:
#   1. nav: Start pauses; the D-pad walks every page (Items with a hovered card, Run, Fight, Options: text size,
#      effects, shake, Abandon asked and cancelled); the native pause camera moves (stick, C-stick, X); Start resumes;
#      then L + R + A + Start quits the match. Checks: the overlay logged each page, the pause camera moved (the
#      shots before and after differ), no D-pad press reached the sim while paused (no tag / no stray item fire from
#      the D-pad), the quit ended the match, the run was NOT abandoned. Screenshots of every page.
#   2. abandon: pause, Options, Abandon, right twice: the sim logs "run abandoned" and the match ends.
#   GW_ISO_VANILLA=... EXE=... MODS=... OUT=... [SIZE=1280x720] [ONLY="nav abandon"] bash pause_check.sh
set -u
here=$(cd "$(dirname "$0")" && pwd)
repo=$(cd "$here/../../.." && pwd)
. "$here/exe_check.sh"; exe_pick pause_check
mods=${MODS:?MODS (a folder with envoy_sim.wasm, envoy_ui.wasm and envoy_ui/)}
out=${OUT:-$repo/build/pause-check}; size=${SIZE:-1280x720}; only=${ONLY:-nav abandon}
[ -n "${GW_ISO_VANILLA:-}" ] || { echo "pause_check: GW_ISO_VANILLA is not set"; exit 2; }
fail=0
SHOTS="343,397,441,473,549,637,700,847"
run() { # name pad
  local name=$1 pad=$2 d="$out/$1" shotargs=()
  [ -n "${PERF:-}" ] || shotargs=(--shots --shot-dir "$d/shots")
  rm -rf "$d"; mkdir -p "$d/shots" "$d/save"; mkdir -p "$d/card/USA/Card A"; cp -r "${CARD:?CARD (the test card files: copied to <card>/USA/Card A)}"/. "$d/card/USA/Card A/"
  (cd "$d" && env MELEE_VOLUME=3 MELEE_WINDOW_TITLE="pause1 agent: pause_check $name $size (port/envoy-pause1)" \
    MELEE_WASM_MODS="$mods" MELEE_ENVOY_DIR="$d/save" MELEE_CARD_PATH="$d/card" MELEE_ENVOY_ARM=1 MELEE_ENVOY_START_FIGHT=1 MELEE_ENVOY_TEST_DRIVES="${ITEMS:-spark_ring,hop_shot,hop_shot,moon_drops,coin_magnet,fire_lace,noshop}" \
    MELEE_PAD_SCRIPT="$pad" MELEE_PAD_IGNORE_ADAPTER=1 MELEE_WIDESCREEN=1 MELEE_FULLSCREEN=off MELEE_WINDOW="$size" \
    MELEE_UI_LOG=1 MELEE_RETAIL_SHOT_TICKS="$SHOTS" ${PERF:+MELEE_PROFILE=1} \
    timeout 900 "$exe" play --iso "$GW_ISO_VANILLA" --match falcon,fox,32 --max-frames 2200 \
    "${shotargs[@]}" >"$d/run.log" 2>&1)
  echo "$name: exit $? ($(ls "$d/shots" 2>/dev/null | wc -l) shots)"
  # PERF=1: real time with the frame profiler and no shots; every frame interval and tick under 16.7 ms
  if [ -n "${PERF:-}" ]; then
    local pf; pf=$(ls -t "$(dirname "$exe")/profiles/"profile-*.csv 2>/dev/null | head -1)
    if [ -n "$pf" ]; then python "$repo/tools/rust-port/perf/zero_hitch_check.py" "$pf" --label "pause $name" || fail=1
    else echo "FAIL perf $name: no profile"; fail=1; fi
  fi
}
has() { grep -aq "$2" "$out/$1/run.log"; }
if case " $only " in *" nav "*) true ;; *) false ;; esac; then
  exe_banner "pause_check nav"
  run nav "$here/pause_pad.txt"
  miss=""
  has nav "envoy pause: page" || miss="$miss (overlay never logged)"
  for pg in Items Run Fight Options; do has nav "envoy pause: page $pg" || miss="$miss (page $pg)"; done
  has nav "envoy pause: option" || miss="$miss (no option changed)"
  has nav "run abandoned" && miss="$miss (the run was abandoned: it must not be)"
  has nav "envoy: quick tag" && miss="$miss (a tag fired while paused: input leaked)"
  # the quit: the match ended by the pause quit (the scene left the match before max frames)
  has nav "envoy pause: closed" || miss="$miss (the overlay never closed)"
  grep -aqE "scene (results|css) [(]" "$out/nav/run.log" && [ "$(grep -ac "game pause by player" "$out/nav/run.log")" -ge 2 ] || miss="$miss (no match end after L+R+A+Start)"
  a="$out/nav/shots"; f1=$(ls "$a" | grep 0637 | head -1); f2=$(ls "$a" | grep 0847 | head -1)
  if [ -n "$f1" ] && [ -n "$f2" ] && cmp -s "$a/$f1" "$a/$f2"; then miss="$miss (the pause camera did not move)"; fi
  [ -z "$miss" ] && echo "ok   nav: every page, options, native camera, resume, quit" || { echo "FAIL nav:$miss (see $out/nav/run.log)"; fail=1; }
fi
if case " $only " in *" abandon "*) true ;; *) false ;; esac; then
  exe_banner "pause_check abandon"
  p="$out/abandon_pad.txt"; mkdir -p "$out"
  printf '%s\n' "300 0000 0 0" "3 1000 0 0" "30 0000 0 0" "2 0001 0 0" "10 0000 0 0" "2 0004 0 0" "10 0000 0 0" \
    "2 0004 0 0" "6 0000 0 0" "2 0004 0 0" "6 0000 0 0" "2 0004 0 0" "10 0000 0 0" "2 0002 0 0" "10 0000 0 0" \
    "2 0002 0 0" "900 0000 0 0" > "$p"
  SHOTS="380,395" run abandon "$p"
  miss=""
  has abandon "envoy pause: abandon sent" || miss="$miss (the overlay never sent it)"
  has abandon "envoy: run abandoned from the pause overlay" || miss="$miss (the sim never saw it)"
  [ -z "$miss" ] && echo "ok   abandon: confirm, synced signal, the sim ended the run" || { echo "FAIL abandon:$miss (see $out/abandon/run.log)"; fail=1; }
fi
grep -ahoE "frame (max|worst)[^,;]*" "$out"/*/run.log 2>/dev/null | sort | uniq -c | head -4
exit $fail
