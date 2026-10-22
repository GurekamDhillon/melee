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


SIS_PRINT = dict(
    c="int HSD_SisLib_803A6B98(HSD_Text* a0, float a1, float a2, const char* a3, ...) { T(@ID@); T(PI(a0)); T(B(a1)); "
      "T(B(a2)); T(SH(a3)); return (int) fv(); }",
    ext="fn HSD_SisLib_803A6B98(a0: *mut HSD_Text, a1: f32, a2: f32, a3: *const u8) -> i32;",
    hook="|a0, a1, a2, a3| { t(@ID@); t(pi(a0)); t(a1.to_bits()); t(a2.to_bits()); t(sh(a3)); fv() as i32 }")

# A minimal sprintf (%d with optional 0 flag and width, literals) for the C oracle: the UCRT's is an inline.
SPRINTF_C = (
    "int sprintf(char* s, const char* f, ...) { __builtin_va_list ap; char* o = s; __builtin_va_start(ap, f); "
    "while (*f) { if (*f != 37) { *o++ = *f++; continue; } f++; { int zero = 0, w = 0, v, neg, n = 0; char t[16]; "
    "if (*f == 48) { zero = 1; f++; } while (*f >= 48 && *f <= 57) { w = w * 10 + (*f - 48); f++; } f++; "
    "v = __builtin_va_arg(ap, int); neg = v < 0; { unsigned u = neg ? 0u - (unsigned) v : (unsigned) v; "
    "do { t[n++] = (char) (48 + u % 10); u /= 10; } while (u); } "
    "if (neg && zero) { *o++ = 45; w--; } while (w > n + (neg && !zero ? 1 : 0)) { *o++ = zero ? 48 : 32; w--; } "
    "if (neg && !zero) *o++ = 45; while (n) *o++ = t[--n]; } } *o = 0; __builtin_va_end(ap); return (int) (o - s); }"
    + chr(10))

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

G0 = ("raw", "&P_HSD_GObj[0]", "&raw mut g::P_HSD_GObj[0]")

