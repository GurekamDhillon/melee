"""Oracle spec + driver of src/melee/gr/gricemt.c (see ../gen_tu.py, ../mk_tu.py)."""
import os, sys
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
from drvlib import *

V3 = ("{a}->x = rw(); {a}->y = rw(); {a}->z = rw();", "(*{a}).x = rw(); (*{a}).y = rw(); (*{a}).z = rw();")
V3T = ("T(B({a}->x)); T(B({a}->y)); T(B({a}->z));", "t((*{a}).x.to_bits()); t((*{a}).y.to_bits()); t((*{a}).z.to_bits());")

SPEC = dict(
    src="src/melee/gr/gricemt.c",
    types=["HSD_GObj", "Ground", "HSD_JObj", "StageCallbacks", "StageData", "grIceMt_YakumonoParam", "CollData", "Vec",
           "GrJoint", "IceMtRowData", "UnkArchiveStruct", "StageInfo"],
    # views of the union GroundVars: 0 icemt (shorts: the x100/x108 animation records), 1 icemt1 (the map blocks),
    # 2 icemt9, 3 icemt10 (the two scrolling-field controllers), 4 icemt_bg
    unions={"GroundVars": ["icemt", "icemt1", "icemt9", "icemt10", "icemt_bg"]},
    patch=[
        # the animation records of stage 2 / 4 live in the union's x100 / x108 shorts (gp + 0x100 / 0x108 on the GameCube);
        # native `gp + 0x100` is somewhere else in the (pointer-wider) struct, so the C is pointed at the same fields
        ("(s16*) ((u8*) gp + 0x100)", "gp->u.icemt.x100"),
        ("grIceMt_801F929C(param1, &gp->u.icemt1.x34[2]);", "grIceMt_801F929C(param1, gp->u.icemt.x100);"),
        ("grIceMt_801F929C(gobj, &gp->u.icemt1.x34[4]);", "grIceMt_801F929C(gobj, gp->u.icemt.x108);"),
    ],
    null_ret_ok=["HSD_GObj_SetupProc", "grZakoGenerator_801CAE04", "grMaterial_801C8CFC", "grLib_801C96F8", "Ground_GetP1Fighter",
                 "Ground_GetMapGObj", "Ground_GetStageGObj", "Ground_801C3FA4"],
    custom_stubs={
        "efSync_Spawn": dict(
            c="void* efSync_Spawn(s32 a0, HSD_GObj* a1, ...) { __builtin_va_list ap; Vec* v; __builtin_va_start(ap, a1); "
              "v = __builtin_va_arg(ap, Vec*); __builtin_va_end(ap); T(@ID@); T((u32) a0); T(PI(a1)); T(PI(v)); "
              "T(B(v->x)); T(B(v->y)); T(B(v->z)); return 0; }",
            ext="fn efSync_Spawn(a0: i32, a1: *mut HSD_GObj, a2: *mut Vec) -> *mut c_void;",
            hook="|a0, a1, a2| { t(@ID@); t(a0 as u32); t(pi(a1)); t(pi(a2)); unsafe { t((*a2).x.to_bits()); t((*a2).y.to_bits()); "
                 "t((*a2).z.to_bits()); } core::ptr::null_mut() }"),
    },
    ret={
        "Ground_801C3FA4": ("((NJ && (fv() & 3) == 0) ? 0 : &JO[(u32) a1 % 24])",
                            "if super::NJ && fv() & 3 == 0 { core::ptr::null_mut() } else { &raw mut super::JO[(a1 as u32 % 24) as usize] }"),
        "Ground_GetMapGObj": ("((NM && (fv() & 3) == 0) ? 0 : &GS[(u32) a0 % 12])",
                              "if super::NM && fv() & 3 == 0 { core::ptr::null_mut() } else { &raw mut super::GS[(a0 as u32 % 12) as usize] }"),
        "Ground_GetStageGObj": ("((NOK && a0 == NID && (fv() & 7) == 0) ? 0 : &GS[(u32) a0 % 12])",
                                "if super::NOK && a0 == super::NID && fv() & 7 == 0 { core::ptr::null_mut() } else { &raw mut super::GS[(a0 as u32 % 12) as usize] }"),
        "Ground_GetYakumonoParam": ("&YP", "&raw mut super::YP as *mut c_void"),
        "Ground_GetP1Fighter": ("(((fv() & 3) == 0) ? 0 : &FG)", "if fv() & 3 == 0 { core::ptr::null_mut() } else { &raw mut super::FG }"),
        "Stage_80225194": ("((fv() & 1) ? 76u : (((fv() & 1) ? 212u : (fv() % 5))))",
                           "if fv() & 1 != 0 { 76 } else if fv() & 1 != 0 { 212 } else { fv() % 5 }"),
        "HSD_JObjLoadJoint": ("&JO[fv() % 24]", "&raw mut super::JO[(fv() % 24) as usize]"),
        "grDatFiles_GetArchive": ("&ARCH", "&raw mut super::ARCH"),
        "grMaterial_801C8CFC": ("(((fv() & 3) == 0) ? 0 : &IG[fv() & 7])",
                                "if fv() & 3 == 0 { core::ptr::null_mut() } else { &raw mut super::IG[(fv() & 7) as usize] }"),
        # the stage scale is never NaN (two NaN operands of one add have no defined result)
        "Ground_801C0498": ("rgn()", "super::rgn()"),
        "Ground_801C5764": ("(int) (fv() % 4)", "(fv() % 4) as i32"),
    },
    effects={
        "mpJointGetCb1": ("*a1 = (fv() & 1) ? onJointCollision : 0; *a2 = 0;",
                          "*a1 = if fv() & 1 != 0 { Some(super::m::onJointCollision) } else { None }; *a2 = core::ptr::null_mut();"),
        "ftLib_GetPos": (V3[0].format(a="a1"), V3[1].format(a="a1")),
        "it_8026B294": (V3[0].format(a="a1"), V3[1].format(a="a1")),
        "Camera_RequestQuake": (V3T[0].format(a="a1"), V3T[1].format(a="a1")),
        "grLib_801C96F8": (V3T[0].format(a="a2"), V3T[1].format(a="a2")),
        "memzero": ("zero(a0, (unsigned) a1);", "core::ptr::write_bytes(a0 as *mut u8, 0, a1 as usize);"),
    })

