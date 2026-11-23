#!/bin/bash
# SPDX-License-Identifier: MIT
# The time-sync matrix of engine-vs-Slippi-Dolphin runs (np_dolphin_pair_linux.sh), one pair at a time under the
# laptop's game slots: decider x stage x mock relay lag (one way, both directions) x time sync rule x N matches.
#   np_dolphin_matrix_linux.sh OUTDIR [N] [FRAMES]
# Env: DECIDERS (default "engine"), STAGES (default "0x1C 0x20"), LAGS (default "30 60"), RULES (default "old new":
# old = the default, new = MELEE_TIMESYNC_NEW=1). Each match uses its own pad seeds (Dolphin's driver and the engine's
# --slippi-padgen). EXTRA_PAIR_ARGS: more np_dolphin_pair_linux.sh options for every run (e.g. "--engine-out-ms 80").
# OUTDIR/summary.tsv: one row per match (compared, equal, soft, hard, first bad frame).
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
O=${1:?outdir}; N=${2:-3}; FR=${3:-3600}; mkdir -p "$O"; O="$(cd "$O" && pwd)"
lock=(); [ -x "$HOME/melee_out/locks/game.sh" ] && lock=("$HOME/melee_out/locks/game.sh" 2)
[ -f "$O/summary.tsv" ] || printf 'decider\tstage\tlag\trule\tmatch\tengine_rc\tcompared\tequal\tsoft\thard\tfirst_bad\tengine_line\n' > "$O/summary.tsv"
for dec in ${DECIDERS:-engine}; do for st in ${STAGES:-0x1C 0x20}; do for lag in ${LAGS:-30 60}; do
for rule in ${RULES:-old new}; do for m in $(seq "$N"); do
  R=$O/$dec-$st-$lag-$rule-$m
  [ -f "$R/result.txt" ] && continue
  seed=$((m * 7 + lag))
  new=(); [ "$rule" = new ] && new=(MELEE_TIMESYNC_NEW=1)
  env "${new[@]}" GAME_TIMEOUT=$((FR / 60 + 240)) "${lock[@]}" bash "$here/np_dolphin_pair_linux.sh" "$R" --decider $dec \
    --stage $st --lag-ms $lag ${EXTRA_PAIR_ARGS} --frames $FR --pads $seed -- --slippi-padgen $((seed + 1000)) > /dev/null 2>&1
  python3 -I - "$R" "$dec" "$st" "$lag" "$rule" "$m" >> "$O/summary.tsv" <<'EOF'
import os, re, sys
o, *k = sys.argv[1:]
res = open(os.path.join(o, "result.txt")).read() if os.path.exists(os.path.join(o, "result.txt")) else ""
rc = re.search(r"engine_rc=(\d+)", res)
c = re.search(r": (\d+)/(\d+) equal .*?, (\d+) soft, (\d+) hard", res)
first = "-"
m = re.search(r"frame (\d+): dolphin", open(os.path.join(o, "cmp.txt")).read()) if os.path.exists(os.path.join(o, "cmp.txt")) else None
if m:
    first = m.group(1)
log = open(os.path.join(o, "engine.log"), errors="replace").read() if os.path.exists(os.path.join(o, "engine.log")) else ""
live = re.search(r"(\d+) compared with the peer: (\d+) soft, (\d+) HARD", log)
ts = re.search(r"time sync held (\d+) advanced (\d+)", log)
rb = re.search(r"rollbacks (\d+) \(max depth (\d+)", log)
line = (f"live {live.group(1)} cmp {live.group(2)} soft {live.group(3)} hard" if live else "live -") + \
    (f"; ts held {ts.group(1)} adv {ts.group(2)}" if ts else "") + (f"; rb {rb.group(1)} d{rb.group(2)}" if rb else "")
print("\t".join(k + [rc.group(1) if rc else "-", c.group(2) if c else "-", c.group(1) if c else "-",
                     c.group(3) if c else "-", c.group(4) if c else "-", first, line]))
EOF
done; done; done; done; done
column -t -s $'\t' "$O/summary.tsv" | cut -c1-150
