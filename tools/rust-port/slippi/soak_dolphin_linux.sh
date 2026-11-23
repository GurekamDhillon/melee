#!/bin/bash
# SPDX-License-Identifier: MIT
# Long soak: ONE engine process (`play --slippi-menus on`, the in-process online loop) plays Slippi Unranked against
# the local-test Slippi Dolphin back to back, through the loopback mock (np_dolphin_pair_linux.sh's setup: unshare -rn,
# throwaway profiles, the disc from GW_ISO_VANILLA, never printed). After every match both go back to the online CSS
# and search again (engine: MELEE_SLIPPI_BACK_PAD_SCRIPT; Dolphin: dolphin_drive.py --mode unranked --games N).
#   soak_dolphin_linux.sh OUTDIR [--minutes M] [--lag-ms N] [--jitter-ms N] [--stage S] [--mm-port P]
# The engine logs its `mem after` line (arena, system heap, caches) every MELEE_MEM_EVERY ticks (default 3600).
# OUTDIR/matches.tsv: per match (engine log order) the engine's RSS / PSS / private dirty right after the match, its
# Slippi line (compared, soft, HARD, ticks over 16 ms) and rollbacks; OUTDIR/rss.tsv: a sample every 30 s.
# The engine's window title starts with "soak:" and the mock runs with MELEE_SOAK=1 (the lane watchdog's 3 h limit).
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"
if [ -z "$NP_IN_NETNS" ]; then
  NP_IN_NETNS=1 exec unshare -rn bash -c 'ip link set lo up && exec "$@"' _ bash "$0" "$@"
fi
O=$1; shift; mkdir -p "$O"; O="$(cd "$O" && pwd)"
mins=120; lag=30; jit=5; stage=0x1C; mmp=43181
while [ $# -gt 0 ]; do
  case "$1" in
    --minutes) mins=$2; shift ;; --lag-ms) lag=$2; shift ;; --jitter-ms) jit=$2; shift ;;
    --stage) stage=$2; shift ;; --mm-port) mmp=$2; shift ;;
    *) echo "soak_dolphin_linux: unknown option $1" >&2; exit 2 ;;
  esac; shift