# which view of GroundVars each map's Ground is filled / hashed under
VIEWC = "((i >= 1 && i <= 6) ? 1 : ((i == 8) ? 4 : ((i == 9) ? 2 : ((i == 10) ? 3 : 0))))"
VIEWR = "(if (1..=6).contains(&i) { 1 } else if i == 8 { 4 } else if i == 9 { 2 } else if i == 10 { 3 } else { 0 })"

SETUP_C = """
            for (i = 0; i < 24; i++) { JO[i].child = (2 * i + 1 < 24) ? &JO[2 * i + 1] : 0; JO[i].next = (2 * i + 2 < 24) ? &JO[2 * i + 2] : 0; JO[i].parent = 0; }
            for (i = 0; i < 24; i++) { if (2 * i + 1 < 24) JO[2 * i + 1].parent = &JO[i]; if (2 * i + 2 < 24) JO[2 * i + 2].parent = JO[i].parent; }
            for (i = 0; i < 8; i++) { IG[i].user_data = &IT; IG[i].hsd_obj = &JO[(i + 9) % 24]; }
            IT.xDD4_itemVar.mato.x4 = &JO[3];
            ARCH.unk4 = &SD; SD.unk8 = X8; X8[7].unk0 = (HSD_Joint*) &JO[5];
            FG.user_data = 0; FG.hsd_obj = 0;
            for (i = 0; i < 12; i++) {
                GS[i].user_data = &GG[i]; GS[i].hsd_obj = &JO[i];
                zero(&GG[i], sizeof GG[i]);
                USEL = @VIEW@;
                fill_Ground(&GG[i]);
                GG[i].gobj = &GS[i];
            }
            USEL = OPUSEL[op];
            for (i = 1; i <= 6; i++) {
                HSD_JObj** pj = &GG[i].u.icemt1.x4;
                for (j = 0; j < 12; j++) pj[j] = ((fv() & 1) ? &JO[(i * 5 + j) % 24] : 0);
                for (j = 0; j < 5; j++) GG[i].u.icemt1.x34[j] = ((fv() & 1) ? &IG[j] : 0);
            }
            yakumono_param = &YP;
            YP.field_ixs = FX; YP.xB0 = TB0; YP.xB4 = TB4;
            for (j = 0; j < 8; j++) FX[j] = (j < 5 || (fv() & 3) != 0) ? (short) (fv() % 6) : (short) -1;
            for (j = 0; j < 16; j++) {
                TB0[j] = (j == 0 || (fv() & 3) != 0) ? (short) (1 + fv() % 30) : (short) -1;
                TB4[j] = (j == 0 || (fv() & 3) != 0) ? (short) (fv() % 20) : (short) -1;
            }
            GG[9].u.icemt9.x14 = (short) ((fv() & 3) == 0);
            GG[10].u.icemt10.x1A = (short) (fv() % 8); GG[10].u.icemt10.x1C = (short) (fv() % 7);
            for (j = 0; j < 12; j++) GG[9].u.icemt9.x18[j] = ((fv() & 3) == 0) ? (short) (fv() % 3) : (short) 0;
            GG[9].u.icemt9.x18[4 + fv() % 8] = 0;
            for (j = 0; j < 6; j++) GG[9].u.icemt9.x30[j] = ((fv() & 7) == 0) ? (short) (1 + fv() % 3) : (short) 0;
            NOK = 0; NID = 99; NM = 0; NJ = 0; H2A = 0; H2N = 0; H4A = 0; H4N = 0;
            grIm_804D69E8 = 0; grIm_804D69EC = 0; grIm_804D69F0 = 0;
            for (j = 0; j < 20; j++) OA[j] = 0;
"""

