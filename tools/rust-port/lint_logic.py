#!/usr/bin/env python3
"""Logic lint: game logic lives only in rust/*/src/tu/ (consolidation.md section 5).

  python tools/rust-port/lint_logic.py --report            counts per rule and crate, exit 0
  python tools/rust-port/lint_logic.py                      fail (exit 1) on a violation not in lint_baseline.txt
  python tools/rust-port/lint_logic.py --write-baseline [--allow-growth]
  python tools/rust-port/lint_logic.py --dedupe-table       print the canonical home / extra bodies per C name
  python tools/rust-port/lint_logic.py --list RULE          print every violation of one rule (R1..R5)

Rules (reported as `rule<TAB>path<TAB>name`, paths relative to the repo root):
  R1 logic outside tu   a fn named like a C-defined function, with a real body, in a file outside */src/tu/ and not
                        allowlisted (lint_allow.txt)
  R2 duplicate body     the same C name has a body in more than one file; every body except the canonical one
                        (the tu file named after the C file; else the first tu file, else the first file)
  R3 wrong home         a C function's only body is in a tu file whose name is not the C file's stem
  R4 hook default       an ext_fns! entry that is more than `todo!`, a path, or a closure that calls into tu/melee_link,
                        for a name that is not a host seam
  R5 driver logic       in allowlisted glue files (rust/melee/src/*), a write to a Fighter/Item/Ground field or an
                        HSD_Rand draw (heuristic regex)
The C definition index is every function defined in src/melee/**/*.c, src/sysdolphin/**/*.c and pc/gameworld/*.c, minus
static names defined in more than one file. Skipped bodies: declarations, `todo!`/`unimplemented!` in the first five
lines, hook forwarders. Rust files under raw/, c2r/, tests/ and gen.rs are not scanned."""
import re, sys, fnmatch, argparse
from pathlib import Path
from collections import defaultdict, Counter

REPO = Path(__file__).resolve().parents[2]
ALLOW = REPO / 'tools/rust-port/lint_allow.txt'
BASE = REPO / 'tools/rust-port/lint_baseline.txt'
SEAMS = ('lbAudioAx_', 'HSD_DVD', 'DVDRead', 'DVDOpen', 'GXDraw', 'GX', 'OSReport', 'ScriptGame_', 'Mex_', 'Geno_', 'OS')
KW = {'if', 'while', 'for', 'switch', 'return', 'sizeof', 'else', 'do', 'defined'}


def strip_c(t):
    t = re.sub(r'/\*.*?\*/', lambda m: re.sub(r'[^\n]', ' ', m.group(0)), t, flags=re.S)
    t = re.sub(r'//[^\n]*', '', t)
    return re.sub(r'^[ \t]*#(?:[^\n]*\\\n)*[^\n]*', lambda m: re.sub(r'[^\n]', ' ', m.group(0)), t, flags=re.M)


CDEF = re.compile(r'^(?P<pre>[A-Za-z_][\w \t\*]*?)\b(?P<name>[A-Za-z_]\w*)\s*\((?P<args>[^;{}]*?)\)\s*\{', re.M)


def c_index():
    """lower(name) -> (name, c file stem); static duplicates dropped."""
    defs = defaultdict(list)
    files = list((REPO / 'src/melee').rglob('*.c')) + list((REPO / 'src/sysdolphin').rglob('*.c')) + list((REPO / 'pc/gameworld').glob('*.c'))
    for f in files:
        try:
            t = strip_c(f.read_text(errors='replace'))
        except OSError:
            continue
        for m in CDEF.finditer(t):
            n = m.group('name')
            if n in KW or m.group('pre').strip() in ('', 'else', 'return'):
                continue
            defs[n].append((f.stem, 'static' in m.group('pre').split()))
    out = {}
    for n, v in defs.items():
        stems = {s for s, _ in v}
        if len(stems) > 1 and all(st for _, st in v):
            continue  # static name shared by several files
        out[n.lower()] = (n, sorted(stems)[0])
    return out


def allow_patterns():
    pats = []
    if ALLOW.exists():
        for l in ALLOW.read_text().splitlines():
            l = l.strip()
            if not l or l.startswith('#'):
                continue
            pats.append(l.split()[0])
    return pats


def expand_braces(p):
    m = re.search(r'\{([^{}]*)\}', p)
    if not m:
        return [p]
    out = []
    for alt in m.group(1).split(','):
        out += expand_braces(p[:m.start()] + alt + p[m.end():])
    return out


def allowed(rel, pats):
    return any(fnmatch.fnmatch(rel, q) for p in pats for q in expand_braces(p))


