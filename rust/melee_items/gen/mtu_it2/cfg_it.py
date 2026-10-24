"""Driver descriptions for mk_it_driver.py (one per TU)."""
from mk_it_driver import hot

CFG = {}
def _hots(stem, items, **kw):
    cfg = dict(kw)
    cfg["setup_c"] = ""
    cfg["setup_rs"] = ""
    for p, v in items:
        c, r = hot(p, v)
        cfg["setup_c"] += c
        cfg["setup_rs"] += r
    CFG[stem] = cfg


def _hots3(stem, items, **kw):
    """_hots with (path, vals, kind) triples (kind "f": float literals)."""
    cfg = dict(kw)
    cfg["setup_c"] = cfg.get("setup_c", "")
    cfg["setup_rs"] = cfg.get("setup_rs", "")
    for p, v, k in items:
        c, r = hot(p, v, k)
        cfg["setup_c"] += c
        cfg["setup_rs"] += r
    CFG[stem] = cfg



_NORMPTR_C = lambda f: """    { u32 id = PI(%s); unsigned char* b = (unsigned char*) &%s; int i2; for (i2 = 0; i2 < 8; i2++) b[i2] = 0; *(u32*) b = id; }
""" % (f, f)
_NORMPTR_RS = lambda f: """    { let id = pi(%s); let b = &raw mut %s as *mut u8; for i2 in 0..8 { *b.add(i2) = 0; } *(b as *mut u32) = id; }
""" % (f, f)
_hots3("itlinkbow", [("IT.kind", ["It_Kind_Link_Bow", "It_Kind_CLink_Bow", "It_Kind_Kirby_CLinkBow", "It_Kind_Kirby_LinkBow", "0"], "i"),
                     ("IT.msid", ["0", "1", "2", "3", "4", "5", "6"], "i"),
                     ("IT.x5CC_currentAnimFrame", ["0.0", "24.0", "30.0", "5.0"], "f")],
       icd=False, tables=[("it_803F6E98", 7)])
CFG["itlinkbow"]["setup_c"] += "    { int q = rnd() % 4; IT.xDD4_itemVar.linkbow.x4 = q == 0 ? 0 : q == 1 ? IT.owner : q == 2 ? &OWN : &G2; if (rnd() & 1) IT.owner = IT.xDD4_itemVar.linkbow.x4; }\n"
CFG["itlinkbow"]["setup_rs"] += "    { let q = rnd() % 4; IT.xDD4_itemVar.linkbow.x4 = if q == 0 { null_mut() } else if q == 1 { IT.owner } else if q == 2 { &raw mut OWN } else { &raw mut G2 }; if rnd() & 1 != 0 { IT.owner = IT.xDD4_itemVar.linkbow.x4; } }\n"
CFG["itlinkbow"]["pre_hash_c"] = _NORMPTR_C("IT.xDD4_itemVar.linkbow.x4") + _NORMPTR_C("IT2.xDD4_itemVar.linkbow.x4")
CFG["itlinkbow"]["pre_hash_rs"] = _NORMPTR_RS("IT.xDD4_itemVar.linkbow.x4") + _NORMPTR_RS("IT2.xDD4_itemVar.linkbow.x4")

_hots3("itlipstick", [("IT.msid", ["0", "1", "2", "3", "4", "5"], "i"), ("IT.xD4C", ["0", "1", "2", "5"], "i"), ("IT.xDAC_itcmd_var0", ["0", "1"], "i")],
       icd=True, tables=[("it_803F6310", 6)])

_hots3("itlipstickspore", [("IT.xDD4_itemVar.lipstickspore.xDD4", ["0", "1", "2", "3"], "i"),
                           ("IT.xDD4_itemVar.lipstickspore.xDD8", ["0.0", "0.5", "2.0", "-1.0"], "f"),
                           ("IT.x40_vel.x", ["0.0", "0.25", "-0.25", "3.0", "-3.0"], "f")],
       icd=False, tables=[("it_803F6680", 2)])

_hots3("itlizardon", [("IT.msid", ["0", "1", "2", "3"], "i"), ("IT.xDD4_itemVar.lizardon.x60", ["-1", "0", "1", "3"], "i"),
                      ("IT.xDD4_itemVar.lizardon.x64", ["0", "1"], "i"), ("IT.xDD4_itemVar.lizardon.x6C", ["0", "1"], "i"),
                      ("IT.xDD4_itemVar.lizardon.x68", ["0.0", "1.0", "2.0", "5.0"], "f"),
                      ("IT.xDD4_itemVar.lizardon.x70", ["0", "1", "-1", "7"], "i"),
                      ("IT.xDD4_itemVar.lizardon.x74[0]", ["0", "1", "5"], "i"), ("IT.xDD4_itemVar.lizardon.x74[1]", ["0", "1", "5"], "i"),
                      ("IT.xDD4_itemVar.lizardon.x74[2]", ["0", "1", "5"], "i"), ("IT.xDD4_itemVar.lizardon.x74[3]", ["0", "1", "5"], "i"),
                      ("IT.xDAC_itcmd_var0", ["0", "1"], "i"), ("IT.xDB0_itcmd_var1", ["0", "1"], "i"),
                      ("IT.xD44_lifeTimer", ["0.0", "1.0", "-1.0", "30.0"], "f"),
                      ("IT.ground_or_air", ["GA_Ground", "GA_Air"], "i")],
       icd=False, tables=[("it_803F7BC0", 4), ("it_803F7C00", 1)])

