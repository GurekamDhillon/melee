#!/bin/bash
export MELEE_WINDOW_HIDDEN=${MELEE_WINDOW_HIDDEN:-1}  # agents' game windows are invisible (MELEE_WINDOW_HIDDEN=0 shows them)
# The Envoy Item Lab (envoy_data lab.rs, envoy_sim lab.rs, envoy_ui lab.rs), driven by a pad script (lab_pad.py) from
# the title screen: the main menu's Item Lab cell, the setup, then the panel mid-fight (D-pad only):
#   1. flow: every item fired on demand; every item equipped one at a time with its trigger fired; every trigger with
#      all held; 10 of one item; every modifier and category off and on; preset 1 saved, cleared, loaded; slow motion,
#      pause and frame advance; reset; then everything at once. Checks the logs for each, no crash, nothing written
#      to the run record (no run.bin, no best.bin). Screenshots of every page (SHOTS from lab_pad.py --marks).
#   2. perf: the same script in real time with the frame profiler, no shots: no frame over 16 ms (zero_hitch_check).
#   3. sync: a Lab session as a synctest (MELEE_ENVOY_LAB, the lab@frame:op:a:b knob drives the same commands
#      with no UI): DEPTH (4), FRAMES (3000), 0 mismatches.
#   GW_ISO_VANILLA=... EXE=... MODS=<folder with envoy_sim.wasm, envoy_ui.wasm, envoy_ui/> CARD=... OUT=... bash lab_check.sh
#   [ONLY="flow perf sync"] [SIZE=1280x720]
set -u
here=$(cd "$(dirname "$0")" && pwd)
repo=$(cd "$here/../../.." && pwd)
. "$here/exe_check.sh"; exe_pick lab_check
mods=${MODS:?MODS (a folder with envoy_sim.wasm, envoy_ui.wasm and envoy_ui/)}
out=${OUT:-$repo/build/lab-check}; size=${SIZE:-1280x720}; only=${ONLY:-flow perf sync}
[ -n "${GW_ISO_VANILLA:-}" ] || { echo "lab_check: GW_ISO_VANILLA is not set"; exit 2; }
mkdir -p "$out"
python "$here/lab_pad.py" > "$out/lab_pad.txt"
python "$here/lab_pad.py" --marks > "$out/marks.txt"
frames=$(head -1 "$out/lab_pad.txt" | grep -oE "[0-9]+ frames" | grep -oE "[0-9]+")
shots=$(awk '{printf "%s%d", (NR>1?",":""), $2+8}' "$out/marks.txt")
fail=0
on() { case " $only " in *" $1 "*) return 0 ;; esac; return 1; }
run() { # name extra-env...
  local name=$1; shift; local d="$out/$name"
  rm -rf "$d"; mkdir -p "$d/shots" "$d/save" "$d/card/USA/Card A"
  cp -r "${CARD:?CARD (the test card files: copied to <card>/USA/Card A)}"/. "$d/card/USA/Card A/"
  (cd "$d" && env MELEE_VOLUME=3 MELEE_WINDOW_TITLE="itemlab agent: lab_check $name $size (port/envoy-itemlab)" \
    MELEE_WASM_MODS="$mods" MELEE_ENVOY_DIR="$d/save" MELEE_CARD_PATH="$d/card" \
    MELEE_PAD_SCRIPT="$out/lab_pad.txt" MELEE_PAD_IGNORE_ADAPTER=1 MELEE_PAD_CHANNELS=01 MELEE_WIDESCREEN=1 MELEE_FULLSCREEN=off \
    MELEE_WINDOW="$size" MELEE_UI_LOG=1 "$@" \
    timeout 1800 "$exe" play --iso "$GW_ISO_VANILLA" --max-frames $((frames + 200)) >"$d/run.log" 2>&1)
  echo "$name: exit $? ($(ls "$d/shots" 2>/dev/null | wc -l) shots)"
}
has() { grep -aq -- "$2" "$out/$1/run.log"; }
cnt() { grep -ac -- "$2" "$out/$1/run.log"; }
if on flow; then
  exe_banner "lab_check flow"
  run flow --shots --shot-dir shots MELEE_RETAIL_SHOT_TICKS="$shots"
  miss=""
  has flow "envoy lab: setup open" || miss="$miss (setup never opened from the menu cell)"
  has flow "envoy: lab session armed" || miss="$miss (no lab session)"
  has flow "envoy lab: panel open" || miss="$miss (panel never opened)"
  for page in Items Loadout Triggers Effects Sandbox Synergy; do has flow "envoy lab: page $page" || [ $page = Items ] || miss="$miss (page $page)"; done
  n=$(python -c "import re,sys;print(len(re.findall(r'Item \{ id:', open(sys.argv[1],encoding='utf-8').read())))" "$repo/rust/mods/envoy_data/src/items.rs")
  fired=$(grep -aoE "envoy: item demo fires [a-z_]+" "$out/flow/run.log" | sort -u | wc -l)
  [ "$fired" -ge "$n" ] || miss="$miss (items fired on demand: $fired of $n)"
  has flow "lab cannot fire yet" && miss="$miss (an item the Lab cannot fire)"
  has flow "envoy lab: trigger fired" || miss="$miss (no trigger fired)"
  has flow "envoy lab: modifier off" || miss="$miss (no modifier switched)"
  has flow "envoy lab: category off" || miss="$miss (no category switched)"
  has flow "envoy lab: presets saved, rc" || miss="$miss (preset not saved)"
  has flow "envoy lab: Preset 1 loaded" || miss="$miss (preset not loaded)"
  [ -f "$out/flow/save/run.bin" ] && miss="$miss (the run record was written)"
  [ -f "$out/flow/save/best.bin" ] && miss="$miss (the best loop was written)"
  # 10 copies: the SET command op 1 with b = 10 (op*65536 + a*256 + b)
  grep -aE "envoy: lab command op.* [0-9]+$" "$out/flow/run.log" | awk '{v=$NF; if (int(v/65536)==1 && v%256==10) f=1} END{exit !f}' || miss="$miss (no stack of 10)"
  grep -aE "envoy: lab command op.* 720896$" -q "$out/flow/run.log" || miss="$miss (no reset)"
  grep -aqiE "panicked|wasm trap|crash|fatal" "$out/flow/run.log" && miss="$miss (a crash or trap in the log)"
  [ -z "$miss" ] && echo "ok   flow: menu cell, setup, every item fired and equipped, triggers, stack 10, modifiers, presets, time, reset" \
    || { echo "FAIL flow:$miss (see $out/flow/run.log)"; fail=1; }
