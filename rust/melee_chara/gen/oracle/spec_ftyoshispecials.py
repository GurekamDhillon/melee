T = "t::"
PARTS = ["Start_0", "Loop_0", "Loop_1", "End", "Start_1", "Loop_2", "Loop_3", "Landing"]

# Loop_1_Anim: `x28 % xA4` must not divide by zero, and `delta / total * M_PI` feeds a while loop that never
# ends for an infinite angle (x10 == 0 gives total == 0): keep both away from zero on both sides.
PRE_C_LOOP1 = ("{ ftYoshiAttributes* a = (ftYoshiAttributes*) OATTR; if (a->xA4 == 0) a->xA4 = 3; "
               "if (OF.mv.ys.specials.x10 > -0.01f && OF.mv.ys.specials.x10 < 0.01f) OF.mv.ys.specials.x10 = 2.5f; }")
PRE_RS_LOOP1 = ("let a = &mut *(w.attr.as_mut_ptr() as *mut ftYoshiAttributes); if a.xA4 == 0 { a.xA4 = 3; } "
                "let x10 = &mut w.f.mv.ys.specials.x10; if *x10 > -0.01 && *x10 < 0.01 { *x10 = 2.5; }")

funcs = [
    "ftYs_SpecialS_8012EB48",
    "fn_8012EC7C",
    "fn_8012EDE8",
    "fn_8012EFC0",
    "fn_8012EFF4",
    "ftYs_SpecialS_8012F0DC",
    "ftYs_SpecialS_8012F35C",
    "ftYs_SpecialS_Enter",
    "ftYs_SpecialAirS_Enter",
]
for p in PARTS:
    funcs.append(f"ftYs_SpecialAirS{p}_Anim" if p != "Loop_1" else
                 dict(name="ftYs_SpecialAirSLoop_1_Anim", pre_c=PRE_C_LOOP1, pre_rs=PRE_RS_LOOP1))
for p in PARTS:
    funcs.append(f"ftYs_SpecialAirS{p}_IASA")
for p in PARTS:
    funcs.append(f"ftYs_SpecialAirS{p}_Phys")
for p in PARTS:
    funcs.append(f"ftYs_SpecialAirS{p}_Coll")
funcs.append("fn_8013295C")

# the round-1 ports taking the fighter instead of the gobj
FP = "&mut *((*g).user_data as *mut Fighter)"
funcs = [dict(name=f, rs_args=[FP]) if f in ("ftYs_SpecialAirSLanding_IASA", "ftYs_SpecialAirSStart_0_Phys",
                                              "ftYs_SpecialAirSStart_1_IASA") else f for f in funcs]

SPEC = dict(
    src="src/melee/ft/kinds/ftYoshi/ftyoshispecials.c",
    rust_mod="ftyoshispecials",
    ext_mods=["ftinl"],
    legacy_stubs=True,
    attr_types=["ftYoshiAttributes"],
    prefer=["mv.ys.specials", "u.ys"],
    outs={"lb_8000B1CC": [2]},
    bitfields=[("x221E_b4", "x221E_b4")],
    default_bf=["x221E_b4"],
    motions=[0x16B, 0x167, 0x16A, 0x165, 0x166, 0x168, 0x169],
    fnvals={
        "fn_8012EC7C": T + "cb_fn_8012EC7C",
        "fn_8012EDE8": T + "cb_fn_8012EDE8",
        "fn_8012EFC0": T + "cb_fn_8012EFC0",
        "fn_8012EFF4": T + "cb_fn_8012EFF4",
        "fn_8013295C": T + "cb_fn_8013295C",
    },
    funcs=funcs,
)
