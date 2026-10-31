#!/usr/bin/env python3
"""Generate rust/melee_link/src/retail.rs: the per-character static initialiser tables of ftdata.c.

The C fills `ftData_OnLoad[]`, `ftData_SpecialS[]`, ... (one slot per fighter kind, 33 retail rows) with static
initialisers.  The native `melee_fighter::tu::ftdata` statics start empty, so the engine calls `install_retail()`
once at boot.  The element lists are read from the transpiled initialisers in `melee_fighter/src/c2r/ft/ftdata.rs`
(the data only; no c2r function is ever called) and every symbol is resolved to the canonical `tu/` item:

  Some(sym as unsafe extern "C" fn(..) -> R)  ->  Some(cabi::cN(|a0, ..| canonical(cast(a0), ..)))
  &raw const|mut sym as *mut T                ->  the canonical static's address
  Type { f: expr, .. }                        ->  the same per field

A symbol with no canonical item leaves its slot empty and is listed in retail_unresolved.txt.
Usage: gen_retail.py            (from anywhere)
"""
import os, re, sys, collections
sys.path.insert(0, os.path.dirname(__file__))
import gen_link as g

C2R = os.path.join(g.RUST, 'melee_fighter', 'src', 'c2r', 'ft', 'ftdata.rs')
TU = os.path.join(g.RUST, 'melee_fighter', 'src', 'tu', 'ftdata.rs')
OUT = os.path.join(g.RUST, 'melee_link', 'src', 'retail.rs')
UNRES = os.path.join(g.RUST, 'melee_link', 'retail_unresolved.txt')

STATIC_RE = re.compile(r'(?m)^[ \t]*pub\s+static\s+(?:mut\s+)?(\w+)\s*:')


def static_index():
    idx = collections.defaultdict(list)
    for crate, f, rel in g.tu_files():
        if not rel.startswith('tu/') or 'test' in rel: continue
        t = g.blank(open(f, encoding='utf8', errors='replace').read())
        c, mp = g.modpath_of(f)
        d = 0; last = 0
        for m in STATIC_RE.finditer(t):
            d += t.count('{', last, m.start()) - t.count('}', last, m.start()); last = m.start()
            if d != 0: continue
            idx[m.group(1)].append(f'{c}::{mp}::{m.group(1)}')
    return idx


CONST_RE = re.compile(r'(?m)^[ 	]*pub\s+const\s+(\w+)\s*:\s*([^=]+?)\s*=')


def const_index():
    idx = collections.defaultdict(list)
    for crate, f, rel in g.tu_files():
        if not rel.startswith('tu/') or 'test' in rel: continue
        t = g.blank(open(f, encoding='utf8', errors='replace').read())
        c, mp = g.modpath_of(f)
        d = 0; last = 0
        for m in CONST_RE.finditer(t):
            d += t.count('{', last, m.start()) - t.count('}', last, m.start()); last = m.start()
            if d != 0: continue
            idx[m.group(1)].append((f'{c}::{mp}::{m.group(1)}', re.sub(r'\s+', '', m.group(2))))
    return idx


def pick_fn(cands, name, cmap):
    cands = [c for c in cands if c[0] in ('melee_chara', 'melee_fighter', 'melee_items', 'melee_world', 'melee_runtime', 'melee_modes')]
    if len(cands) == 1: return cands[0]
    if not cands: return None
    stem = cmap.get(name)
    if stem:
        pick = [c for c in cands if os.path.splitext(os.path.basename(c[4]))[0].lower().replace('_', '') == stem.replace('_', '')]
        if len(pick) == 1: return pick[0]
    pref = [c for c in cands if c[0] == 'melee_chara']
    if len(pref) == 1: return pref[0]
    return None


