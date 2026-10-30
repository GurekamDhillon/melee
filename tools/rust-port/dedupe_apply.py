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
ALLOWED_ATTRS = re.compile(r'^\s*(///|//!|#\[(inline|allow|must_use|doc|track_caller|deprecated)\b)')


def reach(ec, hc):
    if ec == hc: return True
    if ec in DEPS: return hc in DEPS[ec]
    return True  # melee bin, script, etc. depend on everything we care about (checked by compiler)


def plan():
    defs, canon, bodies, r5 = L.analyze()
    cands = []  # dicts
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
            cands.append(dict(name=name, ename=b['name'], rel=b['rel'], crate=b['crate'], start=st, end=end, use=use,
                              home=home, hstart=h['start'], hsig=h['sig'], key='%s|%s' % (b['rel'], b['name'])))
    return cands, skipped, srcs


def apply(cands, active, srcs):
    byfile = collections.defaultdict(list)
    homepub = {}
    for c in cands:
        if c['key'] not in active: continue
        byfile[c['rel']].append((c['start'], c['end'], c['use']))
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
    ap.add_argument('--limit', type=int, default=0)
    ap.add_argument('--crates')
    a = ap.parse_args()
    cands, skipped, srcs = plan()
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
