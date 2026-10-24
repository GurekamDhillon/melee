import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from common import *
TU = "src/melee/gr/grzebesroute.c"
RUST_MOD = "melee_world::tu::grzebesroute"
SIZE_OK = ["HSD_JObj"]
SKIP_LEAF_TYPES = ["HSD_IDKey"]
RUST_TYPES = {"grZebesRoute_YakumonoParam": "melee_world::tu::grzebesroute::grZebesRoute_YakumonoParam"}
CASES = {
    "grZebesRoute_8020B160": dict(n=4),
    "grZebesRoute_8020B164": dict(n=40),
    "grZebesRoute_8020B1D4": dict(n=4),
    "grZebesRoute_8020B1F4": dict(n=200),
    "grZebesRoute_8020B258": dict(n=4),
    "grZebesRoute_8020B260": dict(n=60),
    "stageGObj0_OnInit": dict(n=10, args=["p:HSD_GObj"]),
    "grZebesRoute_8020B374": dict(n=4, args=["p:HSD_GObj"]),
    "grZebesRoute_8020B37C": dict(n=4, args=["p:HSD_GObj"]),
    "grZebesRoute_8020B380": dict(n=4, args=["p:HSD_GObj"]),
    "stageGObj2_OnInit": dict(n=10, args=["p:HSD_GObj"]),
    "grZebesRoute_8020B3B0": dict(n=4, args=["p:HSD_GObj"]),
    "grZebesRoute_8020B3B8": dict(n=4, args=["p:HSD_GObj"]),
    "grZebesRoute_8020B3BC": dict(n=4, args=["p:HSD_GObj"]),
    "grZebesRoute_8020B3C0": dict(n=60, args=["p:HSD_GObj"]),
    "grZebesRoute_8020B424": dict(n=4, args=["p:HSD_GObj"]),
    "grZebesRoute_8020B42C": dict(n=300, args=["p:HSD_GObj"]),
    "grZebesRoute_8020B4D4": dict(n=4, args=["p:HSD_GObj"]),
    "fn_8020B4D8": dict(n=400, args=["p:Ground", "i32s", "p:CollData", "i32s", "i32u", "f32"]),
    "grZebesRoute_8020B548": dict(n=400),
    "grZebesRoute_8020B854": dict(n=4),
    "grZebesRoute_8020B85C": dict(n=4, args=["p:Vec3", None, "p:HSD_JObj"]),
}
VARIADIC = {}
FMT_VARIADIC = ["OSReport"]
OUT_F32S = {"ftLib_80086644": {1: 3}}
EXTERN_ARRAY_LEN = {"HSD_GObjGXLinkHead": 64}
DOM = {
    "ret:Ground_801C0498": [f"f:{b:08x}" for b in (0x3f800000, 0x40000000, 0x3f000000, 0x40400000, 0x3dcccccd)],
    "ret:HSD_LObjGetPosition": [1],
    "ret:HSD_RandiHack": [0],
    "ret:HSD_Randi": [0, 0, 1, 2, 3],
    "g:yakumono_param": [],
    ".flags": [0, 1, 2, 3, 4, 5, 6, 7, 0x10, 0x13],
    ".zako_spawn_chance": [0, 0, 1, 2, 5, 100],
    ".camera_timer": [0, 1, 5, 100, 1000, 40000, -1],
    "out:ftLib_80086644#1[1]": [f"f:{b:08x}" for b in (0xc2480000, 0xc2000000, 0x42c80000, 0x0, 0xc2a00000, 0x41200000)],
    "out:ftLib_80086644#1[0]": ["f:3f800000", "f:0"],
    ".x34_flags.b1234": [0, 1, 1, 1, 2, 3, 15],
    "arg:fn_8020B4D8#4": [0, 1, 1, 2, 3],
    "arg:grZebesRoute_8020B260#0": [0, 1, 2, 3],
}
LINKS = {
    "g:yakumono_param": "new:struct~grZebesRoute_YakumonoParam",
    "ret:Ground_GetYakumonoParam": "new:struct~grZebesRoute_YakumonoParam",
    ".user_data": "new:Ground",
    "ret:Ground_GetStageGObj": "maybe:HSD_GObj",
    "ret:Ground_GetP1Fighter": "maybe:HSD_GObj",
    "ret:ftLib_80086644": "null",
    ".x0_gobj": "maybe:HSD_GObj",
    "g:HSD_GObjGXLinkHead": "ptrs:HSD_GObj:8",
    ".hsd_obj": "new:HSD_LObj",
    ".next": "maybe:HSD_LObj",
}
PRUNE = {"CollData": ["x0_gobj", "x34_flags"]}

CUSTOM_STUBS = {"Mex_GrBind": MEX_GRBIND}
