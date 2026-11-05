#!/bin/bash
# Regression (port/pt-fix): back out of the main menu to the title with B, press B on the title, then Start. Before the fix
# the title's PC-only B / Y shortcut opened the debug menu, which crashes (DevText_StrLen, read of 0x1800). Runs with the
# in-repo wasm mods loaded (browser + lab, built by tools/rust-port/modhost/build_mods.sh when present) and MELEE_WIDESCREEN=0,
# as in the play-test report. Exit 0 when the title's B leads to the retail attract movie, with no debug-menu scene and no crash.
#   bash tools/rust-port/play/title_back_b.sh [--windowed]   (TB_DEBUG_MENU=1 turns the shortcut back on: the crash reproduces)
set -u
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"
ws="${GW_WORKSPACE:-E:/Projects/Melee Workspace}"
if [ -z "${GW_ISO_VANILLA:-}" ] && [ -f "$ws/.env" ]; then set -a; . "$ws/.env"; set +a; fi
mode=--headless; [ "${1:-}" = --windowed ] && mode=""
exe="$repo/build/cargo/${PROFILE:-release}/melee.exe"
out="${TEMP:-/tmp}/title_back_b.$$"; mkdir -p "$out"
mods=""
if [ -f "$repo/build/mods/browser.wasm" ] && [ -f "$repo/build/mods/lab.wasm" ]; then
  mods="$out/mods"; mkdir -p "$mods"; cp "$repo/build/mods/browser.wasm" "$repo/build/mods/lab.wasm" "$mods/"
fi
MELEE_PAD_IGNORE_ADAPTER=1 MELEE_WINDOW_UNFOCUSED=1 MELEE_CARD=0 MELEE_VOLUME=3 MELEE_WIDESCREEN=0 MELEE_DEBUG_MENU=${TB_DEBUG_MENU:-} \
  MELEE_WASM_MODS="$mods" MELEE_WINDOW_TITLE="title_back_b" MELEE_PAD_SCRIPT="$here/title_back_b.txt" \
  "$exe" play $mode --no-audio --skip-intro --iso "$GW_ISO_VANILLA" --max-frames 800 >"$out/run.log" 2>&1; rc=$?
grep "^play: tick [0-9]*: scene \|^modhost: ui: " "$out/run.log"
titles=$(grep -c "^play: tick [0-9]*: scene title" "$out/run.log")
last=$(grep "^play: tick [0-9]*: scene " "$out/run.log" | tail -1)
# pass: back on the title once (B from the main menu), then the title's B leads to the retail attract movie (mode 24)
if [ $rc -ne 0 ] || grep -Eq "crash report|scene [a-z]* \(mode (6|7|14)," "$out/run.log" || [ "$titles" -lt 2 ] || ! echo "$last" | grep -q "(mode 24,"; then
  echo "FAIL title_back_b: rc $rc, title scenes $titles, last: $last (log $out/run.log)"; exit 1
fi
echo "OK title_back_b"
