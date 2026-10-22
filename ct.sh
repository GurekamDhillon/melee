#!/bin/bash
# usage: ct.sh <test-name> : run one melee_modes tu oracle test with the low-disk env
cd "$(dirname "$0")/rust"
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
cargo test -p melee_modes --test "$1" > "$TMP/ct_out.txt" 2>&1
grep -E "^error|panicked|assertion|left:|right:|test result|^test " "$TMP/ct_out.txt" | head -${2:-40}
grep -A6 "^error" "$TMP/ct_out.txt" | head -${3:-60}
