#!/usr/bin/env python3
"""State-oracle generator for the it/kinds TU ports in rust/melee_items/src/tu/ (shard 705-712).

    python rust/melee_world/gen/mtu/mk_itk.py gen <stem> [...]    (from the worktree root)
    python rust/melee_world/gen/mtu/mk_itk.py run <stem> [...]
    python rust/melee_world/gen/mtu/mk_itk.py all <stem> [...]    (gen + run)

gen: writes gen/mtu/tu/<stem>.c (C driver; the game's .c included verbatim), gen/mtu/tu/<stem>_gen.h
and rust/melee_items/tests/mtu/<stem>_gen.rs (gen_tu.py: recording stubs for the callees, field-by-field
fill/hash of the state records) and rust/melee_items/tests/tu_<stem>.rs (the Rust half, draw for draw).
It also refreshes the generated `ext` block of rust/melee_items/src/tu/<stem>.rs.
run: compiles the driver natively with the shipping float contraction (+ gwtool's lowering), links
gekko_fp.c, MSL/trigf.c, lb/lbtrigf.c and lb/lbvector.c (the math the ports take from melee_runtime), runs
it and writes rust/melee_items/tests/data/mtu_<stem>.txt.

The state a case starts from is generated (Item + the spawned Item + JObjs + attribute records, all by
fill_<T>); the hash after the call covers every field of those records (hash_<T>). Item bitfield
unions (UnkFlagStruct, flag32) are compared BY VALUE (melee_types keeps bitfields in GameCube bit
order, native C does not); the item-var union is walked as the one variant the TU uses.
"""
import os, re, subprocess, sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
sys.path.insert(0, str(HERE))
import gen_tu  # noqa: E402
from gen_tu import K, T, canon  # noqa: E402

CASES = 400
MATH_SKIP = ["__frsqrte", "sinf", "cosf", "atan2f", "lbVector_Normalize", "lbVector_Diff", "lbVector_Angle",
             "lbVector_Mirror"]
LINK = ["pc/gameworld/gekko_fp.c", "src/melee/lb/lbtrigf.c", "src/melee/lb/lbvector.c"]
SPAWN_EFFECT_C = (
    "{ T(PI(a0->x0_parent_gobj)); T(PI(a0->x4_parent_gobj2)); T((u32) a0->kind); T((u32) a0->hold_kind); T((u32) a0->x10); "
    "T(B(a0->pos.x)); T(B(a0->pos.y)); T(B(a0->pos.z)); T(B(a0->prev_pos.x)); T(B(a0->prev_pos.y)); T(B(a0->prev_pos.z)); "
    "T(B(a0->vel.x)); T(B(a0->vel.y)); T(B(a0->vel.z)); T(B(a0->facing_dir)); T((u32) (u16) a0->x3C_damage); T((u32) (u16) a0->x3E); "
    "T((u32) a0->x40); T(FLG(a0->x44_flag)); T(FLG(a0->x45_flag)); T(FLG(a0->x46_flag)); T(FLG(a0->x47_flag)); T((u32) a0->x48_ground_or_air); }")
SPAWN_EFFECT_R = (
    "{ t(pi((*a0).x0_parent_gobj)); t(pi((*a0).x4_parent_gobj2)); t((*a0).kind as u32); t((*a0).hold_kind as u32); t((*a0).x10 as u32); "
    "t((*a0).pos.x.to_bits()); t((*a0).pos.y.to_bits()); t((*a0).pos.z.to_bits()); t((*a0).prev_pos.x.to_bits()); t((*a0).prev_pos.y.to_bits()); t((*a0).prev_pos.z.to_bits()); "
    "t((*a0).vel.x.to_bits()); t((*a0).vel.y.to_bits()); t((*a0).vel.z.to_bits()); t((*a0).facing_dir.to_bits()); t((*a0).x3C_damage as u16 as u32); t((*a0).x3E as u16 as u32); "
    "t((*a0).x40 as u32); t(super::flg(&(*a0).x44_flag)); t(super::flg(&(*a0).x45_flag)); t(super::flg(&(*a0).x46_flag)); t(super::flg(&(*a0).x47_flag)); t((*a0).x48_ground_or_air as u32); }")


def vec_out(n):
    """effect: aN is a Vec3* the callee writes (three draws)."""
    return (f"if (a{n}) {{ a{n}->x = rg(); a{n}->y = rg(); a{n}->z = rg(); }}",
            f"if !a{n}.is_null() {{ (*a{n}).x = rg(); (*a{n}).y = rg(); (*a{n}).z = rg(); }}")


def both(*effs):
    return ("{ " + " ".join(e[0] for e in effs) + " }", "{ " + " ".join(e[1] for e in effs) + " }")


