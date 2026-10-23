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
