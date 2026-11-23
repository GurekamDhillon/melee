#!/bin/bash
# SPDX-License-Identifier: MIT
# The packet-reorder matrix: engine-vs-Slippi-Dolphin pairs (reorder_pair.sh) with the mock relay's jitter (datagrams
# reordered), one pair at a time under the laptop's game slots (game.sh 2, as the other harness runs queue).
#   reorder_matrix.sh OUTDIR [FRAMES]
# Env: CELLS: lines "jitter stage lag seed" (default: jitter 8, Dream Land, 30 ms, seeds 4001-4020 first, then jitter 15,
# lag 60, Pokemon Stadium), DECIDER (engine). Seed = Dolphin's --pads and the engine's --slippi-padgen.
# OUTDIR/summary.tsv: one row per run (checksum compare, the first input mismatch and the first state difference from the
# two .slp files: slp_input_diff.py).
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
O=${1:?outdir}; FR=${2:-3000}; mkdir -p "$O"; O="$(cd "$O" && pwd)"
lock=(); [ -x "$HOME/melee_out/locks/game.sh" ] && lock=("$HOME/melee_out/locks/game.sh" 2)
if [ -z "$CELLS" ]; then
  CELLS=$(for c in "8 0x1C 30" "15 0x1C 30" "8 0x1C 60" "15 0x1C 60" "8 3 30" "15 3 30" "8 3 60" "15 3 60"; do
    for s in $(seq 4001 4020); do echo "$c $s"; done; done)
fi
[ -f "$O/summary.tsv" ] || printf 'jitter\tstage\tlag\tseed\tengine_rc\tcompared\tequal\tsoft\thard\tfirst_bad_cs\tfirst_input\tfirst_pre\tengine_line\n' > "$O/summary.tsv"
while read -r jit st lag seed; do
  [ -n "$seed" ] || continue
  cell=j$jit-$st-$lag-$seed; R=$O/$cell
  grep -qP "^$jit\t$st\t$lag\t$seed\t" "$O/summary.tsv" && continue
  # a run whose result is there but not its row (the loop was stopped mid-run): only the row is made
  [ -f "$R/result.txt" ] || GAME_TIMEOUT=$((FR / 60 + 240)) "${lock[@]}" bash "$here/reorder_pair.sh" "$R" --decider ${DECIDER:-engine} --stage $st \
    --lag-ms $lag --jitter-ms $jit --mm-port $((44100 + seed % 100)) --title "$cell" --frames $FR --pads $seed \
    -- --slippi-padgen $seed > /dev/null 2>&1
  [ -f "$R/result.txt" ] || echo "no result (rc $?)" > "$R/result.txt"
  python3 -I - "$R" "$jit" "$st" "$lag" "$seed" >> "$O/summary.tsv" <<'EOF'
import os, re, sys
o, *k = sys.argv[1:]
rd = lambda n: open(os.path.join(o, n), errors="replace").read() if os.path.exists(os.path.join(o, n)) else ""
res, cmp_, inp, log = rd("result.txt"), rd("cmp.txt"), rd("inputs.txt"), rd("engine.log")
rc = re.search(r"engine_rc=(\d+)", res)
c = re.search(r": (\d+)/(\d+) equal .*?, (\d+) soft, (\d+) hard", cmp_)
m = re.search(r"frame (\d+): dolphin", cmp_)
fi = re.search(r"FIRST INPUT MISMATCH: frame (-?\d+) port (\d)", inp)
fp = re.search(r"first Pre-Frame difference: frame (-?\d+) port (\d)", inp)
cl = re.search(r"CLASS (\w+)", inp)
live = re.search(r"(\d+) compared with the peer: (\d+) soft, (\d+) HARD", log)
rb = re.search(r"rollbacks (\d+) \(max depth (\d+)", log)
line = (f"live {live.group(1)} cmp {live.group(2)} soft {live.group(3)} hard" if live else "live -") + \
    (f"; rb {rb.group(1)} d{rb.group(2)}" if rb else "")
print("\t".join(k + [rc.group(1) if rc else "-", c.group(2) if c else "-", c.group(1) if c else "-",
                     c.group(3) if c else "-", c.group(4) if c else "-", m.group(1) if m else "-",
                     f"{fi.group(1)}/p{int(fi.group(2)) + 1}" if fi else ("none" if "every common frame" in inp else "-"),
                     (f"{fp.group(1)}/p{int(fp.group(2)) + 1}" if fp else "-") + (f" {cl.group(1)}" if cl else ""), line]))
EOF
  # a passing run's bulky files go (the summary row and result/cmp/inputs/logs stay)
  if grep -qP "\t0\t-\tnone\t-\t" <(tail -1 "$O/summary.tsv"); then rm -rf "$R/user_P3A" "$R/cap.txt" "$R/state_P3A.txt" "$R/dslp" "$R/engine.slp" "$R/checksums_P3A.txt"; fi
done <<< "$CELLS"
column -t -s $'\t' "$O/summary.tsv" | cut -c1-200
