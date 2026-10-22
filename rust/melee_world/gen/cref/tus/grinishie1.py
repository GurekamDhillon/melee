"""Oracle spec + driver of src/melee/gr/grinishie1.c (see ../gen_tu.py, ../mk_tu.py)."""
import os, sys
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
from drvlib import *

V3 = ("{a}->x = rw(); {a}->y = rw(); {a}->z = rw();", "(*{a}).x = rw(); (*{a}).y = rw(); (*{a}).z = rw();")
V3T = ("T(B({a}->x)); T(B({a}->y)); T(B({a}->z));", "t((*{a}).x.to_bits()); t((*{a}).y.to_bits()); t((*{a}).z.to_bits());")

SPEC = dict(
    src="src/melee/gr/grinishie1.c",
    types=["HSD_GObj", "Ground", "HSD_JObj", "StageCallbacks", "StageData", "grInishie1_YakumonoParam", "CollData", "Vec",
           "HSD_DObj", "grInishie1_Block", "GrJoint", "block_table_struct"],
    # views of the union GroundVars: 0 = the map's (inishie1), 1 = the hatena block's (inishie13, same layout as inishie12)
    unions={"GroundVars": ["inishie1", "inishie13"]},
    # xC4 overlays the bitfield byte inishie2.xC4_flags / map.xC4_b0 (melee_types keeps bitfields in GameCube
    # bit order, native C does not): never compared raw, b0 is compared by value in the driver
    skip=(".inishie1.xC4",),
    null_ret_ok=["HSD_GObj_SetupProc", "grZakoGenerator_801CAE04", "it_8026F7C8", "efSync_Spawn"],
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
        "Ground_801C3FA4": ("&JO[a1 & 7]", "&raw mut super::JO[(a1 & 7) as usize]"),
        "Ground_GetMapGObj": ("&G", "&raw mut super::G"),
        "Ground_GetStageGObj": ("((NOK && (a0 == NID || a0 == 0 || a0 == 3) && (fv() & 7) == 0) ? 0 : &GS[a0 & 7])",
                                "if super::NOK && (a0 == super::NID || a0 == 0 || a0 == 3) && fv() & 7 == 0 { core::ptr::null_mut() } "
                                "else { &raw mut super::GS[(a0 & 7) as usize] }"),
        "Ground_GetYakumonoParam": ("&YP", "&raw mut super::YP as *mut c_void"),
        "HSD_JObjGetDObj": ("(((fv() & 3) == 0) ? 0 : &DO[fv() & 7])",
                            "if fv() & 3 == 0 { core::ptr::null_mut() } else { &raw mut super::DO[(fv() & 7) as usize] }"),
        "HSD_MemAlloc": ("BL", "&raw mut super::BL as *mut c_void"),
        "grMaterial_801C8CFC": ("(((fv() & 3) == 0) ? 0 : &GS[4 + (fv() & 3)])",
                                "if fv() & 3 == 0 { core::ptr::null_mut() } else { &raw mut super::GS[4 + (fv() & 3) as usize] }"),
        "Stage_GetBlastZoneBottomOffset": ("rw()", "rw()"),
        "gm_8016AE80": ("((fv() & 1) ? -1 : (int) fv())", "if fv() & 1 != 0 { -1 } else { fv() as i32 }"),
        "gm_8016B238": ("((fv() & 1) ? 0 : (int) fv())", "if fv() & 1 != 0 { 0 } else { fv() as i32 }"),
        "mpJointFromLine": ("(18 + (int) (fv() % 6))", "18 + (fv() % 6) as i32"),
        "Ground_801C5A94": ("((fv() & 1) ? 0x29 : ((fv() & 1) ? 0x2B : (int) fv()))",
                            "if fv() & 1 != 0 { 0x29 } else if fv() & 1 != 0 { 0x2B } else { fv() as i32 }"),
    },
    effects={
        "lb_8000B1CC": (V3[0].format(a="a2"), V3[1].format(a="a2")),
        "it_8026B294": (V3[0].format(a="a1"), V3[1].format(a="a1")),
        "it_8026F7C8": (V3T[0].format(a="a0") + " " + V3T[0].format(a="a1"), V3T[1].format(a="a0") + " " + V3T[1].format(a="a1")),
        "Camera_RequestQuake": (V3T[0].format(a="a1"), V3T[1].format(a="a1")),
        "GetMatchTimer": ("*a0 = (int) (fv() % 40);", "*a0 = (fv() % 40) as i32;"),
    })