EFSYNC = dict(
    c="void* efSync_Spawn(s32 a0, HSD_GObj* a1, ...) { __builtin_va_list ap; T(@ID@); T((u32) a0); T(PI(a1)); __builtin_va_start(ap, a1); "
      "switch (a0) { case 0x473: { void* j = __builtin_va_arg(ap, void*); T(PI(j)); break; } "
      "case 0x44E: case 0x44F: case 0x450: case 0x451: { void* j = __builtin_va_arg(ap, void*); float* f = __builtin_va_arg(ap, float*); float* s = __builtin_va_arg(ap, float*); T(PI(j)); T(PI(f)); T(B(*f)); T(PI(s)); T(B(*s)); break; } "
      "case 0x46C: { void* j = __builtin_va_arg(ap, void*); float* f = __builtin_va_arg(ap, float*); T(PI(j)); T(PI(f)); T(B(*f)); break; } "
      "case 0x405: { Vec3* p = __builtin_va_arg(ap, Vec3*); T(B(p->x)); T(B(p->y)); T(B(p->z)); break; } "
      "default: T(0xDEAD); } __builtin_va_end(ap); return 0; }",
    ext="fn efSync_Spawn(a0: i32, a1: *mut HSD_GObj, a2: &[*const c_void]) -> *mut c_void;",
    hook="|a0, a1, a2| { t(@ID@); t(a0 as u32); t(pi(a1)); match a0 { 0x473 => { t(pi(a2[0])); } "
         "0x44E..=0x451 => { t(pi(a2[0])); t(pi(a2[1])); t(*(a2[1] as *const f32) .to_bits()); t(pi(a2[2])); t(*(a2[2] as *const f32).to_bits()); } "
         "0x46C => { t(pi(a2[0])); t(pi(a2[1])); t((*(a2[1] as *const f32)).to_bits()); } "
         "0x405 => { let p = a2[0] as *const Vec; t((*p).x.to_bits()); t((*p).y.to_bits()); t((*p).z.to_bits()); } "
         "_ => { t(0xDEAD); } } core::ptr::null_mut() }")
EFSYNC["hook"] = EFSYNC["hook"].replace("t(*(a2[1] as *const f32) .to_bits());", "t((*(a2[1] as *const f32)).to_bits());").replace(
    "t(*(a2[2] as *const f32).to_bits());", "t((*(a2[2] as *const f32)).to_bits());")

# effects shared by the TUs
EFFECTS = {
    "Item_80268B18": (SPAWN_EFFECT_C, SPAWN_EFFECT_R),
    "it_8026BB88": vec_out(1),
    "it_8026BB68": vec_out(1),
    "ftLib_80086644": vec_out(1),
    "lb_8000B1CC": both(vec_out(1), vec_out(2)),
    "mpGetSpeed": vec_out(2),
    "mpCheckAllRemap": (
        "{ if (a0) { a0->x = rg(); a0->y = rg(); a0->z = rg(); } if (a1) *a1 = (int) fv(); }",
        "{ if !a0.is_null() { (*a0).x = rg(); (*a0).y = rg(); (*a0).z = rg(); } if !a1.is_null() { *a1 = fv() as i32; } }"),
}

# ---------------------------------------------------------------------------------------- per TU
TUS = {}
TUS["ithinoarashi"] = dict(
    attr="itHinoarashiAttributes", var=("hinoarashi", "itHinoarashi_ItemVars"),
    spawn="rand", pun=["it_802D61C8", "it_802D6310"],
)
TUS["ithitodeman"] = dict(
    attr="itHitodemanAttributes", var=("hitodeman", "itHitodeman_ItemVars"), spawn="rand",
    ret={"ftLib_80086198": ("((fv() & 1) ? &REFG : 0)", "if fv() & 1 != 0 { &raw mut super::REFG } else { core::ptr::null_mut() }")},
    setup_c="    ITEM.xDD4_itemVar.hitodeman.x90 = (rnd() & 1) ? &OWNG : 0;",
    setup_r="    ITEM.xDD4_itemVar.hitodeman.x90 = if rnd() & 1 != 0 { &raw mut OWNG } else { null_mut() };",
)
TUS["ithouou"] = dict(
    attr="itHououAttr", var=("houou", "itHouou_ItemVars"), spawn="always",
    # the "fell 10 below the start" threshold: draw pos.y around it
    pre={"itHouou_UnkMotion4_Phys": ("ITEM.pos.y = ITEM.xDD4_itemVar.houou.start_pos.y - 10.0f + rg() * 2.0f;",
                                     "ITEM.pos.y = ITEM.xDD4_itemVar.houou.start_pos.y - 10.0 + rg() * 2.0;")})
TUS["itkabigon"] = dict(attr="itKabigonAttributes", var=("kabigon", "itKabigon_ItemVars"), spawn=None, attr2="itPokemonAttributes")
TUS["itkamex"] = dict(attr="itKamexAttributes", var=("pokemon", "itPokemon_ItemVars"), spawn="rand")
TUS["itkinoko"] = dict(attr="KinokoAttrs", var=("kinoko", "itKinoko_ItemVars"), spawn=None)
TUS["itkirbycutterbeam"] = dict(
    attr="itKirbyCutterBeamAttributes", var=("kirbycutterbeam", "itKirbyCutterBeam_ItemVars"), spawn="rand",
    # the hide-the-beam threshold is exactly 5.0: draw it
    pre={"itKirbycutterbeam_UnkMotion0_Phys": ("ITEM.xD44_lifeTimer = (rnd() % 3 == 0) ? 5.0f : rw();",
                                               "ITEM.xD44_lifeTimer = if rnd() % 3 == 0 { 5.0 } else { rw() };")})
