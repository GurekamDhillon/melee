#!/usr/bin/env python3
"""Writes the C driver and the Rust test of the it/kinds TU ports from specs_it.CFG (one description, so the
two halves cannot drift):

    python rust/melee_world/gen/mtu/mk_itkinds.py [stem ...]       (from the worktree root)

  rust/melee_world/gen/mtu/tu/<stem>.c     C driver (the game's .c included verbatim, shipping contraction)
  rust/melee_items/tests/tu_<stem>.rs      Rust test, draw for draw the same

Both build the same synthetic world per case (a seeded xorshift): the item gobj G with its Item, an owner gobj,
a reference gobj, a parent gobj, a gobj the spawn stub returns (with its own Item), a JObj chain, the article /
attribute / dynamic-bone records the Item points at, and the kind's item-var union member; then run one function
and hash the whole world (every field of every record, generated fill_/hash_ walkers) plus the return value and
the call trace of the stubs (gen_tu.py).
"""
import re
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
sys.path.insert(0, str(HERE))
import specs_it  # noqa: E402

CASES = 300
GOBJS = ["G", "OWN", "REFG", "PARG", "NEWG"]
ITEMS = ["ITEM", "NEWITEM", "REFITEM"]
JOBJS = ["JO[0]", "JO[1]", "JO[2]", "JO[3]", "JO[4]", "JO[5]", "JO[6]", "JO[7]", "OJO", "GJO", "LJO"]


def rfix(t):
    """C unsigned literal suffix -> Rust (`6u` -> `6u32`)"""
    return re.sub(r'(?<![\w.])(\d+)u\b', r'\1u32', t)


def arg(tok, i):
    """-> (C draw lines, Rust draw lines, C expr, Rust expr, kind)"""
    a = f"a{i}"
    if tok == "G":
        return [], [], "&G", "&raw mut G"
    if tok == "GN":
        return ([f"HSD_GObj* {a} = (rnd() % 8u == 0) ? 0 : &G;"],
                [f"let {a}: *mut HSD_GObj = if rnd() % 8u == 0 {{ null_mut() }} else {{ &raw mut G }};"], a, a)
    if tok == "REF":
        return [], [], "&REFG", "&raw mut REFG"
    if tok == "REFN":
        return ([f"HSD_GObj* {a} = (rnd() % 4u == 0) ? 0 : &REFG;"],
                [f"let {a}: *mut HSD_GObj = if rnd() % 4u == 0 {{ null_mut() }} else {{ &raw mut REFG }};"], a, a)
    if tok == "PAR":
        return [], [], "&PARG", "&raw mut PARG"
    if tok == "IT":
        return [], [], "&ITEM", "&raw mut ITEM"
    if tok == "JO":
        return [], [], "&JO[0]", "&raw mut JO[0]"
    if tok == "NULL":
        return [], [], "0", "null_mut()"
    if tok in ("VEC", "VEC1"):
        n = "VEC0" if tok == "VEC" else "VEC1"
        return ([f"fillvec(&{n});"], [f"fillvec(&raw mut {n});"], f"&{n}", f"&raw mut {n}")
    if tok == "f":
        return [f"float {a} = rw();"], [f"let {a} = rw();"], a, a + " as _"
    if tok == "i":
        return [f"int {a} = (int) fv();"], [f"let {a} = fv() as i32;"], a, a + " as _"
    if tok == "u":
        return [f"u32 {a} = fv();"], [f"let {a} = fv();"], a, a + " as _"
    if tok.startswith(("i:", "u:")):
        k, lo, hi = tok.split(":")
        lo, hi = int(lo), int(hi)
        n = hi - lo + 1
        if k == "i":
            return ([f"int {a} = ({lo}) + (int) (rnd() % {n}u);"], [f"let {a} = (({lo}) + (rnd() % {n}u) as i64) as i32;"], a, a + " as _")
        return ([f"u32 {a} = {lo}u + rnd() % {n}u;"], [f"let {a} = {lo}u32 + rnd() % {n}u;"], a, a + " as _")
    raise SystemExit(f"unknown argument token {tok}")


