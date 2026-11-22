#!/bin/bash
# `--online-mode <mode>` (Hero's launcher restarting the game for YouGame, H:launcher.cpp take_yougame_restart): the
# launcher's command line must land straight in that mode's online select, no pad input. One instance at a time,
# hidden, test settings only; no network (the select is idle until Start).
# usage: online_mode.sh OUTDIR   (EXE=... to pick the build)
set -u
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"
ws="${GW_WORKSPACE:-E:/Projects/Melee Workspace}"; set -a; . "$ws/.env" >/dev/null 2>&1; set +a
B="${EXE:-$repo/build/cargo/${PROFILE:-fast}/melee.exe}"; O=$1; mkdir -p "$O"
export MELEE_INPUT=n MELEE_VOLUME=3 MELEE_WINDOW_HIDDEN=1 MELEE_PAD_IGNORE_ADAPTER=1 MELEE_WINDOW_UNFOCUSED=1 MELEE_KEYBOARD=0 MELEE_CARD=0 MELEE_RETAIL_FIRST=1
fail=0
for m in unranked direct ranked; do
  want=Unranked; [ $m = direct ] && want=Direct
  # an empty pad script: no input at all (agent rule: pad scripts only)
  printf '600 0000 0 0\n' > "$O/idle.txt"
  ( cd "$O" && MELEE_WINDOW_TITLE="ygroom:online-mode-$m" MELEE_PAD_SCRIPT="$O/idle.txt" MELEE_LOG_FILE="$O/$m.log" \
    timeout 120 "$B" --iso "$GW_ISO_VANILLA" --online-mode $m --max-frames 900 > "$O/$m.out" 2>&1 < /dev/null )
  log="$O/$m.log"; [ -s "$log" ] || log="$O/$m.out"
  grep -hE "online-mode|online menu|online css: entered|slippi menus" "$log" | sed 's/^/  /'
  if grep -q "straight to $want's online select" "$log" && grep -q "online css: entered ($want)" "$log"; then echo "$m: PASS ($want select)"; else echo "$m: FAIL"; fail=1; fi
done
exit $fail
