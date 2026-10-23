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
