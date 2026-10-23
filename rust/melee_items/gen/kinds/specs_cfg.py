"""Driver descriptions (fixture tweaks, typed attribute buffers, argument hints) of the item-kind TUs, read by
mk_driver.py. Every entry is a (C, Rust) pair drawing the same numbers in the same order."""
import re

CFG = {}


def per_item(c_body, r_body):
    """A tweak applied to each of the three items (k = 0..2); the bodies use IT[k]."""
    return ("    for (k = 0; k < 3; k++) {\n" + c_body + "\n    }",
            "    for k in 0..3 {\n" + r_body + "\n    }")


def sbuf(rty):
    return f"let a = SPECB.0[0].as_mut_ptr() as *mut {rty};"


# ------------------------------------------------------------------------------------------ itbat
CFG["itbat"] = dict(
    spec=[("""        itBatAttributes* a = (itBatAttributes*) SPEC[0];
        a->x0 = (int) (rnd() % 5); a->x4 = (int) (rnd() % 5); a->x8 = (int) (rnd() % 5); a->xC_vel = rw();""",
           """            let a = SPECB.0[0].as_mut_ptr() as *mut itBatAttributes;
            (*a).x0 = (rnd() % 5) as i32; (*a).x4 = (rnd() % 5) as i32; (*a).x8 = (rnd() % 5) as i32; (*a).xC_vel = rw();""")],
    tweak=("", ""),
    ptrs=("", ""),
)

# ------------------------------------------------------------------------------------- itarwinglaser
_aw = "IT[k].xDD4_itemVar.arwinglaser"
CFG["itarwinglaser"] = dict(
    spec=[("""        ArwingLaserAttr* a = (ArwingLaserAttr*) SPEC[0];
        a->x0 = &ATTR[0]; a->x4 = rw(); a->x8 = rw();""",
           """            let a = SPECB.0[0].as_mut_ptr() as *mut m::ArwingLaserAttr;
            (*a).x0 = &raw mut ATTR[0]; (*a).x4 = rw(); (*a).x8 = rw();""")],
    tweak=per_item(
        f"""        {_aw}.xE38 = (int) (rnd() % 6);
        {_aw}.xE30 = (s16) (rnd() % 3);
        {_aw}.xDF4 = (rnd() & 1) ? &GO[(k + 1) % 3] : 0;
        i = (int) (rnd() % 3);
        {_aw}.xDF8 = (rnd() & 1) ? &JO[3 + i] : 0;
        {_aw}.xE40 = (rnd() & 1) ? &GO[(k + 2) % 3] : 0;
        {_aw}.xE44 = (rnd() & 1) ? &GO[(k + 1) % 3] : 0;
        {_aw}.xE48 = (int) (rnd() % 5);
        {_aw}.xE4C = (int) (rnd() % 3);
        i = (int) (rnd() % 80) - 40;
        {_aw}.xE24.z = (rnd() & 1) ? (float) i : rw();""",
        """        let aw = &mut IT[k].xDD4_itemVar.arwinglaser;
        aw.xE38 = (rnd() % 6) as i32;
        aw.xE30 = (rnd() % 3) as i16;
        aw.xDF4 = if rnd() & 1 != 0 { &raw mut GO[(k + 1) % 3] } else { null_mut() };
        let i = (rnd() % 3) as usize;
        aw.xDF8 = if rnd() & 1 != 0 { &raw mut JO[3 + i] } else { null_mut() };
        aw.xE40 = if rnd() & 1 != 0 { &raw mut GO[(k + 2) % 3] } else { null_mut() };
        aw.xE44 = if rnd() & 1 != 0 { &raw mut GO[(k + 1) % 3] } else { null_mut() };
        aw.xE48 = (rnd() % 5) as i32;
        aw.xE4C = (rnd() % 3) as i32;
        let i = (rnd() % 80) as i32 - 40;
        aw.xE24.z = if rnd() & 1 != 0 { i as f32 } else { rw() };"""),
    ptrs=("""    for (k = 0; k < 3; k++) {
        HM(PI(%s.xDF4)); HM(PI(%s.xDF8)); HM(PI(%s.xE40)); HM(PI(%s.xE44));
        %s.xDF4 = 0; %s.xDF8 = 0; %s.xE40 = 0; %s.xE44 = 0;
    }""" % ((_aw,) * 8),
          """    for k in 0..3 {
        let aw = &mut IT[k].xDD4_itemVar.arwinglaser;
        hm(pi(aw.xDF4)); hm(pi(aw.xDF8)); hm(pi(aw.xE40)); hm(pi(aw.xE44));
        aw.xDF4 = null_mut(); aw.xDF8 = null_mut(); aw.xE40 = null_mut(); aw.xE44 = null_mut();
    }"""),
    hints={"type": ("(int) (rnd() % 6)", "(rnd() % 6) as i32"), "arg4": ("(int) (rnd() % 5)", "(rnd() % 5) as i32")},
    noret=["it_802E7654"],
)

