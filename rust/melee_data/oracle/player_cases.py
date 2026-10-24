"""Case list for the player oracle: "s|name|flags" (synthetic corpus) and "r|relative/path|flags" (real
replays). usage: player_cases.py SYNTH_DIR REAL_LIST"""
import os, sys

synth, real_list = sys.argv[1], sys.argv[2]
sets = [
    "",
    "trace,rec,parity",
    "trace,rec,resync",
    "parity,badpost=-110",
    "parity,badseed=-100",
    "live,frames=40",
    "local=0,rec,frames=30",
    "codes=online,det=0,frozen,skin,rec,ucf=off,noucf073",
    "resim,trace,rec",
    "ucf=0,codes=tournament,det=1",
]
names = sorted(f for f in os.listdir(synth) if f.endswith(".slp"))
out = []
for i, n in enumerate(names):
    picks = [0, 1, 2] if i < 8 else [1 + (i % 9), 1 + ((i * 3) % 9)]
    for k in picks:
        out.append(f"s|{n}|{sets[k]}")
real = [l.strip() for l in open(real_list) if l.strip()]
for j in range(2, len(real), 20):
    out.append(f"r|{real[j]}|trace,rec,parity,frames=30000")
    out.append(f"r|{real[j]}|trace,rec,resync,frames=400")
print("\n".join(out))
