"""Oracle spec of tu::gronett (see gen_drv.py for the keys)."""
SETUP_C = """NJ = 0; NG = 0; zero(&YP, sizeof YP); zero(&stage_info, sizeof stage_info); fill_StageInfo(&stage_info);
zero(&CS, sizeof CS); fill_CmSubject(&CS); zero(GENB, sizeof GENB); zero(PL, sizeof PL); zero(&ARCH, sizeof ARCH);
zero(&USD, sizeof USD); zero(X8, sizeof X8);
YP.awning_initial = rg(); YP.max_velocity = 0.5f + (float) (fv() % 4); YP.vel_threshold = rg(); YP.pos_threshold = rg();
YP.damping = rg(); YP.spring_force = rg(); YP.spring_constant = rg(); YP.max_displacement = (float) (1 + fv() % 8);
YP.awning_delta = rg();
YP.x24 = (float) (fv() % 40); YP.x28 = (float) (fv() % 40); YP.x2C = (float) (fv() % 40); YP.x30 = (float) (fv() % 40);
YP.x34 = (float) (fv() % 6); YP.x38 = (float) (fv() % 6); YP.x3C = (float) (fv() % 40); YP.x40 = (float) (fv() % 40);
YP.x44 = (float) (fv() % 30); YP.x48 = (float) (fv() % 30); YP.x4C = (float) (fv() % 30); YP.x50 = (float) (fv() % 30);
YP.x54 = rg(); YP.x58 = rg(); YP.x5C = rg(); YP.x60 = (float) (fv() % 100); YP.x64 = rg() * 300.0f;
yakumono_param = &YP;
{
    int n = (int) (fv() % 5);
    for (i = 0; i < 4; i++) GO[i].next = (i + 1 < n) ? &GO[i + 1] : 0;
    PL[8] = n ? &GO[0] : 0;
    HSD_GObjPLinkHead = PL;
}
ARCH.unk4 = ((fv() & 3) == 0) ? 0 : &USD;
USD.unk8 = X8;
grOt_804D69C4 = 0;"""
SETUP_R = """NJ = false; NG = false; YP = core::mem::zeroed(); m::ext::stage_info = core::mem::zeroed(); g::fill_StageInfo(&mut m::ext::stage_info);
CS = core::mem::zeroed(); g::fill_CmSubject(&mut CS); GENB = [[0; 64]; 2]; PL = core::mem::zeroed(); ARCH = core::mem::zeroed();
USD = core::mem::zeroed(); X8 = core::mem::zeroed();
YP.awning_initial = rg(); YP.max_velocity = 0.5f32 + (fv() % 4) as f32; YP.vel_threshold = rg(); YP.pos_threshold = rg();
YP.damping = rg(); YP.spring_force = rg(); YP.spring_constant = rg(); YP.max_displacement = (1 + fv() % 8) as f32;
YP.awning_delta = rg();
YP.x24 = (fv() % 40) as f32; YP.x28 = (fv() % 40) as f32; YP.x2C = (fv() % 40) as f32; YP.x30 = (fv() % 40) as f32;
YP.x34 = (fv() % 6) as f32; YP.x38 = (fv() % 6) as f32; YP.x3C = (fv() % 40) as f32; YP.x40 = (fv() % 40) as f32;
YP.x44 = (fv() % 30) as f32; YP.x48 = (fv() % 30) as f32; YP.x4C = (fv() % 30) as f32; YP.x50 = (fv() % 30) as f32;
YP.x54 = rg(); YP.x58 = rg(); YP.x5C = rg(); YP.x60 = (fv() % 100) as f32; YP.x64 = rg() * 300.0;
m::yakumono_param = &raw mut YP;
{
    let n = (fv() % 5) as usize;
    for i in 0..4usize { GO[i].next = if i + 1 < n { &raw mut GO[i + 1] } else { core::ptr::null_mut() }; }
    PL[8] = if n != 0 { &raw mut GO[0] } else { core::ptr::null_mut() };
    m::ext::HSD_GObjPLinkHead = (&raw mut PL) as *mut *mut HSD_GObj;
}
ARCH.unk4 = if fv() & 3 == 0 { core::ptr::null_mut() } else { &raw mut USD };
USD.unk8 = (&raw mut X8) as *mut UnkStageDat_x8_t;
m::grOt_804D69C4 = 0;"""

