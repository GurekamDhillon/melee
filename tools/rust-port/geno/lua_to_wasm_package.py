#!/usr/bin/env python3
"""Convert a geno 10/11 define package from fighter Lua to the geno 12 `wasm` block (melee_engine::geno_wasm).

  lua_to_wasm_package.py <mod folder> <module.wasm> [--out <folder>]

Rewrites the folder's geno.json:
- "geno" becomes 12;
- each fighter's "lua": {"script"|"source", "state"} becomes "wasm": {"module": "wasm/<name>.wasm", "state"}, with the
  same state slots;
- each state's "lua": {enter, frame} becomes "wasm": {enter, frame} (the module must export those names).
The module is copied to <folder>/wasm/. Nothing else in the package changes. With --out the folder is copied first
and the copy is converted; otherwise it is converted in place. The Lua script file is left where it was (unused).
Text only: no game data is read or written.
"""
import json, os, shutil, sys


def convert(folder, module):
    path = os.path.join(folder, 'geno.json')
    with open(path, encoding='utf8') as f:
        doc = json.load(f)
    rel = 'wasm/' + os.path.basename(module)
    n = 0
    for e in doc.get('fighters', []):
        lb = e.pop('lua', None)
        if lb is None:
            continue
        w = {'module': rel}
        if 'state' in lb:
            w['state'] = lb['state']
        e['wasm'] = w
        for st in e.get('states', []):
            if 'lua' in st:
                st['wasm'] = st.pop('lua')
        n += 1
    if n == 0:
        sys.exit(f'{path}: no fighter carries a "lua" block')
    doc['geno'] = 12
    os.makedirs(os.path.join(folder, 'wasm'), exist_ok=True)
    shutil.copyfile(module, os.path.join(folder, rel))
    with open(path, 'w', encoding='utf8', newline='\n') as f:
        json.dump(doc, f, indent=2)
        f.write('\n')
    print(f'{path}: {n} fighter(s) converted to geno 12, module {rel}')


def main():
    a = sys.argv[1:]
    if len(a) < 2:
        sys.exit(__doc__)
    folder, module = a[0], a[1]
    if '--out' in a:
        out = a[a.index('--out') + 1]
        shutil.copytree(folder, out)
        folder = out
    convert(folder, module)


if __name__ == '__main__':
    main()
