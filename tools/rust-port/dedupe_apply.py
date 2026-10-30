#!/usr/bin/env python3
"""Compile-driven dedupe: replace non-canonical depth-0 fn bodies by re-exports of the canonical tu/ body.

Usage: dedupe_apply.py [--rounds N] [--crates a,b] [--dry]
Each round: apply all active replacements from the original (git HEAD) text, run cargo check, map errors back to
replacements, deactivate those, repeat until clean. State: dedupe_state.json (next to scratch dir given by --state).
"""
import argparse, collections, json, os, re, subprocess, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import lint_logic_location as L

ROOT = L.ROOT
DEPS = {
    'melee_core': [], 'melee_types': ['melee_core'], 'melee_runtime': ['melee_core', 'melee_types'],
    'melee_world': ['melee_core', 'melee_types', 'melee_runtime'],
    'melee_items': ['melee_core', 'melee_types', 'melee_runtime', 'melee_world'],
    'melee_fighter': ['melee_core', 'melee_types', 'melee_runtime', 'melee_items', 'melee_world'],
    'melee_chara': ['melee_core', 'melee_types', 'melee_fighter', 'melee_runtime', 'melee_world', 'melee_items'],
    'melee_modes': ['melee_core', 'melee_types', 'melee_runtime', 'melee_world', 'melee_fighter', 'melee_chara', 'melee_items'],
    'melee_audio': ['melee_types', 'melee_runtime'], 'melee_net': ['melee_types', 'melee_fighter', 'melee_runtime'],
}
HOOKISH = re.compile(r'EXT_HOOK|_HOOK|need[(]|HOOKS|ext_call|oracle|hook[(]|call[(]"')
GUARD_TESTS = False
# bodies that tests/oracles call directly as reference models (replacing them broke player_ref, plbonus*_ref, oracle_gobj)
BANNED = ['melee_fighter/src/player.rs', 'melee_fighter/src/plbonus.rs', 'melee_fighter/src/plbonus2.rs', 'melee_fighter/src/plbonus3.rs', 'melee_runtime/src/gobj.rs']
ALLOWED_ATTRS = re.compile(r'^\s*(///|//!|#\[(inline|allow|must_use|doc|track_caller|deprecated)\b)')


def reach(ec, hc):
    if ec == hc: return True
    if ec in DEPS: return hc in DEPS[ec]
    return True  # melee bin, script, etc. depend on everything we care about (checked by compiler)



def split_top(s):
    s = s.replace('->', '@ARROW@')
    out, cur, d = [], '', 0
    for ch in s:
        if ch in '([{<': d += 1
        elif ch in ')]}>': d -= 1
        if ch == ',' and d == 0: out.append(cur); cur = ''
        else: cur += ch
    if cur.strip(): out.append(cur)
    return [x.replace('@ARROW@', '->').strip() for x in out]


def parse_sig(sig):
    m = re.match(r'(?:pub(?:\([a-z: ]+\))?\s+)?(unsafe\s+)?(?:extern\s+"C"\s+)?fn\s+(\w+)\s*\(', sig)
    if not m: return None
    i = m.end(); d = 1; j = i
    while j < len(sig) and d:
        if sig[j] == '(': d += 1
        elif sig[j] == ')': d -= 1
        j += 1
    params = split_top(sig[i:j - 1])
    ret = sig[j:].strip()
    ret = ret[2:].strip() if ret.startswith('->') else ''
    ps = []
    for p in params:
        mm = re.match(r'(?:mut\s+)?(\w+)\s*:\s*(.*)$', p, re.S)
        if not mm or mm.group(1) == '_': return None
        ps.append((mm.group(1), mm.group(2).strip()))
    return dict(unsafe=bool(m.group(1)), name=m.group(2), params=ps, ret=ret.split(' where ')[0].strip(), ptext=sig[i:j - 1])


