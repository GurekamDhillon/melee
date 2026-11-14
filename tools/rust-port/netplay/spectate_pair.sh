#!/bin/bash
export MELEE_WINDOW_HIDDEN=${MELEE_WINDOW_HIDDEN:-1}  # agents' game windows are invisible (MELEE_WINDOW_HIDDEN=0 shows them)
# A headless netplay pair on loopback with one read-only spectator (notes/spectate-replay.md). No OS input, no window.
#   bash tools/rust-port/netplay/spectate_pair.sh NAME direct|relay JOIN_AFTER_S [LAG_MS LOSS_PCT]
# direct: the spectator dials the host's spectate port; relay: both go through a local melee_relay (spectate code).
# JOIN_AFTER_S: seconds after the host starts before the spectator connects (0 = from the start; > 15 = mid-match).
# Checks: host and guest per-frame hashes equal; the spectator's playback compares every match frame with the host's
# hash (0 mismatches) and ends on the host's final hash. Three game processes run (host, guest, spectator), headless.
set -u
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"
ws="${GW_WORKSPACE:-E:/Projects/Melee Workspace}"
if [ -z "${GW_ISO_VANILLA:-}" ] && [ -f "$ws/.env" ]; then set -a; . "$ws/.env"; set +a; fi
name=$1; via=$2; after=$3; lag=${4:-0}; loss=${5:-0}
exe="$repo/build/cargo/${PROFILE:-release}/melee.exe"
relay_exe="$repo/build/cargo/${PROFILE:-release}/melee_relay.exe"
out="${NP_SCRATCH:-${TEMP:-/tmp}/np}"; mkdir -p "$out"
port=$((51600 + RANDOM % 300)); sport=$((port + 400)); rport=$((port + 800))
export MELEE_VOLUME=3 MELEE_PAD_IGNORE_ADAPTER=1 MELEE_WINDOW_UNFOCUSED=1
common=(play --headless --no-audio --iso "$GW_ISO_VANILLA" --time-secs 60 --np-lag-ms "$lag" --np-loss-pct "$loss" --max-frames 9000)
spec_host=(--np-spectate $sport)
target="127.0.0.1:$sport"
rpid=""
if [ "$via" = relay ]; then
  "$relay_exe" --port $rport --bind 127.0.0.1 --bind6 off >"$out/$name-relay.log" 2>&1 &
  rpid=$!
  spec_host+=(--np-spectate-relay 127.0.0.1:$rport --np-spectate-code T$port)
  target="relay:127.0.0.1:$rport/T$port"
fi
MELEE_WINDOW_TITLE="spectate: $name host" MELEE_PAD_SCRIPT="$here/np_fight.txt" bash "$here/idle.sh" "$exe" "${common[@]}" \
  --netplay host --np-port $port --match falcon,32 --np-seed 4242 "${spec_host[@]}" --np-hash-out "$out/$name-host.csv" --record "$out/$name-host.mrpl" >"$out/$name-host.log" 2>&1 &
h=$!
sleep 1
MELEE_WINDOW_TITLE="spectate: $name guest" MELEE_PAD_SCRIPT="$here/np_fight2.txt" bash "$here/idle.sh" "$exe" "${common[@]}" \
  --netplay join "127.0.0.1:$port" --match fox --np-hash-out "$out/$name-guest.csv" >"$out/$name-guest.log" 2>&1 &
g=$!
sleep "$after"
# SPEC_REFUSE=1: the spectator pretends to be another build (MELEE_NP_BUILD_HASH) and must be refused in the handshake
senv=(); [ -n "${SPEC_REFUSE:-}" ] && senv=(env MELEE_NP_BUILD_HASH=123456789abcdef)
MELEE_WINDOW_TITLE="spectate: $name spectator" "${senv[@]}" bash "$here/idle.sh" "$exe" play --headless --no-audio --iso "$GW_ISO_VANILLA" --spectate "$target" --spec-delay 6 \
  >"$out/$name-spec.log" 2>&1
rs=$?
wait $g; rg=$?
wait $h; rh=$?
[ -n "$rpid" ] && kill $rpid 2>/dev/null
fail=0
[ $rh -eq 0 ] && [ $rg -eq 0 ] || { echo "FAIL $name: exit host $rh guest $rg"; fail=1; }
diffs=$(join -t, <(tail -n +2 "$out/$name-host.csv" | sort -t, -k1,1) <(tail -n +2 "$out/$name-guest.csv" | sort -t, -k1,1) | awk -F, '$2!=$3' | wc -l)
nh=$(($(wc -l <"$out/$name-host.csv") - 1))
echo "$name: host $nh frames, $diffs host/guest differences"
[ "$diffs" -eq 0 ] && [ "$nh" -gt 1000 ] || { echo "FAIL $name: player hashes"; fail=1; }
grep -h "spectate: \(host: \(spectator\|SPECTATE\|stream\|[0-9]* spectators\)\|joined\)" "$out/$name-host.log" "$out/$name-spec.log"
sum=$(grep -h "^play: replay: " "$out/$name-spec.log" | tail -1); echo "$name spectator: $sum"
hfin=$(grep -o "netplay host: final hash [0-9A-F]*" "$out/$name-host.log" | awk '{print $NF}')
echo "$name: host final $hfin"
if [ -n "${SPEC_REFUSE:-}" ]; then
  grep -h "refused" "$out/$name-spec.log" | head -2
  { [ $rs -ne 0 ] && grep -q "the host refused: different build" "$out/$name-spec.log"; } || { echo "FAIL $name: not refused"; fail=1; }
  [ $fail -eq 0 ] && echo "OK $name (refused)" || echo "FAILED $name"; exit $fail
fi
[ $rs -eq 0 ] || { echo "FAIL $name: spectator exit $rs"; fail=1; }
echo "$sum" | grep -q " 0 mismatches" || { echo "FAIL $name: spectator mismatches"; fail=1; }
echo "$sum" | grep -q "final hash $hfin" || { echo "FAIL $name: spectator final hash"; fail=1; }
[ $fail -eq 0 ] && echo "OK $name" || echo "FAILED $name"
exit $fail
