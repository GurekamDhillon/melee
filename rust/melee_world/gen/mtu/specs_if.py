"""Oracle specs of the `if` TUs (shard 625-632); keys as in specs.py plus the pool-driver keys of
if_drv.py (pools, rules, statics, ops, setup_c / setup_rs, defaults)."""
COMMON_RET = {
    "HSD_Randi": ("(a0 > 0 ? (int) (rnd() % (u32) a0) : 0)", "if a0 > 0 { (rnd() % a0 as u32) as i32 } else { 0 }"),
    "HSD_Randf": ("(float) (rnd() % 1000) / 1000.0f", "(rnd() % 1000) as f32 / 1000.0"),
}

SPECS = {}


def fnv(name):
    h = 0x811C9DC5
    for b in name.encode():
        h = ((h ^ b) * 0x01000193) & 0xFFFFFFFF
    return h


def sections_stub(fname, a0, table, ret=None, a1ty="void**"):
    """A variadic `(a0, void* symbols, name, [dst, name]..., 0)` loader stub (lbArchive_LoadSections,
    lbArchive_80016DBC): traces a0, each destination slot and name hash, stores table[name] there.
    a0 = (C param decl, C trace stmt, Rust param type, Rust trace stmt); ret = (C type, C expr, Rust type, Rust expr).
    table = {name: (C expr, Rust expr)}. The Rust ext takes the first pair and then the rest as a slice."""
    cdecl, ctrace, rty, rtrace = a0
    arms_c = " else ".join(f"if (h == 0x{fnv(n):X}u) *cur = (void*) ({c});" for n, (c, _) in table.items())
    arms_r = " ".join(f"0x{fnv(n):X} => {r} as *mut c_void," for n, (_, r) in table.items())
    rt = ret[0] if ret else "void"
    c = (f"{rt} {fname}({cdecl} a0, {a1ty} a1, ...) {{ __builtin_va_list ap; void** cur = (void**) a1; const char* nm; u32 h; "
         f"__builtin_va_start(ap, a1); T(@ID@); {ctrace} "
         f"while (cur) {{ nm = __builtin_va_arg(ap, const char*); h = SH(nm); T(PI(cur)); T(h); "
         f"{arms_c + ' else ' if arms_c else ''}*cur = 0; cur = __builtin_va_arg(ap, void**); }} "
         f"__builtin_va_end(ap); " + (f"return ({ret[0]}) ({ret[1]}); " if ret else "") + "}")
    rr = f" -> {ret[2]}" if ret else ""
    ext = f"fn {fname}(a0: {rty}, a1: *mut c_void, a2: *const u8, rest: &[(*mut c_void, *const u8)]){rr};"
    hook = ("|a0, a1, a2, rest| { t(@ID@); " + rtrace + " let mut cur = a1; let mut nm = a2; let mut i = 0; "
            "loop { let h = sh(nm); t(pi(cur)); t(h); unsafe { *(cur as *mut *mut c_void) = match h { " + arms_r +
            " _ => core::ptr::null_mut() }; } if i >= rest.len() { break; } cur = rest[i].0; nm = rest[i].1; i += 1; } " +
            (ret[3] if ret else "") + " }")
    return dict(c=c, ext=ext, hook=hook)


def jobjs_stub(fname="lb_80011E24"):
    """int lb_80011E24(HSD_JObj* root, HSD_JObj** out, id..., -1): out[k] = a pool joint for each id."""
    c = (f"int {fname}(HSD_JObj* a0, HSD_JObj** a1, ...) {{ __builtin_va_list ap; int id; int i = 0; "
         f"__builtin_va_start(ap, a1); T(@ID@); T(PI(a0)); T(PI(a1)); "
         f"for (;;) {{ id = __builtin_va_arg(ap, int); T((u32) id); if (id == -1) break; a1[i++] = pick_HSD_JObj(0); }} "
         f"__builtin_va_end(ap); return (int) fv(); }}")
    ext = f"fn {fname}(a0: *mut HSD_JObj, a1: *mut *mut HSD_JObj, ids: &[i32]) -> i32;"
    hook = ("|a0, a1, ids| { t(@ID@); t(pi(a0)); t(pi(a1)); let mut i = 0usize; "
            "for &id in ids { t(id as u32); unsafe { *a1.add(i) = pick_HSD_JObj(0); } i += 1; } t(-1i32 as u32); fv() as i32 }")
    return dict(c=c, ext=ext, hook=hook)


