#!/bin/bash
# Three Unranked games in a row against the same opponent on one link (owner bug Nov 27: against real Slippi Dolphin
# every set ended after game 1). Two of our builds through mock_mm, loopback only, both hidden, test identities only
# (AAAA#001 / BBBB#002; no account, no user.json, no real server), one window per side for the whole loop.
# Slippi Dolphin's order at a game's end is reproduced: its VSSceneDecide goes back to the CSS the frame the VS scene
# ends and its player locks in with Start there, while our side still lingers in its match. Side A lingers 5 s
# (MELEE_SLIPPI_TEST_LINGER_MS) so B is back on its select and locks in first, on the match's link, every game.
# usage: menu_rematch_three.sh OUTDIR [--mm-port P] [--stocks N] [--linger-ms MS]
# Each game: GAME!, back to the online select linked (no new search, no new connection), pick, Start. After game 3
# both are back on the select for game 4, still linked (MELEE_SLIPPI_SET_GAMES=4; no game-4 script: idle until the
# frame cap ends the run).
set -u
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"
ws="${GW_WORKSPACE:-E:/Projects/Melee Workspace}"; set -a; . "$ws/.env" >/dev/null 2>&1; set +a
B="${EXE:-$repo/build/cargo/${PROFILE:-fast}/melee.exe}"; O=$1; shift; mkdir -p "$O"
mmp=43370; stocks=1; linger=5000
while [ $# -gt 0 ]; do case "$1" in --mm-port) mmp=$2; shift ;; --stocks) stocks=$2; shift ;; --linger-ms) linger=$2; shift ;; esac; shift; done
export MELEE_INPUT=n MELEE_VOLUME=3 MELEE_WINDOW_HIDDEN=1 MELEE_PAD_IGNORE_ADAPTER=1 MELEE_WINDOW_UNFOCUSED=1 MELEE_KEYBOARD=0
M="$repo/tools/rust-port/menus"
rm -rf "$O"/*.csv "$O"/*.log "$O"/shots-a "$O"/shots-b; mkdir -p "$O/shots-a" "$O/shots-b"
python -I "$here/mock_mm.py" --port $mmp > "$O/mm.log" 2>&1 < /dev/null & mm=$!
Bw="$(cygpath -w "$B" 2>/dev/null || echo "$B")"
games() { powershell -NoProfile -Command "Get-CimInstance Win32_Process -Filter \"Name='melee.exe'\" | Where-Object { \$_.ExecutablePath -eq '$Bw' -and \$_.CommandLine -like '*--slippi-menus*' } | ForEach-Object { \$_.ProcessId }" 2>/dev/null | tr -d '\r'; }
sleep 2
side() { # side NAME CODE NPPORT SCRIPT BACKSCRIPT LINGER
  local s; s=$(echo $1 | tr AB ab)
  ( cd "$repo/rust" && MELEE_WINDOW_TITLE="rematch agent: $1 three-game set" MELEE_RETAIL_FIRST=1 MELEE_CARD=0 MELEE_PAD_SCRIPT="$4" \
    MELEE_SLIPPI_SET_PAD_SCRIPT_G2="$5" MELEE_SLIPPI_SET_PAD_SCRIPT_G3="$5" MELEE_SLIPPI_SET_GAMES=4 MELEE_SLIPPI_SET_MAX_FRAMES=1500 \
    MELEE_SLIPPI_TEST_STOCKS=$stocks MELEE_SLIPPI_TEST_LINGER_MS=$6 \
    MELEE_SLIPPI_MM_URL="ws://127.0.0.1:$mmp/v1/queue" MELEE_SLIPPI_NP_PORT=$3 MELEE_SLIPPI_TEST_CODE="$2" \
    MELEE_SLIPPI_NP_OUT="$O/$s.csv" MELEE_SLIPPI_NP_MAX_SECONDS=600 MELEE_SLIPPI_NP_PADGEN=7 \
    timeout 600 "$B" play --skip-intro --max-frames 4800 --slippi-menus on --shot-dir "$O/shots-$s" --shot-ages 90,240 >> "$O/$s-menu.log" 2>&1 < /dev/null )
}
side A "AAAA#001" $((mmp + 100)) "$M/slippi_online_unranked_css_1start.txt" "$M/slippi_back_a.txt" "$linger" & pa=$!
sleep 1
side B "BBBB#002" $((mmp + 101)) "$M/slippi_online_b_unranked_css_1start.txt" "$M/slippi_back_b.txt" 1500 & pb=$!
# below-normal priority for this run's game processes (other agents' sweeps share the machine)
prio() { powershell -NoProfile -Command "Get-CimInstance Win32_Process -Filter \"Name='melee.exe'\" | Where-Object { \$_.ExecutablePath -eq '$Bw' -and \$_.CommandLine -like '*--slippi-menus*' } | ForEach-Object { try { (Get-Process -Id \$_.ProcessId).PriorityClass = 'BelowNormal' } catch {} }" > /dev/null 2>&1; }
( for _ in $(seq 1 600); do prio; echo "$(date +%s) $(games | tr '
' ' ')"; sleep 1; done ) > "$O/pids.log" 2>/dev/null & watch=$!
wait $pa $pb
kill $watch 2>/dev/null
left=""
for _ in $(seq 1 100); do left=$(games); [ -z "$left" ] && break; sleep 3; done
if [ -n "$left" ]; then echo "deadline: stopping this run's game processes: $left" | tr '\n' ' '; echo
  for p in $left; do taskkill //F //PID $p > /dev/null 2>&1; done; fi
sleep 1
kill $mm 2>/dev/null; wait $mm 2>/dev/null
grep -hE "join from|pairing" "$O/mm.log" | sed -E 's/[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+/<ip>/g'
for s in a b; do echo "== $s"; grep -E "online (css|select)|rematch|link is down|locked in for the next game|match end|match over|HARD desyncs|transition" "$O/$s-menu.log" | grep -v "exit request" | sed 's/^/  /'; done
fail=0; bad() { echo "FAIL: $*"; fail=1; }
if grep -hE "HARD desyncs" "$O"/a-menu.log "$O"/b-menu.log | grep -qvE " 0 HARD desyncs"; then bad "a desync"; fi
for s in a b; do
  n=$(grep -c "match over at game frame" "$O/$s-menu.log"); [ "$n" -eq 3 ] || bad "$s: $n matches (want 3)"
  grep -q "back to the select for game 2" "$O/$s-menu.log" || bad "$s: not back on the select, linked, after game 1"
  grep -q "back to the select for game 3" "$O/$s-menu.log" || bad "$s: not back on the select, linked, after game 2"
  g4=$(grep -n "back to the select for game 4" "$O/$s-menu.log" | head -1 | cut -d: -f1)
  [ -n "$g4" ] || bad "$s: not back on the select, linked, after game 3"
  n=$(grep -c "the set's link taken over (Connected)" "$O/$s-menu.log"); [ "$n" -eq 3 ] || bad "$s: $n rematch selects took a live link (want 3)"
  # inside the set: through the select for game 4 (the run ends there on its frame cap; the side that exits first
  # closes the link then)
  head -n "${g4:-999999}" "$O/$s-menu.log" | grep -qE "the opponent disconnected|the opponent left|link is down|link taken over \((Disconnected|Failed)" && bad "$s: the link dropped inside the set"
done
# the Dolphin order: B's next lock-in reached A while A's match still lingered, at the end of games 1 and 2
n=$(grep -c "locked in for the next game before this match was over" "$O/a-menu.log"); [ "$n" -ge 2 ] || bad "A: the peer's early lock-in seen $n times (want 2: the scenario did not happen)"
n=$(grep -c "join from AAAA#001" "$O/mm.log"); [ "$n" -eq 1 ] || bad "A: $n joins (want 1: three games on one link)"
n=$(grep -c "join from BBBB#002" "$O/mm.log"); [ "$n" -eq 1 ] || bad "B: $n joins (want 1: three games on one link)"
[ -z "$(games)" ] || bad "game processes still up"
pids=$(awk '{for(i=2;i<=NF;i++) print $i}' "$O/pids.log" | sort -u)
np=$(echo "$pids" | grep -c . ); echo "distinct game processes seen: $np (want 2)"; [ "$np" -le 2 ] || bad "a window closed and reopened ($np processes)"
if [ $fail = 0 ]; then echo "PASS"; exit 0; else echo "FAIL"; exit 1; fi