def op_case(op, stem):
    """-> (C lines, Rust lines) of one op's case body"""
    fn, toks, rk, fixes = op
    cl, rl = [], []
    for c, r in fixes:
        cl.append(c)
        rl.append(r)
    cargs, rargs = [], []
    for i, t in enumerate(toks.split()):
        c1, r1, ce, re_ = arg(t, i)
        cl += c1
        rl += r1
        cargs.append(ce)
        rargs.append(re_)
    ccall = f"{fn}({', '.join(cargs)})"
    # scalars are cast to the port's parameter types (`as _`); pointers pass as they are
    rcall = f"m::{fn}({', '.join(rargs)})"
    if rk == "v":
        cl.append(ccall + ";")
        rl.append(rcall + ";")
    elif rk == "i":
        cl.append(f"ret = (u32) {ccall};")
        rl.append(f"ret = {rcall} as u32;")
    elif rk == "f":
        cl.append(f"ret = B({ccall});")
        rl.append(f"ret = {rcall}.to_bits();")
    elif rk == "p":
        cl.append(f"ret = PI({ccall});")
        rl.append(f"ret = pi({rcall});")
    return cl, rl


def table_hash(name, kind, n):
    if kind == "state":
        c = (f"for (i = 0; i < {n}; i++) {{ HM((u32) {name}[i].anim_id); HM(FI((const void*) {name}[i].animated)); "
             f"HM(FI((const void*) {name}[i].physics_updated)); HM(FI((const void*) {name}[i].collided)); }}")
        r = (f"for i in 0..{n} {{ hm(m::{name}[i].anim_id as u32); hm(fi(m::{name}[i].animated)); "
             f"hm(fi(m::{name}[i].physics_updated)); hm(fi(m::{name}[i].collided)); }}")
    elif kind == "f32":
        c = f"for (i = 0; i < {n}; i++) HM(B({name}[i]));"
        r = f"for i in 0..{n} {{ hm(m::{name}[i].to_bits()); }}"
    else:
        c = f"for (i = 0; i < {n}; i++) HM((u32) {name}[i]);"
        r = f"for i in 0..{n} {{ hm(m::{name}[i] as u32); }}"
    return c, r


