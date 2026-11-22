#!/bin/bash
# Menu-driven Slippi Direct or Unranked (plan step 8), loopback only, both peers rendered.
# usage: menu_direct_pair.sh OUTDIR [--mm-port P] [--frames N] [--mode direct|unranked] [--type-code] [--css]
# --css: both peers walk the menus to the online select (plan step 7): the pick searches through the mock while the
# select is up, each picks a fighter by pad script (*_css.txt) and locks in with Start; the match follows.
# --type-code (Direct): A types BBBB#002 on the name-entry keyboard (slippi_online_code.txt) instead of MELEE_SLIPPI_DIRECT_CODE.
# Peer A boots at the main menu with --slippi-menus on and walks 1P -> Online -> Direct (or Unranked) by pad script
# (tools/rust-port/menus/slippi_online*.txt); the pick relaunches the game window into `play --slippi-mm` (the normal
# renderer, audio and pads) against the local mock (mock_mm.py). Peer B is also `play --slippi-mm`, started straight
# into matchmaking (Direct: searching for A's code). Both play generated pads (--slippi-padgen) and write per-frame
# Slippi checksums; they must be equal. Windows are hidden.
# Test identities only (AAAA#001 / BBBB#002): no Slippi account, no user.json, no real server. At most two game
# instances at a time: B starts once A's menu process has handed over to its match process.
set -u
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"
ws="${GW_WORKSPACE:-E:/Projects/Melee Workspace}"; set -a; . "$ws/.env" >/dev/null 2>&1; set +a
B="${EXE:-$repo/build/cargo/${PROFILE:-fast}/melee.exe}"; O=$1; shift; mkdir -p "$O"
mmp=43170; frames=900; mode=direct; typed=0; css=0
while [ $# -gt 0 ]; do case "$1" in --mm-port) mmp=$2; shift ;; --frames) frames=$2; shift ;; --mode) mode=$2; shift ;; --type-code) typed=1 ;; --css) css=1 ;; esac; shift; done
export MELEE_INPUT=n MELEE_VOLUME=3 MELEE_WINDOW_HIDDEN=1 MELEE_PAD_IGNORE_ADAPTER=1 MELEE_WINDOW_UNFOCUSED=1 MELEE_KEYBOARD=0
rm -f "$O/a.csv" "$O/b.csv"
python -I "$here/mock_mm.py" --port $mmp > "$O/mm.log" 2>&1 & mm=$!
sleep 2
# A: the menus. Its relaunch inherits the MELEE_SLIPPI_* settings (the relaunched window is the netplay peer).
( cd "$repo/rust" && MELEE_WINDOW_TITLE="slippimenu:A:menu->$mode" MELEE_RETAIL_FIRST=1 MELEE_CARD=0 \
  MELEE_PAD_SCRIPT="$repo/tools/rust-port/menus/slippi_online$([ $mode = unranked ] && echo _unranked)$([ $typed = 1 ] && echo _code)$([ $css = 1 ] && echo _css).txt" \
  MELEE_SLIPPI_MM_URL="ws://127.0.0.1:$mmp/v1/queue" MELEE_SLIPPI_NP_PORT=$((mmp + 100)) MELEE_SLIPPI_TEST_CODE="AAAA#001" \
  MELEE_SLIPPI_DIRECT_CODE="$([ $typed = 1 ] || echo BBBB#002)" MELEE_SLIPPI_NP_FRAMES=$frames MELEE_SLIPPI_NP_OUT="$O/a.csv" MELEE_SLIPPI_NP_MAX_SECONDS=240 \
  MELEE_SLIPPI_NP_PADGEN=${PADGEN:-7} \
  timeout 180 "$B" play --skip-intro --max-frames $([ $css = 1 ] && echo 3000 || ([ $typed = 1 ] && echo 900 || echo 400)) --slippi-menus on > "$O/a-menu.log" 2>&1 ) &
apid=$!
if [ $css = 1 ]; then
  # B: also through the menus and the select at the same time (Direct: A's code from the env, no keyboard)
  ( cd "$repo/rust" && MELEE_WINDOW_TITLE="slippimenu:B:menu->$mode" MELEE_RETAIL_FIRST=1 MELEE_CARD=0 \
    MELEE_PAD_SCRIPT="$repo/tools/rust-port/menus/slippi_online_b$([ $mode = unranked ] && echo _unranked)_css.txt" \
    MELEE_SLIPPI_MM_URL="ws://127.0.0.1:$mmp/v1/queue" MELEE_SLIPPI_NP_PORT=$((mmp + 101)) MELEE_SLIPPI_TEST_CODE="BBBB#002" \
    MELEE_SLIPPI_DIRECT_CODE="AAAA#001" MELEE_SLIPPI_NP_FRAMES=$frames MELEE_SLIPPI_NP_OUT="$O/b.csv" MELEE_SLIPPI_NP_MAX_SECONDS=240 \
    MELEE_SLIPPI_NP_PADGEN=${PADGEN:-7} \
    timeout 180 "$B" play --skip-intro --max-frames 3000 --slippi-menus on > "$O/b-menu.log" 2>&1 )
  echo "B menu rc=$? : $(grep -E 'online (menu|css|select)|relaunch' "$O/b-menu.log" | tr '\n' ' ')"
fi
wait $apid
echo "A menu rc=$? : $(grep -E 'online menu|relaunch|connect code' "$O/a-menu.log" | tr '\n' ' ')"
# B: rendered, straight into matchmaking
[ $css = 1 ] || ( cd "$repo/rust" && MELEE_WINDOW_TITLE="slippimenu:B:$mode" MELEE_CARD=0 timeout 300 "$B" play --skip-intro \
  --slippi-mm "ws://127.0.0.1:$mmp/v1/queue" --slippi-port $((mmp + 101)) --slippi-code "BBBB#002" --slippi-mm-mode $mode \
  $([ $mode = direct ] && echo --slippi-search AAAA#001) --slippi-frames $frames --slippi-out "$O/b.csv" --slippi-max-seconds 240 \
  --slippi-padgen 7 > "$O/b.log" 2>&1 )
[ $css = 1 ] || echo "B rc=$?"
# --css: each match window writes its csv; wait for both (bounded)
[ $css = 1 ] && for _ in $(seq 1 90); do [ -s "$O/a.csv" ] && [ -s "$O/b.csv" ] && [ "$(wc -l < "$O/a.csv")" -gt "$frames" ] && [ "$(wc -l < "$O/b.csv")" -gt "$frames" ] && break; sleep 2; done
# A's match window writes a.csv; wait for it (bounded)
for _ in $(seq 1 60); do [ -s "$O/a.csv" ] && break; sleep 2; done
sleep 3
kill $mm 2>/dev/null
grep -E "join from|paired|match" "$O/mm.log" | head -4
grep -h "Slippi:" "$O/b.log" | sed 's/^/B /'
python -I -c "import sys
a=open(sys.argv[1]).read().split()[1:]; b=open(sys.argv[2]).read().split()[1:]; n=min(len(a),len(b))
d=[x for x,y in zip(a,b) if x!=y]; u=len(set(r.split(',')[1] for r in a))
print(('CSV-EQUAL' if not d else 'CSV-DIFFER %d rows, first %s' % (len(d), d[0])), n, 'common rows,', u, 'distinct checksums')" "$O/a.csv" "$O/b.csv"
