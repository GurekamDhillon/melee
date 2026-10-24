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

# ================================================================================ shard 721-728
def tw_cat(a, b):
    return (a[0] + chr(10) + b[0], a[1] + chr(10) + b[1])



def all3(c_body, r_body):
    """A special-attribute fill applied to the buffers of all three items (spawned items read ART[2])."""
    nl = chr(10)
    return ("        { int q; for (q = 0; q < 3; q++) {" + nl + c_body.replace("SPEC[0]", "SPEC[q]") + nl + "        } }",
            "            for q in 0..3usize {" + nl + r_body.replace("SPECB.0[0]", "SPECB.0[q]") + nl + "            }")


# ------------------------------------------------------------------------------ itkyasarinegg
CFG["itkyasarinegg"] = dict(
    spec=[all3("""        itKyasarinEggAttributes* a = (itKyasarinEggAttributes*) SPEC[0];
        a->x0 = 0; a->x4 = rw(); a->x8 = rw(); a->xC = (int) (rnd() % 4); a->x10 = (int) (rnd() % 3);""",
           """            let a = SPECB.0[0].as_mut_ptr() as *mut itKyasarinEggAttributes;
            (*a).x0 = null_mut(); (*a).x4 = rw(); (*a).x8 = rw(); (*a).xC = (rnd() % 4) as i32; (*a).x10 = (rnd() % 3) as i32;""")],
    tweak=per_item("        IT[k].xDD4_itemVar.kyasarinEgg.x20 = (int) (rnd() % 3);\n        IT[k].xDD4_itemVar.kyasarinEgg.x24 = rw();",
                   "        IT[k].xDD4_itemVar.kyasarinEgg.x20 = (rnd() % 3) as i32;\n        IT[k].xDD4_itemVar.kyasarinEgg.x24 = rw();"),
    ptrs=("", ""),
    noret=["itKyasarinegg_UnkMotion4_Anim"],
)

# ------------------------------------------------------------------------------------------ itlgun
CFG["itlgun"] = dict(
    spec=[("""        ItLGunAttr* a = (ItLGunAttr*) SPEC[0];
        a->max_ammo = (int) (rnd() % 20); a->pos.x = rw(); a->pos.y = rw(); a->pos.z = rw();""",
           """            let a = SPECB.0[0].as_mut_ptr() as *mut ItLGunAttr;
            (*a).max_ammo = (rnd() % 20) as i32; (*a).pos.x = rw(); (*a).pos.y = rw(); (*a).pos.z = rw();""")],
    tweak=per_item("        IT[k].xDD4_itemVar.lgun.timer = (int) (rnd() % 60) - 10;\n        IT[k].xD4C = (int) (rnd() % 4) - 1;",
                   "        IT[k].xDD4_itemVar.lgun.timer = (rnd() % 60) as i32 - 10;\n        IT[k].xD4C = (rnd() % 4) as i32 - 1;"),
    ptrs=("", ""),
)

# --------------------------------------------------------------------------------------- itlgunray
CFG["itlgunray"] = dict(
    hints={"article": ("&ART[1]", "&raw mut ART[1]")},
    spec=[all3("""        ItLGunRayAttr* a = (ItLGunRayAttr*) SPEC[0];
        a->speed = rw(); a->lifetime = rw(); a->max_scale = rw();""",
           """            let a = SPECB.0[0].as_mut_ptr() as *mut ItLGunRayAttr;
            (*a).speed = rw(); (*a).lifetime = rw(); (*a).max_scale = rw();""")],
    tweak=per_item("        IT[k].xDD4_itemVar.lgunray.angle = rw(); IT[k].xDD4_itemVar.lgunray.scale = rw(); IT[k].xDD4_itemVar.lgunray.speed = rw();",
                   "        IT[k].xDD4_itemVar.lgunray.angle = rw(); IT[k].xDD4_itemVar.lgunray.scale = rw(); IT[k].xDD4_itemVar.lgunray.speed = rw();"),
    ptrs=("", ""),
)

# -------------------------------------------------------------------------------------- itlgunbeam
CFG["itlgunbeam"] = dict(
    spec=[all3("""        ItLGunBeamAttr* a = (ItLGunBeamAttr*) SPEC[0];
        a->lifetime = rw(); a->x4 = rw(); a->x8 = rw(); a->xC = rw(); a->x10 = rw();""",
           """            let a = SPECB.0[0].as_mut_ptr() as *mut ItLGunBeamAttr;
            (*a).lifetime = rw(); (*a).x4 = rw(); (*a).x8 = rw(); (*a).xC = rw(); (*a).x10 = rw();""")],
    tweak=per_item("        IT[k].xDD4_itemVar.lgunbeam.angle0 = rw(); IT[k].xDD4_itemVar.lgunbeam.angle1 = rw();",
                   "        IT[k].xDD4_itemVar.lgunbeam.angle0 = rw(); IT[k].xDD4_itemVar.lgunbeam.angle1 = rw();"),
    ptrs=("", ""),
)

# ---------------------------------------------------------------------------------------- itleadead
CFG["itleadead"] = dict(
    c_decl="static itLeadeadAttr_x0 LDX[3]; static it_804D6D40_t D40;",
    rs_decl="static mut LDX: [itLeadeadAttr_x0; 3] = unsafe { core::mem::zeroed() }; static mut D40: it_804D6D40_t = unsafe { core::mem::zeroed() };",
    spec=[all3("""        itLeadeadAttributes* a = (itLeadeadAttributes*) SPEC[0];
        LDX[q].x0 = (int) (rnd() % 60); LDX[q].x4 = rw();
        a->x0 = &LDX[q]; a->x4 = (int) (rnd() % 30); a->x8 = (float) (rnd() % 4); a->xC = rw(); a->x10 = rw(); a->x14 = rw();
        a->x18 = (s16) (rnd() % 5); a->x1A = (s16) (rnd() % 4); a->x1C = (s16) (rnd() % 4); a->x1E = (s8) (rnd() % 4);""",
           """            let a = SPECB.0[0].as_mut_ptr() as *mut itLeadeadAttributes;
            LDX[q].x0 = (rnd() % 60) as i32; LDX[q].x4 = rw();
            (*a).x0 = &raw mut LDX[q]; (*a).x4 = (rnd() % 30) as i32; (*a).x8 = (rnd() % 4) as f32; (*a).xC = rw(); (*a).x10 = rw(); (*a).x14 = rw();
            (*a).x18 = (rnd() % 5) as i16; (*a).x1A = (rnd() % 4) as i16; (*a).x1C = (rnd() % 4) as i16; (*a).x1E = (rnd() % 4) as i8;""")],
    tweak=tw_cat(
        ("        it_804D6D40 = &D40; D40.x8 = (float) (rnd() % 1000) / 1000.0f;", "        m::ext::it_804D6D40 = &raw mut D40; D40.x8 = (rnd() % 1000) as f32 / 1000.0;"),
        per_item(
        """        IT[k].xDD4_itemVar.leadead.x38 = (rnd() & 1) ? &GO[(k + 1) % 3] : 0;
        IT[k].xDD4_itemVar.leadead.x3C = rw();
        IT[k].xDD4_itemVar.leadead.x44 = (int) (rnd() % 0x120);
        IT[k].xDD4_itemVar.leadead.x48 = (int) (rnd() % 0x20) - 3;
        IT[k].xDD4_itemVar.leadead.x4C = (int) (rnd() % 4) - 1;
        IT[k].xDD4_itemVar.leadead.x50 = (int) (rnd() % 4);
        IT[k].xDD4_itemVar.leadead.xE14 = (int) (rnd() % 8);
        IT[k].xC9C = (int) (rnd() % 80);""",
        """        IT[k].xDD4_itemVar.leadead.x38 = if rnd() & 1 != 0 { &raw mut GO[(k + 1) % 3] } else { null_mut() };
        IT[k].xDD4_itemVar.leadead.x3C = rw();
        IT[k].xDD4_itemVar.leadead.x44 = (rnd() % 0x120) as i32;
        IT[k].xDD4_itemVar.leadead.x48 = (rnd() % 0x20) as i32 - 3;
        IT[k].xDD4_itemVar.leadead.x4C = (rnd() % 4) as i32 - 1;
        IT[k].xDD4_itemVar.leadead.x50 = (rnd() % 4) as i32;
        IT[k].xDD4_itemVar.leadead.xE14 = (rnd() % 8) as i32;
        IT[k].xC9C = (rnd() % 80) as i32;""")),
    ptrs=("""    for (k = 0; k < 3; k++) { HM(PI(IT[k].xDD4_itemVar.leadead.x38)); IT[k].xDD4_itemVar.leadead.x38 = 0; }""",
          """    for k in 0..3 { hm(pi(IT[k].xDD4_itemVar.leadead.x38)); IT[k].xDD4_itemVar.leadead.x38 = null_mut(); }"""),
    hints={"arg1": ("(int) (rnd() % 5)", "(rnd() % 5) as i32"), "arg2": ("(int) (rnd() % 6)", "(rnd() % 6) as i32"),
           "part": ("(int) (rnd() % 8)", "(rnd() % 8) as i32"), "facing_dir": ("(rnd() & 1) ? 1 : -1", "if rnd() & 1 != 0 { 1 } else { -1 }")},
)

