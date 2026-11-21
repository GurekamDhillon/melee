#!/bin/bash
# Slippi Direct as a set through the online select (Hero's mu_slippi_css.c rules), loopback only, both peers rendered
# and hidden, test identities only (AAAA#001 / BBBB#002; no account, no user.json, no real server).
# usage: menu_direct_set.sh OUTDIR [set|zdisc] [--mm-port P] [--frames N]
#  set:   both walk 1P -> Online -> Direct -> the online select and play a 2-game set. Each game the loser (the first
#         game: slot 0, the first queued) goes to the stage select on Start and picks (A seeks Dreamland 28, B Yoshi's
#         Story 8: the logs show whose pick a game played on); the winner locks in without a stage. After game 1 each
#         match window relaunches into the select (MELEE_SLIPPI_REMATCH) and game 2 follows. Each game's per-frame
#         Slippi checksums must be equal (a.csv/b.csv, a-g2.csv/b-g2.csv).
#  zdisc: A queues first (slot 0) and never presses Start; B locks in, tries to unselect (refused), holds Z: B goes back
#         to the online menu, A sees the opponent leave and goes back with B held.
# One game process per side at a time (each relaunch hands over), so at most two game instances.
set -u
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"
ws="${GW_WORKSPACE:-E:/Projects/Melee Workspace}"; set -a; . "$ws/.env" >/dev/null 2>&1; set +a
B="${EXE:-$repo/build/cargo/${PROFILE:-fast}/melee.exe}"; O=$1; shift; mkdir -p "$O"
sc=set; [ "${1:-}" = set ] || [ "${1:-}" = zdisc ] && { sc=$1; shift; }
mmp=43170; frames=960
while [ $# -gt 0 ]; do case "$1" in --mm-port) mmp=$2; shift ;; --frames) frames=$2; shift ;; esac; shift; done
export MELEE_INPUT=n MELEE_VOLUME=3 MELEE_WINDOW_HIDDEN=1 MELEE_PAD_IGNORE_ADAPTER=1 MELEE_WINDOW_UNFOCUSED=1 MELEE_KEYBOARD=0
M="$repo/tools/rust-port/menus"
rm -f "$O"/*.csv "$O"/*.log
python -I "$here/mock_mm.py" --port $mmp > "$O/mm.log" 2>&1 & mm=$!
sleep 2
side() { # side NAME CODE OTHER NPPORT SCRIPT [G2SCRIPT]
  ( cd "$repo/rust" && MELEE_WINDOW_TITLE="slippimenu:$1:$sc" MELEE_RETAIL_FIRST=1 MELEE_CARD=0 MELEE_PAD_SCRIPT="$5" \
    MELEE_SLIPPI_SET_PAD_SCRIPT_G2="${6:-}" MELEE_SLIPPI_SET_GAMES=2 MELEE_SLIPPI_SET_MAX_FRAMES=4000 \
    MELEE_SLIPPI_MM_URL="ws://127.0.0.1:$mmp/v1/queue" MELEE_SLIPPI_NP_PORT=$4 MELEE_SLIPPI_TEST_CODE="$2" \
    MELEE_SLIPPI_DIRECT_CODE="$3" MELEE_SLIPPI_NP_FRAMES=$frames MELEE_SLIPPI_NP_OUT="$O/$(echo $1 | tr AB ab).csv" \
    MELEE_SLIPPI_NP_MAX_SECONDS=200 MELEE_SLIPPI_NP_PADGEN=7 \
    timeout 200 "$B" play --skip-intro --max-frames 4000 --slippi-menus on >> "$O/$(echo $1 | tr AB ab)-menu.log" 2>&1 )
}
if [ $sc = set ]; then
  side A "AAAA#001" "BBBB#002" $((mmp + 100)) "$M/slippi_set_a_g1.txt" "$M/slippi_set_a_g2.txt" &
  sleep 1
  side B "BBBB#002" "AAAA#001" $((mmp + 101)) "$M/slippi_set_b_g1.txt" "$M/slippi_set_b_g2.txt" &
  wait
  # the later processes are the relaunched windows (each bounded: --slippi-max-seconds, --max-frames): wait for game 2
  for _ in $(seq 1 150); do [ -s "$O/a-g2.csv" ] && [ -s "$O/b-g2.csv" ] && break; sleep 2; done
else
  side A "AAAA#001" "BBBB#002" $((mmp + 100)) "$M/slippi_set_a_zdisc.txt" &
  sleep 10
  side B "BBBB#002" "AAAA#001" $((mmp + 101)) "$M/slippi_set_b_zdisc.txt" &
  wait
fi
sleep 3
kill $mm 2>/dev/null
grep -hE "pairing|match\b" "$O/mm.log" | head -3
for s in a b; do echo "== $s"; grep -E "online (css|select)|stage ext|selections exchanged|match end|Slippi: [0-9]|unselect|relaunch" "$O/$s-menu.log" | grep -v "exit request" | sed 's/^/  /'; done
cmp() { python -I -c "import sys
a=open(sys.argv[1]).read().split()[1:]; b=open(sys.argv[2]).read().split()[1:]; n=min(len(a),len(b))
d=[x for x,y in zip(a,b) if x!=y]; u=len(set(r.split(',')[1] for r in a))
print(sys.argv[3], ('CSV-EQUAL' if not d else 'CSV-DIFFER %d rows, first %s' % (len(d), d[0])), n, 'common rows,', u, 'distinct checksums')" "$1" "$2" "$3"; }
if [ $sc = set ]; then cmp "$O/a.csv" "$O/b.csv" game1; cmp "$O/a-g2.csv" "$O/b-g2.csv" game2; fi
