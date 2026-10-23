#!/usr/bin/env python3
"""Writes the two halves of a melee_items::tu state-oracle driver from one description, so the C
driver (gen/mtu_it2/tu/<stem>.c) and the Rust test (tests/tu_<stem>.rs) cannot drift apart:

  python rust/melee_items/gen/mtu_it2/mk_it_driver.py <stem> [...]      (worktree root)

The fixture is one item: gobj G (user_data = Item IT, hsd_obj = a 4-joint set JO[0] -> JO[1] -> JO[2]
with JO[3] a sibling of JO[1]), IT.xC4_article_data = ART whose x4_specialAttributes is the 1 KiB
buffer SA, IT.xCC_item_attr = IA, IT.xBBC_dynamicBoneTable = DBT, a second item G2/IT2 (what
`Item_80268B18` hands back), an owner gobj OWN, two Vec3 arguments V1/V2 and the common item data ICD
(`it_804D6D28`). Every state record is filled from the shared xorshift stream (generated fill_<T>:
bitfields and flags by value) and hashed field by field afterwards (generated hash_<T>), together
with the joints, the vectors and the special-attribute words; each op returns a value that is
printed / compared too. Callees out of the TU are the recording stubs of gen_tu.py.
"""
import os
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
sys.path.insert(0, str(HERE))
import gen_it  # noqa: E402
gen_tu = gen_it.gen_tu
ci, K, T = gen_tu.ci, gen_tu.K, gen_tu.T

N_CASES = 150

C_DECL = """static HSD_GObj G, G2, OWN;
static Item IT, IT2;
static Article ART;
static ItemAttr IA;
static unsigned int SA[256];
static HSD_JObj JO[4];
static DynamicBoneTable DBT;
static Vec3 V1, V2;
static ItemCommonData ICD;
ItemCommonData* it_804D6D28;
"""
RS_DECL = """static mut G: HSD_GObj = unsafe { core::mem::zeroed() };
static mut G2: HSD_GObj = unsafe { core::mem::zeroed() };
static mut OWN: HSD_GObj = unsafe { core::mem::zeroed() };
static mut IT: Item = unsafe { core::mem::zeroed() };
static mut IT2: Item = unsafe { core::mem::zeroed() };
static mut ART: Article = unsafe { core::mem::zeroed() };
static mut IA: ItemAttr = unsafe { core::mem::zeroed() };
static mut SA: [u32; 256] = [0; 256];
static mut JO: [HSD_JObj; 4] = unsafe { core::mem::zeroed() };
static mut DBT: DynamicBoneTable = unsafe { core::mem::zeroed() };
static mut V1: Vec = unsafe { core::mem::zeroed() };
static mut V2: Vec = unsafe { core::mem::zeroed() };
static mut ICD: ItemCommonData = unsafe { core::mem::zeroed() };
"""
PREG = [("&G", "sizeof G", "&raw const G", "core::mem::size_of::<HSD_GObj>()"),
        ("&G2", "sizeof G2", "&raw const G2", "core::mem::size_of::<HSD_GObj>()"),
        ("&OWN", "sizeof OWN", "&raw const OWN", "core::mem::size_of::<HSD_GObj>()"),
        ("&IT", "sizeof IT", "&raw const IT", "core::mem::size_of::<Item>()"),
        ("&IT2", "sizeof IT2", "&raw const IT2", "core::mem::size_of::<Item>()"),
        ("&ART", "sizeof ART", "&raw const ART", "core::mem::size_of::<Article>()"),
        ("&IA", "sizeof IA", "&raw const IA", "core::mem::size_of::<ItemAttr>()"),
        ("SA", "sizeof SA", "&raw const SA", "1024"),
        ("JO", "sizeof JO", "&raw const JO", "core::mem::size_of::<[HSD_JObj; 4]>()"),
        ("&DBT", "sizeof DBT", "&raw const DBT", "core::mem::size_of::<DynamicBoneTable>()"),
        ("&V1", "sizeof V1", "&raw const V1", "core::mem::size_of::<Vec>()"),
        ("&V2", "sizeof V2", "&raw const V2", "core::mem::size_of::<Vec>()"),
        ("&ICD", "sizeof ICD", "&raw const ICD", "core::mem::size_of::<ItemCommonData>()")]

