"""Per-TU tables for gen.py."""

MENU_FLOW = [
    "    /// TODO(group6 mn): replace with melee_types' MenuFlow once mnmain.h's types are generated. Native",
    "    /// layout of the fields gmscene.c reads (cur_menu +0, hovered_selection +2, confirmed_selection +4).",
    "    #[repr(C)]",
    "    #[derive(Clone, Copy)]",
    "    pub struct MenuFlow { pub cur_menu: u8, pub prev_menu: u8, pub hovered_selection: u16, pub confirmed_selection: u8, pub pad_5: [u8; 3], pub rest: [u8; 24] }",
]

VS_MODE = {  # the six VS-variant mode TUs share one shape
    "ext_types": [],
}

# Variadic callees cannot be written as `extern "C" fn` on stable Rust: they take their variadic
# arguments as a slice of [`A`] (the hook sees the fixed arguments followed by those).
VARIADIC_RS = [
    "    /// `void OSReport(char* fmt, ...)`",
    "    pub unsafe fn OSReport(fmt: *const c_char, va: &[A]) {",
    "        if let Some(h) = HOOK { let mut v = vec![A::S(fmt)]; v.extend_from_slice(va); h(\"OSReport\", &v); return; }",
    "        panic!(\"TODO(group6): OSReport is not ported\")",
    "    }",
    "    /// The console clock the C reads as `__OSBusClock` (u32 at 0x800000F8): 162 MHz.",
    "    pub static mut __OSBusClock: u32 = 162_000_000;",
]

STAFF_RS = [
    "    /// `int lb_80011E24(HSD_JObj*, HSD_JObj**, ...)`: `ids` ends with -1; one JObj per id is written to `out`.",
    "    pub unsafe fn lb_80011E24(root: *mut HSD_JObj, out: *mut *mut HSD_JObj, ids: &[i32]) -> i32 {",
    "        if let Some(h) = HOOK { let mut v = vec![A::P(root as usize), A::O(out as usize)]; for &i in ids { v.push(A::I(i as u32 as u64)); } return h(\"lb_80011E24\", &v) as i32; }",
    "        panic!(\"TODO(group6): lb_80011E24 is not ported\")",
    "    }",
    "    /// `int HSD_SisLib_803A6B98(HSD_Text*, float, float, const char* fmt, ...)`",
    "    pub unsafe fn HSD_SisLib_803A6B98(text: *mut HSD_Text, x: f32, y: f32, fmt: *const c_char, va: &[A]) -> i32 {",
    "        if let Some(h) = HOOK { let mut v = vec![A::P(text as usize), A::F(x), A::F(y), A::S(fmt)]; v.extend_from_slice(va); return h(\"HSD_SisLib_803A6B98\", &v) as i32; }",
    "        panic!(\"TODO(group6): HSD_SisLib_803A6B98 is not ported\")",
    "    }",
    "    /// `HSD_Archive* lbArchive_80016DBC(const char* file, void* out0, ...)`: (name, out, name, out, ..., NULL) pairs follow.",
    "    pub unsafe fn lbArchive_80016DBC(file: *const c_char, out0: *mut c_void, va: &[A]) -> *mut HSD_Archive {",
    "        if let Some(h) = HOOK { let mut v = vec![A::S(file), A::P(out0 as usize)]; v.extend_from_slice(va); return h(\"lbArchive_80016DBC\", &v) as usize as *mut HSD_Archive; }",
    "        panic!(\"TODO(group6): lbArchive_80016DBC is not ported\")",
    "    }",
]

