"""Driver description of itsamusgrapple (shard 777-784): the grapple beam's chain, fighter and hitbox world.
Registered into specs_cfg.CFG by specs_cfg.py (`register`)."""

_ANIMS = ["x74", "x78", "x7C", "x80", "x84", "x88", "x8C", "x90", "x94", "x98", "x9C", "xA0", "xA4", "xA8", "xAC"]
_gc_anims = "\n".join(f"        a->{n} = (rnd() % 3) ? (void*) &ANIMP[rnd() % 8] : 0;" for n in _ANIMS)
_gr_anims = "\n".join(
    f"            (*a).{n} = if rnd() % 3 != 0 {{ &raw mut ANIMP[(rnd() % 8) as usize] as *mut _ }} else {{ null_mut() }};"
    for n in _ANIMS)

_G_C_DECL = """static ItemLink LK[24];
static HSD_GObj LGO[24];
static Fighter FT[2];
static FighterBone FB[256];
static ftSs_DatAttrs DA;
static ftData FD;
static itSamusGrappleAttributes AG[3];
static itSamusGrapple_HitboxData HBD;
static void* ANIMP[8];
static int LKN, LGN, NL;
"""
_G_R_DECL = """static mut LK: [ItemLink; 24] = unsafe { core::mem::zeroed() };
static mut LGO: [HSD_GObj; 24] = unsafe { core::mem::zeroed() };
static mut FT: [Fighter; 2] = unsafe { core::mem::zeroed() };
static mut FB: [FighterBone; 256] = unsafe { core::mem::zeroed() };
static mut DA: ftSs_DatAttrs = unsafe { core::mem::zeroed() };
static mut FD: ftData = unsafe { core::mem::zeroed() };
static mut AG: [itSamusGrappleAttributes; 3] = unsafe { core::mem::zeroed() };
static mut HBD: itSamusGrapple_HitboxData = unsafe { core::mem::zeroed() };
static mut ANIMP: [*mut core::ffi::c_void; 8] = [null_mut(); 8];
static mut LKN: usize = 0;
static mut LGN: usize = 0;
static mut NL: usize = 0;
"""

