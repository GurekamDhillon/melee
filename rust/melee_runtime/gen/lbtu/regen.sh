#!/usr/bin/env bash
# Regenerates the oracle of the lb TUs (shard 841-848): drivers, stub/state headers, C reference output.
# From the worktree root:  bash rust/melee_runtime/gen/lbtu/regen.sh [stem ...]
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../../../.." && pwd)"
cd "$root"
stems=("$@")
if [ ${#stems[@]} -eq 0 ]; then
  stems=(lbtime lbtrigf lb_00CE lbvector lbsnap lb_00B0 lb_00F9 lbspdisplay)
fi
for s in "${stems[@]}"; do
  python rust/melee_runtime/gen/lbtu/mk_driver.py "$s"
  python rust/melee_runtime/gen/lbtu/run_tu.py "$s"
done
rm -f rust/melee_runtime/gen/lbtu/tu/*_ext.rs.txt
