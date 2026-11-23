#!/bin/bash
export MELEE_WINDOW_HIDDEN=${MELEE_WINDOW_HIDDEN:-1}  # agents' game windows are invisible (MELEE_WINDOW_HIDDEN=0 shows them)
# Live spectator output proof (live_out.rs): a headless netplay pair on loopback, the host with --spectate-port and/or
# --relay-url, consumers outside the game: a Slippi spectate client and/or a relay receiver (CONSUMER commands).
#   SLIPPI_CMD='node slippi_live.js {port} {out}' RELAY_CMD='node relay_recv.js {port} {out}' RELAY_KEY=FILE \
#     bash tools/rust-port/netplay/live_pair.sh NAME
# Writes $NP_SCRATCH/NAME-host.slp (the host's final .slp, from its --record playback) and NAME-slippi.bin /
# NAME-relay.bin (the event bytes each consumer received); the caller compares them. Loopback only, no OS input.
set -u
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"
ws="${GW_WORKSPACE:-E:/Projects/Melee Workspace}"
if [ -z "${GW_ISO_VANILLA:-}" ] && [ -f "$ws/.env" ]; then set -a; . "$ws/.env"; set +a; fi
name=$1
exe="$repo/build/cargo/${PROFILE:-release}/melee.exe"
out="${NP_SCRATCH:-${TEMP:-/tmp}/np}"; mkdir -p "$out"
port=$((52600 + RANDOM % 300)); slport=$((port + 400)); rport=$((port + 800))
export MELEE_VOLUME=3 MELEE_PAD_IGNORE_ADAPTER=1 MELEE_WINDOW_UNFOCUSED=1
common=(play ${HEADLESS---headless} --no-audio --iso "$GW_ISO_VANILLA" --time-secs ${SECS:-60} --max-frames 9000)
live=()
pids=()
[ "${OFF:-}" = 1 ] && SLIPPI_CMD= RELAY_CMD=
if [ -n "${SLIPPI_CMD:-}" ]; then
  live+=(--spectate-port $slport)
  c=${SLIPPI_CMD//\{port\}/$slport}; c=${c//\{out\}/$out/$name-slippi.bin}
  # a Slippi client dials once: start it when the output child's server is up
  rm -f "$out/$name-child.log"
  (for _ in $(seq 600); do grep -q "Slippi spectate server" "$out/$name-child.log" 2>/dev/null && break; sleep 0.2; done; eval "$c") >"$out/$name-slippi.log" 2>&1 &
  pids+=($!)
fi
if [ -n "${RELAY_CMD:-}" ]; then
  live+=(--relay-url "ws://127.0.0.1:$rport/v1/live/test/$name" --relay-key "$RELAY_KEY")
  c=${RELAY_CMD//\{port\}/$rport}; c=${c//\{out\}/$out/$name-relay.bin}
  (eval "$c") >"$out/$name-relay.log" 2>&1 &
  pids+=($!)
fi
rm -f "$out/$name-host.slp" "$out/$name-child.slp"
export MELEE_LIVE_CHILD_LOG="$out/$name-child.log" MELEE_LIVE_CHILD_SLP="$out/$name-child.slp"
MELEE_WINDOW_TITLE="livespec: $name host" MELEE_PAD_SCRIPT="$here/np_fight.txt" "$exe" "${common[@]}" ${HOST_EXTRA:-} \
  --netplay host --np-port $port --match falcon,32 --np-seed 4242 "${live[@]}" --np-hash-out "$out/$name-host.csv" \
  --record "$out/$name-host.mrpl" --slp "$out/$name-host.slp" >"$out/$name-host.log" 2>&1 &
h=$!
sleep 1
MELEE_PROFILE= MELEE_WINDOW_TITLE="livespec: $name guest" MELEE_PAD_SCRIPT="$here/np_fight2.txt" "$exe" "${common[@]}" \
  --netplay join "127.0.0.1:$port" --match fox --np-hash-out "$out/$name-guest.csv" >"$out/$name-guest.log" 2>&1 &
g=$!
wait $g; rg=$?
wait $h; rh=$?
for p in "${pids[@]}"; do wait $p; done
echo "$name: host exit $rh guest exit $rg"
grep -h "live\b\|live:" "$out/$name-host.log" | head -5
grep -h "^live:" "$out/$name-child.log" 2>/dev/null | head -20
for f in slippi relay; do [ -f "$out/$name-$f.log" ] && grep -h "RESULT\|gap\|drop\|stall\|refused" "$out/$name-$f.log"; done
diffs=$(join -t, <(tail -n +2 "$out/$name-host.csv" | sort -t, -k1,1) <(tail -n +2 "$out/$name-guest.csv" | sort -t, -k1,1) | awk -F, '$2!=$3' | wc -l)
echo "$name: $diffs host/guest hash differences"
