#!/bin/bash
# The consolidation gate (docs/rust-port/notes/consolidation.md section 4). Run from anywhere; needs GW_ISO_VANILLA
# (the workspace .env, never printed) and, for the diff part, C snapshots (tools/rust-port/snap/dump.sh).
#   bash tools/rust-port/gate.sh [--engine legacy] [--update-progress] [--skip-build] [--scenarios a,b]
# Checks (any failure exits non-zero):
#   1. each golden scenario matches its CSV from frame 0 for at least the count recorded for the engine in
#      docs/rust-port/engine_progress.tsv (latest step's row per scenario/engine);
#   2. baseline files tools/rust-port/diff/baseline/*.txt have not grown (baseline_max.tsv), and
#      `melee diff --frames` shows no difference outside the baseline in the subsystems of docs/rust-port/migrated.txt
#      at the sampled frames of tools/rust-port/diff/frames.tsv (capped at the engine's count);
#   3. tools/rust-port/lint_logic.py introduces no new violations (when the script exists).
# --update-progress appends the measured counts (only when none went down) and lowers baseline_max.tsv.
set -u
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo="$(cd "$here/../.." && pwd)"
cd "$repo"
engine=legacy; update=0; build=1; only=""
while [ $# -gt 0 ]; do
  case "$1" in
    --engine) engine="$2"; shift ;;
    --update-progress) update=1 ;;
    --skip-build) build=0 ;;
    --scenarios) only="$2"; shift ;;
    *) echo "gate: unknown option $1" >&2; exit 2 ;;
  esac; shift
done
ws="${GW_WORKSPACE:-E:/Projects/Melee Workspace}"
if [ -z "${GW_ISO_VANILLA:-}" ] && [ -f "$ws/.env" ]; then set -a; . "$ws/.env"; set +a; fi
[ -n "${GW_ISO_VANILLA:-}" ] || { echo "gate: GW_ISO_VANILLA not set" >&2; exit 2; }
export MELEE_WINDOW_UNFOCUSED=1
case "$engine" in legacy|tu) ;; *) echo "gate: unknown engine '$engine' (legacy | tu)" >&2; exit 2 ;; esac
exe="$repo/build/cargo/release/melee.exe"
tmp="${TEMP:-/tmp}/gate.$$"; mkdir -p "$tmp"
fail=0
if [ $build = 1 ]; then
  (cd rust && CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 cargo build -p melee --release 2>&1 | tail -2) || fail=1
fi
progress=docs/rust-port/engine_progress.tsv
declare -A counts
scenarios="pad_falcon_fox_fd pad_marth_sheik_bf cpu_fox_falco_dl cpu_peach_samus_ys_items"
[ -n "$only" ] && scenarios="${only//,/ }"
for s in $scenarios; do
  idle=""; case $s in cpu_*) idle="--idle-pads" ;; esac
  if [ "$engine" = tu ]; then
    # the consolidated engine (melee_engine): frames written as they are produced, so a later crash keeps the count.
    # Pre-match draws of subsystems the engine has not taken over yet (memory-card trophy pick, stage init, BGM pick):
    # tools/rust-port/engine/pre_draws.tsv, shrinking to empty. A scenario with a row is compared with the seed column
    # (and so the hash): the particle system's draws must agree with the C build's. One without a row is not.
    pre=$(awk -F'	' -v s="$s" '$1==s{print $2}' tools/rust-port/engine/pre_draws.tsv 2>/dev/null)
    seedflag="--ignore-seed"; [ -n "$pre" ] && seedflag=""
    MELEE_ENGINE_PRE_DRAWS="${pre:-0}" "$exe" golden --engine tu --iso "$GW_ISO_VANILLA" --scenario "$s" --frames "${ENGINE_FRAMES:-3601}" --out "$tmp/$s.csv" >"$tmp/$s.log" 2>&1 || true
    [ -f "$tmp/$s.csv" ] || : > "$tmp/$s.csv"
    n=$(python tools/rust-port/compare_golden.py "docs/rust-port/golden/$s.csv" "$tmp/$s.csv" --allow-short $seedflag --print-count 2>/dev/null | awk '/^COUNT/{print $2}'); n=${n:-0}
  else
  "$exe" golden --engine legacy --iso "$GW_ISO_VANILLA" --scenario "$s" $idle --out "$tmp/$s.csv" >"$tmp/$s.log" 2>&1
  n=$(python tools/rust-port/compare_golden.py "docs/rust-port/golden/$s.csv" "$tmp/$s.csv" --allow-short --print-count | awk '/^COUNT/{print $2}')
  fi
  want=$(awk -F'\t' -v s="$s" -v e="$engine" 'NR>1 && $2==s && $3==e {w=$4} END{print w+0}' "$progress")
  counts[$s]=$n
  if [ "${n:-0}" -lt "$want" ]; then echo "FAIL golden $s: $n frames match, recorded $want"; fail=1; else echo "ok   golden $s: $n frames match (recorded $want)"; fi