_hots3("itlucky", [("IT.xDD4_itemVar.lucky.x60", ["0", "1", "2", "3", "4"], "i"), ("IT.xDD4_itemVar.lucky.x64", ["0", "1", "2", "5"], "i"),
                   ("IT.xDD4_itemVar.lucky.x68", ["0", "1"], "i"), ("IT.xDAC_itcmd_var0", ["0", "1"], "i"),
                   ("IT.xD44_lifeTimer", ["0.0", "1.0", "-1.0", "30.0", "1.5"], "f"),
                   ("IT.xC9C", ["0", "1", "100"], "i")],
       icd=False, tables=[("it_803F8190", 7), ("it_803F8200", 5)])

_hots3("itlugia", [("IT.msid", ["0", "1", "2", "3", "4", "5"], "i"), ("IT.xDD4_itemVar.lugia.x60", ["-1", "0", "1", "3"], "i"),
                   ("IT.xDD4_itemVar.lugia.xE50.y", ["0.0", "0.5", "1.0", "1.5"], "f"), ("IT.xDD4_itemVar.lugia.xE50.z", ["0.0", "0.5", "1.0", "1.5"], "f"),
                   ("IT.xDD4_itemVar.lugia.x88", ["0.0", "0.5", "1.0", "1.5"], "f"),
                   ("IT.xDD4_itemVar.lugia.xA4", ["0", "90", "-45"], "i"),
                   ("IT.xDAC_itcmd_var0", ["0", "1"], "i"), ("IT.xD44_lifeTimer", ["0.0", "1.0", "-1.0", "30.0"], "f"),
                   ("IT.kind", ["It_Kind_Lugia_Aeroblast", "It_Kind_Lugia_Aeroblast2", "It_Kind_Lugia_Aeroblast3", "0"], "i")],
       icd=False, tables=[("it_803F7EE8", 6), ("it_803F7F48", 1)])
CFG["itlugia"]["setup_c"] += """    { itLugiaAttributes* A = (itLugiaAttributes*) SA; A->x34 = (rnd() & 1) ? 40.0f : 90.0f; A->x38 = (rnd() & 1) ? 10.0f : 30.0f; A->x3C = (rnd() & 1) ? 0.5f : 1.0f; { u32 q = rnd() % 3; A->x40 = q == 0 ? 1.0f : q == 1 ? 2.0f : 1e9f; }
      if (rnd() & 1) { IT.xDD4_itemVar.lugia.x64.x = 1.0f; IT.xDD4_itemVar.lugia.x64.y = 2.0f; IT.xDD4_itemVar.lugia.x64.z = 0.0f; } IT.xDD4_itemVar.lugia.x8C = IT.xDD4_itemVar.lugia.x64; }
"""
CFG["itlugia"]["setup_rs"] += """    { let a = SA.as_mut_ptr() as *mut itLugiaAttributes; (*a).x34 = if rnd() & 1 != 0 { 40.0 } else { 90.0 }; (*a).x38 = if rnd() & 1 != 0 { 10.0 } else { 30.0 }; (*a).x3C = if rnd() & 1 != 0 { 0.5 } else { 1.0 }; { let q = rnd() % 3; (*a).x40 = if q == 0 { 1.0 } else if q == 1 { 2.0 } else { 1e9 }; }
      if rnd() & 1 != 0 { IT.xDD4_itemVar.lugia.x64.x = 1.0; IT.xDD4_itemVar.lugia.x64.y = 2.0; IT.xDD4_itemVar.lugia.x64.z = 0.0; } IT.xDD4_itemVar.lugia.x8C = IT.xDD4_itemVar.lugia.x64; }
"""
CFG["itlugia"]["ops"] = {"it_802D1F64": (
    "{ Vec3 v = it_802D1F64(&G, rw()); ret = B(v.x); HM(B(v.y)); HM(B(v.z)); }",
    "{ let v = m::it_802D1F64(&raw mut G, rw()); ret = v.x.to_bits(); hm(v.y.to_bits()); hm(v.z.to_bits()); }")}

_hots3("itlinkboomerang", [("IT.msid", ["0", "1", "2", "3"], "i"), ("IT.xDAC_itcmd_var0", ["0", "1", "2"], "i"),
                           ("IT.xDB0_itcmd_var1", ["0", "1", "2"], "i"), ("IT.xDB4_itcmd_var2", ["0", "1", "2"], "i"),
                           ("IT.xDD4_itemVar.linkboomerang.xDE8", ["0", "1"], "i"), ("IT.xDD4_itemVar.linkboomerang.xDEC", ["6", "1"], "i"),
                           ("IT.xDD4_itemVar.linkboomerang.xDE4", ["0", "3"], "i"),
                           ("IT.xDD4_itemVar.linkboomerang.xF70", ["0.0", "1.0", "5.0"], "f"),
                           ("IT.xDD4_itemVar.linkboomerang.xF8C", ["0.0", "1.0", "-2.0", "5.0"], "f"),
                           ("IT.xDD4_itemVar.linkboomerang.xF80", ["0.0", "1.0", "3.0"], "f"),
                           ("IT.xDD4_itemVar.linkboomerang.xF84", ["0.1", "0.5", "1.0"], "f"),
                           ("IT.xDD4_itemVar.linkboomerang.xF74", ["0.0", "1.0", "3.0", "-3.0", "7.0", "-7.0", "0.5"], "f"),
                           ("IT.xD44_lifeTimer", ["0.0", "1.0", "-1.0", "30.0"], "f"),
                           ("IT.facing_dir", ["1.0", "-1.0"], "f"),
                           ("IT.x378_itemColl.env_flags", ["0", "0xFC0", "0x3F", "0x6000", "0x18000"], "i")],
       icd=False, tables=[("it_803F6920", 4)])
