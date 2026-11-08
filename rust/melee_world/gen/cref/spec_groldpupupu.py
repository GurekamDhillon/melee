"""Oracle spec of tu::groldpupupu (see gen_drv.py for the keys)."""
SETUP_C = """zero(&YP, sizeof YP); zero(&stage_info, sizeof stage_info); fill_StageInfo(&stage_info);
YP.x0 = (s16) (fv() % 40); YP.x2 = (s16) (fv() % 40); YP.x4 = (s16) (fv() % 40); YP.x6 = (s16) (fv() % 40);
YP.x8 = (int) (fv() % 40); YP.xC = (int) (fv() % 40);
YP.x10 = rg(); YP.x14 = rg(); YP.x18 = rg(); YP.x1C = rg(); YP.x20 = rg(); YP.x24 = rg(); YP.x28 = rg();
YP.x2C = (float) (fv() % 20); YP.x30 = (float) (fv() % 20);
yakumono_param = &YP;
grOp_804D6A9C = (fv() & 1) ? 0 : (void*) &GO[0];"""
SETUP_R = """m::ext::stage_info = core::mem::zeroed(); YP = core::mem::zeroed(); g::fill_StageInfo(&mut m::ext::stage_info);
YP.x0 = (fv() % 40) as i16; YP.x2 = (fv() % 40) as i16; YP.x4 = (fv() % 40) as i16; YP.x6 = (fv() % 40) as i16;
YP.x8 = (fv() % 40) as i32; YP.xC = (fv() % 40) as i32;
YP.x10 = rg(); YP.x14 = rg(); YP.x18 = rg(); YP.x1C = rg(); YP.x20 = rg(); YP.x24 = rg(); YP.x28 = rg();
YP.x2C = (fv() % 20) as f32; YP.x30 = (fv() % 20) as f32;
m::yakumono_param = &raw mut YP;
m::grOp_804D6A9C = if fv() & 1 != 0 { core::ptr::null_mut() } else { &raw mut GO[0] as *mut c_void };"""

RAND_GOBJ = ("((fv() & 3) == 0 ? 0 : &GO[fv() & 3])",
             "if fv() & 3 == 0 { core::ptr::null_mut() } else { &raw mut super::GO[(fv() & 3) as usize] }")

# table indices: the C indexes grOp_804D48xx[xCC][xD4/xD8] and grOp_803E67D8[xC4] unchecked
FIX_C = ("for (i = 0; i < 4; i++) { GR[i].u.oldpupupu.xCC = 0; GR[i].u.oldpupupu.xD4 &= 1; GR[i].u.oldpupupu.xD8 &= 1;\n"
         "  GR[i].u.oldpupupu.xC4 = (s32) ((u32) GR[i].u.oldpupupu.xC4 % 3); }")
FIX_R = ("for i in 0..4 { GR[i].u.oldpupupu.xCC = 0; GR[i].u.oldpupupu.xD4 &= 1; GR[i].u.oldpupupu.xD8 &= 1;\n"
         "  GR[i].u.oldpupupu.xC4 = ((GR[i].u.oldpupupu.xC4 as u32) % 3) as i32; }")
# 802113E0 sets xCC itself before every table read, so xCC stays random there
FIX2_C = ("for (i = 0; i < 4; i++) { GR[i].u.oldpupupu.xD8 &= 1; GR[i].u.oldpupupu.xC4 = (s32) ((u32) GR[i].u.oldpupupu.xC4 % 3);\n"
          "  if (fv() & 1) GR[i].u.oldpupupu.xCC = (s32) (fv() % 4); else GR[i].u.oldpupupu.xCC = 0; "
          "  if (fv() % 3 == 0) { GR[i].u.oldpupupu.xC8 = 2; GR[i].u.oldpupupu.xD4 = 0; GR[i].u.oldpupupu.xD0 = (fv() & 1) ? 0x2C : (s32) (49 + fv() % 3); } }")
FIX2_R = ("for i in 0..4 { GR[i].u.oldpupupu.xD8 &= 1; GR[i].u.oldpupupu.xC4 = ((GR[i].u.oldpupupu.xC4 as u32) % 3) as i32;\n"
          "  if fv() & 1 != 0 { GR[i].u.oldpupupu.xCC = (fv() % 4) as i32; } else { GR[i].u.oldpupupu.xCC = 0; } "
          "  if fv() % 3 == 0 { GR[i].u.oldpupupu.xC8 = 2; GR[i].u.oldpupupu.xD4 = 0; GR[i].u.oldpupupu.xD0 = if fv() & 1 != 0 { 0x2C } else { (49 + fv() % 3) as i32 }; } }")