# every block of the map's table, as the game keeps them: hatena_gobj only while status is 1..3
BLOCKS_C = """
            for (i = 0; i < 19; i++) {
                BL[i].jobj2 = &JO[fv() & 7]; BL[i].status = (s16) (fv() % 5); BL[i].x2 = (s16) (fv() % 6);
                BL[i].hatena_gobj = (BL[i].status >= 1 && BL[i].status <= 3) ? &GS[1 + (i & 3)] : 0;
                BL[i].item_gobj = &GS[4 + (i & 3)];
                BL[i].xC = rw(); BL[i].x10 = rg();
            }
"""
BLOCKS_R = """
                for i in 0..19usize {
                    BL[i].jobj2 = &raw mut JO[(fv() & 7) as usize]; BL[i].status = (fv() % 5) as i16; BL[i].x2 = (fv() % 6) as i16;
                    BL[i].hatena_gobj = if BL[i].status >= 1 && BL[i].status <= 3 { &raw mut GS[1 + (i & 3)] } else { core::ptr::null_mut() };
                    BL[i].item_gobj = &raw mut GS[4 + (i & 3)];
                    BL[i].xC = rw(); BL[i].x10 = rg();
                }
"""

# 801FB3F0 / 801FAB68: the "every block becomes a hatena block" branch asserts that no block has one yet, and the
# respawn rolls need status-0 blocks that are not the last picked
SAFE_FB3F0_C = """
            for (j = 10; j < 19; j++) { BL[j].status = 0; BL[j].hatena_gobj = 0; }
            GP.u.inishie1.xCA = (s16) ((int) (fv() % 11) - 1); GP.u.inishie1.xCC = (s16) ((int) (fv() % 11) - 1);
            { int d8 = GP.u.inishie1.xD8 > 0 ? GP.u.inishie1.xD8 - 1 : GP.u.inishie1.xD8;
              if (d8 == 0 && ((GP.u.inishie1.xC6 == 1 && GP.u.inishie1.xC8 > 0 && GP.u.inishie1.xC8 < YP.unk14) ||
                              (GP.u.inishie1.xC8 == 1 && GP.u.inishie1.xC6 > 0 && GP.u.inishie1.xC6 < YP.unk14)))
                  for (j = 0; j < 19; j++) BL[j].hatena_gobj = 0; }
"""
SAFE_FB3F0_R = """
            for j in 10..19usize { BL[j].status = 0; BL[j].hatena_gobj = core::ptr::null_mut(); }
            GP.u.inishie1.xCA = ((fv() % 11) as i32 - 1) as i16; GP.u.inishie1.xCC = ((fv() % 11) as i32 - 1) as i16;
            { let d8 = if GP.u.inishie1.xD8 > 0 { GP.u.inishie1.xD8 - 1 } else { GP.u.inishie1.xD8 };
              if d8 == 0 && ((GP.u.inishie1.xC6 == 1 && GP.u.inishie1.xC8 > 0 && GP.u.inishie1.xC8 < YP.unk14) ||
                             (GP.u.inishie1.xC8 == 1 && GP.u.inishie1.xC6 > 0 && GP.u.inishie1.xC6 < YP.unk14)) {
                  for j in 0..19usize { BL[j].hatena_gobj = core::ptr::null_mut(); } } }
"""


def bop(name, pre_c, pre_r, c, r, **kw):
    return dict(name=name, pre_c=pre_c, pre_r=pre_r, c=c, r=r, **kw)


IXC = "u32 ix = fv() % 19;"
IXR = "let ix = fv() % 19;"

