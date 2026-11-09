#!/bin/bash
# Envoy milestone 0, probe (a) in a real netplay pair: CPU seats agreed in the handshake (the host's --cpu, carried in
# the scene's np_rules like the other host rules). Two loopback `melee play --netplay` pairs, scripted pads on ports 1-2:
#   1. two CPU9 seats (Falco, Peach), 20 ms / 2 % loss;  2. one CPU9 seat (Marth), 40 ms / 5 % loss.
# Each: 0 desyncs, every finalised frame's hash equal on both peers, the host's replay equal; the guest's log names the
# CPU seats it adopted ("netplay scene players").
#   GW_ISO_VANILLA=... bash tools/rust-port/envoy/m0_cpu_netplay.sh
set -u
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"; cd "$repo"
[ -n "${GW_ISO_VANILLA:-}" ] || { echo "m0_cpu_netplay: GW_ISO_VANILLA not set"; exit 2; }
export NP_SCRATCH="${NP_SCRATCH:-${TEMP:-/tmp}/np}" MELEE_VOLUME=3; mkdir -p "$NP_SCRATCH"
fail=0
pair() { # name lag loss cpus want-players
  local name=$1 lag=$2 loss=$3 cpus=$4 want=$5
  if bash tools/rust-port/netplay/play_pair.sh "$name" "$lag" "$loss" --headless --cpu "$cpus" >"$NP_SCRATCH/$name.out" 2>&1 &&
     grep -q "netplay scene players: $want\$" "$NP_SCRATCH/$name-guest.log"; then
    echo "ok   $name ($cpus, $lag ms, $loss %): $(grep 'frames in both' "$NP_SCRATCH/$name.out"); $(grep '^play: replay: [0-9]' "$NP_SCRATCH/$name.out" | cut -c15-)"
  else echo "FAIL $name (see $NP_SCRATCH/$name.out)"; fail=1; fi
}
pair m0_cpu2 20 2 falco/9,peach/9 "P1 ck 0 human, P2 ck 2 human, P3 ck 20 CPU 9, P4 ck 12 CPU 9"
pair m0_cpu1 40 5 marth/9 "P1 ck 0 human, P2 ck 2 human, P3 ck 9 CPU 9"
exit $fail
