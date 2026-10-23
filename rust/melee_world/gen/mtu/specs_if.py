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

def _pl(k):
    return ("raw", f"&ifMagnify_804A1DE0.player[{k}]", f"&raw mut m::ifMagnify_804A1DE0.player[{k}]")


def _gcol_ret(k):
    return (f"((NULLRET && (fv() & 1)) ? 0 : &GCOL[{k}])",
            f"if super::NULLRET != 0 && fv() & 1 != 0 {{ null_mut() }} else {{ &raw mut super::GCOL[{k}] }}")


SPECS["ifmagnify"] = dict(
    src="src/melee/if/ifmagnify.c",
    # The C reaches ifMagnifyPlayer / ifMagnify members through GameCube-layout address arithmetic
    # ((u32) base + i * sizeof(ifMagnifyPlayer) + offsetof(...), (HSD_ImageDesc*) base + n, memzero 0x74);
    # native pointers are 8 bytes, so the compiled copy spells the same members and the same head
    # length the native layout has. The port does exactly that.
    patch=[("            HSD_GObjFree(*(HSD_GObj**) ((u32) base +\n"
            "                                        i * (s32) sizeof(ifMagnifyPlayer) +\n"
            "                                        (s32) offsetof(ifMagnify, player)));\n"
            "            *(HSD_GObj**) ((u32) base + i * (s32) sizeof(ifMagnifyPlayer) +\n"
            "                           (s32) offsetof(ifMagnify, player)) = NULL;\n",
            "            HSD_GObjFree(base->player[i].gobj);\n"
            "            base->player[i].gobj = NULL;\n"),
           ("            (base = (ifMagnify*) ((HSD_ImageDesc*) &ifMagnify_804A1DE0 +\n"
            "                                  (slot - 1)))\n"
            "                ->image_descs;\n",
            "            &base->image_descs[slot - 1];\n"),
           ("memzero(&ifMagnify_804A1DE0, 0x74);", "memzero(&ifMagnify_804A1DE0, offsetof(ifMagnify, image_descs));")],
    pools={"HSD_GObj": 6, "HSD_JObj": 8, "HSD_DObj": 4, "HSD_MObj": 3, "HSD_TObj": 3, "HSD_Material": 3,
           "HSD_ImageDesc": 3, "HSD_AObj": 2, "HSD_CObj": 2, "DynamicModelDesc": 2, "HSD_Joint": 2, "VsSceneController": 1,
           "HSD_Archive": 1},
    rules={**CHAIN_RULES,
           "HSD_DObj.next": ("ptr", "HSD_DObj", False),
           "HSD_TObj.imagedesc": ("ptr", "HSD_ImageDesc", False),
           "DynamicModelDesc.joint": ("ptr", "HSD_Joint", False),
           "ifMagnifyPlayer.gobj": ("ptr", "HSD_GObj", False), "ifMagnifyPlayer.jobj": ("ptr", "HSD_JObj", False),
           "ifMagnifyPlayer.idesc": ("ptr", "HSD_ImageDesc", False),
           "ifMagnify.model_desc": ("expr", "PMDL", "&raw mut super::PMDL")},
    statics=[dict(c="ifMagnify_804A1DE0", ty="ifMagnify"),
             dict(c="ifMagnify_LogicArmed", kind="int", fill="r:0:1"),
             dict(c="ifMagnify_LogicOffscreen", n=6, kind="int", fill="r:0:1"),
             dict(c="ifMagnify_803F97C0", ty="HSD_WObjDesc", keep=True, init=True),
             dict(c="ifMagnify_803F97D4", ty="HSD_WObjDesc", keep=True, init=True),
             dict(c="ifMagnify_803F97E8", ty="HSD_CameraDescFrustum", keep=True, init=True),
             dict(c="ifMagnify_804D57E8", n=5, kind="int", keep=True, init=True),
             dict(c="ifMagnify_803F9828", n=9, kind="fnp", keep=True, init=True),
             dict(c="ifMagnify_803F984C", flat=("unsigned char", "u8"), n=64, kind="int", keep=True, init=True),
             dict(c="GCOL", rs="super::GCOL", ty="GXColor", n=9, region=True),
             dict(c="STG", rs="super::STG", n=4, kind="f32", fill="f", region=True),
             dict(c="VEC2A", rs="super::VEC2A", ty="Vec2", n=2, region=True),
             dict(c="V3A", rs="super::V3A", ty="Vec3", region=True),
             dict(c="NULLRET", rs="super::NULLRET", kind="int", fill="k:0"),
             dict(c="PMDL", rs="super::PMDL", n=2, kind="ptr", fill="p:DynamicModelDesc", region=True),
             dict(c="HEAPB", rs="super::HEAPB", n=16, kind="int", region=True),
             dict(c="ARCP", rs="super::ARCP", kind="ptr", fill="k:0", region=True),
             dict(c="HSD_GObj_804D7814", ext=True, kind="ptr", fill="pn:HSD_GObj"),
             dict(c="HSD_GObj_JObjKind", ext=True, kind="int", fill="k:20"),
             dict(c="HSD_GObj_CameraKind", ext=True, kind="int", fill="k:21")],
    skip_data=["result"],
    c_decl="static GXColor GCOL[9];\nstatic float STG[4];\nstatic Vec2 VEC2A[2];\nstatic Vec3 V3A;\nstatic int NULLRET;\n"
           "static DynamicModelDesc* PMDL[2];\nstatic u8 HEAPB[16];\nstatic HSD_Archive* ARCP;\n"
           "static GXColor rcol(void) { GXColor c; c.r = (u8) fv(); c.g = (u8) fv(); c.b = (u8) fv(); c.a = (u8) fv(); return c; }\n"
           "static int view_bits(void) { static const u32 k[3] = { 0x3FAAAAABu, 0x3FE38E39u, 0x40000000u }; u32 q = fv() & 3; "
           "return (int) (q < 3 ? k[q] : fv()); }\n",
    rs_decl="pub static mut GCOL: [_GXColor; 9] = unsafe { core::mem::zeroed() };\npub static mut STG: [f32; 4] = [0.0; 4];\n"
            "pub static mut VEC2A: [Vec2; 2] = unsafe { core::mem::zeroed() };\npub static mut V3A: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };\npub static mut NULLRET: i32 = 0;\n"
            "pub static mut PMDL: [*mut DynamicModelDesc; 2] = [null_mut(); 2];\npub static mut HEAPB: [u8; 16] = [0; 16];\n"
            "pub static mut ARCP: *mut HSD_Archive = null_mut();\n"
            "fn rcol() -> _GXColor { let r = fv() as u8; let g = fv() as u8; let b = fv() as u8; let a = fv() as u8; _GXColor { r, g, b, a } }\n"
            "fn view_bits() -> i32 { const K: [u32; 3] = [0x3FAAAAAB, 0x3FE38E39, 0x40000000]; let q = fv() & 3; (if q < 3 { K[q as usize] } else { fv() }) as i32 }\n",
    defaults={"memzero": "super::memzero_default"},
    ret={"GObj_Create": ("pick_HSD_GObj(0)", "pick_HSD_GObj(0)"),
         "HSD_JObjLoadJoint": ("pick_HSD_JObj(0)", "pick_HSD_JObj(0)"),
         "lb_80013B14": ("&P_HSD_CObj[0]", "&raw mut P_HSD_CObj[0]"),
         "gmVs_GetSceneController": ("&P_VsSceneController[0]", "&raw mut P_VsSceneController[0]"),
         "ifAll_GetArchive": ("&ARCP", "&raw mut super::ARCP"),
         "HSD_MemAlloc": ("HEAPB", "&raw mut super::HEAPB"),
         "Stage_GetCamBoundsTopOffset": ("STG[0]", "super::STG[0]"), "Stage_GetCamBoundsBottomOffset": ("STG[1]", "super::STG[1]"),
         "Stage_GetCamBoundsLeftOffset": ("STG[2]", "super::STG[2]"), "Stage_GetCamBoundsRightOffset": ("STG[3]", "super::STG[3]"),
         "Ground_801C0604": _gcol_ret(0), "Ground_801C0618": _gcol_ret(2), "Ground_801C062C": _gcol_ret(6),
         "Ground_801C0640": _gcol_ret(8), "Ground_801C0654": _gcol_ret(3), "Ground_801C0668": _gcol_ret(5),
         "Ground_801C067C": _gcol_ret(7), "Ground_801C0690": _gcol_ret(1), "Ground_801C06A4": _gcol_ret(4),
         "View_AspectBits": ("view_bits()", "super::view_bits()"),
         "Camera_80030A50": ("((fv() & 1) ? &P_HSD_GObj[0] : 0)", "if fv() & 1 != 0 { &raw mut P_HSD_GObj[0] } else { null_mut() }"),
         "gm_80160968": ("rcol()", "super::rcol()"),
         "Player_GetEntity": ("((fv() & 3) == 0 ? 0 : &P_HSD_GObj[1 + fv() % 5])",
                              "if fv() & 3 == 0 { null_mut() } else { &raw mut P_HSD_GObj[(1 + fv() % 5) as usize] }"),
         "Player_GetTeam": ("(fv() % 5)", "(fv() % 5) as _"), "Player_GetPlayerSlotType": ("(fv() % 4)", "fv() % 4"),
         "gm_8016B168": ("(fv() & 1)", "(fv() & 1) as _"), "gm_80160854": ("fv()", "fv()"),
         "GXGetTexBufferSize": ("(fv() % 100000)", "fv() % 100000"),
         "Camera_80030BBC": ("(fv() & 1)", "(fv() & 1) as _"),
         "Camera_80030130": ("(fv() & 1)", "(fv() & 1) as _"), "ifAll_IsHUDHidden": ("(fv() & 1)", "(fv() & 1) as _"),
         "ftLib_80086B64": ("(fv() & 1)", "(fv() & 1) as _"), "ftLib_80086ED0": ("(fv() & 1)", "(fv() & 1) as _"),
         "HSD_CObjSetCurrent": ("(fv() & 1)", "(fv() & 1) as _"),
         "Wb_RenderActive": ("(fv() & 1)", "(fv() & 1) as _")},
    effects={**MEMZERO,
             "ftLib_80086B90": ("a1->x = rw(); a1->y = rw(); a1->z = rw();", "(*a1).x = rw(); (*a1).y = rw(); (*a1).z = rw();"),
             "Player_80036978": ("a1->x = rw(); a1->y = rw(); a1->z = rw();", "(*a1).x = rw(); (*a1).y = rw(); (*a1).z = rw();"),
             "Camera_80030BBC": ("if (a1) { a1->x = (int) (fv() % 800) - 80; a1->y = (int) (fv() % 600) - 60; }",
                                 "if !a1.is_null() { (*a1).x = (fv() % 800) as i32 - 80; (*a1).y = (fv() % 600) as i32 - 60; }"),
             "HSD_CObjGetOrtho": ("*a1 = rw(); *a2 = rw(); *a3 = rw(); *a4 = rw();",
                                  "*a1 = rw(); *a2 = rw(); *a3 = rw(); *a4 = rw();")},
    deref={"Camera_80030BBC": [0], "HSD_CObjSetViewport": [1], "un_802FD928": [2]},
    null_ret_ok=["HSD_GObj_SetupProc"],
    custom_stubs={"lbArchive_LoadSections": sections_stub(
        "lbArchive_LoadSections", ("HSD_Archive*", "T(PI(a0));", "*mut HSD_Archive", "t(pi(a0));"),
        {"lupe": ("PMDL", "&raw mut super::PMDL")}), "lb_80011E24": jobjs_stub()},
    ops=[dict(name="ifMagnify_ViewAspect"),
         dict(name="ifMagnify_VisualOffscreen", args=["p:HSD_GObj"]),
         "ifMagnify_IsHUDVisible",
         dict(name="ifMagnify_GetPlayerColor", args=[("raw", "&GCOL[0]", "&raw mut GCOL[0]"), "i:6"]),
         dict(name="ifMagnify_802FB6E8", args=["i:6"]),
         dict(name="ifMagnify_802FB73C", args=[_pl(0), ("raw", "&VEC2A[0]", "&raw mut VEC2A[0]"),
                                               ("raw", "&VEC2A[1]", "&raw mut VEC2A[1]")]),
         dict(name="ifMagnify_802FB8C0", args=[G0, "r:0:2"],
              pre_c="P_HSD_GObj[0].user_data = &ifMagnify_804A1DE0.player[rnd() % 6u];",
              pre_rs="g::P_HSD_GObj[0].user_data = &raw mut m::ifMagnify_804A1DE0.player[(rnd() % 6) as usize] as *mut _;"),
         dict(name="ifMagnify_GetCornerColors", args=[("raw", "&GCOL[0]", "&raw mut GCOL[0]"), ("raw", "&V3A", "&raw mut V3A")]),
         "ifMagnify_802FBBDC", "ifMagnify_802FC3BC", dict(name="ifMagnify_802FC3C0", args=["i:6"]),
         "ifMagnify_802FC618", "ifMagnify_802FC750",
         dict(name="ifMagnify_802FC7C0", args=[("raw", "&ifMagnify_804A1DE0", "&raw mut m::ifMagnify_804A1DE0")],
              pre_c="NULLRET = 1;", pre_rs="NULLRET = 1;"),
         "ifMagnify_802FC870", "ifMagnify_802FC8E8", "ifMagnify_802FC940", "ifMagnify_LogicDisarm",
         "ifMagnify_UpdateLogicOffscreen", dict(name="ifMagnify_802FC998", args=["i:6"])],
)

