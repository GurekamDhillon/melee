#!/usr/bin/env python3
"""Round 6 generator (run from the worktree root): python rust/melee_items/tests/c_oracle/gen_round6.py

Like gen_round5.py, with three changes: callees ported in round 5 count as ported (second-order
forwarders); a forwarded argument may carry a pointer cast (`(HSD_GObj*) arg1`); and bodies that
are only `return true;` / `return 1;` / `return;` (state-table callbacks) are generated too.

Scans src/melee/it for one-statement forwarders (`[return] f(gobj[, ref]);`) whose callee is
already ported (defined in this oracle's C), and writes
  - rust/melee_items/src/itfwd6.rs                   the Rust forwarders
  - rust/melee_items/tests/round6/fwd_cases.rs       their Rust oracle cases
  - rust/melee_items/tests/c_oracle/round6_fwd.inc   their C oracle cases (T lines)
  - rust/melee_items/tests/c_oracle/round6_bodies.h  verbatim C bodies (forwarders + HAND list,
                                                     the hand ports of src/itkinds6.rs)
Never edit the outputs by hand."""
import glob, re, sys
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
CR = ROOT / "rust/melee_items"
OR = CR / "tests/c_oracle"
defre = re.compile(r'^(?:static\s+|inline\s+)*[A-Za-z_][\w\s\*]*?\b([A-Za-z_]\w*)\s*\(([^;{}]*)\)\s*\n?\{', re.M)
NOTFN = {'if', 'while', 'for', 'switch', 'return', 'sizeof'}


def funcs(path):
    s = open(path, encoding='utf-8', errors='replace').read()
    out = {}
    for m in defre.finditer(s):
        name = m.group(1)
        if name in NOTFN:
            continue
        i = m.end() - 1
        d = 0
        j = i
        while j < len(s):
            if s[j] == '{':
                d += 1
            elif s[j] == '}':
                d -= 1
                if d == 0:
                    break
            j += 1
        out[name] = (m.group(0) + s[i + 1:j + 1], path, m.group(2))
    return out


ported = set()
for f in sorted(glob.glob(str(OR / '*.c')) + glob.glob(str(OR / '*.h'))):
    if 'round6' in f:
        continue
    ported |= set(funcs(f))
allf = {}
for p in sorted(glob.glob(str(ROOT / 'src/melee/it/**/*.c'), recursive=True)):
    for k, v in funcs(p).items():
        allf.setdefault(k, v)


def snake(n):
    return re.sub(r'(?<=[a-z])(?=[A-Z])', '_', n).lower()


# Rust signatures of everything already in the crate
rs = {}
for f in sorted((CR / 'src').glob('*.rs')):
    if f.name in ('itfwd6.rs', 'itkinds6.rs'):
        continue
    for m in re.finditer(r'pub (unsafe )?fn (\w+)\(([^)]*)\)\s*(->\s*([\w:]+))?', f.read_text(encoding='utf-8')):
        rs[m.group(2)] = (bool(m.group(1)), m.group(3), m.group(5), f.stem)

REV = {'it_802E657C': 24}  # it_2E5A.x18
SKIP = {'itKyasarinegg_UnkMotion4_Anim', 'checkHitLag'}  # `bool` function without a return (UB in C)
HAND = """it_8027574C it_802762B0 it_8026F3D4_check_kind it_2725_Logic70_EvtUnk it_2725_Logic59_EvtUnk
itCLinkMilk_Destroy itNessbat_RemoveItem it_802DB358 it_802AE608 itFoxBlaster_SetShotOffset
itFoxBlaster_SetCommandVars itFoxBlaster_ClearShot itNesspkthunderball_ShiftAngles
itNesspkthunderball_ShiftPositions itDrMarioPill_UnkMotion0_Phys itTogepy_UnkMotion6_Phys
itTools_UnkMotion4_Phys itHassam_UnkMotion1_Phys it_8028EDBC it_802874F0 it_802962E0
it_2725_Logic101_Reflected itMaril_UnkMotion1_Coll_inline it_80291DAC_level it_802944AC inlineA1
inlineA2 inlineC1 inlineA3 inlineD1 it_802DDBE8 itZrshell_UnkMotion3_Anim it_802DFF14 it_802EA988
it_802A43B8 it_802A43EC it_802A4420 test_comp it_link_attr_math samus_grapple_fighter_compare""".split()