# ---------------------------------------------------------------------------------------- itlikelike
_ll = "IT[k].xDD4_itemVar.likelike"
CFG["itlikelike"] = dict(
    c_decl="static Vec3 LLX[3]; static it_804D6D40_t D40;",
    rs_decl="static mut LLX: [Vec3; 3] = unsafe { core::mem::zeroed() }; static mut D40: it_804D6D40_t = unsafe { core::mem::zeroed() };",
    spec=[all3("""        itLikelikeAttributes* a = (itLikelikeAttributes*) SPEC[0];
        *(int*) &LLX[q].x = (int) (rnd() % 40); LLX[q].y = rw(); LLX[q].z = rw();
        a->x0.x0_f32 = &LLX[q]; a->x4 = (int) (rnd() % 20); a->x8 = (int) (rnd() % 10); a->xC = (int) (rnd() % 14) + 1;
        a->x18 = (int) (rnd() % 4); a->x1C = (float) (rnd() % 30); a->x20 = (float) (rnd() % 30); a->x24 = (float) (rnd() % 30);
        a->x28 = (float) (rnd() % 30); a->x2C = (float) (rnd() % 30); a->x30 = rw(); a->x34 = rw(); a->x38 = (float) (rnd() % 45);
        a->x3C = (s8) (rnd() % 3); a->x3D = (s8) (rnd() % 4); a->x3E = (s8) (rnd() % 4);""",
           """            let a = SPECB.0[0].as_mut_ptr() as *mut itLikelikeAttributes;
            *(&raw mut LLX[q].x as *mut i32) = (rnd() % 40) as i32; LLX[q].y = rw(); LLX[q].z = rw();
            (*a).x0.x0_f32 = &raw mut LLX[q]; (*a).x4 = (rnd() % 20) as i32; (*a).x8 = (rnd() % 10) as i32; (*a).xC = (rnd() % 14) as i32 + 1;
            (*a).x18 = (rnd() % 4) as i32; (*a).x1C = (rnd() % 30) as f32; (*a).x20 = (rnd() % 30) as f32; (*a).x24 = (rnd() % 30) as f32;
            (*a).x28 = (rnd() % 30) as f32; (*a).x2C = (rnd() % 30) as f32; (*a).x30 = rw(); (*a).x34 = rw(); (*a).x38 = (rnd() % 45) as f32;
            (*a).x3C = (rnd() % 3) as i8; (*a).x3D = (rnd() % 4) as i8; (*a).x3E = (rnd() % 4) as i8;""")],
    tweak=tw_cat(
        ("        it_804D6D40 = &D40; D40.x8 = (float) (rnd() % 1000) / 1000.0f;", "        m::ext::it_804D6D40 = &raw mut D40; D40.x8 = (rnd() % 1000) as f32 / 1000.0;"),
        per_item(
        f"""        {_ll}.x18 = (int) (rnd() % 4);
        {_ll}.x20.x = rw(); {_ll}.x20.y = rw(); {_ll}.x20.z = rw();
        {_ll}.x38 = (int) (rnd() % 3);
        {_ll}.x3C = (rnd() & 1) ? 1 : ((rnd() & 1) ? -1 : 0);
        {_ll}.x44 = (int) (rnd() % 0x40) - 2;
        {_ll}.x48 = (int) (rnd() % 7);
        {_ll}.x4C = (int) (rnd() % 8) - 2;
        {_ll}.x50 = (rnd() & 1) ? &GO[(k + 1) % 3] : 0;
        IT[k].grab_victim = ((rnd() & 3) == 0) ? {_ll}.x50 : (((rnd() & 1) ? &GO[(k + 2) % 3] : 0));
        IT[k].atk_victim = (rnd() & 1) ? &GO[(k + 1) % 3] : 0;
        IT[k].xC9C = (int) (rnd() % 60);
        IT[k].x378_itemColl.ecb.top.y = rw(); IT[k].x378_itemColl.ecb.bottom.y = rw();""",
        """        IT[k].xDD4_itemVar.likelike.x18 = (rnd() % 4) as i32;
        IT[k].xDD4_itemVar.likelike.x20.x = rw(); IT[k].xDD4_itemVar.likelike.x20.y = rw(); IT[k].xDD4_itemVar.likelike.x20.z = rw();
        IT[k].xDD4_itemVar.likelike.x38 = (rnd() % 3) as i32;
        IT[k].xDD4_itemVar.likelike.x3C = if rnd() & 1 != 0 { 1 } else if rnd() & 1 != 0 { -1 } else { 0 };
        IT[k].xDD4_itemVar.likelike.x44 = (rnd() % 0x40) as i32 - 2;
        IT[k].xDD4_itemVar.likelike.x48 = (rnd() % 7) as i32;
        IT[k].xDD4_itemVar.likelike.x4C = (rnd() % 8) as i32 - 2;
        IT[k].xDD4_itemVar.likelike.x50 = if rnd() & 1 != 0 { &raw mut GO[(k + 1) % 3] } else { null_mut() };
        IT[k].grab_victim = if rnd() & 3 == 0 { IT[k].xDD4_itemVar.likelike.x50 } else if rnd() & 1 != 0 { &raw mut GO[(k + 2) % 3] } else { null_mut() };
        IT[k].atk_victim = if rnd() & 1 != 0 { &raw mut GO[(k + 1) % 3] } else { null_mut() };
        IT[k].xC9C = (rnd() % 60) as i32;
        IT[k].x378_itemColl.ecb.top.y = rw(); IT[k].x378_itemColl.ecb.bottom.y = rw();""")),
    ptrs=("""    for (k = 0; k < 3; k++) { HM(PI(IT[k].xDD4_itemVar.likelike.x50)); IT[k].xDD4_itemVar.likelike.x50 = 0; }""",
          """    for k in 0..3 { hm(pi(IT[k].xDD4_itemVar.likelike.x50)); IT[k].xDD4_itemVar.likelike.x50 = null_mut(); }"""),
    hints={"arg0": ("(int) (rnd() % 3)", "(rnd() % 3) as i32"), "arg1": ("(int) (rnd() % 3) - 1", "(rnd() % 3) as i32 - 1"),
           "arg2": ("(int) (rnd() % 3)", "(rnd() % 3) as i32")},
    deep_ecb=True,
)

