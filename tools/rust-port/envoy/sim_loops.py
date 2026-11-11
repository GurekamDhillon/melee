#!/usr/bin/env python3
"""Envoy endless loops: the pacing model (envoy-content-pass.md 5.1, with the drive output measured by sim_v3.py).

The content pass worked this by hand; here B (the damage a build lands per 1% of the player's own moves) comes from
sim_v3's Fight on the real pool, at each loop's levels, against that loop's foe toughness, with the fire cap
measured either before toughness (as built until v3g) or on what lands (the retune). The rest is the hand model:

  damage taken in a loop = pilot x foe load x 40% x foe damage / B_landed
  B_landed = (own moves / T + drive damage landed) / own moves x keystones x sets
  a stock is 120% at loop 1 and 3% shorter per loop; the pool starts at 4, +1 per loop cleared, +1 bought, to 8.

    python tools/rust-port/envoy/sim.py --loops [--seeds 12] [--old]
"""
import random
import statistics

import sim_v3

LEVEL_TOP = 9


def level_scale(l):
    l = max(1, min(LEVEL_TOP, l))
    if l <= 3:
        return [1000, 1600, 2400][l - 1]
    k = l - 3
    return 2400 + 800 * k + 100 * k * (k + 1)


# foe load (units of one plain foe stock) by loop: the content pass's table, straight lines between its columns
LOAD = {1: 12.2, 2: 15.9, 3: 23.4, 5: 30.5, 8: 40.5}


def load(lp):
    ks = sorted(LOAD)
    if lp in LOAD:
        return LOAD[lp]
    if lp > ks[-1]:
        return LOAD[ks[-1]] + (lp - ks[-1]) * (LOAD[8] - LOAD[5]) / 3
    lo = max(k for k in ks if k < lp)
    hi = min(k for k in ks if k > lp)
    return LOAD[lo] + (LOAD[hi] - LOAD[lo]) * (lp - lo) / (hi - lo)


def foe_atk(e, old):
    """Foe damage dealt, x1 (e = loops cleared)."""
    if old:
        return 1 + 0.20 * e
    if e <= 0:
        return 1.0
    if e == 1:
        return 1.05
    if e <= 4:
        return 1.05 + 0.10 * (e - 1)
    return 1.35 + 0.15 * (e - 4)


def foe_tough(e, old):
    if old:
        return 1 + 0.15 * e
    if e <= 4:
        return 1 + 0.10 * e
    return 1.40 + 0.15 * (e - 4)


# the builds: drives in slot order, and their levels by loop (content pass 5.4); slots 3 / 4 / 5 open in loop 1
BUILDS = {
    "strong": dict(pilot=1.25, drives=["nova", "ember", "bolt", "shatter", "frost"],
                   levels={1: 2, 2: 3, 3: 3, 4: 3, 5: 4, 6: 4, 7: 4, 8: 4}),
    "great": dict(pilot=1.0, drives=["echo", "ember", "bolt", "nova", "wildfire"],
                  levels={1: 2, 2: 3, 3: 4, 4: 4, 5: 5, 6: 5, 7: 6, 8: 6}),
}


def keys_mult(lp):
    return 1.2 if lp == 1 else 1.6


def set_mult(lp):
    return 1.15 if lp >= 3 else 1.0


def b_landed(name, lp, tough, old, seeds):
    b = BUILDS[name]
    by = {r["key"]: r for r in sim_v3.parse_pool(sim_v3_pool()) if r["type"] == "drive"}
    lv = b["levels"].get(lp, b["levels"][8] + (lp - 8) // 2)
    n = 3 if lp == 1 else 5
    build = [(by[k], 3) for k in b["drives"][:n] if k in by]
    # the model's level table to 9, and the fire cap's place
    sim_v3.LEVEL = [level_scale(l) for l in range(1, LEVEL_TOP + 1)]
    build = [(r, min(lv, LEVEL_TOP)) for r, _ in build]
    t_m = int(tough * 1000)
    raw = ORIG_FIRE_DMG

    def fire_dmg(number, level, st, launch):
        n = number * sim_v3.LEVEL[level - 1] // 1000 * sim_v3.STATE_SCALE[st] // 1000
        cap = sim_v3.LAUNCH_CAP if launch else sim_v3.FIRE_CAP
        if old:
            return min(n, cap) * 1000 // t_m  # capped first, then toughness
        return min(n * 1000 // t_m, cap)  # the cap on what lands

    sim_v3.fire_dmg = fire_dmg
    try:
        outs = [100.0 * f.extra / max(1, f.move_dmg) for f in
                (sim_v3.Fight(build, sim_v3.POLICIES["brawler"], random.Random(s * 7919 + 1)).run() for s in range(seeds))]
    finally:
        sim_v3.fire_dmg = raw
    extra = statistics.mean(outs) / 100.0
    return (1.0 / tough + extra) * keys_mult(lp) * set_mult(lp)


_POOL = None


def sim_v3_pool():
    return _POOL


ORIG_FIRE_DMG = sim_v3.fire_dmg


def run(pool_path, seeds=12, old=False, loops=14):
    global _POOL
    _POOL = pool_path
    out = {}
    for name, b in BUILDS.items():
        pool, rows, end = 4.0, [], None
        for lp in range(1, loops + 1):
            e = lp - 1
            t = foe_tough(e, old)
            bl = b_landed(name, lp, t, old, seeds)
            taken = b["pilot"] * load(lp) * 40.0 * foe_atk(e, old) / bl
            stock = 120.0 * 0.97 ** e
            lost = taken / stock
            rows.append(dict(loop=lp, B=round(bl, 2), lost=round(lost, 2), pool=round(pool, 2)))
            pool -= lost
            if pool <= 0 and end is None:
                end = lp
                break
            pool = min(8.0, pool + 2.0)  # +1 for the clear, +1 bought
        out[name] = dict(rows=rows, ends=end)
    return out


def main(args, pool_path):
    for old in ([True, False] if getattr(args, "old", False) else [False]):
        res = run(pool_path, seeds=max(4, args.seeds // 10), old=old)
        print("Envoy loops model (%s): B landed, stocks lost, pool at the loop's start" % (
            "as built: +20% / +15% a loop, fire cap before toughness" if old else
            "retune: foe damage 5/15/25/35 then +15, toughness +10 then +15, fire cap on what lands"))
        for name, r in res.items():
            cells = "  ".join(f"L{x['loop']} B{x['B']:.1f} -{x['lost']:.1f} ({x['pool']:.1f})" for x in r["rows"])
            print(f"  {name:6s} ends in loop {r['ends'] or '>14'}:  {cells}")
    return 0
