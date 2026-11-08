"""Oracle spec + driver of src/melee/gr/grgreatbay.c (see ../gen_tu.py, ../mk_tu.py)."""
import os, sys
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
from drvlib import *

V3 = ("{a}->x = rw(); {a}->y = rw(); {a}->z = rw();", "(*{a}).x = rw(); (*{a}).y = rw(); (*{a}).z = rw();")
V3T = ("T(B({a}->x)); T(B({a}->y)); T(B({a}->z));", "t((*{a}).x.to_bits()); t((*{a}).y.to_bits()); t((*{a}).z.to_bits());")

SPEC = dict(
    src="src/melee/gr/grgreatbay.c",
    types=["HSD_GObj", "Ground", "HSD_JObj", "StageCallbacks", "StageData", "CollData", "Vec", "grGb_StageAttr", "GrJoint",
           "HSD_psAppSRT", "HSD_Generator", "SpawnItem", "StageInfo"],
    # views of GroundVars: 0 = the turtle's, 1 = the moon's, 2 = the floating platform's (main stage), 3 = Tingle's balloon
    unions={"GroundVars": ["greatbay", "greatbay2", "greatbay3", "greatbay4"]},
    null_ret_ok=["HSD_GObj_SetupProc", "grZakoGenerator_801CAE04", "Item_80268B18"],
    ret={
        "Ground_801C3FA4": ("&JO[a1 & 7]", "&raw mut super::JO[(a1 & 7) as usize]"),
        "Ground_GetMapGObj": ("(((fv() & 7) == 0) ? 0 : &GS[a0 % 11])", "if fv() & 7 == 0 { core::ptr::null_mut() } else { &raw mut super::GS[(a0 % 11) as usize] }"),
        "Ground_GetStageGObj": ("((NOK && a0 == NID && (fv() & 7) == 0) ? 0 : &GS[a0 % 11])",
                                "if super::NOK && a0 == super::NID && fv() & 7 == 0 { core::ptr::null_mut() } else { &raw mut super::GS[(a0 % 11) as usize] }"),
        "Ground_GetYakumonoParam": ("&YP", "&raw mut super::YP as *mut c_void"),
        "grLib_801C96F8": ("(((fv() & 3) == 0) ? 0 : &GEN)", "if fv() & 3 == 0 { core::ptr::null_mut() } else { &raw mut super::GEN }"),
        "it_802ECA70": ("&IG", "&raw mut super::IG"),
        "Stage_GetBlastZoneBottomOffset": ("rw()", "rw()"),
        "ftLib_80086B80": ("rw()", "rw()"),
        "Ground_801C3F20": ("rw()", "rw()"),
    },
    effects={
        "lb_8000B1CC": (V3[0].format(a="a2"), V3[1].format(a="a2")),
        "ftLib_GetPos": (V3[0].format(a="a1"), V3[1].format(a="a1")),
        "ftLib_GetPrevPos": (V3[0].format(a="a1"), V3[1].format(a="a1")),
        "grLib_801C96F8": (V3T[0].format(a="a2"), V3T[1].format(a="a2")),
        "it_802ECCA4": ("*a1 = (int) (fv() % 14); a2->x = rw(); a2->y = rw(); a2->z = rw();",
                        "*a1 = (fv() % 14) as i32; (*a2).x = rw(); (*a2).y = rw(); (*a2).z = rw();"),
        "Item_80268B18": ("T(PI(a0->x0_parent_gobj)); T(PI(a0->x4_parent_gobj2)); T((u32) a0->kind); T((u32) a0->hold_kind); T((u32) a0->x10); "
                          "T(B(a0->pos.x)); T(B(a0->pos.y)); T(B(a0->pos.z)); T(B(a0->prev_pos.x)); T(B(a0->prev_pos.y)); T(B(a0->prev_pos.z)); "
                          "T(B(a0->vel.x)); T(B(a0->vel.y)); T(B(a0->vel.z)); T(B(a0->facing_dir)); T((u32) a0->x3C_damage); T((u32) a0->x3E); T((u32) a0->x40); "
                          "T((u32) a0->x44_flag.byte); T((u32) a0->x45_flag.byte); T((u32) a0->x46_flag.byte); T((u32) a0->x47_flag.byte); T((u32) a0->x48_ground_or_air);",
                          "t(pi((*a0).x0_parent_gobj)); t(pi((*a0).x4_parent_gobj2)); t((*a0).kind as u32); t((*a0).hold_kind as u32); t((*a0).x10 as u32); "
                          "t((*a0).pos.x.to_bits()); t((*a0).pos.y.to_bits()); t((*a0).pos.z.to_bits()); t((*a0).prev_pos.x.to_bits()); t((*a0).prev_pos.y.to_bits()); t((*a0).prev_pos.z.to_bits()); "
                          "t((*a0).vel.x.to_bits()); t((*a0).vel.y.to_bits()); t((*a0).vel.z.to_bits()); t((*a0).facing_dir.to_bits()); t((*a0).x3C_damage as i32 as u32); t((*a0).x3E as i32 as u32); t((*a0).x40 as u32); "
                          "t((*a0).x44_flag.byte as u32); t((*a0).x45_flag.byte as u32); t((*a0).x46_flag.byte as u32); t((*a0).x47_flag.byte as u32); t((*a0).x48_ground_or_air as u32);"),
    })

