#!/usr/bin/env python3
"""Replay picker, layer 2 (notes/referee.md section 5): a cheap local scan of .slp files for frames whose recorded
Post-Frame state plausibly reaches a target, before any engine run.

usage:
  slpscan.py EXPR FILE_OR_DIR... [--max-hits N] [--list]
EXPR is a Python expression evaluated for every fighter on every frame, with
  a   this fighter's Post-Frame record, o  the other port's record on the same frame (None in a non-1v1 frame),
  pa  this fighter's record on the previous frame (None on the first),
each having: frame port follower char action x y facing percent shield last_attack combo last_hit_by stocks
state_frame airborne (char = the internal character id; action = the action state id; last_attack = the "last
attack landed" move id; last_hit_by = the port that last hit this fighter; state_frame = the action-state frame
counter), plus d_percent = a.percent - pa.percent.
  e.g. Zelda (internal 0x13) f-tilt (move id 9) connected on someone in Run (action 0x15):
       "a.char==0x13 and a.last_attack==9 and pa and pa.last_attack!=9 and o and o.action==0x15"
Prints one line per file with hits (count, first frames); --list prints only the matching paths (for layer 3).
Reads the files only; no replay data is written anywhere. Python standard library only.
"""
import os, struct, sys
from types import SimpleNamespace

FIELDS = [  # (name, offset in the event incl. its command byte, struct format)
    ("frame", 0x1, ">i"), ("port", 0x5, ">B"), ("follower", 0x6, ">B"), ("char", 0x7, ">B"), ("action", 0x8, ">H"),
    ("x", 0xA, ">f"), ("y", 0xE, ">f"), ("facing", 0x12, ">f"), ("percent", 0x16, ">f"), ("shield", 0x1A, ">f"),
    ("last_attack", 0x1E, ">B"), ("combo", 0x1F, ">B"), ("last_hit_by", 0x20, ">B"), ("stocks", 0x21, ">B"),
    ("state_frame", 0x22, ">f"), ("airborne", 0x2F, ">B"),
]


def raw_events(path):
    b = open(path, "rb").read()
    i = b.find(b"raw[$U#l")
    if i < 0:
        return
    n = struct.unpack(">I", b[i + 8:i + 12])[0]
    raw = memoryview(b)[i + 12:i + 12 + n]
    if not raw or raw[0] != 0x35:
        return
    psize = raw[1]
    sizes = {}
    for k in range(2, 1 + psize, 3):
        sizes[raw[k]] = struct.unpack(">H", bytes(raw[k + 1:k + 3]))[0]
    p = 1 + psize
    while p < len(raw):
        cmd = raw[p]
        sz = sizes.get(cmd)
        if sz is None:
            return
        yield cmd, raw[p:p + 1 + sz]
        p += 1 + sz


def post_frames(path):
    """frame -> {port: record}, followers (Nana) skipped."""
    frames = {}
    for cmd, ev in raw_events(path):
        if cmd != 0x38:
            continue
        rec = {}
        for name, off, fmt in FIELDS:
            sz = struct.calcsize(fmt)
            rec[name] = struct.unpack(fmt, bytes(ev[off:off + sz]))[0] if off + sz <= len(ev) else None
        if rec["follower"]:
            continue
        frames.setdefault(rec["frame"], {})[rec["port"]] = SimpleNamespace(**rec)
    return frames


def scan(path, code, max_hits):
    fr = post_frames(path)
    hits = []
    prev = {}
    for f in sorted(fr):
        ports = fr[f]
        for port, a in ports.items():
            others = [r for p, r in ports.items() if p != port]
            o = others[0] if len(others) == 1 else None
            pa = prev.get(port)
            a.d_percent = (a.percent - pa.percent) if pa and a.percent is not None and pa.percent is not None else 0.0
            try:
                ok = eval(code, {"__builtins__": {"range": range, "abs": abs, "min": min, "max": max}}, {"a": a, "o": o, "pa": pa})
            except Exception:
                ok = False
            if ok:
                hits.append((f, port))
                if len(hits) >= max_hits:
                    return hits
        prev = ports
    return hits


def main():
    a = sys.argv[1:]
    if len(a) < 2:
        print(__doc__)
        return 2
    max_hits = 1000
    if "--max-hits" in a:
        k = a.index("--max-hits")
        max_hits = int(a[k + 1])
        del a[k:k + 2]
    only_list = "--list" in a
    a = [x for x in a if x != "--list"]
    code = compile(a[0], "<expr>", "eval")
    files = []
    for t in a[1:]:
        if os.path.isdir(t):
            files += [os.path.join(t, n) for n in sorted(os.listdir(t)) if n.endswith(".slp")]
        else:
            files.append(t)
    nfiles = 0
    for p in files:
        h = scan(p, code, max_hits)
        if not h:
            continue
        nfiles += 1
        if only_list:
            print(p)
        else:
            print(f"{os.path.basename(p)}\t{len(h)}\t" + " ".join(f"f{f}p{port}" for f, port in h[:8]))
    if not only_list:
        print(f"slpscan: {nfiles} of {len(files)} files match", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
