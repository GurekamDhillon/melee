#!/usr/bin/env python3
"""The LAB's features in real play (lab_flow.sh): the retail flow's LAB match (MELEE_SCENE mode=lab, the lab mod
loaded), P1 a scripted pad (MELEE_PAD_SCRIPT), P2 the dummy: a CPU slot (`cpu`) or a human slot nobody holds
(`human`). The commands are the ones the lab menu's gd_lab_* imports queue (MELEE_LAB_CMDS, at command-clock frames;
the pad script's frame k is clock k), and every check reads the fighters from MELEE_LAB_TRACE.

  lab_flow.py gen DIR            write DIR/pads.txt and DIR/cmds.txt
  lab_flow.py check LOG KIND     check a run's log (KIND cpu|human); exit 0 when every feature took effect
"""
import re
import sys

# P1's pad: (from clock, frames, buttons, stick x, stick y)
P1 = [
    (640, 60, "0000", 127, 0),    # walk right: the recording
    (1330, 3, "0100", 127, 0),    # forward smash: DI / SDI / tech
    (1530, 3, "0100", 127, 0),    # forward smash: missed tech, getup
    (1790, 3, "0100", 0, 0),      # jab into the shield: block, out of shield
]


def f(x):
    return "f%s" % x


# (clock, op, port, a, b, c, d): what menu.rs's send_dummy / toggle_record / play / play_all / reset_pos / save_slot /
# load_slot and lib.rs's freeze / step / rewind send through the gd_lab_* imports
CMDS = [
    (5, 3, 0, 300),                                   # ring 300 (rewind)
    (100, 13, 1, 1), (200, 13, 1, 2), (300, 13, 1, 3), (400, 13, 1, 4),  # Control: stand, crouch, jump, shield
    (500, 13, 1, 5, 9),                               # Control: CPU level 9
    (600, 13, 1, 1),                                  # stand
    (620, 8, 0, 1, 1), (720, 9, 0, 0, 1),             # record P1 (mask 1) as rec 1, stop
    (740, 16, 0, 0, f(-60), f(10)), (740, 16, 1, 0, f(20), f(10)),
    (800, 10, 0, 2, 1, 0),                            # play rec 1 on P2
    (900, 16, 1, 0, f(20), f(10)), (905, 10, 0, 2, 1, 2),  # mirror
    (1000, 11, 0, 15),                                # stop playback
    (1005, 13, 1, 6, 1),                              # Control: recording (rec 1)
    (1100, 13, 1, 1), (1101, 16, 1, 0, f(20), f(10)),
    (1105, 17, 1, 0, 0xFFFFFF01, 0xFFFFFFFF, 1),      # play all (one slot)
    (1200, 11, 0, 15), (1201, 13, 1, 1), (1202, 16, 1, 0, f(20), f(10)),
    (1250, 15, 1, 0, f(60)),                          # percent
    (1290, 14, 1, 0, 7), (1290, 14, 1, 1, 7, 2), (1290, 14, 1, 2, 1), (1291, 15, 1, 0, f(10)),  # DI down, SDI down every 2, tech in place
    (1300, 16, 0, 0, f(0), f(10)), (1300, 16, 1, 0, f(14), f(10)),
    (1490, 14, 1, 0, 0), (1490, 14, 1, 1, 0), (1490, 14, 1, 2, 4), (1490, 14, 1, 3, 1, 10),  # tech miss, getup stand
    (1495, 15, 1, 0, f(10)),
    (1500, 16, 0, 0, f(0), f(10)), (1500, 16, 1, 0, f(14), f(10)),
    (1740, 14, 1, 2, 0), (1740, 14, 1, 3, 0), (1740, 14, 1, 5, 1), (1740, 14, 1, 6, 1, 0),  # block all, OoS jump
    (1745, 15, 1, 0, f(0)),
    (1750, 16, 0, 0, f(0), f(10)), (1750, 16, 1, 0, f(12), f(10)),
    (1950, 14, 1, 5, 0), (1950, 14, 1, 6, 0), (1950, 14, 1, 4, 1, 10),  # ledge: stand
    (1955, 16, 0, 0, f(-30), f(10)), (1955, 16, 1, 0, f(90), f(5)),
    (2150, 16, 1, 0, f(20), f(10)), (2190, 13, 1, 3),  # jump (a moving, deterministic dummy)
    (2200, 1, 0, 0),                                  # save slot 0
    (2260, 2, 0, 0),                                  # load slot 0 (running)
    (2300, 5, 0, 1), (2310, 4, 0, 0, 30), (2320, 6, 0, 0, 1),  # freeze, rewind 30, step 1
    (2340, 2, 0, 0),                                  # load slot 0 while frozen: drawn at once
    (2345, 16, 1, 0, f(30), f(10)),                   # position while frozen: drawn at once
    (2346, 2, 0, 0), (2350, 7, 0), (2350, 5, 0, 0),   # load again, go live, resume
]


