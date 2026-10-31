#!/usr/bin/env python3
"""Generate rust/melee_link/src/gen_table.rs: install() points every `ext_fns!` hook at the canonical TU function.

Consolidation plan section 1.2 / step 4.  Usage:
  gen_link.py            regenerate gen_table.rs and unresolved.txt
  gen_link.py --report   print the counts only
Inputs : rust/<crate>/src/tu/**/*.rs (never src/raw), tools/rust-port/link/link_skip.txt (entries that do not link).
Outputs: rust/melee_link/src/gen_table.rs, rust/melee_link/unresolved.txt

A hook entry NAME is linked when exactly one `tu/` file (other than the hook's own file) defines
`#[no_mangle] pub [unsafe] extern "C" fn NAME` with the same arity and the same return-ness, or when
several do and the C-function -> C-file map (docs/rust-port/trace/*.csv) picks one by file stem.
The generated adapter converts each argument and the result with `cast` (a size-checked transmute), so
`&mut T` / `*mut T`, `Option<fn>` / `Option<extern fn>` and i32/u32 pairs all connect.
Hooks that are not `pub mod hook` statics (the `$hook` form in ft_0899 and the EXT_HOOK form in melee_modes)
are listed in unresolved.txt as kind `other`.
"""
import glob, os, re, sys, collections

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), '..', '..', '..'))
RUST = os.path.join(ROOT, 'rust')
LEAVES = ['melee_runtime', 'melee_world', 'melee_fighter', 'melee_chara', 'melee_items', 'melee_modes']


def blank(s):
    """Replace comments, strings and char literals by spaces (keeps offsets and newlines)."""
    out = list(s); i = 0; n = len(s)
    while i < n:
        c = s[i]
        if c == '/' and s[i:i+2] == '//':
            j = s.find('\n', i); j = n if j < 0 else j
            for k in range(i, j): out[k] = ' '
            i = j
        elif c == '/' and s[i:i+2] == '/*':
            depth = 1; j = i + 2
            while j < n and depth:
                if s[j:j+2] == '/*': depth += 1; j += 2
                elif s[j:j+2] == '*/': depth -= 1; j += 2
                else: j += 1
            for k in range(i, j):
                if s[k] != '\n': out[k] = ' '
            i = j
        elif c == 'r' and re.match(r'r#*"', s[i:i+4]) and (i == 0 or not (s[i-1].isalnum() or s[i-1] == '_')):
            m = re.match(r'r(#*)"', s[i:]); h = m.group(1); end = '"' + h
            j = s.find(end, i + len(m.group(0)));  j = n if j < 0 else j + len(end)
            for k in range(i, j):
                if s[k] != '\n': out[k] = ' '
            i = j
        elif c == '"':
            j = i + 1
            while j < n and s[j] != '"':
                j += 2 if s[j] == '\\' else 1
            j += 1
            for k in range(i, min(j, n)):
                if s[k] != '\n': out[k] = ' '
            i = j
        elif c == "'":
            m = re.match(r"'(\\.[^']*|[^\\'])'", s[i:i+12])
            if m:
                for k in range(i, i + len(m.group(0))): out[k] = ' '
                i += len(m.group(0))
            else:
                i += 1
        else:
            i += 1
    return ''.join(out)


def match_close(t, i, o='{', c='}'):
    depth = 0
    for j in range(i, len(t)):
        if t[j] == o: depth += 1
        elif t[j] == c:
            depth -= 1
            if depth == 0: return j
    return -1


def split_top(t, sep):
    parts = []; depth = 0; cur = 0
    for j, ch in enumerate(t):
        if ch in '([{': depth += 1
        elif ch in ')]}': depth -= 1
        elif ch == '<':
            depth += 0
        elif ch == sep and depth == 0:
            parts.append(t[cur:j]); cur = j + 1
    parts.append(t[cur:])
    return parts


def count_params(plist):
    """Top-level comma count honouring <> as well as ()[]{}."""
    s = plist.strip()
    if not s: return 0
    depth = 0; n = 1; prev = ''
    for ch in s:
        if ch in '([{<': depth += 1
        elif ch in ')]}': depth -= 1
        elif ch == '>' and prev != '-': depth -= 1
        elif ch == ',' and depth == 0: n += 1
        prev = ch
    if s.rstrip().endswith(','): n -= 1
    return n


