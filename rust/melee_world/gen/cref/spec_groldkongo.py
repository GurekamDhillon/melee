"""Oracle spec of tu::groldkongo (see gen_drv.py for the keys)."""
SETUP_C = """NJ = 0; zero(&YP, sizeof YP); zero(&stage_info, sizeof stage_info); fill_StageInfo(&stage_info);
YP.rframe_bird_wait_a = (s16) (fv() % 40); YP.rframe_bird_wait_b = (s16) (fv() % 40);
YP.rrange_bird_random_offset_y = rg(); YP.rframe_barrel_shoot_a = rg(); YP.rframe_barrel_shoot_b = rg();
YP.rframe_barrel_in = rg();
YP.rframe_barrel_wait_a = (float) (fv() % 40); YP.rframe_barrel_wait_b = (float) (fv() % 40);
YP.rspeed_barrel_rot_accel = 0.5f * (float) (1 + fv() % 10); YP.rspeed_barrel_rot_max = (float) (1 + fv() % 20);
YP.rframe_barrel_roll_a = (float) (fv() % 40); YP.rframe_barrel_roll_b = (float) (fv() % 40);
YP.rrate_barrel_ld = (s16) (1 + fv() % 9); YP.rrate_barrel_l = (s16) (fv() % 10); YP.rrate_barrel_lu = (s16) (fv() % 10);
YP.rrate_barrel_u = (s16) (fv() % 10); YP.rrate_barrel_ru = (s16) (fv() % 10); YP.rrate_barrel_r = (s16) (fv() % 10);
YP.rrate_barrel_rd = (s16) (fv() % 10); YP.rrate_barrel_d = (s16) (fv() % 10);
YP.rframe_barrel_interval_a = (s32) (fv() % 40); YP.rframe_barrel_interval_b = (s32) (fv() % 40);
YP.rspeed_barrel_move_accel = 0.25f * (float) (1 + fv() % 8); YP.rspeed_barrel_move_max = (float) (1 + fv() % 8);
YP.rframe_barrel_stop_a = (s32) (fv() % 40); YP.rframe_barrel_stop_b = (s32) (fv() % 40);
YP.rpower_barrel_attack = (s32) (fv() % 100); YP.rvector_barrel_attack = (s32) (fv() % 400);
YP.rreff_barrel_attack = (s32) (fv() % 100); YP.rrfix_barrel_attack = (s32) (fv() % 100);
YP.rradd_barrel_attack = (s32) (fv() % 100); YP.x68 = (s32) (fv() % 4); YP.x6C = 0;
yakumono_param = &YP;"""
SETUP_R = """NJ = false; m::ext::stage_info = core::mem::zeroed(); YP = core::mem::zeroed(); g::fill_StageInfo(&mut m::ext::stage_info);
YP.rframe_bird_wait_a = (fv() % 40) as i16; YP.rframe_bird_wait_b = (fv() % 40) as i16;
YP.rrange_bird_random_offset_y = rg(); YP.rframe_barrel_shoot_a = rg(); YP.rframe_barrel_shoot_b = rg();
YP.rframe_barrel_in = rg();
YP.rframe_barrel_wait_a = (fv() % 40) as f32; YP.rframe_barrel_wait_b = (fv() % 40) as f32;
YP.rspeed_barrel_rot_accel = 0.5f32 * (1 + fv() % 10) as f32; YP.rspeed_barrel_rot_max = (1 + fv() % 20) as f32;
YP.rframe_barrel_roll_a = (fv() % 40) as f32; YP.rframe_barrel_roll_b = (fv() % 40) as f32;
YP.rrate_barrel_ld = (1 + fv() % 9) as i16; YP.rrate_barrel_l = (fv() % 10) as i16; YP.rrate_barrel_lu = (fv() % 10) as i16;
YP.rrate_barrel_u = (fv() % 10) as i16; YP.rrate_barrel_ru = (fv() % 10) as i16; YP.rrate_barrel_r = (fv() % 10) as i16;
YP.rrate_barrel_rd = (fv() % 10) as i16; YP.rrate_barrel_d = (fv() % 10) as i16;
YP.rframe_barrel_interval_a = (fv() % 40) as i32; YP.rframe_barrel_interval_b = (fv() % 40) as i32;
YP.rspeed_barrel_move_accel = 0.25f32 * (1 + fv() % 8) as f32; YP.rspeed_barrel_move_max = (1 + fv() % 8) as f32;
YP.rframe_barrel_stop_a = (fv() % 40) as i32; YP.rframe_barrel_stop_b = (fv() % 40) as i32;
YP.rpower_barrel_attack = (fv() % 100) as i32; YP.rvector_barrel_attack = (fv() % 400) as i32;
YP.rreff_barrel_attack = (fv() % 100) as i32; YP.rrfix_barrel_attack = (fv() % 100) as i32;
YP.rradd_barrel_attack = (fv() % 100) as i32; YP.x68 = (fv() % 4) as i32; YP.x6C = core::ptr::null_mut();
m::yakumono_param = &raw mut YP;"""

