# SPDX-License-Identifier: MIT
"""Read words of main.dol from the vanilla disc (GW_ISO_VANILLA in the workspace .env; the path is never printed).
   python -I dol.py ADDR [N]   prints N big-endian words from ADDR (hex)
Used to check the instructions the covtest boot codes patch (gen_matches.py, notes/coverage-testing.md)."""
import os, re, struct, sys

def iso_path():
    ws = os.environ.get("GW_WORKSPACE", "E:/Projects/Melee Workspace")
    for line in open(os.path.join(ws, ".env"), encoding="utf-8"):
        m = re.match(r'^(?:export )?GW_ISO_VANILLA=(.*)$', line.strip())
        if m:
            return m.group(1).strip().strip('"')
    sys.exit("GW_ISO_VANILLA is not in .env")

class Dol:
    def __init__(self):
        with open(iso_path(), "rb") as f:
            f.seek(0x420)
            off = struct.unpack(">I", f.read(4))[0]
            f.seek(off)
            hdr = f.read(0x100)
            self.secs = []
            for i in range(18):
                o = struct.unpack(">I", hdr[i * 4:i * 4 + 4])[0]
                a = struct.unpack(">I", hdr[0x48 + i * 4:0x4C + i * 4])[0]
                s = struct.unpack(">I", hdr[0x90 + i * 4:0x94 + i * 4])[0]
                if s:
                    f.seek(off + o)
                    self.secs.append((a, f.read(s)))

    def word(self, addr):
        for a, d in self.secs:
            if a <= addr < a + len(d):
                return struct.unpack(">I", d[addr - a:addr - a + 4])[0]
        raise KeyError(hex(addr))

if __name__ == "__main__":
    d = Dol()
    a = int(sys.argv[1], 16)
    for k in range(int(sys.argv[2]) if len(sys.argv) > 2 else 1):
        print(f"{a + 4 * k:08X} {d.word(a + 4 * k):08X}")
