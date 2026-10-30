#!/usr/bin/env python3
"""Lint: game logic only in tu/ (docs/rust-port/notes/consolidation.md section 5).

Rules: R1 logic outside tu, R2 duplicate body, R3 wrong home, R4 hook default logic, R5 driver logic.
Modes: --report (counts, exit 0), default (fail on violations not in lint_logic_baseline.txt),
--write-baseline [--allow-growth], --dedupe-table (Appendix A), --json OUT (machine-readable dump for tools).
"""
import argparse, collections, fnmatch, json, os, re, sys

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), '..', '..'))
HERE = os.path.dirname(os.path.abspath(__file__))
ALLOW = os.path.join(HERE, 'lint_logic_allow.txt')
BASELINE = os.path.join(HERE, 'lint_logic_baseline.txt')

CRATE_OF_DIR = [
    ('src/melee/ft/kinds/', 'melee_chara'),
    ('src/melee/ft/', 'melee_fighter'), ('src/melee/pl/', 'melee_fighter'),
    ('src/melee/it/', 'melee_items'), ('src/melee/ty/', 'melee_items'), ('src/melee/vi/', 'melee_items'),
    ('src/melee/gr/', 'melee_world'), ('src/melee/mp/', 'melee_world'), ('src/melee/cm/', 'melee_world'),
    ('src/melee/ef/', 'melee_world'), ('src/melee/if/', 'melee_world'),
    ('src/melee/gm/', 'melee_modes'), ('src/melee/mn/', 'melee_modes'),
    ('src/melee/sfx/', 'melee_fighter'), ('src/melee/lb/', 'melee_runtime'), ('src/melee/db/', 'melee_runtime'), ('src/sysdolphin/', 'melee_runtime'),
]


def blank_code(s):
    """Replace comments, strings and char literals by spaces (keeps offsets and newlines)."""
    out = list(s)
    i, n = 0, len(s)
    while i < n:
        c = s[i]
        if c == '/' and i + 1 < n and s[i + 1] == '/':
            j = s.find('\n', i)
            j = n if j < 0 else j
            for k in range(i, j): out[k] = ' '
            i = j
        elif c == '/' and i + 1 < n and s[i + 1] == '*':
            depth, j = 1, i + 2
            while j < n and depth:
                if s.startswith('/*', j): depth += 1; j += 2
                elif s.startswith('*/', j): depth -= 1; j += 2
                else: j += 1
            for k in range(i, j):
                if out[k] != '\n': out[k] = ' '
            i = j
        elif c == 'r' and re.match(r'r#*"', s[i:i + 8]) and (i == 0 or not (s[i - 1].isalnum() or s[i - 1] == '_')):
            m = re.match(r'r(#*)"', s[i:])
            end = '"' + m.group(1)
            j = s.find(end, i + len(m.group(0)))
            j = n if j < 0 else j + len(end)
            for k in range(i, j):
                if out[k] != '\n': out[k] = ' '
            i = j
        elif c == '"':
            j = i + 1
            while j < n and s[j] != '"':
                j += 2 if s[j] == '\\' else 1
            j = min(n, j + 1)
            for k in range(i, j):
                if out[k] != '\n': out[k] = ' '
            i = j
        elif c == "'":
            m = re.match(r"'(?:\\(?:x[0-9a-fA-F]{2}|u\{[0-9a-fA-F]+\}|.)|[^\\'])'", s[i:i + 14])
            if m:
                for k in range(i, i + len(m.group(0))): out[k] = ' '
                i += len(m.group(0))
            else:
                i += 1
        else:
            i += 1
    return ''.join(out)


FN_RE = re.compile(r'^[ \t]*(?:pub(?:\([a-z: ]+\))?[ \t]+)?(?:const[ \t]+)?(?:unsafe[ \t]+)?(?:extern[ \t]+"C"[ \t]+)?fn[ \t]+([A-Za-z_]\w*)', re.M)


