#!/bin/bash
# SPDX-License-Identifier: MIT
# Linux port of np_dolphin_pair.sh: our engine (`melee play --slippi-mm`, the live client path) vs the local-test Slippi
# Dolphin (build_dolphin_linux.sh, run_np_dolphin.sh), paired by the loopback mock matchmaking (mock_mm.py).
#   np_dolphin_pair_linux.sh OUTDIR [--decider engine|dolphin] [--stage 0x1C] [--lag-ms N] [--loss-pct P]
#                            [--engine-out-ms N] [--dolphin-out-ms N]  (extra one-way delay on one direction only)
#                            [--mm-port P] [--jitter-ms N]  (mock port; mock relay jitter, reorders datagrams)
#                            [--engine-hitch-ms N] [--engine-hitch-at S|scene]  (SIGSTOP the engine N ms, S seconds
#                            after its first peer pad, default 1, or at once when its match scene is up, before its clock
#                            starts: the engine then starts late)
#                            [--frames N] [--pads SEED] [--css X,Y] [-- engine play args]
# --decider: the mock makes the FIRST queued ticket the host (isHost, the Slippi decider, port 1), so the decider
# joins first: `dolphin` (the old harness order: Dolphin TSTA#001 joins, then the engine TSTB#001) or `engine` (the
# engine joins and waits in the queue; Dolphin is driven through its menus and joins second as the guest, port 2).
# Isolation: the whole run is re-executed in a private user+network namespace (unshare -rn) whose only interface is
# loopback, so neither Dolphin nor the engine can reach any Slippi service or player, whatever their config.
# Output: OUTDIR/{mm.log,dolphin stdout/stderr,drive.log,engine.log,engine.csv,checksums_P3A.txt,cap.txt} and
# OUTDIR/result.txt (np_compare.py: Dolphin's TX checksums vs the engine's finalized ones, plus the engine's live line).
# Env: MELEE (the engine exe), DOLPHIN_BIN, GW_ISO_VANILLA (else the "*Smash*.iso" in ~/melee_linux_test, never
# printed), MAXS (engine max seconds). Only processes started here are stopped.
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
if [ -z "$NP_IN_NETNS" ]; then
  NP_IN_NETNS=1 exec unshare -rn bash -c 'ip link set lo up && exec "$@"' _ bash "$0" "$@"
fi
O=$1; shift; mkdir -p "$O"; O="$(cd "$O" && pwd)"
dn=P3A; mmp=43131; lag=0; loss=0; frames=3600; pads=1; css="-31,9"; stage=0x20; decider=dolphin; eout=0; dout=0; hitch=0; hitch_at=1; jit=0; eng=()
while [ $# -gt 0 ]; do
  case "$1" in
    --decider) decider=$2; shift ;; --stage) stage=$2; shift ;;
    --lag-ms) lag=$2; shift ;; --loss-pct) loss=$2; shift ;; --engine-out-ms) eout=$2; shift ;; --dolphin-out-ms) dout=$2; shift ;;
    --engine-hitch-ms) hitch=$2; shift ;; --mm-port) mmp=$2; shift ;; --jitter-ms) jit=$2; shift ;; --engine-hitch-at) hitch_at=$2; shift ;;
    --frames) frames=$2; shift ;; --pads) pads=$2; shift ;; --css) css=$2; shift ;;
    --) shift; eng=("$@"); break ;;
    *) echo "np_dolphin_pair_linux: unknown option $1" >&2; exit 2 ;;
  esac; shift
