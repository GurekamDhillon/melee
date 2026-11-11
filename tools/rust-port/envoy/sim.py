#!/usr/bin/env python3
"""Envoy drive balancing harness (envoy-core-v2.md 3.3, 6 V2: "sim.py, seeds x policies").

A model of the sim's drive rules, not the engine: it reads rust/mods/envoy_data/pool.toml and plays fights as
streams of events (hits landed, aerial hits, hits on a foe at 100%+, hits taken, landings, jumps, tags, KOs, stage
shifts) drawn from a per-policy pace, frame by frame at 60 Hz, with the sim's rules on top: level scale, the 20-frame
gap per set, Relay's volley, Echo, Echo Oath, Overclock, Twin Soul, cooldowns, Surge, Wildfire on a Burning KO,
Burning / Chilled / Marked / Haste windows. It reports, per policy and over seeds:

  * the fire rate: seconds between fires with three level-1 drives (the early game) and with a full build
    (4 drives in play at levels 2-3, 8 with Twin Soul, and three keystones); the bands are the doc's "about one fire every 2-3 s early, about
    one a second with a full build" (Q5: start loud);
  * the drives' share of the damage dealt, and the run's win rate against a fight ladder (a CPU-proxy guard, not
    the judge: the owner's own play sets the dial);
  * each drive's level-1 fire period alone (the lint's band, 2-16 s).

    python tools/rust-port/envoy/sim.py [--seeds 200] [--json]     exit 1 when a band is missed
"""
import argparse
import json
import os
import random
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
POOL = os.path.join(HERE, "..", "..", "..", "rust", "mods", "envoy_data", "pool.toml")

FPS = 60
GAP = 20          # one fire per set every 20 frames
RELAY_GAP = 6
ECHO_DELAY = 12
LEVEL = [1.0, 1.6, 2.4]
BURN_TICK_PCT, BURN_TICK, BURN_FRAMES = 2.0, 60, 240
MARK_MORE, HASTE = 0.30, 1.30

# a CPU-proxy pace per policy (events per second) and the player's own damage per hit (percent)
POLICIES = {
    "brawler": dict(hit=1.1, aerial=0.25, taken=0.35, landing=0.45, jump=0.6, tag=1 / 14, hit_pct=7.0),
    "aerial": dict(hit=0.9, aerial=0.6, taken=0.3, landing=0.7, jump=1.0, tag=1 / 18, hit_pct=8.0),
    "tagger": dict(hit=0.8, aerial=0.25, taken=0.3, landing=0.45, jump=0.6, tag=1 / 6.5, hit_pct=7.0),
    "turtle": dict(hit=0.55, aerial=0.15, taken=0.2, landing=0.35, jump=0.45, tag=1 / 20, hit_pct=9.0),
}
KO_PCT = 130.0          # a foe is KO'd at about this percent (proxy)
SHIFT_EVERY = 30.0      # stage shifts (fights 2+)
# full_s: "about one a second" with a full build, the low end open to 0.6 s while the fire rate starts loud (Q5)
BANDS = {"early_s": (2.0, 3.6), "full_s": (0.6, 1.4), "win": (0.45, 0.9), "drive_share_full": (0.25, 0.65)}
# the Short run: (foes' combined stocks, foe damage per second on the player, an elite or boss carries drives)
LADDER = [(1, 3.5, 0), (2, 5.0, 1), (2, 4.5, 0), (2, 6.0, 2), (3, 7.0, 2)]


def parse_pool(path):
    recs, cur = [], None
    for line in open(path, encoding="utf-8"):
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        if line in ("[[drive]]", "[[keystone]]"):
            cur = {"kind": line[2:-2], "cooldown": 0, "gives": "", "reads": ""}
            recs.append(cur)
            continue
        k, v = [x.strip() for x in line.split("=", 1)]
        cur[k] = v.strip('"') if v.startswith('"') else int(v)
    return recs


