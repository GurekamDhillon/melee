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

# ----------------------------------------------------------------------------------- shard 777-784
CFG["itporygon2"] = dict(spec=[("        ;", "            ")], tweak=("", ""), ptrs=("", ""))
CFG["itrabbitc"] = dict(spec=[("        ;", "            ")], tweak=("", ""), ptrs=("", ""))
CFG["itraikou"] = dict(
    spec=[("""        itPokemonAttributes* a = (itPokemonAttributes*) SPEC[0];
        a->x0 = rw(); a->timer = (int) (rnd() % 5);""",
           """            let a = SPECB.0[0].as_mut_ptr() as *mut itPokemonAttributes;
            (*a).x0 = rw(); (*a).timer = (rnd() % 5) as i32;""")],
    tweak=per_item("        IT[k].xDD4_itemVar.pokemon.timer = (int) (rnd() % 3);",
                   "        IT[k].xDD4_itemVar.pokemon.timer = (rnd() % 3) as i32;"),
    ptrs=("", ""),
)

_sm = "IT[k].xDD4_itemVar.samusmissile"
CFG["itsamusmissile"] = dict(
    spec=[("""        itSamusMissileAttributes* a = (itSamusMissileAttributes*) SPEC[0];
        a->x0 = rw(); a->x4 = (float) (rnd() % 40); a->x8 = (float) (rnd() % 10); a->xC = rw(); a->x10 = rw(); a->x14 = rw();
        a->x18 = rw(); a->x1C = rw(); a->x20 = rw(); a->x24 = (float) (rnd() % 40); a->x28 = (float) (rnd() % 10);
        a->x2C = rw(); a->x30 = rw(); a->x34 = rw();""",
           """            let a = SPECB.0[0].as_mut_ptr() as *mut itSamusMissileAttributes;
            (*a).x0 = rw(); (*a).x4 = (rnd() % 40) as f32; (*a).x8 = (rnd() % 10) as f32; (*a).xC = rw(); (*a).x10 = rw(); (*a).x14 = rw();
            (*a).x18 = rw(); (*a).x1C = rw(); (*a).x20 = rw(); (*a).x24 = (rnd() % 40) as f32; (*a).x28 = (rnd() % 10) as f32;
            (*a).x2C = rw(); (*a).x30 = rw(); (*a).x34 = rw();""")],
    tweak=per_item(
        f"""        IT[k].xD44_lifeTimer = (float) ((int) (rnd() % 40) - 3);
        {_sm}.is_smash_missile = (int) (rnd() & 1);
        {_sm}.owner = (rnd() & 1) ? &GO[(k + 1) % 3] : 0;
        {_sm}.x4 = (int) (rnd() % 4);
        {_sm}.x8 = rg();
        {_sm}.x10 = 0;
        if (rnd() & 1) IT[k].xDB8_itcmd_var3 = 0;
        if (rnd() & 1) IT[k].xDAC_itcmd_var0 = 0;""",
        f"""        IT[k].xD44_lifeTimer = ((rnd() % 40) as i32 - 3) as f32;
        let sm = &mut IT[k].xDD4_itemVar.samusmissile;
        sm.is_smash_missile = (rnd() & 1) as i32;
        sm.owner = if rnd() & 1 != 0 {{ &raw mut GO[(k + 1) % 3] }} else {{ null_mut() }};
        sm.x4 = (rnd() % 4) as i32;
        sm.x8 = rg();
        sm.x10 = null_mut();
        if rnd() & 1 != 0 {{ IT[k].xDB8_itcmd_var3 = 0; }}
        if rnd() & 1 != 0 {{ IT[k].xDAC_itcmd_var0 = 0; }}"""),
    ptrs=("""    for (k = 0; k < 3; k++) { HM(PI(%s.owner)); %s.owner = 0; HM(PI(%s.x10)); %s.x10 = 0; }""" % ((_sm.replace("IT[k]","IT[k]"),)*4),
          """    for k in 0..3 { let sm = &mut IT[k].xDD4_itemVar.samusmissile; hm(pi(sm.owner)); sm.owner = null_mut(); hm(pi(sm.x10)); sm.x10 = null_mut(); }"""),
    hints={"is_smash_missile": ("(int) (fv() & 1)", "(fv() & 1) as i32")},
)

