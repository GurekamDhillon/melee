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