done
# rollback on the engine: a short synctest (consolidation.md step 17)
if [ "$engine" = tu ] && [ -z "$only" ]; then
  if "$exe" synctest --engine tu --depth 4 --frames "${SYNCTEST_FRAMES:-240}" --golden docs/rust-port/golden/pad_falcon_fox_fd.csv >"$tmp/synctest.log" 2>&1; then echo "ok   synctest: $(grep 'melee: engine synctest' "$tmp/synctest.log" | cut -c8-)"; else echo "FAIL synctest (see $tmp/synctest.log)"; fail=1; fi
fi
# engine audio: the game's own sound calls mix to exactly the C build's capture (melee/tests/audio_match.rs pins the same hash)
if [ "$engine" = tu ] && [ -z "$only" ]; then
  "$exe" golden --iso "$GW_ISO_VANILLA" --scenario pad_falcon_fox_fd --frames 1800 --audio --audio-preroll 29 --audio-wav "$tmp/a.wav" --out "$tmp/a.csv" >"$tmp/audio.log" 2>&1
  ah=$(python tools/rust-port/wavhash.py "$tmp/a.wav" 2>/dev/null)
  if [ "$ah" = "6106 0xf4956676e79a4037" ]; then echo "ok   audio: 6106 sub-frames, mix hash equal to the C capture"; else echo "FAIL audio: got '$ah', want '6106 0xf4956676e79a4037'"; fail=1; fi
fi
# baselines may only shrink
bmax=tools/rust-port/diff/baseline_max.tsv
while IFS=$'\t' read -r sub max; do
  [ "$sub" = subsystem ] && continue
  cur=$(grep -vc '^#\|^$' "tools/rust-port/diff/baseline/$sub.txt" 2>/dev/null || true)
  if [ "${cur:-0}" -gt "$max" ]; then echo "FAIL baseline $sub grew: $cur lines, max $max"; fail=1; fi
done < "$bmax"
echo "ok   baselines: $(cat tools/rust-port/diff/baseline/*.txt | grep -vc '^#\|^$') entries in total"
# migrated subsystems: zero fields outside the baseline at the sampled frames
if [ "$engine" = tu ]; then migrated=''; echo 'ok   diff: not run for the engine (melee diff --engine tu comes with the first migrated subsystem)'; else
migrated=$(grep -v '^#' docs/rust-port/migrated.txt | tr -s ' \n' ',' | sed 's/^,//;s/,$//')
if [ -n "$migrated" ]; then
  while IFS=$'\t' read -r s frames; do
    case "$s" in '#'*|'') continue ;; esac
    case " $scenarios " in *" $s "*) ;; *) continue ;; esac
    cap=${counts[$s]:-0}
    fl=""; for f in ${frames//,/ }; do [ "$f" -lt "$cap" ] && fl="$fl,$f"; done
    [ "$s" = cpu_fox_falco_dl ] && [ "$cap" -gt 0 ] && fl="$fl,$((cap-1))"
    fl=${fl#,}; [ -n "$fl" ] || continue
    idle=""; case $s in cpu_*) idle="--idle-pads" ;; esac
    "$exe" diff --engine legacy --iso "$GW_ISO_VANILLA" --scenario "$s" --frames "$fl" --subsystems "$migrated" --baseline-check tools/rust-port/diff/baseline --max 0 --print-hits >"$tmp/diff.$s" 2>&1 || { echo "FAIL diff $s"; tail -20 "$tmp/diff.$s"; fail=1; }
  done < tools/rust-port/diff/frames.tsv
  # a baseline line of a migrated subsystem that no sampled frame hit is STALE: remove it (baselines only shrink)
  for sub in ${migrated//,/ }; do
    while IFS= read -r key; do
      cat "$tmp"/diff.* 2>/dev/null | grep -qxF "HIT $key" || { echo "FAIL STALE $sub: $key"; fail=1; }
    done < <(grep -v '^#' "tools/rust-port/diff/baseline/$sub.txt" | grep -v '^$')
  done
else
  echo "ok   diff: no subsystem is marked migrated yet (docs/rust-port/migrated.txt)"
fi
fi
if [ -f tools/rust-port/lint_logic.py ]; then
  python tools/rust-port/lint_logic.py || { echo "FAIL lint"; fail=1; }
fi
if [ $update = 1 ] && [ $fail = 0 ]; then
  step=$(awk -F'\t' 'NR>1{s=$1} END{print s+0}' "$progress")
  for s in $scenarios; do printf '%s\t%s\t%s\t%s\n' "$step" "$s" "$engine" "${counts[$s]}" >> "$progress"; done
  echo "gate: progress rows appended for step $step (edit the step column if this is a new step)"
fi
rm -rf "$tmp"
[ $fail = 0 ] && echo "GATE OK" || echo "GATE FAILED"
exit $fail