def build(stem):
    cfg = specs_it.CFG[stem]
    var = cfg["var"]
    attr = cfg["attr"]
    ops = cfg["ops"]
    nops = len(ops) + 1  # + the tables op
    tblop = len(ops)

    # ----------------------------------------------------------------------------- world setup
    cs, rs = [], []

    def both(c, r=None):
        cs.append(c)
        rs.append(c if r is None else r)

    for g in GOBJS:
        both(f"fill_HSD_GObj(&{g});", f"fill_HSD_GObj(&mut *(&raw mut {g}));")
    both("G.user_data = &ITEM; G.hsd_obj = &JO[0]; OWN.hsd_obj = &OJO; REFG.user_data = &REFITEM; REFG.hsd_obj = &OJO; "
         "PARG.hsd_obj = &OJO; NEWG.user_data = &NEWITEM; NEWG.hsd_obj = &OJO;",
         "G.user_data = &raw mut ITEM as *mut c_void; G.hsd_obj = &raw mut JO[0] as *mut c_void; "
         "OWN.hsd_obj = &raw mut OJO as *mut c_void; REFG.user_data = &raw mut REFITEM as *mut c_void; "
         "REFG.hsd_obj = &raw mut OJO as *mut c_void; PARG.hsd_obj = &raw mut OJO as *mut c_void; "
         "NEWG.user_data = &raw mut NEWITEM as *mut c_void; NEWG.hsd_obj = &raw mut OJO as *mut c_void;")
    for j in JOBJS:
        both(f"fill_HSD_JObj(&{j}); {j}.flags &= ~(u32) JOBJ_USE_QUATERNION;",
             f"fill_HSD_JObj(&mut *(&raw mut {j})); {j}.flags &= !(JOBJ_USE_QUATERNION as u32);")
    for k in range(4):
        both(f"if (rnd() % 6u != 0) JO[{k}].child = &JO[{k + 1}];",
             f"if rnd() % 6u != 0 {{ JO[{k}].child = &raw mut JO[{k + 1}]; }}")
    both("JO[4].next = &JO[5]; JO[5].next = &JO[6]; JO[6].next = &JO[7];",
         "JO[4].next = &raw mut JO[5]; JO[5].next = &raw mut JO[6]; JO[6].next = &raw mut JO[7];")
    both("for (i = 0; i < 100; i++) DBT.bones[i] = &JO[i % 8];",
         "for i in 0..100usize { DBT.bones[i] = &raw mut JO[i % 8]; }")
    for it in ITEMS:
        both(f"fill_Item(&{it});", f"fill_Item(&mut *(&raw mut {it}));")
        if var:
            both(f"fill_{var[1]}(&{it}.xDD4_itemVar.{var[0]});",
                 f"fill_{var[1]}(&mut {it}.xDD4_itemVar.{var[0]});")
        both(f"{it}.xC4_article_data = &ART; {it}.xCC_item_attr = &IATTR; {it}.xBBC_dynamicBoneTable = &DBT;",
             f"{it}.xC4_article_data = &raw mut ART; {it}.xCC_item_attr = &raw mut IATTR; "
             f"{it}.xBBC_dynamicBoneTable = &raw mut DBT;")
    both("ITEM.entity = &G; REFITEM.entity = &REFG; NEWITEM.entity = &NEWG;",
         "ITEM.entity = &raw mut G; REFITEM.entity = &raw mut REFG; NEWITEM.entity = &raw mut NEWG;")
    both("for (i = 0; i < 64; i++) ATTR[i] = rw();",
         "for i in 0..64usize { *attr_ptr().add(i) = rw(); }")
    if attr:
        both(f"fill_{attr}((void*) ATTR);", f"fill_{attr}(&mut *(attr_ptr() as *mut {attr}));")
    both("fill_ItemAttr(&IATTR); fill_ItemCommonData(&ICD);",
         "fill_ItemAttr(&mut *(&raw mut IATTR)); fill_ItemCommonData(&mut *(&raw mut ICD));")
    both("ART.x0_common_attr = &IATTR; ART.x4_specialAttributes = (union ItemSpecialAttributes*) ATTR; ART.x10_modelDesc = &MD;",
         "ART.x0_common_attr = &raw mut IATTR; ART.x4_specialAttributes = attr_ptr() as *mut c_void; "
         "ART.x10_modelDesc = &raw mut MD;")
    both("MD.x0_joint = (rnd() % 3u == 0) ? 0 : &JNT;",
         "MD.x0_joint = if rnd() % 3u == 0 { null_mut() } else { &raw mut JNT };")
    both("MD.x4_bone_count = fv(); MD.x8_bone_attach_id = (int) fv();",
         "MD.x4_bone_count = fv(); MD.x8_bone_attach_id = fv() as i32;")
    both("switch (rnd() % 4u) { case 0: ITEM.owner = 0; break; case 1: ITEM.owner = &OWN; break; default: ITEM.owner = &OWN; break; }",
         "match rnd() % 4u { 0 => ITEM.owner = null_mut(), 1 => ITEM.owner = &raw mut OWN, _ => ITEM.owner = &raw mut OWN }")
    for sym, (ce, re_) in cfg["data"].items():
        cs.append(f"{sym} = {ce};")
        rs.append(f"m::ext::{sym} = {re_};")
    for c, r in cfg["fix"]:
        cs.append(c)
        rs.append(r)
    setup_c = "\n".join("    " + x for x in cs)
    setup_r = rfix("\n".join("    " + x for x in rs))

    # ----------------------------------------------------------------------------- hash
    hc, hr = [], []
    for g in GOBJS:
        hc.append(f"hash_HSD_GObj(&{g});")
        hr.append(f"hash_HSD_GObj(&*(&raw const {g}));")
    for it in ITEMS:
        hc.append(f"hash_Item(&{it});")
        hr.append(f"hash_Item(&*(&raw const {it}));")
        if var:
            hc.append(f"hash_{var[1]}(&{it}.xDD4_itemVar.{var[0]});")
            hr.append(f"hash_{var[1]}(&{it}.xDD4_itemVar.{var[0]});")
    for j in JOBJS:
        hc.append(f"hash_HSD_JObj(&{j});")
        hr.append(f"hash_HSD_JObj(&*(&raw const {j}));")
    hc.append("HM(RH(ATTRD, sizeof ATTRD)); hash_ItemAttr(&IATTR);")
    hr.append("hm(rh(&*(&raw const ATTRD))); hash_ItemAttr(&*(&raw const IATTR));")
    for v in ("VEC0", "VEC1"):
        for a in "xyz":
            hc.append(f"HM(B({v}.{a}));")
            hr.append(f"hm({v}.{a}.to_bits());")
    hash_c = "\n".join("    " + x for x in hc)
    hash_r = "\n".join("    " + x for x in hr)

    # ----------------------------------------------------------------------------- ops
    cases_c, cases_r = [], []
    for n, op in enumerate(ops):
        cl, rl = op_case(op, stem)
        cases_c.append(f"            case {n}: {{\n" + "\n".join("                " + x for x in cl) + "\n                break;\n            }")
        cases_r.append(rfix(f"                {n} => {{\n" + "\n".join("                    " + x for x in rl) + "\n                }"))
    tc, tr = [], []
    for name, kind, n in cfg["tables"]:
        c, r = table_hash(name, kind, n)
        tc.append("                " + c)
        tr.append("                    " + r)
    cases_c.append(f"            case {tblop}: {{\n" + "\n".join(tc) + "\n                break;\n            }")
    cases_r.append(f"                {tblop} => {{\n" + "\n".join(tr) + "\n                }")

    # ----------------------------------------------------------------------------- C driver
    C = f'''/* TU: src/melee/it/kinds/{stem}.c (written by gen/mtu/mk_itkinds.py: do not edit) */
/* C oracle for melee_items src/tu/{stem}.rs: the whole TU included verbatim; every callee out of it is a
 * recording stub. Each line: op case ret state-hash trace-hash. */
#include TU_SRC
#include "mtu_common.h"
static HSD_GObj G, OWN, REFG, PARG, NEWG;
static Item ITEM, NEWITEM, REFITEM;
static HSD_JObj JO[8], OJO, GJO, LJO;
static Article ART;
static ItemModelDesc MD;
static ItemAttr IATTR;
static ItemCommonData ICD;
static HSD_Joint JNT;
static DynamicBoneTable DBT;
static double ATTRD[32];
#define ATTR ((float*) ATTRD)
static Vec3 VEC0, VEC1;
#include "{stem}_gen.h"

static void fillvec(Vec3* v)
{{
    v->x = rw();
    v->y = rw();
    v->z = rw();
}}

static void setup(void)
{{
    int i;
    zero(&G, sizeof G); zero(&OWN, sizeof OWN); zero(&REFG, sizeof REFG); zero(&PARG, sizeof PARG); zero(&NEWG, sizeof NEWG);
    zero(&ITEM, sizeof ITEM); zero(&NEWITEM, sizeof NEWITEM); zero(&REFITEM, sizeof REFITEM);
    zero(JO, sizeof JO); zero(&OJO, sizeof OJO); zero(&GJO, sizeof GJO); zero(&LJO, sizeof LJO);
    zero(&ART, sizeof ART); zero(&MD, sizeof MD); zero(&IATTR, sizeof IATTR); zero(&ICD, sizeof ICD);
    zero(&JNT, sizeof JNT); zero(&DBT, sizeof DBT); zero(ATTRD, sizeof ATTRD); zero(&VEC0, sizeof VEC0); zero(&VEC1, sizeof VEC1);
{setup_c}
}}

static void hash_all(void)
{{
{hash_c}
}}

int main(void)
{{
    int op, k, i;
    preg(&G, sizeof G); preg(&OWN, sizeof OWN); preg(&REFG, sizeof REFG); preg(&PARG, sizeof PARG); preg(&NEWG, sizeof NEWG);
    preg(&ITEM, sizeof ITEM); preg(&NEWITEM, sizeof NEWITEM); preg(&REFITEM, sizeof REFITEM);
    preg(JO, sizeof JO); preg(&OJO, sizeof OJO); preg(&GJO, sizeof GJO); preg(&LJO, sizeof LJO);
    preg(&ART, sizeof ART); preg(&MD, sizeof MD); preg(&IATTR, sizeof IATTR); preg(&ICD, sizeof ICD);
    preg(&JNT, sizeof JNT); preg(&DBT, sizeof DBT); preg(ATTRD, sizeof ATTRD); preg(&VEC0, sizeof VEC0); preg(&VEC1, sizeof VEC1);
    freg_all();
    for (op = 0; op < {nops}; op++) {{
        for (k = 0; k < (op == {tblop} ? 1 : {CASES}); k++) {{
            u32 ret = 0;
            tu_case(op, k);
            setup();
            switch (op) {{
{chr(10).join(cases_c)}
            }}
            hash_all();
            printf("%d %d %08X %08X %08X\\n", op, k, ret, H, TR);
            fflush(0);
        }}
    }}
    return 0;
}}
'''
    (HERE / "tu" / f"{stem}.c").write_text(C, encoding="utf-8", newline="\n")

    # ----------------------------------------------------------------------------- Rust test
    R = f'''//! melee_items src/tu/{stem}.rs vs the C (gen/mtu/tu/{stem}.c: the whole TU, shipping contraction).
//! Written by rust/melee_world/gen/mtu/mk_itkinds.py from the same description as the C driver;
//! regenerate the expected file with `python rust/melee_world/gen/mtu/run_tu.py {stem}`.
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables)]
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_items::tu::{stem} as m;
use melee_types::*;
use mtu_harness::*;
#[path = "mtu/{stem}_gen.rs"]
mod g;
use g::*;

static mut G: HSD_GObj = unsafe {{ core::mem::zeroed() }};
static mut OWN: HSD_GObj = unsafe {{ core::mem::zeroed() }};
static mut REFG: HSD_GObj = unsafe {{ core::mem::zeroed() }};
static mut PARG: HSD_GObj = unsafe {{ core::mem::zeroed() }};
static mut NEWG: HSD_GObj = unsafe {{ core::mem::zeroed() }};
static mut ITEM: Item = unsafe {{ core::mem::zeroed() }};
static mut NEWITEM: Item = unsafe {{ core::mem::zeroed() }};
static mut REFITEM: Item = unsafe {{ core::mem::zeroed() }};
static mut JO: [HSD_JObj; 8] = unsafe {{ core::mem::zeroed() }};
static mut OJO: HSD_JObj = unsafe {{ core::mem::zeroed() }};
static mut GJO: HSD_JObj = unsafe {{ core::mem::zeroed() }};
static mut LJO: HSD_JObj = unsafe {{ core::mem::zeroed() }};
static mut ART: Article = unsafe {{ core::mem::zeroed() }};
static mut MD: ItemModelDesc = unsafe {{ core::mem::zeroed() }};
static mut IATTR: ItemAttr = unsafe {{ core::mem::zeroed() }};
static mut ICD: ItemCommonData = unsafe {{ core::mem::zeroed() }};
static mut JNT: HSD_Joint = unsafe {{ core::mem::zeroed() }};
static mut DBT: DynamicBoneTable = unsafe {{ core::mem::zeroed() }};
static mut ATTRD: [f64; 32] = [0.0; 32];
static mut VEC0: Vec = unsafe {{ core::mem::zeroed() }};
static mut VEC1: Vec = unsafe {{ core::mem::zeroed() }};

unsafe fn attr_ptr() -> *mut f32 {{
    &raw mut ATTRD as *mut f32
}}

unsafe fn fillvec(v: *mut Vec) {{
    (*v).x = rw();
    (*v).y = rw();
    (*v).z = rw();
}}

unsafe fn setup() {{
    G = core::mem::zeroed(); OWN = core::mem::zeroed(); REFG = core::mem::zeroed(); PARG = core::mem::zeroed();
    NEWG = core::mem::zeroed(); ITEM = core::mem::zeroed(); NEWITEM = core::mem::zeroed(); REFITEM = core::mem::zeroed();
    JO = core::mem::zeroed(); OJO = core::mem::zeroed(); GJO = core::mem::zeroed(); LJO = core::mem::zeroed();
    ART = core::mem::zeroed(); MD = core::mem::zeroed(); IATTR = core::mem::zeroed(); ICD = core::mem::zeroed();
    JNT = core::mem::zeroed(); DBT = core::mem::zeroed(); ATTRD = [0.0; 32]; VEC0 = core::mem::zeroed();
    VEC1 = core::mem::zeroed();
{setup_r}
}}

unsafe fn hash_all() {{
{hash_r}
}}

#[test]
fn {stem}_matches_c() {{
    unsafe {{
        g::install();
        preg(&raw const G, core::mem::size_of::<HSD_GObj>());
        preg(&raw const OWN, core::mem::size_of::<HSD_GObj>());
        preg(&raw const REFG, core::mem::size_of::<HSD_GObj>());
        preg(&raw const PARG, core::mem::size_of::<HSD_GObj>());
        preg(&raw const NEWG, core::mem::size_of::<HSD_GObj>());
        preg(&raw const ITEM, core::mem::size_of::<Item>());
        preg(&raw const NEWITEM, core::mem::size_of::<Item>());
        preg(&raw const REFITEM, core::mem::size_of::<Item>());
        preg(&raw const JO, core::mem::size_of_val(&JO));
        preg(&raw const OJO, core::mem::size_of::<HSD_JObj>());
        preg(&raw const GJO, core::mem::size_of::<HSD_JObj>());
        preg(&raw const LJO, core::mem::size_of::<HSD_JObj>());
        preg(&raw const ART, core::mem::size_of::<Article>());
        preg(&raw const MD, core::mem::size_of::<ItemModelDesc>());
        preg(&raw const IATTR, core::mem::size_of::<ItemAttr>());
        preg(&raw const ICD, core::mem::size_of::<ItemCommonData>());
        preg(&raw const JNT, core::mem::size_of::<HSD_Joint>());
        preg(&raw const DBT, core::mem::size_of::<DynamicBoneTable>());
        preg(&raw const ATTRD, core::mem::size_of_val(&ATTRD));
        preg(&raw const VEC0, core::mem::size_of::<Vec>());
        preg(&raw const VEC1, core::mem::size_of::<Vec>());
        g::freg_all();
        let mut c = Check::new(include_str!("data/mtu_{stem}.txt"));
        let mut total = 0u32;
        for op in 0..{nops}u32 {{
            let ncase = if op == {tblop} {{ 1 }} else {{ {CASES} }};
            for k in 0..ncase {{
                case(op, k);
                setup();
                let mut ret = 0u32;
                match op {{
{chr(10).join(cases_r)}
                    _ => unreachable!(),
                }}
                hash_all();
                c.line(op, k, ret);
                total += 1;
            }}
        }}
        assert_eq!(c.done(), total);
    }}
}}
'''
    (ROOT / "rust/melee_items/tests" / f"tu_{stem}.rs").write_text(R, encoding="utf-8", newline="\n")
    print(f"mk_itkinds: {stem}: {len(ops)} ops + tables")


def main():
    stems = sys.argv[1:] or list(specs_it.CFG)
    for s in stems:
        build(s)


if __name__ == "__main__":
    main()
