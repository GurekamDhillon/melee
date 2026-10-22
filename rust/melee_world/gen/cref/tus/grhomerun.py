"""Oracle spec + driver of src/melee/gr/grhomerun.c (see ../gen_tu.py, ../mk_tu.py)."""
import os, sys
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
from drvlib import *

V3 = ("{a}->x = rw(); {a}->y = rw(); {a}->z = rw();", "(*{a}).x = rw(); (*{a}).y = rw(); (*{a}).z = rw();")
V3T = ("T(B({a}->x)); T(B({a}->y)); T(B({a}->z));", "t((*{a}).x.to_bits()); t((*{a}).y.to_bits()); t((*{a}).z.to_bits());")

SPEC = dict(
    src="src/melee/gr/grhomerun.c",
    types=["HSD_GObj", "Ground", "HSD_JObj", "StageCallbacks", "StageData", "CollData", "Vec", "HSD_Text", "CmSubject",
           "HSD_CObj", "StageInfo", "UnkArchiveStruct", "GrJoint"],
    # views of GroundVars: 0 = the map's (homerun), 1 = a part's / a text stage's (homerun2)
    unions={"GroundVars": ["homerun", "homerun2"]},
    # xC8 / xCC of the text stages hold pointers as ints (compared through PI in the driver)
    skip=(".homerun2.xC8", ".homerun2.xCC"),
    null_ret_ok=["HSD_GObj_SetupProc", "grZakoGenerator_801CAE04", "it_8026BE84"],
    custom_stubs={
        "HSD_SisLib_803A6B98": dict(
            c="int HSD_SisLib_803A6B98(HSD_Text* a0, float a1, float a2, const char* a3, ...) { __builtin_va_list ap; int v; "
              "__builtin_va_start(ap, a3); v = __builtin_va_arg(ap, int); __builtin_va_end(ap); T(@ID@); T(PI(a0)); T(B(a1)); T(B(a2)); "
              "T(SH(a3)); T((u32) v); return (int) fv(); }",
            ext="fn HSD_SisLib_803A6B98(a0: *mut HSD_Text, a1: f32, a2: f32, a3: *const i8, a4: i32) -> i32;",
            hook="|a0, a1, a2, a3, a4| { t(@ID@); t(pi(a0)); t(a1.to_bits()); t(a2.to_bits()); t(sh(a3 as *const u8)); t(a4 as u32); fv() as i32 }"),
    },
    ret={
        "Ground_801C0498": ("(0.6f + (float) (fv() % 100) / 50.0f)", "0.6f32 + (fv() % 100) as f32 / 50.0f32"),
        "Camera_80030A50": ("(((fv() & 3) == 0) ? 0 : &MCG)", "if fv() & 3 == 0 { core::ptr::null_mut() } else { &raw mut super::MCG }"),
        "GObj_Create": ("&TG", "&raw mut super::TG"),
        "Ground_801C2CF4": ("&JO[a0 & 7]", "&raw mut super::JO[(a0 & 7) as usize]"),
        "Ground_801C3FA4": ("&JO[a1 & 7]", "&raw mut super::JO[(a1 & 7) as usize]"),
        "Ground_GetMapGObj": ("(((fv() & 7) == 0) ? 0 : &GS[10])", "if fv() & 7 == 0 { core::ptr::null_mut() } else { &raw mut super::GS[10] }"),
        "Ground_GetP1Fighter": ("(((fv() & 7) == 0) ? 0 : &FTR[0])", "if fv() & 7 == 0 { core::ptr::null_mut() } else { &raw mut super::FTR[0] }"),
        "Ground_GetP1Fighter2": ("(((fv() & 3) == 0) ? 0 : &FTR[1])", "if fv() & 3 == 0 { core::ptr::null_mut() } else { &raw mut super::FTR[1] }"),
        "Ground_GetStageGObj": ("((NOK && a0 == NID && (fv() & 7) == 0) ? 0 : &GS[a0 % 11])",
                                "if super::NOK && a0 == super::NID && fv() & 7 == 0 { core::ptr::null_mut() } else { &raw mut super::GS[(a0 % 11) as usize] }"),
        "Ground_GetYakumonoParam": ("YPB", "&raw mut super::YPB as *mut c_void"),
        "HSD_ArchiveGetPublicAddress": ("SISB", "&raw mut super::SISB as *mut c_void"),
        "HSD_MemAlloc": ("(a0 >= 512 ? (void*) PARTS : (void*) BACKA)", "if a0 >= 512 { &raw mut super::PARTS as *mut c_void } else { &raw mut super::BACKA as *mut c_void }"),
        "HSD_SisLib_803A6754": ("&TXT[TN++ & 3]", "{ let i = super::TN; super::TN += 1; &raw mut super::TXT[i & 3] }"),
        "ftLib_80086B74": ("(((fv() & 3) == 0) ? 0 : &CS[fv() & 1])", "if fv() & 3 == 0 { core::ptr::null_mut() } else { &raw mut super::CS[(fv() & 1) as usize] }"),
        "gm_80180AF4": ("(((fv() & 3) == 0) ? 0 : &SG)", "if fv() & 3 == 0 { core::ptr::null_mut() } else { &raw mut super::SG }"),
        "grDatFiles_GetArchive": ("&ARCH", "&raw mut super::ARCH"),
        "lb_80013B14": ("&MC", "&raw mut super::MC"),
        "Stage_GetCamBoundsRightOffset": ("rw()", "rw()"),
        "Stage_GetBlastZoneRightOffset": ("rw()", "rw()"),
        "HSD_CObjGetNear": ("rw()", "rw()"), "HSD_CObjGetFar": ("rw()", "rw()"),
    },
    effects={
        "Camera_GetTransformInterest": ("a0->x = (float) ((int) (fv() % 80000) - 40000); a0->y = rw(); a0->z = rw();",
                                        "(*a0).x = ((fv() % 80000) as i32 - 40000) as f32; (*a0).y = rw(); (*a0).z = rw();"),
        "Stage_UnkSetVec3TCam_Offset": (V3[0].format(a="a0"), V3[1].format(a="a0")),
        "ftLib_80086644": (V3[0].format(a="a1"), V3[1].format(a="a1")),
        "lb_8000B1CC": (V3[0].format(a="a2"), V3[1].format(a="a2")),
        "HSD_CObjGetViewportf": ("a1->xmin = rw(); a1->xmax = rw(); a1->ymin = rw(); a1->ymax = rw();",
                                 "(*a1).xmin = rw(); (*a1).xmax = rw(); (*a1).ymin = rw(); (*a1).ymax = rw();"),
        "HSD_CObjSetViewportf": ("T(B(a1->xmin)); T(B(a1->xmax)); T(B(a1->ymin)); T(B(a1->ymax));",
                                 "t((*a1).xmin.to_bits()); t((*a1).xmax.to_bits()); t((*a1).ymin.to_bits()); t((*a1).ymax.to_bits());"),
        "HSD_CObjGetScissor": ("a1->left = (u16) fv(); a1->right = (u16) fv(); a1->top = (u16) fv(); a1->bottom = (u16) fv();",
                               "(*a1).left = fv() as u16; (*a1).right = fv() as u16; (*a1).top = fv() as u16; (*a1).bottom = fv() as u16;"),
        "HSD_CObjSetScissor": ("T(a1->left); T(a1->right); T(a1->top); T(a1->bottom);",
                               "t((*a1).left as u32); t((*a1).right as u32); t((*a1).top as u32); t((*a1).bottom as u32);"),
        "HSD_CObjGetPerspective": ("*a1 = rw(); *a2 = rw();", "*a1 = rw(); *a2 = rw();"),
        "HSD_CObjGetEyePosition": (V3[0].format(a="a1"), V3[1].format(a="a1")),
        "HSD_CObjGetInterest": (V3[0].format(a="a1"), V3[1].format(a="a1")),
        "HSD_CObjSetEyePosition": (V3T[0].format(a="a1"), V3T[1].format(a="a1")),
        "HSD_CObjSetInterest": (V3T[0].format(a="a1"), V3T[1].format(a="a1")),
        "lb_80013B14": ("T(a0->flags); T(a0->projection_type); T((u32) a0->viewport.xmin); T((u32) a0->viewport.xmax); T((u32) a0->viewport.ymin); "
                        "T((u32) a0->viewport.ymax); T(a0->scissor.left); T(a0->scissor.right); T(a0->scissor.top); T(a0->scissor.bottom); "
                        "T(PI(a0->class_name)); T(PI(a0->eyepos->class_name)); T(B(a0->eyepos->pos.x)); T(B(a0->eyepos->pos.y)); T(B(a0->eyepos->pos.z)); T(PI(a0->eyepos->robjdesc)); "
                        "T(PI(a0->interest->class_name)); T(B(a0->interest->pos.x)); T(B(a0->interest->pos.y)); T(B(a0->interest->pos.z)); T(PI(a0->interest->robjdesc)); "
                        "T(B(a0->roll)); T(PI(a0->up_vector)); T(B(a0->nnear)); T(B(a0->ffar)); T(B(a0->fov)); T(B(a0->aspect));",
                        "t((*a0).flags as u32); t((*a0).projection_type as u32); t((*a0).viewport.xmin as i32 as u32); t((*a0).viewport.xmax as i32 as u32); t((*a0).viewport.ymin as i32 as u32); "
                        "t((*a0).viewport.ymax as i32 as u32); t((*a0).scissor.left as u32); t((*a0).scissor.right as u32); t((*a0).scissor.top as u32); t((*a0).scissor.bottom as u32); "
                        "t(pi((*a0).class_name)); t(pi((*(*a0).eyepos).class_name)); t((*(*a0).eyepos).pos.x.to_bits()); t((*(*a0).eyepos).pos.y.to_bits()); t((*(*a0).eyepos).pos.z.to_bits()); t(pi((*(*a0).eyepos).robjdesc)); "
                        "t(pi((*(*a0).interest).class_name)); t((*(*a0).interest).pos.x.to_bits()); t((*(*a0).interest).pos.y.to_bits()); t((*(*a0).interest).pos.z.to_bits()); t(pi((*(*a0).interest).robjdesc)); "
                        "t((*a0).roll.to_bits()); t(pi((*a0).up_vector)); t((*a0).nnear.to_bits()); t((*a0).ffar.to_bits()); t((*a0).fov.to_bits()); t((*a0).aspect.to_bits());"),
        "it_8026BE84": ("HM(PI(a0->x0)); HM(PI(a0->x4)); HM(B(a0->x8_vec.x)); HM(B(a0->x8_vec.y)); HM(B(a0->x8_vec.z)); "
                        "HM((u32) a0->x14); HM((u32) a0->x18); HM((u32) a0->x1C.b0);",
                        "hm(pi((*a0).x0)); hm(pi((*a0).x4)); hm((*a0).x8_vec.x.to_bits()); hm((*a0).x8_vec.y.to_bits()); "
                        "hm((*a0).x8_vec.z.to_bits()); hm((*a0).x14 as u32); hm((*a0).x18 as u32); hm((*a0).x1C.anon_0.b0() as u32);"),
    })