def find_bodies(src, blank):
    """Yield (name, line, start, end_of_sig, body_start, body_end) for each fn with a body."""
    depth_at = {}
    pos, d = 0, 0
    ms = list(FN_RE.finditer(blank))
    for m in ms:
        d += blank.count('{', pos, m.start()) - blank.count('}', pos, m.start())
        pos = m.start()
        depth_at[m.start()] = d
    for m in ms:
        name = m.group(1)
        i = m.end()
        # skip generics / params: find first '(' then matching ')'
        j = blank.find('(', i)
        if j < 0: continue
        depth = 0
        k = j
        n = len(blank)
        while k < n:
            ch = blank[k]
            if ch == '(': depth += 1
            elif ch == ')':
                depth -= 1
                if depth == 0: break
            k += 1
        # after params: find '{' or ';' at depth 0 (ignore braces inside <>/where for simplicity)
        p = k + 1
        while p < n and blank[p] not in '{;':
            p += 1
        if p >= n or blank[p] == ';': continue
        if '=' in blank[k + 1:p].replace('->', '').replace('=>', '='): continue  # ext_fns! entry `fn f(..) -> R = |..| {..}`
        depth, q = 0, p
        while q < n:
            ch = blank[q]
            if ch == '{': depth += 1
            elif ch == '}':
                depth -= 1
                if depth == 0: break
            q += 1
        line = src.count('\n', 0, m.start(1)) + 1
        yield name, line, m.start(), p, p + 1, q, depth_at[m.start()]


def c_index():
    """name -> list of (relpath, crate, tu_stem, is_static) for C definitions."""
    defs = collections.defaultdict(list)
    roots = [os.path.join(ROOT, 'src', 'melee'), os.path.join(ROOT, 'src', 'sysdolphin'), os.path.join(ROOT, 'pc', 'gameworld')]
    rx = re.compile(r'^(?P<pre>[A-Za-z_][\w \t\*]*?)\b(?P<name>[A-Za-z_]\w*)[ \t]*\((?P<args>[^;{}()]*(?:\([^;{}()]*\)[^;{}()]*)*)\)[ \t]*\r?\n?[ \t]*\{', re.M)
    for r in roots:
        for dp, _, fs in os.walk(r):
            for f in fs:
                if not f.endswith('.c') or f.endswith('_tests.c'): continue  # LAB test harness TUs, not game logic
                p = os.path.join(dp, f)
                rel = os.path.relpath(p, ROOT).replace('\\', '/')
                try: txt = open(p, encoding='utf-8', errors='replace').read()
                except OSError: continue
                txt = blank_code(txt)
                crate = None
                for pref, c in CRATE_OF_DIR:
                    if rel.startswith(pref): crate = c; break
                if crate is None and rel.startswith('pc/gameworld/'): crate = 'melee_runtime'
                for m in rx.finditer(txt):
                    pre = m.group('pre')
                    if re.search(r'\b(if|while|for|switch|return|else|sizeof)\b', pre) or '=' in pre: continue
                    name = m.group('name')
                    if name in ('if', 'while', 'for', 'switch', 'sizeof'): continue
                    defs[name].append((rel, crate, os.path.splitext(f)[0].lower(), bool(re.search(r'\bstatic\b', pre))))
    return defs


def canonical_index(defs):
    """name -> (crate, tu_stem, rel) for names with a unique canonical home."""
    out = {}
    for name, lst in defs.items():
        files = {x[0] for x in lst}
        statics = [x for x in lst if x[3]]
        if len(files) > 1 and statics: continue  # shared static name
        if len(files) > 1: continue
        rel, crate, stem, _ = lst[0]
        out[name] = (crate, stem, rel)
    return out


def rust_files():
    for crate in sorted(os.listdir(os.path.join(ROOT, 'rust'))):
        sd = os.path.join(ROOT, 'rust', crate, 'src')
        if not os.path.isdir(sd): continue
        for dp, dns, fs in os.walk(sd):
            rel_dir = os.path.relpath(dp, sd).replace('\\', '/')
            parts = rel_dir.split('/')
            if parts[0] in ('raw', 'c2r', 'tests'): dns[:] = []; continue
            for f in sorted(fs):
                if f.endswith('.rs') and f != 'gen.rs':
                    yield crate, os.path.join(dp, f), os.path.relpath(os.path.join(dp, f), ROOT).replace('\\', '/')


def load_allow():
    pats = []
    if os.path.exists(ALLOW):
        for ln in open(ALLOW, encoding='utf-8'):
            ln = ln.split('#')[0].strip()
            if not ln: continue
            # a line holds globs then (optionally) 2+ spaces and a reason
            globs = re.split(r'\s{2,}', ln)[0].split()
            pats += globs
    return pats


def expand_braces(p):
    m = re.search(r'\{([^{}]*)\}', p)
    if not m: return [p]
    out = []
    for alt in m.group(1).split(','):
        out += expand_braces(p[:m.start()] + alt + p[m.end():])
    return out