CFG["itlinkboomerang"]["c_decl"] = """static ItemLogicTable LT;
static ItemStateArray ISA;
static void lt_thrown(HSD_GObj* g) { T(0xBEEF); T(PI(g)); }
"""
CFG["itlinkboomerang"]["rs_decl"] = """static mut LT: ItemLogicTable = unsafe { core::mem::zeroed() };
static mut ISA: ItemStateArray = unsafe { core::mem::zeroed() };
unsafe extern "C" fn lt_thrown(g: *mut HSD_GObj) { t(0xBEEF); t(pi(g)); }
"""
CFG["itlinkboomerang"]["setup_c"] += """    zero(&LT, sizeof LT); zero(&ISA, sizeof ISA); LT.thrown = lt_thrown; IT.xB8_itemLogicTable = &LT; IT2.xB8_itemLogicTable = &LT; ART.xC_itemStates = &ISA;
    IT.xDD4_itemVar.linkboomerang.xDDC[0] = rnd() % 16; IT.xDD4_itemVar.linkboomerang.xDDC[1] = rnd() % 16; IT.xDD4_itemVar.linkboomerang.xDD8 = rnd() % 16;
    IT2.xDD4_itemVar.linkboomerang.xDDC[0] = rnd() % 16; IT2.xDD4_itemVar.linkboomerang.xDDC[1] = rnd() % 16; IT2.xDD4_itemVar.linkboomerang.xDD8 = rnd() % 16;
    IT.xDD4_itemVar.linkboomerang.xF90[0] = (rnd() & 1) ? &JO[1] : 0; IT.xDD4_itemVar.linkboomerang.xF90[1] = (rnd() & 1) ? &JO[2] : 0;
    IT2.xDD4_itemVar.linkboomerang.xF90[0] = (rnd() & 1) ? &JO[1] : 0; IT2.xDD4_itemVar.linkboomerang.xF90[1] = (rnd() & 1) ? &JO[2] : 0;
    { int q = rnd() % 4; IT.xDD4_itemVar.linkboomerang.xF98 = q == 0 ? 0 : q == 1 ? &OWN : q == 2 ? &G2 : IT.owner; }
    IT2.xDD4_itemVar.linkboomerang.xF98 = 0;
"""
CFG["itlinkboomerang"]["setup_rs"] += """    LT = core::mem::zeroed(); ISA = core::mem::zeroed(); LT.thrown = Some(lt_thrown); IT.xB8_itemLogicTable = &raw mut LT; IT2.xB8_itemLogicTable = &raw mut LT; ART.xC_itemStates = &raw mut ISA;
    IT.xDD4_itemVar.linkboomerang.xDDC[0] = (rnd() % 16) as i32; IT.xDD4_itemVar.linkboomerang.xDDC[1] = (rnd() % 16) as i32; IT.xDD4_itemVar.linkboomerang.xDD8 = (rnd() % 16) as i32;
    IT2.xDD4_itemVar.linkboomerang.xDDC[0] = (rnd() % 16) as i32; IT2.xDD4_itemVar.linkboomerang.xDDC[1] = (rnd() % 16) as i32; IT2.xDD4_itemVar.linkboomerang.xDD8 = (rnd() % 16) as i32;
    IT.xDD4_itemVar.linkboomerang.xF90[0] = if rnd() & 1 != 0 { &raw mut JO[1] } else { null_mut() }; IT.xDD4_itemVar.linkboomerang.xF90[1] = if rnd() & 1 != 0 { &raw mut JO[2] } else { null_mut() };
    IT2.xDD4_itemVar.linkboomerang.xF90[0] = if rnd() & 1 != 0 { &raw mut JO[1] } else { null_mut() }; IT2.xDD4_itemVar.linkboomerang.xF90[1] = if rnd() & 1 != 0 { &raw mut JO[2] } else { null_mut() };
    { let q = rnd() % 4; IT.xDD4_itemVar.linkboomerang.xF98 = if q == 0 { null_mut() } else if q == 1 { &raw mut OWN } else if q == 2 { &raw mut G2 } else { IT.owner }; }
    IT2.xDD4_itemVar.linkboomerang.xF98 = null_mut();
"""

