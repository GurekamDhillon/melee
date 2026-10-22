"""Oracle spec + driver of src/melee/gr/grheal.c (see ../gen_tu.py, ../mk_tu.py)."""
import os, sys
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
from drvlib import *

VEC3_OUT = ("a{n}->x = rw(); a{n}->y = rw(); a{n}->z = rw();", "(*a{n}).x = rw(); (*a{n}).y = rw(); (*a{n}).z = rw();")

SPEC = dict(
    src="src/melee/gr/grheal.c",
    types=["HSD_GObj", "Ground", "HSD_JObj", "StageCallbacks", "StageData", "grHeal_UnkData", "CollData", "Vec",
           "UnkAllstarData"],
    unions={"GroundVars": ["unk", "flatzone2"]},
    null_ret_ok=["HSD_GObj_SetupProc", "grZakoGenerator_801CAE04", "HSD_JObjReparent", "it_8026BE84", "it_802F2094"],
    ret={
        "Ground_801C3FA4": ("&JO[a1 & 7]", "&raw mut super::JO[(a1 & 7) as usize]"),
        "Ground_GetMapGObj": ("(((fv() & 3) == 0) ? 0 : &GM[a0 & 3])",
                              "if fv() & 3 == 0 { core::ptr::null_mut() } else { &raw mut super::GM[(a0 & 3) as usize] }"),
        "Ground_GetStageGObj": ("((NOK && (a0 == NID || a0 == 0 || a0 == 1 || a0 == 3) && (fv() & 7) == 0) ? 0 : &GS[a0 & 7])",
                                "if super::NOK && (a0 == super::NID || a0 == 0 || a0 == 1 || a0 == 3) && fv() & 7 == 0 { core::ptr::null_mut() } else { &raw mut super::GS[(a0 & 7) as usize] }"),
        "Ground_GetYakumonoParam": ("&YP[0]", "&raw mut super::YP[0] as *mut c_void"),
        "Ground_801C5840": ("(((fv() & 3) == 0) ? -1 : (int) (fv() % 100))", "if fv() & 3 == 0 { -1 } else { (fv() % 100) as i32 }"),
        "Stage_GetBlastZoneTopOffset": ("rw()", "rw()"),
    },
    effects={
        "Ground_801C2D24": (VEC3_OUT[0].format(n=1), VEC3_OUT[1].format(n=1)),
        "lb_8000B1CC": (VEC3_OUT[0].format(n=2), VEC3_OUT[1].format(n=2)),
        "it_802F2094": ("T(B(a1->x)); T(B(a1->y)); T(B(a1->z));", "t((*a1).x.to_bits()); t((*a1).y.to_bits()); t((*a1).z.to_bits());"),
        "it_8026BE84": ("HM(PI(a0->x0)); HM(PI(a0->x4)); HM(B(a0->x8_vec.x)); HM(B(a0->x8_vec.y)); HM(B(a0->x8_vec.z)); "
                        "HM((u32) a0->x14); HM((u32) a0->x18); HM((u32) a0->x1C.b0);",
                        "hm(pi((*a0).x0)); hm(pi((*a0).x4)); hm((*a0).x8_vec.x.to_bits()); hm((*a0).x8_vec.y.to_bits()); "
                        "hm((*a0).x8_vec.z.to_bits()); hm((*a0).x14 as u32); hm((*a0).x18 as u32); hm((*a0).x1C.anon_0.b0() as u32);"),
    })