_sb = "IT[k].xDD4_itemVar.samusbomb"
CFG["itsamusbomb"] = dict(
    spec=[("""        itSamusBombAttributes* a = (itSamusBombAttributes*) SPEC[0];
        a->x0 = rw(); a->x4 = (float) (rnd() % 30); a->x8 = rw(); a->xC = (float) (rnd() % 5) / 2.0f; a->x10 = rw(); a->x14 = rw(); a->x18 = rw();""",
           """            let a = SPECB.0[0].as_mut_ptr() as *mut itSamusBombAttributes;
            (*a).x0 = rw(); (*a).x4 = (rnd() % 30) as f32; (*a).x8 = rw(); (*a).xC = (rnd() % 5) as f32 / 2.0; (*a).x10 = rw(); (*a).x14 = rw(); (*a).x18 = rw();""")],
    tweak=per_item(
        f"""        IT[k].xD44_lifeTimer = (float) ((int) (rnd() % 30) - 2);
        {_sb}.x0 = (int) (rnd() & 1);
        {_sb}.owner = (rnd() & 1) ? &GO[(k + 1) % 3] : 0;
        if (rnd() & 1) {{ IT[k].x7C.x = (float) ((int) (rnd() % 7) - 3) / 2.0f; IT[k].x7C.y = (float) ((int) (rnd() % 7) - 3) / 2.0f; }}
        if (rnd() & 1) IT[k].x40_vel.x = 0.0f;
        if (k == 0 && (rnd() & 1)) {{
            float m = (float) (rnd() % 4 + 1) / 2.0f;
            union {{ float f; unsigned u; }} cv;
            IT[k].x7C.x = 3.0f * m; IT[k].x7C.y = 4.0f * m;
            cv.f = 5.0f * m; cv.u += (unsigned) ((int) (rnd() % 3) - 1);
            ((itSamusBombAttributes*) SPEC[0])->xC = cv.f;
        }}""",
        """        let sb = &mut IT[k].xDD4_itemVar.samusbomb;
        IT[k].xD44_lifeTimer = ((rnd() % 30) as i32 - 2) as f32;
        sb.x0 = (rnd() & 1) as i32;
        sb.owner = if rnd() & 1 != 0 { &raw mut GO[(k + 1) % 3] } else { null_mut() };
        if rnd() & 1 != 0 { IT[k].x7C.x = ((rnd() % 7) as i32 - 3) as f32 / 2.0; IT[k].x7C.y = ((rnd() % 7) as i32 - 3) as f32 / 2.0; }
        if rnd() & 1 != 0 { IT[k].x40_vel.x = 0.0; }
        if k == 0 && rnd() & 1 != 0 {
            let m = (rnd() % 4 + 1) as f32 / 2.0;
            IT[k].x7C.x = 3.0 * m; IT[k].x7C.y = 4.0 * m;
            let d = (rnd() % 3) as i32 - 1;
            (*(SPECB.0[0].as_mut_ptr() as *mut itSamusBombAttributes)).xC = f32::from_bits((5.0 * m).to_bits().wrapping_add(d as u32));
        }"""),
    ptrs=("""    for (k = 0; k < 3; k++) { HM(PI(%s.owner)); %s.owner = 0; }""" % (_sb, _sb),
          """    for k in 0..3 { let sb = &mut IT[k].xDD4_itemVar.samusbomb; hm(pi(sb.owner)); sb.owner = null_mut(); }"""),
)

_cs = "IT[k].xDD4_itemVar.samuschargeshot"
CFG["itsamuschargeshot"] = dict(
    spec=[("""        itSamusChargeShot_Attributes* a = (itSamusChargeShot_Attributes*) SPEC[0];
        a->lifetime = rw(); a->x4 = fv(); a->x8 = rw(); a->xC = rw(); a->x10 = rw(); a->x14 = rw(); a->x18 = rw(); a->x1C = rw();""",
           """            let a = SPECB.0[0].as_mut_ptr() as *mut itSamusChargeShot_Attributes;
            (*a).lifetime = rw(); (*a).x4 = fv(); (*a).x8 = rw(); (*a).xC = rw(); (*a).x10 = rw(); (*a).x14 = rw(); (*a).x18 = rw(); (*a).x1C = rw();""")],
    tweak=per_item(
        f"""        switch (rnd() % 3) {{ case 0: IT[k].kind = It_Kind_Samus_Charge; break; case 1: IT[k].kind = It_Kind_Kirby_SamusCharge; break; default: break; }}
        {_cs}.xE00 = (rnd() & 3) ? IT[k].owner : &GO[2];
        {_cs}.xDEC = (int) (rnd() % 11) - 2;
        {_cs}.xDF0 = (int) (rnd() % 9);
        {_cs}.xDF4 = (int) (rnd() % 5);
        {_cs}.xDE8 = (int) (rnd() & 1);
        {_cs}.xDD8 = rw();
        {_cs}.xDDC = rw();
        {_cs}.xDE4 = rw();
        JO[3 + k].child = &JO[40 + k];""",
        """        match rnd() % 3 { 0 => IT[k].kind = It_Kind_Samus_Charge, 1 => IT[k].kind = It_Kind_Kirby_SamusCharge, _ => {} }
        let cs = &mut IT[k].xDD4_itemVar.samuschargeshot;
        cs.xE00 = if rnd() & 3 != 0 { IT[k].owner } else { &raw mut GO[2] };
        cs.xDEC = (rnd() % 11) as i32 - 2;
        cs.xDF0 = (rnd() % 9) as i32;
        cs.xDF4 = (rnd() % 5) as i32;
        cs.xDE8 = (rnd() & 1) as i32;
        cs.xDD8 = rw();
        cs.xDDC = rw();
        cs.xDE4 = rw();
        JO[3 + k].child = &raw mut JO[40 + k];"""),
    ptrs=("""    for (k = 0; k < 3; k++) { HM(PI(%s.xE00)); %s.xE00 = 0; }""" % (_cs, _cs),
          """    for k in 0..3 { let cs = &mut IT[k].xDD4_itemVar.samuschargeshot; hm(pi(cs.xE00)); cs.xE00 = null_mut(); }"""),
    over={"it_802B5518": ("ret = (u32) it_802B5518(&GO[0], &IT[0].x378_itemColl);",
                          "ret = m::it_802B5518(&raw mut GO[0], &raw mut IT[0].x378_itemColl) as u32;")},
)

