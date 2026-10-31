#!/usr/bin/env python3
"""Generate rust/melee_link/src/retail_it.rs: the item logic and render-link tables of the it/*.c data files.

The C fills `it_803F14C4` / `it_803F3100` / `it_803F23CC` / `it_803F4D20` (struct ItemLogicTable, one row per item kind:
the ItemStateTable of the kind and 14 event / predicate callbacks) and the render-link tables `it_803F1418` /
`it_803F2310` / `it_803F2F28` / `it_803F4CA8` (struct sdata_ItemGXLink, one `GObj_RenderFunc` per kind) with static
initialisers of function and table addresses owned by the item TUs.  The native statics start empty (tu/it_3F14.rs,
it_279C.rs, it_3F2F.rs), so the engine calls `install_retail_it()` once at boot.  The rows are read from the C source
and every symbol is resolved to the canonical `tu/` item (a function through a `cabi` wrapper, a table by address):

  src/melee/it/it_3F14.c  it_803F1418[43]  it_803F14C4[43]
  src/melee/it/it_279C.c  it_803F2310[47]  it_803F23CC[47]
  src/melee/it/it_3F2F.c  it_803F2F28[118] it_803F3100[118] it_803F4CA8[30] it_803F4D20[30]

A symbol with no canonical item leaves its slot empty and is listed in retail_it_unresolved.txt.  Usage: gen_retail_it.py
"""
import os, re, sys
sys.path.insert(0, os.path.dirname(__file__))
import gen_link as g
import gen_retail as r

OUT = os.path.join(g.RUST, 'melee_link', 'src', 'retail_it.rs')
UNRES = os.path.join(g.RUST, 'melee_link', 'retail_it_unresolved.txt')
IT = os.path.join(g.ROOT, 'src', 'melee', 'it')

# (C file, table, rust path of the static, kind)
TABLES = [
    ('it_3F14.c', 'it_803F1418', 'melee_items::tu::it_3F14::it_803F1418', 'gx'),
    ('it_3F14.c', 'it_803F14C4', 'melee_items::tu::it_3F14::it_803F14C4', 'logic'),
    ('it_279C.c', 'it_803F2310', 'melee_items::tu::it_3F14::it_803F2310', 'gx'),
    ('it_279C.c', 'it_803F23CC', 'melee_items::tu::it_3F14::it_803F23CC', 'logic'),
    ('it_3F2F.c', 'it_803F2F28', 'melee_items::tu::it_3F14::it_803F2F28', 'gx'),
    ('it_3F2F.c', 'it_803F3100', 'melee_items::tu::it_3F14::it_803F3100', 'logic'),
    ('it_3F2F.c', 'it_803F4CA8', 'melee_items::tu::it_3F14::it_803F4CA8', 'gx'),
    ('it_3F2F.c', 'it_803F4D20', 'melee_items::tu::it_3F14::it_803F4D20', 'logic'),
]
# ItemLogicTable fields after `states`: (field, number of C parameters, returns a value)
LOGIC = [('spawned', 1, False), ('destroyed', 1, False), ('picked_up', 1, False), ('dropped', 1, False), ('thrown', 1, False),
         ('dmg_dealt', 1, True), ('dmg_received', 1, True), ('entered_air', 1, False), ('reflected', 1, True),
         ('clanked', 1, True), ('absorbed', 1, True), ('shield_bounced', 1, True), ('hit_shield', 1, True),
         ('evt_unk', 2, False)]


def strip(c):
    c = re.sub(r'/\*.*?\*/', '', c, flags=re.S)
    c = re.sub(r'//[^\n]*', '', c)
    return re.sub(r'(?m)^\s*#.*$', '', c)


def body_of(text, name):
    m = re.search(r'\b' + name + r'\s*\[[^\]]*\]\s*=\s*\{', text)
    if not m: return None
    s = m.end() - 1
    e = g.match_close(text, s, '{', '}')
    return text[s + 1:e]


