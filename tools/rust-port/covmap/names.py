# SPDX-License-Identifier: MIT
"""Name tables for the coverage map: Slippi external character ids, Slippi internal (FighterKind) ids, StKind
stages (Slippi's stage field), ItemKind (parsed from the decomp header at run time)."""
import os, re

EXT = ["Falcon", "DK", "Fox", "GnW", "Kirby", "Bowser", "Link", "Luigi", "Mario", "Marth", "Mewtwo", "Ness", "Peach",
       "Pikachu", "ICs", "Puff", "Samus", "Yoshi", "Zelda", "Sheik", "Falco", "YLink", "Doc", "Roy", "Pichu", "Ganon"]
INTERNAL = ["Mario", "Fox", "Falcon", "DK", "Kirby", "Bowser", "Link", "Sheik", "Ness", "Peach", "Popo", "Nana",
            "Pikachu", "Samus", "Yoshi", "Puff", "Mewtwo", "Luigi", "Marth", "Zelda", "YLink", "Doc", "Falco", "Pichu",
            "GnW", "Ganon", "Roy"]
STAGES = {2: "Fountain of Dreams", 3: "Pokemon Stadium", 4: "Peach's Castle", 5: "Kongo Jungle", 6: "Brinstar",
          7: "Corneria", 8: "Yoshi's Story", 9: "Onett", 10: "Mute City", 11: "Rainbow Cruise", 12: "Jungle Japes",
          13: "Great Bay", 14: "Hyrule Temple", 15: "Brinstar Depths", 16: "Yoshi's Island", 17: "Green Greens",
          18: "Fourside", 19: "Mushroom Kingdom", 20: "Mushroom Kingdom II", 22: "Venom", 23: "Poke Floats",
          24: "Big Blue", 25: "Icicle Mountain", 27: "Flat Zone", 28: "Dream Land", 29: "Yoshi's Island 64",
          30: "Kongo Jungle 64", 31: "Battlefield", 32: "Final Destination"}
LEGAL = [2, 3, 8, 28, 31, 32]


def items(root):
    """ItemKind value -> name, from the decomp's enum (aliases such as It_Kind_Common_Start = ... are skipped)."""
    t = open(os.path.join(root, "src/melee/it/forward.h"), encoding="utf-8").read()
    body = re.sub(r"//[^\n]*|/\*.*?\*/", "", t[t.index("typedef enum ItemKind {") + 23:t.index("} ItemKind;")], flags=re.S)
    body = re.sub(r"#[^\n]*", "", body)
    val, out, n = {}, {}, 0
    for ent in body.split(","):
        ent = ent.strip()
        if not ent:
            continue
        name, _, rhs = (x.strip() for x in ent.partition("="))
        if rhs:
            try:
                rhs_v = eval(re.sub(r"\b[A-Za-z_]\w*", lambda m: str(val.get(m.group(0), 0)), rhs))
            except SyntaxError:
                continue
            alias = bool(re.search(r"\b[A-Za-z_]\w*", rhs))
            n = rhs_v
            val[name] = n
            if not alias:
                out.setdefault(n, name.split("Kind_", 1)[-1])
        else:
            val[name] = n
            out.setdefault(n, name.split("Kind_", 1)[-1])
        n += 1
    return out
