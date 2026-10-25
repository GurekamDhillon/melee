import sys, numpy as np
from PIL import Image
cd,rd=sys.argv[1],sys.argv[2]
fr=[int(x) for x in sys.argv[3].split(',')]
def L(p): return np.asarray(Image.open(p).convert("RGB").resize((640,480),Image.BOX)).astype(int)
tot=[]
for f in fr:
    c=L(f"{cd}/c_{f:04d}.png"); r=L(f"{rd}/native_{f:04d}.png")
    d=np.abs(c-r); m=d.mean(); w=(d.max(axis=2)<=16).mean()*100
    tot.append(m); print(f"frame {f:5d}: mean abs {m:6.2f}  within16 {w:5.1f}%")
    if len(sys.argv)>4:
        Image.fromarray(np.concatenate([c,r,np.clip(d*4,0,255)],axis=1).astype('uint8')).save(f"{sys.argv[4]}_{f}.png")
print("avg",round(sum(tot)/len(tot),2))
