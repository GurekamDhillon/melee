#!/bin/bash
# LAB recorder acceptance (R3): builds a mods dir (goku-xv2 combo package + this lab + the test script)
# and runs the game in turbo against the labrec build root. Needs GW_MELEE and GW_BUILD_ROOT exported.
# Usage: run.sh <run-name> [console-port]
set -e
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ws="${GW_WORKSPACE:-E:/Projects/Melee Workspace}"
name="${1:-labrec-acc}"
pkg="$GW_BUILD_ROOT/rectest/mods"
rm -rf "$GW_BUILD_ROOT/rectest"; mkdir -p "$pkg"
cp -r "$ws/_build/goku-xv2/combo-package/mods/goku-xv2" "$pkg/goku-xv2"
cp -r "$GW_MELEE/pc/geno/mods/geno-lab" "$pkg/geno-lab"
cp -r "$here/lab-rec-tests" "$pkg/lab-rec-tests"
printf 'geno-lab\ngoku-xv2\nlab-rec-tests\n' > "$pkg/enabled.txt"
source "$ws/.env"
export MELEE_MODS_DIR="$(cd "$pkg" && pwd -W)"
export MELEE_SCENE='mode=lab;p1=geno:goku-xv2/hu;p2=mario/hu;stage=fd;items=off;time=0'
export MELEE_UNATTENDED=1 MELEE_MAX_SECONDS=420 MELEE_LAB_UI=0 MELEE_TURBO=1
export MELEE_WINDOW_X=30000 MELEE_WINDOW_Y=30000 MELEE_NETPLAY_BIND=127.0.0.1
export MELEE_PAD_SCRIPT="$(cd "$ws/ports/goku-xv2/combo/tests" && pwd -W)/turbo_pad.lua"
[ -n "${2:-}" ] && export MELEE_CONSOLE_PORT="$2"
bash "$ws/tools/port/run.sh" --max-seconds 420 "$name" --iso "$GW_ISO_VANILLA"
grep -a "LABREC" "$GW_BUILD_ROOT/runs/$name/melee-pc.log" || true