# the player record the op acts on, drawn once (the C text and the Rust text draw identically)
HPK = ("raw", "&ifStatus_HudInfo.players[rnd() % 6u]", "&raw mut m::ifStatus_HudInfo.players[(rnd() % 6) as usize]")
UNKX = ("raw", "(UnkX*) &ifStatus_HudInfo.players[rnd() % 6u]", "&raw mut m::ifStatus_HudInfo.players[(rnd() % 6) as usize]")

_UNKX_BLOCK = ("typedef struct UnkX {\n    u8 filler1[0x10];\n    FlagsX x10_flags;\n    u8 filler2[0x34 - 0x14];\n"
               "    Vec4 x34_vec; // or float[4] instead of Vec4\n    Vec4 x44_vec;\n    HSD_JObj* x54_jobj[4];\n"
               "} UnkX; // HudIndex\n")

_PARENT_C = "ifStatus_HudInfo.players[rnd() % 6u].HUD_parent_entity = &P_HSD_GObj[0];"
_PARENT_R = "m::ifStatus_HudInfo.players[(rnd() % 6) as usize].HUD_parent_entity = &raw mut g::P_HSD_GObj[0];"
_NEXT_C = "ifStatus_HudInfo.players[rnd() % 6u].next = &P_HSD_GObj[0];"
_NEXT_R = "m::ifStatus_HudInfo.players[(rnd() % 6) as usize].next = &raw mut g::P_HSD_GObj[0];"
# VsSceneFighter's anonymous flag byte is raw random bytes; the native C reads x4_b5 at LSB bit 5, the
# GameCube-order accessor at bit 2: bytes with both set (0x24) or neither read the same on both sides.
_FIGHTERS_C = " { int q; for (q = 0; q < 6; q++) *((u8*) &P_VsSceneController[0].state.fighters[q] + 4) = (rnd() & 1) ? 0x24 : 0; }"
_FIGHTERS_R = " for q in 0..6usize { *(&raw mut g::P_VsSceneController[0].state.fighters[q] as *mut u8).add(4) = if rnd() & 1 != 0 { 0x24 } else { 0 }; }"

_UNK9_C = "{ int q; for (q = 0; q < 6; q++) ifStatus_HudInfo.players[q].unk9 = 0; }"
_UNK9_R = "for q in 0..6usize { m::ifStatus_HudInfo.players[q].unk9 = 0; }"

