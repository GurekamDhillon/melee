#!/usr/bin/env python3
"""Generate docs/rust-port/golden/coverage/scenarios.json (the coverage set). Re-run after editing; commit both.
Pads live in pads/ (gen_pads.py). `pad` is relative to tools/rust-port/golden/. Every run quits itself at a fixed frame.
1P step lists come from onep_steps.json (measured: steps that reach a match)."""
import json, os
here = os.path.dirname(os.path.abspath(__file__))
OUT = os.path.join(here, "..", "..", "..", "..", "docs", "rust-port", "golden", "coverage", "scenarios.json")

CK = ["captain", "dk", "fox", "gaw", "kirby", "bowser", "link", "luigi", "mario", "marth", "mewtwo", "ness", "peach",
      "pikachu", "icies", "jigglypuff", "samus", "yoshi", "zelda", "sheik", "falco", "younglink", "drmario", "roy",
      "pichu", "ganondorf"]
assert len(CK) == 26
# external stage ids (gw_sl_stage_names in pc/platform/gw_runtime.c): 2..32 = every retail VS stage
STAGES = {2: "fountain", 3: "stadium", 4: "castle", 5: "jungle", 6: "brinstar", 7: "corneria", 8: "yoshistory",
          9: "onett", 10: "mutecity", 11: "rainbowcruise", 12: "garden", 13: "greatbay", 14: "temple",
          15: "kraid", 16: "yoshiisland", 17: "greens", 18: "fourside", 19: "mushroomkingdom",
          20: "mushroomkingdom2", 21: "akaneia", 22: "venom", 23: "pokefloats", 24: "bigblue", 25: "icemt",
          26: "icetop", 27: "flatzone", 28: "dreamland64", 29: "yoshistory64", 30: "kongojungle64",
          31: "battlefield", 32: "finaldestination"}
LEGAL = [32, 31, 8, 3, 28, 2]  # FD BF YS PS DL64 FoD
IDLE = "coverage/pads/idle_3600.lua"
sc = {}


def vs(name, group, p1, p2, stage, pad, frames, items="off", extra="", note="", p3="none", p4="none", mode="vs"):
    scene = "mode=%s;at=match;p1=%s;p2=%s;p3=%s;p4=%s;stage=ext:%d;match=time;minutes=0;items=%s%s" % (
        mode, p1, p2, p3, p4, stage, items, (";" + extra) if extra else "")
    sc[name] = {"group": group, "scene": scene, "pad": pad, "frames": frames, "kind": "vs", "note": note}


# 1. every character, CPU 9 vs a different character
for i, n in enumerate(CK):
    j = (i + 9) % 26
    vs("char_%s" % n, "chars", "ck:%d/cpu9" % i, "ck:%d/cpu9" % j, LEGAL[i % 6], IDLE, 3600,
       note="CPU9 %s vs %s" % (n, CK[j]))
vs("zelda_sheik_transform", "chars", "ck:18/human", "ck:2/cpu9", 31, "coverage/pads/zelda_sheik.lua", 2400,
   note="Zelda Down+B -> Sheik, fight, Down+B -> Zelda, fight; Fox CPU9")
vs("kirby_copy_a", "chars", "ck:4/human", "ck:2/idle", 32, "coverage/pads/kirby_copy_a.lua", 2400, p3="ck:5/idle",
   note="Kirby inhales idle Fox and Bowser to the right (copy abilities)")
vs("kirby_copy_b", "chars", "ck:4/human", "ck:10/idle", 8, "coverage/pads/kirby_copy_b.lua", 3000, p3="ck:25/idle",
   p4="ck:20/idle", note="Kirby inhales idle Mewtwo, Ganondorf, Falco to the left (copy abilities)")
vs("ic_vs_fox_human", "chars", "ck:14/human", "ck:2/human", 31, "coverage/pads/rand_s1.lua", 1800,
   note="Ice Climbers human pad")