fw = []
triv = []
for name, (body, path, params) in sorted(allf.items(), key=lambda kv: (kv[1][1], kv[0])):
    if name in ported or name in SKIP or name in HAND:
        continue
    inner = body[body.index('{') + 1:body.rindex('}')].strip()
    inner = re.sub(r'\s+', ' ', re.sub(r'//[^\n]*', '', inner)).strip()
    cast = r'(?:\(\s*(?:HSD_GObj|Item_GObj)\s*\*\s*\)\s*)?'
    m = re.fullmatch(r'(return\s+)?(\w+)\(\s*' + cast + r'(\w+)\s*(?:,\s*' + cast + r'(\w+)\s*)?\);', inner)
    if not m:
        k = {'return true;': 'true', 'return 1;': 'true', 'return;': 'void'}.get(inner)
        ps = [p.strip() for p in params.split(',')]
        head = body[:body.index(name)]
        gp = r'\b(?:Item_GObj|HSD_GObj)\s*\*'
        if k and len(ps) in (1, 2) and re.search(gp, ps[0]) and \
                (len(ps) == 1 or re.search(gp, ps[1])) and \
                (k == 'void') == bool(re.search(r'\bvoid\b', head)) and \
                (k == 'void' or re.search(r'\bbool\b', head)) and 'static' not in head:
            triv.append((name, path, len(ps) == 2, k))
        continue
    callee = m.group(2)
    if callee not in ported or callee == 'GET_ITEM':
        continue
    ps = [p.strip() for p in params.split(',')]
    pnames = [re.findall(r'\w+', p)[-1] for p in ps]
    if m.group(3) != pnames[0]:
        continue
    if m.group(4) is not None and (len(pnames) != 2 or m.group(4) != pnames[1]):
        continue
    if m.group(4) is None and len(pnames) != 1:
        continue
    head = body[:body.index(name)]
    ret = 'bool' if re.search(r'\bbool\b', head) else ('void' if re.search(r'\bvoid\b', head) else None)
    if ret is None or (ret == 'bool' and not m.group(1)):
        continue
    rn = snake(callee)
    if rn not in rs:
        sys.exit(f'no Rust port named {rn} for {callee}')
    fw.append((name, path, callee, rn, m.group(4) is not None, ret))

names = set()
for f in fw + triv:
    n = snake(f[0])
    if n in rs or n in names:
        sys.exit(f'name clash: {n}')
    names.add(n)


def rel(p):
    return Path(p).resolve().relative_to(ROOT).as_posix()


# ---- Rust forwarders
o = ["//! @generated by tests/c_oracle/gen_round6.py (port round 6): the one-statement forwarders",
     "//! of `src/melee/it/**/*.c` whose callee is already ported (now including round 5's), e.g.",
     "//! `void itBat_Logic11_EvtUnk(Item_GObj* gobj, HSD_GObj* ref_gobj) { it_8026B894(gobj, ref_gobj); }`.",
     "//! Each is verified against its verbatim C copy (tests/c_oracle/round6_bodies.h,",
     "//! tests/oracle_round6.rs). A forwarder has no arithmetic of its own, so no fused sites.",
     "//! The GObj becomes the `Item`; a C `void` forwarder of a `bool` callee drops the result.",
     "#![allow(non_snake_case)]", "",
     "use crate::layout::{HSD_GObj, Item" + (", ItemCommonData" if any('ItemCommonData' in rs[f[3]][1] for f in fw) else "") + "};"]
for md in sorted({rs[f[3]][3] for f in fw}):
    o.append(f"use crate::{md}::{{{', '.join(sorted({f[3] for f in fw if rs[f[3]][3] == md}))}}};")
o.append('')
for (name, path, callee, rn, two, ret) in fw:
    unsafe, rparams, rret, _ = rs[rn]
    common = 'ItemCommonData' in rparams
    args = ['ip'] + (['ref_gobj'] if two else []) + (['common'] if common else [])
    sig = ['ip: &mut Item'] + (['ref_gobj: *mut HSD_GObj'] if two else []) + \
        (['common: &ItemCommonData'] if common else [])
    call = f"{rn}({', '.join(args)})"
    o.append(f"/// `{name}` ({rel(path)}): `{'return ' if ret == 'bool' else ''}{callee}(..)`.")
    if unsafe:
        o.append("/// # Safety")
        o.append(f"/// As `{rn}`.")
    rt = ' -> bool' if ret == 'bool' else ''
    o.append(f"pub {'unsafe ' if unsafe else ''}fn {snake(name)}({', '.join(sig)}){rt} {{")
    o.append(f"    {call if ret == 'bool' else call + ';'}")
    o.append("}")
    o.append('')
