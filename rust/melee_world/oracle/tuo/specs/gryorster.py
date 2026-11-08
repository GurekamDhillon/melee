import os, sys, struct
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from common import *
TU = "src/melee/gr/gryorster.c"
RUST_MOD = "melee_world::tu::gryorster"
SIZE_OK = ["HSD_JObj"]
SKIP_LEAF_TYPES = ["HSD_IDKey"]
RUST_TYPES = {"YorsterParams": "melee_world::tu::gryorster::YorsterParams", "grYt_804D6A20_t": "melee_world::tu::gryorster::grYt_804D6A20_t"}
PATCH = [("(Ground*) ((s32) gp + 0)", "gp")]


def F(x):
    return "f:%08x" % struct.unpack("<I", struct.pack("<f", x))[0]


CASES = {
    "grYorster_80201FA0": dict(n=4),
    "grYorster_80201FA4": dict(n=20),
    "grYorster_8020200C": dict(n=4),
    "grYorster_80202010": dict(n=4),
    "grYorster_80202034": dict(n=4),
    "grYorster_8020203C": dict(n=60),
    "stageGObj0_OnInit": dict(n=20, args=["p:HSD_GObj"]),
    "grYorster_80202150": dict(n=4, args=["p:HSD_GObj"]),
    "grYorster_80202158": dict(n=4, args=["p:HSD_GObj"]),
    "grYorster_8020215C": dict(n=4, args=["p:HSD_GObj"]),
    "grYorster_80202160": dict(n=60, args=["p:HSD_GObj"]),
    "grYorster_802021AC": dict(n=40, args=["p:HSD_GObj"]),
    "grYorster_8020224C": dict(n=4, args=["p:HSD_GObj"]),
    "grYorster_80202254": dict(n=300, args=["p:HSD_GObj"]),
    "grYorster_802022A0": dict(n=4, args=["p:HSD_GObj"]),
    "grYorster_802022A4": dict(n=60, args=["p:HSD_GObj"]),
    "grYorster_80202428": dict(n=300, args=["p:HSD_GObj", "p:Ground", "p:Vec3", "p:HSD_GObj", "f32"]),
    "grYorster_802024F0": dict(n=500, args=["p:Ground", "i32s", "p:CollData", "i32s", "i32u", "f32"]),
    "grYorster_8020266C": dict(n=2000, args=["p:HSD_GObj"]),
    "grYorster_80202B5C": dict(n=4),
    "grYorster_80202B64": dict(n=4, args=["p:Vec3", None, "p:HSD_JObj"]),
}
FMT_VARIADIC = ["OSReport"]
OUT_F32S = {"ftLib_GetPosDelta": {1: 3}}
PTR_BYTES = {"grLib_801C9EE8": {0: 12}}
_els = range(9)
DOM = {}
for i in _els:
    DOM[f".elements[{i}].x01"] = [0, 0, 1, 1, 2, 3, 4, 5]
    DOM[f".elements[{i}].x04"] = [F(v) for v in (0.0, 0.0, 1.0, 2.5, 5.0, 10.0)]
    DOM[f".elements[{i}].x08"] = [F(v) for v in (0.0, 0.5, 1.0, 2.0, 5.0)]
    DOM[f".elements[{i}].x0C"] = [-1, 0, 1, 24, 25, 0x142, 0x143, 0x144]
    DOM[f".elements[{i}].x10"] = [0, 1, 5, 17, 30]
    DOM[f".elements[{i}].x14"] = [0xA, 0xB, 0xC, 0xD, 0xE, 0xF, 0x10, 0x11, 0x12, 3]
DOM.update({
    "p0.x00": [], 
})
del DOM["p0.x00"]
DOM.update({
    ".x00": [F(v) for v in (-5.0, 0.0, 0.5, 1.0, 2.0)],
    ".x04": [F(v) for v in (0.5, 1.0, 2.0, 5.0)],
    ".x08": [F(v) for v in (0.5, 1.0, 2.0, 3.0)],
    ".x0C": [F(v) for v in (1.0, 5.0, 10.0, 60.0)],
    ".x18": [0, 1, 5],
    ".x1C": [10, 30, 100],
    ".x34_flags.b1234": [0, 1, 1, 1, 2, 3, 15],
    "ret:Ground_801C32D4": [0, 1, 2, 3, 0xA, 0xB, 0xC, 0xD, 0xE, 0xF, 0x10, 0x11, 0x12],
    "arg:grYorster_802024F0#1": [0, 1, 2, 0xA, 0xB, 0xC, 0xD, 0xE, 0xF, 0x10, 0x11, 0x12],
    "arg:grYorster_802024F0#5": [F(v) for v in (-3.0, 0.0, 0.5, 1.0, 4.0, 9.0)],
    "arg:grYorster_80202428#4": [F(v) for v in (0.0, 0.25, 1.0, 3.5)],
    "ret:lbGetJObjEndFrame": [F(v) for v in (0.0, 5.0, 50.0, 100.5, 200.0)],
    "ret:Ground_801C0498": [F(v) for v in (0.5, 1.0, 2.0)],
    "ret:grAnime_801C84A4": [0, 1],
    "ret:grLib_801C9EE8": [0, 1],
    "ret:ftLib_IsFighter": [0, 1],
    "ret:ftLib_IsEntry": [0, 1],
    "out:ftLib_GetPosDelta#1[1]": [F(v) for v in (-3.0, 0.0, 0.5, 1.0, 4.0, 9.0)],
    "arg:grYorster_8020203C#0": [0, 1],
})
LINKS = {
    ".x1C": "mayarg:0:null",
    "g:grYt_804D6A20.x0": "maybe:YorsterParams",
    "ret:Ground_GetYakumonoParam": "new:YorsterParams",
    ".user_data": "new:Ground",
    ".hsd_obj": "new:HSD_JObj",
    ".x18": "new:HSD_JObj",
    "ret:Ground_GetStageGObj": "maybe:HSD_GObj",
    "ret:Ground_801C3FA4": "new:HSD_JObj",
    "ret:grMaterial_801C8CFC": "new:HSD_GObj",
    ".x0_gobj": "new:HSD_GObj",
}
LINK_FORCE = {"g:grYt_804D6A20.x0": [
    "grYorster_80202428", "grYorster_802024F0", "grYorster_8020266C"]}
PRUNE = {"Ground": ["map_id", "u", "x8_callback", "xC_callback", "x1C_callback"], "CollData": ["x0_gobj", "x34_flags"]}
UNION_VIEW = {"Ground.u": "yorster"}
CUSTOM_STUBS = {"Mex_GrBind": MEX_GRBIND}
