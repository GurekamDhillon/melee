"""Oracle spec of tu::grpstadium (see gen_drv.py for the keys).

World: the usual GO/GR/JP plus
  XG[10] / XGR[10]  the stage objects `Ground_GetStageGObj(id)` / `Ground_GetMapGObj(id)` hand out (id % 10);
                    each has its own Ground, hashed through the view its id uses (1 display, 2 stadium, 3..5 stadium
                    bits, 6 and 9 stadium9)
  TG / TWB / TX     the jumbotron text objects (gobj -> TextWrapper -> two HSD_Text)
  IG / IWB          the camera feeds (gobj -> ImageDescWrapper), COB the camera object they render with
  TOB / MOB / DP / IDS   the texture objects the jumbotron's image desc is looked up in
"""
CHR = chr(10)


def hash_lines(items):
    """items: (kind, c_expr, rust_expr) -> (c_text, rust_text); kind u32 / f32 / pi / fi"""
    c, r = [], []
    for k, ce, re_ in items:
        if k == "u32":
            c.append(f"HM((u32) ({ce}));"); r.append(f"hm(({re_}) as u32);")
        elif k == "f32":
            c.append(f"HM(B({ce}));"); r.append(f"hm(({re_}).to_bits());")
        elif k == "pi":
            c.append(f"HM(PI((const void*) ({ce})));"); r.append(f"hm(pi({re_}));")
        elif k == "fi":
            c.append(f"HM(FI((const void*) ({ce})));"); r.append(f"hm(fi({re_}));")
        else:
            raise SystemExit(k)
    return " ".join(c), " ".join(r)


def ground_common(i):
    g = f"XGR[{i}]"
    return [("fi", f"{g}.x8_callback", f"{g}.x8_callback"), ("fi", f"{g}.xC_callback", f"{g}.xC_callback"),
            ("fi", f"{g}.x1C_callback", f"{g}.x1C_callback"),
            ("u32", f"{g}.x10_flags.b5", f"{g}.x10_flags.b5()"), ("u32", f"{g}.x11_flags.b012", f"{g}.x11_flags.b012()")]


def display_view(g):
    u = f"{g}.u.display"
    out = [("u32", f"{u}.xC4_b0", f"{u}.xC4_b0()"), ("u32", f"{u}.xC4_b1", f"{u}.xC4_b1()")]
    for f in ("xC8", "xCC", "xD0", "xD4", "xD8", "xDC", "xF4"):
        out.append(("pi", f"{u}.{f}", f"{u}.{f}"))
    for f in ("xE0", "xE4", "xE6", "xE8", "xEA", "xEC", "xEE", "xF0", "xF2"):
        out.append(("u32", f"{u}.{f}", f"{u}.{f}"))
    for f in ("xF8_0", "xF8_1", "xF8_2"):
        out.append(("u32", f"{u}.{f}", f"{u}.{f}()"))
    return out


def stadium_view(g):
    u = f"{g}.u.stadium"
    out = [("u32", f"{u}.xC4_b0", f"{u}.xC4_b0()"), ("u32", f"{u}.xC4_b1", f"{u}.xC4_b1()"), ("u32", f"{u}.xC8", f"{u}.xC8"),
           ("pi", f"{u}.xCC", f"{u}.xCC"), ("pi", f"{u}.xD0", f"{u}.xD0"), ("f32", f"{u}.xD4", f"{u}.xD4"),
           ("u32", f"{u}.xD8", f"{u}.xD8")]
    for f in ("xDC", "xDE", "xE0", "xE2"):
        out.append(("u32", f"{u}.{f}", f"{u}.{f}"))
    out += [("pi", f"{u}.xE4", f"{u}.xE4"), ("pi", f"{u}.xE8", f"{u}.xE8")]
    return out


def stadium9_view(g):
    u = f"{g}.u.stadium9"
    return [("u32", f"{u}.xC4_b0", f"{u}.xC4_b0()"), ("u32", f"{u}.xC4_b1", f"{u}.xC4_b1()"), ("pi", f"{u}.xC8", f"{u}.xC8"),
            ("pi", f"{u}.xCC_gobj", f"{u}.xCC_gobj"), ("pi", f"{u}.xD0_gobj", f"{u}.xD0_gobj"), ("pi", f"{u}.xD4_jobj", f"{u}.xD4_jobj")]


def build_hash():
    items = [("pi", "yakumono_param", "m::yakumono_param"), ("pi", "HSD_SisLib_804D1124[1]", "m::ext::HSD_SisLib_804D1124[1]"),
             ("pi", "SISB[2].textures", "SISB[2].textures"), ("pi", "SISB[2].kerning", "SISB[2].kerning")]
    for i in range(10):
        items += ground_common(i)
    items += display_view("XGR[1]")
    for i in (2, 3, 4, 5):
        items += stadium_view(f"XGR[{i}]")
    for i in (6, 9):
        items += stadium9_view(f"XGR[{i}]")
    for i in range(4):
        for g in ("TG", "IG"):
            items += [("u32", f"{g}[{i}].gxlink_prios", f"{g}[{i}].gxlink_prios"), ("fi", f"{g}[{i}].render_cb", f"{g}[{i}].render_cb"),
                      ("pi", f"{g}[{i}].user_data", f"{g}[{i}].user_data"), ("pi", f"{g}[{i}].hsd_obj", f"{g}[{i}].hsd_obj")]
        for w in (f"IWB[{i}]", f"TWB[{i}]"):
            d = f"{w}.desc"
            items += [("pi", f"{d}.image_ptr", f"{d}.image_ptr"), ("u32", f"{d}.width", f"{d}.width"), ("u32", f"{d}.height", f"{d}.height"),
                      ("u32", f"{d}.format", f"{d}.format"), ("u32", f"{d}.mipmap", f"{d}.mipmap"),
                      ("f32", f"{d}.minLOD", f"{d}.minLOD"), ("f32", f"{d}.maxLOD", f"{d}.maxLOD")]
        items += [("u32", f"IWB[{i}].flag", f"IWB[{i}].flag()"), ("u32", f"IWB[{i}].x1A", f"IWB[{i}].x1A"), ("u32", f"IWB[{i}].x1C", f"IWB[{i}].x1C"),
                  ("pi", f"TWB[{i}].win_static_p", f"TWB[{i}].win_static_p"), ("pi", f"TWB[{i}].win_dynamic_p", f"TWB[{i}].win_dynamic_p")]
        t = f"TX[{i}]"
        items += [("u32", f"{t}.default_fitting", f"{t}.default_fitting"), ("u32", f"{t}.default_kerning", f"{t}.default_kerning"),
                  ("u32", f"{t}.default_alignment", f"{t}.default_alignment"), ("f32", f"{t}.x34.x", f"{t}.x34.x"), ("f32", f"{t}.x34.y", f"{t}.x34.y"),
                  ("u32", f"{t}.text_color.r", f"{t}.text_color.r"), ("u32", f"{t}.text_color.g", f"{t}.text_color.g"),
                  ("u32", f"{t}.text_color.b", f"{t}.text_color.b"), ("u32", f"{t}.text_color.a", f"{t}.text_color.a"),
                  ("f32", f"{t}.pos_z", f"{t}.pos_z"), ("f32", f"{t}.box_size_x", f"{t}.box_size_x"), ("f32", f"{t}.box_size_y", f"{t}.box_size_y"),
                  ("pi", f"TOB[{i}].imagedesc", f"TOB[{i}].imagedesc")]
    c, r = hash_lines(items)
    c = "hash_StageInfo(&stage_info); hash_CmSubject(&CS[0]); hash_CmSubject(&CS[1]); " + c
    r = "g::hash_StageInfo(&m::ext::stage_info); g::hash_CmSubject(&CS[0]); g::hash_CmSubject(&CS[1]); " + r
    return c, r