SPECS["ifnametag"] = dict(
    src="src/melee/if/ifnametag.c",
    pools={"HSD_GObj": 6, "HSD_JObj": 6, "HSD_Text": 1, "Fighter": 2, "DynamicModelDesc": 2, "HSD_Joint": 3,
           "HSD_AnimJoint": 6, "HSD_CObj": 1, "VsSceneController": 1,
           "HSD_Archive": 1},
    rules={"HSD_GObj.hsd_obj": ("ptr", "HSD_JObj", False),
           "HSD_JObj.child": ("ptr", "HSD_JObj", True), "HSD_JObj.next": ("ptr", "HSD_JObj", True),
           "DynamicModelDesc.joint": ("ptr", "HSD_Joint", False),
           "DynamicModelDesc.anims": ("ptr", "HSD_AnimJoint", True),
           "DynamicModelDesc.matanims": ("ptr", "HSD_AnimJoint", True),
           "DynamicModelDesc.shapeanims": ("ptr", "HSD_AnimJoint", True),
           "HSD_AnimJoint.child": ("ptr", "HSD_AnimJoint", True),
           "StaticModelDesc.joint": ("ptr", "HSD_Joint", True),
           "StaticModelDesc.animjoint": ("ptr", "HSD_AnimJoint", True),
           "StaticModelDesc.matanim_joint": ("ptr", "HSD_AnimJoint", True),
           "StaticModelDesc.shapeanim_joint": ("ptr", "HSD_AnimJoint", True)},
    statics=[dict(c="un_803F98B8", flat=("float", "f32"), n=25, kind="f32", keep=True, init=True),
             dict(c="un_803F991C", n=4, kind="f32", keep=True, init=True),
             dict(c="nametag_eyepos", ty="HSD_WObjDesc", keep=True, init=True),
             dict(c="nametag_interest", ty="HSD_WObjDesc", keep=True, init=True),
             dict(c="nametag_CObjDesc", ty="HSD_CameraDescFrustum", keep=True, init=True),
             dict(c="un_804A1ED0", ty="StaticModelDesc"),
             dict(c="un_804A1EE0", n=6, kind="ptr", fill="p:HSD_GObj"),
             dict(c="un_804A1EF8", n=6, kind="int", fill="v"),
             dict(c="un_804D6D68", kind="ptr", fill="pn:HSD_GObj"),
             dict(c="un_804D6D6C", kind="int", fill="r:0:1"),
             dict(c="un_804D6D70", n=6, kind="int", fill="r:0:1"),
             dict(c="un_804D6D78", kind="ptr", fill="p:HSD_Text"),
             dict(c="un_804D6D7C", kind="int", fill="v"),
             dict(c="SLOTB", rs="super::SLOTB", kind="int", fill="r:0:5", region=True),
             dict(c="HEAPB", rs="super::HEAPB", n=16, kind="int", region=True),
             dict(c="NMODELS", rs="super::NMODELS", n=2, kind="ptr", fill="p:DynamicModelDesc", region=True),
             dict(c="ARCP", rs="super::ARCP", kind="ptr", fill="k:0", region=True),
             dict(c="HSD_GObj_JObjKind", ext=True, kind="int", fill="k:20"),
             dict(c="HSD_GObj_CameraKind", ext=True, kind="int", fill="k:21")],
    skip_data=["result"],
    c_decl="static u8 SLOTB;\nstatic u8 HEAPB[16];\nstatic DynamicModelDesc* NMODELS[2];\nstatic HSD_Archive* ARCP;\n",
    rs_decl="pub static mut SLOTB: u8 = 0;\npub static mut HEAPB: [u8; 16] = [0; 16];\n"
            "pub static mut NMODELS: [*mut DynamicModelDesc; 2] = [null_mut(); 2];\n"
            "pub static mut ARCP: *mut HSD_Archive = null_mut();\n",
    setup_c="int k;\n    P_HSD_GObj[0].user_data = &SLOTB;\n"
            "    for (k = 1; k < 4; k++) P_HSD_GObj[k].user_data = &P_Fighter[k & 1];\n"
            "    for (k = 0; k < 2; k++) { P_Fighter[k].kind = (rnd() & 1) ? Ft_Kind_Mewtwo : (int) fv();"
            " P_Fighter[k].motion_id = (rnd() & 1) ? 0xEC : (int) fv(); }",
    setup_rs="g::P_HSD_GObj[0].user_data = &raw mut SLOTB as *mut _;\n"
             "    for k in 1..4usize { g::P_HSD_GObj[k].user_data = &raw mut g::P_Fighter[k & 1] as *mut _; }\n"
             "    for k in 0..2usize { g::P_Fighter[k].kind = if rnd() & 1 != 0 { Ft_Kind_Mewtwo as _ } else { fv() as _ };"
             " g::P_Fighter[k].motion_id = if rnd() & 1 != 0 { 0xEC } else { fv() as _ }; }",
    ret={"GObj_Create": ("pick_HSD_GObj(0)", "pick_HSD_GObj(0)"),
         "HSD_JObjLoadJoint": ("pick_HSD_JObj(0)", "pick_HSD_JObj(0)"),
         "gmVs_GetSceneController": ("&P_VsSceneController[0]", "&raw mut P_VsSceneController[0]"),
         "ifAll_GetArchive": ("&ARCP", "&raw mut super::ARCP"),
         "Camera_80030A50": ("&P_HSD_GObj[0]", "&raw mut P_HSD_GObj[0]"),
         "HSD_MemAlloc": ("HEAPB", "&raw mut super::HEAPB"),
         "HSD_SisLib_803A6754": ("&P_HSD_Text[0]", "&raw mut P_HSD_Text[0]"),
         "lb_80013B14": ("&P_HSD_CObj[0]", "&raw mut P_HSD_CObj[0]"),
         "Player_GetEntity": ("&P_HSD_GObj[1 + fv() % 3]", "&raw mut P_HSD_GObj[(1 + fv() % 3) as usize]"),
         "Player_GetPlayerId": ("(fv() % 4)", "(fv() % 4) as _"),
         "Player_GetTeam": ("(fv() % 5)", "(fv() % 5) as _"),
         "Player_GetPlayerSlotType": ("(fv() % 4)", "fv() % 4"),
         "Player_GetNametagSlotID": ("((fv() & 1) ? 120 : (int) (fv() % 100))",
                                     "if fv() & 1 != 0 { 120 } else { (fv() % 100) as _ }"),
         "Player_GetStocks": ("(int) (fv() % 4)", "(fv() % 4) as _"),
         "gm_8016B168": ("(fv() & 1)", "(fv() & 1) as _"), "gm_8016B258": ("(fv() & 1)", "(fv() & 1) as _"),
         "gm_8016B0E8": ("(fv() & 1)", "(fv() & 1) as _"), "Player_GetPlayerState": ("(fv() & 1)", "(fv() & 1) as _"),
         "Player_80036058": ("(fv() & 1)", "(fv() & 1) as _"),
         "lbVector_WorldToScreen": ("a2", "a2")},
    defaults={"memzero": "super::memzero_default"},
    effects={**MEMZERO, "Player_LoadPlayerCoords": ("a1->x = rw(); a1->y = rw(); a1->z = rw();",
                                         "(*a1).x = rw(); (*a1).y = rw(); (*a1).z = rw();"),
             "lbVector_WorldToScreen": ("a2->x = rw(); a2->y = rw(); a2->z = rw();",
                                        "(*a2).x = rw(); (*a2).y = rw(); (*a2).z = rw();")},
    null_ret_ok=["HSD_GObj_SetupProc", "GetNameText"],
    deref={"lbVector_WorldToScreen": [1]},
    custom_stubs={"lbArchive_LoadSections": sections_stub(
        "lbArchive_LoadSections", ("HSD_Archive*", "T(PI(a0));", "*mut HSD_Archive", "t(pi(a0));"),
        {"ScInfPnm_scene_models": ("NMODELS", "&raw mut super::NMODELS")}),
        "HSD_SisLib_803A6B98": SIS_PRINT},
    ops=[dict(name="un_802FC9B4", args=["i:6", "r:0:4", "v", "v"]),
         dict(name="NameTag_RenderCallback", args=["pn:HSD_GObj", "v"]),
         dict(name="fn_802FCAC4", args=["p:HSD_GObj", "v"]),
         "un_802FCBA0", dict(name="has_nametag", args=["i:6"]),
         dict(name="fn_802FCC44", args=[G0]), dict(name="inlineA1", args=["w"]),
         dict(name="NameTag_Create", args=["i:6"]), dict(name="un_802FD28C", args=["i:6"]),
         "un_802FD404", "un_802FD428", "un_802FD450", "un_802FD45C",
         dict(name="un_802FD468", pre_c="{ int q; for (q = 0; q < 6; q++) if (rnd() & 1) un_804A1EE0[q] = 0; }",
              pre_rs="for q in 0..6usize { if rnd() & 1 != 0 { m::un_804A1EE0[q] = null_mut(); } }"),
         dict(name="un_802FD4C8_inline", args=["v"]), "un_802FD4C8"],
)