TUS = {
    "gmtiny": {"path": "src/melee/gm/gmtiny.c"},
    "gmsupersudden": {"path": "src/melee/gm/gmsupersudden.c"},
    "gmsinglebutton": {"path": "src/melee/gm/gmsinglebutton.c"},
    "gmslomo": {"path": "src/melee/gm/gmslomo.c"},
    "gmstamina": {"path": "src/melee/gm/gmstamina.c"},
    "gmscdata": {"path": "src/melee/gm/gmscdata.c"},
    "gmscene": {"path": "src/melee/gm/gmscene.c",
                "ext_types": MENU_FLOW,
                "ext_extra": VARIADIC_RS,
                "logmem": {("Snap_CuratedMix", 0): 4},
                "forward": {"gm_GetAllGameScenes": "super::super::gmscdata::gm_GetAllGameScenes",
                            "gm_801677C0": "|crate::gm_ptr::gm_801677C0(&mut *a0)"},
                "ext_var_types": {"mn_804A04F0": "MenuFlow"}},
    "gmstaffroll": {"path": "src/melee/gm/gmstaffroll.c",
                    "logmem": {("HSD_SisLib_803A74F0", 2): 4},
                    "ext_extra": STAFF_RS,
                    "skip_callees": {"PSMTXConcat", "PSMTXCopy", "PSMTXMultVec", "sqrtf", "tanf", "__frsqrte"},
                    "forward": {"gm_801A4B60": "super::super::gmscene::gm_801A4B60",
                                "memzero": "|crate::lb_r6::memzero(core::slice::from_raw_parts_mut(a0 as *mut u8, a1 as usize), a1 as isize)"}},
}


# ---------------------------------------------------------------------------------------- oracle
# C bodies of the variadic callees' stubs (after the fixed arguments p0..pk). The Rust twins are
# the `ext` wrappers in VARIADIC_RS / each TU's ext_extra and the `custom` arms in ORACLE.
_FMT_LOOP = """
    for (q = fmt; *q; q++) {
        if (*q != '%') continue;
        q++;
        if (*q == '%') continue;
        while ((*q >= '0' && *q <= '9') || *q == 'l') q++;
        if (*q == 's') tr_s(__builtin_va_arg(ap, const char*));
        else tr_i((unsigned long long) (unsigned) __builtin_va_arg(ap, int));
    }
"""

VARIADIC_C = {
    "OSReport": """void OSReport(char* fmt, ...)
{
    __builtin_va_list ap;
    const char* q;
    tr_begin("OSReport");
    tr_s(fmt);
    __builtin_va_start(ap, fmt);""" + _FMT_LOOP + """    __builtin_va_end(ap);
    tr_end();
}
""",
    "HSD_SisLib_803A6B98": """int HSD_SisLib_803A6B98(HSD_Text* p0, float p1, float p2, const char* fmt, ...)
{
    __builtin_va_list ap;
    const char* q;
    tr_begin("HSD_SisLib_803A6B98");
    tr_p(p0);
    tr_f(__builtin_bit_cast(unsigned, p1));
    tr_f(__builtin_bit_cast(unsigned, p2));
    tr_s(fmt);
    __builtin_va_start(ap, fmt);""" + _FMT_LOOP + """    __builtin_va_end(ap);
    tr_end();
    return (int) ret_i("HSD_SisLib_803A6B98", 32);
}
""",
    "lbArchive_80016DBC": """HSD_Archive* lbArchive_80016DBC(const char* file, void* out0, ...)
{
    __builtin_va_list ap;
    void** out;
    tr_begin("lbArchive_80016DBC");
    tr_s(file);
    tr_p(out0);
    *(void**) out0 = ret_p("lbArchive_80016DBC.out");
    __builtin_va_start(ap, out0);
    for (;;) {
        const char* name = __builtin_va_arg(ap, const char*);
        tr_s(name);
        out = __builtin_va_arg(ap, void**);
        tr_p(out);
        if (!out) break;
        *out = ret_p("lbArchive_80016DBC.out");
    }
    __builtin_va_end(ap);
    tr_end();
    return (HSD_Archive*) ret_p("lbArchive_80016DBC");
}
""",
    "lb_80011E24": """int lb_80011E24(HSD_JObj* p0, HSD_JObj** out, ...)
{
    __builtin_va_list ap;
    int k = 0;
    tr_begin("lb_80011E24");
    tr_p(p0);
    tr_o();
    __builtin_va_start(ap, out);
    for (;;) {
        int id = __builtin_va_arg(ap, int);
        tr_i((unsigned long long) (unsigned) id);
        if (id == -1) break;
        out[k++] = (HSD_JObj*) ret_p("lb_80011E24.out");
    }
    __builtin_va_end(ap);
    tr_end();
    return (int) ret_i("lb_80011E24", 32);
}
""",
}

