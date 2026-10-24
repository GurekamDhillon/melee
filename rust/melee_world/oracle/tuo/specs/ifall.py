import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from common import *
TU = "src/melee/if/ifall.c"
RUST_MOD = "melee_world::tu::ifall"
SIZE_OK = ["HSD_JObj"]
SKIP_LEAF_TYPES = ["HSD_IDKey"]
RUST_TYPES = {"ifAll_804A0FD8_t": "melee_world::tu::ifall::ifAll_804A0FD8_t"}
CASES = {
    "ifAll_802F3394": dict(n=4),
    "ifAll_802F33CC": dict(n=4),
    "ifAll_GetHUDGObj": dict(n=20),
    "ifAll_GetTimerPosition": dict(n=4),
    "ifAll_GetPlayerHUDPosition": dict(n=12),
    "ifAll_802F343C": dict(n=500),
    "ifAll_GetArchive": dict(n=4),
    "ifAll_HideHUD": dict(n=4),
    "ifAll_ShowHUD": dict(n=4),
    "ifAll_IsHUDHidden": dict(n=40),
    "fn_802F36B8": dict(n=80, args=["p:HSD_GObj", None]),
    "ifAll_802F370C": dict(n=40, args=["p:SceneDesc"]),
    "ifAll_802F390C": dict(n=40),
    "ifAll_802F3A64": dict(n=80),
}
VARIADIC = {
    "lb_80011E24": dict(ret="i", args=[("p", "*mut melee_types::HSD_JObj"), ("p", "*mut *mut melee_types::HSD_JObj"), ("i", "i32"), ("i", "i32")]),
    "lbArchive_LoadSections": dict(ret="v", args=[("p", "*mut melee_types::HSD_Archive"), ("p", "*mut *mut core::ffi::c_void"), ("s", "*const i8"), ("i", "i32")]),
}
OUTS = {"lb_80011E24": [1], "lbArchive_LoadSections": [1]}
OUT_F32S = {"lb_8000B1CC": {2: 3}}
DOM = {
    "ret:Wb_RenderOption": [0, 1],
    "ret:lb_80011E24": [0, 1],
    "ret:HSD_CObjSetCurrent": [0, 1],
    "arg:ifAll_GetPlayerHUDPosition#0": [0, 1, 2, 3, 4, 5],
    "arg:ifAll_802F343C#0": [0, 1, 2, 3, 4, 5, 6, 6, 6, 7],
    "g:hidden": [0, 1],
}
LINKS = {
    ".hsd_obj": "new:HSD_CObj",
    ".models": "ptrs:DynamicModelDesc:1",
    ".cameras": "ptrs:SceneCameraDesc:1",
    ".lights": "ptrs:LightList:1",
    "out:lb_80011E24#1": "new:HSD_JObj",
    "out:lbArchive_LoadSections#1": "new:SceneDesc",
    "ret:GObj_Create": "new:HSD_GObj",
    "ret:HSD_JObjLoadJoint": "new:HSD_JObj",
    "g:ifAll_804A0FD8.gobj": "maybe:HSD_GObj",
    "g:ifAll_804A0FD8.gobj_2": "maybe:HSD_GObj",
    "g:ifAll_804A0FD8.x8": "new:HSD_JObj",
}
