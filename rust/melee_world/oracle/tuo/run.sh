#!/bin/bash
# run.sh <stem>: regenerate the C side, refresh the port's ext block, run the Rust test.
# (from anywhere; Git Bash)
set -e
HERE="$(cd "$(dirname "$0")" && pwd)"
WT="$(cd "$HERE/../../../.." && pwd)"
bash "$HERE/build.sh" "$1" | tail -4
python "$HERE/inject_ext.py" "$1" >/dev/null
cd "$WT/rust"
OUT=$(CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 cargo test -p melee_world --test "tuo_$1" 2>&1 || true)
echo "$OUT" | grep -E "^error" -A7 | head -${LINES_MAX:-70}
echo "$OUT" | grep -E "^test |panicked|^ *(left|right):|mismatch|test result" | head -20
