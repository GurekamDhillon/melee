#!/usr/bin/env python3
"""Fold re-exported bodies into the tu file that re-exports them (consolidation step 20/21).

For every `pub use crate::<PFX>::<MODPATH>::{a, b};` (or `use`) in a tu file, where MODPATH resolves to
a `pub mod X { ... }` block in a source file, the named items (and the private items they use)
are moved out of the source block into the tu file; the source block gets a
`pub use crate::tu::<file>::{names};` back-reference so existing paths (tests, other crates) keep resolving.

  fold_reexports.py --crate rust/melee_chara --prefix chars [--dry] [--only FILE.rs]
"""
import re, sys, os, argparse


def skip_str(s, i):
    n = len(s)
    j = i + 1
    while j < n:
        if s[j] == '\\':
            j += 2
            continue
        if s[j] == '"':
            return j + 1
        j += 1
    return n


def scan_close(s, i):
    """s[i] == '{' ; return index of matching '}'"""
    depth = 0
    n = len(s)
    while i < n:
        c = s[i]
        if c == '/' and s.startswith('//', i):
            j = s.find('\n', i)
            i = n if j < 0 else j
            continue
        if c == '/' and s.startswith('/*', i):
            j = s.find('*/', i)
            i = n if j < 0 else j + 2
            continue
        if c == 'r' and re.match(r'r#*"', s[i:i + 8]) and (i == 0 or not (s[i - 1].isalnum() or s[i - 1] == '_')):
            m = re.match(r'r(#*)"', s[i:])
            h = m.group(1)
            end = s.find('"' + h, i + len(m.group(0)))
            i = n if end < 0 else end + 1 + len(h)
            continue
        if c == '"':
            i = skip_str(s, i)
            continue
        if c == "'":
            m = re.match(r"'(\\.[^']*|[^\\'])'", s[i:i + 12])
            if m:
                i += len(m.group(0))
                continue
            i += 1
            continue
        if c == '{':
            depth += 1
        elif c == '}':
            depth -= 1
            if depth == 0:
                return i
        i += 1
    return n


HEAD = re.compile(r'^\s*(?:pub(?:\([a-z]+\))?\s+)?(?:unsafe\s+)?(?:extern\s+"C"\s+)?(fn|const|static\s+mut|static|struct|enum|type|mod|use|impl|trait|union)\b\s*([A-Za-z_][A-Za-z0-9_]*)?')
BRACE_KINDS = {'fn', 'struct', 'enum', 'mod', 'impl', 'trait', 'union'}


def split_items(body):
    """top-level items of a module's inner text: (start, end, kind, name, text)"""
    items = []
    i = 0
    n = len(body)
    while i < n:
        while i < n and body[i] in ' \t\r\n':
            i += 1
        if i >= n:
            break
        start = i
        inner_attr = False
        while True:
            if body.startswith('#![', i):
                inner_attr = True
                break
            if body.startswith('//', i):
                j = body.find('\n', i)
                i = n if j < 0 else j + 1
            elif body.startswith('#[', i):
                k = i + 1
                d = 0
                while k < n:
                    if body[k] == '[':
                        d += 1
                    elif body[k] == ']':
                        d -= 1
                        if d == 0:
                            break
                    elif body[k] == '"':
                        k = skip_str(body, k)
                        continue
                    k += 1
                i = k + 1
            else:
                break
            while i < n and body[i] in ' \t\r\n':
                i += 1
        if inner_attr:
            k = i + 2
            d = 0
            while k < n:
                if body[k] == '[':
                    d += 1
                elif body[k] == ']':
                    d -= 1
                    if d == 0:
                        break
                elif body[k] == '"':
                    k = skip_str(body, k)
                    continue
                k += 1
            items.append((start, k + 1, 'inner', None, body[start:k + 1]))
            i = k + 1
            continue
        if i >= n:
            items.append((start, n, 'trail', None, body[start:n]))
            break
        hs = i
        m = HEAD.match(body[hs:hs + 200])
        kind = m.group(1).split()[0] if m else 'other'
        name = m.group(2) if m else None
        j = i
        if kind in BRACE_KINDS:
            k = j
            pd = 0
            while k < n:
                c = body[k]
                if c == '"':
                    k = skip_str(body, k)
                    continue
                if c in '([':
                    pd += 1
                elif c in ')]':
                    pd -= 1
                elif c == '{' and pd == 0:
                    break
                elif c == ';' and pd == 0:
                    break
                k += 1
            if k < n and body[k] == '{':
                e = scan_close(body, k) + 1
            else:
                e = k + 1
        else:
            k = j
            d = 0
            while k < n:
                c = body[k]
                if c == '"':
                    k = skip_str(body, k)
                    continue
                if c in '({[':
                    d += 1
                elif c in ')}]':
                    d -= 1
                elif c == ';' and d == 0:
                    break
                k += 1
            e = k + 1
        items.append((start, e, kind, name, body[start:e]))
        i = e
    return items


