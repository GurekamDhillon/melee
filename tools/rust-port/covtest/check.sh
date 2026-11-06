#!/bin/bash
# SPDX-License-Identifier: MIT
# One recorded CPU seed match against the console, in both engine modes (notes/coverage-testing.md):
#   bash tools/rust-port/covtest/check.sh MATCH.slp OUT_DIR
# 1. the console: the .slp played twice in the patched Slippi Dolphin (run_dolphin.ps1, concurrently; the two-run
#    nondeterminism check of notes/dolphin-oracle.md), raw2csv -> dolphin.csv (run2/dolphin.csv)
# 2. `melee slpcheck --cpu-slots` (the CPUs as CPU slots fed their recorded inputs) -> inputs/rust.csv
#    `melee slpcheck --cpu-ai`    (the engine's own CPU AI from the same start)        -> ai/rust.csv
#    each with LLVM_PROFILE_FILE (the coverage build), compared with compare_golden.py --ignore-seed --allow-short
# Last line: `COVCHECK <name> inputs=<match|F:cols> ai=<...> post_inputs=<slpcheck> post_ai=<slpcheck> nondet=<F|->`.
# Env: MELEE_EXE (default: the coverage build), PROFRAW_DIR (default OUT_DIR).
set -u
SLP=$1; OUT=$2
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
root="$(cd "$here/../../.." && pwd)"
ws="${GW_WORKSPACE:-E:/Projects/Melee Workspace}"
EXE=${MELEE_EXE:-$ws/_build/cargo-agents/covtest/cov/melee.exe}
PROF=${PROFRAW_DIR:-$OUT}
set -a; [ -f "$ws/.env" ] && . "$ws/.env" >/dev/null 2>&1; set +a
mkdir -p "$OUT/run2" "$OUT/inputs" "$OUT/ai"
name=$(basename "$SLP" .slp)
dol="$here/../dolphin"
SLPW=$(cygpath -w "$SLP")
# each run its own Dolphin user directory (USER_BASE/pb1, pb2): concurrent runs sharing one hang at start-up
UB=${USER_BASE:-$OUT/user}
DOLPHIN_USER_DIR="$(cygpath -w "$UB/pb1")" powershell -NoProfile -ExecutionPolicy Bypass -File "$(cygpath -w "$dol/run_dolphin.ps1")" -Slp "$SLPW" -Out "$(cygpath -w "$OUT")" -TimeoutSec 240 > "$OUT/dolphin.run.txt" 2>&1 &
DOLPHIN_USER_DIR="$(cygpath -w "$UB/pb2")" powershell -NoProfile -ExecutionPolicy Bypass -File "$(cygpath -w "$dol/run_dolphin.ps1")" -Slp "$SLPW" -Out "$(cygpath -w "$OUT/run2")" -TimeoutSec 240 > "$OUT/run2/dolphin.run.txt" 2>&1 &
# the engine runs while Dolphin plays
cmp_one() { # mode dir
  LLVM_PROFILE_FILE="$PROF/$name.$1.%p.profraw" MELEE_WINDOW_UNFOCUSED=1 MELEE_VOLUME=3 \
    "$EXE" slpcheck --slp "$SLP" --full --iso "$GW_ISO_VANILLA" "--$1" --nhv1-out "$OUT/$2/rust.csv" > "$OUT/$2/rust.log" 2>&1
}
cmp_one cpu-slots inputs
cmp_one cpu-ai ai
wait
nondet=-
for d in "$OUT" "$OUT/run2"; do
  if [ -s "$d/dolphin.nhraw" ]; then python -I "$dol/raw2csv.py" "$d/dolphin.nhraw" "$d/dolphin.csv" > /dev/null 2>&1; fi
  rm -f "$d/dolphin.nhraw"
done
if [ ! -s "$OUT/dolphin.csv" ]; then
  echo "COVCHECK $name error: Dolphin wrote no records"; exit 2
fi
if [ -s "$OUT/run2/dolphin.csv" ]; then
  if ! python "$root/tools/rust-port/compare_golden.py" "$OUT/run2/dolphin.csv" "$OUT/dolphin.csv" --allow-short --max-report 1 > "$OUT/run2/compare_runs.txt" 2>&1; then
    nondet=$(grep -m1 -o 'first divergence at frame -\?[0-9]*' "$OUT/run2/compare_runs.txt" | grep -o -- '-\?[0-9]*$'); nondet=${nondet:-?}
  fi
else
  nondet=norun2
fi
res=()
for m in inputs ai; do
  if [ ! -s "$OUT/$m/rust.csv" ]; then res+=("$m=error"); continue; fi
  if python "$root/tools/rust-port/compare_golden.py" "$OUT/$m/rust.csv" "$OUT/dolphin.csv" --ignore-seed --allow-short --max-report 1 > "$OUT/$m/compare.txt" 2>&1; then
    res+=("$m=match")
  else
    F=$(grep -m1 -o 'first divergence at frame -\?[0-9]*' "$OUT/$m/compare.txt" | grep -o -- '-\?[0-9]*$')
    cols=$(grep -m1 '^frame .*differs in' "$OUT/$m/compare.txt" | sed 's/.*differs in //; s/  */,/g')
    res+=("$m=${F:-?}:${cols:-?}")
  fi
done
pi=$(grep -a -m1 '^SLPCHECK' "$OUT/inputs/rust.log" | sed 's/^SLPCHECK [^ ]* //; s/ /_/g' | cut -c1-120)
pa=$(grep -a -m1 '^SLPCHECK' "$OUT/ai/rust.log" | sed 's/^SLPCHECK [^ ]* //; s/ /_/g' | cut -c1-120)
echo "COVCHECK $name ${res[*]} post_inputs=$pi post_ai=$pa nondet=$nondet"