# ---------------------------------------------------------------------------------------- itlinkarrow
_la = "IT[k].xDD4_itemVar.linkarrow"
CFG["itlinkarrow"] = dict(
    spec=[all3("""        itLinkArrowAttributes* a = (itLinkArrowAttributes*) SPEC[0];
        a->x0 = (float) (rnd() % 100); a->x4 = rw(); a->x8 = rw(); a->xC = rw(); a->x10 = rw(); a->x14 = rw();
        a->x18 = (float) (rnd() % 100); a->x1C = rw(); a->x20 = rw();
        a->x24 = (rnd() & 1) ? (HSD_Joint*) &JO[3 + q] : 0; a->x28 = (rnd() & 1) ? (HSD_Joint*) &JO[3 + q] : 0; a->x2C = rw();""",
           """            let a = SPECB.0[0].as_mut_ptr() as *mut itLinkArrowAttributes;
            (*a).x0 = (rnd() % 100) as f32; (*a).x4 = rw(); (*a).x8 = rw(); (*a).xC = rw(); (*a).x10 = rw(); (*a).x14 = rw();
            (*a).x18 = (rnd() % 100) as f32; (*a).x1C = rw(); (*a).x20 = rw();
            (*a).x24 = if rnd() & 1 != 0 { &raw mut JO[3 + q] as *mut HSD_Joint } else { null_mut() };
            (*a).x28 = if rnd() & 1 != 0 { &raw mut JO[3 + q] as *mut HSD_Joint } else { null_mut() };
            (*a).x2C = rw();""")],
    tweak=per_item(
        f"""        {_la}.x18.x = rw(); {_la}.x18.y = rw(); {_la}.x18.z = rw();
        {_la}.x24.x = rw(); {_la}.x24.y = rw(); {_la}.x24.z = rw();
        {_la}.x30.x = rw(); {_la}.x30.y = rw(); {_la}.x30.z = rw();
        {_la}.x3C.x = rw(); {_la}.x3C.y = rw(); {_la}.x3C.z = rw();
        {_la}.x48.x = rw(); {_la}.x48.y = rw(); {_la}.x48.z = rw();
        {_la}.x54.x = rw(); {_la}.x54.y = rw(); {_la}.x54.z = rw(); {_la}.x54.w = rw();
        {_la}.x64.x = rw(); {_la}.x64.y = rw(); {_la}.x64.z = rw(); {_la}.x64.w = rw();
        {_la}.x74.x = rw(); {_la}.x74.y = rw(); {_la}.x74.z = rw(); {_la}.x74.w = rw();
        {_la}.x84.x = rw(); {_la}.x84.y = rw(); {_la}.x84.z = rw(); {_la}.x84.w = rw();
        {_la}.x94 = rw(); {_la}.x98 = rw();
        {_la}.x9C = (rnd() & 7) == 0 ? (u32) (rnd() % 12) : (u32) (rnd() % 9);
        {_la}.xA0 = (int) (rnd() % 2); {_la}.xA4 = rnd() % 10; {_la}.xA8 = rw(); {_la}.xAC = rw();
        {_la}.xB0 = (int) (rnd() % 4);
        {_la}.xB4[0] = (rnd() & 1) ? &JO[6 + 14 * k + 0] : 0; {_la}.xB4[1] = (rnd() & 1) ? &JO[6 + 14 * k + 1] : 0; {_la}.xB4[2] = 0;
        {_la}.xC0 = rw();
        {_la}.xC4 = (rnd() & 1) ? &GO[(k + 1) % 3] : 0;
        {_la}.xC8 = rw(); {_la}.xCC = rw(); {_la}.xD0 = 0; {_la}.xD4 = rw(); {_la}.xD8 = rw(); {_la}.xDC = rw();
        {_la}.xE0 = (rnd() & 1) ? &GO[(k + 1) % 3] : 0;
        IT[k].owner = (rnd() & 3) ? {_la}.xE0 : (((rnd() & 1) ? &GO[(k + 2) % 3] : 0));
        {_la}.xE4 = (rnd() & 3) ? (int) (rnd() % 20) : -1;
        {_la}.xE8 = rw(); {_la}.xEC = rw(); {_la}.xF0 = ((rnd() & 3) == 0) ? 0x12B + (int) (rnd() % 3) : (int) (rnd() % 0x130) - 1;
        IT[k].kind = (rnd() & 1) ? (rnd() % 4 == 0 ? It_Kind_Kirby_LinkArrow : It_Kind_Link_Arrow) : (rnd() % 4 == 0 ? It_Kind_Kirby_CLinkArrow : It_Kind_CLink_Arrow);
        if ((rnd() & 7) == 0) IT[k].kind = (rnd() % 400);
        IT[k].xCF4_fighterGObjUnk = (rnd() & 1) ? &GO[(k + 1) % 3] : 0;
        IT[k].xDAC_itcmd_var0 = (rnd() & 3) ? (int) (rnd() % 3) : 1; IT[k].xDB0_itcmd_var1 = (int) (rnd() % 3); IT[k].xDB4_itcmd_var2 = (int) (rnd() % 2);""",
        f"""        let la = &mut IT[k].xDD4_itemVar.linkarrow;
        la.x18.x = rw(); la.x18.y = rw(); la.x18.z = rw();
        la.x24.x = rw(); la.x24.y = rw(); la.x24.z = rw();
        la.x30.x = rw(); la.x30.y = rw(); la.x30.z = rw();
        la.x3C.x = rw(); la.x3C.y = rw(); la.x3C.z = rw();
        la.x48.x = rw(); la.x48.y = rw(); la.x48.z = rw();
        la.x54.x = rw(); la.x54.y = rw(); la.x54.z = rw(); la.x54.w = rw();
        la.x64.x = rw(); la.x64.y = rw(); la.x64.z = rw(); la.x64.w = rw();
        la.x74.x = rw(); la.x74.y = rw(); la.x74.z = rw(); la.x74.w = rw();
        la.x84.x = rw(); la.x84.y = rw(); la.x84.z = rw(); la.x84.w = rw();
        la.x94 = rw(); la.x98 = rw();
        la.x9C = if rnd() & 7 == 0 {{ rnd() % 12 }} else {{ rnd() % 9 }};
        la.xA0 = (rnd() % 2) as i32; la.xA4 = rnd() % 10; la.xA8 = rw(); la.xAC = rw();
        la.xB0 = (rnd() % 4) as i32;
        la.xB4[0] = if rnd() & 1 != 0 {{ &raw mut JO[6 + 14 * k + 0] }} else {{ null_mut() }};
        la.xB4[1] = if rnd() & 1 != 0 {{ &raw mut JO[6 + 14 * k + 1] }} else {{ null_mut() }};
        la.xB4[2] = null_mut();
        la.xC0 = rw();
        la.xC4 = if rnd() & 1 != 0 {{ &raw mut GO[(k + 1) % 3] }} else {{ null_mut() }};
        la.xC8 = rw(); la.xCC = rw(); la.xD0 = null_mut(); la.xD4 = rw(); la.xD8 = rw(); la.xDC = rw();
        la.xE0 = if rnd() & 1 != 0 {{ &raw mut GO[(k + 1) % 3] }} else {{ null_mut() }};
        IT[k].owner = if rnd() & 3 != 0 {{ la.xE0 }} else if rnd() & 1 != 0 {{ &raw mut GO[(k + 2) % 3] }} else {{ null_mut() }};
        la.xE4 = if rnd() & 3 != 0 {{ (rnd() % 20) as i32 }} else {{ -1 }};
        la.xE8 = rw(); la.xEC = rw(); la.xF0 = if rnd() & 3 == 0 {{ 0x12B + (rnd() % 3) as i32 }} else {{ (rnd() % 0x130) as i32 - 1 }};
        IT[k].kind = if rnd() & 1 != 0 {{ if rnd() % 4 == 0 {{ It_Kind_Kirby_LinkArrow }} else {{ It_Kind_Link_Arrow }} }} else {{ if rnd() % 4 == 0 {{ It_Kind_Kirby_CLinkArrow }} else {{ It_Kind_CLink_Arrow }} }};
        if rnd() & 7 == 0 {{ IT[k].kind = (rnd() % 400) as i32; }}
        IT[k].xCF4_fighterGObjUnk = if rnd() & 1 != 0 {{ &raw mut GO[(k + 1) % 3] }} else {{ null_mut() }};
        IT[k].xDAC_itcmd_var0 = if rnd() & 3 != 0 {{ rnd() % 3 }} else {{ 1 }}; IT[k].xDB0_itcmd_var1 = rnd() % 3; IT[k].xDB4_itcmd_var2 = rnd() % 2;"""),
    ptrs=(f"""    for (k = 0; k < 3; k++) {{
        HM(PI({_la}.xB4[0])); HM(PI({_la}.xB4[1])); HM(PI({_la}.xB4[2])); HM(PI({_la}.xC4)); HM(PI({_la}.xD0)); HM(PI({_la}.xE0));
        {_la}.xB4[0] = 0; {_la}.xB4[1] = 0; {_la}.xB4[2] = 0; {_la}.xC4 = 0; {_la}.xD0 = 0; {_la}.xE0 = 0;
    }}""", """    for k in 0..3 {
        let la = &mut IT[k].xDD4_itemVar.linkarrow;
        hm(pi(la.xB4[0])); hm(pi(la.xB4[1])); hm(pi(la.xB4[2])); hm(pi(la.xC4)); hm(pi(la.xD0)); hm(pi(la.xE0));
        la.xB4[0] = null_mut(); la.xB4[1] = null_mut(); la.xB4[2] = null_mut(); la.xC4 = null_mut(); la.xD0 = null_mut(); la.xE0 = null_mut();
    }"""),
    hints={"arg1": ("(int) (rnd() % 3)", "(rnd() % 3) as i32"), "arg4": ("(int) (rnd() % 3)", "(rnd() % 3) as i32")},
    noret=["itLinkArrow_802A81C4"],
    over={"itLinkArrow_802A850C": (
        "float v2 = rw(); float v3 = rw(); float v4 = rw(); "
        "int cond = (IT[0].xDD4_itemVar.linkarrow.xE0 != 0 && IT[0].owner == IT[0].xDD4_itemVar.linkarrow.xE0); "
        "u32 r = (u32) itLinkArrow_802A850C(&GO[0], &VA[0], &VA[1], v2, v3, v4); if (cond) ret = r;",
        "let v2 = rw(); let v3 = rw(); let v4 = rw(); "
        "let cond = !IT[0].xDD4_itemVar.linkarrow.xE0.is_null() && IT[0].owner == IT[0].xDD4_itemVar.linkarrow.xE0; "
        "let r = m::itLinkArrow_802A850C(&raw mut GO[0], &raw mut VA[0], &raw mut VA[1], v2, v3, v4) as u32; if cond { ret = r; }")},
    deep_ecb=False,
)