def modpath_of(f):
    rel = os.path.relpath(f, RUST).replace('\\', '/')
    crate, _, rest = rel.partition('/src/')
    rest = rest[:-3]
    parts = rest.split('/')
    if parts[-1] == 'mod': parts = parts[:-1]
    return crate, '::'.join(parts)


def tu_files():
    for crate in LEAVES:
        for f in sorted(glob.glob(os.path.join(RUST, crate, 'src', '**', '*.rs'), recursive=True)):
            rel = os.path.relpath(f, os.path.join(RUST, crate, 'src')).replace('\\', '/')
            if rel.startswith('raw/'): continue
            yield crate, f, rel


DEF_RE = re.compile(r'(?m)^[ 	]*pub\s+(?:unsafe\s+)?(?:extern\s+)?fn\s+(\w+)\s*\(')  # blank() erases the "C"


MOD_RE = re.compile(r'(?m)^[ 	]*(pub(?:\([a-z]+\))?\s+)?mod\s+(\w+)\s*\{')
BAD_MODS = {'ext', 'cb', 'tests', 'test', 'hook', 'oracle', 'oracles', 'c', 'mtu'}


def nested_defs(t):
    """Yield (name, mod_chain, open_paren_index) of `pub fn`s at the top level or inside nested `pub mod`s (not ext/cb/tests)."""
    mods = []  # (start, end, name, is_pub)
    for m in MOD_RE.finditer(t):
        e = match_close(t, m.end() - 1)
        if e > 0: mods.append((m.start(), e, m.group(2), bool(m.group(1))))
    for m in DEF_RE.finditer(t):
        enc = [x for x in mods if x[0] < m.start() < x[1]]
        enc.sort()
        if any((not x[3]) or x[2] in BAD_MODS or x[2].startswith('test') for x in enc): continue
        # braces of anything else (impl blocks, fn bodies) between the innermost mod and the def are not allowed
        lo = enc[-1][0] if enc else 0
        seg = t[lo:m.start()]
        base = seg.count('{') - seg.count('}')
        if enc and base != 1: continue
        if not enc and base != 0: continue
        yield m, [x[2] for x in enc]


def index_defs():
    defs = collections.defaultdict(list)   # name -> [(crate, modpath, nparams, hasret, file)]
    for crate, f, rel in tu_files():
        if not rel.startswith('tu/') or 'test' in rel: continue
        t = blank(open(f, encoding='utf8', errors='replace').read())
        for m, chain in nested_defs(t):
            op = m.end() - 1
            cl = match_close(t, op, '(', ')')
            if cl < 0: continue
            plist = t[op+1:cl]
            if '...' in plist: continue
            if '<' in t[m.start():op] and re.search(r'fn\s+\w+\s*<', t[m.start():op+1]): continue   # generic: no hook can call it
            after = t[cl+1:cl+80].lstrip()
            hasret = after.startswith('->') and not re.match(r'->\s*!', after)
            c, mp = modpath_of(f)
            defs[m.group(1)].append((c, '::'.join([mp] + chain), count_params(plist), hasret, rel))
    # `pub use path::{a, b};` in a tu file re-exports a function defined outside tu/ (melee_world::gm ...): the hook can
    # be pointed at the re-export, with the signature of the original (found anywhere in the crates, if unique).
    allf = collections.defaultdict(list)
    for crate, f, rel in tu_files():
        t = blank(open(f, encoding='utf8', errors='replace').read())
        for m, _chain in nested_defs(t):
            op = m.end() - 1
            cl = match_close(t, op, '(', ')')
            if cl < 0: continue
            plist = t[op+1:cl]
            if '...' in plist: continue
            after = t[cl+1:cl+80].lstrip()
            allf[m.group(1)].append((count_params(plist), after.startswith('->') and not re.match(r'->\s*!', after)))
    for crate, f, rel in tu_files():
        if not rel.startswith('tu/') or 'test' in rel: continue
        t = blank(open(f, encoding='utf8', errors='replace').read())
        c, mp = modpath_of(f)
        for m in re.finditer(r'(?m)^pub\s+use\s+([\w:]+)::(\{[^}]*\}|\w+)\s*;', t):
            names = [x.strip() for x in m.group(2).strip('{}').split(',') if x.strip()]
            for n in names:
                if ' as ' in n or n == '*' or n.startswith('self'): continue
                if n in defs and any(d[1] == mp for d in defs[n]): continue
                sig = set(allf.get(n, []))
                if len(sig) == 1:
                    npar, hasret = next(iter(sig))
                    defs[n].append((c, mp, npar, hasret, rel))
    return defs


