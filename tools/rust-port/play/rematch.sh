#!/bin/bash
# Scripted `melee play` rematch tests (no OS input: MELEE_PAD_SCRIPT drives port 1, the adapter is ignored, the window never
# takes focus, and every run quits itself). Needs GW_ISO_VANILLA (the workspace .env).
#   bash tools/rust-port/play/rematch.sh [--windowed] [css|x5|all]
#   css: css_rematch_4cpu.txt  CSS -> 4 CPUs, 1-min time, items on, Brinstar -> results -> CSS -> 2v2 teams, 1 stock, Onett -> results
#   x5:  rematch_x5.txt        5 consecutive 4-CPU matches on Brinstar; prints the memory lines (working set per match)
# Exit 0 when each run reached its last results screen and quit cleanly.
set -u
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"
ws="${GW_WORKSPACE:-E:/Projects/Melee Workspace}"
if [ -z "${GW_ISO_VANILLA:-}" ] && [ -f "$ws/.env" ]; then set -a; . "$ws/.env"; set +a; fi
mode=--headless; which=all
for a in "$@"; do case "$a" in --windowed) mode="" ;; css|x5|all) which=$a ;; esac; done
exe="$repo/build/cargo/${PROFILE:-release}/melee.exe"
out="${TEMP:-/tmp}/rematch.$$"; mkdir -p "$out"
export MELEE_PAD_IGNORE_ADAPTER=1 MELEE_WINDOW_UNFOCUSED=1
fail=0
run() { # name script matches
  MELEE_STANDIN_MENUS=1 MELEE_PAD_SCRIPT="$here/$2" "$exe" play $mode --no-audio --skip-intro --iso "$GW_ISO_VANILLA" --time-secs 60 --quit-at-results \
    --quit-after-matches "$3" --max-frames 60000 >"$out/$1.log" 2>&1; rc=$?
  n=$(grep -c "^play: results of match" "$out/$1.log")
  grep "^play: \(loading\|results of\|mem after\)" "$out/$1.log"
  if [ $rc -ne 0 ] || [ "$n" -ne "$3" ]; then echo "FAIL $1: rc $rc, $n of $3 results (log $out/$1.log)"; fail=1; else echo "OK $1: $n matches"; fi
}
case $which in css|all) run css css_rematch_4cpu.txt 2 ;; esac
case $which in x5|all) run x5 rematch_x5.txt 5 ;; esac
exit $fail
