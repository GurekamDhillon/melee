# Usage: slp_rollback_frames.py FILE.slp FROM TO
# Count Post-Frame (0x38) events per (frame, port) in a .slp's raw event stream; report frames recorded more than once.
import sys, struct, collections
d = open(sys.argv[1], 'rb').read()
i = d.index(b'raw[$U#l') + 8
n = struct.unpack('>I', d[i:i+4])[0]; i += 4
raw = d[i:i+n]
assert raw[0] == 0x35
sz = raw[1]; sizes = {}
j = 2
while j < 1 + sz:
    sizes[raw[j]] = struct.unpack('>H', raw[j+1:j+3])[0]; j += 3
p = 1 + sz
cnt = collections.Counter(); order = []
while p < len(raw):
    c = raw[p]
    if c == 0x38:
        fr = struct.unpack('>i', raw[p+1:p+5])[0]; port = raw[p+5]; fol = raw[p+6]
        x = raw[p+0x0A:p+0x0E].hex(); y = raw[p+0x0E:p+0x12].hex()
        cnt[(fr, port, fol)] += 1; order.append((fr, port, fol, x, y))
    if c not in sizes: break
    p += 1 + sizes[c]
dup = sorted(k for k, v in cnt.items() if v > 1)
print('post events', len(order), 'distinct', len(cnt), 'frames recorded more than once:', len(dup))
lo, hi = int(sys.argv[2]), int(sys.argv[3])
for o in order:
    if lo <= o[0] <= hi and o[2] == 0: print(o)
