#!/usr/bin/env python3
"""C reference for the ftCommon Capture* TUs of game_tus.txt lines 121-128
(tests/tu_ftCo_CapturePulled.rs, tests/tu_ftCo_CaptureKirby.rs).

  python rust/melee_fighter/gen/cref/tu_shard121.py      (worktree root)

Reuses run.py's build (shipping contraction + gwtool lowering); only adds EXTRACTS entries.
Output: rust/melee_fighter/tests/data/tu_shard121.txt.
"""
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import run  # noqa: E402

CO = "src/melee/ft/kinds/ftCommon/"
MINE = {
    "tu_shard121": [*[(CO + "ftCo_CapturePulled.c", "fn:" + n, None) for n in
                      ("ftCo_800DA824", "fn_800DAC78", "fn_800DAA40", "fn_800DAD18",
                       "ftCo_CapturePulledHi_Phys", "ftCo_800DB464")],
                    (CO + "ftCo_CaptureKirby.c", "fn:ftCo_800BD19C", None),
                    (CO + "ftCo_CaptureKirby.c", "static inline void inlineB0(", None)],
}
run.EXTRACTS.update(MINE)

if __name__ == "__main__":
    sys.argv = [sys.argv[0], *(sys.argv[1:] or sorted(MINE))]
    run.main()