# --------------------------------------------------------------------------------------- itbombhei
_bh = "IT[k].xDD4_itemVar.bombhei"
CFG["itbombhei"] = dict(
    spec=[("""        itBombHeiAttributes* a = (itBombHeiAttributes*) SPEC[0];
        a->x0 = rw(); a->x4 = (float) (rnd() % 12); a->x8 = (float) (rnd() % 12); a->xC = rw();
        a->x10 = (float) (rnd() % 12); a->x14 = (float) (rnd() % 12); a->x18 = (float) (rnd() % 12); a->x1C = rw();
        a->x20.x = (float) (rnd() % 3) / 4.0f; a->x20.y = (float) (rnd() % 3) / 4.0f; a->x20.z = rw();""",
           """            let a = SPECB.0[0].as_mut_ptr() as *mut itBombHeiAttributes;
            (*a).x0 = rw(); (*a).x4 = (rnd() % 12) as f32; (*a).x8 = (rnd() % 12) as f32; (*a).xC = rw();
            (*a).x10 = (rnd() % 12) as f32; (*a).x14 = (rnd() % 12) as f32; (*a).x18 = (rnd() % 12) as f32; (*a).x1C = rw();
            (*a).x20.x = (rnd() % 3) as f32 / 4.0; (*a).x20.y = (rnd() % 3) as f32 / 4.0; (*a).x20.z = rw();""")],
    tweak=per_item(
        f"""        {_bh}.xDD4 = (int) (rnd() % 12) - 3;
        {_bh}.xDD8 = (rnd() & 1) ? 1 : -1;
        {_bh}.xDDC = (int) (rnd() & 1);
        {_bh}.xDE0 = (int) (rnd() & 1);
        {_bh}.xDE4 = (int) (rnd() & 1);
        {_bh}.xDE8 = (rnd() & 1) ? 1.0f : rw();
        i = (int) (rnd() % 10) - 3;
        {_bh}.xDEC = (float) i;
        i = (int) (rnd() % 6) - 2;
        {_bh}.xDF0 = (float) i;
        {_bh}.xDF4 = (rnd() & 1) ? 1.0f : -1.0f;
        {_bh}.xDF8 = rw();
        {_bh}.xDFC = rw();
        {_bh}.xE04 = rw();
        {_bh}.xE0C.x = rw(); {_bh}.xE0C.y = rw(); {_bh}.xE0C.z = rw();""",
        """        let bh = &mut IT[k].xDD4_itemVar.bombhei;
        bh.xDD4 = (rnd() % 12) as i32 - 3;
        bh.xDD8 = if rnd() & 1 != 0 { 1 } else { -1 };
        bh.xDDC = (rnd() & 1) as i32;
        bh.xDE0 = (rnd() & 1) as i32;
        bh.xDE4 = (rnd() & 1) as i32;
        bh.xDE8 = if rnd() & 1 != 0 { 1.0 } else { rw() };
        let i = (rnd() % 10) as i32 - 3;
        bh.xDEC = i as f32;
        let i = (rnd() % 6) as i32 - 2;
        bh.xDF0 = i as f32;
        bh.xDF4 = if rnd() & 1 != 0 { 1.0 } else { -1.0 };
        bh.xDF8 = rw();
        bh.xDFC = rw();
        bh.xE04 = rw();
        bh.xE0C.x = rw(); bh.xE0C.y = rw(); bh.xE0C.z = rw();"""),
    ptrs=("", ""),
)

