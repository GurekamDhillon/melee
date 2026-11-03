#!/usr/bin/env python3
"""Triage the todo!()s of melee_modes/melee_runtime tu/ files: an ext_fns! default whose hook some non-test file
installs (`<crate>::tu::<stem>::ext::hook::<name> = Some`, also through `use ...::ext::hook as h`) is "hooked" (the
default never runs once the engine is up); EXT-UNHOOKED and BODY lines need a reachability check by hand.
  python tools/rust-port/link/todo_triage.py <repo-root>   (prints the non-hooked ones, counts on stderr)"""
import re,sys,glob,os
R=sys.argv[1]
src=''
for f in [g for g in glob.glob(R+'/rust/**/*.rs',recursive=True) if '/tests/' not in g.replace(chr(92),'/') and '/benches/' not in g.replace(chr(92),'/')]:
        src+=open(f,encoding='latin-1').read()
inst=set(re.findall(r'(\w+)::tu::(\w+)::ext::hook::(\w+)\s*=\s*Some',src))
for f in [g for g in glob.glob(R+'/rust/**/*.rs',recursive=True) if '/tests/' not in g.replace(chr(92),'/') and '/benches/' not in g.replace(chr(92),'/')]:
    s=open(f,encoding='latin-1').read()
    for m in re.finditer(r'use\s+(\w+)::tu::(\w+)::ext::hook\s+as\s+(\w+)',s):
        for n in re.findall(r'(?<![A-Za-z0-9_:])'+m.group(3)+r'::(\w+)\s*=\s*Some',s): inst.add((m.group(1),m.group(2),n))

tot={}
for crate in ['melee_modes','melee_runtime']:
    for f in sorted(glob.glob(f'{R}/rust/{crate}/src/tu/*.rs')):
        stem=os.path.basename(f)[:-3]
        t=open(f,encoding='latin-1').read()
        if 'todo!(' not in t: continue
        # ext block spans
        spans=[]
        for m in re.finditer(r'ext_fns!\s*\{',t):
            d=0;k=m.end()-1
            while k<len(t):
                if t[k]=='{':d+=1
                elif t[k]=='}':
                    d-=1
                    if d==0:break
                k+=1
            spans.append((m.end(),k))
        for m in re.finditer(r'todo!\(',t):
            p=m.start(); line=t.count('\n',0,p)+1
            ls=t.rfind('\n',0,p)+1; le=t.find('\n',p); L=t[ls:le].strip()
            inext=any(a<=p<=b for a,b in spans)
            if inext:
                nm=re.match(r'(\w+)\s*\(',L) or re.match(r'(\w+)\s*\(',t[t.rfind(chr(10),0,ls-1)+1:ls].strip())
                nm=nm.group(1) if nm else '?'
                k='hooked' if (crate,stem,nm) in inst else 'EXT-UNHOOKED'
            else:
                k='BODY'; nm=L[:100]
            tot[(crate,k)]=tot.get((crate,k),0)+1
            if k!='hooked': print(k,crate,stem,line,nm,sep='\t')
print(tot,file=sys.stderr)
