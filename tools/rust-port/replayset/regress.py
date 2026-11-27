#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Build the regression replay set: a fixed list small enough to run after every engine change (~15 min at 8 jobs),
chosen so that what it covers is what has broken before or is most likely to break.

    python -I tools/rust-port/replayset/regress.py [--sweep CSV] [--v320 CSV] [--out TSV]

Inputs: the full sweep's re-scored rows (set,relpath,status,first_divergence,frames,first_hard,max_delta,version,...),
the 3.20.0 batch's scores, and the replay files themselves (Game Start: stage, characters, player types, teams).
Output: TSV `set relpath sha256 status reasons` (the frozen.tsv layout plus a reasons column), read by
tools/rust-port/slippi/hard_desync_sweep.sh with LIST=<out>. Picks are deterministic (ordered by a hash of the path).

What goes in, and why:
- every replay that ever failed (not byte-exact, a hard desync, or a crash), any version: they are the known bugs,
  and a fix is proven when one of them turns to match; a regression when a match turns into one of them
- hero: all of the owner's own online games
- ext: all (a small external set)
- 3.20.0: all (the current Slippi; the newest replay format and RNG scheme)
- high risk, from matching 3.x games, capped per tag: characters with a second entity or transform (Ice Climbers,
  Zelda/Sheik), random specials and items (Peach, Game & Watch, Luigi, Link, Young Link, Samus), unusual hurtbox or
  movement (Mewtwo, Kirby, Pikachu, Pichu, Ness, Yoshi, Jigglypuff), stages with randomness or hazards and every
  non-tournament stage, more than two players, teams, CPUs, the longest games
