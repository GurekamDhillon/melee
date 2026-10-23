#!/usr/bin/env bash
# First differing trace / state words of one case:  bash rust/melee_world/gen/mtu/dbg_if.sh <stem> <op>:<case>
cd "$(dirname "$0")/../../../.." || exit 1
stem=$1; oc=$2
MTU_DBG=$oc python rust/melee_world/gen/mtu/run_if.py "$stem" >/dev/null || exit 1
(cd rust && MTU_DBG=$oc CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 cargo test -p melee_world --test "tu_$stem" -- --nocapture 2>/dev/null) \
  | grep -E "^(T|H|B|PAD) " > rust/target/mtu/dbg_rs_$stem.txt
grep -E "^(T|H|B|PAD) " rust/target/mtu/dbg_$stem.txt > rust/target/mtu/dbg_c_$stem.txt
diff rust/target/mtu/dbg_c_$stem.txt rust/target/mtu/dbg_rs_$stem.txt | head -${3:-12}
# restore the normal expected file
python rust/melee_world/gen/mtu/run_if.py "$stem" >/dev/null
