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
}
for _s in SPECS.values():
    _s["common_ret"] = COMMON_RET