SPECS["ifstatus"] = dict(
    src="src/melee/if/ifstatus.c",
    # UnkX is a GameCube-layout view of IfDamageState (fixed 0x10 / 0x34 / 0x54 offsets); natively the
    # records have 8-byte pointers, so the compiled copy spells the same members through IfDamageState.
    patch=[(_UNKX_BLOCK, "typedef IfDamageState UnkX;\n"),
           ("    if (value->x10_flags.b40) {", "    if (value->flags.randomize_velocity) {"),
           ("        value->x10_flags.b40 = 0;", "        value->flags.randomize_velocity = 0;"),
           ("            (&value->x34_vec.x)[i] =\n                foo(0.6083f * HSD_Randf(), 0.3041f); // var_f0;\n"
            "            (&value->x44_vec.x)[i] = 0.811f * HSD_Randf() + 1.2165f;",
            "            value->velocity_x[i] =\n                foo(0.6083f * HSD_Randf(), 0.3041f); // var_f0;\n"
            "            value->velocity_y[i] = 0.811f * HSD_Randf() + 1.2165f;"),
           ("            float f = (&value->x34_vec.x)[i];", "            float f = value->velocity_x[i];"),
           ("            float f = (&value->x44_vec.x)[i];", "            float f = value->velocity_y[i];"),
           ("            (&value->x44_vec.x)[i] -= 0.2028f;", "            value->velocity_y[i] -= 0.2028f;"),
           ("    return value->x54_jobj[i];", "    return value->jobjs[i];"),
           ("        HSD_JObj* jobj_r30 = value->x54_jobj[i];", "        HSD_JObj* jobj_r30 = value->jobjs[i];")],
    pools={"HSD_GObj": 6, "HSD_JObj": 10, "HSD_DObj": 4, "HSD_MObj": 4, "HSD_TObj": 4, "HSD_Material": 4, "HSD_AObj": 3,
           "HSD_MatAnimJoint": 6, "HSD_MatAnim": 2, "DynamicModelDesc": 2, "VsSceneController": 1, "HSD_Archive": 1},
    rules={**CHAIN_RULES,
           "HSD_JObj.child": ("ptr", "HSD_JObj", False), "HSD_JObj.next": ("ptr", "HSD_JObj", False),
           "HSD_MatAnimJoint.child": ("ptr", "HSD_MatAnimJoint", False), "HSD_MatAnimJoint.next": ("ptr", "HSD_MatAnimJoint", False),
           "HSD_MatAnimJoint.matanim": ("ptr", "HSD_MatAnim", False),
           "IfDamageState.HUD_parent_entity": ("ptr", "HSD_GObj", True), "IfDamageState.next": ("ptr", "HSD_GObj", True),
           "IfDamageState.jobjs": ("ptr", "HSD_JObj", False),
           "IfDamageState.player_slot": ("range", 0, 5),
           "HudIndex.damage_num_matanims": ("expr", "MAJP", "&raw mut super::MAJP"),
           "HudIndex.damage_mark_matanims": ("expr", "MAJP", "&raw mut super::MAJP"),
           "VsSceneState.unk_D": ("range", 0, 5)},
    statics=[dict(c="ifStatus_HudInfo", ty="HudIndex"),
             dict(c="ifStatus_804D57A8", n=4, kind="int", keep=True, init=True),
             dict(c="ifStatus_804D57AC", n=4, kind="int", keep=True, init=True),
             dict(c="ifStatus_804D6D60", kind="int", fill="r:0:7"),
             dict(c="ifStatus_804D6D61", kind="int", fill="r:0:1"),
             dict(c="MAJP", rs="super::MAJP", n=2, kind="ptr", fill="p:HSD_MatAnimJoint", region=True),
             dict(c="NMOD", rs="super::NMOD", n=2, kind="ptr", fill="p:DynamicModelDesc", region=True),
             dict(c="ARCP", rs="super::ARCP", kind="ptr", fill="p:HSD_Archive", region=True),
             dict(c="V3A", rs="super::V3A", ty="Vec3", region=True),
             dict(c="GCOL", rs="super::GCOL", ty="GXColor", n=3, region=True),
             dict(c="GOP", rs="super::GOP", n=1, kind="ptr", fill="k:0", region=True),
             dict(c="CHR", rs="super::CHR", n=1, kind="int", fill="v", region=True),
             dict(c="HSD_GObj_JObjKind", ext=True, kind="int", fill="k:20")],
    skip_data=["result", "num_models_name", "mrk_models_name"],
    c_decl="static GXColor GCOL[3];\nstatic HSD_GObj* GOP[1];\nstatic u32 CHR[1];\nstatic void cbA(s32 a) { T(7001); T((u32) a); }\nstatic void cbB(s32 a) { T(7002); T((u32) a); }\n"
           "static void* cbsel(void) { u32 q = rnd() % 3; return q == 0 ? (void*) 0 : q == 1 ? (void*) cbA : (void*) cbB; }\n"
           "static HSD_MatAnimJoint* MAJP[2];\nstatic DynamicModelDesc* NMOD[2];\nstatic HSD_Archive* ARCP;\nstatic Vec3 V3A;\n"
           "static GXColor rcol(void) { GXColor c; c.r = (u8) fv(); c.g = (u8) fv(); c.b = (u8) fv(); c.a = (u8) fv(); return c; }\n",
    rs_decl="use melee_world::tu::ifstatus::UnkX;\npub static mut GCOL: [_GXColor; 3] = unsafe { core::mem::zeroed() };\npub static mut GOP: [*mut HSD_GObj; 1] = [null_mut(); 1];\npub static mut CHR: [u32; 1] = [0; 1];\n"
            "unsafe extern \"C\" fn cbA(a: i32) { t(7001); t(a as u32); }\nunsafe extern \"C\" fn cbB(a: i32) { t(7002); t(a as u32); }\n"
            "fn cbsel() -> usize { let q = rnd() % 3; if q == 0 { 0 } else if q == 1 { cbA as usize } else { cbB as usize } }\n"
            "pub static mut MAJP: [*mut HSD_MatAnimJoint; 2] = [null_mut(); 2];\n"
            "pub static mut NMOD: [*mut DynamicModelDesc; 2] = [null_mut(); 2];\npub static mut ARCP: *mut HSD_Archive = null_mut();\n"
            "pub static mut V3A: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };\n"
            "fn rcol() -> _GXColor { let r = fv() as u8; let g = fv() as u8; let b = fv() as u8; let a = fv() as u8; _GXColor { r, g, b, a } }\n",
    defaults={"memzero": "super::memzero_default"},
    ret={"GObj_Create": ("pick_HSD_GObj(0)", "pick_HSD_GObj(0)"),
         "HSD_JObjLoadJoint": ("pick_HSD_JObj(0)", "pick_HSD_JObj(0)"),
         "gmVs_GetSceneController": ("&P_VsSceneController[0]", "&raw mut P_VsSceneController[0]"),
         "gmVs_GetSceneState": ("&P_VsSceneController[0].state", "&raw mut P_VsSceneController[0].state"),
         "gm_GetStartMeleeRules": ("&P_VsSceneController[0].start", "&raw mut P_VsSceneController[0].start"),
         "ifAll_GetArchive": ("&ARCP", "&raw mut super::ARCP"),
         "ifAll_GetPlayerHUDPosition": ("&V3A", "&raw mut super::V3A"),
         "lbDvd_8001819C": ("((fv() & 1) ? &P_HSD_Archive[0] : 0)", "if fv() & 1 != 0 { &raw mut P_HSD_Archive[0] } else { null_mut() }"),
         "HSD_ArchiveGetPublicAddress": ("((fv() & 1) ? (void*) &P_HSD_MatAnimJoint[1] : 0)",
                                         "if fv() & 1 != 0 { &raw mut P_HSD_MatAnimJoint[1] as *mut c_void } else { null_mut() }"),
         "Mex_InsigniaForExt": ("((fv() & 1) ? -1 : (int) (fv() % 20))", "if fv() & 1 != 0 { -1 } else { (fv() % 20) as i32 }"),
         "Player_GetEntity": ("((fv() & 3) == 0 ? 0 : &P_HSD_GObj[1 + fv() % 5])",
                              "if fv() & 3 == 0 { null_mut() } else { &raw mut P_HSD_GObj[(1 + fv() % 5) as usize] }"),
         "Player_GetMoreFlagsBit2": ("(fv() & 1)", "(fv() & 1) as _"), "lb_8000B09C": ("(fv() & 1)", "(fv() & 1) as _"),
         "Player_GetPlayerSlotType": ("(fv() % 4)", "fv() % 4"), "Player_GetTeam": ("(fv() % 5)", "(fv() % 5) as _"),
         "gm_8016B168": ("(fv() & 1)", "(fv() & 1) as _"), "gm_8016B238": ("(fv() & 1)", "(fv() & 1) as _"),
         "gm_80160854": ("fv()", "fv()"), "gm_80160968": ("rcol()", "super::rcol()"),
         "Player_GetRemainingHP": ("((fv() & 1) ? (int) (fv() % 1200) - 100 : (int) fv())", "if fv() & 1 != 0 { (fv() % 1200) as i32 - 100 } else { fv() as i32 }"),
         "Player_GetDamage": ("((fv() & 1) ? (int) (fv() % 1200) - 100 : (int) fv())", "if fv() & 1 != 0 { (fv() % 1200) as i32 - 100 } else { fv() as i32 }"),
         "Player_GetStocks": ("(fv() % 3)", "(fv() % 3) as _"),
         "Script_StatusHUDVisible": ("((fv() & 3) != 0)", "((fv() & 3) != 0) as i32"),
         "Ui_RetailHidden": ("((fv() & 3) == 0)", "((fv() & 3) == 0) as i32")},
    effects=MEMZERO,
    extra_freg=[("cbA", "cbA"), ("cbB", "cbB")],
    null_ret_ok=["HSD_GObj_SetupProc", "HSD_JObjAddAnimAll"],
    custom_stubs={"lbArchive_LoadSections": sections_stub(
        "lbArchive_LoadSections", ("HSD_Archive*", "T(PI(a0));", "*mut HSD_Archive", "t(pi(a0));"),
        {"DmgNum_scene_models": ("NMOD", "&raw mut super::NMOD"), "DmgMrk_scene_models": ("&NMOD[1]", "&raw mut super::NMOD[1]")})},
    setup_c="int k;\n    for (k = 0; k < 6; k++) P_HSD_GObj[k].hsd_obj = &P_HSD_JObj[k];",
    setup_rs="for k in 0..6usize { g::P_HSD_GObj[k].hsd_obj = &raw mut g::P_HSD_JObj[k] as *mut _; }",
    ops=[dict(name="ifStatus_GetHUDInfo"),
         dict(name="ifStatus_PercentOnDeathAnimationThink", args=[UNKX, "v", "v"]),
         dict(name="foo", args=["f", "f"]),
         "offset_rand",
         dict(name="jobj_flagCheckSetMtxDirtySub", args=["p:HSD_JObj"]),
         dict(name="ifStatus_802F4B84", args=[HPK, "v"]),
         dict(name="ifStatus_InitDamageDigits", args=[HPK, ("raw", "(HSD_MatAnimJoint***) &ifStatus_HudInfo.damage_num_matanims", "&raw mut m::ifStatus_HudInfo.damage_num_matanims")]),
         dict(name="ifStatus_GetStoredTranslationX", args=[HPK, "i:4"]),
         dict(name="ifStatus_CreateMarkGObj", args=[("raw", "&GOP[0]", "&raw mut GOP[0]")]),
         dict(name="ifStatus_GetPlayerCharacter", args=["v", ("raw", "(CharacterKind*) &CHR[0]", "&raw mut CHR[0]")]),
         dict(name="ifStatus_UpdateDamageDisplay", args=[HPK, "p:HSD_JObj", ("raw", "&GCOL[0]", "&raw mut GCOL[0]"),
                                                         ("raw", "&GCOL[1]", "&raw mut GCOL[1]"), ("raw", "&GCOL[2]", "&raw mut GCOL[2]")]),
         dict(name="ifStatus_802F4EDC", args=[G0], pre_c=_PARENT_C, pre_rs=_PARENT_R),
         dict(name="ifStatus_802F5B48", args=[G0], pre_c=_PARENT_C + _FIGHTERS_C, pre_rs=_PARENT_R + _FIGHTERS_R),
         dict(name="ifStatus_802F5DE0", args=[G0, "v"], pre_c=_PARENT_C, pre_rs=_PARENT_R),
         dict(name="ifStatus_802F5E50", args=[G0, "v"], pre_c=_NEXT_C, pre_rs=_NEXT_R),
         dict(name="ifStatus_802F5EC0", args=[HPK, "i:6"],
              pre_c="{ int q; for (q = 0; q < 6; q++) ifStatus_HudInfo.players[q].unk9 = 0; }" + _FIGHTERS_C,
              pre_rs="for q in 0..6usize { m::ifStatus_HudInfo.players[q].unk9 = 0; }" + _FIGHTERS_R),
         dict(name="ifStatus_802F6194", args=["pn:HSD_JObj", "r:-2:5"],
              pre_c="if (rnd() & 1) P_HSD_JObj[rnd() % 10u].next = 0;", pre_rs="if rnd() & 1 != 0 { g::P_HSD_JObj[(rnd() % 10) as usize].next = null_mut(); }"),
         dict(name="ifStatus_802F61FC", args=[HPK, "i:6"]),
         dict(name="ifStatus_802F6508", args=["i:6"], pre_c=_UNK9_C + _FIGHTERS_C, pre_rs=_UNK9_R + _FIGHTERS_R),
         dict(name="ifStatus_802F665C", args=["r:0:7"], pre_c=_UNK9_C + _FIGHTERS_C, pre_rs=_UNK9_R + _FIGHTERS_R),
         "ifStatus_802F66A4",
         dict(name="ifStatus_802F6788", args=["i:6"]),
         "ifStatus_802F6804", "ifStatus_802F6898", "ifStatus_802F68F0",
         dict(name="ifStatus_802F6948", args=["i:6"]),
         dict(name="ifStatus_TriggerStockLoss", args=["i:6", ("raw", "(void (*)(s32)) cbsel()", "core::mem::transmute::<usize, Option<unsafe extern \"C\" fn(i32)>>(cbsel())")]),
         dict(name="ifStatus_802F69C0", args=["i:6", "v"]),
         dict(name="ifStatus_802F6AF8", args=["i:6"]), dict(name="ifStatus_802F6C04", args=["i:6"]),
         dict(name="ifStatus_802F6D10", args=["i:6"]), dict(name="ifStatus_802F6E1C", args=["i:6"], pre_c=_UNK9_C + _FIGHTERS_C, pre_rs=_UNK9_R + _FIGHTERS_R),
         dict(name="ifStatus_802F6E3C", args=["i:6"])],
)

_COINS = ("raw", "((rnd() & 3) == 0 ? (int) (rnd() % 12u) : (int) (rnd() % 120000u))",
          "if rnd() & 3 == 0 { (rnd() % 12) as i32 } else { (rnd() % 120000) as i32 }")
_FPTR = ("raw", "&FPTR", "&raw mut super::FPTR")
_FPTR_C = "FBUF[63] = 0; FPTR = &FBUF[rnd() % 8u];"
_FPTR_R = "super::FBUF[63] = 0; super::FPTR = &raw mut super::FBUF[(rnd() % 8) as usize];"
_MATCHFLAGS_C = (" { int q; for (q = 0; q < 6; q++) { u32 m = rnd(); u32 bb = (m & 1) | (((m >> 1) & 1) << 1); "
                 "*((u8*) &P_VsSceneController[0].state.fighters[q] + 4) = (u8) (bb | ((bb & 1) << 7) | ((bb & 2) << 5)); } }")
_MATCHFLAGS_R = (" for q in 0..6usize { let m = rnd(); let bb = (m & 1) | (((m >> 1) & 1) << 1); "
                 "*(&raw mut g::P_VsSceneController[0].state.fighters[q] as *mut u8).add(4) = (bb | ((bb & 1) << 7) | ((bb & 2) << 5)) as u8; }")
_COINS = ("raw", "((rnd() & 3) == 0 ? (int) (rnd() % 12u) : (int) (rnd() % 120000u))",
          "if rnd() & 3 == 0 { (rnd() % 12) as i32 } else { (rnd() % 120000) as i32 }")
