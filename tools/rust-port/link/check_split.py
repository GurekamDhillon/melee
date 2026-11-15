# usage: cmp.py old_gen_table.rs new_src_dir_with_gen_table.rs   -> compares ordered hook statements + HOOKS + call order
import sys,re,glob,os
def old(p):
    t=open(p,encoding='utf8').read().replace('\r\n','\n')
    return t
def stmts(t):
    return [l for l in t.split('\n') if l.startswith('    //') or l.startswith('    unsafe {')]
def hooks(t):
    i=t.index('pub static HOOKS'); return t[i:]
a=old(sys.argv[1])
g=sys.argv[2]; d=os.path.join(os.path.dirname(g),'gen_table')
n=old(g)
order=re.findall(r'f(\d\d)::(c\d+)\(\)',n)
bodies={}
for f in sorted(glob.glob(d+'/f*.rs')):
    t=old(f); fi=os.path.basename(f)[1:3]
    for m in re.finditer(r'pub\(super\) fn (c\d+)\(\) \{\n(.*?)\n\}\n',t,re.S):
        bodies[(fi,m.group(1))]=m.group(2).split('\n')
new=[l for k in order for l in bodies[k]]
# every chunk fn called exactly once, in file order
assert len(order)==len(bodies)==len(set(order)), 'chunk set mismatch'
sa=stmts(a)
print('old stmts',len(sa),'new stmts',len(new),'chunks',len(order),'max hooks/chunk',max(sum(1 for l in b if l.startswith('    unsafe')) for b in bodies.values()))
assert sa==new,'STATEMENT SEQUENCE DIFFERS'
assert hooks(a)==hooks(n),'HOOKS DIFFERS'
print('IDENTICAL: ordered install statements and HOOKS table')
