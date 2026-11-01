#!/usr/bin/env python3
"""Link-cast check: tu bodies the link installs (gen_table.rs) whose parameter is a Fighter/Item/Ground reference or pointer where the C definition takes a GObj (or the reverse). The link `cast` transmutes same-size pointers, so such a mismatch reads garbage at run time. Run from the repo root; prints the offenders (0 expected)."""
import re,glob,os
tab=open('rust/melee_link/src/gen_table.rs',encoding='utf8').read()
csrc={}
for root,_,files in os.walk('src/melee'):
    for fn in files:
        if fn.endswith('.c'):
            t=open(os.path.join(root,fn),encoding='utf8',errors='ignore').read()
            for m in re.finditer(r'^[\w\*\s]+?\b(\w+)\(([^)]*)\)\s*\n?\{',t,re.M):
                csrc.setdefault(m.group(1),m.group(2))
def splitargs(a):
    out=[];d=0;cur=''
    for ch in a:
        if ch in '(<[': d+=1
        if ch in ')>]': d-=1
        if ch==',' and d==0: out.append(cur);cur=''
        else: cur+=ch
    if cur.strip(): out.append(cur)
    return [x.strip() for x in out]
bad=[]
for f in glob.glob('rust/melee_*/src/**/*.rs',recursive=True):
    if '/tests/' in f.replace(chr(92),'/'): continue
    s=open(f,encoding='utf8').read()
    parts=f.replace(chr(92),'/').split('/')
    crate=parts[1]
    if '/tu/' not in f.replace(chr(92),'/'): continue
    mod=parts[-1][:-3]
    for m in re.finditer(r'pub (?:unsafe )?(?:extern "C" )?fn (\w+)\(([^)]*)\)',s):
        name,args=m.groups()
        key=f'{crate}::tu::{mod}::{name}('
        if key not in tab: continue
        a=csrc.get(name)
        if a is None: continue
        ca=splitargs(a); ra=splitargs(args)
        for i,(r,c) in enumerate(zip(ra,ca)):
            if re.search(r':\s*(&(mut )?|\*(mut|const) )(Fighter|Item|Ground|ftData)\b',r) and re.search(r'GObj',c):
                bad.append((f.split('/')[-1],name,i,r,c)); break
            if re.search(r':\s*(&(mut )?|\*(mut|const) )(HSD_GObj)\b',r) and re.search(r'(Fighter|Item|Ground)\s*\*',c) and 'GObj' not in c:
                bad.append((f.split('/')[-1],name,i,r,c)); break
print(len(bad))
for b in bad: print(b)
