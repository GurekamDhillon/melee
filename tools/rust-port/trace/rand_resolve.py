#!/usr/bin/env python3
"""Resolve a C-build `MELEE_RAND_TRACE=<file>` (rows `frame,caller,seed_after,global`, the C exe writes it, gw_replay.c) against
melee-pc.map: one line per script frame with the callers of the HSD_Rand* draws, consecutive repeats compressed.

  python tools/rust-port/trace/rand_resolve.py <melee-pc.map> <rand trace> [first frame [last frame]]

The script frame of match frame 0 is later than 0 (61 on FD, 468 on BF with the golden pad scripts): the first row of a frame
that holds the particle procs' draws is match frame 0. Draws before it are boot / stage init / fighter creation."""
import sys, bisect, re, collections
mp=sys.argv[1]; tr=sys.argv[2]
syms=[]
for l in open(mp,errors='replace'):
    m=re.match(r'\s*0001:[0-9a-f]+\s+(\S+)\s+([0-9a-f]{8})\s+(f|)\s',l)
    if m: syms.append((int(m.group(2),16),m.group(1)))
syms.sort(); addrs=[a for a,_ in syms]
def name(a):
    i=bisect.bisect_right(addrs,a)-1
    return syms[i][1].replace('_gw_','') if i>=0 else '?'
rows=[l.strip().split(',') for l in open(tr)]
per=collections.OrderedDict()
for fr,caller,seed,gl in rows:
    per.setdefault(int(fr),[]).append((name(int(caller,16)),seed))
lo=int(sys.argv[3]) if len(sys.argv)>3 else 0
hi=int(sys.argv[4]) if len(sys.argv)>4 else 10**9
for fr,v in per.items():
    if fr<lo or fr>hi: continue
    # compress consecutive identical names
    out=[];last=None;n=0
    for nm,_ in v:
        if nm==last: n+=1
        else:
            if last: out.append(f'{last}x{n}' if n>1 else last)
            last=nm;n=1
    out.append(f'{last}x{n}' if n>1 else last)
    print(fr,len(v),' '.join(out))
