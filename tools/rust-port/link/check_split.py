# usage: check_split.py old_gen_table.rs rust_dir
#   compares the ordered hook statements + HOOKS table of an old single-file gen_table.rs against the split layout:
#   rust_dir/melee_link/src/gen_table.rs (install_all -> melee_link_p<N>::install() in order) and each part crate's
#   lib.rs (install -> f<NN>::c<NNN>() in order) and src/f<NN>.rs chunk bodies.
import sys, re, glob, os
rd = lambda p: open(p, encoding='utf8').read().replace('\r\n', '\n')
def stmts(t):
    return [l for l in t.split('\n') if l.startswith('    //') or l.startswith('    unsafe {')]
def hooks(t):
    i = t.index('pub static HOOKS'); return t[i:]
a = rd(sys.argv[1])
rust = sys.argv[2]
n = rd(os.path.join(rust, 'melee_link', 'src', 'gen_table.rs'))
parts = re.findall(r'^    melee_link_p(\d+)::install\(\);$', n, re.M)
assert parts == [str(i) for i in range(len(parts))] and len(parts) == len(glob.glob(os.path.join(rust, 'melee_link_p*'))), 'part set/order mismatch'
new, nchunks, maxh, per = [], 0, 0, []
for p in parts:
    pd = os.path.join(rust, f'melee_link_p{p}', 'src')
    order = re.findall(r'^    f(\d\d)::c(\d+)\(\);$', rd(os.path.join(pd, 'lib.rs')), re.M)
    bodies = {}
    for f in glob.glob(os.path.join(pd, 'f*.rs')):
        fi = os.path.basename(f)[1:3]
        for m in re.finditer(r'pub\(super\) fn c(\d+)\(\) \{\n(.*?)\n\}\n', rd(f), re.S):
            bodies[(fi, m.group(1))] = m.group(2).split('\n')
    assert len(order) == len(bodies) == len(set(order)) and set(order) == set(bodies), f'part {p}: chunk set mismatch'
    ph = 0
    for k in order:
        new += bodies[k]; h = sum(1 for l in bodies[k] if l.startswith('    unsafe')); ph += h; maxh = max(maxh, h)
    nchunks += len(order); per.append(ph)
sa = stmts(a)
print('old stmts', len(sa), 'new stmts', len(new), 'parts', len(parts), 'hooks/part', per, 'chunks', nchunks, 'max hooks/chunk', maxh)
assert sa == new, 'STATEMENT SEQUENCE DIFFERS'
assert hooks(a) == hooks(n), 'HOOKS DIFFERS'
print('IDENTICAL: ordered install statements and HOOKS table')
