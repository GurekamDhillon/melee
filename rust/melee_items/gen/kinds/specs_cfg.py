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
