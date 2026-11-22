#!/bin/bash
# One window per side for the whole online loop (slippi_link::inprocess): menu_unranked_back.sh's run, plus the
# process list sampled every second (each side's game process must be the same one from boot to the select after
# the match: no close / reopen), the transition timings (no tick over 16 ms) and no relaunch in the logs.
# Slippi Unranked through the online select, then back (Slippi Dolphin / Hero: the results, then the online select,
# where Start searches again). Loopback only, both peers rendered and hidden, test identities only (AAAA#001 /
# BBBB#002; no account, no user.json, no real server).
# usage: menu_unranked_back.sh OUTDIR [--mm-port P] [--stocks N]
# Both walk 1P -> Online -> Unranked -> the online select, search, lock in and play one match on generated pads
# (MELEE_SLIPPI_NP_PADGEN) with N stocks (MELEE_SLIPPI_TEST_STOCKS, default 1) until it ends on stocks (or time). Each
# match window shows the results (generated A presses), then relaunches into the menus' Unranked online select
# (MELEE_SLIPPI_BACK); its pad script picks a fighter and presses Start once: the mock must log a second join per side.
# Shots: OUTDIR/shots-a, OUTDIR/shots-b (every scene at ages 90 and 240; the results come from the match window).
# One game process per side at a time (each relaunch hands over), so at most two game instances.
set -u
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"
ws="${GW_WORKSPACE:-E:/Projects/Melee Workspace}"; set -a; . "$ws/.env" >/dev/null 2>&1; set +a
B="${EXE:-$repo/build/cargo/${PROFILE:-fast}/melee.exe}"; O=$1; shift; mkdir -p "$O"
mmp=43270; stocks=1
while [ $# -gt 0 ]; do case "$1" in --mm-port) mmp=$2; shift ;; --stocks) stocks=$2; shift ;; esac; shift; done
export MELEE_INPUT=n MELEE_VOLUME=3 MELEE_WINDOW_HIDDEN=1 MELEE_PAD_IGNORE_ADAPTER=1 MELEE_WINDOW_UNFOCUSED=1 MELEE_KEYBOARD=0
M="$repo/tools/rust-port/menus"
rm -rf "$O"/*.csv "$O"/*.log "$O"/shots-a "$O"/shots-b; mkdir -p "$O/shots-a" "$O/shots-b"
python -I "$here/mock_mm.py" --port $mmp > "$O/mm.log" 2>&1 < /dev/null & mm=$!
Bw="$(cygpath -w "$B" 2>/dev/null || echo "$B")"
games() { powershell -NoProfile -Command "Get-CimInstance Win32_Process -Filter \"Name='melee.exe'\" | Where-Object { \$_.ExecutablePath -eq '$Bw' -and \$_.CommandLine -like '*--slippi-menus*' } | ForEach-Object { \$_.ProcessId }" 2>/dev/null | tr -d '\r'; }
sleep 2
side() { # side NAME CODE NPPORT SCRIPT BACKSCRIPT
  local s; s=$(echo $1 | tr AB ab)
  ( cd "$repo/rust" && MELEE_WINDOW_TITLE="slippiseamless:$1:unranked" MELEE_RETAIL_FIRST=1 MELEE_CARD=0 MELEE_PAD_SCRIPT="$4" \
    MELEE_SLIPPI_BACK_PAD_SCRIPT="$5" MELEE_SLIPPI_SET_MAX_FRAMES=1500 MELEE_SLIPPI_TEST_STOCKS=$stocks \
    MELEE_SLIPPI_MM_URL="ws://127.0.0.1:$mmp/v1/queue" MELEE_SLIPPI_NP_PORT=$3 MELEE_SLIPPI_TEST_CODE="$2" \
    MELEE_SLIPPI_NP_OUT="$O/$s.csv" MELEE_SLIPPI_NP_MAX_SECONDS=420 MELEE_SLIPPI_NP_PADGEN=7 \
    timeout 240 "$B" play --skip-intro --max-frames 4000 --slippi-menus on --shot-dir "$O/shots-$s" --shot-ages 90,240 >> "$O/$s-menu.log" 2>&1 < /dev/null )
}
side A "AAAA#001" $((mmp + 100)) "$M/slippi_online_unranked_css.txt" "$M/slippi_back_a.txt" & pa=$!
sleep 1
side B "BBBB#002" $((mmp + 101)) "$M/slippi_online_b_unranked_css.txt" "$M/slippi_back_b.txt" & pb=$!
( for _ in $(seq 1 300); do echo "$(date +%s) $(games | tr '
' ' ')"; sleep 1; done ) > "$O/pids.log" 2>/dev/null & watch=$!
wait $pa $pb
kill $watch 2>/dev/null
left=""
for _ in $(seq 1 200); do left=$(games); [ -z "$left" ] && break; sleep 3; done
if [ -n "$left" ]; then echo "deadline: stopping this run's game processes: $left" | tr '\n' ' '; echo
  for p in $left; do taskkill //F //PID $p > /dev/null 2>&1; done; fi
sleep 1
kill $mm 2>/dev/null; wait $mm 2>/dev/null
grep -hE "join from|pairing" "$O/mm.log" | sed -E 's/[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+/<ip>/g'
for s in a b; do echo "== $s"; grep -E "online (css|select|menu)|match end|match over|Slippi: [0-9]|relaunch|scene results|stall at|slow tick|closed after|transition|HARD desyncs" "$O/$s-menu.log" | grep -v "exit request" | sed 's/^/  /'; done
fail=0; bad() { echo "FAIL: $*"; fail=1; }
if grep -hE "HARD desyncs" "$O"/a-menu.log "$O"/b-menu.log | grep -qvE " 0 HARD desyncs"; then bad "a desync"; fi
for s in a b; do
  grep -q "match over at game frame" "$O/$s-menu.log" || bad "$s: no match end"
  grep -qE "scene results" "$O/$s-menu.log" || bad "$s: no results scene"
  grep -q "the results are over, back to the online select" "$O/$s-menu.log" || bad "$s: no return after the results"
  grep -q "back from the match, Unranked again" "$O/$s-menu.log" || bad "$s: the menus did not land on the Unranked select"
done
grep -q "no results after the match" "$O/a-menu.log" "$O/b-menu.log" && bad "the results did not end on a press (two-minute fallback)"
for c in AAAA#001 BBBB#002; do n=$(grep -c "join from $c" "$O/mm.log"); [ "$n" -ge 2 ] || bad "$c: $n joins (want 2)"; done
[ -z "$(games)" ] || bad "game processes still up"
# one process per side for the whole loop: every pid seen is one of the first two, and never more than two at once
pids=$(awk '{for(i=2;i<=NF;i++) print $i}' "$O/pids.log" | sort -u)
np=$(echo "$pids" | grep -c . ); echo "distinct game processes seen: $np (want 2)"; [ "$np" -le 2 ] || bad "a window closed and reopened ($np processes)"
grep -qE "relaunching for|online select: relaunch" "$O/a-menu.log" "$O/b-menu.log" && bad "a relaunch in the logs"
for s in a b; do grep -q "the match in this window (no relaunch" "$O/$s-menu.log" || bad "$s: the match did not start in this window"; done
grep -hE "transition (select|match)" "$O/a-menu.log" "$O/b-menu.log" | sed 's/^/  /'
grep -hE "transition .*took" "$O/a-menu.log" "$O/b-menu.log" | grep -q . && bad "a transition tick over 16 ms"
if [ $fail = 0 ]; then echo "PASS"; exit 0; else echo "FAIL"; exit 1; fi
