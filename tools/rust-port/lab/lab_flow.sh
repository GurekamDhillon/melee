#!/bin/bash
# The LAB's features in real play (lab_flow.py): the retail flow's LAB match headless, the lab mod loaded, P1 a pad
# script, P2 the dummy as a CPU slot and as a human slot nobody holds; the menu's lab commands at fixed frames
# (MELEE_LAB_CMDS) and every feature checked on the fighters (MELEE_LAB_TRACE). No OS input.
#   bash tools/rust-port/lab/lab_flow.sh [cpu|human]   (both by default; EXE= / LAB_WASM= override the builds)
set -u
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"
ws="${GW_WORKSPACE:-E:/Projects/Melee Workspace}"
if [ -z "${GW_ISO_VANILLA:-}" ] && [ -f "$ws/.env" ]; then set -a; . "$ws/.env"; set +a; fi
exe="${EXE:-$repo/build/cargo/${PROFILE:-release}/melee.exe}"
wasm="${LAB_WASM:-$repo/build/mods/lab.wasm}"
out="${LAB_SCRATCH:-${TEMP:-/tmp}/labflow}"; mkdir -p "$out/mods"; rm -f "$out/mods"/*.wasm
cp "$wasm" "$out/mods/lab.wasm" || { echo "FAIL: $wasm missing (tools/rust-port/modhost/build_mods.sh)"; exit 1; }
python "$here/lab_flow.py" gen "$out"
kinds="${1:-cpu human}"
fail=0
for kind in $kinds; do
  p2=fox/cpu9; [ "$kind" = human ] && p2=fox/human
  MELEE_SCENE="mode=lab;at=match;p1=falcon/human;p2=$p2;p3=none;p4=none;stage=fd;match=time;minutes=0;items=off" \
  MELEE_LAB_TRACE=1 MELEE_LAB_CMDS="$(cat "$out/cmds.txt")" MELEE_VOLUME=3 MELEE_WINDOW_TITLE="labfix lab_flow $kind" \
  MELEE_PAD_SCRIPT="$out/pads.txt" MELEE_WASM_MODS="$out/mods" MELEE_PAD_IGNORE_ADAPTER=1 \
    "$exe" play --headless --no-audio --iso "$GW_ISO_VANILLA" --max-frames 2450 >"$out/$kind.log" 2>&1
  python "$here/lab_flow.py" check "$out/$kind.log" "$kind" || fail=1
done
exit $fail
