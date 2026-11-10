"""Driver descriptions of the shard 809-816 item-kind TUs (appended to specs_cfg.CFG by specs_cfg.py)."""
from specs_cfg import CFG, per_item

_NOSPEC = [("", "")]


def _floats(n, allk=False):
    """spec profile: n random floats at the start of SPEC[0] (or of all three)."""
    ks = "0..3" if allk else "0..1"
    return [(f"""        for (i = 0; i < {3 if allk else 1}; i++) {{ int j; float* a = (float*) SPEC[i]; for (j = 0; j < {n}; j++) a[j] = rw(); }}""",
             f"""            for k in {ks} {{ let a = SPECB.0[k].as_mut_ptr() as *mut f32; for j in 0..{n} {{ *a.add(j) = rw(); }} }}""")]


# ------------------------------------------------------------------------------------ ityoshitongue
CFG["ityoshitongue"] = dict(
    spec=_NOSPEC,
    c_decl="static ItemModelDesc MD;",
    rs_decl="static mut MD: ItemModelDesc = unsafe { core::mem::zeroed() };",
    over={"it_802F2CE0": ("int v = (int) (fv() & 1); it_802F2CE0(&GO[0], v);", "let v = fv() & 1 != 0; m::it_802F2CE0(&raw mut GO[0], v);")},
    tweak=("    zero(&MD, sizeof MD); MD.x8_bone_attach_id = (int) (rnd() % 40);\n    for (k = 0; k < 3; k++) ART[k].x10_modelDesc = &MD;",
           "    MD = core::mem::zeroed(); MD.x8_bone_attach_id = (rnd() % 40) as i32;\n    for k in 0..3 { ART[k].x10_modelDesc = &raw mut MD; }"),
    ptrs=("", ""),
)

# -------------------------------------------------------------------------------------- ityoshistar
CFG["ityoshistar"] = dict(spec=_floats(2, True), tweak=("", ""), ptrs=("", ""))

# ---------------------------------------------------------------------------------- ityoshieggthrow
CFG["ityoshieggthrow"] = dict(spec=_floats(2, True), tweak=("", ""), ptrs=("", ""))

# ------------------------------------------------------------------------------------ ityoshiegglay
CFG["ityoshiegglay"] = dict(
    spec=_NOSPEC,
    c_decl="static itYoshiEggLay_DatAttrs DAT;",
    rs_decl="static mut DAT: itYoshiEggLay_DatAttrs = unsafe { core::mem::zeroed() };",
    tweak=("""    zero(&DAT, sizeof DAT);
    DAT.pos.x = rw(); DAT.pos.y = rw(); DAT.pos.z = rw();
    DAT.vel.x = rw(); DAT.vel.y = rw(); DAT.vel.z = rw();
    DAT.lifetime = rw(); DAT.float2 = rw(); DAT.float3 = rw(); DAT.x24 = rw();
    DAT.kind = (int) (rnd() % 200);""",
           """    DAT = core::mem::zeroed();
    DAT.pos.x = rw(); DAT.pos.y = rw(); DAT.pos.z = rw();
    DAT.vel.x = rw(); DAT.vel.y = rw(); DAT.vel.z = rw();
    DAT.lifetime = rw(); DAT.float2 = rw(); DAT.float3 = rw(); DAT.x24 = rw();
    DAT.kind = (rnd() % 200) as _;"""),
    ptrs=("", ""),
    over={"it_802F2F34": ("it_802F2F34(&GO[0], &DAT);", "m::it_802F2F34(&raw mut GO[0], &raw mut DAT);")},
)

