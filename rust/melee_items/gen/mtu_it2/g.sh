#!/bin/bash
# g.sh <stem>: regen driver, C oracle, rust test
cd "$(dirname "$0")/../../../.."
python rust/melee_items/gen/mtu_it2/mk_it_driver.py $1 && python rust/melee_items/gen/mtu_it2/run_it.py $1 2>&1 | tail -3 && rust/melee_items/gen/mtu_it2/t.sh $1 ${2:-8} ${3:-60}
