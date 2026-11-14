#!/usr/bin/env python3
"""The Item Lab check's pad script (lab_check.sh): boot -> main menu -> the Envoy row -> Right (the Item Lab cell)
-> A (the Lab setup) -> A (Start the Lab, a standing dummy on Battlefield) -> the match, then the Lab panel by D-pad
up mid-fight, driven with the D-pad only (as a player would):
  1. fire every item on demand (Triggers page, item rows), no item held;
  2. equip every item one at a time (Items page +1) and fire its trigger at once (Triggers page, trigger row);
  3. fire every trigger with all of them held;
  4. stack 10 of one item (Loadout: Stack 10 of the last item);
  5. switch every modifier off and on again, every category off and on (Effects page);
  6. save preset 1, clear, load preset 1 (Loadout);
  7. slow motion 1/2, 1/4, pause + 5 frame advances, normal (Effects page);
  8. reset (Sandbox), then Add all and a long run of every trigger fired with everything held (the perf part).
The item and trigger counts come from envoy_data (read from the source table), so new items extend the script.
Prints the script on stdout; `--marks` prints the frame of each phase (for the screenshots)."""
import re
import sys
from pathlib import Path

repo = Path(__file__).resolve().parents[3]
src = (repo / "rust/mods/envoy_data/src/items.rs").read_text(encoding="utf-8")
items = re.findall(r'Item \{ id: (IT_[A-Z_]+), key: "([a-z_]+)".*?trigger: (TR_[A-Z_]+)', src)
trig_text = re.search(r"TRIGGER_TEXT: \[&str; (\d+)\]", src)
NTRIG = int(trig_text.group(1))
TR = {m.group(1): int(m.group(2)) for m in re.finditer(r"pub const (TR_[A-Z_]+): u8 = (\d+);", src)}
NITEMS = len(items)

L, R, D, U = 0x1, 0x2, 0x4, 0x8
lines = []
t = 0
marks = []


def idle(n):
    global t
    lines.append(f"{n} 0000 0 0")
    t += n


def press(b, gap=5):
    global t
    lines.append(f"2 {b:04X} 0 0")
    t += 2
    idle(gap)


def mark(name):
    marks.append((name, t))


# pages: Items 0, Loadout 1, Triggers 2, Effects 3, Sandbox 4, Synergy 5
page = 0
row = 0


def to_row(r):
    global row
    while row < r:
        press(D, 3)
        row += 1
    while row > r:
        press(U, 3)
        row -= 1


def to_page(p):
    global page
    to_row(0)
    while page != p:
        press(R if p > page else L, 4)
        page += 1 if p > page else -1


def stream(n):
    """Wait for n commands to reach the sim (26 frames each on the wire)."""
    idle(26 * n + 6)


# boot to the main menu (as camp_pad.txt), the Envoy row, the Item Lab cell, the setup, Start
idle(300); press(0x1000, 0); idle(200); press(0x1000, 0); idle(60)
press(D, 20); press(D, 60)
press(R, 30)
mark("menu_cell")
press(0x0100, 120)
mark("setup")
press(0x0100, 0)
idle(1100)
mark("match")
# the panel, mid-fight
press(U, 20)
mark("page_items")
# 1. every item fired on demand (Triggers page: trigger rows, then one row per item)
to_page(2)
mark("page_triggers")
for i in range(NITEMS):
    to_row(1 + NTRIG + i)
    press(R, 4)
    stream(1)
    idle(30)
mark("fired_all")
# 2. one at a time: Items page +1 (row 2 + i), then its trigger on the Triggers page
for i, (_, key, tr) in enumerate(items):
    to_page(0)
    to_row(2 + i)
    press(R, 4)
    stream(1)
    to_page(2)
    to_row(1 + TR[tr])
    press(R, 4)
    stream(1)
    idle(40)
mark("equipped_all")
# 3. every trigger with everything held
for k in range(NTRIG):
    to_row(1 + k)
    press(R, 4)
    stream(1)
    idle(20)
# 4. stack 10 of the last item chosen
to_page(1)
to_row(4)
press(R, 4)
stream(1)
mark("page_loadout")
# 5. every modifier and category off and on (Effects: time 1, step 2, freeze 3, cats 4-7, modifiers 8..)
nmod = sum(1 for _, _, tr in items if tr == "TR_MODIFIER")
to_page(3)
mark("page_effects")
for k in range(4 + nmod):
    to_row(4 + k)
    press(R, 4)
    stream(1)
    idle(30)
    press(R, 4)
    stream(1)
# 6. preset 1: save, clear, load
to_page(1)
to_row(5)
press(R, 10)
to_row(2)
press(R, 4)
stream(1)
to_row(5)
press(L, 4)
stream(NITEMS + 2)
mark("preset_loaded")
# 7. time: 1/2, 1/4, paused, five frame advances, normal (row 1 time, row 2 step)
to_page(3)
to_row(1)
press(R, 120)
press(R, 240)
press(R, 60)
to_row(2)
for _ in range(5):
    press(R, 20)
to_row(1)
press(R, 30)
# the Sandbox and Synergy pages, for the shots
to_page(4)
mark("page_sandbox")
idle(20)
to_row(4)
press(R, 4)
stream(1)
mark("reset")
to_page(5)
mark("page_synergy")
idle(30)
# 8. everything at once: Add all, then every trigger fired over and over (the perf window)
to_page(1)
to_row(1)
press(R, 4)
stream(1)
to_page(2)
mark("all_at_once")
for rep in range(6):
    for k in range(NTRIG):
        to_row(1 + k)
        press(R, 2)
    to_row(1)
    idle(26 * NTRIG)
to_row(0)
press(U, 30)
idle(600)
mark("end")
if "--marks" in sys.argv:
    for n, f in marks:
        print(f"{n} {f}")
else:
    print(f"# lab_pad.py: {NITEMS} items, {NTRIG} triggers, {t} frames")
    print("\n".join(lines))
