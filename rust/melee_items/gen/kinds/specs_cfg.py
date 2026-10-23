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


# model descriptor + joint for the functions that follow article->x10_modelDesc->x0_joint
MD_CDECL = """static ItemModelDesc MD;
static HSD_Joint JNT[1];
"""
MD_RDECL = """static mut MD: ItemModelDesc = unsafe { core::mem::zeroed() };
static mut JNT: [HSD_Joint; 1] = unsafe { core::mem::zeroed() };
"""
MD_CSET = """        ART[k].x10_modelDesc = &MD;
        MD.x0_joint = &JNT[0];"""
MD_RSET = """        ART[k].x10_modelDesc = &raw mut MD;
        MD.x0_joint = &raw mut JNT[0];"""
MD_CPREG = ["preg(&MD, sizeof MD); preg(JNT, sizeof JNT);"]
MD_RPREG = ["preg(&raw const MD, core::mem::size_of_val(&MD)); preg(&raw const JNT, core::mem::size_of_val(&JNT));"]

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

# --------------------------------------------------------------------------------------- itsscopebeam
CFG["itsscopebeam"] = dict(
    spec=[("""        float* a = (float*) SPEC[0];
        for (i = 0; i < 9; i++) { a[3 * i] = rw(); a[3 * i + 1] = rw(); a[3 * i + 2] = (float) (rnd() % 4); }
        a[30] = rw(); a[31] = rw();""",
           """            let a = SPECB.0[0].as_mut_ptr() as *mut f32;
            for i in 0..9 { *a.add(3 * i) = rw(); *a.add(3 * i + 1) = rw(); *a.add(3 * i + 2) = (rnd() % 4) as f32; }
            *a.add(30) = rw(); *a.add(31) = rw();""")],
    tweak=per_item("        IT[k].xDD4_itemVar.scopebeam.x0 = (int) (rnd() % 10);\n        IT[k].xD44_lifeTimer = (float) (rnd() % 5);\n        if (rnd() & 1) IT[k].x378_itemColl.env_flags = 0x18000;",
                   "        IT[k].xDD4_itemVar.scopebeam.x0 = (rnd() % 10) as i32;\n        IT[k].xD44_lifeTimer = (rnd() % 5) as f32;\n        if rnd() & 1 != 0 { IT[k].x378_itemColl.env_flags = 0x18000; }"),
    ptrs=("", ""),
    hints={"arg2": ("(int) (rnd() % 10)", "(rnd() % 10) as i32")},
)

# ------------------------------------------------------------------------------------------ itstar
CFG["itstar"] = dict(
    spec=[("""        float* a = (float*) SPEC[0];
        for (i = 0; i < 6; i++) a[i] = rw();""",
           """            let a = SPECB.0[0].as_mut_ptr() as *mut f32;
            for i in 0..6 { *a.add(i) = rw(); }""")],
    tweak=("", ""),
    ptrs=("", ""),
)

# ---------------------------------------------------------------------------------------- itstarrod
CFG["itstarrod"] = dict(
    spec=[("""        int* a = (int*) SPEC[0];
        a[0] = (int) (rnd() % 4); ((float*) a)[1] = rw(); ((float*) a)[2] = rw(); ((float*) a)[3] = rw();""",
           """            let a = SPECB.0[0].as_mut_ptr() as *mut i32;
            *a = (rnd() % 4) as i32; *(a.add(1) as *mut f32) = rw(); *(a.add(2) as *mut f32) = rw(); *(a.add(3) as *mut f32) = rw();""")],
    tweak=per_item("        IT[k].xD4C = (int) (rnd() % 4) - 1;", "        IT[k].xD4C = (rnd() % 4) as i32 - 1;"),
    ptrs=("", ""),
    hints={"arg3": ("(int) (rnd() % 4)", "(rnd() % 4) as i32")},
)