done
if [ -z "$GW_ISO_VANILLA" ]; then
  for f in "$HOME"/melee_linux_test/*Smash*.iso; do GW_ISO_VANILLA=$f; break; done
fi
export GW_ISO_VANILLA
MELEE=${MELEE:-$HOME/melee_rust/build/cargo-deciderharness/release/melee}
pids=()
cleanup() { for p in "${pids[@]}"; do kill -9 "$p" 2>/dev/null; done; }
trap cleanup EXIT; trap "exit 143" TERM INT
# asymmetric routes: the mock's A is the first queued player (the decider)
if [ "$decider" = engine ]; then ab=$eout; ba=$dout; else ab=$dout; ba=$eout; fi
python3 -I "$here/mock_mm.py" --port $mmp --stages "$stage" --lag-ms $lag --loss-pct $loss --extra-ab-ms $ab --extra-ba-ms $ba --jitter-ms $jit \
  --capture "$O/cap.txt" > "$O/mm.log" 2>&1 &
pids+=($!)
sleep 1
engine() {
  MELEE_INPUT=n MELEE_PAD_IGNORE_ADAPTER=1 MELEE_WINDOW_HIDDEN=1 MELEE_WINDOW_UNFOCUSED=1 MELEE_KEYBOARD=0 MELEE_VOLUME=3 \
  MELEE_VSYNC=off MELEE_CARD=0 MELEE_WINDOW_TITLE="deciderharness:engine-$decider-decides" \
  exec "$MELEE" play --iso "$GW_ISO_VANILLA" --skip-intro --slippi-mm "ws://127.0.0.1:$mmp/v1/queue" --slippi-mm-mode direct \
    --slippi-port $((mmp + 100)) --slippi-stage $((stage)) --slippi-code TSTB#001 --slippi-search TSTA#001 --slippi-frames $frames \
    --slippi-max-seconds ${MAXS:-400} --slippi-out "$O/engine.csv" --slp "$O/engine.slp" "${eng[@]}" > "$O/engine.log" 2>&1
}
if [ "$decider" = engine ]; then
  engine & epid=$!; pids+=($epid)
  for i in $(seq 240); do grep -q "join from TSTB" "$O/mm.log" && break; sleep 0.5; done
  grep -q "join from TSTB" "$O/mm.log" || { echo "np_dolphin_pair_linux: the engine never joined"; exit 3; }
  # NP_QUEUE_WAIT=S: leave the engine alone in the queue S seconds first (a slow Dolphin boot, as on Windows)
  sleep ${NP_QUEUE_WAIT:-0}
fi
mkdir -p "$O/dslp"
dpid=$(NP_REPLAY_DIR="$O/dslp" bash "$here/run_np_dolphin.sh" $dn "$O" $mmp TSTA#001 TSTB#001)
pids+=($dpid)
python3 -I "$here/dolphin_drive.py" --name $dn --pipe "$O/user_$dn/Pipes/slippibot1" --state "$O/state_$dn.txt" \
  --play "$css" --pads $pads --frames $((frames + 300)) --timeout 300 > "$O/drive.log" 2>&1 &
drv=$!; pids+=($drv)
if [ "$decider" != engine ]; then
  # the engine joins once Dolphin is searching (second in the queue: the guest, port 2)
  for i in $(seq 600); do grep -q "join from TSTA" "$O/mm.log" && break; kill -0 $drv 2>/dev/null || break; sleep 0.5; done
  grep -q "join from TSTA" "$O/mm.log" || { echo "np_dolphin_pair_linux: Dolphin never joined"; tail -1 "$O/drive.log"; exit 3; }
  engine & epid=$!; pids+=($epid)
fi
if [ "$hitch" -gt 0 ]; then
  if [ "$hitch_at" = scene ]; then pat="match scene, waiting for the peer"; hs=0; else pat="Slippi match started"; hs=$hitch_at; fi
  ( for i in $(seq 12000); do grep -q "$pat" "$O/engine.log" 2>/dev/null && break; sleep 0.01; done
    sleep $hs; kill -STOP $epid; sleep $(awk "BEGIN{print $hitch/1000}"); kill -CONT $epid
    echo "hitch: engine stopped $hitch ms" >> "$O/hitch.log" ) &
fi
wait $epid; erc=$?
wait $drv
kill -9 $dpid 2>/dev/null
{
  echo "decider=$decider stage=$stage lag=$lag loss=$loss engine_out=+$eout dolphin_out=+$dout jitter=$jit mm_port=$mmp hitch=$hitch@$hitch_at NEW=${MELEE_TIMESYNC_NEW:-0} engine_rc=$erc"
  grep -h "join from\|pairing" "$O/mm.log"
  grep -h "^melee: \(decider\|guest\)" "$O/engine.log" | tail -2
  tail -2 "$O/drive.log"
  python3 -I "$here/np_compare.py" "$O" $dn > "$O/cmp.txt" 2>&1; head -8 "$O/cmp.txt"
  # post hoc, from both replays: the inputs and seed each game ran with (A = Dolphin, B = the engine)
  ds=$(ls "$O"/dslp/*.slp 2>/dev/null | head -1)
  if [ -n "$ds" ] && [ -f "$O/engine.slp" ]; then python3 -I "$here/slp_inputs_diff.py" "$ds" "$O/engine.slp" > "$O/slpdiff.txt" 2>&1; head -4 "$O/slpdiff.txt"; fi
} > "$O/result.txt"
cat "$O/result.txt"