done
if [ -z "$GW_ISO_VANILLA" ]; then
  for f in "$HOME"/melee_linux_test/*Smash*.iso; do GW_ISO_VANILLA=$f; break; done
fi
export GW_ISO_VANILLA
MELEE=${MELEE:-$HOME/melee_rust/build/cargo/release/melee}
M=$repo/tools/rust-port/menus
# the online menus' Slippi files (MELEE_SLIPPI_SYS_DIR): this namespace has no network, so a cache filled before is
# needed (the .diff files come from Slippi's public Ishiiruka repo, sysfiles.rs)
export MELEE_SLIPPI_SYS_DIR=${MELEE_SLIPPI_SYS_DIR:-$HOME/melee_out/catchup/SlippiSys}
pids=()
cleanup() { for p in "${pids[@]}"; do kill -9 "$p" 2>/dev/null; done; }
trap cleanup EXIT; trap "exit 143" TERM INT
MELEE_SOAK=1 python3 -I "$here/mock_mm.py" --port $mmp --stages "$stage" --lag-ms $lag --jitter-ms $jit > "$O/mm.log" 2>&1 &
pids+=($!)
sleep 1
# Slippi Unranked keeps the opponent after a game (the next game's select, the same link): the engine's per-game
# select pad scripts (MELEE_SLIPPI_SET_PAD_SCRIPT_G<n>): pick a fighter, Start
for g in $(seq 2 400); do export MELEE_SLIPPI_SET_PAD_SCRIPT_G$g="$M/slippi_back_a.txt"; done
( cd "$repo/rust" && MELEE_INPUT=n MELEE_PAD_IGNORE_ADAPTER=1 MELEE_WINDOW_HIDDEN=1 MELEE_WINDOW_UNFOCUSED=1 MELEE_KEYBOARD=0 \
  MELEE_VOLUME=3 MELEE_VSYNC=off MELEE_CARD=0 MELEE_RETAIL_FIRST=1 MELEE_WINDOW_TITLE="soak:engine-vs-dolphin" \
  MELEE_PAD_SCRIPT="$M/slippi_online_unranked_css_1start.txt" MELEE_SLIPPI_BACK_PAD_SCRIPT="$M/slippi_back_a.txt" \
  MELEE_SLIPPI_MM_URL="ws://127.0.0.1:$mmp/v1/queue" MELEE_SLIPPI_NP_PORT=$((mmp + 100)) MELEE_SLIPPI_TEST_CODE="TSTB#001" \
  MELEE_SLIPPI_NP_OUT="$O/engine.csv" MELEE_SLIPPI_NP_MAX_SECONDS=900 MELEE_SLIPPI_NP_PADGEN=7 \
  MELEE_MEM_EVERY=${MELEE_MEM_EVERY:-3600} MELEE_ARENA_STATS=1 \
  exec "$MELEE" play --iso "$GW_ISO_VANILLA" --skip-intro --slippi-menus on > "$O/engine.log" 2>&1 < /dev/null ) &
epid=$!; pids+=($epid)
dpid=$(bash "$here/run_np_dolphin.sh" P3A "$O" $mmp TSTA#001 TSTB#001)
pids+=($dpid)
python3 -I "$here/dolphin_drive.py" --name P3A --pipe "$O/user_P3A/Pipes/slippibot1" --state "$O/state_P3A.txt" \
  --play "-31,9" --mode unranked --games 1000 --frames 30000 --pads 11 --timeout 600 > "$O/drive.log" 2>&1 &
drv=$!; pids+=($drv)
mem() { # rss pss private_dirty (kB) of the engine
  local r p d
  r=$(awk '/^VmRSS:/{print $2}' /proc/$epid/status 2>/dev/null)
  read -r p d < <(awk '/^Pss:/{p=$2} /^Private_Dirty:/{d=$2} END{print p, d}' /proc/$epid/smaps_rollup 2>/dev/null)
  echo "${r:-0} ${p:-0} ${d:-0}"
}
printf 'match\tt_s\trss_kB\tpss_kB\tprivate_dirty_kB\tload1\tslippi_line\trollbacks_line\n' > "$O/matches.tsv"
printf 't_s\trss_kB\tpss_kB\tprivate_dirty_kB\n' > "$O/rss.tsv"
t0=$(date +%s); seen=0; last=0
while kill -0 $epid 2>/dev/null && [ $(( $(date +%s) - t0 )) -lt $((mins * 60)) ]; do
  now=$(( $(date +%s) - t0 ))
  if [ $((now - last)) -ge 30 ]; then last=$now; echo -e "$now\t$(mem | tr ' ' '\t')" >> "$O/rss.tsv"; fi
  n=$(grep -c "final hashes (through" "$O/engine.log" 2>/dev/null)
  if [ "${n:-0}" -gt "$seen" ]; then
    sleep 3  # the match's teardown
    seen=$n
    sl=$(grep "Slippi: [0-9]* final checksums" "$O/engine.log" | sed -n "${n}p" | sed 's/.*Slippi: //' | cut -c1-200)
    rl=$(grep "final hashes (through" "$O/engine.log" | sed -n "${n}p" | grep -o "rollbacks [0-9]* (max depth [0-9]*" )
    echo -e "$n\t$(( $(date +%s) - t0 ))\t$(mem | tr ' ' '\t')\t$(cut -d' ' -f1 /proc/loadavg)\t$sl\t$rl" >> "$O/matches.tsv"
  fi
  sleep 2
done
echo "soak: $seen matches in $(( $(date +%s) - t0 )) s; engine alive: $(kill -0 $epid 2>/dev/null && echo yes || echo no)" | tee "$O/result.txt"
tail -2 "$O/drive.log" >> "$O/result.txt"
