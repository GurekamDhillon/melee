#!/bin/bash
# usage: run_throw_banks_rewind.sh <run-name>   (needs GW_MELEE / GW_BUILD_ROOT exported, mods in $GW_BUILD_ROOT/mods:
# geno-lab, geno-fox, geno-falco, geno-falcon, enabled.txt listing them). Runs pc/tests/throw_banks_rewind.lua in a LAB scene with
# one retail victim and three Geno throwers of different thrown_as families, one throw-bank arena (MELEE_THROW_BANKS=1).
set -euo pipefail
GW_ROOT="${GW_ROOT:-E:/Projects/Melee Workspace}"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -W)"
run="$1"
cfg="$GW_BUILD_ROOT/runs_cfg"; mkdir -p "$cfg"
{
  printf 'CFG = {}\n'
  cat "$HERE/throw_banks_rewind.lua"
} > "$cfg/$run.lua"
export MELEE_MODS_DIR="$(cd "$GW_BUILD_ROOT/mods" && pwd -W)"
export MELEE_THROW_BANKS="${MELEE_THROW_BANKS:-1}"
export MELEE_LOG=all,-render,-gobj,-heap,-dvd,-mex,-watchdog,-tex,-audio,-frontend
scene="mode=lab;p1=mario/hu;p2=geno:geno-fox/hu;p3=geno:geno-falco/hu;p4=geno:geno-falcon/hu;stage=fd;items=off;time=0"
bash "$GW_ROOT/tools/geno/moves_check/run_scene.sh" "$run" "$scene" "$cfg/$run.lua" "${TBR_TIMEOUT:-900}" 2>&1 | tail -5
log="$GW_BUILD_ROOT/runs/$run/melee-pc.log"
grep -E "TBR |throw: stand-in|define budgets: throw banks|snap: .*(MISMATCH|mismatch)|SYNCTEST|FATAL|crash" "$log" | sed -E 's/^\[frame=[0-9]+ ms=[0-9]+\] (script \[pad_script\] )?//' | sed -n "1,${TBR_LINES:-60}p"
python "$HERE/check_throw_banks.py" rewind "$log"
