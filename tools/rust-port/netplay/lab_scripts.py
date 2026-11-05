#!/usr/bin/env python3
"""lab_pair.sh with LAB_MOD=lab: the --np-css pad scripts with LAB actions from both peers inserted into the first
fight (the lab mod's controls, rust/mods/lab): the host saves, freezes, rewinds 5 frames and resumes; the guest
freezes, rewinds 10 + 3 frames, takes over, and later loads the host's slot. Then the host holds START for the full
menu and sets its dummy (P2: Block all, Tech random; lab commands both peers apply), the guest opens its own menu
and sets P1's DI away. Writes np_lab_host.txt / np_lab_guest.txt
next to the css scripts in <dir>."""
import os, sys

d = sys.argv[1]
# (frame, buttons[, frames held]): D-down 4 = Block, D-right 2, D-down 4 x4 = Tech, D-right x5 = Random, B closes
MENU_HOST = [(2400, "1000", 40)] + [(2450 + 8 * i, "0004") for i in range(2)] + [(2470, "0002")] + [(2480 + 8 * i, "0004") for i in range(4)]     + [(2520 + 8 * i, "0002") for i in range(5)] + [(2570, "0200")]
MENU_GUEST = [(2700, "1000", 40)] + [(2750 + 8 * i, "0004") for i in range(7)] + [(2810 + 8 * i, "0002") for i in range(3)] + [(2840, "0200")]
HOST = [(900, "0004"), (1100, "1000")] + [(1110 + 8 * i, "0001") for i in range(5)] + [(1180, "0200"), (1500, "0018")] + MENU_HOST
GUEST = [(1700, "1000"), (1712, "0040")] + [(1724 + 8 * i, "0001") for i in range(3)] + [(1760, "0100"), (2100, "0008")] + MENU_GUEST


def patch(src, dst, marker, acts):
    lines = open(os.path.join(d, src)).read().split("\n")
    i = next(k for k, l in enumerate(lines) if l.startswith(marker))
    # the fight starts after the marker's segments (host: 3 SSS segments; guest: the wait)
    i += 4 if "SSS" in marker else 2
    head, segs = lines[:i], [l.split() for l in lines[i:] if l and not l.startswith("#")]
    flat, t = [], 0
    for s in segs:
        # the fight's own D-pad presses would be lab actions (save / load / steps): masked
        flat.append([t, int(s[0]), f"{int(s[1], 16) & ~0x000F:04x}", s[2], s[3]])
        t += int(s[0])
    # while frozen (and around each action) the fight's own buttons would be lab controls too: blank the fight there
    quiet = []
    for k, (at, *_) in enumerate(acts):
        nxt = acts[k + 1][0] if k + 1 < len(acts) else at + 3
        quiet.append((at - 10, (nxt if nxt - at < 100 else at) + 13))
    def blank(st, n):
        # split [st, st+n) at the quiet windows' edges; segments inside one are neutral
        cuts = sorted({st, st + n} | {e for q in quiet for e in q if st < e < st + n})
        return [(a, b - a, any(q0 <= a < q1 for q0, q1 in quiet)) for a, b in zip(cuts, cuts[1:])]
    flat2 = []
    for st, n, b, x, y in flat:
        for a, m, q in blank(st, n):
            flat2.append([a, m, "0000" if q else b, "0" if q else x, "0" if q else y])
    flat = flat2
    out, t, ai, cons = [], 0, 0, 0
    for st, n, b, x, y in flat:
        # frames a held action already covered
        if st + n <= cons:
            continue
        if st < cons:
            n -= cons - st
            st = cons
        while ai < len(acts) and st <= acts[ai][0] < st + n:
            at, btn, *hold = acts[ai]
            dur = hold[0] if hold else 3
            pre = at - st
            if pre:
                out.append(f"{pre} {b} {x} {y}")
            out.append(f"{dur} {btn} 0 0")
            n -= pre + dur
            st = at + dur
            cons = st
            ai += 1
            if n <= 0:
                break
        if n > 0:
            out.append(f"{n} {b} {x} {y}")
    open(os.path.join(d, dst), "w", newline="\n").write("\n".join(head + out) + "\n")


patch("np_css_host.txt", "np_lab_host.txt", "# SSS", HOST)
patch("np_css_guest.txt", "np_lab_guest.txt", "# wait through", GUEST)
print("written")