TUS["itkirbygamewatchchefpan"] = dict(attr=None, var=None, spawn="rand")


class Sub:
    """A record declaration that exposes only some of its children (for per-child walking)."""
    def __init__(self, d, kids):
        self.d, self.kids = d, kids

    def get_children(self):
        return iter(self.kids)

    def __getattr__(self, n):
        return getattr(self.d, n)


BaseWalk = gen_tu.Walk


class MyWalk(BaseWalk):
    cfg = None  # set per build

    def gen(self, name):
        cdef, rdef = super().gen(name)
        a, b = rdef.split("\n}\npub fn hash_")
        a = a.replace("{\n", "{ unsafe {\n", 1) + "\n}}\n"
        b = "pub fn hash_" + b
        b = b.replace("{\n", "{ unsafe {\n", 1)
        b = b.rstrip()
        assert b.endswith("}")
        b = b[:-1] + "}}\n"
        return cdef, a + b

    def one(self, t, f, cexpr, rexpr, cf, ch, rf, rh, depth):
        if t.spelling == "UnkFlagStruct":
            for i in range(8):
                cf.append(f"{cexpr}.b{i} = rnd() & 1;")
                rf.append(f"{rexpr}.anon_0.set_b{i}((rnd() & 1) as u8);")
                ch.append(f"HM((u32) {cexpr}.b{i});")
                rh.append(f"hm({rexpr}.anon_0.b{i}() as u32);")
            return
        return super().one(t, f, cexpr, rexpr, cf, ch, rf, rh, depth)

    def fields(self, decl, cp, rp, base_off, cf, ch, rf, rh, rootc, depth):
        if not isinstance(decl, Sub) and decl.spelling == "Item" and cp == "p->":
            for f in list(decl.get_children()):
                if f.kind == K.FIELD_DECL and f.spelling == "xDD4_itemVar":
                    self.variant(cf, ch, rf, rh, depth)
                    continue
                if f.kind in (K.STRUCT_DECL, K.UNION_DECL) and f.is_anonymous_record_decl():
                    mem = [c for c in f.get_children() if c.kind == K.FIELD_DECL]
                    rmap = {"xDAA_flag": "anon_daa", "xDBC_itcmd_var4": "anon_dbc"}
                    if mem and mem[0].spelling in rmap:
                        m = mem[0]
                        self.one(m.type, m, f"p->{m.spelling}", f"p.{rmap[m.spelling]}.{m.spelling}", cf, ch, rf, rh, depth)
                        continue
                BaseWalk.fields(self, Sub(decl, [f]), cp, rp, base_off, cf, ch, rf, rh, rootc, depth)
            return
        BaseWalk.fields(self, decl, cp, rp, base_off, cf, ch, rf, rh, rootc, depth)

    def variant(self, cf, ch, rf, rh, depth):
        var = self.cfg["var"]
        if var is None:
            return
        vn, vt = var
        vdecl = self.rec_of(vt)
        cp, rp = f"p->xDD4_itemVar.{vn}.", f"p.xDD4_itemVar.{vn}."
        if vn != "hinoarashi":
            BaseWalk.fields(self, vdecl, cp, rp, 0, cf, ch, rf, rh, "p", depth)
            return
        # hinoarashi: bytes +0x60..+0x6C are also read / written as a Vec3 / f32 (the flame's normal);
        # in the "pun" view they are compared as raw bytes, otherwise as flags + x61 + x64 + x68 by value
        pun = ([], [], [], [])
        rest = ([], [], [], [])
        for ch_ in vdecl.get_children():
            if ch_.kind != K.FIELD_DECL:
                continue
            tgt = pun if ch_.spelling in ("x60", "x61", "x64", "x68") else rest
            n = ch_.spelling
            self.one(ch_.type, ch_, cp + n, rp + n, cf, tgt[1], rf, tgt[3], depth)
        ch.extend(rest[1])
        rh.extend(rest[3])
        ch.append("if (VIEWPUN) { int q; for (q = 0; q < 12; q++) HM(((const unsigned char*) &p->xDD4_itemVar)[0x60 + q]); } else {")
        ch.extend(pun[1])
        ch.append("}")
        rh.append("if super::VIEWPUN { for q in 0..12 { hm(*(&p.xDD4_itemVar as *const _ as *const u8).add(0x60 + q) as u32); } } else {")
        rh.extend(pun[3])
        rh.append("}")


def cty(tu, name):
    return ("struct " + name) if name in tu.tagged else name


def sig_info(c):
    """-> (ret_kind, [param kinds]) for a defined function cursor."""
    rt = c.result_type
    rc = canon(rt)
    if rc.kind == T.VOID:
        rk = "void"
    elif rc.kind == T.POINTER:
        rk = "ptr"
    elif rc.kind == T.FLOAT:
        rk = "f32"
    else:
        rk = "int"
    ps = []
    gobjs = 0
    for a in c.get_arguments():
        s = a.type.spelling
        cc = canon(a.type)
        if "GObj" in s and cc.kind == T.POINTER:
            ps.append(("gobj", gobjs, a.spelling))
            gobjs += 1
        elif s.startswith("Vec3"):
            ps.append(("vec", 0, a.spelling))
        elif cc.kind == T.POINTER:
            ps.append(("fptr", 0, a.spelling))
        elif cc.kind == T.FLOAT:
            ps.append(("f32", 0, a.spelling))
        else:
            ps.append(("int", 0, a.spelling))
    return rk, ps


