"""Spec of the camera oracle (cm/camera.c, ported as tu::camera); driven by gen/mtu/mk_cm.py."""
import os

SPECS = {}
# MTU_PATCHFILE=<file>: debugging only; blocks "old\n=====\nnew\n#####\n" of C text replaced in the TU
DBG_PATCH = []
if os.environ.get("MTU_PATCHFILE"):
    for blk in open(os.environ["MTU_PATCHFILE"], encoding="utf-8").read().split("#####\n"):
        if blk.strip():
            o, n = blk.split("=====\n")
            DBG_PATCH.append((o, n))

VEC_C = "if (%s) { %s->x = RNN(); %s->y = RNN(); %s->z = RNN(); }"
VEC_R = "if !%s.is_null() { (*%s).x = super::rnn(); (*%s).y = super::rnn(); (*%s).z = super::rnn(); }"


def vec_out(a):
    return (VEC_C % (a, a, a, a), VEC_R % (a, a, a, a))


FLOAT_GETTERS = [
    "Stage_GetBlastZoneBottomOffset", "Stage_GetBlastZoneLeftOffset", "Stage_GetBlastZoneRightOffset", "Stage_GetBlastZoneTopOffset",
    "Stage_GetCamAngleRadiansDown", "Stage_GetCamAngleRadiansLeft", "Stage_GetCamAngleRadiansRight", "Stage_GetCamAngleRadiansUp",
    "Stage_GetCamBoundsBottomOffset", "Stage_GetCamBoundsLeftOffset", "Stage_GetCamBoundsRightOffset", "Stage_GetCamBoundsTopOffset",
    "Stage_GetCamFixedFov", "Stage_GetCamInfoX20", "Stage_GetCamInfoX24", "Stage_GetCamMaxDepth", "Stage_GetCamPanAngleRadians",
    "Stage_GetCamTrackRatio", "Stage_GetCamTrackSmooth", "Stage_GetCamZoomRate", "Stage_GetPauseCamZPosInit", "Stage_GetPauseCamZPosMax",
    "Stage_GetPauseCamZPosMin", "grCastle_801D0FF0", "grCorneria_801E2FCC", "grGarden_80203624", "grHomeRun_8021EF10",
    "grKinokoRoute_802087B0", "grZebes_801DCCC8", "HSD_CObjGetAspect", "HSD_CObjGetAuthoredAspect", "HSD_CObjGetFar", "HSD_CObjGetFov",
]

RET = {
    "HSD_MemAlloc": (
        "((a0 > 0 && a0 % (int) sizeof(CmSubject) == 0 && a0 / (int) sizeof(CmSubject) <= 8) ? (void*) SUBJ : (void*) 0)",
        "if a0 > 0 && a0 % core::mem::size_of::<CmSubject>() as i32 == 0 && a0 / core::mem::size_of::<CmSubject>() as i32 <= 8 { &raw mut super::SUBJ as *mut c_void } else { core::ptr::null_mut() }"),
    "GObj_Create": ("&GO[fv() % 3]", "&raw mut super::GO[(fv() % 3) as usize]"),
    "Ground_GetP1Fighter": ("((fv() & 3) ? &GO[2] : 0)", "if fv() & 3 != 0 { &raw mut super::GO[2] } else { core::ptr::null_mut() }"),
    "HSD_CObjGetCurrent": ("&CO[fv() % 3]", "&raw mut super::CO[(fv() % 3) as usize]"),
    "HSD_CObjGetViewingMtxPtr": ("(float(*)[4]) MTXA", "&raw mut super::MTXA as *mut [f32; 4]"),
    "Player_GetEntity": ("((fv() & 3) ? &GO[1 + fv() % 2] : 0)", "if fv() & 3 != 0 { &raw mut super::GO[1 + (fv() % 2) as usize] } else { core::ptr::null_mut() }"),
    "ftLib_80086B74": ("((fv() & 3) ? &SUBJ[fv() % 8] : 0)", "if fv() & 3 != 0 { &raw mut super::SUBJ[(fv() % 8) as usize] } else { core::ptr::null_mut() }"),
    "grLib_801C9CEC": ("((fv() & 1) ? &GO[2] : 0)", "if fv() & 1 != 0 { &raw mut super::GO[2] } else { core::ptr::null_mut() }"),
    "lb_80013B14": ("&CO[fv() % 3]", "&raw mut super::CO[(fv() % 3) as usize]"),
    "lbVector_WorldToScreen": ("((fv() & 3) ? a2 : 0)", "if fv() & 3 != 0 { a2 } else { core::ptr::null_mut() }"),
    "Player_GetPlayerId": ("(int) (fv() % 4)", "(fv() % 4) as i32"),
    "Player_GetPlayerCharacter": ("(CharacterKind) (fv() % 30)", "(fv() % 30) as _"),
    "HSD_Randf": ("(float) (rnd() % 1000) / 1000.0f", "(rnd() % 1000) as f32 / 1000.0"),
    "HSD_Randi": ("(a0 > 0 ? (int) (rnd() % (u32) a0) : 0)", "if a0 > 0 { (rnd() % a0 as u32) as i32 } else { 0 }"),
    "ScriptGame_CameraFov": ("a0 + (float) (rnd() % 3)", "a0 + (rnd() % 3) as f32"),
}
def rng(lo, hi, div=1):
    return (f"RNG({lo}, {hi}, {div})", f"super::rng({lo}, {hi}, {div})")


