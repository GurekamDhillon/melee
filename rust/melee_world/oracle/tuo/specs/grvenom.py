import os, sys, struct
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from common import *
TU = "src/melee/gr/grvenom.c"
RUST_MOD = "melee_world::tu::grvenom"
LOWMEM = True  # pointer slots overlap u32 views (Ground.u): pool objects below 4 GB, same addresses on both sides
SIZE_OK = ["HSD_JObj"]
SKIP_LEAF_TYPES = ["HSD_IDKey"]
RUST_TYPES = {"grVe_Data": "melee_world::tu::grvenom::grVe_Data",
              "grVenom_YakumonoParam": "melee_world::tu::grvenom::grVenom_YakumonoParam"}


def F(x):
    return "f:%08x" % struct.unpack("<I", struct.pack("<f", x))[0]


G_ = "p:HSD_GObj"
CASES = {
    "grVenom_8020362C": dict(n=800),
    "grVenom_80203B14": dict(n=4),
    "grVenom_80203B18": dict(n=60),
    "grVenom_80203DD0": dict(n=60),
    "grVenom_80203E80": dict(n=4),
    "grVenom_80203EA4": dict(n=4),
    "grVenom_80203EAC": dict(n=200),
    "stageGObj0_OnInit": dict(n=10, args=[G_]),
    "grVenom_80203FC4": dict(n=4, args=[G_]),
    "grVenom_80203FCC": dict(n=4, args=[G_]),
    "grVenom_80203FD0": dict(n=4, args=[G_]),
    "grVenom_80203FD4": dict(n=20, args=[G_]),
    "grVenom_802040A4": dict(n=4, args=[G_]),
    "grVenom_802040AC": dict(n=4, args=[G_]),
    "grVenom_802040B0": dict(n=4, args=[G_]),
    "fn_802040B4": dict(n=20, args=[G_]),
    "grVenom_802040F0": dict(n=60, args=[G_]),
    "grVenom_8020427C": dict(n=4, args=[G_]),
    "grVenom_80204284": dict(n=300, args=[G_]),
    "grVenom_80204424": dict(n=4, args=[G_]),
    "grVenom_80204428": dict(n=20, args=[G_]),
    "grVenom_80204544": dict(n=4, args=[G_]),
    "grVenom_8020454C": dict(n=300, args=[G_]),
    "grVenom_80204B84": dict(n=4, args=[G_]),
    "grVenom_80204B88": dict(n=200, args=[G_]),
    "grVenom_80204CE0": dict(n=4, args=[G_]),
    "grVenom_80204CE4": dict(n=4, args=[G_]),
    "grVenom_80204CEC": dict(n=200, args=[G_]),
    "grVenom_80204DB0": dict(n=4, args=[G_]),
    "grVenom_80204DD4": dict(n=20, args=[G_]),
    "grVenom_80204EF4": dict(n=4, args=[G_]),
    "stageGObj9_GObjProc": dict(n=10, args=[G_]),
    "grVenom_80204F1C": dict(n=4, args=[G_]),
    "grVenom_80204F20": dict(n=200, args=[G_]),
    "grVenom_802052D8": dict(n=4, args=[G_]),
    "grVenom_802052E0": dict(n=100, args=[G_, "p:Vec3"]),
    "grVenom_802053B0": dict(n=400, args=[G_]),
    "grVenom_802056AC": dict(n=4, args=[G_]),
    "grVenom_802056B0": dict(n=40, args=[G_]),
    "grVenom_80205750": dict(n=4, args=[G_]),
    "grVenom_80205758": dict(n=200, args=[G_]),
    "grVenom_80205AD0": dict(n=4, args=[G_]),
    "grVenom_80205AD4": dict(n=200, args=[G_]),
    "grVenom_80205DF0": dict(n=4, args=[G_]),
    "grVenom_80205DF8": dict(n=60, args=["p:Vec3"]),
    "grVenom_80205E84": dict(n=60, args=["p:Vec3"]),
    "grVenom_80205F30": dict(n=1500, args=[G_]),
    "grVenom_80206870": dict(n=4, args=[G_]),
    "grVenom_80206874": dict(n=200, args=[G_]),
    "grVenom_80206B68": dict(n=4, args=[G_]),
    "grVenom_80206B70": dict(n=100, args=[G_]),
    "grVenom_80206B90": dict(n=4, args=[G_]),
    "grVenom_80206B94": dict(n=20, args=[G_]),
    "grVenom_80206BBC": dict(n=4, args=[G_]),
    "grVenom_80206BC4": dict(n=20, args=[G_]),
    "grVenom_80206BEC": dict(n=4, args=[G_]),
    "grVenom_80206BF0": dict(n=60),
    "grVenom_80206CB0": dict(n=20),
    "grVenom_80206D10": dict(n=60),
    "grVenom_80206D74": dict(n=4),
    "grVenom_80206D7C": dict(n=100, args=["p:Vec3", None, "p:HSD_JObj"]),
}
CUSTOM_STUBS = {"Mex_GrBind": MEX_GRBIND, "HSD_ForeachAnim": HSD_FOREACHANIM}