# ------------------------------------------------------------------------------------ itstarrodstar
CFG["itstarrodstar"] = dict(
    spec=[("""        float* a = (float*) SPEC[0];
        a[0] = rw(); a[1] = rw(); a[2] = rw(); a[3] = rw(); a[4] = rw();
        ((int*) a)[5] = (int) (rnd() % 6); ((int*) a)[6] = (int) (rnd() % 6); a[7] = rw();""",
           """            let a = SPECB.0[0].as_mut_ptr() as *mut f32;
            *a = rw(); *a.add(1) = rw(); *a.add(2) = rw(); *a.add(3) = rw(); *a.add(4) = rw();
            *(a.add(5) as *mut i32) = (rnd() % 6) as i32; *(a.add(6) as *mut i32) = (rnd() % 6) as i32; *a.add(7) = rw();""")],
    tweak=per_item("        IT[k].xDD4_itemVar.starrodstar.x0 = (int) (rnd() & 1);\n        IT[k].xD44_lifeTimer = (float) (rnd() % 4);\n        IT[k].xDD4_itemVar.starrodstar.x4 = rg();",
                   "        IT[k].xDD4_itemVar.starrodstar.x0 = (rnd() & 1) as i32;\n        IT[k].xD44_lifeTimer = (rnd() % 4) as f32;\n        IT[k].xDD4_itemVar.starrodstar.x4 = rg();"),
    ptrs=("", ""),
    hints={"arg2": ("(int) (rnd() & 1)", "(rnd() & 1) as i32")},
)

# ---------------------------------------------------------------------------------------- itsuikun
_pk = "IT[k].xDD4_itemVar.pokemon"
CFG["itsuikun"] = dict(
    spec=[("""        float* a = (float*) SPEC[0];
        a[0] = rw(); ((int*) a)[1] = (int) (rnd() % 5);""",
           """            let a = SPECB.0[0].as_mut_ptr() as *mut f32;
            *a = rw(); *(a.add(1) as *mut i32) = (rnd() % 5) as i32;""")],
    tweak=per_item(f"""        {_pk}.timer = (int) (rnd() % 3);
        IT[k].xDAC_itcmd_var0 = (int) (rnd() & 1);
        IT[k].xDB0_itcmd_var1 = (int) (rnd() & 1);
        IT[k].xDB4_itcmd_var2 = (int) (rnd() & 1);""",
                   """        IT[k].xDD4_itemVar.pokemon.timer = (rnd() % 3) as i32;
        IT[k].xDAC_itcmd_var0 = (rnd() & 1) as u32;
        IT[k].xDB0_itcmd_var1 = (rnd() & 1) as u32;
        IT[k].xDB4_itcmd_var2 = (rnd() & 1) as u32;"""),
    ptrs=("", ""),
)

# ----------------------------------------------------------------------------------------- itsword
_sw = "IT[k].xDD4_itemVar.sword"
CFG["itsword"] = dict(
    spec=[("""        float* a = (float*) SPEC[0];
        a[3] = rw(); a[4] = rw(); a[5] = rw(); ((int*) a)[6] = (rnd() & 1) ? 4 : (int) (rnd() % 3);""",
           """            let a = SPECB.0[0].as_mut_ptr() as *mut f32;
            *a.add(3) = rw(); *a.add(4) = rw(); *a.add(5) = rw();
            *(a.add(6) as *mut i32) = if rnd() & 1 != 0 { 4 } else { (rnd() % 3) as i32 };""")],
    tweak=per_item(f"""        {_sw}.x4 = (int) (rnd() % 6); {_sw}.x8 = (int) (rnd() % 6); {_sw}.xC = (int) (rnd() % 3) - 1;
        {_sw}.x10 = rg(); {_sw}.x14 = rg(); {_sw}.x18 = rg();
        {_sw}.x1C = (int) (rnd() % 6); {_sw}.x20 = (int) (rnd() % 6); {_sw}.x24 = (int) (rnd() % 3) - 1;
        {_sw}.x28 = rg(); {_sw}.x2C = rg(); {_sw}.x30 = rg();
        {_sw}.x34 = 4 * (int) (rnd() % 2); {_sw}.x38 = 4 * (int) (rnd() % 2); {_sw}.x3C = (int) (rnd() % 3) - 1;
        {_sw}.x40 = rg(); {_sw}.x44 = rg(); {_sw}.x48 = rg(); {_sw}.x4C = rg();""",
                   """        let sw = &mut IT[k].xDD4_itemVar.sword;
        sw.x4 = (rnd() % 6) as i32; sw.x8 = (rnd() % 6) as i32; sw.xC = (rnd() % 3) as i32 - 1;
        sw.x10 = rg(); sw.x14 = rg(); sw.x18 = rg();
        sw.x1C = (rnd() % 6) as i32; sw.x20 = (rnd() % 6) as i32; sw.x24 = (rnd() % 3) as i32 - 1;
        sw.x28 = rg(); sw.x2C = rg(); sw.x30 = rg();
        sw.x34 = 4 * (rnd() % 2) as i32; sw.x38 = 4 * (rnd() % 2) as i32; sw.x3C = (rnd() % 3) as i32 - 1;
        sw.x40 = rg(); sw.x44 = rg(); sw.x48 = rg(); sw.x4C = rg();"""),
    ptrs=("", ""),
    hints={"arg1": ("(int) (rnd() % 6)", "(rnd() % 6) as i32")},
    over={"it_802852B8": (
        "{ float o1 = rw(); float o2 = rw(); it_802852B8(&GO[0], &o1, &o2); ret = B(o1) + 31u * B(o2); }",
        "{ let mut o1 = rw(); let mut o2 = rw(); m::it_802852B8(&raw mut GO[0], &mut o1, &mut o2); ret = o1.to_bits().wrapping_add(31u32.wrapping_mul(o2.to_bits())); }")},
    extra_ops=[("it_802852B8(NULL, NULL)", "it_802852B8(&GO[0], 0, 0);", "m::it_802852B8(&raw mut GO[0], null_mut(), null_mut());")],
)

