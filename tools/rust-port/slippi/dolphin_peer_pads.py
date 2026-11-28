#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Pad scripts (MELEE_PAD_SCRIPT) that take our build from its main menu to the Unranked online select, put the CSS
hand on one character and lock in, for dolphin_peer.sh. No OS input: these are the engine's own scripted pads.

    python -I dolphin_peer_pads.py CHAR OUTDIR     -> OUTDIR/pads_g1.txt (boot -> Unranked -> pick -> Start)
                                                      OUTDIR/pads_gn.txt (a set's next select: Start locks in again)

The hand starts at (-31, -21.5) on our online select and moves 1.24 units a frame at full deflection on either axis
(measured with MELEE_SLIPPI_CSS_TRACE, 976a7b995). Targets are the icon centres of dolphin_drive.py's CSS table (the same
retail coordinates), so the hand lands within 0.62 of a centre: well inside an icon (about 7 x 7).
A set's later select keeps the fighter (Slippi's CSS does too), so its script only waits for the select and presses
Start; dolphin_peer.sh checks the agreed scene of every game, so a wrong pick shows up in the summary.
"""
import importlib.util
import os
import sys

START = (-31.0, -21.5)
SPEED = 1.24
# the main menu -> 1P -> Online -> Unranked (tools/rust-port/menus/slippi_online_unranked_css_1start.txt)
NAV = """# dolphin_peer_pads.py: main menu -> 1P -> Online -> Unranked
60 0000 0 0
2 0100 0 0
40 0000 0 0
6 0000 0 -100
20 0000 0 0
6 0000 0 -100
20 0000 0 0
2 0100 0 0
60 0000 0 0
60 0000 0 0
2 0100 0 0
60 0000 0 0
150 0000 0 0
"""


def css_table():
    here = os.path.dirname(os.path.abspath(__file__))
    spec = importlib.util.spec_from_file_location("dolphin_drive", os.path.join(here, "dolphin_drive.py"))
    m = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(m)
    return m.CSS


def main():
    if len(sys.argv) != 3:
        raise SystemExit(__doc__)
    name, out = sys.argv[1], sys.argv[2]
    css = css_table()
    if name not in css:
        raise SystemExit(f"dolphin_peer_pads: unknown character {name} (one of {' '.join(sorted(css))})")
    tx, ty = css[name]
    nx = round((tx - START[0]) / SPEED)
    ny = round((ty - START[1]) / SPEED)
    lines = [NAV.rstrip("\n"), f"# {name}: hand to {tx},{ty}"]
    if nx:
        lines.append(f"{abs(nx)} 0000 {100 if nx > 0 else -100} 0")
        lines.append("6 0000 0 0")
    if ny:
        lines.append(f"{abs(ny)} 0000 0 {100 if ny > 0 else -100}")
    lines += ["10 0000 0 0", "2 0100 0 0", "30 0000 0 0", "2 1000 0 0", "30000 0000 0 0"]
    os.makedirs(out, exist_ok=True)
    with open(os.path.join(out, "pads_g1.txt"), "w", newline="\n") as f:
        f.write("\n".join(lines) + "\n")
    with open(os.path.join(out, "pads_gn.txt"), "w", newline="\n") as f:
        f.write("# dolphin_peer_pads.py: a set's next select (the fighter is kept): wait for it, Start locks in\n"
                "until 8\n90 0000 0 0\n2 1000 0 0\n30000 0000 0 0\n")


if __name__ == "__main__":
    main()