FN = re.compile(r'^[ \t]*(?:pub(?:\([a-z]+\))?[ \t]+)?(?:const[ \t]+)?(?:unsafe[ \t]+)?(?:extern[ \t]+"C"[ \t]+)?fn[ \t]+([A-Za-z_]\w*)', re.M)


def rust_files():
    for f in sorted((REPO / 'rust').glob('*/src/**/*.rs')):
        rel = f.relative_to(REPO).as_posix()
        parts = rel.split('/')
        if any(x in parts for x in ('raw', 'c2r', 'tests')) or f.name == 'gen.rs':
            continue
        yield f, rel


def body_of(t, pos):
    """(start, end) of the braces of the fn whose keyword match begins at pos, or None for a declaration."""
    i = t.index('(', pos)
    d = 0
    while i < len(t):
        if t[i] == '(':
            d += 1
        elif t[i] == ')':
            d -= 1
            if d == 0:
                break
        i += 1
    j = i + 1
    while j < len(t) and t[j] not in '{;':
        j += 1
    if j >= len(t) or t[j] == ';':
        return None
    d = 0
    k = j
    while k < len(t):
        c = t[k]
        if c == '{':
            d += 1
        elif c == '}':
            d -= 1
            if d == 0:
                return j, k
        k += 1
    return j, len(t) - 1


def trivial(body, name=None):
    b = re.sub(r'melee_core::ct!\("[^"]*"\);', '', body)
    b = re.sub(r'//[^\n]*', '', b).strip()
    head = '\n'.join(b.split('\n')[:5])
    if 'todo!(' in head or 'unimplemented!(' in head:
        return True
    if b == '':
        return False
    if 'match unsafe { HOOK }' in b or re.search(r'Some\(f\) => f\(', b):
        return True
    flat = ' '.join(b.split())
    if len(b.split('\n')) <= 4 and re.search(r'\b(crate|melee_\w+)::tu::', flat) and not re.search(r'\b(if|match|while|for|loop|let)\b', flat):
        return True
    if re.search(r'panic!\("TODO\(', flat) and len(flat) < 900:
        return True   # an unported stub
    if re.search(r'HOOKS\.read\(\)', flat) and len(flat) < 300 and not re.search(chr(92)+'b(if|match|while|for|loop)'+chr(92)+'b', flat):
        return True   # a hook-table forwarder
    if name:
        # a forwarder: after removing the optional `if let Some(h) = HOOK { .. }` preamble, the body only calls a tu path of the
        # same name (or another crate's tu function of that name), with no loop / match / branch of its own
        f2 = re.sub(r'if let Some\(h\) = (?:unsafe \{ )?HOOK(?: \})? \{.*?\}\s*(?=\{|$)', '', flat)
        if len(f2) < 700 and re.search(r'(?:crate|melee_\w+|super)(?:::\w+)*::' + re.escape(name) + r'\s*\(', f2, re.I) \
                and not re.search(r'\b(if|match|while|for|loop)\b', f2):
            return True
    return False


def scan(pats):
    cidx = c_index()
    bodies = defaultdict(list)    # lower C name -> [(rel, line, in_tu, name)]
    r4, r5 = [], []
    for f, rel in rust_files():
        t = f.read_text(errors='replace')
        in_tu = '/src/tu/' in rel
        for m in FN.finditer(t):
            n = m.group(1)
            if n.lower() not in cidx:
                continue
            b = body_of(t, m.start())
            if b is None or trivial(t[b[0] + 1:b[1]], n):
                continue
            bodies[n.lower()].append((rel, t.count('\n', 0, m.start()) + 1, in_tu, n))
        # R4: ext_fns! entries
        for blk in re.finditer(r'ext_fns!\s*\{', t):
            d, k = 0, blk.end() - 1
            while k < len(t):
                if t[k] == '{':
                    d += 1
                elif t[k] == '}':
                    d -= 1
                    if d == 0:
                        break
                k += 1
            text = t[blk.end():k]
            for e in re.split(r';[ \t]*\n', text):
                m2 = re.match(r'\s*([A-Za-z_]\w*)\s*\(.*?\)\s*(?:->\s*[^=]+?)?=\s*(.*)$', e, re.S)
                if not m2:
                    continue
                name, rhs = m2.group(1), m2.group(2).strip()
                if name.startswith(SEAMS) or 'todo!(' in rhs or 'unimplemented!(' in rhs:
                    continue
                if re.fullmatch(r'[\w:]+', rhs):
                    continue
                flat = ' '.join(rhs.split())
                if rhs.startswith('|') and re.search(r'\b(tu|melee_link)::', flat) and not re.search(r'\b(if|match|while|for|loop)\b', flat):
                    continue
                r4.append((rel, name))
        # R5: glue-file driver logic
        if allowed(rel, pats) and rel.startswith('rust/melee/src/'):
            for i, line in enumerate(t.split('\n'), 1):
                s = line.strip()
                if s.startswith('//'):
                    continue
                if re.search(r'\.(fp|ip|gp)\.[a-z_0-9]+\s*=[^=]', s) or re.search(r'\brand\.(randi|randf|rand)\b', s):
                    r5.append((rel, 'line%d' % i))
    return cidx, bodies, r4, r5


