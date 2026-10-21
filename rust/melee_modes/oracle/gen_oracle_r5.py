#!/usr/bin/env python3
"""Group 6 round 5 C oracle: leaf functions over nested pointers that round 3 skipped
(lbcommand's Command_04/05/07, textdraw's list and string helpers, ...).

  python rust/melee_modes/oracle/gen_oracle_r5.py <outdir>     (from the worktree root)

Reuses gen_oracle_r4.py's machinery unchanged (verbatim cut, callee closure, buffers, links,
pointer-normalised hash); only the function list, domains and row builders differ.
"""
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import gen_oracle_r4 as g4  # noqa: E402

R = g4.R
R.seed(0x726F756E6435)
NULL = g4.NULL

g4.FUNCS = {
    "src/melee/lb/lbcommand.c": ["Command_04", "Command_05", "Command_07"],
    "src/melee/if/textdraw.c": ["DevText_StrLen", "DevText_NumToStr", "DevText_AddToList"],
    "src/melee/mn/mnnamenew.c": ["GetAutoNameCharacter"],
    "src/melee/if/ifstock.c": ["copyColor"],
    "src/melee/gm/gmmain_lib.c": ["SetDefaultHandicaps"],
    "src/melee/ty/toy.c": ["_Toy_8030FE48_link_entries"],
}
# main() reads lbAudioAx_UserData / HSD_JObj / CameraBlurData from these two; parsed, not cut.
g4.CALLEE_TUS = ["src/melee/lb/lbaudio_ax.c", "src/melee/lb/lbspdisplay.c"]
g4.EXTRA = {}
g4.DOMAIN = {
    "GetAutoNameCharacter": [None, list(range(0, 512))],
    "_Toy_8030FE48_link_entries": [None, list(range(0, 14))],
}
g4.NULLABLE = {"DevText_StrLen": 0}
g4.FORCEBUF = {}
g4.NROWS = {"DevText_NumToStr": 400}
g4.SCALAR_ROWS = {}

# native CommandInfo (x86_64): timer 0, frame_count 4, u 8, loop_count 16, event_return 24
CI_U, CI_LC = 8, 16


def cmd_bufs(r, a, lcs, keep0=False):
    """keep0: event_return[0] is never a link (Command_04 with loop_count 3/4 decrements one of
    its halves, and a buffer address moved by -1 or -2^32 no longer compares by target)."""
    info = a[0]
    cu = r.buf(CMD_T, 16)
    r.links.append((info, CI_U, cu))
    r.fix.append((info, CI_LC, 4, R.choice(lcs)))
    # some event_return slots point into the command buffer, some are NULL / low-32-zero
    for k in range(3):
        c = R.random()
        if keep0 and k == 0 and c < 0.3:
            c = 0.55
        if c < 0.3:
            r.links.append((info, 24 + 8 * k, cu))
        elif c < 0.5:
            r.links.append((info, 24 + 8 * k, NULL))
        elif c < 0.6:
            r.fix.append((info, 24 + 8 * k, 8, 0x1234_0000_0000))
    # Command_05/07 read u[1].ptr: sometimes a real link
    if R.random() < 0.5:
        r.links.append((cu, 8, cu))


def b_cmd04(r, a):
    cmd_bufs(r, a, [2, 3, 4], keep0=True)  # lc 1: the decrement hits loop_count itself, then event_return[0u - 1] (u32) faults


def b_cmd05(r, a):
    cmd_bufs(r, a, [0, 1, 2])


def b_cmd07(r, a):
    cmd_bufs(r, a, [0, 1, 2, 3])


def b_strlen(r, a):
    k = a[0]
    if r.bufs[k][0]:
        sz = r.bufs[k][1]
        r.fix.append((k, sz - 1, 1, 0))
        if R.random() < 0.5:
            r.fix.append((k, R.randrange(0, 40), 1, 0))


def b_addlist(r, a):
    lst, text = a
    n = R.randrange(0, 6)
    nodes = [r.buf(DEVTEXT_T) for _ in range(n)]
    ids = sorted(R.choice([-5, 0, 1, 2, 3, 3, 7, 100, -128, 127]) for _ in range(n))
    if R.random() < 0.2:
        R.shuffle(ids)
    for k, (nd, i) in enumerate(zip(nodes, ids)):
        r.fix.append((nd, DT["id"], 1, i & 0xFF))
        r.links.append((nd, DT["next"], nodes[k + 1] if k + 1 < n else NULL))
    r.fix.append((text, DT["id"], 1, R.choice([-5, 0, 1, 2, 3, 7, 100, -128, 127]) & 0xFF))
    r.links.append((lst, 0, nodes[0] if nodes else NULL))


def b_names(r, a):
    r.links.append((a[0], 0, r.buf(U8_T)))


def b_color(r, a):
    r.link(a[0], "mat")


g4.BUILD = {"Command_04": b_cmd04, "Command_05": b_cmd05, "Command_07": b_cmd07,
            "DevText_StrLen": b_strlen, "DevText_AddToList": b_addlist,
            "GetAutoNameCharacter": b_names, "copyColor": b_color}

_orig_main = g4.main


def main(out):
    global CMD_T, DEVTEXT_T, DT, U8_T
    ci, K = g4.ci, g4.K
    idx = ci.Index.create()
    tu = idx.parse("src/melee/lb/lbcommand.c", args=g4.NAT)
    CMD_T = next(c.type for c in tu.cursor.walk_preorder()
                 if c.kind == K.UNION_DECL and c.spelling == "CmdUnion" and c.is_definition()).get_canonical()
    tu2 = idx.parse("src/melee/if/textdraw.c", args=g4.NAT)
    DEVTEXT_T = next(c.type for c in tu2.cursor.walk_preorder()
                     if c.kind == K.TYPEDEF_DECL and c.spelling == "DevText").get_canonical()
    DT = {f.spelling: DEVTEXT_T.get_offset(f.spelling) // 8 for f in DEVTEXT_T.get_fields()}
    U8_T = next(c.type for c in tu2.cursor.walk_preorder()
                if c.kind == K.TYPEDEF_DECL and c.spelling == "u8").get_canonical()
    _orig_main(out)


if __name__ == "__main__":
    main(sys.argv[1])
