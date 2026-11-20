#!/bin/bash
export MELEE_WINDOW_HIDDEN=${MELEE_WINDOW_HIDDEN:-1}  # agents' game windows are invisible (MELEE_WINDOW_HIDDEN=0 shows them)
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
# NP_BIND: the address both peers bind and the guest dials (default loopback; e.g. 172.21.48.1 or ::1)
bind="${NP_BIND:-127.0.0.1}"; case "$bind" in *:*) dial="[$bind]:$port";; *) dial="$bind:$port";; esac
# NP_SERVER=HOST:PORT: connect by match code through a rendezvous server (rust/melee_relay) instead of the address
srv=(); [ -n "${NP_SERVER:-}" ] && { srv=(--np-server "$NP_SERVER"); case "$bind" in *:*) ;; *) srv+=(--np-lan "$bind");; esac; }
export MELEE_NP_SCRIPT_NO_START=1 MELEE_VOLUME=3 MELEE_PAD_IGNORE_ADAPTER=1 MELEE_WINDOW_UNFOCUSED=1 MELEE_SHOW_FPS=1
# NP_CSS_STAGES=fd,bf,ps,ys,dl,fod: one match per listed stage (each never seen before in the session: cold-stage starts)
nm=2; [ -n "${NP_CSS_STAGES:-}" ] && nm=$(echo "$NP_CSS_STAGES" | tr ',' ' ' | wc -w)
common=(play --iso "$GW_ISO_VANILLA" --np-css --time-secs 60 --quit-after-matches $nm --np-lag-ms "$lag" --np-loss-pct "$loss" --max-frames $((nm * 14000 + 10000)) "$@")
# NP_PERF=1 (frame times of the host): the host at normal priority with MELEE_PROFILE=1, the guest headless without
# audio at idle priority, so only one peer renders and the guest never competes with the host for the CPU
hrun=(bash "$here/idle.sh"); gx=(); hpro=()
[ -n "${NP_PERF:-}" ] && { hrun=(); gx=(--headless --no-audio); hpro=(MELEE_PROFILE=1); }
# NP_HOST_ENV="A=1 B=2": extra environment for the host only (e.g. MELEE_SAMPLE=report.txt MELEE_SAMPLE_SLOW_MS=14)
hpro+=(${NP_HOST_ENV:-})
env "${hpro[@]}" MELEE_WINDOW_TITLE="netplay css: $name host" MELEE_PAD_SCRIPT="$out/np_css_host.txt" "${hrun[@]}" "$exe" "${common[@]}" \
  --netplay host --np-bind "$bind" "${srv[@]}" --np-port $port --np-seed 4242 --np-hash-out "$out/$name-host.csv" --record "$out/$name-host.mrpl" >"$out/$name-host.log" 2>&1 &
h=$!
sleep 1
if [ -n "${NP_SERVER:-}" ]; then
  for _ in $(seq 100); do dial=$(grep -o "MATCH CODE [A-Z0-9]*" "$out/$name-host.log" | head -1 | cut -d" " -f3); [ -n "$dial" ] && break; sleep 0.2; done
  echo "$name: match code $dial"
fi
MELEE_WINDOW_TITLE="netplay css: $name guest" MELEE_PAD_SCRIPT="$out/np_css_guest.txt" bash "$here/idle.sh" "$exe" "${common[@]}" "${gx[@]}" \
  --netplay join "$dial" --np-bind "$bind" "${srv[@]}" --np-hash-out "$out/$name-guest.csv" --record "$out/$name-guest.mrpl" >"$out/$name-guest.log" 2>&1
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