_G_C_TWEAK = """    {
        int j;
        zero(LK, sizeof LK); zero(LGO, sizeof LGO); zero(FT, sizeof FT); zero(FB, sizeof FB); zero(&DA, sizeof DA);
        zero(&FD, sizeof FD); zero(AG, sizeof AG); zero(&HBD, sizeof HBD); zero(ANIMP, sizeof ANIMP);
        LKN = 0; LGN = 0;
        NL = 2 + (int) (rnd() % 5);
        fill_Fighter(&FT[0]); fill_Fighter(&FT[1]); fill_ftSs_DatAttrs(&DA); fill_itSamusGrapple_HitboxData(&HBD);
        for (j = 0; j < 24; j++) { fill_ItemLink(&LK[j]); fill_HSD_GObj(&LGO[j]); }
        for (j = 0; j < 8; j++) ANIMP[j] = (rnd() & 1) ? (void*) &ANIMP[(j + 1) % 8] : 0;
        for (j = 0; j < 256; j++) FB[j].joint = &JO[j % 48];
        for (j = 0; j < 8; j++) JO[16 + j].child = &JO[32 + j];
        for (j = 0; j < NL; j++) {
            LK[j].next = j > 0 ? &LK[j - 1] : 0;
            LK[j].prev = j < NL - 1 ? &LK[j + 1] : 0;
            LK[j].gobj = &LGO[j];
            LK[j].jobj = &JO[3 + (int) (rnd() % 3)];
            LGO[j].hsd_obj = &JO[24 + j];
        }
        for (j = 0; j < 3; j++) ART[j].x4_specialAttributes = (union ItemSpecialAttributes*) &AG[j];
        for (j = 0; j < 4; j++) FT[0].x914[j].x134.owner = 0;
        IT[0].owner = &GO[1];
        GO[1].user_data = &FT[0];
        FT[0].gobj = &GO[1];
        FT[0].ft_data = &FD; FD.ext_attr = (union ftData_ExtAttr*) &DA; FT[0].dat_attrs = &DA; FT[0].parts = FB;
        DA.xA0 = (int) (rnd() % 4); DA.xB0 = (int) (rnd() % 4); DA.xC0 = (int) (rnd() % 4);
        DA.xD0 = (int) (rnd() % 40);
        FT[0].u.ss.x223C = (prof == 1 && (rnd() & 3) == 0) ? 0 : &GO[0];
        FT[0].u.ss.x2240 = (u8) (rnd() % 6);
        {
            static const int mids[6] = { 0xD4, 0xD6, 0x165, 0x166, 0x100, 0x14 };
            FT[0].motion_id = mids[rnd() % 6];
        }
        FT[0].mv.ss.unk7.x0 = (float) (rnd() % 4);
        FT[0].mv.ss.grapple.x4 = (float) ((int) (rnd() % 6) - 2);
        FT[0].input.held_buttons[0] = rnd() & 0x4C;
        FT[0].input.pressed_buttons = (rnd() & 1) ? 0x100 : 0;
        FT[0].ground_or_air = (rnd() & 1) ? GA_Air : GA_Ground;
        FT[0].x34_scale.y = (float) (rnd() % 4 + 1) / 2.0f;
        HBD.create_hitbox.create_hitbox_0.id = rnd() % 4;
        {
            itSamusGrapple_ItemVars* g = &IT[0].xDD4_itemVar.samusgrapple;
            g->x0 = &LK[NL - 1]; g->x4 = &LK[0]; g->x8 = (rnd() & 7) ? &GO[1] : 0; g->xC = &JO[3];
            g->unk_10 = (rnd() & 1) ? fn_802B7E34 : 0;
            g->x14 = rnd() & 1; g->x15 = (u8) rnd(); g->x16 = (u8) (rnd() % 3);
        }
        {
            itSamusGrappleAttributes* a = &AG[0];
            a->x0 = rg(); a->x4 = rw(); a->x8 = rg(); a->xC = (int) (rnd() % 5);
            a->x10 = (float) (rnd() % 8 + 1) / 4.0f; a->x14 = (float) (rnd() % 8 + 1) / 4.0f; a->x18 = (float) (rnd() % 8 + 1) / 4.0f;
            a->x1C = (float) (rnd() % 8 + 1) / 4.0f; a->x20 = (float) (rnd() % 8 + 1) / 4.0f; a->x24 = (float) (rnd() % 8 + 1) / 4.0f;
            a->x28 = (float) (rnd() % 8 + 1) / 4.0f; a->x2C = (float) (rnd() % 8 + 1) / 4.0f; a->x30 = (float) (rnd() % 8 + 1) / 4.0f;
            a->x34 = (int) (rnd() % 6);
            a->x38 = (float) (rnd() % 30 + 1) / 4.0f; a->x3C = (float) (rnd() % 10) / 4.0f;
            a->x40 = rw(); a->x44 = rw(); a->x48 = rw(); a->x4C = (float) (rnd() % 8) / 2.0f; a->x50 = rw(); a->x54 = (float) (rnd() % 8) / 2.0f; a->x58 = (float) (rnd() % 8) / 4.0f;
            a->x5C = (float) (rnd() % 4 + 1) / 2.0f; a->x60 = (float) (rnd() % 4 + 1) / 2.0f;
            a->x64 = (void*) &JO[40]; a->x68 = (void*) &JO[41]; a->x6C = (void*) &JO[42]; a->x70 = (void*) &JO[43];
""" + _gc_anims + """
        }
    }"""

