"""Per-TU oracle specs for gen_tu.py / run_tu.py (see gen_tu.py).

Keys: src, types (state records to fill / hash), ret {callee: (C expr, Rust expr)} (a stub's
return value, in terms of its args a0..), effects {callee: (C stmts, Rust stmts)} (writes
through out-parameters, run before the return), null_ret_ok, data_size {extern array: n},
patch [(old, new)] (literal edits of the compiled copy of the TU, each must match once)."""
# Return values of callees that the C TUs share (a stub returns what the real function would).
COMMON_RET = {
    "gm_GetGameModeStateEnterData": ("a0->info.enter_data", "(*a0).info.enter_data"),
    "gm_GetGameModeStateExitData": ("a0->info.exit_data", "(*a0).info.exit_data"),
    "HSD_Randi": ("(a0 > 0 ? (int) (rnd() % (u32) a0) : 0)", "if a0 > 0 { (rnd() % a0 as u32) as i32 } else { 0 }"),
}

SPECS = {
    "gmhanyucss": dict(
        src="src/melee/gm/gmhanyucss.c", types=["CSSData", "VsModeData", "GameModeState"],
        ret={"gmVsMelee_GetVsData": ("&VSD", "&raw mut super::VSD")}),
    "gmgover": dict(src="src/melee/gm/gmgover.c", types=["GameModeState"]),
    "gmgiant": dict(src="src/melee/gm/gmgiant.c", types=["GameModeState", "PlayerInitData"]),
    "gmfixedcamera": dict(src="src/melee/gm/gmfixedcamera.c", types=["GameModeState", "StartMeleeData"]),
    "gmcameramode": dict(
        src="src/melee/gm/gmcameramode.c",
        types=["GameModeState", "CSSData", "SSSData", "StartMeleeData", "VsModeData", "PreloadedGameModeState"],
        null_ret_ok=["lbDvd_80017740"],
        ret={"gmVsMelee_GetKOCounts": ("KOC", "&raw mut super::KOC as *mut u8"),
             "lbDvd_GetPreloadCacheScene": ("&PCS", "&raw mut super::PCS"),
             "lbDvd_GetPreloadedArchive": ("(void*) (AR + (a0 & 15))",
                                           "(&raw mut super::AR as *mut u8).wrapping_add((a0 & 15) as usize) as *mut c_void")}),
    "gmdebugmode": dict(
        src="src/melee/gm/gmdebugmode.c",
        types=["GameModeState", "ResultsMatchInfo", "StartMeleeData", "MatchExitInfo", "DebugGameOverData",
               "DebugResultsData", "DebugMemcardData", "enterMenu0_Data", "exitIntroData", "un_80301C64_t",
               "HSD_PadStatus"],
        data_size={"un_803FA258": 128}),
    "gmclassic": dict(
        src="src/melee/gm/gmclassic.c",
        # On the console gmClassicIntroDataBuffer and gm_804908A0 are adjacent (intro, then state:
        # gmClassicRuntimeData); the code reaches the state through the intro pointer. A native
        # link would place them anywhere, so the oracle makes them the one object they are.
        patch=[("u8 gm_804908A0[112];\n", ""),
               ("gmClassicIntroData gmClassicIntroDataBuffer;\n",
                "gmClassicRuntimeData gmClassicRuntimeBuf;\n"
                "#define gmClassicIntroDataBuffer (gmClassicRuntimeBuf.intro)\n"
                "#define gm_804908A0 (gmClassicRuntimeBuf.state.bytes)\n")],
        types=["GameModeState", "UnkAllstarData", "gmm_x0_528_t", "PreloadedGameModeState", "CSSData",
               "StartMeleeData", "MatchExitInfo", "DebugGameOverData", "gm_803DDEC8Struct", "gmClassicMatchup",
               "gmClassicIntroData", "gmClassicRuntimeData", "gmClassic_803DDEC8Data", "gmm_retval_ED98",
               "gmm_retval_EDB0", "gmm_retval_EDBC"],
        data_size={},
        ret={
            "gmMainLib_8015CDC8": ("&G528", "&raw mut super::G528"),
            "gmMainLib_8015D438": ("&TIMEV", "&raw mut super::TIMEV"),
            "gmMainLib_8015D450": ("&BESTV", "&raw mut super::BESTV"),
            "gmMainLib_8015ED98": ("&R98", "&raw mut super::R98"),
            "gmMainLib_8015EDB0": ("&RB0", "&raw mut super::RB0"),
            "gmMainLib_8015EDBC": ("&RBC", "&raw mut super::RBC"),
            "gm_GetAllStarData": ("&AD", "&raw mut super::AD"),
            "lbDvd_GetPreloadCacheScene": ("&PCS", "&raw mut super::PCS"),
            # step number of a scene id (1..10) and a select index (< 25); unlock checks follow UNL
            "gm_8017BE84": ("(1 + fv() % 10)", "1 + (fv() % 10) as i32"),
            "gm_CKindToSelKind": ("(a0 % 25)", "a0 % 25"),
            "Stage_8022519C": ("(a0 % 5)", "a0 % 5"),
            "gm_80164430": ("(UNL ? 1 : (int) fv())", "if super::UNL { 1 } else { fv() as i32 }"),
            "gm_IsCKindUnlocked": ("(UNL ? 1 : (int) fv())", "if super::UNL { 1 } else { fv() as i32 }"),
        },
        effects={
            "Ground_801C1DE4": ("*a0 = (fv() & 1) ? 0 : (int) (fv() % 500); *a1 = (int) (fv() % 700);",
                                "*a0 = if fv() & 1 != 0 { 0 } else { (fv() % 500) as i32 }; *a1 = (fv() % 700) as i32;"),
            "gm_801B0730": ("if (a1) *a1 = (s8) fv(); if (a2) *a2 = (u8) fv(); if (a3) *a3 = (u8) fv(); "
                            "if (a4) *a4 = (u8) fv(); if (a5) *a5 = (u8) fv();",
                            "if !a1.is_null() { *a1 = fv() as i8 } if !a2.is_null() { *a2 = fv() as u8 } "
                            "if !a3.is_null() { *a3 = fv() as u8 } if !a4.is_null() { *a4 = fv() as u8 } "
                            "if !a5.is_null() { *a5 = fv() as u8 }"),
        }),
}
SPECS["gmevent"] = dict(
    src="src/melee/gm/gmevent.c",
    types=["GameModeState", "EventData", "PlayerInitData", "StartMeleeData", "CSSData", "PreloadedGameModeState",
           "MatchExitInfo", "VsSceneController", "MatchEnd", "gm_801BAB40_src", "gm_evinit", "gm_evbonus",
           "gm_evstage_table", "gm_804D6900_t", "gm_804D6900_x4_t", "StartMeleeRules"],
    null_ret_ok=["HSD_GObj_SetupProc"],
    custom_stubs={
        # lbArchive_LoadSymbols(file, &dst, "symbol", 0): variadic; the Rust ext is fixed-arity
        "lbArchive_LoadSymbols": dict(
            c="HSD_Archive* lbArchive_LoadSymbols(const char* a0, void* a1, ...) { __builtin_va_list ap; "
              "const char* sym; int z; __builtin_va_start(ap, a1); sym = __builtin_va_arg(ap, const char*); "
              "z = __builtin_va_arg(ap, int); __builtin_va_end(ap); T(@ID@); T(SH(a0)); T(PI(a1)); T(SH(sym)); "
              "T((u32) z); return 0; }",
            ext="fn lbArchive_LoadSymbols(a0: *const u8, a1: *mut c_void, a2: *const u8, a3: i32) -> *mut c_void;",
            hook="|a0, a1, a2, a3| { t(@ID@); t(sh(a0)); t(pi(a1)); t(sh(a2)); t(a3 as u32); core::ptr::null_mut() }"),
    },
    ret={
        "GObj_Create": ("&GNEW", "&raw mut super::GNEW"),
        "Player_GetEntity": ("((fv() & 1) ? &GOA[a0 & 3] : 0)",
                             "if fv() & 1 != 0 { &raw mut super::GOA[(a0 & 3) as usize] } else { core::ptr::null_mut() }"),
        "Player_GetEntityAtIndex": ("((fv() & 1) ? &GOA[a0 & 3] : 0)",
                                    "if fv() & 1 != 0 { &raw mut super::GOA[(a0 & 3) as usize] } else { core::ptr::null_mut() }"),
        "gmVs_GetSceneController": ("&VSC", "&raw mut super::VSC"),
        "gmVs_GetSceneState": ("&VSS", "&raw mut super::VSS"),
        "gm_8016B774": ("&MEND", "&raw mut super::MEND"),
        "lbDvd_GetPreloadCacheScene": ("&PCS", "&raw mut super::PCS"),
        "gm_8016AEEC": ("((fv() & 1) ? 0 : fv())", "if fv() & 1 != 0 { 0 } else { fv() }"),
        "gm_8016AEFC": ("((fv() & 1) ? 0x3B : fv() % 70)", "if fv() & 1 != 0 { 0x3B } else { (fv() % 70) as u16 }"),
        "Player_GetStocks": ("((int) (fv() % 5) - 1)", "(fv() % 5) as i32 - 1"),
        "Player_GetP1Stock": ("((int) (fv() % 5) - 1)", "(fv() % 5) as i32 - 1"),
        "Player_GetRemainingHP": ("((int) (fv() % 5) - 1)", "(fv() % 5) as i32 - 1"),
        "Player_GetPlayerSlotType": ("(fv() % 4)", "fv() % 4"),
        "ftLib_GetKind": ("(fv() % 9)", "fv() % 9"),
        "Player_80036394": ("(fv() % 9)", "fv() % 9"),
        "ftBossLib_8015C74C": ("((int) (fv() % 40))", "(fv() % 40) as i32"),
        "ftBossLib_8015C7EC": ("((int) (fv() % 40))", "(fv() % 40) as i32"),
        "ftBossLib_8015C92C": ("((int) (fv() % 40))", "(fv() % 40) as i32"),
    },
    effects={
        "gm_8016EDDC": ("hash_PlayerInitData(a1);", "hash_PlayerInitData(&*a1);"),
        "itEvYoshiEgg_Spawn": ("T(B(a0->x)); T(B(a0->y)); T(B(a0->z));",
                               "t((*a0).x.to_bits()); t((*a0).y.to_bits()); t((*a0).z.to_bits());"),
        "gm_80166378": ("hash_MatchEnd(a0); a0->n_team_winners = (u8) (fv() % 3); a0->team_winners[0] = (u8) (fv() % 6);",
                        "hash_MatchEnd(&*a0); (*a0).n_team_winners = (fv() % 3) as u8; (*a0).team_winners[0] = (fv() % 6) as u8;"),
        "gm_801B0730": ("if (a1) *a1 = (s8) fv(); if (a2) *a2 = (u8) fv(); if (a3) *a3 = (u8) fv(); "
                        "if (a4) *a4 = (u8) fv(); if (a5) *a5 = (u8) fv();",
                        "if !a1.is_null() { *a1 = fv() as i8 } if !a2.is_null() { *a2 = fv() as u8 } "
                        "if !a3.is_null() { *a3 = fv() as u8 } if !a4.is_null() { *a4 = fv() as u8 } "
                        "if !a5.is_null() { *a5 = fv() as u8 }"),
    },
    # (s32) of a function address: the trace names the function by its low 32 bits
    fn32_args={"gm_8016A404": [0]},
)
for _s in SPECS.values():
    _s["common_ret"] = COMMON_RET