# The six VS-variant mode TUs share one shape.
VS_STATE_GLOBALS = [
    ("gmMainLib_804D3EE0", False),
    ("gmVsMelee_CssData", False), ("gmVsMelee_SssData", False), ("gmVsMelee_StartData", False),
    ("gmVsMelee_VsExitInfo", False), ("gmVsMelee_SuddenDeathExitInfo", False),
    ("gmVsMelee_ResultsEnterData", False), ("gmVsMelee_ApproachData", False), ("if_Scene_Prize_EnterData", False),
]


def vs_oracle(states, funcs, dump_types):
    return {
        "funcs": funcs,
        "dump_types": ["GameModeState"] + dump_types,
        "globals": VS_STATE_GLOBALS + [(states, False)],
    }


ORACLE = {
    "gmtiny": vs_oracle("gm_Mode_TinyVs_States",
                        ["gm_801B8DA8", "gm_801B8DD4", "gm_801B8DFC", "gm_801B8E24", "fn_801B8E50", "gm_801B8E74", "gm_801B8EA8",
                         "gm_801B8ED0", "gm_801B8F04", "gm_801B8F24", "gm_801B8F44", "gm_Mode_TinyVs_OnInit", "gm_Mode_TinyVs_OnLoad"],
                        ["PlayerInitData"]),
}

ORACLE["gmsupersudden"] = vs_oracle(
    "gm_Mode_SuperSuddenDeath_States",
    ["gm_801B8BB4", "gm_801B8BE0", "gm_801B8C08", "gm_801B8C30", "fn_801B8C5C", "gm_801B8C68", "gm_801B8C9C",
     "gm_801B8CC4", "gm_801B8CF4", "gm_801B8D14", "gm_801B8D34", "gm_Mode_SuperSuddenDeath_OnInit",
     "gm_Mode_SuperSuddenDeath_OnLoad"], ["PlayerInitData"])
ORACLE["gmsinglebutton"] = vs_oracle(
    "gm_Mode_SingleButtonVs_States",
    ["gm_801BA10C", "gm_801BA138", "gm_801BA160", "gm_801BA188", "fn_801BA1B4", "gm_801BA1C8", "gm_801BA1FC",
     "gm_801BA224", "gm_801BA258", "gm_801BA278", "gm_801BA298", "gm_Mode_SingleButtonVs_OnInit",
     "gm_Mode_SingleButtonVs_OnLoad"], ["StartMeleeData"])
ORACLE["gmslomo"] = vs_oracle(
    "gm_Mode_SlowMo_States",
    ["gm_801BA50C", "gm_801BA538", "gm_801BA560", "gm_801BA588", "fn_801BA5B4", "gm_801BA5C0", "gm_801BA5F4",
     "gm_801BA61C", "gm_801BA650", "gm_801BA670", "gm_801BA690", "gm_Mode_SlowMo_OnInit", "gm_Mode_SlowMo_OnLoad"],
    ["StartMeleeData"])

ORACLE["gmstamina"] = {
    "scenario": "build_stamina",
    "funcs": ["gm_801B91C8", "gm_801B922C", "gm_801B9254", "gm_801B927C", "gm_801B931C", "gm_801B9560",
              "gm_Mode_StaminaVs_OnInit", "gm_Mode_StaminaVs_OnLoad", "gm_801B9600", "fn_801B96E8", "gm_801B97C4",
              "fn_801B9850"],
    "dump_types": ["GameModeState", "CSSData", "SSSData", "StartMeleeData", "VsModeData"],
    "globals": [("gmMainLib_804D3EE0", False), ("gmVsMelee_CssData", False), ("gmVsMelee_SssData", False),
                ("gmVsMelee_StartData", False), ("gmVsMelee_VsExitInfo", False), ("gm_804975F8", True),
                ("gm_Mode_StaminaVs_States", False)],
}


