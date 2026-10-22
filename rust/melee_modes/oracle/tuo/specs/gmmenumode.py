TU = "src/melee/gm/gmmenumode.c"
RUST_MOD = "melee_modes::tu::gmmenumode"
RUST_TYPES = {"DebugSoundTestData": "melee_modes::tu::gmmenumode::DebugSoundTestData"}
SIZE_OK = ["HSD_JObj"]
SKIP_LEAF_TYPES = ["HSD_IDKey"]
CASES = {
    "onEnterDebug": dict(n=30, args=["p:GameModeState"]),
    "onEnter": dict(n=1200, args=["p:GameModeState"]),
    "onExit": dict(n=100, args=["p:GameModeState"]),
}
OUT_INTS = {"gmFrontend_NativeRequest": {0: 8, 1: 8}}
_modes = list(range(0, 64)) + [0xFF]
DOM = {
    "ret:gm_GetPreviousGameMode": _modes,
    "ret:gm_801737D8": _modes,
    "ret:gmFrontend_NativeRequest": [0, 0, 0, 1],
    "force_main_menu": [0, 1, 2],
}
LINKS = {
    ".info.enter_data": "new:MenuEnterData",
    ".info.exit_data": "new:MenuExitData",
    "ret:gm_GetGameModeStateEnterData": "new:struct~DebugSoundTestData",
    "ret:gmMainLib_GetGameRules": "new:GameRules",
}
TYPES = ["MenuEnterData", "MenuExitData", "GameRules"]
