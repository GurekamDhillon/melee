#!/bin/bash
# Run coverage scenarios against the Rust engine and compare with the C goldens (docs/rust-port/golden/coverage/<name>.csv).
#   bash tools/rust-port/golden/coverage/run_rust.sh [-j N] [--out RESULTS.tsv] [NAME...|--all]
# Output rows: name <TAB> status (PASS|FAIL|NOT_WIRED|NO_GOLDEN) <TAB> matched frames <TAB> golden rows <TAB> first divergence / reason.
# COV_MODS_DIR=<dir> mounts <dir>/<name> as the scenario's mods folder.
# Headless, unfocused, no OS input. Needs a built build/cargo/${PROFILE:-release}/melee.exe (PROFILE=fast: cargo_fast.sh build -p melee --profile fast) and GW_ISO_VANILLA (workspace .env, never printed).
set -u
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; repo="$(cd "$here/../../../.." && pwd)"; cd "$repo"
ws="${GW_WORKSPACE:-E:/Projects/Melee Workspace}"
[ -z "${GW_ISO_VANILLA:-}" ] && [ -f "$ws/.env" ] && { set -a; . "$ws/.env"; set +a; }
export MELEE_WINDOW_UNFOCUSED=1
exe_default="build/cargo/${PROFILE:-release}/melee.exe"; MELEE_EXE="${MELEE_EXE:-$exe_default}"
jobs=3; out=""; names=()
while [ $# -gt 0 ]; do case "$1" in -j) jobs="$2"; shift ;; --out) out="$2"; shift ;; --all) mapfile -t names < <(grep -v '^#' docs/rust-port/golden/coverage/scenarios.tsv | cut -f1) ;; *) names+=("$1") ;; esac; shift; done
tmp="${TEMP:-/tmp}/covrun.$$"; mkdir -p "$tmp"
one() {
  s="$1"; g="docs/rust-port/golden/coverage/$s.csv"
  export MELEE_WINDOW_TITLE="${COV_TITLE:-coverage}: $s"   # every visible window names its scenario
  [ -f "$g" ] || { printf '%s\tNO_GOLDEN\t0\t0\t-\n' "$s"; return; }
  rows=$(( $(wc -l < "$g") - 1 ))
  # the scenario's env (scenarios.json "env", e.g. MELEE_MODS_DIR for the Geno mods), as record_coverage.sh passes to the C
  envs=$(python tools/rust-port/golden/coverage/scn.py tsv --rust "$s" 2>/dev/null | cut -f6)
  IFS=',' read -ra kv <<< "$envs"; for e in "${kv[@]}"; do [ -n "$e" ] && export "$e"; done
  # COV_MODS_DIR=<dir>: the scenario's mods from <dir>/<name> instead (e.g. its packages converted to geno 12 by
  # tools/rust-port/geno/lua_to_wasm_package.py), compared against the same C golden
  [ -n "${COV_MODS_DIR:-}" ] && export MELEE_MODS_DIR="$COV_MODS_DIR/$s"
  iso="$GW_ISO_VANILLA"; if [ -n "${COV_ISO:-}" ]; then v="GW_ISO_$COV_ISO"; iso="${!v:-}"; [ -n "$iso" ] || { printf '%s	FAIL	0	%s	%s not set
' "$s" "$rows" "$v"; return; }; fi
  # a VS golden whose match ends (stock matches) goes on into the results screen, which the C hashes as a second live
  # "match" (frame restarts at 0) while the engine's VS boot stops at the match end: compare the match rows only
  kind=$(awk -F'	' -v s="$s" '$1==s{print $2}' docs/rust-port/golden/coverage/scenarios.tsv)
  if [ "$kind" != "1p" ]; then
    m=$(awk -F, 'NR>2 && $1+0==0 {print NR-2; exit}' "$g")
    [ -n "$m" ] && rows=$m
  fi
  # the C boot makes HSD_Rand draws the engine's boot flow does not yet (BGM pick, stage init): MELEE_ENGINE_PRE_DRAWS per scenario,
  # found by the first 2 frames (0..3); the chosen N is part of the detail so the gap stays visible (tools/rust-port/engine/pre_draws.tsv)
  pre=0
  for k in 0 1 2 3; do
    MELEE_ENGINE_PRE_DRAWS=$k "$MELEE_EXE" golden --engine tu --iso "$iso" --scenario "$s" --frames 2 --out "$tmp/$s.cal.csv" >"$tmp/$s.log" 2>&1
    [ -s "$tmp/$s.cal.csv" ] || break
    c=$(python tools/rust-port/compare_golden.py "$g" "$tmp/$s.cal.csv" --allow-short --print-count 2>&1 | awk '/^COUNT/{print $2}')
    [ "${c:-0}" -ge 2 ] && { pre=$k; break; }
  done
  MELEE_ENGINE_PRE_DRAWS=$pre "$MELEE_EXE" golden --engine tu --iso "$iso" --scenario "$s" --frames "$rows" --out "$tmp/$s.csv" >"$tmp/$s.log" 2>&1
  if grep -q "not wired" "$tmp/$s.log" && [ ! -s "$tmp/$s.csv" ]; then printf '%s\tNOT_WIRED\t0\t%s\t%s\n' "$s" "$rows" "$(grep -m1 'not wired' "$tmp/$s.log" | sed 's/.*not wired/not wired/' | cut -c1-120)"; return; fi
  [ -f "$tmp/$s.csv" ] || : > "$tmp/$s.csv"
  res=$(python tools/rust-port/compare_golden.py "$g" "$tmp/$s.csv" --allow-short --print-count 2>&1)
  n=$(echo "$res" | awk '/^COUNT/{print $2}'); n=${n:-0}
  crows=$(( $(wc -l < "$tmp/$s.csv") - 1 ))
  if [ "$n" -lt "$rows" ] && [ "$n" -ge "$crows" ]; then
    why=$(grep -h -A1 "panicked at" "$tmp/$s.log" | grep -v "^--" | head -2 | tr '
' ' ' | sed 's/thread .main. ([0-9]*) panicked at //' | cut -c1-200)
    printf '%s	FAIL	%s	%s	pre=%s; engine stopped after %s rows, no mismatch before: %s
' "$s" "$n" "$rows" "$pre" "$crows" "${why:-no panic message}"; return
  fi
  if [ "$n" -ge "$rows" ]; then st=PASS; det="-"; else st=FAIL; det=$(echo "$res" | grep -v '^COUNT' | head -2 | tr '\n' ' ' | cut -c1-200); fi
  printf '%s\t%s\t%s\t%s\t%s\n' "$s" "$st" "$n" "$rows" "$det"
}
export MELEE_EXE; export -f one; export tmp
printf '%s\n' "${names[@]}" | xargs -P "$jobs" -I{} bash -c 'one {}' | tee ${out:+"$out"}