def build_spec(stem, cfg):
    src = f"src/melee/it/kinds/{stem}.c"
    types = ["Item", "HSD_JObj", "ItemAttr"] + ([cfg["attr"]] if cfg.get("attr") else [])
    if cfg.get("attr2"):
        types.append(cfg["attr2"])
    ret = {"HSD_Randi": gen_tu_common_ret()}
    if cfg["spawn"] == "rand":
        ret["Item_80268B18"] = ("((fv() & 1) ? &SG : 0)", "if fv() & 1 != 0 { &raw mut super::SG } else { core::ptr::null_mut() }")
    elif cfg["spawn"] == "always":
        ret["Item_80268B18"] = ("&SG", "&raw mut super::SG")
    ret.update(cfg.get("ret", {}))
    return dict(crate="melee_items", src=src, types=types, skip_data=["ip"],
                skip_stubs=MATH_SKIP, ret=ret, effects=dict(EFFECTS), custom_stubs={"efSync_Spawn": EFSYNC},
                null_ret_ok=["Item_80268B18", "ftLib_80086198"])


def gen_tu_common_ret():
    from specs import COMMON_RET
    return COMMON_RET["HSD_Randi"]


def functions(stem):
    tu = gen_tu.Tu(f"src/melee/it/kinds/{stem}.c", (), "src/melee/it/kinds")
    return tu, [(n, c) for n, c in tu.defs.items() if c.storage_class != gen_tu.ci.StorageClass.STATIC
                or n.startswith(("itHinoarashi_UnkMotion2_Anim",))]


# ------------------------------------------------------------------------------------- C driver

def pre_args(params, rets=None):
    """-> (c_pre, c_args, r_pre, r_args)"""
    cp, ca, rp, ra = [], [], [], []
    for j, (kind, idx, nm) in enumerate(params):
        if kind == "gobj":
            ca.append("&G" if idx == 0 else "&REFG")
            ra.append("&raw mut G" if idx == 0 else "&raw mut REFG")
        elif kind == "vec":
            cp.append("VP.x = rg(); VP.y = rg(); VP.z = rg();")
            rp.append("VP.x = rg(); VP.y = rg(); VP.z = rg();")
            ca.append("&VP")
            ra.append("&raw mut VP")
        elif kind == "fptr":
            cp.append("FP1 = rg();")
            rp.append("FP1 = rg();")
            ca.append("&FP1")
            ra.append("&raw mut FP1")
        elif kind == "f32":
            cp.append(f"float a{j} = rw();")
            rp.append(f"let a{j}: f32 = rw();")
            ca.append(f"a{j}")
            ra.append(f"a{j}")
        else:
            cp.append(f"s32 a{j} = (s32) fv();")
            rp.append(f"let a{j} = fv() as i32;")
            ca.append(f"a{j}")
            ra.append(f"a{j} as _")
    return cp, ca, rp, ra


KINOKO_DECL_C = ("static KinokoAnim KANIM[8];\nextern Article* it_804D6D24[];\n")


