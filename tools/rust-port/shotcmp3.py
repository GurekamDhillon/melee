import sys, numpy as np
from PIL import Image
rd=sys.argv[1]
def L(p): return np.asarray(Image.open(p).convert("RGB").resize((640,480),Image.BOX)).astype(int)
tot=[]
for f in (60,120,300,600,900,1200,1500):
    c=L(f"cshots3/c_{f:04d}.png"); r=L(f"{rd}/native_{f:04d}.png")
    d=np.abs(c-r); m=d.mean(); w=(d.max(axis=2)<=16).mean()*100
    tot.append(m); print(f"frame {f:5d}: mean abs {m:6.2f}  within16 {w:5.1f}%")
print("avg",round(sum(tot)/len(tot),2))