C_HELP = """static void fill_jo(HSD_JObj* j)
{
    j->flags = rnd() & (JOBJ_MTX_INDEP_SRT | JOBJ_MTX_DIRTY | JOBJ_USER_DEF_MTX | JOBJ_HIDDEN);
    j->translate.x = rw(); j->translate.y = rw(); j->translate.z = rw();
    j->rotate.x = rw(); j->rotate.y = rw(); j->rotate.z = rw(); j->rotate.w = rw();
    j->scale.x = rw(); j->scale.y = rw(); j->scale.z = rw();
}
static void hash_jo(const HSD_JObj* j)
{
    HM(j->flags);
    HM(B(j->translate.x)); HM(B(j->translate.y)); HM(B(j->translate.z));
    HM(B(j->rotate.x)); HM(B(j->rotate.y)); HM(B(j->rotate.z)); HM(B(j->rotate.w));
    HM(B(j->scale.x)); HM(B(j->scale.y)); HM(B(j->scale.z));
    HM(PI(j->child)); HM(PI(j->next));
}
"""
RS_HELP = """unsafe fn fill_jo(j: &mut HSD_JObj) {
    j.flags = rnd() & ((JOBJ_MTX_INDEP_SRT | JOBJ_MTX_DIRTY | JOBJ_USER_DEF_MTX | JOBJ_HIDDEN) as u32);
    j.translate.x = rw(); j.translate.y = rw(); j.translate.z = rw();
    j.rotate.x = rw(); j.rotate.y = rw(); j.rotate.z = rw(); j.rotate.w = rw();
    j.scale.x = rw(); j.scale.y = rw(); j.scale.z = rw();
}
unsafe fn hash_jo(j: &HSD_JObj) {
    hm(j.flags);
    hm(j.translate.x.to_bits()); hm(j.translate.y.to_bits()); hm(j.translate.z.to_bits());
    hm(j.rotate.x.to_bits()); hm(j.rotate.y.to_bits()); hm(j.rotate.z.to_bits()); hm(j.rotate.w.to_bits());
    hm(j.scale.x.to_bits()); hm(j.scale.y.to_bits()); hm(j.scale.z.to_bits());
    hm(pi(j.child)); hm(pi(j.next));
}
"""
SETUP_C = """    int i;
    zero(&G, sizeof G); zero(&G2, sizeof G2); zero(&OWN, sizeof OWN); zero(&IT, sizeof IT); zero(&IT2, sizeof IT2);
    zero(&ART, sizeof ART); zero(&IA, sizeof IA); zero(SA, sizeof SA); zero(JO, sizeof JO); zero(&DBT, sizeof DBT);
    zero(&ICD, sizeof ICD);
    fill_HSD_GObj(&G); fill_HSD_GObj(&G2); fill_HSD_GObj(&OWN);
    fill_Item(&IT); fill_Item(&IT2); fill_ItemAttr(&IA); fill_ItemCommonData(&ICD);
    for (i = 0; i < 4; i++) fill_jo(&JO[i]);
    JO[0].child = &JO[1]; JO[1].child = &JO[2]; JO[1].next = &JO[3]; JO[0].next = 0;
    for (i = 0; i < 256; i++) SA[i] = (rnd() & 1) ? fv() : B(rw());
    V1.x = rw(); V1.y = rw(); V1.z = rw(); V2.x = rw(); V2.y = rw(); V2.z = rw();
    G.user_data = &IT; G.hsd_obj = &JO[0]; G2.user_data = &IT2; G2.hsd_obj = &JO[3]; OWN.user_data = 0; OWN.hsd_obj = 0;
    IT.entity = &G; IT2.entity = &G2;
    IT.xC4_article_data = &ART; IT2.xC4_article_data = &ART; ART.x4_specialAttributes = SA; ART.x0_common_attr = &IA;
    IT.xCC_item_attr = &IA; IT2.xCC_item_attr = &IA;
    IT.xBBC_dynamicBoneTable = &DBT; IT2.xBBC_dynamicBoneTable = &DBT;
    for (i = 0; i < 100; i++) DBT.bones[i] = &JO[i & 3];
    IT.owner = (rnd() & 1) ? &OWN : 0; IT2.owner = 0;
    it_804D6D28 = &ICD;
"""
SETUP_RS = """    let mut i: usize;
    G = core::mem::zeroed(); G2 = core::mem::zeroed(); OWN = core::mem::zeroed(); IT = core::mem::zeroed(); IT2 = core::mem::zeroed();
    ART = core::mem::zeroed(); IA = core::mem::zeroed(); SA = [0; 256]; JO = core::mem::zeroed(); DBT = core::mem::zeroed();
    ICD = core::mem::zeroed();
    g::fill_HSD_GObj(&mut G); g::fill_HSD_GObj(&mut G2); g::fill_HSD_GObj(&mut OWN);
    g::fill_Item(&mut IT); g::fill_Item(&mut IT2); g::fill_ItemAttr(&mut IA); g::fill_ItemCommonData(&mut ICD);
    for i in 0..4 { fill_jo(&mut JO[i]); }
    JO[0].child = &raw mut JO[1]; JO[1].child = &raw mut JO[2]; JO[1].next = &raw mut JO[3]; JO[0].next = null_mut();
    for i in 0..256 { SA[i] = if rnd() & 1 != 0 { fv() } else { rw().to_bits() }; }
    V1.x = rw(); V1.y = rw(); V1.z = rw(); V2.x = rw(); V2.y = rw(); V2.z = rw();
    G.user_data = &raw mut IT as *mut _; G.hsd_obj = &raw mut JO[0] as *mut _; G2.user_data = &raw mut IT2 as *mut _; G2.hsd_obj = &raw mut JO[3] as *mut _;
    OWN.user_data = null_mut(); OWN.hsd_obj = null_mut();
    IT.entity = &raw mut G; IT2.entity = &raw mut G2;
    IT.xC4_article_data = &raw mut ART; IT2.xC4_article_data = &raw mut ART; ART.x4_specialAttributes = (&raw mut SA) as *mut _; ART.x0_common_attr = &raw mut IA;
    IT.xCC_item_attr = &raw mut IA; IT2.xCC_item_attr = &raw mut IA;
    IT.xBBC_dynamicBoneTable = &raw mut DBT; IT2.xBBC_dynamicBoneTable = &raw mut DBT;
    for i in 0..100 { DBT.bones[i] = &raw mut JO[i & 3]; }
    IT.owner = if rnd() & 1 != 0 { &raw mut OWN } else { null_mut() }; IT2.owner = null_mut();
"""
HASH_C = """    int i;
    hash_HSD_GObj(&G); hash_HSD_GObj(&G2); hash_HSD_GObj(&OWN);
    hash_Item(&IT); hash_Item(&IT2); hash_ItemAttr(&IA);
    for (i = 0; i < 4; i++) hash_jo(&JO[i]);
    for (i = 0; i < 256; i++) HM(SA[i]);
    HM(B(V1.x)); HM(B(V1.y)); HM(B(V1.z)); HM(B(V2.x)); HM(B(V2.y)); HM(B(V2.z));
"""
HASH_RS = """    g::hash_HSD_GObj(&G); g::hash_HSD_GObj(&G2); g::hash_HSD_GObj(&OWN);
    g::hash_Item(&IT); g::hash_Item(&IT2); g::hash_ItemAttr(&IA);
    for i in 0..4 { hash_jo(&JO[i]); }
    for i in 0..256 { hm(SA[i]); }
    hm(V1.x.to_bits()); hm(V1.y.to_bits()); hm(V1.z.to_bits()); hm(V2.x.to_bits()); hm(V2.y.to_bits()); hm(V2.z.to_bits());
"""