def fl(*a):
    return ["grVenom_" + x if x[0] in "0123456789" else x for x in a]


NL = chr(10)
_pp = {
    "state = base[gp->u.venom.xC8 + 11];": "state = grVe_803E5348.arwing.arwing_type[gp->u.venom.xC8];",
    "data_idx = new_var[spawn_idx + 11];": "data_idx = grVe_803E5348.arwing.arwing_type[spawn_idx];",
    "if ((u32) ptr[8] == (u32) gobj) {": "if (grVe_803E5348.arwing.arwing_gobj[gp->u.venom.xC8] == gobj) {",
    "state = ptr[11];": "state = grVe_803E5348.arwing.arwing_type[gp->u.venom.xC8];",
    "base[gp->u.venom.xC8 + 8] = 0;": "grVe_803E5348.arwing.arwing_gobj[gp->u.venom.xC8] = 0;",
    "if ((u32) entry[8] != 0U) {": "if (grVe_803E5348.arwing.arwing_gobj[gp->u.venom.xC8] != 0) {",
    "if ((other = (HSD_GObj*) base[gp->u.venom.xC8 + 8]) != NULL) {": "if ((other = grVe_803E5348.arwing.arwing_gobj[gp->u.venom.xC8]) != NULL) {",
    "if ((far_other = (HSD_GObj*) base[gp->u.venom.xC8 + 8]) !=": "if ((far_other = grVe_803E5348.arwing.arwing_gobj[gp->u.venom.xC8]) !=",
    "type_idx = base[gp->u.venom.xC8 + 11];": "type_idx = grVe_803E5348.arwing.arwing_type[gp->u.venom.xC8];",
    "*(Vec3*) &sub->u.venom.xE0 = sp94;": "sub->u.arwing.xE0 = sp94;",
    "env_flags = *(u8*) &gp->u.venom2.xE0_state.xE0_state_pad;":
        "env_flags = (gp->u.venom2.xE0_state.b0 << 7) | (gp->u.venom2.xE0_state.b1 << 6) | (gp->u.venom2.xE0_state.b2 << 5) | (gp->u.venom2.xE0_state.b3 << 4) | (gp->u.venom2.xE0_state.b4 << 3) | (gp->u.venom2.xE0_state.b5 << 2) | (gp->u.venom2.xE0_state.b6 << 1);",
}
for _k, _o in enumerate(("xE0", "xE4", "xE8", "xEC")):
    _pp[f"if (*(u32*) &gp->u.venom.{_o} != 0U) {{"] = f"if (gp->u.starfox.article_gobjs[{_k}] != 0) {{"
    _pp[f"Ground_801C4A08(*(HSD_GObj**) &gp->u.venom.{_o});"] = f"Ground_801C4A08(gp->u.starfox.article_gobjs[{_k}]);"
# The port's native-layout relocations (tu/grvenom.rs), mirrored so the native C reads/writes the same words:
# - `u.venom.xE4` of the stage gobj (grVenom_80204428 / 8020454C): natively it would overlap the 8-byte `u.venom2`
#   JObj pointers, so the float lives right after `u.venom2.xE0_state` (tu/grvenom.rs venom2_xe4)
_XE4 = "(*(float*) ((char*) &gp->u.venom2.xE0_state + 4))"
_CRLF2 = chr(10) + chr(10) + "    "
_pp["gp->u.venom2.xE0_state.b6 = 0;" + _CRLF2 + "gp->u.venom.xE4 = 0.0F;"] = "gp->u.venom2.xE0_state.b6 = 0;" + _CRLF2 + _XE4 + " = 0.0F;"
_pp["gp->u.venom.xE4 <"] = _XE4 + " <"
_pp["gp->u.venom.xE4 >"] = _XE4 + " >"
_pp["gp->u.venom.xE4 = frame;"] = _XE4 + " = frame;"
# - the arwing counters `u.venom.xF0..x100` are the GC words of `u.starfox.xF0..x100` (grVenom_80205F30 reads them
#   through that view; natively the two views' offsets differ): the port uses the starfox view throughout
for _f in ("xF0", "xF4", "xF8", "xFC", "x100"):
    _pp[f"gp->u.venom.{_f}"] = f"gp->u.starfox.{_f}"
