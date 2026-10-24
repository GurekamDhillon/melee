"""Random memory-card scenarios (GCI folder) for the card oracle. One script per scenario, separated by
"=== name" lines. usage: card_scenarios.py COUNT"""
import random, struct, sys

count = int(sys.argv[1]) if len(sys.argv) > 1 else 80
r = random.Random(0x63617264)
H = lambda b: (b if isinstance(b, bytes) else b.encode()).hex() or "-"

NAMES = ["SuperSmashBros0001", "SuperSmashBros0002", "Melee_Save", "a", "b", "name32................................"[:32],
         "x" * 33, "weird<>:\"/\\|?*chars", "dot.name", "UPPER", "upper", "SuperSmashBros0001 ".strip(), "geno_records"]
GAMES = [("GALE", "01"), ("GALE", "02"), ("GALJ", "01"), ("ZZZZ", "00")]


def gci(name, game=b"GALE", maker=b"01", data_blocks=1, time=0x12345678, icon=0xFFFFFFFF, comment=0xFFFFFFFF, banner=0, fmt=0, speed=1,
        blocks=None, extra=0):
    """A valid .gci file: 64 byte header (big endian) + data."""
    nb = data_blocks if blocks is None else blocks
    h = bytearray(64)
    h[0:4] = game
    h[4:6] = maker
    h[6] = 0xFF
    h[7] = banner
    nm = name.encode("latin1")[:32]
    h[8:8 + len(nm)] = nm
    struct.pack_into(">I", h, 0x28, time)
    struct.pack_into(">I", h, 0x2C, icon)
    struct.pack_into(">H", h, 0x30, fmt)
    struct.pack_into(">H", h, 0x32, speed)
    h[0x34] = 4
    h[0x35] = 0
    struct.pack_into(">H", h, 0x36, 5)
    struct.pack_into(">H", h, 0x38, nb)
    struct.pack_into(">H", h, 0x3A, 0xFFFF)
    struct.pack_into(">I", h, 0x3C, comment)
    body = bytes((i * 31 + data_blocks) & 255 for i in range(data_blocks * 8192 + extra))
    return bytes(h) + body


def build(name):
    rr = random.Random(r.getrandbits(32))
    out = [f"=== {name}"]
    base = "USA/Card~A"  # "~" stands for a space in script paths
    pre = rr.choice([0, 0, 1, 2, 3, 4])
    if pre >= 1:
        out.append(f"MD {base}")
    names = []
    for i in range(rr.randint(0, 4) if pre else 0):
        nm = rr.choice(NAMES[:6] + ["seed%d" % i])
        g, m = rr.choice(GAMES[:2] + GAMES[:1])
        f = gci(nm, g.encode(), m.encode(), data_blocks=rr.randint(1, 3), banner=rr.choice([0, 1, 2]), icon=rr.choice([0xFFFFFFFF, 64, 0x240]),
                fmt=rr.choice([0, 1, 2, 0x5555, 0xAAAA, 0x4]), comment=rr.choice([0xFFFFFFFF, 0, 0x100]))
        out.append(f"MK {base}/{g}-{m}-{nm}.gci {H(f)}")
    if pre == 2:
        # duplicate identity -> BROKEN
        f = gci("dup", data_blocks=1)
        out.append(f"MK {base}/dup-a.gci {H(f)}")
        out.append(f"MK {base}/dup-b.gci {H(f)}")
    if pre == 3:
        out.append(f"MK {base}/short.gci {H(b'0123456789')}")
        out.append(f"MK {base}/notes.txt {H(b'hello')}")
        out.append(f"MK {base}/UP.GCI {H(gci('upper_ext'))}")
        out.append(f"MD {base}/sub")
    if pre == 4 and rr.random() < 0.3:
        for i in range(128):
            out.append(f"MK {base}/f{i:03d}.gci {H(gci('n%d' % i, data_blocks=1))}")
    out.append("INIT")
    out.append("PROBE 0")
    out.append("PROBE 1")
    out.append("PROBEEX 0")
    out.append("PROBEEX 1")
    out.append("RESULT 0")
    out.append("RESULT 1")
    out.append("CHECK 0")
    out.append("FREE 0")
    out.append("ENCODING 0")
    out.append("SECTOR 0")
    out.append("SERIAL 0")
    out.append("MOUNT 0")
    out.append("UNMOUNT 0")
    out.append("XFER 0")
    nops = rr.randint(10, 60)
    slots_open = []
    for _ in range(nops):
        c = rr.random()
        ch = 0 if rr.random() < 0.9 else rr.choice([1, 2, -1])
        nm = rr.choice(NAMES + names)
        slot = rr.randint(0, 3)
        if c < 0.14:
            sz = rr.choice([8192, 8192, 16384, 24576, 0, 100, 8192 * 5, 8192 * 20, 8192 * 400, 8191])
            out.append(f"CREATE {ch} {nm} {sz} {slot}")
        elif c < 0.24:
            out.append(f"OPEN {ch} {nm} {slot}")
        elif c < 0.30:
            out.append(f"FASTOPEN {ch} {rr.randint(-1, 6)} {slot}")
        elif c < 0.35:
            out.append(f"CLOSE {slot}")
        elif c < 0.50:
            ln = rr.choice([0, 1, 16, 100, 4096, 8192, 8193, 20000, -1])
            off = rr.choice([0, 0, 16, 4096, 8191, 8192, 100000, -4])
            out.append(f"READ {slot} {ln} {off}")
        elif c < 0.66:
            ln = rr.choice([0, 1, 16, 100, 4096, 8192, 8193, 20000, -1])
            off = rr.choice([0, 0, 16, 4096, 8191, 8192, 100000, -4])
            out.append(f"WRITE {slot} {ln} {off} {rr.randint(0, 255)}")
        elif c < 0.72:
            out.append(f"DELETE {ch} {nm}")
        elif c < 0.75:
            out.append(f"FASTDELETE {ch} {rr.randint(-1, 5)}")
        elif c < 0.80:
            out.append(f"RENAME {ch} {nm} {rr.choice(NAMES + names)}")
        elif c < 0.88:
            out.append(f"STATUS {ch} {rr.randint(-1, 7)}")
        elif c < 0.93:
            out.append(f"SETSTATUS {ch} {rr.randint(-1, 5)} {rr.choice([0, 1, 2, 3])} {rr.choice([0xFFFFFFFF, 0, 64, 128, 0x240])} "
                       f"{rr.choice([0, 1, 2, 0x5555, 0x6, 0xAAAA, 0x1234])} {rr.choice([0, 1, 2, 3, 0x1111])} {rr.choice([0xFFFFFFFF, 0, 0x100, 64])}")
        elif c < 0.95:
            out.append(f"FREE {ch}")
        elif c < 0.97:
            g, m = rr.choice(GAMES)
            out.append(f"GAME {ch} {g} {m}")
        elif c < 0.99:
            out.append(f"REMOUNT {ch}")
        else:
            out.append("FORMAT 0")
    out.append("FILES")
    out.append("REMOUNT 0")
    for i in range(6):
        out.append(f"STATUS 0 {i}")
    out.append("FREE 0")
    out.append("FILES")
    return "\n".join(out)


print("\n".join(build(f"cd{i:02d}") for i in range(count)))
