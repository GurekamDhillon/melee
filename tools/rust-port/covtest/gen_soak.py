# SPDX-License-Identifier: MIT
"""Move-soak pad scripts (notes/coverage-testing.md, the third seed source): port 1 goes through every move of its
character at a few spacings and both facings against port 2, a human with no controller who just stands.
   python -I gen_soak.py OUT_DIR
Writes OUT_DIR/<char>.txt (a MELEE_PAD_SCRIPT, txt format `frames buttons_hex stickX stickY [trigL trigR]`) and
OUT_DIR/soak.jsonl (id, char, script, frames). record_soak.sh plays each in `melee play --match` (vanilla rules,
no Gecko codes or mods) with MELEE_SLP_RECORD; check.sh verifies the .slp in the Dolphin oracle.
The C-stick is not in the txt format: smashes are A with a full stick on the same frame, tilts a half stick."""
import json, os, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from gen_matches import CHARS  # noqa: E402

A, B, X, Z, R = 0x0100, 0x0200, 0x0400, 0x0010, 0x0020
WAIT = 70  # frames of neutral after each move (endlag, landing, the dummy's hitstun)


def seg(n, btn=0, x=0, y=0, tl=0, tr=0):
    return f"{n} {btn:04x} {x} {y} {tl} {tr}"


def move(lines, *segs, wait=WAIT):
    lines += list(segs)
    lines.append(seg(wait))


def moves(f):
    """One pass of the move list; f = +1 toward the dummy, -1 away (a back move)."""
    L = []
    move(L, seg(2, A), seg(4), seg(2, A), seg(4), seg(2, A))                  # jab 1-2-3
    move(L, seg(3, A, 64 * f, 0))                                             # forward tilt
    move(L, seg(3, A, 0, 64)), move(L, seg(3, A, 0, -64))                     # up / down tilt
    move(L, seg(3, A, 127 * f, 0)), move(L, seg(3, A, 0, 127)), move(L, seg(3, A, 0, -127))  # smashes
    move(L, seg(20, A, 127 * f, 0, ), seg(1), seg(2, A))                      # charged forward smash
    move(L, seg(8, 0, 127 * f, 0), seg(2, A, 127 * f, 0))                     # dash attack
    for d in ((0, 0), (127 * f, 0), (0, 127), (0, -127)):                    # ground specials
        move(L, seg(3, B, *d), wait=110)
    move(L, seg(2, Z), seg(12), seg(2, A), seg(10), seg(2, A))               # grab, pummels
    for d in ((127 * f, 0), (-127 * f, 0), (0, 127), (0, -127)):              # throws
        move(L, seg(2, Z), seg(12), seg(3, 0, *d), wait=100)
    for d in ((0, 0), (64 * f, 0), (-64 * f, 0), (0, 64), (0, -64)):          # aerials: n f b u d
        move(L, seg(3, X), seg(6), seg(3, A, *d), wait=80)
    move(L, seg(3, X), seg(4), seg(2, A, 0, 0), seg(1, 0, 0, -127), seg(10, 0, 0, -127))  # fast-fall nair
    for d in ((0, 0), (127 * f, 0), (0, -127)):                               # air specials (up-B on the ground set)
        move(L, seg(3, X), seg(6), seg(3, B, *d), wait=140)
    move(L, seg(20, R, 0, 0, 0, 255))                                         # shield
    move(L, seg(8, R, 0, 0, 0, 255), seg(3, R, 127 * f, 0, 0, 255))           # roll
    move(L, seg(8, R, 0, 0, 0, 255), seg(3, R, 0, -127, 0, 255))              # spot dodge
    move(L, seg(3, X), seg(8), seg(3, R, 64 * f, 64, 0, 255), wait=90)        # air dodge
    move(L, seg(4, 0, 0, -127), seg(30, 0, 0, -127))                          # crouch
    move(L, seg(3, X), seg(1), seg(3, X))                                     # double jump
    return L


def script(ck):
    L = ["# covtest move soak (gen_soak.py): port 1 " + CHARS[ck], seg(240)]  # READY/GO
    for space in (0, 18, 36):          # walk toward the dummy between passes
        if space:
            L.append(seg(space, 0, 60, 0))
            L.append(seg(20))
        L += moves(+1)
        L += [seg(2, 0, -60, 0), seg(20)]  # turn around: the same moves facing away (back moves, turnaround inputs)
        L += moves(-1)
        L += [seg(2, 0, 60, 0), seg(20)]
    return L


def main():
    out = sys.argv[1]
    os.makedirs(out, exist_ok=True)
    rows = []
    for ck in range(26):
        L = script(ck)
        frames = sum(int(l.split()[0]) for l in L if not l.startswith("#"))
        p = os.path.join(out, f"{CHARS[ck]}.txt")
        open(p, "w").write("\n".join(L) + "\n")
        rows.append({"id": f"soak_{CHARS[ck]}", "set": "soak", "char": ck, "script": p, "frames": frames})
    with open(os.path.join(out, "soak.jsonl"), "w") as f:
        for r in rows:
            f.write(json.dumps(r) + "\n")
    print(f"{len(rows)} soak scripts, {rows[0]['frames']} frames each -> {out}")


if __name__ == "__main__":
    main()
