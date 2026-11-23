#!/bin/bash
# slippi_pair.sh OUTDIR [FRAMES] [extra play args...]: two hidden windowed `play --slippi-mm` peers (Slippi Direct
# against the local mock matchmaker, loopback only, test codes AAAA#001 / BBBB#002, generated pads) with the input-
# latency probe on (MELEE_INPUTLAT_CSV). Prints the CSV check and the join (join.py). Extra args go to both, e.g.
# --slippi-lag-ms 30. Run under the game slots: ~/melee_out/locks/game.sh 2 tools/rust-port/inputlat/slippi_pair.sh OUT
# Never prints the disc path.
set -u
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"
O=${1:?outdir}; FRAMES=${2:-1200}; shift; shift || true
mkdir -p "$O"; O=$(cd "$O" && pwd); rm -f "$O"/*.csv
MELEE=${MELEE:-$HOME/melee_rust/build/cargo-p2pdirect/release/melee}
if [ -z "${GW_ISO_VANILLA:-}" ]; then GW_ISO_VANILLA=$(find "$HOME/melee_linux_test" -maxdepth 1 -name '*Smash*.iso' | head -1); fi
export GW_ISO_VANILLA MELEE_INPUT=n MELEE_PAD_IGNORE_ADAPTER=1 MELEE_WINDOW_HIDDEN=1 MELEE_WINDOW_UNFOCUSED=1 MELEE_KEYBOARD=0 MELEE_VOLUME=3 MELEE_CARD=0
mmp=$((43000 + RANDOM % 2000))
python3 -I "$repo/tools/rust-port/slippi/mock_mm.py" --port $mmp > "$O/mm.log" 2>&1 & mm=$!
sleep 1
side() { # name code other port
  MELEE_WINDOW_TITLE="inputlat slippi $1" MELEE_INPUTLAT_CSV="$O/lat_$1.csv" "$MELEE" play --iso "$GW_ISO_VANILLA" --skip-intro \
    --slippi-mm "ws://127.0.0.1:$mmp/v1/queue" --slippi-port $4 --slippi-code "$2" --slippi-mm-mode direct --slippi-search "$3" \
    --slippi-frames "$FRAMES" --slippi-out "$O/$1.csv" --slippi-max-seconds 240 --slippi-padgen 7 "${@:5}" > "$O/$1.log" 2>&1
}
side a "AAAA#001" "BBBB#002" $((mmp + 100)) "$@" & A=$!
side b "BBBB#002" "AAAA#001" $((mmp + 101)) "$@" & B=$!
wait $A; RA=$?; wait $B; RB=$?
kill $mm 2>/dev/null; wait $mm 2>/dev/null
echo "slippi_pair: exit A=$RA B=$RB"
python3 -I -c "import sys
a=open(sys.argv[1]).read().split()[1:]; b=open(sys.argv[2]).read().split()[1:]; n=min(len(a),len(b))
d=[x for x,y in zip(a[:n],b[:n]) if x!=y]
print(('CSV-EQUAL' if not d else 'CSV-DIFFER %d rows' % len(d)), n, 'common rows')" "$O/a.csv" "$O/b.csv"
grep -h "rollbacks" "$O/a.log" "$O/b.log" | grep "final hashes" | sed 's/.*rollbacks/rollbacks/; s/, session.*//'
python3 -I "$here/join.py" "$O/lat_a.csv" "$O/lat_b.csv"