class Fighter:
    """One player's set in play (and the bench set with Twin Soul)."""

    def __init__(self, drives, keys, rng, pol):
        self.d = drives          # list of (rec, level)
        self.keys = {k["effect"] for k in keys}
        self.prices = {k["price"] for k in keys}
        self.rng, self.pol = rng, pol
        self.charge = [0.0] * len(drives)
        self.cd = [0] * len(drives)
        self.last_fire = -GAP
        self.relay = []
        self.pend = []
        self.fires = 0
        self.dmg = 0.0
        self.last = None
        self.haste = 0
        self.player_dmg = 0.0
        self.f = 0

    def need(self, r):
        return r["need"] * FPS / 1000 if r["charge"] == "time" else max(1, r["need"])

    def add(self, src, n=1, foe=None):
        m = 2 if "overclock" in self.keys else 1
        for i, (r, lv) in enumerate(self.d):
            if r["charge"] != src or self.f < self.cd[i]:
                continue
            if r["reads"] == "Chilled" and not (foe and foe.chill > self.f):
                continue
            self.charge[i] = min(self.need(r), self.charge[i] + n * m)

    def fire(self, i, foe, frac=1.0, echo=False):
        r, lv = self.d[i]
        n = r["number"] / 1000 * LEVEL[lv - 1] * frac
        e = r["effect"]
        hit = 0.0
        if e in ("burst", "bolt", "shatter", "spire", "thorns", "wildfire"):
            hit = n * (2 if (e in ("bolt", "burst") and lv >= 3 and self.rng.random() < 0.3) else 1)
            if e == "burst":
                foe.burn = max(foe.burn, self.f + BURN_FRAMES)
            if e == "shatter":
                foe.chill = 0
            if e == "thorns":
                foe.mark = max(foe.mark, self.f + 300)
        elif e in ("nova", "shockwave"):
            hit = n if self.rng.random() < (0.55 if e == "nova" else 0.45) else 0.0  # in reach
        elif e == "feathers":
            hit = n
        elif e == "chill":
            foe.chill = max(foe.chill, self.f + int(n * FPS))
        elif e == "mark":
            foe.mark = max(foe.mark, self.f + int(n * FPS))
        elif e == "haste":
            self.haste = max(self.haste, self.f + int(n * FPS))
        elif e == "heal":
            self.dmg -= 0  # survival only (taken into account by the ladder)
        elif e == "surge":
            for j, (q, _) in enumerate(self.d):
                if q["effect"] != "surge" and q["charge"] != "passive":
                    self.charge[j] = self.need(q)
        elif e == "relay":
            self.relay = [j for j, (q, _) in enumerate(self.d) if q["effect"] not in ("relay", "tailwind")]
        elif e == "repeat" and self.last is not None:
            self.pend.append((self.f + 1, self.last, 0.5))
        if foe.mark > self.f:
            hit *= 1 + MARK_MORE
        self.dmg += hit
        foe.pct += hit
        self.fires += 1
        if not echo and e != "repeat":
            self.last = i
            if "echo" in self.keys and e != "relay":
                self.pend.append((self.f + ECHO_DELAY, i, 0.5))
            self.add("fire")

    def step(self, foe):
        """One frame: events from the pace, time charges, the queue, Relay, pending repeats, Burning."""
        p, rng = self.pol, self.rng
        speed = HASTE if self.haste > self.f else 1.0
        if rng.random() < p["hit"] * speed / FPS:
            mult = 1.0 + (0.25 * self._tailwind() if self.haste > self.f else 0.0)
            if foe.mark > self.f:
                mult += MARK_MORE
            base = p["hit_pct"] * mult * (0.8 if "Weak" in self.prices else 1.0) * (1.6 if "glass" in self.keys else 1.0)
            foe.pct += base
            self.player_dmg += base
            self.add("hit", foe=foe)
            if "plague" in self.keys:
                foe.burn = max(foe.burn, self.f + BURN_FRAMES)
                foe.stacks = min(3, foe.stacks + 1)
            if "winter" in self.keys:
                foe.chill = max(foe.chill, self.f + 120)
            if rng.random() < p["aerial"] / max(p["hit"], 1e-6):
                self.add("aerial_hit")
            if foe.pct >= 100:
                self.add("hit_100")
        if rng.random() < p["taken"] / FPS:
            self.add("taken")
        if rng.random() < p["landing"] / FPS:
            self.add("landing")
        if rng.random() < p["jump"] / FPS:
            self.add("jump")
        if rng.random() < p["tag"] / FPS:
            self.add("tag_in")
        self.add("time")
        if self.f - self.last_fire >= GAP:
            for i, (r, _) in enumerate(self.d):
                if r["charge"] != "passive" and self.charge[i] >= self.need(r):
                    self.charge[i] = 0
                    self.cd[i] = self.f + r["cooldown"] * FPS // 1000
                    self.last_fire = self.f
                    self.fire(i, foe)
                    break
        if self.relay and self.f % RELAY_GAP == 0:
            j = self.relay.pop(0)
            self.charge[j] = 0
            self.fire(j, foe)
        for q in [q for q in self.pend if q[0] <= self.f]:
            self.pend.remove(q)
            self.fire(q[1], foe, q[2], echo=True)
        if foe.burn > self.f and (self.f % BURN_TICK) == 0:
            hit = BURN_TICK_PCT * max(1, foe.stacks)
            foe.pct += hit
            self.dmg += hit
        self.f += 1

    def _tailwind(self):
        return max((LEVEL[lv - 1] for r, lv in self.d if r["effect"] == "tailwind"), default=0.0)


class Foe:
    def __init__(self):
        self.pct = 0.0
        self.player_dmg = 0.0
        self.burn = self.chill = self.mark = 0
        self.stacks = 1


def fire_period(drives, keys, pol, seed, seconds=120, shifts=False):
    rng = random.Random(seed)
    f = Fighter(drives, keys, rng, pol)
    foe = Foe()
    for t in range(seconds * FPS):
        f.step(foe)
        if foe.pct >= KO_PCT:
            # a KO: Wildfire reads a Burning foe
            for i, (r, _) in enumerate(f.d):
                if r["charge"] == "ko" and (r["reads"] != "Burning" or foe.burn > f.f):
                    f.charge[i] = f.need(r)
            foe = Foe()
        if shifts and t and t % int(SHIFT_EVERY * FPS) == 0:
            f.add("shift")
    return seconds / max(f.fires, 1), f.dmg, f.player_dmg