def c_driver(stem, cfg, tu, funcs):
    attr = cfg.get("attr")
    atc = cty(tu, attr) if attr else None
    d = []
    d.append(f"/* TU: src/melee/it/kinds/{stem}.c (written by gen/mtu/mk_itk.py: do not edit) */")
    d.append("#include TU_SRC\n#include \"mtu_common.h\"")
    d.append("static int VIEWPUN;")
    d.append(f"#define FLG(f) ((u32) ((f).b0 | (f).b1 << 1 | (f).b2 << 2 | (f).b3 << 3 | (f).b4 << 4 | (f).b5 << 5 | (f).b6 << 6 | (f).b7 << 7))")
    d.append("static HSD_GObj G, REFG, OWNG, SG;")
    d.append(f"static {cty(tu,'Item')} ITEM, SITEM;")
    d.append("static HSD_JObj JT, JC, SJT, BONES[24];")
    d.append("static DynamicBoneTable DBT, SDBT;")
    d.append("static Article ART, SART;")
    if atc:
        d.append(f"static {atc} ATTR, SATTR;")
    elif stem == "itkirbygamewatchchefpan":
        d.append("static void* ATTRP[2]; static void* SATTRP[2];")
    d.append(f"static {cty(tu,'ItemAttr')} IATTR, SIATTR;")
    d.append("static Vec3 VP;\nstatic float FP1;")
    if stem == "itkinoko":
        d.append("static KinokoAnim KANIM[8];\nstatic Article* KART[0x9F];\nstatic HSD_AnimJoint AJ[8];")
    d.append(f"#include \"{stem}_gen.h\"")
    if stem == "itkinoko":
        # it_804D6D24 is the stub header's data definition (Article* [..]); point it at KART
        pass
    # setup
    d.append("static void setup(void)\n{\n    int i;")
    d.append("    zero(&G, sizeof G); zero(&REFG, sizeof REFG); zero(&OWNG, sizeof OWNG); zero(&SG, sizeof SG);")
    d.append("    zero(&ITEM, sizeof ITEM); zero(&SITEM, sizeof SITEM); zero(&JT, sizeof JT); zero(&JC, sizeof JC); zero(&SJT, sizeof SJT);")
    d.append("    zero(BONES, sizeof BONES); zero(&DBT, sizeof DBT); zero(&SDBT, sizeof SDBT); zero(&ART, sizeof ART); zero(&SART, sizeof SART);")
    d.append("    zero(&IATTR, sizeof IATTR); zero(&SIATTR, sizeof SIATTR); VIEWPUN = 0;")
    if atc:
        d.append("    zero(&ATTR, sizeof ATTR); zero(&SATTR, sizeof SATTR);")
    d.append("    fill_Item(&ITEM); fill_Item(&SITEM);")
    d.append("    fill_HSD_JObj(&JT); fill_HSD_JObj(&JC); fill_HSD_JObj(&SJT);")
    d.append("    fill_ItemAttr(&IATTR); fill_ItemAttr(&SIATTR);")
    if atc:
        d.append(f"    fill_{attr}(&ATTR); fill_{attr}(&SATTR);")
    d.append("    JT.flags &= ~0x20000u; JC.flags &= ~0x20000u; SJT.flags &= ~0x20000u;")
    d.append("    G.user_data = &ITEM; G.hsd_obj = &JT; SG.user_data = &SITEM; SG.hsd_obj = &SJT;")
    d.append("    JT.child = (rnd() & 1) ? &JC : 0;")
    d.append("    ITEM.xC4_article_data = &ART; SITEM.xC4_article_data = &SART;")
    if atc:
        d.append("    ART.x4_specialAttributes = &ATTR; SART.x4_specialAttributes = &SATTR;")
    d.append("    ITEM.xCC_item_attr = &IATTR; SITEM.xCC_item_attr = &SIATTR;")
    d.append("    ITEM.xBBC_dynamicBoneTable = &DBT; SITEM.xBBC_dynamicBoneTable = &SDBT;")
    d.append("    for (i = 0; i < 100; i++) { DBT.bones[i] = &BONES[i % 24]; SDBT.bones[i] = &BONES[(i + 5) % 24]; }")
    d.append("    ITEM.owner = (rnd() & 1) ? &OWNG : 0; SITEM.owner = (rnd() & 1) ? &OWNG : 0;")
    if stem == "itkirbygamewatchchefpan":
        d.append("    ART.x4_specialAttributes = ATTRP; SART.x4_specialAttributes = SATTRP;")
        d.append("    SATTRP[0] = (rnd() & 1) ? (void*) &SATTRP[1] : 0; SATTRP[1] = 0;")
    if cfg.get("setup_c"):
        d.append(cfg["setup_c"])
    if stem == "itkinoko":
        d.append("    for (i = 0; i < 8; i++) { KANIM[i].joint = &AJ[i]; }")
        d.append("    zero(KART, sizeof KART); KART[It_Kind_Kinoko] = &SART; SART.x4_specialAttributes = KANIM; it_804D6D24 = KART;")
        d.append("    ART.x4_specialAttributes = &ATTR;")
    d.append("}\n")
    d.append("static void hash_all(void)\n{")
    d.append("    hash_Item(&ITEM); hash_Item(&SITEM); hash_HSD_JObj(&JT); hash_HSD_JObj(&JC); hash_HSD_JObj(&SJT);")
    d.append("    hash_ItemAttr(&IATTR);")
    if atc:
        d.append(f"    hash_{attr}(&ATTR); hash_{attr}(&SATTR);")
    d.append("    HM(B(VP.x)); HM(B(VP.y)); HM(B(VP.z)); HM(B(FP1));")
    d.append("}\n")
    d.append("int main(void)\n{\n    int op, k;")
    d.append("    preg(&G, sizeof G); preg(&REFG, sizeof REFG); preg(&OWNG, sizeof OWNG); preg(&SG, sizeof SG);")
    d.append("    preg(&ITEM, sizeof ITEM); preg(&SITEM, sizeof SITEM); preg(&JT, sizeof JT); preg(&JC, sizeof JC); preg(&SJT, sizeof SJT);")
    d.append("    preg(BONES, sizeof BONES); preg(&DBT, sizeof DBT); preg(&SDBT, sizeof SDBT); preg(&ART, sizeof ART); preg(&SART, sizeof SART);")
    d.append("    preg(&IATTR, sizeof IATTR); preg(&SIATTR, sizeof SIATTR); preg(&VP, sizeof VP); preg(&FP1, sizeof FP1);")
    if atc:
        d.append("    preg(&ATTR, sizeof ATTR); preg(&SATTR, sizeof SATTR);")
    elif stem == "itkirbygamewatchchefpan":
        d.append("    preg(ATTRP, sizeof ATTRP); preg(SATTRP, sizeof SATTRP);")
    if stem == "itkinoko":
        d.append("    preg(KANIM, sizeof KANIM); preg(AJ, sizeof AJ);")
    d.append("    freg_all();")
    d.append(f"    for (op = 0; op < {len(funcs)}; op++) {{\n        for (k = 0; k < {CASES}; k++) {{\n            u32 ret = 0;")
    d.append("            tu_case(op, k);\n            setup();\n            switch (op) {")
    for i, (n, c) in enumerate(funcs):
        rk, ps = sig_info(c)
        cp, ca, rp, ra = pre_args(ps)
        pun = n in cfg.get("pun", [])
        body = []
        if n in cfg.get("pre", {}):
            body.append(cfg["pre"][n][0])
        if pun:
            body.append("VIEWPUN = 1; { float* q = (float*) ((char*) &ITEM.xDD4_itemVar + 0x60); q[0] = rg(); q[1] = rg(); q[2] = rg(); q = (float*) ((char*) &SITEM.xDD4_itemVar + 0x60); q[0] = rg(); q[1] = rg(); q[2] = rg(); }")
        body += cp
        if n == "it_80293660":
            call = f"it_80293660((int) (fv() % 6))"
        else:
            call = f"{n}({', '.join(ca)})"
        if rk == "void":
            body.append(call + ";")
        elif rk == "f32":
            body.append(f"ret = B({call});")
        elif rk == "ptr":
            body.append(f"ret = PI({call});")
        else:
            body.append(f"ret = (u32) {call};")
        d.append(f"            case {i}: {{ " + " ".join(body) + " break; }")
    d.append("            }\n            hash_all();")
    d.append('            printf("%d %d %08X %08X %08X%c", op, k, ret, H, TR, 10);\n        }\n    }\n    return 0;\n}')
    return "\n".join(d) + "\n"