def _scdata_oracle():
    import model
    tu, defs, callees, ext_vars, own = model.load("src/melee/gm/gmscdata.c")
    mine = model.mine()
    states = sorted(n for n in ext_vars if n.endswith("_States"))
    gr = {}
    for n in states:
        if n in mine and mine[n] != "gmscdata":
            gr[n] = f"melee_modes::tu::{mine[n]}::{n}"
    return {
        "scenario": "build_scdata",
        "funcs": ["gm_GetAllGameScenes", "gm_GetAllGameModes", "gm_GetNumGameModes", "gm_GetNumGameScenes"],
        "dump_types": ["GameScene", "GameMode"],
        "globals": [("scenes", False), ("modes", False)] + [(n, False) for n in states],
        "gr_name": gr,
    }


ORACLE["gmscdata"] = _scdata_oracle()


# ---- helpers shared by the scene-loop oracles (test callbacks: a trace line, then a scripted effect)
RS_COMMON = """
fn ap(args: &[A], i: usize) -> usize {
    match args[i] {
        A::P(p) => p,
        A::S(p) => p as usize,
        A::M(p, _) => p,
        A::O(p) => p,
        A::I(v) => v as usize,
        _ => 0,
    }
}
"""

SCENE_C_EXTRA = """
int tcb_pred_a(void) { tr_begin("tcb_pred_a"); tr_end(); return (int) ret_i("tcb_pred_a", 32); }
int tcb_pred_b(void) { tr_begin("tcb_pred_b"); tr_end(); return (int) ret_i("tcb_pred_b", 32); }
void tcb_pre(void) { tr_begin("tcb_pre"); tr_end(); }
void tcb_hook(void) { tr_begin("tcb_hook"); tr_end(); }
void tcb_frame(void) { tr_begin("tcb_frame"); tr_end(); if (ret_i("tcb_frame_end", 32)) gm_801A4B60(); }
void tcb_frame2(void) { tr_begin("tcb_frame2"); tr_end(); if (ret_i("tcb_frame2_end", 32)) gm_801A4B74(); }
"""

SCENE_TCBS = {
    "tcb_pred_a": 'unsafe extern "C" fn tcb_pred_a() -> i32 { hook("tcb_pred_a", &[]) as i32 }',
    "tcb_pred_b": 'unsafe extern "C" fn tcb_pred_b() -> i32 { hook("tcb_pred_b", &[]) as i32 }',
    "tcb_pre": 'unsafe extern "C" fn tcb_pre() { hook("tcb_pre", &[]); }',
    "tcb_hook": 'unsafe extern "C" fn tcb_hook() { hook("tcb_hook", &[]); }',
    "tcb_frame": 'unsafe extern "C" fn tcb_frame() { hook("tcb_frame", &[]); if s497_rt::with(|rt| rt.ret_int("tcb_frame_end", 32)) != 0 { melee_modes::tu::gmscene::gm_801A4B60() } }',
    "tcb_frame2": 'unsafe extern "C" fn tcb_frame2() { hook("tcb_frame2", &[]); if s497_rt::with(|rt| rt.ret_int("tcb_frame2_end", 32)) != 0 { melee_modes::tu::gmscene::gm_801A4B74() } }',
}