# ------------------------------------------------------------------------------------------ ittaru
_tr = "IT[k].xDD4_itemVar.taru"
_ENV = "(rnd() % 5 == 0 ? 0 : (rnd() & 1) ? 0x18000 : (rnd() & 1) ? 0x3F : (rnd() & 1) ? 0xFC0 : 0x18000 | 0x3F)"
CFG["ittaru"] = dict(
    spec=[("""        int* a = (int*) SPEC[0];
        a[0] = (int) (rnd() % 4); a[1] = (int) (rnd() % 4);
        for (i = 2; i < 14; i++) ((float*) a)[i] = rw();
        ((float*) a)[2] = (float) (rnd() % 3); ((float*) a)[3] = (float) (rnd() % 120);""",
           """            let a = SPECB.0[0].as_mut_ptr() as *mut i32;
            *a = (rnd() % 4) as i32; *a.add(1) = (rnd() % 4) as i32;
            for i in 2..14 { *(a.add(i) as *mut f32) = rw(); }
            *(a.add(2) as *mut f32) = (rnd() % 3) as f32; *(a.add(3) as *mut f32) = (rnd() % 120) as f32;""")],
    tweak=per_item(f"""        {_tr}.xDD4 = (int) (rnd() % 3 == 0);
        {_tr}.xDD8 = (int) (rnd() % 4) - 1;
        {_tr}.xDDC = rg(); {_tr}.xDE0 = rg(); {_tr}.xDE4 = rg();
        {_tr}.xDE8.x = rg(); {_tr}.xDE8.y = rg(); {_tr}.xDE8.z = rg();
        IT[k].xDAC_itcmd_var0 = (u32) (rnd() % 3);
        IT[k].xDB0_itcmd_var1 = (u32) (rnd() & 1);
        IT[k].msid = (int) (rnd() % 8);
        IT[k].xC9C = (int) (rnd() % 120);
        IT[k].xD44_lifeTimer = (float) (rnd() % 4);
        IT[k].facing_dir = (rnd() % 3 == 0) ? rw() : ((rnd() & 1) ? 1.0f : -1.0f);
        IT[k].ground_or_air = (rnd() & 1) ? GA_Air : GA_Ground;
        IT[k].x378_itemColl.env_flags = {_ENV};
{MD_CSET}""",
                   """        let tr = &mut IT[k].xDD4_itemVar.taru;
        tr.xDD4 = (rnd() % 3 == 0) as i32;
        tr.xDD8 = (rnd() % 4) as i32 - 1;
        tr.xDDC = rg(); tr.xDE0 = rg(); tr.xDE4 = rg();
        tr.xDE8.x = rg(); tr.xDE8.y = rg(); tr.xDE8.z = rg();
        IT[k].xDAC_itcmd_var0 = (rnd() % 3) as u32;
        IT[k].xDB0_itcmd_var1 = (rnd() & 1) as u32;
        IT[k].msid = (rnd() % 8) as i32;
        IT[k].xC9C = (rnd() % 120) as i32;
        IT[k].xD44_lifeTimer = (rnd() % 4) as f32;
        IT[k].facing_dir = if rnd() % 3 == 0 { rw() } else if rnd() & 1 != 0 { 1.0 } else { -1.0 };
        IT[k].ground_or_air = if rnd() & 1 != 0 { GA_Air } else { GA_Ground };
        IT[k].x378_itemColl.env_flags = if rnd() % 5 == 0 { 0 } else if rnd() & 1 != 0 { 0x18000 } else if rnd() & 1 != 0 { 0x3F } else if rnd() & 1 != 0 { 0xFC0 } else { 0x18000 | 0x3F };
""" + MD_RSET),
    ptrs=("", ""),
    c_decl=MD_CDECL, rs_decl=MD_RDECL, c_preg=MD_CPREG, rs_preg=MD_RPREG,
)