# the view of each stage gobj's Ground (index into the union list above)
VIEWS = {1: 0, 2: 2, 9: 1, 10: 3}


def gs(name, k, usel=None, pre_c=None, pre_r=None):
    d = vg(name, VIEWS.get(k, 0) if usel is None else usel, f"GS[{k}]")
    if pre_c:
        d["pre_c"], d["pre_r"] = pre_c, pre_r
    return d


def pgs(name, k):
    return pg(name, VIEWS.get(k, 0), f"GS[{k}]")


VIEWC = "(i == 1 ? 0 : i == 2 ? 2 : i == 9 ? 1 : i == 10 ? 3 : 0)"
VIEWR = "(match i { 1 => 0, 2 => 2, 9 => 1, 10 => 3, _ => 0 })"

SETUP_C = """
            for (i = 0; i < 24; i++) { JO[i].child = &JO[(i + 1) % 24]; JO[i].flags &= ~0x20000u; }
            for (i = 0; i < 11; i++) {
                GS[i].user_data = &GG[i]; GS[i].hsd_obj = &JO[8 + i];
                zero(&GG[i], sizeof GG[i]);
                USEL = @VIEW@;
                fill_Ground(&GG[i]);
                GG[i].gobj = &GS[i];
                GG[i].x8_callback = grGreatBay_801F4438; GG[i].xC_callback = grGreatBay_801F443C;
            }
            USEL = OPUSEL[op];
            GEN.appsrt = &SRT;
            grGb_804D69E0.x0 = &YP;
            YP.kame_dir_prob[0] = (s16) (1 + fv() % 14); YP.kame_dir_prob[1] = (s16) (fv() % 15);
            YP.kame_dir_prob[2] = (s16) (fv() % 15); YP.kame_dir_prob[3] = (s16) (fv() % 15);
            for (i = 0; i < 10; i++) { YP.items[i].kind = ((fv() & 3) == 0) ? -1 : (s16) (fv() % 25); YP.items[i].weight = (s16) (fv() % 10); }
            YP.kame_x_fb_offset_a = rw(); YP.kame_x_fb_offset_b = rw(); YP.kame_x_lr_offset_a = rw(); YP.kame_x_lr_offset_b = rw();
            GG[2].u.greatbay3.jobj = (fv() & 3) == 0 ? 0 : &JO[fv() & 7];
            GG[9].u.greatbay2.gobjs[0] = (fv() & 3) == 0 ? 0 : &GS[5]; GG[9].u.greatbay2.gobjs[1] = (fv() & 3) == 0 ? 0 : &GS[6];
            GG[9].u.greatbay2.gobjs[2] = (fv() & 3) == 0 ? 0 : &GS[7]; GG[9].u.greatbay2.gobjs[3] = (fv() & 3) == 0 ? 0 : &GS[8];
            GG[10].u.greatbay4.xF0 = &IG;
            GG[1].u.greatbay.xC8 = (fv() & 1) ? &GEN : 0;
            NOK = 0; NID = 99;
"""

