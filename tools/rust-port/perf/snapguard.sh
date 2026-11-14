#!/bin/bash
export MELEE_WINDOW_HIDDEN=${MELEE_WINDOW_HIDDEN:-1}  # agents' game windows are invisible (MELEE_WINDOW_HIDDEN=0 shows them)
# Rollback snapshot perf guard (docs/rust-port/notes/perf.md "Snapshot perf guard"): depth-7 synctest of cpu_fox_falco_dl,
# N runs (default 3), the minimum of each of save / load / total ms per tick against snap_baseline.tsv. Fails when one is
# more than 25% (and more than 0.05 ms) over its baseline. The minimum of N runs keeps it usable on a loaded machine.
#   bash tools/rust-port/perf/snapguard.sh [--runs N] [--update] [--exe EXE]
# Headless, unfocused, no OS input; every run has a hard --timeout so a hang ends itself.
set -u
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"; cd "$repo"
ws="${GW_WORKSPACE:-E:/Projects/Melee Workspace}"
[ -z "${GW_ISO_VANILLA:-}" ] && [ -f "$ws/.env" ] && { set -a; . "$ws/.env"; set +a; }
runs=3; update=0; exe="build/cargo/${PROFILE:-release}/melee.exe"
[ -e "$exe" ] || [ ! -e "${exe%.exe}" ] || exe="${exe%.exe}"
# Linux (notes/linux.md): no `python` on PATH means python3; the exe has no .exe suffix
command -v python >/dev/null 2>&1 || python() { python3 "$@"; }
while [ $# -gt 0 ]; do case "$1" in --runs) runs="$2"; shift ;; --update) update=1 ;; --exe) exe="$2"; shift ;; *) echo "snapguard: unknown option $1" >&2; exit 2 ;; esac; shift; done
base="$here/snap_baseline.tsv"
export MELEE_WINDOW_UNFOCUSED=1 MELEE_VOLUME=3 MELEE_PAD_IGNORE_ADAPTER=1 MELEE_WINDOW_TITLE="snapguard: cpu_fox_falco_dl"
best_s=""; best_l=""; best_t=""
for i in $(seq "$runs"); do
  out=$("$exe" synctest --engine tu --iso "$GW_ISO_VANILLA" --scenario cpu_fox_falco_dl --depth 7 --frames 600 --timeout 300 2>&1)
  line=$(grep "per tick ms" <<<"$out")
  grep -q " 0 MISMATCHES" <<<"$out" || { echo "FAIL snapguard: synctest run $i did not finish clean"; grep -v archive <<<"$out" | tail -4; exit 1; }
  s=$(sed -E 's/.* save ([0-9.]+) .*/\1/' <<<"$line"); l=$(sed -E 's/.* load ([0-9.]+) .*/\1/' <<<"$line"); t=$(sed -E 's/.*total ([0-9.]+).*/\1/' <<<"$line")
  echo "snapguard: run $i save $s load $l total $t ms/tick"
  best_s=$(python -c "print(min(float(x) for x in '$best_s $s'.split()))"); best_l=$(python -c "print(min(float(x) for x in '$best_l $l'.split()))"); best_t=$(python -c "print(min(float(x) for x in '$best_t $t'.split()))")
done
load=$(powershell -NoProfile -Command "(Get-CimInstance Win32_Processor | Measure-Object -Property LoadPercentage -Average).Average" 2>/dev/null)
echo "snapguard: min of $runs: save $best_s load $best_l total $best_t ms/tick (CPU load now ${load:-?}%)"
if [ $update = 1 ]; then
  printf '# metric\tms per tick (min of %s runs, %s, CPU load %s%%)\nsave\t%s\nload\t%s\ntotal\t%s\n' "$runs" "$(date +%F)" "${load:-?}" "$best_s" "$best_l" "$best_t" > "$base"
  echo "snapguard: baseline written to $base"; exit 0
fi
[ -f "$base" ] || { echo "FAIL snapguard: no baseline ($base); run with --update"; exit 1; }
fail=0
for m in save load total; do
  b=$(awk -F'\t' -v m=$m '$1==m{print $2}' "$base"); v=$(eval echo \$best_${m:0:1})
  if python -c "import sys; b=$b; v=$v; sys.exit(0 if v <= max(b*1.25, b+0.05) else 1)"; then echo "ok   snapguard $m $v ms (baseline $b)"; else echo "FAIL snapguard $m $v ms > 1.25 x baseline $b"; fail=1; fi
done
exit $fail