_FPTR = ("raw", "&FPTR", "&raw mut super::FPTR")
_FPTR_C = "FBUF[63] = 0; FPTR = &FBUF[rnd() % 8u];"
_FPTR_R = "super::FBUF[63] = 0; super::FPTR = &raw mut super::FBUF[(rnd() % 8) as usize];"
_MATCHFLAGS_C = (" { int q; for (q = 0; q < 6; q++) { u32 m = rnd(); u32 bb = (m & 1) | (((m >> 1) & 1) << 1); "
                 "*((u8*) &P_VsSceneController[0].state.fighters[q] + 4) = (u8) (bb | ((bb & 1) << 7) | ((bb & 2) << 5)); } }")
_MATCHFLAGS_R = (" for q in 0..6usize { let m = rnd(); let bb = (m & 1) | (((m >> 1) & 1) << 1); "
                 "*(&raw mut g::P_VsSceneController[0].state.fighters[q] as *mut u8).add(4) = (bb | ((bb & 1) << 7) | ((bb & 2) << 5)) as u8; }")
_COINS = ("raw", "((rnd() & 3) == 0 ? (int) (rnd() % 12u) : (int) (rnd() % 120000u))",
          "if rnd() & 3 == 0 { (rnd() % 12) as i32 } else { (rnd() % 120000) as i32 }")
_FPTR = ("raw", "&FPTR", "&raw mut super::FPTR")
_FPTR_C = "FBUF[63] = 0; FPTR = &FBUF[rnd() % 8u];"
_FPTR_R = "super::FBUF[63] = 0; super::FPTR = &raw mut super::FBUF[(rnd() % 8) as usize];"
_MATCHFLAGS_C = (" { int q; for (q = 0; q < 6; q++) { u32 m = rnd(); u32 bb = (m & 1) | (((m >> 1) & 1) << 1); "
                 "*((u8*) &P_VsSceneController[0].state.fighters[q] + 4) = (u8) (bb | ((bb & 1) << 7) | ((bb & 2) << 5)); } }")
_MATCHFLAGS_R = (" for q in 0..6usize { let m = rnd(); let bb = (m & 1) | (((m >> 1) & 1) << 1); "
                 "*(&raw mut g::P_VsSceneController[0].state.fighters[q] as *mut u8).add(4) = (bb | ((bb & 1) << 7) | ((bb & 2) << 5)) as u8; }")
_COINS = ("raw", "((rnd() & 3) == 0 ? (int) (rnd() % 12u) : (int) (rnd() % 120000u))",
          "if rnd() & 3 == 0 { (rnd() % 12) as i32 } else { (rnd() % 120000) as i32 }")
_FPTR = ("raw", "&FPTR", "&raw mut super::FPTR")
_FPTR_C = "FBUF[63] = 0; FPTR = &FBUF[rnd() % 8u];"
_FPTR_R = "super::FBUF[63] = 0; super::FPTR = &raw mut super::FBUF[(rnd() % 8) as usize];"
_MATCHFLAGS_C = (" { int q; for (q = 0; q < 6; q++) { u32 m = rnd(); u32 bb = (m & 1) | (((m >> 1) & 1) << 1); "
                 "*((u8*) &P_VsSceneController[0].state.fighters[q] + 4) = (u8) (bb | ((bb & 1) << 7) | ((bb & 2) << 5)); } }")
_MATCHFLAGS_R = (" for q in 0..6usize { let m = rnd(); let bb = (m & 1) | (((m >> 1) & 1) << 1); "
                 "*(&raw mut g::P_VsSceneController[0].state.fighters[q] as *mut u8).add(4) = (bb | ((bb & 1) << 7) | ((bb & 2) << 5)) as u8; }")
_COINS = ("raw", "((rnd() & 3) == 0 ? (int) (rnd() % 12u) : (int) (rnd() % 120000u))",
          "if rnd() & 3 == 0 { (rnd() % 12) as i32 } else { (rnd() % 120000) as i32 }")
_FPTR = ("raw", "&FPTR", "&raw mut super::FPTR")
_FPTR_C = "FBUF[63] = 0; FPTR = &FBUF[rnd() % 8u];"
_FPTR_R = "super::FBUF[63] = 0; super::FPTR = &raw mut super::FBUF[(rnd() % 8) as usize];"
_MATCHFLAGS_C = (" { int q; for (q = 0; q < 6; q++) { u32 m = rnd(); u32 bb = (m & 1) | (((m >> 1) & 1) << 1); "
                 "*((u8*) &P_VsSceneController[0].state.fighters[q] + 4) = (u8) (bb | ((bb & 1) << 7) | ((bb & 2) << 5)); } }")
_MATCHFLAGS_R = (" for q in 0..6usize { let m = rnd(); let bb = (m & 1) | (((m >> 1) & 1) << 1); "
                 "*(&raw mut g::P_VsSceneController[0].state.fighters[q] as *mut u8).add(4) = (bb | ((bb & 1) << 7) | ((bb & 2) << 5)) as u8; }")
_COINS = ("raw", "((rnd() & 3) == 0 ? (int) (rnd() % 12u) : (int) (rnd() % 120000u))",
          "if rnd() & 3 == 0 { (rnd() % 12) as i32 } else { (rnd() % 120000) as i32 }")
_FPTR = ("raw", "&FPTR", "&raw mut super::FPTR")
_FPTR_C = "FBUF[63] = 0; FPTR = &FBUF[rnd() % 8u];"
_FPTR_R = "super::FBUF[63] = 0; super::FPTR = &raw mut super::FBUF[(rnd() % 8) as usize];"
_MATCHFLAGS_C = (" { int q; for (q = 0; q < 6; q++) { u32 m = rnd(); u32 bb = (m & 1) | (((m >> 1) & 1) << 1); "
                 "*((u8*) &P_VsSceneController[0].state.fighters[q] + 4) = (u8) (bb | ((bb & 1) << 7) | ((bb & 2) << 5)); } }")
_MATCHFLAGS_R = (" for q in 0..6usize { let m = rnd(); let bb = (m & 1) | (((m >> 1) & 1) << 1); "
                 "*(&raw mut g::P_VsSceneController[0].state.fighters[q] as *mut u8).add(4) = (bb | ((bb & 1) << 7) | ((bb & 2) << 5)) as u8; }")
_COINS = ("raw", "((rnd() & 3) == 0 ? (int) (rnd() % 12u) : (int) (rnd() % 120000u))",
          "if rnd() & 3 == 0 { (rnd() % 12) as i32 } else { (rnd() % 120000) as i32 }")
_FPTR = ("raw", "&FPTR", "&raw mut super::FPTR")
_FPTR_C = "FBUF[63] = 0; FPTR = &FBUF[rnd() % 8u];"
_FPTR_R = "super::FBUF[63] = 0; super::FPTR = &raw mut super::FBUF[(rnd() % 8) as usize];"
_MATCHFLAGS_C = (" { int q; for (q = 0; q < 6; q++) { u32 m = rnd(); u32 bb = (m & 1) | (((m >> 1) & 1) << 1); "
                 "*((u8*) &P_VsSceneController[0].state.fighters[q] + 4) = (u8) (bb | ((bb & 1) << 7) | ((bb & 2) << 5)); } }")
_MATCHFLAGS_R = (" for q in 0..6usize { let m = rnd(); let bb = (m & 1) | (((m >> 1) & 1) << 1); "
                 "*(&raw mut g::P_VsSceneController[0].state.fighters[q] as *mut u8).add(4) = (bb | ((bb & 1) << 7) | ((bb & 2) << 5)) as u8; }")
_COINS = ("raw", "((rnd() & 3) == 0 ? (int) (rnd() % 12u) : (int) (rnd() % 120000u))",
          "if rnd() & 3 == 0 { (rnd() % 12) as i32 } else { (rnd() % 120000) as i32 }")
_FPTR = ("raw", "&FPTR", "&raw mut FPTR")
_FPTR_C = "FBUF[63] = 0; FPTR = &FBUF[rnd() % 8u];"
_FPTR_R = "FBUF[63] = 0; FPTR = &raw mut FBUF[(rnd() % 8) as usize];"
_MATCHFLAGS_C = (" { int q; for (q = 0; q < 6; q++) { u32 m = rnd(); u32 bb = (m & 1) | (((m >> 1) & 1) << 1); "
                 "*((u8*) &P_VsSceneController[0].state.fighters[q] + 4) = (u8) (bb | ((bb & 1) << 7) | ((bb & 2) << 5)); } }")
_MATCHFLAGS_R = (" for q in 0..6usize { let m = rnd(); let bb = (m & 1) | (((m >> 1) & 1) << 1); "
                 "*(&raw mut g::P_VsSceneController[0].state.fighters[q] as *mut u8).add(4) = (bb | ((bb & 1) << 7) | ((bb & 2) << 5)) as u8; }")
_COINS = ("raw", "((rnd() & 3) == 0 ? (int) (rnd() % 12u) : (int) (rnd() % 120000u))",
          "if rnd() & 3 == 0 { (rnd() % 12) as i32 } else { (rnd() % 120000) as i32 }")
_FPTR = ("raw", "&FPTR", "&raw mut FPTR")
_FPTR_C = "FBUF[63] = 0; FPTR = &FBUF[rnd() % 8u];"
_FPTR_R = "FBUF[63] = 0; FPTR = &raw mut FBUF[(rnd() % 8) as usize];"
_MATCHFLAGS_C = (" { int q; for (q = 0; q < 6; q++) { u32 m = rnd(); u32 bb = (m & 1) | (((m >> 1) & 1) << 1); "
                 "*((u8*) &P_VsSceneController[0].state.fighters[q] + 4) = (u8) (bb | ((bb & 1) << 7) | ((bb & 2) << 5)); } }")
_MATCHFLAGS_R = (" for q in 0..6usize { let m = rnd(); let bb = (m & 1) | (((m >> 1) & 1) << 1); "
                 "*(&raw mut g::P_VsSceneController[0].state.fighters[q] as *mut u8).add(4) = (bb | ((bb & 1) << 7) | ((bb & 2) << 5)) as u8; }")
_COINS = ("raw", "((rnd() & 3) == 0 ? (int) (rnd() % 12u) : (int) (rnd() % 120000u))",
          "if rnd() & 3 == 0 { (rnd() % 12) as i32 } else { (rnd() % 120000) as i32 }")