RAND_GOBJ = ("((fv() & 3) == 0 ? 0 : &GO[fv() & 3])",
             "if fv() & 3 == 0 { core::ptr::null_mut() } else { &raw mut super::GO[(fv() & 3) as usize] }")

SPEC = dict(
    src="src/melee/gr/groldkongo.c",
    types=["StageInfo", "StageCallbacks", "StageData", "lbColl_80008D30_arg1"],
    null_ret_ok=["HSD_GObj_SetupProc", "grZakoGenerator_801CAE04", "efSync_Spawn"],
    custom_stubs={
        "efSync_Spawn": dict(
            c="void* efSync_Spawn(s32 a0, HSD_GObj* a1, ...) { __builtin_va_list ap; Vec* v; __builtin_va_start(ap, a1); "
              "v = __builtin_va_arg(ap, Vec*); __builtin_va_end(ap); T(@ID@); T((u32) a0); T(PI(a1)); TV(v); return 0; }",
            ext="fn efSync_Spawn(a0: i32, a1: *mut HSD_GObj, a2: *mut Vec) -> *mut c_void;",
            hook="|a0, a1, a2| { t(@ID@); t(a0 as u32); t(pi(a1)); super::tv(a2); core::ptr::null_mut() }"),
    },
    ret={
        "Ground_GetYakumonoParam": ("&YP", "&raw mut super::YP as *mut c_void"),
        "Ground_GetStageGObj": RAND_GOBJ,
        "Ground_GetMapGObj": RAND_GOBJ,
        "Ground_801C3FA4": ("&JP[fv() % 16]", "&raw mut super::JP[(fv() % 16) as usize]"),
    },
    effects={
        "lb_8000B1CC": ("WV(a2);", "super::wv(a2);"),
        "Ground_801C4D70": ("TV(a1);", "super::tv(a1);"),
        "Ground_801C4DA0": ("WV(a0); *a1 = rg();", "super::wv(a0); *a1 = rg();"),
        "ftLib_GetPos": ("WV(a1);", "super::wv(a1);"),
        "ftCo_8009EC70": ("TV(a1); T(RH(a2, 36));", "super::tv(a1); t(rh(&*(a2 as *const [u8; 36])));"),
        "Camera_LogicBounds": ("*a0 = rg() * 300.0f; *a1 = rg() * 300.0f; *a2 = rg() * 300.0f;",
                               "*a0 = rg() * 300.0; *a1 = rg() * 300.0; *a2 = rg() * 300.0;"),
    },
    decl_c="static struct grOldKongo_YakumonoParam YP;\nstatic int NJ;",
    decl_r="static mut YP: m::grOldKongo_YakumonoParam = unsafe { core::mem::zeroed() };\nstatic mut NJ: bool = false;",
    setup_c=SETUP_C,
    setup_r=SETUP_R,
    hash_c="hash_StageInfo(&stage_info); HM(RH(&YP, sizeof YP)); HM(PI(yakumono_param));",
    hash_r="g::hash_StageInfo(&m::ext::stage_info); hm(rh(&YP)); hm(pi(m::yakumono_param));",
    preg_c="preg(&YP, sizeof YP); preg(stage_callbacks, sizeof stage_callbacks); preg(&grOk_StageData, sizeof grOk_StageData); preg(grOk_803E6580, sizeof grOk_803E6580);",
    preg_r="preg(&raw const YP, core::mem::size_of_val(&YP)); preg(&raw const m::stage_callbacks, core::mem::size_of_val(&m::stage_callbacks)); preg(&raw const m::grOk_StageData, core::mem::size_of_val(&m::grOk_StageData)); preg(&raw const m::grOk_803E6580, core::mem::size_of_val(&m::grOk_803E6580));",
    data=[("stage_callbacks", "StageCallbacks", 4), ("grOk_StageData", "StageData", None), ("grOk_803E6580", "raw", None)],
    view_null={"keep": 4},
    # the dispatchers run on_init callbacks of several views; the taru and unk views both lack bitfields
    view={"grOldKongo_8020F46C": "-", "setupStageCallbacks": "-"},
    hview={"grOldKongo_8020F46C": "taru", "setupStageCallbacks": "taru"},
    args={
        "setupStageCallbacks": [("(int) (fv() % 4)", "(fv() % 4) as i32")],
    },
    pre={
        # state 2/3 of the rotation needs a nonzero angular velocity (HSD_ASSERT otherwise); a hit in flight needs a fighter
        "stageGObj1_GObjProc": (
            "for (i = 0; i < 4; i++) { if (fv() % 3 == 0) GR[i].u.taru.xC6 = (s16) (2 + fv() % 3); if (fv() % 3 == 0) GR[i].u.taru.hit_timer = (s16) (10 + fv() % 3); if (fv() & 1) GO[i].p_link = 8; "
            "  if ((GR[i].u.taru.xC4 == 2 || GR[i].u.taru.xC4 == 3) && !(GR[i].u.taru.xE4 > 0.0f) && !(GR[i].u.taru.xE4 < 0.0f)) GR[i].u.taru.xE4 = 0.5f;\n"
            "  if (GR[i].u.taru.keep == 0 && GR[i].u.taru.xC6 >= 2 && GR[i].u.taru.xC6 <= 3) GR[i].u.taru.keep = &GO[1]; }",
            "for i in 0..4 { if fv() % 3 == 0 { GR[i].u.taru.xC6 = (2 + fv() % 3) as i16; } if fv() % 3 == 0 { GR[i].u.taru.hit_timer = (10 + fv() % 3) as i16; } if fv() & 1 != 0 { GO[i].p_link = 8; } "
            "  if (GR[i].u.taru.xC4 == 2 || GR[i].u.taru.xC4 == 3) && !(GR[i].u.taru.xE4 > 0.0) && !(GR[i].u.taru.xE4 < 0.0) { GR[i].u.taru.xE4 = 0.5; }\n"
            "  if GR[i].u.taru.keep.is_null() && GR[i].u.taru.xC6 >= 2 && GR[i].u.taru.xC6 <= 3 { GR[i].u.taru.keep = &raw mut GO[1]; } }"),
        "grOldKongo_802105C8": (
            "for (i = 0; i < 4; i++) { if (GR[i].u.taru.keep == 0) GR[i].u.taru.keep = &GO[2]; if (fv() & 1) GO[2].p_link = 8; }",
            "for i in 0..4 { if GR[i].u.taru.keep.is_null() { GR[i].u.taru.keep = &raw mut GO[2]; } if fv() & 1 != 0 { GO[2].p_link = 8; } }"),
    },
    cases={"stageGObj1_GObjProc": 1500, "stageGObj2_GObjProc": 800, "grOldKongo_80210454": 600, "grOldKongo_80210650": 600},
)