def tclass(t):
    t = re.sub(r'\s+', '', t)
    if t.startswith('*mut') or t.startswith('*const'): return 'ptr'
    if re.fullmatch(r'[iu](8|16|32|64|size)', t): return 'int'
    if t in ('f32', 'f64'): return 'float'
    return None


def fwd_text(c, es, hs):
    e, h = parse_sig(es), parse_sig(hs)
    if not e or not h or len(e['params']) != len(h['params']): return None
    args = []
    for (en, et), (hn, ht) in zip(e['params'], h['params']):
        if re.sub(r'\s+', '', et) == re.sub(r'\s+', '', ht): args.append(en)
        elif tclass(et) and tclass(et) == tclass(ht): args.append(en + ' as _')
        else: return None
    rr = re.sub(r'\s+', '', e['ret']); hr = re.sub(r'\s+', '', h['ret'])
    if bool(rr) != bool(hr): return None
    cast = ''
    if rr and rr != hr:
        if tclass(rr) and tclass(rr) == tclass(hr): cast = ' as _'
        else: return None
    path = re.sub(r'^(pub(\([a-z: ]+\))?\s+)?use\s+', '', c['use']).split(' as ')[0].rstrip(';').strip()
    kw = (c['vis'] + ' ' if c['vis'] else '') + ('unsafe ' if e['unsafe'] else '')
    return '%sfn %s(%s)%s {\n    unsafe { %s(%s)%s }\n}' % (kw, e['name'], e['ptext'], (' -> ' + e['ret']) if e['ret'] else '', path, ', '.join(args), cast)


def expand_use(tree, prefix=''):
    """Expand a use tree into [(visible, full_path, is_glob)]."""
    tree = tree.strip()
    out = []
    if '{' in tree:
        i = tree.index('{')
        pre = tree[:i].strip()
        inner = tree[i + 1:tree.rindex('}')]
        for part in split_top(inner):
            if part.strip() == 'self': out.append((pre.rstrip(':').split('::')[-1], (prefix + pre).rstrip(':'), False))
            else: out += expand_use(part, prefix + pre)
        return out
    if tree.endswith('*'): return [('*', prefix + tree, True)]
    if ' as ' in tree:
        a, b = [x.strip() for x in tree.split(' as ')]
        return [(b, prefix + a, False)]
    return [(tree.split('::')[-1], prefix + tree, False)]


def module_path(rel):
    m = re.match(r'rust/([^/]+)/src/(.*)\.rs$', rel)
    parts = m.group(2).split('/')
    if parts[-1] in ('mod', 'lib'): parts = parts[:-1]
    return parts


def abs_use(path, rel):
    mp = module_path(rel)
    if path.startswith('self::'): return 'crate::' + '::'.join(mp + [path[6:]]) if mp else 'crate::' + path[6:]
    if path.startswith('super::'):
        p = path; base = mp[:]
        while p.startswith('super::'): p = p[7:]; base = base[:-1]
        return 'crate::' + '::'.join(base + [p]) if base else 'crate::' + p
    return path


def top_uses(text, rel):
    blank = L.blank_code(text)
    res = []
    for m in re.finditer(r'(?m)^(?:pub(?:\([^)]*\))?[ \t]+)?use[ \t]+([^;]+);', blank):
        for vis, path, glob in expand_use(re.sub(r'\s+', ' ', m.group(1))):
            res.append((vis, abs_use(path, rel), glob))
    return res


def defined_names(text):
    blank = L.blank_code(text)
    return set(re.findall(r'(?m)^(?:pub(?:\([^)]*\))?[ \t]+)?(?:unsafe[ \t]+)?(?:extern[ \t]+"C"[ \t]+)?(?:const[ \t]+|static[ \t]+(?:mut[ \t]+)?|fn[ \t]+|struct[ \t]+|enum[ \t]+|type[ \t]+|union[ \t]+|mod[ \t]+|trait[ \t]+)(\w+)', blank))


