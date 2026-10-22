TU = "src/melee/gm/gmmultiman.c"
RUST_MOD = "melee_modes::tu::gmmultiman"
SIZE_OK = ["HSD_JObj"]
SKIP_LEAF_TYPES = ["HSD_IDKey"]
VARIADIC = {"OSReport": dict(ret="v", args=[("s", "*mut i8"), ("i", "i32")])}
OUT_INTS = {"Ground_801C1DE4": {0: 32, 1: 32}}

# (table, [CSS enter, CSS exit, VS enter, VS exit])
_TABLES = {
    "gm_Mode_TargetTest_States": ["gm_801B632C", "gm_801B63C4", "gm_801B6428", "gm_801B65D4"],
    "gm_Mode_10ManVs_States": ["gm_801B6AD8", "gm_801B6B70", "gm_801B6BE8", "gm_801B6F44"],
    "gm_Mode_100ManVs_States": ["gm_801B7044", "gm_801B70DC", "gm_801B7154", "gm_801B74F0"],
    "gm_Mode_3Min_States": ["gm_801B75F0", "gm_801B7688", "gm_801B7700", "gm_801B7AA0"],
    "gm_Mode_15Min_States": ["gm_801B7B74", "gm_801B7C0C", "gm_801B7C84", "gm_801B8024"],
    "gm_Mode_EndlessVs_States": ["gm_801B8110", "gm_801B81A8", "gm_801B8220", "gm_801B8580"],
    "gm_Mode_CruelVs_States": ["gm_801B863C", "gm_801B86D4", "gm_801B874C", "gm_801B8AF8"],
}
CASES = {
    "gm_801B6320": dict(n=10),
    "gm_Mode_TargetTest_OnInit": dict(n=10),
    "gm_Mode_TargetTest_OnLoad": dict(n=60),
    "gm_Mode_10ManVs_OnInit": dict(n=10),
    "gm_Mode_10ManVs_OnLoad": dict(n=10),
    "gm_801B688C": dict(n=300, args=["i32s"]),
    "gm_801B69C0": dict(n=40, args=["p:StartMeleeData"]),
    "gmMultiman_LeaveFinish": dict(n=20, args=["e:Multiman", "p:CSSData"]),
    "gm_801B6AD8_inline": dict(n=60, args=["e:CruelCSS", None]),
}
ARG_EXPRS = {"Multiman": ("&gmMainLib_804D3EE0->modes.table[GmVsMode_Multiman]",
                          "&mut (*melee_modes::tu::gmmultiman::ext::gmMainLib_804D3EE0).modes.table[melee_types::GmVsMode_Multiman as usize] as *mut melee_types::VsModeData")}
for _t, _fs in _TABLES.items():
    ARG_EXPRS[_t + "0"] = (f"&{_t}[0]", f"&raw mut melee_modes::tu::gmmultiman::{_t}[0]")
    ARG_EXPRS[_t + "1"] = (f"&{_t}[1]", f"&raw mut melee_modes::tu::gmmultiman::{_t}[1]")
    CASES[_fs[0]] = dict(n=60 if "Target" in _t else 30, args=["e:" + _t + "0"])
    CASES[_fs[1]] = dict(n=60 if "Target" in _t else 30, args=["e:" + _t + "0"])
    CASES[_fs[2]] = dict(n=60 if "Target" in _t else 40, args=["e:" + _t + "1"])
    CASES[_fs[3]] = dict(n=300 if "Target" in _t else 80, args=["e:" + _t + "1"])
ARG_EXPRS["CruelCSS"] = ARG_EXPRS["gm_Mode_CruelVs_States0"]
CASES["gm_801B6AD8_inline"] = dict(n=30, args=["e:gm_Mode_10ManVs_States0", "i32s"])
CASES["gmMultiman_LeaveFinish"] = dict(n=30, args=["e:Multiman", "p:CSSData"])

_sel = list(range(0, 32)) + [32, 40, 255]
DOM = {
    "ret:gm_CKindToSelKind": _sel,
    "out:Ground_801C1DE4#0": [0, 0, 1, 5000],
    "out:Ground_801C1DE4#1": [0, 100, 5000, 100000],
    "ret:gmMainLib_8015D48C": [0, 1],
    "ret:gm_801733D8": [33, 33, 5, 0, 1],
    "ret:gm_8017341C": [0x148, 0x148, 0, 5, 0x149],
    "ret:gm_80181A14": [0, 1, 2],
    "ret:gm_GetCurrentGameMode": [33, 34, 35, 36, 37, 38, 0],
    "ret:gm_80173498": [0x148, 0x148, 0x10, 0],
    "ret:gm_80173460": [33, 33, 5, 0],
    "ret:gm_80173754": [0, 1],
    "ret:TestTargetTestCKind": [-1, -1, -1, 0, 5, 25, 0x7F],
    "ret:gmMainLib_8015D6BC": [0, 1, 2],
    "ret:gmMainLib_8015D710": [0, 1, 2],
    "ret:gmMainLib_8015D74C": [0, 0, 1, 77, 0x7FFF, -1],
    "ret:gmMainLib_8015D7A4": [0, 0, 1, 77, 0x7FFF, -1],
    "ret:gmMainLib_8015D6A4": [0, 5, 100, 100000, -1],
    "ret:gmMainLib_8015D6F8": [0, 5, 100, 100000, -1],
    "ret:gmMainLib_8015D7BC": [0, 5, 100, 100000, -1],
    "ret:gmMainLib_8015D7D4": [0, 5, 100, 100000, -1],
    "ret:gmMainLib_8015D438": [0, 5, 100, 0xFFFFFFFF],
    "ret:gmMainLib_8015D450": [0, 5, 100, 100000, -1],
    ".pending_scene_change": [0, 1, 2, 2],
    ".match_end.outcome": [0, 1, 2, 3, 4, 5, 6, 7, 8, 8],
    ".match_end.frame_count": [0, 59, 60, 100, 3600, 100000, 0xFFFFFFFF],
    ".x0_0": [0, 1],
    "g:gm_804D68E9": [0, 1],
    "g:gm_804D68E8": [0, 1, 5, 255],
    "g:gm_804D68F0": [0, 1, 5, 255],
    ".x8": [0, 0, 1, 2, 0xFF, -1],
    ".x4": [0, 5, 50, 0x7FFFFFFF],
    ".x2": [0, 1, 99, 0xFFFF],
}
LINKS = {
    "g:gmMainLib_804D3EE0": "new:struct~gmm_x0",
    "ret:lbDvd_GetPreloadCacheScene": "new:PreloadedGameModeState",
    "ret:gmMainLib_GetGameRules": "new:GameRules",
    "ret:gmMainLib_8015D438": "cell:32",
    "ret:gmMainLib_8015D450": "cell:32",
    "ret:gmMainLib_8015D6A4": "cell:32",
    "ret:gmMainLib_8015D6F8": "cell:32",
    "ret:gmMainLib_8015D7BC": "cell:32",
    "ret:gmMainLib_8015D7D4": "cell:32",
    "ret:gmMainLib_8015D74C": "cell:16",
    "ret:gmMainLib_8015D7A4": "cell:16",
}
RET_FIELD = {
    "gm_GetGameModeStateEnterData": ("GameModeState", ["info", "enter_data"]),
    "gm_GetGameModeStateExitData": ("GameModeState", ["info", "exit_data"]),
}
# the save data is large: only the parts this TU touches
PRUNE = {"struct gmm_x0": ["vs.unk_530.unk_584", "modes.table[12]"]}
