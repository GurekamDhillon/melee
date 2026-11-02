#!/usr/bin/env python3
"""Wire entries of rust/melee_link/retail_unresolved.txt whose canonical tu/ item now exists into the generated
retail_ms.rs / retail.rs, and drop them from the list.

gen_retail.py / gen_retail_ms.py read transpiled initialisers (melee_fighter/src/c2r, */src/raw) that were deleted in
the consolidation, so they cannot be re-run; this patches their output in place with the same expressions they emit
(gen_retail.Gen.fn_expr / static_expr, sites numbered from 800000 up so they never collide). Idempotent.
Usage: wire_unresolved.py
"""
import os, re, sys
sys.path.insert(0, os.path.dirname(__file__))
import gen_link as g
import gen_retail as r

MS = os.path.join(g.RUST, 'melee_link', 'src', 'retail_ms.rs')
RT = os.path.join(g.RUST, 'melee_link', 'src', 'retail.rs')
OPT_CONST = {r'\[\[Option<&str>;\d+\];\d+\]': 'costume_strings_opt', r'\[Option<&str>;\d+\]': 'demo_strings_opt'}


def main():
    gen = r.Gen()
    ms = open(MS, encoding='utf8').read()
    rt = open(RT, encoding='utf8').read()
    used = [int(x) for x in re.findall(r'cast::<_, _, (\d+)>', ms + rt)]
    gen.site = max(8000, max(used) // 100 + 1)
    keep, n = [], 0
    for line in open(r.UNRES, encoding='utf8').read().splitlines():
        m = re.match(r'fn (\w+) \((\w+)\[(\d+)\](?:\.(\w+))?\): no canonical', line)
        c = re.match(r'const (\w+) \((\w+)\[(\d+)\]\): unsupported type (.+)', line)
        if m:
            name, table, idx, field = m.groups()
            gen.unres = []
            ex = gen.fn_expr(name, '*mut HSD_GObj', None, f'{table}[{idx}]')
            if ex is None:
                keep.append(line); continue
            if field:   # a MotionState row in retail_ms.rs
                hm = re.search(r'(?m)^        let t = &raw mut \w+::%s;\n' % table, ms)
                if not hm: keep.append(line); continue
                ms = ms[:hm.end()] + f'        (*t)[{idx}].{field} = {ex};\n' + ms[hm.end():]
            else:
                hm = re.search(r'(?m)^        let (t\d+) = &raw mut melee_fighter::tu::ftdata::%s;\n' % table, rt)
                if not hm: keep.append(line); continue
                rt = rt[:hm.end()] + f'        (*{hm.group(1)})[{idx}] = {ex};\n' + rt[hm.end():]
            n += 1
        elif c:
            name, table, idx, ty = c.groups()
            k = gen.consts.get(name, [])
            helper = next((h for pat, h in OPT_CONST.items() if re.fullmatch(pat, ty.replace(' ', ''))), None)
            hm = re.search(r'(?m)^        let (t\d+) = &raw mut melee_fighter::tu::ftdata::%s;\n' % table, rt)
            if len(k) != 1 or not helper or not hm:
                keep.append(line); continue
            rt = rt[:hm.end()] + f'        (*{hm.group(1)})[{idx}] = crate::strs::{helper}(&{k[0][0]});\n' + rt[hm.end():]
            n += 1
        else:
            keep.append(line)
    open(MS, 'w', encoding='utf8', newline='\n').write(ms)
    open(RT, 'w', encoding='utf8', newline='\n').write(rt)
    open(r.UNRES, 'w', encoding='utf8', newline='\n').write(''.join(l + '\n' for l in keep))
    print(f'wired {n}, {len(keep)} left')


if __name__ == '__main__':
    main()
