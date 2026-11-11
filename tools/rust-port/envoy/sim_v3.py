#!/usr/bin/env python3
"""Envoy V3 balancing harness (envoy-synergy-v3.md 12.10 B): a model of the sim's Heat rules, not the engine.

It reads rust/mods/envoy_data/pool.toml (the V3 fields: kind, coef, when, launch) and plays fights as streams of
moves (damage pieces) and hits taken, drawn from a per-policy pace, in 0.1 s steps, with the sim's integer Heat on
top (the same products as envoy_data/src/heat.rs: 1 Heat = 2e9 ticks, the 150% half, the gain multiplier, the cap),
the Heat states' fire scale, chains (rightward charge by coefficient, 4 links, 6 frames apart, 20 between chains),
Echo, the +Heat and xGain drives, Insulate, Burning ticks as word damage, the cash-out (coins, pips, the flare, half
kept) and a lost stock halving Heat. The measures and bands are 12.10 B's:

  1 build spread   best five-drive build's output per 100% of move damage over a random build's   2.5 - 6
  2 heat spread    median Heat at a KO, best build over a random build                              1.5 - 2.5
  3 char spread    one build, four damage profiles of the same total: highest output over lowest  <= 1.25
  4 split          the split test on the model (one record against 2 / 7 / 30)                     identical
  5 use            drives of the slice's 16 in at least one top-ten build                          >= 12
  6 ko delay       coins of a policy that delays KOs to cash out higher over one that does not      < 15%
  7 pace           no drives: reference Heat at the first KO 80-100 and not Blazing before the 4th; two-for-one
                   Warm by the first KO; an even trade never leaves Cold

    python tools/rust-port/envoy/sim.py --v3 [--seeds 120] [--json]       exit 1 when a band is missed
"""
import itertools
import json
import random
import statistics

TICK = 2_000_000_000            # 1 Heat
MARKS = [40, 90, 165]
CAP = 225 * TICK
STATE_SCALE = [1000, 1250, 1500, 2000]
LEVEL = [1000, 1600, 2400]
OVERKILL = 150_000
GAIN_CAP = 4000
CHARGE_UNIT = 1_000_000
COIN = 30_000_000_000_000
PIP = 300_000_000_000_000
FLARE_DIV = 30_000_000_000
FLARE_MAX = 15_000
FIRE_CAP, LAUNCH_CAP = 30_000, 40_000
WORD_COEF = 200
BURN_PCT = 1500                 # per stack a second
DT = 6                          # frames per step
SLICE = ["ember", "bolt", "nova", "wildfire", "rush", "tailwind", "clean_landing", "perfect_form", "riposte",
         "insulate", "feather_storm", "gatekeeper", "frost", "shatter", "echo", "metronome"]
KO_PCT = 120_000

# policies: damage dealt per second (x1000 percent), mean move size, the share of aerials, L-cancel rate, offstage
# share, damage taken per second, techs per second, distinct moves used (Metronome)
POLICIES = {
    "brawler": dict(rate=6500, size=9000, aerial=0.25, lc=0.4, off=0.10, taken=2600, tech=0.05, moves=6),
    "aerial": dict(rate=6500, size=8000, aerial=0.6, lc=0.7, off=0.15, taken=2400, tech=0.04, moves=5),
    "tagger": dict(rate=6000, size=8000, aerial=0.3, lc=0.4, off=0.10, taken=2600, tech=0.04, moves=8),
    "turtle": dict(rate=4500, size=11000, aerial=0.15, lc=0.2, off=0.05, taken=1500, tech=0.08, moves=4),
}
# damage profiles (measure 3): pieces per move and move size; the same damage rate
PROFILES = {
    "big hits": (1, 15000),
    "small hits": (1, 4000),
    "multi-hit": (7, 2000),
    "chip": (1, 2000),
}


def parse_pool(path):
    recs, cur = [], None
    for line in open(path, encoding="utf-8"):
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        if line in ("[[drive]]", "[[keystone]]"):
            cur = {"type": line[2:-2], "gives": "", "reads": "", "when": "", "coef": 0, "need": 0, "launch": False,
                   "kind": "", "charge": ""}
            recs.append(cur)
            continue
        k, v = [x.strip() for x in line.split("=", 1)]
        if v in ("true", "false"):
            cur[k] = v == "true"
        else:
            cur[k] = v.strip('"') if v.startswith('"') else int(v)
    return recs