_rs = "IT[k].xDD4_itemVar.rshell"
CFG["itrshell"] = dict(
    spec=[("""        itRShell_Attrs* a = (itRShell_Attrs*) SPEC[0];
        a->x0 = rw(); a->x4 = rw(); a->x8 = rw(); a->xC = (float) (rnd() % 4) / 2.0f; a->x10 = (float) (rnd() % 5) / 2.0f;
        a->x14.x = rw(); a->x14.y = rw(); a->x14.z = rw(); a->x20 = rw(); a->x24 = rw(); a->x28 = rw(); a->x2C = rw();
        a->x38 = rw(); a->x3C = rw(); a->x40 = rw(); a->x44 = rw(); a->x48.x = rw(); a->x48.y = rw(); a->x48.z = rw();
        a->x54 = (int) (rnd() % 4);""",
           """            let a = SPECB.0[0].as_mut_ptr() as *mut m::itRShell_Attrs;
            (*a).x0 = rw(); (*a).x4 = rw(); (*a).x8 = rw(); (*a).xC = (rnd() % 4) as f32 / 2.0; (*a).x10 = (rnd() % 5) as f32 / 2.0;
            (*a).x14.x = rw(); (*a).x14.y = rw(); (*a).x14.z = rw(); (*a).x20 = rw(); (*a).x24 = rw(); (*a).x28 = rw(); (*a).x2C = rw();
            (*a).x38 = rw(); (*a).x3C = rw(); (*a).x40 = rw(); (*a).x44 = rw(); (*a).x48.x = rw(); (*a).x48.y = rw(); (*a).x48.z = rw();
            (*a).x54 = (rnd() % 4) as i32;""")],
    tweak=per_item(
        f"""        IT[k].msid = (int) (rnd() % 10) - 1;
        {_rs}.xDD4 = (float) ((int) (rnd() % 4) - 1);
        {_rs}.xDD8 = (float) ((int) (rnd() % 4) - 1);
        {_rs}.xDDC = (float) ((int) (rnd() % 4) - 1);
        {_rs}.xDE0 = (float) ((int) (rnd() % 5) - 2);
        {_rs}.xDE4 = (float) ((int) (rnd() % 4) - 1);
        {_rs}.xDE8 = (int) (rnd() & 1);
        {_rs}.xDEC = (int) (rnd() % 4);
        {_rs}.xDF0 = (int) (rnd() & 1);
        {_rs}.xDF4_b0 = rnd() & 1;
        IT[k].xDCD_flag.b5 = rnd() & 1;
        if (rnd() & 1) IT[k].x40_vel.x = (float) ((int) (rnd() % 5) - 2) / 2.0f;""",
        """        let rs = &mut IT[k].xDD4_itemVar.rshell;
        IT[k].msid = (rnd() % 10) as i32 - 1;
        rs.xDD4 = ((rnd() % 4) as i32 - 1) as f32;
        rs.xDD8 = ((rnd() % 4) as i32 - 1) as f32;
        rs.xDDC = ((rnd() % 4) as i32 - 1) as f32;
        rs.xDE0 = ((rnd() % 5) as i32 - 2) as f32;
        rs.xDE4 = ((rnd() % 4) as i32 - 1) as f32;
        rs.xDE8 = (rnd() & 1) as i32;
        rs.xDEC = (rnd() % 4) as i32;
        rs.xDF0 = (rnd() & 1) as i32;
        rs.set_xDF4_b0((rnd() & 1) as u8);
        IT[k].xDCD_flag.anon_0.set_b5((rnd() & 1) as u8);
        if rnd() & 1 != 0 { IT[k].x40_vel.x = ((rnd() % 5) as i32 - 2) as f32 / 2.0; }"""),
    ptrs=("""    for (k = 0; k < 3; k++) { HM(%s.xDF4_b0); zero(&%s, sizeof %s); HM(IT[k].xDCD_flag.b5); *(unsigned char*) &IT[k].xDCD_flag = 0; }""" % (_rs, _rs, _rs),
          """    for k in 0..3 { let rs = &mut IT[k].xDD4_itemVar.rshell; hm(rs.xDF4_b0() as u32); core::ptr::write_bytes(rs as *mut _ as *mut u8, 0, core::mem::size_of::<itRShell_ItemVars>()); hm(IT[k].xDCD_flag.anon_0.b5() as u32); *(&raw mut IT[k].xDCD_flag as *mut u8) = 0; }"""),
)

import specs_cfg_grapple
specs_cfg_grapple.register(CFG)
