#!/bin/bash
# Envoy milestone 0, probe (b): gd_hash_word. A stateful sim mod (rust/mods/hashword_probe) under rollback:
#   1. depth-12 synctest, 3600 frames, mod loaded: 0 mismatches, and NHV1 equal to the C golden (the mod is game-inert);
#   2. loopback netplay pair (20 ms, 2 % loss), the mod on both peers: 0 desyncs, the host's replay equal (with the mod);
#   3. the same with the guest's mod word perturbed from game frame 600 (MELEE_NP_TEST_HASHWORD): both peers stop with
#      a DESYNC, and the first frame whose hashes differ is 600.
#   GW_ISO_VANILLA=... bash tools/rust-port/envoy/m0_hashword.sh
set -u
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"; cd "$repo"
[ -n "${GW_ISO_VANILLA:-}" ] || { echo "m0_hashword: GW_ISO_VANILLA not set"; exit 2; }
. "$here/exe_check.sh"; exe_pick m0_hashword; exe_banner m0_hashword   # EXE=path overrides the default build/cargo/release/melee
mods="$repo/build/mods-hashword"; bash rust/mods/hashword_probe/build.sh "$mods" >/dev/null || exit 1
qmods=$(printf %q "$mods") # play_pair reads NP_*_ENV as shell words: the path may hold spaces
export NP_SCRATCH="${NP_SCRATCH:-${TEMP:-/tmp}/np}" MELEE_VOLUME=3; mkdir -p "$NP_SCRATCH"
fail=0
line=$(MELEE_WASM_MODS="$mods" "$exe" synctest --engine tu --iso "$GW_ISO_VANILLA" --scenario cpu_fox_falco_dl --depth 12 --frames 3600 --timeout 1500 --golden docs/rust-port/golden/cpu_fox_falco_dl.csv 2>&1 | grep 'engine synctest')
case "$line" in *" 0 MISMATCHES"*"golden 3600/3600"*) echo "ok   1. ${line#melee: }";; *) echo "FAIL 1. ${line:-no result}"; fail=1;; esac
if NP_HOST_ENV="MELEE_WASM_MODS=$qmods" NP_GUEST_ENV="MELEE_WASM_MODS=$qmods" bash tools/rust-port/netplay/play_pair.sh m0_hw_same 20 2 --headless >"$NP_SCRATCH/m0_hw_same.out" 2>&1; then
  echo "ok   2. $(grep 'frames in both' "$NP_SCRATCH/m0_hw_same.out"); $(grep '^play: replay: [0-9]' "$NP_SCRATCH/m0_hw_same.out" | cut -c15-)"
else echo "FAIL 2. (see $NP_SCRATCH/m0_hw_same.out)"; fail=1; fi
NP_HOST_ENV="MELEE_WASM_MODS=$qmods" NP_GUEST_ENV="MELEE_WASM_MODS=$qmods MELEE_NP_TEST_HASHWORD=600" bash tools/rust-port/netplay/play_pair.sh m0_hw_skew 20 2 --headless >"$NP_SCRATCH/m0_hw_skew.out" 2>&1
first=$(python - "$NP_SCRATCH/m0_hw_skew-host.csv" "$NP_SCRATCH/m0_hw_skew-guest.csv" <<'PY'
import sys, csv
a = {int(r[0]): r[1] for r in list(csv.reader(open(sys.argv[1])))[1:]}
b = {int(r[0]): r[1] for r in list(csv.reader(open(sys.argv[2])))[1:]}
print(min((f for f in set(a) & set(b) if a[f] != b[f]), default=-1))
PY
)
if grep -q 'DESYNC' "$NP_SCRATCH/m0_hw_skew-host.log" && grep -q 'DESYNC' "$NP_SCRATCH/m0_hw_skew-guest.log" && [ "$first" = 600 ]; then
  echo "ok   3. both peers DESYNC, first differing frame $first"
else echo "FAIL 3. first differing frame ${first:-?} (see $NP_SCRATCH/m0_hw_skew.out)"; fail=1; fi
exit $fail
