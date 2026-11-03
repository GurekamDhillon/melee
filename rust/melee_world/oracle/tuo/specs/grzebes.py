import os, sys, struct
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from common import *
TU = "src/melee/gr/grzebes.c"
RUST_MOD = "melee_world::tu::grzebes"
SIZE_OK = ["HSD_JObj"]
SKIP_LEAF_TYPES = ["HSD_IDKey"]
LOWMEM = True  # the C keeps pointers in u32 slots: every pool object lives below 4 GB
_T = "melee_world::tu::grzebes::"
RUST_TYPES = {n: _T + n for n in ("grZe_ColorEntry", "grZe_AcidLevelEntry", "grZe_YakumonoParam", "grZe_BubbleEntry",
                                  "grZe_BubbleScales", "grZe_BubbleConfig", "grZe_AcidState")}


def F(x):
    return "f:%08x" % struct.unpack("<I", struct.pack("<f", x))[0]


_f = lambda *v: [F(x) for x in v]
import random as _r
_rng = _r.Random(617)
def RND(lo, hi, n=24):
    """n seeded pseudo-random floats in [lo, hi] (generic mantissas keep fused multiply-adds discriminating)"""
    return [F(_rng.uniform(lo, hi)) for _ in range(n)]
Z = lambda n: "grZebes_" + n if n[0] in "0123456789" else n
_zf = lambda *a: [Z(x) for x in a]

