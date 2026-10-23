"""Driver descriptions for mk_it_driver.py (one per TU)."""
from mk_it_driver import hot

CFG = {}
CFG["itfire"] = dict(
    tables=[("it_803F7C10", 3)],
)

_c1, _r1 = hot("SA[1]", ["1", "2", "3", "20"])
_c2, _r2 = hot("IT.xDD4_itemVar.egg.x0", ["0", "1"])
_c3, _r3 = hot("IT.xDD4_itemVar.egg.rand_max", ["0", "1", "2", "40"])
CFG["itegg"] = dict(
    icd=True, skip=["attrRand"], tables=[("it_803F5988", 7)],
    setup_c=_c1 + _c2 + _c3, setup_rs=_r1 + _r2 + _r3,
)

CFG["itentei"] = dict(icd=False, tables=[("it_803F7DC8", 2)])
for _p_, _v_ in (("IT.xDD4_itemVar.pokemon.timer", ["0", "1", "2", "3"]), ("IT.ground_or_air", ["0", "1"]),
                 ("IT.xDB0_itcmd_var1", ["0", "1"]), ("IT.xDB4_itcmd_var2", ["0", "1"]), ("IT.xDAC_itcmd_var0", ["0", "1"])):
    _c_, _r_ = hot(_p_, _v_)
    CFG["itentei"]["setup_c"] = CFG["itentei"].get("setup_c", "") + _c_
    CFG["itentei"]["setup_rs"] = CFG["itentei"].get("setup_rs", "") + _r_


def _hots(stem, items, **kw):
    cfg = dict(kw)
    cfg["setup_c"] = ""
    cfg["setup_rs"] = ""
    for p, v in items:
        c, r = hot(p, v)
        cfg["setup_c"] += c
        cfg["setup_rs"] += r
    CFG[stem] = cfg


_hots("itevyoshiegg", [("IT.xDD4_itemVar.evyoshiegg.xDD8", ["0", "1", "40", "41", "60"]), ("IT.xDD4_itemVar.evyoshiegg.xDD4", ["0", "1"]),
                       ("IT.xC9C", ["-1", "0", "1", "5"]), ("SA[0]", ["0", "1", "5"])],
      icd=True, tables=[("it_803F6728", 6)])

_hots("itfflower", [("IT.msid", ["0", "1", "2", "3", "4", "5", "6", "7"]), ("IT.xD4C", ["-1", "0", "1", "5"]),
                    ("IT.xDD4_itemVar.fflower.x0", ["0", "1", "2", "5"]), ("IT.xDD4_itemVar.fflower.x4", ["0", "1"])],
      icd=False, tables=[("it_803F6090", 8)])

_hots("itfflowerflame", [], icd=False, tables=[("it_803F66C8", 5)], extra_ops=[
    ("it_8029A89C_null", "it_8029A89C(0, rw());", "m::it_8029A89C(null_mut(), rw());"),
    ("it_8029A8F4_null", "it_8029A8F4(0, &V1);", "m::it_8029A8F4(null_mut(), &raw mut V1);"),
    ("it_8029A89C_noitem", "G.user_data = 0; it_8029A89C(&G, rw());", "G.user_data = null_mut(); m::it_8029A89C(&raw mut G, rw());"),
    ("it_8029A8F4_noitem", "G.user_data = 0; it_8029A8F4(&G, &V1);", "G.user_data = null_mut(); m::it_8029A8F4(&raw mut G, &raw mut V1);")])

_hots("itfoods", [], icd=False, tables=[("it_803F5DB0", 4)], skip=["getRandMax"],
      extra_ops=[("it_8028FAF4_null", "ret = PI(it_8028FAF4(0, &V1));", "ret = pi(m::it_8028FAF4(null_mut(), &raw mut V1));")])
# the attribute table is indexed by values the TU reads out of state: keep them in range
CFG["itfoods"]["setup_c"] += "    IT.xDD4_itemVar.foods.x0 = rnd() % 4; SA[0] = 1 + rnd() % 8;\n"
CFG["itfoods"]["setup_rs"] += "    IT.xDD4_itemVar.foods.x0 = rnd() % 4; SA[0] = 1 + rnd() % 8;\n"


def hotf(idx, vals):
    """SA[idx] (a float attribute) forced to one of vals with probability 3/4."""
    n = len(vals)
    ex = "%sf" % vals[-1]
    for i in range(n - 2, -1, -1):
        ex = "(q == %d ? %sf : %s)" % (i, vals[i], ex)
    c = "    if (rnd() %% 4 != 0) { u32 q = rnd() %% %d; SA[%d] = B(%s); }\n" % (n, idx, ex)
    arms = " ".join("%d => %s," % (i, vals[i]) for i in range(n - 1))
    r = "    if rnd() %% 4 != 0 { let q = rnd() %% %d; SA[%d] = (match q { %s _ => %s } as f32).to_bits(); }\n" % (n, idx, arms, vals[-1])
    return c, r