def gen(d):
    out, t = [], 0
    for at, n, b, x, y in P1:
        if at > t:
            out.append("%d 0000 0 0" % (at - t))
        out.append("%d %s %d %d" % (n, b, x, y))
        t = at + n
    out.append("9000 0000 0 0")
    open(d + "/pads.txt", "w", newline="\n").write("\n".join(out) + "\n")
    cmds = ";".join("%d:%s" % (c[0], ",".join(str(v) for v in c[1:])) for c in CMDS)
    open(d + "/cmds.txt", "w", newline="\n").write(cmds)


LINE = re.compile(r"labtrace c=(\d+) f=(-?\d+) paused=(\d) drew=(\d) P(\d) motion=(-?\d+) x=(\S+) y=(\S+) pct=(\S+) "
                  r"cpukind=(\d+) air=(\d) btn=(\w+) sx=(-?\d+) sy=(-?\d+) play=(\d)")


def check(log, kind):
    tr = {}
    text = open(log, errors="replace").read()
    for m in LINE.finditer(text):
        c, fr, pa, dr, p, mo, x, y, pct, ck, air, btn, sx, sy, pl = m.groups()
        tr[(int(c), int(p))] = dict(f=int(fr), paused=int(pa), drew=int(dr), motion=int(mo), x=float(x), y=float(y),
                                    pct=float(pct), cpukind=int(ck), air=int(air), btn=int(btn, 16), sx=int(sx),
                                    sy=int(sy), play=int(pl))
    if not tr:
        print("FAIL no trace in %s" % log)
        return 1
    cpu = kind == "cpu"
    res = []

    def rng(a, b, p=2):
        return [tr[(c, p)] for c in range(a, b) if (c, p) in tr]

    def ok(name, cond, ev):
        res.append((name, bool(cond), ev))

    def motions(a, b, p=2):
        return sorted({r["motion"] for r in rng(a, b, p)})

    def span(rows):
        xs = [r["x"] for r in rows] or [0]
        return max(xs) - min(xs), min(xs), max(xs)

    if cpu:
        s = span(rng(20, 100))
        ok("baseline: the CPU moves (AI on)", s[0] > 1 or len(motions(20, 100)) > 2, "x %.1f..%.1f motions %s" % (s[1], s[2], motions(20, 100)))
    st = rng(140, 200)
    ok("Control Stand", st and all(r["motion"] == 14 for r in st) and (not cpu or all(r["cpukind"] == 5 for r in st)),
       "motions %s AI kind %s" % (motions(140, 200), sorted({r["cpukind"] for r in st})))
    ok("Control Crouch", 40 in motions(230, 300) and all(r["motion"] in (39, 40) for r in rng(240, 300)), "motions %s" % motions(230, 300))
    ok("Control Jump", any(r["air"] for r in rng(300, 400)), "airborne frames %d" % sum(r["air"] for r in rng(300, 400)))
    ok("Control Shield", 179 in motions(420, 500), "motions %s" % motions(420, 500))
    if cpu:
        lv = rng(520, 600)
        ok("Control CPU level (AI back)", lv and all(r["cpukind"] != 5 for r in lv) and len(motions(520, 600)) > 2,
           "AI kind %s motions %s" % (sorted({r["cpukind"] for r in lv}), motions(520, 600)))
    rec = re.search(r"recording 0x1 stopped, (\d+) frames", text)
    ok("record P1", rec and int(rec.group(1)) == 100, rec.group(0) if rec else "no stop line")
    a, b = tr.get((810, 2)), tr.get((895, 2))
    ok("play slot on the dummy", a and b and b["x"] - a["x"] > 10 and a["play"], "x %.1f -> %.1f" % (a["x"], b["x"]) if a and b else "-")
    a, b = tr.get((915, 2)), tr.get((995, 2))
    ok("mirror", a and b and a["x"] - b["x"] > 10, "x %.1f -> %.1f" % (a["x"], b["x"]) if a and b else "-")
    pl = rng(1010, 1100)
    s = span(pl)
    ok("Control Recording", any(r["play"] for r in pl) and s[0] > 10, "x %.1f..%.1f" % (s[1], s[2]))
    pa = rng(1110, 1200)
    s = span(pa)
    ok("play all", any(r["play"] for r in pa) and s[0] > 10, "x %.1f..%.1f" % (s[1], s[2]))
    r = tr.get((1260, 2))
    ok("percent", r and abs(r["pct"] - 60) < 0.1, "pct %s" % (r and r["pct"]))
    r = tr.get((1302, 2))
    ok("position", r and abs(r["x"] - 14) < 0.5, "x %s" % (r and r["x"]))
    hl = rng(1330, 1360)
    ok("DI down (last hitlag frame)", any(r["sy"] == -127 for r in hl), "stick y %s" % [r["sy"] for r in hl[16:30]])
    ok("SDI down (pulses in hitlag)", sum(1 for i in range(len(hl) - 1) if hl[i]["sy"] != hl[i + 1]["sy"]) >= 2, "")
    ok("tech in place", 199 in motions(1335, 1490), "motions %s" % motions(1335, 1490))
    mg = motions(1535, 1740)
    ok("missed tech + getup stand", (184 in mg or 192 in mg) and (186 in mg or 194 in mg), "motions %s" % mg)
    mb = motions(1790, 1830)
    ok("block (shield on the jab)", 179 in mb or 181 in mb, "motions %s" % mb)
    mo = motions(1795, 1900)
    ok("out of shield jump", 24 in mo and any(r["air"] for r in rng(1795, 1900)), "motions %s" % mo)
    ml = motions(1960, 2150)
    ok("ledge: stand", 253 in ml and (254 in ml or 255 in ml), "motions %s" % ml)
    s = re.search(r"lab: frame (\d+): saved slot 0", text)
    l = re.search(r"lab: loaded slot 0 \(frame (\d+)\)", text)
    # a command queued at clock k applies before the frame traced as c = k + 1
    a, b = tr.get((2201, 2)), tr.get((2261, 2))
    ok("slot save / load", s and l and a and b and b["f"] == a["f"] and abs(b["x"] - a["x"]) < 0.01 and abs(b["y"] - a["y"]) < 0.01,
       "saved at f %s, after load f %s, x %s / %s" % (a and a["f"], b and b["f"], a and a["x"], b and b["x"]))
    a, b, c = tr.get((2310, 2)), tr.get((2311, 2)), tr.get((2321, 2))
    ok("freeze / rewind 30 / step 1", a and b and c and a["paused"] and b["f"] == a["f"] - 30 and c["f"] == b["f"] + 1,
       "f %s -> %s -> %s" % (a and a["f"], b and b["f"], c and c["f"]))
    a, b, c = tr.get((2340, 2)), tr.get((2341, 2)), tr.get((2346, 2))
    ok("frozen load / position drawn at once", a and b and c and b["paused"] and not a["drew"] and b["drew"] and c["drew"],
       "drew: idle %s, load %s, position %s" % (a and a["drew"], b and b["drew"], c and c["drew"]))
    # the frozen redraw leaves no trace: the frames after the frozen load equal the frames after the running load
    r1 = [(r["f"], r["x"], r["y"], r["motion"]) for r in rng(2262, 2292)]
    r2 = [(r["f"], r["x"], r["y"], r["motion"]) for r in rng(2352, 2382)]
    ok("frozen redraw is state-neutral", r1 and r1 == r2, "30 frames after each load: %s" % ("equal" if r1 == r2 else "differ"))
    bad = 0
    for n, g, ev in res:
        print("%s %-38s %s" % ("ok  " if g else "FAIL", n, ev))
        bad += not g
    print("%s: %d/%d LAB features took effect (P2 %s dummy)" % ("OK" if not bad else "FAILED", len(res) - bad, len(res), kind))
    return 1 if bad else 0


if __name__ == "__main__":
    if sys.argv[1] == "gen":
        gen(sys.argv[2])
    else:
        sys.exit(check(sys.argv[2], sys.argv[3]))