EVT = ("raw", "(Event) cbsel()", "core::mem::transmute::<usize, Event>(cbsel())")

SPECS["ifhazard"] = dict(
    src="src/melee/if/ifhazard.c",
    pools={"HSD_GObj": 4, "HSD_JObj": 4, "DynamicModelDesc": 2},
    rules={"HSD_GObj.hsd_obj": ("ptr", "HSD_JObj", False)},
    statics=[dict(c="un_804D6D80", kind="ptr", fill="p:HSD_GObj"), dict(c="un_804D6D84", kind="int", fill="r:0:3"),
             dict(c="un_804D6D88", kind="int", fill="r:0:2"), dict(c="un_804D6D8C", kind="int", fill="r:0:2"),
             dict(c="HSD_GObj_JObjKind", ext=True, kind="int", fill="k:20")],
    skip_data=["result"],
    ret={"GObj_Create": ("pick_HSD_GObj(0)", "pick_HSD_GObj(0)"),
         "HSD_JObjLoadJoint": ("pick_HSD_JObj(0)", "pick_HSD_JObj(0)")},
    null_ret_ok=["HSD_GObj_SetupProc"],
    ops=[dict(name="un_802FD604"), "un_802FD65C", "un_802FD668", "un_802FD674", "fn_802FD680", "fn_802FD6CC",
         "un_802FD704", "un_802FD708", "un_802FD8A0", "un_802FD8C4", "un_802FD8E8", "un_802FD90C"],
)

