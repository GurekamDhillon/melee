#!/usr/bin/env python3
"""Round 5 generator (run from the worktree root): python rust/melee_items/tests/c_oracle/gen_round5.py

Scans src/melee/it for one-statement forwarders (`[return] f(gobj[, ref]);`) whose callee is
already ported (defined in this oracle's C), and writes
  - rust/melee_items/src/itfwd.rs                    the Rust forwarders
  - rust/melee_items/tests/round5/fwd_cases.rs       their Rust oracle cases
  - rust/melee_items/tests/c_oracle/round5_fwd.inc   their C oracle cases (T lines)
  - rust/melee_items/tests/c_oracle/round5_bodies.h  verbatim C bodies (forwarders + HAND list,
                                                     the hand ports of src/itkinds5.rs)
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
    if 'round5' in f:
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
    if f.name in ('itfwd.rs', 'itkinds5.rs'):
        continue
    for m in re.finditer(r'pub (unsafe )?fn (\w+)\(([^)]*)\)\s*(->\s*([\w:]+))?', f.read_text(encoding='utf-8')):
        rs[m.group(2)] = (bool(m.group(1)), m.group(3), m.group(5), f.stem)

REV = {'it_802E657C': 24}  # it_2E5A.x18
SKIP = {'itKyasarinegg_UnkMotion4_Anim'}  # `bool` function without a return (UB in C)
HAND = """itSeakNeedleThrown_Logic109_Destroyed it_8028ECE0 it_802B1DEC it_802B3544 it_802E5EF4
it_802ECC8C it_802ECC98 it_802ECD1C it_802F2014 itCapsule_UnkMotion3_Phys itGshell_UnkMotion4_Anim
itSamusGrapple_Logic53_Spawned itStarrod_UnkMotion1_Phys itZeldadinfireexplode_UnkMotion0_Phys
it_802A2418 it_802EAF28 itSamusBomb_Logic50_EvtUnk itTosakinto_UnkMotion2_Phys it_2E5A_UnkMotion1_Anim
it_8028428C it_8028EC98 it_80292EF8 it_802960B8 it_802960CC it_802E37A4 it_802F046C it_802F23AC
itBat_UnkMotion3_Phys itHammerhead_UnkMotion2_Phys itUnknown_UnkMotion0_Phys itZrshell_UnkMotion11_Phys
it_27CF_UnkMotion1_Phys it_8028D56C it_8028E774 it_802D43B0 it_802DFFA0 it_802ECCA4
itClimbersIce_Logic90_EvtUnk itPikachuThunder_Logic39_EvtUnk itSamusGrapple_Logic53_EvtUnk
itSeakChain_Logic54_EvtUnk it_2725_Logic52_EvtUnk it_802A7D40 it_8029FD84 itMewtwoDisable_Logic67_EvtUnk
itPeachTurnip_Logic56_EvtUnk it_802A23CC itSword_Logic12_DmgDealt itTomato_UnkMotion0_Phys
itFoods_UnkMotion0_Phys it_27CF_Logic114_DmgReceived it_8028ECF0 itTaru_UnkMotion7_Anim
itTarucann_UnkMotion2_Anim itWstar_UnkMotion5_Anim it_802B43B0 it_802AC5D8 it_802B3EFC it_802C3D44
it_802AB568 it_802B3368 it_802AAA50 itWhispyapple_UnkMotion0_Phys it_27CF_UnkMotion2_Anim it_80284FC4
it_80285024 it_802A8398 it_802B1FE8 fn_802C5E18 itFlipper_Inflight_Phys it_802852B8 it_802AB3F0
itThunder_UnkMotion2_Phys itSscopebeam_UnkMotion9_Anim it_802AB468 it_80285084 it_80285140 it_802851FC
it_802D48B0""".split()

fw = []
for name, (body, path, params) in sorted(allf.items(), key=lambda kv: (kv[1][1], kv[0])):
    if name in ported or name in SKIP or name in HAND:
        continue
    inner = body[body.index('{') + 1:body.rindex('}')].strip()
    m = re.fullmatch(r'(return\s+)?(\w+)\(\s*(\w+)\s*(?:,\s*(\w+)\s*)?\);', inner)
    if not m:
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
for f in fw:
    n = snake(f[0])
    if n in rs or n in names:
        sys.exit(f'name clash: {n}')
    names.add(n)


def rel(p):
    return Path(p).resolve().relative_to(ROOT).as_posix()


# ---- Rust forwarders
o = ["//! @generated by tests/c_oracle/gen_round5.py (port round 5): the one-statement forwarders",
     "//! of `src/melee/it/kinds/*.c` whose callee is already ported, e.g.",
     "//! `void itBat_Logic11_EvtUnk(Item_GObj* gobj, HSD_GObj* ref_gobj) { it_8026B894(gobj, ref_gobj); }`.",
     "//! Each is verified against its verbatim C copy (tests/c_oracle/round5_bodies.h,",
     "//! tests/oracle_round5.rs). A forwarder has no arithmetic of its own, so no fused sites.",
     "//! The GObj becomes the `Item`; a C `void` forwarder of a `bool` callee drops the result.",
     "#![allow(non_snake_case)]", "",
     "use crate::layout::{HSD_GObj, Item, ItemCommonData};"]
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
(CR / 'src/itfwd.rs').write_text('\n'.join(o), encoding='utf-8', newline='\n')

# ---- C bodies
b = ["/* @generated by gen_round5.py: verbatim copies of the decomp C bodies ported in round 5",
     " * (src/itfwd.rs, src/itkinds5.rs), grouped by source file. #included by round5.c. */"]
lastp = None
for name in [f[0] for f in fw] + HAND:
    body, path, _ = allf[name]
    if path != lastp:
        b.append(f"/* ---- {rel(path)} */")
        lastp = path
    b.append(body.strip())
    b.append('')
(OR / 'round5_bodies.h').write_text('\n'.join(b) + '\n', encoding='utf-8', newline='\n')

# ---- cases
ci = []
ri = ["// @generated by tests/c_oracle/gen_round5.py; included by tests/oracle_round5.rs.",
      f"cases! {{ fwd_match_c, {len(fw)};"]
for k, (name, path, callee, rn, two, ret) in enumerate(fw):
    tid = 700 + k
    T = f"R5_{name.upper()}"
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
ri.append('}')
(OR / 'round5_fwd.inc').write_text('\n'.join(ci) + '\n', encoding='utf-8', newline='\n')
(CR / 'tests/round5/fwd_cases.rs').write_text('\n'.join(ri) + '\n', encoding='utf-8', newline='\n')
print(f'{len(fw)} forwarders, {len(HAND)} hand bodies')
print(Counter(f[2] for f in fw))
