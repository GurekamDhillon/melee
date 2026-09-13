#!/usr/bin/env python3
"""Extract the data blobs melee's sources #include from the game's own main.dol.

Two arrays (the SIS font atlas and the debug font atlas) live in the original DOL's .data
rather than in the repository, so the matching build extracts them with decomp-toolkit. The
port does the same thing here without dtk: read main.dol straight out of the user's disc image
(or from orig/GALE01/sys/main.dol), then write the byte blobs as C initializer .inc files into
build/GALE01/include, which is already on the compiler's include path.
"""
import argparse
import hashlib
import os
import re
import struct
import sys

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))


def read_dol_from_iso(path):
    with open(path, "rb") as f:
        header = f.read(0x440)
        game_id = header[0:6].decode("ascii", "replace")
        revision = header[7]
        if struct.unpack(">I", header[0x1C:0x20])[0] != 0xC2339F3D:
            raise SystemExit(f"{path}: not a GameCube disc image")
        if game_id != "GALE01":
            raise SystemExit(f"{path}: game ID is {game_id}, expected GALE01 (Melee NTSC-U)")
        if revision != 2:
            print(f"warning: disc revision is {revision}, the decomp targets revision 2 (v1.02)")
        dol_offset = struct.unpack(">I", header[0x420:0x424])[0]
        f.seek(dol_offset)
        dol_header = f.read(0x100)
        offsets = struct.unpack(">18I", dol_header[0x00:0x48])
        sizes = struct.unpack(">18I", dol_header[0x90:0xD8])
        dol_size = max(o + s for o, s in zip(offsets, sizes) if s)
        f.seek(dol_offset)
        return f.read(dol_size)


def dol_reader(dol):
    """Return a function mapping a virtual address + length to bytes."""
    offsets = struct.unpack(">18I", dol[0x00:0x48])
    addresses = struct.unpack(">18I", dol[0x48:0x90])
    sizes = struct.unpack(">18I", dol[0x90:0xD8])
    sections = [(a, s, o) for o, a, s in zip(offsets, addresses, sizes) if s]

    def read(address, length):
        for start, size, offset in sections:
            if start <= address and address + length <= start + size:
                pos = offset + (address - start)
                return dol[pos:pos + length]
        raise SystemExit(f"address {address:#x}+{length:#x} is not in any DOL section")

    return read


def parse_symbols(path, wanted):
    """Pull `name = .section:0xADDR; // ... size:0xN` entries out of config symbols.txt."""
    found = {}
    pattern = re.compile(
        r"^(?P<name>\w+)\s*=\s*(?:[.\w]+:)?(?P<addr>0x[0-9A-Fa-f]+);.*?size:(?P<size>0x[0-9A-Fa-f]+)")
    with open(path, "r", encoding="utf-8") as f:
        for line in f:
            m = pattern.match(line.strip())
            if m and m.group("name") in wanted:
                found[m.group("name")] = (int(m.group("addr"), 16), int(m.group("size"), 16))
    missing = wanted - found.keys()
    if missing:
        raise SystemExit(f"symbols not found in {path}: {', '.join(sorted(missing))}")
    return found


def write_inc(path, data):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8", newline="\n") as f:
        for i in range(0, len(data), 16):
            f.write("".join(f"0x{b:02X}, " for b in data[i:i + 16]).rstrip() + "\n")


# symbol -> .inc path relative to the generated include directory (from config/GALE01/config.yml)
EXTRACTS = {
    "HSD_SisLib_FontAtlas": "sysdolphin/baselib/sislib_font.inc",
    "HSD_DebugFontAtlas": "sysdolphin/baselib/debug_font.inc",
}
EXPECTED_DOL_SHA1 = "08e0bf20134dfcb260699671004527b2d6bb1a45"


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--iso", help="path to a Melee (GALE01, v1.02) disc image")
    ap.add_argument("--dol", help="path to main.dol (default: orig/GALE01/sys/main.dol)")
    ap.add_argument("--out", default=os.path.join(REPO, "build", "GALE01", "include"),
                    help="generated include directory")
    args = ap.parse_args()

    dol_path = args.dol or os.path.join(REPO, "orig", "GALE01", "sys", "main.dol")
    if args.iso:
        dol = read_dol_from_iso(args.iso)
        os.makedirs(os.path.dirname(dol_path), exist_ok=True)
        with open(dol_path, "wb") as f:
            f.write(dol)
        print(f"extracted {len(dol)} bytes of main.dol from {args.iso} -> {dol_path}")
    else:
        if not os.path.exists(dol_path):
            raise SystemExit(f"{dol_path} not found; pass --iso to extract it from a disc image")
        with open(dol_path, "rb") as f:
            dol = f.read()

    digest = hashlib.sha1(dol).hexdigest()
    if digest != EXPECTED_DOL_SHA1:
        print(f"warning: main.dol sha1 is {digest}, expected {EXPECTED_DOL_SHA1}")

    read = dol_reader(dol)
    symbols = parse_symbols(os.path.join(REPO, "config", "GALE01", "symbols.txt"), set(EXTRACTS))
    for name, rel in EXTRACTS.items():
        address, size = symbols[name]
        write_inc(os.path.join(args.out, rel), read(address, size))
        print(f"{name}: {size:#x} bytes at {address:#010x} -> {rel}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
