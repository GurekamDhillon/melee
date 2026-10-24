#!/bin/bash
# Dump the C build's gameplay state at chosen match frames (MELEE_SNAP_DUMP, pc/platform/gw_snap.c)
# for the Rust golden simulation to resume from (`melee golden --from-snapshot`).
#
#   export GW_MELEE=<this worktree>  GW_BUILD_ROOT=<a private build root>   (built with tools/port/build.sh)
#   bash tools/rust-port/snap/dump.sh <scenario> <frame>[,<frame>...] [out-dir]
#
# Same scenes, pad scripts and seed as tools/rust-port/golden/record.sh, so the dumped state belongs to
# the golden run: frame N of <scenario>.csv. Output: <out-dir>/<scenario>.f<N>.snap (default
# $GW_BUILD_ROOT/snaps). The files hold disc-derived memory: NEVER commit them.
set -u
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ws="${GW_WORKSPACE:-E:/Projects/Melee Workspace}"
scenario="${1:?scenario (pad_falcon_fox_fd|pad_marth_sheik_bf|cpu_fox_falco_dl|cpu_peach_samus_ys_items)}"
frames="${2:?frame[,frame...]}"
out="${3:-$GW_BUILD_ROOT/snaps}"
mkdir -p "$out"
set -a; source "$ws/.env"; set +a
export MELEE_UNATTENDED=1 MELEE_WINDOW_X=30000 MELEE_WINDOW_Y=30000 MELEE_SCRIPTS=0 MELEE_TEST_SEED=12345
padw="$(cd "$here/../golden" && pwd -W)"
outw="$(cd "$out" && pwd -W)"
case "$scenario" in
pad_falcon_fox_fd)  pad=pad_fixed.lua; secs=240; scene='mode=vs;at=match;p1=falcon/human;p2=fox/human;p3=none;p4=none;stage=fd;match=time;minutes=0;items=off' ;;
pad_marth_sheik_bf) pad=pad_fixed.lua; secs=240; scene='mode=vs;at=match;p1=marth/human;p2=sheik/human;p3=none;p4=none;stage=bf;match=time;minutes=0;items=off' ;;
cpu_fox_falco_dl)   pad=pad_idle.lua;  secs=300; scene='mode=vs;at=match;p1=fox/cpu9;p2=falco/cpu9;p3=none;p4=none;stage=dreamland;match=time;minutes=0;items=off' ;;
cpu_peach_samus_ys_items) pad=pad_idle.lua; secs=300; scene='mode=vs;at=match;p1=peach/cpu9;p2=samus/cpu9;p3=none;p4=none;stage=ys;match=time;minutes=0;items=4' ;;
*) echo "unknown scenario $scenario"; exit 2 ;;
esac
env MELEE_TURBO=1 MELEE_PAD_SCRIPT="$padw/$pad" "MELEE_SCENE=$scene" \
    "MELEE_SNAP_DUMP=$frames:$outw/$scenario.f%d.snap" \
    MELEE_NEUTRAL_HASHLOG="$outw/$scenario.dump-run.csv" \
    bash "$ws/tools/port/run.sh" --max-seconds "$secs" "snapdump-$scenario" --iso "$GW_ISO_VANILLA" \
    > "$out/$scenario.dump.out" 2>&1
echo "run rc=$? ; files:"; ls -la "$out"/$scenario.f*.snap 2>/dev/null | awk '{print $5, $9}'
