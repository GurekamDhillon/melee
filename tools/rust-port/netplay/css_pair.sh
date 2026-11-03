#!/bin/bash
# Two `melee play --netplay ... --np-css` peers on loopback: the players pick on the retail character select (lockstep
# menus), two matches on one connection (the second a rematch from the results), scripted pads (no OS input).
#   bash tools/rust-port/netplay/css_pair.sh NAME LAG_MS LOSS_PCT [extra play args]
# Writes $NP_SCRATCH/NAME-{host,guest}.{log,csv,mrpl}; compares the per-frame final hashes of both matches on both
# peers, then plays the host's replay back (menus and both matches) to the same hashes. Exit 0 = all equal.
set -u
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"
ws="${GW_WORKSPACE:-E:/Projects/Melee Workspace}"
if [ -z "${GW_ISO_VANILLA:-}" ] && [ -f "$ws/.env" ]; then set -a; . "$ws/.env"; set +a; fi
name=$1; lag=$2; loss=$3; shift 3
exe="$repo/build/cargo/${PROFILE:-release}/melee.exe"
out="${NP_SCRATCH:-${TEMP:-/tmp}/np}"; mkdir -p "$out"
python "$here/gen_css_scripts.py" "$out" >/dev/null
port=$((51600 + RANDOM % 300))
export MELEE_VOLUME=3 MELEE_PAD_IGNORE_ADAPTER=1 MELEE_WINDOW_UNFOCUSED=1 MELEE_SHOW_FPS=1
common=(play --iso "$GW_ISO_VANILLA" --np-css --time-secs 60 --quit-after-matches 2 --np-lag-ms "$lag" --np-loss-pct "$loss" --max-frames 20000 "$@")
MELEE_WINDOW_TITLE="netplay css: $name host" MELEE_PAD_SCRIPT="$out/np_css_host.txt" bash "$here/idle.sh" "$exe" "${common[@]}" \
  --netplay host --np-port $port --np-seed 4242 --np-hash-out "$out/$name-host.csv" --record "$out/$name-host.mrpl" >"$out/$name-host.log" 2>&1 &
h=$!
sleep 1
MELEE_WINDOW_TITLE="netplay css: $name guest" MELEE_PAD_SCRIPT="$out/np_css_guest.txt" bash "$here/idle.sh" "$exe" "${common[@]}" \
  --netplay join 127.0.0.1:$port --np-hash-out "$out/$name-guest.csv" --record "$out/$name-guest.mrpl" >"$out/$name-guest.log" 2>&1
rg=$?
wait $h; rh=$?
grep -h "netplay \(host\|guest\): \(match\|final\|ERROR\)\|scene \(vs\|results\)\|quit after" "$out/$name-host.log" | head -20
fail=0
[ $rh -eq 0 ] && [ $rg -eq 0 ] || { echo "FAIL $name: exit host $rh guest $rg"; fail=1; }
nh=$(($(wc -l <"$out/$name-host.csv") - 1)); ng=$(($(wc -l <"$out/$name-guest.csv") - 1))
m2=$(awk -F, 'NR>1 && $1>=1000000' "$out/$name-host.csv" | wc -l)
diffs=$(join -t, <(tail -n +2 "$out/$name-host.csv" | sort -t, -k1,1) <(tail -n +2 "$out/$name-guest.csv" | sort -t, -k1,1) | awk -F, '$2!=$3' | wc -l)
echo "$name: host $nh frames ($m2 in match 2), guest $ng frames, $diffs with different hashes"
[ "$diffs" -eq 0 ] && [ "$m2" -gt 1000 ] && [ "$nh" -eq "$ng" ] || { echo "FAIL $name: hashes"; fail=1; }
MELEE_WINDOW_TITLE="netplay css: $name replay" bash "$here/idle.sh" "$exe" play --headless --no-audio --iso "$GW_ISO_VANILLA" --replay "$out/$name-host.mrpl" >"$out/$name-replay.log" 2>&1 || { echo "FAIL $name: replay"; fail=1; }
grep -h "^play: replay: " "$out/$name-replay.log"
[ $fail -eq 0 ] && echo "OK $name" || echo "FAILED $name"
exit $fail
