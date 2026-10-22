"""Oracle patch for src/melee/gm/gmresult.c (applied to a copy; see gen/tu/run.py).

Four globals are `u32` words that the big-endian game reads as SJIS text through their address
(`81 7C` = a dash, `81 46` = a colon, ...). On a little-endian x86 build the words' bytes come out
reversed. The Rust port stores the bytes in text order; the patch writes the C words byte-swapped
so the native C holds the same bytes.
"""


def bswap(v):
    return int.from_bytes(v.to_bytes(4, "big"), "little")


def patch(t):
    subs = [
        ("{ 0x817C817C, 0x817C0000 }", "{ 0x%08X, 0x%08X }" % (bswap(0x817C817C), bswap(0x817C0000))),
        ("u32 lbl_804D3FA0 = 0x817C0000;", "u32 lbl_804D3FA0 = 0x%08X;" % bswap(0x817C0000)),
        ("u32 lbl_804D3FA4 = 0x817B0000;", "u32 lbl_804D3FA4 = 0x%08X;" % bswap(0x817B0000)),
        ("{ { 0x817C8146, 0x817C0000 } }", "{ { 0x%08X, 0x%08X } }" % (bswap(0x817C8146), bswap(0x817C0000))),
    ]
    for a, b in subs:
        assert t.count(a) == 1, a
        t = t.replace(a, b)
    return t