# ---------------------------------------------------------------------------------------- itlinkbomb
_lb = "IT[k].xDD4_itemVar.linkbomb"
CFG["itlinkbomb"] = dict(
    c_decl="static ItemStateArray ISA[3];",
    rs_decl="static mut ISA: [ItemStateArray; 3] = unsafe { core::mem::zeroed() };",
    c_preg=["preg(ISA, sizeof ISA);"],
    rs_preg=["preg(&raw const ISA, core::mem::size_of_val(&ISA));"],
    spec=[all3("""        itLinkBombAttributes* a = (itLinkBombAttributes*) SPEC[0];
        a->lifetime = rnd() % 100; a->x4 = rw(); a->x8 = rw(); a->xC = rnd() % 60; a->x10 = (int) (rnd() % 10);
        a->x14 = rw(); a->x18 = rw(); a->x1C = rw(); a->x20 = rw(); a->x24 = rw(); a->x28 = rw(); a->x2C = rw(); a->x30 = rw();""",
           """            let a = SPECB.0[0].as_mut_ptr() as *mut itLinkBombAttributes;
            (*a).lifetime = rnd() % 100; (*a).x4 = rw(); (*a).x8 = rw(); (*a).xC = rnd() % 60; (*a).x10 = (rnd() % 10) as i32;
            (*a).x14 = rw(); (*a).x18 = rw(); (*a).x1C = rw(); (*a).x20 = rw(); (*a).x24 = rw(); (*a).x28 = rw(); (*a).x2C = rw(); (*a).x30 = rw();""")],
    tweak=per_item(
        f"""        ART[k].xC_itemStates = &ISA[k]; IT[k].xD0_itemStateDesc = 0;
        zero(&{_lb}.x0, sizeof {_lb}.x0); {_lb}.x0.b0 = rnd() & 1; {_lb}.x0.b1 = rnd() & 1; {_lb}.x0.b2 = rnd() & 1;
        {_lb}.x4 = rw(); {_lb}.x8 = rw(); {_lb}.xC = rw();
        {_lb}.x10 = (rnd() & 1) ? &GO[(k + 1) % 3] : 0;
        IT[k].xD44_lifeTimer = ((rnd() % 3) == 0) ? 0.0f : (float) (rnd() % 80) - 5.0f;
        IT[k].xCA0 = (int) (rnd() % 12);
        IT[k].xCCC_incDamageDirection = (rnd() & 1) ? 1.0f : -1.0f;
        IT[k].owner = (rnd() & 1) ? {_lb}.x10 : (((rnd() & 1) ? &GO[(k + 2) % 3] : 0));""",
        """        ART[k].xC_itemStates = &raw mut ISA[k]; IT[k].xD0_itemStateDesc = null_mut();
        core::ptr::write_bytes(&raw mut IT[k].xDD4_itemVar.linkbomb.x0 as *mut u8, 0, core::mem::size_of_val(&IT[k].xDD4_itemVar.linkbomb.x0)); IT[k].xDD4_itemVar.linkbomb.x0.anon_0.set_b0((rnd() & 1) as u8); IT[k].xDD4_itemVar.linkbomb.x0.anon_0.set_b1((rnd() & 1) as u8); IT[k].xDD4_itemVar.linkbomb.x0.anon_0.set_b2((rnd() & 1) as u8);
        IT[k].xDD4_itemVar.linkbomb.x4 = rw(); IT[k].xDD4_itemVar.linkbomb.x8 = rw(); IT[k].xDD4_itemVar.linkbomb.xC = rw();
        IT[k].xDD4_itemVar.linkbomb.x10 = if rnd() & 1 != 0 { &raw mut GO[(k + 1) % 3] } else { null_mut() };
        IT[k].xD44_lifeTimer = if rnd() % 3 == 0 { 0.0 } else { (rnd() % 80) as f32 - 5.0 };
        IT[k].xCA0 = (rnd() % 12) as i32;
        IT[k].xCCC_incDamageDirection = if rnd() & 1 != 0 { 1.0 } else { -1.0 };
        IT[k].owner = if rnd() & 1 != 0 { IT[k].xDD4_itemVar.linkbomb.x10 } else if rnd() & 1 != 0 { &raw mut GO[(k + 2) % 3] } else { null_mut() };"""),
    ptrs=(f"""    for (k = 0; k < 3; k++) {{
        HM(PI({_lb}.x10));
        HM({_lb}.x0.b0); HM({_lb}.x0.b1); HM({_lb}.x0.b2); HM({_lb}.x0.b3); HM({_lb}.x0.b4); HM({_lb}.x0.b5); HM({_lb}.x0.b6); HM({_lb}.x0.b7);
        {_lb}.x10 = 0; zero(&{_lb}.x0, sizeof {_lb}.x0);
    }}""", """    for k in 0..3 {
        let lb = &mut IT[k].xDD4_itemVar.linkbomb;
        hm(pi(lb.x10));
        hm(lb.x0.anon_0.b0() as u32); hm(lb.x0.anon_0.b1() as u32); hm(lb.x0.anon_0.b2() as u32); hm(lb.x0.anon_0.b3() as u32);
        hm(lb.x0.anon_0.b4() as u32); hm(lb.x0.anon_0.b5() as u32); hm(lb.x0.anon_0.b6() as u32); hm(lb.x0.anon_0.b7() as u32);
        lb.x10 = null_mut(); core::ptr::write_bytes(&raw mut lb.x0 as *mut u8, 0, core::mem::size_of_val(&lb.x0));
    }"""),
    hints={"msid": ("(int) (rnd() % 8)", "(rnd() % 8) as i32"), "arg2": ("rnd() % 32", "rnd() % 32"),
           "arg3": ("(int) (rnd() % 400)", "(rnd() % 400) as i32")},
    deep_ecb=True,
)

