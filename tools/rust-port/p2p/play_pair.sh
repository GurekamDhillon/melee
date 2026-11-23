#!/bin/bash
# play_pair.sh OUTDIR [FRAMES] [extra play args...]: two hidden windowed `melee play` instances started with the line
# Hero's launcher gives a P2P Direct match (--p2p-* ... --p2p-connect-seconds 45 --p2p-games 1), on loopback, pads
# from the session's generator (--mu-test-padgen), the match ended after FRAMES final checksums (--mu-test-frames).
# Pass: both exit 0 and the two signed result files agree (check_results.py). Run under the game slots:
#   ~/melee_out/locks/game.sh 2 tools/rust-port/p2p/play_pair.sh OUT 1200
# B_WRAP="timeout -s KILL 30": B dies mid-match (A must still exit 0 and write a "disconnected" result).
# Never prints the disc path.
set -u
OUT=${1:?outdir}; FRAMES=${2:-1200}; shift; shift || true
mkdir -p "$OUT"; OUT=$(cd "$OUT" && pwd)
HERE=$(cd "$(dirname "$0")" && pwd)
MELEE=${MELEE:-$HOME/melee_rust/build/cargo-p2pdirect/release/melee}
if [ -z "${GW_ISO_VANILLA:-}" ]; then
  GW_ISO_VANILLA=$(find "$HOME/melee_linux_test" -maxdepth 1 -name '*Smash*.iso' | head -1)
fi
export GW_ISO_VANILLA
export MELEE_INPUT=n MELEE_PAD_IGNORE_ADAPTER=1 MELEE_WINDOW_HIDDEN=1 MELEE_WINDOW_UNFOCUSED=1 MELEE_KEYBOARD=0 MELEE_VOLUME=3 MELEE_CARD=0
PA=$((40000 + RANDOM % 10000)); PB=$((PA + 1))
SEED=$(printf '%08X' $((RANDOM * 65536 + RANDOM)))
IA="$OUT/a/lobby-peer-identity.json"; IB="$OUT/b/lobby-peer-identity.json"
mkdir -p "$OUT/a" "$OUT/b"; rm -f "$OUT"/a/res*.json "$OUT"/b/res*.json
"$MELEE" mu-netplay --p2p-port 1 --p2p-peer 127.0.0.1:1 --p2p-identity "$IA" --p2p-connect-seconds 1 >/dev/null 2>"$OUT/ida.log"
"$MELEE" mu-netplay --p2p-port 1 --p2p-peer 127.0.0.1:1 --p2p-identity "$IB" --p2p-connect-seconds 1 >/dev/null 2>"$OUT/idb.log"
KA=$(grep -o 'identity [0-9a-f]\{64\}' "$OUT/ida.log" | head -1 | cut -d' ' -f2)
KB=$(grep -o 'identity [0-9a-f]\{64\}' "$OUT/idb.log" | head -1 | cut -d' ' -f2)
line() { # slot port peer identity expect result
  echo --p2p-port "$2" --p2p-peer "127.0.0.1:$3" --p2p-slot "$1" --p2p-chars 2/0:9/1 --p2p-stage 31 --p2p-seed "$SEED" --p2p-delay 2 \
    --p2p-identity "$4" --p2p-expect "$5" --p2p-result "$6" --p2p-names "Alpha:Beta" --p2p-connect-seconds 45 --p2p-games 1
}
MELEE_WINDOW_TITLE="p2pdirect play A" "$MELEE" play --iso "$GW_ISO_VANILLA" --skip-intro --max-frames 20000 \
  $(line 0 $PA $PB "$IA" "$KB" "$OUT/a/res.json") --mu-test-padgen 7 --mu-test-frames "$FRAMES" "$@" >"$OUT/a.out" 2>"$OUT/a.log" &
A=$!
sleep "${B_DELAY:-2}"
MELEE_WINDOW_TITLE="p2pdirect play B" ${B_WRAP:-} "$MELEE" play --iso "$GW_ISO_VANILLA" --skip-intro --max-frames 20000 \
  $(line 1 $PB $PA "$IB" "$KA" "$OUT/b/res.json") --mu-test-padgen 7 --mu-test-frames "$FRAMES" "$@" >"$OUT/b.out" 2>"$OUT/b.log" &
B=$!
wait $A; RA=$?
wait $B; RB=$?
grep -h "P2P\|p2p:" "$OUT/a.log" "$OUT/b.log" | grep -E "final checksums|DESYNC|could not|result file|connected" | sed "s|$OUT/||g"
echo "play_pair: exit A=$RA B=$RB"
python3 -I "$HERE/check_results.py" "$OUT/a/res.json" "$OUT/b/res.json" "$KA" "$KB" && [ $RA -eq 0 ] && [ $RB -eq 0 ]
