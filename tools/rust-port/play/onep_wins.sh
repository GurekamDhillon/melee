#!/bin/bash
export MELEE_WINDOW_HIDDEN=${MELEE_WINDOW_HIDDEN:-1}  # agents' game windows are invisible (MELEE_WINDOW_HIDDEN=0 shows them)
# The 1P win paths, unattended (no OS input; port 1 is handed to a level-9 CPU with the TEST-ONLY MELEE_TEST_P1_CPU):
#   classic / adventure / allstar: MELEE_SCENE at the last stage -> the clear, the ending movie, the staff roll, the
#   prize scene, back to the main menu;  events: every event 11-51 from the menus (MELEE_TEST_EVENTS_UNLOCKED opens the
#   list) must reach its match and come back to the menu.
#   bash tools/rust-port/play/onep_wins.sh [classic adventure allstar events]
# Exit 0 when every run did. Needs GW_ISO_VANILLA (the workspace .env).
set -u
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"
ws="${GW_WORKSPACE:-E:/Projects/Melee Workspace}"
if [ -z "${GW_ISO_VANILLA:-}" ] && [ -f "$ws/.env" ]; then set -a; . "$ws/.env"; set +a; fi
exe="$repo/build/cargo/${PROFILE:-release}/melee.exe"
out="${TEMP:-/tmp}/onep_wins.$$"; mkdir -p "$out"
export MELEE_PAD_IGNORE_ADAPTER=1 MELEE_WINDOW_UNFOCUSED=1 MELEE_CARD=0 MELEE_VOLUME=3 MELEE_MENU_LOG=1 MELEE_TEST_P1_CPU=9
names=("$@"); [ ${#names[@]} -eq 0 ] && names=(classic adventure allstar events)
# START then A every 120 frames: the stage intros, the bonus screen, the ending and prize scenes
press="$out/press.txt"; { echo "60 0000 0 0"; for i in $(seq 400); do printf '3 1000 0 0\n117 0000 0 0\n3 0100 0 0\n117 0000 0 0\n'; done; } >"$press"
run() { # name pad frames
  MELEE_WINDOW_TITLE="onep_wins:$1" MELEE_PAD_SCRIPT="$2" nice -n 19 "$exe" play --headless --no-audio --skip-intro \
    --iso "$GW_ISO_VANILLA" --max-frames "$3" >"$out/$1.log" 2>&1
}
fail=0
for n in "${names[@]}"; do
  case "$n" in
  classic|adventure|allstar)
    step=$(case $n in classic) echo 10 ;; adventure) echo 11 ;; allstar) echo 12 ;; esac)
    end=$(case $n in classic) echo 21 ;; adventure) echo 22 ;; allstar) echo 23 ;; esac)
    MELEE_SCENE="mode=$n;p1=fox/stocks5;step=$step;difficulty=0" run "$n" "$press" 22000; rc=$?
    # the ending mode (GM_CLASSIC/ADVENTURE/ALLSTAR_GOVER), the prize scene (mode 20), then the main menu
    if [ $rc -eq 0 ] && grep -q "scene scene (mode $end, scene kind 43" "$out/$n.log" && grep -q "scene menu (mode 1" "$out/$n.log"; then
      echo "OK $n"; else echo "FAIL $n: rc $rc (log $out/$n.log)"; fail=1; fi ;;
  events)
    src="$here/onep_event10.txt"; i=$(grep -n '^# event menu' "$src" | cut -d: -f1)
    for e in $(seq 11 51); do
      p="$out/ev$e.txt"
      { head -n "$i" "$src"; for k in $(seq $((e - 1))); do printf '4 0004 0 0\n16 0000 0 0\n'; done
        printf '3 0100 0 0\n200 0000 0 0\n30 0000 0 127\n6 0000 127 0\n10 0000 0 0\n3 0100 0 0\n60 0000 0 0\n3 1000 0 0\n'
        tail -n +2 "$press"; } >"$p"
      MELEE_TEST_EVENTS_UNLOCKED=1 run "ev$e" "$p" 14000; rc=$?
      idx=$(grep -m1 "^play: tick [0-9]*: event" "$out/ev$e.log" | awk '{print $NF}')
      if [ $rc -eq 0 ] && [ "$idx" = "$((e - 1))" ] && grep -q "scene vs (mode 43" "$out/ev$e.log" && grep -q "scene menu (mode 1" "$out/ev$e.log"; then
        echo "OK event $e"; else echo "FAIL event $e: rc $rc, event index '$idx' (log $out/ev$e.log)"; fail=1; fi
    done ;;
  *) echo "FAIL $n: unknown run"; fail=1 ;;
  esac
done
exit $fail