def state_of(h):
    return sum(1 for m in MARKS if h >= m * TICK)


def heat_gain(dmg, foe_m, coef, gain, no_line=False):
    below = dmg if no_line else max(0, min(dmg, OVERKILL - foe_m))
    above = dmg - below
    return (2 * below + above) * coef * gain


def fire_dmg(number, level, st, launch):
    n = number * LEVEL[level - 1] // 1000 * STATE_SCALE[st] // 1000
    return min(n, LAUNCH_CAP if launch else FIRE_CAP)


class Fight:
    """One player against a stream of foes (a new foe at each KO), `drives` = [(rec, level)] in slot order."""

    def __init__(self, drives, pol, rng, profile=None, delay=False, glass=False):
        self.d = drives
        self.pol, self.rng, self.profile, self.delay, self.glass = pol, rng, profile, delay, glass
        self.heat = 0
        self.chg = [0] * len(drives)
        self.time_chg = [0] * len(drives)
        self.foe = 0            # the foe's percent x1000 (the sim's own)
        self.tally = 0
        self.purse = 0
        self.pips = 0
        self.f = 0
        self.haste = 0
        self.chill = 0
        self.burn = 0
        self.burn_stacks = 0
        self.mark = 0
        self.streak = 0
        self.hist = []
        self.free = 20_000
        self.move_dmg = 0
        self.extra = 0          # fires, word damage, flares (x1000)
        self.ko_heat = []
        self.ch = None          # (slot, link, coef, next frame)
        self.ch_end = -99
        self.pend = []
        self.fires = 0

    def has(self, key):
        return any(r["key"] == key for r, _ in self.d)

    def gain(self, tags):
        g = 1000
        for r, lv in self.d:
            if r["kind"] != "gain_mul":
                continue
            ok = {"haste": self.haste > self.f, "lcancel_streak": self.streak >= 3, "offstage": "off" in tags,
                  "aerial": "air" in tags}.get(r["when"], False)
            if ok:
                g = min(GAIN_CAP, g * (1000 + (r["number"] - 1000) * LEVEL[lv - 1] // 1000) // 1000)
        if self.glass:
            g = min(GAIN_CAP, g * 2)
        return g

    def add_heat(self, t):
        self.heat = max(0, min(CAP, self.heat + t))

    def heat_add(self, when):
        for r, lv in self.d:
            if r["kind"] == "heat_add" and r["when"] == when:
                self.add_heat(r["number"] * LEVEL[lv - 1] // 1000 * (TICK // 1000))

    def deal(self, dmg, coef, tags, src_slot=None):
        """One record on the foe (V3 3.3)."""
        if self.mark > self.f and "word" not in tags:
            extra = dmg * 300 // 1000
            self.extra += extra
            self.deal(extra, WORD_COEF, {"word"})
        self.add_heat(heat_gain(dmg, self.foe, coef, self.gain(tags)))
        self.foe += dmg
        self.tally += dmg
        if "word" in tags:
            return
        for i, (r, lv) in enumerate(self.d):
            if r["kind"] != "trigger" or (src_slot is not None and i <= src_slot):
                continue
            src = r["charge"]
            if src == "dealt" or (src == "aerial_dealt" and "air" in tags and src_slot is None) or \
                    (src == "chilled_dealt" and self.chill > self.f) or (src == "offstage_dealt" and "off" in tags):
                self.chg[i] += dmg * coef * 1000

    def need(self, r):
        if r["charge"] == "time":
            return r["need"] * 60 // 1000
        if r["charge"] in ("tag_in", "shift", "cashout"):
            return 1
        return r["need"] * CHARGE_UNIT

    def full(self, i):
        r = self.d[i][0]
        if r["kind"] != "trigger":
            return False
        c = self.time_chg[i] if r["charge"] == "time" else self.chg[i]
        return c >= self.need(r)

    def fire(self, i, coef, frac=1000, repeat=False):
        r, lv = self.d[i]
        st = state_of(self.heat)
        n = fire_dmg(r["number"], lv, st, r["launch"]) * frac // 1000
        e = r["effect"]
        dealt = 0
        if e in ("burst", "bolt", "chill", "shatter", "mark", "nova", "shockwave", "spire", "wildfire"):
            dealt = n
            if e == "bolt" and (lv >= 3 or st >= 2):
                dealt += n // 2       # the chain to a second foe, counted at half (a foe is not always there)
            if e == "burst":
                self.burn = self.f + 240
                self.burn_stacks = max(self.burn_stacks, 1)
            if e == "chill":
                self.chill = self.f + 240
            if e == "shatter":
                self.chill = 0
            if e == "mark":
                self.mark = self.f + 300
        elif e == "feathers":
            dealt = n * (3 + (st >= 2) + (st >= 3))
        elif e == "haste":
            self.haste = max(self.haste, self.f + n * 60 // 1000)
        self.fires += 1
        if dealt:
            self.extra += dealt
            self.deal(dealt, coef, {"fire"}, src_slot=i)
        if not repeat and i > 0 and self.d[i - 1][0]["kind"] == "neighbour":
            frac2 = self.d[i - 1][0]["number"] * LEVEL[self.d[i - 1][1] - 1] // 1000
            self.pend.append((self.f + 6, i, coef, frac2))

    def chain(self):
        n = len(self.d)
        if self.ch is None:
            if self.f - self.ch_end < 20:
                return
            k = next((i for i in range(n) if self.full(i)), None)
            if k is None:
                return
            self.ch = (k, 1, 1000, self.f)
        k, link, pc, at = self.ch
        if self.f < at:
            return
        r = self.d[k][0]
        if r["charge"] == "time":
            self.time_chg[k] -= self.need(r)
        elif r["charge"] in ("tag_in", "shift", "cashout"):
            self.chg[k] = 0
        else:
            self.chg[k] -= self.need(r)
        coef = r["coef"] * pc // 1000
        self.fire(k, coef)
        nxt = next((j for j in range(k + 1, n) if self.full(j)), None)
        if nxt is not None and link < 4:
            self.ch = (nxt, link + 1, max(1, coef), self.f + 6)
        else:
            self.ch, self.ch_end = None, self.f

    def ko(self):
        h = self.heat
        self.ko_heat.append(h / TICK)
        c = h * 1000
        self.purse += c
        self.pips += c
        fl = min(FLARE_MAX, h * 1000 // FLARE_DIV)
        self.extra += fl            # the flare reaches the next foe in a pack; counted as output
        burning = self.burn > self.f
        for i, (r, lv) in enumerate(self.d):
            if r["charge"] == "cashout" and (r["when"] != "burning_foe" or burning):
                self.chg[i] = 1
        self.heat //= 2
        self.free = 20_000
        self.foe = 0
        self.tally = 0
        self.burn = 0

    def run(self, seconds=150, kos=None):
        p, rng = self.pol, self.rng
        pieces, size = self.profile if self.profile else (1, p["size"])
        moves = p["moves"]
        per_step_moves = p["rate"] / (pieces * size) * DT / 60
        per_step_taken = p["taken"] / 9000 * DT / 60
        end = seconds * 60
        while self.f < end and (kos is None or len(self.ko_heat) < kos):
            self.f += DT
            for i, (r, lv) in enumerate(self.d):
                if r["kind"] == "trigger" and r["charge"] == "time":
                    self.time_chg[i] = min(self.time_chg[i] + DT, 2 * self.need(r))
            if rng.random() < per_step_moves:
                air = rng.random() < p["aerial"]
                off = rng.random() < p["off"]
                tags = {"air"} if air else set()
                if off:
                    tags.add("off")
                mv = rng.randrange(moves)
                if mv not in self.hist:
                    self.heat_add("new_move")
                self.hist = ([mv] + [x for x in self.hist if x != mv])[:3]
                for _ in range(pieces):
                    d = max(500, int(rng.gauss(size, size * 0.25)))
                    self.move_dmg += d
                    self.deal(d, 1000, tags)
                if air:
                    lc = rng.random() < p["lc"]
                    self.streak = min(3, self.streak + 1) if lc else 0
                    if lc:
                        self.heat_add("lcancel")
            if rng.random() < per_step_taken:
                d = int(rng.gauss(9000, 2500))
                d = max(1000, d)
                if self.has("insulate"):
                    free = min(self.free, d)
                    self.free -= free
                    d -= free
                self.heat = max(0, self.heat - d * 1000 * 2000)
            if rng.random() < p["tech"] * DT / 60:
                self.heat_add("tech")
            if self.burn > self.f and self.f % 60 < DT:
                w = BURN_PCT * self.burn_stacks
                self.extra += w
                self.deal(w, WORD_COEF, {"word"})
            for e in [e for e in self.pend if e[0] <= self.f]:
                self.pend.remove(e)
                self.fire(e[1], e[2], e[3], repeat=True)
            self.chain()
            ko_at = KO_PCT + (25_000 if self.delay else 0)
            if self.foe >= ko_at:
                self.ko()
        return self


def output_per_100(drives, pol, seeds, profile=None, glass=False):
    outs = []
    for s in range(seeds):
        f = Fight(drives, pol, random.Random(s * 7919 + 1), profile=profile, glass=glass).run()
        outs.append(100.0 * f.extra / max(1, f.move_dmg))
    return statistics.mean(outs)


def median_ko_heat(drives, pol, seeds):
    hs = []
    for s in range(seeds):
        hs += Fight(drives, pol, random.Random(s * 104729 + 3)).run().ko_heat
    return statistics.median(hs) if hs else 0.0


def pace():
    """Measure 7 by the model's own integers: four lots of 30, taking between them, then the KO, repeated."""
    out = {}
    for name, take in (("reference", 10_000), ("two_for_one", 15_000), ("even", 30_000)):
        f = Fight([], POLICIES["brawler"], random.Random(0))
        heats, blazing_before = [], None
        for ko in range(5):
            for i in range(4):
                f.deal(30_000, 1000, set())
                if i < 3:
                    f.heat = max(0, f.heat - take * 1000 * 2000)
            heats.append(f.heat / TICK)
            if state_of(f.heat) >= 3 and blazing_before is None:
                blazing_before = ko + 1
            f.ko()
            f.heat = max(0, f.heat - take * 1000 * 2000)
        out[name] = dict(heats=heats, first_state=state_of(int(heats[0] * TICK)), blazing_at=blazing_before)
    return out


def split_model():
    """Measure 4: the model's deal() on one record against 2, 7 and 30 that add up to it."""
    for n in (2_000, 14_000, 30_000, 160_000):
        a = Fight([], POLICIES["brawler"], random.Random(0))
        a.deal(n, 1000, set())
        for k in (2, 7, 30):
            b = Fight([], POLICIES["brawler"], random.Random(0))
            left = n
            for i in range(k):
                d = left if i == k - 1 else n // k
                b.deal(d, 1000, set())
                left -= d
            if (a.heat, a.tally, a.foe) != (b.heat, b.tally, b.foe):
                return False
    return True


def main(args, pool_path):
    recs = parse_pool(pool_path)
    by = {r["key"]: r for r in recs if r["type"] == "drive"}
    slice_ = [by[k] for k in SLICE if k in by]
    seeds = max(8, args.seeds // 10)
    rng = random.Random(12345)
    res, ok = {}, True

    def build(keys, lv=1):
        return [(by[k], lv) for k in keys]

    # candidate five-drive builds: random rows of the slice, plus the hand-built archetypes of 9
    archetypes = {
        "firestorm": ["echo", "ember", "bolt", "frost", "shatter"],
        "tech monk": ["clean_landing", "perfect_form", "riposte", "metronome", "ember"],
        "sky": ["gatekeeper", "feather_storm", "echo", "bolt", "ember"],
        "frozen": ["frost", "echo", "shatter", "bolt", "nova"],
        "inferno": ["ember", "echo", "bolt", "nova", "wildfire"],
    }
    rows = [rng.sample(SLICE, 5) for _ in range(60)] + list(archetypes.values())
    lines = []
    for pname, pol in POLICIES.items():
        scored = sorted(((output_per_100(build(r, 2), pol, seeds), tuple(r)) for r in rows), reverse=True)
        rand_out = statistics.mean(o for o, _ in scored[: -len(archetypes)]) if len(scored) > len(archetypes) else scored[-1][0]
        best_out, best = scored[0]
        spread = best_out / max(0.01, rand_out)
        # Heat spread: the build that runs hottest (median Heat at a KO) over the random rows' mean
        heats = [(median_ko_heat(build(r, 2), pol, max(4, seeds // 2)), tuple(r)) for r in rows]
        rand_heat = statistics.mean(h for h, _ in heats[: -len(archetypes)])
        hot_heat, hot = max(heats)
        hs = hot_heat / max(1.0, rand_heat)
        res[pname] = dict(best=list(best), best_out=round(best_out, 1), random_out=round(rand_out, 1),
                          build_spread=round(spread, 2), heat_spread=round(hs, 2),
                          top10=[list(r) for _, r in scored[:10]])
        lines.append(f"  {pname:8s} build spread {spread:5.2f} (best {best_out:6.1f}% / random {rand_out:5.1f}% per 100%)"
                     f"   heat spread {hs:4.2f} (hottest {','.join(hot)})   best: {','.join(best)}")
    # 3 character spread: one build (Firestorm), four profiles of the same damage rate
    outs = {name: output_per_100(build(archetypes["firestorm"], 2), POLICIES["brawler"], seeds, profile=prof)
            for name, prof in PROFILES.items()}
    char = max(outs.values()) / max(0.01, min(outs.values()))
    lines.append("  character spread %.2f (%s)" % (char, ", ".join(f"{k} {v:.1f}" for k, v in outs.items())))
    # 5 use
    used = set()
    for p in res.values():
        for r in p["top10"]:
            used |= set(r)
    lines.append(f"  use {len(used & set(SLICE))} of {len(SLICE)} slice drives in a top-ten build")
    # 6 KO delay
    fb = build(archetypes["firestorm"], 2)
    norm = statistics.mean(Fight(fb, POLICIES["brawler"], random.Random(s), delay=False).run(kos=4).purse for s in range(seeds))
    dly = statistics.mean(Fight(fb, POLICIES["brawler"], random.Random(s), delay=True).run(kos=4).purse for s in range(seeds))
    delay = dly / max(1, norm) - 1.0
    lines.append(f"  ko delay {100 * delay:+.1f}% coins (four KOs, delayed by 25%)")
    # 7 pace and 4 split
    pc = pace()
    pace_ok = (80 <= pc["reference"]["heats"][0] <= 100 and (pc["reference"]["blazing_at"] or 99) >= 4
               and pc["two_for_one"]["first_state"] >= 1 and all(h < 40 for h in pc["even"]["heats"]))
    lines.append("  pace: reference %s (Blazing at KO %s), two-for-one %s, even %s" % (
        [round(h) for h in pc["reference"]["heats"]], pc["reference"]["blazing_at"],
        [round(h) for h in pc["two_for_one"]["heats"]], [round(h) for h in pc["even"]["heats"]]))
    split = split_model()
    lines.append(f"  split {'identical' if split else 'DIFFERS'}")
    bands = []
    for pname, r in res.items():
        bands.append((f"{pname} build spread", 2.5 <= r["build_spread"] <= 6.0))
        bands.append((f"{pname} heat spread", 1.5 <= r["heat_spread"] <= 2.5))
    bands += [("character spread", char <= 1.25), ("use", len(used & set(SLICE)) >= 12),
              ("ko delay", delay < 0.15), ("pace", pace_ok), ("split", split)]
    print("Envoy V3 harness (model of the Heat rules; envoy-synergy-v3.md 12.10 B)")
    for line in lines:
        print(line)
    missed = [n for n, good in bands if not good]
    print("bands: " + ("all met" if not missed else "missed: " + "; ".join(missed)))
    if args.json:
        print(json.dumps(dict(policies=res, character=outs, use=sorted(used), ko_delay=delay, pace=pc), indent=1))
    return 1 if missed and not args.no_fail else 0
