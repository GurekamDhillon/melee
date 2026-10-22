TU = "src/melee/gm/gmomake15.c"
RUST_MOD = "melee_modes::tu::gmomake15"
CASES = {
    "gm_Scene_Omake15_OnEnter": dict(n=20, args=["null"]),
    "gm_Scene_Omake15_OnFrame": dict(n=400),
}
DOM = {
    "ret:lbMthp_8001F604": [0, 1],
    "ret:gm_GetButtonsTriggered": [0, 0x1000, 0x100, 0x1100, 0x10, 0x1110],
    ".xC": [0, 1],
}

PORTED = {"gm_801A9DD0": "crate::tu::gmopening::gm_801A9DD0", "fn_801AA0E8": "crate::tu::gmopening::fn_801AA0E8"}
