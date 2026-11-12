# compare two MELEE_ASSET_DIGEST_DUMP dirs load by load. Word classes: 0 raw, 1 own block, 2 file, 3 exe, 4 arena,
# 5 host ptr (value 1: self-relative), 6 file image - block start; host words (5) are equal to each other; 0 vs 5/6 is equal iff the raw words match.
import sys,os,collections
import numpy as np
a,b=sys.argv[1],sys.argv[2]
fa=sorted(x for x in os.listdir(a) if x.endswith('.bin')); fb=sorted(x for x in os.listdir(b) if x.endswith('.bin'))
print('loads',len(fa),len(fb))
bad=0; kinds=collections.Counter(); ex=[]; hostpairs=0
def words(T):
    i=1+2*int(T[0]); out=[]
    while i<len(T):
        n=int(T[i]); i+=1; w=n//8
        out.append((n,T[i:i+2*w:2],T[i+1:i+2*w:2],T[i+2*w:i+2*w+2*(n-8*w)])); i+=2*w+2*(n-8*w)
    return out
for x,y in zip(fa,fb):
    A=np.fromfile(os.path.join(a,x),dtype='<u8'); B=np.fromfile(os.path.join(b,y),dtype='<u8')
    if x.split('_')[1]!=y.split('_')[1]: bad+=1; ex.append((x,y,'file differs')); continue
    if len(A)==len(B) and (A==B).all(): continue
    ne=int(A[0])
    if A[0]!=B[0] or not (A[1:1+2*ne]==B[1:1+2*ne]).all():
        bad+=1; ex.append((x,'events differ',int(A[0]),int(B[0]))); continue
    WA=words(A); WB=words(B)
    ra=np.fromfile(os.path.join(a,x[:-4]+'.raw'),dtype='<u8'); rb=np.fromfile(os.path.join(b,y[:-4]+'.raw'),dtype='<u8')
    ca_all=np.concatenate([w[1] for w in WA]) if WA else np.zeros(0,'<u8'); cb_all=np.concatenate([w[1] for w in WB]) if WB else np.zeros(0,'<u8')
    va_all=np.concatenate([w[2] for w in WA]) if WA else ca_all; vb_all=np.concatenate([w[2] for w in WB]) if WB else cb_all
    if [w[0] for w in WA]!=[w[0] for w in WB]: bad+=1; ex.append((x,'block sizes')); continue
    if not all((p[3]==q[3]).all() for p,q in zip(WA,WB)): bad+=1; ex.append((x,'tails')); continue
    ha=(ca_all==5); hb=(cb_all==5); xa=(ca_all>=3); xb=(cb_all>=3)
    rawA=va_all.copy(); rawA[xa]=ra; rawB=vb_all.copy(); rawB[xb]=rb   # class-0 raw value is the value itself
    same=((ca_all==cb_all)&(va_all==vb_all)) | (ha&hb)
    amb=~same & (((ca_all==0)&xb)|((cb_all==0)&xa))
    same|= (rawA==rawB)  # equal bytes are equal, however either run classified them
    hostpairs+=int((ha&hb).sum())
    d=np.nonzero(~same)[0]
    if len(d):
        bad+=1
        for k in d[:3]:
            kinds[(int(ca_all[k]),int(cb_all[k]))]+=1
            ex.append((x,'word',int(k),(int(ca_all[k]),int(cb_all[k])),hex(int(rawA[k])),hex(int(rawB[k]))))
        kinds['words']+=len(d)
print('loads with differences',bad,'| host-pointer word pairs (value not compared):',hostpairs); print(kinds)
for e in ex[:25]: print(e)
