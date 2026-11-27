#!/bin/bash
# Run `melee slpcheck --full --slippi-checksum-out` over the frozen replay sets (scratch/replayset/frozen.tsv) and score
# each replay with hard_desync.py: would Slippi's game-ending checksum test have fired, and at which frame.
#   hard_desync_sweep.sh OUT_DIR          (env: JOBS=3, MELEE_EXE, TIMEOUT=1200, SETS="ext hero lunar gang lunar2")
# OUT_DIR/rows.csv: set,relpath,status,first_divergence,frames,first_hard,max_delta,version,frames_other_set  (one row per replay)
# Headless; nothing here sends OS input. Per-frame checksum files are kept gzipped for re-scoring.
set -u
OUT=$1
W="E:/Projects/Melee Workspace"
EXE=${MELEE_EXE:?set MELEE_EXE}
JOBS=${JOBS:-3}; TIMEOUT=${TIMEOUT:-1200}
SETS=${SETS:-"ext hero lunar gang lunar2"}
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
mkdir -p "$OUT/tmp"
export MELEE_WINDOW_UNFOCUSED=1 MELEE_WINDOW_HIDDEN=1 MELEE_INPUT=n MELEE_PAD_IGNORE_ADAPTER=1 MELEE_VOLUME=3
set -a; [ -f "$W/.env" ] && . "$W/.env"; set +a
declare -A DIRS=([ext]="$W/scratch/replays-ext" [hero]="$W/_build/replays/hero-2026-11-06" [lunar]="$W/_build/slp" [gang]="$W/Gang-Steals" [lunar2]="$W/_build/replays/lunar-2026-11-20")
one() {
  s=$1; rel=$2; f="$3/$rel"; b="${s}_$(echo "$rel" | tr '/ ' '__')"; b=${b%.slp}
  [ -s "$OUT/tmp/$b.row" ] && [ -s "$OUT/tmp/$b.ck.gz" ] && return 0
  ck="$OUT/tmp/$b.ck"
  MELEE_WINDOW_TITLE="referee:hard-desync:$b" timeout "$TIMEOUT" "$EXE" slpcheck --slp "$f" --full --csv "$OUT/tmp/$b.csv" --slippi-checksum-out "$ck" > "$OUT/tmp/$b.log" 2>&1
  st=$(python -I -c "import csv,sys
r=list(csv.DictReader(open(sys.argv[1],encoding='utf-8',errors='replace')))
print((r[0].get('status','?')+','+str(r[0].get('first_frame','')) ) if r else 'crash,')" "$OUT/tmp/$b.csv" 2>/dev/null || echo "crash,")
  sc=$( [ -s "$ck" ] && python -I "$HERE/hard_desync.py" "$f" "$ck" 2>/dev/null || echo "0,,,,")
  echo "$s,$rel,$st,$sc" > "$OUT/tmp/$b.row"
  gzip -f "$ck"; rm -f "$OUT/tmp/$b.log"  # kept gzipped: a scorer fix re-scores without engine runs
}
export -f one; export OUT EXE TIMEOUT HERE
for s in $SETS; do
  awk -F'\t' -v s="$s" 'NR>1 && $1==s {print $2}' "$W/scratch/replayset/frozen.tsv" | while IFS= read -r rel; do printf '%s\t%s\t%s\n' "$s" "$rel" "${DIRS[$s]}"; done
done | xargs -P "$JOBS" -d '\n' -I{} bash -c 'IFS=$'"'"'\t'"'"' read -r s rel d <<< "$1"; one "$s" "$rel" "$d"' _ {}
echo "set,relpath,status,first_divergence,frames,first_hard,max_delta,version,frames_other_set" > "$OUT/rows.csv"
cat "$OUT"/tmp/*.row >> "$OUT/rows.csv"
echo "done: $(($(wc -l < "$OUT/rows.csv") - 1)) rows in $OUT/rows.csv"