SPECS["if_2F6E"] = dict(
    src="src/melee/if/if_2F6E.c",
    pools={"HSD_GObj": 4, "HSD_JObj": 4, "DynamicModelDesc": 3, "VsSceneController": 1, "HSD_Archive": 1},
    rules={"HSD_GObj.hsd_obj": ("ptr", "HSD_JObj", False),
           "Element_803F9628.x0": ("ptr", "HSD_GObj", True),
           "Element_803F9628.x14": ("ptr", "DynamicModelDesc", False),
           "Element_803F9628.x8": ("fn", [("if_802F74D0", "m::ext::cb::if_802F74D0"),
                                          ("if_802F73C4", "m::ext::cb::if_802F73C4")], True),
           "Element_803F9628.x18": ("fn", [("cbA", "super::cbA"), ("cbB", "super::cbB")], True),
           "Element_803F9628.x1C": ("fn", [("cbA", "super::cbA"), ("cbB", "super::cbB")], True),
           "VsSceneState.unk_B": ("range", 0, 8), "VsSceneState.match_result": ("range", 0, 8)},
    statics=[dict(c="ifStatus_803F9628", ty="Element_803F9628", n=8, init=True),
             dict(c="HSD_GObj_JObjKind", ext=True, kind="int", fill="k:20"),
             dict(c="MODELS", rs="super::MODELS", n=8, kind="ptr", fill="pn:DynamicModelDesc", region=True),
             dict(c="ARCP", rs="super::ARCP", kind="ptr", fill="p:HSD_Archive", region=True)],
    skip_data=["result"],
    c_decl="static DynamicModelDesc* MODELS[8];\nstatic HSD_Archive* ARCP;\n"
           "static void cbA(s32 a) { T(7001); T((u32) a); }\nstatic void cbB(s32 a) { T(7002); T((u32) a); }\n"
           "static void* cbsel(void) { u32 q = rnd() % 3; return q == 0 ? (void*) 0 : q == 1 ? (void*) cbA : (void*) cbB; }\n",
    rs_decl="pub static mut MODELS: [*mut DynamicModelDesc; 8] = [null_mut(); 8];\n"
            "pub static mut ARCP: *mut HSD_Archive = null_mut();\n"
            "unsafe extern \"C\" fn cbA(a: i32) { t(7001); t(a as u32); }\n"
            "unsafe extern \"C\" fn cbB(a: i32) { t(7002); t(a as u32); }\n"
            "fn cbsel() -> usize { let q = rnd() % 3; if q == 0 { 0 } else if q == 1 { cbA as usize } else { cbB as usize } }\n",
    extra_freg=[("cbA", "cbA"), ("cbB", "cbB")],
    ret={"GObj_Create": ("pick_HSD_GObj(0)", "pick_HSD_GObj(0)"),
         "HSD_JObjLoadJoint": ("pick_HSD_JObj(0)", "pick_HSD_JObj(0)"),
         "gmVs_GetSceneController": ("&P_VsSceneController[0]", "&raw mut P_VsSceneController[0]"),
         "ifAll_GetArchive": ("&ARCP", "&raw mut super::ARCP")},
    null_ret_ok=["HSD_GObj_SetupProc"],
    custom_stubs={"lbArchive_LoadSections": sections_stub(
        "lbArchive_LoadSections", ("HSD_Archive*", "T(PI(a0));", "*mut HSD_Archive", "t(pi(a0));"),
        {"ScInfCnt_scene_models": ("MODELS", "&raw mut super::MODELS")})},
    ops=[dict(name="ifStatus_802F6EA4", args=["r:0:8", "v", "v", "v", EVT, EVT]),
         dict(name="ifStatus_802F7034", args=[("raw", "cbsel()", "cbsel() as *mut c_void")]),
         "ifStatus_802F7134", "ifStatus_802F7220"],
)

# --- the jobj / dobj / mobj / tobj / aobj chain every HUD element walks
CHAIN_RULES = {
    "HSD_GObj.hsd_obj": ("ptr", "HSD_JObj", False),
    "HSD_JObj.u": ("uptr", "dobj", "HSD_DObj", False),
    "HSD_JObj.child": ("ptr", "HSD_JObj", True),
    "HSD_JObj.next": ("ptr", "HSD_JObj", True),
    "HSD_DObj.mobj": ("ptr", "HSD_MObj", False),
    "HSD_DObj.next": ("ptr", "HSD_DObj", True),
    "HSD_MObj.tobj": ("ptr", "HSD_TObj", False),
    "HSD_MObj.mat": ("ptr", "HSD_Material", False),
    "HSD_TObj.aobj": ("ptr", "HSD_AObj", False),
}
MEMZERO = {"memzero": ("zero(a0, (unsigned) a1);",
                       "core::ptr::write_bytes(a0 as *mut u8, 0, a1 as usize);")}