HASH_C, HASH_R = build_hash()

# ---------------------------------------------------------------------------------------------------
DECL_C = CHR.join([
    "static struct grPStadium_YakumonoParam YP;", "static HSD_GObj XG[10];", "static Ground XGR[10];",
    "static HSD_GObj TG[4];", "static HSD_GObj IG[4];", "static TextWrapper TWB[4];", "static ImageDescWrapper IWB[4];",
    "static HSD_Text TX[4];", "static CmSubject CS[2];", "static HSD_CObj COB;", "static HSD_GObj CAMG;", "static SIS SISB[4];",
    "static StartMeleeRules SMR;", "static HSD_TObj TOB[4];", "static HSD_MObj MOB[4];", "static HSD_ImageDesc IDS[4];",
    "static HSD_DObj DP[8];", "static UnkArchiveStruct ARCH;", "static u8 ARCHB[64];", "static u8 GEN[2][128];",
    "static u8 MEMB[256];", "static HSD_MObj* MOUT;", "static int NG, GMF;", "static HSD_JObj RT[6];",
    "static const char* STRS[4] = { \"Mario\", \"Luigi\", \"Kirby\", 0 };",
    """static void disp_init(Ground* g)
{
    g->u.display.xC8 = &TOB[fv() & 3]; g->u.display.xCC = ((fv() % 5) == 0) ? 0 : &MOB[fv() & 3]; g->u.display.xD0 = &IDS[fv() & 3];
    g->u.display.xD4 = &TG[fv() & 3]; g->u.display.xD8 = &IG[0]; g->u.display.xDC = &IG[1];
    g->u.display.xE0 = (s32) (fv() % 6) - 2; g->u.display.xE4 = (s16) (fv() % 18);
    g->u.display.xE6 = (s16) (((fv() & 3) == 0) ? 0x40 : (1 << (fv() % 8)));
    g->u.display.xE8 = (s16) fv(); g->u.display.xEA = (s16) (fv() % 18); g->u.display.xEC = (s16) fv();
    g->u.display.xEE = (s16) (((fv() % 5) == 0) ? 99 : fv() % 6); g->u.display.xF0 = (s16) (fv() % 4); g->u.display.xF2 = (s16) (fv() % 7);
    g->u.display.xF4 = ((fv() % 3) == 0) ? 0 : &CS[fv() & 1];
    g->u.display.xF8_0 = fv() & 1; g->u.display.xF8_1 = fv() & 1; g->u.display.xF8_2 = fv() & 1;
    g->u.display.xC4_b0 = fv() & 1; g->u.display.xC4_b1 = fv() & 1;
}
static void stad_init(Ground* g)
{
    static const s16 K[5] = { 3, 4, 5, 6, 9 };
    g->u.stadium.xC4_b0 = fv() & 1; g->u.stadium.xC4_b1 = fv() & 1; g->u.stadium.xC8 = fv();
    g->u.stadium.xCC = ((fv() & 1) == 0) ? 0 : &MOB[fv() & 3]; g->u.stadium.xD0 = ((fv() % 3) == 0) ? 0 : &ARCH;
    g->u.stadium.xD4 = ((fv() % 3) == 0) ? 0.0f : (float) (fv() % 400);
    g->u.stadium.xD8 = (s32) (fv() % 12) - 4; g->u.stadium.xDC = (s16) (fv() % 7);
    g->u.stadium.xDE = K[fv() % 5]; g->u.stadium.xE0 = K[fv() % 5]; g->u.stadium.xE2 = (s16) (fv() % 10) - 1;
    g->u.stadium.xE4 = &XG[5]; g->u.stadium.xE8 = &XG[6];
}"""])