# ------------------------------------------------------------------------------------------ itbox
_bx = "IT[k].xDD4_itemVar.box"
CFG["itbox"] = dict(
    spec=[("""        itBoxAttributes* a = (itBoxAttributes*) SPEC[0];
        a->spawn_weight_0 = (int) (rnd() % 5); a->spawn_weight_1 = (int) (rnd() % 5); a->spawn_weight_2 = (int) (rnd() % 5);
        a->empty_weight = (int) (rnd() % 5); a->special_spawn_weight = (int) (rnd() % 5);
        a->damage_threshold = (float) (rnd() % 100); a->bounce_angle_threshold = rw(); a->break_vel_threshold = rw();""",
           """            let a = SPECB.0[0].as_mut_ptr() as *mut itBoxAttributes;
            (*a).spawn_weight_0 = (rnd() % 5) as i32; (*a).spawn_weight_1 = (rnd() % 5) as i32; (*a).spawn_weight_2 = (rnd() % 5) as i32;
            (*a).empty_weight = (rnd() % 5) as i32; (*a).special_spawn_weight = (rnd() % 5) as i32;
            (*a).damage_threshold = (rnd() % 100) as f32; (*a).bounce_angle_threshold = rw(); (*a).break_vel_threshold = rw();""")],
    tweak=per_item(
        f"""        {_bx}.opened = (int) (rnd() & 1);
        {_bx}.despawn_timer = (int) (rnd() % 5) - 1;
        {_bx}.spawned_gobj = (rnd() & 1) ? &GO[(k + 1) % 3] : 0;
        {_bx}.rot_vel_x = rg();
        {_bx}.rot_vel_y = rg();
        IT[k].xC9C = (int) (rnd() % 120);
        if (rnd() & 1) IT[k].x378_itemColl.env_flags = Collide_FloorPush | Collide_FloorHug;""",
        """        let bx = &mut IT[k].xDD4_itemVar.r#box;
        bx.opened = (rnd() & 1) as i32;
        bx.despawn_timer = (rnd() % 5) as i32 - 1;
        bx.spawned_gobj = if rnd() & 1 != 0 { &raw mut GO[(k + 1) % 3] } else { null_mut() };
        bx.rot_vel_x = rg();
        bx.rot_vel_y = rg();
        IT[k].xC9C = (rnd() % 120) as i32;
        if rnd() & 1 != 0 {
            IT[k].x378_itemColl.env_flags = Collide_FloorPush | Collide_FloorHug;
        }"""),
    ptrs=("""    for (k = 0; k < 3; k++) { HM(PI(%s.spawned_gobj)); %s.spawned_gobj = 0; }""" % (_bx, _bx),
          """    for k in 0..3 { hm(pi(IT[k].xDD4_itemVar.r#box.spawned_gobj)); IT[k].xDD4_itemVar.r#box.spawned_gobj = null_mut(); }"""),
    hints={n: ("(int) (rnd() % 6)", "(rnd() % 6) as i32") for n in ("arg1", "arg2", "arg3", "arg4")},
    extra_ops=[
        ("it_80286088(NULL)", "ret = PI(it_80286088(0));", "ret = pi(m::it_80286088(null_mut()));"),
        ("it_80286088(no jobj)", "GO[0].hsd_obj = 0; ret = PI(it_80286088(&GO[0]));",
         "GO[0].hsd_obj = null_mut(); ret = pi(m::it_80286088(&raw mut GO[0]));"),
    ],
)

# -------------------------------------------------------------------------------------- itcapsule
CFG["itcapsule"] = dict(
    spec=[("""        ItCapsuleAttr* a = (ItCapsuleAttr*) SPEC[0];
        a->x0 = (int) (rnd() % 5); a->x4 = (int) (rnd() % 3);""",
           """            let a = SPECB.0[0].as_mut_ptr() as *mut ItCapsuleAttr;
            (*a).x0 = (rnd() % 5) as i32; (*a).x4 = (rnd() % 3) as i32;""")],
    tweak=per_item("        IT[k].xDD4_itemVar.capsule.x0 = (int) (rnd() & 1);\n        IT[k].xDD4_itemVar.capsule.x4 = (int) (rnd() & 1);",
                   "        IT[k].xDD4_itemVar.capsule.x0 = (rnd() & 1) as i32;\n        IT[k].xDD4_itemVar.capsule.x4 = (rnd() & 1) as i32;"),
    ptrs=("", ""),
)