def plan_move(srcs):
    defs, canon, bodies, r5 = L.analyze()
    def text(rel):
        if rel not in srcs: srcs[rel] = open(os.path.join(ROOT, rel), encoding='utf-8', newline='').read()
        return srcs[rel]
    cands, skipped = [], collections.Counter()
    for name, lst in bodies.items():
        if len(lst) < 2: continue
        cc = canon[name][0]
        if cc is None: continue
        home = L.tu_file(canon[name])
        if any(b['rel'] == home and b['depth'] == 0 for b in lst): continue
        if len([b for b in lst if b['istu'] and b['depth'] == 0 and b['crate'] == cc]) >= 1: continue
        if not os.path.exists(os.path.join(ROOT, home)): skipped['nohome'] += 1; continue
        srcb = [b for b in lst if b['depth'] == 0 and not b['istu'] and not b['allowed'] and b['crate'] == cc]
        if not srcb: skipped['nosrc'] += 1; continue
        b = max(srcb, key=lambda x: x['nlines'])
        if len([x for x in lst if x['rel'] == b['rel']]) > 1: skipped['multi_in_file'] += 1; continue
        s = text(b['rel'])
        st = s.rfind('\n', 0, b['start']) + 1
        ok = True
        while st > 0:
            pe = st - 1; ps = s.rfind('\n', 0, pe) + 1
            line = s[ps:pe].rstrip('\r')
            if line.strip() == '': break
            if ALLOWED_ATTRS.match(line) or line.strip().startswith('//'): st = ps; continue
            if line.strip().startswith('#['): ok = False
            break
        if not ok: skipped['attr'] += 1; continue
        end = b['bend'] + 1
        body = s[st:end]
        bl = L.blank_code(body)
        words = set(re.findall(r'[A-Za-z_]\w*', bl))
        srcdef = defined_names(s)
        # private helpers defined in source and used by the body (other than the fn itself) -> unsupported
        priv = [w for w in words if w in srcdef and w != b['name']]
        if priv: skipped['helpers'] += 1; continue
        hs = text(home)
        hdef = defined_names(hs)
        hvis = {v for v, _, g in top_uses(hs, home) if not g}
        uses = []
        for vis, path, glob in top_uses(s, b['rel']):
            if glob:
                uses.append('use %s;' % path)
            elif vis in words and vis not in hvis and vis not in hdef:
                uses.append('use %s%s;' % (path, '' if path.split('::')[-1] == vis else ' as ' + vis))
        hmod = os.path.splitext(os.path.basename(home))[0]
        path = ('crate::tu::%s::%s' % (hmod, name)) if cc == b['crate'] else '%s::tu::%s::%s' % (cc, hmod, name)
        vis = re.match(r'\s*(pub(?:\([a-z: ]+\))?\s+)?', s[b['start']:]).group(1) or ''
        vis = vis.strip()
        if not vis.startswith('pub'): body2 = re.sub(r'^(\s*)', r'\1pub ', body[b['start'] - st:], count=1); body = body[:b['start'] - st] + body2
        alias = '' if b['name'] == name else ' as ' + b['name']
        use = '%s%suse %s%s;' % (vis, ' ' if vis else '', path, alias)
        hglobs = {p for v, p, g in top_uses(hs, home) if g}
        uses = [u for u in dict.fromkeys(uses) if not (u.startswith('use ') and u[4:-1] in hglobs)]
        cands.append(dict(kind='move', name=name, ename=b['name'], rel=b['rel'], crate=b['crate'], start=st, end=end, use=use,
                          home=home, hstart=0, hsig='pub fn', moved='\n' + '\n'.join(uses) + '\n' + body.rstrip() + '\n',
                          idents=sorted(words), key='%s|%s' % (b['rel'], b['name'])))
    return cands, skipped