SA_ALIGN_DECL = """#[repr(C, align(16))]
struct SaBuf([u32; 256]);
impl core::ops::Deref for SaBuf { type Target = [u32; 256]; fn deref(&self) -> &[u32; 256] { &self.0 } }
impl core::ops::DerefMut for SaBuf { fn deref_mut(&mut self) -> &mut [u32; 256] { &mut self.0 } }
static mut SA: SaBuf = SaBuf([0; 256]);"""


def hot(path, vals, kind="i"):
    """(C, Rust) setup statements: with probability 3/4 force `path` (a field expression valid in
    both languages) to one of `vals` (literals valid in both; kind "f" appends the C `f` suffix)."""
    n = len(vals)
    cv = [(v + "f") if kind == "f" and "." in v else v for v in vals]
    expr = cv[-1]
    for i in range(n - 2, -1, -1):
        expr = f"(q == {i} ? {cv[i]} : {expr})"
    c = f"    if (rnd() % 4 != 0) {{ u32 q = rnd() % {n}; {path} = {expr}; }}\n"
    arms = " ".join(f"{i} => {vals[i]}," for i in range(n - 1))
    r = f"    if rnd() % 4 != 0 {{ let q = rnd() % {n}; {path} = (match q {{ {arms} _ => {vals[-1]} }}) as _; }}\n"
    return c, r


def rs_ty(t):
    return gen_tu.rust_ty(t)