_G_R_TWEAK = """    {
        core::ptr::write_bytes(&raw mut LK, 0, 1); core::ptr::write_bytes(&raw mut LGO, 0, 1); core::ptr::write_bytes(&raw mut FT, 0, 1);
        core::ptr::write_bytes(&raw mut FB, 0, 1); core::ptr::write_bytes(&raw mut DA, 0, 1); core::ptr::write_bytes(&raw mut FD, 0, 1);
        core::ptr::write_bytes(&raw mut AG, 0, 1); core::ptr::write_bytes(&raw mut HBD, 0, 1); core::ptr::write_bytes(&raw mut ANIMP, 0, 1);
        LKN = 0; LGN = 0;
        NL = 2 + (rnd() % 5) as usize;
        g::fill_Fighter(&mut FT[0]); g::fill_Fighter(&mut FT[1]); g::fill_ftSs_DatAttrs(&mut DA); g::fill_itSamusGrapple_HitboxData(&mut HBD);
        for j in 0..24 { g::fill_ItemLink(&mut LK[j]); g::fill_HSD_GObj(&mut LGO[j]); }
        for j in 0..8 { ANIMP[j] = if rnd() & 1 != 0 { &raw mut ANIMP[(j + 1) % 8] as *mut _ } else { null_mut() }; }
        for j in 0..256 { FB[j].joint = &raw mut JO[j % 48]; }
        for j in 0..8 { JO[16 + j].child = &raw mut JO[32 + j]; }
        for j in 0..NL {
            LK[j].next = if j > 0 { &raw mut LK[j - 1] } else { null_mut() };
            LK[j].prev = if j < NL - 1 { &raw mut LK[j + 1] } else { null_mut() };
            LK[j].gobj = &raw mut LGO[j];
            LK[j].jobj = &raw mut JO[3 + (rnd() % 3) as usize];
            LGO[j].hsd_obj = &raw mut JO[24 + j] as *mut _;
        }
        for j in 0..3 { ART[j].x4_specialAttributes = &raw mut AG[j] as *mut _; }
        for j in 0..4 { core::ptr::write_bytes(&raw mut FT[0].x914[j].x134, 0, 1); }
        IT[0].owner = &raw mut GO[1];
        GO[1].user_data = &raw mut FT[0] as *mut _;
        FT[0].gobj = &raw mut GO[1];
        FT[0].ft_data = &raw mut FD; FD.ext_attr = &raw mut DA as *mut _; FT[0].dat_attrs = &raw mut DA as *mut _; FT[0].parts = (&raw mut FB) as *mut FighterBone;
        DA.xA0 = (rnd() % 4) as i32; DA.xB0 = (rnd() % 4) as i32; DA.xC0 = (rnd() % 4) as i32;
        DA.xD0 = (rnd() % 40) as i32;
        FT[0].u.ss.x223C = if prof == 1 && (rnd() & 3) == 0 { null_mut() } else { &raw mut GO[0] };
        FT[0].u.ss.x2240 = (rnd() % 6) as u8;
        {
            const MIDS: [i32; 6] = [0xD4, 0xD6, 0x165, 0x166, 0x100, 0x14];
            FT[0].motion_id = MIDS[(rnd() % 6) as usize] as _;
        }
        FT[0].mv.ss.unk7.x0 = (rnd() % 4) as f32;
        FT[0].mv.ss.grapple.x4 = ((rnd() % 6) as i32 - 2) as f32;
        FT[0].input.held_buttons[0] = (rnd() & 0x4C) as _;
        FT[0].input.pressed_buttons = if rnd() & 1 != 0 { 0x100 } else { 0 };
        FT[0].ground_or_air = if rnd() & 1 != 0 { GA_Air } else { GA_Ground };
        FT[0].x34_scale.y = (rnd() % 4 + 1) as f32 / 2.0;
        HBD.create_hitbox.create_hitbox_0.set_id(rnd() % 4);
        {
            let g = &mut IT[0].xDD4_itemVar.samusgrapple;
            g.x0 = &raw mut LK[NL - 1]; g.x4 = &raw mut LK[0]; g.x8 = if rnd() & 7 != 0 { &raw mut GO[1] } else { null_mut() }; g.xC = &raw mut JO[3];
            g.unk_10 = if rnd() & 1 != 0 { Some(m::fn_802B7E34 as unsafe extern "C" fn(*mut Item_GObj)) } else { None };
            g.x14 = (rnd() & 1) as u8; g.x15 = rnd() as u8; g.x16 = (rnd() % 3) as u8;
        }
        {
            let a = &raw mut AG[0];
            (*a).x0 = rg(); (*a).x4 = rw(); (*a).x8 = rg(); (*a).xC = (rnd() % 5) as i32;
            (*a).x10 = (rnd() % 8 + 1) as f32 / 4.0; (*a).x14 = (rnd() % 8 + 1) as f32 / 4.0; (*a).x18 = (rnd() % 8 + 1) as f32 / 4.0;
            (*a).x1C = (rnd() % 8 + 1) as f32 / 4.0; (*a).x20 = (rnd() % 8 + 1) as f32 / 4.0; (*a).x24 = (rnd() % 8 + 1) as f32 / 4.0;
            (*a).x28 = (rnd() % 8 + 1) as f32 / 4.0; (*a).x2C = (rnd() % 8 + 1) as f32 / 4.0; (*a).x30 = (rnd() % 8 + 1) as f32 / 4.0;
            (*a).x34 = (rnd() % 6) as i32;
            (*a).x38 = (rnd() % 30 + 1) as f32 / 4.0; (*a).x3C = (rnd() % 10) as f32 / 4.0;
            (*a).x40 = rw(); (*a).x44 = rw(); (*a).x48 = rw(); (*a).x4C = (rnd() % 8) as f32 / 2.0; (*a).x50 = rw(); (*a).x54 = (rnd() % 8) as f32 / 2.0; (*a).x58 = (rnd() % 8) as f32 / 4.0;
            (*a).x5C = (rnd() % 4 + 1) as f32 / 2.0; (*a).x60 = (rnd() % 4 + 1) as f32 / 2.0;
            (*a).x64 = &raw mut JO[40] as *mut _; (*a).x68 = &raw mut JO[41] as *mut _; (*a).x6C = &raw mut JO[42] as *mut _; (*a).x70 = &raw mut JO[43] as *mut _;
""" + _gr_anims + """
        }
    }"""

