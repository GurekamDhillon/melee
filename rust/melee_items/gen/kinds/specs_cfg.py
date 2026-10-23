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

# ------------------------------------------------------------------------------------------- itmato
_mt = "IT[k].xDD4_itemVar.mato"
CFG["itmato"] = dict(
    spec=[("        (void) 0;", "            ();".replace("();", ""))],
    tweak=per_item(
        f"""        i = (int) (rnd() % 3);
        {_mt}.x0 = (rnd() & 1) ? &JO[3 + i] : 0;
        i = (int) (rnd() % 3);
        {_mt}.x4 = (rnd() & 1) ? &JO[3 + i] : 0;""",
        """        let i = (rnd() % 3) as usize;
        IT[k].xDD4_itemVar.mato.x0 = if rnd() & 1 != 0 { &raw mut JO[3 + i] } else { null_mut() };
        let i = (rnd() % 3) as usize;
        IT[k].xDD4_itemVar.mato.x4 = if rnd() & 1 != 0 { &raw mut JO[3 + i] } else { null_mut() };"""),
    ptrs=("""    for (k = 0; k < 3; k++) {
        HM(PI(%s.x0)); HM(PI(%s.x4)); %s.x0 = 0; %s.x4 = 0;
    }""" % ((_mt,) * 4),
          """    for k in 0..3 {
        let mt = &mut IT[k].xDD4_itemVar.mato;
        hm(pi(mt.x0)); hm(pi(mt.x4)); mt.x0 = null_mut(); mt.x4 = null_mut();
    }"""),
)

# ----------------------------------------------------------------------------------------- itmetalb
CFG["itmetalb"] = dict(
    spec=[("        (void) 0;", "")],
    tweak=("        GO[2].classifier = (rnd() & 1) ? 4 : (u16) (rnd() % 8);",
           "        GO[2].classifier = if rnd() & 1 != 0 { 4 } else { (rnd() % 8) as u16 };"),
    ptrs=("", ""),
)

# ---------------------------------------------------------------------------------------- itmetamon
CFG["itmetamon"] = dict(
    spec=[("""        float* a = (float*) SPEC[0];
        a[0] = rw();""",
           """            let a = SPECB.0[0].as_mut_ptr() as *mut f32;
            *a = rw();""")],
    tweak=("", ""),
    ptrs=("    HM(PI(it_804D6D50)); it_804D6D50 = 0;",
          "    hm(pi(m::it_804D6D50)); m::it_804D6D50 = null_mut();"),
)

# -------------------------------------------------------------------------------------------- itmew
CFG["itmew"] = dict(
    spec=[("""        MewVars* a = (MewVars*) SPEC[0];
        a->x0 = rw(); a->x4 = rw(); a->x8 = rw(); a->xC = rw();""",
           """            let a = SPECB.0[0].as_mut_ptr() as *mut MewVars;
            (*a).x0 = rw(); (*a).x4 = rw(); (*a).x8 = rw(); (*a).xC = rw();""")],
    tweak=("", ""),
    ptrs=("", ""),
)

# ------------------------------------------------------------------------------------ itmewtwodisable
_md = "IT[k].xDD4_itemVar.mdisable"
CFG["itmewtwodisable"] = dict(
    spec=[("""        itMDisableAttributes* a = (itMDisableAttributes*) SPEC[0];
        a->lifetime = rw(); a->x_vel = rw();""",
           """            let a = SPECB.0[0].as_mut_ptr() as *mut itMDisableAttributes;
            (*a).lifetime = rw(); (*a).x_vel = rw();""")],
    tweak=per_item(
        f"""        i = (int) (rnd() % 3);
        {_md}.owner = (rnd() & 1) ? &GO[i] : 0;""",
        """        let i = (rnd() % 3) as usize;
        IT[k].xDD4_itemVar.mdisable.owner = if rnd() & 1 != 0 { &raw mut GO[i] } else { null_mut() };"""),
    ptrs=("""    for (k = 0; k < 3; k++) { HM(PI(%s.owner)); %s.owner = 0; }""" % (_md, _md),
          """    for k in 0..3 { hm(pi(IT[k].xDD4_itemVar.mdisable.owner)); IT[k].xDD4_itemVar.mdisable.owner = null_mut(); }"""),
)