def param_expr(i, ptype, name, cfg, seen):
    """(C expr, Rust expr) for parameter `name` of type ptype."""
    c = gen_tu.canon(ptype)
    sp = ptype.spelling
    k = c.kind
    if k == T.POINTER:
        pn = c.get_pointee().spelling.replace("struct ", "").replace("const ", "").strip()
        if "HSD_GObj" in pn or "Item_GObj" in pn:
            return ("&G", "&raw mut G") if not seen["gobj"] else ("&G2", "&raw mut G2")
        if pn in ("Vec3", "Vec"):
            return ("&V1", "&raw mut V1")
        if "HSD_JObj" in pn:
            return ("&JO[1]", "&raw mut JO[1]")
        if pn == "Item":
            return ("&IT", "&raw mut IT")
        pp = cfg.get("ptr_params", {})
        if pn in pp:
            opts = pp[pn]
            n = seen.get("pp:" + pn, 0)
            seen["pp:" + pn] = n + 1
            return opts[min(n, len(opts) - 1)]
        raise SystemExit(f"param {name} of type {sp}: give an override")
    pp = cfg.get("ptr_params", {})
    if sp in pp:
        return pp[sp][0]
    if k == T.FLOAT:
        return ("rw()", "rw()")
    if k in gen_tu.INTS or k == T.ENUM:
        return ("(" + sp + ") fv()", "fv() as _")
    raise SystemExit(f"param {name} of type {sp}: give an override")


def auto_op(fn, cfg):
    """(name, C stmt, Rust stmt) calling fn with fixture arguments."""
    seen = {"gobj": False}
    cargs, rargs = [], []
    for i, p in enumerate(fn.get_arguments()):
        ce, re_ = param_expr(i, p.type, p.spelling, cfg, seen)
        if ce in ("&G", "&G2"):
            seen["gobj"] = True
        cargs.append(ce)
        rargs.append(re_)
    rt = fn.result_type
    rk = gen_tu.canon(rt).kind
    call_c = f"{fn.spelling}({', '.join(cargs)})"
    call_r = f"m::{fn.spelling}({', '.join(rargs)})"
    if rk == T.VOID:
        return (fn.spelling, call_c + ";", call_r + ";")
    if rk == T.POINTER:
        return (fn.spelling, f"ret = PI({call_c});", f"ret = pi({call_r});")
    if rk == T.FLOAT:
        return (fn.spelling, f"ret = B({call_c});", f"ret = {call_r}.to_bits();")
    return (fn.spelling, f"ret = (u32) {call_c};", f"ret = {call_r} as u32;")


