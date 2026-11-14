#!/bin/bash
export MELEE_WINDOW_HIDDEN=${MELEE_WINDOW_HIDDEN:-1}  # agents' game windows are invisible (MELEE_WINDOW_HIDDEN=0 shows them)
# SPDX-License-Identifier: MIT
# One side of the real NAT test (REAL_NAT_TEST.md): our engine on THIS machine against the other machine, through
# Slippi's production matchmaking (mm.slippi.gg:43113), direct mode, unranked-free (direct only; ranked is refused by
# the engine). THIS USES THE REAL NETWORK, and, unlike the single-machine runs, the game traffic crosses two NATs.
#   real_nat_side.sh OUTDIR USER.json OTHER_CODE [-- extra engine args]
# USER.json: a COPY of this side's Slippi user.json (never printed). OTHER_CODE: the other side's connect code
# (e.g. ABCD#123). Runs under bash on Linux (machine B) or Git Bash on Windows (machine A). Needs GW_ISO_VANILLA (env or the workspace .env) and a release build with --features real-net.
# Writes OUTDIR/engine.log, engine.csv (online checksums per frame), engine.trace (pads per frame).
# Compare afterwards: real_nat_side.sh --compare A/engine.csv B/engine.csv
set -u
if [ "${1:-}" = "--compare" ]; then
  "${PYTHON:-$(command -v python3 || command -v python)}" -I - "$2" "$3" <<'PY'
import sys
def load(p):
    d = {}
    for l in open(p):
        f = l.strip().split(",")
        if len(f) >= 2 and f[0].lstrip("-").isdigit():
            d[int(f[0])] = f[1]
    return d
a, b = load(sys.argv[1]), load(sys.argv[2])
common = sorted(set(a) & set(b))
bad = [f for f in common if a[f] != b[f]]
print(f"common frames {len(common)}, differing {len(bad)}" + (f", first {bad[0]}" if bad else ""))
sys.exit(1 if bad or not common else 0)
PY
  exit $?
fi
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"
ws="${GW_WORKSPACE:-$(cd "$repo/../.." && pwd)}"
if [ -z "${GW_ISO_VANILLA:-}" ] && [ -f "$ws/.env" ]; then set -a; . "$ws/.env"; set +a; fi
[ -n "${GW_ISO_VANILLA:-}" ] || { echo "set GW_ISO_VANILLA (the vanilla disc image path)" >&2; exit 2; }
O=$1; U=$2; C=$3; shift 3; [ "${1:-}" = "--" ] && shift
mkdir -p "$O"; O="$(cd "$O" && pwd)"
# the binary: melee on Linux (machine B), melee.exe on Windows (machine A, Git Bash)
case "$(uname -s)" in MINGW*|MSYS*|CYGWIN*) bin=melee.exe ;; *) bin=melee ;; esac
B="${EXE:-${CARGO_TARGET_DIR:-$repo/build/cargo}/release/$bin}"
[ -x "$B" ] || { echo "no engine at $B: cargo build --release -p melee --features real-net (in rust/)" >&2; exit 2; }
export MELEE_VOLUME=3 MELEE_WINDOW_UNFOCUSED=1 MELEE_PAD_IGNORE_ADAPTER=1
P=${PORT:-$((41000 + RANDOM % 9000))}
command -v hostname >/dev/null || hostname() { uname -n; }
echo "netplay UDP port $P (no forwarding needed: the matchmaking connection punches it)"
MELEE_WINDOW_TITLE="nat-test:engine:$(hostname):real" "$B" slippi-netplay --mm enet://mm.slippi.gg:43113 --mm-real \
  --user-json "$U" --search "$C" --port $P --frames ${FRAMES:-2400} --max-seconds ${MAXS:-600} \
  --out "$O/engine.csv" --trace "$O/engine.trace" --padgen ${SEED:-$RANDOM} --char ${CHAR:-2} "$@" > "$O/engine.log" 2>&1
rc=$?
grep "^melee: \(matched\|decider\|guest\|selections\)" "$O/engine.log" | sed 's/ at [0-9.:]*//' | tail -4
echo "exit $rc (0: the game ran with no desync); send OUTDIR/engine.csv to the other side for --compare"
exit $rc
