#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Content index of a replay list for the coverage map (docs/rust-port/notes/coverage-map.md).
   python -I meta.py LIST.tsv OUT.tsv
LIST.tsv: regress.tsv layout (set, relpath, ...). OUT.tsv: one row per replay: set, relpath, version, stage (StKind),
ext chars (Slippi external ids), internal chars seen in Post-Frame (CharacterKind: Sheik/Zelda transforms, Nana), item
kinds seen in Item Update (ItemKind), frames, item spawn setting, players. Reads only the .slp game-start/frame events;
no player names are read or written."""
import os, struct, sys

W = "E:/Projects/Melee Workspace"
DIRS = {"ext": f"{W}/scratch/replays-ext", "hero": f"{W}/_build/replays/hero-2026-11-06", "lunar": f"{W}/_build/slp",
        "gang": f"{W}/Gang-Steals", "lunar2": f"{W}/_build/replays/lunar-2026-11-20", "v320": f"{W}/_build/replays/lunar-2026-11-27/v320"}


def events(b):
    i = b.find(b"raw[$U#l")
    if i < 0:
        return
    n = struct.unpack(">I", b[i + 8:i + 12])[0]
    raw = memoryview(b)[i + 12:i + 12 + n]
    if not raw or raw[0] != 0x35:
        return
    psize = raw[1]
    sizes = {raw[k]: struct.unpack(">H", bytes(raw[k + 1:k + 3]))[0] for k in range(2, 1 + psize, 3)}
    p = 1 + psize
    while p < len(raw):
        cmd = raw[p]
        sz = sizes.get(cmd)
        if sz is None:
            return
        yield cmd, raw[p:p + 1 + sz]
        p += 1 + sz


def scan(path):
    b = open(path, "rb").read()
    ver = stage = None; ext = []; items_on = None; internal = set(); items = set(); frames = 0; nplay = 0
    for cmd, ev in events(b):
        if cmd == 0x36:
            ver = ".".join(str(x) for x in ev[1:4])
            stage = struct.unpack(">H", bytes(ev[0x13:0x15]))[0]
            items_on = struct.unpack(">b", bytes(ev[0x10:0x11]))[0]
            for k in range(4):
                o = 0x65 + 0x24 * k
                if ev[o + 1] != 3:
                    ext.append(ev[o]); nplay += 1
        elif cmd == 0x38:
            internal.add(ev[7])
            frames = max(frames, struct.unpack(">i", bytes(ev[1:5]))[0])
        elif cmd == 0x3B:
            items.add(struct.unpack(">H", bytes(ev[5:7]))[0])
    return ver, stage, ext, sorted(internal), sorted(items), frames, items_on, nplay


def main():
    lst, out = sys.argv[1], sys.argv[2]
    rows = [l.rstrip("\n").split("\t") for l in open(lst, encoding="utf-8")][1:]
    with open(out, "w") as fo:
        fo.write("set\trelpath\tversion\tstage\text_chars\tinternal_chars\titems\tframes\titem_spawn\tplayers\n")
        for r in rows:
            s, rel = r[0], r[1]
            p = os.path.join(DIRS[s], rel)
            try:
                ver, st, ext, ic, it, fr, ion, n = scan(p)
            except OSError:
                continue
            fo.write(f"{s}\t{rel}\t{ver}\t{st}\t{','.join(map(str, ext))}\t{','.join(map(str, ic))}\t{','.join(map(str, it))}\t{fr}\t{ion}\t{n}\n")


if __name__ == "__main__":
    main()