# ---------------------------------------------------------------------------------------- itcerebi
CFG["itcerebi"] = dict(
    spec=[("""        float* a = (float*) SPEC[0];
        a[0] = rw(); a[1] = rw(); a[2] = rw(); a[3] = rw();""",
           """            let a = SPECB.0[0].as_mut_ptr() as *mut f32;
            *a = rw(); *a.add(1) = rw(); *a.add(2) = rw(); *a.add(3) = rw();""")],
    tweak=("", ""),
    ptrs=("", ""),
)

# -------------------------------------------------------------------------------------- itchicorita
_ch = "IT[k].xDD4_itemVar.chicorita"
CFG["itchicorita"] = dict(
    spec=[("""        itChicoritaAttr* a = (itChicoritaAttr*) SPEC[0];
        a->scale = rw(); a->x4 = (int) (rnd() % 4);""",
           """            let a = SPECB.0[0].as_mut_ptr() as *mut itChicoritaAttr;
            (*a).scale = rw(); (*a).x4 = (rnd() % 4) as i32;"""),
          ("""        itChicoritaLeafAttr* a = (itChicoritaLeafAttr*) SPEC[0];
        a->timer = (float) (rnd() % 20); a->x4 = rw(); a->x8 = rw(); a->xC = rw(); a->x10 = rw();""",
           """            let a = SPECB.0[0].as_mut_ptr() as *mut itChicoritaLeafAttr;
            (*a).timer = (rnd() % 20) as f32; (*a).x4 = rw(); (*a).x8 = rw(); (*a).xC = rw(); (*a).x10 = rw();""")],
    prof=lambda n: 1 if re.search(r"(?i)leaf|802C9B20", n) else 0,
    tweak=per_item(f"        {_ch}.x60 = (int) (rnd() % 5) - 1;\n        {_ch}.x64 = rg();",
                   "        IT[k].xDD4_itemVar.chicorita.x60 = (rnd() % 5) as i32 - 1;\n        IT[k].xDD4_itemVar.chicorita.x64 = rg();"),
    ptrs=("", ""),
)

# ------------------------------------------------------------------------------ itclimbersblizzard
CFG["itclimbersblizzard"] = dict(
    spec=[("""        itClimbersBlizzardAttributes* a = (itClimbersBlizzardAttributes*) SPEC[0];
        a->x0 = rw(); a->x4 = rw(); a->x8 = rw(); a->xC = rw(); a->x10 = rw();""",
           """            let a = SPECB.0[0].as_mut_ptr() as *mut itClimbersBlizzardAttributes;
            (*a).x0 = rw(); (*a).x4 = rw(); (*a).x8 = rw(); (*a).xC = rw(); (*a).x10 = rw();""")],
    tweak=per_item("        IT[k].xDD4_itemVar.climbersblizzard.flag0 = rnd() & 1;",
                   "        IT[k].xDD4_itemVar.climbersblizzard.set_flag0((rnd() & 1) as u8);"),
    ptrs=("""    for (k = 0; k < 3; k++) {
        HM(B(IT[k].xDD4_itemVar.climbersblizzard.x0)); HM(IT[k].xDD4_itemVar.climbersblizzard.flag0);
        zero(&IT[k].xDD4_itemVar.climbersblizzard, sizeof IT[k].xDD4_itemVar.climbersblizzard);
    }""", """    for k in 0..3 {
        let cb = &mut IT[k].xDD4_itemVar.climbersblizzard;
        hm(cb.x0.to_bits()); hm(cb.flag0() as u32);
        core::ptr::write_bytes(cb as *mut _ as *mut u8, 0, core::mem::size_of::<itClimbersBlizzard_ItemVars>());
    }"""),
)


# ------------------------------------------------------------------------------ shard 761-768
CFG["itpeachexplode"] = dict(spec=[("", "")], tweak=("", ""), ptrs=("", ""))