DECL_R = CHR.join([
    "static mut YP: m::GrPStadiumYakumonoParam = unsafe { core::mem::zeroed() };", "static mut XG: [HSD_GObj; 10] = unsafe { core::mem::zeroed() };",
    "static mut XGR: [Ground; 10] = unsafe { core::mem::zeroed() };", "static mut TG: [HSD_GObj; 4] = unsafe { core::mem::zeroed() };",
    "static mut IG: [HSD_GObj; 4] = unsafe { core::mem::zeroed() };", "static mut TWB: [m::TextWrapper; 4] = unsafe { core::mem::zeroed() };",
    "static mut IWB: [m::ImageDescWrapper; 4] = unsafe { core::mem::zeroed() };", "static mut TX: [HSD_Text; 4] = unsafe { core::mem::zeroed() };",
    "static mut CS: [CmSubject; 2] = unsafe { core::mem::zeroed() };", "static mut COB: HSD_CObj = unsafe { core::mem::zeroed() };",
    "static mut CAMG: HSD_GObj = unsafe { core::mem::zeroed() };", "static mut SISB: [SIS; 4] = unsafe { core::mem::zeroed() };",
    "static mut SMR: StartMeleeRules = unsafe { core::mem::zeroed() };", "static mut TOB: [HSD_TObj; 4] = unsafe { core::mem::zeroed() };",
    "static mut MOB: [HSD_MObj; 4] = unsafe { core::mem::zeroed() };", "static mut IDS: [HSD_ImageDesc; 4] = unsafe { core::mem::zeroed() };",
    "static mut DP: [HSD_DObj; 8] = unsafe { core::mem::zeroed() };", "static mut ARCH: UnkArchiveStruct = unsafe { core::mem::zeroed() };",
    "static mut ARCHB: [u8; 64] = [0; 64];", "static mut GEN: [[u8; 128]; 2] = [[0; 128]; 2];",
    "static mut MEMB: [u8; 256] = [0; 256];", "static mut MOUT: *mut HSD_MObj = core::ptr::null_mut();",
    "static mut NG: bool = false;", "static mut GMF: bool = false;", "static mut RT: [HSD_JObj; 6] = unsafe { core::mem::zeroed() };",
    "static STRS: [*const i8; 4] = [b\"Mario\\0\".as_ptr() as *const i8, b\"Luigi\\0\".as_ptr() as *const i8, b\"Kirby\\0\".as_ptr() as *const i8, core::ptr::null()];",
    "unsafe impl Sync for StrsSync {} struct StrsSync;",
    """unsafe fn disp_init(g: *mut Ground) {
    (*g).u.display.xC8 = &raw mut TOB[(fv() & 3) as usize];
    (*g).u.display.xCC = if fv() % 5 == 0 { core::ptr::null_mut() } else { &raw mut MOB[(fv() & 3) as usize] };
    (*g).u.display.xD0 = &raw mut IDS[(fv() & 3) as usize];
    (*g).u.display.xD4 = &raw mut TG[(fv() & 3) as usize]; (*g).u.display.xD8 = &raw mut IG[0]; (*g).u.display.xDC = &raw mut IG[1];
    (*g).u.display.xE0 = (fv() % 6) as i32 - 2; (*g).u.display.xE4 = (fv() % 18) as i16;
    (*g).u.display.xE6 = (if fv() & 3 == 0 { 0x40 } else { 1i32 << (fv() % 8) }) as i16;
    (*g).u.display.xE8 = fv() as i16; (*g).u.display.xEA = (fv() % 18) as i16; (*g).u.display.xEC = fv() as i16;
    (*g).u.display.xEE = (if fv() % 5 == 0 { 99 } else { fv() % 6 }) as i16; (*g).u.display.xF0 = (fv() % 4) as i16; (*g).u.display.xF2 = (fv() % 7) as i16;
    (*g).u.display.xF4 = if fv() % 3 == 0 { core::ptr::null_mut() } else { &raw mut CS[(fv() & 1) as usize] };
    (*g).u.display.set_xF8_0((fv() & 1) as u8); (*g).u.display.set_xF8_1((fv() & 1) as u8); (*g).u.display.set_xF8_2((fv() & 1) as u8);
    (*g).u.display.set_xC4_b0((fv() & 1) as u8); (*g).u.display.set_xC4_b1((fv() & 1) as u8);
}
unsafe fn stad_init(g: *mut Ground) {
    const K: [i16; 5] = [3, 4, 5, 6, 9];
    (*g).u.stadium.set_xC4_b0((fv() & 1) as u8); (*g).u.stadium.set_xC4_b1((fv() & 1) as u8); (*g).u.stadium.xC8 = fv() as usize;
    (*g).u.stadium.xCC = if fv() & 1 == 0 { core::ptr::null_mut() } else { &raw mut MOB[(fv() & 3) as usize] };
    (*g).u.stadium.xD0 = if fv() % 3 == 0 { core::ptr::null_mut() } else { &raw mut ARCH };
    (*g).u.stadium.xD4 = if fv() % 3 == 0 { 0.0 } else { (fv() % 400) as f32 };
    (*g).u.stadium.xD8 = (fv() % 12) as i32 - 4; (*g).u.stadium.xDC = (fv() % 7) as i16;
    (*g).u.stadium.xDE = K[(fv() % 5) as usize]; (*g).u.stadium.xE0 = K[(fv() % 5) as usize]; (*g).u.stadium.xE2 = (fv() % 10) as i16 - 1;
    (*g).u.stadium.xE4 = &raw mut XG[5]; (*g).u.stadium.xE8 = &raw mut XG[6];
}"""])
# the `StrsSync` line above is a no-op marker: a `static` of raw pointers needs Sync
DECL_R = DECL_R.replace("static STRS: [*const i8; 4]", "static mut STRS: [*const i8; 4]").replace("unsafe impl Sync for StrsSync {} struct StrsSync;" + CHR, "")

