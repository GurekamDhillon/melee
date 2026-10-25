#!/bin/bash
# run_c.sh LUA OUTDIR: run the C build (a copy of the baseline exe in _build/agents/plat-menus/run1)
# with the retail menus (MELEE_ATLAS=0), memory card off, window parked off screen, no OS input.
set -e
WS="/e/Projects/Melee Workspace"
set -a; . "$WS/.env"; set +a
R="$WS/_build/agents/plat-menus/run1"
cp "$1" "$R/driver.lua"
cd "$R"; rm -rf scripts-data/pad_script; mkdir -p scripts-data/pad_script
MELEE_FRONTEND=0 MELEE_UNLOCK_ALL=1 MELEE_ISO="$GW_ISO_VANILLA" MELEE_ATLAS=0 MELEE_UNATTENDED=1 MELEE_WINDOW_X=30000 MELEE_WINDOW_Y=30000 \
 MELEE_PAD_IGNORE_ADAPTER=1 MELEE_NO_ONBOARD=1 MELEE_PAD_SCRIPT="$R/driver.lua" MELEE_NATIVE_CSS=1 timeout 300 ./melee-pc.exe > out.txt 2>&1 || true
mkdir -p "$2"; cp scripts-data/pad_script/c_*.png "$2"/
grep MENUPROOF out.txt | head -20
