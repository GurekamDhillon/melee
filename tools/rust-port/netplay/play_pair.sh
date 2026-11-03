#!/bin/bash
# Two `melee play --netplay` instances on loopback (127.0.0.1), scripted pads (no OS input), a 60 s match.
#   bash tools/rust-port/netplay/play_pair.sh NAME LAG_MS LOSS_PCT [--headless] [extra play args]
# Each peer delays its INBOUND datagrams by LAG_MS and drops LOSS_PCT % of them (--np-lag-ms / --np-loss-pct), so
# the round trip is 2 x LAG_MS. Writes $NP_SCRATCH/NAME-{host,guest}.{log,csv,mrpl}; compares the per-frame final hash
# CSVs of both peers and plays the host's recorded replay back to the same final hash. Exit 0 = all equal.
set -u
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"
ws="${GW_WORKSPACE:-E:/Projects/Melee Workspace}"
if [ -z "${GW_ISO_VANILLA:-}" ] && [ -f "$ws/.env" ]; then set -a; . "$ws/.env"; set +a; fi
name=$1; lag=$2; loss=$3; shift 3
exe="$repo/build/cargo/${PROFILE:-release}/melee.exe"
out="${NP_SCRATCH:-${TEMP:-/tmp}/np}"; mkdir -p "$out"
port=$((51600 + RANDOM % 300))
export MELEE_VOLUME=3 MELEE_PAD_IGNORE_ADAPTER=1 MELEE_WINDOW_UNFOCUSED=1 MELEE_SHOW_FPS=1
common=(play --iso "$GW_ISO_VANILLA" --time-secs 60 --np-lag-ms "$lag" --np-loss-pct "$loss" --max-frames 9000 "$@")
MELEE_WINDOW_TITLE="netplay: $name host" MELEE_PAD_SCRIPT="$here/np_fight.txt" bash "$here/idle.sh" "$exe" "${common[@]}" \
  --netplay host --np-port $port --match falcon,32 --np-seed 4242 --np-hash-out "$out/$name-host.csv" --record "$out/$name-host.mrpl" >"$out/$name-host.log" 2>&1 &
h=$!
sleep 1
MELEE_WINDOW_TITLE="netplay: $name guest" MELEE_PAD_SCRIPT="$here/np_fight2.txt" bash "$here/idle.sh" "$exe" "${common[@]}" \
  --netplay join 127.0.0.1:$port --match fox --np-hash-out "$out/$name-guest.csv" --record "$out/$name-guest.mrpl" >"$out/$name-guest.log" 2>&1
rg=$?
wait $h; rh=$?
grep -h "netplay \(host\|guest\): \(match over\|final\|ERROR\|[0-9]* final\)" "$out/$name-host.log" "$out/$name-guest.log"
grep -h "status:" "$out/$name-host.log" | tail -1
fail=0
[ $rh -eq 0 ] && [ $rg -eq 0 ] || { echo "FAIL $name: exit host $rh guest $rg"; fail=1; }
nh=$(($(wc -l <"$out/$name-host.csv") - 1)); ng=$(($(wc -l <"$out/$name-guest.csv") - 1))
# frames both peers finalised must carry the same hash
common_rows=$(join -t, <(tail -n +2 "$out/$name-host.csv" | sort) <(tail -n +2 "$out/$name-guest.csv" | sort) | wc -l)
diffs=$(join -t, <(tail -n +2 "$out/$name-host.csv" | sort -t, -k1,1) <(tail -n +2 "$out/$name-guest.csv" | sort -t, -k1,1) | awk -F, '$2!=$3' | wc -l)
echo "$name: host $nh frames, guest $ng frames, $common_rows frames in both, $diffs with different hashes"
[ "$diffs" -eq 0 ] && [ "$nh" -gt 1000 ] || { echo "FAIL $name: hashes"; fail=1; }
MELEE_WINDOW_TITLE="netplay: $name replay" bash "$here/idle.sh" "$exe" play --headless --no-audio --iso "$GW_ISO_VANILLA" --replay "$out/$name-host.mrpl" >"$out/$name-replay.log" 2>&1 || { echo "FAIL $name: replay"; fail=1; }
grep -h "^play: replay: " "$out/$name-replay.log"
[ $fail -eq 0 ] && echo "OK $name" || echo "FAILED $name"
exit $fail