_FPTR = ("raw", "&FPTR", "&raw mut FPTR")
_FPTR_C = "FBUF[63] = 0; FPTR = &FBUF[rnd() % 8u];"
_FPTR_R = "FBUF[63] = 0; FPTR = &raw mut FBUF[(rnd() % 8) as usize];"
_MATCHFLAGS_C = (" { int q; for (q = 0; q < 6; q++) { u32 m = rnd(); u32 bb = (m & 1) | (((m >> 1) & 1) << 1); "
                 "*((u8*) &P_VsSceneController[0].state.fighters[q] + 4) = (u8) (bb | ((bb & 1) << 7) | ((bb & 2) << 5)); } }")
_MATCHFLAGS_R = (" for q in 0..6usize { let m = rnd(); let bb = (m & 1) | (((m >> 1) & 1) << 1); "
                 "*(&raw mut g::P_VsSceneController[0].state.fighters[q] as *mut u8).add(4) = (bb | ((bb & 1) << 7) | ((bb & 2) << 5)) as u8; }")
_COINS = ("raw", "((rnd() & 3) == 0 ? (int) (rnd() % 12u) : (int) (rnd() % 120000u))",
          "if rnd() & 3 == 0 { (rnd() % 12) as i32 } else { (rnd() % 120000) as i32 }")
_FPTR = ("raw", "&FPTR", "&raw mut FPTR")
_FPTR_C = "FBUF[63] = 0; FPTR = &FBUF[rnd() % 8u];"
_FPTR_R = "FBUF[63] = 0; FPTR = &raw mut FBUF[(rnd() % 8) as usize];"
_MATCHFLAGS_C = (" { int q; for (q = 0; q < 6; q++) { u32 m = rnd(); u32 bb = (m & 1) | (((m >> 1) & 1) << 1); "
                 "*((u8*) &P_VsSceneController[0].state.fighters[q] + 4) = (u8) (bb | ((bb & 1) << 7) | ((bb & 2) << 5)); } }")
_MATCHFLAGS_R = (" for q in 0..6usize { let m = rnd(); let bb = (m & 1) | (((m >> 1) & 1) << 1); "
                 "*(&raw mut g::P_VsSceneController[0].state.fighters[q] as *mut u8).add(4) = (bb | ((bb & 1) << 7) | ((bb & 2) << 5)) as u8; }")
_COINS = ("raw", "((rnd() & 3) == 0 ? (int) (rnd() % 12u) : (int) (rnd() % 120000u))",
          "if rnd() & 3 == 0 { (rnd() % 12) as i32 } else { (rnd() % 120000) as i32 }")
_FPTR = ("raw", "&FPTR", "&raw mut FPTR")
_FPTR_C = "FBUF[63] = 0; FPTR = &FBUF[rnd() % 8u];"
_FPTR_R = "FBUF[63] = 0; FPTR = &raw mut FBUF[(rnd() % 8) as usize];"
_MATCHFLAGS_C = (" { int q; for (q = 0; q < 6; q++) { u32 m = rnd(); u32 bb = (m & 1) | (((m >> 1) & 1) << 1); "
                 "*((u8*) &P_VsSceneController[0].state.fighters[q] + 4) = (u8) (bb | ((bb & 1) << 7) | ((bb & 2) << 5)); } }")
_MATCHFLAGS_R = (" for q in 0..6usize { let m = rnd(); let bb = (m & 1) | (((m >> 1) & 1) << 1); "
                 "*(&raw mut g::P_VsSceneController[0].state.fighters[q] as *mut u8).add(4) = (bb | ((bb & 1) << 7) | ((bb & 2) << 5)) as u8; }")
_COINS = ("raw", "((rnd() & 3) == 0 ? (int) (rnd() % 12u) : (int) (rnd() % 120000u))",
          "if rnd() & 3 == 0 { (rnd() % 12) as i32 } else { (rnd() % 120000) as i32 }")
_FPTR = ("raw", "&FPTR", "&raw mut FPTR")
_FPTR_C = "FBUF[63] = 0; FPTR = &FBUF[rnd() % 8u];"
_FPTR_R = "FBUF[63] = 0; FPTR = &raw mut FBUF[(rnd() % 8) as usize];"
_MATCHFLAGS_C = (" { int q; for (q = 0; q < 6; q++) { u32 m = rnd(); u32 bb = (m & 1) | (((m >> 1) & 1) << 1); "
                 "*((u8*) &P_VsSceneController[0].state.fighters[q] + 4) = (u8) (bb | ((bb & 1) << 7) | ((bb & 2) << 5)); } }")
_MATCHFLAGS_R = (" for q in 0..6usize { let m = rnd(); let bb = (m & 1) | (((m >> 1) & 1) << 1); "
                 "*(&raw mut g::P_VsSceneController[0].state.fighters[q] as *mut u8).add(4) = (bb | ((bb & 1) << 7) | ((bb & 2) << 5)) as u8; }")
_COINS = ("raw", "((rnd() & 3) == 0 ? (int) (rnd() % 12u) : (int) (rnd() % 120000u))",
          "if rnd() & 3 == 0 { (rnd() % 12) as i32 } else { (rnd() % 120000) as i32 }")
_FPTR = ("raw", "&FPTR", "&raw mut FPTR")
_FPTR_C = "FBUF[63] = 0; FPTR = &FBUF[rnd() % 8u];"
_FPTR_R = "FBUF[63] = 0; FPTR = &raw mut FBUF[(rnd() % 8) as usize];"
_MATCHFLAGS_C = (" { int q; for (q = 0; q < 6; q++) { u32 m = rnd(); u32 bb = (m & 1) | (((m >> 1) & 1) << 1); "
                 "*((u8*) &P_VsSceneController[0].state.fighters[q] + 4) = (u8) (bb | ((bb & 1) << 7) | ((bb & 2) << 5)); } }")
_MATCHFLAGS_R = (" for q in 0..6usize { let m = rnd(); let bb = (m & 1) | (((m >> 1) & 1) << 1); "
                 "*(&raw mut g::P_VsSceneController[0].state.fighters[q] as *mut u8).add(4) = (bb | ((bb & 1) << 7) | ((bb & 2) << 5)) as u8; }")
_COINS = ("raw", "((rnd() & 3) == 0 ? (int) (rnd() % 12u) : (int) (rnd() % 120000u))",
          "if rnd() & 3 == 0 { (rnd() % 12) as i32 } else { (rnd() % 120000) as i32 }")
_FPTR = ("raw", "&FPTR", "&raw mut FPTR")
_FPTR_C = "FBUF[63] = 0; FPTR = &FBUF[rnd() % 8u];"
_FPTR_R = "FBUF[63] = 0; FPTR = &raw mut FBUF[(rnd() % 8) as usize];"
_MATCHFLAGS_C = (" { int q; for (q = 0; q < 6; q++) { u32 m = rnd(); u32 bb = (m & 1) | (((m >> 1) & 1) << 1); "
                 "*((u8*) &P_VsSceneController[0].state.fighters[q] + 4) = (u8) (bb | ((bb & 1) << 7) | ((bb & 2) << 5)); } }")
_MATCHFLAGS_R = (" for q in 0..6usize { let m = rnd(); let bb = (m & 1) | (((m >> 1) & 1) << 1); "
                 "*(&raw mut g::P_VsSceneController[0].state.fighters[q] as *mut u8).add(4) = (bb | ((bb & 1) << 7) | ((bb & 2) << 5)) as u8; }")
_COINS = ("raw", "((rnd() & 3) == 0 ? (int) (rnd() % 12u) : (int) (rnd() % 120000u))",
          "if rnd() & 3 == 0 { (rnd() % 12) as i32 } else { (rnd() % 120000) as i32 }")
_FPTR = ("raw", "&FPTR", "&raw mut FPTR")
_FPTR_C = "FBUF[63] = 0; FPTR = &FBUF[rnd() % 8u];"
_FPTR_R = "FBUF[63] = 0; FPTR = &raw mut FBUF[(rnd() % 8) as usize];"
_MATCHFLAGS_C = (" { int q; for (q = 0; q < 6; q++) { u32 m = rnd(); u32 bb = (m & 1) | (((m >> 1) & 1) << 1); "
                 "*((u8*) &P_VsSceneController[0].state.fighters[q] + 4) = (u8) (bb | ((bb & 1) << 7) | ((bb & 2) << 5)); } }")
_MATCHFLAGS_R = (" for q in 0..6usize { let m = rnd(); let bb = (m & 1) | (((m >> 1) & 1) << 1); "
                 "*(&raw mut g::P_VsSceneController[0].state.fighters[q] as *mut u8).add(4) = (bb | ((bb & 1) << 7) | ((bb & 2) << 5)) as u8; }")
_COINS = ("raw", "((rnd() & 3) == 0 ? (int) (rnd() % 12u) : (int) (rnd() % 120000u))",
          "if rnd() & 3 == 0 { (rnd() % 12) as i32 } else { (rnd() % 120000) as i32 }")
_FPTR = ("raw", "&FPTR", "&raw mut FPTR")
_FPTR_C = "FBUF[63] = 0; FPTR = &FBUF[rnd() % 8u];"
_FPTR_R = "FBUF[63] = 0; FPTR = &raw mut FBUF[(rnd() % 8) as usize];"
_MATCHFLAGS_C = (" { int q; for (q = 0; q < 6; q++) { u32 m = rnd(); u32 bb = (m & 1) | (((m >> 1) & 1) << 1); "
                 "*((u8*) &P_VsSceneController[0].state.fighters[q] + 4) = (u8) (bb | ((bb & 1) << 7) | ((bb & 2) << 5)); } }")
_MATCHFLAGS_R = (" for q in 0..6usize { let m = rnd(); let bb = (m & 1) | (((m >> 1) & 1) << 1); "
                 "*(&raw mut g::P_VsSceneController[0].state.fighters[q] as *mut u8).add(4) = (bb | ((bb & 1) << 7) | ((bb & 2) << 5)) as u8; }")
_COINS = ("raw", "((rnd() & 3) == 0 ? (int) (rnd() % 12u) : (int) (rnd() % 120000u))",
          "if rnd() & 3 == 0 { (rnd() % 12) as i32 } else { (rnd() % 120000) as i32 }")
_FPTR = ("raw", "&FPTR", "&raw mut FPTR")
_FPTR_C = "FBUF[63] = 0; FPTR = &FBUF[rnd() % 8u];"
_FPTR_R = "FBUF[63] = 0; FPTR = &raw mut FBUF[(rnd() % 8) as usize];"
_MATCHFLAGS_C = (" { int q; for (q = 0; q < 6; q++) { u32 m = rnd(); u32 bb = (m & 1) | (((m >> 1) & 1) << 1); "
                 "*((u8*) &P_VsSceneController[0].state.fighters[q] + 4) = (u8) (bb | ((bb & 1) << 7) | ((bb & 2) << 5)); } }")
