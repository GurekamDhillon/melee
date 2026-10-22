TU = "src/melee/gm/gmmovieend.c"
RUST_MOD = "melee_modes::tu::gmmovieend"
SIZE_OK = ["HSD_JObj"]
SKIP_LEAF_TYPES = ["HSD_IDKey"]
CASES = {
    "gm_ModeState_ApproachVs_OnExit": dict(n=400, args=["p:GameModeState"]),
    "gm_ModeState_Prize_OnExit": dict(n=20, args=["p:GameModeState"]),
    "gm_Scene_DebugMenu_OnEnter": dict(n=50, args=["buf:16"]),
    "gm_Scene_MovieEnd_OnEnter": dict(n=100, args=["null"]),
    "gm_Scene_MovieEnd_OnFrame": dict(n=3000),
}
TYPES = ["GameModeState"]
DOM = {
    ".match_end.outcome": [0, 1, 2, 3, 4, 5, 6, 7, 8],
    ".match_end.frame_count": [0, 59, 60, 61, 3600, 100000, -5],
    "ret:gm_801BEFB0": list(range(0, 26)),
    "ret:gm_CKindToUnlockIndex": [0, 1, 5, 0xFF],
    "ret:gm_801721EC": [0, 1],
    "ret:lbMthp_8001F604": [0, 1],
    "ret:lbMthp_8001F5C4": [0, 5, 100, 419, 420, 421, 1000],
    "ret:gm_801BF010": [0, 1, 2, 3, 4],
    "ret:gm_GetButtonsTriggered": [0, 0, 0, 0x1000, 0x100, 0x1100],
    "gmMainLib_8046B0F0.xC": [0, 1],
    "g:gm_804D6738": [0, 1, 0x1A2, 0x1A3, 0x1A4, 0x1A5, 0x2000],
    "g:gm_804D673C": [0, 0, 1, 2, 30],
}
LINKS = {
    "ret:gm_GetGameModeStateExitData": "new:MatchExitInfo",
    "ret:gm_GetChallengerData": "new:ChallengerData",
    "ret:GObj_Create": "new:HSD_GObj",
    "ret:lbMthp_8001F624": "new:HSD_SObj",
}

PORTED = {"gm_801A9DD0": "crate::tu::gmopening::gm_801A9DD0", "fn_801AA0E8": "crate::tu::gmopening::fn_801AA0E8"}
