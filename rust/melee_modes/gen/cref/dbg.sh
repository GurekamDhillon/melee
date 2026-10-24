#!/bin/bash
# usage: dbg.sh <stem> <test> <op:case> : diff the C and Rust trace/state words of one case
cd "$(dirname "$0")/../../../.."
MTU_DBG=$3 python rust/melee_modes/gen/cref/run_tu.py $1 > /dev/null 2>&1
cd rust
MTU_DBG=$3 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 cargo test -p melee_modes --test $2 -- --nocapture 2>&1 | tr -d '\r' | grep -E "^(T|H|D|S) " > "$TMP/r_t.txt"
tr -d '\r' < target/mtu/dbg_$1.txt > "$TMP/c_t.txt"
diff "$TMP/r_t.txt" "$TMP/c_t.txt" | head -${4:-20}