SPECS["iftime"] = dict(
    src="src/melee/if/iftime.c",
    pools={"HSD_GObj": 4, "HSD_JObj": 12, "HSD_DObj": 3, "HSD_MObj": 3, "HSD_TObj": 3, "HSD_AObj": 3, "HSD_Material": 3,
           "DynamicModelDesc": 3, "StartMeleeRules": 1, "HSD_Joint": 4},
    rules={**CHAIN_RULES,
           "ifTime_data.match_timer": ("ptr", "HSD_GObj", True),
           "ifTime_data.countdown_timer": ("ptr", "HSD_GObj", True),
           "ifTime_data.digits": ("ptr", "HSD_JObj", False),
           "ifTime_data.countdown_timer_models": ("expr", "CDM", "&raw mut super::CDM"),
           "DynamicModelDesc.joint": ("ptr", "HSD_Joint", True), "DynamicModelDesc.anims": ("ptr", "HSD_Joint", True),
           "DynamicModelDesc.matanims": ("ptr", "HSD_Joint", True), "DynamicModelDesc.shapeanims": ("ptr", "HSD_Joint", True)},
    statics=[dict(c="ifTime_data", ty="ifTime_data"), dict(c="ifTime_match_timer_models", ty="DynamicModelDesc"),
             dict(c="TIMM", rs="super::TIMM", n=4, kind="ptr", fill="pn:DynamicModelDesc", region=True),
             dict(c="CDM", rs="super::CDM", n=4, kind="ptr", fill="p:DynamicModelDesc", region=True),
             dict(c="ARCP", rs="super::ARCP", kind="ptr", fill="k:0", region=True),
             dict(c="TPOS", rs="super::TPOS", ty="Vec3", region=True),
             dict(c="HSD_GObj_JObjKind", ext=True, kind="int", fill="k:20")],
    skip_data=["result"],
    c_decl="static DynamicModelDesc* TIMM[4];\nstatic DynamicModelDesc* CDM[4];\nstatic HSD_Archive* ARCP;\nstatic Vec3 TPOS;\n"
           "void OSPanic(const char* f, int l, const char* m, ...) { for (;;) {} }\n",
    rs_decl="pub static mut TIMM: [*mut DynamicModelDesc; 4] = [null_mut(); 4];\n"
            "pub static mut CDM: [*mut DynamicModelDesc; 4] = [null_mut(); 4];\n"
            "pub static mut ARCP: *mut HSD_Archive = null_mut();\n"
            "pub static mut TPOS: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };\n",

    defaults={"memzero": "super::memzero_default"},
    ret={"GObj_Create": ("pick_HSD_GObj(0)", "pick_HSD_GObj(0)"),
         "HSD_JObjLoadJoint": ("pick_HSD_JObj(0)", "pick_HSD_JObj(0)"),
         "gm_GetStartMeleeRules": ("&P_StartMeleeRules[0]", "&raw mut P_StartMeleeRules[0]"),
         "ifAll_GetArchive": ("&ARCP", "&raw mut super::ARCP"),
         "ifAll_GetTimerPosition": ("&TPOS", "&raw mut super::TPOS"),
         "gm_8016AEEC": ("(int) (fv() % 8)", "fv() % 8"),
         "gm_8016AF0C": ("((fv() & 1) ? 0 : (int) (fv() % 100))", "if fv() & 1 != 0 { 0 } else { (fv() % 100) as u16 }"),
         "gm_8016B110": ("(int) (fv() & 1)", "(fv() & 1) as i32")},
    effects=MEMZERO,
    null_ret_ok=["HSD_GObj_SetupProc"],
    custom_stubs={"lbArchive_LoadSections": sections_stub(
        "lbArchive_LoadSections", ("HSD_Archive*", "T(PI(a0));", "*mut HSD_Archive", "t(pi(a0));"),
        {"ScInfTim_scene_models": ("TIMM", "&raw mut super::TIMM"), "tdsce": ("CDM", "&raw mut super::CDM")})},
    ops=["ifTime_LoadModels", dict(name="ifTime_SetDigit", args=["p:HSD_JObj", "v"]),
         dict(name="ifTime_SetTime", args=["p:HSD_JObj", "v", "r:0:99"]),
         "ifTime_HideTimers", "ifTime_ShowTimers", "ifTime_GetCountdownSeconds",
         "ifTime_GetCountdownSeconds_dontinline", "ifTime_RenderGuarded", "ifTime_UpdateCountdown", "ifTime_FreeCountdown",
         "ifTime_UpdateTimers", "ifTime_CreateTimers", "ifTime_Reset", "ifTime_FreeTimers", "ifTime_IsTimerHidden"],
)

for _s in SPECS.values():
    _s["common_ret"] = COMMON_RET