_ANG_C = "float a = (float) ((int) (rnd() % 2001) - 1000) / 100.0f;"
_ANG_RS = "let mut a: f32 = ((rnd() % 2001) as i32 - 1000) as f32 / 100.0;"
CFG["itlinkboomerang"]["ops"] = {
    "loop_lb_8000BA0C_gobj": ("loop_lb_8000BA0C_gobj(&G, &JO[1], (itLinkBoomerangAttributes*) SA);",
                              "m::loop_lb_8000BA0C_gobj(&raw mut G, &raw mut JO[1], SA.as_mut_ptr() as *mut itLinkBoomerangAttributes);"),
    "it_802A013C_LoadAnim": ("ret = PI(it_802A013C_LoadAnim((itLinkBoomerangAttributes*) SA, (int) (fv() % 2)));",
                             "ret = pi(m::it_802A013C_LoadAnim(SA.as_mut_ptr() as *mut itLinkBoomerangAttributes, (fv() % 2) as i32));"),
    "itLinkboomerang_get_x18": ("ret = B(itLinkboomerang_get_x18((itLinkBoomerangAttributes*) SA));",
                                "ret = m::itLinkboomerang_get_x18(SA.as_mut_ptr() as *mut itLinkBoomerangAttributes).to_bits();"),
}
for _n in ("clamp_tau", "clamp_pi_tau", "clamp_angle_pi"):
    CFG["itlinkboomerang"]["ops"][_n] = ("{ %s %s(&a); ret = B(a); }" % (_ANG_C, _n), "{ %s m::%s(&mut a); ret = a.to_bits(); }" % (_ANG_RS, _n))
for _n in ("it_8029FE64", "it_8029FE64_no_inline"):
    CFG["itlinkboomerang"]["ops"][_n] = ("%s(&G, (s32) (fv() %% 2));" % _n, "m::%s(&raw mut G, (fv() %% 2) as i32);" % _n)
_BF = ["xDD4_itemVar.linkboomerang.xF90[0]", "xDD4_itemVar.linkboomerang.xF90[1]", "xDD4_itemVar.linkboomerang.xF98"]
CFG["itlinkboomerang"]["pre_hash_c"] = "".join(_NORMPTR_C("%s.%s" % (v, f)) for v in ("IT", "IT2") for f in _BF)
CFG["itlinkboomerang"]["pre_hash_rs"] = "".join(_NORMPTR_RS("%s.%s" % (v, f)) for v in ("IT", "IT2") for f in _BF)

# ---- itlinkhookshot -----------------------------------------------------------------------------------------
_hots3("itlinkhookshot", [("IT.xDD4_itemVar.linkhookshot.x14", ["0", "1"], "i")],
       icd=False, tables=[("it_803F6998", 9)])
_H = CFG["itlinkhookshot"]
_H["c_decl"] = """static ItemLink LK[16];
static HSD_GObj LG[16];
static Fighter FTR;
static FighterBone FPB[160];
static ftData FTD;
static ftLk_DatAttrs LKA;
static mp_UnkStruct0 MPU;
static float MTXB[3][4];
static float MF;
static ItemLink *LTAIL, *LHEAD, *LP;
static int ACNT, GCNT;
extern f32 it_804D6D48;
"""
_H["rs_decl"] = """static mut LK: [ItemLink; 16] = unsafe { core::mem::zeroed() };
static mut LG: [HSD_GObj; 16] = unsafe { core::mem::zeroed() };
static mut FTR: Fighter = unsafe { core::mem::zeroed() };
static mut FPB: [FighterBone; 160] = unsafe { core::mem::zeroed() };
static mut FTD: ftData = unsafe { core::mem::zeroed() };
static mut LKA: ftLk_DatAttrs = unsafe { core::mem::zeroed() };
static mut MPU: mp_UnkStruct0 = unsafe { core::mem::zeroed() };
static mut MTXB: [[f32; 4]; 3] = [[0.0; 4]; 3];
static mut MF: f32 = 0.0;
static mut LTAIL: *mut ItemLink = null_mut();
static mut LHEAD: *mut ItemLink = null_mut();
static mut LP: *mut ItemLink = null_mut();
static mut ACNT: usize = 0;
static mut GCNT: usize = 0;
"""
_H["pregs"] = [("LK", "sizeof LK", "&raw const LK", "core::mem::size_of::<[ItemLink; 16]>()"),
               ("LG", "sizeof LG", "&raw const LG", "core::mem::size_of::<[HSD_GObj; 16]>()"),
               ("&FTR", "sizeof FTR", "&raw const FTR", "core::mem::size_of::<Fighter>()"),
               ("FPB", "sizeof FPB", "&raw const FPB", "core::mem::size_of::<[FighterBone; 160]>()"),
               ("&MPU", "sizeof MPU", "&raw const MPU", "core::mem::size_of::<mp_UnkStruct0>()")]