ENTRY_HEAD = re.compile(r'\s*(?:(?P<hook>[A-Za-z_]\w*)\s+(?=[A-Za-z_]\w*\s*\()|fn\s+)?(?P<name>[A-Za-z_]\w*)\s*\(')


def _ptypes(text):
    out = []
    for x in split_top(text, ','):
        x = x.strip()
        if not x: continue
        m = re.match(r'^(?:mut\s+)?\w+\s*:\s*(.*)$', x, re.S)
        out.append(re.sub(r'\s+', ' ', (m.group(1) if m else x).strip()))
    return out


class _Swapped:
    """A regex match whose first two groups are exchanged (macros that take the C name before the static's name)."""
    def __init__(self, m): self.m = m
    def group(self, i): return self.m.group({1: 2, 2: 1}.get(i, i))
    def end(self): return self.m.end()


def parse_hooks():
    """Return list of dicts: crate, modpath(of invocation), name, nparams, hasret, todo, kind, path."""
    hooks = []
    for crate, f, rel in tu_files():
        raw = open(f, encoding='utf8', errors='replace').read()
        if 'ext_fns' not in raw and 'stubs!' not in raw and 'hooks!' not in raw: continue
        t = blank(raw)
        c, mp = modpath_of(f)
        # classify macro definitions in this file
        dm = re.search(r'macro_rules!\s+(?:ext_fns|stubs|hooks)\s*\{', t)
        kind = 'none'
        if dm:
            end = match_close(t, dm.end() - 1)
            body = t[dm.start():end]
            if 'pub mod hook' in body: kind = 'mod_hook'
            elif '$hook' in body or re.search(r'pub\s+static\s+mut\s+\$stat\b', body): kind = 'static_hook'
            else: kind = 'other'
        else:
            kind = 'mod_hook' if re.search(r'ext_fns_[ch]\b', t) else 'external_macro'
        for im in re.finditer(r'\b(?:ext_fns(?:_[ch])?|stubs|hooks)!\s*\{', t):
            if dm and dm.start() <= im.start() <= match_close(t, dm.end() - 1): continue
            if t[max(0, im.start()-20):im.start()].strip().endswith('$crate::'): pass
            op = im.end() - 1
            cl = match_close(t, op)
            if cl < 0: continue
            # enclosing inline mod chain
            chain = []
            for mm in re.finditer(r'\bmod\s+(\w+)\s*\{', t[:im.start()]):
                o = mm.end() - 1
                e = match_close(t, o)
                if e < 0 or e > im.start(): chain.append(mm.group(1))
            body = t[op+1:cl]
            for ent in split_top(body, ';'):
                if not ent.strip(): continue
                ent = re.sub(r'^\s*extern\s+', ' ', ent)
                m = ENTRY_HEAD.match(ent)
                if not m: continue
                name = m.group('name'); hk = m.group('hook')
                if hk == 'fn': hk = None
                p0 = m.end() - 1
                p1 = match_close(ent, p0, '(', ')')
                if p1 < 0: continue
                _pt = ent[p0+1:p1]
                npar = count_params(_pt)
                rest = ent[p1+1:]
                rm = re.match(r'\s*->', rest)
                hasret = bool(rm)
                dflt = ''
                dm2 = re.search(r'(?<![=!<>])=>?(?![=>])', rest)
                if dm2: dflt = rest[dm2.end():]
                todo = ('todo!' in dflt or 'panic!' in dflt or 'unimplemented!' in dflt) or not dflt.strip()
                mod_chain = '::'.join([mp] + chain)
                if kind == 'mod_hook':
                    path = f'{mod_chain}::hook::{name}'
                elif kind == 'static_hook' and hk:
                    path = f'{mod_chain}::{hk}'
                else:
                    path = None
                rt = re.match(r'\s*->\s*(.+?)\s*(?:(?<![=!<>])=>?(?![=>])|$)', rest, re.S)
                hooks.append(dict(ptypes=_ptypes(_pt), crate=c, file=rel, name=name, npar=npar, hasret=hasret, todo=todo,
                                  kind=kind, path=path, mod=mod_chain, ret=rt.group(1).strip() if rt else None,
                                  byname=(kind == 'external_macro' and crate == 'melee_modes')))
    # `lb_hook!(H_NAME, NAME, (a: T, ..) -> R [= default]);` (melee_runtime lb* TUs): the static is `<mods>::H_NAME`
    for crate, f, rel in tu_files():
        raw = open(f, encoding='utf8', errors='replace').read()
        if 'lb_hook!(' not in raw: continue
        t = blank(raw)
        c, mp = modpath_of(f)
        for im in re.finditer(r'(?<![\w!])lb_hook!\s*\(', t):
            if re.match(r'\s*\$h', t[im.end():im.end() + 8]): continue
            cl = match_close(t, im.end() - 1, '(', ')')
            body = t[im.end():cl]
            m = re.match(r'\s*(\w+)\s*,\s*(\w+)\s*,\s*\(', body)
            if not m: continue
            p1 = match_close(body, m.end() - 1, '(', ')')
            _pt = body[m.end():p1]
            npar = count_params(_pt)
            rest = body[p1 + 1:]
            hasret = bool(re.match(r'\s*->', rest))
            dm2 = re.search(r'(?<![=!<>])=(?![=>])', rest)
            dflt = rest[dm2.end():] if dm2 else ''
            chain = []
            for mm in re.finditer(r'\bmod\s+(\w+)\s*\{', t[:im.start()]):
                o = mm.end() - 1
                e = match_close(t, o)
                if e < 0 or e > im.start(): chain.append(mm.group(1))
            mod_chain = '::'.join([mp] + chain)
            todo = ('todo!' in dflt or 'panic!' in dflt or 'unimplemented!' in dflt) or not dflt.strip()
            hooks.append(dict(ptypes=_ptypes(_pt), crate=c, file=rel, name=m.group(2), npar=npar, hasret=hasret, todo=todo,
                              kind='lb_hook', path=f'{mod_chain}::{m.group(1)}', mod=mod_chain,
                              ret=(re.match(r'\s*->\s*(.+?)\s*(?:=|$)', rest, re.S).group(1).strip() if hasret else None)))
    # `hook!(H_NAME, name, (a: T, ..) [-> R] [, default_path]);` (melee_modes gm* TUs): static `<mods>::H_NAME`
    for crate, f, rel in tu_files():
        raw = open(f, encoding='utf8', errors='replace').read()
        if not re.search(r'(?<![\w!])hook!\s*\(', raw): continue
        t = blank(raw)
        dm = re.search(r'macro_rules!\s+hook\s*\{', t)
        swap_args = False
        if dm:
            mb = t[dm.start():match_close(t, dm.end() - 1)]
            first = re.search(r'\(\s*\$(\w+)\s*:\s*ident\s*,\s*\$(\w+)\s*:\s*ident', mb)
            st = re.search(r'pub\s+static\s+mut\s+\$(\w+)', mb)
            if first and st and st.group(1) == first.group(2): swap_args = True
        macro_cabi = bool(dm and re.search(r'Option<\s*unsafe\s+extern', t[dm.start():match_close(t, dm.end() - 1)]))
        if dm:
            if not re.search(r'pub\s+static\s+mut\s+\$\w+', t[dm.start():match_close(t, dm.end() - 1)]): continue
        elif not re.search(r'hookmac::(?:\{[^}]*\bhook\b[^}]*\}|hook\b)', t): continue
        c, mp = modpath_of(f)
        for im in re.finditer(r'(?<![\w!])hook!\s*\(', t):
            cl = match_close(t, im.end() - 1, '(', ')')
            body = t[im.end():cl]
            m = re.match(r'\s*([A-Za-z_]\w*)\s*,\s*([A-Za-z_]\w*)\s*,\s*\(', body)
            if not m or m.group(1).startswith('$'): continue
            if swap_args: m = _Swapped(m)
            p1 = match_close(body, m.end() - 1, '(', ')')
            _pt = body[m.end():p1]
            npar = count_params(_pt)
            rest = body[p1 + 1:]
            rm = re.match(r'\s*->\s*(.+?)\s*(?:,\s*[\w:]+\s*)?$', rest, re.S)
            hasret = bool(re.match(r'\s*->', rest))
            ret = rm.group(1).strip() if rm else None
            has_default = bool(re.search(r',\s*[A-Za-z_][\w:]*\s*$', rest))
            chain = []
            for mm in re.finditer(r'\bmod\s+(\w+)\s*\{', t[:im.start()]):
                o = mm.end() - 1
                e = match_close(t, o)
                if e < 0 or e > im.start(): chain.append(mm.group(1))
            mod_chain = '::'.join([mp] + chain)
            hooks.append(dict(ptypes=_ptypes(_pt), crate=c, file=rel, name=m.group(2), npar=npar, hasret=hasret, todo=not has_default,
                              kind='hook_macro', path=f'{mod_chain}::{m.group(1)}', mod=mod_chain, ret=ret, cabi=macro_cabi, ref='&' in body[m.end():p1]))
    # generic pass: `pub static mut NAME: Option<fn(..) [-> R]> = None;` directly inside any `mod hook { }` (hand-written
    # hook modules and macro forms that expand to one), when no earlier form already produced that path.
    have = {(h['crate'], h['path']) for h in hooks if h['path']}
    for crate, f, rel in tu_files():
        raw = open(f, encoding='utf8', errors='replace').read()
        if 'mod hook' not in raw: continue
        t = blank(raw)
        c, mp = modpath_of(f)
        for hm in re.finditer(r'(?:pub\s+)?mod\s+hook\s*\{', t):
            e = match_close(t, hm.end() - 1)
            if e < 0: continue
            chain = []
            for mm in re.finditer(r'(?:^|[^\w])mod\s+(\w+)\s*\{', t[:hm.start()]):
                o = mm.end() - 1
                e2 = match_close(t, o)
                if e2 < 0 or e2 > hm.start(): chain.append(mm.group(1))
            mod_chain = '::'.join([mp] + chain + ['hook'])
            for sm in re.finditer(r'pub\s+static\s+mut\s+(\w+)\s*:\s*Option<\s*(?:unsafe\s+)?(extern\s+)?fn\s*\(', t[hm.end():e]):
                name = sm.group(1)
                path = f'{mod_chain}::{name}'
                if (c, path) in have: continue
                op = hm.end() + sm.end() - 1
                cl = match_close(t, op, '(', ')')
                if cl < 0: continue
                _pt = t[op + 1:cl]
                npar = count_params(_pt)
                rest = t[cl + 1:cl + 200]
                rm = re.match(r'\s*->\s*(.+?)\s*>\s*=\s*None', rest, re.S)
                hooks.append(dict(ptypes=_ptypes(_pt), crate=c, file=rel, name=name, npar=npar, hasret=bool(rm), todo=True,
                                  kind='mod_hook_static', path=path, mod=mod_chain, ret=rm.group(1).strip() if rm else None, cabi=bool(sm.group(2)), ref='&' in t[op + 1:cl]))
    # `pub static mut H_name: Option<[unsafe] [extern "C"] fn(..) [-> R]> = None;` next to a function `name` (the
    # hand-written / generated `H_` form of melee_world and melee_runtime TUs)
    have = {(h['crate'], h['path']) for h in hooks if h['path']}
    for crate, f, rel in tu_files():
        raw = open(f, encoding='utf8', errors='replace').read()
        if 'static mut H_' not in raw: continue
        t = blank(raw)
        c, mp = modpath_of(f)
        for sm in re.finditer(r'pub\s+static\s+mut\s+H_(\w+)\s*:\s*Option<\s*(?:unsafe\s+)?(extern\s+)?fn\s*\(', t):
            name = sm.group(1)
            chain = []
            for mm in re.finditer(r'(?:^|[^\w])mod\s+(\w+)\s*\{', t[:sm.start()]):
                o = mm.end() - 1
                e2 = match_close(t, o)
                if e2 < 0 or e2 > sm.start(): chain.append(mm.group(1))
            path = '::'.join([mp] + chain + ['H_' + name])
            if (c, path) in have: continue
            op = sm.end() - 1
            cl = match_close(t, op, '(', ')')
            if cl < 0: continue
            _pt = t[op + 1:cl]
            npar = count_params(_pt)
            rm = re.match(r'\s*->\s*(.+?)\s*>\s*=\s*None', t[cl + 1:cl + 200], re.S)
            hooks.append(dict(ptypes=_ptypes(_pt), crate=c, file=rel, name=name, npar=npar, hasret=bool(rm), todo=True,
                              kind='h_static', path=path, mod='::'.join([mp] + chain), ret=rm.group(1).strip() if rm else None, cabi=bool(sm.group(2)), ref='&' in t[op + 1:cl]))
    # `pub static mut UPPER_NAME: Option<fn(..)> = None;` followed by `pub fn upper_name(..)` (case-insensitive name match)
    have = {(h['crate'], h['path']) for h in hooks if h['path']}
    for crate, f, rel in tu_files():
        raw = open(f, encoding='utf8', errors='replace').read()
        if not re.search(r'static\s+mut\s+[A-Z][A-Z0-9_]*\s*:\s*Option<', raw): continue
        t = blank(raw)
        c, mp = modpath_of(f)
        fnames = {}
        for fm in re.finditer(r'\bpub\s+(?:unsafe\s+)?(?:extern\s+)?fn\s+(\w+)\s*\(', t):
            fnames.setdefault(fm.group(1).upper().replace('_', ''), fm.group(1))
        for sm in re.finditer(r'pub\s+static\s+mut\s+([A-Z][A-Z0-9_]*)\s*:\s*Option<\s*(?:unsafe\s+)?(extern\s+)?fn\s*\(', t):
            nm = fnames.get(sm.group(1).replace('_', ''))
            if not nm: continue
            chain = []
            for mm in re.finditer(r'(?:^|[^\w])mod\s+(\w+)\s*\{', t[:sm.start()]):
                o = mm.end() - 1
                e2 = match_close(t, o)
                if e2 < 0 or e2 > sm.start(): chain.append(mm.group(1))
            path = '::'.join([mp] + chain + [sm.group(1)])
            if (c, path) in have: continue
            op = sm.end() - 1
            cl = match_close(t, op, '(', ')')
            if cl < 0: continue
            _pt = t[op + 1:cl]
            npar = count_params(_pt)
            rm = re.match(r'\s*->\s*(.+?)\s*>\s*=\s*None', t[cl + 1:cl + 200], re.S)
            hooks.append(dict(ptypes=_ptypes(_pt), crate=c, file=rel, name=nm, npar=npar, hasret=bool(rm), todo=True,
                              kind='upper_static', path=path, mod='::'.join([mp] + chain), ret=rm.group(1).strip() if rm else None,
                              cabi=bool(sm.group(2)), ref='&' in t[op + 1:cl]))
    return hooks


