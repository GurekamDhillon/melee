#!/bin/bash
# SPDX-License-Identifier: MIT
# Slippi Dolphin on Slippi's real ENet matchmaking (mm.slippi.gg:43113), direct mode, our own two accounts.
#   np_real_dolphin_pair.sh OUTDIR MODE [-- engine args]
#   MODE dd: Dolphin (acct 1) vs Dolphin (acct 2); de: Dolphin (acct 1) vs engine (acct 2); ed: engine (acct 1) vs
#   Dolphin (acct 2); eh: as ed, the engine queued first; hd / hd2: Dolphin (acct 2 / acct 1) queues, the engine
#   (the other account) creates its ticket only after Dolphin's create-ticket-resp: the server makes the player who
#   completes the pair (the second ticket) the host, so the engine is the decider.
# Phase 7: the user dirs are made here (account copies, the template config of _build/dolphin-netplay/userP3B with
# SaveReplays on and ReplayDir OUTDIR/slp_R1|R2) and deleted at the end; the engine writes --trace OUTDIR/engine.trace
# (for `melee slippi-replay --slp`, the engine's own .slp of the game).
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
ACCT="$ws/_build/dolphin-netplay"
mkuser() {  # NAME ACCTNUM OTHERNUM: a throwaway user dir (never prints an account field)
  local u="$DOL/user$1"; rm -rf "$u"; mkdir -p "$u/Config" "$u/Slippi" "$O/slp_$1"
  cp "$ACCT/userP3B/Config/GCPadNew.ini" "$ACCT/userP3B/Config/Logger.ini" "$u/Config/"
  sed -e 's/^SaveReplays = .*/SaveReplays = True/' "$ACCT/userP3B/Config/Dolphin.ini" > "$u/Config/Dolphin.ini"
  local rd; rd="$(cygpath -w "$O/slp_$1")"
  RD="$rd" awk '{print} /^\[Slippi\]/{print "ReplayDir = " ENVIRON["RD"]; print "ReplayMonthlyFolders = False"}' "$u/Config/Dolphin.ini" > "$u/Config/d.tmp" && mv "$u/Config/d.tmp" "$u/Config/Dolphin.ini"
  cp "$ACCT/slippi-acct-$2/user.json" "$u/Slippi/user.json"
  python -I -c 'import json,sys; c=json.load(open(sys.argv[1],encoding="utf-8"))["connectCode"]; json.dump([{"connectCode":c,"lastPlayed":1}],open(sys.argv[2],"w"))' "$ACCT/slippi-acct-$3/user.json" "$u/Slippi/direct-codes.json"
}
cleanup() { for _ in 1 2 3 4 5 6 7 8 9 10; do rm -rf "$DOL/userR1" "$DOL/userR2" 2>/dev/null && [ ! -e "$DOL/userR1" ] && [ ! -e "$DOL/userR2" ] && return; sleep 1; done; echo "could not delete the user dirs (account copies) in $DOL" >&2; }
trap cleanup EXIT
mkuser R1 1 2; mkuser R2 2 1
waitticket() {  # NAME: Dolphin is searching (its driver confirmed the code; this build writes no log file, so the
  # create-ticket-resp itself is not visible): then HDWAIT more seconds for its ticket to reach the server
  for _ in $(seq 1 ${MAXS:-420}); do grep -q "^drive: searching" "$O/drive_$1.log" 2>/dev/null && return 0; sleep 1; done
  return 1
}
code() { python -I -c 'import json,sys; print(json.load(open(sys.argv[1],encoding="utf-8"))["connectCode"])' "$DOL/user$1/Slippi/user.json"; }
pids=(); drvs=()
dolphin() {  # NAME CSS PADSEED
  MELEE_WINDOW_TITLE="slippi7:dolphin:$1:real" powershell -NoProfile -File "$here/run_np_dolphin.ps1" -Name $1 -Out "$W" -Dir "$DW" \
    -MmEnet mm.slippi.gg:43113 -RealNet -AppVersion "$AV" > "$O/dolphin_$1.pid" 2>&1
  pids+=("$(tr -d '\r\n' < "$O/dolphin_$1.pid")")
  python -I "$here/dolphin_drive.py" --name $1 --state "$W\\state_$1.txt" --play "$2" --pads $3 --frames $((F + 300)) --timeout ${MAXS:-420} > "$O/drive_$1.log" 2>&1 &
  drvs+=($!)
}
engine() {  # USER SEARCHUSER CHAR
  MELEE_WINDOW_TITLE="slippi7:engine:$1:real" "$B" slippi-netplay --mm enet://mm.slippi.gg:43113 --mm-real \
    --user-json "$DOL/user$1/Slippi/user.json" --search "$(code $2)" --port $((41000 + RANDOM % 5000)) --frames $F \
    --max-seconds ${MAXS:-420} --app-version "$AV" --out "$O/engine.csv" --trace "$O/engine.trace" --char $3 "${EA[@]}" > "$O/engine.log" 2>&1
}
case "$M" in
  dd) dolphin R1 0,15 11; sleep 3; dolphin R2 fox 22 ;;
  de) dolphin R1 0,15 11; sleep 8; engine R2 R1 2 ;;
  ed) dolphin R2 0,15 22; sleep 8; engine R1 R2 2 ;;
  eh) engine R1 R2 2 & e=$!; sleep 8; dolphin R2 0,15 22; wait $e ;;  # engine first (it turned out the guest)
  hd) dolphin R2 ${CSS:-0,15} 22; waitticket R2 && sleep ${HDWAIT:-5}; engine R1 R2 ${ECHAR:-2} ;;
  hd2) dolphin R1 ${CSS:-0,15} 11; waitticket R1 && sleep ${HDWAIT:-5}; engine R2 R1 ${ECHAR:-2} ;;
  *) echo "mode dd|de|ed|eh|hd|hd2" >&2; exit 2 ;;
esac
for d in "${drvs[@]}"; do wait $d; done
# Dolphin writes the replay as the game ends; give it a moment before the stop
sleep 3
for p in "${pids[@]}"; do powershell -NoProfile -Command "Stop-Process -Id $p -Force -ErrorAction SilentlyContinue"; done
for f in "$O"/drive_*.log; do echo "$(basename $f): $(tail -n 1 "$f")"; done
[ -f "$O/engine.log" ] && grep "^melee: \(decider\|guest\|matched\)" "$O/engine.log" | tail -3
true