SPEC = dict(
    src="src/melee/gr/groldpupupu.c",
    types=["StageInfo", "StageCallbacks", "StageData"],
    null_ret_ok=["HSD_GObj_SetupProc", "grZakoGenerator_801CAE04"],
    ret={
        "Ground_GetYakumonoParam": ("&YP", "&raw mut super::YP as *mut c_void"),
        "Ground_GetStageGObj": RAND_GOBJ,
        "Ground_GetMapGObj": ("&GO[fv() & 3]", "&raw mut super::GO[(fv() & 3) as usize]"),
        "Ground_801C3FA4": ("&JP[fv() % 16]", "&raw mut super::JP[(fv() % 16) as usize]"),
    },
    effects={
        "lb_8000B1CC": ("WV(a2);", "super::wv(a2);"),
        "ftLib_GetPos": ("WV(a1);", "super::wv(a1);"),
        "Camera_LogicBounds": ("*a0 = rg() * 300.0f; *a1 = rg() * 300.0f; *a2 = rg() * 300.0f;",
                               "*a0 = rg() * 300.0; *a1 = rg() * 300.0; *a2 = rg() * 300.0;"),
    },
    decl_c="static struct grOldpupupu_YakumonoParam YP;",
    decl_r="static mut YP: m::grOldpupupu_YakumonoParam = unsafe { core::mem::zeroed() };",
    setup_c=SETUP_C,
    setup_r=SETUP_R,
    hash_c="hash_StageInfo(&stage_info); HM(RH(&YP, sizeof YP)); HM(PI(yakumono_param)); HM(PI(grOp_804D6A9C));",
    hash_r="g::hash_StageInfo(&m::ext::stage_info); hm(rh(&YP)); hm(pi(m::yakumono_param)); hm(pi(m::grOp_804D6A9C));",
    preg_c="preg(&YP, sizeof YP); preg(grOp_StageCallbacks, sizeof grOp_StageCallbacks); preg(&grOp_StageData, sizeof grOp_StageData); preg(grOp_803E67E4, sizeof grOp_803E67E4);",
    preg_r="preg(&raw const YP, core::mem::size_of_val(&YP)); preg(&raw const m::grOp_StageCallbacks, core::mem::size_of_val(&m::grOp_StageCallbacks)); preg(&raw const m::grOp_StageData, core::mem::size_of_val(&m::grOp_StageData)); preg(&raw const m::grOp_803E67E4, core::mem::size_of_val(&m::grOp_803E67E4));",
    data=[("grOp_StageCallbacks", "StageCallbacks", 9), ("grOp_StageData", "StageData", None),
          ("grOp_803E67B0", "raw", None), ("grOp_803E67D8", "raw", None), ("grOp_803E67E4", "raw", None),
          ("grOp_804D48A0", "raw", None), ("grOp_804D48A8", "raw", None), ("grOp_804D48B0", "raw", None),
          ("grOp_804D48C8", "raw", None), ("grOp_804D48D0", "raw", None), ("grOp_804D48D8", "raw", None),
          ("grOp_804D48E0", "raw", None)],
    # the dispatchers run on_init callbacks of the oldpupupu / oldpupupu2 views (no bitfields)
    view={"grOldPupupu_802107E0": "-", "grOldPupupu_802108B4": "-"},
    hview={"grOldPupupu_802107E0": "oldpupupu", "grOldPupupu_802108B4": "oldpupupu"},
    args={
        "grOldPupupu_802108B4": [("(int) (fv() % 9)", "(fv() % 9) as i32")],
    },
    pre={
        "grOldPupupu_80210A24": (FIX_C, FIX_R),
        "grOldPupupu_80211C1C": (FIX_C, FIX_R),
        "grOldPupupu_802113E0": (FIX2_C, FIX2_R),
    },
    cases={"grOldPupupu_802113E0": 2000, "grOldPupupu_80210D10": 1000, "grOldPupupu_80210A24": 600,
           "fn_802112F4": 600, "grOldPupupu_8021128C": 600},
)