# text stages (view 1 = homerun2): xC8 / xCC hold pointers as ints
TXT_PRE_C = ("GG[@K@].u.unk.xCC = (fv() & 1) ? (int) (intptr_t) &JO[fv() & 7] : 0; "
             "GG[@K@].u.unk.xC8 = (fv() & 1) ? (int) (intptr_t) &TXT[fv() & 3] : 0;")
TXT_PRE_R = ("GG[@K@].u.unk.xCC = if fv() & 1 != 0 { (&raw mut JO[(fv() & 7) as usize]) as isize as i32 } else { 0 }; "
             "GG[@K@].u.unk.xC8 = if fv() & 1 != 0 { (&raw mut TXT[(fv() & 3) as usize]) as isize as i32 } else { 0 };")


def gs(name, k, usel=1, pre=False):
    d = vg(name, usel, f"GS[{k}]")
    if pre:
        d["pre_c"] = TXT_PRE_C.replace('@K@', str(k))
        d["pre_r"] = TXT_PRE_R.replace('@K@', str(k))
    return d


def pgs(name, k):
    return pg(name, 1, f"GS[{k}]")


DRV = dict(
    state=[("GS", "HSD_GObj", 11), ("JO", "HSD_JObj", 24), ("PG", "HSD_GObj", 64), ("FTR", "HSD_GObj", 2), ("CS", "CmSubject", 2),
           ("SG", "HSD_GObj", 1), ("MCG", "HSD_GObj", 1), ("MC", "HSD_CObj", 1), ("TCG", "HSD_GObj", 1), ("TC", "HSD_CObj", 1),
           ("TG", "HSD_GObj", 1), ("TXT", "HSD_Text", 4), ("ARCH", "UnkArchiveStruct", 1), ("CD", "CollData", 1), ("VW", "Vec", 2),
           ("SI", "StageInfo", 1, "m::ext::stage_info")],
    decl_c=("static Ground GG[11]; static Ground PGG[64]; static HSD_GObj* PARTS[64]; static HSD_GObj* BACKA[16]; "
            "static u8 YPB[64]; static u8 SISB[64]; static int TN; static int NOK; static int NID;"),
    decl_r=chr(10).join([
        "static mut GG: [Ground; 11] = unsafe { core::mem::zeroed() };",
        "static mut PGG: [Ground; 64] = unsafe { core::mem::zeroed() };",
        "static mut PARTS: [*mut HSD_GObj; 64] = [core::ptr::null_mut(); 64];",
        "static mut BACKA: [*mut HSD_GObj; 16] = [core::ptr::null_mut(); 16];",
        "static mut YPB: [u8; 64] = [0; 64];",
        "static mut SISB: [u8; 64] = [0; 64];",
        "static mut TN: usize = 0;",
        "static mut NOK: bool = false;",
        "static mut NID: i32 = 0;"]),
    regs=[("GG", "sizeof GG", "&raw const GG", "core::mem::size_of_val(&GG)"),
          ("PGG", "sizeof PGG", "&raw const PGG", "core::mem::size_of_val(&PGG)"),
          ("PARTS", "sizeof PARTS", "&raw const PARTS", "core::mem::size_of_val(&PARTS)"),
          ("BACKA", "sizeof BACKA", "&raw const BACKA", "core::mem::size_of_val(&BACKA)"),
          ("YPB", "sizeof YPB", "&raw const YPB", "core::mem::size_of_val(&YPB)"),
          ("SISB", "sizeof SISB", "&raw const SISB", "core::mem::size_of_val(&SISB)")],
    setup_c="""
            for (i = 0; i < 24; i++) JO[i].child = &JO[(i + 1) % 24];
            for (i = 0; i < 11; i++) {
                GS[i].user_data = &GG[i]; GS[i].hsd_obj = &JO[8 + i];
                zero(&GG[i], sizeof GG[i]);
                USEL = (i == 10) ? 0 : 1;
                fill_Ground(&GG[i]);
                GG[i].gobj = &GS[i];
                GG[i].x8_callback = grHomeRun_8021CB18; GG[i].xC_callback = grHomeRun_8021CB1C;
            }
            { int n = 3 + (int) (fv() % 55); int b = (int) (fv() % 300);
              zero(PARTS, sizeof PARTS); zero(BACKA, sizeof BACKA);
              for (i = 0; i < 64; i++) {
                  zero(&PGG[i], sizeof PGG[i]);
                  USEL = 1; fill_Ground(&PGG[i]);
                  PG[i].user_data = &PGG[i]; PG[i].hsd_obj = &JO[i % 8];
                  PGG[i].u.homerun2.xC4 = (u16) (b + i);
                  PARTS[i] = (i < n) ? &PG[i] : 0;
              } }
            USEL = OPUSEL[op];
            GG[10].u.homerun.parts = PARTS; GG[10].u.homerun.back = BACKA;
            for (i = 0; i < 4; i++) GG[10].u.homerun.bg_gobj[i] = &GS[i];
            GG[10].x8_callback = grHomeRun_8021CB18; GG[10].xC_callback = grHomeRun_8021CB1C;
            MCG.hsd_obj = ((fv() & 3) == 0) ? 0 : &MC; TCG.hsd_obj = &TC;
            CD.x0_gobj = (fv() & 1) ? &SG : &GS[0];
            yakumono_param = YPB;
            grHr_804D6AD8 = rw(); grHr_804D6ADC = 1 + (int) (fv() % 6); grHr_804D6AE0 = rw(); grHr_804D6AE4 = rw();
            ARCH.unk0 = 0; TN = 0; NOK = 0; NID = 99;
            HSD_GObj_CameraKind = (u8) fv();
            for (i = 0; i < 5; i++) HSD_SisLib_804D1124[i] = 0;
""",
    setup_r="""
                for i in 0..24usize { JO[i].child = &raw mut JO[(i + 1) % 24]; }
                for i in 0..11usize {
                    GS[i].user_data = &raw mut GG[i] as *mut c_void; GS[i].hsd_obj = &raw mut JO[8 + i] as *mut c_void;
                    GG[i] = core::mem::zeroed();
                    USEL = if i == 10 { 0 } else { 1 };
                    g::fill_Ground(&mut GG[i]);
                    GG[i].gobj = &raw mut GS[i];
                    GG[i].x8_callback = Some(m::grHomeRun_8021CB18); GG[i].xC_callback = Some(m::grHomeRun_8021CB1C);
                }
                { let n = 3 + (fv() % 55) as usize; let b = (fv() % 300) as usize;
                  PARTS = [core::ptr::null_mut(); 64]; BACKA = [core::ptr::null_mut(); 16];
                  for i in 0..64usize {
                      PGG[i] = core::mem::zeroed();
                      USEL = 1; g::fill_Ground(&mut PGG[i]);
                      PG[i].user_data = &raw mut PGG[i] as *mut c_void; PG[i].hsd_obj = &raw mut JO[i % 8] as *mut c_void;
                      PGG[i].u.homerun2.xC4 = (b + i) as u16;
                      PARTS[i] = if i < n { &raw mut PG[i] } else { core::ptr::null_mut() };
                  } }
                USEL = OPUSEL[op as usize];
                GG[10].u.homerun.parts = PARTS.as_mut_ptr(); GG[10].u.homerun.back = BACKA.as_mut_ptr();
                for i in 0..4usize { GG[10].u.homerun.bg_gobj[i] = &raw mut GS[i]; }
                GG[10].x8_callback = Some(m::grHomeRun_8021CB18); GG[10].xC_callback = Some(m::grHomeRun_8021CB1C);
                MCG.hsd_obj = if fv() & 3 == 0 { core::ptr::null_mut() } else { &raw mut MC as *mut c_void }; TCG.hsd_obj = &raw mut TC as *mut c_void;
                CD.x0_gobj = if fv() & 1 != 0 { &raw mut SG } else { &raw mut GS[0] };
                m::yakumono_param = &raw mut YPB as *mut c_void;
                m::grHr_804D6AD8 = rw(); m::grHr_804D6ADC = 1 + (fv() % 6) as i32; m::grHr_804D6AE0 = rw(); m::grHr_804D6AE4 = rw();
                ARCH.unk0 = core::ptr::null_mut(); TN = 0; NOK = false; NID = 99;
                m::ext::HSD_GObj_CameraKind = fv() as u8;
                for i in 0..5usize { m::ext::HSD_SisLib_804D1124[i] = core::ptr::null_mut(); }
""",
    post_c="""
            for (i = 0; i < 11; i++) { USEL = (i == 10) ? 0 : 1; hash_Ground(&GG[i]); HM(PI((void*) (intptr_t) GG[i].u.unk.xC8)); HM(PI((void*) (intptr_t) GG[i].u.unk.xCC)); }
            USEL = 1;
            for (i = 0; i < 64; i++) hash_Ground(&PGG[i]);
            USEL = OPUSEL[op];
            for (i = 0; i < 64; i++) HM(PI(PARTS[i]));
            for (i = 0; i < 16; i++) HM(PI(BACKA[i]));
            HM(B(grHr_804D6AD8)); HM((u32) grHr_804D6ADC); HM(B(grHr_804D6AE0)); HM(B(grHr_804D6AE4)); HM(PI(yakumono_param));
            for (i = 0; i < 5; i++) HM(PI(HSD_SisLib_804D1124[i]));
            HM((u32) HSD_GObj_CameraKind); HM((u32) TN);
""",
    post_r="""
                for i in 0..11usize { USEL = if i == 10 { 0 } else { 1 }; g::hash_Ground(&GG[i]); hm(pi((GG[i].u.unk.xC8 as isize) as *const u8)); hm(pi((GG[i].u.unk.xCC as isize) as *const u8)); }
                USEL = 1;
                for i in 0..64usize { g::hash_Ground(&PGG[i]); }
                USEL = OPUSEL[op as usize];
                for i in 0..64usize { hm(pi(PARTS[i])); }
                for i in 0..16usize { hm(pi(BACKA[i])); }
                hm(m::grHr_804D6AD8.to_bits()); hm(m::grHr_804D6ADC as u32); hm(m::grHr_804D6AE0.to_bits()); hm(m::grHr_804D6AE4.to_bits()); hm(pi(m::yakumono_param));
                for i in 0..5usize { hm(pi(m::ext::HSD_SisLib_804D1124[i])); }
                hm(m::ext::HSD_GObj_CameraKind as u32); hm(TN as u32);
""",
    ops=[
        dict(name="grHomeRun_8021C750", c="grHomeRun_8021C750((bool) fv());", r="m::grHomeRun_8021C750(fv() as i32);"),
        dict(name="grHomeRun_8021C754", pre_c="yakumono_param = 0;", pre_r="m::yakumono_param = core::ptr::null_mut();",
             c="grHomeRun_8021C754();", r="m::grHomeRun_8021C754();"),
        v0("grHomeRun_8021C7FC"), v0("grHomeRun_8021C800"), p0("grHomeRun_8021C824"),
        dict(name="grHomeRun_8021C82C", pre_c="int a = (int) (fv() % 11); NOK = 1; NID = a;", pre_r="let a = (fv() % 11) as i32; NOK = true; NID = a;",
             c="ret = PI(grHomeRun_8021C82C(a));", r="ret = pi(m::grHomeRun_8021C82C(a));"),
        gs("grHomeRun_8021C914", 0), pgs("grHomeRun_8021CB10", 0), gs("grHomeRun_8021CB18", 0), gs("grHomeRun_8021CB1C", 0),
        gs("grHomeRun_8021CB20", 10, 0), pgs("grHomeRun_8021D678", 10),
        dict(name="grHomeRun_8021D680", usel=0,
             pre_c="GG[10].u.unk.xD0 = (fv() & 1) ? (int) (intptr_t) &JO[fv() & 7] : 0; GG[10].u.unk.xCC = (fv() & 1) ? 0 : (int) (fv() & 0xFF);",
             pre_r="GG[10].u.unk.xD0 = if fv() & 1 != 0 { (&raw mut JO[(fv() & 7) as usize]) as isize as i32 } else { 0 }; "
                   "GG[10].u.unk.xCC = if fv() & 1 != 0 { 0 } else { (fv() & 0xFF) as i32 };",
             c="grHomeRun_8021D680(&GS[10]);", r="m::grHomeRun_8021D680(&raw mut GS[10]);"),
        gs("grHomeRun_8021DEB0", 10), gs("grHomeRun_8021DEB4", 4), pgs("grHomeRun_8021DEE0", 4), gs("grHomeRun_8021DEE8", 4), gs("grHomeRun_8021DEEC", 4),
        gs("grHomeRun_8021DEF0", 5), pgs("grHomeRun_8021DF48", 5), gs("grHomeRun_8021DF50", 5, 1, True), gs("grHomeRun_8021E008", 5, 1, True),
        gs("grHomeRun_8021E038", 1), pgs("grHomeRun_8021E064", 1), gs("grHomeRun_8021E06C", 1), gs("grHomeRun_8021E070", 1),
        gs("grHomeRun_8021E074", 2), pgs("grHomeRun_8021E0CC", 2), gs("grHomeRun_8021E0D4", 2, 1, True), gs("grHomeRun_8021E18C", 2, 1, True),
        gs("grHomeRun_8021E1BC", 6), pgs("grHomeRun_8021E1E8", 6), gs("grHomeRun_8021E1F0", 6), gs("grHomeRun_8021E1F4", 6),
        gs("grHomeRun_8021E1F8", 7), pgs("grHomeRun_8021E250", 7), gs("grHomeRun_8021E258", 7, 1, True), gs("grHomeRun_8021E310", 7, 1, True),
        gs("grHomeRun_8021E340", 8), pgs("grHomeRun_8021E36C", 8), gs("grHomeRun_8021E374", 8), gs("grHomeRun_8021E378", 8),
        gs("grHomeRun_8021E37C", 9), pgs("grHomeRun_8021E3D4", 9), gs("grHomeRun_8021E3DC", 9, 1, True), gs("grHomeRun_8021E494", 9, 1, True),
        gs("grHomeRun_8021E4C4", 3), pgs("grHomeRun_8021E4F0", 3), gs("grHomeRun_8021E4F8", 3), gs("grHomeRun_8021E4FC", 3),
        dict(name="grHomeRun_8021E500", cases=400, pre_c="int a = (int) (fv() % 400);", pre_r="let a = (fv() % 400) as i32;",
             c="ret = PI(grHomeRun_8021E500(a));", r="ret = pi(m::grHomeRun_8021E500(a));"),
        dict(name="fn_8021E994", pre_c="int ja = (int) fv(); int jb = (int) fv(); u32 jk = fv() % 3;", pre_r="let ja = fv() as i32; let jb = fv() as i32; let jk = fv() % 3;",
             c="fn_8021E994(&GG[10], ja, &CD, jb, jk, 0.0f);", r="m::fn_8021E994(&raw mut GG[10] as *mut c_void, ja, &raw mut CD, jb, jk, 0.0);"),
        dict(name="grHomeRun_8021EA30", pre_c="VW[0].x = rw(); if (fv() & 1) { grHr_804D6AE0 = 100.0f; }", pre_r="VW[0].x = rw(); if fv() & 1 != 0 { m::grHr_804D6AE0 = 100.0; }",
             c="grHomeRun_8021EA30(&VW[0].x);", r="m::grHomeRun_8021EA30(&raw const VW[0].x);"),
        v0("grHomeRun_8021EAF8"),
        dict(name="fn_8021EB10", pre_c="int a = (int) fv();", pre_r="let a = fv() as i32;", c="fn_8021EB10(&TCG, a);", r="m::fn_8021EB10(&raw mut TCG, a);"),
        dict(name="grHomeRun_8021EC58", pre_c="int a = (int) fv();", pre_r="let a = fv() as i32;", c="ret = PI(grHomeRun_8021EC58(a));", r="ret = pi(m::grHomeRun_8021EC58(a));"),
        v0("grHomeRun_8021ED74"), v0("grHomeRun_8021EDD4"),
        dict(name="grHomeRun_8021EEB4", pre_c="int a = (int) fv();", pre_r="let a = fv() as i32;", c="ret = PI(grHomeRun_8021EEB4(a));", r="ret = pi(m::grHomeRun_8021EEB4(a));"),
        dict(name="grHomeRun_8021EEBC", pre_c="int a = (int) fv(); VW[0].y = rw();", pre_r="let a = fv() as i32; VW[0].y = rw();",
             c="ret = (u32) grHomeRun_8021EEBC(&VW[0], a, &JO[2]);", r="ret = m::grHomeRun_8021EEBC(&raw mut VW[0], a, &raw mut JO[2]) as u32;"),
        f0("grHomeRun_8021EF10"),
        dict(name="tables", cases=1,
             c="for (j = 0; j < 11; j++) hash_StageCallbacks(&grHr_StageCallbacks[j]); hash_StageData(&grHr_StageData); "
               "HM(SH(grHr_StageData.data1)); HM(PI(grHr_StageData.callbacks)); HM(PI(grHr_StageData.joints)); "
               "HM((u32) grHr_804D4998[0]); HM((u32) grHr_804D4998[1]); HM(SH(grHr_804D49A0));",
             r="for j in 0..11 { g::hash_StageCallbacks(&m::grHr_StageCallbacks[j]); } g::hash_StageData(&m::grHr_StageData); "
               "hm(sh(m::grHr_StageData.data1 as *const u8)); hm(pi(m::grHr_StageData.callbacks)); hm(pi(m::grHr_StageData.joints)); "
               "hm(m::grHr_804D4998[0] as u32); hm(m::grHr_804D4998[1] as u32); hm(sh(&raw const m::grHr_804D49A0 as *const u8));"),
    ],
)