_G_C_PTRS = """    {
        int j;
        HM(PI(FT[0].u.ss.x223C)); HM(FT[0].u.ss.x2240); zero(&FT[0].u, sizeof FT[0].u);
        for (j = 0; j < 4; j++) { HM(FT[0].x914[j].x134.hit_grabbed_victim_only); FT[0].x914[j].x134.owner = 0; }
        for (j = 0; j < 2; j++) hash_Fighter(&FT[j]);
        for (j = 0; j < 24; j++) { hash_ItemLink(&LK[j]); hash_HSD_GObj(&LGO[j]); }
        for (j = 0; j < 256; j++) hash_FighterBone(&FB[j]);
        hash_ftSs_DatAttrs(&DA); hash_itSamusGrappleAttributes(&AG[0]); hash_itSamusGrapple_HitboxData(&HBD);
        for (j = 0; j < 8; j++) HM(PI(ANIMP[j]));
        HM(LKN); HM(LGN);
        {
            itSamusGrapple_ItemVars* g = &IT[0].xDD4_itemVar.samusgrapple;
            HM(PI(g->x0)); HM(PI(g->x4)); HM(PI(g->x8)); HM(PI(g->xC)); HM(FI((const void*) g->unk_10)); HM(g->x14); HM(g->x15); HM(g->x16);
        }
        for (j = 0; j < 3; j++) zero(&IT[j].xDD4_itemVar, sizeof IT[j].xDD4_itemVar);
    }"""

_G_R_PTRS = """    {
        hm(pi(FT[0].u.ss.x223C)); hm(FT[0].u.ss.x2240 as u32); core::ptr::write_bytes(&raw mut FT[0].u, 0, 1);
        for j in 0..4 { hm(FT[0].x914[j].x134.hit_grabbed_victim_only() as u32); core::ptr::write_bytes(&raw mut FT[0].x914[j].x134, 0, 1); }
        for j in 0..2 { g::hash_Fighter(&FT[j]); }
        for j in 0..24 { g::hash_ItemLink(&LK[j]); g::hash_HSD_GObj(&LGO[j]); }
        for j in 0..256 { g::hash_FighterBone(&FB[j]); }
        g::hash_ftSs_DatAttrs(&DA); g::hash_itSamusGrappleAttributes(&AG[0]); g::hash_itSamusGrapple_HitboxData(&HBD);
        for j in 0..8 { hm(pi(ANIMP[j])); }
        hm(LKN as u32); hm(LGN as u32);
        {
            let g = &IT[0].xDD4_itemVar.samusgrapple;
            hm(pi(g.x0)); hm(pi(g.x4)); hm(pi(g.x8)); hm(pi(g.xC)); hm(fi(g.unk_10)); hm(g.x14 as u32); hm(g.x15 as u32); hm(g.x16 as u32);
        }
        for j in 0..3 { core::ptr::write_bytes(&raw mut IT[j].xDD4_itemVar, 0, 1); }
    }"""