def cfile_map():
    """C function name -> C file stem (lowercase) from the committed traces."""
    m = {}
    for p in glob.glob(os.path.join(ROOT, 'docs', 'rust-port', 'trace', '*.csv')):
        for ln in open(p, encoding='utf8', errors='replace').read().splitlines()[1:]:
            parts = ln.split(',')
            if len(parts) >= 2:
                m.setdefault(parts[0], os.path.splitext(os.path.basename(parts[1]))[0].lower())
    return m


def choose(cands, name, cmap, own_file):
    cands = [c for c in cands if c[4] != own_file]
    if len(cands) == 1: return cands[0]
    if not cands: return None
    stem = cmap.get(name)
    if stem:
        pick = [c for c in cands if os.path.splitext(os.path.basename(c[4]))[0].lower() == stem
                or os.path.splitext(os.path.basename(c[4]))[0].lower().replace('_', '') == stem.replace('_', '')]
        if len(pick) == 1: return pick[0]
    return None


LB_FAMILY = {'lbArchive_LoadSymbols': ('LoadSymbols', 0), 'lbArchive_80016DBC': ('80016DBC', 0),
             'lbArchive_80017040': ('80017040', 1), 'lbArchive_800171CC': ('800171CC', 1)}


def variadic_adapter(h):
    """Body for a hook that spells out the variadic `lbArchive_*(file, symbols, name, ..., NULL)` family with a fixed
    parameter list or a slice: build the `rest` slice the native loader takes. None if the shape is not understood."""
    kind, has_dst = LB_FAMILY[h['name']]
    pt = h.get('ptypes')
    if pt is None or len(pt) < 1 + has_dst: return None
    npar = len(pt)
    def cls(t):
        t = t.replace('::core::ffi::c_char', 'i8').replace('core::ffi::c_char', 'i8').replace('c_char', 'i8')
        if re.fullmatch(r'\*(?:const|mut) (?:i8|u8)', t): return 'str'
        if re.fullmatch(r'\*mut \*mut [\w:]+', t): return 'dst'
        if re.fullmatch(r'\*(?:const|mut) (?:::)?(?:core::ffi::)?c_void', t): return 'ptr'
        if t in ('i32', 'u32', 'c_int'): return 'int'
        if re.fullmatch(r'&\[\*(?:const|mut) (?:::)?(?:core::ffi::)?c_void\]', t): return 'slice'
        if re.fullmatch(r'&\[\(\*mut c_void, \*const u8\)\]', t): return 'tuples'
        return None
    kinds = [cls(t) for t in pt]
    if None in kinds: return None
    i = 0
    dst = 'core::ptr::null_mut()'
    if has_dst:
        if kinds[0] != 'dst': return None
        dst = 'a0 as *mut *mut melee_types::HSD_Archive'
        i = 1
    if kinds[i] != 'str': return None
    file = f'a{i} as *const core::ffi::c_char'
    lines = ['let mut v: std::vec::Vec<*mut core::ffi::c_void> = std::vec::Vec::new();']
    for j in range(i + 1, npar):
        k = kinds[j]
        if k in ('str', 'ptr'): lines.append(f'v.push(a{j} as *mut core::ffi::c_void);')
        elif k == 'int': pass
        elif k == 'slice': lines.append(f'v.extend(a{j}.iter().map(|p| *p as *mut core::ffi::c_void));')
        elif k == 'tuples': lines.append(f'for (p, q) in a{j}.iter() {{ v.push(*p as *mut core::ffi::c_void); v.push(*q as *mut core::ffi::c_void); }}')
        else: return None
    lines.append('if v.last().map_or(true, |p| !p.is_null()) { v.push(core::ptr::null_mut()); }')
    call = {'LoadSymbols': f'melee_runtime::tu::lbarchive::lbArchive_LoadSymbols({file}, &v)',
            '80016DBC': f'melee_runtime::tu::lbarchive::lbArchive_80016DBC({file}, &v)',
            '80017040': f'melee_runtime::tu::lbarchive::lbArchive_80017040({dst}, {file}, &v)',
            '800171CC': f'melee_runtime::tu::lbarchive::lbArchive_800171CC({dst}, {file}, &v)'}[kind]
    if not h['hasret']: lines.append(f'let _ = {call};')
    else: lines.append(f'{call} as _')
    return ' '.join(lines)


