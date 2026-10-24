#!/bin/bash
# C-vs-C scripted netplay: cnet.sh <port> <tag> [extra env assignments as VAR=val ...]
export GW_ROOT="E:/Projects/Melee Workspace" GW_MELEE="E:/Projects/Melee Workspace/worktrees/plat-net" GW_BUILD_ROOT="${CREF_ROOT:-E:/Projects/Melee Workspace/_build/agents/plat-net-ref}"
set -a; . "$GW_ROOT/.env"; set +a
S="${NP_SCRATCH:-$TEMP/np}"
P=$1; T=$2; shift 2
for v in "$@"; do export "$v"; done
common() { export MELEE_UNATTENDED=1 MELEE_WATCHDOG_SECS=120 MELEE_WINDOW_X=30000 MELEE_WINDOW_Y=30000 MELEE_SCRIPTS=0 MELEE_NETPLAY_BIND=127.0.0.1 MELEE_NETPLAY_STAGE=32 MELEE_NETPLAY_STOCKS=3 MELEE_NETPLAY_MINUTES=0 MELEE_NETPLAY_SEED=12345 MELEE_NETPLAY_PORT=$P; }
( common; export MELEE_NETPLAY=host:$P MELEE_NETPLAY_CHAR=0 MELEE_PAD_SCRIPT="$S/pad_np_host.lua" MELEE_NEUTRAL_HASHLOG="$S/c_${T}_host.csv"; rm -f "$MELEE_NEUTRAL_HASHLOG"; bash "$GW_ROOT/tools/port/run.sh" --max-seconds 150 "cnet-$T-host" --iso "$GW_ISO_VANILLA" > "$S/c_${T}_host.out" 2>&1 ) &
sleep 6
( common; export MELEE_NETPLAY=join:127.0.0.1:$P MELEE_NETPLAY_CHAR=2 MELEE_PAD_SCRIPT="$S/pad_np_guest.lua" MELEE_NEUTRAL_HASHLOG="$S/c_${T}_guest.csv"; rm -f "$MELEE_NEUTRAL_HASHLOG"; bash "$GW_ROOT/tools/port/run.sh" --max-seconds 150 "cnet-$T-guest" --iso "$GW_ISO_VANILLA" > "$S/c_${T}_guest.out" 2>&1 ) &
wait
tail -n 4 "$S/c_${T}_host.out" "$S/c_${T}_guest.out"
