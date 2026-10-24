#!/bin/bash
# mix.sh <rust-role: host|guest> <port> [rust args...]   Rust peer vs C peer
ROLE=$1; P=$2; shift 2
export GW_ROOT="E:/Projects/Melee Workspace" GW_MELEE="E:/Projects/Melee Workspace/worktrees/plat-net" GW_BUILD_ROOT="E:/Projects/Melee Workspace/_build/agents/plat-net"
set -a; . "$GW_ROOT/.env"; set +a
S="${NP_SCRATCH:-$TEMP/np}"
B="$GW_MELEE/build/cargo/release/melee.exe"; O="${NP_SCRATCH:-$TEMP/np}"
EXE=$(grep -a -o "exe=[0-9a-f]*" "$GW_BUILD_ROOT/runs/cnet-c-host/melee-pc.log" | head -1 | cut -d= -f2)
MODS=$(grep -a -o "mods=[0-9a-f]*" "$GW_BUILD_ROOT/runs/cnet-c-host/melee-pc.log" | head -1 | cut -d= -f2)
export MELEE_NP_GLOBAL_HASH=$MODS
export MELEE_NP_IDS="${MELEE_NP_IDS:-}"
rm -f "$S/m_c.csv"
cenv() { export MELEE_UNATTENDED=1 MELEE_WATCHDOG_SECS=120 MELEE_WINDOW_X=30000 MELEE_WINDOW_Y=30000 MELEE_SCRIPTS=0 MELEE_NETPLAY_BIND=127.0.0.1 MELEE_NETPLAY_STAGE=32 MELEE_NETPLAY_STOCKS=3 MELEE_NETPLAY_MINUTES=0 MELEE_NETPLAY_SEED=12345 MELEE_NETPLAY_PORT=$P MELEE_RB_NEUTRAL_HASH=1 MELEE_NEUTRAL_HASHLOG="$S/m_c.csv"; }
if [ "$ROLE" = host ]; then
  "$B" netplay --host --port $P --exe-hash $EXE --live-rng --seed 12345 --ck 0 --out "$O/h.csv" --raw-out "$O/raw.csv" "$@" > "$O/mh.log" 2>&1 &
  sleep 2
  ( cenv; export MELEE_NETPLAY=join:127.0.0.1:$P MELEE_NETPLAY_CHAR=2 MELEE_PAD_SCRIPT="$S/pad_np_guest.lua"; bash "$GW_ROOT/tools/port/run.sh" --max-seconds 100 "mix-$ROLE" --iso "$GW_ISO_VANILLA" > "$S/m_c.out" 2>&1 )
  wait
else
  ( cenv; export MELEE_NETPLAY=host:$P MELEE_NETPLAY_CHAR=0 MELEE_PAD_SCRIPT="$S/pad_np_host.lua"; bash "$GW_ROOT/tools/port/run.sh" --max-seconds 100 "mix-$ROLE" --iso "$GW_ISO_VANILLA" > "$S/m_c.out" 2>&1 ) &
  sleep 12
  "$B" netplay --join 127.0.0.1:$P --exe-hash $EXE --live-rng --ck 2 --out "$O/g.csv" --raw-out "$O/raw.csv" "$@" > "$O/mh.log" 2>&1
  wait
fi
tail -n 4 "$O/mh.log" | cut -c1-300
grep -a "netplay:\|DESYNC\|rb:" "$GW_BUILD_ROOT/runs/mix-$ROLE/melee-pc.log" | tail -n 12 | cut -c1-220
