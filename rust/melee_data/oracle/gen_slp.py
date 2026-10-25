"""Synthetic .slp corpus for the replay oracle: modern (3.x + GD suffix) streams, older layouts,
truncations and refusals. Deterministic. usage: gen_slp.py OUTDIR"""
import os, random, struct, sys
out = sys.argv[1]
os.makedirs(out, exist_ok=True)
r = random.Random(0x5150)
be32 = lambda v: struct.pack(">I", v & 0xFFFFFFFF)
SZ_START = 0x2F8 + 8 + 20


def rb(n):
    return bytes(r.getrandbits(8) for _ in range(n))


def fl():
    c = r.random()
    if c < 0.05:
        return struct.pack(">I", r.getrandbits(32))  # any bits incl NaN/inf
    if c < 0.4:
        return struct.pack(">f", r.choice([0.0, -0.0, 1.0, -1.0, 0.5, 0.0125]))
    return struct.pack(">f", r.uniform(-1.2, 1.2))


def start(sz, rules=None, ctl=None, stage=31, types=(0, 0, 3, 3), online=False, frozen=1):
    """Payload of a Game Start event (the C sees it at offset +1, after the command byte)."""
    b = bytearray(rb(sz))
    b[0:4] = bytes([r.choice([1, 2, 3]), r.randint(0, 20), r.randint(0, 3), 0])
    gi = bytearray(rb(0x138))
    gi[0x0E] = stage >> 8
    gi[0x0F] = stage & 255
    for p in range(4):
        o = 0x60 + 0x24 * p
        gi[o] = r.randint(0, 25)
        gi[o + 1] = types[p]
        gi[o + 3] = r.randint(0, 5)
    b[4:4 + 0x138] = gi
    b[0x13C:0x140] = be32(r.getrandbits(32))
    if sz > 0x1A4:
        b[0x1A1] = frozen
        b[0x1A3] = 8 if online else 2
    if sz >= 0x2F8 + 8:
        if rules is not None:
            b[0x2F8:0x2F8 + 4] = b"GDT1"
            b[0x2F8 + 4:0x2F8 + 8] = be32(rules)
        if ctl is not None:
            b[0x300:0x304] = b"GDC1"
            for i, w in enumerate(ctl):
                b[0x304 + 4 * i:0x308 + 4 * i] = be32(w)
    return bytes(b)


def pre(sz, frame, port, fol):
    b = bytearray(rb(sz))
    b[0:4] = be32(frame)
    b[4] = port
    b[5] = fol
    for off in (0x18, 0x1C, 0x20, 0x24, 0x28):
        if off + 4 <= sz:
            b[off:off + 4] = fl()
    return bytes(b)


def post(sz, frame, port, fol):
    b = bytearray(rb(sz))
    b[0:4] = be32(frame)
    b[4] = port
    b[5] = fol
    for off in (0x09, 0x0D, 0x11, 0x15):
        if off + 4 <= sz:
            b[off:off + 4] = fl()
    return bytes(b)


def build(events, table, meta=b"U\x08metadata{U\x08playedOnSU\x08melee-pc}}", rawlen=None, tablemut=None):
    ev = b"".join(events)
    tab = bytes([0x35, 1 + 3 * len(table)]) + b"".join(bytes([c]) + struct.pack(">H", s) for c, s in table)
    if tablemut:
        tab = tablemut(tab)
    raw = tab + ev
    n = len(raw) if rawlen is None else rawlen
    return b"{U\x03raw[$U#l" + be32(n) + raw + meta


def game(nframes, first=-123, ver="modern", **kw):
    szs = {"old": (0x140, 0x3B, 0x40, 0x0A), "mid": (0x1A4, 0x40, 0x3E, 0x0C), "modern": (SZ_START, 0x42, 0x54, 0x0C)}[ver]
    sstart, spre, spost, sfs = szs
    table = [(0x36, sstart), (0x37, spre), (0x38, spost), (0x39, 6)]
    if ver != "old":
        table.append((0x3A, sfs))
    ev = [bytes([0x36]) + start(sstart, **kw)]
    ports = [p for p in range(4) if kw.get("types", (0, 0, 3, 3))[p] != 3]
    fols = r.random() < 0.3
    for f in range(first, first + nframes):
        if ver != "old":
            ev.append(bytes([0x3A]) + be32(f) + be32(r.getrandbits(32)) + rb(sfs - 8))
        for p in ports:
            for fol in ([0, 1] if (fols and p == 0) else [0]):
                ev.append(bytes([0x37]) + pre(spre, f, p, fol))
        for p in ports:
            ev.append(bytes([0x38]) + post(spost, f, p, 0))
    ev.append(bytes([0x39]) + bytes([7]) + b"\0" * 5)
    return ev, table