_H["c_extra"] = """
static void fill_ftr(void)
{
    u32 q;
    zero(&FTR, sizeof FTR); zero(FPB, sizeof FPB); zero(&FTD, sizeof FTD); zero(&LKA, sizeof LKA);
    q = rnd() % 6; FTR.motion_id = q == 0 ? 0x168 : q == 1 ? 0x169 : q == 2 ? 0xD4 : q == 3 ? 0xD6 : q == 4 ? 0x100 : 0x168;
    FTR.kind = (rnd() & 1) ? 20 : 6;
    FTR.ground_or_air = (rnd() & 1) ? 1 : 0;
    FTR.facing_dir = (rnd() & 1) ? 1.0f : -1.0f;
    { u32 q3 = rnd() % 3; FTR.x34_scale.y = q3 == 0 ? 0.5f : q3 == 1 ? 1.0f : 2.0f; }
    FTR.cur_pos.x = rw(); FTR.cur_pos.y = rw(); FTR.cur_pos.z = rw();
    FTR.self_vel.x = rw(); FTR.self_vel.y = rw(); FTR.pos_delta.x = rw(); FTR.pos_delta.y = rw();
    FTR.input.pressed_buttons = (rnd() & 1) ? 0x100 : (rnd() & 0xFF);
    LKA.x88 = rnd() % 4; LKA.x98 = rnd() % 4; LKA.xA8 = rnd() % 4; LKA.xB4 = rw(); LKA.xB8 = rnd() % 4;
    if (rnd() & 1) { q = rnd() % 5; FTR.mv.ca.specials.grav = q == 0 ? 0.0f : q == 1 ? 1.0f : q == 2 ? 2.0f : q == 3 ? 3.0f : rw(); }
    else { q = rnd() % 4; FTR.mv.lk.specialn.x0.y = q == 0 ? 0.0f : q == 1 ? 1.0f : q == 2 ? -1.0f : 5.0f; }
    FTR.u.lk.xC = (rnd() & 1) ? &G : 0;
    FTR.accessory2_cb = (rnd() & 1) ? it_802A7AAC : 0;
    FTR.death1_cb = (rnd() & 1) ? it_802A7AF0 : 0;
    FTR.accessory3_cb = (rnd() & 1) ? it_802A7B34 : 0;
    FTR.ft_data = &FTD; FTD.ext_attr = &LKA; FTR.dat_attrs = &LKA; FTR.parts = FPB;
    { int i; for (i = 0; i < 160; i++) FPB[i].joint = &JO[i & 3]; }
}
static void hash_ftr(void)
{
    HM((u32) FTR.motion_id); HM((u32) FTR.kind); HM((u32) FTR.ground_or_air); HM(B(FTR.facing_dir)); HM(B(FTR.x34_scale.y));
    HM(B(FTR.cur_pos.x)); HM(B(FTR.cur_pos.y)); HM(B(FTR.cur_pos.z));
    HM(B(FTR.self_vel.x)); HM(B(FTR.self_vel.y)); HM(B(FTR.pos_delta.x)); HM(B(FTR.pos_delta.y));
    HM((u32) FTR.input.pressed_buttons); HM(B(FTR.mv.ca.specials.grav)); HM(B(FTR.mv.lk.specialn.x0.y));
    HM(PI(FTR.u.lk.xC)); HM(FI((const void*) FTR.accessory2_cb)); HM(FI((const void*) FTR.death1_cb)); HM(FI((const void*) FTR.accessory3_cb));
    HM(PI(FPB[139].joint));
}
"""
_H["rs_extra"] = """
unsafe fn fill_ftr() {
    FTR = core::mem::zeroed(); FPB = core::mem::zeroed(); FTD = core::mem::zeroed(); LKA = core::mem::zeroed();
    let q = rnd() % 6; FTR.motion_id = match q { 0 => 0x168, 1 => 0x169, 2 => 0xD4, 3 => 0xD6, 4 => 0x100, _ => 0x168 } as _;
    FTR.kind = (if rnd() & 1 != 0 { 20 } else { 6 }) as _;
    FTR.ground_or_air = (if rnd() & 1 != 0 { 1 } else { 0 }) as _;
    FTR.facing_dir = if rnd() & 1 != 0 { 1.0 } else { -1.0 };
    { let q3 = rnd() % 3; FTR.x34_scale.y = if q3 == 0 { 0.5 } else if q3 == 1 { 1.0 } else { 2.0 }; }
    FTR.cur_pos.x = rw(); FTR.cur_pos.y = rw(); FTR.cur_pos.z = rw();
    FTR.self_vel.x = rw(); FTR.self_vel.y = rw(); FTR.pos_delta.x = rw(); FTR.pos_delta.y = rw();
    FTR.input.pressed_buttons = if rnd() & 1 != 0 { 0x100 } else { rnd() & 0xFF };
    LKA.x88 = (rnd() % 4) as _; LKA.x98 = (rnd() % 4) as _; LKA.xA8 = (rnd() % 4) as _; LKA.xB4 = rw(); LKA.xB8 = (rnd() % 4) as _;
    if rnd() & 1 != 0 { let q = rnd() % 5; FTR.mv.ca.specials.grav = match q { 0 => 0.0, 1 => 1.0, 2 => 2.0, 3 => 3.0, _ => rw() }; }
    else { let q = rnd() % 4; FTR.mv.lk.specialn.x0.y = match q { 0 => 0.0, 1 => 1.0, 2 => -1.0, _ => 5.0 }; }
    FTR.u.lk.xC = if rnd() & 1 != 0 { &raw mut G } else { null_mut() };
    FTR.accessory2_cb = if rnd() & 1 != 0 { Some(m::it_802A7AAC) } else { None };
    FTR.death1_cb = if rnd() & 1 != 0 { Some(m::it_802A7AF0) } else { None };
    FTR.accessory3_cb = if rnd() & 1 != 0 { Some(m::it_802A7B34) } else { None };
    FTR.ft_data = &raw mut FTD; FTD.ext_attr = (&raw mut LKA) as *mut c_void; FTR.dat_attrs = (&raw mut LKA) as *mut _; FTR.parts = FPB.as_mut_ptr();
    for i in 0..160usize { FPB[i].joint = &raw mut JO[i & 3]; }
}
unsafe fn hash_ftr() {
    hm(FTR.motion_id as u32); hm(FTR.kind as u32); hm(FTR.ground_or_air as u32); hm(FTR.facing_dir.to_bits()); hm(FTR.x34_scale.y.to_bits());
    hm(FTR.cur_pos.x.to_bits()); hm(FTR.cur_pos.y.to_bits()); hm(FTR.cur_pos.z.to_bits());
    hm(FTR.self_vel.x.to_bits()); hm(FTR.self_vel.y.to_bits()); hm(FTR.pos_delta.x.to_bits()); hm(FTR.pos_delta.y.to_bits());
    hm(FTR.input.pressed_buttons as u32); hm(FTR.mv.ca.specials.grav.to_bits()); hm(FTR.mv.lk.specialn.x0.y.to_bits());
    hm(pi(FTR.u.lk.xC)); hm(fi(FTR.accessory2_cb)); hm(fi(FTR.death1_cb)); hm(fi(FTR.accessory3_cb));
    hm(pi(FPB[139].joint));
}
"""
_FL = lambda e: "(float) (%s)" % e
# attribute block: (field, C expr, Rust expr)
_ATTR = [("x0", "rw()", "rw()"), ("x4", "rw()", "rw()"), ("x8", "(float) (rnd() % 5) / 4.0f", "(rnd() % 5) as f32 / 4.0"),
         ("xC", "2 + rnd() % 3", "(2 + rnd() % 3) as i32"),
         ("x10", "0.5f + (float) (rnd() % 4) * 0.5f", "0.5 + (rnd() % 4) as f32 * 0.5"),
         ("x14", "(float) (rnd() % 8) * 0.25f", "(rnd() % 8) as f32 * 0.25"), ("x18", "(float) (rnd() % 8) * 0.25f", "(rnd() % 8) as f32 * 0.25"),
         ("x1C", "(float) (rnd() % 8) * 0.25f", "(rnd() % 8) as f32 * 0.25"), ("x20", "(float) (rnd() % 8) * 0.25f", "(rnd() % 8) as f32 * 0.25"),
         ("x24", "(float) (rnd() % 8) * 0.25f", "(rnd() % 8) as f32 * 0.25"), ("x28", "(float) (rnd() % 8) * 0.25f", "(rnd() % 8) as f32 * 0.25"),
         ("x2C", "2 + rnd() % 4", "(2 + rnd() % 4) as i32"),
         ("x30", "0.5f + (float) (rnd() % 4) * 0.5f", "0.5 + (rnd() % 4) as f32 * 0.5"),
         ("x34", "(float) (rnd() % 8) * 0.25f", "(rnd() % 8) as f32 * 0.25"), ("x38", "(float) (rnd() % 8) * 0.25f", "(rnd() % 8) as f32 * 0.25"),
         ("x3C", "(float) (rnd() % 4) * 0.05f", "(rnd() % 4) as f32 * 0.05"),
         ("x40", "(float) (rnd() % 6) * 0.5f", "(rnd() % 6) as f32 * 0.5"), ("x44", "(float) (rnd() % 6) * 0.5f", "(rnd() % 6) as f32 * 0.5"),
         ("x48", "(float) (rnd() % 4) * 0.1f", "(rnd() % 4) as f32 * 0.1"),
         ("x4C", "1.0f + (float) (rnd() % 3) * 0.5f", "1.0 + (rnd() % 3) as f32 * 0.5"), ("x50", "1.0f + (float) (rnd() % 3) * 0.5f", "1.0 + (rnd() % 3) as f32 * 0.5")]
