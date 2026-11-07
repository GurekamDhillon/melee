#!/usr/bin/env bash
# Float-trace sweep (docs/rust-port/notes/semantics.md, round 3): trace the fp-audit "Mismatching functions"
# on both sides of one replay and match them by value.
#
#   sweep.sh <replay.slp> <out dir> [per=2] [minops=4]
#
# 1. the Rust trace build (RUSTFLAGS=--cfg fptrace; $FPTRACE_EXE, default _build/cargo-agents/semantics/trace/fast/melee.exe)
#    writes the first <per> calls of each function with >= <minops> float ops as <name>.f<frame>.c<call>.tsv
# 2. a Dolphin with the dump patch ($DOLPHIN_ORACLE_DIR; docs/rust-port/notes/semantics.md) dumps exactly those
#    calls (MELEE_DOLPHIN_NHV1_DUMP_AT=pc:frame:call)
# 3. gekko.py replays each dump, match.py compares, and a summary line per call goes to <out dir>/summary.tsv
# Dumps hold the console's RAM (disc-derived): keep <out dir> outside the repo and delete it afterwards.
set -u
slp=$1; out=$2; per=${3:-2}; minops=${4:-4}
here=$(cd "$(dirname "$0")" && pwd); ws=$(cd "$here/../../.." && pwd)
exe=${FPTRACE_EXE:-$ws/../../_build/cargo-agents/semantics/trace/fast/melee.exe}
[ -f "$exe" ] || exe="$ws/_build/cargo-agents/semantics/trace/fast/melee.exe"
mkdir -p "$out/rust" "$out/dump"
# name -> address from the fp-audit table (Rust names compared case-insensitively)
map=$(sed -n '/^## Mismatching functions/,/^## /p' "$ws/docs/rust-port/notes/fp-audit.md" | awk -F'|' 'NF>4 && $2 ~ /[0-9A-F]{8}/ {gsub(/ /,"",$2); gsub(/ /,"",$3); print $2, $3}')
fns=$(echo "$map" | awk '{print $2}' | tr '\n' ',')
fns="$fns$(echo "$map" | awk '{print tolower($2)}' | tr '\n' ',')"
MELEE_WINDOW_UNFOCUSED=1 MELEE_VOLUME=3 MELEE_FPTRACE_FN="$fns" MELEE_FPTRACE_MINOPS=$minops MELEE_FPTRACE_PER=$per \
  MELEE_FPTRACE_DIR="$(cygpath -w "$out/rust")" MELEE_FPTRACE_EXIT=1 \
  "$exe" slpcheck --slp "$slp" --iso "$GW_ISO_VANILLA" > "$out/rust.log" 2>&1
at=""
for f in "$out"/rust/*.tsv; do
  [ -f "$f" ] || continue
  b=$(basename "$f" .tsv); name=${b%%.f*}; rest=${b#*.f}; fr=${rest%%.c*}; c=${rest#*.c}
  addr=$(echo "$map" | awk -v n="$name" 'tolower($2)==tolower(n) {print $1; exit}')
  [ -n "$addr" ] && at="$at${at:+,}$addr:$fr:$c"
done
[ -n "$at" ] || { echo "sweep: no traced calls"; exit 0; }
# one Dolphin run per address: an HLE hook cannot tell which probe address it is at (the JIT leaves pc stale)
for pc in $(echo "$at" | tr ',' '\n' | cut -d: -f1 | sort -u); do
  want=$(echo "$at" | tr ',' '\n' | grep "^$pc:" | paste -sd,)
  MELEE_DOLPHIN_NHV1_PROBE_AT=$pc MELEE_DOLPHIN_NHV1_DUMP="$(cygpath -w "$out/dump")\\d" MELEE_DOLPHIN_NHV1_DUMP_AT=$want \
    powershell -NoProfile -ExecutionPolicy Bypass -File "$(cygpath -w "$ws/tools/rust-port/dolphin/run_dolphin.ps1")" \
    -Slp "$(cygpath -w "$slp")" -Out "$(cygpath -w "$out/dolphin")" -TimeoutSec ${SWEEP_TIMEOUT:-900} >> "$out/dolphin.log" 2>&1
done
: > "$out/summary.tsv"
for f in "$out"/rust/*.tsv; do
  b=$(basename "$f" .tsv); name=${b%%.f*}; rest=${b#*.f}; fr=${rest%%.c*}; c=${rest#*.c}
  addr=$(echo "$map" | awk -v n="$name" 'tolower($2)==tolower(n) {print $1; exit}')
  d="$out/dump/d.$addr.f$fr.c$c.bin"
  if [ ! -f "$d" ]; then printf '%s\tf%s c%s\tno dolphin call\n' "$name" "$fr" "$c" >> "$out/summary.tsv"; continue; fi
  python -I "$here/gekko.py" -o "$out/rust/$b.ppc" "$d" > /dev/null 2>&1
  python -I "$here/match.py" "$f" "$out/rust/$b.ppc" > "$out/rust/$b.match" 2>&1
  printf '%s\tf%s c%s\t%s\n' "$name" "$fr" "$c" "$(tail -1 "$out/rust/$b.match")" >> "$out/summary.tsv"
  [ -n "${SWEEP_KEEP:-}" ] || rm -f -- "$out/dump/d.$addr.f$fr.c$c.bin"
done
cat "$out/summary.tsv"
