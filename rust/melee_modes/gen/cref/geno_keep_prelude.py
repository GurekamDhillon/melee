"""Carry the hand-wired ext prelude of a committed Geno TU across a regeneration.

usage: geno_keep_prelude.py <previous tu .rs> <regenerated tu .rs>
Keeps the previous header/hook macros, each ext fn whose signature is unchanged (with its
`= crate::tu::adapt::...` default) and each ext static re-export; new or changed entries stay as generated."""
import re,sys
old=open(sys.argv[1]).read(); newp=sys.argv[2]; new=open(newp).read()
def split(s):
    a=s.index('    hooks! {\n')+len('    hooks! {\n')
    b=s.index('\n}\nuse ext::*;\n')
    hk_end=s.index('\n}\n',a)  # end of hooks block (column-0 '}')
    return s[:a], s[a:hk_end], s[hk_end+3:b], s[b:]
def entries(block):
    out=[];depth=0;cur=''
    for c in block:
        cur+=c
        if c in '([':depth+=1
        elif c in ')]':depth-=1
        elif c==';' and depth==0: out.append(cur.strip());cur=''
    return out
def name(e): return re.search(r'fn (\w+)',e)[1]
def sig(e): return re.sub(r'\s+','',e.split(' = ')[0].rstrip(';'))
import os
ADAPT=open(os.path.join(os.path.dirname(os.path.abspath(newp)),'adapt.rs')).read()
def tnorm(t): return re.sub(r'::core::ffi::c_int|c_int','i32',re.sub(r'\s+','',t))
def adapter(e):
    m=re.match(r'fn (\w+)\(([^)]*)\)\s*(?:->\s*(.+?))?;$',' '.join(e.split()))
    if not m: return None
    hits=[a for a in re.finditer(r'(?m)^pub fn ('+m[1]+r'__[0-9a-f]+)\(([^)]*)\)\s*(?:->\s*(.+?))?\s*\{',ADAPT)]
    want=(tnorm(m[2]).rstrip(','),tnorm(m[3] or ''))
    ok=[a[1] for a in hits if (tnorm(a[2]).rstrip(','),tnorm(a[3] or ''))==want]
    return ok[0] if len(ok)==1 else None
oh,ohk,odata,_=split(old); nh,nhk,ndata,ntail=split(new)
om={name(e):e for e in entries(ohk)}
res=[];kept=0;newf=[]
for e in entries(nhk):
    n=name(e);o=om.get(n)
    if o and sig(o)==sig(e): res.append(o);kept+=1
    else:
        ad=adapter(e)
        res.append(e[:-1]+' = crate::tu::adapt::'+ad+';' if ad else e);newf.append(n+('->'+ad if ad else ''))
od={}
for l in odata.strip('\n').split('\n'):
    m=re.search(r'(?:static mut |::)(\w+)(?::| *;)',l)
    if m: od[m[1]]=l
    else: print('old data line kept out:',l[:120])
dres=[]
for l in ndata.strip('\n').split('\n'):
    m=re.search(r'static mut (\w+):',l)
    dres.append(od.get(m[1],l) if m else l)
out=oh+'\n'.join(res)+'\n}\n'+'\n'.join(dres)+ntail
open(newp,'w').write(out)
print('kept',kept,'new/changed',newf)
print('dropped old',[n for n in om if n not in {name(e) for e in entries(nhk)}])
