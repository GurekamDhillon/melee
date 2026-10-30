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


DEF_RE = re.compile(r'(?m)^[ \t]*pub\s+(?:unsafe\s+)?(?:extern\s+"C"\s+)?fn\s+(\w+)\s*\(')


def index_defs():
    defs = collections.defaultdict(list)   # name -> [(crate, modpath, nparams, hasret, file)]
    for crate, f, rel in tu_files():
        if not rel.startswith('tu/'): continue
        t = blank(open(f, encoding='utf8', errors='replace').read())
        depth_at = {}
        d = 0; last = 0
        for m in DEF_RE.finditer(t):
            d += t.count('{', last, m.start()) - t.count('}', last, m.start()); last = m.start()
            if d != 0: continue
            # make sure #[no_mangle] sits right before this fn (no `}` in between guaranteed by [^{;]*?)
            op = m.end() - 1
            cl = match_close(t, op, '(', ')')
            if cl < 0: continue
            plist = t[op+1:cl]
            if '...' in plist: continue
            after = t[cl+1:cl+80].lstrip()
            hasret = after.startswith('->') and not re.match(r'->\s*!', after)
            c, mp = modpath_of(f)
            defs[m.group(1)].append((c, mp, count_params(plist), hasret, rel))
    return defs


ENTRY_HEAD = re.compile(r'\s*(?:(?P<hook>[A-Za-z_]\w*)\s+(?=[A-Za-z_]\w*\s*\()|fn\s+)?(?P<name>[A-Za-z_]\w*)\s*\(')


def parse_hooks():
    """Return list of dicts: crate, modpath(of invocation), name, nparams, hasret, todo, kind, path."""
    hooks = []
    for crate, f, rel in tu_files():
        raw = open(f, encoding='utf8', errors='replace').read()
        if 'ext_fns!' not in raw: continue
        t = blank(raw)
        c, mp = modpath_of(f)
        # classify macro definitions in this file
        dm = re.search(r'macro_rules!\s+ext_fns\s*\{', t)
        kind = 'none'
        if dm:
            end = match_close(t, dm.end() - 1)
            body = t[dm.start():end]
            if 'pub mod hook' in body: kind = 'mod_hook'
            elif '$hook' in body: kind = 'static_hook'
            else: kind = 'other'
        else:
            kind = 'external_macro'
        for im in re.finditer(r'\bext_fns!\s*\{', t):
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
                m = ENTRY_HEAD.match(ent)
                if not m: continue
                name = m.group('name'); hk = m.group('hook')
                p0 = m.end() - 1
                p1 = match_close(ent, p0, '(', ')')
                if p1 < 0: continue
                npar = count_params(ent[p0+1:p1])
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
                hooks.append(dict(crate=c, file=rel, name=name, npar=npar, hasret=hasret, todo=todo,
                                  kind=kind, path=path, mod=mod_chain))
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
            npar = count_params(body[m.end():p1])
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
            hooks.append(dict(crate=c, file=rel, name=m.group(2), npar=npar, hasret=hasret, todo=todo,
                              kind='lb_hook', path=f'{mod_chain}::{m.group(1)}', mod=mod_chain))
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
        if h['path'] is None:
            why = 'other'
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
            cc, cmod, npar, hasret, _ = h['canon']
            args = ', '.join(f'a{i}' for i in range(npar))
            site += 1
            S = site * 100
            call_args = ', '.join(f'cast::<_, _, {S + i + 1}>(a{i})' for i in range(npar))
            callee = f'{cc}::{cmod}::{h["name"]}'
            body = f'cast::<_, _, {S}>({callee}({call_args}))' if hasret else f'{callee}({call_args})'
            out.append(f'    // {h["file"]}')
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