# ---------------------------------------------------------------------------------------- itwstar
CFG["itwstar"] = dict(
    spec=[("""        itWstarAttributes* a = (itWstarAttributes*) SPEC[0];
        itWstarAttrEntry* en = (itWstarAttrEntry*) &a->x28_entries[0];
        a->x0 = rw(); a->x4 = rw(); a->x8 = rw(); a->xC = rw(); a->x10 = rw(); a->x14 = rw(); a->x18 = rw();
        a->x1C = rw(); a->x20 = rw();
        a->x24_count = 1 + (int) (rnd() % 4);
        for (i = 0; i < 4; i++) { en[i].x0_anim_joint = (HSD_AnimJoint*) &JO[3 + i]; en[i].x4_sfx = (int) (rnd() % 500); }""",
           """            let a = SPECB.0[0].as_mut_ptr() as *mut itWstarAttributes;
            let en = (&raw mut (*a).x28_entries) as *mut itWstarAttrEntry;
            (*a).x0 = rw(); (*a).x4 = rw(); (*a).x8 = rw(); (*a).xC = rw(); (*a).x10 = rw(); (*a).x14 = rw(); (*a).x18 = rw();
            (*a).x1C = rw(); (*a).x20 = rw();
            (*a).x24_count = 1 + (rnd() % 4) as i32;
            for i in 0..4 { (*en.add(i)).x0_anim_joint = &raw mut JO[3 + i] as *mut HSD_AnimJoint; (*en.add(i)).x4_sfx = (rnd() % 500) as _; }""")],
    c_decl="static ftCollisionBox BOXV;",
    rs_decl="static mut BOXV: ftCollisionBox = unsafe { core::mem::zeroed() };",
    tweak=("    zero(&BOXV, sizeof BOXV);\n    it_804D6D00 = (s8) ((int) (rnd() % 6) - 1);",
           "    BOXV = core::mem::zeroed();\n    m::ext::it_804D6D00 = ((rnd() % 6) as i32 - 1) as i8;"),
    ptrs=("    HM((u32) (int) it_804D6D00);\n    HM(B(BOXV.top)); HM(B(BOXV.bottom)); HM(B(BOXV.left.x)); HM(B(BOXV.left.y)); HM(B(BOXV.right.x)); HM(B(BOXV.right.y));",
          "    hm(m::ext::it_804D6D00 as i32 as u32);\n    hm(BOXV.top.to_bits()); hm(BOXV.bottom.to_bits()); hm(BOXV.left.x.to_bits()); hm(BOXV.left.y.to_bits()); hm(BOXV.right.x.to_bits()); hm(BOXV.right.y.to_bits());"),
    over={"it_802944AC": ("it_802944AC(&GO[0], &BOXV);", "m::it_802944AC(&raw mut GO[0], &raw mut BOXV);")},
)

