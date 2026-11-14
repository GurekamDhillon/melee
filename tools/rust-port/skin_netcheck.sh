#!/bin/bash
# Is a costume skin netplay-safe? Runs the engine with and without a skin mods folder and compares.
#
# usage: tools/rust-port/skin_netcheck.sh MODS_DIR FIGHTER REPLAY_DIR OUT_DIR [--costume N] [--peer-frames N]
#   MODS_DIR    the local mods folder holding the skin (passed as MELEE_MODS_DIR to the "skin" side only)
#   FIGHTER     a fighter token, e.g. luigi; replays are filtered to ones featuring it (char dirs from lunar_fetch,
#               or every .slp under REPLAY_DIR/FIGHTER if that directory exists)
#   --costume N sets MELEE_FORCE_COSTUME=FIGHTER:N on the skin side. NOTE: no engine reads that variable yet (checked
#               2026-11-11); until one does, install the skin as a replacement of the colour file the replays use
#               (skin.py build --slot Nr etc.) so slpcheck actually loads it
#
# (a) replays: run_all.sh twice (base, skin); verdict FRAME-EXACT when every replay has the same status, last frame
#     and first divergence in both runs (a replay that already diverges without the skin must diverge identically).
# (b) two peers: `melee netplay --host` without the skin and `--join` with it (loopback, --seed 12345,
#     --lag-ms 70 --loss-pct 5), each writing its per-frame hash CSV; verdict 0 desyncs when both CSVs are identical
#     and neither log reports a desync. Skipped with a note if this build's netplay scene cannot pick FIGHTER.
#
# Every engine run goes through the shared build lock as owner $LOCK_OWNER (default skin-netcheck), with
# MELEE_VOLUME=3, a MELEE_WINDOW_TITLE naming the job, MELEE_WINDOW_UNFOCUSED=1 and a timeout. No OS input.
# Exit 0 = netplay-safe on everything that ran; 1 = a difference; 2 = usage/setup error.
set -u
[ $# -ge 4 ] || { sed -n 2,20p "$0"; exit 2; }
MODS=$1; FIGHTER=$2; REPLAYS=$3; OUT=$4; shift 4
COSTUME=""; PEER_FRAMES=3000
while [ $# -gt 0 ]; do case $1 in --costume) COSTUME=$2; shift 2;; --peer-frames) PEER_FRAMES=$2; shift 2;; *) echo "unknown $1"; exit 2;; esac; done
HERE=$(cd "$(dirname "$0")" && pwd)
EXE=${MELEE_EXE:-build/cargo/release/melee.exe}
LOCK=${BUILDLOCK:-"/e/Projects/Melee Workspace/scratch/buildlock.sh"}
OWNER=${LOCK_OWNER:-skin-netcheck}
TO=${TIMEOUT:-900}
export MELEE_VOLUME=3 MELEE_WINDOW_UNFOCUSED=1 TIMEOUT=$TO MELEE_EXE=$EXE
locked() { if [ -x "$LOCK" ] && [ -z "${NO_LOCK:-}" ]; then "$LOCK" run "$OWNER" "$@"; else "$@"; fi; }
mkdir -p "$OUT"
[ -d "$MODS" ] || { echo "no mods dir $MODS"; exit 2; }

# ---- (a) replays --------------------------------------------------------------------------------------------------
SET="$OUT/replays"; mkdir -p "$SET"; rm -f "$SET"/*.slp
if [ -d "$REPLAYS/$FIGHTER" ]; then src="$REPLAYS/$FIGHTER"; else src="$REPLAYS"; fi
n=0; for f in "$src"/*.slp; do [ -e "$f" ] || continue; cp "$f" "$SET/"; n=$((n+1)); done
echo "replays: $n from $(basename "$src")"
[ $n -gt 0 ] || { echo "no replays featuring $FIGHTER"; exit 2; }
MELEE_WINDOW_TITLE="skin-netcheck base $FIGHTER" locked env -u MELEE_MODS_DIR bash "$HERE/slp/run_all.sh" "$SET" "$OUT/base" >/dev/null
MELEE_WINDOW_TITLE="skin-netcheck skin $FIGHTER" MELEE_MODS_DIR="$MODS" ${COSTUME:+MELEE_FORCE_COSTUME="$FIGHTER:$COSTUME"} \
  locked bash "$HERE/slp/run_all.sh" "$SET" "$OUT/skin" >/dev/null
python - "$OUT/base/all.csv" "$OUT/skin/all.csv" > "$OUT/replays.txt" <<'EOF'
import csv, sys
a = {r["file"]: r for r in csv.DictReader(open(sys.argv[1]))}; b = {r["file"]: r for r in csv.DictReader(open(sys.argv[2]))}
keys = ("status", "last_frame", "first_frame", "first_field", "frames_compared")
bad = [f for f in a if f not in b or any(a[f].get(k) != b[f].get(k) for k in keys)]
for f in sorted(a): print(f, "same" if f not in bad else "DIFF base=%s skin=%s" % ({k: a[f].get(k) for k in keys}, {k: b.get(f, {}).get(k) for k in keys}))
print("REPLAYS: %s (%d/%d identical)" % ("FRAME-EXACT" if not bad else "DIFFERENT", len(a) - len(bad), len(a)))
sys.exit(1 if bad else 0)
EOF
ra=$?; tail -1 "$OUT/replays.txt"

# ---- (b) two peers ------------------------------------------------------------------------------------------------
rb=0
if "$EXE" netplay --help 2>&1 | grep -q -- "--p1\|--chars"; then
  port=$((20000 + RANDOM % 20000))
  ( MELEE_WINDOW_TITLE="skin-netcheck host(no skin)" locked timeout "$TO" env -u MELEE_MODS_DIR "$EXE" netplay --host --port $port --seed 12345 \
      --lag-ms 70 --loss-pct 5 --frames "$PEER_FRAMES" --p1 "$FIGHTER" --csv "$OUT/peer_host.csv" > "$OUT/peer_host.log" 2>&1 ) &
  hp=$!
  MELEE_WINDOW_TITLE="skin-netcheck guest(skin)" MELEE_MODS_DIR="$MODS" timeout "$TO" "$EXE" netplay --join 127.0.0.1:$port --lag-ms 70 \
      --loss-pct 5 --frames "$PEER_FRAMES" --csv "$OUT/peer_guest.csv" > "$OUT/peer_guest.log" 2>&1
  wait $hp
  if cmp -s "$OUT/peer_host.csv" "$OUT/peer_guest.csv" && ! grep -qi "desync" "$OUT/peer_host.log" "$OUT/peer_guest.log"; then
    echo "PEERS: 0 desyncs over $(wc -l < "$OUT/peer_host.csv") frames"
  else echo "PEERS: DESYNC (see $OUT/peer_*.log)"; rb=1; fi
else
  echo "PEERS: skipped - this build's netplay scene has no fighter selection flag (--p1/--chars); see docs/rust-port/notes/net.md"
fi
[ $ra -eq 0 ] && [ $rb -eq 0 ] && { echo "VERDICT: netplay-safe on everything that ran"; exit 0; }
echo "VERDICT: DIFFERENT"; exit 1
