#!/usr/bin/env python3
"""Writes both halves of the melee_items::tu item-kind state oracle from one description, so the C driver
(gen/kinds/tu/<stem>.c) and the Rust test (tests/kinds_<stem>.rs) cannot drift apart:

  python rust/melee_items/gen/kinds/mk_driver.py <stem> [...]      (worktree root)

World: three gobjs / items (GO[0] is the subject, GO[1] its owner / parent, GO[2] the item a spawn returns),
48 jobjs (3 roots, 3 children, a 14-bone dynamic bone table per item), articles + attributes (the
`x4_specialAttributes` block is a per-TU typed buffer), the common item data, and 4 Vec3. Every case draws
all of it from the seeded generator (the generated fill_<T>), links the pointers the functions follow and
applies the TU's tweaks (specs_cfg.py); each function of the TU (statics included, header inlines left out)
becomes one op whose arguments are drawn by parameter type. After the call the whole world, the return value
and the call trace of the recording stubs are hashed; the state-table arrays of the TU are hashed once.
Then `python rust/melee_items/gen/kinds/run_tu.py <stem>` (C reference output) and
`cargo test -p melee_items --test kinds_<stem>`.
"""
import re, sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
sys.path.insert(0, str(HERE))
import gen_tu  # noqa: E402
from gen_tu import ci, K, T, canon, Tu  # noqa: E402

N_CASES = 400
NJO = 48

C_WORLD_DECL = """static HSD_GObj GO[3];
static Item IT[3];
static HSD_JObj JO[48];
static DynamicBoneTable DBT[3];
static Article ART[3];
static ItemAttr ATTR[3];
static ItemCommonData ICD;
static unsigned char SPEC[3][128] __attribute__((aligned(16)));
static Vec3 VA[4];
"""

RS_WORLD_DECL = """#[repr(C, align(16))]
struct SpecBuf([[u8; 128]; 3]);
static mut GO: [HSD_GObj; 3] = unsafe { core::mem::zeroed() };
static mut IT: [Item; 3] = unsafe { core::mem::zeroed() };
static mut JO: [HSD_JObj; 48] = unsafe { core::mem::zeroed() };
static mut DBT: [DynamicBoneTable; 3] = unsafe { core::mem::zeroed() };
static mut ART: [Article; 3] = unsafe { core::mem::zeroed() };
static mut ATTR: [ItemAttr; 3] = unsafe { core::mem::zeroed() };
static mut ICD: ItemCommonData = unsafe { core::mem::zeroed() };
static mut SPECB: SpecBuf = SpecBuf([[0; 128]; 3]);
static mut VA: [Vec3; 4] = unsafe { core::mem::zeroed() };
"""

C_SETUP = """static void setup(int prof)
{
    int i, k;
    (void) i; (void) k;
    zero(GO, sizeof GO); zero(IT, sizeof IT); zero(JO, sizeof JO); zero(DBT, sizeof DBT); zero(ART, sizeof ART);
    zero(ATTR, sizeof ATTR); zero(&ICD, sizeof ICD); zero(SPEC, sizeof SPEC); zero(VA, sizeof VA);
    for (k = 0; k < 3; k++) { fill_HSD_GObj(&GO[k]); fill_Item(&IT[k]); fill_ItemAttr(&ATTR[k]); }
    for (i = 0; i < 48; i++) { fill_HSD_JObj(&JO[i]); JO[i].flags &= ~JOBJ_USE_QUATERNION; }
    fill_ItemCommonData(&ICD);
    for (i = 0; i < 4; i++) { VA[i].x = rw(); VA[i].y = rw(); VA[i].z = rw(); }
    for (k = 0; k < 3; k++) {
        GO[k].user_data = &IT[k];
        GO[k].hsd_obj = &JO[k];
        JO[k].child = &JO[3 + k];
        IT[k].entity = &GO[k];
        IT[k].xBBC_dynamicBoneTable = &DBT[k];
        IT[k].xC4_article_data = &ART[k];
        IT[k].xCC_item_attr = &ATTR[k];
        ART[k].x0_common_attr = &ATTR[k];
        ART[k].x4_specialAttributes = SPEC[k];
        for (i = 0; i < 14; i++) DBT[k].bones[i] = &JO[6 + 14 * k + i];
    }
@ICD_C@
    IT[0].owner = (rnd() & 3) ? &GO[1] : 0;
    IT[0].xC64_reflectGObj = (rnd() & 3) ? &GO[1] : 0;
    if (rnd() & 1) IT[0].msid = (int) (rnd() % 14);
    if (rnd() & 1) IT[0].facing_dir = (rnd() & 1) ? 1.0f : -1.0f;
    IT[0].ground_or_air = (rnd() & 1) ? GA_Air : GA_Ground;
@TWEAK@
}
"""

