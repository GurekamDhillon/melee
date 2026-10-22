"""Per-TU oracle specs for gen_tu.py / run_tu.py (see gen_tu.py)."""
# Return values of callees that the C TUs share (a stub returns what the real function would).
COMMON_RET = {
    "gm_GetGameModeStateEnterData": ("a0->info.enter_data", "(*a0).info.enter_data"),
    "gm_GetGameModeStateExitData": ("a0->info.exit_data", "(*a0).info.exit_data"),
}

SPECS = {
    "gmhanyucss": dict(
        src="src/melee/gm/gmhanyucss.c", types=["CSSData", "VsModeData", "GameModeState"],
        ret={"gmVsMelee_GetVsData": ("&VSD", "&raw mut super::VSD")}),
    "gmgover": dict(src="src/melee/gm/gmgover.c", types=["GameModeState"]),
    "gmgiant": dict(src="src/melee/gm/gmgiant.c", types=["GameModeState", "PlayerInitData"]),
    "gmfixedcamera": dict(src="src/melee/gm/gmfixedcamera.c", types=["GameModeState", "StartMeleeData"]),
    "gmcameramode": dict(src="src/melee/gm/gmcameramode.c",
                         types=["GameModeState", "CSSData", "SSSData", "StartMeleeData", "VsModeData", "PreloadedGameModeState"],
                         null_ret_ok=["lbDvd_80017740"],
                         ret={"gmVsMelee_GetKOCounts": ("KOC", "&raw mut super::KOC as *mut u8"),
                              "lbDvd_GetPreloadCacheScene": ("&PCS", "&raw mut super::PCS"),
                              "lbDvd_GetPreloadedArchive": ("(void*) (AR + (a0 & 15))",
                                                            "(&raw mut super::AR as *mut u8).wrapping_add((a0 & 15) as usize) as *mut c_void")}),
    "gmdebugmode": dict(src="src/melee/gm/gmdebugmode.c", types=["GameModeState", "ResultsMatchInfo", "StartMeleeData", "MatchExitInfo", "DebugGameOverData",
                               "DebugResultsData", "DebugMemcardData", "enterMenu0_Data", "exitIntroData", "un_80301C64_t",
                               "HSD_PadStatus"],
                        data_size={"un_803FA258": 128}),
}
for _s in SPECS.values():
    _s["common_ret"] = COMMON_RET