# ------------------------------------------------------------------------------------- Rust test

def r_test(stem, cfg, funcs):
    attr = cfg.get("attr")
    a = lambda s: s
    d = []
    d.append(f"//! melee_items src/tu/{stem}.rs vs the C (gen/mtu/tu/{stem}.c: the whole TU, shipping")
    d.append("//! contraction). Written by rust/melee_world/gen/mtu/mk_itk.py from the same description as the C driver;")
    d.append(f"//! regenerate the expected file with `python rust/melee_world/gen/mtu/mk_itk.py run {stem}`.")
    d.append("#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables)]")
    d.append("mod mtu_harness;\nuse core::ffi::c_void;\nuse core::ptr::null_mut;")
    d.append(f"use melee_items::tu::{stem} as m;\nuse melee_types::*;\nuse mtu_harness::*;")
    d.append(f"#[path = \"mtu/{stem}_gen.rs\"]\nmod g;")
    d.append("pub static mut VIEWPUN: bool = false;")
    d.append("fn flg(f: &UnkFlagStruct) -> u32 { unsafe { (f.anon_0.b0() as u32) | (f.anon_0.b1() as u32) << 1 | (f.anon_0.b2() as u32) << 2 | (f.anon_0.b3() as u32) << 3 | (f.anon_0.b4() as u32) << 4 | (f.anon_0.b5() as u32) << 5 | (f.anon_0.b6() as u32) << 6 | (f.anon_0.b7() as u32) << 7 } }")
    z = "unsafe { core::mem::zeroed() }"
    for nm, ty in [("G", "HSD_GObj"), ("REFG", "HSD_GObj"), ("OWNG", "HSD_GObj"), ("SG", "HSD_GObj"), ("ITEM", "Item"), ("SITEM", "Item"),
                   ("JT", "HSD_JObj"), ("JC", "HSD_JObj"), ("SJT", "HSD_JObj"), ("DBT", "DynamicBoneTable"), ("SDBT", "DynamicBoneTable"),
                   ("ART", "Article"), ("SART", "Article"), ("IATTR", "ItemAttr"), ("SIATTR", "ItemAttr"), ("VP", "Vec")]:
        d.append(f"static mut {nm}: {ty} = {z};")
    d.append(f"static mut BONES: [HSD_JObj; 24] = {z};\nstatic mut FP1: f32 = 0.0;")
    if attr:
        d.append(f"static mut ATTR: {attr} = {z};\nstatic mut SATTR: {attr} = {z};")
    elif stem == "itkirbygamewatchchefpan":
        d.append("static mut ATTRP: [*mut c_void; 2] = [null_mut(); 2];\nstatic mut SATTRP: [*mut c_void; 2] = [null_mut(); 2];")
    if stem == "itkinoko":
        d.append(f"static mut KANIM: [KinokoAnim; 8] = {z};\nstatic mut KART: [*mut Article; 0x9F] = [null_mut(); 0x9F];\nstatic mut AJ: [HSD_AnimJoint; 8] = {z};")
    d.append("\nunsafe fn setup() {")
    d.append("    G = core::mem::zeroed(); REFG = core::mem::zeroed(); OWNG = core::mem::zeroed(); SG = core::mem::zeroed();")
    d.append("    ITEM = core::mem::zeroed(); SITEM = core::mem::zeroed(); JT = core::mem::zeroed(); JC = core::mem::zeroed(); SJT = core::mem::zeroed();")
    d.append("    BONES = core::mem::zeroed(); DBT = core::mem::zeroed(); SDBT = core::mem::zeroed(); ART = core::mem::zeroed(); SART = core::mem::zeroed();")
    d.append("    IATTR = core::mem::zeroed(); SIATTR = core::mem::zeroed(); VIEWPUN = false;")
    if attr:
        d.append("    ATTR = core::mem::zeroed(); SATTR = core::mem::zeroed();")
    d.append("    g::fill_Item(&mut ITEM); g::fill_Item(&mut SITEM);")
    d.append("    g::fill_HSD_JObj(&mut JT); g::fill_HSD_JObj(&mut JC); g::fill_HSD_JObj(&mut SJT);")
    d.append("    g::fill_ItemAttr(&mut IATTR); g::fill_ItemAttr(&mut SIATTR);")
    if attr:
        d.append(f"    g::fill_{attr}(&mut ATTR); g::fill_{attr}(&mut SATTR);")
    d.append("    JT.flags &= !0x20000u32; JC.flags &= !0x20000u32; SJT.flags &= !0x20000u32;")
    d.append("    G.user_data = &raw mut ITEM as *mut c_void; G.hsd_obj = &raw mut JT as *mut c_void; SG.user_data = &raw mut SITEM as *mut c_void; SG.hsd_obj = &raw mut SJT as *mut c_void;")
    d.append("    JT.child = if rnd() & 1 != 0 { &raw mut JC } else { null_mut() };")
    d.append("    ITEM.xC4_article_data = &raw mut ART; SITEM.xC4_article_data = &raw mut SART;")
    if attr:
        d.append("    ART.x4_specialAttributes = &raw mut ATTR as *mut c_void; SART.x4_specialAttributes = &raw mut SATTR as *mut c_void;")
    d.append("    ITEM.xCC_item_attr = &raw mut IATTR; SITEM.xCC_item_attr = &raw mut SIATTR;")
    d.append("    ITEM.xBBC_dynamicBoneTable = &raw mut DBT; SITEM.xBBC_dynamicBoneTable = &raw mut SDBT;")
    d.append("    for i in 0..100usize { DBT.bones[i] = &raw mut BONES[i % 24]; SDBT.bones[i] = &raw mut BONES[(i + 5) % 24]; }")
    d.append("    ITEM.owner = if rnd() & 1 != 0 { &raw mut OWNG } else { null_mut() }; SITEM.owner = if rnd() & 1 != 0 { &raw mut OWNG } else { null_mut() };")
    if stem == "itkirbygamewatchchefpan":
        d.append("    ART.x4_specialAttributes = &raw mut ATTRP as *mut c_void; SART.x4_specialAttributes = &raw mut SATTRP as *mut c_void;")
        d.append("    SATTRP[0] = if rnd() & 1 != 0 { &raw mut SATTRP[1] as *mut c_void } else { null_mut() }; SATTRP[1] = null_mut();")
        d.append("    ATTRP = [null_mut(); 2];")
    if cfg.get("setup_r"):
        d.append(cfg["setup_r"])
    if stem == "itkinoko":
        d.append("    for i in 0..8usize { KANIM[i].joint = &raw mut AJ[i]; }")
        d.append("    KART = [null_mut(); 0x9F]; KART[It_Kind_Kinoko as usize] = &raw mut SART; SART.x4_specialAttributes = &raw mut KANIM as *mut c_void;")
        d.append("    m::ext::it_804D6D24 = (&raw mut KART) as *mut *mut Article;")
        d.append("    ART.x4_specialAttributes = &raw mut ATTR as *mut c_void;")
    d.append("}\n")
    d.append("unsafe fn hash_all() {")
    d.append("    g::hash_Item(&ITEM); g::hash_Item(&SITEM); g::hash_HSD_JObj(&JT); g::hash_HSD_JObj(&JC); g::hash_HSD_JObj(&SJT);")
    d.append("    g::hash_ItemAttr(&IATTR);")
    if attr:
        d.append(f"    g::hash_{attr}(&ATTR); g::hash_{attr}(&SATTR);")
    d.append("    hm(VP.x.to_bits()); hm(VP.y.to_bits()); hm(VP.z.to_bits()); hm(FP1.to_bits());")
    d.append("}\n")
    d.append(f"#[test]\nfn {stem}_matches_c() {{\n    unsafe {{\n        g::install();")
    regs = ["G", "REFG", "OWNG", "SG", "ITEM", "SITEM", "JT", "JC", "SJT", "BONES", "DBT", "SDBT", "ART", "SART", "IATTR", "SIATTR", "VP", "FP1"]
    if attr:
        regs += ["ATTR", "SATTR"]
    elif stem == "itkirbygamewatchchefpan":
        regs += ["ATTRP", "SATTRP"]
    if stem == "itkinoko":
        regs += ["KANIM", "AJ"]
    for r in regs:
        d.append(f"        preg(&raw const {r}, core::mem::size_of_val(&{r}));")
    d.append("        g::freg_all();")
    d.append(f"        let mut c = Check::new(include_str!(\"data/mtu_{stem}.txt\"));")
    d.append(f"        for op in 0..{len(funcs)}u32 {{\n            for k in 0..{CASES}u32 {{")
    d.append("                case(op, k);\n                setup();\n                let mut ret = 0u32;\n                match op {")
    for i, (n, c) in enumerate(funcs):
        rk, ps = sig_info(c)
        cp, ca, rp, ra = pre_args(ps)
        pun = n in cfg.get("pun", [])
        body = []
        if n in cfg.get("pre", {}):
            body.append(cfg["pre"][n][1])
        if pun:
            body.append("VIEWPUN = true; { let q = (&raw mut ITEM.xDD4_itemVar as *mut u8).add(0x60) as *mut f32; *q = rg(); *q.add(1) = rg(); *q.add(2) = rg(); let q = (&raw mut SITEM.xDD4_itemVar as *mut u8).add(0x60) as *mut f32; *q = rg(); *q.add(1) = rg(); *q.add(2) = rg(); }")
        body += rp
        if n == "it_80293660":
            call = "m::it_80293660((fv() % 6) as i32)"
        else:
            call = f"m::{n}({', '.join(ra)})"
        if rk == "void":
            body.append(call + ";")
        elif rk == "f32":
            body.append(f"ret = ({call}).to_bits();")
        elif rk == "ptr":
            body.append(f"ret = pi({call});")
        else:
            body.append(f"ret = ({call}) as u32;")
        d.append(f"                    {i} => {{ " + " ".join(body) + " }")
    d.append("                    _ => unreachable!(),\n                }\n                hash_all();\n                c.line(op, k, ret);\n            }\n        }")
    d.append(f"        assert_eq!(c.done(), {len(funcs)} * {CASES});\n    }}\n}}")
    return "\n".join(d) + "\n"