_ac = "    { itLinkHookshotAttributes* A = (itLinkHookshotAttributes*) SA;\n" + "".join("      A->%s = %s;\n" % (f, c) for f, c, r in _ATTR) + "    }\n"
_ar = "    { let A = SA.as_mut_ptr() as *mut itLinkHookshotAttributes;\n" + "".join("      (*A).%s = %s;\n" % (f, r) for f, c, r in _ATTR) + "    }\n"
_H["setup_c"] += _ac + """    fill_ftr();
    ACNT = 0; GCNT = 0; LP = 0;
    { int i; for (i = 0; i < 4; i++) { int r, c; for (r = 0; r < 3; r++) for (c = 0; c < 4; c++) JO[i].mtx[r][c] = rw(); } }
    { int r, c; for (r = 0; r < 3; r++) for (c = 0; c < 4; c++) MTXB[r][c] = rw(); }
    MF = rw();
    MPU.x8.x = rw(); MPU.x8.y = rw(); MPU.x8.z = rw(); MPU.x14.x = rw(); MPU.x14.y = rw(); MPU.x14.z = rw();
    {
        int K = 3 + rnd() % 3, i;
        for (i = 0; i < 16; i++) { zero(&LK[i], sizeof LK[i]); fill_ItemLink(&LK[i]); zero(&LG[i], sizeof LG[i]); fill_HSD_GObj(&LG[i]); }
        for (i = 0; i < 16; i++) {
            LK[i].next = (i < K && i) ? &LK[i - 1] : 0; LK[i].prev = (i < K - 1) ? &LK[i + 1] : 0;
            LK[i].gobj = &LG[i]; LK[i].jobj = &JO[(i + 1) & 3];
            LG[i].user_data = &LK[i]; LG[i].hsd_obj = &JO[i & 3];
            LK[i].x2C_b0 = (i == 0) | (i == K - 1) | (rnd() & 1); LK[i].x2C_b1 = rnd() & 1; LK[i].x2C_b2 = rnd() & 1;
        }
        LTAIL = &LK[K - 1]; LHEAD = &LK[0];
        IT.xDD4_itemVar.linkhookshot.x0 = LTAIL; IT.xDD4_itemVar.linkhookshot.x4 = LHEAD;
        IT.xDD4_itemVar.linkhookshot.x8 = &OWN; IT.xDD4_itemVar.linkhookshot.xC = &JO[1];
        IT.xDD4_itemVar.linkhookshot.x10 = (rnd() & 1) ? it_802A2EE4 : 0;
        IT2.xDD4_itemVar.linkhookshot.x0 = 0; IT2.xDD4_itemVar.linkhookshot.x4 = 0; IT2.xDD4_itemVar.linkhookshot.x8 = 0;
        IT2.xDD4_itemVar.linkhookshot.xC = 0; IT2.xDD4_itemVar.linkhookshot.x10 = 0;
        IT.owner = &OWN; IT2.owner = &OWN; OWN.user_data = &FTR; OWN.hsd_obj = &JO[0];
    }
    it_804D6D48 = (rnd() & 1) ? 0.0f : 6.0f;
"""
_H["setup_rs"] += _ar + """    fill_ftr();
    ACNT = 0; GCNT = 0; LP = null_mut();
    for i in 0..4usize { for r in 0..3usize { for c in 0..4usize { JO[i].mtx[r][c] = rw(); } } }
    for r in 0..3usize { for c in 0..4usize { MTXB[r][c] = rw(); } }
    MF = rw();
    MPU.x8.x = rw(); MPU.x8.y = rw(); MPU.x8.z = rw(); MPU.x14.x = rw(); MPU.x14.y = rw(); MPU.x14.z = rw();
    {
        let k = 3 + (rnd() % 3) as usize;
        for i in 0..16usize { LK[i] = core::mem::zeroed(); g::fill_ItemLink(&mut LK[i]); LG[i] = core::mem::zeroed(); g::fill_HSD_GObj(&mut LG[i]); }
        for i in 0..16usize {
            LK[i].next = if i < k && i != 0 { &raw mut LK[i - 1] } else { null_mut() };
            LK[i].prev = if i + 1 < k { &raw mut LK[i + 1] } else { null_mut() };
            LK[i].gobj = &raw mut LG[i]; LK[i].jobj = &raw mut JO[(i + 1) & 3];
            LG[i].user_data = &raw mut LK[i] as *mut _; LG[i].hsd_obj = &raw mut JO[i & 3] as *mut _;
            LK[i].set_x2C_b0(((i == 0) as u32 | (i + 1 == k) as u32 | (rnd() & 1)) as u8); LK[i].set_x2C_b1((rnd() & 1) as u8); LK[i].set_x2C_b2((rnd() & 1) as u8);
        }
        LTAIL = &raw mut LK[k - 1]; LHEAD = &raw mut LK[0];
        IT.xDD4_itemVar.linkhookshot.x0 = LTAIL; IT.xDD4_itemVar.linkhookshot.x4 = LHEAD;
        IT.xDD4_itemVar.linkhookshot.x8 = &raw mut OWN; IT.xDD4_itemVar.linkhookshot.xC = &raw mut JO[1];
        IT.xDD4_itemVar.linkhookshot.x10 = if rnd() & 1 != 0 { Some(m::it_802A2EE4) } else { None };
        IT2.xDD4_itemVar.linkhookshot.x0 = null_mut(); IT2.xDD4_itemVar.linkhookshot.x4 = null_mut(); IT2.xDD4_itemVar.linkhookshot.x8 = null_mut();
        IT2.xDD4_itemVar.linkhookshot.xC = null_mut(); IT2.xDD4_itemVar.linkhookshot.x10 = None;
        IT.owner = &raw mut OWN; IT2.owner = &raw mut OWN; OWN.user_data = &raw mut FTR as *mut _; OWN.hsd_obj = &raw mut JO[0] as *mut _;
    }
    m::it_804D6D48 = if rnd() & 1 != 0 { 0.0 } else { 6.0 };
"""
# normalise the pointer-valued members of the item-var union (hashed as raw bytes by the walker)
_PTRS = ["x0", "x4", "x8", "xC"]
_nc = ""
_nr = ""
for _v in ("IT", "IT2"):
    for _f in _PTRS:
        _nc += _NORMPTR_C("%s.xDD4_itemVar.linkhookshot.%s" % (_v, _f))
        _nr += _NORMPTR_RS("%s.xDD4_itemVar.linkhookshot.%s" % (_v, _f))
    _nc += """    { u32 id = FI((const void*) %s.xDD4_itemVar.linkhookshot.x10); unsigned char* b = (unsigned char*) &%s.xDD4_itemVar.linkhookshot.x10; int i2; for (i2 = 0; i2 < 8; i2++) b[i2] = 0; *(u32*) b = id; }
""" % (_v, _v)
    _nr += """    { let id = fi(%s.xDD4_itemVar.linkhookshot.x10); let b = &raw mut %s.xDD4_itemVar.linkhookshot.x10 as *mut u8; for i2 in 0..8 { *b.add(i2) = 0; } *(b as *mut u32) = id; }
""" % (_v, _v)
_H["pre_hash_c"] = _nc
_H["pre_hash_rs"] = _nr
_H["hash_c"] = """    hash_ftr();
    { int i; for (i = 0; i < 16; i++) { hash_ItemLink(&LK[i]); hash_HSD_GObj(&LG[i]); } }
    { int i, r, c; for (i = 0; i < 4; i++) for (r = 0; r < 3; r++) for (c = 0; c < 4; c++) HM(B(JO[i].mtx[r][c])); for (r = 0; r < 3; r++) for (c = 0; c < 4; c++) HM(B(MTXB[r][c])); }
    HM(B(MF)); HM(B(it_804D6D48)); HM(PI(LP)); HM(ACNT); HM(GCNT);
"""
_H["hash_rs"] = """    hash_ftr();
    for i in 0..16usize { g::hash_ItemLink(&LK[i]); g::hash_HSD_GObj(&LG[i]); }
    for i in 0..4usize { for r in 0..3usize { for c in 0..4usize { hm(JO[i].mtx[r][c].to_bits()); } } }
    for r in 0..3usize { for c in 0..4usize { hm(MTXB[r][c].to_bits()); } }
    hm(MF.to_bits()); hm(m::it_804D6D48.to_bits()); hm(pi(LP)); hm(ACNT as u32); hm(GCNT as u32);
"""
_H["ptr_params"] = {
    "ItemLink": [("LTAIL", "LTAIL"), ("LHEAD", "LHEAD")],
    "ItemLink *": [("&LP", "&raw mut LP")],
    "itLinkHookshotAttributes": [("(itLinkHookshotAttributes*) SA", "SA.as_mut_ptr() as *mut itLinkHookshotAttributes")],
    "Fighter": [("&FTR", "&raw mut FTR")],
    "float[4]": [("MTXB", "MTXB.as_mut_ptr()")],
    "Mtx": [("MTXB", "MTXB.as_mut_ptr()")],
    "float": [("&MF", "&raw mut MF")],
    "void": [("&IT", "(&raw mut IT) as *mut c_void")],
}