G_ = "p:HSD_GObj"
CASES = {
    "grZebes_801D84A0": dict(n=2),
    "grZebes_801D84A4": dict(n=40),
    "grZebes_801D8528": dict(n=2),
    "grZebes_801D852C": dict(n=2),
    "grZebes_801D8550": dict(n=2),
    "grZebes_801D8558": dict(n=60, args=["i32s"]),
    "grZebes_801D8644": dict(n=40, args=[G_]),
    "grZebes_801D8814": dict(n=2, args=[G_]),
    "grZebes_801D881C": dict(n=800, args=[G_]),
    "grZebes_801D90FC": dict(n=2, args=[G_]),
    "grZebes_801D9100": dict(n=40, args=[G_]),
    "grZebes_801D9254": dict(n=2, args=[G_]),
    "grZebes_801D925C": dict(n=200, args=[G_]),
    "grZebes_801D93D8": dict(n=2, args=[G_]),
    "stageGObj5_OnInit": dict(n=10, args=[G_]),
    "grZebes_801D9408": dict(n=2, args=[G_]),
    "grZebes_801D9410": dict(n=60, args=[G_]),
    "grZebes_801D94EC": dict(n=2, args=[G_]),
    "fn_801D94F0": dict(n=10, args=[G_]),
    "grZebes_801D9508": dict(n=40, args=[G_]),
    "grZebes_801D95B0": dict(n=2, args=[G_]),
    "grZebes_801D95B8": dict(n=200, args=[G_]),
    "grZebes_801D9754": dict(n=2, args=[G_]),
    "grZebes_801D9758": dict(n=10, args=[G_]),
    "grZebes_801D9798": dict(n=60, args=[G_]),
    "grZebes_801D99D8": dict(n=2, args=[G_]),
    "grZebes_801D99E0": dict(n=1500, args=[G_]),
    "grZebes_801D9F2C": dict(n=2, args=[G_]),
    "grZebes_801D9F30": dict(n=10, args=[G_]),
    "grZebes_801D9F7C": dict(n=2, args=[G_]),
    "grZebes_801D9F84": dict(n=100, args=[G_]),
    "grZebes_801DA0C0": dict(n=2, args=[G_]),
    "grZebes_801DA0C4": dict(n=400, args=["f32"]),
    "grZebes_801DA254": dict(n=200, args=[G_, "f32"]),
    "grZebes_801DA3E8": dict(n=2),
    "grZebes_801DA3F4": dict(n=60, args=["p:Vec3"]),
    "grZebes_801DA4FC": dict(n=10, args=[G_]),
    "grZebes_801DA528": dict(n=800, args=[G_, "p:grZe_AcidState", "i32s", "i32s"]),
    "fn_801DA9D8": dict(n=20, args=[G_, "p:Ground", "p:Vec3", G_, "f32"]),
    "fn_801DA9F0": dict(n=20, args=[G_, "p:Ground", "p:Vec3", G_, "f32"]),
    "grZebes_801DAA08": dict(n=200),
    "fn_801DAC90": dict(n=100, args=[G_, "p:Ground", "p:Vec3", G_, "f32"]),
    "grZebes_801DAE70": dict(n=300, args=["i32s", "i32u", "f32", "f32", "f32"]),
    "grZebes_801DB088": dict(n=600, args=["p:Ground", "i32s"]),
    "grZebes_801DB3CC": dict(n=600, args=[G_]),
    "grZebes_801DBB60": dict(n=800, args=[G_]),
    "grZebes_801DC260": dict(n=100),
    "grZebes_801DC408": dict(n=600, args=[G_]),
    "grZebes_801DC744": dict(n=200, args=["i32s", "i32u"]),
    "grZebes_801DC9DC": dict(n=40, args=["i32s"]),
    "grZebes_801DCB64": dict(n=20, args=["p:Vec3", "i32s"]),
    "grZebes_801DCBB0": dict(n=20, args=["p:Vec3", "i32s"]),
    "grZebes_801DCBFC": dict(n=100, args=[G_, G_, "cell:64"]),
    "grZebes_801DCCB8": dict(n=2, args=["i32s"]),
    "grZebes_801DCCC0": dict(n=2, args=["p:Vec3", "i32s", "p:HSD_JObj"]),
    "grZebes_801DCCC8": dict(n=20),
}
FMT_VARIADIC = ["OSReport"]
OUT_F32S = {"Ground_801C4368": {0: 1, 1: 1}, "lb_8000B1CC": {2: 3}, "ftLib_80086644": {1: 3}, "ftLib_80086684": {1: 3}}
OUT_INTS = {"Ground_801C2090": {0: 32}}
PTR_BYTES = {"Ground_801C205C": {0: 4}}
EXTERN_ARRAY_LEN = {"HSD_GObjGXLinkHead": 8}
PATCH = [
    # the port's native layout (tu/grzebes.rs, debda3c36): the +0x2C word is a native pointer to the acid hit record,
    # which the C reads as `((HSD_GObj*) yakumono_param)->user_data` (GC offset 0x2C)
    ("/* 0x14 */ u8 pad_14[0x30 - 0x14];", "/* 0x14 */ u8 pad_14[0x2C - 0x14]; /* 0x2C */ void* x2C_dyn;"),
    ("*(void**) arg = ((HSD_GObj*) yakumono_param)->user_data;", "*(void**) arg = yakumono_param->x2C_dyn;"),
    ("/* +14 */ HSD_JObj* x14_jobj1;", "/* +14 */ u32 x14_jobj1;"),
    ("/* +18 */ HSD_JObj* x18_jobj2;", "/* +18 */ u32 x18_jobj2;"),
    ("/* +1C */ Item_GObj* x1C_mat;", "/* +1C */ u32 x1C_mat;"),
    ("->x14_jobj1 = child_jobj;", "->x14_jobj1 = (u32) (uintptr_t) child_jobj;"),
    ("->x18_jobj2 =", "->x18_jobj2 = (u32) (uintptr_t)"),
    ("->x1C_mat = mat_gobj2 = mat_gobj;", "->x1C_mat = (u32) (uintptr_t) (mat_gobj2 = mat_gobj);"),
    ("grMaterial_801C8E08(st->x1C_mat);", "grMaterial_801C8E08((Item_GObj*) (uintptr_t) st->x1C_mat);"),
    ("grMaterial_801C8E28((HSD_GObj*) st->x1C_mat);", "grMaterial_801C8E28((HSD_GObj*) (uintptr_t) st->x1C_mat);"),
    ("HSD_JObjSetTranslateX(st->x14_jobj1,", "HSD_JObjSetTranslateX((HSD_JObj*) (uintptr_t) st->x14_jobj1,"),
    ("HSD_JObjSetTranslateX(st->x18_jobj2,", "HSD_JObjSetTranslateX((HSD_JObj*) (uintptr_t) st->x18_jobj2,"),
    # the port reads the GC bytes +0x6C/+0x70/+0x74 of `unk4->unk8` as the typed `unk8[2].unk4/unk8/unkC` (UnkStageDat_x8_t
    # is 0x34 bytes; tu/grzebes.rs, 303222ac9): the native C does the same so both walk the same native records
    ("*(HSD_ShapeAnimJoint***) (dat + 0x74)", "((struct UnkStageDat_x8_t*) dat)[2].unkC"),
    ("*(HSD_AnimJoint***) (dat + 0x6C)", "((struct UnkStageDat_x8_t*) dat)[2].unk4"),
    ("*(HSD_MatAnimJoint***) (dat + 0x70)", "((struct UnkStageDat_x8_t*) dat)[2].unk8"),
]
DOM = {
    "arg:grZebes_801D8558#0": list(range(12)),
    "arg:grZebes_801DAE70#0": list(range(20)),
    "arg:grZebes_801DAE70#1": [0, 1, 2, 3, 4],
    "arg:grZebes_801DAE70#2": _f(-13.37, -7.123, 0.25, 3.14159, 9.81),
    "arg:grZebes_801DAE70#3": _f(-12.5, -2.7, 1.77, 6.02, 20.1),
    "arg:grZebes_801DAE70#4": _f(0.7, 1.0, 1.1, 1.37, 1.9),
    "arg:grZebes_801DB088#1": list(range(20)),
    "arg:grZebes_801DC744#0": [0, 1, 2, 3, 3],
    "arg:grZebes_801DC744#1": [0, 1, 4],
    "arg:grZebes_801DCB64#1": [0, 5, 8, 9, 20],
    "arg:grZebes_801DCBB0#1": [0, 2, 3, 20],
    "arg:grZebes_801DA0C4#0": _f(-400.0, -320.0, -80.0, 90.0, 120.0) + RND(-320.0, 90.0, 40),
    "arg:grZebes_801DA254#1": RND(-300.0, 120.0, 40),
    "ret:Ground_801C0498": _f(0.5, 1.0, 2.0),
    "ret:HSD_Randi": [0, 1, 2, 3, 4, 5, 7, 9, 10, 19, 20, 59, 100, 599],
    "ret:HSD_Randf": _f(0.0, 1.0) + RND(0.0, 1.0, 30),
    "ret:grAnime_801C84A4": [0, 1],
    "ret:grAnime_801C83D0": [0, 1],
    "ret:Ground_801C2090": [0, 1, 1],
    "out:Ground_801C2090#0": [0x00FFEE11, 0x7F3A19C2, 0x0A0B0C0D],
    "out:Ground_801C4368#0[0]": _f(-30.0, -10.0, -1.0, 0.0, 3.0, 6.0, 20.0),
    "out:Ground_801C4368#1[0]": _f(0.0, 1.0, 5.0),
    "out:lb_8000B1CC#2[0]": _f(-60.3, -5.1, 0.0, 5.7, 60.9),
    "out:lb_8000B1CC#2[1]": _f(-60.3, -5.1, 0.0, 5.7, 60.9),
    "out:lb_8000B1CC#2[2]": _f(-5.0, 0.0, 5.0),
    "out:ftLib_80086644#1[0]": _f(-30.0, 0.0, 30.0),
    "out:ftLib_80086644#1[1]": _f(-30.0, -3.0, 0.0, 8.0, 30.0),
    "out:ftLib_80086684#1[1]": _f(-3.0, 0.0, 8.0),
    # grZe_YakumonoParam
    ".x00": _f(1.5, 3.3, 10.0), ".x04": _f(4.4, 12.1, 30.0), ".x08": _f(2.0, 5.5, 40.0), ".x0C": _f(8.0, 21.7, 60.0),
    ".x10": [1, 2, 3, 5, 10],
    ".x30": _f(0.9, 2.1, 5.0), ".x34": _f(0.05, 0.31, 1.0), ".x38": _f(0.01, 0.07, 0.2), ".x3C": RND(0.0, 0.33, 8),
    ".x40": _f(0.001, 0.01, 0.2), ".x44": RND(0.05, 1.1, 8), ".x48": _f(0.0, 5.0, 60.0, 400.0), ".x4C": _f(1.0, 3.5, 6.0),
    ".x50": RND(10.0, 90.0), ".x54": RND(95.0, 130.0), ".x58": _f(0.8, 1.2), ".x5C": _f(1.3, 1.7, 2.2),
    ".x60": RND(10.0, 100.0, 8), ".x64": RND(20.0, 150.0, 8), ".x68": _f(0.01, 0.05, 0.12),
    ".x6C": RND(3.0, 6.2, 8), ".x70": RND(0.6, 2.4, 8), ".x74": RND(0.9, 2.8, 8), ".x78": _f(0.01, 0.05, 0.4),
    ".x7C": _f(0.0, 1.0), ".x80": _f(0.1, 0.5, 1.5), ".x84": _f(0.005, 0.02, 0.1), ".x88": _f(0.1, 0.4, 1.0),
    ".x8C": RND(150.0, 400.0, 6), ".x90": RND(-400.0, -300.0, 6), ".x94": RND(5.0, 60.0),
    ".x98": RND(0.4, 3.0), ".x9C": RND(0.01, 0.31),
    ".x0_base": [0, 30, 100, 600], ".x2_delay_min": [0, 10, 50, 90], ".x4_delay_max": [0, 10, 50, 120],
    ".x6_level": [0, 0, -250, -100, 20, 50],
    # grZe_BubbleEntry
    ".x00_active": [0, 0, 0, 1, 1, 2, 3, 4],
    "g:grZe_804D6994": [0, 0, 0, 5],
    ".x02_timer": [-2, -1, 0, 5, 30],
    ".x08_x": _f(-13.37, -7.123, -2.5, 0.25, 1.77, 3.14159, 6.02, 9.81, 12.5, 20.1),
    ".x0C_y": _f(-12.5, -6.4, -2.7, 0.9, 1.77, 4.2, 7.7, 11.1),
    ".x14": _f(-0.3, -0.05, 0.0, 0.07, 0.4),
    ".x18_size": _f(0.5, 0.8, 1.0, 1.1, 1.2, 1.37),
    ".x1C": _f(0.5, 0.8, 1.0, 1.1, 1.3, 1.9),
    # grZe_AcidState
    ".x00_state": [0, 1, 2, 3, 0xFF], ".x01_next": [0, 1, 2, 3], ".x04_base_x": _f(-3.1, 0.0, 7.7),
    ".x08_offset": _f(-3.1, -0.4, 0.0, 0.6, 2.9), ".x0C_velocity": _f(-0.3, -0.01, 0.0, 0.02, 0.4),
    ".x10_damage": _f(0.0, 1.5, 3.7, 6.2), ".x20_anim_idx": [0x1C, 0xD, 6, 1],
    # Ground.u overlays
    ".u.zebes4.xC4": [0, 1, 2, 3, 0xFF], ".u.zebes4.xC5": [0, 1, 2, 3], ".u.zebes4.xC6": [0, 1, 5, 100, 0xFFFF],
    ".u.zebes4.xC8": _f(-3.1, 0.0, 7.7), ".u.zebes4.xCC": _f(-3.1, -0.4, 0.0, 0.6, 2.9),
    ".u.zebes4.xD0": _f(-0.3, 0.0, 0.4), ".u.zebes4.xD4": _f(0.0, 1.5, 3.7, 6.2),
    ".u.zebes4.xE4": [0xD, 6, 1], ".u.zebes4.xE8": [0, 1, 2, 3],
    ".u.zebes5.xC4": [0, 1, 2, 3, 5, 28, 29],
    ".u.zebes5.xC6": [-2, -1, 0, 1, 5, 0x77, 0x78, 0x79, 59, 60, 100],
    ".u.zebes5.xCC": _f(-5.0, 0.0, 1.3, 999.0, 0.4), ".u.zebes5.xD0": _f(0.0, 0.1, 0.35, 1.0),
    ".u.zebes5.xD4": _f(-250.5, -120.3, 0.0, 33.3), ".u.zebes5.xD8": _f(-250.5, -120.3, -55.5, 0.0, 33.3),
    ".u.zebes5.xEC": [0, 1, 2, 3], ".u.zebes5.xE8": [0x1C],
    ".u.zebes5.xF4": [-1, 0, 1, 5], ".u.zebes5.xF6": [0, 1, 2, 5, 6, 10, 11, 12],
    ".u.zebes5.xF8": [0, 1, 5, 0xFFFF, 0xFFFE],
    ".u.zebes.x8": [-1, 0, 1], ".u.zebes.xA": [0, 1, 2, 3000, 3300],
    ".u.zebes2.xC4": [0, 1],
    ".u.zebes3.xC8": list(range(20)),
    ".u.zebes5.xC8": [s | (n << 8) | t for s in range(5) for n in range(4) for t in (0, 0xFFFF0000, 0x00050000)] + [0, 1, 2, 3, 4] * 12,
}
LINKS = {
    "g:yakumono_param": "new:grZe_YakumonoParam",
    "ret:Ground_GetYakumonoParam": "new:grZe_YakumonoParam",
    ".x14_jobj1": "int:new:HSD_JObj", ".x18_jobj2": "int:new:HSD_JObj", ".x1C_mat": "int:new:HSD_JObj",
    ".u.zebes5.xDC": "int:new:HSD_JObj",
    "u.zebes5.xDC": "int:maybe:HSD_JObj",
    ".u.zebes5.xE0": "int:new:HSD_JObj", ".u.zebes5.xE4": "int:new:HSD_JObj", ".u.zebes5.xF0": "int:new:HSD_JObj",
    ".u.zebes5.xFC": "int:new:HSD_JObj", ".u.zebes5.x100": "int:new:HSD_JObj",
    ".u.zebes4.xD8": "int:new:HSD_JObj", ".u.zebes4.xE0": "int:new:HSD_JObj", ".u.zebes4.xEC": "int:new:HSD_JObj",
    ".u.zebes4.xDC": "int:maybe:HSD_JObj",
    ".u.zebes.x4": "int:maybe:HSD_JObj",
    ".x20_gobj": "new:HSD_GObj",
    "x20_gobj": "maybe:HSD_GObj",
    ".user_data": "new:Ground",
    ".hsd_obj": "new:HSD_JObj",
    "ret:Ground_GetStageGObj": "maybe:HSD_GObj",
    "ret:Ground_GetMapGObj": "new:HSD_GObj",
    "ret:Ground_801C3FA4": "new:HSD_JObj",
    "ret:Ground_801C13D0": "maybe:HSD_JObj",
    "ret:grMaterial_801C8CFC": "new:HSD_GObj",
    "ret:grMaterial_801C8D44": "new:HSD_GObj",
    "ret:grZakoGenerator_801CA394": "new:HSD_GObj",
    "ret:grDatFiles_801C6330": "new:UnkArchiveStruct",
    ".unk4": "new:UnkStageDat",
    # unk4->unk8 as the native UnkStageDat_x8_t records the port reads (0x68 bytes each): record [2]'s anim / matanim /
    # shapeanim list pointers at native +8 / +0x10 / +0x18
    ".unk8": "datblkp:312:216:224:232",
    "g:HSD_GObjGXLinkHead": "ptrs:HSD_GObj:8",
    ".next": "maybe:HSD_LObj",
    ".x04": "maybe:HSD_JObj",
    ".xC4": "maybe:HSD_GObj",
}
LINK_GATE = {
    ".u.zebes5.xDC": [Z("801D881C")],
    "u.zebes5.xDC": [Z("801D99E0")],
    ".u.zebes5.xE0": [Z("801D881C")], ".u.zebes5.xE4": [Z("801D881C")], ".u.zebes5.xF0": [Z("801D881C")],
    ".u.zebes5.xFC": [Z("801D881C")], ".u.zebes5.x100": [Z("801D881C")],
    ".u.zebes4.xD8": [Z("801D925C")], ".u.zebes4.xE0": [Z("801D925C")], ".u.zebes4.xEC": [Z("801D925C")],
    ".u.zebes4.xDC": [Z("801D925C"), Z("801DA254")],
    ".u.zebes.x4": [Z("801D95B8")],
    ".x14_jobj1": [Z("801DA528")], ".x18_jobj2": [Z("801DA528")], ".x1C_mat": [Z("801DA528")],
    ".x20_gobj": _zf("801DAA08", "801D881C", "801DC260", "801DB3CC"),
}
LINK_FORCE = {".u.zebes4.xDC": [Z("801D925C")], ".u.zebes.x4": [Z("801D95B8")]}
# zebes.x0_b0 is bit 0 of the byte that zebes2/zebes5.xC4 hold: native and GameCube bit order differ, so xC4 is not
# compared where the id-4 init (801D9508) runs next to the others (801D84A4, 801D8558)
FIELD_FNS = {"grZebes_GroundVars.x0_b0": _zf("801D9508", "801D95B8", "fn_801D94F0"),
             "grZebes_GroundVars5.xC4": _zf("801D8644", "801D881C", "801D9798", "801D99E0", "801DC408"),
             "grZebes_GroundVars2.xC4": _zf("801D9F30", "801D9F84")}