SETUP_R = """
                for i in 0..24usize { JO[i].child = &raw mut JO[(i + 1) % 24]; JO[i].flags &= !0x20000u32; }
                for i in 0..11usize {
                    GS[i].user_data = &raw mut GG[i] as *mut c_void; GS[i].hsd_obj = &raw mut JO[8 + i] as *mut c_void;
                    GG[i] = core::mem::zeroed();
                    USEL = @VIEW@;
                    g::fill_Ground(&mut GG[i]);
                    GG[i].gobj = &raw mut GS[i];
                    GG[i].x8_callback = Some(m::grGreatBay_801F4438); GG[i].xC_callback = Some(m::grGreatBay_801F443C);
                }
                USEL = OPUSEL[op as usize];
                GEN.appsrt = &raw mut SRT;
                m::grGb_804D69E0.x0 = &raw mut YP;
                YP.kame_dir_prob[0] = (1 + fv() % 14) as i16; YP.kame_dir_prob[1] = (fv() % 15) as i16;
                YP.kame_dir_prob[2] = (fv() % 15) as i16; YP.kame_dir_prob[3] = (fv() % 15) as i16;
                for i in 0..10usize { YP.items[i].kind = if fv() & 3 == 0 { -1 } else { (fv() % 25) as i16 }; YP.items[i].weight = (fv() % 10) as i16; }
                YP.kame_x_fb_offset_a = rw(); YP.kame_x_fb_offset_b = rw(); YP.kame_x_lr_offset_a = rw(); YP.kame_x_lr_offset_b = rw();
                GG[2].u.greatbay3.jobj = if fv() & 3 == 0 { core::ptr::null_mut() } else { &raw mut JO[(fv() & 7) as usize] };
                GG[9].u.greatbay2.gobjs[0] = if fv() & 3 == 0 { core::ptr::null_mut() } else { &raw mut GS[5] }; GG[9].u.greatbay2.gobjs[1] = if fv() & 3 == 0 { core::ptr::null_mut() } else { &raw mut GS[6] };
                GG[9].u.greatbay2.gobjs[2] = if fv() & 3 == 0 { core::ptr::null_mut() } else { &raw mut GS[7] }; GG[9].u.greatbay2.gobjs[3] = if fv() & 3 == 0 { core::ptr::null_mut() } else { &raw mut GS[8] };
                GG[10].u.greatbay4.xF0 = &raw mut IG;
                GG[1].u.greatbay.xC8 = if fv() & 1 != 0 { &raw mut GEN } else { core::ptr::null_mut() };
                NOK = false; NID = 99;
"""

POST_C = """
            for (i = 0; i < 11; i++) { USEL = @VIEW@; hash_Ground(&GG[i]); }
            USEL = OPUSEL[op];
            HM(PI(grGb_804D69E0.x0));
"""

POST_R = """
                for i in 0..11usize { USEL = @VIEW@; g::hash_Ground(&GG[i]); }
                USEL = OPUSEL[op as usize];
                hm(pi(m::grGb_804D69E0.x0));
"""

