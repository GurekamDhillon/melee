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

# ============================================================================== shard 769-776
_NOSPEC = [("        (void) i;", "            let _ = 0;")]


def _all3(c_body, r_body):
    """An attribute-buffer profile applied to the three SPEC buffers (j = 0..2; a = the buffer)."""
    return ("        { int j; for (j = 0; j < 3; j++) {\n" + c_body + "\n        } }",
            "            for j in 0..3 {\n" + r_body + "\n            }")


# ------------------------------------------------------------------------------------ itpeachparasol
CFG["itpeachparasol"] = dict(
    spec=_NOSPEC,
    tweak=("""    if (rnd() & 1) IT[0].msid = 2;
    IT[0].x5CC_currentAnimFrame = (rnd() & 1) ? 16.0f : rw();""",
           """    if rnd() & 1 != 0 {
        IT[0].msid = 2;
    }
    IT[0].x5CC_currentAnimFrame = if rnd() & 1 != 0 { 16.0 } else { rw() };"""),
    ptrs=("", ""),
    # it_803F74F8[i].anim_id: -1 would index it_804D5518[-1] (UB), so the oracle keeps to rows 1 and 2
    hints={"i": ("(int) (1 + rnd() % 2)", "(1 + rnd() % 2) as i32")},
)

# -------------------------------------------------------------------------------------- itpeachtoad
CFG["itpeachtoad"] = dict(
    spec=_NOSPEC,
    c_decl="static const float FR[5] = { 7.0f, 53.0f, 60.0f, 59.0f, 0.0f };",
    rs_decl="const FR: [f32; 5] = [7.0, 53.0, 60.0, 59.0, 0.0];",
    tweak=("""    IT[0].kind = (rnd() & 1) ? It_Kind_Peach_Toad : It_Kind_Peach_Toad + 1 + (int) (rnd() % 40);
    IT[0].x5CC_currentAnimFrame = (rnd() % 6 == 5) ? rw() : FR[rnd() % 5];""",
           """    IT[0].kind = if rnd() & 1 != 0 { It_Kind_Peach_Toad } else { It_Kind_Peach_Toad + 1 + (rnd() % 40) as i32 };
    IT[0].x5CC_currentAnimFrame = if rnd() % 6 == 5 { rw() } else { FR[(rnd() % 5) as usize] };"""),
    ptrs=("", ""),
)

# ---------------------------------------------------------------------------------- itpeachtoadspore
CFG["itpeachtoadspore"] = dict(
    spec=[_all3("            float* a = (float*) SPEC[j];\n            a[0] = rw(); a[1] = rw(); a[2] = rw(); a[3] = rw();",
                "                let a = SPECB.0[j].as_mut_ptr() as *mut f32;\n                *a = rw(); *a.add(1) = rw(); *a.add(2) = rw(); *a.add(3) = rw();")],
    tweak=per_item("""        IT[k].xDAC_itcmd_var0 = (int) (rnd() & 1);
        IT[k].xD44_lifeTimer = (rnd() & 1) ? 1.0f : rw();
        IT[k].kind = (rnd() & 1) ? It_Kind_Peach_ToadSpore : (int) (rnd() % 150);""",
                   """        IT[k].xDAC_itcmd_var0 = (rnd() & 1) as u32;
        IT[k].xD44_lifeTimer = if rnd() & 1 != 0 { 1.0 } else { rw() };
        IT[k].kind = if rnd() & 1 != 0 { It_Kind_Peach_ToadSpore } else { (rnd() % 150) as i32 };"""),
    ptrs=("", ""),
)

