"""Summary of a `kblog` run (envoy-balance.md, round 3): python tools/rust-port/envoy/kblog_summary.py a.log [more logs]

Per log: every foe-drive source that reached a player (blast capsules as applied and before the cap, percent fires,
Mark extra, Burning ticks), the count of foe-drive hits over 12% applied, and the stocks lost split into falls
(launch speed^2 under 100k in the log units) and launches, with the ones lost to a blast marked `*`."""
import re
import sys
from collections import defaultdict

CAP = 12000


def parse(path):
    hits, kos, fires, marks, burns = [], [], [], [], []
    cur = None
    for i, l in enumerate(open(path, errors='replace')):
        m = re.search(r'envoy: kblog (.*?) (-?\d+)\s*$', l)
        if not m:
            continue
        name, v = m.group(1), int(m.group(2))
        if name == 'hit fight*10+loop':
            cur = {'fl': v, 'line': i + 1}
            hits.append(cur)
        elif name.startswith('hit ') and cur is not None:
            cur[name[4:]] = v
        elif name.startswith('blast raw') and hits:
            hits[-1]['raw'] = v
        elif name.startswith('blast launches') and hits:
            hits[-1]['launches'] = v
        elif name.startswith('blast ') and hits:
            hits[-1]['drive'] = name[6:]
            hits[-1]['src'] = v
        elif name.startswith('fire '):
            fires.append((name[5:], v, i + 1))
        elif name == 'mark extra':
            marks.append(v)
        elif name == 'burn tick':
            burns.append(v)
        elif name == 'KO fight*10+loop':
            kos.append({'fl': v, 'line': i + 1})
        elif name.startswith('KO ') and kos:
            kos[-1][name[3:]] = v
    return hits, kos, fires, marks, burns


def pc(v):
    return '%.1f' % (v / 1000)


for path in sys.argv[1:]:
    hits, kos, fires, marks, burns = parse(path)
    print('==', path)
    loops = sorted(set(h['fl'] % 10 for h in hits))
    print('loops seen in hits:', loops)
    bl = [h for h in hits if h.get('blast*100+ck', 0) // 100 == 1]
    mv = [h for h in hits if h.get('blast*100+ck', 0) // 100 == 0]
    print('hit records: %d (foe blast %d, ordinary moves %d)' % (len(hits), len(bl), len(mv)))
    by = defaultdict(list)
    for h in bl:
        by[h.get('drive', '?')].append(h)
    for d, hs in sorted(by.items()):
        ds = [h['dmg'] for h in hs]
        print('  blast %-8s n=%3d applied min/max %s/%s  raw(before cap) max %s  launches %d  over 12%%: %d  srcs %s' % (
            d, len(hs), pc(min(ds)), pc(max(ds)), pc(max(h.get('raw', 0) for h in hs)), sum(h.get('launches', 0) for h in hs),
            sum(1 for x in ds if x > CAP), sorted(set(h.get('src', -1) for h in hs))))
    fb = defaultdict(list)
    for k, v, _ in fires:
        fb[k].append(v)
    for k, vs in sorted(fb.items()):
        print('  fire  %-8s n=%3d applied min/max %s/%s  over 12%%: %d' % (k, len(vs), pc(min(vs)), pc(max(vs)), sum(1 for x in vs if x > CAP)))
    if marks:
        print('  mark extra    n=%3d max %s  over 12%%: %d' % (len(marks), pc(max(marks)), sum(1 for x in marks if x > CAP)))
    if burns:
        print('  burn tick     n=%3d max %s' % (len(burns), pc(max(burns))))
    over = sum(1 for h in bl if h['dmg'] > CAP) + sum(1 for _, v, _ in fires if v > CAP) + sum(1 for v in marks if v > CAP)
    print('FOE-DRIVE HITS OVER 12%% APPLIED: %d' % over)
    if mv:
        ds = sorted(h['dmg'] for h in mv)
        print('  ordinary move hits: max %s, over 20%%: %d, over 30%%: %d' % (pc(ds[-1]), sum(1 for x in ds if x > 20000), sum(1 for x in ds if x > 30000)))
    falls = [k for k in kos if k.get('last speed', 0) < 100000]
    launches = [k for k in kos if k.get('last speed', 0) >= 100000]
    print('KOs: %d (falls %d, launches %d)' % (len(kos), len(falls), len(launches)))
    for nm, ks in (('falls', falls), ('launches', launches)):
        if not ks:
            continue
        ps = sorted(k['pct'] / 1000 for k in ks)
        q = lambda f: ps[min(len(ps) - 1, int(f * len(ps)))]
        print('  %-8s KO%% min %.1f  p25 %.1f  median %.1f  p75 %.1f  max %.1f | under 60%%: %d  under 80%%: %d | by a blast: %d' % (
            nm, ps[0], q(.25), q(.5), q(.75), ps[-1], sum(1 for x in ps if x < 60), sum(1 for x in ps if x < 80), sum(k.get('last blast', 0) for k in ks)))
        print('    ', ' '.join('%.0f%s' % (k['pct'] / 1000, '*' if k.get('last blast') else '') for k in sorted(ks, key=lambda k: k['pct'])))
    bk = [k for k in kos if k.get('last blast')]
    for k in bk:
        print('  KO by blast: loop %d fight %d at %s%% last dmg %s speed2 %d' % (k['fl'] % 10, k['fl'] // 10, pc(k['pct']), pc(k['last dmg']), k['last speed']))