# ---------------------------------------------------------------------------------------- itmball
CFG["itmball"] = dict(
    spec=[("""        itMBallAttributes* a = (itMBallAttributes*) SPEC[0];
        a->x0 = (float) (rnd() % 20); a->x4 = (float) (rnd() % 20);""",
           """            let a = SPECB.0[0].as_mut_ptr() as *mut itMBallAttributes;
            (*a).x0 = (rnd() % 20) as f32; (*a).x4 = (rnd() % 20) as f32;""")],
    tweak=("""        for (k = 0; k < 3; k++) { IT[k].xDD4_itemVar.mball.b0 = rnd() & 1; IT[k].xDD4_itemVar.mball.b1 = rnd() & 1; }
        Item_804A0C64.x1C = (int) (rnd() % 20);
        if (rnd() & 1) IT[0].xD44_lifeTimer = ((itMBallAttributes*) SPEC[0])->x4;
        else IT[0].xD44_lifeTimer = (float) (rnd() % 4) - 1.0f;""",
           """        for k in 0..3 { IT[k].xDD4_itemVar.mball.set_b0((rnd() & 1) as u8); IT[k].xDD4_itemVar.mball.set_b1((rnd() & 1) as u8); }
        m::ext::Item_804A0C64.x1C = (rnd() % 20) as i32;
        if rnd() & 1 != 0 {
            IT[0].xD44_lifeTimer = (*(SPECB.0[0].as_mut_ptr() as *mut itMBallAttributes)).x4;
        } else {
            IT[0].xD44_lifeTimer = (rnd() % 4) as f32 - 1.0;
        }"""),
    ptrs=("""    HM((u32) Item_804A0C64.x1C);
    for (k = 0; k < 3; k++) { HM(IT[k].xDD4_itemVar.mball.b0); HM(IT[k].xDD4_itemVar.mball.b1); zero(&IT[k].xDD4_itemVar.mball, sizeof IT[k].xDD4_itemVar.mball); }""",
          """    hm(m::ext::Item_804A0C64.x1C as u32);
    for k in 0..3 { let mb = &mut IT[k].xDD4_itemVar.mball; hm(mb.b0() as u32); hm(mb.b1() as u32); core::ptr::write_bytes(mb as *mut _ as *mut u8, 0, core::mem::size_of::<itMBall_ItemVars>()); }"""),
)

# ---------------------------------------------------------------------------------------- itmsbomb
CFG["itmsbomb"] = dict(
    spec=[("""        itMsBomb_Attrs* a = (itMsBomb_Attrs*) SPEC[0];
        a->x0 = rw(); a->x4 = rw(); a->x8.top = rw(); a->x8.bottom = rw(); a->x8.right = rw(); a->x8.left = rw();""",
           """            let a = SPECB.0[0].as_mut_ptr() as *mut m::itMsBomb_Attrs;
            (*a).x0 = rw(); (*a).x4 = rw(); (*a).x8.top = rw(); (*a).x8.bottom = rw(); (*a).x8.right = rw(); (*a).x8.left = rw();""")],
    tweak=("""        for (k = 0; k < 3; k++) { IT[k].xDD4_itemVar.msbomb.xDD4_b0 = rnd() & 1; }
        IT[0].msid = (int) (rnd() % 8);""",
           """        for k in 0..3 { IT[k].xDD4_itemVar.msbomb.set_xDD4_b0((rnd() & 1) as u8); }
        IT[0].msid = (rnd() % 8) as i32;"""),
    ptrs=("""    for (k = 0; k < 3; k++) { HM(IT[k].xDD4_itemVar.msbomb.xDD4_b0); zero(&IT[k].xDD4_itemVar.msbomb, sizeof IT[k].xDD4_itemVar.msbomb); }""",
          """    for k in 0..3 { let mb = &mut IT[k].xDD4_itemVar.msbomb; hm(mb.xDD4_b0() as u32); core::ptr::write_bytes(mb as *mut _ as *mut u8, 0, core::mem::size_of::<itMsBomb_ItemVars>()); }"""),
)