CFG["itparasol"] = dict(
    spec=[("""        float* a = (float*) SPEC[0];
        a[0] = rw(); a[1] = rw(); a[2] = rw(); a[3] = rw();""",
           """            let a = SPECB.0[0].as_mut_ptr() as *mut f32;
            *a = rw(); *a.add(1) = rw(); *a.add(2) = rw(); *a.add(3) = rw();""")],
    tweak=("", ""),
    ptrs=("", ""),
    hints={"statenum": ("((const int[]){1, 2, 4, 5, 6, 7, 8, 9, 10})[rnd() % 9]", "[1, 2, 4, 5, 6, 7, 8, 9, 10][(rnd() % 9) as usize]")},
)

_D40C = ("    it_804D6D40 = &D40; D40.x8 = (float) (rnd() % 1000) / 1000.0f;",
         "    m::ext::it_804D6D40 = &raw mut D40; D40.x8 = (rnd() % 1000) as f32 / 1000.0;")
_D40D = ("static it_804D6D40_t D40;", "static mut D40: it_804D6D40_t = unsafe { core::mem::zeroed() };")

CFG["itoctarockstone"] = dict(spec=[("", "")], tweak=("", ""), ptrs=("", ""),
                              hints={"dir": ("(rnd() & 1) ? 1 : -1", "if rnd() & 1 != 0 { 1 } else { -1 }")})

_oc = "IT[k].xDD4_itemVar.octarock"
CFG["itoctarock"] = dict(
    spec=[("""        itOctarockAttributes* a = (itOctarockAttributes*) SPEC[0];
        a->x0 = (s32*) (SPEC[0] + 64); *a->x0 = (int) (rnd() % 30);
        a->x4 = rw(); a->x8 = (float) (rnd() % 40); a->xC = (float) (rnd() % 40); a->x10 = rw(); a->x14 = rw(); a->x18 = rw();
        a->x1C = (s16) (rnd() % 30);""",
           """            let a = SPECB.0[0].as_mut_ptr() as *mut itOctarockAttributes;
            (*a).x0 = SPECB.0[0].as_mut_ptr().add(64) as *mut i32; *(*a).x0 = (rnd() % 30) as i32;
            (*a).x4 = rw(); (*a).x8 = (rnd() % 40) as f32; (*a).xC = (rnd() % 40) as f32; (*a).x10 = rw(); (*a).x14 = rw(); (*a).x18 = rw();
            (*a).x1C = (rnd() % 30) as i16;""")],
    tweak=(_D40C[0] + "\n" + per_item(f"        {_oc}.x32 = (s16) (rnd() % 4);\n        {_oc}.x2C = (int) (rnd() & 1);", "")[0],
           _D40C[1] + "\n" + per_item("", "        IT[k].xDD4_itemVar.octarock.x32 = (rnd() % 4) as i16;\n        IT[k].xDD4_itemVar.octarock.x2C = (rnd() & 1) as i32;")[1]),
    ptrs=("", ""),
    c_decl=_D40D[0], rs_decl=_D40D[1],
    hints={"dir": ("(rnd() & 1) ? 1 : -1", "if rnd() & 1 != 0 { 1 } else { -1 }")},
)

_ok = "IT[k].xDD4_itemVar.oldkuri"
CFG["itoldkuri"] = dict(
    spec=[("""        itOldkuriAttributes* a = (itOldkuriAttributes*) SPEC[0];
        a->x0 = (s32*) (SPEC[0] + 64); *a->x0 = (int) (rnd() % 30); ((float*) a->x0)[1] = rw();
        a->x4 = rw(); a->x8 = rw(); a->xC = (float) (rnd() % 10);""",
           """            let a = SPECB.0[0].as_mut_ptr() as *mut m::itOldkuriAttributes;
            (*a).x0 = SPECB.0[0].as_mut_ptr().add(64) as *mut i32; *(*a).x0 = (rnd() % 30) as i32; *((*a).x0 as *mut f32).add(1) = rw();
            (*a).x4 = rw(); (*a).x8 = rw(); (*a).xC = (rnd() % 10) as f32;""")],
    tweak=(_D40C[0] + "\n" + per_item(f"        {_ok}.xDF8 = (int) (rnd() % 3);\n        {_ok}.xDFC = (int) (rnd() & 1);", "")[0],
           _D40C[1] + "\n" + per_item("", "        IT[k].xDD4_itemVar.oldkuri.xDF8 = (rnd() % 3) as i32;\n        IT[k].xDD4_itemVar.oldkuri.xDFC = (rnd() & 1) as i32;")[1]),
    ptrs=("", ""),
    c_decl=_D40D[0], rs_decl=_D40D[1],
)

