#!/bin/bash
# usage: run_throw_banks_cpu_synctest.sh <run-name> [frames] [bank-arenas]   (GW_MELEE / GW_BUILD_ROOT exported; mods as run_throw_banks_rewind.sh)
# Five CPUs (geno-fox, geno-falco, geno-falcon throwing retail mario, luigi), SyncTest rolling back 12 frames every frame, curated hash.
set -euo pipefail
GW_ROOT="${GW_ROOT:-E:/Projects/Melee Workspace}"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -W)"
run="$1"; frames="${2:-6000}"; banks="${3:-1}"
cfg="$GW_BUILD_ROOT/runs_cfg"; mkdir -p "$cfg"
{ printf 'CFG = {FRAMES = %s}\n' "$frames"; cat "$HERE/throw_banks_cpu_synctest.lua"; } > "$cfg/$run.lua"
export MELEE_MODS_DIR="$(cd "$GW_BUILD_ROOT/mods" && pwd -W)"
export MELEE_THROW_BANKS="$banks"
export MELEE_LOG=all,-render,-gobj,-heap,-dvd,-mex,-watchdog,-tex,-audio,-frontend
export MELEE_SYNCTEST_BENCH=1 MELEE_SYNCTEST_CURATED=1 MELEE_SYNCTEST=12
scene="mode=vs;p1=geno:geno-fox/cpu9;p2=mario/cpu9;p3=geno:geno-falco/cpu9;p4=luigi/cpu9;p5=geno:geno-falcon/cpu9;stocks=99;items=off;time=0;stage=fd"
bash "$GW_ROOT/tools/geno/moves_check/run_scene.sh" "$run" "$scene" "$cfg/$run.lua" "${TBR_TIMEOUT:-1500}" 2>&1 | tail -3
log="$GW_BUILD_ROOT/runs/$run/melee-pc.log"
echo "loads:  $(grep -c 'read into a throw-bank arena' "$log")   evictions: $(grep -c 'evicted kind' "$log")"
grep -E "TBC|curated hash" "$log" | tail -3 | sed -E 's/^\[[^]]*\] //'
python "$HERE/check_throw_banks.py" cpu "$log" --frames "$frames" --banks "$banks"
