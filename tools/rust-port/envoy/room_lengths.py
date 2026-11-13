# Envoy room lengths from a loop_run.sh log (report only): per loop and fight, the seconds of live play each room took.
# Reads the sim's "room cleared, loop*10 + fight" / "room time, live frames" lines; older builds: vs-scene tick spans (include frozen picks).
import re, sys
L = open(sys.argv[1], encoding='utf-8', errors='replace').read().splitlines()
rows = []
live = [l for l in L if 'room time, live frames' in l]
if live:
    tag = None
    for l in L:
        m = re.search(r'room cleared, loop\*10 \+ fight\D*(\d+)', l)
        if m: tag = int(m.group(1))
        m = re.search(r'room time, live frames\D*(\d+)', l)
        if m and tag is not None:
            rows.append((tag // 10, tag % 10, int(m.group(1)), 'live')); tag = None
else:
    lp, fight, start = 1, None, None
    for l in L:
        m = re.search(r'play: tick (\d+): scene (\w+)', l)
        if m:
            t, sc = int(m.group(1)), m.group(2)
            if sc == 'vs': start = t
            elif start is not None and fight is not None:
                rows.append((lp, fight, t - start, 'scene')); start = None; fight = None
        m = re.search(r'match (\d+) rewritten', l)
        if m: fight = int(m.group(1))
        m = re.search(r'loop now (\d+)', l)
        if m: lp = int(m.group(1))
seen = set()
print('loop fight seconds (%s)' % (rows[0][3] if rows else '-'))
for lp, f, n, k in rows:
    if (lp, f, n) in seen: continue
    seen.add((lp, f, n)); print(f'{lp:4} {f:5} {n/60:7.1f}')