ORACLE["gmscene"] = {
    "scenario": "build_scene",
    "pre": ["#include <dolphin/os.h>", "#undef __OSBusClock", "unsigned int s497_busclock;", "#define __OSBusClock s497_busclock"],
    "funcs": ["ProfBenchFrame", "gm_GetDbPauseFlag", "gm_801A4624", "gm_SetDbPauseFlag", "gm_ClearDbPauseFlag", "gm_801A46B8",
              "fn_801A46F4", "fn_801A47E4", "gm_801A48A4", "gm_801A4970", "gm_SetDbPauseInputHandlers", "gm_801A4B1C",
              "gm_SetPreGObjProcCallback", "gm_801A4B50", "gm_801A4B60", "gm_801A4B74", "gm_801A4B88",
              "gm_GetCurrentSceneEnterData", "gm_GetCurrentSceneExitData", "gm_801A4BA8", "gm_801A4BB8", "gm_801A4BC8",
              "fn_801A4BD0", "gm_801A4BD4", "mnLoadScreen_Release", "mnLoadScreen_Begin", "mnLoadScreen_Frame",
              "mnOverlay_Make", "mnOverlay_Begin", "mnOverlay_Line", "mnOverlay_Reshow", "mnOverlay_Frame",
              "gm_FindGameSceneHandler", "gm_801A4D34", "gm_OnePInterstage"],
    "no_ftab": ["maybe_gm_801A48A4"],
    "dump_types": ["gm_80479D58_t", "gmMainLib_8046B0F0_t", "HSD_GObjLibInitDataType", "DevText", "GameSceneInfo", "GameScene"],
    "globals": [("gobj_init_data", True), ("gm_80479D58", True), ("gm_804D672C", False), ("gm_804D6728", False),
                ("gm_804D6724", False), ("gm_804D6720", False), ("gm_803DA888", False), ("gm_803DA8C8", False),
                ("mnLoadScreen_panel", False), ("mnLoadScreen_text", False), ("mnLoadScreen_panelbuf", False),
                ("mnLoadScreen_textbuf", False), ("mnLoadScreen_holding", True), ("mnLoadScreen_frames", True),
                ("mnLoadScreen_settled", True), ("mnLoadScreen_created", True), ("mnLoadScreen_warm", True),
                ("mnLoadScreen_started", True), ("mnOverlay_panel", False), ("mnOverlay_caption", False),
                ("mnOverlay_toast", False), ("mnOverlay_panelbuf", False), ("mnOverlay_captionbuf", False),
                ("mnOverlay_toastbuf", False), ("gm_onep_interstage", True),
                ("HSD_PadMasterStatus", True), ("HSD_SObjLib_804D7960", True), ("HSD_SObjLib_8040C3A4", False),
                ("mn_804A04F0", True), ("gmMainLib_8046B0F0", True), ("DbLevel", True), ("HSD_RandSeedPtr", False),
                ("__OSBusClock", False)],
    "gc_name": {"__OSBusClock": "s497_busclock"},
    "gr_name": {"__OSBusClock": "melee_modes::tu::gmscene::ext::__OSBusClock"},
    "c_extra": SCENE_C_EXTRA,
    "tcbs": SCENE_TCBS,
    "retkind_extra": {"tcb_pred_a": "Rk::I(32)", "tcb_pred_b": "Rk::I(32)"},
    "rs_extra": RS_COMMON,
    "custom": {
        "gm_801677C0": {
            "c": "p0->x0 = 0; p0->x1 = 0; p0->x2 = 0; p0->x3 = 0; p0->unk_28 = 0; p0->unk_20 = 0;",
            "rs": "            let p = ap(args, 0) as *mut gm_801677C0_s;\n            (*p).x0 = 0; (*p).x1 = 0; (*p).x2 = 0; (*p).x3 = 0; (*p).unk_28 = 0; (*p).unk_20 = 0;\n            None",
        },
    },
}


# ---- gmstaffroll
_BE_WORD = ("(s32) (((u32) ((const unsigned char*) color)[0] << 24) | ((u32) ((const unsigned char*) color)[1] << 16) | "
            "((u32) ((const unsigned char*) color)[2] << 8) | (u32) ((const unsigned char*) color)[3])")
_STAFF_GLOBALS = ["staffInfo", "staffInfoSortBuf", "gm_804D680C", "gm_804D682C", "gm_804D6830", "gm_804D6834", "gm_804D6838",
                  "gm_804D683C", "gm_804D6840", "gm_804D6844", "gm_80480D58"]
_STAFF_INTS = ["gm_804D6800", "gm_804D6810", "gm_804D6814", "gm_804D6818", "gm_804D681C", "gm_804D6820", "gm_804D6824",
               "gm_804D6828", "gm_804D6804", "gm_804D6848", "gm_804D684C", "gm_803DD1E4", "gm_803DD1C8"]
_STAFF_CONST = ["gm_803DBFD8", "gm_803DBFD8_extra", "gm_803DD0C8", "gm_803DD0DC", "gm_803DD0F0", "gm_804D42B8", "gm_804D42BC",
                "gm_804D42C0", "gm_804D42C4", "gm_804D42C8", "gm_804D42CC", "gm_804DAAEC"]

