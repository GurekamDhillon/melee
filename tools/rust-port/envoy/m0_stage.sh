#!/bin/bash
export MELEE_WINDOW_HIDDEN=${MELEE_WINDOW_HIDDEN:-1}  # agents' game windows are invisible (MELEE_WINDOW_HIDDEN=0 shows them)
# Envoy milestone 0, probe (c): stage switching under rollback with items off (pad_falcon_fox_fd: items=off), using the
# stage_switch mod's C twin (rust/mods/stage_switch/c: clang, no wasm32 Rust target needed; same id and rules):
#   1. two golden runs with the mod: byte-identical per-frame hash CSVs, 5 switches;
#   2. synctest depth 12 over all 3601 frames with MELEE_SNAP_CHECK=1: 0 mismatches; every switch is resimulated
#      12 times (5 + 60 = 65 switch executions logged).
# The Rust switch reads the destination GrXx.dat at the switch, inside the sim (stage_hop.rs); nothing is preloaded.
#   GW_ISO_VANILLA=... bash tools/rust-port/envoy/m0_stage.sh
set -u
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"; cd "$repo"
[ -n "${GW_ISO_VANILLA:-}" ] || { echo "m0_stage: GW_ISO_VANILLA not set"; exit 2; }
. "$here/exe_check.sh"; exe_pick m0_stage; exe_banner m0_stage   # EXE=path overrides the default build/cargo/release/melee
d="$(mktemp -d)"; bash rust/mods/stage_switch/c/build.sh "$d" >/dev/null || exit 1
export MELEE_WASM_MODS="$d" MELEE_ENGINE_PRE_DRAWS=0 MELEE_VOLUME=3
fail=0
for r in 1 2; do MELEE_WINDOW_TITLE="m0 stage golden $r" "$exe" golden --engine tu --iso "$GW_ISO_VANILLA" --scenario pad_falcon_fox_fd --frames 3601 --out "$d/r$r.csv" >"$d/r$r.log" 2>&1; done
n=$(grep -c "^stage switch:" "$d/r1.log")
if cmp -s "$d/r1.csv" "$d/r2.csv" && [ "$n" -eq 5 ] && [ "$(wc -l <"$d/r1.csv")" -eq 3602 ]; then echo "ok   1. goldens with the mod: 2 runs identical, $n switches"; else echo "FAIL 1. runs differ or $n switches"; fail=1; fi
MELEE_SNAP_CHECK=1 MELEE_WINDOW_TITLE="m0 stage synctest" "$exe" synctest --engine tu --iso "$GW_ISO_VANILLA" --depth 12 --scenario pad_falcon_fox_fd --frames 3601 --timeout 2400 >"$d/sync.log" 2>&1
line=$(grep 'engine synctest' "$d/sync.log"); k=$(grep -c "^stage switch:" "$d/sync.log")
if [ -n "$line" ] && grep -q " 0 MISMATCHES" "$d/sync.log" && ! grep -q "MELEE_SNAP_CHECK:" "$d/sync.log" && [ "$k" -eq 65 ]; then
  echo "ok   2. ${line#melee: }; $k switch executions"
else echo "FAIL 2. ${line:-no result}; $k switch executions (logs: $d)"; fail=1; fi
exit $fail
