#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Post-hoc input diff of two Slippi replays of the same online game (Slippi Dolphin's .slp and the engine's
`play --slp`): per frame and port, the pre-frame event (0x37) each side's game ran with, the last copy of a frame
(a rolled-back frame is written again). Nothing is logged during the match, so the run's timing is unchanged.
    python -I slp_inputs_diff.py A.slp B.slp [--shift N] [--show K]
Without --shift the frame offset is the one whose post-frame states agree most. Prints common port-frames, the first frames whose inputs (physical buttons, stick, c-stick, analog triggers) or
pre-frame random seed differ, and the first post-frame (0x38) state difference for context."""
import struct
import sys

sys.path.insert(0, __import__("os").path.dirname(__file__))
from slp_rollbacks import events  # noqa: E402


def load(path):
    d = open(path, "rb").read()
    k = d.find(b"raw[$U#l")
    ln = struct.unpack(">I", d[k + 8:k + 12])[0]
    pre, post = {}, {}
    for c, p in events(d[k + 12:k + 12 + ln]):
        if c == 0x37 and p[5] == 0:
            f, port = struct.unpack(">i", p[0:4])[0], p[4]
            seed = struct.unpack(">I", p[6:10])[0]
            jx, jy, cx, cy, trig = struct.unpack(">fffff", p[24:44])
            phys = struct.unpack(">H", p[48:50])[0]
            pl, pr = struct.unpack(">ff", p[50:58])
            pre[(f, port)] = {"buttons": f"{phys:04x}", "stick": (round(jx, 4), round(jy, 4)),
                              "cstick": (round(cx, 4), round(cy, 4)), "LR": (round(pl, 3), round(pr, 3)),
                              "seed": f"{seed:08x}"}
        elif c == 0x38 and p[5] == 0:
            f, port = struct.unpack(">i", p[0:4])[0], p[4]
            act = struct.unpack(">H", p[7:9])[0]
            x, y, fc, pct = struct.unpack(">ffff", p[9:25])
            post[(f, port)] = (act, round(x, 3), round(y, 3), fc, round(pct, 2))
    return pre, post


def main():
    a_pre, a_post = load(sys.argv[1])
    b_pre, b_post = load(sys.argv[2])
    if "--shift" in sys.argv:
        shift = int(sys.argv[sys.argv.index("--shift") + 1])
    else:
        # the frame numbering of the two files can differ by a frame (the engine's .slp vs Dolphin's): take the shift
        # whose post-frame states agree most
        def agree(k):
            return sum(1 for (f, port), v in b_post.items() if a_post.get((f + k, port)) == v)
        shift = max(range(-3, 4), key=agree)
    print(f"shift {shift} (B frame f = A frame f{shift:+d})")
    show = int(sys.argv[sys.argv.index("--show") + 1]) if "--show" in sys.argv else 6
    common, bad = 0, []
    for (f, port), v in sorted(b_pre.items()):
        w = a_pre.get((f + shift, port))
        if w is None:
            continue
        common += 1
        diff = {k: (w[k], v[k]) for k in v if w[k] != v[k]}
        if diff:
            bad.append((f, port, diff))
    # before frame -39 (the first actionable frame, Slippi's counting) the engine's .slp records its own stick a frame
    # or two early; the game's state is unaffected (the post-frame events agree), so those are counted apart
    early = [b for b in bad if b[0] < -39]
    bad = [b for b in bad if b[0] >= -39]
    print(f"pre-frame: common port-frames {common}, differing {len(bad)} from frame -39 ({len(early)} before)")
    for f, port, diff in bad[:show]:
        print(f"  frame {f} port {port}: " + "; ".join(f"{k} A={x} B={y}" for k, (x, y) in diff.items()))
    pc = [(f, port) for (f, port), v in sorted(b_post.items()) if a_post.get((f + shift, port), v) != v]
    if pc:
        f, port = pc[0]
        print(f"post-frame: first difference frame {f} port {port}: A={a_post[(f + shift, port)]} B={b_post[(f, port)]}"
              f" ({len(pc)} differing)")
    else:
        print("post-frame: no difference")
    sys.exit(1 if bad or pc else 0)


if __name__ == "__main__":
    main()
