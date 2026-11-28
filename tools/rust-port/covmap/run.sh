#!/bin/bash
# SPDX-License-Identifier: MIT
# Coverage-map runs (notes/coverage-map.md) over a pick.py subset with the cov build (built with --features referee).
#   MODE=count  run.sh SUBSET.tsv OUT_DIR   per replay: slpcheck --full with LLVM_PROFILE_FILE, then fncounts.py
#                                           -> OUT_DIR/count/<set>_<file>.tsv (raw profile and export deleted)
#   MODE=referee run.sh SUBSET.tsv OUT_DIR  per replay: slpcheck --full with MELEE_REFEREE_NATIVE -> OUT_DIR/ref/<..>.txt
# Env: EXE (cov build), JOBS (default 4). Headless, no input, adapter ignored, volume 3, below-normal priority.
# Pauses while the owner plays (owner-play.sh holds the build locks, or a melee-slippi.exe is running).
set -u
SUB=$1; OUT=$2; MODE=${MODE:-count}; JOBS=${JOBS:-4}
W="E:/Projects/Melee Workspace"
EXE=${EXE:-$W/_build/agents/covmap/target/cov/melee.exe}
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
L="$(cd "$HERE/../../../rust" && rustc --print sysroot)/lib/rustlib/x86_64-pc-windows-msvc/bin"
set -a; [ -f "$W/.env" ] && . "$W/.env"; set +a
export MELEE_INPUT=n MELEE_PAD_IGNORE_ADAPTER=1 MELEE_WINDOW_HIDDEN=1 MELEE_WINDOW_UNFOCUSED=1 MELEE_VOLUME=3
declare -A DIRS=([ext]="$W/scratch/replays-ext" [hero]="$W/_build/replays/hero-2026-11-06" [lunar]="$W/_build/slp" [gang]="$W/Gang-Steals" [lunar2]="$W/_build/replays/lunar-2026-11-20" [v320]="$W/_build/replays/lunar-2026-11-27/v320")
mkdir -p "$OUT/count" "$OUT/ref" "$OUT/tmp"
owner_playing() {
  grep -qs owner-play "$W"/scratch/.cargo-build.lock*/owner && return 0
  tasklist //FI "IMAGENAME eq melee-slippi.exe" 2>/dev/null | grep -qi melee-slippi && return 0
  return 1
}
one() {
  s=$1; rel=$2; f="$3/$rel"; b="${s}_${rel%.slp}"; b=$(echo "$b" | tr '/ ' '__')
  while owner_playing; do sleep 60; done
  if [ "$MODE" = count ]; then
    [ -s "$OUT/count/$b.tsv" ] && return 0
    d="$OUT/tmp/$b"; mkdir -p "$d"
    MELEE_WINDOW_TITLE="covmap agent: count $b" LLVM_PROFILE_FILE="$d/x.%p.profraw" nice -n 10 "$EXE" slpcheck --slp "$f" --full > "$d/log.txt" 2>&1
    grep -a "^SLPCHECK" "$d/log.txt" > "$OUT/count/$b.status" || echo "SLPCHECK none" > "$OUT/count/$b.status"
    "$L/llvm-profdata" merge -sparse "$d"/*.profraw -o "$d/r.profdata" && \
    "$L/llvm-cov" export "$EXE" -instr-profile="$d/r.profdata" -skip-expansions -skip-branches > "$d/cov.json" && \
    python -I "$HERE/fncounts.py" "$d/cov.json" "$OUT/count/$b.tsv.part" && mv "$OUT/count/$b.tsv.part" "$OUT/count/$b.tsv"
    rm -rf "$d"
  else
    [ -s "$OUT/ref/$b.txt" ] && return 0
    MELEE_WINDOW_TITLE="covmap agent: referee $b" MELEE_REFEREE_NATIVE="$OUT/ref/$b.part" nice -n 10 "$EXE" slpcheck --slp "$f" --full > "$OUT/ref/$b.log" 2>&1
    [ -s "$OUT/ref/$b.part" ] && mv "$OUT/ref/$b.part" "$OUT/ref/$b.txt"
    grep -a "^SLPCHECK" "$OUT/ref/$b.log" > "$OUT/ref/$b.status"; rm -f "$OUT/ref/$b.log"
  fi
}
export -f one owner_playing; export OUT EXE MODE HERE L W
awk -F'\t' 'NR>1 {print $1"\t"$2}' "$SUB" | while IFS=$'\t' read -r s rel; do printf '%s\t%s\t%s\n' "$s" "$rel" "${DIRS[$s]}"; done \
  | xargs -P "$JOBS" -d '\n' -I{} bash -c 'IFS=$'"'"'\t'"'"' read -r s rel d <<< "$1"; one "$s" "$rel" "$d"' _ {}
echo "done: $(ls "$OUT/$([ "$MODE" = count ] && echo count || echo ref)" | grep -c '\.\(tsv\|txt\)$') files"