def _link_arg(n, total, i):
    if total >= 2:
        return ("", "", "&LK[0]" if n == 0 else "&LK[NL - 1]", "&raw mut LK[0]" if n == 0 else "&raw mut LK[NL - 1]")
    return (f"ItemLink* v{i} = &LK[rnd() % NL];", f"let v{i}: *mut ItemLink = &raw mut LK[(rnd() % (NL as u32)) as usize];",
            f"v{i}", f"v{i}")


def register(CFG):
    fl = ("(float) (rnd() % 40) / 4.0f", "(rnd() % 40) as f32 / 4.0")
    CFG["itsamusgrapple"] = dict(
        spec=[("        ;", "            "), ("        ;", "            ")],
        prof=lambda n: 1 if n in ("it_802BAC3C", "it_802BAC80", "it_802BACC4") else 0,
        c_decl=_G_C_DECL, rs_decl=_G_R_DECL,
        c_preg=["preg(LK, sizeof LK); preg(LGO, sizeof LGO); preg(FT, sizeof FT); preg(FB, sizeof FB); preg(&DA, sizeof DA);",
                "preg(&FD, sizeof FD); preg(AG, sizeof AG); preg(&HBD, sizeof HBD); preg(ANIMP, sizeof ANIMP);"],
        rs_preg=["preg(&raw const LK, core::mem::size_of_val(&LK)); preg(&raw const LGO, core::mem::size_of_val(&LGO)); preg(&raw const FT, core::mem::size_of_val(&FT));",
                 "preg(&raw const FB, core::mem::size_of_val(&FB)); preg(&raw const DA, core::mem::size_of_val(&DA));",
                 "preg(&raw const FD, core::mem::size_of_val(&FD)); preg(&raw const AG, core::mem::size_of_val(&AG)); preg(&raw const HBD, core::mem::size_of_val(&HBD)); preg(&raw const ANIMP, core::mem::size_of_val(&ANIMP));"],
        tweak=(_G_C_TWEAK, _G_R_TWEAK),
        ptrs=(_G_C_PTRS, _G_R_PTRS),
        ptr_args={
            "ptr:ItemLink": _link_arg,
            "ptr:Fighter": lambda n, t, i: ("", "", "&FT[0]", "&raw mut FT[0]"),
            "ptr:itSamusGrappleAttributes": lambda n, t, i: ("", "", "&AG[0]", "&raw mut AG[0]"),
        },
        hints={"dist": fl, "target_dist": fl,
               "scale": ("(float) (rnd() % 4 + 1) / 2.0f", "(rnd() % 4 + 1) as f32 / 2.0"),
               "facing_dir": ("rg()", "rg()"),
               "arg2": ("(int) (rnd() % 2)", "(rnd() % 2) as i32")},
        over={
            "it_802B7160": ("it_802B7160(&GO[1], &HBD);", "m::it_802B7160(&raw mut GO[1], &raw mut HBD);"),
            "it_802B743C": ("it_802B743C(&LGO[0], &IT[0], (int) (fv() % 3));", "m::it_802B743C(&raw mut LGO[0], &raw mut IT[0], (fv() % 3) as i32);"),
            "it_802B7C18": ("float v0 = rw(); ret = PI(it_802B7C18(&GO[1], &VA[0], v0));",
                            "let v0: f32 = rw(); ret = pi(m::it_802B7C18(&raw mut GO[1], &raw mut VA[0], v0));"),
            "it_802BAC3C": ("it_802BAC3C(&GO[1]);", "m::it_802BAC3C(&raw mut GO[1]);"),
            "it_802BAC80": ("it_802BAC80(&GO[1]);", "m::it_802BAC80(&raw mut GO[1]);"),
            "it_802BACC4": ("it_802BACC4(&GO[1]);", "m::it_802BACC4(&raw mut GO[1]);"),
        },
    )
