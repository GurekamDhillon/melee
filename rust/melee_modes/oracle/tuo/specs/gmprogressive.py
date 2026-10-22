TU = "src/melee/gm/gmprogressive.c"
RUST_MOD = "melee_modes::tu::gmprogressive"
RUST_GLOBAL_TYPES = {"gm_80480D70": "melee_modes::tu::gmprogressive::ProgScanState"}
SIZE_OK = ["HSD_JObj"]
SKIP_LEAF_TYPES = ["HSD_IDKey"]
CASES = {
    "gm_801AD088": dict(n=50),
    "gm_801AD254": dict(n=300),
    "gm_Scene_ProgScan_OnFrame": dict(n=2000),
    "gm_Scene_ProgScan_OnEnter": dict(n=100, args=["cell:32"]),
    "gm_Scene_ProgScan_OnExit": dict(n=50, args=["cell:32"]),
}
_f = [a | b | c | d for a in (0, 0x40) for b in (0, 0x800000) for c in (0, 0x2000000) for d in (0, 0x10)]
_btn = [0, 0, 0, 1 << 32, 1 << 38, 1 << 39, (1 << 32) | (1 << 38), (1 << 38) | (1 << 39), 0x1000, (1 << 32) | (1 << 39)]
DOM = {
    ".flags": _f,
    "arg:gm_801AD254#0": [0, 1, 2, 3, 4, 5, 6, -1],
    "arg:gm_Scene_ProgScan_OnEnter#0": [0, 1, 2, 3, 4, 5],
    "gm_80480D70.x1C": [0, 1, 2, 9, 10, 11, 12, 0x6D, 0x6E, 0x6F, 0xFFFFFFFF],
    "gm_80480D70.x10": [0, 1, 2, 3, 4, 5],
    "gm_80480D70.x14": [0, 0, 1, 2],
    "gm_80480D70.x18": [0, 1, 0x12B, 0x12C, 0x12D, 0x1000],
    "gm_80480D70.x20": [0, 1],
    "ret:gm_GetButtonsTriggered": _btn,
    "ret:lbLang_IsSavedLanguageUS": [0, 1],
}
LINKS = {
    "gm_80480D70.x4": "new:HSD_JObj",
    "gm_80480D70.x8": "new:HSD_Text",
    "out:lbArchive_80016DBC#1": "new:SceneDesc",
    ".models": "ptrs:DynamicModelDesc:1",
    ".cameras": "new:SceneCameraDesc",
}
VARIADIC = {
    "lbArchive_80016DBC": dict(ret="p", ret_rust="*mut melee_types::HSD_Archive", link="buf:64",
                               args=[("s", "*const i8"), ("p", "*mut core::ffi::c_void"), ("s", "*const i8"), ("i", "i32")]),
}
OUTS = {"lbArchive_80016DBC": [1]}