# ============================================================================ shard 737-744
def _farr(n, extra=None):
    """A `float[]` attribute block of n rw() values (extra: {index: (C expr, Rust expr)})."""
    extra = extra or {}
    c = ["        float* a = (float*) SPEC[0];"]
    r = ["            let a = SPECB.0[0].as_mut_ptr() as *mut f32;"]
    for i in range(n):
        ce, re_ = extra.get(i, ("rw()", "rw()"))
        c.append(f"        a[{i}] = {ce};")
        r.append(f"            *a.add({i}) = {re_};")
    return ("\n".join(c), "\n".join(r))


CFG["itluigifireball"] = dict(spec=[_farr(5)], tweak=("", ""), ptrs=("", ""))
CFG["itmariofireball"] = dict(spec=[_farr(5)], tweak=("", ""), ptrs=("", ""))
CFG["itmariocape"] = dict(spec=[("", "")], tweak=("", ""), ptrs=("", ""))

_ml = "IT[k].xDD4_itemVar.maril"
CFG["itmaril"] = dict(
    spec=[_farr(6, {5: ("(float) (rnd() % 50)", "(rnd() % 50) as f32")})],
    tweak=per_item(
        f"""        {_ml}.x60.flags = 0;
        {_ml}.x60.x0 = rnd() & 1;
        IT[k].xC9C = (int) (rnd() % 60);""",
        """        let ml = &mut IT[k].xDD4_itemVar.maril;
        ml.x60.flags = 0;
        ml.x60.anon_0.set_x0((rnd() & 1) as u8);
        IT[k].xC9C = (rnd() % 60) as i32;"""),
    ptrs=(f"""    for (k = 0; k < 3; k++) {{ HM({_ml}.x60.x0); {_ml}.x60.flags = 0; }}""",
          """    for k in 0..3 { let ml = &mut IT[k].xDD4_itemVar.maril; hm(ml.x60.anon_0.x0() as u32); ml.x60.flags = 0; }"""),
)

_pk = "IT[k].xDD4_itemVar.pokemon"
CFG["itmarumine"] = dict(
    spec=[("""        itPokemonAttributes* a = (itPokemonAttributes*) SPEC[0];
        a->x0 = rw(); a->timer = (int) (rnd() % 3); a->max = (int) (rnd() % 200); a->xC = (int) (rnd() % 4);""",
           """            let a = SPECB.0[0].as_mut_ptr() as *mut itPokemonAttributes;
            (*a).x0 = rw(); (*a).timer = (rnd() % 3) as i32; (*a).max = (rnd() % 200) as i32; (*a).xC = (rnd() % 4) as i32;""")],
    tweak=per_item(
        f"""        {_pk}.timer = (int) (rnd() % 5) - 2;
        {_pk}.x64 = (int) (rnd() % 3);""",
        """        IT[k].xDD4_itemVar.pokemon.timer = (rnd() % 5) as i32 - 2;
        IT[k].xDD4_itemVar.pokemon.x64 = (rnd() % 3) as i32;"""),
    ptrs=("", ""),
)

_mg = "IT[k].xDD4_itemVar.matadogas"
CFG["itmatadogas"] = dict(
    spec=[("""        itMatadogasAttributes* a = (itMatadogasAttributes*) SPEC[0];
        a->x0 = rw(); a->x4 = (float) (rnd() % 6); a->x8 = rw(); a->xC = rw();""",
           """            let a = SPECB.0[0].as_mut_ptr() as *mut itMatadogasAttributes;
            (*a).x0 = rw(); (*a).x4 = (rnd() % 6) as f32; (*a).x8 = rw(); (*a).xC = rw();""")],
    tweak=per_item(
        f"""        {_mg}.x60 = (int) (rnd() % 4);
        {_mg}.x64 = (int) (rnd() & 1);""",
        """        IT[k].xDD4_itemVar.matadogas.x60 = (rnd() % 4) as i32;
        IT[k].xDD4_itemVar.matadogas.x64 = (rnd() & 1) as i32;"""),
    ptrs=("", ""),
)

