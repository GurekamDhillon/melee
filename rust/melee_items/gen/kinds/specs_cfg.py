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