def draw_build(drives, keystones, rng, n, levels, nkeys):
    """A build in play: `n` drives on the fighter in play (4 at most), the bench's too with Twin Soul."""
    keys = rng.sample(keystones, nkeys)
    twin = any(k["effect"] == "twin" for k in keys)
    pick = rng.sample(drives, min(len(drives), n * (2 if twin else 1)))
    return [(r, rng.choice(levels)) for r in pick], keys


def run_ladder(rng, pol, drives, keystones):
    """A Short run with a build that grows (3 drives -> 8 at levels 2-3, keystones 1 -> 3): True when cleared."""
    pool = 4
    for fight, (stocks, foe_dps, carries) in enumerate(LADDER):
        n = min(4, 3 + (fight >= 2))
        levels = [1] if fight == 0 else [1, 2] if fight < 3 else [2, 3]
        build, keys = draw_build(drives, keystones, rng, n, levels, min(3, 1 + (fight >= 2) + (fight >= 4)))
        period, ddmg, _ = fire_period(build, keys, pol, rng.randrange(1 << 30), seconds=20)
        drive_dps = ddmg / 20
        player_dps = pol["hit"] * pol["hit_pct"]
        foe_mult = (1.25 if any(k["price"] == "Fragile" for k in keys) else 1.0) * (1.6 if any(k["effect"] == "glass" for k in keys) else 1.0)
        t_ko_foe = KO_PCT / (player_dps + drive_dps)
        t_ko_me = KO_PCT / (foe_dps * foe_mult * (1.15 if carries else 1.0))
        # stocks trade at the ratio of the two clocks, with noise
        for _ in range(stocks):
            ratio = t_ko_me / t_ko_foe * rng.uniform(0.75, 1.3)
            pool -= max(0, int(1 / ratio)) if ratio < 1 else (1 if rng.random() < 0.35 / ratio else 0)
            if pool <= 0:
                return False
    return True


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--seeds", type=int, default=200)
    ap.add_argument("--json", action="store_true")
    a = ap.parse_args()
    recs = parse_pool(POOL)
    drives = [r for r in recs if r["kind"] == "drive"]
    keystones = [r for r in recs if r["kind"] == "keystone"]
    out = {"policies": {}, "drives": {}}
    ok = True
    std = POLICIES["brawler"]
    for r in drives:
        if r["charge"] in ("passive", "ko", "shift", "fire", "tag_in", "hit_100") or r["reads"]:
            continue
        per = [fire_period([(r, 1)], [], std, s, seconds=90)[0] for s in range(20)]
        out["drives"][r["key"]] = round(sum(per) / len(per), 2)
    for name, pol in POLICIES.items():
        rng = random.Random(1234)
        early, full, share, wins = [], [], [], 0
        for s in range(a.seeds):
            b, k = draw_build(drives, keystones, rng, 3, [1], 1)
            early.append(fire_period(b, k, pol, s, seconds=60)[0])
            b, k = draw_build(drives, keystones, rng, 4, [2, 3], 3)
            per, ddmg, pdmg = fire_period(b, k, pol, s, seconds=60, shifts=True)
            full.append(per)
            share.append(ddmg / max(ddmg + pdmg, 1e-9))
            wins += run_ladder(rng, pol, drives, keystones)
        med = lambda v: sorted(v)[len(v) // 2]
        row = {
            "early_s": round(med(early), 2),
            "full_s": round(med(full), 2),
            "drive_share_full": round(med(share), 2),
            "win": round(wins / a.seeds, 2),
        }
        out["policies"][name] = row
    # the bands hold for the standard policy (brawler) and the early / full fire rate for every policy
    for name, row in out["policies"].items():
        for k, (lo, hi) in BANDS.items():
            if name != "brawler" and k in ("win", "drive_share_full"):
                continue
            if not lo <= row[k] <= hi:
                ok = False
                row.setdefault("miss", []).append(f"{k} {row[k]} not in [{lo}, {hi}]")
    for key, per in out["drives"].items():
        if not 2.0 <= per <= 16.0:
            ok = False
            out.setdefault("drive_miss", []).append(f"{key} {per}s")
    if a.json:
        print(json.dumps(out, indent=1))
    else:
        print("policy      early s/fire  full s/fire  drive share  win")
        for name, row in out["policies"].items():
            print(f"{name:10s}  {row['early_s']:12.2f}  {row['full_s']:11.2f}  {row['drive_share_full']:11.2f}  {row['win']:.2f}  {' '.join(row.get('miss', []))}")
        print("level-1 period alone (brawler pace): " + ", ".join(f"{k} {v}s" for k, v in out["drives"].items()))
        if out.get("drive_miss"):
            print("outside 2-16 s: " + ", ".join(out["drive_miss"]))
        print("BANDS OK" if ok else "BANDS MISSED")
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