class Gen:
    def __init__(self):
        self.defs = g.index_defs()
        self.statics = static_index()
        self.consts = const_index()
        self.cmap = g.cfile_map()
        self.site = 0
        self.unres = []
        self.out = []

    def fn_expr(self, name, args_text, ret_text, where):
        cands = self.defs.get(name, [])
        c = pick_fn(cands, name, self.cmap)
        if c is None:
            self.unres.append(f'fn {name} ({where}): ' + ('ambiguous' if cands else 'no canonical'))
            return None
        narg = g.count_params(args_text) if args_text.strip() else 0
        if c[2] != narg:
            self.unres.append(f'fn {name} ({where}): arity {c[2]} vs cast {narg}')
            return None
        self.site += 1
        S = self.site * 100
        params = ', '.join(f'a{i}' for i in range(narg))
        call_args = ', '.join(f'cast::<_, _, {S + i + 1}>(a{i})' for i in range(narg))
        callee = f'{c[0]}::{c[1]}::{name}'
        unit = (ret_text is None) or ret_text.strip() in ('()', '')
        if unit:
            body = f'let _ = {callee}({call_args});'
        else:
            body = f'cast::<_, _, {S}>({callee}({call_args}))'
        return f'Some(crate::cabi::c{narg}(|{params}| unsafe {{ {body} }}))'

    def static_expr(self, name, mut_kw, ty, where):
        k = self.consts.get(name, [])
        if len(k) == 1 and name not in self.statics:
            path, cty = k[0]
            if cty == '&str': return f'crate::strs::cstr({path})'
            if re.fullmatch(r'\[&str;\d+\]', cty): return f'crate::strs::demo_strings(&{path})'
            if re.fullmatch(r'\[\[&str;\d+\];\d+\]', cty): return f'crate::strs::costume_strings(&{path})'
            self.unres.append(f'const {name} ({where}): unsupported type {cty}')
            return None
        c = self.statics.get(name, [])
        if len(c) != 1:
            self.unres.append(f'static {name} ({where}): ' + ('ambiguous' if c else 'no canonical'))
            return None
        return f'&raw {mut_kw} {c[0]} as *mut _'

    def expr(self, e, where):
        """Rust expression for one initialiser element, or None to leave the slot as the default."""
        e = e.strip()
        if not e or e == 'None' or 'ptr::null' in e or e.startswith('::core::ptr::null'): return None
        fm = re.fullmatch(r'Some\(\s*(\w+)\s+as\s+unsafe\s+extern\s*"?C"?\s*fn\((.*?)\)\s*(?:->\s*(.+?))?\s*,?\s*\)', e, re.S)
        if fm:
            return self.fn_expr(fm.group(1), fm.group(2), fm.group(3), where)
        sm = re.fullmatch(r'&raw\s+(const|mut)\s+(\w+)\s+as\s+\*mut\s+.+', e, re.S)
        if sm:
            return self.static_expr(sm.group(2), sm.group(1), None, where)
        return 'STRUCT:' + e

    def table(self, tname, elems):
        n = 0
        for i, e in enumerate(elems):
            ex = self.expr(e, f'{tname}[{i}]')
            if ex is None: continue
            if ex.startswith('STRUCT:'):
                body = ex[7:]
                m = re.fullmatch(r'(\w+)\s*\{(.*)\}', body, re.S)
                if not m:
                    self.unres.append(f'{tname}[{i}]: unsupported element {body[:60]!r}')
                    continue
                for fld in g.split_top(m.group(2), ','):
                    fld = fld.strip()
                    if not fld: continue
                    fm = re.fullmatch(r'(\w+)\s*:\s*(.+)', fld, re.S)
                    fx = self.expr(fm.group(2), f'{tname}[{i}].{fm.group(1)}') if fm else None
                    if fx is None: continue
                    if fx.startswith('STRUCT:'):
                        self.unres.append(f'{tname}[{i}].{fm.group(1)}: unsupported {fx[7:60]!r}')
                        continue
                    self.out.append(f'    (*t{self.tn})[{i}].{fm.group(1)} = {fx};')
                    n += 1
                continue
            self.out.append(f'    (*t{self.tn})[{i}] = {ex};')
            n += 1
        return n

    def run(self):
        c2r = open(C2R, encoding='utf8').read()
        bl = g.blank(c2r)
        tu = open(TU, encoding='utf8').read()
        names = re.findall(r'(?m)^pub static mut (\w+): \[[^;\]]+; KMAX\] =\s*\[(?:None|core::ptr::null_mut\(\)|\w+ \{.*?\}); KMAX\];', tu)
        names = [n for n in names if n.startswith('ftData_') or n.startswith('ftKind')]
        body = []
        self.tn = 0
        for name in names:
            m = re.search(r'(?m)^pub static mut ' + name + r': \[[^;\]]+; 127\]\s*=\s*', bl)
            if not m: continue
            s = bl.find('[', m.end())
            e = g.match_close(bl, s, '[', ']')
            elems = g.split_top(c2r[s + 1:e], ',')
            self.tn += 1
            self.out.append(f'    let t{self.tn} = &raw mut melee_fighter::tu::ftdata::{name};')
            k = self.table(name, elems)
            self.out.append(f'    // {name}: {k} slots')
        # the model event / getter pair
        m = re.search(r'(?m)^pub static mut ftData_UnkIntBoolFunc0: \w+\s*=\s*unsafe\s*\{', bl)
        if m:
            end = g.match_close(bl, bl.find('{', m.end() - 1))
            blk = c2r[m.end():end]
            blb = bl[m.end():end]
            for fld in ('model_events', 'getter'):
                fm = re.search(fld + r'\s*:\s*\[', blb)
                if not fm: continue
                s = fm.end() - 1
                e = g.match_close(blb, s, '[', ']')
                elems = g.split_top(blk[s + 1:e], ',')
                self.tn += 1
                self.out.append(f'    let t{self.tn} = &raw mut melee_fighter::tu::ftdata::ftData_UnkIntBoolFunc0.{fld};')
                k = self.table('ftData_UnkIntBoolFunc0.' + fld, elems)
                self.out.append(f'    // ftData_UnkIntBoolFunc0.{fld}: {k} slots')
        # costume lists: the rows come from the C source (the c2rust initialiser lost them); the count is the length of
        # the canonical static array (`ARRAY_SIZE`, `FTNANA_COSTUME_COUNT`)
        csrc = open(os.path.join(g.ROOT, 'src', 'melee', 'ft', 'ftdata.c'), encoding='utf8', errors='replace').read()
        m = re.search(r'CostumeListsForeachCharacter\[Ft_Kind_Max\]\s*=\s*\{(.*?)' + chr(10) + r'\};', csrc, re.S)
        if m:
            self.out.append('    let tc = &raw mut melee_fighter::tu::ftdata::CostumeListsForeachCharacter;')
            n = 0
            for i, row in enumerate(re.findall(r'\{\s*(\w+)\s*,', m.group(1))):
                ex = self.static_expr(row, 'mut', None, f'CostumeLists[{i}]')
                if ex is None: continue
                path = ex[len('&raw mut '):-len(' as *mut _')]
                self.out.append(f'    (*tc)[{i}].costume_list = {ex};')
                self.out.append(f'    (*tc)[{i}].numCostumes = (core::mem::size_of_val(&*(&raw const {path})) / core::mem::size_of::<melee_types::UnkCostumeStruct>()) as u8;')
                n += 1
            self.out.append(f'    // CostumeListsForeachCharacter: {n} rows')
        text = ['// @generated by tools/rust-port/link/gen_retail.py. Do not edit.',
                '#![allow(non_snake_case, unused_unsafe, unused_variables, clippy::all)]',
                'use crate::cast;', '',
                '/// ftdata.c\'s static initialisers (the per-kind tables of the 33 retail rows). Call once at boot.',
                'pub fn install_retail() {', '    unsafe {'] + ['    ' + l for l in self.out] + ['    }', '}', '']
        open(OUT, 'w', newline='\n').write('\n'.join(text))
        open(UNRES, 'w', newline='\n').write('\n'.join(sorted(set(self.unres))) + '\n')
        print(f'{len(self.out)} lines, {len(set(self.unres))} unresolved')


if __name__ == '__main__':
    Gen().run()