SPECS["ifprize"] = dict(
    src="src/melee/if/ifprize.c",
    pools={"HSD_GObj": 6, "HSD_JObj": 6, "HSD_DObj": 3, "HSD_Text": 3, "SceneDesc": 1, "SceneCameraDesc": 2,
           "DynamicModelDesc": 2, "HSD_Archive": 1, "HSD_CObj": 1, "un_802FEBE0_OnEnter_arg0": 4},
    rules={"HSD_GObj.hsd_obj": ("ptr", "HSD_JObj", False),
           "HSD_JObj.u": ("uptr", "dobj", "HSD_DObj", False),
           "HSD_JObj.child": ("ptr", "HSD_JObj", False),
           "un_803F9D48.x3": ("range", 0, 2), "un_803F9D48.x1": ("range", 0, 5), "un_803F9D48.x2": ("range", -2, 20),
           "un_803F9D48.x10": ("ptr", "HSD_JObj", False),
           "un_803F9D48.x14": ("ptr", "HSD_GObj", False), "un_803F9D48.x18": ("ptr", "HSD_GObj", False),
           "un_803F9D48.x1C": ("ptr", "HSD_GObj", False),
           "un_803F9D48.x20": ("ptr", "HSD_Text", False), "un_803F9D48.x24": ("ptr", "HSD_Text", False),
           "un_803F9D48.x28": ("ptr", "un_802FEBE0_OnEnter_arg0", False),
           "un_803F9D48.x2C": ("ptr", "un_802FEBE0_OnEnter_arg0", True),
           "un_802FEBE0_OnEnter_arg0.x8": ("ptr", "un_802FEBE0_OnEnter_arg0", True),
           "SceneDesc.models": ("expr", "PRZM", "&raw mut super::PRZM"),
           "SceneDesc.lights": ("expr", "PRZM", "&raw mut super::PRZM"),
           "SceneDesc.cameras": ("ptr", "SceneCameraDesc", False)},
    statics=[dict(c="un_803F9D48", ty="un_803F9D48", init=True),
             dict(c="un_803F9A00", n=76, kind="int", keep=True, init=True),
             dict(c="un_803F9B30", ty="un_803F9B30", n=67, keep=True, init=True),
             dict(c="un_804D6D98", kind="ptr", fill="p:HSD_Archive"),
             dict(c="un_804D6D9C", kind="ptr", fill="p:SceneDesc"),
             dict(c="PRZM", rs="super::PRZM", n=2, kind="ptr", fill="p:DynamicModelDesc", region=True),
             dict(c="S16A", rs="super::S16A", n=4, kind="int", region=True),
             dict(c="U8A", rs="super::U8A", n=4, kind="int", fill="r:0:3", region=True),
             dict(c="S32A", rs="super::S32A", n=4, kind="int", fill="r:0:3", region=True),
             dict(c="HSD_GObj_JObjKind", ext=True, kind="int", fill="k:20"),
             dict(c="HSD_GObj_CameraKind", ext=True, kind="int", fill="k:21"),
             dict(c="HSD_GObj_LightKind", ext=True, kind="int", fill="k:22")],
    skip_data=["result"],
    c_decl=SPRINTF_C + "static DynamicModelDesc* PRZM[2];\nstatic s16 S16A[4];\nstatic u8 U8A[4];\nstatic int S32A[4];\n",
    rs_decl="pub static mut PRZM: [*mut DynamicModelDesc; 2] = [null_mut(); 2];\npub static mut S16A: [i16; 4] = [0; 4];\npub static mut U8A: [u8; 4] = [0; 4];\npub static mut S32A: [i32; 4] = [0; 4];\n",
    ret={"GObj_Create": ("pick_HSD_GObj(0)", "pick_HSD_GObj(0)"),
         "HSD_JObjLoadJoint": ("pick_HSD_JObj(0)", "pick_HSD_JObj(0)"),
         "HSD_CObjLoadDesc": ("&P_HSD_CObj[0]", "&raw mut P_HSD_CObj[0]"),
         "HSD_SisLib_803A5ACC": ("&P_HSD_Text[1]", "&raw mut P_HSD_Text[1]"),
         "HSD_SisLib_803A6754": ("&P_HSD_Text[2]", "&raw mut P_HSD_Text[2]"),
         "gm_GetButtonsTriggered": ("((fv() & 1) ? 0x1100ULL : (unsigned long long) fv())",
                                    "if fv() & 1 != 0 { 0x1100 } else { fv() as u64 }"),
         "lbLang_IsSavedLanguageUS": ("(fv() & 1)", "(fv() & 1) as _"),
         "lbArchive_80016DBC": ("&P_HSD_Archive[0]", "&raw mut P_HSD_Archive[0]")},
    effects={"gm_801692E8": ("a1->year = (u16) fv(); a1->month = (u8) (fv() % 14); a1->day = (u8) (fv() % 33); "
                             "a1->hour = (u8) (fv() % 25); a1->minute = (u8) (fv() % 61); a1->second = (u8) (fv() % 61);",
                             "(*a1).year = fv() as u16; (*a1).month = (fv() % 14) as u8; (*a1).day = (fv() % 33) as u8; "
                             "(*a1).hour = (fv() % 25) as u8; (*a1).minute = (fv() % 61) as u8; (*a1).second = (fv() % 61) as u8;")},
    null_ret_ok=["HSD_GObj_SetupProc", "lb_80011AC4", "HSD_SisLib_803A6B98", "HSD_SisLib_803A6530"],
    custom_stubs={"lbArchive_80016DBC": sections_stub(
        "lbArchive_80016DBC", ("const char*", "T(SH(a0));", "*const u8", "t(sh(a0));"),
        {"ScInfPrize_scene_data": ("&P_SceneDesc[0]", "&raw mut P_SceneDesc[0]")},
        ret=("HSD_Archive*", "&P_HSD_Archive[0]", "*mut HSD_Archive", "&raw mut P_HSD_Archive[0]"), a1ty="void*"),
        "HSD_SisLib_803A6B98": SIS_PRINT, "lb_80011E24": jobjs_stub()},
    ops=[dict(name="un_802FE3F8", args=["r:0:70", "r:0:30", ("raw", "((rnd() & 1) ? &S16A[0] : 0)", "if rnd() & 1 != 0 { &raw mut S16A[0] } else { null_mut() }"),
                                         ("raw", "((rnd() & 1) ? &S16A[2] : 0)", "if rnd() & 1 != 0 { &raw mut S16A[2] } else { null_mut() }")]),
         dict(name="un_802FE3F8_inner", args=["r:0:70", "r:0:30", ("raw", "((rnd() & 1) ? &S16A[0] : 0)", "if rnd() & 1 != 0 { &raw mut S16A[0] } else { null_mut() }"),
                                               ("raw", "((rnd() & 1) ? &S16A[2] : 0)", "if rnd() & 1 != 0 { &raw mut S16A[2] } else { null_mut() }")]),
         dict(name="un_802FE3F8_noinline", args=["r:0:70", "r:0:30", ("raw", "((rnd() & 1) ? &S16A[0] : 0)", "if rnd() & 1 != 0 { &raw mut S16A[0] } else { null_mut() }"),
                                                  ("raw", "((rnd() & 1) ? &S16A[2] : 0)", "if rnd() & 1 != 0 { &raw mut S16A[2] } else { null_mut() }")]),
         "ifPrize_RenderGuarded", "fn_802FE470", "un_802FE6A8", "un_802FE8CC", "setArchive",
         dict(name="un_802FE918_update_x3", args=[("raw", "&U8A[0]", "&raw mut U8A[0]"), ("raw", "&S32A[0]", "&raw mut S32A[0]")]),
         dict(name="un_802FE918", args=["r:0:70", "r:0:30", "v"]),
         dict(name="ifPrize_Scene_OnEnter", args=["p:un_802FEBE0_OnEnter_arg0"]), "ifPrize_Scene_OnExit"],
)

for _s in SPECS.values():
    _s["common_ret"] = COMMON_RET