class Gen(r.Gen):
    def fn_slot(self, name, narg, ret, where):
        cands = self.defs.get(name, [])
        c = r.pick_fn(cands, name, self.cmap)
        if c is None:
            self.unres.append(f'fn {name} ({where}): ' + ('ambiguous' if cands else 'no canonical'))
            return None
        if c[2] != narg:
            self.unres.append(f'fn {name} ({where}): arity {c[2]} vs {narg}')
            return None
        self.site += 1
        S = self.site * 100
        params = ', '.join(f'a{i}' for i in range(narg))
        call_args = ', '.join(f'cast::<_, _, {S + i + 1}>(a{i})' for i in range(narg))
        callee = f'{c[0]}::{c[1]}::{name}'
        body = f'cast::<_, _, {S}>({callee}({call_args}))' if ret else f'let _ = {callee}({call_args});'
        return f'Some(crate::cabi::c{narg}(|{params}| unsafe {{ {body} }}))'

    def run(self):
        for cfile, tname, path, kind in TABLES:
            text = strip(open(os.path.join(IT, cfile), encoding='utf8', errors='replace').read())
            body = body_of(text, tname)
            if body is None:
                self.unres.append(f'table {tname}: not found in {cfile}')
                continue
            self.out.append(f'let t = &raw mut {path};')
            n = 0
            if kind == 'gx':
                for i, e in enumerate(x.strip() for x in body.split(',') if x.strip()):
                    if e == 'NULL': continue
                    ex = self.fn_slot(e, 2, False, f'{tname}[{i}]')
                    if ex is None: continue
                    self.out.append(f'(*t)[{i}].x0_renderFunc = {ex};')
                    n += 1
            else:
                rows = []
                i = 0
                while True:
                    s = body.find('{', i)
                    if s < 0: break
                    e = g.match_close(body, s, '{', '}')
                    rows.append([x.strip() for x in body[s + 1:e].split(',') if x.strip()])
                    i = e + 1
                for i, row in enumerate(rows):
                    if len(row) != 15:
                        self.unres.append(f'{tname}[{i}]: {len(row)} fields')
                        continue
                    if row[0] != 'NULL':
                        om = re.fullmatch(r'&(\w+)\[(\d+)\]', row[0])
                        ex = self.static_expr(om.group(1) if om else row[0], 'const', None, f'{tname}[{i}].states')
                        if ex is not None and om:
                            ex = f'({ex[:-len(" as *mut _")]} as *mut melee_types::ItemStateTable).add({om.group(2)})'
                        if ex is not None:
                            self.out.append(f'(*t)[{i}].states = {ex};')
                            n += 1
                    for (fld, narg, ret), e in zip(LOGIC, row[1:]):
                        if e == 'NULL': continue
                        ex = self.fn_slot(e, narg, ret, f'{tname}[{i}].{fld}')
                        if ex is None: continue
                        self.out.append(f'(*t)[{i}].{fld} = {ex};')
                        n += 1
            self.out.append(f'// {tname}: {n} slots')
        text = ['// @generated by tools/rust-port/link/gen_retail_it.py. Do not edit.',
                '#![allow(non_snake_case, unused_unsafe, unused_variables, clippy::all)]',
                'use crate::cast;', '',
                "/// The item logic and render-link tables of it_3F14.c, it_279C.c and it_3F2F.c (static initialisers). Call once at boot.",
                'pub fn install_retail_it() {', '    unsafe {'] + ['        ' + l for l in self.out] + ['    }', '}', '']
        open(OUT, 'w', newline='\n').write('\n'.join(text))
        open(UNRES, 'w', newline='\n').write('\n'.join(sorted(set(self.unres))) + '\n')
        print(f'{len(self.out)} lines, {len(set(self.unres))} unresolved')


if __name__ == '__main__':
    Gen().run()
