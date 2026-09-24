#!/bin/bash
# Vanilla-parity check: replays real console games (Slippi .slp) through the port and asserts that
# every vanilla fighter matches the console frame for frame - action state, x, y, facing, percent,
# stocks, and the RNG seed wherever the replay records it (gw_replay.c, MELEE_SLP_PARITY=1).
#
#   bash pc/geno/tools/parity.sh                 all replays in parity.conf
#   PARITY_CONF=my.conf bash pc/geno/tools/parity.sh
#   PARITY_TIMEOUT=240 ...                       seconds per replay (default 180)
#
# Uses the same GW_MELEE / GW_BUILD_ROOT as tools/port/build.sh and run.sh; each replay runs in its
# own sandbox (runs/parity-<name>) through run.sh. Missing replays or a missing disc are skipped,
# not failed. Exit status: 0 = every replay that ran passed, 1 = a divergence (the first diverging
# frame and field is printed), 2 = nothing could run.
set -uo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
conf="${PARITY_CONF:-$here/parity.conf}"
timeout_s="${PARITY_TIMEOUT:-180}"

# tools/port/run.sh lives in the root repo, above this melee checkout (worktrees/<lane> or melee/)
root="${GW_ROOT:-}"
if [ -z "$root" ]; then
    d="$here"
    while [ "$d" != "/" ] && [ -n "$d" ]; do
        if [ -f "$d/tools/port/run.sh" ]; then root="$d"; break; fi
        d="$(dirname "$d")"
    done
fi
[ -n "$root" ] && [ -f "$root/tools/port/run.sh" ] || { echo "parity: cannot find tools/port/run.sh" >&2; exit 2; }
[ -f "$conf" ] || { echo "parity: no config $conf" >&2; exit 2; }

iso=""
slps=()
while IFS= read -r line || [ -n "$line" ]; do
    line="${line%%#*}"
    line="$(echo "$line" | sed -e 's/[[:space:]]*$//' -e 's/^[[:space:]]*//')"
    case "$line" in
    iso=*) iso="${line#iso=}" ;;
    slp=*) slps+=("${line#slp=}") ;;
    esac
done < "$conf"

if [ -z "$iso" ] || [ ! -f "$iso" ]; then
    echo "parity: SKIP - the disc \"$iso\" is missing"
    exit 2
fi

build_root="${GW_BUILD_ROOT:-$root/_build}"
pass=0 fail=0 skip=0
for slp in "${slps[@]}"; do
    name="$(basename "$slp" .slp)"
    if [ ! -f "$slp" ]; then
        echo "SKIP  $name (missing: $slp)"
        skip=$((skip + 1))
        continue
    fi
    sandbox="$build_root/runs/parity-$name"
    log="$sandbox/melee-pc.log"
    rm -f "$log"
    MELEE_SLP="$slp" MELEE_SLP_PARITY=1 MELEE_VOLUME="${MELEE_VOLUME:-3}" \
        MELEE_RUN_LABEL="parity / $name" \
        bash "$root/tools/port/run.sh" "parity-$name" --iso "$iso" > /dev/null 2>&1 &
    runner=$!
    t=0
    while kill -0 "$runner" 2> /dev/null && [ "$t" -lt "$timeout_s" ]; do
        sleep 1
        t=$((t + 1))
    done
    if kill -0 "$runner" 2> /dev/null; then
        # a hang: end the one game this script started (its sandbox's own exe copy)
        winpid="$(ps -W | grep -iF "parity-$name" | grep -i "melee-pc.exe" | awk '{print $4}' | head -1)"
        [ -n "$winpid" ] && taskkill //F //PID "$winpid" > /dev/null 2>&1
        wait "$runner" 2> /dev/null
        echo "FAIL  $name: no verdict within ${timeout_s}s (log $log)"
        grep -a "replay:" "$log" 2> /dev/null | tail -3 | sed 's/^/      /'
        fail=$((fail + 1))
        continue
    fi
    wait "$runner" 2> /dev/null
    verdict="$(grep -a "parity: " "$log" 2> /dev/null | tail -1 | sed 's/.*parity: //')"
    case "$verdict" in
    PASS*) echo "PASS  $name: ${verdict#PASS }"; pass=$((pass + 1)) ;;
    FAIL*) echo "FAIL  $name: ${verdict#FAIL }"; fail=$((fail + 1)) ;;
    *)
        echo "FAIL  $name: the game exited without a verdict (log $log)"
        grep -a "FATAL\|replay:" "$log" 2> /dev/null | tail -3 | sed 's/^/      /'
        fail=$((fail + 1))
        ;;
    esac
done
echo "parity: $pass passed, $fail failed, $skip skipped"
if [ "$fail" -gt 0 ]; then exit 1; fi
if [ "$pass" -eq 0 ]; then exit 2; fi
exit 0