# --------------------------------------------------------------------------------------- ittarucann
_tc = "IT[k].xDD4_itemVar.tarucann"
CFG["ittarucann"] = dict(
    spec=[("""        float* a = (float*) SPEC[0];
        for (i = 0; i < 21; i++) a[i] = rw();
        ((int*) a)[10] = (int) (rnd() % 5); ((int*) a)[11] = (int) (rnd() % 5); ((int*) a)[12] = rnd() % 20; ((int*) a)[13] = (int) (rnd() % 400);
        for (i = 14; i < 18; i++) ((int*) a)[i] = (int) (rnd() % 10);""",
           """            let a = SPECB.0[0].as_mut_ptr() as *mut f32;
            for i in 0..21 { *a.add(i) = rw(); }
            let ai = a as *mut i32;
            *ai.add(10) = (rnd() % 5) as i32; *ai.add(11) = (rnd() % 5) as i32; *ai.add(12) = (rnd() % 20) as i32; *ai.add(13) = (rnd() % 400) as i32;
            for i in 14..18 { *ai.add(i) = (rnd() % 10) as i32; }""")],
    tweak=per_item(f"""        {_tc}.x4 = (int) (rnd() % 4) - 1;
        {_tc}.x8 = rg(); {_tc}.xC = rg(); {_tc}.x10 = rg(); {_tc}.x14.x = rg(); {_tc}.x14.y = rg(); {_tc}.x14.z = rg();
        {_tc}.x20 = (rnd() & 1) ? &GO[(k + 1) % 3] : 0;
        {_tc}.x24 = (int) (rnd() % 4); {_tc}.x28 = rg();
        IT[k].grab_victim = (rnd() & 1) ? &GO[(k + 2) % 3] : 0;
        IT[k].xDAC_itcmd_var0 = (u32) (rnd() % 3);
        IT[k].xDB0_itcmd_var1 = (u32) (rnd() & 1);
        IT[k].msid = (int) (rnd() % 8);
        IT[k].xD44_lifeTimer = (float) (rnd() % 4);
        IT[k].facing_dir = (rnd() % 3 == 0) ? rw() : ((rnd() & 1) ? 1.0f : -1.0f);
        IT[k].ground_or_air = (rnd() & 1) ? GA_Air : GA_Ground;
        IT[k].x378_itemColl.env_flags = {_ENV};
{MD_CSET}""",
                   """        let tc = &mut IT[k].xDD4_itemVar.tarucann;
        tc.x4 = (rnd() % 4) as i32 - 1;
        tc.x8 = rg(); tc.xC = rg(); tc.x10 = rg(); tc.x14.x = rg(); tc.x14.y = rg(); tc.x14.z = rg();
        tc.x20 = if rnd() & 1 != 0 { &raw mut GO[(k + 1) % 3] } else { null_mut() };
        tc.x24 = (rnd() % 4) as i32; tc.x28 = rg();
        IT[k].grab_victim = if rnd() & 1 != 0 { &raw mut GO[(k + 2) % 3] } else { null_mut() };
        IT[k].xDAC_itcmd_var0 = (rnd() % 3) as u32;
        IT[k].xDB0_itcmd_var1 = (rnd() & 1) as u32;
        IT[k].msid = (rnd() % 8) as i32;
        IT[k].xD44_lifeTimer = (rnd() % 4) as f32;
        IT[k].facing_dir = if rnd() % 3 == 0 { rw() } else if rnd() & 1 != 0 { 1.0 } else { -1.0 };
        IT[k].ground_or_air = if rnd() & 1 != 0 { GA_Air } else { GA_Ground };
        IT[k].x378_itemColl.env_flags = if rnd() % 5 == 0 { 0 } else if rnd() & 1 != 0 { 0x18000 } else if rnd() & 1 != 0 { 0x3F } else if rnd() & 1 != 0 { 0xFC0 } else { 0x18000 | 0x3F };
""" + MD_RSET),
    ptrs=("""    for (k = 0; k < 3; k++) { HM(PI(%s.x20)); %s.x20 = 0; HM(PI(IT[k].grab_victim)); IT[k].grab_victim = 0; }""" % (_tc, _tc),
          """    for k in 0..3 { hm(pi(IT[k].xDD4_itemVar.tarucann.x20)); IT[k].xDD4_itemVar.tarucann.x20 = null_mut(); hm(pi(IT[k].grab_victim)); IT[k].grab_victim = null_mut(); }"""),
    c_decl=MD_CDECL, rs_decl=MD_RDECL, c_preg=MD_CPREG, rs_preg=MD_RPREG,
)
