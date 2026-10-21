#!/usr/bin/env python3
"""C oracle for tests/tu_ft_0d27.rs (game_tus shard 73-80). Reuses run.py's shipping-contraction
build without editing it: adds this ref's EXTRACTS entry, then runs run.main() on it.
  python rust/melee_fighter/gen/cref/tu_ft_0d27_run.py      (worktree root)
"""
import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parent))
import run  # noqa: E402

run.EXTRACTS["tu_ft_0d27"] = [
    *[("src/melee/ft/ft_0D27.c", "fn:" + n, None)
      for n in ("ftCo_800D27C4", "fn_800D2818", "fn_800D2A3C", "fn_800D2B30", "fn_800D2BF8")],
    ("src/melee/ft/ft_0CD1.c", "fn:ftCo_800CD278", None),
    ("src/melee/ft/ft_0CDD.c", "fn:ftCo_800CDE94", None),
]
sys.argv = [sys.argv[0], "tu_ft_0d27"]
run.main()
