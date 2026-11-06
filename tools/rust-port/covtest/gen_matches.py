# SPDX-License-Identifier: MIT
"""The CPU seed-match configs (notes/coverage-testing.md), one JSON object per line:
   python -I gen_matches.py OUT.jsonl [--sets pairs,stages,levels,items,doubles] [--pair-stage 31]
id, set, players [[ckind, cpu_level, costume, team]], stage (external StKind), items, minutes, stocks, teams.
Sets:
  pairs    every ordered pair of the 26 selectable characters (CharacterKind 0-25: Sheik and Zelda both, the Ice
           Climbers as one slot), CPU level 9, items off, on one stage (--pair-stage, default Battlefield): 676
  stages   six matchups on each of the 29 stages below (legal + the casual VS stages), level 9, items off
  levels   four matchups at CPU levels 1, 3, 5, 7 on Final Destination
  items    the stage list's legal six with items on (medium, all switches) for four matchups
  doubles  2v2 teams on the legal six for four team pairings, level 9
Every match: 4 stocks, a 1-minute stock-match timer (3600 frames at most).
The CharacterKind and StKind numbers are the decomp's (src/melee/ft/forward.h, src/melee/gr/forward.h)."""
import json, sys

CHARS = ["falcon", "dk", "fox", "gw", "kirby", "bowser", "link", "luigi", "mario", "marth", "mewtwo", "ness", "peach",
         "pikachu", "ics", "jigglypuff", "samus", "yoshi", "zelda", "sheik", "falco", "younglink", "drmario", "roy",
         "pichu", "ganon"]
LEGAL = {2: "fod", 3: "ps", 8: "ys", 28: "dl", 31: "bf", 32: "fd"}
CASUAL = {4: "castle", 5: "kongo", 6: "brinstar", 7: "corneria", 9: "onett", 10: "mutecity", 11: "rcruise", 12: "japes",
          13: "greatbay", 14: "temple", 15: "depths", 16: "yoshiisland", 17: "greens", 18: "fourside", 19: "mk1",
          20: "mk2", 22: "venom", 23: "pokefloats", 24: "bigblue", 25: "icicle", 27: "flatzone", 29: "pastyi",
          30: "pastkongo"}
STAGES = {**LEGAL, **CASUAL}
SWEEP = [(2, 20), (9, 15), (14, 19), (12, 0), (5, 13), (17, 10)]  # fox-falco, marth-puff, ics-sheik, peach-falcon, bowser-pika, yoshi-mewtwo


def cfg(set_, players, stage, items=False, teams=False):
    ids = "-".join(f"{CHARS[c]}{l}" for c, l, _, _ in players)
    return {"id": f"{set_}_{ids}_{STAGES[stage]}{'_items' if items else ''}", "set": set_, "players": players,
            "stage": stage, "items": items, "minutes": 1, "stocks": 4, "teams": teams}


def duel(a, b, lvl=9):
    return [[a, lvl, 0, 0], [b, lvl, 1 if a == b else 0, 1]]


def main():
    out = sys.argv[1]
    sets = sys.argv[sys.argv.index("--sets") + 1].split(",") if "--sets" in sys.argv else ["pairs", "stages", "levels", "items", "doubles"]
    pst = int(sys.argv[sys.argv.index("--pair-stage") + 1]) if "--pair-stage" in sys.argv else 31
    rows = []
    if "pairs" in sets:
        rows += [cfg("pairs", duel(a, b), pst) for a in range(26) for b in range(26)]
    if "stages" in sets:
        rows += [cfg("stages", duel(a, b), s) for s in STAGES for a, b in SWEEP if s != pst]
    if "levels" in sets:
        rows += [cfg("levels", duel(a, b, l), 32) for l in (1, 3, 5, 7) for a, b in SWEEP[:4]]
    if "items" in sets:
        rows += [cfg("items", duel(a, b), s, items=True) for s in LEGAL for a, b in SWEEP[:4]]
    if "doubles" in sets:
        teams = [((2, 20), (9, 15)), ((19, 14), (12, 0)), ((10, 17), (5, 13)), ((22, 25), (6, 21))]
        for s in LEGAL:
            for (a, b), (c, d) in teams:
                pl = [[a, 9, 0, 0], [b, 9, 0, 0], [c, 9, 1, 1], [d, 9, 1, 1]]
                rows.append(cfg("doubles", pl, s, teams=True))
    with open(out, "w") as f:
        for r in rows:
            f.write(json.dumps(r) + "\n")
    print(f"{len(rows)} configs -> {out}")


if __name__ == "__main__":
    main()
