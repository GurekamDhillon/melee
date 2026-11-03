# generates the CSS part of the melee-unlocked scenario script
import sys
# per port: (char moves as list of (axis, sign, frames))
plans = {
 1: [('sy',1,10),('sx',1,5)],      # Fox
 2: [('sy',1,4),('sx',1,22)],      # Marth
 3: [('sy',1,10),('sx',-1,24)],    # Falco
 4: [('sy',1,15),('sx',-1,15)],    # Peach
}
ports = [int(c) for c in sys.argv[1]] if len(sys.argv)>1 else [1,2,3,4]
level_dx = int(sys.argv[2]) if len(sys.argv)>2 else 10
ev = {}  # frame -> list of lines
def add(f, line): ev.setdefault(f, []).append(line)
def p(k): return '' if k==1 else f' p={k}'
T=1900
KX={4:int(sys.argv[3]) if len(sys.argv)>3 else 1}
def moves(k, t, seq):
    for ax, s, n in seq:
        add(t, f'{t}{p(k)} {ax}={127*s}'); t += n; add(t, f'{t}{p(k)}'); t += 4
    return t
for k in ports:
    t = T
    if k != 1: add(t, f'{t}{p(k)} A'); add(t+6, f'{t+6}{p(k)}')
    t += 20
    t = moves(k, t, plans[k])
    add(t, f'{t}{p(k)} A'); add(t+6, f'{t+6}{p(k)}'); t += 20
    back = [(ax, -s, n) for ax, s, n in reversed(plans[k])]
    t = moves(k, t, back)
    add(t, f'{t}{p(k)} A'); add(t+6, f'{t+6}{p(k)}'); t += 20   # HMN -> CPU
    t = moves(k, t, [('sy',-1,10),('sx',-1,KX.get(k,1))])
    add(t, f'{t}{p(k)} A'); add(t+6, f'{t+6}{p(k)}'); t += 12
    t = moves(k, t, [('sx',1,level_dx)])
    add(t, f'{t}{p(k)} A'); add(t+6, f'{t+6}{p(k)}'); t += 12
for f in sorted(ev):
    for l in ev[f]: print(l)