# ----------------------------------------------------------------------------------- itpeachturnip
_pt = "IT[k].xDD4_itemVar.peachturnip"
CFG["itpeachturnip"] = dict(
    spec=[_all3("""            int* w = (int*) SPEC[j];
            int m;
            ((float*) SPEC[j])[0] = rw();
            w[1] = (int) (rnd() % 10) - 1;
            for (m = 0; m < 9; m++) { w[2 + 2 * m] = (int) (rnd() % 6); w[3 + 2 * m] = (int) (rnd() % 20); }""",
                """                let w = SPECB.0[j].as_mut_ptr() as *mut i32;
                *(w as *mut f32) = rw();
                *w.add(1) = (rnd() % 10) as i32 - 1;
                for m in 0..9usize {
                    *w.add(2 + 2 * m) = (rnd() % 6) as i32;
                    *w.add(3 + 2 * m) = (rnd() % 20) as i32;
                }""")],
    c_decl="static const int KK[6] = { 99, 6, 7, 12, 5, 99 };",
    rs_decl="const KK: [i32; 6] = [99, 6, 7, 12, 5, 99];",
    tweak=per_item(
        f"""        IT[k].kind = KK[rnd() % 6];
        {_pt}.xDD8 = (int) (rnd() % 10) - 1;
        {_pt}.xDD4.b0 = (int) (rnd() & 1);
        {_pt}.xDDC_damage = (int) (rnd() % 30);
        {_pt}.xDE0_scl = rw();
        {_pt}.xDE4_owner = (rnd() & 1) ? &GO[(k + 1) % 3] : 0;
        {_pt}.xDE8_scl = rw();
        IT[k].xD44_lifeTimer = (rnd() & 1) ? 1.0f : rw();""",
        """        IT[k].kind = KK[(rnd() % 6) as usize];
        let pt = &mut IT[k].xDD4_itemVar.peachturnip;
        pt.xDD8 = (rnd() % 10) as i32 - 1;
        pt.xDD4.anon_0.set_b0((rnd() & 1) as u8);
        pt.xDDC_damage = (rnd() % 30) as i32;
        pt.xDE0_scl = rw();
        pt.xDE4_owner = if rnd() & 1 != 0 { &raw mut GO[(k + 1) % 3] } else { null_mut() };
        pt.xDE8_scl = rw();
        IT[k].xD44_lifeTimer = if rnd() & 1 != 0 { 1.0 } else { rw() };"""),
    ptrs=("""    for (k = 0; k < 3; k++) { HM(PI(%s.xDE4_owner)); %s.xDE4_owner = 0; HM(%s.xDD4.b0); zero(&%s.xDD4, sizeof %s.xDD4); }""" % ((_pt,) * 5),
          """    for k in 0..3 {
        let pt = &mut IT[k].xDD4_itemVar.peachturnip;
        hm(pi(pt.xDE4_owner)); pt.xDE4_owner = null_mut();
        hm(pt.xDD4.anon_0.b0() as u32);
        core::ptr::write_bytes(&raw mut pt.xDD4 as *mut u8, 0, core::mem::size_of_val(&pt.xDD4));
    }"""),
    hints={"kind": ("KK[rnd() % 6]", "KK[(rnd() % 6) as usize]")},
)

# --------------------------------------------------------------------------------- itpikachuthunder
_th = "IT[k].xDD4_itemVar.pikachuthunder"
CFG["itpikachuthunder"] = dict(
    spec=[_all3("""            float* a = (float*) SPEC[j];
            a[0] = rw(); a[1] = (rnd() & 1) ? (float) (1 + rnd() % 40) : rw(); a[2] = rw();""",
                """                let a = SPECB.0[j].as_mut_ptr() as *mut f32;
                *a = rw(); *a.add(1) = if rnd() & 1 != 0 { (1 + rnd() % 40) as f32 } else { rw() }; *a.add(2) = rw();""")],
    tweak=per_item(
        f"""        {_th}.x0 = (int) (rnd() % 3);
        {_th}.x4 = (int) (rnd() & 1);
        {_th}.x8 = (int) (rnd() % 4) - 1;
        {_th}.xC = (rnd() & 1) ? (float) (rnd() % 40) : rw();
        {_th}.x10 = rw();
        {_th}.x14 = (rnd() & 1) ? 3.0f : rw();
        {_th}.x18 = rw();
        {_th}.x1C.x = rw();
        {_th}.x1C.y = (rnd() & 1) ? -(float) (rnd() % 8) : rw();
        {_th}.x1C.z = rw();
        {_th}.x28.x = rw(); {_th}.x28.y = rw(); {_th}.x28.z = rw();
        {_th}.x34 = (rnd() & 1) ? &GO[(k + 1) % 3] : 0;
        {_th}.x38 = (rnd() & 1) ? &GO[(k + 2) % 3] : 0;
        IT[k].xD44_lifeTimer = (rnd() & 1) ? 1.0f : rw();""",
        """        let th = &mut IT[k].xDD4_itemVar.pikachuthunder;
        th.x0 = (rnd() % 3) as i32;
        th.x4 = (rnd() & 1) as i32;
        th.x8 = (rnd() % 4) as i32 - 1;
        th.xC = if rnd() & 1 != 0 { (rnd() % 40) as f32 } else { rw() };
        th.x10 = rw();
        th.x14 = if rnd() & 1 != 0 { 3.0 } else { rw() };
        th.x18 = rw();
        th.x1C.x = rw();
        th.x1C.y = if rnd() & 1 != 0 { -((rnd() % 8) as f32) } else { rw() };
        th.x1C.z = rw();
        th.x28.x = rw(); th.x28.y = rw(); th.x28.z = rw();
        th.x34 = if rnd() & 1 != 0 { &raw mut GO[(k + 1) % 3] } else { null_mut() };
        th.x38 = if rnd() & 1 != 0 { &raw mut GO[(k + 2) % 3] } else { null_mut() };
        IT[k].xD44_lifeTimer = if rnd() & 1 != 0 { 1.0 } else { rw() };"""),
    ptrs=("""    for (k = 0; k < 3; k++) { HM(PI(%s.x34)); HM(PI(%s.x38)); %s.x34 = 0; %s.x38 = 0; }""" % ((_th,) * 4),
          """    for k in 0..3 {
        let th = &mut IT[k].xDD4_itemVar.pikachuthunder;
        hm(pi(th.x34)); hm(pi(th.x38)); th.x34 = null_mut(); th.x38 = null_mut();
    }"""),
    hints={"count": ("(int) (rnd() % 4)", "(rnd() % 4) as i32"), "delay": ("(int) (rnd() % 4)", "(rnd() % 4) as i32")},
)