def test_referenced_modules():
    """Modules (crate, path) named by integration tests: they test those bodies directly, so leave them."""
    refs = set()
    for crate in os.listdir(os.path.join(ROOT, 'rust')):
        td = os.path.join(ROOT, 'rust', crate, 'tests')
        if not os.path.isdir(td): continue
        for dp, _, fs in os.walk(td):
            for f in fs:
                if not f.endswith('.rs'): continue
                t = open(os.path.join(dp, f), encoding='utf-8', errors='replace').read()
                for m in re.finditer(r'(melee_\w+)::((?:\w+::)*\w+)', t):
                    refs.add((m.group(1), tuple(m.group(2).split('::'))))
    return refs


def referenced(rel, refs):
    m = re.match(r'rust/([^/]+)/src/', rel)
    mp = tuple(module_path(rel))
    for crate, path in refs:
        if crate == m.group(1) and (path[:len(mp)] == mp):
            return True
    return False


def plan(mode='use'):
    defs, canon, bodies, r5 = L.analyze()
    cands = []  # dicts
    trefs = test_referenced_modules()
    skipped = collections.Counter()
    srcs = {}
    def text(rel):
        if rel not in srcs: srcs[rel] = open(os.path.join(ROOT, rel), encoding='utf-8', newline='').read()
        return srcs[rel]
    for name, lst in bodies.items():
        if len(lst) < 2: continue
        home = L.tu_file(canon[name])
        hb = [b for b in lst if b['rel'] == home]
        if not hb:
            hb = [b for b in lst if b['istu'] and b['depth'] == 0 and b['crate'] == canon[name][0]]
            if len(hb) == 1: home = hb[0]['rel']
        if len(hb) != 1 or hb[0]['depth'] != 0:
            skipped['no_home_body'] += len(lst) - len(hb); continue
        h = hb[0]
        hmod = os.path.splitext(os.path.basename(home))[0]
        hcrate = h['crate']
        for b in lst:
            if b is h: continue
            if b['depth'] != 0: skipped['nested'] += 1; continue
            if not reach(b['crate'], hcrate): skipped['upward'] += 1; continue
            if b['rel'].endswith('/mod.rs') and False: continue
            if HOOKISH.search(L.blank_code(text(b['rel'])[b['bstart']:b['bend']])): skipped['hook_dispatch'] += 1; continue
            if referenced(b['rel'], trefs) and GUARD_TESTS: skipped['test_referenced'] += 1; continue
            if any(b['rel'].endswith(x) for x in BANNED): skipped['banned'] += 1; continue
            hbody = L.blank_code(text(h['rel'])[h['bstart']:h['bend']])
            if re.search(r'(?<![A-Za-z0-9_])(%s|%s)(?![A-Za-z0-9_])' % (re.escape(b['name']), re.escape(name)), hbody): skipped['home_delegates'] += 1; continue
            same = [x for x in lst if x['rel'] == b['rel']]
            if len(same) > 1: skipped['multi_in_file'] += 1; continue
            s = text(b['rel'])
            # include attached attrs/docs
            ls = s.rfind('\n', 0, b['start']) + 1
            st = ls
            ok = True
            while st > 0:
                pe = st - 1
                ps = s.rfind('\n', 0, pe) + 1
                line = s[ps:pe].rstrip('\r')
                if line.strip() == '': break
                if ALLOWED_ATTRS.match(line) or line.strip().startswith('//'):
                    st = ps; continue
                if line.strip().startswith('#['):
                    ok = False
                break
            if not ok: skipped['attr'] += 1; continue
            end = b['bend'] + 1
            vis = re.match(r'\s*(pub(?:\([a-z: ]+\))?\s+)?', s[b['start']:]).group(1) or ''
            vis = vis.strip()
            tail = ''
            alias = '' if b['name'] == name else ' as ' + b['name']
            if b['crate'] == hcrate: path = 'crate::tu::%s::%s' % (hmod, name)
            else: path = '%s::tu::%s::%s' % (hcrate, hmod, name)
            use = '%s%suse %s%s;' % (vis, ' ' if vis else '', path, alias)
            if mode == 'fwd':
                use = fwd_text(dict(use=use, vis=vis), b['sig'], h['sig'])
                if use is None: skipped['fwd_sig'] += 1; continue
            cands.append(dict(name=name, ename=b['name'], rel=b['rel'], crate=b['crate'], start=st, end=end, use=use,
                              home=home, hstart=h['start'], hsig=h['sig'], key='%s|%s' % (b['rel'], b['name'])))
    return cands, skipped, srcs


