# Two .slp files of one online match (one per peer): same Game Start fields that should agree, and every common frame's
# Pre-Frame (0x37) and Post-Frame (0x38) events byte-equal. Usage: slp_agree.py A.slp B.slp
import struct, sys


def events(path):
    d = open(path, 'rb').read()
    i = d.index(b'raw[$U#l') + 8
    n = struct.unpack('>I', d[i:i + 4])[0]
    raw = d[i + 4:i + 4 + n]
    assert raw[0] == 0x35, 'no Event Payloads'
    sz = raw[1]
    sizes = {}
    j = 2
    while j < 1 + sz:
        sizes[raw[j]] = struct.unpack('>H', raw[j + 1:j + 3])[0]
        j += 3
    p = 1 + sz
    ev = []
    while p < len(raw):
        c = raw[p]
        if c not in sizes:
            raise SystemExit(f'{path}: unknown event 0x{c:02x} at {p}')
        ev.append((c, raw[p:p + 1 + sizes[c]]))
        p += 1 + sizes[c]
    return ev, d[i + 4 + n:]


def frames(ev):
    out = {}
    for c, b in ev:
        if c in (0x37, 0x38):
            fr = struct.unpack('>i', b[1:5])[0]
            out.setdefault(fr, []).append(b)
    return out


ea, ma = events(sys.argv[1])
eb, mb = events(sys.argv[2])
for name, e in (('A', ea), ('B', eb)):
    kinds = {}
    for c, _ in e:
        kinds[c] = kinds.get(c, 0) + 1
    end = [b for c, b in e if c == 0x39]
    print(f'{name}: events ' + ' '.join(f'{k:02x}:{v}' for k, v in sorted(kinds.items())) + f'  Game End method {end[0][1] if end else None}')
gs = [b for c, b in ea if c == 0x36][0], [b for c, b in eb if c == 0x36][0]
# Game Info Block (0x5..0x13D) and the seed: the agreed match
print('Game Start info block equal:', gs[0][5:0x141] == gs[1][5:0x141])
fa, fb = frames(ea), frames(eb)
common = sorted(set(fa) & set(fb))
bad = [f for f in common if sorted(fa[f]) != sorted(fb[f])]
print(f'frames A {min(fa)}..{max(fa)} ({len(fa)}), B {min(fb)}..{max(fb)} ({len(fb)}), common {len(common)}, differing {len(bad)}' + (f', first {bad[0]}' if bad else ''))
sys.exit(1 if bad or gs[0][5:0x141] != gs[1][5:0x141] else 0)