vs("sheik_cpu_vs_zelda_cpu", "chars", "ck:19/cpu9", "ck:18/cpu9", 3, IDLE, 3600)

# 2. every VS stage
for k, (e, n) in enumerate(sorted(STAGES.items())):
    a = (k * 5) % 26
    b = (a + 11) % 26
    vs("stage_%02d_%s" % (e, n), "stages", "ck:%d/cpu9" % a, "ck:%d/cpu9" % b, e, IDLE, 3600,
       note="%s vs %s" % (CK[a], CK[b]))

# 3. items
for k, e in enumerate([32, 31, 8, 3, 28]):
    vs("items_hi_%s" % STAGES[e], "items", "ck:%d/cpu9" % ((k * 7 + 1) % 26), "ck:%d/cpu9" % ((k * 7 + 14) % 26), e,
       IDLE, 3600, items="4", note="item frequency 4 (high)")
vs("items_hi_4p_ffa", "items", "ck:2/cpu9", "ck:9/cpu9", 31, IDLE, 3600, items="4", p3="ck:12/cpu9", p4="ck:5/cpu9",
   note="4-player FFA, item frequency 4")
vs("items_low_fd", "items", "ck:8/cpu9", "ck:7/cpu9", 32, IDLE, 3600, items="1", note="item frequency 1")
vs("items_hi_human_bf", "items", "ck:0/human", "ck:1/human", 31, "coverage/pads/rand_s2.lua", 1800, items="4",
   note="items high, human pads")
vs("items_off_4p_ps", "items", "ck:2/cpu9", "ck:9/cpu9", 3, IDLE, 3600, items="off", p3="ck:12/cpu9", p4="ck:5/cpu9",
   note="items explicitly off, 4 players")
vs("items_off_fd", "items", "ck:20/cpu9", "ck:16/cpu9", 32, IDLE, 3600, items="off")

# 4. CPU levels
for lv in (1, 5, 9):
    for tag, a, b, st in (("fox_falco", 2, 20, 32), ("marth_sheik", 9, 19, 31), ("ganon_peach", 25, 12, 8),
                          ("puff_link", 15, 6, 3)):
        vs("cpulv%d_%s" % (lv, tag), "cpulevels", "ck:%d/cpu%d" % (a, lv), "ck:%d/cpu%d" % (b, lv), st, IDLE, 3600)
vs("cpu_mixed_1_9", "cpulevels", "ck:2/cpu1", "ck:9/cpu9", 32, IDLE, 3600)
vs("cpu_ffa4_levels", "cpulevels", "ck:2/cpu1", "ck:9/cpu3", 31, IDLE, 3600, p3="ck:12/cpu5", p4="ck:5/cpu9",
   note="4 CPUs, levels 1/3/5/9")
vs("cpu_teams4", "cpulevels", "ck:2/cpu9/team0", "ck:9/cpu9/team0", 8, IDLE, 3600, extra="teams=1",
   p3="ck:12/cpu9/team1", p4="ck:5/cpu9/team1", note="2v2 teams")
vs("cpu_stock2_fd", "cpulevels", "ck:2/cpu9", "ck:20/cpu9", 32, IDLE, 3600, extra="match=stock;stocks=2",
   note="stock match (the trailing match=stock overrides match=time; the matched game may end before 3600)")
vs("cpu_stock1_bf", "cpulevels", "ck:5/cpu9", "ck:25/cpu9", 31, IDLE, 3600, extra="match=stock;stocks=1")

# 5. human-style pad matches
pairs = [(0, 2, 32), (9, 19, 31), (5, 25, 8), (14, 15, 3), (12, 16, 28), (7, 22, 2)]
for k, (a, b, st) in enumerate(pairs):
    vs("pad_%s_%s_%s" % (CK[a], CK[b], STAGES[st]), "pads", "ck:%d/human" % a, "ck:%d/human" % b, st,
       "coverage/pads/rand_s%d.lua" % (k + 1), 1800)
