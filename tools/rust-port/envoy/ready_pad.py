#!/usr/bin/env python
"""Envoy pad-script generator (the readiness checks, ready_checks.sh): title -> main menu -> the Envoy row -> the
camp -> (the Fighters row: the picker, main and partner by grid cell) -> Start run -> then Start / A alternately
(the picks, the shop, the road, the results). One line per segment: frames, buttons (hex), stick x, stick y.
  ready_pad.py OUT MAIN_CELL PARTNER_CELL [--nopick] [--n CELLS] [--gap 40] [--mid "line;line"] [--loop 1000,0100]
               [--segs 8000] [--ace]
--nopick: Start run with the last party (Fox + Marth), the picker is not opened. --ace: the ACE disc's boot (no
opening movie). --n: the picker's cell count (24 on the retail disc: 23 fighters + Random). --mid: extra segments
after the run's start, before the Start / A loop.
Retail grid (9 columns): 0 Dr. Mario, 1 Mario, 2 Luigi, 3 Bowser, 4 Peach, 5 Yoshi, 6 DK, 7 Falcon, 8 Ganondorf,
9 Falco, 10 Fox, 11 Ness, 12 Kirby, 13 Samus, 14 Link, 15 Young Link, 16 Pichu, 17 Pikachu, 18 Jigglypuff,
19 Mewtwo, 20 Game & Watch, 21 Marth, 22 Roy, 23 Random; m-ex fighters follow the retail ones on an m-ex disc.
"""
import sys, argparse, collections
ap = argparse.ArgumentParser(); ap.add_argument('out'); ap.add_argument('main', type=int); ap.add_argument('partner', type=int)
ap.add_argument('--n', type=int, default=24); ap.add_argument('--gap', type=int, default=40)
ap.add_argument('--start', type=int, default=10); ap.add_argument('--pstart', type=int, default=21)
ap.add_argument('--mid', default='')   # extra lines (; separated) after the picker, before the mash loop
ap.add_argument('--loop', default='1000,0100')  # buttons alternated in the tail
ap.add_argument('--segs', type=int, default=8000); ap.add_argument('--ace', action='store_true'); ap.add_argument('--nopick', action='store_true')
a = ap.parse_args()
COLS = 9
def step(c, d, n):
    if d == 'L': return min(c + COLS - 1, n - 1) if c % COLS == 0 else c - 1
    if d == 'R': return c - c % COLS if (c % COLS == COLS - 1 or c + 1 >= n) else c + 1
    if d == 'U': return c - COLS if c >= COLS else min(c + (n // COLS) * COLS, n - 1)
    if d == 'D': return c + COLS if c + COLS < n else c % COLS
def path(a_, b, n):
    q = collections.deque([(a_, '')]); seen = {a_}
    while q:
        c, p = q.popleft()
        if c == b: return p
        for d in 'LRUD':
            x = step(c, d, n)
            if x not in seen: seen.add(x); q.append((x, p + d))
    raise SystemExit('no path')
BTN = {'L': '0001', 'R': '0002', 'D': '0004', 'U': '0008'}
L = ['300 0000 0 0', '3 1000 0 0', '200 0000 0 0', '3 1000 0 0', '60 0000 0 0', '3 0004 0 0', '20 0000 0 0', '3 0004 0 0',
     '60 0000 0 0', '3 0100 0 0', '60 0000 0 0', '3 1000 0 0', '280 0000 0 0']
if a.ace:  # the ACE disc has no opening movie: the title is up at once, one Start reaches the menu
    L = ['300 0000 0 0', '3 1000 0 0', '260 0000 0 0', '3 0004 0 0', '20 0000 0 0', '3 0004 0 0',
         '60 0000 0 0', '3 0100 0 0', '60 0000 0 0', '3 1000 0 0', '280 0000 0 0']
def nav(p):
    for d in p: L.extend([f'3 {BTN[d]} 0 0', '10 0000 0 0'])
if not a.nopick:
    # drop the camp's Start (the last two header lines), open the Fighters row, pick, Done, then Start run
    L = L[:-2]
    L.extend(['3 0004 0 0', '20 0000 0 0', '3 0100 0 0', '40 0000 0 0'])
    nav(path(a.start, a.main, a.n)); L.extend(['3 0100 0 0', '30 0000 0 0'])
    nav(path(a.pstart, a.partner, a.n)); L.extend(['3 0100 0 0', '30 0000 0 0', '3 0100 0 0', '90 0000 0 0'])
    L.extend(['3 1000 0 0', '280 0000 0 0'])
for x in a.mid.split(';'):
    if x.strip(): L.append(x.strip())
loop = a.loop.split(',')
i = 0
while len(L) < a.segs:
    L.extend([f'3 {loop[i % len(loop)]} 0 0', f'{a.gap} 0000 0 0']); i += 1
open(a.out, 'w').write('# ready_pad.py: main cell %d partner cell %d\n' % (a.main, a.partner) + '\n'.join(L) + '\n')
