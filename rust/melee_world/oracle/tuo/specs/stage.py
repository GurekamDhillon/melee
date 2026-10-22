import os, sys, struct
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from common import *
NL = chr(10)
TU = "src/melee/gr/stage.c"
RUST_MOD = "melee_world::tu::stage"
SIZE_OK = ["HSD_JObj"]
SKIP_LEAF_TYPES = ["HSD_IDKey"]
RUST_TYPES = {"StageSelection": "melee_world::tu::stage::StageSelection", "StageIdMapEntry": "melee_world::tu::stage::StageIdMapEntry"}
# the function-local `static bool done` of Stage_MexInitIds becomes a file-scope global (cases independent)
PATCH = [("    static bool done;" + NL, ""),
         ("void Stage_MexInitIds(void)" + NL + "{", "static bool done;" + NL + "void Stage_MexInitIds(void)" + NL + "{")]


def F(x):
    return "f:%08x" % struct.unpack("<I", struct.pack("<f", x))[0]


_kinds = list(range(0, 8)) + [60, 120, 200, 285, 286, 300, 383]
CASES = {}
for fn in ("Stage_GetCamBoundsLeftOffset", "Stage_GetCamBoundsRightOffset", "Stage_GetCamBoundsTopOffset",
           "Stage_GetCamBoundsBottomOffset", "Stage_GetCamPanAngleRadians", "Stage_GetCamMaxDepth",
           "Stage_GetCamZoomRate", "Stage_GetCamInfoX20", "Stage_GetCamInfoX24", "Stage_GetCamFixedZoom",
           "Stage_GetCamTrackRatio", "Stage_GetCamTrackSmooth", "Stage_GetBlastZoneRightOffset",
           "Stage_GetBlastZoneLeftOffset", "Stage_GetBlastZoneTopOffset", "Stage_GetBlastZoneBottomOffset",
           "Stage_CalcUnkCamY", "Stage_CalcUnkCamYBounds", "Stage_GetPauseCamZPosMin", "Stage_GetPauseCamZPosInit",
           "Stage_GetPauseCamZPosMax", "Stage_GetCamAngleRadiansUp", "Stage_GetCamAngleRadiansDown",
           "Stage_GetCamAngleRadiansLeft", "Stage_GetCamAngleRadiansRight", "Stage_GetCamFixedFov",
           "Stage_80225194", "Stage_8022524C", "Stage_80225298"):
    CASES[fn] = dict(n=40)
