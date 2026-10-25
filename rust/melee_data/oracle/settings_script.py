"""Deterministic random operation script for the settings oracle."""
import random
r = random.Random(20261024)
H = lambda s: s.encode().hex() if s else "-"
keys = ["name", "server", "delay", "widescreen", "ucf", "ctl00_id", "ctl01_p0_map", "ctl01_p1_name",
        "fc_tapjump", "gdlp_record", "zz", "a" * 31, "b" * 32, "c" * 40, "k=1", "UPPER", "stage_mode"]
vals = ["", "1", "0", "42", "-7", "  pad  ", "GD", "127.0.0.1:7777", "x=y=z", "tab\there", "#hash",
        "9" * 20, "-" + "9" * 20, "12abc", " 33", "v" * 700, "+5", "0x10"]
out = []
def op(s): out.append(s)
op("FILE")
for i in range(1500):
    c = r.random()
    k = r.choice(keys)
    if c < 0.25: op(f"S {k} {H(r.choice(vals))}")
    elif c < 0.35: op(f"I {k} {r.choice([0,1,-5,2147483647,-2147483648,300])}")
    elif c < 0.55: op(f"GS {k} {r.choice([0,1,2,5,16,64,600])} {r.choice(['~', H('dflt'), '-'])}")
    elif c < 0.7: op(f"GI {k} {r.choice([0,5,-1])}")
    elif c < 0.75: op(f"R {r.choice(['ctl', 'ctl01', 'fc_', 'z', '', 'a'*31, 'a'*32, 'nomatch'])}")
    elif c < 0.8: op(f"SUM {r.choice([1,8,20,64,400,5000])}")
    elif c < 0.86:
        e = "MELEE_" + k.upper()
        op(f"ENV {e} {H(r.choice(['5', '', 'envval', ' 12', '-3', 'z']))}")
    elif c < 0.88: op(f"UNENV MELEE_{k.upper()}")
    elif c < 0.93: op("RELOAD")
    elif c < 0.95:
        op("BADPATH"); op(f"S {k} {H('failed')}"); op(f"I {k} 9"); op(f"R {r.choice(['ctl','z'])}"); op("GOODPATH")
    elif c < 0.97:
        # raw file with awkward content
        lines = []
        for _ in range(r.randint(0, 12)):
            lines.append(r.choice(["a=1", "  b = 2 ", "# c=3", "", "noeq", "=empty", "d=" + "x" * r.choice([5, 520, 560, 1200]),
                                   "e=1\r", "name=Bob", "server=h", "\tf=g"]))
        sep = r.choice(["\n", "\r\n"])
        data = sep.join(lines) + r.choice(["", sep])
        if r.random() < 0.2: data += "\x1a" + "late=1" + sep
        op("RAW " + (data.encode("latin1").hex() or "-"))
        op("FILE")
for i in range(520):
    op(f"S cap{i} {H(str(i))}")
op("S overflow 31")
op("RELOAD")
op("SUM 100000")
print("\n".join(out))