def main():
    report = '--report' in sys.argv
    skip = set()
    sp = os.path.join(os.path.dirname(__file__), 'link_skip.txt')
    if os.path.exists(sp):
        skip = {l.split()[0] for l in open(sp) if l.strip() and not l.startswith('#')}
    defs = index_defs()
    hooks = parse_hooks()
    cmap = cfile_map()
    linked = []; unresolved = []
    seen = set()
    for h in hooks:
        key = h['path'] or (h['file'] + '::' + h['name'])
        if key in seen: continue
        seen.add(key)
        why = None
        if h['name'] in LB_FAMILY and h['path'] and not (h.get('cabi') and h.get('ref')):
            ad = variadic_adapter(h)
            if ad:
                h['custom'] = ad
                linked.append(h)
                continue
        if h['path'] is None:
            why = 'other'
        elif h.get('cabi') and h.get('ref'):
            why = 'cabiref'
        else:
            cands = defs.get(h['name'], [])
            if not cands: why = 'nodef'
            else:
                ok = [c for c in cands if c[2] == h['npar'] and c[3] == h['hasret']]
                pick = choose(ok, h['name'], cmap, h['file'])
                if pick is None:
                    why = 'ambiguous' if len([c for c in ok if c[4] != h['file']]) > 1 else ('selfdef' if cands and all(c[4] == h['file'] for c in cands) else 'sigmismatch')
                elif key in skip or h['name'] in skip:
                    why = 'skipped'
                else:
                    h['canon'] = pick
        if why:
            unresolved.append((why, h))
        else:
            linked.append(h)
    cnt = collections.Counter(w for w, _ in unresolved)
    print(f'hooks {len(seen)}: linked {len(linked)}; unresolved {dict(cnt)}')
    reached = set()
    for p in glob.glob(os.path.join(ROOT, 'docs', 'rust-port', 'trace', '*.csv')):
        for ln in open(p, encoding='utf8', errors='replace').read().splitlines()[1:]:
            reached.add(ln.split(',')[0])
    todo_unlinked = [h for w, h in unresolved if h['todo']]
    print(f'  of which default is todo/panic: {len(todo_unlinked)}; trace-reached: {sum(1 for h in todo_unlinked if h["name"] in reached)}')
    if report: return

    link_dir = os.path.join(RUST, 'melee_link')
    os.makedirs(os.path.join(link_dir, 'src'), exist_ok=True)
    out = ['// @generated by tools/rust-port/link/gen_link.py. Do not edit.',
           '#![allow(non_snake_case, unused_unsafe, clippy::all)]',
           'use crate::cast;', '']
    by_crate = collections.defaultdict(list)
    for h in linked: by_crate[h['crate']].append(h)
    site = 0
    for crate in LEAVES:
        out.append(f'fn install_{crate}() {{')
        for h in sorted(by_crate.get(crate, []), key=lambda x: x['path']):
            if h.get('custom'):
                npar = h['npar']
                args = ', '.join(f'a{i}' for i in range(npar))
                out.append(f'    // {h["file"]}')
                if h.get('cabi'):
                    out.append(f'    unsafe {{ {crate}::{h["path"]} = Some(crate::cabi::c{npar}(|{args}| unsafe {{ {h["custom"]} }})); }}')
                else:
                    out.append(f'    unsafe {{ {crate}::{h["path"]} = Some(|{args}| unsafe {{ {h["custom"]} }}); }}')
                continue
            cc, cmod, npar, hasret, _ = h['canon']
            args = ', '.join(f'a{i}' for i in range(npar))
            site += 1
            S = site * 100
            call_args = ', '.join(f'cast::<_, _, {S + i + 1}>(a{i})' for i in range(npar))
            callee = f'{cc}::{cmod}::{h["name"]}'
            body = f'cast::<_, _, {S}>({callee}({call_args}))' if hasret else f'{callee}({call_args})'
            out.append(f'    // {h["file"]}')
            if h.get('cabi'):
                out.append(f'    unsafe {{ {crate}::{h["path"]} = Some(crate::cabi::c{npar}(|{args}| unsafe {{ {body} }})); }}')
            else:
                out.append(f'    unsafe {{ {crate}::{h["path"]} = Some(|{args}| unsafe {{ {body} }}); }}')
        out.append('}')
        out.append('')
    out.append('pub fn install_all() {')
    for crate in LEAVES: out.append(f'    install_{crate}();')
    out.append('}')
    out.append('')
    out.append('/// (hook path, linked) for every hook entry the generator saw; `todo` = default body is a todo!()/panic.')
    out.append('pub static HOOKS: &[(&str, &str, bool, bool)] = &[')
    for h in sorted(linked, key=lambda x: (x['crate'], x['path'])):
        out.append(f'    ("{h["name"]}", "{h["crate"]}::{h["path"]}", true, {str(h["todo"]).lower()}),')
    for w, h in sorted(unresolved, key=lambda x: (x[1]['crate'], x[1]['file'], x[1]['name'])):
        out.append(f'    ("{h["name"]}", "{h["crate"]}::{h["path"] or h["file"]}", false, {str(h["todo"]).lower()}),')
    out.append('];')
    open(os.path.join(link_dir, 'src', 'gen_table.rs'), 'w', encoding='utf8', newline='\n').write('\n'.join(out) + '\n')
    with open(os.path.join(link_dir, 'unresolved.txt'), 'w', newline='\n') as fh:
        fh.write('# kind name file todo(default is todo!/panic)  [regenerated by gen_link.py]\n')
        for w, h in sorted(unresolved, key=lambda x: (x[0], x[1]['file'], x[1]['name'])):
            fh.write(f'{w} {h["name"]} {h["crate"]}/{h["file"]} {int(h["todo"])}\n')
    # key list used by link_skip.txt
    with open(os.path.join(link_dir, 'linked_keys.txt'), 'w', newline='\n') as fh:
        for h in linked: fh.write(h['path'] + '\n')


if __name__ == '__main__':
    main()