SETUP_C = """NG = 0; GMF = 0; zero(&stage_info, sizeof stage_info); fill_StageInfo(&stage_info);
zero(&YP, sizeof YP); zero(XG, sizeof XG); zero(XGR, sizeof XGR); zero(TG, sizeof TG); zero(IG, sizeof IG); zero(TWB, sizeof TWB); zero(IWB, sizeof IWB);
zero(TX, sizeof TX); zero(CS, sizeof CS); zero(&COB, sizeof COB); zero(&CAMG, sizeof CAMG); zero(SISB, sizeof SISB); zero(&SMR, sizeof SMR);
zero(TOB, sizeof TOB); zero(MOB, sizeof MOB); zero(IDS, sizeof IDS); zero(DP, sizeof DP); zero(&ARCH, sizeof ARCH); zero(ARCHB, sizeof ARCHB);
zero(GEN, sizeof GEN); zero(MEMB, sizeof MEMB); MOUT = 0;
YP.x0 = (int) (fv() % 30); YP.x4 = (int) (fv() % 30); YP.x8 = (int) (fv() % 30); YP.xC = (int) (fv() % 30);
YP.x10 = (int) (fv() % 8); YP.x14 = (int) (1 + fv() % 20); YP.x18 = (int) (fv() % 8);
YP.r = (u8) fv(); YP.g = (u8) fv(); YP.b = (u8) fv();
YP.x20 = fv() % 50; YP.x24 = fv() % 50; YP.x28 = fv() % 50; YP.x2C = fv() % 50; YP.x30 = fv() % 50; YP.x34 = fv() % 50;
YP.x38 = fv() % 50; YP.x3C = fv() % 50; YP.x40 = fv() % 50; YP.x44 = fv() % 50;
YP.x48 = (s16) (1 + fv() % 20); YP.x4A = (s16) (1 + fv() % 20); YP.x4C = (s16) (1 + fv() % 20); YP.x4E = (s16) (1 + fv() % 20); YP.x50 = (s16) (fv() % 5);
yakumono_param = &YP;
for (i = 0; i < 10; i++) { XG[i].user_data = &XGR[i]; XG[i].hsd_obj = &JP[i]; }
for (i = 0; i < 4; i++) {
    TG[i].user_data = &TWB[i]; TG[i].hsd_obj = &COB; IG[i].user_data = &IWB[i]; IG[i].hsd_obj = &COB;
    TWB[i].win_static_p = &TX[i]; TWB[i].win_dynamic_p = &TX[(i + 1) & 3];
    IWB[i].flag = fv() & 1; IWB[i].x1A = (u16) fv(); IWB[i].x1C = (u16) fv();
}
CAMG.hsd_obj = &COB;
COB.viewport.xmin = rg() * 100.0f; COB.viewport.xmax = rg() * 100.0f; COB.viewport.ymin = rg() * 100.0f; COB.viewport.ymax = rg() * 100.0f;
for (i = 0; i < 5; i++) HSD_SisLib_804D1124[i] = 0;
HSD_SisLib_804D1124[1] = SISB;
HSD_GObj_CameraKind = (u8) (fv() & 15);
SMR.time_limit = fv() % 3; SMR.timer_enabled = fv() & 1;
ARCH.unk0 = (HSD_Archive*) ARCHB; ARCH.unk4 = 0;
for (i = 0; i < 4; i++) { TOB[i].next = (i + 1 < 4 && (fv() & 1)) ? &TOB[i + 1] : 0; TOB[i].imagedesc = &IDS[fv() & 3]; MOB[i].tobj = ((fv() % 4) == 0) ? 0 : &TOB[fv() & 3]; }
for (i = 0; i < 8; i++) { DP[i].next = (i + 1 < 8 && (fv() & 1)) ? &DP[i + 1] : 0; DP[i].mobj = ((fv() % 4) == 0) ? 0 : &MOB[fv() & 3]; }
zero(RT, sizeof RT);
for (i = 0; i < 6; i++) { fill_HSD_JObj(&RT[i]); RT[i].flags &= ~(JOBJ_USE_QUATERNION | JOBJ_INSTANCE); if (fv() % 4 != 0) RT[i].flags &= ~(JOBJ_PTCL | JOBJ_SPLINE); RT[i].u.dobj = ((fv() & 3) == 0) ? 0 : &DP[fv() & 7]; if (fv() % 5 == 0) RT[i].flags |= JOBJ_INSTANCE; RT[i].next = 0; RT[i].parent = 0; RT[i].child = 0; }
RT[0].child = &RT[1]; RT[1].parent = &RT[0]; RT[1].next = &RT[2]; RT[1].child = &RT[5]; RT[5].parent = &RT[1]; RT[2].parent = &RT[0]; RT[2].child = &RT[3]; RT[3].parent = &RT[2]; RT[3].next = &RT[4]; RT[4].parent = &RT[2];
XG[1].hsd_obj = &RT[0];
disp_init(&XGR[1]); stad_init(&XGR[2]);"""
SETUP_R = """NG = false; GMF = false; m::ext::stage_info = core::mem::zeroed(); g::fill_StageInfo(&mut m::ext::stage_info);
YP = core::mem::zeroed(); XG = core::mem::zeroed(); XGR = core::mem::zeroed(); TG = core::mem::zeroed(); IG = core::mem::zeroed(); TWB = core::mem::zeroed(); IWB = core::mem::zeroed();
TX = core::mem::zeroed(); CS = core::mem::zeroed(); COB = core::mem::zeroed(); CAMG = core::mem::zeroed(); SISB = core::mem::zeroed(); SMR = core::mem::zeroed();
TOB = core::mem::zeroed(); MOB = core::mem::zeroed(); IDS = core::mem::zeroed(); DP = core::mem::zeroed(); ARCH = core::mem::zeroed(); ARCHB = [0; 64];
GEN = core::mem::zeroed(); MEMB = [0; 256]; MOUT = core::ptr::null_mut();
YP.x0 = (fv() % 30) as i32; YP.x4 = (fv() % 30) as i32; YP.x8 = (fv() % 30) as i32; YP.xC = (fv() % 30) as i32;
YP.x10 = (fv() % 8) as i32; YP.x14 = (1 + fv() % 20) as i32; YP.x18 = (fv() % 8) as i32;
YP.r = fv() as u8; YP.g = fv() as u8; YP.b = fv() as u8;
YP.x20 = fv() % 50; YP.x24 = fv() % 50; YP.x28 = fv() % 50; YP.x2C = fv() % 50; YP.x30 = fv() % 50; YP.x34 = fv() % 50;
YP.x38 = fv() % 50; YP.x3C = fv() % 50; YP.x40 = fv() % 50; YP.x44 = fv() % 50;
YP.x48 = (1 + fv() % 20) as i16; YP.x4A = (1 + fv() % 20) as i16; YP.x4C = (1 + fv() % 20) as i16; YP.x4E = (1 + fv() % 20) as i16; YP.x50 = (fv() % 5) as i16;
m::yakumono_param = &raw mut YP;
for i in 0..10usize { XG[i].user_data = (&raw mut XGR[i]) as *mut c_void; XG[i].hsd_obj = (&raw mut JP[i]) as *mut c_void; }
for i in 0..4usize {
    TG[i].user_data = (&raw mut TWB[i]) as *mut c_void; TG[i].hsd_obj = (&raw mut COB) as *mut c_void; IG[i].user_data = (&raw mut IWB[i]) as *mut c_void; IG[i].hsd_obj = (&raw mut COB) as *mut c_void;
    TWB[i].win_static_p = &raw mut TX[i]; TWB[i].win_dynamic_p = &raw mut TX[(i + 1) & 3];
    IWB[i].set_flag((fv() & 1) as u16); IWB[i].x1A = fv() as u16; IWB[i].x1C = fv() as u16;
}
CAMG.hsd_obj = (&raw mut COB) as *mut c_void;
COB.viewport.xmin = rg() * 100.0f32; COB.viewport.xmax = rg() * 100.0f32; COB.viewport.ymin = rg() * 100.0f32; COB.viewport.ymax = rg() * 100.0f32;
for i in 0..5usize { m::ext::HSD_SisLib_804D1124[i] = core::ptr::null_mut(); }
m::ext::HSD_SisLib_804D1124[1] = (&raw mut SISB) as *mut SIS;
m::ext::HSD_GObj_CameraKind = (fv() & 15) as u8;
SMR.time_limit = fv() % 3; SMR.set_timer_enabled(fv() & 1);
ARCH.unk0 = (&raw mut ARCHB) as *mut HSD_Archive; ARCH.unk4 = core::ptr::null_mut();
for i in 0..4usize { TOB[i].next = if i + 1 < 4 && (fv() & 1) != 0 { &raw mut TOB[i + 1] } else { core::ptr::null_mut() }; TOB[i].imagedesc = &raw mut IDS[(fv() & 3) as usize]; MOB[i].tobj = if fv() % 4 == 0 { core::ptr::null_mut() } else { &raw mut TOB[(fv() & 3) as usize] }; }
for i in 0..8usize { DP[i].next = if i + 1 < 8 && (fv() & 1) != 0 { &raw mut DP[i + 1] } else { core::ptr::null_mut() }; DP[i].mobj = if fv() % 4 == 0 { core::ptr::null_mut() } else { &raw mut MOB[(fv() & 3) as usize] }; }
RT = core::mem::zeroed();
for i in 0..6usize { g::fill_HSD_JObj(&mut RT[i]); RT[i].flags &= !((JOBJ_USE_QUATERNION | JOBJ_INSTANCE) as u32); if fv() % 4 != 0 { RT[i].flags &= !((JOBJ_PTCL | JOBJ_SPLINE) as u32); } RT[i].u.dobj = if fv() & 3 == 0 { core::ptr::null_mut() } else { &raw mut DP[(fv() & 7) as usize] }; if fv() % 5 == 0 { RT[i].flags |= JOBJ_INSTANCE as u32; } RT[i].next = core::ptr::null_mut(); RT[i].parent = core::ptr::null_mut(); RT[i].child = core::ptr::null_mut(); }
RT[0].child = &raw mut RT[1]; RT[1].parent = &raw mut RT[0]; RT[1].next = &raw mut RT[2]; RT[1].child = &raw mut RT[5]; RT[5].parent = &raw mut RT[1]; RT[2].parent = &raw mut RT[0]; RT[2].child = &raw mut RT[3]; RT[3].parent = &raw mut RT[2]; RT[3].next = &raw mut RT[4]; RT[4].parent = &raw mut RT[2];
XG[1].hsd_obj = (&raw mut RT[0]) as *mut c_void;
disp_init(&raw mut XGR[1]); stad_init(&raw mut XGR[2]);"""