- baseline: up to BASE matching games per (Slippi version, stage), so every version x stage seen keeps coverage
- old (pre-3.0, the 2019 set): a small sample of matching games, for the old format paths only
"""
import argparse
import collections
import csv
import hashlib
import os
import struct
import sys

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "slippi"))
from slp_rollbacks import events  # noqa: E402

W = "E:/Projects/Melee Workspace"
DIRS = {"ext": W + "/scratch/replays-ext", "hero": W + "/_build/replays/hero-2026-11-06", "lunar": W + "/_build/slp",
        "gang": W + "/Gang-Steals", "lunar2": W + "/_build/replays/lunar-2026-11-20",
        "v320": W + "/_build/replays/lunar-2026-11-27/v320"}
LEGAL = {2, 3, 8, 28, 31, 32}  # FoD, Stadium, Yoshi's, Dream Land, Battlefield, FD (Slippi external stage ids)
RNG_STAGES = {2: "fod", 3: "stadium", 8: "yoshis", 28: "dreamland", 24: "bigblue"}
CHAR_TAGS = {14: "ics", 18: "zelda-sheik", 19: "zelda-sheik", 12: "peach", 3: "gnw", 7: "luigi", 6: "link",
             21: "younglink", 16: "samus", 10: "mewtwo", 4: "kirby", 13: "pikachu", 24: "pichu", 11: "ness",
             17: "yoshi", 15: "puff"}
CAP = {"char": 25, "rng-stage": 30, "stage": 20, "players": 40, "teams": 30, "cpu": 20, "long": 30}
BASE = 8
OLD_SAMPLE = 60


def key(s):
    return hashlib.sha256(s.encode()).hexdigest()


def game_start(path):
    with open(path, "rb") as f:
        d = f.read(8192)
    k = d.find(b"raw[$U#l")
    if k < 0:
        return None
    for c, p in events(d[k + 12:]):
        if c == 0x36 and len(p) > 0x64 + 0x24 * 4:
            pl = [(p[0x64 + 0x24 * i], p[0x65 + 0x24 * i]) for i in range(4)]
            return {"version": "%d.%d.%d" % (p[0], p[1], p[2]), "stage": struct.unpack(">H", p[0x12:0x14])[0],
                    "teams": p[0xC], "players": [ch for ch, t in pl if t != 3], "cpu": any(t == 1 for _, t in pl)}
    return None


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--sweep", default=W + "/_build/agents/referee/sweep2/rescored.csv")
    ap.add_argument("--v320", default=W + "/_build/agents/referee/v320/scores.csv")
    ap.add_argument("--out", default=W + "/scratch/replayset/regress.tsv")
    a = ap.parse_args()

    rows = []  # (set, relpath, status, first_hard, frames)
    for r in csv.reader(open(a.sweep, encoding="utf-8")):
        if len(r) >= 6:
            rows.append((r[0], r[1], r[2], r[5], int(r[4] or 0)))
    if os.path.exists(a.v320):
        for r in csv.reader(open(a.v320, encoding="utf-8")):
            if len(r) >= 7:
                rows.append(("v320", r[0] + ".slp", r[1], r[6], int(r[5] or 0)))

    meta = {}
    for s, rel, *_ in rows:
        p = os.path.join(DIRS[s], rel)
        try:
            meta[(s, rel)] = game_start(p)
        except OSError:
            meta[(s, rel)] = None

    pick = collections.defaultdict(set)
    tags = collections.defaultdict(list)
    for s, rel, st, fh, fr in rows:
        m = meta[(s, rel)]
        failed = st != "match" or (fh not in ("", "old"))
        if failed:
            pick[(s, rel)].add("failed")
        if s in ("hero", "ext"):
            pick[(s, rel)].add(s)
        if m and m["version"] == "3.20.0":
            pick[(s, rel)].add("v3.20")
        if failed or not m:
            continue
        if fh == "old":
            tags["old"].append((s, rel))
            continue
        for ch in set(m["players"]):
            if ch in CHAR_TAGS:
                tags["char:" + CHAR_TAGS[ch]].append((s, rel))
        if m["stage"] in RNG_STAGES:
            tags["rng-stage:" + RNG_STAGES[m["stage"]]].append((s, rel))
        elif m["stage"] not in LEGAL:
            tags["stage:%d" % m["stage"]].append((s, rel))
        if len(m["players"]) > 2:
            tags["players:%d" % len(m["players"])].append((s, rel))
        if m["teams"]:
            tags["teams"].append((s, rel))
        if m["cpu"]:
            tags["cpu"].append((s, rel))
        tags["base:%s:%d" % (m["version"], m["stage"])].append((s, rel))
    longest = sorted(((fr, s, rel) for s, rel, st, fh, fr in rows if st == "match" and fh == "" and meta[(s, rel)]),
                     reverse=True)[:CAP["long"]]
    for _, s, rel in longest:
        pick[(s, rel)].add("long")

    for t, lst in tags.items():
        kind = t.split(":")[0]
        cap = OLD_SAMPLE if kind == "old" else BASE if kind == "base" else CAP.get(kind, 20)
        for s, rel in sorted(set(lst), key=lambda x: key(x[0] + "/" + x[1]))[:cap]:
            pick[(s, rel)].add(t)

    status = {(s, rel): st for s, rel, st, *_ in rows}
    os.makedirs(os.path.dirname(a.out), exist_ok=True)
    with open(a.out, "w", encoding="utf-8", newline="\n") as f:
        f.write("set\trelpath\tsha256\tstatus\treasons\n")
        for (s, rel) in sorted(pick):
            h = hashlib.sha256(open(os.path.join(DIRS[s], rel), "rb").read()).hexdigest()
            f.write(f"{s}\t{rel}\t{h}\t{status[(s, rel)]}\t{','.join(sorted(pick[(s, rel)]))}\n")
    by = collections.Counter()
    for v in pick.values():
        for t in v:
            by[t.split(":")[0]] += 1
    print(f"{len(pick)} replays -> {a.out}")
    print("  by reason (a replay can have several):", dict(sorted(by.items())))
    print("  by set:", dict(collections.Counter(s for s, _ in pick)))


if __name__ == "__main__":
    main()