for k, (a, b) in enumerate([(2, 9), (13, 24), (23, 21)]):
    vs("padcpu_%s_vs_cpu_%s" % (CK[a], CK[b]), "pads", "ck:%d/human" % a, "ck:%d/cpu9" % b, 31,
       "coverage/pads/rand_s%d.lua" % (k + 4), 1800, note="port 1 human pad; port 2 is a CPU slot")

# 7. Target Test (every character) and the 1P modes (step = stage index, difficulty 0..4)
def onep(name, mode, ck, step, diff, pad, frames):
    sc[name] = {"group": mode, "scene": "mode=%s;p1=ck:%d;step=%d;difficulty=%d" % (mode, ck, step, diff), "pad": pad,
                "frames": frames, "kind": "1p", "note": "random human pad; screens advanced by A/Start taps"}


for i, n in enumerate(CK):
    sc["targettest_%s" % n] = {"group": "targettest", "scene": "mode=tt;p1=ck:%d" % i,
                               "pad": "coverage/pads/randabs_s%d_1800.lua" % (i % 4 + 1), "frames": 1800, "kind": "1p",
                               "note": "Target Test, random human pad; hash rows end when the stage ends"}
sp = os.path.join(here, "onep_steps.json")
steps_by_mode = json.load(open(sp)) if os.path.exists(sp) else {}
for mode, steps in steps_by_mode.items():
    for k, s in enumerate(steps):
        onep("%s_step%02d" % (mode, s), mode, (k * 3 + 2) % 26, s, k % 5, "coverage/pads/randabs_s%d_3600.lua" % (k % 4 + 1), 3600)

# blocked: documented, never run by --all
BLOCKED = {
    "homerun_contest": "GM_HOME_RUN_CONTEST (0x20): no MELEE_SCENE keyword (gw_sl_modes in pc/platform/gw_runtime.c); retail menus only",
    "multiman_10": "GM_10MAN_VS (0x21): no MELEE_SCENE keyword; retail menus only",
    "multiman_100": "GM_100MAN_VS (0x22): no MELEE_SCENE keyword; retail menus only",
    "multiman_3min": "GM_3MIN_VS (0x23): no MELEE_SCENE keyword; retail menus only",
    "multiman_15min": "GM_15MIN_VS (0x24): no MELEE_SCENE keyword; retail menus only",
    "multiman_endless": "GM_ENDLESS_VS (0x25): no MELEE_SCENE keyword; retail menus only",
    "multiman_cruel": "GM_CRUEL_VS (0x26): no MELEE_SCENE keyword; retail menus only",
    "event_match": "GM_EVENT (0x2B): no MELEE_SCENE keyword; needs the event select screen",
}
for n, why in BLOCKED.items():
    sc[n] = {"group": "blocked", "status": "blocked", "scene": "", "pad": "", "frames": 0, "kind": "menu", "note": why}

d = {"defaults": {"seed": 12345, "max_seconds": 240}, "scenarios": sc, "batches": {}}
# 6 batches of roughly equal run time (weight = frames + fixed per-run cost); already-recorded scenarios are left out
outdir = os.path.dirname(OUT)
todo = [n for n, v in sc.items() if v.get("status") != "blocked" and not os.path.exists(os.path.join(outdir, n + ".csv"))]
todo.sort(key=lambda n: -(sc[n]["frames"] + 700))
bins = [[0, []] for _ in range(6)]
for n in todo:
    b = min(bins, key=lambda x: x[0])
    b[0] += sc[n]["frames"] + 700
    b[1].append(n)
d["batches"] = {str(i + 1): sorted(b[1]) for i, b in enumerate(bins)}
json.dump(d, open(OUT, "w"), indent=1)
print(len(sc), "scenarios;", len(todo), "to record; batch weights", [b[0] for b in bins])