UNION_VIEW = {"Ground.u": {
    "zebes": _zf("801D9508", "801D95B8", "801D9758", "fn_801D94F0", "fn_801DA9D8", "fn_801DA9F0", "801D84A4", "801D8558"),
    "zebes2": _zf("801D9F30", "801D9F84", "801D84A4", "801D8558"),
    "zebes3": _zf("801DAE70", "801DB088", "801DB3CC", "801DC9DC", "801DC408", "801DC744", "801D881C", "801D8644"),
    "zebes4": _zf("801D9100", "801D925C", "801DA254", "801D8644"),
    # (grZebes_801D8558 runs grZebes_801D9508, which sets the `u.zebes.x0_b0` bit the port keeps in GC bit order (bit 7):
    #  the zebes4 byte view of +0xC4 would compare that aliased byte, so 801D8558 dumps through the zebes view only)
    "zebes5": _zf("801D8644", "801D881C", "801D9798", "801D99E0", "801DC408", "801D84A4", "801D8558"),
}}
# HSD_Randi(n) is in [0, n): the generated value is reduced modulo n
HSD_RANDI = {
    "c": ('int HSD_Randi(int a0) { tuo_call_begin("HSD_Randi"); tuo_arg_i(a0); tuo_call_end(); '
          'long long r = (long long) tuo_ret_int("HSD_Randi", 32, 1); if (a0 <= 0) return 0; return (int) (((r % a0) + a0) % a0); }'),
    "rust": ('unsafe extern "C" fn s_HSD_Randi(a0: i32) -> i32 { let v = vv(); v.call_begin("HSD_Randi"); v.arg_i(a0 as i64); v.call_end(); '
             'let r = v.ret_int("HSD_Randi", 32, 1) as i32 as i64; if a0 <= 0 { return 0; } (((r % a0 as i64) + a0 as i64) % a0 as i64) as i32 }'),
    "ext": '    hook!(H_HSD_Randi, HSD_Randi, (a0: i32) -> i32);',
}
CUSTOM_STUBS = {"Mex_GrBind": MEX_GRBIND, "HSD_Randi": HSD_RANDI}
# the MSL sqrtf calls __frsqrte: the real Gekko estimate, not a logging stub
SKIP_STUBS = ["__frsqrte"]
EXTRA_C = ["pc/gameworld/gekko_fp.c"]