def allowed(rel, pats):
    for p in pats:
        for q in expand_braces(p):
            q2 = q.replace('**', '\0').replace('*', '[^/]*').replace('\0', '.*')
            if re.fullmatch(q2, rel): return True
    return False


def is_trivial(body):
    b = body.strip()
    if not b: return True
    if 'todo!(' in '\n'.join(b.split('\n')[:5]) or 'unimplemented!(' in '\n'.join(b.split('\n')[:5]): return True
    if re.search(r'match\s+unsafe\s*\{\s*[A-Z][A-Z0-9_]+\s*\}', b) or 'Some(f) => f(' in b: return True
    if 'ext_call(' in b and b.count(chr(10)) < 15: return True
    if 'HOOKS.read()' in b and b.count(';') <= 3 and 'if ' not in b: return True
    if re.fullmatch(r'\s*call\("\w+", &\[.*\]\);?\s*', b, re.S): return True
    flat = re.sub(r'\s+', ' ', b)
    flat = re.sub(r'\b(?:unsafe|return)\b|let _ =|\bas _(?![\w])|[{};]', ' ', flat)
    flat = re.sub(r'\s+', ' ', flat).strip()
    if re.fullmatch(r'(?:[\w]+::)*(?:tu|ftlib|lbtrigf|lbdvd)::[\w:]+\([^()]*(?:\([^()]*\)[^()]*)*\)', flat) and flat.count('(') <= 12: return True
    one = re.sub(r'\s+', ' ', b)
    one = re.sub(r'^unsafe \{ (.*) \}$', r'\1', one)
    if re.fullmatch(r'(?:crate::|super::)?(?:tu|melee_[a-z]+::tu)::[\w:]+\([^;]*\);?', one): return True
    if re.fullmatch(r'(?:[\w:]+::)?tu::[\w:]+\([^;]*\);?', one): return True
    return False


HOOK_RE = re.compile(r'^[ 	]*(?:pub[ 	]+)?(?:unsafe[ 	]+)?fn[ 	]+(\w+)[ 	]*\(', re.M)


def hook_defaults(src, blank):
    """ext_fns! entries `fn name(args) -> R => body;` (and `= |..| body`): yield (name, line, body)."""
    for m in HOOK_RE.finditer(blank):
        i, d, n = m.end(), 1, len(blank)
        while i < n and d:
            d += (blank[i] == '(') - (blank[i] == ')'); i += 1
        j = i
        while j < n and blank[j] not in '{;=': j += 1
        if j >= n or blank[j] != '=': continue
        k, d = j, 0
        while k < n:
            ch = blank[k]
            if ch in '{(': d += 1
            elif ch in '})': d -= 1
            elif ch == ';' and d <= 0: break
            if d < 0: break
            k += 1
        yield m.group(1), src.count(chr(10), 0, m.start(1)) + 1, src[j + 1:k]


def analyze():
    defs = c_index()
    canon = canonical_index(defs)
    cname = {n.lower(): n for n in canon}
    pats = load_allow()
    bodies = collections.defaultdict(list)  # cname -> [(crate, rel, line, kind, trivial, istu, start, end)]
    r5 = []
    r4 = []
    glue_cache = []
    for crate, path, rel in rust_files():
        src = open(path, encoding='utf-8', errors='replace', newline='').read()
        blank = blank_code(src)
        istu = '/src/tu/' in rel
        for name, line, s, p, bs, be, dep in find_bodies(src, blank):
            key = cname.get(name.lower())
            if key is None: continue
            body = blank[bs:be]
            triv = is_trivial(src[bs:be])
            if triv: continue
            bodies[key].append(dict(crate=crate, rel=rel, line=line, istu=istu, allowed=allowed(rel, pats),
                                    depth=dep, start=s, bstart=bs, bend=be, name=name, sig=re.sub(r'\s+', ' ', blank[s:p]).strip(), nlines=src.count('\n', bs, be)))
        if not allowed(rel, pats):
            for name, line, body in hook_defaults(src, blank):
                key = cname.get(name.lower())
                if key is not None and not is_trivial(body) and key not in SEAMS:
                    r4.append((rel, line, key))
        if allowed(rel, pats) and not istu:
            for m in re.finditer(r'\b(?:fp|ip|gp)\.[a-z_0-9]+\s*=[^=]|\brand\.', blank):
                r5.append((rel, src.count('\n', 0, m.start()) + 1, 'glue_write'))
    return defs, canon, bodies, r5 + [(a, b, 'hook:' + c) for a, b, c in r4]


