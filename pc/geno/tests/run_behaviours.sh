#!/bin/bash
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 GurekamDhillon
#
# Live acceptance run of the Geno v5.1 behaviours: a Kirby with the behaviour-test mod, driven by
# pad_behaviours.lua (MELEE_PAD_SCRIPT), headless and unfocused, no OS input. Then check_behaviours.py
# reads the log.
#   export GW_MELEE=<this worktree> GW_BUILD_ROOT=<a private build root>
#   bash pc/geno/tests/run_behaviours.sh
set -u
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ws="${GW_WORKSPACE:-E:/Projects/Melee Workspace}"
set -a; source "$ws/.env"; set +a
export MELEE_UNATTENDED=1 MELEE_WINDOW_UNFOCUSED=1 MELEE_SCRIPTS=0 MELEE_TEST_SEED=12345 MELEE_TURBO=1
export MELEE_MODS_DIR="$(cd "$here/mods" && pwd -W)"
export MELEE_PAD_SCRIPT="$(cd "$here" && pwd -W)/pad_behaviours.lua"
export MELEE_SCENE="mode=vs;at=match;p1=kirby/human;p2=fox/human;p3=none;p4=none;stage=fd;match=time;minutes=0;items=off"
name="${1:-geno-behaviours}"
bash "$ws/tools/port/run.sh" --max-seconds 300 "$name" --iso "$GW_ISO_VANILLA" > "$GW_BUILD_ROOT/$name.out" 2>&1
echo "run rc=$?"
python "$here/check_behaviours.py" "$GW_BUILD_ROOT/runs/$name/melee-pc.log"