def _op(n, c, r):
    _H.setdefault("ops", {})[n] = (c, r)


_op("it_802A2428", "it_802A2428(&G);", "m::it_802A2428(&raw mut G);")
for _n in ("normalize_diff", "normalize_diff_rev"):
    pass
for _n in ("it_802A6A78_normalize_diff", "it_802A6A78_normalize_diff_rev", "it_802A4BFC_normalize_diff"):
    _op(_n, "{ double d = %s(&V1, &V2, &IT.pos); HM64(d); }" % _n, "{ let d = m::%s(&raw mut V1, &raw mut V2, &raw mut IT.pos); hm64(d); }" % _n)
_op("it_802A3C98", "ret = B(it_802A3C98(&V1, &V2, &IT.pos));", "ret = m::it_802A3C98(&raw mut V1, &raw mut V2, &raw mut IT.pos).to_bits();")
_op("test_comp", "test_comp(&IT.pos, &V1, &V2, &MF);", "m::test_comp(&raw mut IT.pos, &raw mut V1, &raw mut V2, &raw const MF);")
_op("it_802A2BA4", "ret = PI(it_802A2BA4(&OWN, &V1, rw(), (s32) fv()));", "ret = pi(m::it_802A2BA4(&raw mut OWN, &raw mut V1, rw(), fv() as i32));")
for _n in ("it_802A7AAC", "it_802A7AF0", "it_802A7B34"):
    _op(_n, "%s(&OWN);" % _n, "m::%s(&raw mut OWN);" % _n)
