#!/bin/bash
# Envoy milestone 0, risk R1 (full-game plan): CPUs under TWO-PEER rollback for an Envoy-length match. Two loopback
# `melee play --netplay` peers (play_pair.sh), two CPU9 seats (ports 3-4), injected inbound delay and loss (so both peers
# roll back), the tag mod with a partner on ports 1 and 2 (scripted D-pad-down tags), and the stage_switch mod (C twin)
# shifting the stage every PERIOD logic frames. The match runs TIME seconds (default 240 = four 1-minute Envoy fights'
# worth of frames). Pass: play_pair.sh's checks (every frame both peers finalised has the same hash; the host's replay
# ends on the same hash), at least one tag and one stage switch on BOTH peers, rollbacks seen.
#   GW_ISO_VANILLA=... bash tools/rust-port/envoy/m0_np_envoy.sh [NAME] [LAG_MS] [LOSS_PCT]
#   TIME=240 PERIOD=1800 TAG_INPLACE=0|1 (the in-place tag prototype on both peers)
set -u
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"; cd "$repo"
ws="${GW_WORKSPACE:-E:/Projects/Melee Workspace}"
if [ -z "${GW_ISO_VANILLA:-}" ] && [ -f "$ws/.env" ]; then set -a; . "$ws/.env"; set +a; fi
[ -n "${GW_ISO_VANILLA:-}" ] || { echo "m0_np_envoy: GW_ISO_VANILLA not set"; exit 2; }
name="${1:-m0_np_envoy}"; lag="${2:-30}"; loss="${3:-3}"
time="${TIME:-240}"; period="${PERIOD:-1800}"
export NP_SCRATCH="${NP_SCRATCH:-${TEMP:-/tmp}/np}" MELEE_VOLUME=3; mkdir -p "$NP_SCRATCH"
mods="$NP_SCRATCH/$name-mods"; rm -rf "$mods"; mkdir -p "$mods"
cp "$repo/build/mods/tag_fighter.wasm" "$mods/" || { echo "m0_np_envoy: build the mods first (tools/rust-port/modhost/build_mods.sh)"; exit 2; }
PERIOD="$period" bash rust/mods/stage_switch/c/build.sh "$mods" >/dev/null || exit 1
envs="MELEE_WASM_MODS=$(printf %q "$mods") MELEE_TAG_PARTNER=marth,peach MELEE_TAG_INPLACE=${TAG_INPLACE:-0}"
export NP_HOST_ENV="$envs" NP_GUEST_ENV="$envs"
export NP_HOST_SCRIPT="$here/np_envoy_host.txt" NP_GUEST_SCRIPT="$here/np_envoy_guest.txt"
export NP_MATCH_HOST="falcon,32" NP_MATCH_GUEST="fox"
NP_TIME_SECS="$time" NP_MAX_FRAMES=$(( time * 60 + 3000 )) bash tools/rust-port/netplay/play_pair.sh "$name" "$lag" "$loss" --headless --cpu falco/9,samus/9 \
  >"$NP_SCRATCH/$name.out" 2>&1
rc=$?
cat "$NP_SCRATCH/$name.out"
fail=$rc
for side in host guest; do
  l="$NP_SCRATCH/$name-$side.log"
  t=$(grep -c "^tag: slot" "$l"); s=$(grep -c "^stage switch:" "$l"); rb=$(grep -o "rollbacks[ =:]*[0-9]*" "$l" | tail -1)
  echo "$side: $t tag swap executions, $s stage switch executions (resimulations included), ${rb:-no rollback count}"
  [ "$t" -ge 1 ] && [ "$s" -ge 1 ] || { echo "FAIL $side: no tag or no stage switch"; fail=1; }
done
grep -h "netplay scene players" "$NP_SCRATCH/$name-guest.log" | head -1
[ $fail -eq 0 ] && echo "R1 PROBE OK" || echo "R1 PROBE FAILED (logs: $NP_SCRATCH/$name-*)"
exit $fail