# -------------------------------------------------------------------------------- itpikachutjoltair
_ja = "IT[k].xDD4_itemVar.pikachujoltair"
CFG["itpikachutjoltair"] = dict(
    spec=_NOSPEC,
    c_decl="static const int KJ[6] = { It_Kind_Kirby_PikachuTJolt_Air, It_Kind_Pikachu_TJolt_Air, It_Kind_Kirby_PichuTJolt_Air, It_Kind_Pichu_TJolt_Air, 5, 0 };",
    rs_decl="const KJ: [i32; 6] = [It_Kind_Kirby_PikachuTJolt_Air, It_Kind_Pikachu_TJolt_Air, It_Kind_Kirby_PichuTJolt_Air, It_Kind_Pichu_TJolt_Air, 5, 0];",
    tweak=per_item(
        f"""        IT[k].kind = KJ[rnd() % 6];
        {_ja}.xDD4 = (rnd() & 1) ? &GO[(k + 1) % 3] : 0;
        {_ja}.xDD8 = (rnd() & 1) ? &GO[(k + 2) % 3] : 0;
        {_ja}.xDE8.x = rw(); {_ja}.xDE8.y = rw(); {_ja}.xDE8.z = rw();
        {_ja}.xDF4 = (int) (rnd() % 30) - 2;""",
        """        IT[k].kind = KJ[(rnd() % 6) as usize];
        let ja = &mut IT[k].xDD4_itemVar.pikachujoltair;
        ja.xDD4 = if rnd() & 1 != 0 { &raw mut GO[(k + 1) % 3] } else { null_mut() };
        ja.xDD8 = if rnd() & 1 != 0 { &raw mut GO[(k + 2) % 3] } else { null_mut() };
        ja.xDE8.x = rw(); ja.xDE8.y = rw(); ja.xDE8.z = rw();
        ja.xDF4 = (rnd() % 30) as i32 - 2;"""),
    ptrs=("""    for (k = 0; k < 3; k++) { HM(PI(%s.xDD4)); HM(PI(%s.xDD8)); %s.xDD4 = 0; %s.xDD8 = 0; }""" % ((_ja,) * 4),
          """    for k in 0..3 {
        let ja = &mut IT[k].xDD4_itemVar.pikachujoltair;
        hm(pi(ja.xDD4)); hm(pi(ja.xDD8)); ja.xDD4 = null_mut(); ja.xDD8 = null_mut();
    }"""),
    # the CollData the port copies: the (filled) one of item 1
    hints={"coll": ("&IT[1].x378_itemColl", "(&raw mut IT[1].x378_itemColl)")},
)