RS_SETUP = """unsafe fn setup(prof: i32) {
    SPECB = SpecBuf([[0; 128]; 3]);
    GO = core::mem::zeroed();
    IT = core::mem::zeroed();
    JO = core::mem::zeroed();
    DBT = core::mem::zeroed();
    ART = core::mem::zeroed();
    ATTR = core::mem::zeroed();
    ICD = core::mem::zeroed();
    VA = core::mem::zeroed();
    for k in 0..3 {
        g::fill_HSD_GObj(&mut GO[k]);
        g::fill_Item(&mut IT[k]);
        g::fill_ItemAttr(&mut ATTR[k]);
    }
    for i in 0..48 {
        g::fill_HSD_JObj(&mut JO[i]);
        JO[i].flags &= !(JOBJ_USE_QUATERNION as u32);
    }
    g::fill_ItemCommonData(&mut ICD);
    for i in 0..4 {
        VA[i].x = rw();
        VA[i].y = rw();
        VA[i].z = rw();
    }
    for k in 0..3 {
        GO[k].user_data = &raw mut IT[k] as *mut _;
        GO[k].hsd_obj = &raw mut JO[k] as *mut _;
        JO[k].child = &raw mut JO[3 + k];
        IT[k].entity = &raw mut GO[k];
        IT[k].xBBC_dynamicBoneTable = &raw mut DBT[k];
        IT[k].xC4_article_data = &raw mut ART[k];
        IT[k].xCC_item_attr = &raw mut ATTR[k];
        ART[k].x0_common_attr = &raw mut ATTR[k];
        ART[k].x4_specialAttributes = SPECB.0[k].as_mut_ptr() as *mut _;
        for i in 0..14 {
            DBT[k].bones[i] = &raw mut JO[6 + 14 * k + i];
        }
    }
@ICD_R@
    IT[0].owner = if rnd() & 3 != 0 { &raw mut GO[1] } else { null_mut() };
    IT[0].xC64_reflectGObj = if rnd() & 3 != 0 { &raw mut GO[1] } else { null_mut() };
    if rnd() & 1 != 0 {
        IT[0].msid = (rnd() % 14) as i32;
    }
    if rnd() & 1 != 0 {
        IT[0].facing_dir = if rnd() & 1 != 0 { 1.0 } else { -1.0 };
    }
    IT[0].ground_or_air = if rnd() & 1 != 0 { GA_Air } else { GA_Ground };
@TWEAK@
}
"""

C_HASH = """static void hash_all(void)
{
    int i, k;
    (void) i; (void) k;
@PTRS@
    for (k = 0; k < 3; k++) { hash_HSD_GObj(&GO[k]); hash_Item(&IT[k]); hash_ItemAttr(&ATTR[k]); }
    for (i = 0; i < 48; i++) hash_HSD_JObj(&JO[i]);
    for (i = 0; i < 4; i++) { HM(B(VA[i].x)); HM(B(VA[i].y)); HM(B(VA[i].z)); }
}
"""

RS_HASH = """unsafe fn hash_all() {
@PTRS@
    for k in 0..3 {
        g::hash_HSD_GObj(&GO[k]);
        g::hash_Item(&IT[k]);
        g::hash_ItemAttr(&ATTR[k]);
    }
    for i in 0..48 {
        g::hash_HSD_JObj(&JO[i]);
    }
    for i in 0..4 {
        hm(VA[i].x.to_bits());
        hm(VA[i].y.to_bits());
        hm(VA[i].z.to_bits());
    }
}
"""


def classify(t):
    """-> (kind, c type spelling) of a parameter / return type."""
    c = canon(t)
    k = c.kind
    if k == T.POINTER:
        nm = canon(c.get_pointee()).get_declaration().spelling
        return {"HSD_GObj": "gobj", "Item": "item", "HSD_JObj": "jobj", "Vec": "vec"}.get(nm, "ptr:" + nm)
    if k == T.VOID:
        return "void"
    if k == T.BOOL:
        return "bool"
    if k == T.FLOAT:
        return "f32"
    if k == T.DOUBLE:
        return "f64"
    if k == T.ENUM:
        return "int"
    if k in (T.INT, T.SHORT, T.CHAR_S, T.SCHAR):
        return "int"
    if k in (T.UINT, T.USHORT, T.UCHAR, T.CHAR_U):
        return "uint"
    return "other:" + str(k)


def is_inline(fn):
    toks = [x.spelling for x in fn.get_tokens()]
    head = toks[: toks.index("(")] if "(" in toks else toks
    return "inline" in head


def build_ops(tu, cfg):
    ops = []
    over = cfg.get("over", {})
    hints = cfg.get("hints", {})
    for name in sorted(tu.defs, key=lambda n: tu.defs[n].location.offset):
        fn = tu.defs[name]
        if name in cfg.get("skip_ops", ()):
            continue
        if is_inline(fn) and name not in over:
            continue
        if name in over:
            ops.append((name, over[name][0], over[name][1]))
            continue
        ft = canon(fn.type)
        params = list(fn.get_arguments())
        cnt = {}
        decl_c, decl_r, args = [], [], []
        for i, p in enumerate(params):
            kind = classify(p.type)
            n = cnt[kind] = cnt.get(kind, 0)
            cnt[kind] += 1
            pn = p.spelling
            if kind == "gobj":
                a_c, a_r = f"&GO[{n}]", f"&raw mut GO[{n}]"
            elif kind == "item":
                a_c, a_r = f"&IT[{n}]", f"&raw mut IT[{n}]"
            elif kind == "jobj":
                a_c, a_r = f"&JO[{3 + n}]", f"&raw mut JO[{3 + n}]"
            elif kind == "vec":
                a_c, a_r = f"&VA[{n}]", f"&raw mut VA[{n}]"
            elif kind in cfg.get("ptr_args", {}):
                total = sum(1 for q in params if classify(q.type) == kind)
                dc, dr, a_c, a_r = cfg["ptr_args"][kind](n, total, i)
                if dc:
                    decl_c.append(dc)
                    decl_r.append(dr)
            elif kind.startswith("ptr:") and kind[4:] in cfg.get("ptrargs", {}):
                a_c, a_r = [x.replace("{n}", str(n)) for x in cfg["ptrargs"][kind[4:]]]