# the car state machines index car_jobjs / car_items by curr_car / next_car (0..3 in play)
CAR_C = ("for (i = 0; i < 4; i++) { GR[i].u.onettcar.curr_car = (s8) (fv() % 4); GR[i].u.onettcar.next_car = (s8) (fv() % 4);\n"
         "  GR[i].u.onettcar.state_a = (u8) (fv() % 8); GR[i].u.onettcar.state_b = (u8) (fv() % 6); }")
CAR_R = ("for i in 0..4 { GR[i].u.onettcar.curr_car = (fv() % 4) as i8; GR[i].u.onettcar.next_car = (fv() % 4) as i8;\n"
         "  GR[i].u.onettcar.state_a = (fv() % 8) as u8; GR[i].u.onettcar.state_b = (fv() % 6) as u8; }")
# the same plus: a decided x108 / x110 so that the deeper branches are reached
CAR2_C = CAR_C + "\nfor (i = 0; i < 4; i++) { if (fv() % 3 == 0) GR[i].u.onettcar.x108 = (s32) (fv() % 3); if (fv() % 3 == 0) GR[i].u.onettcar.x110 = (s32) (fv() % 5); if (fv() % 4 != 0) GR[i].u.onettcar.x0_b0 = 0; }"
CAR2_R = CAR_R + "\nfor i in 0..4 { if fv() % 3 == 0 { GR[i].u.onettcar.x108 = (fv() % 3) as i32; } if fv() % 3 == 0 { GR[i].u.onettcar.x110 = (fv() % 5) as i32; } if fv() % 4 != 0 { GR[i].u.onettcar.set_x0_b0(0); } }"


# values of JObj.translate.x around the thresholds the car code compares against
NEAR_C = ("for (i = 0; i < 8; i++) { u32 w = fv() % 6; float d = 0.75f * (float) ((int) (fv() % 3) - 1); "
          "  if (w == 0) JP[i].translate.x = 580.0f + d; else if (w == 1) JP[i].translate.x = -642.0f + d; "
          "  else if (w == 2) JP[i].translate.x = -513.0f + d; else if (w == 3) JP[i].translate.x = 726.0f + d; }")
NEAR_R = ("for i in 0..8usize { let w = fv() % 6; let d = 0.75f32 * ((fv() % 3) as i32 - 1) as f32; "
          "  if w == 0 { JP[i].translate.x = 580.0 + d; } else if w == 1 { JP[i].translate.x = -642.0 + d; } "
          "  else if w == 2 { JP[i].translate.x = -513.0 + d; } else if w == 3 { JP[i].translate.x = 726.0 + d; } }")
CAR2_C = CAR2_C + chr(10) + NEAR_C
CAR2_R = CAR2_R + chr(10) + NEAR_R
# the building: frame / hit counts just before the values it compares against
BLD_C = ("for (i = 0; i < 4; i++) { u32 w = fv() % 8; "
         "  if (w == 0) GR[i].u.onett_building.frame = 0x13; else if (w == 1) GR[i].u.onett_building.frame = 0x45; else if (w == 2) GR[i].u.onett_building.frame = 0x81; "
         "  else if (w == 3) GR[i].u.onett_building.frame = 0x59; else if (w == 4) GR[i].u.onett_building.frame = 0x8B; "
         "  if (fv() % 3 == 0) GR[i].u.onett_building.state = (s16) (fv() % 8); }")
BLD_R = ("for i in 0..4 { let w = fv() % 8; "
         "  if w == 0 { GR[i].u.onett_building.frame = 0x13; } else if w == 1 { GR[i].u.onett_building.frame = 0x45; } else if w == 2 { GR[i].u.onett_building.frame = 0x81; } "
         "  else if w == 3 { GR[i].u.onett_building.frame = 0x59; } else if w == 4 { GR[i].u.onett_building.frame = 0x8B; } "
         "  if fv() % 3 == 0 { GR[i].u.onett_building.state = (fv() % 8) as i16; } }")
HIT_C = ("for (i = 0; i < 4; i++) { if (fv() & 1) GR[i].u.onett_building.hit_count = (s32) (fv() % 7); if (fv() % 3 == 0) GR[i].u.onett_building.state = (s16) (fv() % 7); }")
HIT_R = ("for i in 0..4 { if fv() & 1 != 0 { GR[i].u.onett_building.hit_count = (fv() % 7) as i32; } if fv() % 3 == 0 { GR[i].u.onett_building.state = (fv() % 7) as i16; } }")