DRV = dict(
    state=[("G", "HSD_GObj", 1), ("GP", "Ground", 1), ("JO", "HSD_JObj", 16), ("GS", "HSD_GObj", 8),
           ("BL", "grInishie1_Block", 19), ("YP", "struct grInishie1_YakumonoParam", 1), ("DO", "HSD_DObj", 8)],
    decl_c="static Ground GG[8]; static Item IT; static HSD_GObj IG; static CollData CD; static Vec VW; static int NOK; static int NID;",
    decl_r="use m::grInishie1_YakumonoParam;" + chr(10) +
           "static mut GG: [Ground; 8] = unsafe { core::mem::zeroed() };" + chr(10) +
           "static mut IT: Item = unsafe { core::mem::zeroed() };" + chr(10) +
           "static mut IG: HSD_GObj = unsafe { core::mem::zeroed() };" + chr(10) +
           "static mut CD: CollData = unsafe { core::mem::zeroed() };" + chr(10) +
           "static mut VW: Vec = unsafe { core::mem::zeroed() };" + chr(10) +
           "static mut NOK: bool = false;" + chr(10) + "static mut NID: i32 = 0;",
    regs=[("GG", "sizeof GG", "&raw const GG", "core::mem::size_of_val(&GG)"),
          ("&IT", "sizeof IT", "&raw const IT", "core::mem::size_of_val(&IT)"),
          ("&IG", "sizeof IG", "&raw const IG", "core::mem::size_of_val(&IG)"),
          ("&CD", "sizeof CD", "&raw const CD", "core::mem::size_of_val(&CD)"),
          ("&VW", "sizeof VW", "&raw const VW", "core::mem::size_of_val(&VW)")],
    setup_c="""
            zero(&IT, sizeof IT); zero(&IG, sizeof IG); zero(&CD, sizeof CD); zero(&VW, sizeof VW);
            fill_CollData(&CD);
            G.user_data = &GP; G.hsd_obj = &JO[0]; GP.gobj = &G; IG.user_data = &IT;
            for (i = 0; i < 16; i++) JO[i].child = (i < 15) ? &JO[i + 1] : 0;
            for (i = 0; i < 8; i++) {
                GS[i].user_data = &GG[i]; GS[i].hsd_obj = &JO[8 + i];
                zero(&GG[i], sizeof GG[i]);
                USEL = (i == 3) ? 0 : 1;
                fill_Ground(&GG[i]);
                GG[i].gobj = &GS[i];
                GG[i].x8_callback = grInishie1_801FAAD4; GG[i].xC_callback = grInishie1_801FAAD8;
            }
            USEL = OPUSEL[op];
            GG[3].u.inishie1.block = BL;
            GP.x8_callback = grInishie1_801FAAD4; GP.xC_callback = grInishie1_801FAAD8;
            GP.u.inishie1.block = BL;
            GP.u.inishie1.x100 = &JO[6]; GP.u.inishie1.x104 = &JO[7]; GP.u.inishie1.x108 = &JO[4]; GP.u.inishie1.x10C = &JO[5];
            for (i = 0; i < 8; i++) DO[i].next = (i == 7 || (fv() & 3) == 0) ? 0 : &DO[i + 1];
            yakumono_param = &YP;
            YP.unk0 = rw(); YP.unk4 = rw(); YP.unk8 = rw(); YP.unkC = rw(); YP.unk10 = rw(); YP.unk4C = rw();
            NOK = 0; NID = 99;
""" + BLOCKS_C,
    setup_r="""
                IT = core::mem::zeroed(); IG = core::mem::zeroed(); CD = core::mem::zeroed(); VW = core::mem::zeroed();
                g::fill_CollData(&mut CD);
                G.user_data = &raw mut GP as *mut c_void; G.hsd_obj = &raw mut JO[0] as *mut c_void; GP.gobj = &raw mut G; IG.user_data = &raw mut IT as *mut c_void;
                for i in 0..16usize { JO[i].child = if i < 15 { &raw mut JO[i + 1] } else { core::ptr::null_mut() }; }
                for i in 0..8usize {
                    GS[i].user_data = &raw mut GG[i] as *mut c_void; GS[i].hsd_obj = &raw mut JO[8 + i] as *mut c_void;
                    GG[i] = core::mem::zeroed();
                    USEL = if i == 3 { 0 } else { 1 };
                    g::fill_Ground(&mut GG[i]);
                    GG[i].gobj = &raw mut GS[i];
                    GG[i].x8_callback = Some(m::grInishie1_801FAAD4); GG[i].xC_callback = Some(m::grInishie1_801FAAD8);
                }
                USEL = OPUSEL[op as usize];
                GG[3].u.inishie1.block = BL.as_mut_ptr();
                GP.x8_callback = Some(m::grInishie1_801FAAD4); GP.xC_callback = Some(m::grInishie1_801FAAD8);
                GP.u.inishie1.block = BL.as_mut_ptr();
                GP.u.inishie1.x100 = &raw mut JO[6]; GP.u.inishie1.x104 = &raw mut JO[7]; GP.u.inishie1.x108 = &raw mut JO[4]; GP.u.inishie1.x10C = &raw mut JO[5];
                for i in 0..8usize { DO[i].next = if i == 7 || fv() & 3 == 0 { core::ptr::null_mut() } else { &raw mut DO[i + 1] }; }
                m::yakumono_param = &raw mut YP;
                YP.unk0 = rw(); YP.unk4 = rw(); YP.unk8 = rw(); YP.unkC = rw(); YP.unk10 = rw(); YP.unk4C = rw();
                NOK = false; NID = 99;
""" + BLOCKS_R,
    post_c="""
            for (i = 0; i < 8; i++) { USEL = (i == 3) ? 0 : 1; hash_Ground(&GG[i]); }
            USEL = OPUSEL[op];
            HM(PI(yakumono_param)); HM((u32) GP.u.inishie2.xC4_flags.b0); HM((u32) GG[3].u.inishie2.xC4_flags.b0);
""",
    post_r="""
                for i in 0..8usize { USEL = if i == 3 { 0 } else { 1 }; g::hash_Ground(&GG[i]); }
                USEL = OPUSEL[op as usize];
                hm(pi(m::yakumono_param)); hm(GP.u.inishie2.xC4_flags.b0() as u32); hm(GG[3].u.inishie2.xC4_flags.b0() as u32);
""",
    ops=[
        dict(name="grInishie1_801FA908", c="grInishie1_801FA908((bool) fv());", r="m::grInishie1_801FA908(fv() as i32);"),
        dict(name="grInishie1_801FA90C", pre_c="NOK = 1; NID = 99; yakumono_param = 0;", pre_r="NOK = true; NID = 99; m::yakumono_param = core::ptr::null_mut();",
             c="grInishie1_801FA90C();", r="m::grInishie1_801FA90C();"),
        v0("grInishie1_801FA984"), v0("grInishie1_801FA988"), p0("grInishie1_801FA9AC"),
        dict(name="setupStageCallbacks", pre_c="int a = (int) (fv() % 4); NOK = 1; NID = a;", pre_r="let a = (fv() % 4) as i32; NOK = true; NID = a;",
             c="ret = PI(setupStageCallbacks(a));", r="ret = pi(m::setupStageCallbacks(a));"),
        vg("stageGObj0_OnInit", 0, "GS[0]"), pg("grInishie1_801FAACC"), vg("grInishie1_801FAAD4"), vg("grInishie1_801FAAD8"),
        vg("grInishie1_801FAADC"), pg("grInishie1_801FAB60"),
        dict(name="grInishie1_801FAB68", pre_c=SAFE_FB3F0_C, pre_r=SAFE_FB3F0_R, c="grInishie1_801FAB68(&G);", r="m::grInishie1_801FAB68(&raw mut G);"),
        vg("grInishie1_801FABB0"), vg("grInishie1_801FABB4", 0, "GS[1]"), pg("grInishie1_801FABF4"), vg("grInishie1_801FABFC"),
        vg("grInishie1_801FAC00"), vg("grInishie1_801FAC04", 1, "GS[2]"), pg("grInishie1_801FAC50"),
        dict(name="grInishie1_801FAC58", usel=1, pre_c="GG[2].u.inishie13.xC4 = (fv() & 1) ? &JO[fv() & 7] : 0;",
             pre_r="GG[2].u.inishie13.xC4 = if fv() & 1 != 0 { &raw mut JO[(fv() & 7) as usize] } else { core::ptr::null_mut() };",
             c="grInishie1_801FAC58(&GS[2]);", r="m::grInishie1_801FAC58(&raw mut GS[2]);"),
        vg("grInishie1_801FAD80"), vg("grInishie1_801FAD84"),
        dict(name="grInishie1_801FB0AC", pre_c=IXC, pre_r=IXR, c="grInishie1_801FB0AC(&G, ix);", r="m::grInishie1_801FB0AC(&raw mut G, ix);"),
        dict(name="grInishie1_801FB3F0", pre_c=SAFE_FB3F0_C, pre_r=SAFE_FB3F0_R, c="grInishie1_801FB3F0(&G);", r="m::grInishie1_801FB3F0(&raw mut G);"),
        dict(name="grInishie1_801FBA34", pre_c="int a = (int) (fv() & 7);", pre_r="let a = (fv() & 7) as usize;",
             c="grInishie1_801FBA34(&G, &JO[a]);", r="m::grInishie1_801FBA34(&raw mut G, &raw mut JO[a]);"),
        dict(name="grInishie1_801FBAA0", pre_c="int ix = (int) (fv() % 19); BL[ix].hatena_gobj = 0;",
             pre_r="let ix = (fv() % 19) as i32; BL[ix as usize].hatena_gobj = core::ptr::null_mut();",
             c="grInishie1_801FBAA0(&G, ix);", r="m::grInishie1_801FBAA0(&raw mut G, ix);"),
        dict(name="grInishie1_801FBC4C", pre_c=IXC, pre_r=IXR, c="grInishie1_801FBC4C(&G, ix);", r="m::grInishie1_801FBC4C(&raw mut G, ix);"),
        dict(name="grInishie1_801FBCEC", pre_c=IXC, pre_r=IXR, c="grInishie1_801FBCEC(&G, ix);", r="m::grInishie1_801FBCEC(&raw mut G, ix);"),
        dict(name="fn_801FBEB8", pre_c="int jid = grI1_803E49B8[fv() % 19].idx; int a = (int) fv(); u32 k = fv() % 4; float d = rw();",
             pre_r="let jid = m::grI1_803E49B8[(fv() % 19) as usize].idx as i32; let a = fv() as i32; let k = fv() % 4; let d = rw();",
             c="fn_801FBEB8(&GP, jid, &CD, a, k, d);", r="m::fn_801FBEB8(&raw mut GP as *mut c_void, jid, &raw mut CD, a, k, d);"),
        dict(name="fn_801FBF6C", pre_c="u32 k = fv() % 20; IT.xDD4_itemVar.it_266F.x4 = (k == 19) ? (u8*) &JO[8] : (u8*) BL[k].jobj2; float d = rw();",
             pre_r="let k = (fv() % 20) as usize; IT.xDD4_itemVar.it_266F.x4 = if k == 19 { &raw mut JO[8] as *mut u8 } else { BL[k].jobj2 as *mut u8 }; let d = rw();",
             c="fn_801FBF6C(&IG, &GP, &VW, &GS[0], d);", r="m::fn_801FBF6C(&raw mut IG, &raw mut GP, &raw mut VW, &raw mut GS[0], d);"),
        vg("grInishie1_801FC018"), vg("grInishie1_801FC110"), vg("grInishie1_801FC4A0"),
        dict(name="grInishie1_801FC664", pre_c="GP.u.inishie1.xEE = (s16) (fv() % 5);", pre_r="GP.u.inishie1.xEE = (fv() % 5) as i16;",
             c="grInishie1_801FC664(&G);", r="m::grInishie1_801FC664(&raw mut G);"),
        dict(name="fn_801FC9AC", pre_c="int jid = ((fv() & 3) == 0) ? 3 : 0x14 + (int) (fv() & 1); int a = (int) (fv() % 2000) - 1000; u32 k = fv() % 3; float d = rw();",
             pre_r="let jid = if fv() & 3 == 0 { 3 } else { 0x14 + (fv() & 1) as i32 }; let a = (fv() % 2000) as i32 - 1000; let k = fv() % 3; let d = rw();",
             c="fn_801FC9AC(&GP, jid, &CD, a, k, d);", r="m::fn_801FC9AC(&raw mut GP as *mut c_void, jid, &raw mut CD, a, k, d);"),
        dict(name="grInishie1_801FCAAC",
             pre_c="stage_info.grkind = (fv() & 1) ? Gr_Kind_Inishie1 : 3; int lid = ((fv() & 3) == 0) ? -1 : (int) (fv() % 50);",
             pre_r="m::ext::stage_info.grkind = if fv() & 1 != 0 { Gr_Kind_Inishie1 } else { 3 }; let lid = if fv() & 3 == 0 { -1 } else { (fv() % 50) as i32 };",
             c="ret = (u32) grInishie1_801FCAAC(lid);", r="ret = m::grInishie1_801FCAAC(lid) as u32;"),
        dict(name="grInishie1_801FCB10", pre_c="GP.u.inishie2.xC4_flags.b0 = fv() & 1;", pre_r="GP.u.inishie2.xC4_flags.set_b0((fv() & 1) as u8);",
             c="grInishie1_801FCB10(&G);", r="m::grInishie1_801FCB10(&raw mut G);"),
        dict(name="grInishie1_801FCBB0", pre_c="int a = (int) fv();", pre_r="let a = fv() as i32;",
             c="ret = PI(grInishie1_801FCBB0(a));", r="ret = pi(m::grInishie1_801FCBB0(a));"),
        dict(name="grInishie1_801FCBB8", pre_c="int a = (int) fv();", pre_r="let a = fv() as i32;",
             c="ret = (u32) grInishie1_801FCBB8(&VW, a, &JO[2]);", r="ret = m::grInishie1_801FCBB8(&raw mut VW, a, &raw mut JO[2]) as u32;"),
        # the static inline helpers of the TU
        dict(name="fabsf_inline", pre_c="float a = rw();", pre_r="let a = rw();", c="ret = B(fabsf_inline(a));", r="ret = m::fabsf_inline(a).to_bits();"),
        dict(name="get_block_id", pre_c="int a = grI1_803E49B8[fv() % 19].idx;", pre_r="let a = m::grI1_803E49B8[(fv() % 19) as usize].idx as i32;",
             c="ret = (u32) get_block_id(a);", r="ret = m::get_block_id(a) as u32;"),
        dict(name="grInishie1_SetRespawnTimer", pre_c=IXC + " int lo = (int) (fv() % 20) - 5; int hi = (int) (fv() % 20) - 5;",
             pre_r=IXR + " let lo = (fv() % 20) as i32 - 5; let hi = (fv() % 20) as i32 - 5;",
             c="grInishie1_SetRespawnTimer(&GP, ix, lo, hi);", r="m::grInishie1_SetRespawnTimer(&raw mut GP, ix, lo, hi);"),
        dict(name="grInishie1_ShowBlock", pre_c="int a = (int) (fv() & 7);", pre_r="let a = (fv() & 7) as usize;",
             c="grInishie1_ShowBlock(&JO[a]);", r="m::grInishie1_ShowBlock(&raw mut JO[a]);"),
        dict(name="grInishie1_801FB3F0_UpdateY", pre_c=IXC, pre_r=IXR,
             c="grInishie1_801FB3F0_UpdateY(&BL[ix]);", r="m::grInishie1_801FB3F0_UpdateY(&raw mut BL[ix as usize]);"),
        dict(name="grInishie1_801FB3F0_IsFar", pre_c="int a = (int) (fv() & 7);", pre_r="let a = (fv() & 7) as usize;",
             c="ret = (u32) grInishie1_801FB3F0_IsFar(&JO[a], &VW);", r="ret = m::grInishie1_801FB3F0_IsFar(&raw mut JO[a], &raw mut VW) as u32;"),
        dict(name="grInishie1_UpdateDObjFlags", pre_c="int a = (int) (fv() & 7); bool hid = (bool) (fv() & 1); HSD_DObj* w = 0;",
             pre_r="let a = (fv() & 7) as usize; let hid = fv() & 1 != 0; let mut w: *mut HSD_DObj = core::ptr::null_mut();",
             c="grInishie1_UpdateDObjFlags(&JO[a], &w, hid); ret = PI(w);", r="m::grInishie1_UpdateDObjFlags(&raw mut JO[a], &mut w, hid); ret = pi(w);"),
        dict(name="GET_ITEM2", c="ret = PI(GET_ITEM2(&IG));", r="ret = pi(m::GET_ITEM2(&raw mut IG));"),
        dict(name="tables", cases=1,
             c="for (j = 0; j < 4; j++) hash_StageCallbacks(&grI1_StageCallbacks[j]); hash_StageData(&grI1_StageData); "
               "HM(SH(grI1_StageData.data1)); HM(PI(grI1_StageData.callbacks)); HM(PI(grI1_StageData.joints)); "
               "for (j = 0; j < 7; j++) hash_GrJoint(&grI1_803E48C8[j]); "
               "for (j = 0; j < 19; j++) { HM((u32) grI1_803E49B8[j].idx); HM((u32) grI1_803E49B8[j].jobj_idx); } "
               "HM(B(grI1_803B8268.x)); HM(B(grI1_803B8268.y)); HM(B(grI1_803B8268.z));",
             r="for j in 0..4 { g::hash_StageCallbacks(&m::grI1_StageCallbacks[j]); } g::hash_StageData(&m::grI1_StageData); "
               "hm(sh(m::grI1_StageData.data1 as *const u8)); hm(pi(m::grI1_StageData.callbacks)); hm(pi(m::grI1_StageData.joints)); "
               "for j in 0..7 { g::hash_GrJoint(&m::grI1_803E48C8[j]); } "
               "for j in 0..19 { hm(m::grI1_803E49B8[j].idx as i32 as u32); hm(m::grI1_803E49B8[j].jobj_idx as i32 as u32); } "
               "hm(m::grI1_803B8268.x.to_bits()); hm(m::grI1_803B8268.y.to_bits()); hm(m::grI1_803B8268.z.to_bits());"),
    ],
)
