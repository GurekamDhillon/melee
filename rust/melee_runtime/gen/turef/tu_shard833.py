#!/usr/bin/env python3
"""C references (state oracles) for the lb TUs of game_tus.txt lines 833-840.

  python rust/melee_runtime/gen/turef/tu_shard833.py [--gen-only] [tu_lbfile ...]     (worktree root)

lb_specs.py lists the callees each driver stubs; lbgen.py writes the C stubs (gen/turef/tu_<stem>_stubs.inc),
the `ext` hook block in src/tu/<stem>.rs and the Rust trace hooks (tests/lbtest/<stem>_stubs.rs). The driver
tu_<stem>_ref.c (the game TU #included verbatim) is built like melee_fighter/gen/cref/run.py builds its
references (shipping contraction + gwtool lowering) and run; stdout goes to tests/data/tu_<stem>.txt.
"""
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
sys.path.insert(0, str(HERE))
sys.path.insert(0, str(ROOT / "rust/melee_fighter/gen/cref"))
import run  # noqa: E402
import lbgen  # noqa: E402
from lb_specs import SPECS, DRIVERS, GROUPS  # noqa: E402

run.HERE = HERE
run.DATA = ROOT / "rust/melee_runtime/tests/data"
run.FLAGS += ["-I", str(HERE)]
run.LINK_EXTRA["tu_lbheap"] = ["-Wl,/base:0x10000000", "-Wl,/dynamicbase:no"]
run.EXTRACTS.update({f"tu_{k}": [] for k in DRIVERS})


def main():
    args = sys.argv[1:]
    gen_only = "--gen-only" in args
    names = [a[3:] if a.startswith("tu_") else a for a in args if not a.startswith("--")] or sorted(DRIVERS)
    run.BUILD.mkdir(parents=True, exist_ok=True)
    for stem in names:
        if any(SPECS.get(s) for s in GROUPS.get(stem, [stem])):
            lbgen.build(SPECS, stem, GROUPS.get(stem, [stem]), ROOT)
    if gen_only:
        return
    sys.argv = [sys.argv[0], *[f"tu_{n}" for n in names]]
    run.main()


if __name__ == "__main__":
    main()