def stem_ok(rel, cstem):
    return Path(rel).stem.lower() == cstem.lower()


def violations(pats):
    cidx, bodies, r4, r5 = scan(pats)
    v = []
    canon = {}
    for ln, bs in bodies.items():
        cname, cstem = cidx[ln]
        tu = [b for b in bs if b[2]]
        pick = next((b for b in tu if stem_ok(b[0], cstem)), None) or (tu[0] if tu else bs[0])
        canon[ln] = (cname, pick[0])
        files = {}
        for b in bs:
            files.setdefault(b[0], b)
        for rel, b in files.items():
            if not b[2] and not allowed(rel, pats):
                v.append(('R1', rel, cname))
            if rel != pick[0]:
                v.append(('R2', rel, cname))
        if len(files) == 1 and pick[2] and not stem_ok(pick[0], cstem):
            v.append(('R3', pick[0], cname))
    for rel, n in r4:
        v.append(('R4', rel, n))
    for rel, n in r5:
        v.append(('R5', rel, n))
    return sorted(set(v)), canon, bodies


def crate_of(rel):
    return rel.split('/')[1]


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('--report', action='store_true')
    ap.add_argument('--write-baseline', action='store_true')
    ap.add_argument('--allow-growth', action='store_true')
    ap.add_argument('--dedupe-table', action='store_true')
    ap.add_argument('--list')
    a = ap.parse_args()
    pats = allow_patterns()
    v, canon, bodies = violations(pats)
    if a.dedupe_table:
        by = defaultdict(list)
        for ln, bs in bodies.items():
            files = {b[0] for b in bs}
            if len(files) > 1:
                cname, home = canon[ln]
                by[home].append((cname, sorted(files - {home})))
        for home in sorted(by):
            print('- `%s` (%d): ' % (home, len(by[home])) + '; '.join('%s [%s]' % (n, ', '.join(x)) for n, x in sorted(by[home])))
        return 0
    if a.list:
        for r, p, n in v:
            if r == a.list:
                print('%s\t%s\t%s' % (r, p, n))
        return 0
    base = set()
    if BASE.exists():
        base = {tuple(l.split('\t')) for l in BASE.read_text().splitlines() if l.strip() and not l.startswith('#')}
    if a.report:
        c = Counter(r for r, _, _ in v)
        dup = sum(1 for bs in bodies.values() if len({b[0] for b in bs}) > 1)
        print('rule counts: ' + ', '.join('%s=%d' % (r, c[r]) for r in ('R1', 'R2', 'R3', 'R4', 'R5')))
        print('C names with bodies in more than one file: %d (of %d C names with a Rust body)' % (dup, len(bodies)))
        per = Counter((r, crate_of(p)) for r, p, _ in v)
        for (r, cr), n in sorted(per.items()):
            print('  %s %-16s %d' % (r, cr, n))
        print('baseline: %d entries; new violations: %d' % (len(base), len(set(v) - base)))
        return 0
    if a.write_baseline:
        new = set(v)
        if base and len(new) > len(base) and not a.allow_growth:
            print('lint: refusing to grow the baseline (%d -> %d); pass --allow-growth (CI never does)' % (len(base), len(new)), file=sys.stderr)
            return 1
        BASE.write_text('# rule<TAB>path<TAB>name; violations accepted until the step that removes them (consolidation.md 5)\n' + ''.join('\t'.join(x) + '\n' for x in sorted(new)), newline='\n') if False else None
        with open(BASE, 'w', newline='\n') as fh:
            fh.write('# rule<TAB>path<TAB>name; violations accepted until the step that removes them (consolidation.md 5)\n')
            for x in sorted(new):
                fh.write('\t'.join(x) + '\n')
        print('lint: wrote %d baseline entries' % len(new))
        return 0
    new = sorted(set(v) - base)
    if new:
        for r, p, n in new[:60]:
            print('lint: new %s %s %s' % (r, p, n))
        print('lint: %d violation(s) not in lint_baseline.txt' % len(new))
        return 1
    print('lint OK (%d baselined violations)' % len(v))
    return 0


if __name__ == '__main__':
    sys.exit(main())
