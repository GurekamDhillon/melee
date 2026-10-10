"""Assert heap 3 headroom after a real six-full-Geno-fighter load.

Run geno_six_full_heap3.lua with the stress6 fixtures, then pass the run's
melee-pc.log here. --require-headless also requires the Dawn null backend.
This complements the registry-only native geno_define_six_full test: missing
load evidence, the wrong external stage, incomplete definitions, or missing
players can never pass. Supply the ACE disc as shown in report-heap3.md.
No disc-derived assets are read or written by this checker.
"""
import argparse
from pathlib import Path
import re

MIN_FREE = 4 * 1024 * 1024


def check(log, require_headless=False):
    def require(condition, message):
        if not condition:
            raise ValueError(message)

    require(not re.search(r"FATAL|PANIC|ALLOC_FAIL|TEST geno_six_full_heap3 players FAIL", log),
            "run contains a crash, allocation failure, or player-test failure")
    if require_headless:
        require("workbench: headless (no GPU device: Dawn null backend; window offscreen)" in log,
                "GPU-free workbench startup was not observed")
    require("TEST geno_six_full_heap3 players PASS" in log,
            "six-fighter load/play did not finish")
    stages = re.findall(r"HEAP3_SIX_FULL stage_ext=(\d+) stage_internal=(\d+)", log)
    require(len(stages) == 1 and int(stages[0][0]) == 121,
            "active external stage 121 was not observed")
    players = re.findall(r"HEAP3_SIX_FULL player=(\d+) kind=(\d+) name=Geno Falcon (\d+)", log)
    require(len(players) == 6, "expected exactly six loaded player records")
    require({int(p) for p, _, _ in players} == set(range(1, 7)),
            "ports 1 through 6 were not all loaded")
    require(all(p == name for p, _, name in players), "wrong Geno fighter on a port")
    kinds = {int(kind) for _, kind, _ in players}
    require(len(kinds) == 6, "expected six distinct Geno definitions")
    budgets = {
        int(kind): (int(overlays), int(pool))
        for kind, overlays, pool in re.findall(
            r"geno: define budgets: kind (\d+) overlays (\d+)/\d+, pool (\d+) words", log)
    }
    # geno_define_six_full uses 300 overlays of 14 words for each of six
    # definitions. The stress6 Captain fixtures have 313 overlays each.
    require(all(kind in budgets and budgets[kind][0] >= 300 and budgets[kind][1] >= 4200
                for kind in kinds), "a loaded definition is missing its full overlay/pool budget")
    heaps = re.findall(
        r"six-slot after-load heap=3 free=(\d+) capacity=(\d+) fighter_files=(\d+)", log)
    require(bool(heaps), "missing heap 3 after-load measurement (planned capacity is insufficient)")
    minimum = None
    for free, capacity, files in heaps:
        free, capacity, files = int(free), int(capacity), int(files)
        require(files > 0 and 0 <= free <= capacity, "invalid after-load measurement")
        require(free >= MIN_FREE,
                f"heap 3 has {free:,} bytes free after load; requires at least {MIN_FREE:,}")
        minimum = free if minimum is None else min(minimum, free)
    return minimum


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("log", type=Path)
    parser.add_argument("--require-headless", action="store_true")
    args = parser.parse_args()
    try:
        free = check(args.log.read_text(encoding="utf-8", errors="replace"), args.require_headless)
    except (OSError, ValueError) as error:
        print(f"FAIL geno_six_full_heap3: {error}")
        return 1
    print(f"PASS geno_six_full_heap3: six full definitions loaded, heap 3 free={free:,} >= {MIN_FREE:,}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