def put(name, data):
    open(os.path.join(out, name), "wb").write(data)


# valid streams
for i in range(24):
    ver = ["modern", "modern", "mid", "old"][i % 4]
    kw = {}
    if ver == "modern":
        kw["rules"] = r.choice([None, 0, 0x5FB, 0x1000000 | 0x1FF | 0x400, 0x1])
        kw["ctl"] = r.choice([None, [0, 0, 0, 0], [0xff3d, 0x01, 0x20, 0x0400]])
        kw["online"] = r.random() < 0.4
    kw["types"] = r.choice([(0, 0, 3, 3), (0, 1, 3, 3), (0, 0, 0, 0), (1, 0, 2, 3)])
    kw["stage"] = r.choice([2, 31, 32, 258])
    ev, table = game(r.randint(2, 40), ver=ver, **kw)
    put(f"ok{i:02d}_{ver}.slp", build(ev, table))
# skins metadata
ev, table = game(5)
put("skin.slp", build(ev, table, meta=b"U\x08metadata{U\x08playedOnSU\x08melee-pcU\x09stageSkinSU\x0f0123456789ab|Frosty}}"))
put("skin_long.slp", build(ev, table, meta=b"U\x09stageSkinSU\x30" + b"x" * 40 + b"}}"))
put("skin_cut.slp", build(ev, table, meta=b"U\x09stageSkinSU\x30xx"))
# frame range edge cases
# (a first frame of exactly 0 makes the C write out of bounds, so no corpus file: see replay.rs)
ev, table = game(3, first=-200)
put("below123.slp", build(ev, table))
ev, table = game(1, first=-123)
put("oneframe.slp", build(ev, table))
# structural damage
ev, table = game(30)
base = build(ev, table)
for j, cut in enumerate([20, 40, 60, 100, 400, 1500, len(base) - 3, len(base) // 2]):
    put(f"trunc{j}.slp", base[:cut])
put("zerolen.slp", build(ev, table, rawlen=0))
put("shortlen.slp", build(ev, table, rawlen=900))
put("longlen.slp", build(ev, table, rawlen=10 ** 7))
put("badtable_len.slp", build(ev, table, tablemut=lambda t: bytes([t[0], 2]) + t[2:]))
put("badtable_cmd.slp", build(ev, table, tablemut=lambda t: bytes([0x34]) + t[1:]))
put("table_huge.slp", build(ev, table, tablemut=lambda t: bytes([t[0], 250]) + t[2:]))
put("noraw.slp", b"{U\x03xyz" + rb(100))
# rule / control words
for name, kw in [("rule_unsupported", dict(rules=0x8000 | 1)), ("rule_off_bit", dict(rules=2)),
                 ("rule_frames", dict(rules=1 | (121 << 24))), ("rule_frames_ok", dict(rules=1 | (120 << 24))),
                 ("ctl_bad", dict(rules=0, ctl=[0x10000, 0, 0, 0])), ("ctl_ok", dict(rules=0, ctl=[0x3d, 0, 0, 0xff00]))]:
    ev, table = game(4, **kw)
    put(name + ".slp", build(ev, table))
# torn tags
ev, table = game(4, rules=0x5FB, ctl=[1, 2, 3, 4])
d = bytearray(build(ev, table))
i = d.find(b"GDT1")
j = d.find(b"GDC1")
assert i > 0 and j > 0
for nm, pos, val in [("rule_gdt2", i + 3, ord("2")), ("rule_gdx", i + 2, ord("X")), ("rule_notg", i, ord("H")),
                     ("ctl_gdc2", j + 3, ord("2")), ("ctl_foreign", j + 1, ord("Z")), ("ctl_notg", j, ord("H"))]:
    d2 = bytearray(d)
    d2[pos] = val
    put(nm + ".slp", bytes(d2))
# random byte flips inside a valid stream (the reader must agree even when the data is nonsense)
ev, table = game(12)
base = build(ev, table)
for j in range(40):
    d = bytearray(base)
    for _ in range(r.randint(1, 6)):
        pos = r.randint(40, len(d) - 1)
        d[pos] = r.getrandbits(8)
    put(f"flip{j:02d}.slp", bytes(d))
# no pre-frame events at all, and no game start
put("nopre.slp", build([bytes([0x36]) + start(SZ_START, rules=0, ctl=[0] * 4), bytes([0x39]) + b"\x07" + b"\0" * 5], [(0x36, SZ_START), (0x39, 6)]))
put("nostart.slp", build([bytes([0x37]) + pre(0x42, -123, 0, 0)], [(0x37, 0x42)]))
put("short_pre.slp", build([bytes([0x37]) + pre(0x2F, -123, 0, 0)], [(0x37, 0x2F)]))
