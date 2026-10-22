#!/bin/bash
# usage: dbg_gr.sh <stem> <op:case> [lines] : first differing trace / hash line of one case, C vs Rust
cd "$(dirname "$0")/../../../.."
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
T="${TMP:-/tmp}"
MTU_DBG="$2" python rust/melee_world/gen/cref/run_gr.py "$1" > /dev/null || exit 1
(cd rust && MTU_DBG="$2" cargo test -p melee_world --test "tu_$1" -- --nocapture 2>&1) | grep -E "^[THN] " > "$T/dbg_rust_$1.txt"
tr -d '\r' < rust/target/mtu/dbg_$1.txt > "$T/dbg_c_$1.txt"
echo "C lines: $(wc -l < "$T/dbg_c_$1.txt")  Rust lines: $(wc -l < "$T/dbg_rust_$1.txt")"
diff "$T/dbg_c_$1.txt" "$T/dbg_rust_$1.txt" | head -${3:-12}
