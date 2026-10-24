#!/bin/bash
# t.sh <stem> : run cargo test for stem showing only errors / result
cd "$(dirname "$0")/../../.."
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
cargo test -p melee_items --test tu_$1 2>&1 | grep -E "^error|test result|panicked|^test " -A${2:-8} | head -${3:-70}