SETUP_R = """
                for i in 0..24usize {
                    JO[i].child = if 2 * i + 1 < 24 { &raw mut JO[2 * i + 1] } else { core::ptr::null_mut() };
                    JO[i].next = if 2 * i + 2 < 24 { &raw mut JO[2 * i + 2] } else { core::ptr::null_mut() };
                    JO[i].parent = core::ptr::null_mut();
                }
                for i in 0..24usize {
                    if 2 * i + 1 < 24 { JO[2 * i + 1].parent = &raw mut JO[i]; }
                    if 2 * i + 2 < 24 { JO[2 * i + 2].parent = JO[i].parent; }
                }
                for i in 0..8usize { IG[i].user_data = &raw mut IT as *mut c_void; IG[i].hsd_obj = &raw mut JO[(i + 9) % 24] as *mut c_void; }
                IT.xDD4_itemVar.mato.x4 = &raw mut JO[3];
                ARCH.unk4 = &raw mut SD; SD.unk8 = X8.as_mut_ptr(); X8[7].unk0 = &raw mut JO[5] as *mut HSD_Joint;
                FG.user_data = core::ptr::null_mut(); FG.hsd_obj = core::ptr::null_mut();
                for i in 0..12usize {
                    GS[i].user_data = &raw mut GG[i] as *mut c_void; GS[i].hsd_obj = &raw mut JO[i] as *mut c_void;
                    GG[i] = core::mem::zeroed();
                    USEL = @VIEW@;
                    g::fill_Ground(&mut GG[i]);
                    GG[i].gobj = &raw mut GS[i];
                }
                USEL = OPUSEL[op as usize];
                for i in 1..=6usize {
                    let pj = &raw mut GG[i].u.icemt1.x4 as *mut *mut HSD_JObj;
                    for j in 0..12usize { *pj.add(j) = if fv() & 1 != 0 { &raw mut JO[(i * 5 + j) % 24] } else { core::ptr::null_mut() }; }
                    for j in 0..5usize { GG[i].u.icemt1.x34[j] = if fv() & 1 != 0 { &raw mut IG[j] } else { core::ptr::null_mut() }; }
                }
                m::yakumono_param = &raw mut YP;
                YP.field_ixs = FX.as_mut_ptr(); YP.xB0 = TB0.as_mut_ptr(); YP.xB4 = TB4.as_mut_ptr();
                for j in 0..8usize { FX[j] = if j < 5 || fv() & 3 != 0 { (fv() % 6) as i16 } else { -1 }; }
                for j in 0..16usize {
                    TB0[j] = if j == 0 || fv() & 3 != 0 { (1 + fv() % 30) as i16 } else { -1 };
                    TB4[j] = if j == 0 || fv() & 3 != 0 { (fv() % 20) as i16 } else { -1 };
                }
                GG[9].u.icemt9.x14 = (fv() & 3 == 0) as i16;
                GG[10].u.icemt10.x1A = (fv() % 8) as i16; GG[10].u.icemt10.x1C = (fv() % 7) as i16;
                for j in 0..12usize { GG[9].u.icemt9.x18[j] = if fv() & 3 == 0 { (fv() % 3) as i16 } else { 0 }; }
                GG[9].u.icemt9.x18[(4 + fv() % 8) as usize] = 0;
                for j in 0..6usize { GG[9].u.icemt9.x30[j] = if fv() & 7 == 0 { (1 + fv() % 3) as i16 } else { 0 }; }
                NOK = false; NID = 99; NM = false; NJ = false; H2A = 0; H2N = 0; H4A = 0; H4N = 0;
                m::grIm_804D69E8 = core::ptr::null_mut(); m::grIm_804D69EC = core::ptr::null_mut(); m::grIm_804D69F0 = core::ptr::null_mut();
                for j in 0..20usize { OA[j] = core::ptr::null_mut(); }
"""

POST_C = """
            { short* p2 = (short*) &GG[2].u.icemt + 0x3C / 2; short* p4 = (short*) &GG[4].u.icemt + 0x3C / 2;
              for (j = H2A; j < H2A + H2N; j++) HM((u32) (unsigned short) p2[j]);
              for (j = H4A; j < H4A + H4N; j++) HM((u32) (unsigned short) p4[j]); }
            /* the pointer slots x1C..x2C of the maps 2 and 4 overlap the animation records (their bytes are shorts there) */
            for (i = 2; i <= 4; i += 2) { GG[i].u.icemt1.x1C = 0; GG[i].u.icemt1.x20 = 0; GG[i].u.icemt1.x24 = 0; GG[i].u.icemt1.x28 = 0; GG[i].u.icemt1.x2C = 0; }
            for (i = 0; i < 12; i++) { USEL = @VIEW@; hash_Ground(&GG[i]); }
            USEL = OPUSEL[op];
            HM(PI(grIm_804D69E8)); HM(PI(grIm_804D69EC)); HM(PI(grIm_804D69F0)); HM(PI(yakumono_param));
            for (j = 0; j < 20; j++) HM(PI(OA[j]));
"""

POST_R = """
                {
                    let p2 = (&raw mut GG[2].u.icemt as *mut i16).add(0x3C / 2);
                    let p4 = (&raw mut GG[4].u.icemt as *mut i16).add(0x3C / 2);
                    for j in H2A..H2A + H2N { hm(*p2.add(j) as u16 as u32); }
                    for j in H4A..H4A + H4N { hm(*p4.add(j) as u16 as u32); }
                }
                for i in [2usize, 4] {
                    GG[i].u.icemt1.x1C = core::ptr::null_mut(); GG[i].u.icemt1.x20 = core::ptr::null_mut(); GG[i].u.icemt1.x24 = core::ptr::null_mut();
                    GG[i].u.icemt1.x28 = core::ptr::null_mut(); GG[i].u.icemt1.x2C = core::ptr::null_mut();
                }
                for i in 0..12usize { USEL = @VIEW@; g::hash_Ground(&GG[i]); }
                USEL = OPUSEL[op as usize];
                hm(pi(m::grIm_804D69E8)); hm(pi(m::grIm_804D69EC)); hm(pi(m::grIm_804D69F0)); hm(pi(m::yakumono_param));
                for j in 0..20usize { hm(pi(OA[j])); }
"""


