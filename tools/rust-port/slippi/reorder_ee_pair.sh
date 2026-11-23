#!/bin/bash
# SPDX-License-Identifier: MIT
# Engine-vs-engine counterpart of reorder_pair.sh: two `melee play --slippi-mm` peers paired by the loopback mock
# (mock_mm.py with --lag-ms / --jitter-ms: datagrams reordered), inside `unshare -rn` (loopback only), both recording
# their .slp. OUTDIR/{a,b}.log, a.csv/b.csv (finalized checksums), engine.slp (A, the decider) and dslp/b.slp (B), cap.txt,
# result.txt (both sides' live compare line, the CSV compare and slp_input_diff.py on the two .slp files).
#   reorder_ee_pair.sh OUTDIR [--stage S] [--lag-ms N] [--jitter-ms N] [--mm-port P] [--frames N] [--pads SEED] [--title T]
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
if [ -z "$NP_IN_NETNS" ]; then
  NP_IN_NETNS=1 exec unshare -rn bash -c 'ip link set lo up && exec "$@"' _ bash "$0" "$@"
fi
O=$1; shift; mkdir -p "$O/dslp"; O="$(cd "$O" && pwd)"
mmp=44531; lag=0; jit=0; frames=3000; pads=1; stage=0x1C; title=reorder-ee
while [ $# -gt 0 ]; do
  case "$1" in
    --stage) stage=$2; shift ;; --lag-ms) lag=$2; shift ;; --jitter-ms) jit=$2; shift ;; --mm-port) mmp=$2; shift ;;
    --frames) frames=$2; shift ;; --pads) pads=$2; shift ;; --title) title=$2; shift ;;
    *) echo "reorder_ee_pair: unknown option $1" >&2; exit 2 ;;
  esac; shift
done
if [ -z "$GW_ISO_VANILLA" ]; then
  for f in "$HOME"/melee_linux_test/*Smash*.iso; do GW_ISO_VANILLA=$f; break; done
fi
export GW_ISO_VANILLA
MELEE=${MELEE:-$HOME/melee_rust/build/cargo-reorder/release/melee}
pids=()
cleanup() { for p in "${pids[@]}"; do kill -9 "$p" 2>/dev/null; done; }
trap cleanup EXIT; trap "exit 143" TERM INT
python3 -I "$here/mock_mm.py" --port $mmp --stages "$stage" --lag-ms $lag --jitter-ms $jit --capture "$O/cap.txt" > "$O/mm.log" 2>&1 &
pids+=($!)
sleep 1
side() { # name code other port slp
  MELEE_INPUT=n MELEE_PAD_IGNORE_ADAPTER=1 MELEE_WINDOW_HIDDEN=1 MELEE_WINDOW_UNFOCUSED=1 MELEE_KEYBOARD=0 MELEE_VOLUME=3 \
  MELEE_VSYNC=off MELEE_CARD=0 MELEE_WINDOW_TITLE="reorder:$title-$1" \
  exec "$MELEE" play --iso "$GW_ISO_VANILLA" --skip-intro --slippi-mm "ws://127.0.0.1:$mmp/v1/queue" --slippi-mm-mode direct \
    --slippi-port $4 --slippi-stage $((stage)) --slippi-code "$2" --slippi-search "$3" --slippi-frames $frames \
    --slippi-max-seconds 400 --slippi-out "$O/$1.csv" --slp "$5" --slippi-padgen $pads > "$O/$1.log" 2>&1
}
side a TSTB#001 TSTA#001 $((mmp + 100)) "$O/engine.slp" & A=$!; pids+=($A)
for i in $(seq 240); do grep -q "join from TSTB" "$O/mm.log" && break; sleep 0.5; done
side b TSTA#001 TSTB#001 $((mmp + 101)) "$O/dslp/b.slp" & B=$!; pids+=($B)
wait $A; RA=$?; wait $B; RB=$?
{
  echo "engine-vs-engine stage=$stage lag=$lag jitter=$jit pads=$pads rc=$RA/$RB"
  grep -h "compared with the peer" "$O/a.log" "$O/b.log" | sed 's/.*Slippi: //'
  grep -h "final hashes" "$O/a.log" "$O/b.log" | sed 's/.*rollbacks/rollbacks/; s/, session.*//'
  python3 -I - "$O/a.csv" "$O/b.csv" <<'EOF'
import sys
def rd(p):
    d = {}
    for ln in open(p).read().split("\n")[1:]:
        if ln:
            f, c = ln.split(","); d[int(f)] = int(c, 16)
    return d
a, b = rd(sys.argv[1]), rd(sys.argv[2])
s16 = lambda v: (v & 0xFFFF) - 0x10000 if v & 0x8000 else v & 0xFFFF
com = sorted(set(a) & set(b)); bad = [f for f in com if a[f] != b[f]]; hard = [f for f in bad if abs(s16(a[f]) - s16(b[f])) > 1]
print(f"csv: {len(com) - len(bad)}/{len(com)} equal, {len(bad) - len(hard)} soft, {len(hard)} hard" + (f", first frame {bad[0]}: a {a[bad[0]]:08X} b {b[bad[0]]:08X}" if bad else ""))
EOF
  python3 -I "$here/slp_input_diff.py" "$O" > "$O/inputs.txt" 2>&1; head -30 "$O/inputs.txt"
} > "$O/result.txt"
cat "$O/result.txt"
