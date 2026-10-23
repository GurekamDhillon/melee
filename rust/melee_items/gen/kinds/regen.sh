#!/usr/bin/env bash
# Regenerates the oracle of the item-kind TUs (shard 657-664): drivers, stub/state headers, C reference output.
# From the worktree root:  bash rust/melee_items/gen/kinds/regen.sh [stem ...]
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../../../.." && pwd)"
cd "$root"
stems=("$@")
if [ ${#stems[@]} -eq 0 ]; then
  stems=(itarwinglaser itbat itbombhei itbox itcapsule itcerebi itchicorita itclimbersblizzard)
fi
for s in "${stems[@]}"; do
  python rust/melee_items/gen/kinds/mk_driver.py "$s"
  python rust/melee_items/gen/kinds/run_tu.py "$s"
done
rm -f rust/melee_items/gen/kinds/tu/*_ext.rs.txt
