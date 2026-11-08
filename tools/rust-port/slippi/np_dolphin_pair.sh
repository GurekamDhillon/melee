#!/bin/bash
# SPDX-License-Identifier: MIT
# Our engine vs the local-test Slippi Dolphin, paired through the local mock matchmaking server (loopback only).
#   np_dolphin_pair.sh OUTDIR [--mm-port P] [--lag-ms N] [--loss-pct P] [--frames N] [--pads SEED] [--css X,Y] [-- engine args]
# Dolphin (user dir _build/dolphin-netplay/userP3A, a throwaway test profile, code TSTA#001) joins first, so it is the
# decider and player 1; the engine (TSTB#001) is player 2. Lag/loss are applied by the mock's relay to both directions.
# Checksums: Dolphin's sent/received ones go to OUTDIR/checksums_P3A.txt (SLIPPI_CHECKSUM_LOG), the engine compares every
# finalized frame with the ones Dolphin sends and writes its own to OUTDIR/engine.csv. Only processes started here are
# stopped.
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"
ws="${GW_WORKSPACE:-E:/Projects/Melee Workspace}"; set -a; . "$ws/.env"; set +a
O=$1; shift; mkdir -p "$O"; O="$(cd "$O" && pwd)"
dn=${NP_NAME:-P3A}; mmp=43131; lag=0; loss=0; frames=3600; pads=1; css="-31,9"; eng=()
while [ $# -gt 0 ]; do
  case "$1" in
    --mm-port) mmp=$2; shift ;; --lag-ms) lag=$2; shift ;; --loss-pct) loss=$2; shift ;;
    --frames) frames=$2; shift ;; --pads) pads=$2; shift ;; --css) css=$2; shift ;;
    --) shift; eng=("$@"); break ;;
    *) echo "np_dolphin_pair: unknown option $1" >&2; exit 2 ;;
  esac; shift
done
B="${EXE:-$repo/build/cargo/release/melee.exe}"
export MELEE_VOLUME=3 MELEE_WINDOW_UNFOCUSED=1 MELEE_PAD_IGNORE_ADAPTER=1
python -I "$here/mock_mm.py" --port $mmp --stages ${NP_STAGES:-0x20} --lag-ms $lag --loss-pct $loss --capture "$O/cap.txt" > "$O/mm.log" 2>&1 &
mpid=$!
sleep 1
W=$(cygpath -w "$O")
MELEE_WINDOW_TITLE="slippi4:dolphin:$dn" powershell -NoProfile -File "$here/run_np_dolphin.ps1" -Name $dn -Out "$W" -MmPort $mmp ${NP_ADDRS:+-Addrs $NP_ADDRS} > "$O/dolphin.pid" 2>&1 &
for i in $(seq 100); do [ -s "$O/dolphin.pid" ] && break; sleep 0.2; done
dpid=$(tr -d '\r\n' < "$O/dolphin.pid")
echo "dolphin pid $dpid"
python -I "$here/dolphin_drive.py" --name $dn --state "$W\\state_$dn.txt" --play "$css" --pads $pads --frames $((frames + 300)) --timeout 300 > "$O/drive.log" 2>&1 &
drv=$!
# the engine joins once Dolphin is searching (it must be second in the queue to be player 2)
for i in $(seq 600); do grep -q "join from TSTA" "$O/mm.log" && break; sleep 0.5; done
MELEE_WINDOW_TITLE="slippi4:engine:vs-dolphin" "$B" slippi-netplay --mm ws://127.0.0.1:$mmp/v1/queue --port $((mmp + 100)) \
  --code TSTB#001 --search TSTA#001 --frames $frames --max-seconds ${MAXS:-400} --out "$O/engine.csv" "${eng[@]}" > "$O/engine.log" 2>&1
erc=$?
wait $drv
powershell -NoProfile -Command "Stop-Process -Id $dpid -Force -ErrorAction SilentlyContinue"
kill $mpid 2>/dev/null
grep -v '"type": *"status"\|"type":"status"' "$O/engine.log" | grep "^melee: \(decider\|guest\|matched\)" | tail -4
tail -2 "$O/drive.log"
echo "engine rc=$erc"
