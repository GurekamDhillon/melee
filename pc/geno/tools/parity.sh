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
if [ -f "$here/parity.local.conf" ]; then default_conf="$here/parity.local.conf"; else default_conf="$here/parity.conf"; fi
conf="${PARITY_CONF:-$default_conf}"
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
declare -A pins # slp path -> "frame=F port=P field=NAME console=0x.. port=0x.." (normalised)
norm_pin() {    # the exe's FAIL line or a conf pin -> "F P FIELD CONSOLEBITS PORTBITS", upper-case hex
    echo "$1" | tr 'a-f' 'A-F' | sed -E 's/0X/0x/g'
}
while IFS= read -r line || [ -n "$line" ]; do
    line="${line%%#*}"
    line="$(echo "$line" | sed -e 's/[[:space:]]*$//' -e 's/^[[:space:]]*//')"
    case "$line" in
    iso=*) iso="${line#iso=}" ;;
    slp=*) slps+=("${line#slp=}") ;;
    expect=*)
        # the slp path may contain spaces: everything before " frame=" is the path
        rest="${line#expect=}"
        p="${rest%% frame=*}"
        spec="frame=${rest#* frame=}"
        f="$(echo "$spec" | sed -nE 's/.*frame=(-?[0-9]+).*/\1/p')"
        po="$(echo "$spec" | sed -nE 's/.*port=([0-9]+) .*/\1/p')"
        fi_="$(echo "$spec" | sed -nE 's/.*field=([a-z_]+).*/\1/p')"
        cb="$(echo "$spec" | sed -nE 's/.*console=(0x[0-9A-Fa-f]+).*/\1/p')"
        pb="$(echo "$spec" | sed -nE 's/.* port=(0x[0-9A-Fa-f]+).*/\1/p')"
        pins["$p"]="$(norm_pin "$f $po $fi_ $cb $pb")"
        ;;
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
    pin="${pins[$slp]:-}"
    case "$verdict" in
    PASS*)
        if [ -n "$pin" ]; then
            echo "FAIL  $name: matches fully now - pin can be removed - fixed (pin: $pin)"
            fail=$((fail + 1))
        else
            echo "PASS  $name: ${verdict#PASS }"; pass=$((pass + 1))
        fi
        ;;
    FAIL*)
        got="$(echo "$verdict" | sed -nE 's/^FAIL frame (-?[0-9]+) port ([0-9]+)[^f]* field ([a-z_]+):.* bits console (0x[0-9A-Fa-f]+) port (0x[0-9A-Fa-f]+).*/\1 \2 \3 \4 \5/p')"
        got="$(norm_pin "$got")"
        if [ -n "$pin" ] && [ "$got" = "$pin" ]; then
            echo "PASS  $name: pinned divergence, exactly as pinned ($pin)"; pass=$((pass + 1))
        else
            echo "FAIL  $name: ${verdict#FAIL }${pin:+ (pinned: $pin)}"; fail=$((fail + 1))
        fi
        ;;
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
