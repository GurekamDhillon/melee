#!/bin/bash
# SPDX-License-Identifier: MIT
# Slippi Dolphin on Slippi's real ENet matchmaking (mm.slippi.gg:43113), direct mode, our own two accounts.
#   np_real_dolphin_pair.sh OUTDIR MODE [-- engine args]
#   MODE dd: Dolphin (acct 1) vs Dolphin (acct 2); de: Dolphin (acct 1) vs engine (acct 2); ed: engine (acct 1) vs
#   Dolphin (acct 2); eh: as ed, the engine queued first (the decider).
# THIS USES THE REAL NETWORK (Slippi's matchmaking service, then the addresses it hands out).
# Dolphin is the enet_mm.patch build in $DOL (default _build/dolphin-slippi6) with throwaway user dirs userR1/userR2
# made from copies of the two account files (Slippi/user.json, and direct-codes.json holding the other's code, which
# the menu driver takes from the autocomplete). No account field is printed: the engine's search code is read into a
# variable and passed on its command line only. Checksums: OUTDIR/checksums_R1.txt / checksums_R2.txt (Dolphin),
# OUTDIR/engine.csv (engine). Only processes started here are stopped.
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"
ws="${GW_WORKSPACE:-E:/Projects/Melee Workspace}"; set -a; . "$ws/.env"; set +a
O=$1; M=$2; shift 2; [ "$1" = "--" ] && shift
EA=("$@")
mkdir -p "$O"; O="$(cd "$O" && pwd)"; W=$(cygpath -w "$O")
DOL="${DOL:-$ws/_build/dolphin-slippi6}"; DW=$(cygpath -w "$DOL")
B="${EXE:-$repo/build/cargo/release/melee.exe}"
AV="${APPV:-4.0.0-mainline-beta.20}"   # the engine's default appVersion, sent by both clients
F=${FRAMES:-2400}
export MELEE_VOLUME=3 MELEE_WINDOW_UNFOCUSED=1 MELEE_PAD_IGNORE_ADAPTER=1
code() { python -I -c 'import json,sys; print(json.load(open(sys.argv[1],encoding="utf-8"))["connectCode"])' "$DOL/user$1/Slippi/user.json"; }
pids=(); drvs=()
dolphin() {  # NAME CSS PADSEED
  MELEE_WINDOW_TITLE="slippi6:dolphin:$1:real" powershell -NoProfile -File "$here/run_np_dolphin.ps1" -Name $1 -Out "$W" -Dir "$DW" \
    -MmEnet mm.slippi.gg:43113 -RealNet -AppVersion "$AV" > "$O/dolphin_$1.pid" 2>&1
  pids+=("$(tr -d '\r\n' < "$O/dolphin_$1.pid")")
  python -I "$here/dolphin_drive.py" --name $1 --state "$W\\state_$1.txt" --play "$2" --pads $3 --frames $((F + 300)) --timeout ${MAXS:-420} > "$O/drive_$1.log" 2>&1 &
  drvs+=($!)
}
engine() {  # USER SEARCHUSER CHAR
  MELEE_WINDOW_TITLE="slippi6:engine:$1:real" "$B" slippi-netplay --mm enet://mm.slippi.gg:43113 --mm-real \
    --user-json "$DOL/user$1/Slippi/user.json" --search "$(code $2)" --port $((41000 + RANDOM % 5000)) --frames $F \
    --max-seconds ${MAXS:-420} --app-version "$AV" --out "$O/engine.csv" --char $3 "${EA[@]}" > "$O/engine.log" 2>&1
}
case "$M" in
  dd) dolphin R1 0,15 11; sleep 3; dolphin R2 fox 22 ;;
  de) dolphin R1 0,15 11; sleep 8; engine R2 R1 2 ;;
  ed) dolphin R2 0,15 22; sleep 8; engine R1 R2 2 ;;
  eh) engine R1 R2 2 & e=$!; sleep 8; dolphin R2 0,15 22; wait $e ;;  # engine first: the engine is the decider
  *) echo "mode dd|de|ed|eh" >&2; exit 2 ;;
esac
for d in "${drvs[@]}"; do wait $d; done
for p in "${pids[@]}"; do powershell -NoProfile -Command "Stop-Process -Id $p -Force -ErrorAction SilentlyContinue"; done
for f in "$O"/drive_*.log; do echo "$(basename $f): $(tail -n 1 "$f")"; done
[ -f "$O/engine.log" ] && grep "^melee: \(decider\|guest\|matched\)" "$O/engine.log" | tail -3
true
