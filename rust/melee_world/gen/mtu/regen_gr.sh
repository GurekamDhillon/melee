#!/usr/bin/env bash
# Regenerates every gr-shard (553-560) oracle: stub/state headers, drivers, C reference output.
# From the worktree root:  bash rust/melee_world/gen/mtu/regen_gr.sh
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../../../.." && pwd)"
cd "$root"
for s in grdisplay grdynamicattr grfigure1 grfigure2 grfigure3 grfigureget grflatzone grfourside; do
  case "$s" in
    grdisplay|grdynamicattr) ;;                      # hand-written drivers (gen/mtu/tu/<stem>.c)
    *) python rust/melee_world/gen/mtu/mk_stage_driver.py "$s" ;;
  esac
  python rust/melee_world/gen/mtu/run_tu.py "$s"
done
rm -f rust/melee_world/gen/mtu/tu/*_ext.rs.txt