# ---------------------------------------------------------------------------------- itmewtwoshadowball
_sb = "IT[k].xDD4_itemVar.mewtwoshadowball"
CFG["itmewtwoshadowball"] = dict(
    spec=[("""        itMewtwoShadowball_DatAttrs* a = (itMewtwoShadowball_DatAttrs*) SPEC[0];
        a->x0 = rw(); a->x4 = rw(); a->x8 = rw(); a->xC = rw(); a->x10 = rw(); a->x14 = rw(); a->x18 = rw(); a->x1C = rw();
        a->x20 = (int) (rnd() % 6); a->x24 = rw(); a->x28 = rw(); a->x2C = rw(); a->x30 = rw(); a->x34 = rw(); a->x38 = rw(); a->x3C = rw();""",
           """            let a = SPECB.0[0].as_mut_ptr() as *mut itMewtwoShadowball_DatAttrs;
            (*a).x0 = rw(); (*a).x4 = rw(); (*a).x8 = rw(); (*a).xC = rw(); (*a).x10 = rw(); (*a).x14 = rw(); (*a).x18 = rw(); (*a).x1C = rw();
            (*a).x20 = (rnd() % 6) as i32; (*a).x24 = rw(); (*a).x28 = rw(); (*a).x2C = rw(); (*a).x30 = rw(); (*a).x34 = rw(); (*a).x38 = rw(); (*a).x3C = rw();""")],
    tweak=per_item(
        f"""        JO[3 + k].child = &JO[9 + k];
        {_sb}.x0 = rw();
        i = (int) (rnd() % 2000) - 1000;
        {_sb}.x4.x = (float) i / 37.0f; {_sb}.x4.y = rw(); {_sb}.x4.z = rw();
        {_sb}.x10 = rw();
        {_sb}.x14 = (int) (rnd() % 2);
        {_sb}.x18 = (int) (rnd() % 6) - 1;
        {_sb}.x1C = (int) (rnd() % 5) + 1;
        {_sb}.x20 = (int) (rnd() % 5);
        {_sb}.x24 = rnd() % 9;
        {_sb}.x28 = (int) (rnd() % 2);
        {_sb}.x2C = (rnd() & 3) ? &GO[1] : 0;
        {_sb}.x30.x = rw(); {_sb}.x30.y = rw(); {_sb}.x30.z = rw();
        {_sb}.x3C = rw(); {_sb}.x40 = rw(); {_sb}.x44 = rw();
        {_sb}.x48 = (int) (rnd() % 6);
        {_sb}.x4C = (int) (rnd() % 4) - 1;
        {_sb}.x50 = rw(); {_sb}.x54 = rw();
        if (rnd() % 3 == 0) {{ {_sb}.x58.x = 0.0f; {_sb}.x58.y = 0.0f; {_sb}.x58.z = 0.0f; }}
        else {{ {_sb}.x58.x = rw(); {_sb}.x58.y = rw(); {_sb}.x58.z = rw(); }}
        {_sb}.x64 = rw();
        if (k == 0) {{
            i = (int) (rnd() % 3);
            if (i == 0) IT[0].kind = It_Kind_Mewtwo_ShadowBall;
            else if (i == 1) IT[0].kind = It_Kind_Kirby_MewtwoShadowBall;
        }}""",
        """        JO[3 + k].child = &raw mut JO[9 + k];
        let sb = &mut IT[k].xDD4_itemVar.mewtwoshadowball;
        sb.x0 = rw();
        let i = (rnd() % 2000) as i32 - 1000;
        sb.x4.x = i as f32 / 37.0; sb.x4.y = rw(); sb.x4.z = rw();
        sb.x10 = rw();
        sb.x14 = (rnd() % 2) as i32;
        sb.x18 = (rnd() % 6) as i32 - 1;
        sb.x1C = (rnd() % 5) as i32 + 1;
        sb.x20 = (rnd() % 5) as i32;
        sb.x24 = rnd() % 9;
        sb.x28 = (rnd() % 2) as i32;
        sb.x2C = if rnd() & 3 != 0 { &raw mut GO[1] } else { null_mut() };
        sb.x30.x = rw(); sb.x30.y = rw(); sb.x30.z = rw();
        sb.x3C = rw(); sb.x40 = rw(); sb.x44 = rw();
        sb.x48 = (rnd() % 6) as i32;
        sb.x4C = (rnd() % 4) as i32 - 1;
        sb.x50 = rw(); sb.x54 = rw();
        if rnd() % 3 == 0 { sb.x58.x = 0.0; sb.x58.y = 0.0; sb.x58.z = 0.0; }
        else { sb.x58.x = rw(); sb.x58.y = rw(); sb.x58.z = rw(); }
        sb.x64 = rw();
        if k == 0 {
            let i = rnd() % 3;
            if i == 0 { IT[0].kind = It_Kind_Mewtwo_ShadowBall; }
            else if i == 1 { IT[0].kind = It_Kind_Kirby_MewtwoShadowBall; }
        }"""),
    ptrs=("""    for (k = 0; k < 3; k++) { HM(PI(%s.x2C)); %s.x2C = 0; }""" % (_sb, _sb),
          """    for k in 0..3 { hm(pi(IT[k].xDD4_itemVar.mewtwoshadowball.x2C)); IT[k].xDD4_itemVar.mewtwoshadowball.x2C = null_mut(); }"""),
    hints={"max_charge": ("(int) (rnd() % 5) + 1", "((rnd() % 5) as i32 + 1)"),
           "charge": ("(float) (rnd() % 9) * 0.5f - 1.0f", "((rnd() % 9) as f32 * 0.5 - 1.0)")},
    noret=["it_802C4D10"],
    over={"it_802C4F50": ("ret = (u32) it_802C4F50(&GO[0], &IT[0].x378_itemColl);",
                          "ret = m::it_802C4F50(&raw mut GO[0], &raw mut IT[0].x378_itemColl) as u32;")},
)
