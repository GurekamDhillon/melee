# Usage: rollback_vs_divergence.py REPLAY_ROOT SLPCHECK_CSV_DIR (Slippi 3.19.1 rows only; not run to completion, see notes/slp3191.md).
# For each diverged replay: is its first divergence frame a rolled-back (re-recorded) frame, and was the frame before it?
import sys, struct, collections, csv, os
def dups(path):
    d = open(path, 'rb').read(); i = d.index(b'raw[$U#l') + 8
    n = struct.unpack('>I', d[i:i+4])[0]; raw = d[i+4:i+4+n]
    sz = raw[1]; sizes = {}; j = 2
    while j < 1 + sz: sizes[raw[j]] = struct.unpack('>H', raw[j+1:j+3])[0]; j += 3
    p = 1 + sz; cnt = collections.Counter()
    while p < len(raw):
        c = raw[p]
        if c == 0x38: cnt[struct.unpack('>i', raw[p+1:p+5])[0]] += 1
        if c not in sizes: break
        p += 1 + sizes[c]
    frames = sorted(cnt)
    rb = {f for f in frames if cnt[f] > min(cnt.values())}
    return rb, frames
root, outdir = sys.argv[1], sys.argv[2]
tot = hit = prev = base_n = base_d = 0
for fn in sorted(os.listdir(outdir)):
    if not fn.endswith('.csv'): continue
    row = list(csv.reader(open(os.path.join(outdir, fn))))[-1]
    if row[2] != '3.19.1': continue
    rb, frames = dups(os.path.join(root, row[0]))
    base_n += len(rb); base_d += len(frames)
    if row[1] != 'match' and row[13]:
        f = int(row[13]); tot += 1; hit += f in rb; prev += (f - 1) in rb
print(f'3.19.1 replays: rolled-back frame share overall {base_n}/{base_d} = {base_n/base_d:.2f}')
print(f'diverged: {tot}; first-divergence frame rolled back: {hit}; frame before it rolled back: {prev}')