_oo = "IT[k].xDD4_itemVar.oldottosea"
CFG["itoldottosea"] = dict(
    spec=[("""        itOldottoseaAttributes* a = (itOldottoseaAttributes*) SPEC[0];
        a->x0 = (void*) (SPEC[0] + 64); ((int*) (SPEC[0] + 64))[0] = (int) (rnd() % 30); ((float*) (SPEC[0] + 64))[1] = rw();
        a->x4 = rw(); a->x8 = rw(); a->xC = rw(); a->x10 = (s8) (rnd() % 4);
        a->x14 = rw(); a->x18 = rw(); a->x1C = rw(); a->x20 = rw(); a->x24 = rw(); a->x28 = (s8) (rnd() % 3);""",
           """            let a = SPECB.0[0].as_mut_ptr() as *mut itOldottoseaAttributes;
            (*a).x0 = SPECB.0[0].as_mut_ptr().add(64) as *mut _; *(SPECB.0[0].as_mut_ptr().add(64) as *mut i32) = (rnd() % 30) as i32; *(SPECB.0[0].as_mut_ptr().add(68) as *mut f32) = rw();
            (*a).x4 = rw(); (*a).x8 = rw(); (*a).xC = rw(); (*a).x10 = (rnd() % 4) as i8;
            (*a).x14 = rw(); (*a).x18 = rw(); (*a).x1C = rw(); (*a).x20 = rw(); (*a).x24 = rw(); (*a).x28 = (rnd() % 3) as i8;""")],
    tweak=(_D40C[0] + "\n" + per_item(f"        {_oo}.x20 = (rnd() & 1) ? &GO[(k + 1) % 3] : 0;\n        {_oo}.x24 = ((const int[]){{0, 10, 25, 42, 80, 81}})[rnd() % 6];\n        {_oo}.x28 = (int) (rnd() % 3);\n        {_oo}.x2C = (s16) (rnd() & 1);", "")[0],
           _D40C[1] + "\n" + per_item("", "        IT[k].xDD4_itemVar.oldottosea.x20 = if rnd() & 1 != 0 { &raw mut GO[(k + 1) % 3] } else { null_mut() };\n        IT[k].xDD4_itemVar.oldottosea.x24 = [0, 10, 25, 42, 80, 81][(rnd() % 6) as usize];\n        IT[k].xDD4_itemVar.oldottosea.x28 = (rnd() % 3) as i32;\n        IT[k].xDD4_itemVar.oldottosea.x2C = (rnd() & 1) as i16;")[1]),
    ptrs=("    for (k = 0; k < 3; k++) { HM(PI(%s.x20)); %s.x20 = 0; }" % (_oo, _oo),
          "    for k in 0..3 { hm(pi(IT[k].xDD4_itemVar.oldottosea.x20)); IT[k].xDD4_itemVar.oldottosea.x20 = null_mut(); }"),
    c_decl=_D40D[0], rs_decl=_D40D[1],
)