RANDG = ("((NG && (fv() & 3) == 0) ? 0 : &GO[a0 & 3])",
         "if super::NG && fv() & 3 == 0 { core::ptr::null_mut() } else { &raw mut super::GO[(a0 & 3) as usize] }")

SPEC = dict(
    src="src/melee/gr/gronett.c",
    types=["StageInfo", "StageCallbacks", "StageData", "CmSubject"],
    null_ret_ok=["HSD_GObj_SetupProc", "grZakoGenerator_801CAE04", "grLib_801C9808"],
    pools={
        "HSD_JObj": ("&JP[fv() % 8]", "&raw mut JP[(fv() % 8) as usize]"),
        "CmSubject": ("&CS", "&raw mut CS"),
        "HSD_Generator": ("(HSD_Generator*) GENB[fv() & 1]", "(&raw mut GENB[(fv() & 1) as usize]) as *mut HSD_Generator"),
    },
    ret={
        "Camera_80029020": ("&CS", "&raw mut super::CS"),
        "Ground_GetYakumonoParam": ("&YP", "&raw mut super::YP as *mut c_void"),
        "Ground_GetStageGObj": RANDG,
        "Ground_801C3FA4": ("((NJ && (fv() & 7) == 0) ? 0 : &JP[fv() % 8])",
                            "if super::NJ && fv() & 7 == 0 { core::ptr::null_mut() } else { &raw mut super::JP[(fv() % 8) as usize] }"),
        "grDatFiles_801C6330": ("&ARCH", "&raw mut super::ARCH"),
        "grLib_801C96F8": ("(HSD_Generator*) GENB[fv() & 1]", "(&raw mut super::GENB[(fv() & 1) as usize]) as *mut HSD_Generator"),
        "grMaterial_801C8CFC": ("&GO[fv() & 3]", "&raw mut super::GO[(fv() & 3) as usize]"),
        "gm_8016C6C0": ("(int) (fv() % 4)", "(fv() % 4) as i32"),
    },
    effects={
        "lb_8000B1CC": ("TV(a1); WV(a2);", "super::tv(a1); super::wv(a2);"),
        "ftLib_80086644": ("WV(a1);", "super::wv(a1);"),
        "grLib_801C96F8": ("TV(a2);", "super::tv(a2);"),
        "Camera_LogicBounds": ("*a0 = rg() * 300.0f; *a1 = rg() * 300.0f; *a2 = rg() * 300.0f;",
                               "*a0 = rg() * 300.0; *a1 = rg() * 300.0; *a2 = rg() * 300.0;"),
    },
    decl_c=("static struct grOnett_StageParam YP;\nstatic CmSubject CS;\nstatic u8 GENB[2][64];\nstatic HSD_GObj* PL[16];\n"
            "static UnkArchiveStruct ARCH;\nstatic UnkStageDat USD;\nstatic struct UnkStageDat_x8_t X8[3];\nstatic int NJ, NG;\n"
            "static s32 SFXT[4][2] = { { 0x5F377, 0x5F376 }, { 0x5F377, 0x5F376 }, { 0x5F379, 0x5F378 }, { 0x5F37B, 0x5F37A } };"),
    decl_r=("static mut YP: m::grOnett_StageParam = unsafe { core::mem::zeroed() };\nstatic mut CS: CmSubject = unsafe { core::mem::zeroed() };\n"
            "static mut GENB: [[u8; 64]; 2] = [[0; 64]; 2];\nstatic mut PL: [*mut HSD_GObj; 16] = [core::ptr::null_mut(); 16];\n"
            "static mut ARCH: UnkArchiveStruct = unsafe { core::mem::zeroed() };\nstatic mut USD: UnkStageDat = unsafe { core::mem::zeroed() };\n"
            "static mut X8: [UnkStageDat_x8_t; 3] = unsafe { core::mem::zeroed() };\nstatic mut NJ: bool = false;\nstatic mut NG: bool = false;\n"
            "static SFXT: [[i32; 2]; 4] = [[0x5F377, 0x5F376], [0x5F377, 0x5F376], [0x5F379, 0x5F378], [0x5F37B, 0x5F37A]];"),
    setup_c=SETUP_C,
    setup_r=SETUP_R,
    hash_c="hash_StageInfo(&stage_info); hash_CmSubject(&CS); HM(RH(&YP, sizeof YP)); HM(PI(yakumono_param)); HM((u32) grOt_804D69C4);",
    hash_r="g::hash_StageInfo(&m::ext::stage_info); g::hash_CmSubject(&CS); hm(rh(&YP)); hm(pi(m::yakumono_param)); hm(m::grOt_804D69C4 as u32);",
    preg_c=("preg(&YP, sizeof YP); preg(&CS, sizeof CS); preg(GENB, sizeof GENB); preg(PL, sizeof PL); preg(&ARCH, sizeof ARCH); "
            "preg(&USD, sizeof USD); preg(X8, sizeof X8); preg(grOt_803E27E0, sizeof grOt_803E27E0); preg(&grOt_StageData, sizeof grOt_StageData);"),
    preg_r=("preg(&raw const YP, core::mem::size_of_val(&YP)); preg(&raw const CS, core::mem::size_of_val(&CS)); preg(&raw const GENB, core::mem::size_of_val(&GENB)); "
            "preg(&raw const PL, core::mem::size_of_val(&PL)); preg(&raw const ARCH, core::mem::size_of_val(&ARCH)); preg(&raw const USD, core::mem::size_of_val(&USD)); "
            "preg(&raw const X8, core::mem::size_of_val(&X8)); preg(&raw const m::grOt_803E27E0, core::mem::size_of_val(&m::grOt_803E27E0)); "
            "preg(&raw const m::grOt_StageData, core::mem::size_of_val(&m::grOt_StageData));"),
    data=[("grOt_803E27E0", "StageCallbacks", 6), ("grOt_StageData", "StageData", None)],
    view_null={"gen": 3, "subject": 3},
    # the dispatchers run on_init callbacks of the awnings, building and car views (the car one holds a bitfield
    # that a raw-byte alias of the others would read in the other bit order): no Ground::u in their hash
    view={"grOnett_801E3738": "-", "grOnett_801E37F4": "-"},
    hview={"grOnett_801E3738": "-", "grOnett_801E37F4": "-"},
    args={
        "grOnett_801E37F4": [("(int) (fv() % 6)", "(fv() % 6) as i32")],
        "grOnett_801E40E4": [("(void*) &GR[fv() & 3]", "&raw mut GR[(fv() & 3) as usize] as *mut c_void"),
                             ("(int) (fv() % 6)", "(fv() % 6) as i32"), ("&CD", "&raw mut CD"), ("(int) fv()", "fv() as i32"),
                             ("(mpLib_GroundEnum) (fv() % 3)", "fv() % 3"), ("rg()", "rg()")],
        "grOnett_801E54B4": [("(void*) &GR[fv() & 3]", "&raw mut GR[(fv() & 3) as usize] as *mut c_void"),
                             ("(int) (fv() % 3)", "(fv() % 3) as i32"), ("&CD", "&raw mut CD"), ("(int) fv()", "fv() as i32"),
                             ("(mpLib_GroundEnum) (fv() % 3)", "fv() % 3"), ("rg()", "rg()")],
        "grOnett_801E5140": [("(s32) (fv() % 4)", "(fv() % 4) as i32")],
        "grOnett_801E5194": [("&GR[fv() & 3]", "&raw mut GR[(fv() & 3) as usize]"), ("(s32) (fv() % 4)", "(fv() % 4) as i32"),
                             ("(s32) (4 + fv() % 2)", "(4 + fv() % 2) as i32")],
        "grOnett_SfxLookup": [("SFXT", "SFXT.as_ptr() as *mut c_void"), ("(s32) (fv() % 4)", "(fv() % 4) as i32"), ("(s32) (4 + fv() % 2)", "(4 + fv() % 2) as i32")],
        "grOnett_WaitCar": [("&GR[fv() & 3]", "&raw mut GR[(fv() & 3) as usize]"), ("(s8) (fv() % 4)", "(fv() % 4) as i8")],
    },
    pre={
        "grOnett_801E37F4": ("NG = 1;", "NG = true;"),
        "grOnett_801E5538": ("NJ = 1;", "NJ = true;"),
        "grOnett_801E3DA0": (BLD_C, BLD_R),
        "grOnett_801E40E4": (HIT_C, HIT_R),
        "grOnett_801E43E0": (CAR2_C, CAR2_R),
        "grOnett_StartNextCar": (CAR_C, CAR_R),
        "grOnett_WaitCar": (CAR_C, CAR_R),
        "grOnett_801E5030": (CAR_C, CAR_R),
    },
    cases={"grOnett_801E43E0": 3000, "grOnett_801E3DA0": 1000, "grOnett_801E5214": 1000, "grOnett_801E5538": 600,
           "grOnett_801E40E4": 600, "grOnett_801E54B4": 600},
)
