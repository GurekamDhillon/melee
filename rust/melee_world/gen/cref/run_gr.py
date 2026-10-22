#!/usr/bin/env python3
"""Driver for the gr/ TU ports of shard 577-584: runs gen_tu.py / run_tu.py with specs_gr.SPECS
(instead of specs.py) and the gr C/Rust driver generator gen_drv.py.

  python rust/melee_world/gen/cref/run_gr.py <stem> [...]      (from the worktree root)
"""
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import specs_gr  # noqa: E402

# gen_tu.build / run_tu.main import `specs`: hand them ours
sys.modules["specs"] = specs_gr
import gen_drv  # noqa: E402

if __name__ == "__main__":
    gen_drv.main(sys.argv[1:])