def find_mod_block(src, modname):
    m = re.search(r'(?m)^(?:pub(?:\([a-z]+\))?\s+)?mod\s+' + re.escape(modname) + r'\s*\{', src)
    if not m:
        return None
    ob = src.index('{', m.start())
    cb = scan_close(src, ob)
    return m.start(), ob, cb


USE_RE = r'(?ms)^[ \t]*((?:pub(?:\([a-z]+\))?\s+)?use\s+crate::PFX::([A-Za-z0-9_:]+?)::(\{[^}]*\}|[A-Za-z_][A-Za-z0-9_]*)\s*;)'


def use_leaves(u):
    m = re.match(r'\s*(?:#\[[^\]]*\]\s*)*(?:pub(?:\([a-z]+\))?\s+)?use\s+(.*?);', u, re.S)
    p = m.group(1)
    out = []

    def rec(prefix, t):
        t = t.strip()
        if t.startswith('{') and t.endswith('}'):
            depth = 0
            cur = ''
            parts = []
            for ch in t[1:-1]:
                if ch == '{':
                    depth += 1
                if ch == '}':
                    depth -= 1
                if ch == ',' and depth == 0:
                    parts.append(cur)
                    cur = ''
                else:
                    cur += ch
            parts.append(cur)
            for q in parts:
                if q.strip():
                    rec(prefix, q)
            return
        if '{' in t:
            a, b = t.split('{', 1)
            rec(prefix + a, '{' + b)
            return
        full = prefix + t
        if full.endswith('*'):
            out.append(full)
        elif ' as ' in full:
            out.append(full.split(' as ')[1].strip())
        else:
            out.append(full.split('::')[-1].strip())

    rec('', p)
    return out


