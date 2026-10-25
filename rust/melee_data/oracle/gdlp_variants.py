"""Damaged / altered copies of the C-recorded whole-match .gdlp files, and the playback cases for them.
usage: gdlp_variants.py GDLP_DIR   (prints "g|name|flags" case lines)"""
import os, struct, sys

d = sys.argv[1]
out = []
base = sorted(f for f in os.listdir(d) if f.startswith("g") and f.endswith(".gdlp") and "_" not in f)
for name in base:
    data = open(os.path.join(d, name), "rb").read()
    stem = name[:-5]

    def put(suffix, b):
        open(os.path.join(d, f"{stem}_{suffix}.gdlp"), "wb").write(bytes(b))
        return f"{stem}_{suffix}.gdlp"

    # event offsets: table at 15, then events; find the HEAD event (0x90) right after the table
    tab_end = 15 + 1 + data[16]
    assert data[tab_end] == 0x90, hex(data[tab_end])
    head = tab_end + 1
    # fighters 6 x (1+30), then ident (0x92)
    ident = head + 16 + 6 * 31 + 1
    assert data[ident - 1] == 0x92
    cases = [name]
    cases.append(put("trunc", data[: len(data) // 2]))
    cases.append(put("trunc_head", data[: head + 20]))
    b = bytearray(data); b[ident + 20] ^= 1; cases.append(put("mods", b))          # id[2]
    b = bytearray(data); b[ident + 3] ^= 1; cases.append(put("build", b))          # id[0]
    b = bytearray(data); b[ident + 12] ^= 4; cases.append(put("disc", b))          # id[1]
    b = bytearray(data); b[ident + 28] ^= 4; cases.append(put("geno", b))          # id[3]
    b = bytearray(data); b[ident + 39] ^= 1; cases.append(put("genover", b))       # geno_version
    b = bytearray(data); b[head + 1] = 4; cases.append(put("ver4", b))             # format newer than this reads
    b = bytearray(data); b[head + 1] = 2; cases.append(put("ver2", b))             # no controls expected
    b = bytearray(data); b[head + 2] &= ~4; cases.append(put("notwhole", b))
    b = bytearray(data); b[head + 2] &= ~8; cases.append(put("notslippi", b))
    b = bytearray(data); b[head] = 0; b[head + 1] = 0; cases.append(put("ver0", b))
    # controls record is the last GD event before Game Start (0x36)
    c = data.index(b"GDC1")
    b = bytearray(data); b[c + 3] = ord("2"); cases.append(put("badtag", b))
    b = bytearray(data); b[c + 4 + 8 + 4] |= 0x80; cases.append(put("badchoice", b))
    b = bytearray(data); b[c + 4 + 3] = 0x90; cases.append(put("badck", b))
    b = bytearray(data[:16]) ; cases.append(put("tiny", b))
    cases.append(put("garbage", b"not a recording at all, just text"))
    for c in cases:
        out.append(f"g|{c}|play_gdlp,trace,rec,parity,frames=60")
        out.append(f"g|{c}|play_gdlp,resync,det=1,codes=c")
print("\n".join(out))