_pp = "IT[k].xDD4_itemVar.patapata"
CFG["itpatapata"] = dict(
    spec=[("""        itPatapataAttributes* a = (itPatapataAttributes*) SPEC[0];
        a->x0 = (void*) (SPEC[0] + 64); ((float*) (SPEC[0] + 64))[1] = rw();
        a->x4 = rw(); a->x8 = rw(); a->xC = rw(); a->x10 = (int) (rnd() % 6); a->x14 = rw(); a->x18 = rw();
        a->x1C = (int) (rnd() % 20); a->x20 = rw(); a->x24 = (int) (rnd() % 20); a->x28 = (float) (rnd() % 30);
        a->x2C = (int) (rnd() % 6); a->x30 = (float) (rnd() % 5); a->x34 = (float) (rnd() % 5); a->x38 = rw(); a->x3C = rw();""",
           """            let a = SPECB.0[0].as_mut_ptr() as *mut itPatapataAttributes;
            (*a).x0 = SPECB.0[0].as_mut_ptr().add(64) as *mut _; *(SPECB.0[0].as_mut_ptr().add(68) as *mut f32) = rw();
            (*a).x4 = rw(); (*a).x8 = rw(); (*a).xC = rw(); (*a).x10 = (rnd() % 6) as i32; (*a).x14 = rw(); (*a).x18 = rw();
            (*a).x1C = (rnd() % 20) as i32; (*a).x20 = rw(); (*a).x24 = (rnd() % 20) as i32; (*a).x28 = (rnd() % 30) as f32;
            (*a).x2C = (rnd() % 6) as i32; (*a).x30 = (rnd() % 5) as f32; (*a).x34 = (rnd() % 5) as f32; (*a).x38 = rw(); (*a).x3C = rw();""")],
    tweak=per_item(f"""        for (i = 14; i < 19; i++) DBT[k].bones[i] = &JO[6 + 14 * k + (i - 14)];
        {_pp}.x40 = (int) (rnd() % 3);
        {_pp}.x20 = (int) (rnd() % 5) - 1;
        {_pp}.x24 = (int) (rnd() % 5) - 1;
        {_pp}.x28 = (int) (rnd() % 5) - 1;
        {_pp}.x2C = rw();
        {_pp}.x30 = rw();
        {_pp}.x44 = (int) (rnd() & 1);
        {_pp}.x4 = (int) (rnd() % 4);""",
        """        for i in 14..19 { DBT[k].bones[i] = &raw mut JO[6 + 14 * k + (i - 14)]; }
        let pp = &mut IT[k].xDD4_itemVar.patapata;
        pp.x40 = (rnd() % 3) as i32;
        pp.x20 = (rnd() % 5) as i32 - 1;
        pp.x24 = (rnd() % 5) as i32 - 1;
        pp.x28 = (rnd() % 5) as i32 - 1;
        pp.x2C = rw();
        pp.x30 = rw();
        pp.x44 = (rnd() & 1) as i32;
        pp.x4 = (rnd() % 4) as i32;"""),
    ptrs=("", ""),
    hints={"arg0": ("(int) (rnd() % 3)", "(rnd() % 3) as i32"), "facing": ("(rnd() & 1) ? 1 : -1", "if rnd() & 1 != 0 { 1 } else { -1 }")},
)

_nk = "IT[k].xDD4_itemVar.nokonoko"
CFG["itnokonoko"] = dict(
    spec=[("""        itNokoNoko_DatAttrs* a = (itNokoNoko_DatAttrs*) SPEC[0];
        a->x0 = (void*) (SPEC[0] + 64); ((int*) (SPEC[0] + 64))[0] = (int) (rnd() % 30); ((float*) (SPEC[0] + 64))[1] = rw();
        a->x4 = rw(); a->x8 = rw();""",
           """            let a = SPECB.0[0].as_mut_ptr() as *mut itNokoNoko_DatAttrs;
            (*a).x0 = SPECB.0[0].as_mut_ptr().add(64) as *mut _; *(SPECB.0[0].as_mut_ptr().add(64) as *mut i32) = (rnd() % 30) as i32; *(SPECB.0[0].as_mut_ptr().add(68) as *mut f32) = rw();
            (*a).x4 = rw(); (*a).x8 = rw();""")],
    tweak=per_item(f"""        {_nk}.x20 = (int) (rnd() % 4) - 1;
        {_nk}.x28 = (int) (rnd() % 4) - 1;
        {_nk}.x34 = rw();
        {_nk}.x38 = (rnd() & 1) ? 1.0f : -1.0f;
        {_nk}.x3C = (int) (rnd() % 8);
        {_nk}.x40 = (int) (rnd() % 4);
        {_nk}.x4 = (int) (rnd() % 4);""",
        """        let nk = &mut IT[k].xDD4_itemVar.nokonoko;
        nk.x20 = (rnd() % 4) as i32 - 1;
        nk.x28 = (rnd() % 4) as i32 - 1;
        nk.x34 = rw();
        nk.x38 = if rnd() & 1 != 0 { 1.0 } else { -1.0 };
        nk.x3C = (rnd() % 8) as i32;
        nk.x40 = (rnd() % 4) as i32;
        nk.x4 = (rnd() % 4) as i32;"""),
    ptrs=("", ""),
    hints={"arg0": ("(int) (rnd() % 4)", "(rnd() % 4) as i32"), "arg3": ("(rnd() & 1) ? 1 : -1", "if rnd() & 1 != 0 { 1 } else { -1 }")},
)