_op("it_802A2568", "ret = PI(it_802A2568(&IT, &JO[1], (s32) (fv() % 2), (rnd() & 1) ? ((rnd() & 1) ? 0.5f : 1.0f) : 2.0f));",
    "ret = pi(m::it_802A2568(&raw mut IT, &raw mut JO[1], (fv() % 2) as i32, if rnd() & 1 != 0 { if rnd() & 1 != 0 { 0.5 } else { 1.0 } } else { 2.0 }));")
_op("it_link_attr_math", "it_link_attr_math((itLinkHookshotAttributes*) SA, (s32) (fv() % 2), (rnd() & 1) ? ((rnd() & 1) ? 0.5f : 1.0f) : 2.0f);",
    "m::it_link_attr_math(SA.as_mut_ptr() as *mut itLinkHookshotAttributes, (fv() % 2) as i32, if rnd() & 1 != 0 { if rnd() & 1 != 0 { 0.5 } else { 1.0 } } else { 2.0 });")
_op("it_802A3E50", "ret = (u32) it_802A3E50(LTAIL, (rnd() & 1) ? 20 : 6, rw());", "ret = m::it_802A3E50(LTAIL, if rnd() & 1 != 0 { 20 } else { 6 }, rw()) as u32;")
_op("it_802A6DC8", "it_802A6DC8(&JO[1], &V1, &V2);", "m::it_802A6DC8(&raw mut JO[1], &raw mut V1, &raw mut V2);")
_op("it_802A6F80", "it_802A6F80(&JO[1], &V1, &V2, rw());", "m::it_802A6F80(&raw mut JO[1], &raw mut V1, &raw mut V2, rw());")
_op("it_802A24A0", "it_802A24A0(&LG[0], (int) fv());", "m::it_802A24A0(&raw mut LG[0], fv() as i32);")
_H["sa_align"] = True
_H["skip"] = ["it_802A2EE4"]
_H["skip"] = []