# ---------------------------------------------------------------------------------------- ityaku
_yk = "IT[k].xDD4_itemVar.yaku"
CFG["ityaku"] = dict(
    spec=_NOSPEC,
    c_decl="""static unsigned char GRD[64];
static Article* ARTP[118];
static void CB6(Item_GObj* g, Ground* gr) { T(7001); T(PI(g)); T(PI(gr)); }
static void CB7(Item_GObj* g, Ground* gr, Vec3* v, HSD_GObj* f, float x) { T(7002); T(PI(g)); T(PI(gr)); TVEC(v); T(PI(f)); T(B(x)); }
static void CB8(Item_GObj* g, Ground* gr, HSD_GObj* t) { T(7003); T(PI(g)); T(PI(gr)); T(PI(t)); }""",
    rs_decl="""static mut GRD: [u8; 64] = [0; 64];
static mut ARTP: [*mut Article; 118] = [null_mut(); 118];
unsafe extern "C" fn cb6(g: *mut HSD_GObj, gr: *mut Ground) { t(7001); t(pi(g)); t(pi(gr)); }
unsafe extern "C" fn cb7(g: *mut HSD_GObj, gr: *mut Ground, v: *mut Vec3, f: *mut HSD_GObj, x: f32) { t(7002); t(pi(g)); t(pi(gr)); tvec(v); t(pi(f)); t(x.to_bits()); }
unsafe extern "C" fn cb8(g: *mut HSD_GObj, gr: *mut Ground, tt: *mut HSD_GObj) { t(7003); t(pi(g)); t(pi(gr)); t(pi(tt)); }""",
    c_preg=["preg(GRD, sizeof GRD); preg(&it_803F8C08, sizeof it_803F8C08);",
            "freg((const void*) CB6); freg((const void*) CB7); freg((const void*) CB8);"],
    rs_preg=["preg(&raw const GRD, 64); preg(&raw const m::it_803F8C08, core::mem::size_of::<ItemAttr>());",
             "freg(cb6 as *const () as usize); freg(cb7 as *const () as usize); freg(cb8 as *const () as usize);"],
    tweak=per_item(
        f"""        {_yk}.x2 = (s16) (1 + (rnd() & 1));
        if ({_yk}.x2 == 1) {{ {_yk}.x4_u.x4 = &JO[3 + (rnd() % 3)]; }} else {{ {_yk}.x4_u.x4_vec.x = rw(); {_yk}.x4_u.x4_vec.y = rw(); {_yk}.x4_u.x4_vec.z = rw(); }}
        {_yk}.x10 = (rnd() & 1) ? (Ground*) GRD : 0;
        {_yk}.x14 = (rnd() & 1) ? CB6 : 0;
        {_yk}.x18 = (rnd() & 1) ? CB7 : 0;
        {_yk}.x1C = (rnd() & 1) ? CB8 : 0;
        IT[k].toucher = (rnd() & 1) ? &GO[(k + 1) % 3] : 0;
        IT[k].xCEC_fighterGObj = (rnd() & 1) ? &GO[(k + 2) % 3] : 0;""",
        """        let yk = &mut IT[k].xDD4_itemVar.yaku;
        yk.x2 = (1 + (rnd() & 1)) as i16;
        if yk.x2 == 1 { yk.x4_u.x4 = &raw mut JO[3 + (rnd() % 3) as usize]; } else { yk.x4_u.x4_vec.x = rw(); yk.x4_u.x4_vec.y = rw(); yk.x4_u.x4_vec.z = rw(); }
        yk.x10 = if rnd() & 1 != 0 { GRD.as_mut_ptr() as *mut Ground } else { null_mut() };
        yk.x14 = if rnd() & 1 != 0 { Some(cb6) } else { None };
        yk.x18 = if rnd() & 1 != 0 { Some(cb7) } else { None };
        yk.x1C = if rnd() & 1 != 0 { Some(cb8) } else { None };
        IT[k].toucher = if rnd() & 1 != 0 { &raw mut GO[(k + 1) % 3] } else { null_mut() };
        IT[k].xCEC_fighterGObj = if rnd() & 1 != 0 { &raw mut GO[(k + 2) % 3] } else { null_mut() };"""),
    ptrs=(f"""    HM(PI(ART[0].x0_common_attr));
    for (k = 0; k < 3; k++) {{
        HM((u32) {_yk}.x2);
        if ({_yk}.x2 == 1) HM(PI({_yk}.x4_u.x4)); else {{ HM(B({_yk}.x4_u.x4_vec.x)); HM(B({_yk}.x4_u.x4_vec.y)); HM(B({_yk}.x4_u.x4_vec.z)); }}
        HM(PI({_yk}.x10)); HM(FI((const void*) {_yk}.x14)); HM(FI((const void*) {_yk}.x18)); HM(FI((const void*) {_yk}.x1C));
        {_yk}.x4_u.x4 = 0; {_yk}.x4_u.x4_vec.x = 0; {_yk}.x4_u.x4_vec.y = 0; {_yk}.x4_u.x4_vec.z = 0;
        {_yk}.x10 = 0; {_yk}.x14 = 0; {_yk}.x18 = 0; {_yk}.x1C = 0;
    }}""",
          """    hm(pi(ART[0].x0_common_attr));
    for k in 0..3 {
        let yk = &mut IT[k].xDD4_itemVar.yaku;
        hm(yk.x2 as u32);
        if yk.x2 == 1 { hm(pi(yk.x4_u.x4)); } else { hm(yk.x4_u.x4_vec.x.to_bits()); hm(yk.x4_u.x4_vec.y.to_bits()); hm(yk.x4_u.x4_vec.z.to_bits()); }
        hm(pi(yk.x10)); hm(fi(yk.x14)); hm(fi(yk.x18)); hm(fi(yk.x1C));
        yk.x4_u.x4 = null_mut(); yk.x4_u.x4_vec.x = 0.0; yk.x4_u.x4_vec.y = 0.0; yk.x4_u.x4_vec.z = 0.0;
        yk.x10 = null_mut(); yk.x14 = None; yk.x18 = None; yk.x1C = None;
    }"""),
    over={"it_802E6AEC": (
        "Ground* a0 = (Ground*) GRD; int a1 = (int) fv(); int a2 = (int) (fv() % 20); "
        "HSD_JObj* a3 = (fv() & 1) ? &JO[3 + (fv() % 3)] : 0; Vec3* a4 = (fv() & 1) ? &VA[0] : 0; int a5 = (int) (fv() & 1); "
        "void (*a6)(Item_GObj*, Ground*) = (fv() & 1) ? CB6 : 0; "
        "void (*a7)(Item_GObj*, Ground*, Vec3*, HSD_GObj*, f32) = (fv() & 1) ? CB7 : 0; "
        "void (*a8)(Item_GObj*, Ground*, HSD_GObj*) = (fv() & 1) ? CB8 : 0; "
        "ret = PI(it_802E6AEC(a0, a1, a2, a3, a4, a5, a6, a7, a8));",
        "let a0 = GRD.as_mut_ptr() as *mut Ground; let a1 = fv() as i32; let a2 = (fv() % 20) as i32; "
        "let a3: *mut HSD_JObj = if fv() & 1 != 0 { &raw mut JO[3 + (fv() % 3) as usize] } else { null_mut() }; "
        "let a4: *mut Vec3 = if fv() & 1 != 0 { &raw mut VA[0] } else { null_mut() }; let a5 = (fv() & 1) as i32; "
        "let a6: Option<unsafe extern \"C\" fn(*mut HSD_GObj, *mut Ground)> = if fv() & 1 != 0 { Some(cb6) } else { None }; "
        "let a7: Option<unsafe extern \"C\" fn(*mut HSD_GObj, *mut Ground, *mut Vec3, *mut HSD_GObj, f32)> = if fv() & 1 != 0 { Some(cb7) } else { None }; "
        "let a8: Option<unsafe extern \"C\" fn(*mut HSD_GObj, *mut Ground, *mut HSD_GObj)> = if fv() & 1 != 0 { Some(cb8) } else { None }; "
        "ret = pi(m::it_802E6AEC(a0, a1, a2, a3, a4, a5, a6, a7, a8));")},
)
CFG["ityaku"]["tweak"] = (
    "    zero(ARTP, sizeof ARTP); ARTP[117] = &ART[0]; it_804D6D38 = ARTP;\n" + CFG["ityaku"]["tweak"][0],
    "    ARTP = [null_mut(); 118]; ARTP[117] = &raw mut ART[0]; m::ext::it_804D6D38 = ARTP.as_mut_ptr();\n" + CFG["ityaku"]["tweak"][1])

