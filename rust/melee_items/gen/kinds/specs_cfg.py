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