_MATCHFLAGS_R = (" for q in 0..6usize { let m = rnd(); let bb = (m & 1) | (((m >> 1) & 1) << 1); "
                 "*(&raw mut g::P_VsSceneController[0].state.fighters[q] as *mut u8).add(4) = (bb | ((bb & 1) << 7) | ((bb & 2) << 5)) as u8; }")
_COINS = ("raw", "((rnd() & 3) == 0 ? (int) (rnd() % 12u) : (int) (rnd() % 120000u))",
          "if rnd() & 3 == 0 { (rnd() % 12) as i32 } else { (rnd() % 120000) as i32 }")
_FPTR = ("raw", "&FPTR", "&raw mut FPTR")
_FPTR_C = "FBUF[63] = 0; FPTR = &FBUF[rnd() % 8u];"
_FPTR_R = "FBUF[63] = 0; FPTR = &raw mut FBUF[(rnd() % 8) as usize];"
_MATCHFLAGS_C = (" { int q; for (q = 0; q < 6; q++) { u32 m = rnd(); u32 bb = (m & 1) | (((m >> 1) & 1) << 1); "
                 "*((u8*) &P_VsSceneController[0].state.fighters[q] + 4) = (u8) (bb | ((bb & 1) << 7) | ((bb & 2) << 5)); } }")
_MATCHFLAGS_R = (" for q in 0..6usize { let m = rnd(); let bb = (m & 1) | (((m >> 1) & 1) << 1); "
                 "*(&raw mut g::P_VsSceneController[0].state.fighters[q] as *mut u8).add(4) = (bb | ((bb & 1) << 7) | ((bb & 2) << 5)) as u8; }")
_COINS = ("raw", "((rnd() & 3) == 0 ? (int) (rnd() % 12u) : (int) (rnd() % 120000u))",
          "if rnd() & 3 == 0 { (rnd() % 12) as i32 } else { (rnd() % 120000) as i32 }")
_FPTR = ("raw", "&FPTR", "&raw mut FPTR")
_FPTR_C = "FBUF[63] = 0; FPTR = &FBUF[rnd() % 8u];"
_FPTR_R = "FBUF[63] = 0; FPTR = &raw mut FBUF[(rnd() % 8) as usize];"
_MATCHFLAGS_C = (" { int q; for (q = 0; q < 6; q++) { u32 m = rnd(); u32 bb = (m & 1) | (((m >> 1) & 1) << 1); "
                 "*((u8*) &P_VsSceneController[0].state.fighters[q] + 4) = (u8) (bb | ((bb & 1) << 7) | ((bb & 2) << 5)); } }")
_MATCHFLAGS_R = (" for q in 0..6usize { let m = rnd(); let bb = (m & 1) | (((m >> 1) & 1) << 1); "
                 "*(&raw mut g::P_VsSceneController[0].state.fighters[q] as *mut u8).add(4) = (bb | ((bb & 1) << 7) | ((bb & 2) << 5)) as u8; }")
_COINS = ("raw", "((rnd() & 3) == 0 ? (int) (rnd() % 12u) : (int) (rnd() % 120000u))",
          "if rnd() & 3 == 0 { (rnd() % 12) as i32 } else { (rnd() % 120000) as i32 }")
_FPTR = ("raw", "&FPTR", "&raw mut FPTR")
_FPTR_C = "FBUF[63] = 0; FPTR = &FBUF[rnd() % 8u];"
_FPTR_R = "FBUF[63] = 0; FPTR = &raw mut FBUF[(rnd() % 8) as usize];"
_MATCHFLAGS_C = (" { int q; for (q = 0; q < 6; q++) { u32 m = rnd(); u32 bb = (m & 1) | (((m >> 1) & 1) << 1); "
                 "*((u8*) &P_VsSceneController[0].state.fighters[q] + 4) = (u8) (bb | ((bb & 1) << 7) | ((bb & 2) << 5)); } }")
_MATCHFLAGS_R = (" for q in 0..6usize { let m = rnd(); let bb = (m & 1) | (((m >> 1) & 1) << 1); "
                 "*(&raw mut g::P_VsSceneController[0].state.fighters[q] as *mut u8).add(4) = (bb | ((bb & 1) << 7) | ((bb & 2) << 5)) as u8; }")
_SC_FILL = ("raw", "(rnd() % 4u == 0 ? (int) (rnd() % 64u) - 3 : (int) (rnd() % 64u) - 1)",
            "if rnd() % 4 == 0 { (rnd() % 64) as i32 - 3 } else { (rnd() % 64) as i32 - 1 }")
_STOCK_US = ("raw", "(struct IfStockUserData*) &ifStock_804A1378.x204[rnd() % 6u]",
             "(&raw mut m::ifStock_804A1378.x204[(rnd() % 6) as usize]) as *mut IfStockUserData")
_STOCK_SLOTS = ("raw", "ifStock_804A1A8C", "(&raw mut m::ifStock_804A1A8C) as *mut *mut HSD_GObj")
_GCOL0 = ("raw", "&GCOL[0]", "&raw mut GCOL[0]")
_FRAME = ("raw", "((rnd() % 3u) == 0 ? 1000000.0f + (float) ((rnd() % 6u) * 256u + rnd() % 4u) : rg())",
          "if rnd() % 3 == 0 { 1000000.0f32 + ((rnd() % 6) * 256 + rnd() % 4) as f32 } else { rg() }")
_PLAYER_X4 = ["ifStock_804A1378_per_player.x4"] + ["ifStock_804A1378_per_player." + n for n in
              ("x24", "x28", "x2C", "x30", "x34", "x38", "x3C", "x40", "x44")]
_PIDT = ("expr", "&PIDT[rnd() % 8u]", "unsafe { &raw mut super::PIDT[(rnd() % 8) as usize] }")

