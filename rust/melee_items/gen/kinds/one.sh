#!/usr/bin/env bash
# One TU, all steps: gen ext, driver, C reference, Rust test.   bash rust/melee_items/gen/kinds/one.sh <stem>   (worktree root)
set -e
s=$1
python rust/melee_items/gen/kinds/gen_tu.py $s
python rust/melee_items/gen/kinds/mk_driver.py $s
python rust/melee_items/gen/kinds/run_tu.py $s | tail -1
rm -f rust/melee_items/gen/kinds/tu/${s}_ext.rs.txt
cd rust && CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 cargo test -p melee_items --test kinds_$s 2>&1 | grep -E "^(error|test |thread|  left|  right)|-->|panicked" | head -40
