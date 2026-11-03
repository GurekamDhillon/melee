#!/bin/bash
# Scripted `melee play` runs of the 1P Event Match and Stadium modes from the retail menus (no OS input: MELEE_PAD_SCRIPT drives
# port 1, the adapter is ignored, the window never takes focus). Needs GW_ISO_VANILLA (the workspace .env).
#   bash tools/rust-port/play/onep_modes.sh [--windowed] [name ...]     (names: event1 event10 target homerun multiman_10 ...)
# Each run must reach the mode's match scene, come back to the menu and quit cleanly at --max-frames.
# Exit 0 when every run did.
set -u
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"
ws="${GW_WORKSPACE:-E:/Projects/Melee Workspace}"
if [ -z "${GW_ISO_VANILLA:-}" ] && [ -f "$ws/.env" ]; then set -a; . "$ws/.env"; set +a; fi
mode=--headless; names=()
for a in "$@"; do case "$a" in --windowed) mode="" ;; *) names+=("$a") ;; esac; done
# name: game mode (GameModeKind) and frames
all="event1:43:6500 event10:43:6500 target:15:4000 homerun:32:4500 multiman_10:33:4800 multiman_100:34:5500
multiman_3min:35:4800 multiman_15min:36:5000 multiman_endless:37:4800 multiman_cruel:38:4800"
[ ${#names[@]} -eq 0 ] && for r in $all; do names+=("${r%%:*}"); done
exe="$repo/build/cargo/${PROFILE:-release}/melee.exe"
out="${TEMP:-/tmp}/onep_modes.$$"; mkdir -p "$out"
export MELEE_PAD_IGNORE_ADAPTER=1 MELEE_WINDOW_UNFOCUSED=1 MELEE_CARD=0 MELEE_VOLUME=3 MELEE_MENU_LOG=1
fail=0
for n in "${names[@]}"; do
  r=$(printf '%s\n' $all | grep "^$n:") || { echo "FAIL $n: unknown run"; fail=1; continue; }
  gm=$(echo "$r" | cut -d: -f2); frames=$(echo "$r" | cut -d: -f3)
  MELEE_WINDOW_TITLE="onep_modes:$n" MELEE_PAD_SCRIPT="$here/onep_$n.txt" "$exe" play $mode --no-audio --skip-intro --iso "$GW_ISO_VANILLA" \
    --max-frames "$frames" >"$out/$n.log" 2>&1; rc=$?
  grep "^play: tick [0-9]*: \(scene\|event\)" "$out/$n.log" | sed -n '4,$p'
  played=$(grep -c "^play: tick [0-9]*: scene vs (mode $gm," "$out/$n.log")
  last=$(grep "^play: tick [0-9]*: scene " "$out/$n.log" | tail -1)
  if [ $rc -ne 0 ] || [ "$played" -lt 1 ] || ! echo "$last" | grep -q "scene menu"; then
    echo "FAIL $n: rc $rc, match scenes $played, last: $last (log $out/$n.log)"; fail=1
  else echo "OK $n"; fi
done
exit $fail
