import re,os,glob,sys
sys.path.insert(0,'tools/rust-port/golden/coverage')
tab=open('rust/melee_link/src/gen_table.rs',encoding='utf8').read()
csrc={}
for root,_,files in os.walk('src/melee'):
    for fn in files:
        if fn.endswith('.c'):
            t=open(os.path.join(root,fn),encoding='utf8',errors='ignore').read()
            for m in re.finditer(r'^[\w\*\s]+?\b(\w+)\(([^)]*)\)\s*\n?\{',t,re.M):
                csrc.setdefault(m.group(1),m.group(2))
rs={}
for f in glob.glob('rust/melee_*/src/**/*.rs',recursive=True):
    if '/tests/' in f.replace(chr(92),'/'): continue
    s=open(f,encoding='utf8').read()
    for m in re.finditer(r'pub (?:unsafe )?(?:extern "C" )?fn (\w+)\(([^)]*)\)',s):
        rs.setdefault(m.group(1),[]).append(m.group(2))
def cls(t):
    t=t.replace(' ','')
    if 'GObj' in t or t in('G','*mutG'): return 'gobj'
    if re.search(r'(&|\*mut|\*const)(mut)?(Fighter|F)$',t): return 'fighter'
    if re.search(r'(&|\*mut|\*const)(mut)?(Item|It)$',t): return 'item'
    return None
def ccls(t):
    t=t.strip()
    if 'GObj' in t: return 'gobj'
    if re.match(r'(const\s+)?Fighter\s*\*',t): return 'fighter'
    if re.match(r'(const\s+)?Item\s*\*',t): return 'item'
    return None
for m in re.finditer(r'hook::(\w+) = Some\(\|[^|]*\| unsafe \{ ([\w:]+)\((.*?)\) \}\);',tab):
    cname,path,args=m.groups()
    fn=path.split('::')[-1]
    if cname not in csrc or fn not in rs: continue
    cp=[x.strip() for x in csrc[cname].split(',') if x.strip()]
    for sig in rs[fn][:1]:
        rp=[x.split(':',1)[1].strip() for x in re.split(r',(?![^<]*>)',sig) if ':' in x]
        for i,(a,b) in enumerate(zip(cp,rp)):
            ca,cb=ccls(a),cls(b)
            if ca and cb and ca!=cb: print(cname,'->',path,i,a,'|',b)