# per-function setup (run after the world is built, before the arguments are drawn)
DISP_C = "for (i = 0; i < 4; i++) disp_init(&GR[i]);"
DISP_R = "for i in 0..4usize { disp_init(&raw mut GR[i]); }"
STAD_C = "for (i = 0; i < 4; i++) stad_init(&GR[i]);"
STAD_R = "for i in 0..4usize { stad_init(&raw mut GR[i]); }"
GMF_C = "GMF = (fv() % 3 == 0);"
GMF_R = "GMF = fv() % 3 == 0;"
KIND_C = "stage_info.grkind = (fv() % 4 != 0) ? Gr_Kind_PStadium : (fv() % 8);"
KIND_R = "m::ext::stage_info.grkind = if fv() % 4 != 0 { Gr_Kind_PStadium as _ } else { fv() % 8 };"
NG_C = "NG = 1;"
NG_R = "NG = true;"


def both(*pairs):
    return (CHR.join(p[0] for p in pairs), CHR.join(p[1] for p in pairs))


SPEC = dict(
    src="src/melee/gr/grpstadium.c",
    # grStadium_801D4548 case 2: `u.display.xD8 = NULL` is the 4-byte countdown `u.stadium.xD8` (gp+D8) on the GC
    # layout; the two views differ natively (see the module comment of tu::grpstadium)
    # grStadium_801D3BBC: `colors.entries[n].packed` over four bytes is big-endian on the GameCube
    patch=[
        ("temp_r31->u.display.xD8 = NULL;", "temp_r31->u.stadium.xD8 = 0;"),
        ("vertical_offset = colors.entries[player_num].packed;",
         "vertical_offset = ((u32) colors.entries[player_num].color.r << 24) | ((u32) colors.entries[player_num].color.g << 16) | "
         "((u32) colors.entries[player_num].color.b << 8) | (u32) colors.entries[player_num].color.a;"),
    ],
    types=["StageInfo", "StageCallbacks", "StageData", "CmSubject"],
    null_ret_ok=["HSD_GObj_SetupProc", "grZakoGenerator_801CAE04", "Ground_801C3FA4", "Ground_GetMapGObj", "Ground_GetStageGObj",
                 "lbVector_Add", "lbVector_Diff", "lbVector_WorldToScreen", "lbDvd_80017740", "lbDvd_GetPreloadedArchive", "lb_800121FC",
                 "grLib_801C96F8", "grLib_801C9808", "HSD_SisLib_803A6530", "gm_80160A60", "Player_GetEntity", "Camera_80029020",
                 "HSD_MemAlloc", "HSD_ArchiveGetPublicAddress", "grDatFiles_801C6478", "HSD_SisLib_803A5ACC", "HSD_SisLib_803A6754"],
    # `size_t xC8` is 4 bytes in the oracle's C and `usize` (8 bytes) in melee_types: &xC8 differs by 4 inside the Ground
    trace={"lbFile_80016580": {2: ("PI((const void*) a2)", "pi(a2) - 4")}},
    custom_stubs={
        "HSD_SisLib_803A6B98": dict(
            c="int HSD_SisLib_803A6B98(HSD_Text* a0, float a1, float a2, const char* a3, ...) { __builtin_va_list ap; const char* p; "
              "__builtin_va_start(ap, a3); T(@ID@); T(PI(a0)); T(B(a1)); T(B(a2)); T(SH(a3)); "
              "for (p = a3; *p; p++) { if (*p == 37) { p++; if (*p == 37) continue; "
              "while (*p == 45 || *p == 43 || *p == 32 || *p == 35 || *p == 46 || (*p >= 48 && *p <= 57)) p++; "
              "if (*p == 115) { const char* s = __builtin_va_arg(ap, const char*); T(SH(s)); } else { T((u32) __builtin_va_arg(ap, int)); } } } "
              "__builtin_va_end(ap); return 0; }",
            ext="fn HSD_SisLib_803A6B98(a0: *mut HSD_Text, a1: f32, a2: f32, a3: *const i8, a4: usize, a5: usize, a6: usize) -> i32;",
            hook="|a0, a1, a2, a3, a4, a5, a6| { t(@ID@); t(pi(a0)); t(a1.to_bits()); t(a2.to_bits()); t(sh(a3 as *const u8)); "
                 "let args = [a4, a5, a6]; let mut n = 0usize; let mut p = a3 as *const u8; "
                 "while *p != 0 { if *p == 37 { p = p.add(1); if *p == 37 { p = p.add(1); continue; } "
                 "while *p == 45 || *p == 43 || *p == 32 || *p == 35 || *p == 46 || (*p >= 48 && *p <= 57) { p = p.add(1); } "
                 "if *p == 115 { t(sh(args[n] as *const u8)); } else { t(args[n] as u32); } n += 1; } p = p.add(1); } 0 }"),
    },
    ret={
        "HSD_Randf": ("(float) (fv() % 1000) / 1000.0f", "(fv() % 1000) as f32 / 1000.0"),
        "Camera_80029020": ("((fv() & 3) == 0) ? 0 : &CS[fv() & 1]", "if fv() & 3 == 0 { core::ptr::null_mut() } else { &raw mut super::CS[(fv() & 1) as usize] }"),
        "Camera_80030A50": ("&CAMG", "&raw mut super::CAMG"),
        "GObj_Create": ("(a1 == 0x13) ? &TG[fv() & 3] : &IG[fv() & 3]",
                        "if a1 == 0x13 { &raw mut super::TG[(fv() & 3) as usize] } else { &raw mut super::IG[(fv() & 3) as usize] }"),
        "Ground_GetStageGObj": ("((NG && (fv() & 3) == 0) ? 0 : &XG[(unsigned) a0 % 10])",
                                "if super::NG && fv() & 3 == 0 { core::ptr::null_mut() } else { &raw mut super::XG[(a0 as u32 % 10) as usize] }"),
        "Ground_GetMapGObj": ("((NG && a0 == 1 && (fv() & 3) == 0) ? 0 : &XG[(unsigned) a0 % 10])",
                              "if super::NG && a0 == 1 && fv() & 3 == 0 { core::ptr::null_mut() } else { &raw mut super::XG[(a0 as u32 % 10) as usize] }"),
        "Ground_801C3FA4": ("((fv() & 3) == 0) ? 0 : &JP[fv() % 16]",
                            "if fv() & 3 == 0 { core::ptr::null_mut() } else { &raw mut super::JP[(fv() % 16) as usize] }"),
        "Ground_GetYakumonoParam": ("&YP", "&raw mut super::YP as *mut c_void"),
        "HSD_ArchiveGetPublicAddress": ("(a1[0] == 83) ? (void*) SISB : (((fv() % 4) == 0) ? 0 : (void*) &IDS[fv() & 3])",
                                        "if *a1 == 83 { (&raw mut super::SISB) as *mut c_void } else if fv() % 4 == 0 { core::ptr::null_mut() } else { (&raw mut super::IDS[(fv() & 3) as usize]) as *mut c_void }"),
        "grDatFiles_801C6330": ("&ARCH", "&raw mut super::ARCH"),
        "grDatFiles_GetArchive": ("&ARCH", "&raw mut super::ARCH"),
        "grDatFiles_801C6478": ("((fv() & 1) == 0) ? 0 : &ARCH", "if fv() & 1 == 0 { core::ptr::null_mut() } else { &raw mut super::ARCH }"),
        "lb_80013B14": ("&COB", "&raw mut super::COB"),
        "HSD_MemAlloc": ("(a0 == 0x1C || a0 == (int) sizeof(ImageDescWrapper)) ? (void*) &IWB[fv() & 3] : (a0 == (int) sizeof(TextWrapper)) ? (void*) &TWB[fv() & 3] : (void*) MEMB",
                         "if a0 == 0x1C || a0 == core::mem::size_of::<m::ImageDescWrapper>() as i32 { (&raw mut super::IWB[(fv() & 3) as usize]) as *mut c_void } "
                         "else if a0 == core::mem::size_of::<m::TextWrapper>() as i32 { (&raw mut super::TWB[(fv() & 3) as usize]) as *mut c_void } else { (&raw mut super::MEMB) as *mut c_void }"),
        "lbDvd_GetPreloadedArchive": ("((fv() & 1) == 0) ? 0 : (void*) MEMB", "if fv() & 1 == 0 { core::ptr::null_mut() } else { (&raw mut super::MEMB) as *mut c_void }"),
        "HSD_SisLib_803A5ACC": ("&TX[fv() & 3]", "&raw mut super::TX[(fv() & 3) as usize]"),
        "HSD_SisLib_803A6754": ("&TX[fv() & 3]", "&raw mut super::TX[(fv() & 3) as usize]"),
        "gm_GetStartMeleeRules": ("&SMR", "&raw mut super::SMR"),
        "gm_80160A60": ("STRS[fv() & 3]", "super::STRS[(fv() & 3) as usize]"),
        "Player_GetEntity": ("((fv() & 1) == 0) ? 0 : &GO[fv() & 3]", "if fv() & 1 == 0 { core::ptr::null_mut() } else { &raw mut super::GO[(fv() & 3) as usize] }"),
        "Player_GetPlayerSlotType": ("(Gm_PKind) (fv() % 5)", "fv() % 5"),
        "Stage_80225194": ("(fv() % 5 == 0) ? 0xF0 : fv()", "if fv() % 5 == 0 { 0xF0 } else { fv() }"),
        "gm_8018841C": ("GMF", "super::GMF as i32"),
        "grLib_801C96F8": ("((fv() & 3) == 0) ? 0 : (HSD_Generator*) GEN[fv() & 1]", "if fv() & 3 == 0 { core::ptr::null_mut() } else { (&raw mut super::GEN[(fv() & 1) as usize]) as *mut HSD_Generator }"),
        "grLib_801C9808": ("((fv() & 3) == 0) ? 0 : (HSD_Generator*) GEN[fv() & 1]", "if fv() & 3 == 0 { core::ptr::null_mut() } else { (&raw mut super::GEN[(fv() & 1) as usize]) as *mut HSD_Generator }"),
        "gm_80160968": ("({ GXColor c_; c_.r = (u8) fv(); c_.g = (u8) fv(); c_.b = (u8) fv(); c_.a = (u8) fv(); c_; })",
                        "{ let mut c = _GXColor { r: 0, g: 0, b: 0, a: 0 }; c.r = fv() as u8; c.g = fv() as u8; c.b = fv() as u8; c.a = fv() as u8; c }"),
    },
    effects={
        "lbVector_Diff": ("TV(a0); TV(a1); WV(a2);", "super::tv(a0); super::tv(a1); super::wv(a2);"),
        "lbVector_Add": ("TV(a0); TV(a1); WV(a0);", "super::tv(a0); super::tv(a1); super::wv(a0);"),
        "lbVector_WorldToScreen": ("TV(a1); WV(a2);", "super::tv(a1); super::wv(a2);"),
        "ftLib_GetCameraSubjectBonePos": ("WV(a1);", "super::wv(a1);"),
        "lb_8000B1CC": ("TV(a1); WV(a2);", "super::tv(a1); super::wv(a2);"),
        "grLib_801C96F8": ("TV(a2);", "super::tv(a2);"),
        "memzero": ("if (a0) zero(a0, (unsigned) a1);", "if !a0.is_null() { core::ptr::write_bytes(a0 as *mut u8, 0, a1 as usize); }"),
    },
    decl_c=DECL_C,
    decl_r=DECL_R,
    setup_c=SETUP_C,
    setup_r=SETUP_R,
    hash_c=HASH_C,
    hash_r=HASH_R,
    preg_c=("preg(&YP, sizeof YP); preg(XG, sizeof XG); preg(XGR, sizeof XGR); preg(TG, sizeof TG); preg(IG, sizeof IG); preg(TWB, sizeof TWB); "
            "preg(IWB, sizeof IWB); preg(TX, sizeof TX); preg(CS, sizeof CS); preg(&COB, sizeof COB); preg(&CAMG, sizeof CAMG); preg(SISB, sizeof SISB); "
            "preg(&SMR, sizeof SMR); preg(TOB, sizeof TOB); preg(MOB, sizeof MOB); preg(IDS, sizeof IDS); preg(DP, sizeof DP); preg(&ARCH, sizeof ARCH); "
            "preg(ARCHB, sizeof ARCHB); preg(GEN, sizeof GEN); preg(MEMB, sizeof MEMB); preg(&MOUT, sizeof MOUT); preg(RT, sizeof RT); preg(grPs_StageCallbacks, sizeof grPs_StageCallbacks); "
            "preg(grPs_803E1248, sizeof grPs_803E1248); preg(lbl_803E1630, sizeof lbl_803E1630); preg(&grPs_803E14FC, sizeof grPs_803E14FC); "
            "preg(grPs_8049F040, sizeof grPs_8049F040);"),
    preg_r=("preg(&raw const YP, core::mem::size_of_val(&YP)); preg(&raw const XG, core::mem::size_of_val(&XG)); preg(&raw const XGR, core::mem::size_of_val(&XGR)); "
            "preg(&raw const TG, core::mem::size_of_val(&TG)); preg(&raw const IG, core::mem::size_of_val(&IG)); preg(&raw const TWB, core::mem::size_of_val(&TWB)); "
            "preg(&raw const IWB, core::mem::size_of_val(&IWB)); preg(&raw const TX, core::mem::size_of_val(&TX)); preg(&raw const CS, core::mem::size_of_val(&CS)); "
            "preg(&raw const COB, core::mem::size_of_val(&COB)); preg(&raw const CAMG, core::mem::size_of_val(&CAMG)); preg(&raw const SISB, core::mem::size_of_val(&SISB)); "
            "preg(&raw const SMR, core::mem::size_of_val(&SMR)); preg(&raw const TOB, core::mem::size_of_val(&TOB)); preg(&raw const MOB, core::mem::size_of_val(&MOB)); "
            "preg(&raw const IDS, core::mem::size_of_val(&IDS)); preg(&raw const DP, core::mem::size_of_val(&DP)); preg(&raw const ARCH, core::mem::size_of_val(&ARCH)); "
            "preg(&raw const ARCHB, core::mem::size_of_val(&ARCHB)); preg(&raw const GEN, core::mem::size_of_val(&GEN)); preg(&raw const MEMB, core::mem::size_of_val(&MEMB)); "
            "preg(&raw const MOUT, core::mem::size_of_val(&MOUT)); preg(&raw const RT, core::mem::size_of_val(&RT)); preg(&raw const m::grPs_StageCallbacks, core::mem::size_of_val(&m::grPs_StageCallbacks)); "
            "preg(&raw const m::grPs_803E1248, core::mem::size_of_val(&m::grPs_803E1248)); preg(&raw const m::lbl_803E1630, core::mem::size_of_val(&m::lbl_803E1630)); "
            "preg(&raw const m::grPs_803E14FC, core::mem::size_of_val(&m::grPs_803E14FC)); preg(&raw const m::grPs_8049F040, core::mem::size_of_val(&m::grPs_8049F040));"),
    data=[("grPs_StageCallbacks", "StageCallbacks", 10), ("grPs_StageData", "StageData", None), ("grPs_803E1248", "raw", None),
          ("lbl_803E1630", "raw", None), ("grPs_8049F040", "raw", None)],
    ptr_unions={"u": "dobj"},
    pools={
        "HSD_JObj": ("&JP[fv() % 16]", "&raw mut JP[(fv() % 16) as usize]"),
        "HSD_TObj": ("&TOB[fv() & 3]", "&raw mut TOB[(fv() & 3) as usize]"),
        "_HSD_TObj": ("&TOB[fv() & 3]", "&raw mut TOB[(fv() & 3) as usize]"),
        "HSD_MObj": ("&MOB[fv() & 3]", "&raw mut MOB[(fv() & 3) as usize]"),
        "_HSD_MObj": ("&MOB[fv() & 3]", "&raw mut MOB[(fv() & 3) as usize]"),
        "HSD_ImageDesc": ("&IDS[fv() & 3]", "&raw mut IDS[(fv() & 3) as usize]"),
        "_HSD_ImageDesc": ("&IDS[fv() & 3]", "&raw mut IDS[(fv() & 3) as usize]"),
        "HSD_DObj": ("&DP[fv() & 7]", "&raw mut DP[(fv() & 7) as usize]"),
        "CmSubject": ("&CS[fv() & 1]", "&raw mut CS[(fv() & 1) as usize]"),
        "HSD_Generator": ("(HSD_Generator*) GEN[fv() & 1]", "(&raw mut GEN[(fv() & 1) as usize]) as *mut HSD_Generator"),
        "UnkArchiveStruct": ("&ARCH", "&raw mut ARCH"),
    },
    view_null={"xC8": 4, "xCC_gobj": 3, "xD0_gobj": 3},
    # the dispatchers run on_init callbacks of the other stage objects (different views of Ground::u)
    view={"grStadium_OnInit": "-", "grStadium_801D10F8": "-", "grStadium_801D4548": "stadium", "grStadium_801D1520": "stadium",
          "grStadium_801D13E0": "stadium", "grStadium_801D1A38": "stadium9"},
    hview={"grStadium_OnInit": "-", "grStadium_801D10F8": "-", "grStadium_801D4548": "stadium", "grStadium_801D1520": "stadium",
           "grStadium_801D13E0": "stadium", "grStadium_801D1A38": "stadium9"},
    args={
        "grStadium_801D10F8": [("(StadiumGrType) (fv() % 10)", "fv() % 10")],
        "grStadium_801D2528": [("&GO[fv() & 3]", "&raw mut GO[(fv() & 3) as usize]"),
                               ("({ u32 v_ = fv() % 19; (GMF && v_ >= 2 && v_ <= 6) ? 7 : v_; })",
                                "{ let v = fv() % 19; if GMF && v >= 2 && v <= 6 { 7 } else { v as i32 } }"),
                               ("(int) (fv() % 5 == 0 ? 0 : (s32) (fv() % 9) - 3)", "if fv() % 5 == 0 { 0 } else { (fv() % 9) as i32 - 3 }")],
        "grStadium_801D21E4": [("&GO[fv() & 3]", "&raw mut GO[(fv() & 3) as usize]"), ("(bool) (fv() & 1)", "(fv() & 1) as i32")],
        "grStadium_801D3F40": [("&GO[fv() & 3]", "&raw mut GO[(fv() & 3) as usize]"), ("(int) (9 + fv() % 6)", "(9 + fv() % 6) as i32")],
        "fn_801D2ED0": [("&IG[fv() & 3]", "&raw mut IG[(fv() & 3) as usize]"), ("(int) fv()", "fv() as i32")],
        "grStadium_801D2FD0": [("&IG[fv() & 3]", "&raw mut IG[(fv() & 3) as usize]"), ("(int) fv()", "fv() as i32")],
        "grStadium_801D3084": [("&IG[fv() & 3]", "&raw mut IG[(fv() & 3) as usize]"), ("(int) fv()", "fv() as i32")],
        "grStadium_801D1290": [("&XG[1]", "&raw mut XG[1]")],
        "grStadium_801D3138": [("&XG[1]", "&raw mut XG[1]"), ("&IDS[fv() & 3]", "&raw mut IDS[(fv() & 3) as usize]"),
                               ("&MOUT", "&raw mut MOUT")],
        "grStadium_801D3B4C": [("(int) (fv() % 6)", "(fv() % 6) as i32"), ("(int) (fv() % 4)", "(fv() % 4) as i32")],
        "grStadium_801D4FF8": [("(int) (fv() % 6)", "(fv() % 6) as i32")],
        "fn_801D4220": [("(int) (fv() % 4)", "(fv() % 4) as i32"), ("(uintptr_t) fv()", "fv()"), ("(void*) 0", "core::ptr::null_mut()"), ("0", "0")],
        "grStadium_OnTouchLine": [("(enum_t) (fv() % 4)", "(fv() % 4) as i32")],
    },
    pre={
        "grStadium_OnInit": (NG_C.replace("1", "0"), NG_R.replace("true", "false")),
        "grStadium_801D10F8": (NG_C, NG_R),
        "fn_801D11E4": (DISP_C, DISP_R),
        "grStadium_801D1290": (DISP_C, DISP_R),
        "grStadium_801D1390": both((DISP_C, DISP_R), (GMF_C, GMF_R)),
        "grStadium_801D13E0": (NG_C, NG_R),
        "grStadium_801D1520": both((STAD_C, STAD_R), (GMF_C, GMF_R), (KIND_C, KIND_R)),
        "grStadium_801D1EF8": (DISP_C, DISP_R),
        "grStadium_801D21E4": (DISP_C, DISP_R),
        "grStadium_801D2278": (DISP_C, DISP_R),
        "grStadium_801D2344": both((DISP_C, DISP_R), (GMF_C, GMF_R)),
        "grStadium_801D2528": both((DISP_C, DISP_R), (GMF_C, GMF_R)),
        "grStadium_801D2A60": (DISP_C, DISP_R),
        "grStadium_801D32D0": ((DISP_C + chr(10) + '{ float t_ = 0.5f * (float) (fv() % 9); COB.viewport.xmin = -64.0f + t_; t_ = 0.5f * (float) (fv() % 9); COB.viewport.xmax = 60.0f + t_; t_ = 0.5f * (float) (fv() % 9); COB.viewport.ymin = -42.0f + t_; t_ = 0.5f * (float) (fv() % 9); COB.viewport.ymax = 38.0f + t_; }'), (DISP_R + chr(10) + '{ let mut t = 0.5f32 * (fv() % 9) as f32; COB.viewport.xmin = -64.0f32 + t; t = 0.5f32 * (fv() % 9) as f32; COB.viewport.xmax = 60.0f32 + t; t = 0.5f32 * (fv() % 9) as f32; COB.viewport.ymin = -42.0f32 + t; t = 0.5f32 * (fv() % 9) as f32; COB.viewport.ymax = 38.0f32 + t; }')),
        "grStadium_801D3460": (DISP_C, DISP_R),
        "grStadium_801D384C": (DISP_C, DISP_R),
        "grStadium_801D39A0": (DISP_C, DISP_R),
        "grStadium_801D3A0C": (DISP_C, DISP_R),
        "grStadium_801D3B4C": both((KIND_C, KIND_R), (GMF_C, GMF_R)),
        "grStadium_801D3BBC": (DISP_C, DISP_R),
        "grStadium_801D3F40": (DISP_C, DISP_R),
        "grStadium_801D4040": both((KIND_C, KIND_R), (GMF_C, GMF_R)),
        "grStadium_801D4084": both((KIND_C, KIND_R), (GMF_C, GMF_R)),
        "grStadium_801D40C8": both((KIND_C, KIND_R), (GMF_C, GMF_R)),
        "grStadium_801D410C": both((KIND_C, KIND_R), (GMF_C, GMF_R)),
        "grStadium_801D4150": both((KIND_C, KIND_R), (GMF_C, GMF_R)),
        "grStadium_801D4194": (DISP_C, DISP_R),
        "grStadium_801D435C": (STAD_C, STAD_R),
        "grStadium_801D4548": both((STAD_C, STAD_R), (GMF_C, GMF_R), (NG_C, NG_R)),
        "grStadium_801D4FF8": both((KIND_C, KIND_R), (NG_C, NG_R)),
        "fn_801D5074": (DISP_C, DISP_R),
    },
    cases={"grStadium_801D4548": 3000, "grStadium_801D1520": 1500, "grStadium_801D2528": 2000, "grStadium_801D2344": 1500,
           "grStadium_801D1E20": 4000, "grStadium_801D2A60": 600, "grStadium_801D3BBC": 800, "grStadium_801D32D0": 800,
           "grStadium_801D3138": 600, "grStadium_801D1EF8": 800, "grStadium_801D4FF8": 600},
)
