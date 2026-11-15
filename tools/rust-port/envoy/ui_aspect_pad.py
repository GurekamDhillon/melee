#!/usr/bin/env python3
"""The pad scripts of the UI aspect check (ui_aspect.sh): ui_aspect_pad.py pause|lab|camp OUT.txt MARKS.txt.
MARKS: one `name tick` per line: the ticks at which a surface is on screen (the screenshots)."""
import sys
L, R, D, U, A, START = 0x1, 0x2, 0x4, 0x8, 0x100, 0x1000
lines, marks, t = [], [], 0
def idle(n):
    global t
    lines.append(f"{n} 0000 0 0"); t += n
def press(b, gap=8):
    global t
    lines.append(f"2 {b:04X} 0 0"); t += 2; idle(gap)
def mark(name, wait=14):
    idle(wait); marks.append((name, t)); idle(4)
kind = sys.argv[1]
if kind == "pause":
    idle(240); mark("hud")
    idle(40); press(START, 30); mark("pause_items")
    press(D); press(R); mark("pause_items_card")
    press(R); mark("pause_items_card3")
    press(U)
    for name in ["run", "fight", "options"]:
        press(R); mark("pause_" + name)
    press(D); press(D); press(D); press(D); press(R); mark("pause_options_abandon")
    press(L); press(U); press(U); press(U); press(U)
    press(U); mark("pause_words")
    press(U); mark("pause_combos")
    press(D); press(D); press(D); mark("pause_combos_entry")
    press(U); press(U); press(U); press(U); press(U); press(U)
    press(U); press(D); press(D); press(D); mark("pause_words_scrolled")
    idle(60)
elif kind == "lab":
    idle(300); press(START, 0); idle(200); press(START, 0); idle(60)
    press(D, 20); press(D, 60); press(R, 0); mark("menu_lab_cell", 30)
    press(A, 120); mark("lab_setup")
    press(A, 0); idle(1100); mark("lab_hud")
    press(U, 20); mark("lab_items")
    press(D); press(D); press(D); mark("lab_items_card")
    press(U); press(U); press(U)
    press(R); mark("lab_loadout_empty")
    press(D); press(R); idle(60); mark("lab_loadout_all")   # Add all
    press(U)
    press(R); mark("lab_triggers")
    press(D); press(D); press(R); idle(40); mark("lab_trigger_fired")
    press(U); press(U)
    press(R); mark("lab_effects")
    for _ in range(5): press(D, 4)
    mark("lab_effects_row")
    for _ in range(5): press(U, 4)
    press(R); mark("lab_sandbox")
    press(R); mark("lab_synergy")
    press(L); press(L); press(L); press(L); press(L); mark("lab_items_held")
    press(D); press(D); press(D); press(D); press(D); press(D); press(D); mark("lab_items_scrolled")
    idle(60)
elif kind == "camp":
    # boot -> main menu -> the Envoy row -> the camp; its rows, the fighter picker, How to play, then Start run
    # (the road and the map before fight 1)
    B = 0x200
    idle(300); press(START, 0); idle(200); press(START, 0); idle(60)
    press(D, 20); press(D, 60); mark("menu_envoy_row", 30)
    press(A, 150); mark("camp")
    press(D, 30); mark("camp_fighters_row")
    press(A, 60); mark("picker")
    press(R, 10); press(D, 10); press(A, 40); mark("picker_main_picked")
    press(R, 10); press(A, 40); mark("picker_partner_picked")
    press(A, 40); mark("picker_done")
    idle(40); mark("camp_after_picker")
    press(D, 10); press(D, 10); press(D, 30); mark("camp_how_row")
    press(A, 60); mark("how_to_play")
    press(B, 40)
    press(U, 10); press(U, 10); press(U, 10); press(U, 10); press(U, 30); mark("camp_start_row")
    press(A, 90); mark("camp_road")
    idle(60)
open(sys.argv[2], "w").write(f"# ui_aspect {kind} pad, {t} frames\n" + "\n".join(lines) + "\n")
open(sys.argv[3], "w").write("".join(f"{n} {k}\n" for n, k in marks))