IDENT = re.compile(r'[A-Za-z_][A-Za-z0-9_]*')


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--crate', required=True)
    ap.add_argument('--prefix', default='chars')
    ap.add_argument('--dry', action='store_true')
    ap.add_argument('--only')
    a = ap.parse_args()
    src_root = os.path.join(a.crate, 'src')
    tu_dir = os.path.join(src_root, 'tu')
    use_re = re.compile(USE_RE.replace('PFX', a.prefix))
    cache = {}

    def load(p):
        if p not in cache:
            cache[p] = open(p, encoding='utf-8').read()
        return cache[p]

    moved_total = 0
    unresolved = []
    for fn in sorted(os.listdir(tu_dir)):
        if not fn.endswith('.rs'):
            continue
        if a.only and fn != a.only:
            continue
        tp = os.path.join(tu_dir, fn)
        tu = load(tp)
        tuname = fn[:-3]
        new_imports = []
        appended = []
        edits = []
        for m in list(use_re.finditer(tu)):
            stmt, path, names = m.group(1), m.group(2), m.group(3)
            parts = path.split('::')
            modname = parts[-1]
            srcparts = parts[:-1]
            cand = os.path.join(src_root, a.prefix, *srcparts) + '.rs'
            if not os.path.exists(cand):
                unresolved.append((fn, path, 'nofile'))
                continue
            sp = cand
            s = load(sp)
            blk = find_mod_block(s, modname)
            if not blk:
                unresolved.append((fn, path, 'nomod'))
                continue
            bs, ob, cb = blk
            inner = s[ob + 1:cb]
            items = split_items(inner)
            want = [x.strip() for x in names.strip('{}').split(',') if x.strip()]
            wantset = set(w.split(' as ')[0].strip() for w in want)
            by_name = {}
            for it in items:
                if it[3]:
                    by_name.setdefault(it[3], []).append(it)
            sel = set()
            work = [w for w in wantset if w in by_name]
            missing = [w for w in wantset if w not in by_name]
            if missing:
                unresolved.append((fn, path, 'missing:' + ','.join(missing)))
            while work:
                w = work.pop()
                if w in sel:
                    continue
                sel.add(w)
                for it in by_name[w]:
                    if it[2] == 'use':
                        continue
                    for idn in set(IDENT.findall(it[4])):
                        if idn in by_name and idn not in sel and idn != w:
                            work.append(idn)
            movable = [it for it in items if it[3] in sel and it[2] != 'use']
            if not movable:
                continue
            uses = [it for it in items if it[2] == 'use']
            for it in movable:
                appended.append(it[4].strip('\n'))
            for u in uses:
                new_imports.append(u[4].strip())
            keep = []
            for it in items:
                if it[3] in sel and it[2] != 'use':
                    continue
                keep.append(it[4])
            moved_names = [it[3] for it in movable]
            back = 'pub use crate::tu::%s::{%s};' % (tuname, ', '.join(sorted(set(moved_names))))
            newinner = '\n' + back + '\n' + ''.join('\n' + k.strip('\n') + '\n' for k in keep if k.strip()) + '\n'
            cache[sp] = s[:ob + 1] + newinner + s[cb:]
            rem = [w for w in want if w.split(' as ')[0].strip() not in sel]
            if rem:
                rep = re.sub(r'::(\{[^}]*\}|[A-Za-z_][A-Za-z0-9_]*)\s*;$', '', stmt)
                rep = rep + '::{' + ', '.join(rem) + '};'
            else:
                rep = ''
            edits.append((m.start(1), m.end(1), rep))
            moved_total += len(movable)
        if not edits:
            continue
        for st, en, rep in sorted(edits, reverse=True):
            tu = tu[:st] + rep + tu[en:]
        have = set()
        for um in re.finditer(r'(?ms)^(?:pub(?:\([a-z]+\))?\s+)?use\s+.*?;', tu):
            try:
                have.update(use_leaves(um.group(0)))
            except Exception:
                pass
        add = []
        seen_stmt = set()
        for u in new_imports:
            u = re.sub(r'^(#\[[^\]]*\]\s*)+', '', u)
            mu = re.search(r'(?m)^(?:pub(?:\([a-z]+\))?\s+)?use\s', u)
            u = u[mu.start():] if mu else u
            if u in seen_stmt:
                continue
            seen_stmt.add(u)
            try:
                lv = use_leaves(u)
            except Exception:
                continue
            fresh = [l for l in lv if l not in have]
            if not fresh:
                continue
            mb = re.match(r'((?:pub(?:\([a-z]+\))?\s+)?use\s+[A-Za-z0-9_:]+::)\{(.*)\};', u, re.S)
            if mb and len(fresh) != len(lv):
                u = mb.group(1) + '{' + ', '.join(fresh) + '};'
            have.update(fresh)
            add.append('#[allow(unused_imports)]\n' + re.sub(r'^(?:pub(?:\([a-z]+\))?\s+)?use', 'use', u))
        body = '\n\n// ---- folded from %s (consolidation step 20/21) ----\n' % a.prefix + '\n\n'.join(appended) + '\n'
        lines = tu.split('\n')
        pos = 0
        for idx, ln in enumerate(lines):
            if ln.startswith('//!') or ln.startswith('#![') or ln.strip() == '':
                pos = idx + 1
            else:
                break
        ins = ['#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]'] + add
        lines[pos:pos] = ins
        tu = '\n'.join(lines).rstrip('\n') + body
        cache[tp] = tu
    if not a.dry:
        for p, t in cache.items():
            with open(p, 'w', encoding='utf-8', newline='\n') as f:
                f.write(t)
    print('moved items:', moved_total)
    for u in unresolved:
        print('unresolved', *u)


main()