# valid segment ids (1..6) for the field controller `k` (view v)
def ids_c(k, v):
    return f"GG[{k}].u.{v}.x0.ids.under = (short) (1 + fv() % 6); GG[{k}].u.{v}.x0.ids.upper = (short) (1 + fv() % 6);"


def ids_r(k, v):
    return f"GG[{k}].u.{v}.x0.ids.under = (1 + fv() % 6) as i16; GG[{k}].u.{v}.x0.ids.upper = (1 + fv() % 6) as i16;"


# the free-segment guarantee of fn_801F9038: two zero slots, neither the one of an id in use
def free9_c():
    return "{ int u = GG[9].u.icemt9.x0.ids.under; GG[9].u.icemt9.x30[u % 6] = 0; GG[9].u.icemt9.x30[(u + 1) % 6] = 0; }"


def free9_r():
    return "{ let u = GG[9].u.icemt9.x0.ids.under as i32; GG[9].u.icemt9.x30[(u % 6) as usize] = 0; GG[9].u.icemt9.x30[((u + 1) % 6) as usize] = 0; }"


# the walk state of grIceMt_801FA364 on the controller `k`
def st_c(k, v):
    return (f"GG[{k}].u.{v}.x0.state.phase = (short) (fv() & 1); GG[{k}].u.{v}.x0.state.delay = (short) ((int) (fv() % 5) - 2); "
            f"GG[{k}].u.{v}.x0.state.lerp_count = (short) (fv() % 4); GG[{k}].u.{v}.x0.state.burst_count = (short) (fv() % 3); "
            f"GG[{k}].u.{v}.x0.state.idx = (short) (1 + fv() % 30);")


def st_r(k, v):
    return (f"GG[{k}].u.{v}.x0.state.phase = (fv() & 1) as i16; GG[{k}].u.{v}.x0.state.delay = (fv() % 5) as i16 - 2; "
            f"GG[{k}].u.{v}.x0.state.lerp_count = (fv() % 4) as i16; GG[{k}].u.{v}.x0.state.burst_count = (fv() % 3) as i16; "
            f"GG[{k}].u.{v}.x0.state.idx = (1 + fv() % 30) as i16;")


# a break-animation record: 7 shorts from x100 (+off)
def rec_c(g, off):
    return (f"{{ short* p = (short*) &GG[{g}].u.icemt + 0x3C / 2 + {off}; p[0] = (short) (fv() & 1); p[1] = (short) (fv() % 6); p[2] = (short) (fv() % 40); "
            f"p[3] = (fv() & 1) ? (short) -1 : (short) (fv() % 40); p[4] = (short) (fv() % 700); p[5] = (short) (fv() % 6); p[6] = (short) (fv() % 6); }}")


def rec_r(g, off):
    return (f"{{ let p = (&raw mut GG[{g}].u.icemt as *mut i16).add(0x3C / 2 + {off}); *p = (fv() & 1) as i16; *p.add(1) = (fv() % 6) as i16; "
            f"*p.add(2) = (fv() % 40) as i16; *p.add(3) = if fv() & 1 != 0 {{ -1 }} else {{ (fv() % 40) as i16 }}; *p.add(4) = (fv() % 700) as i16; "
            f"*p.add(5) = (fv() % 6) as i16; *p.add(6) = (fv() % 6) as i16; }}")


def gs(name, k, pre=("", ""), usel=0, cases=None):
    d = vg(name, usel, f"GS[{k}]")
    d["pre_c"], d["pre_r"] = pre
    if cases:
        d["cases"] = cases
    return d


def pgs(name, k, pre=("", "")):
    d = pg(name, 0, f"GS[{k}]")
    d["pre_c"], d["pre_r"] = pre
    return d


def hs(g, a, n):
    return (f"H{g}A = {a}; H{g}N = {n};", f"H{g}A = {a}; H{g}N = {n};")


def with_hs(d, *h):
    d["pre_c"] = " ".join(x[0] for x in h) + " " + d.get("pre_c", "")
    d["pre_r"] = " ".join(x[1] for x in h) + " " + d.get("pre_r", "")
    return d


NMJ = ("NM = 1; NJ = 1;", "NM = true; NJ = true;")

COLL_C = "CD.x0_gobj = (fv() & 1) ? &FG : 0; if (fv() & 1) CD.x34_flags.b1234 = 1;"
COLL_R = "CD.x0_gobj = if fv() & 1 != 0 { &raw mut FG } else { core::ptr::null_mut() }; if fv() & 1 != 0 { CD.x34_flags.set_b1234(1); }"


def joint_cb(name, g, off):
    pre_c = f"NM = 1; {rec_c(g, off)} " + COLL_C + " int jid = (int) fv(); int x50 = (int) fv(); u32 gk = fv(); float dy = rw();"
    pre_r = f"NM = true; {rec_r(g, off)} " + COLL_R + " let jid = fv() as i32; let x50 = fv() as i32; let gk = fv(); let dy = rw();"
    return dict(name=name, pre_c=pre_c, pre_r=pre_r,
                c=f"{name}(&GG[{g}], jid, &CD, x50, gk, dy);",
                r=f"m::{name}(&raw mut GG[{g}] as *mut c_void, jid, &raw mut CD, x50, gk, dy);")


