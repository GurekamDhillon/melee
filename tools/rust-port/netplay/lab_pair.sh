#!/bin/bash
export MELEE_WINDOW_HIDDEN=${MELEE_WINDOW_HIDDEN:-1}  # agents' game windows are invisible (MELEE_WINDOW_HIDDEN=0 shows them)
# The online Lab (LabNetplay, gd-lab-v1 GAP-13): two headless `melee play --netplay ... --np-lab` peers on loopback with
# the lab_demo mod. The players pick on the retail character select (lockstep), the match runs in lockstep with the
# Lab: the host's lab_demo saves, records, rewinds, loads and plays back, the guest's saves and loads slot 2, and every
# command crosses the input stream and applies on the same frame on both peers. Headless (windowed pairs reset the GPU
# on the dev box); scripted pads, no OS input.
#   bash tools/rust-port/netplay/lab_pair.sh NAME LAG_MS LOSS_PCT [extra play args]
# Needs build/mods/lab_demo.wasm (tools/rust-port/modhost/build_mods.sh). Exit 0 = both peers finished the match with
# 0 desyncs over > 1000 hash checks, and both applied the same lab actions on the same frames.
set -u
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"
ws="${GW_WORKSPACE:-E:/Projects/Melee Workspace}"
if [ -z "${GW_ISO_VANILLA:-}" ] && [ -f "$ws/.env" ]; then set -a; . "$ws/.env"; set +a; fi
name=$1; lag=$2; loss=$3; shift 3
exe="$repo/build/cargo/${PROFILE:-release}/melee.exe"
out="${NP_SCRATCH:-${TEMP:-/tmp}/np}"; mkdir -p "$out"
mods="$out/$name-mods"; mkdir -p "$mods"; rm -f "$mods"/*.wasm
mod="${LAB_MOD:-lab_demo}"  # LAB_MOD=lab: the LAB mod, driven by both peers' pads (lab_scripts.py)
cp "$repo/build/mods/$mod.wasm" "$mods/" || { echo "FAIL $name: build/mods/$mod.wasm missing (build_mods.sh)"; exit 1; }
python "$here/gen_css_scripts.py" "$out" >/dev/null
hs=np_css_host.txt; gs=np_css_guest.txt
if [ "$mod" = lab ]; then python "$here/lab_scripts.py" "$out" >/dev/null || { echo "FAIL $name: lab_scripts.py"; exit 1; }; hs=np_lab_host.txt; gs=np_lab_guest.txt; fi
port=$((51900 + RANDOM % 300))
export MELEE_UI_LOG=1 MELEE_VOLUME=3 MELEE_PAD_IGNORE_ADAPTER=1 MELEE_WINDOW_UNFOCUSED=1 MELEE_WASM_MODS="$mods"
common=(play --headless --no-audio --iso "$GW_ISO_VANILLA" --np-lab --time-secs 60 --quit-after-matches 1 --np-lag-ms "$lag" --np-loss-pct "$loss" --max-frames 30000 "$@")
MELEE_WINDOW_TITLE="wasmlab: lab pair $name host" MELEE_PAD_SCRIPT="$out/$hs" bash "$here/idle.sh" "$exe" "${common[@]}" \
  --netplay host --np-bind 127.0.0.1 --np-port $port --np-seed 4242 >"$out/$name-host.log" 2>&1 &
h=$!
sleep 1
MELEE_WINDOW_TITLE="wasmlab: lab pair $name guest" MELEE_PAD_SCRIPT="$out/$gs" bash "$here/idle.sh" "$exe" "${common[@]}" \
  --netplay join "127.0.0.1:$port" --np-bind 127.0.0.1 >"$out/$name-guest.log" 2>&1
rg=$?
wait $h; rh=$?
fail=0
[ $rh -eq 0 ] && [ $rg -eq 0 ] || { echo "FAIL $name: exit host $rh guest $rg"; fail=1; }
for side in host guest; do
  l=$(grep -h "lab netplay: match over" "$out/$name-$side.log" | tail -1)
  echo "$name $side: ${l:-no match-over line}"
  c=$(echo "$l" | sed -n 's/.*over: \([0-9]*\) hash checks, \([0-9]*\) desyncs.*/\1 \2/p')
  set -- $c
  [ "${1:-0}" -gt 1000 ] && [ "${2:-1}" -eq 0 ] || { echo "FAIL $name: $side hash checks / desyncs"; fail=1; }
  grep -h "^lab: frame\|^lab: loaded" "$out/$name-$side.log" | sed 's/ (forks.*//' >"$out/$name-$side.actions"
  grep -h "lab perf" "$out/$name-$side.log" | tail -1
done
na=$(wc -l <"$out/$name-host.actions")
if [ "$na" -lt 8 ] || ! cmp -s "$out/$name-host.actions" "$out/$name-guest.actions"; then
  echo "FAIL $name: lab actions differ or too few ($na)"; diff "$out/$name-host.actions" "$out/$name-guest.actions" | head; fail=1
else
  echo "$name: both peers applied the same $na lab actions on the same frames:"; sed 's/^/  /' "$out/$name-host.actions"
fi
if [ "$mod" = lab ]; then
  # the menus' dummy settings (host: P2 block + tech; guest: P1 DI) are lab commands both peers applied
  nd=$(grep -c "reaction\|control" "$out/$name-host.actions")
  [ "$nd" -ge 3 ] && echo "$name: $nd dummy setting commands on both peers" || { echo "FAIL $name: dummy settings from the menus missing ($nd)"; fail=1; }
fi
[ $fail -eq 0 ] && echo "OK $name" || echo "FAILED $name"
exit $fail
