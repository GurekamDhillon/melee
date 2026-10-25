#!/bin/bash
# bash rust/melee_runtime/gen/mtu/go.sh <stem> : mk_lb + C run + cargo test of one TU oracle
cd "$(dirname "$0")/../../../.."
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
s=$1
python rust/melee_runtime/gen/mtu/mk_lb.py $s 2>&1 | tail -3 || exit 1
python rust/melee_runtime/gen/mtu/run_tu.py $s 2>&1 | grep -v "^  WARN\|header inl" | grep -E "error|mtu:|Error|exited|timed" -A3 | head -${2:-30}
grep -q "name = \"tu_$s\"" rust/melee_runtime/Cargo.toml || printf '\n[[test]]\nname = "tu_%s"\nharness = false\n' $s >> rust/melee_runtime/Cargo.toml
(cd rust && cargo test -p melee_runtime --test tu_$s 2>&1 | grep -E "^error|panicked|left:|right:|tu_$s: ok" -A${3:-7} | head -${2:-40})
