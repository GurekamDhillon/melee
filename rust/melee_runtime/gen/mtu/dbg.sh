#!/bin/bash
# bash rust/melee_runtime/gen/mtu/dbg.sh <stem> <op:case> : diff the C and Rust trace/hash words of one case
cd "$(dirname "$0")/../../../.."
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
s=$1; c=$2
MTU_DBG=$c python rust/melee_runtime/gen/mtu/run_tu.py $s >/dev/null 2>&1
(cd rust && MTU_DBG=$c cargo test -p melee_runtime --test tu_$s 2>&1 | grep -E "^(T|H|B) " | tr -d '\r') > rust/target/mtu/rdbg_$s.txt
tr -d '\r' < rust/target/mtu/dbg_$s.txt > rust/target/mtu/cdbg_$s.txt
wc -l rust/target/mtu/rdbg_$s.txt rust/target/mtu/cdbg_$s.txt | head -2
diff rust/target/mtu/rdbg_$s.txt rust/target/mtu/cdbg_$s.txt | head -${3:-12}
n=$(diff rust/target/mtu/rdbg_$s.txt rust/target/mtu/cdbg_$s.txt | head -1 | sed 's/[^0-9].*//')
echo "first diff at line $n; preceding block:"; head -$n rust/target/mtu/cdbg_$s.txt | grep -n "^B " | tail -1