def emit(stem, cfg):
    from specs_it import SPECS
    spec = SPECS[stem]
    tu = gen_tu.Tu(gen_tu.patched_source(stem, spec), spec.get("args", ()), (ROOT / spec["src"]).parent)
    ops = []
    skip = set(cfg.get("skip", ()))
    ov = cfg.get("ops", {})
    for name, fn in tu.defs.items():
        if name in skip:
            continue
        if name in ov:
            c, r = ov[name]
            ops.append((name, c, r))
        else:
            ops.append(auto_op(fn, cfg))
    ops += list(cfg.get("extra_ops", ()))
    for tbl, n in cfg.get("tables", ()):
        c = (f"{{ int i; for (i = 0; i < {n}; i++) {{ HM((u32) {tbl}[i].anim_id); HM(FI((const void*) {tbl}[i].animated)); "
             f"HM(FI((const void*) {tbl}[i].physics_updated)); HM(FI((const void*) {tbl}[i].collided)); }} }}")
        r = (f"{{ for i in 0..{n} {{ hm(m::{tbl}[i].anim_id as u32); hm(fi(m::{tbl}[i].animated)); "
             f"hm(fi(m::{tbl}[i].physics_updated)); hm(fi(m::{tbl}[i].collided)); }} }}")
        ops.append((f"table_{tbl}", c, r))
    nops = len(ops)
    n = cfg.get("n_cases", N_CASES)
    setup_c = SETUP_C + cfg.get("setup_c", "")
    rs_decl_base = RS_DECL
    setup_rs_base = SETUP_RS
    if cfg.get("sa_align"):
        rs_decl_base = RS_DECL.replace("static mut SA: [u32; 256] = [0; 256];", SA_ALIGN_DECL)
        setup_rs_base = SETUP_RS.replace("SA = [0; 256];", "SA = SaBuf([0; 256]);")
    setup_rs = setup_rs_base + ("    m::ext::it_804D6D28 = &raw mut ICD;\n" if cfg.get("icd") else "") + cfg.get("setup_rs", "")
    hash_c = HASH_C + cfg.get("hash_c", "")
    hash_rs = HASH_RS + cfg.get("hash_rs", "")
    c = [f"/* TU: {spec['src']} */",
         "/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a",
         " * recording stub. Written by gen/mtu_it2/mk_it_driver.py. Each line: op case ret state-hash trace-hash. */",
         "#include TU_SRC", '#include "mtu_common.h"', C_DECL, cfg.get("c_decl", ""), f'#include "{stem}_gen.h"', "", C_HELP,
         cfg.get("c_extra", ""), "", "static void setup(void)", "{", setup_c, "}", "",
         "static void pre_hash(void)", "{", cfg.get("pre_hash_c", ""), "}", "",
         "static void hash_all(void)", "{", hash_c, "}", "",
         "int main(void)", "{", "    int op, k;"]
    for ca, cs, _, _ in PREG + cfg.get("pregs", []):
        c.append(f"    preg({ca}, {cs});")
    c.append("    freg_all();")
    for fname in tu.defs:
        if fname in cfg.get("nofreg", ()):
            continue
        c.append(f"    freg((const void*) {fname});")
    c += [f"    for (op = 0; op < {nops}; op++) {{", f"        for (k = 0; k < {n}; k++) {{", "            u32 ret = 0;",
          "            tu_case(op, k);", ("            printf(\"<%d %d>\", op, k); fflush(0);" if os.environ.get("MTU_TRACE_OPS") else ""), "            setup();", "            switch (op) {"]
    for i, (_, cs, _) in enumerate(ops):
        c.append(f"            case {i}: {{ {cs} }} break;")
    c += ["            }", "            pre_hash();", "            hash_all();", '            printf("%d %d %08X %08X %08X\\n", op, k, ret, H, TR);',
          "        }", "    }", "    return 0;", "}", ""]
    (HERE / "tu" / f"{stem}.c").write_text("\n".join(c), encoding="utf-8", newline="\n")

    r = ["//! Written by gen/mtu_it2/mk_it_driver.py from the same description as gen/mtu_it2/tu/" + stem + ".c.",
         f"//! src/tu/{stem}.rs vs the C (the whole TU, shipping contraction; regenerate the expected file",
         f"//! with `python rust/melee_items/gen/mtu_it2/run_it.py {stem}`).",
         "#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]",
         "#[path = \"mtu_harness_it/mod.rs\"]", "mod mtu_harness;", "use core::ffi::c_void;", "use core::ptr::null_mut;", "use melee_types::*;", f"use melee_items::tu::{stem} as m;",
         "use mtu_harness::*;", f'#[path = "mtu/{stem}_gen.rs"]', "mod g;", "", rs_decl_base, cfg.get("rs_decl", ""), RS_HELP,
         cfg.get("rs_extra", ""), "",
         "unsafe fn setup() {", setup_rs, "}", "", "unsafe fn pre_hash() {", cfg.get("pre_hash_rs", ""), "}", "",
         "unsafe fn hash_all() {", hash_rs, "}", "",
         "#[test]", f"fn {stem}_matches_c() {{", "    unsafe {", "        g::install();"]
    for _, _, ra, rs in PREG + cfg.get("pregs", []):
        r.append(f"        preg({ra}, {rs});")
    r.append("        g::freg_all();")
    for fname in tu.defs:
        if fname in cfg.get("nofreg", ()):
            continue
        r.append(f"        freg(m::{fname} as usize);")
    r += [f'        let mut c = Check::new(include_str!("data/mtu_{stem}.txt"));', f"        for op in 0..{nops}u32 {{",
          f"            for k in 0..{n}u32 {{", "                case(op, k);", "                setup();",
          "                let mut ret = 0u32;", "                match op {"]
    for i, (_, _, rs) in enumerate(ops):
        r.append(f"                    {i} => {{ {rs} }}")
    r += ["                    _ => unreachable!(),", "                }", "                pre_hash();", "                hash_all();", "                c.line(op, k, ret);",
          "            }", "        }", f"        assert_eq!(c.done(), {nops * n});", "    }", "}", ""]
    (ROOT / "rust/melee_items/tests" / f"tu_{stem}.rs").write_text("\n".join(r), encoding="utf-8", newline="\n")
    print(f"mk_it_driver: {stem}: {nops} ops x {n} cases")
    return [o[0] for o in ops]


if __name__ == "__main__":
    from cfg_it import CFG
    for s in sys.argv[1:]:
        emit(s, CFG[s])
