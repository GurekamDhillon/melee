TU = "src/melee/gm/gmopening.c"
RUST_MOD = "melee_modes::tu::gmopening"
SIZE_OK = ["HSD_JObj"]
SKIP_LEAF_TYPES = ["HSD_IDKey"]
CASES = {
    "gm_801A9DD0": dict(n=400, args=["p:HSD_GObj"]),
    "fn_801A9FCC": dict(n=200),
    "fn_801AA0E8": dict(n=5),
    "gm_Scene_Opening_OnEnter": dict(n=100, args=["null"]),
    "gm_Scene_Opening_OnFrame": dict(n=6000),
}
_thr = [0, 1, 0x1C5, 0x1C6, 0x1C7, 0x201, 0x202, 0x203, 0x3B5, 0x3B6, 0x3B7, 0x3CD, 0x3CE, 0x3CF, 0x1389, 0x13B2,
        0x1409, 0x140A, 0x140B, 0x157B, 0x157C, 0x157D, 5400, 5401, 4979, 4980, 5033, 5034, 5099, 5100, 5101, 6000, 20000]
DOM = {
    "arg:gm_801A9DD0#1": [0, 1, 0x248, 0x249, 0x280, 640, 1000, 65535, 100, 0x247],
    "arg:gm_801A9DD0#2": [0, 1, 0x1E0, 480, 65535, 7],
    "arg:gm_801A9DD0#3": [0, 8, 0xB, 0xFF, -1],
    "arg:gm_801A9DD0#4": [0, 1, 2, -1],
    "ret:lbMthp_8001F5C4": _thr,
    "ret:lbMthp_8001F604": [0, 1],
    "ret:lbLang_IsSettingUS": [0, 1],
    "ret:gm_80173754": [0, 1],
    "ret:gm_GetButtonsTriggered": [0, 0, 0x1000, 0x100, 0x1100, 0x10],
    "ret:DVDConvertPathToEntrynum": [-1, 0, 7],
    "ret:lbMthp_8001F5F4": [0, 1, 59, 60, 99, 100, -1, -15, 1000000],
    "ret:lbMthp_8001F5E4": [0, 1, 59, 60, 99, 100, 0xFFFFFFFF, 1000000],
    "ret:lbMthp_8001F5D4": [0, 1, 40499, 40500, 405000, 4050000, 0xFFFFFFFF, 12345678],
    "g:__OSBusClock": [0, 3999, 4000, 162000000, 0xFFFFFFFF],
    "g:gm_804D67EC": _thr,
    "g:gm_804D67E4": _thr,
    "g:gm_804D67D0": [0, 1],
    "g:gm_804D67DC": [0, 1],
    "g:gm_804D67E0": [0, 1],
    "g:gm_804D67E1": [0, 1],
    "g:gm_804D67E2": [0, 1],
    "g:gm_Opening_NoMovie": [0, 0, 1],
    "gmMainLib_8046B0F0.xC": [0, 1],
}
LINKS = {
    "g:gm_804D67D4": "maybe:HSD_GObj",
    "g:gm_804D67D8": "maybe:HSD_GObj",
    "g:gm_804D67F0": "new:HSD_SObjDesc",
    "ret:GObj_Create": "new:HSD_GObj",
    "ret:lbMthp_8001F624": "new:HSD_SObj",
    "ret:HSD_SObjLib_803A477C": "new:HSD_SObj",
    "ret:gmTitle_801A12C4": "new:HSD_GObj",
    "ret:HSD_CObjAlloc": "new:HSD_CObj",
}
PTR_BYTES = {
    "HSD_CObjSetViewport": {1: 8},
    "HSD_CObjSetScissor": {1: 8},
    "HSD_CObjSetEyePosition": {1: 12},
    "HSD_CObjSetInterest": {1: 12},
}
GUEST_VARS = {0x800000F8: ("__OSBusClock", "unsigned int", "u32")}