for n, v in {
        "Stage_GetBlastZoneBottomOffset": rng(-250, -150), "Stage_GetBlastZoneLeftOffset": rng(-300, -180),
        "Stage_GetBlastZoneRightOffset": rng(180, 300), "Stage_GetBlastZoneTopOffset": rng(150, 250),
        "Stage_GetCamAngleRadiansDown": rng(0, 30, 100), "Stage_GetCamAngleRadiansLeft": rng(0, 30, 100),
        "Stage_GetCamAngleRadiansRight": rng(0, 30, 100), "Stage_GetCamAngleRadiansUp": rng(0, 30, 100),
        "Stage_GetCamBoundsBottomOffset": rng(-100, -20), "Stage_GetCamBoundsLeftOffset": rng(-120, -40),
        "Stage_GetCamBoundsRightOffset": rng(40, 120), "Stage_GetCamBoundsTopOffset": rng(30, 100),
        "Stage_GetCamFixedFov": rng(20, 60), "Stage_GetCamInfoX20": rng(0, 12, 100), "Stage_GetCamInfoX24": rng(0, 12, 100),
        "Stage_GetCamMaxDepth": rng(300, 600), "Stage_GetCamPanAngleRadians": rng(-10, 10, 50),
        "Stage_GetCamTrackRatio": rng(0, 10, 10), "Stage_GetCamTrackSmooth": rng(0, 10, 10), "Stage_GetCamZoomRate": rng(0, 100),
        "Stage_GetPauseCamZPosInit": rng(100, 400), "Stage_GetPauseCamZPosMax": rng(300, 500), "Stage_GetPauseCamZPosMin": rng(50, 150),
        "grCastle_801D0FF0": rng(-200, 0), "grCorneria_801E2FCC": rng(-200, 0), "grGarden_80203624": rng(-200, 0),
        "grHomeRun_8021EF10": rng(-200, 0), "grKinokoRoute_802087B0": rng(-200, 0), "grZebes_801DCCC8": rng(-200, 0),
        "HSD_CObjGetAspect": rng(100, 200, 100), "HSD_CObjGetAuthoredAspect": rng(100, 200, 100), "HSD_CObjGetFar": rng(10000, 20000),
        "HSD_CObjGetFov": rng(15, 75)}.items():
    RET[n] = v
for n in FLOAT_GETTERS:
    RET.setdefault(n, ("RNN()", "super::rnn()"))

EFFECTS = {
    "HSD_CObjGetEyePosition": vec_out("a1"),
    "HSD_CObjGetInterest": vec_out("a1"),
    "HSD_CObjGetEyeVector": vec_out("a1"),
    "HSD_CObjGetUpVector": vec_out("a1"),
    "HSD_CObjGetScissor": (
        "if (a1) { a1->left = (s16) (rnd() % 100); a1->right = (s16) (200 + rnd() % 400); a1->top = (s16) (rnd() % 100); a1->bottom = (s16) (200 + rnd() % 300); }",
        "if !a1.is_null() { (*a1).left = (rnd() % 100) as _; (*a1).right = (200 + rnd() % 400) as _; (*a1).top = (rnd() % 100) as _; (*a1).bottom = (200 + rnd() % 300) as _; }"),
    "Ground_801C4368": ("*a0 = RNN(); *a1 = RNN();", "*a0 = super::rnn(); *a1 = super::rnn();"),
    "ScriptGame_CameraFollowTarget": vec_out("a2"),
    "Stage_80224CAC": vec_out("a0"),
    "Stage_SetVecToFixedCamPos": vec_out("a0"),
    "Stage_UnkSetVec3TCam_Offset": vec_out("a0"),
    "ftLib_80086644": vec_out("a1"),
    "lbVector_WorldToScreen": (VEC_C % ("a2", "a2", "a2", "a2"), VEC_R % ("a2", "a2", "a2", "a2")),
}

SPECS["cm"] = dict(
    src="src/melee/cm/camera.c",
    mods=["camera"],
    freg_mod="camera",
    types=["Camera", "CmSubject", "HSD_CObj", "HSD_GObj", "CameraBounds", "CameraDebugMode", "HSD_PadStatus"],
    patch_all=DBG_PATCH,
    skip_data=["guess", "y", "result"],
    # math the port takes from melee_runtime: the real C runs on the oracle side
    skip_stubs=["__frsqrte", "atan2f", "atanf", "cosf", "sinf", "tanf", "lbVector_8000E838", "lbVector_Add", "lbVector_Diff",
                "lbVector_Normalize", "lbVector_Rotate", "lbVector_RotateAboutUnitAxis", "lbVector_Sub", "PSVECCrossProduct", "memzero"],
    ret=RET,
    ignore_args={"MotionWorldDraw": [0]},
    effects=EFFECTS,
    null_ret_ok=["HSD_GObj_SetupProc"],
)