# ------------------------------------------------------------------------------------ itzeldadinfire
_zd = "IT[k].xDD4_itemVar.zeldadinfire"
CFG["itzeldadinfire"] = dict(
    spec=_floats(12, True),
    tweak=per_item(
        f"""        IT[k].owner = (rnd() & 1) ? &GO[(k + 1) % 3] : 0;
        {_zd}.xDD8 = rw();
        {_zd}.xDDC = (int) (rnd() & 1);
        {_zd}.xDE0 = (rnd() & 1) ? IT[k].owner : ((rnd() & 1) ? &GO[(k + 2) % 3] : 0);
        {_zd}.xDE8 = rw(); {_zd}.xDEC = rw(); {_zd}.xDF0 = rw();
        {_zd}.xDF4 = rnd() & 1;""",
        """        IT[k].owner = if rnd() & 1 != 0 { &raw mut GO[(k + 1) % 3] } else { null_mut() };
        let zd = &mut IT[k].xDD4_itemVar.zeldadinfire;
        zd.xDD8 = rw();
        zd.xDDC = (rnd() & 1) as i32;
        zd.xDE0 = if rnd() & 1 != 0 { IT[k].owner } else if rnd() & 1 != 0 { &raw mut GO[(k + 2) % 3] } else { null_mut() };
        zd.xDE8 = rw(); zd.xDEC = rw(); zd.xDF0 = rw();
        zd.xDF4 = rnd() & 1;"""),
    ptrs=(f"    for (k = 0; k < 3; k++) {{ HM(PI({_zd}.xDE0)); {_zd}.xDE0 = 0; }}",
          "    for k in 0..3 { hm(pi(IT[k].xDD4_itemVar.zeldadinfire.xDE0)); IT[k].xDD4_itemVar.zeldadinfire.xDE0 = null_mut(); }"),
    over={"it_802C3AFC": ("ret = (u32) it_802C3AFC(&GO[0], &IT[0].x378_itemColl);",
                          "ret = m::it_802C3AFC(&raw mut GO[0], &raw mut IT[0].x378_itemColl) as u32;")},
)

