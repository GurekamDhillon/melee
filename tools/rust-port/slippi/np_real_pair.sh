#!/bin/bash
# SPDX-License-Identifier: MIT
# Engine vs engine through Slippi's real ENet matchmaking (mm.slippi.gg:43113), direct mode, two real accounts.
#   np_real_pair.sh OUTDIR USER1.json USER2.json [-- engine args]
# USERn.json are COPIES of two accounts' Slippi user.json. Their fields are never printed: each side searches the
# other's connect code, read here into a variable and passed on the command line only. Player order is the server's.
# THIS USES THE REAL NETWORK (Slippi's matchmaking service and, for the game itself, the addresses it hands out).
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"
ws="${GW_WORKSPACE:-E:/Projects/Melee Workspace}"; set -a; . "$ws/.env"; set +a
O=$1; U1=$2; U2=$3; shift 3; [ "$1" = "--" ] && shift
mkdir -p "$O"; O="$(cd "$O" && pwd)"
B="${EXE:-$repo/build/cargo/release/melee.exe}"
code() { python -I -c 'import json,sys; print(json.load(open(sys.argv[1],encoding="utf-8"))["connectCode"])' "$1"; }
C1=$(code "$U1"); C2=$(code "$U2")
export MELEE_VOLUME=3 MELEE_WINDOW_UNFOCUSED=1 MELEE_PAD_IGNORE_ADAPTER=1
P1=${PORT1:-$((41000 + RANDOM % 5000))}; P2=$((P1 + 5000))
F=${FRAMES:-2400}
MELEE_WINDOW_TITLE="slippi5:engine:acct1:real" "$B" slippi-netplay --mm enet://mm.slippi.gg:43113 --mm-real --user-json "$U1" \
  --search "$C2" --port $P1 --frames $F --max-seconds ${MAXS:-300} --out "$O/a.csv" --char 2 "$@" > "$O/a.log" 2>&1 &
a=$!
sleep 2
MELEE_WINDOW_TITLE="slippi5:engine:acct2:real" "$B" slippi-netplay --mm enet://mm.slippi.gg:43113 --mm-real --user-json "$U2" \
  --search "$C1" --port $P2 --frames $F --max-seconds ${MAXS:-300} --out "$O/b.csv" --char 0 "$@" > "$O/b.log" 2>&1 &
b=$!
wait $a; ra=$?; wait $b; rb=$?
echo "rc a=$ra b=$rb"