SPECS["ifstock"] = dict(
    src="src/melee/if/ifstock.c",
    extra_main=["src/melee/if/ifstock_geno_art.inc", "pc/geno/geno_art_anim.inc", "pc/geno/geno_art_text.h"],
    # The C reaches ifStock_804A1378.x204 through GameCube-layout address arithmetic ((IfStockData*) stock + n, then
    # ++ of an IfStockDataOffset: +0x204 bytes); natively the x204 member sits elsewhere (8-byte pointers), so the
    # compiled copy spells the same element as &stock->x204[n]. The port does exactly that.
    patch=[("    arg0_base =\n        (struct IfStockDataOffset*) ((struct IfStockData*) stock + arg0);\n"
            "    arg1_base =\n        (struct IfStockDataOffset*) ((struct IfStockData*) stock + arg1);\n"
            "    arg0_data = (struct IfStockData*) ++arg0_base;\n    arg1_data = (struct IfStockData*) ++arg1_base;\n",
            "    arg0_base = (struct IfStockDataOffset*) &stock->x204[arg0];\n"
            "    arg1_base = (struct IfStockDataOffset*) &stock->x204[arg1];\n"
            "    arg0_data = (struct IfStockData*) arg0_base;\n    arg1_data = (struct IfStockData*) arg1_base;\n"),
           ("((struct IfStockDataOffset*) ((struct ifStock_804A1378_x204*) stock + (p)))",
            "((struct IfStockDataOffset*) &stock->x204[p])"),
           ("(((e) = ifStock_802F8298_elem(p)) + 1))", "((e) = ifStock_802F8298_elem(p)))"),
           ("(struct ifStock_804A1378_x204*) ++elem)", "(struct ifStock_804A1378_x204*) elem)")],
    pools={"HSD_GObj": 6, "HSD_JObj": 10, "HSD_DObj": 4, "HSD_MObj": 4, "HSD_TObj": 4, "HSD_Material": 3,
           "HSD_AObj": 3, "HSD_FObj": 4, "HSD_ImageDesc": 4, "DynamicModelDesc": 2, "HSD_Joint": 2,
           "VsSceneController": 1, "HSD_Archive": 1, "HudIndex": 1, "HSD_MatAnimJoint": 1},
    rules={**CHAIN_RULES,
           "HSD_TObj.imagedesc": ("ptr", "HSD_ImageDesc", True), "HSD_TObj.imagetbl": _PIDT,
           "HSD_AObj.fobj": ("ptr", "HSD_FObj", True),
           "HSD_FObj.next": ("expr", "(p < &P_HSD_FObj[3] && (rnd() & 1)) ? (void*) (p + 1) : (void*) 0",
                             "unsafe { if (p as *mut HSD_FObj) < (&raw mut P_HSD_FObj[3]) && rnd() & 1 != 0 { (p as *mut HSD_FObj).add(1) } else { null_mut() } }"),
           "HSD_FObj.ad_head": ("expr", "&FDAT[rnd() % 512u]", "unsafe { &raw mut super::FDAT[(rnd() % 512) as usize] }"),
           "HSD_FObj.length": ("range", 0, 40), "HSD_FObj.obj_type": ("range", 0, 2),
           "DynamicModelDesc.joint": ("ptr", "HSD_Joint", False),
           "ifStock_804A1378.x0": ("expr", "(void*) PMDL",
                                   "&raw mut super::PMDL as *mut _"),
           "ifStock_804A1378.x4": ("ptr", "DynamicModelDesc", False),
           # pad1F0 is the pointer slot between jobj and jobj_a: lb_80011E24(..., 0, 1, ..., 5) stores a pointer there
           "ifStock_804A1378.pad1F0": ("skip",),
           "ifStock_804A1378.gobj": ("ptr", "HSD_GObj", True), "ifStock_804A1378.jobj": ("ptr", "HSD_JObj", False),
           "ifStock_804A1378.jobj_a": ("ptr", "HSD_JObj", False), "ifStock_804A1378.jobj_b": ("ptr", "HSD_JObj", False),
           "ifStock_804A1378.jobj_c": ("ptr", "HSD_JObj", False), "ifStock_804A1378.jobj_d": ("ptr", "HSD_JObj", False),
           "ifStock_804A1378_per_player.x0": ("ptr", "HSD_GObj", True),
           **{k: ("ptr", "HSD_JObj", False) for k in _PLAYER_X4},
           "ifStock_804A1378_x204.player": ("range", 0, 5), "ifStock_804A1378_x204.mode": ("range", 0, 3),
           "ifStock_804A1378_x204.flag": ("range", 0, 2), "ifStock_804A1378_x204.x3": ("range", 0, 5),
           "ifStock_804A1378_x204.anim": ("range", 0, 12),
           "ifStock_804A1774.x0": ("range", 0, 1), "ifStock_804A1774.x10C": ("ptr", "HSD_GObj", True),
           "ifStock_804A1ACC.x0": ("range", 0, 1), "ifStock_804A1ACC.x10C": ("ptr", "HSD_GObj", True),
           "ifStock_804A1ACC.x108": ("ptr", "HSD_GObj", True),
           "IfStockGenoArt.ck": ("range", 0, 3), "IfStockGenoArt.costume": ("range", 0, 3),
           "IfStockGenoArt.tried": ("range", 0, 1), "IfStockGenoArt.desc": ("ptr", "HSD_ImageDesc", True),
           "IfStockGenoBinding.tobj": ("ptr", "HSD_TObj", True), "IfStockGenoBinding.original_image": ("ptr", "HSD_ImageDesc", True),
           "IfStockGenoBinding.original": _PIDT, "IfStockGenoBinding.table": _PIDT,
           "IfStockGenoBinding.count": ("range", 0, 7)},
    statics=[dict(c="ifStock_804A1378", ty="ifStock_804A1378"),
             dict(c="ifStock_804A1774", ty="ifStock_804A1774"),
             dict(c="ifStock_804A1A8C", n=16, kind="ptr", fill="pn:HSD_GObj"),
             dict(c="ifStock_804A1ACC", ty="ifStock_804A1ACC"),
             dict(c="ifStock_GenoArt", ty="IfStockGenoArt", n=12),
             dict(c="ifStock_GenoBindings", ty="IfStockGenoBinding", n=256),
             dict(c="PMDL", rs="super::PMDL", n=2, kind="ptr", fill="p:DynamicModelDesc", region=True),
             dict(c="SCA", rs="super::SCA", n=140, kind="int", fill=_SC_FILL, region=True),
             dict(c="SCB", rs="super::SCB", n=140, kind="int", fill=_SC_FILL, region=True),
             dict(c="SCC", rs="super::SCC", n=140, kind="int", fill=_SC_FILL, region=True),
             dict(c="SCD", rs="super::SCD", n=140, kind="int", fill=_SC_FILL, region=True),
             dict(c="HEAP", rs="super::HEAP.0", n=4096, zero=True, region=True),
             dict(c="HOFF", rs="super::HOFF", kind="int", fill="k:0"),
             dict(c="NULLJ", rs="super::NULLJ", kind="int", fill="k:0"),
             dict(c="GCOL", rs="super::GCOL", ty="GXColor", n=2, region=True),
             dict(c="V3A", rs="super::V3A", ty="Vec3", region=True),
             dict(c="ARCP", rs="super::ARCP", kind="ptr", fill="k:0", region=True),
             dict(c="PIDT", rs="super::PIDT", n=16, kind="ptr", fill="pn:HSD_ImageDesc", region=True),
             dict(c="FDAT", rs="super::FDAT", n=1024, kind="int", fill=("raw", "(int) (rnd() & 0x7F)", "(rnd() & 0x7F) as i32"), region=True),
             dict(c="FBUF", rs="super::FBUF", n=64, kind="int", fill="v", region=True),
             dict(c="FPTR", rs="super::FPTR", kind="ptr", fill="k:0", region=True),
             dict(c="MAXI", rs="super::MAXI", kind="int", fill="v", region=True),
             dict(c="OUTB", rs="super::OUTB", n=4, kind="int", fill="v", region=True),
             dict(c="STK", rs="super::STK", n=8, kind="int", fill=("raw", "((rnd() & 3) == 0 ? (int) fv() : (int) (rnd() % 12u) - 1)",
                                                                  "if rnd() & 3 == 0 { fv() as i32 } else { (rnd() % 12) as i32 - 1 }")),
             dict(c="COIN", rs="super::COIN", n=8, kind="int", fill=("raw", "((rnd() & 3) == 0 ? (int) fv() : (int) (rnd() % 120000u))",
                                                                    "if rnd() & 3 == 0 { fv() as i32 } else { (rnd() % 120000) as i32 }")),
             dict(c="HSD_GObj_JObjKind", ext=True, kind="int", fill="k:20")],
    skip_data=["result"],
    c_decl="void* memset(void* d, int c, unsigned long n) { unsigned char* p = (unsigned char*) d; while (n--) *p++ = (unsigned char) c; return d; }\nstatic GXColor GCOL[2];\nstatic Vec3 V3A;\nstatic HSD_Archive* ARCP;\nstatic DynamicModelDesc* PMDL[2];\n"
           "static signed char SCA[140], SCB[140], SCC[140], SCD[140];\n"
           "static u8 HEAP[4096] __attribute__((aligned(64)));\nstatic int HOFF;\nstatic int NULLJ;\nstatic int STK[8], COIN[8];\n"
           "static HSD_ImageDesc* PIDT[16];\nstatic u8 FDAT[1024];\nstatic u8 FBUF[64];\nstatic u8* FPTR;\nstatic int MAXI;\nstatic char OUTB[4];\n"
           "static void* halloc(int n) { int sz; void* r; if (n < 0) return 0; sz = (n + 7) & ~7; if (HOFF + sz > 4096) return 0; "
           "r = &HEAP[HOFF]; HOFF += sz; return r; }\n"
           "static const char* const NM[8] = { \"Captain Falcon\", \"mr game & watch\", \"Ness\", \"  --  \", \"7up\", \"a\", \"\", \"Dr. Mario 64\" };\n",
    rs_decl="pub static mut GCOL: [_GXColor; 2] = unsafe { core::mem::zeroed() };\npub static mut V3A: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };\n"
            "pub static mut ARCP: *mut HSD_Archive = null_mut();\npub static mut PMDL: [*mut DynamicModelDesc; 2] = [null_mut(); 2];\n"
            "pub static mut SCA: [i8; 140] = [0; 140];\npub static mut SCB: [i8; 140] = [0; 140];\n"
            "pub static mut SCC: [i8; 140] = [0; 140];\npub static mut SCD: [i8; 140] = [0; 140];\n"
            "#[repr(C, align(64))]\npub struct Heap(pub [u8; 4096]);\npub static mut HEAP: Heap = Heap([0; 4096]);\n"
            "pub static mut HOFF: i32 = 0;\npub static mut NULLJ: i32 = 0;\npub static mut STK: [i32; 8] = [0; 8];\npub static mut COIN: [i32; 8] = [0; 8];\n"
            "pub static mut PIDT: [*mut HSD_ImageDesc; 16] = [null_mut(); 16];\npub static mut FDAT: [u8; 1024] = [0; 1024];\n"
            "pub static mut FBUF: [u8; 64] = [0; 64];\npub static mut FPTR: *mut u8 = null_mut();\npub static mut MAXI: i32 = 0;\n"
            "pub static mut OUTB: [i8; 4] = [0; 4];\n"
            "fn halloc(n: i32) -> *mut c_void { unsafe { if n < 0 { return null_mut(); } let sz = n.wrapping_add(7) & !7; "
            "if HOFF.wrapping_add(sz) > 4096 { return null_mut(); } let r = &raw mut HEAP.0[HOFF as usize]; HOFF += sz; r as *mut c_void } }\n"
            "pub static NM: [&[u8]; 8] = [b\"Captain Falcon\\0\", b\"mr game & watch\\0\", b\"Ness\\0\", b\"  --  \\0\", b\"7up\\0\", b\"a\\0\", b\"\\0\", b\"Dr. Mario 64\\0\"];\n",
    hash_extra=(
        "{ int q, z; for (q = 0; q < 12; q++) { HSD_ImageDesc* d = ifStock_GenoArt[q].desc; if (d) { HM(PI(d->image_ptr)); HM(d->width); HM(d->height); "
        "HM((u32) d->format); HM(d->mipmap); HM(B(d->minLOD)); HM(B(d->maxLOD)); } } "
        "for (q = 0; q < 256; q++) { struct IfStockGenoBinding* b = &ifStock_GenoBindings[q]; if (b->table && b->count > 0 && b->count < 8) "
        "for (z = 0; z <= b->count; z++) HM(PI(b->table[z])); } }",
        "unsafe { for q in 0..12usize { let d = m::ifStock_GenoArt[q].desc; if !d.is_null() { hm(pi((*d).image_ptr)); hm((*d).width as u32); hm((*d).height as u32); "
        "hm((*d).format as u32); hm((*d).mipmap); hm((*d).minLOD.to_bits()); hm((*d).maxLOD.to_bits()); } } "
        "for q in 0..256usize { let b = &m::ifStock_GenoBindings[q]; if !b.table.is_null() && b.count > 0 && b.count < 8 { "
        "for z in 0..=(b.count as usize) { hm(pi(*b.table.add(z))); } } } }"),
    skip_stubs=["memset"],
    setup_c="int k;\n    for (k = 0; k < 6; k++) P_HSD_GObj[k].user_data = &ifStock_804A1378.x204[rnd() % 6u];",
    setup_rs="for k in 0..6usize { g::P_HSD_GObj[k].user_data = &raw mut m::ifStock_804A1378.x204[(rnd() % 6) as usize] as *mut _; }",
    setup_c="int k;\n    if (HDBG) printf(\"PAD %02X %02X %02X %02X rs=%08X\\n\", (u8)ifStock_804A1378.pad1F0[0], (u8)ifStock_804A1378.pad1F0[1], (u8)ifStock_804A1378.pad1F0[2], (u8)ifStock_804A1378.pad1F0[3], rs);\n    for (k = 0; k < 6; k++) P_HSD_GObj[k].user_data = &ifStock_804A1378.x204[rnd() % 6u];",
    setup_rs="if HDBG { let pd = m::ifStock_804A1378.pad1F0; println!(\"PAD {:02X} {:02X} {:02X} {:02X} rs={:08X}\", pd[0] as u8, pd[1] as u8, pd[2] as u8, pd[3] as u8, RS); } for k in 0..6usize { g::P_HSD_GObj[k].user_data = &raw mut m::ifStock_804A1378.x204[(rnd() % 6) as usize] as *mut _; }",
    ret={"GObj_Create": ("pick_HSD_GObj(0)", "pick_HSD_GObj(0)"),
         "HSD_JObjLoadJoint": ("((NULLJ && (fv() & 1)) ? 0 : pick_HSD_JObj(0))",
                               "if super::NULLJ != 0 && fv() & 1 != 0 { null_mut() } else { pick_HSD_JObj(0) }"),
         "HSD_MemAlloc": ("halloc(a0)", "super::halloc(a0)"),
         "gmVs_GetSceneState": ("&P_VsSceneController[0].state", "&raw mut P_VsSceneController[0].state"),
         "gm_80169520": ("SCA", "&raw mut super::SCA"), "gm_80169530": ("SCB", "&raw mut super::SCB"),
         "gm_8016A97C": ("SCC", "&raw mut super::SCC"),
         "gm_8016A98C": ("(void*) SCD", "&raw mut super::SCD as *mut lbl_8046B668_t"),
         "gm_MexStockMatAnim": ("((fv() & 1) ? &P_HSD_MatAnimJoint[0] : 0)",
                                "if fv() & 1 != 0 { &raw mut P_HSD_MatAnimJoint[0] } else { null_mut() }"),
         "ifAll_GetArchive": ("&ARCP", "&raw mut super::ARCP"), "ifAll_GetPlayerHUDPosition": ("&V3A", "&raw mut super::V3A"),
         "ifStatus_GetHUDInfo": ("&P_HudIndex[0]", "&raw mut P_HudIndex[0]"),
         "lbVector_8000DE38": ("a1", "a1"),
         "Player_GetCoins": ("COIN[a0 & 7]", "super::COIN[(a0 & 7) as usize]"),
         "Player_GetStocks": ("STK[a0 & 7]", "super::STK[(a0 & 7) as usize]"),
         "gm_801694A0": ("((fv() & 1) ? (int) (fv() % 11000u) - 500 : (int) fv())",
                         "if fv() & 1 != 0 { (fv() % 11000) as i32 - 500 } else { fv() as i32 }"),
         "gm_8016B184": ("(fv() & 1)", "(fv() & 1) as _"), "gm_8016A1F8": ("(fv() & 1)", "(fv() & 1) as _"),
         "gm_8016A944": ("(fv() & 1)", "(fv() & 1) as _"), "gm_8016B1A8": ("(fv() & 1)", "(fv() & 1) as _"),
         "gm_IsMultimanSmashMode": ("(fv() & 1)", "(fv() & 1) as _"), "gm_80169394": ("(fv() & 1)", "(fv() & 1) as _"),
         "Script_StatusHUDVisible": ("((fv() & 3) != 0)", "((fv() & 3) != 0) as i32"),
         "Ui_RetailHidden": ("((fv() & 3) == 0)", "((fv() & 3) == 0) as i32"),
         "gm_GenoStockFrame": ("((fv() % 3u) == 0 ? 1000000.0f + (float) (fv() % 1536u) : rg())",
                               "if fv() % 3 == 0 { 1000000.0f32 + (fv() % 1536) as f32 } else { rg() }"),
         "Geno_DefineBaseCK": ("((int) (fv() % 5u) - 1)", "(fv() % 5) as i32 - 1"),
         "Geno_DefineArtBaseCK": ("((int) (fv() % 5u) - 1)", "(fv() % 5) as i32 - 1"),
         "Geno_DefineArtOpen": ("((fv() & 3) == 0 ? -1 : (int) (fv() % 3u))", "if fv() & 3 == 0 { -1 } else { (fv() % 3) as i32 }"),
         "Geno_DefineName": ("((fv() & 3) != 0)", "((fv() & 3) != 0) as i32"),
         "GxTex_Width": ("((fv() & 7) == 0 ? (int) (fv() % 2100u) - 20 : (int) (fv() % 24u) + 1)",
                         "if fv() & 7 == 0 { (fv() % 2100) as i32 - 20 } else { (fv() % 24) as i32 + 1 }"),
         "GxTex_Height": ("((fv() & 7) == 0 ? (int) (fv() % 2100u) - 20 : (int) (fv() % 24u) + 1)",
                          "if fv() & 7 == 0 { (fv() % 2100) as i32 - 20 } else { (fv() % 24) as i32 + 1 }"),
         "GxTex_Format": ("((fv() & 7) == 0 ? (int) (fv() % 12u) - 2 : (int) (fv() % 8u))",
                          "if fv() & 7 == 0 { (fv() % 12) as i32 - 2 } else { (fv() % 8) as i32 }"),
         "GxTex_ImageSize": ("((fv() & 7) == 0 ? (int) fv() - 100 : (int) (fv() % 1500u) + 1)",
                             "if fv() & 7 == 0 { (fv() as i32).wrapping_sub(100) } else { (fv() % 1500) as i32 + 1 }")},
    effects={**MEMZERO,
             "Ground_801C1DE4": ("*a0 = (int) (fv() % 20u); *a1 = (int) (fv() % 20u);",
                                 "*a0 = (fv() % 20) as i32; *a1 = (fv() % 20) as i32;"),
             "lbVector_8000DE38": ("a1->x = rw(); a1->y = rw(); a1->z = rw();", "(*a1).x = rw(); (*a1).y = rw(); (*a1).z = rw();"),
             "Geno_DefineName": ("{ const char* s = NM[fv() % 8u]; int k; for (k = 0; s[k] && k < a2 - 1; k++) a1[k] = s[k]; a1[k] = 0; }",
                                 "{ let s = super::NM[(fv() % 8) as usize]; let mut k = 0usize; while s[k] != 0 && (k as i32) < a2 - 1 { *a1.add(k) = s[k] as i8; k += 1; } *a1.add(k) = 0; }")},
    deref={"lbVector_8000DE38": [1]},
    null_ret_ok=["HSD_GObj_SetupProc", "HSD_JObjAddAnimAll"],
    custom_stubs={"lbArchive_LoadSections": sections_stub(
        "lbArchive_LoadSections", ("HSD_Archive*", "T(PI(a0));", "*mut HSD_Archive", "t(pi(a0));"),
        {"Stc_scemdls": ("PMDL", "&raw mut super::PMDL")}), "lb_80011E24": jobjs_stub(),
        "efSync_Spawn": dict(
            c="void* efSync_Spawn(s32 a0, HSD_GObj* a1, ...) { __builtin_va_list ap; Vec3* v; __builtin_va_start(ap, a1); "
              "v = __builtin_va_arg(ap, Vec3*); __builtin_va_end(ap); T(@ID@); T((u32) a0); T(PI(a1)); T(RH(v, sizeof *v)); return 0; }",
            ext="fn efSync_Spawn(a0: i32, a1: *mut HSD_GObj, a2: *mut Vec3) -> *mut c_void;",
            hook="|a0, a1, a2| { t(@ID@); t(a0 as u32); t(pi(a1)); t(rh(unsafe { &*a2 })); core::ptr::null_mut() }")},
    ncases={"ifStock_ReqAnim": 1500, "ifStock_GenoFind": 800, "genoArt_ImageKeyAt": 800, "ifStock_GenoBinding": 600,
            "ifStock_802F98E8": 800, "fn_802FA6C4": 300, "fn_802FAC34": 300},
    ops=[dict(name="ifStock_802F7EFC", args=["i:6", "i:6"]),
         dict(name="ifStock_802F8298_tobj_frame", args=["i:6"]),
         dict(name="ifStock_802F8298_update_stocks", args=[_STOCK_US]),
         dict(name="ifStock_802F8298_step_anim", args=[("raw", "&ifStock_804A1378.x204[rnd() % 6u].anim[rnd() % 7u]",
                                                        "&raw mut m::ifStock_804A1378.x204[(rnd() % 6) as usize].anim[(rnd() % 7) as usize]")]),
         "ifStock_802F8298",
         "ifStock_802F89F8_get_data",
         dict(name="ifStock_802F89F8_count", args=[_COINS]),
         dict(name="ifStock_802F89F8_divisor", args=["r:-2:12"]),
         dict(name="ifStock_802F89F8_digit", args=[_COINS, "r:-1:7"]),
         "ifStock_802F89F8", "fn_802F8DB0", "fn_802F8E08",
         dict(name="fn_802F916C", pre_c="{ int q; for (q = 0; q < 16; q++) if (!ifStock_804A1A8C[q]) ifStock_804A1A8C[q] = &P_HSD_GObj[0]; }",
              pre_rs="for q in 0..16usize { if m::ifStock_804A1A8C[q].is_null() { m::ifStock_804A1A8C[q] = &raw mut g::P_HSD_GObj[0]; } }"),
         dict(name="fn_802F9410_inline", args=["p:HSD_GObj", _STOCK_US]),
         "fn_802F9410", "fn_802F94E0", "fn_802F9548", "fn_802F9598", "fn_802F95E8", "fn_802F9680",
         dict(name="ifStock_MexAttach", args=["pn:HSD_JObj"]),
         dict(name="ifStock_MexFrame", args=["v", "v"]),
         dict(name="ifStock_802F96D0", args=["v", "v", "f", "f"]),
         dict(name="ifStock_802F98E8_get_match_info",
              args=[("raw", "&P_VsSceneController[0].state", "&raw mut g::P_VsSceneController[0].state"), "i:6"]),
         dict(name="ifStock_802F98E8", args=["i:6", "r:0:3"], pre_c="NULLJ = 1; if ((rnd() & 7) == 0) ifStock_804A1378.x0 = 0;" + _MATCHFLAGS_C, pre_rs="NULLJ = 1; if rnd() & 7 == 0 { m::ifStock_804A1378.x0 = null_mut(); }" + _MATCHFLAGS_R),
         dict(name="ifStock_802F9F48_inline", args=["r:0:129"]),
         dict(name="ifStock_802F9F48", args=["r:0:129"]),
         dict(name="ifStock_802FA118", args=["r:0:3"]),
         dict(name="ifStock_802FA2D0", args=["r:-500:11500"]),
         dict(name="ifStock_802FA5BC", args=["r:0:1"]),
         "fn_802FA6C4", "fn_802FA8C0", "fn_802FAC34",
         dict(name="ifStock_802FA5BC_noinline", args=["r:0:1"]),
         dict(name="ifStock_ClearGObjSlots", args=[_STOCK_SLOTS, "r:0:16"]),
         dict(name="ifStock_CreateStockGObjs", args=[_STOCK_SLOTS]),
         "ifStock_802FAEC4", "ifStock_802FB390", "ifStock_802FB41C", "ifStock_802FB484",
         dict(name="copyColor", args=["p:HSD_MObj", _GCOL0]),
         dict(name="ifStock_802FB4EC", args=["i:6", _GCOL0]),
         dict(name="ifStock_802FB650", args=["i:6"]), dict(name="ifStock_802FB6AC", args=["i:6"]),
         "ifStock_GenoReset",
         dict(name="ifStock_GenoFind", args=["r:0:3", "r:0:3", "pn:HSD_ImageDesc"]),
         dict(name="ifStock_GenoBinding", args=["p:HSD_TObj", "i:2"]),
         dict(name="ifStock_ReqAnim", args=["pn:HSD_TObj", _FRAME]),
         dict(name="genoArt_FObjValue", args=[_FPTR, "v"], pre_c=_FPTR_C, pre_rs=_FPTR_R),
         dict(name="genoArt_FObjPacked", args=[_FPTR], pre_c=_FPTR_C, pre_rs=_FPTR_R),
         dict(name="genoArt_ImageKeyAt", args=["pn:HSD_TObj", "r:-1:6", ("raw", "&MAXI", "&raw mut MAXI")]),
         dict(name="genoArt_Initials", c="{ const char* a0 = NM[rnd() % 8u]; genoArt_Initials(a0, OUTB); }",
              rs="let a0 = NM[(rnd() % 8) as usize].as_ptr() as *const i8; m::genoArt_Initials(a0, (&raw mut OUTB) as *mut i8);")],
)


for _s in SPECS.values():
    _s["common_ret"] = COMMON_RET