def apply(cands, active, srcs):
    byfile = collections.defaultdict(list)
    homepub = {}
    for c in cands:
        if c['key'] not in active: continue
        byfile[c['rel']].append((c['start'], c['end'], c['use']))
        if c.get('kind') == 'move':
            hs0 = srcs[c['home']]; byfile[c['home']].append((len(hs0), len(hs0), c['moved'])); continue
        if not re.match(r'\s*pub\b', c['hsig']):
            homepub[(c['home'], c['hstart'])] = 1
    for (home, hs) in homepub:
        s = srcs[home]
        m = re.compile(r'[ \t]*').match(s, hs)
        byfile[home].append((m.end(), m.end(), 'pub '))
    for rel, edits in byfile.items():
        s = srcs[rel]
        out = []
        pos = 0
        for a, b, t in sorted(set(edits)):
            if a < pos: continue
            out.append(s[pos:a]); out.append(t); pos = b
        out.append(s[pos:])
        open(os.path.join(ROOT, rel), 'w', encoding='utf-8', newline='').write(''.join(out))
    return list(byfile)


def restore(rels, srcs):
    for rel in rels:
        open(os.path.join(ROOT, rel), 'w', encoding='utf-8', newline='').write(srcs[rel])


def cargo_check(target_dir):
    env = dict(os.environ, CARGO_PROFILE_DEV_DEBUG='0', CARGO_PROFILE_TEST_DEBUG='0', CARGO_INCREMENTAL='0', CARGO_TARGET_DIR=target_dir)
    p = subprocess.run(['cargo', 'check', '--workspace', '--tests', '--keep-going', '--message-format=json'], cwd=os.path.join(ROOT, 'rust'),
                       env=env, capture_output=True, text=True, encoding='utf-8', errors='replace')
    errs = []
    for ln in p.stdout.splitlines():
        if not ln.startswith('{'): continue
        try: j = json.loads(ln)
        except Exception: continue
        if j.get('reason') != 'compiler-message': continue
        m = j['message']
        if m.get('level') != 'error': continue
        errs.append(m)
    return errs, p.returncode