ORACLE["gmstaffroll"] = {
    "scenario": "build_staff",
    "funcs": ["gm_801AA644", "gm_801AA664", "gm_801AA688", "gm_801AA6D8", "gm_801AA6FC", "gm_801AA774",
              "gm_Scene_StaffRoll_OnFrame", "fn_801AA7F8", "fn_801AA854", "fn_801AAA28", "fn_801AAABC", "fn_801AAB18",
              "fn_801AAB74", "fn_801AB200", "fn_801AC67C", "gm_Scene_StaffRoll_OnEnter", "gm_Scene_StaffRoll_OnExit"],
    "no_ftab": ["gm_801AB200_GetXPos", "gm_801AB200_GetTrigger", "gm_801AB200_HasCheck", "gm_801AB200_SetAppSRT", "gm_801AB200_ptcl"],
    "dump_types": ["StaffEntryData", "StaffRollExtraData", "HSD_WObjDesc", "HSD_CameraDescPerspective", "HSD_Text"],
    "patch": [
        ("p->x4.color = *(s32*) color;", "p->x4.color = " + _BE_WORD + ";"),
        ("tally_color2 = *(GXColor*) &gm_804DAAEC;",
         "tally_color2 = (GXColor) { (u8) ((u32) gm_804DAAEC >> 24), (u8) ((u32) gm_804DAAEC >> 16), (u8) ((u32) gm_804DAAEC >> 8), (u8) gm_804DAAEC };"),
    ],
    "c_pre_stubs": "static long s497_rescale(long n) { return n == 0x948 ? 198 * (long) sizeof(staffInfo[0]) : n == 0x2E68 ? 198 * (long) sizeof(staffInfoSortBuf[0]) : n; }",
    "argexpr": {("HSD_MemAlloc", 0): "s497_rescale(p0)", ("memzero", 1): "s497_rescale(p1)"},
    "globals": [(g, False) for g in _STAFF_GLOBALS] + [(g, True) for g in _STAFF_INTS] + [(g, False) for g in _STAFF_CONST]
               + [("HSD_GObjGXLinkHead", False), ("HSD_PadCopyStatus", True), ("HSD_GObj_CameraKind", True),
                  ("HSD_GObj_LightKind", True), ("HSD_GObj_FogKind", True), ("HSD_GObj_JObjKind", True)],
    "retkind_extra": {"lb_80011E24": "Rk::I(32)", "HSD_SisLib_803A6B98": "Rk::I(32)", "lbArchive_80016DBC": "Rk::P"},
    "rs_extra": RS_COMMON,
    "custom": {
        "HSD_JObjReparent": {
            "c": "if (p0->parent) p0->parent->child = p0->next;",
            "rs": "            let c = ap(args, 0) as *mut HSD_JObj;\n            if !(*c).parent.is_null() { (*(*c).parent).child = (*c).next; }\n            None",
        },
        "memzero": {
            "c": "__builtin_memset(p0, 0, (unsigned long) s497_rescale(p1));",
            "rs": "            let n = match args[1] { A::I(v) => v as u32 as i32 as usize, _ => 0 };\n            core::ptr::write_bytes(ap(args, 0) as *mut u8, 0, n);\n            None",
        },
        "lb_80011E24": {
            "c": "",
            "rs": "            let out = ap(args, 1) as *mut u64;\n            let mut k = 0;\n            for x in &args[2..] {\n                if let A::I(v) = x {\n                    if *v as u32 == 0xFFFF_FFFF { break; }\n                    *out.add(k) = s497_rt::with(|rt| rt.scripted_ptr(\"lb_80011E24.out\"));\n                    k += 1;\n                }\n            }\n            None",
        },
        "lbArchive_80016DBC": {
            "c": "",
            "rs": "            let o0 = ap(args, 1) as *mut u64;\n            *o0 = s497_rt::with(|rt| rt.scripted_ptr(\"lbArchive_80016DBC.out\"));\n            let mut i = 3;\n            while i < args.len() {\n                let o = ap(args, i);\n                if o == 0 { break; }\n                *(o as *mut u64) = s497_rt::with(|rt| rt.scripted_ptr(\"lbArchive_80016DBC.out\"));\n                i += 2;\n            }\n            None",
        },
    },
}