# ------------------------------------------------------------------------------ itpikachutjoltground
_jg = "IT[k].xDD4_itemVar.pikachujoltground"
CFG["itpikachutjoltground"] = dict(
    spec=[_all3("""            float* a = (float*) SPEC[j];
            a[0] = rw(); a[1] = rw(); a[2] = rw(); a[3] = (rnd() & 1) ? 0.0f : rw();""",
                """                let a = SPECB.0[j].as_mut_ptr() as *mut f32;
                *a = rw(); *a.add(1) = rw(); *a.add(2) = rw(); *a.add(3) = if rnd() & 1 != 0 { 0.0 } else { rw() };""")],
    c_decl="static const int KG[6] = { It_Kind_Pikachu_TJolt_Ground, It_Kind_Pichu_TJolt_Ground, It_Kind_Kirby_PikachuTJolt_Ground, It_Kind_Kirby_PichuTJolt_Ground, 5, 0 };",
    rs_decl="const KG: [i32; 6] = [It_Kind_Pikachu_TJolt_Ground, It_Kind_Pichu_TJolt_Ground, It_Kind_Kirby_PikachuTJolt_Ground, It_Kind_Kirby_PichuTJolt_Ground, 5, 0];",
    tweak=per_item(
        f"""        IT[k].kind = KG[rnd() % 6];
        {_jg}.xDD4 = rw();
        {_jg}.xDD8 = (rnd() & 1) ? &GO[(k + 1) % 3] : 0;
        {_jg}.xDDC = (rnd() & 1) ? &GO[(k + 2) % 3] : 0;
        {_jg}.xDE0 = (int) (rnd() % 3);
        {_jg}.xDE4 = (int) (rnd() % 5) - 2;
        {_jg}.xDE8.x = rw(); {_jg}.xDE8.y = rw(); {_jg}.xDE8.z = rw();
        {_jg}.xDF4.x = rw(); {_jg}.xDF4.y = rw(); {_jg}.xDF4.z = rw();
        {_jg}.xE00.x = rw(); {_jg}.xE00.y = rw(); {_jg}.xE00.z = rw();""",
        """        IT[k].kind = KG[(rnd() % 6) as usize];
        let jg = &mut IT[k].xDD4_itemVar.pikachujoltground;
        jg.xDD4 = rw();
        jg.xDD8 = if rnd() & 1 != 0 { &raw mut GO[(k + 1) % 3] } else { null_mut() };
        jg.xDDC = if rnd() & 1 != 0 { &raw mut GO[(k + 2) % 3] } else { null_mut() };
        jg.xDE0 = (rnd() % 3) as i32;
        jg.xDE4 = (rnd() % 5) as i32 - 2;
        jg.xDE8.x = rw(); jg.xDE8.y = rw(); jg.xDE8.z = rw();
        jg.xDF4.x = rw(); jg.xDF4.y = rw(); jg.xDF4.z = rw();
        jg.xE00.x = rw(); jg.xE00.y = rw(); jg.xE00.z = rw();"""),
    ptrs=("""    for (k = 0; k < 3; k++) { HM(PI(%s.xDD8)); HM(PI(%s.xDDC)); %s.xDD8 = 0; %s.xDDC = 0; }""" % ((_jg,) * 4),
          """    for k in 0..3 {
        let jg = &mut IT[k].xDD4_itemVar.pikachujoltground;
        hm(pi(jg.xDD8)); hm(pi(jg.xDDC)); jg.xDD8 = null_mut(); jg.xDDC = null_mut();
    }"""),
)

# --------------------------------------------------------------------------------------- itpippi
CFG["itpippi"] = dict(
    spec=[_all3("""            itPokemonAttributes* a = (itPokemonAttributes*) SPEC[j];
            a->x0 = rw(); a->timer = (int) (rnd() % 5); a->max = (int) (rnd() % 12);
            a->xC = (int) (rnd() % 5); a->x10 = (int) (rnd() % 5); a->x14 = (int) (rnd() % 5); a->x18 = (int) (rnd() % 5);""",
                """                let a = SPECB.0[j].as_mut_ptr() as *mut itPokemonAttributes;
                (*a).x0 = rw(); (*a).timer = (rnd() % 5) as i32; (*a).max = (rnd() % 12) as i32;
                (*a).xC = (rnd() % 5) as i32; (*a).x10 = (rnd() % 5) as i32; (*a).x14 = (rnd() % 5) as i32; (*a).x18 = (rnd() % 5) as i32;""")],
    tweak=per_item("""        IT[k].xDD4_itemVar.pokemon.timer = (int) (rnd() % 4) - 1;
        IT[k].xDAC_itcmd_var0 = (int) (rnd() & 1);""",
                   """        IT[k].xDD4_itemVar.pokemon.timer = (rnd() % 4) as i32 - 1;
        IT[k].xDAC_itcmd_var0 = (rnd() & 1) as u32;"""),
    ptrs=("", ""),
)