# -------------------------------------------------------------------------------------- itwhitebea
_wb = "IT[k].xDD4_itemVar.whitebea"
CFG["itwhitebea"] = dict(
    spec=[("""        itWhiteBeaAttributes* a = (itWhiteBeaAttributes*) SPEC[0];
        a->x0 = (__typeof__(a->x0)) (SPEC[0] + 64);
        a->x0->x0 = (int) (rnd() % 20); a->x0->x4 = rw();
        a->x4 = rw(); a->x8 = (short) (rnd() % 30); a->xA = (short) (rnd() % 30); a->xC = (short) (rnd() % 20);
        a->xE = (short) (a->xC + (int) (rnd() % 25) - 3); a->x10 = rw(); a->x14 = (short) (rnd() % 50);""",
           """            let a = SPECB.0[0].as_mut_ptr() as *mut itWhiteBeaAttributes;
            (*a).x0 = SPECB.0[0].as_mut_ptr().add(64) as *mut _;
            (*(*a).x0).x0 = (rnd() % 20) as i32; (*(*a).x0).x4 = rw();
            (*a).x4 = rw(); (*a).x8 = (rnd() % 30) as i16; (*a).xA = (rnd() % 30) as i16; (*a).xC = (rnd() % 20) as i16;
            (*a).xE = ((*a).xC as i32 + (rnd() % 25) as i32 - 3) as i16; (*a).x10 = rw(); (*a).x14 = (rnd() % 50) as i16;"""),
          ("""        itOldottoseaAttributes* a = (itOldottoseaAttributes*) SPEC[0];
        a->x14 = (float) (rnd() % 30); a->x18 = (float) (rnd() % 40); a->x1C = rw(); a->x20 = rw();""",
           """            let a = SPECB.0[0].as_mut_ptr() as *mut itOldottoseaAttributes;
            (*a).x14 = (rnd() % 30) as f32; (*a).x18 = (rnd() % 40) as f32; (*a).x1C = rw(); (*a).x20 = rw();""")],
    prof=lambda n: 1 if n in ("it_802E35CC", "itOldottosea_UnkMotion11_Coll") else 0,
    c_decl="static it_804D6D40_t D40;\nstatic int SFXA[4];",
    rs_decl="static mut D40: it_804D6D40_t = unsafe { core::mem::zeroed() };\nstatic mut SFXA: [i32; 4] = [0; 4];",
    tweak=("""    zero(&D40, sizeof D40); D40.x8 = (float) (rnd() % 11) / 10.0f; it_804D6D40 = &D40;
    SFXA[0] = -1; for (i = 1; i < 4; i++) SFXA[i] = (int) (rnd() % 500) - 1;
    for (k = 0; k < 3; k++) { @WB@.x20 = (rnd() & 1) ? &GO[(k + 1) % 3] : 0; }""".replace("@WB@", _wb),
           """    D40 = core::mem::zeroed(); D40.x8 = (rnd() % 11) as f32 / 10.0; m::ext::it_804D6D40 = &raw mut D40;
    SFXA[0] = -1; for i in 1..4 { SFXA[i] = (rnd() % 500) as i32 - 1; }
    for k in 0..3 { IT[k].xDD4_itemVar.whitebea.x20 = if rnd() & 1 != 0 { &raw mut GO[(k + 1) % 3] } else { null_mut() }; }"""),
    ptrs=(f"    for (k = 0; k < 3; k++) {{ HM(PI({_wb}.x20)); {_wb}.x20 = 0; }}",
          "    for k in 0..3 { hm(pi(IT[k].xDD4_itemVar.whitebea.x20)); IT[k].xDD4_itemVar.whitebea.x20 = null_mut(); }"),
)