fi
if on perf; then
  exe_banner "lab_check perf"
  run perf MELEE_PROFILE=1
  pf=$(ls -t "$(dirname "$exe")/profiles/"profile-*.csv 2>/dev/null | head -1)
  if [ -n "$pf" ]; then python "$repo/tools/rust-port/perf/zero_hitch_check.py" "$pf" --label "lab all items" || fail=1
  else echo "FAIL perf: no profile"; fail=1; fi
fi
if on sync; then
  exe_banner "lab_check sync"
  # the same commands as the flow, from the sim's own test knob: all items, every trigger fired, stacks, switches
  ks="lab@200:4:0:1,lab@260:8:1:0,lab@300:8:2:0,lab@340:8:3:0,lab@380:8:4:0,lab@420:8:5:0,lab@460:8:6:0,lab@500:8:7:0,lab@540:8:8:0,lab@580:8:9:0"
  ks="$ks,lab@620:1:12:10,lab@660:14:35:1,lab@700:15:3:0,lab@760:8:1:0,lab@800:14:35:0,lab@820:15:0:0,lab@900:11:0:0,lab@1000:8:4:0,lab@1100:16:1:0,lab@1300:18:0:0"
  ks="$ks,lab@1600:7:1:0,lab@1700:7:7:0,lab@1800:10:120:0,lab@1900:8:8:0"
  d=${DEPTH:-4}
  env MELEE_VOLUME=3 MELEE_WINDOW_TITLE="itemlab agent: lab_check sync d$d (port/envoy-itemlab)" MELEE_WASM_MODS="$mods" MELEE_ENVOY_ARM=1 \
    MELEE_ENVOY_LAB="36,1,0,9" MELEE_ENVOY_TEST_DRIVES="$ks" MELEE_ENVOY_DIR="$out/sync-save" \
    timeout 2400 "$exe" synctest --engine tu --iso "$GW_ISO_VANILLA" --scenario pad_falcon_fox_fd --depth "$d" --frames "${FRAMES:-3000}" --timeout 2000 \
    >"$out/sync.log" 2>&1
  line=$(grep 'engine synctest' "$out/sync.log")
  miss=""
  grep -aq "envoy: lab session armed" "$out/sync.log" || miss="$miss (not a lab session)"
  [ "$(grep -ac 'envoy: lab command' "$out/sync.log")" -ge 20 ] || miss="$miss (commands not run)"
  case "$line" in *" 0 MISMATCHES"*) [ -z "$miss" ] && echo "ok   sync d$d: ${line#melee: }" || { echo "FAIL sync:$miss"; fail=1; } ;;
    *) echo "FAIL sync d$d: ${line:-no result line} (see $out/sync.log)"; fail=1 ;; esac
fi
exit $fail