CFG["itmasterhandbullet"] = dict(
    spec=[_farr(1)],
    tweak=per_item("        IT[k].xDD4_itemVar.masterhandbullet.x0 = rw();",
                   "        IT[k].xDD4_itemVar.masterhandbullet.x0 = rw();"),
    ptrs=("", ""),
    hints={"msid": ("(int) (rnd() % 2)", "(rnd() % 2) as i32")},
)

_lz = "IT[k].xDD4_itemVar.masterhandlaser"
CFG["itmasterhandlaser"] = dict(
    spec=[_farr(2, {1: ("(float) (rnd() % 6)", "(rnd() % 6) as f32")})],
    c_decl="static Fighter FT;\nstatic FighterBone FTB[8];\nstatic HSD_Joint JT[2];\n",
    rs_decl="static mut FT: Fighter = unsafe { core::mem::zeroed() };\nstatic mut FTB: [FighterBone; 8] = unsafe { core::mem::zeroed() };\n"
            "static mut JT: [HSD_Joint; 2] = unsafe { core::mem::zeroed() };\n",
    c_preg=["preg(&FT, sizeof FT); preg(FTB, sizeof FTB); preg(JT, sizeof JT);"],
    rs_preg=["preg(&raw const FT, core::mem::size_of_val(&FT)); preg(&raw const FTB, core::mem::size_of_val(&FTB)); "
             "preg(&raw const JT, core::mem::size_of_val(&JT));"],
    tweak=("""    for (i = 0; i < 8; i++) FTB[i].joint = &JO[20 + 3 * i];
    FT.parts = FTB;
    GO[1].user_data = &FT;
    IT[0].owner = &GO[1];
    IT[0].xC8_joint = &JT[0];
    JT[0].child = &JT[1];
    IT[0].xDD4_itemVar.masterhandlaser.x4 = (int) (rnd() % 8);
    IT[0].xDD4_itemVar.masterhandlaser.x8 = (int) (rnd() % 5) - 1;
    IT[0].xDD4_itemVar.masterhandlaser.x0 = (int) (rnd() & 1);
    IT[0].xDD4_itemVar.masterhandlaser.x10 = &JO[30];""",
           """    for i in 0..8 {
        FTB[i].joint = &raw mut JO[20 + 3 * i];
    }
    FT.parts = &raw mut FTB as *mut FighterBone;
    GO[1].user_data = &raw mut FT as *mut _;
    IT[0].owner = &raw mut GO[1];
    IT[0].xC8_joint = &raw mut JT[0];
    JT[0].child = &raw mut JT[1];
    IT[0].xDD4_itemVar.masterhandlaser.x4 = (rnd() % 8) as i32;
    IT[0].xDD4_itemVar.masterhandlaser.x8 = (rnd() % 5) as i32 - 1;
    IT[0].xDD4_itemVar.masterhandlaser.x0 = (rnd() & 1) as i32;
    IT[0].xDD4_itemVar.masterhandlaser.x10 = &raw mut JO[30];"""),
    ptrs=(f"""    for (k = 0; k < 3; k++) {{ HM(PI({_lz}.x10)); {_lz}.x10 = 0; }}""",
          f"""    for k in 0..3 {{ hm(pi(IT[k].xDD4_itemVar.masterhandlaser.x10)); IT[k].xDD4_itemVar.masterhandlaser.x10 = null_mut(); }}"""),
    over={"it_802F0340": (
        "u32 v1 = fv() % 8; int v2 = (int) fv(); float v3 = rw(); ret = PI(it_802F0340(&GO[1], &VA[0], &VA[1], v1, v2, v3));",
        "let v1: u32 = fv() % 8; let v2: i32 = fv() as i32; let v3: f32 = rw(); "
        "ret = pi(m::it_802F0340(&raw mut GO[1], &raw mut VA[0], &raw mut VA[1], v1, v2, v3));")},

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
# ------------------------------------------------------------------------------------------ itscball
CFG["itscball"] = dict(spec=[("", "")], tweak=("", ""), ptrs=("", ""))

# --------------------------------------------------------------------------------------- itseakvanish
CFG["itseakvanish"] = dict(spec=[("", "")], tweak=("", ""), ptrs=("", ""))

# ------------------------------------------------------------------------------------ itseakneedleheld
_nh = "IT[k].xDD4_itemVar.seakneedleheld"
CFG["itseakneedleheld"] = dict(
    spec=[("", "")],
    tweak=per_item(
        f"""        {_nh}.owner = (rnd() & 1) ? IT[k].owner : ((rnd() & 3) ? &GO[(k + 2) % 3] : 0);
        IT[k].kind = (rnd() & 1) ? 80 : ((rnd() & 1) ? 153 : (int) (rnd() % 5));
        JO[3 + k].next = (rnd() & 1) ? &JO[6 + k] : 0;
        JO[6 + k].next = (rnd() & 1) ? &JO[9 + k] : 0;""",
        """        IT[k].xDD4_itemVar.seakneedleheld.owner = if rnd() & 1 != 0 { IT[k].owner } else if rnd() & 3 != 0 { &raw mut GO[(k + 2) % 3] } else { null_mut() };
        IT[k].kind = if rnd() & 1 != 0 { 80 } else if rnd() & 1 != 0 { 153 } else { (rnd() % 5) as _ };
        JO[3 + k].next = if rnd() & 1 != 0 { &raw mut JO[6 + k] } else { null_mut() };
        JO[6 + k].next = if rnd() & 1 != 0 { &raw mut JO[9 + k] } else { null_mut() };"""),
    ptrs=(f"""    for (k = 0; k < 3; k++) {{ HM(PI({_nh}.owner)); {_nh}.owner = 0; }}""",
          """    for k in 0..3 { hm(pi(IT[k].xDD4_itemVar.seakneedleheld.owner)); IT[k].xDD4_itemVar.seakneedleheld.owner = null_mut(); }"""),
    hints={"kind": ("((rnd() & 1) ? 80 : 153)", "(if rnd() & 1 != 0 { 80 } else { 153 })")},
)

# ------------------------------------------------------------------------------------------ itsonans
_so = "IT[k].xDD4_itemVar.sonans"
CFG["itsonans"] = dict(
    spec=[("""        itsonansAttributes* a = (itsonansAttributes*) SPEC[0];
        a->x0 = rw(); a->x4 = rw(); a->x8 = rw(); a->xC = rw(); a->x10 = rw(); a->x14 = rw(); a->x18 = rw(); a->x1C = rw();
        a->x20 = rw(); a->x24 = (int) (rnd() % 100);""",
           """            let a = SPECB.0[0].as_mut_ptr() as *mut itsonansAttributes;
            (*a).x0 = rw(); (*a).x4 = rw(); (*a).x8 = rw(); (*a).xC = rw(); (*a).x10 = rw(); (*a).x14 = rw(); (*a).x18 = rw(); (*a).x1C = rw();
            (*a).x20 = rw(); (*a).x24 = (rnd() % 100) as i32;""")],
    tweak=per_item(
        f"""        {_so}.x60 = rw(); {_so}.x64 = rw(); {_so}.x68 = (rnd() & 1) ? (float) (rnd() % 30) : rw();
        IT[k].xDAC_itcmd_var0 = (int) (rnd() & 1);
        IT[k].xD44_lifeTimer = (rnd() & 1) ? 1.0f : (float) (rnd() % 5);""",
        """        let so = &mut IT[k].xDD4_itemVar.sonans;
        so.x60 = rw(); so.x64 = rw(); so.x68 = if rnd() & 1 != 0 { (rnd() % 30) as f32 } else { rw() };
        IT[k].xDAC_itcmd_var0 = (rnd() & 1) as _;
        IT[k].xD44_lifeTimer = if rnd() & 1 != 0 { 1.0 } else { (rnd() % 5) as f32 };"""),
    ptrs=("", ""),
)

# ---------------------------------------------------------------------------------------- itspycloak
CFG["itspycloak"] = dict(spec=[("", "")], tweak=("", ""), ptrs=("", ""))

# ----------------------------------------------------------------------------------------- itsscope
CFG["itsscope"] = dict(
    spec=[("""        itSScopeAttributes* a = (itSScopeAttributes*) SPEC[0];
        *(int*) SPEC[0] = (int) (rnd() % 40);
        a->x4 = (int) (rnd() % 20); a->x8 = (int) (rnd() % 6);
        for (i = 0; i < 10; i++) a->xC[i] = (float) (rnd() % 12);
        a->x34.x = rw(); a->x34.y = rw(); a->x34.z = rw();""",
           """            let a = SPECB.0[0].as_mut_ptr() as *mut itSScopeAttributes;
            *(SPECB.0[0].as_mut_ptr() as *mut i32) = (rnd() % 40) as i32;
            (*a).x4 = (rnd() % 20) as i32; (*a).x8 = (rnd() % 6) as i32;
            for i in 0..10 { (*a).xC[i] = (rnd() % 12) as f32; }
            (*a).x34.x = rw(); (*a).x34.y = rw(); (*a).x34.z = rw();""")],
    tweak=per_item("        IT[k].xD4C = (int) (rnd() % 40) - 3;", "        IT[k].xD4C = (rnd() % 40) as i32 - 3;"),
    ptrs=("", ""),
    hints={"arg1": ("(int) (rnd() % 120) - 5", "(rnd() % 120) as i32 - 5"), "charge_level": ("(int) (rnd() % 10)", "(rnd() % 10) as i32")},
)

# ---------------------------------------------------------------------------------- itseakneedlethrown
_nt = "IT[k].xDD4_itemVar.seakneedlethrown"
CFG["itseakneedlethrown"] = dict(
    spec=[("""        itSeakNeedleThrownAttributes* a = (itSeakNeedleThrownAttributes*) SPEC[0];
        a->x0 = rw(); a->x4 = rw(); a->x8 = rw();""",
           """            let a = SPECB.0[0].as_mut_ptr() as *mut itSeakNeedleThrownAttributes;
            (*a).x0 = rw(); (*a).x4 = rw(); (*a).x8 = rw();""")],
    tweak=per_item(
        f"""        {_nt}.xDD4 = rw(); {_nt}.xDD8 = rw(); {_nt}.xDDC = rw(); {_nt}.xDE0 = rw();
        {_nt}.xDE4.x = rw(); {_nt}.xDE4.y = rw(); {_nt}.xDE4.z = rw();
        {_nt}.xDF0 = rw();
        {_nt}.xDF4 = (rnd() & 1) ? -1 : (int) (rnd() % 8);
        {_nt}.xDF8 = (rnd() & 1) ? 0.5f : rw(); {_nt}.xDFC = (rnd() & 1) ? 0.5f : rw();""",
        """        let nt = &mut IT[k].xDD4_itemVar.seakneedlethrown;
        nt.xDD4 = rw(); nt.xDD8 = rw(); nt.xDDC = rw(); nt.xDE0 = rw();
        nt.xDE4.x = rw(); nt.xDE4.y = rw(); nt.xDE4.z = rw();
        nt.xDF0 = rw();
        nt.xDF4 = if rnd() & 1 != 0 { -1 } else { (rnd() % 8) as i32 };
        nt.xDF8 = if rnd() & 1 != 0 { 0.5 } else { rw() }; nt.xDFC = if rnd() & 1 != 0 { 0.5 } else { rw() };"""),
    ptrs=("", ""),
    hints={"arg2": ("(rnd() % 3)", "(rnd() % 3)")},
)

# ------------------------------------------------------------------------------------------ itseakchain
_sc = "IT[k].xDD4_itemVar.seakchain"
CFG["itseakchain"] = dict(
    c_decl="""static ItemLink LK[16];
static HSD_GObj LGO[16];
static HSD_JObj LJO[32];
static Fighter FGT;
static FighterBone FBN[32];
static int LKC, LGC;
static unsigned NL;
""",
    rs_decl="""static mut LK: [ItemLink; 16] = unsafe { core::mem::zeroed() };
static mut LGO: [HSD_GObj; 16] = unsafe { core::mem::zeroed() };
static mut LJO: [HSD_JObj; 32] = unsafe { core::mem::zeroed() };
static mut FGT: Fighter = unsafe { core::mem::zeroed() };
static mut FBN: [FighterBone; 32] = unsafe { core::mem::zeroed() };
static mut LKC: usize = 0;
static mut LGC: usize = 0;
static mut NL: u32 = 0;
""",
    c_preg=["preg(LK, sizeof LK); preg(LGO, sizeof LGO); preg(LJO, sizeof LJO); preg(&FGT, sizeof FGT); preg(FBN, sizeof FBN);"],
    rs_preg=["preg(&raw const LK, core::mem::size_of_val(&LK)); preg(&raw const LGO, core::mem::size_of_val(&LGO)); "
             "preg(&raw const LJO, core::mem::size_of_val(&LJO)); preg(&raw const FGT, core::mem::size_of_val(&FGT)); "
             "preg(&raw const FBN, core::mem::size_of_val(&FBN));"],
    ptrargs={"ItemLink": ("&LK[rnd() % NL]", "&raw mut LK[(rnd() % NL) as usize]"),
             "itSeakChain_Attrs": ("(itSeakChain_Attrs*) SPEC[0]", "SPECB.0[0].as_mut_ptr() as *mut itSeakChain_Attrs")},
    over={
        "itSeakChain_Spawn": ("{ float f = rw(); ret = PI(itSeakChain_Spawn(&GO[2], &VA[0], f)); }",
                              "{ let f = rw(); ret = pi(m::itSeakChain_Spawn(&raw mut GO[2], &raw mut VA[0], f)); }"),
        "itSeakChain_Logic54_EvtUnk": ("itSeakChain_Logic54_EvtUnk(&GO[0], (rnd() & 1) ? &GO[2] : &GO[1]);",
                                       "m::itSeakChain_Logic54_EvtUnk(&raw mut GO[0], if rnd() & 1 != 0 { &raw mut GO[2] } else { &raw mut GO[1] });"),
    },
    noret=["it_802BAF2C"],
    spec=[("""        itSeakChain_Attrs* a = (itSeakChain_Attrs*) SPEC[0];
        a->x0 = 3 + (int) (rnd() % 12); a->x4 = (float) (rnd() % 8) + ((rnd() & 1) ? 0.5f : 0.0f);
        a->x10 = rw(); a->x14 = rw(); a->x18 = rw(); a->x1C = rw(); a->x20 = rw(); a->x24 = rw(); a->x28 = rw(); a->x2C = rw();
        a->x30 = rw(); a->x34 = rw(); a->x38 = rw(); a->x3C = rw(); a->x40 = rw(); a->x44 = rw(); a->x48 = rw();
        a->x54 = rw(); a->x58 = rw(); a->x5C = rw(); a->x60 = rw();
        a->x64_joint = (rnd() & 1) ? (HSD_Joint*) &JO[0] : 0; a->x68_joint = (rnd() & 1) ? (HSD_Joint*) &JO[1] : 0;""",
           """            let a = SPECB.0[0].as_mut_ptr() as *mut itSeakChain_Attrs;
            (*a).x0 = 3 + (rnd() % 12) as i32; (*a).x4 = (rnd() % 8) as f32 + if rnd() & 1 != 0 { 0.5 } else { 0.0 };
            (*a).x10 = rw(); (*a).x14 = rw(); (*a).x18 = rw(); (*a).x1C = rw(); (*a).x20 = rw(); (*a).x24 = rw(); (*a).x28 = rw(); (*a).x2C = rw();
            (*a).x30 = rw(); (*a).x34 = rw(); (*a).x38 = rw(); (*a).x3C = rw(); (*a).x40 = rw(); (*a).x44 = rw(); (*a).x48 = rw();
            (*a).x54 = rw(); (*a).x58 = rw(); (*a).x5C = rw(); (*a).x60 = rw();
            (*a).x64_joint = if rnd() & 1 != 0 { &raw mut JO[0] as *mut HSD_Joint } else { null_mut() };
            (*a).x68_joint = if rnd() & 1 != 0 { &raw mut JO[1] as *mut HSD_Joint } else { null_mut() };""")],
    tweak=("""    NL = 3 + rnd() % 5;
    LKC = 0; LGC = 0;
    for (i = 0; i < 16; i++) { fill_ItemLink(&LK[i]); fill_HSD_GObj(&LGO[i]); }
    for (i = 0; i < 32; i++) { fill_HSD_JObj(&LJO[i]); LJO[i].flags &= ~JOBJ_USE_QUATERNION; }
    zero(&FGT, sizeof FGT); zero(FBN, sizeof FBN);
    FGT.u.sk.lstick_delta.x = rw(); FGT.u.sk.lstick_delta.y = rw(); FGT.u.sk.lstick_delta.z = rw();
    FGT.mv.co.common.x1C = (rnd() & 1) ? 0.0f : rw();
    FGT.parts = FBN; FBN[26].joint = &LJO[16];
    GO[2].user_data = &FGT;
    for (i = 0; i < 16; i++) {
        LK[i].prev = (i > 0 && (unsigned) i < NL) ? &LK[i - 1] : 0;
        LK[i].next = ((unsigned) i + 1 < NL) ? &LK[i + 1] : 0;
        LK[i].gobj = &LGO[i]; LGO[i].hsd_obj = &LJO[i]; LK[i].jobj = &LJO[16 + i];
        LK[i].x2C_b0 = (i == 0) ? 1 : (rnd() & 1);
        LK[i].x2C_b1 = rnd() & 1; LK[i].x2C_b2 = rnd() & 1;
    }
    for (k = 0; k < 3; k++) {
        int j;
        @S@.x0 = &LK[0]; @S@.x4 = &LK[NL - 1]; @S@.parent_gobj = &GO[2];
        @S@.x10 = (int) (rnd() & 1); @S@.x14 = (int) (rnd() & 1); @S@.x18 = (int) (rnd() & 1);
        for (j = 0; j < 15; j++) { @S@.history[j].x = rw(); @S@.history[j].y = rw(); @S@.history[j].z = rw(); }
    }""".replace("@S@", _sc),
           """    NL = 3 + rnd() % 5;
    LKC = 0; LGC = 0;
    for i in 0..16 { g::fill_ItemLink(&mut LK[i]); g::fill_HSD_GObj(&mut LGO[i]); }
    for i in 0..32 { g::fill_HSD_JObj(&mut LJO[i]); LJO[i].flags &= !(JOBJ_USE_QUATERNION as u32); }
    FGT = core::mem::zeroed(); FBN = core::mem::zeroed();
    FGT.u.sk.lstick_delta.x = rw(); FGT.u.sk.lstick_delta.y = rw(); FGT.u.sk.lstick_delta.z = rw();
    FGT.mv.co.common.x1C = if rnd() & 1 != 0 { 0.0 } else { rw() };
    FGT.parts = &raw mut FBN as *mut FighterBone; FBN[26].joint = &raw mut LJO[16];
    GO[2].user_data = &raw mut FGT as *mut _;
    for i in 0..16usize {
        LK[i].prev = if i > 0 && (i as u32) < NL { &raw mut LK[i - 1] } else { null_mut() };
        LK[i].next = if (i as u32) + 1 < NL { &raw mut LK[i + 1] } else { null_mut() };
        LK[i].gobj = &raw mut LGO[i]; LGO[i].hsd_obj = &raw mut LJO[i] as *mut _; LK[i].jobj = &raw mut LJO[16 + i];
        LK[i].set_x2C_b0(if i == 0 { 1 } else { (rnd() & 1) as u8 });
        LK[i].set_x2C_b1((rnd() & 1) as u8); LK[i].set_x2C_b2((rnd() & 1) as u8);
    }
    for k in 0..3 {
        let sc = &mut IT[k].xDD4_itemVar.seakchain;
        sc.x0 = &raw mut LK[0]; sc.x4 = &raw mut LK[(NL - 1) as usize]; sc.parent_gobj = &raw mut GO[2];
        sc.x10 = (rnd() & 1) as i32; sc.x14 = (rnd() & 1) as i32; sc.x18 = (rnd() & 1) as i32;
        for j in 0..15 { sc.history[j].x = rw(); sc.history[j].y = rw(); sc.history[j].z = rw(); }
    }"""),
    ptrs=("""    for (k = 0; k < 3; k++) {
        HM(PI(@S@.x0)); HM(PI(@S@.x4)); HM(PI(@S@.parent_gobj));
        @S@.x0 = 0; @S@.x4 = 0; @S@.parent_gobj = 0;
    }
    for (i = 0; i < 16; i++) { hash_ItemLink(&LK[i]); hash_HSD_GObj(&LGO[i]); }
    for (i = 0; i < 32; i++) hash_HSD_JObj(&LJO[i]);
    HM(B(FGT.u.sk.lstick_delta.x)); HM(B(FGT.u.sk.lstick_delta.y)); HM(B(FGT.u.sk.lstick_delta.z)); HM(B(FGT.mv.co.common.x1C));""".replace("@S@", _sc),
          """    for k in 0..3 {
        let sc = &mut IT[k].xDD4_itemVar.seakchain;
        hm(pi(sc.x0)); hm(pi(sc.x4)); hm(pi(sc.parent_gobj));
        sc.x0 = null_mut(); sc.x4 = null_mut(); sc.parent_gobj = null_mut();
    }
    for i in 0..16 { g::hash_ItemLink(&LK[i]); g::hash_HSD_GObj(&LGO[i]); }
    for i in 0..32 { g::hash_HSD_JObj(&LJO[i]); }
    hm(FGT.u.sk.lstick_delta.x.to_bits()); hm(FGT.u.sk.lstick_delta.y.to_bits()); hm(FGT.u.sk.lstick_delta.z.to_bits()); hm(FGT.mv.co.common.x1C.to_bits());"""),
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