DRV = dict(
    state=[("G", "HSD_GObj", 1), ("GP", "Ground", 1), ("JO", "HSD_JObj", 8), ("GS", "HSD_GObj", 8),
           ("GM", "HSD_GObj", 4), ("YP", "grHeal_UnkData", 2), ("CD", "CollData", 1), ("VW", "Vec", 1),
           ("AL", "UnkAllstarData", 1, "m::ext::gm_80473A18")],
    decl_c="static int NOK; static int NID;",
    decl_r="use m::grHeal_UnkData;" + chr(10) + "static mut NOK: bool = false;" + chr(10) + "static mut NID: i32 = 0;",
    setup_c=GROUND_SETUP_C + """
            for (i = 0; i < 4; i++) { GM[i].user_data = &GP; GM[i].hsd_obj = &JO[i]; }
            for (i = 0; i < 7; i++) JO[i].child = &JO[i + 1];
            grHeal_804D6AF0[0] = &YP[0];
            YP[0].x0 = (int) (fv() % 5); YP[0].x4 = 1 + (int) (fv() % 6);
            AL._94[1] = (u8) (fv() % 4);
            NOK = 0;
            GP.x8_callback = stageGObj0_GObjProc; GP.xC_callback = stageGObj2_GObjProc;
""",
    setup_r=GROUND_SETUP_R + """
                for i in 0..4 { GM[i].user_data = &raw mut GP as *mut c_void; GM[i].hsd_obj = &raw mut JO[i] as *mut c_void; }
                for i in 0..7 { JO[i].child = &raw mut JO[i + 1]; }
                m::grHeal_804D6AF0[0] = &raw mut YP[0];
                YP[0].x0 = (fv() % 5) as i32; YP[0].x4 = 1 + (fv() % 6) as i32;
                AL._94[1] = (fv() % 4) as u8;
                NOK = false;
                GP.x8_callback = Some(m::stageGObj0_GObjProc); GP.xC_callback = Some(m::stageGObj2_GObjProc);
""",
    post_c="            HM(PI(grHeal_804D6AF0[0])); HM(PI(grHeal_804D6AF0[1]));",
    post_r="                hm(pi(m::grHeal_804D6AF0[0])); hm(pi(m::grHeal_804D6AF0[1]));",
    ops=[
        dict(name="grHeal_8021EF38", c="grHeal_8021EF38((bool) fv());", r="m::grHeal_8021EF38(fv() as i32);"),
        dict(name="grHeal_8021EF3C", usel=1, pre_c="NOK = 1; NID = 99; grHeal_804D6AF0[0] = 0;", pre_r="NOK = true; NID = 99; m::grHeal_804D6AF0[0] = core::ptr::null_mut();",
             c="grHeal_8021EF3C();", r="m::grHeal_8021EF3C();"),
        v0("grHeal_8021EFBC"), v0("grHeal_8021EFC0"), p0("grHeal_8021EFE4"),
        dict(name="grHeal_8021EFEC", usel=1, pre_c="u32 a = fv() % 5; NOK = 1; NID = (int) a;", pre_r="let a = fv() % 5; NOK = true; NID = a as i32;",
             c="ret = PI(grHeal_8021EFEC(a));", r="ret = pi(m::grHeal_8021EFEC(a));"),
        vg("stageGObj0_OnInit", 1), pg("stageGObj0_Callback1"), vg("stageGObj0_GObjProc"), vg("stageGObj0_Callback3"),
        vg("stageGObj1_OnInit", 1), pg("stageGObj1_Callback1"), p0("fn_8021F424"), vg("stageGObj1_GObjProc", 1),
        vg("stageGObj1_Callback3"),
        dict(name="onJointCollision", usel=1, pre_c="int ja = (int) fv(); int jb = (int) fv(); u32 jk = fv() % 4; float jd = rw();",
             pre_r="let ja = fv() as i32; let jb = fv() as i32; let jk = fv() % 4; let jd = rw();",
             c="onJointCollision(&GP, ja, &CD, jb, jk, jd);", r="m::onJointCollision(&raw mut GP as *mut c_void, ja, &raw mut CD, jb, jk, jd);"),
        dict(name="grHeal_8021F4E8", usel=1, pre_c="int a = (int) fv();", pre_r="let a = fv() as i32;",
             c="grHeal_8021F4E8(a, &JO[3]);", r="m::grHeal_8021F4E8(a, &raw mut JO[3]);"),
        vg("stageGObj2_OnInit"), pg("stageGObj2_Callback1"), vg("stageGObj2_GObjProc"), vg("stageGObj2_Callback3"),
        vg("stageGObj3_OnInit"), pg("stageGObj3_Callback1"), vg("stageGObj3_GObjProc"), vg("stageGObj3_Callback3"),
        dict(name="grHeal_8021F628", usel=1, pre_c="int a = (int) fv();", pre_r="let a = fv() as i32;",
             c="grHeal_8021F628(a, &JO[4]);", r="m::grHeal_8021F628(a, &raw mut JO[4]);"),
        vg("stageGObj4_OnInit"), pg("stageGObj4_Callback1"), vg("stageGObj4_GObjProc"), vg("stageGObj4_Callback3"),
        dict(name="grHeal_8021F70C", cases=300, pre_c="int a = (int) (fv() % 45) - 2;", pre_r="let a = (fv() % 45) as i32 - 2;",
             c="ret = (u32) grHeal_8021F70C(a);", r="ret = m::grHeal_8021F70C(a) as u32;"),
        dict(name="grHeal_8021F79C", usel=1, pre_c="int a = (int) fv(); int b = (int) fv(); int d = (int) fv();",
             pre_r="let a = fv() as i32; let b = fv() as i32; let d = fv() as i32;",
             c="grHeal_8021F79C(a, b, d);", r="m::grHeal_8021F79C(a, b, d);"),
        dict(name="grHeal_8021F830", pre_c="int a = (int) fv();", pre_r="let a = fv() as i32;",
             c="ret = PI(grHeal_8021F830(a));", r="ret = pi(m::grHeal_8021F830(a));"),
        dict(name="grHeal_8021F838", pre_c="int a = (int) fv();", pre_r="let a = fv() as i32;",
             c="ret = (u32) grHeal_8021F838(&VW, a, &JO[2]);", r="ret = m::grHeal_8021F838(&raw mut VW, a, &raw mut JO[2]) as u32;"),
        dict(name="tables", cases=1,
             c="for (j = 0; j < 5; j++) hash_StageCallbacks(&stage_callbacks[j]); hash_StageData(&grHeal_StageData); "
               "HM(SH(grHeal_StageData.data1)); HM(PI(grHeal_StageData.callbacks)); "
               "for (j = 0; j < 4; j++) HM((u32) grHeal_804D49D8[j]); for (j = 0; j < 26; j++) { HM((u32) grHeal_803E83B8[j]); HM((u32) frame_to_character_id[j]); } "
               "HM(B(grHeal_803B84A8.x)); HM(B(grHeal_803B84A8.y)); HM(B(grHeal_803B84A8.z)); HM((u32) char_id_count);",
             r="for j in 0..5 { g::hash_StageCallbacks(&m::stage_callbacks[j]); } g::hash_StageData(&m::grHeal_StageData); "
               "hm(sh(m::grHeal_StageData.data1 as *const u8)); hm(pi(m::grHeal_StageData.callbacks)); "
               "for j in 0..4 { hm(m::grHeal_804D49D8[j] as i32 as u32); } for j in 0..26 { hm(m::grHeal_803E83B8[j] as i32 as u32); hm(m::frame_to_character_id[j] as u32); } "
               "hm(m::grHeal_803B84A8.x.to_bits()); hm(m::grHeal_803B84A8.y.to_bits()); hm(m::grHeal_803B84A8.z.to_bits()); hm(m::char_id_count as u32);"),
    ],
)