def culprits(errs, cands, active, edited_text):
    """Map compiler errors to candidate keys to deactivate."""
    bykey_name = collections.defaultdict(list)
    for c in cands:
        if c['key'] in active: bykey_name[c['ename']].append(c)
    out = set()
    unmapped = []
    for m in errs:
        spans = []
        for s0 in m.get('spans', []):
            if not s0.get('is_primary'): continue
            cur = s0
            spans.append(cur)
            while cur.get('expansion'):
                cur = cur['expansion']['span']
                spans.append(cur)
        txt = m['message'] + ' ' + ' '.join(ch.get('message', '') for ch in m.get('children', []))
        found = set()
        words = set(re.findall(r'\w+', txt))
        for s in spans:
            f = s['file_name'].replace('\\', '/')
            line = ''
            fp = os.path.join(ROOT, 'rust', f) if not os.path.isabs(f) else f
            try:
                key = 'rust/' + f
                txt0 = edited_text[key] if key in edited_text else open(fp, encoding='utf-8', errors='replace').read()
                ls = txt0.split('\n')
                line = ' '.join(ls[s['line_start'] - 1:s['line_end']])
            except OSError: pass
            lw = set(re.findall(r'\w+', line)) | words
            for w in lw:
                for c in bykey_name.get(w, []):
                    found.add(c['key'])
            for c in cands:
                if c['key'] in active and c.get('kind') == 'move' and c['home'] == key and (lw & set(c['idents'])):
                    found.add(c['key'])
            # edits in the same file whose use line is the span line
            for c in cands:
                if c['key'] in active and ('rust/' + f).replace('rust/rust/', 'rust/') == c['rel'] and c['use'].split(' as ')[0].split('::')[-1].rstrip(';') in line:
                    found.add(c['key'])
        # prefer candidates in the same file as the error
        if found:
            files = {s['file_name'].replace('\\', '/') for s in spans}
            same = {k for k in found if any(k.split('|')[0].endswith(f) or f.endswith(k.split('|')[0][5:]) for f in files)}
            out |= (same or found)
        elif any(c['key'] in active and c['rel'] == ('rust/' + sp['file_name'].replace(chr(92), '/')) for sp in spans for c in cands):
            for sp in spans:
                for c in cands:
                    if c['key'] in active and c['rel'] == ('rust/' + sp['file_name'].replace(chr(92), '/')): out.add(c['key'])
            unmapped.append('(file-level) ' + m['message'])
        else:
            unmapped.append(m['rendered'][:200] if m.get('rendered') else m['message'])
    return out, unmapped


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--rounds', type=int, default=30)
    ap.add_argument('--state', required=True)
    ap.add_argument('--target', required=True)
    ap.add_argument('--dry', action='store_true')
    ap.add_argument('--mode', default='use')
    ap.add_argument('--limit', type=int, default=0)
    ap.add_argument('--crates')
    ap.add_argument('--ban-file')
    ap.add_argument('--guard-tests', action='store_true')
    a = ap.parse_args()
    global GUARD_TESTS, BANNED
    GUARD_TESTS = a.guard_tests
    if a.ban_file and os.path.exists(a.ban_file): BANNED += [l.strip() for l in open(a.ban_file) if l.strip()]
    if a.mode == 'move':
        srcs = {}; cands, skipped = plan_move(srcs)
    else: cands, skipped, srcs = plan(a.mode)
    if a.crates: cands = [c for c in cands if c['crate'] in a.crates.split(',')]
    if a.limit: cands = cands[:a.limit]
    print('candidates', len(cands), 'skipped', dict(skipped), flush=True)
    if a.dry: return 0
    active = {c['key'] for c in cands}
    if os.path.exists(a.state):
        st = json.load(open(a.state)); active &= set(st['active'])
    # ensure srcs for all touched files incl. homes
    for c in cands:
        for rel in (c['rel'], c['home']):
            if rel not in srcs: srcs[rel] = open(os.path.join(ROOT, rel), encoding='utf-8', newline='').read()
    clean = False
    for r in range(a.rounds):
        touched = apply(cands, active, srcs)
        errs, rc = cargo_check(a.target)
        edited = {t: open(os.path.join(ROOT, t), encoding='utf-8', newline='').read() for t in touched}
        restore(touched, srcs)
        print('round', r, 'active', len(active), 'errors', len(errs), flush=True)
        if not errs and rc == 0: clean = True; break
        bad, unm = culprits(errs, cands, active, edited)
        for u in unm[:5]: print('  unmapped:', u.replace('\n', ' | ')[:300], flush=True)
        if not bad:
            print('no culprit mapping; stopping WITHOUT applying', flush=True); return 2
        active -= bad
        json.dump(dict(active=sorted(active), dropped=len(cands) - len(active)), open(a.state, 'w'))
        print('  deactivated', len(bad), flush=True)
    json.dump(dict(active=sorted(active), dropped=len(cands) - len(active)), open(a.state, 'w'))
    if not clean:
        print('NOT CLEAN; nothing applied'); return 3
    apply(cands, active, srcs)
    print('final active', len(active), 'of', len(cands))
    return 0


if __name__ == '__main__':
    sys.exit(main())
