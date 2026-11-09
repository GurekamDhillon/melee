#!/bin/bash
# Envoy milestone 0, probe (a) (docs/rust-port/notes/envoy-plan.md section 5): CPUs under rollback.
#   GW_ISO_VANILLA=... bash tools/rust-port/envoy/m0_probe.sh [EXE]
# Depth-12 synctest, 3600 frames each: the all-CPU golden scenario against its C CSV, then two human+CPU scenes
# (m0_scenarios.tsv: ports 1-2 driven by synctest's fixed pads, a level-9 CPU on port 3, and on port 4 in the second).
# The `pad_` prefix makes synctest feed its scripted pads; `idle_` pad files mean no pad script of their own.
# Exit 0 = every run 0 mismatches (and the golden run equal frame for frame).
set -u
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; repo="$here/../../.."
exe="${1:-$repo/build/cargo/release/melee}"; [ -e "$exe" ] || exe="$exe.exe"
[ -n "${GW_ISO_VANILLA:-}" ] || { echo "m0_probe: GW_ISO_VANILLA not set"; exit 2; }
cd "$repo"
fail=0
run() { # name, extra args...
  local name=$1; shift
  line=$("$exe" synctest --engine tu --iso "$GW_ISO_VANILLA" --scenario "$name" --depth 12 --frames 3600 --timeout 1500 "$@" 2>&1 | grep 'engine synctest')
  case "$line" in
    *" 0 MISMATCHES"*) case "$*" in *--golden*) case "$line" in *"golden 3600/3600"*) echo "ok   $name: ${line#melee: }";; *) echo "FAIL $name: ${line#melee: }"; fail=1;; esac;; *) echo "ok   $name: ${line#melee: }";; esac;;
    *) echo "FAIL $name: ${line:-no result}"; fail=1;;
  esac
}
run cpu_fox_falco_dl --golden docs/rust-port/golden/cpu_fox_falco_dl.csv
export MELEE_SCENARIO_TSV="$here/m0_scenarios.tsv"
run pad_envoy_m0a_fd
run pad_envoy_m0a_bf
exit $fail
