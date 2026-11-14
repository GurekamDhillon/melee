import sys, collections
# summarise a MELEE_SFX_LOG: per reason counts and the sfx ids that never sounded
S=collections.Counter(); K=collections.Counter(); bad=collections.defaultdict(collections.Counter); ok=collections.Counter(); X=0
for l in open(sys.argv[1]):
    f=l.split()
    if len(f)<3: continue
    if f[1]=='S':
        S[f[-1]]+=1
        if f[-1]!='ok': bad[int(f[2])][f[-1]]+=1
    elif f[1]=='K':
        K[f[-1]]+=1
        fid=int(f[4])
        if f[-1]=='keyed': ok[fid]+=1
        else: bad[fid][f[-1]]+=1
    elif f[1]=='X': X+=1
print("starts", dict(S), "keyons", dict(K), "steals", X)
silent=[k for k in bad if ok[k]==0]
print("silent ids (never keyed):", len(silent))
for k in sorted(bad): print(f"  {k:7d} bank {k//10000 if k>=10000 else '-':>3}  {dict(bad[k])}  keyed {ok[k]}")