OPS = [
    dict(name="grIceMt_801F6868", c="grIceMt_801F6868((bool) fv());", r="m::grIceMt_801F6868((fv() & 1) as i32);"),
    dict(name="grIceMt_801F686C", cases=400, c="grIceMt_801F686C();", r="m::grIceMt_801F686C();"),
    dict(name="grIceMt_801F7080", cases=300,
         pre_c="NM = 1; grIm_804D69E8 = (fv() & 1) ? &GS[1] : 0; grIm_804D69EC = (fv() & 1) ? &GS[2] : 0; grIm_804D69F0 = (fv() & 1) ? &GS[3] : 0;",
         pre_r="NM = true; m::grIm_804D69E8 = if fv() & 1 != 0 { &raw mut GS[1] } else { core::ptr::null_mut() }; "
               "m::grIm_804D69EC = if fv() & 1 != 0 { &raw mut GS[2] } else { core::ptr::null_mut() }; "
               "m::grIm_804D69F0 = if fv() & 1 != 0 { &raw mut GS[3] } else { core::ptr::null_mut() };",
         c="grIceMt_801F7080();", r="m::grIceMt_801F7080();"),
    v0("grIceMt_801F71DC"), p0("grIceMt_801F71E0"),
    dict(name="setupStageCallbacks", cases=600, pre_c="int a = (int) (fv() % 11); NOK = 1; NID = a;", pre_r="let a = (fv() % 11) as i32; NOK = true; NID = a;",
         c="ret = PI(setupStageCallbacks(a));", r="ret = pi(m::setupStageCallbacks(a));"),
    gs("stageGObj0_OnInit", 0, NMJ), pgs("stageGObj0_Callback1", 0), gs("stageGObj0_GObjProc", 0), gs("stageGObj0_Callback3", 0),
    gs("fn_801F75EC", 9),
    gs("stageGObj9_OnInit", 9), pgs("stageGObj9_Callback1", 9),
    gs("stageGObj9_GObjProc", 9, (ids_c(9, "icemt9") + " " + free9_c() + " " + st_c(9, "icemt9"),
                                  ids_r(9, "icemt9") + " " + free9_r() + " " + st_r(9, "icemt9")), cases=800),
    gs("stageGObj9_Callback3", 9),
    gs("stageGObj10_UnkGroundCallback", 10), gs("stageGObj10_OnInit", 10),
    pgs("stageGObj10_Callback1", 10, (ids_c(10, "icemt10"), ids_r(10, "icemt10"))),
    gs("stageGObj10_GObjProc", 10,
       (ids_c(10, "icemt10") + " " + st_c(10, "icemt10") + " GG[10].u.icemt10.x14_b0 = fv() % 5 == 0; GG[10].u.icemt10.x14_b2 = fv() & 1; "
        "GG[10].u.icemt10.x14_b4 = fv() & 1; GG[10].u.icemt10.x14_b3 = fv() & 1;",
        ids_r(10, "icemt10") + " " + st_r(10, "icemt10") + " GG[10].u.icemt10.set_x14_b0((fv() % 5 == 0) as u8); GG[10].u.icemt10.set_x14_b2((fv() & 1) as u8); "
        "GG[10].u.icemt10.set_x14_b4((fv() & 1) as u8); GG[10].u.icemt10.set_x14_b3((fv() & 1) as u8);"), cases=800),
    gs("stageGObj10_Callback3", 10),
    gs("stageGObj1_OnInit", 1), pgs("stageGObj1_Callback1", 1), gs("stageGObj1_GObjProc", 1), gs("stageGObj1_Callback3", 1),
    with_hs(gs("stageGObj2_OnInit", 2), hs(2, 0, 14)), pgs("stageGObj2_Callback1", 2),
    with_hs(gs("stageGObj2_GObjProc", 2, (rec_c(2, 0) + " " + rec_c(2, 7), rec_r(2, 0) + " " + rec_r(2, 7))), hs(2, 0, 14)),
    gs("stageGObj2_Callback3", 2),
    gs("stageGObj3_OnInit", 3), pgs("stageGObj3_Callback1", 3), gs("stageGObj3_GObjProc", 3), gs("stageGObj3_Callback3", 3),
    with_hs(gs("stageGObj4_OnInit", 4), hs(4, 4, 7)), pgs("stageGObj4_Callback1", 4), with_hs(gs("stageGObj4_GObjProc", 4, (rec_c(4, 4), rec_r(4, 4))), hs(4, 4, 7)), gs("stageGObj4_Callback3", 4),
    gs("stageGObj5_OnInit", 5), pgs("stageGObj5_Callback1", 5), gs("stageGObj5_GObjProc", 5), gs("stageGObj5_Callback3", 5),
    gs("stageGObj6_OnInit", 6), pgs("stageGObj6_Callback1", 6), gs("stageGObj6_GObjProc", 6), gs("stageGObj6_Callback3", 6),
    gs("stageGObj8_OnInit", 8), pgs("stageGObj8_Callback1", 8), gs("stageGObj8_GObjProc", 8, cases=500), gs("stageGObj8_Callback3", 8),
    dict(name="fn_801F8C64", pre_c="float a = rw();", pre_r="let a = rw();",
         c="fn_801F8C64(&IG[2], &GG[2], &VW[0], &GS[3], a);",
         r="m::fn_801F8C64(&raw mut IG[2], &raw mut GG[2], &raw mut VW[0], &raw mut GS[3], a);"),
    dict(name="grIceMt_801F8CDC", cases=300,
         pre_c="short ji[20]; int n = (int) (fv() % 21); for (j = 0; j < 20; j++) ji[j] = (short) (fv() % 24);",
         pre_r="let mut ji = [0i16; 20]; let n = (fv() % 21) as i32; for j in 0..20usize { ji[j] = (fv() % 24) as i16; }",
         c="grIceMt_801F8CDC(&GS[2], ji, n, OA);", r="m::grIceMt_801F8CDC(&raw mut GS[2], ji.as_mut_ptr(), n, OA.as_mut_ptr());"),
    dict(name="fn_801F8E58", pre_c="int o = -777;", pre_r="let mut o = -777i32;",
         c="ret = (u32) fn_801F8E58(&GS[9], &o); T((u32) o);", r="ret = m::fn_801F8E58(&raw mut GS[9], &mut o) as u32; t(o as u32);"),
    dict(name="fn_801F9038", pre_c=ids_c(9, "icemt9") + " " + free9_c(), pre_r=ids_r(9, "icemt9") + " " + free9_r(),
         c="ret = (u32) fn_801F9038(&GS[9]);", r="ret = m::fn_801F9038(&raw mut GS[9]) as u32;"),
    dict(name="fn_801F9150", pre_c="int o = -777;", pre_r="let mut o = -777i32;",
         c="ret = (u32) fn_801F9150(&GS[10], &o); T((u32) o);", r="ret = m::fn_801F9150(&raw mut GS[10], &mut o) as u32; t(o as u32);"),
    dict(name="fn_801F91A8", c="ret = (u32) fn_801F91A8(&GS[10]);", r="ret = m::fn_801F91A8(&raw mut GS[10]) as u32;"),
    with_hs(dict(name="grIceMt_801F91EC",
         pre_c="int p3 = (int) (fv() % 40); int p4 = (fv() & 1) ? -1 : (int) (fv() % 40); int p5 = (int) (fv() % 700); int p6 = (int) (fv() % 700); "
               "int p7 = (int) (fv() % 700); int usecb = (int) (fv() & 1);",
         pre_r="let p3 = (fv() % 40) as i32; let p4 = if fv() & 1 != 0 { -1 } else { (fv() % 40) as i32 }; let p5 = (fv() % 700) as i32; "
               "let p6 = (fv() % 700) as i32; let p7 = (fv() % 700) as i32; let usecb = (fv() & 1) as i32;",
         c="grIceMt_801F91EC(&GS[2], (short*) &GG[2].u.icemt + 0x3C / 2, p3, p4, p5, p6, p7, usecb ? fn_801F9338 : 0);",
         r="m::grIceMt_801F91EC(&raw mut GS[2], (&raw mut GG[2].u.icemt as *mut i16).add(0x3C / 2), p3, p4, p5, p6, p7, "
           "if usecb != 0 { Some(m::fn_801F9338) } else { None });"), hs(2, 0, 7)),
    with_hs(dict(name="grIceMt_801F929C", pre_c=rec_c(2, 0), pre_r=rec_r(2, 0),
         c="grIceMt_801F929C(&GS[2], (short*) &GG[2].u.icemt + 0x3C / 2);",
         r="m::grIceMt_801F929C(&raw mut GS[2], (&raw mut GG[2].u.icemt as *mut i16).add(0x3C / 2) as *mut c_void);"), hs(2, 0, 7)),
    with_hs(joint_cb("fn_801F9338", 2, 0), hs(2, 0, 7)), with_hs(joint_cb("fn_801F9448", 2, 7), hs(2, 7, 7)), with_hs(joint_cb("fn_801F9558", 4, 4), hs(4, 4, 7)),
    dict(name="grIceMt_801F9668", pre_c="float a = rw();", pre_r="let a = rw();", c="grIceMt_801F9668(a);", r="m::grIceMt_801F9668(a);"),
    dict(name="grIceMt_801F96E0",
         pre_c=ids_c(9, "icemt9") + " if ((fv() & 3) == 0) GG[9].u.icemt9.x0.ids.under = -1; if ((fv() & 3) == 0) GG[9].u.icemt9.x0.ids.upper = -1; float a = rw();",
         pre_r=ids_r(9, "icemt9") + " if fv() & 3 == 0 { GG[9].u.icemt9.x0.ids.under = -1; } if fv() & 3 == 0 { GG[9].u.icemt9.x0.ids.upper = -1; } let a = rw();",
         c="(void) grIceMt_801F96E0(&GG[9].u.icemt9.x0.ids, a);", r="let _ = m::grIceMt_801F96E0(&raw mut GG[9].u.icemt9.x0.ids, a);"),
    gs("grIceMt_801F98A8", 1), gs("grIceMt_801F98A8", 3),
    dict(name="grIceMt_801F993C", cases=300, pre_c="int a = (int) (1 + fv() % 6); int b = (int) (1 + fv() % 6);",
         pre_r="let a = (1 + fv() % 6) as i32; let b = (1 + fv() % 6) as i32;",
         c="ret = B(grIceMt_801F993C(a, b));", r="ret = m::grIceMt_801F993C(a, b).to_bits();"),
    dict(name="grIceMt_801F9ACC", cases=1500,
         pre_c=ids_c(9, "icemt9") + " " + free9_c() + " if ((fv() & 7) == 0) GG[9].u.icemt9.x0.ids.under = -1; float a = rw(); "
               "JO[GG[9].u.icemt9.x0.ids.upper].translate.y = rw();",
         pre_r=ids_r(9, "icemt9") + " " + free9_r() + " if fv() & 7 == 0 { GG[9].u.icemt9.x0.ids.under = -1; } let a = rw(); "
               "JO[GG[9].u.icemt9.x0.ids.upper as usize].translate.y = rw();",
         c="ret = (u32) grIceMt_801F9ACC(&GG[9].u.icemt9.x0.ids, a, fn_801F9038, &GS[9]);",
         r="ret = m::grIceMt_801F9ACC(&raw mut GG[9].u.icemt9.x0.ids, a, Some(m::fn_801F9038), &raw mut GS[9]) as u32;"),
    dict(name="grIceMt_801F9ACC_10", cases=1500,
         pre_c=ids_c(10, "icemt10") + " if ((fv() & 7) == 0) GG[10].u.icemt10.x0.ids.upper = -1; float a = rw(); "
               "JO[GG[10].u.icemt10.x0.ids.upper & 15].translate.y = rw();",
         pre_r=ids_r(10, "icemt10") + " if fv() & 7 == 0 { GG[10].u.icemt10.x0.ids.upper = -1; } let a = rw(); "
               "JO[(GG[10].u.icemt10.x0.ids.upper & 15) as usize].translate.y = rw();",
         c="ret = (u32) grIceMt_801F9ACC(&GG[10].u.icemt10.x0.ids, a, fn_801F91A8, &GS[10]);",
         r="ret = m::grIceMt_801F9ACC(&raw mut GG[10].u.icemt10.x0.ids, a, Some(m::fn_801F91A8), &raw mut GS[10]) as u32;"),
    dict(name="grIceMt_801FA0BC", cases=300, pre_c=ids_c(9, "icemt9"), pre_r=ids_r(9, "icemt9"),
         c="grIceMt_801FA0BC(&GG[9].u.icemt9.x0.ids);", r="m::grIceMt_801FA0BC(&raw mut GG[9].u.icemt9.x0.ids);"),
    dict(name="grIceMt_801FA364_8E58", cases=600, pre_c=st_c(9, "icemt9") + " VW[0].x = rw();", pre_r=st_r(9, "icemt9") + " VW[0].x = rw();",
         c="ret = (u32) grIceMt_801FA364(&GG[9].u.icemt9.x0.state, &VW[0].x, fn_801F8E58, &GS[9]);",
         r="ret = m::grIceMt_801FA364(&raw mut GG[9].u.icemt9.x0.state, &raw mut VW[0].x, m::fn_801F8E58, &raw mut GS[9]) as u32;"),
    dict(name="grIceMt_801FA364_9150", cases=600, pre_c=st_c(10, "icemt10") + " VW[0].x = rw();", pre_r=st_r(10, "icemt10") + " VW[0].x = rw();",
         c="ret = (u32) grIceMt_801FA364(&GG[10].u.icemt10.x0.state, &VW[0].x, fn_801F9150, &GS[10]);",
         r="ret = m::grIceMt_801FA364(&raw mut GG[10].u.icemt10.x0.state, &raw mut VW[0].x, m::fn_801F9150, &raw mut GS[10]) as u32;"),
    dict(name="fn_801FA4CC", pre_c="int a = (int) (fv() % 3);", pre_r="let a = (fv() % 3) as i32;",
         c="ret = (u32) fn_801FA4CC(a);", r="ret = m::fn_801FA4CC(a) as u32;"),
    dict(name="grIceMt_801FA500", cases=400,
         pre_c="int k = (int) (1 + fv() % 6); HSD_JObj* t = ((fv() & 7) == 0) ? &JX : &JO[fv() % 24];",
         pre_r="let k = (1 + fv() % 6) as usize; let t = if fv() & 7 == 0 { &raw mut JX } else { &raw mut JO[(fv() % 24) as usize] };",
         c="ret = (u32) grIceMt_801FA500(&GS[k], t);", r="ret = m::grIceMt_801FA500(&raw mut GS[k], t) as u32;"),
    dict(name="grIceMt_801FA6D8", pre_c="NM = 1;", pre_r="NM = true;", c="grIceMt_801FA6D8();", r="m::grIceMt_801FA6D8();"),
    dict(name="grIceMt_801FA728", pre_c="NM = 1;", pre_r="NM = true;", c="grIceMt_801FA728(&VW[1]);", r="m::grIceMt_801FA728(&raw mut VW[1]);"),
    dict(name="onJointCollision",
         pre_c="NM = 1; " + COLL_C + " int jid = (int) fv(); int x50 = (int) fv(); u32 gk = fv(); float dy = rw();",
         pre_r="NM = true; " + COLL_R + " let jid = fv() as i32; let x50 = fv() as i32; let gk = fv(); let dy = rw();",
         c="onJointCollision(&GG[10], jid, &CD, x50, gk, dy);", r="m::onJointCollision(&raw mut GG[10] as *mut c_void, jid, &raw mut CD, x50, gk, dy);"),
    dict(name="grIceMt_801FA854", pre_c="NM = 1; NJ = 1;", pre_r="NM = true; NJ = true;", c="grIceMt_801FA854();", r="m::grIceMt_801FA854();"),
    dict(name="grIceMt_801FA8F8", pre_c="int a = (int) fv();", pre_r="let a = fv() as i32;", c="ret = PI(grIceMt_801FA8F8(a));", r="ret = pi(m::grIceMt_801FA8F8(a));"),
    dict(name="grIceMt_801FA900", pre_c="int a = (int) fv();", pre_r="let a = fv() as i32;",
         c="ret = (u32) grIceMt_801FA900(&VW[0], a, &JO[0]);", r="ret = m::grIceMt_801FA900(&raw mut VW[0], a, &raw mut JO[0]) as u32;"),
    dict(name="tables", cases=1,
         c="for (j = 0; j < 11; j++) hash_StageCallbacks(&stage_callbacks[j]); hash_StageData(&grIm_StageData); "
           "for (j = 0; j < 6; j++) { HM((u32) icemt_field[j].gobj_id); HM(B(icemt_field[j].x4)); HM(B(icemt_field[j].x8)); } "
           "for (j = 0; j < 195; j++) hash_GrJoint(&grIm_803E40B0[j]); for (j = 0; j < 217; j++) HM((u32) grIm_803E4544[j]); "
           "for (j = 0; j < 5; j++) { HM(B(grIm_803B8220[j].x)); HM(B(grIm_803B8220[j].y)); HM(B(grIm_803B8220[j].z)); }",
         r="for j in 0..11usize { g::hash_StageCallbacks(&m::stage_callbacks[j]); } g::hash_StageData(&m::grIm_StageData); "
           "for j in 0..6usize { hm(m::icemt_field[j].gobj_id as u32); hm(m::icemt_field[j].x4.to_bits()); hm(m::icemt_field[j].x8.to_bits()); } "
           "for j in 0..195usize { g::hash_GrJoint(&m::grIm_803E40B0[j]); } for j in 0..217usize { hm(m::grIm_803E4544[j] as u32); } "
           "for j in 0..5usize { hm(m::grIm_803B8220[j].x.to_bits()); hm(m::grIm_803B8220[j].y.to_bits()); hm(m::grIm_803B8220[j].z.to_bits()); }"),
]

