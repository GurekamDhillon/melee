#!/bin/bash
# go.sh <stem>: regenerate stubs+driver, run the C oracle, run the Rust test
cd "$(dirname "$0")/../../../.."
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
python rust/melee_items/gen/mtu_it2/mk_it_driver.py $1 && python rust/melee_items/gen/mtu_it2/run_it.py $1 2>&1 | tail -4 && cd rust && cargo test -p melee_items --test tu_$1 2>&1 | grep -E "^error|test result|panicked" -A7 | head -${2:-60}