DRV = dict(
    state=[("GS", "HSD_GObj", 11), ("JO", "HSD_JObj", 24), ("YP", "grGb_StageAttr", 1), ("GEN", "HSD_Generator", 1),
           ("SRT", "HSD_psAppSRT", 1), ("IG", "HSD_GObj", 1), ("CD", "CollData", 1), ("VW", "Vec", 2),
           ("SI", "StageInfo", 1, "m::ext::stage_info")],
    decl_c="static Ground GG[11]; static int NOK; static int NID;",
    decl_r=chr(10).join(["use m::grGb_StageAttr;", "static mut GG: [Ground; 11] = unsafe { core::mem::zeroed() };",
                         "static mut NOK: bool = false;", "static mut NID: i32 = 0;"]),
    regs=[("GG", "sizeof GG", "&raw const GG", "core::mem::size_of_val(&GG)")],
    setup_c=SETUP_C.replace("@VIEW@", VIEWC),
    setup_r=SETUP_R.replace("@VIEW@", VIEWR),
    post_c=POST_C.replace("@VIEW@", VIEWC),
    post_r=POST_R.replace("@VIEW@", VIEWR),
    ops=[
        dict(name="grGreatBay_801F423C", c="grGreatBay_801F423C((bool) fv());", r="m::grGreatBay_801F423C(fv() as i32);"),
        dict(name="grGreatBay_801F4240", pre_c="grGb_804D69E0.x0 = 0;", pre_r="m::grGb_804D69E0.x0 = core::ptr::null_mut();",
             c="grGreatBay_801F4240();", r="m::grGreatBay_801F4240();"),
        v0("grGreatBay_801F42D0"), v0("grGreatBay_801F42D4"), p0("grGreatBay_801F42F8"),
        dict(name="grGreatBay_801F4300", pre_c="int a = (int) (fv() % 11); NOK = 1; NID = a;", pre_r="let a = (fv() % 11) as i32; NOK = true; NID = a;",
             c="ret = PI(grGreatBay_801F4300(a));", r="ret = pi(m::grGreatBay_801F4300(a));"),
        gs("stageGObj0_OnInit", 0), pgs("grGreatBay_801F4430", 0), gs("grGreatBay_801F4438", 0), gs("grGreatBay_801F443C", 0),
        gs("grGreatBay_801F4440", 3), pgs("grGreatBay_801F44A0", 3), gs("grGreatBay_801F44A8", 3), gs("grGreatBay_801F44AC", 3),
        gs("grGreatBay_801F44B0", 4), pgs("grGreatBay_801F4510", 4), gs("grGreatBay_801F4518", 4), gs("grGreatBay_801F451C", 4),
        gs("fn_801F4520", 2), gs("grGreatBay_801F454C", 2), pgs("grGreatBay_801F4650", 2), gs("grGreatBay_801F4658", 2), gs("grGreatBay_801F4690", 2),
        gs("grGreatBay_801F4694", 1), pgs("grGreatBay_801F4994", 1),
        gs("grGreatBay_801F499C", 1,
           pre_c="GG[1].u.greatbay.xC4 = (u8) (fv() % 5); YP.kame_dir_prob[1] = 0; "
                 "{ u32 v = fv() % 4; if (v == 2) v = 3; GG[1].u.greatbay.xC5.b0123456 = v; } GG[1].u.greatbay.xC5.b7 = fv() & 1; "
                 "GG[1].u.greatbay.xC6 = (s16) ((int) (fv() % 200) - 20);",
           pre_r="GG[1].u.greatbay.xC4 = (fv() % 5) as u8; YP.kame_dir_prob[1] = 0; "
                 "{ let mut v = fv() % 4; if v == 2 { v = 3; } GG[1].u.greatbay.xC5.set_b0123456(v as u8); } GG[1].u.greatbay.xC5.set_b7((fv() & 1) as u8); "
                 "GG[1].u.greatbay.xC6 = ((fv() % 200) as i32 - 20) as i16;"),
        gs("grGreatBay_801F545C", 1),
        gs("grGreatBay_801F5460", 10), pgs("grGreatBay_801F55F8", 10),
        gs("grGreatBay_801F5600", 10,
           pre_c="GG[10].u.greatbay4.xD4 = (int) (fv() & 1); GG[10].u.greatbay4.xD8 = (int) (fv() & 3); GG[10].u.greatbay4.xDC = (int) (fv() & 3);",
           pre_r="GG[10].u.greatbay4.xD4 = (fv() & 1) as i32; GG[10].u.greatbay4.xD8 = (fv() & 3) as i32; GG[10].u.greatbay4.xDC = (fv() & 3) as i32;"),
        dict(name="grGreatBay_801F5914", usel=3, pre_c="int ja = (int) fv(); int jb = (int) (fv() % 20000) - 10000; u32 jk = fv() % 3; float jd = rw();",
             pre_r="let ja = fv() as i32; let jb = (fv() % 20000) as i32 - 10000; let jk = fv() % 3; let jd = rw();",
             c="grGreatBay_801F5914(&GG[10], ja, &CD, jb, jk, jd);", r="m::grGreatBay_801F5914(&raw mut GG[10] as *mut c_void, ja, &raw mut CD, jb, jk, jd);"),
        gs("grGreatBay_801F5988", 10),
        gs("grGreatBay_801F598C", 5), pgs("grGreatBay_801F59F0", 5), gs("grGreatBay_801F59F8", 5), gs("grGreatBay_801F59FC", 5),
        gs("grGreatBay_801F5A00", 9), pgs("grGreatBay_801F5AF0", 9),
        gs("grGreatBay_801F5AF8", 9,
           pre_c="GG[9].u.greatbay2.x10 = (s16) ((int) (fv() % 5) - 1); GG[9].u.greatbay2.x12.b0 = fv() & 1; GG[9].u.greatbay2.x12.b1 = fv() & 1;",
           pre_r="GG[9].u.greatbay2.x10 = ((fv() % 5) as i32 - 1) as i16; GG[9].u.greatbay2.x12.set_b0((fv() & 1) as u8); GG[9].u.greatbay2.x12.set_b1((fv() & 1) as u8);"),
        gs("grGreatBay_801F5D48", 9),
        gs("grGreatBay_801F5D4C", 2), gs("grGreatBay_801F5E28", 2),
        dict(name="grGreatBay_801F60C4", usel=2, pre_c="int ja = (int) fv(); int jb = (int) (fv() % 2000) - 1000; u32 jk = fv() % 3; float jd = rw();",
             pre_r="let ja = fv() as i32; let jb = (fv() % 2000) as i32 - 1000; let jk = fv() % 3; let jd = rw();",
             c="grGreatBay_801F60C4(&GG[2], ja, &CD, jb, jk, jd);", r="m::grGreatBay_801F60C4(&raw mut GG[2] as *mut c_void, ja, &raw mut CD, jb, jk, jd);"),
        dict(name="grGreatBay_801F62F8", cases=300, pre_c="int a = (int) (fv() % 5);", pre_r="let a = (fv() % 5) as i32;",
             c="ret = (u32) grGreatBay_801F62F8(a);", r="ret = m::grGreatBay_801F62F8(a) as u32;"),
        dict(name="grGreatBay_801F63F4", usel=0, c="ret = (u32) grGreatBay_801F63F4(&GS[1]);", r="ret = m::grGreatBay_801F63F4(&raw mut GS[1]) as u32;"),
        gs("grGreatBay_801F660C", 1),
        dict(name="grGreatBay_801F66A4", pre_c="stage_info.grkind = (fv() & 1) ? Gr_Kind_GreatBay : 3;", pre_r="m::ext::stage_info.grkind = if fv() & 1 != 0 { Gr_Kind_GreatBay } else { 3 };",
             c="ret = (u32) grGreatBay_801F66A4();", r="ret = m::grGreatBay_801F66A4() as u32;"),
        dict(name="grGreatBay_801F6708", pre_c="u32 a = fv();", pre_r="let a = fv();",
             c="ret = (u32) grGreatBay_801F6708(a, &GS[0]);", r="ret = m::grGreatBay_801F6708(a, &raw mut GS[0]) as u32;"),
        dict(name="grGreatBay_801F67A4", pre_c="float a = rw();", pre_r="let a = rw();",
             c="grGreatBay_801F67A4(&VW[0], a);", r="m::grGreatBay_801F67A4(&raw mut VW[0], a);"),
        dict(name="grGreatBay_801F680C", pre_c="int a = (int) fv();", pre_r="let a = fv() as i32;",
             c="ret = PI(grGreatBay_801F680C(a));", r="ret = pi(m::grGreatBay_801F680C(a));"),
        dict(name="grGreatBay_801F6814", pre_c="int a = (int) fv(); VW[0].y = rw();", pre_r="let a = fv() as i32; VW[0].y = rw();",
             c="ret = (u32) grGreatBay_801F6814(&VW[0], a, &JO[2]);", r="ret = m::grGreatBay_801F6814(&raw mut VW[0], a, &raw mut JO[2]) as u32;"),
        dict(name="grGreatBay_801F5460_inline", usel=3, pre_c="int a = (int) (fv() & 7);", pre_r="let a = (fv() & 7) as usize;",
             c="grGreatBay_801F5460_inline(&GS[10], &JO[a]);", r="m::grGreatBay_801F5460_inline(&raw mut GS[10], &raw mut JO[a]);"),
        dict(name="tables", cases=1,
             c="for (j = 0; j < 11; j++) hash_StageCallbacks(&grGb_StageCallbacks[j]); hash_StageData(&grGb_StageData); "
               "HM(SH(grGb_StageData.data1)); HM(PI(grGb_StageData.callbacks)); HM(PI(grGb_StageData.joints)); "
               "for (j = 0; j < 6; j++) hash_GrJoint(&grGb_803E3E60[j]); "
               "HM(B(grGb_803B81C8.x)); HM(B(grGb_803B81C8.y)); HM(B(grGb_803B81C8.z)); "
               "for (j = 0; j < 8; j++) { HM((u32) grGb_803E4048[j][0]); HM((u32) grGb_803E4048[j][1]); } "
               "{ SpawnItem s = grGb_803B81D4; HM(PI(s.x0_parent_gobj)); HM(PI(s.x4_parent_gobj2)); HM((u32) s.kind); HM((u32) s.hold_kind); HM((u32) s.x10); "
               "HM(B(s.pos.x)); HM(B(s.pos.y)); HM(B(s.pos.z)); HM(B(s.prev_pos.x)); HM(B(s.prev_pos.y)); HM(B(s.prev_pos.z)); HM(B(s.vel.x)); HM(B(s.vel.y)); HM(B(s.vel.z)); "
               "HM(B(s.facing_dir)); HM((u32) s.x3C_damage); HM((u32) s.x3E); HM((u32) s.x40); HM((u32) s.x44_flag.byte); HM((u32) s.x45_flag.byte); "
               "HM((u32) s.x46_flag.byte); HM((u32) s.x47_flag.byte); HM((u32) s.x48_ground_or_air); }",
             r="for j in 0..11 { g::hash_StageCallbacks(&m::grGb_StageCallbacks[j]); } g::hash_StageData(&m::grGb_StageData); "
               "hm(sh(m::grGb_StageData.data1 as *const u8)); hm(pi(m::grGb_StageData.callbacks)); hm(pi(m::grGb_StageData.joints)); "
               "for j in 0..6 { g::hash_GrJoint(&m::grGb_803E3E60[j]); } "
               "hm(m::grGb_803B81C8.x.to_bits()); hm(m::grGb_803B81C8.y.to_bits()); hm(m::grGb_803B81C8.z.to_bits()); "
               "for j in 0..8 { hm(m::grGb_803E4048[j][0] as i32 as u32); hm(m::grGb_803E4048[j][1] as i32 as u32); } "
               "{ let s = m::grGb_803B81D4; hm(pi(s.x0_parent_gobj)); hm(pi(s.x4_parent_gobj2)); hm(s.kind as u32); hm(s.hold_kind as u32); hm(s.x10 as u32); "
               "hm(s.pos.x.to_bits()); hm(s.pos.y.to_bits()); hm(s.pos.z.to_bits()); hm(s.prev_pos.x.to_bits()); hm(s.prev_pos.y.to_bits()); hm(s.prev_pos.z.to_bits()); hm(s.vel.x.to_bits()); hm(s.vel.y.to_bits()); hm(s.vel.z.to_bits()); "
               "hm(s.facing_dir.to_bits()); hm(s.x3C_damage as i32 as u32); hm(s.x3E as i32 as u32); hm(s.x40 as u32); hm(s.x44_flag.byte as u32); hm(s.x45_flag.byte as u32); "
               "hm(s.x46_flag.byte as u32); hm(s.x47_flag.byte as u32); hm(s.x48_ground_or_air as u32); }"),
    ],
)