DRV = dict(
    state=[("GS", "HSD_GObj", 12), ("JO", "HSD_JObj", 24), ("IG", "HSD_GObj", 8), ("YP", "struct grIceMt_YakumonoParam", 1), ("CD", "CollData", 1),
           ("VW", "Vec", 3), ("ARCH", "UnkArchiveStruct", 1), ("SI", "StageInfo", 1, "m::ext::stage_info")],
    decl_c=chr(10).join([
        "static Ground GG[12]; static Item IT; static struct UnkStageDat SD; static __typeof__(((struct UnkStageDat*) 0)->unk8[0]) X8[8];",
        "static HSD_GObj FG; static HSD_JObj JX; static short FX[8]; static short TB0[16]; static short TB4[16]; static HSD_GObj* OA[20];",
        "static int NM; static int NJ; static int NOK; static int NID; static int H2A, H2N, H4A, H4N;",
        "static float rgn(void) { float v = rg(); return v != v ? 1.0f : v; }"]),
    decl_r=chr(10).join([
        "use m::grIceMt_YakumonoParam;", "static mut GG: [Ground; 12] = unsafe { core::mem::zeroed() };",
        "static mut IT: Item = unsafe { core::mem::zeroed() };", "static mut SD: UnkStageDat = unsafe { core::mem::zeroed() };",
        "static mut X8: [UnkStageDat_x8_t; 8] = unsafe { core::mem::zeroed() };", "static mut FG: HSD_GObj = unsafe { core::mem::zeroed() };",
        "static mut JX: HSD_JObj = unsafe { core::mem::zeroed() };", "static mut FX: [i16; 8] = [0; 8];", "static mut TB0: [i16; 16] = [0; 16];",
        "static mut TB4: [i16; 16] = [0; 16];", "static mut OA: [*mut HSD_GObj; 20] = [core::ptr::null_mut(); 20];",
        "fn rgn() -> f32 { let v = rg(); if v.is_nan() { 1.0 } else { v } }", "static mut H2A: usize = 0;", "static mut H2N: usize = 0;", "static mut H4A: usize = 0;", "static mut H4N: usize = 0;", "static mut NM: bool = false;", "static mut NJ: bool = false;", "static mut NOK: bool = false;", "static mut NID: i32 = 0;"]),
    regs=[("GG", "sizeof GG", "&raw const GG", "core::mem::size_of_val(&GG)"),
          ("&IT", "sizeof IT", "&raw const IT", "core::mem::size_of_val(&IT)"),
          ("&SD", "sizeof SD", "&raw const SD", "core::mem::size_of_val(&SD)"),
          ("X8", "sizeof X8", "&raw const X8", "core::mem::size_of_val(&X8)"),
          ("&FG", "sizeof FG", "&raw const FG", "core::mem::size_of_val(&FG)"),
          ("&JX", "sizeof JX", "&raw const JX", "core::mem::size_of_val(&JX)"),
          ("FX", "sizeof FX", "&raw const FX", "core::mem::size_of_val(&FX)"),
          ("TB0", "sizeof TB0", "&raw const TB0", "core::mem::size_of_val(&TB0)"),
          ("TB4", "sizeof TB4", "&raw const TB4", "core::mem::size_of_val(&TB4)")],
    setup_c=SETUP_C.replace("@VIEW@", VIEWC),
    setup_r=SETUP_R.replace("@VIEW@", VIEWR),
    post_c=POST_C.replace("@VIEW@", VIEWC),
    post_r=POST_R.replace("@VIEW@", VIEWR),
    ops=OPS,
)
