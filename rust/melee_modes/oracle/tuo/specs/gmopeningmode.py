TU = "src/melee/gm/gmopeningmode.c"
RUST_MOD = "melee_modes::tu::gmopeningmode"
SIZE_OK = ["HSD_JObj"]
SKIP_LEAF_TYPES = ["HSD_IDKey"]
_idx = [0, 1, 2, 3]
CASES = {
    "onExitTitle": dict(n=800, args=["p:GameModeState"]),
    "gm_SetupTitleDemo": dict(n=200),
    "gm_PreloadTitleDemo": dict(n=100),
    "onEnterVs": dict(n=100, args=["p:GameModeState"]),
    "gm_801BF634": dict(n=40),
    "gm_801BF648": dict(n=20),
    "gm_801BF65C": dict(n=40),
    "gm_801BF670": dict(n=20),
    "gm_801BF684": dict(n=20),
    "gm_801BF694": dict(n=10),
    "gm_801BF6A8": dict(n=20),
    "gm_801BF6B8": dict(n=10),
    "gm_801BF6C8": dict(n=20),
    "gm_801BF6D8": dict(n=10),
    "gm_801BF6E8": dict(n=20),
    "gm_801BF6F8": dict(n=10),
    "gm_801BF708": dict(n=20),
    "gm_801BF718": dict(n=10),
    "onEnterCutsceneLuigi": dict(n=100, args=["p:GameModeState"]),
    "onExitRegendCongrats": dict(n=10, args=["p:GameModeState"]),
    "onEnterMovie": dict(n=30, args=["p:GameModeState"]),
    "onExitMovie": dict(n=5, args=["p:GameModeState"]),
    "onExitHowto": dict(n=5, args=["p:GameModeState"]),
    "onExitOmake15": dict(n=5, args=["p:GameModeState"]),
}
DOM = {
    "arg:gm_801BF634#0": _idx, "arg:gm_801BF648#0": _idx, "arg:gm_801BF65C#0": _idx, "arg:gm_801BF670#0": _idx,
    "arg:gm_801BF634#1": [0, 1, 5, 25, 26, 0x7F, -1],
    "ret:HSD_Randi": [0, 1, 2, 3, 4, 5, 6, 7],
    "ret:gm_IsCKindUnlocked": [1, 1, 1, 1, 0],
    "ret:gm_80164430": [1, 1, 0],
    "ret:gm_80173754": [0, 1],
    "g:DbLevel": [0, 1, 2, 3, 4, 5],
    "gm_8049E548.unk_A": [0, 1, 2],
}
LINKS = {
    "g:gmMainLib_804D3EE0": "new:struct~gmm_x0",
    "ret:gm_GetGameModeStateEnterData": "new:StartMeleeData",
    "ret:gm_GetGameModeStateExitData": "buf:8",
    "ret:lbDvd_GetPreloadCacheScene": "new:PreloadedGameModeState",
}
# the save data (68 KB) is only needed by these two functions
LINK_GATE = {"g:gmMainLib_804D3EE0": ["gm_SetupTitleDemo", "onEnterVs"]}