o.append("// ---- bodies that are only `return true;` / `return 1;` / `return;` (state-table callbacks)")
o.append('')
for (name, path, two, k) in triv:
    o.append(f"/// `{name}` ({rel(path)}): `{'return true;' if k == 'true' else 'return;'}`.")
    sig = ['_ip: &mut Item'] + (['_ref_gobj: *mut HSD_GObj'] if two else [])
    if k == 'true':
        o += [f"pub fn {snake(name)}({', '.join(sig)}) -> bool {{", '    true', '}']
    else:
        o.append(f"pub fn {snake(name)}({', '.join(sig)}) {{}}")
    o.append('')
(CR / 'src/itfwd6.rs').write_text('\n'.join(o), encoding='utf-8', newline='\n')

# ---- C bodies
b = ["/* @generated by gen_round6.py: verbatim copies of the decomp C bodies ported in round 6",
     " * (src/itfwd6.rs, src/itkinds6.rs), grouped by source file. #included by round6.c. */"]
lastp = None
for name in [f[0] for f in fw] + [t[0] for t in triv] + HAND:
    body, path, _ = allf[name]
    if path != lastp:
        b.append(f"/* ---- {rel(path)} */")
        lastp = path
    b.append(body.strip())
    b.append('')
(OR / 'round6_bodies.h').write_text('\n'.join(b) + '\n', encoding='utf-8', newline='\n')

# ---- cases
ci = []
ri = ["// @generated by tests/c_oracle/gen_round6.py; included by tests/oracle_round6.rs.",
      f"cases! {{ fwd_match_c, {len(fw) + len(triv)};"]
for k, (name, path, callee, rn, two, ret) in enumerate(fw):
    tid = 1300 + k
    T = f"R6_{name.upper()}"
    ccall = f"{name}(g{', PTR(R())' if two else ''})"
    cs = f"ret = (u32) {ccall};" if ret == 'bool' else f"{ccall};"
    ci.append(f'    T({tid}, "{T}", {{ r4_pre(ip); {cs} ret = H(ret, r4_h(ip)); }});')
    common = 'ItemCommonData' in rs[rn][1]
    rargs = ['ip'] + (['r'] if two else []) + (['&x.common'] if common else [])
    pre = 'let r = g.ptr(); ' if two else ''
    # item-var bitfield bytes the callee reads: x86 C order <-> GC order (see oracle_round4.rs)
    rev = REV.get(callee)
    if rev is not None:
        pre += f'r4_rev(ip, {rev}); '
    rcall = f"{snake(name)}({', '.join(rargs)})"
    rb = f"{rcall} as u32" if ret == 'bool' else f"{rcall}; 0"
    post = f'r4_rev(ip, {rev}); ' if rev is not None else ''
    ri.append(f"    {tid} {T} |ip, g, ov, x| {{ r4_pre(ip, g, x); {pre}let ret = {{ {rb} }}; {post}hmix(ret, r4_h(ip)) }};")
for k, (name, path, two, kk) in enumerate(triv):
    tid = 1500 + k
    T = f"R6T_{name.upper()}"
    ccall = f"{name}(g{', PTR(R())' if two else ''})"
    cs = f"ret = (u32) {ccall};" if kk == 'true' else f"{ccall};"
    ci.append(f'    T({tid}, "{T}", {{ r4_pre(ip); {cs} ret = H(ret, r4_h(ip)); }});')
    pre = 'let r = g.ptr(); ' if two else ''
    rcall = f"{snake(name)}({', '.join(['ip'] + (['r'] if two else []))})"
    rb = f"{rcall} as u32" if kk == 'true' else f"{rcall}; 0"
    ri.append(f"    {tid} {T} |ip, g, ov, x| {{ r4_pre(ip, g, x); {pre}let ret = {{ {rb} }}; hmix(ret, r4_h(ip)) }};")
ri.append('}')
(OR / 'round6_fwd.inc').write_text('\n'.join(ci) + '\n', encoding='utf-8', newline='\n')
(CR / 'tests/round6/fwd_cases.rs').write_text('\n'.join(ri) + '\n', encoding='utf-8', newline='\n')
print(f'{len(fw)} forwarders, {len(triv)} trivial, {len(HAND)} hand bodies')
print(Counter(f[2] for f in fw))