_NORM_C = """    {
        Item* its[2]; int k2, i2;
        its[0] = &IT; its[1] = &IT2;
        for (k2 = 0; k2 < 2; k2++) {
            u32 id = PI(its[k2]->xDD4_itemVar.flipper.xDEC_stageAnchor);
            unsigned char* b = (unsigned char*) &its[k2]->xDD4_itemVar.flipper.xDEC_stageAnchor;
            for (i2 = 0; i2 < 8; i2++) b[i2] = 0;
            *(u32*) b = id;
        }
    }
"""
_NORM_RS = """    {
        let its: [*mut Item; 2] = [&raw mut IT, &raw mut IT2];
        for k2 in 0..2 {
            let id = pi((*its[k2]).xDD4_itemVar.flipper.xDEC_stageAnchor);
            let b = &raw mut (*its[k2]).xDD4_itemVar.flipper.xDEC_stageAnchor as *mut u8;
            for i2 in 0..8 { *b.add(i2) = 0; }
            *(b as *mut u32) = id;
        }
    }
"""
_hots("itflipper", [("IT.xC38", ["20", "19"]), ("IT.xDD4_itemVar.flipper.xDD4_flightTimer", ["-1", "0", "1", "5", "6", "10", "11", "12"]),
                    ("IT.xDD4_itemVar.flipper.xDD8_isSettled", ["0", "1"]), ("IT.xDD4_itemVar.flipper.xDE8_isStageFixed", ["0", "1"]),
                    ("IT.xDD4_itemVar.flipper.xDDC_hitboxTimer", ["0", "1", "2"]),
                    ("IT.xDD4_itemVar.flipper.xDE4_spinVel", ["0.0", "0.01", "-0.01", "0.5", "-0.5", "3.5", "-3.5", "7.0"], "f"),
                    ("IT.xD44_lifeTimer", ["0.5", "1.0", "2.0", "30.0"], "f"), ("ICD.x34", ["0", "1", "2", "30"])] if False else [],
      icd=True, tables=[("ItemStateTable_Flipper", 7)])
for _p_, _v_, _k_ in (("IT.xC38", ["20", "19"], "i"), ("IT.xDD4_itemVar.flipper.xDD4_flightTimer", ["-1", "0", "1", "5", "6", "10", "11", "12"], "i"),
                      ("IT.xDD4_itemVar.flipper.xDD8_isSettled", ["0", "1"], "i"), ("IT.xDD4_itemVar.flipper.xDE8_isStageFixed", ["0", "1"], "i"),
                      ("IT.xDD4_itemVar.flipper.xDDC_hitboxTimer", ["0", "1", "2"], "i"),
                      ("IT.xDD4_itemVar.flipper.xDE4_spinVel", ["0.0", "0.01", "-0.01", "0.5", "-0.5", "3.5", "-3.5", "7.0"], "f"),
                      ("IT.xD44_lifeTimer", ["0.5", "1.0", "2.0", "30.0"], "f"), ("ICD.x34", ["0", "1", "2", "30"], "i")):
    _c_, _r_ = hot(_p_, _v_, _k_)
    CFG["itflipper"]["setup_c"] += _c_
    CFG["itflipper"]["setup_rs"] += _r_
for _i_, _v_ in ((8, ["0.0", "0.1", "0.5", "5.0"]), (9, ["0.5", "3.0", "10.0"]), (7, ["0.0", "0.3", "2.0"]), (6, ["0.5", "1.0", "2.0"])):
    _c_, _r_ = hotf(_i_, _v_)
    CFG["itflipper"]["setup_c"] += _c_
    CFG["itflipper"]["setup_rs"] += _r_
CFG["itflipper"]["setup_c"] += """    IT.xDD4_itemVar.flipper.xDEC_stageAnchor = (rnd() & 1) ? &JO[1] : 0; IT2.xDD4_itemVar.flipper.xDEC_stageAnchor = 0;
    IT.xCF4_fighterGObjUnk = (rnd() & 1) ? &OWN : 0; IT.xCEC_fighterGObj = (rnd() & 1) ? &OWN : 0;
"""
CFG["itflipper"]["setup_rs"] += """    IT.xDD4_itemVar.flipper.xDEC_stageAnchor = if rnd() & 1 != 0 { &raw mut JO[1] } else { null_mut() }; IT2.xDD4_itemVar.flipper.xDEC_stageAnchor = null_mut();
    IT.xCF4_fighterGObjUnk = if rnd() & 1 != 0 { &raw mut OWN } else { null_mut() }; IT.xCEC_fighterGObj = if rnd() & 1 != 0 { &raw mut OWN } else { null_mut() };
"""
CFG["itflipper"]["pre_hash_c"] = _NORM_C
CFG["itflipper"]["pre_hash_rs"] = _NORM_RS

# ---- shard 713-720
CFG["itkirbyhammer"] = dict(
    icd=False, tables=[("it_803F6C98", 1)],
    ops={"setupHammerParticles": ("setupHammerParticles(&G, &G2, fv(), rnd() % 3);", "m::setupHammerParticles(&raw mut G, &raw mut G2, fv(), rnd() % 3);"),
         "it_802ADC54": ("ret = PI(it_802ADC54(&G, &V1, fv(), rnd() % 3, rw()));", "ret = pi(m::it_802ADC54(&raw mut G, &raw mut V1, fv(), rnd() % 3, rw()));")},
)