PATCH = list(_pp.items())
PORTED = {"Stage_80225194": "crate::tu::stage::Stage_80225194",
          "Stage_GetBlastZoneRightOffset": "crate::tu::stage::Stage_GetBlastZoneRightOffset",
          "Stage_GetBlastZoneLeftOffset": "crate::tu::stage::Stage_GetBlastZoneLeftOffset",
          "Stage_GetBlastZoneTopOffset": "crate::tu::stage::Stage_GetBlastZoneTopOffset",
          "Stage_GetBlastZoneBottomOffset": "crate::tu::stage::Stage_GetBlastZoneBottomOffset"}
FMT_VARIADIC = ["OSReport"]
EXTERN_ARRAY_LEN = {}
PTR_BYTES = {n: {0: 4} for n in ("Ground_801C052C", "Ground_801C05D4", "Ground_801C0544", "Ground_801C058C",
                                  "Ground_801C05EC", "Ground_801C05A4", "Ground_801C055C", "Ground_801C05BC", "Ground_801C0574")}
PTR_BYTES["lb_80011A50"] = {0: 12}
# b7 of xE0_state shares its byte with `xE0_state_pad.state` (80204B88 writes it, and 8020454C calls that): native bit order
# differs from the GameCube order there, so the bit is not compared (no function reads it as b7)
SKIP_BITS = {"types.h:570:9": ["b7"], "types.h:565:9": ["padding", "padding2"]}  # the anonymous b0..b7 struct of grVenom_GroundVars2.xE0_state
# `Ground_ResetStarFoxArwingState` clears `starfox.xC4.flags.b0`, which shares its byte with the low byte of the pointer views
# venom2.xC4 and venom_platform.target_gobj: b0 is bit 7 on the GameCube (and in melee_types), bit 0 in the native C, so those two
# pointers are compared without bits 0 and 7 of their low byte (see gen_tuo.py PTR_MASK)
PTR_MASK = {"grVenom_GroundVars2": ["xC4"], "grVenom_Platform_GroundVars": ["target_gobj"]}
FORCE_FILL = ["grVe_803E5348", "grVe_803E5380"]
_V = fl("80204F20", "802052E0", "802053B0", "802056B0", "80205758", "80205AD4", "80205F30", "80206874", "80206BF0", "80206B70")
UNION_VIEW = {
    "Ground.u": {"venom": _V,
                 "venom2": fl("8020362C", "80204428", "8020454C", "80204B88", "80203B18"),
                 "venom_platform": fl("80203B18", "fn_802040B4", "802040F0", "80204284", "80206D7C"),
                 "starfox": fl("80204CEC")},
    "grVenom_GroundVars.anon0": {"xC4": [x for x in _V if not x.endswith(("80205AD4", "80206874", "80204F20"))], "xC4_flags": fl("80205AD4", "80206874", "80204F20")},
    "grVenom_GroundVars.anon1": {"xDC": [x for x in _V if not x.endswith(("80205F30", "80206B70", "80205AD4", "80206874", "80204F20"))], "linked_gobj": fl("80205F30", "80206B70", "80205AD4", "80206874", "80204F20")},
    "grVenom_GroundVars2.xE0_state": {"anon:anon_0": ["~grVenom_8020362C"] + fl("80204428", "8020454C", "80204B88", "80203B18"),
                                      "xE0_state_pad": fl("80204B88")},
    "grStarFox_GroundVars.xC4": {"flags": fl("80205AD4", "80206874"), "word": fl("80204CEC")},
}
_f = lambda *v: [F(x) for x in v]
DOM = {
    ".classifier": [0xC, 0xC, 1, 0xB],
    "ret:Ground_801C0498": _f(0.5, 1.0, 2.0),
    "ret:HSD_Randi": [0, 1, 2, 3, 4, 5, 7, 9, 10],
    "ret:HSD_Randf": _f(0.0, 0.25, 0.5, 0.75, 1.0),
    "ret:Stage_80225194": [0xE4, 0xE9, 0xE5, 0],
    "ret:Player_GetPlayerSlotType": [0, 1, 3, 3],
    "ret:mpJointFromLine": [0, 3, 4, 5, 7],
    "ret:grAnime_801C84A4": [0, 1],
    "ret:grAnime_801C83D0": [0, 1],
    "ret:grCorneria_801E2598": [0, 1],
    "ret:HSD_JObjGetFlags": [0, 0x10, 0x4010],
    "ret:Stage_GetBlastZoneRightOffset": _f(100.0, 200.0),
    "ret:Stage_GetBlastZoneLeftOffset": _f(-200.0, -100.0),
    "ret:Stage_GetBlastZoneTopOffset": _f(100.0, 200.0),
    "ret:Stage_GetBlastZoneBottomOffset": _f(-200.0, -100.0),
    "out:lb_8000B1CC#2[0]": _f(-300.0, -50.0, 0.0, 50.0, 300.0),
    "out:lb_8000B1CC#2[1]": _f(-300.0, -50.0, 0.0, 50.0, 300.0),
    "out:lb_8000B1CC#2[2]": _f(-70000.0, -59000.0, -3000.0, -1500.0, -50.0, 0.0, 500.0, 1500.0),
    "out:Player_LoadPlayerCoords#1[0]": _f(-30.0, -5.0, 5.0, 30.0),
    "ret:grAnime_801C8318.curr_frame": _f(0.0, 1033.0, 1100.0, 4500.0, 7300.0, 9300.0, 9900.0),
    "g:yakumono_param": [],
    ".x0": _f(0.0, 5.0, 10.0, 30.0),
    ".x4": _f(0.0, 5.0, 10.0, 30.0),
    ".x8": _f(0.0, 5.0, 10.0, 30.0),
    ".xC": _f(0.0, 5.0, 10.0, 30.0),
    ".x10": _f(0.0, 0.25, 0.5, 1.0),
    ".x2C": _f(0.0, 3.0, 10.0),
    ".x34": _f(0.5, 1.0, 2.0),
    "g:grVe_804D6A34": [0, 1, 2],
    "g:grVe_804D6A38": [-1, 0, 1, 2, 5, 100],
    "g:grVe_804D6A3C": [0, 0, 1],
    "g:grVe_804D6A40": [0, 1],
    ".arwing_type[0]": list(range(12)), ".arwing_type[1]": list(range(12)), ".arwing_type[2]": list(range(12)),
    "g:grVe_803E5380[0]": [0, 1, 2, 3, 4], "g:grVe_803E5380[1]": [0, 1, 2, 3, 4], "g:grVe_803E5380[2]": [0, 1, 2, 3, 4],
    ".xC8": [0, 1, 2],
    ".xF4": [0, 0, 1, 2, 3, 4, 5, -1],
    ".xF8": [-1, 0, 1, 5],
    ".xF0": [0, 1, 8, 0x3D, 0x50, 0x70, 0xE0, 0x100, 0x101],
    ".xFC": [0, 1],
    ".x100": [0, 1],
    ".xD4": [0, 1],
    ".xD8": [0, 1, 2, 3],
    ".smash_taunt_timer": [-1, 0, 1, 0x3B, 0x3C, 0x3D],
    ".state": [0, 1, 2],
    "arg:grVenom_80203EAC#0": list(range(16)),
    "arg:grVenom_80206BF0#0": [2, 20, 0, 5],
    "arg:grVenom_80206CB0#0": [0, 1],
    "arg:grVenom_80206D10#0": [-1, 0, 1, 5],
    "ret:Ground_GetMapGObj": [],
}
del DOM["g:yakumono_param"]; del DOM["ret:Ground_GetMapGObj"]
EXTERN_ARRAY_LEN = {}
LINKS = {
    "g:yakumono_param": "new:struct~grVenom_YakumonoParam",
    "ret:Ground_GetYakumonoParam": "new:struct~grVenom_YakumonoParam",
    ".user_data": "new:Ground",
    "ret:Ground_GetMapGObj": "maybe:HSD_GObj",
    "ret:Ground_GetStageGObj": "maybe:HSD_GObj",
    "ret:Ground_801C498C": "maybe:HSD_GObj",
    "ret:Ground_801C3FA4": "new:HSD_JObj",
    "ret:grAnime_801C8318": "maybe:HSD_AObj",
    "ret:grZakoGenerator_801CAE04": "null",
    "ret:it_802E7654": "null",
    "ret:lb_80011A50": "null",
    "ret:HSD_GObj_SetupProc": "null",
    ".hsd_obj": "new:HSD_LObj",
    ".aobj": "maybe:HSD_AObj",
    "g:HSD_GObjPLinkHead": "ptrs:HSD_GObj:8",
    ".linked_gobj": "maybe:HSD_GObj",
    ".target_gobj": "shallow:HSD_GObj",
    ".upper_jobj": "new:HSD_JObj",
    ".lower_jobj": "maybe:HSD_JObj",
    ".xC4": "new:HSD_JObj",
}
LINK_FORCE = {"ret:Ground_GetMapGObj": fl("8020362C", "80204284"), "ret:Ground_GetStageGObj": fl("80203B18", "80206BF0", "80204F20", "80204CEC"),
              "ret:Ground_801C498C": fl("80203B18", "8020454C")}
CUSTOM_STUBS = {"Mex_GrBind": MEX_GRBIND, "HSD_ForeachAnim": HSD_FOREACHANIM}

# smash_taunt_timer (s32 at u+8) aliases the 8-byte xC8 pointer of venom2 natively: only the platform functions see it as a timer
FIELD_FNS = {"grVenom_Platform_GroundVars.smash_taunt_timer": fl("fn_802040B4", "802040F0", "80204284", "80206D7C")}
