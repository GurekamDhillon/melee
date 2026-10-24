#!/usr/bin/env python3
"""C references for the melee_chara functions the call trace listed as ported-unverified
(pre-melee_chara pass).

  python rust/melee_chara/gen/turef/tu_pre.py [name ...]     (worktree root)

Same mechanism as tu_shard89.py (group 3's rust/melee_fighter/gen/cref/run.py build: shipping
contraction + gwtool's lowering, game source cut verbatim through EXTRACTS), pointed at this
directory's <name>_ref.c drivers and at rust/melee_chara/tests/data.
"""
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
sys.path.insert(0, str(ROOT / "rust/melee_fighter/gen/cref"))
import run  # noqa: E402

MINE = {
    "tu_ftcaptainspeciallw": [("src/melee/ft/ftparts.c", "fn:ftParts_GetBoneIndex", None),
                              ("src/melee/ft/ftlib.c", "fn:ftLib_GetKind", None)],
    "tu_ftfoxspecialn": [],
}
run.EXTRACTS.update(MINE)
run.HERE = HERE
run.DATA = ROOT / "rust/melee_chara/tests/data"

if __name__ == "__main__":
    sys.argv = [sys.argv[0], *(sys.argv[1:] or sorted(MINE))]
    run.main()