def gen(stem):
    cfg = TUS[stem]
    spec = build_spec(stem, cfg)
    MyWalk.cfg = cfg
    gen_tu.Walk = MyWalk
    gen_tu.build(stem, spec, False)
    tu, funcs = functions(stem)
    # static helpers the generated freg_all() names must be pub in the port (see the port files)
    (HERE / "tu" / f"{stem}.c").write_text(c_driver(stem, cfg, tu, funcs), encoding="utf-8", newline="\n")
    (ROOT / "rust/melee_items/tests" / f"tu_{stem}.rs").write_text(r_test(stem, cfg, funcs), encoding="utf-8", newline="\n")
    print(f"mk_itk: {stem}: {len(funcs)} ops")


def run(stem):
    import run_tu
    from specs import SPECS
    cfg = TUS[stem]
    spec = build_spec(stem, cfg)
    BUILD = run_tu.BUILD
    BUILD.mkdir(parents=True, exist_ok=True)
    drv = HERE / "tu" / f"{stem}.c"
    src = gen_tu.patched_source(stem, spec)
    exe = BUILD / f"mtu_{stem}.exe"
    lls = []
    dbg = []
    if os.environ.get("MTU_DBG"):
        dop, dk = os.environ["MTU_DBG"].split(":")
        dbg = [f"-DDBG_OP={dop}", f"-DDBG_K={dk}"]
    for s in [drv] + [ROOT / x for x in LINK]:
        ll = BUILD / f"mtu_{stem}__{s.stem}.ll"
        lo = BUILD / f"mtu_{stem}__{s.stem}.lo.ll"
        run_tu.run([run_tu.CLANG, *run_tu.FLAGS, "-I", str((ROOT / spec["src"]).parent), "-I", str(HERE / "tu"),
                    '-DTU_SRC="' + str(ROOT / src).replace("\\", "/") + '"', *dbg, str(s), "-o", str(ll)], stem)
        run_tu.run([sys.executable, str(run_tu.LOWER), str(ll), str(lo)], stem)
        lls.append(str(lo))
    run_tu.run([run_tu.CLANG, "-O0", "-w", *lls, "-o", str(exe), "-Wl,/OPT:NOICF", "-Wl,/FORCE:UNRESOLVED", "-fuse-ld=lld"], stem)
    out = subprocess.run([str(exe)], capture_output=True, text=True, check=True).stdout
    if dbg:
        tr = [l for l in out.splitlines() if l.startswith(("T ", "H ", "B "))]
        (BUILD / f"dbg_{stem}.txt").write_text(chr(10).join(tr) + chr(10), encoding="utf-8")
        print(f"mk_itk: {stem}: {len(tr)} trace words -> {BUILD / f'dbg_{stem}.txt'}")
        return
    dd = ROOT / "rust/melee_items/tests/data"
    dd.mkdir(parents=True, exist_ok=True)
    (dd / f"mtu_{stem}.txt").write_text(out, encoding="utf-8", newline="\n")
    print(f"mk_itk: {stem}: {len(out.splitlines())} lines")


if __name__ == "__main__":
    os.chdir(ROOT)
    mode = sys.argv[1]
    for s in sys.argv[2:]:
        if mode in ("gen", "all"):
            gen(s)
        if mode in ("run", "all"):
            run(s)