_TU = {}


def tu_file(canon_entry):
    crate, stem, rel = canon_entry
    if crate is None: return None
    if crate not in _TU:
        d = os.path.join(ROOT, 'rust', crate, 'src', 'tu')
        _TU[crate] = {f[:-3].lower(): f for f in os.listdir(d) if f.endswith('.rs')} if os.path.isdir(d) else {}
    f = _TU[crate].get(stem, stem + '.rs')
    return 'rust/%s/src/tu/%s' % (crate, f)


SEAMS = set()


def violations(canon, bodies, r5):
    v = []  # (rule, path, name)
    for name, lst in bodies.items():
        home = tu_file(canon[name])
        tu_bodies = [b for b in lst if b['istu']]
        at_home = [b for b in lst if b['rel'] == home]
        for b in lst:
            if b['allowed']: continue
            if not b['istu']:
                v.append(('R1', b['rel'], name))
        if len(lst) > 1 and not (len(lst) == len([b for b in lst if b['allowed']]) + 1 and at_home):
            for b in lst:
                if b['rel'] == home and len(at_home) == 1: continue
                if b['allowed']: continue
                v.append(('R2', b['rel'], name))
        elif len(lst) > 1 and False:
            pass
        if not at_home and tu_bodies and len(lst) == len(tu_bodies):
            for b in tu_bodies:
                v.append(('R3', b['rel'], name))
    for rel, line, tag in r5:
        if tag.startswith('hook:'): v.append(('R4', rel, tag[5:]))
        else: v.append(('R5', rel, 'line%d' % line))
    return sorted(set(v))


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--report', action='store_true')
    ap.add_argument('--write-baseline', action='store_true')
    ap.add_argument('--allow-growth', action='store_true')
    ap.add_argument('--dedupe-table', action='store_true')
    ap.add_argument('--json')
    a = ap.parse_args()
    defs, canon, bodies, r5 = analyze()
    v = violations(canon, bodies, r5)
    dups = {n: l for n, l in bodies.items() if len(l) > 1}
    if a.json:
        json.dump(dict(canon={n: list(c) for n, c in canon.items()}, bodies=bodies, violations=v), open(a.json, 'w'))
    if a.dedupe_table:
        byhome = collections.defaultdict(list)
        for n, l in sorted(dups.items()):
            home = tu_file(canon[n])
            extra = [b['rel'] for b in l if b['rel'] != home]
            byhome[home].append((n, extra))
        for h in sorted(byhome, key=str):
            print('- `%s` (%d): %s' % (h, len(byhome[h]), '; '.join('%s [%s]' % (n, ', '.join(e)) for n, e in byhome[h])))
        return 0
    cnt = collections.Counter(x[0] for x in v)
    if a.report or not (a.write_baseline):
        print('functions with bodies (non-trivial) matched to C: %d' % len(bodies))
        print('R2 names with >1 body: %d (extra bodies: %d)' % (len(dups), sum(len(l) - 1 for l in dups.values())))
        for r in ('R1', 'R2', 'R3', 'R4', 'R5'): print('%s %d' % (r, cnt.get(r, 0)))
        percrate = collections.Counter((x[0], x[1].split('/')[1]) for x in v)
        for (r, c), k in sorted(percrate.items()): print('  %s %s %d' % (r, c, k))
    base = set()
    if os.path.exists(BASELINE):
        for ln in open(BASELINE, encoding='utf-8'):
            ln = ln.rstrip('\n')
            if ln: base.add(tuple(ln.split('\t')))
    if a.write_baseline:
        if len(v) > len(base) and base and not a.allow_growth:
            print('refusing to grow baseline (%d -> %d) without --allow-growth' % (len(base), len(v))); return 1
        open(BASELINE, 'w', encoding='utf-8', newline='\n').write(''.join('\t'.join(x) + '\n' for x in v))
        print('baseline written: %d lines' % len(v)); return 0
    if a.report: return 0
    new = [x for x in v if x not in base]
    for r, p, n in new: print('%s %s %s' % (p, r, n))
    print('new violations: %d (baseline %d)' % (len(new), len(base)))
    return 1 if new else 0


if __name__ == '__main__':
    sys.exit(main())