CASES.update({
    "Stage_UnkSetVec3TCam_Offset": dict(n=60, args=["p:Vec3"]),
    "Stage_80224CAC": dict(n=200, args=["p:Vec3"]),
    "Stage_SetVecToFixedCamPos": dict(n=40, args=["p:Vec3"]),
    "Stage_80224DC8": dict(n=60),
    "Stage_80224E38": dict(n=40, args=["p:Vec3", None]),
    "Stage_80224E64": dict(n=300, args=["i32s", "p:Vec3"]),
    "Stage_80224FDC": dict(n=300, args=["p:Vec3"]),
    "Stage_80225074": dict(n=200),
    "Stage_MexInitIds": dict(n=200),
    "Stage_8022519C": dict(n=80),
    "Stage_802251B4": dict(n=80),
    "Stage_802251E8": dict(n=80, args=[None, "buf:16"]),
    "Stage_802252E4": dict(n=80, args=[None, "p:HSD_GObj"]),
    "Stage_8022532C": dict(n=80),
    "Stage_StageSlotSelect": dict(n=200),
})
FMT_VARIADIC = ["OSReport"]
OUT_F32S = {"lbVector_ApplyEulerRotation": {0: 3}, "Ground_801C2D24": {1: 3}, "mpCheckMultiple": {4: 3}}
OUT_INTS = {"Ground_801C28AC": {2: 32}}
PTR_BYTES = {
    "lbVector_ApplyEulerRotation": {0: 12, 1: 12},
    "Ground_801C0754": {0: 8}, "Ground_801C0800": {0: 8}, "Ground_OnLoad": {0: 8},
    "Ground_801C0FB8": {0: 8}, "Ground_DemoInit": {0: 8},
}
FORCE_FILL = ["selected_stage"]
RAW_GLOBALS = ["stage_id_map"]
PRUNE = {"StageInfo": ["cam_info", "blast_zone", "unk8C"]}
_p = [F(v) for v in (0.0, 1.0, -1.0, 10.0, -10.0, 100.0)]
DOM = {
    "g:done": [0, 0, 1],
    "ret:Mex_GrExternalCount": [0, 1, 5, 100, 286, 384, 385, 500],
    "ret:Mex_GrKindForExt": [-1, -1, 0, 5, 33, 70, 99],
    "ret:Ground_801C2D24": [0, 0, 0, 1],
    "ret:mpCheckMultiple": [0, 0, 0, 0, 1],
    "ret:gm_8016B238": [0, 1],
    "ret:gm_8016B3A0": [0, 1],
    "ret:HSD_Randi": [0, 1, 2, 3, 5, 10, 20],
    ".b0": [0, 1],
    "arg:Stage_80224DC8#0": [0x3b, 0x3f, 0x42, 0x49, 0x4c, 0, 5, 100, 0x3c],
    "arg:Stage_80224E38#1": [-4, 0, 1, 5, 100],
    "arg:Stage_80224E64#0": [0, 1, 2, 3, 4, 4, 4, 5, 100, 130],
    "arg:Stage_80225074#0": [0, 1, 2, 3],
    "arg:Stage_8022519C#0": _kinds,
    "arg:Stage_802251B4#0": _kinds,
    "arg:Stage_802251E8#0": _kinds,
    "arg:Stage_802252E4#0": _kinds,
    "arg:Stage_8022532C#0": _kinds,
    "arg:Stage_StageSlotSelect#0": [0, 1, 2, 5, 8, 16, 33, 55, 66, 70, 71, 9999, -1],
    "out:Ground_801C2D24#1[0]": _p,
    "out:Ground_801C2D24#1[1]": _p,
    "out:mpCheckMultiple#4[0]": _p,
    "out:mpCheckMultiple#4[1]": _p,
    "out:lbVector_ApplyEulerRotation#0[0]": _p,
    "out:lbVector_ApplyEulerRotation#0[1]": _p,
    "out:lbVector_ApplyEulerRotation#0[2]": [F(v) for v in (-100.0, -50.0, 0.0, 50.0, 100.0, -1.0)],
    "out:Ground_801C28AC#2": [0, 1, 7, 100],
}
LINKS = {
    "g:selected_stage.entry": "&stage_id_map",
    "ret:lbVector_ApplyEulerRotation": "null",
}
CUSTOM_STUBS = {
    "ScriptGame_ArenaBound": dict(
        c=('extern unsigned long long tuo_draw(void); '
           'float ScriptGame_ArenaBound(int f, float fb) { unsigned b, r; float o; tuo_call_begin("ScriptGame_ArenaBound"); tuo_arg_i(f); '
           'memcpy(&b, &fb, 4); tuo_arg_f32(b); tuo_call_end(); r = (unsigned) tuo_draw(); if (r & 3) return fb; '
           'b = tuo_ret_f32("ScriptGame_ArenaBound"); memcpy(&o, &b, 4); return o; }'),
        rust=('unsafe extern "C" fn s_ScriptGame_ArenaBound(a0: i32, a1: f32) -> f32 { let v = vv(); v.call_begin("ScriptGame_ArenaBound"); '
              'v.arg_i(a0 as i64); v.arg_f32(a1.to_bits()); v.call_end(); let r = v.draw(); if r & 3 != 0 { return a1; } '
              'f32::from_bits(v.ret_f32("ScriptGame_ArenaBound")) }'),
        ext='    hook!(H_ScriptGame_ArenaBound, ScriptGame_ArenaBound, (a0: i32, a1: f32) -> f32);',
    ),
}
