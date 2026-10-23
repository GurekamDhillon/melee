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


# ---- shard 713-720
CFG["itkirbyhammer"] = dict(
    icd=False, tables=[("it_803F6C98", 1)],
    ops={"setupHammerParticles": ("setupHammerParticles(&G, &G2, fv(), rnd() % 3);", "m::setupHammerParticles(&raw mut G, &raw mut G2, fv(), rnd() % 3);"),
         "it_802ADC54": ("ret = PI(it_802ADC54(&G, &V1, fv(), rnd() % 3, rw()));", "ret = pi(m::it_802ADC54(&raw mut G, &raw mut V1, fv(), rnd() % 3, rw()));")},
)

# a model descriptor behind ART.x10_modelDesc (the bone id the TU hands to it_80274F48 / it_80275070)
MD_C = "static ItemModelDesc MD;\n"
MD_RS = "static mut MD: ItemModelDesc = unsafe { core::mem::zeroed() };\n"
MD_PREG = [("&MD", "sizeof MD", "&raw const MD", "core::mem::size_of::<ItemModelDesc>()")]
MD_SETUP_C = "    zero(&MD, sizeof MD); MD.x8_bone_attach_id = (s32) (rnd() % 12) - 2; ART.x10_modelDesc = &MD;\n"
MD_SETUP_RS = "    MD = core::mem::zeroed(); MD.x8_bone_attach_id = (rnd() % 12) as i32 - 2; ART.x10_modelDesc = &raw mut MD;\n"
CFG["itkirbyyoshispecialn"] = dict(
    icd=False, tables=[("it_803F9490", 1)], c_decl=MD_C, rs_decl=MD_RS, pregs=MD_PREG, setup_c=MD_SETUP_C, setup_rs=MD_SETUP_RS,
    ops={"it_802F2D98": ("it_802F2D98(&G, &OWN);", "m::it_802F2D98(&raw mut G, &raw mut OWN);"),
         "it_802F2E7C": ("it_802F2E7C(&G, (int) (fv() & 1));", "m::it_802F2E7C(&raw mut G, (fv() & 1) as i32);")},
)

_hots3("itkirby_2F23", [("IT.xDD4_itemVar.kirby2f23.x1D0", ["0.5", "5.0", "20.0"], "f"), ("IT.xDD4_itemVar.kirby2f23.x1D8", ["0.5", "5.0", "100.0"], "f"),
                       ("IT.xDD4_itemVar.kirby2f23.x1DC", ["0.5", "5.0", "100.0"], "f"), ("IT.xDD4_itemVar.kirby2f23.x1C8", ["0.0", "3.0", "-3.0", "12.0", "-12.0"], "f"),
                       ("IT.xDD4_itemVar.kirby2f23.x1CC", ["0.0", "3.0", "-3.0", "12.0", "-12.0"], "f")],
      icd=False, tables=[("it_803F9450", 2)],
      ops={"it_802F23AC": ("{ double d = it_802F23AC(&G, &V1); HM64(d); }", "{ let d = m::it_802F23AC(&raw mut G, &raw mut V1); hm64(d); }"),
           "it_802F23EC": ("it_802F23EC(&G, &OWN, rw());", "m::it_802F23EC(&raw mut G, &raw mut OWN, rw());"),
           "it_802F23EC_inline": ("it_802F23EC_inline(&G);", "m::it_802F23EC_inline(&raw mut G);"),
           "it_802F258C_scale": ("it_802F258C_scale(&G, &V1);", "m::it_802F258C_scale(&raw mut G, &raw mut V1);"),
           "it_802F258C_update": ("it_802F258C_update(&G, &V1);", "m::it_802F258C_update(&raw mut G, &raw mut V1);")})

_hots3("itkireihana", [("IT.xDD4_itemVar.kireihana.timer_unk0", ["-1", "0", "1", "2", "5"], "i"), ("IT.xDD4_itemVar.kireihana.timer_unk1", ["0", "1", "2", "3"], "i"),
                       ("IT.xDD4_itemVar.kireihana.timer_unk2", ["0", "1", "2"], "i"), ("IT.xDD4_itemVar.kireihana.x8", ["-1.0", "0.0", "1.0"], "f"),
                       ("IT.xDD4_itemVar.kireihana.x70", ["-1.0", "0.0", "1.0"], "f"), ("IA.x10_fall_speed", ["0.1", "0.5"], "f"), ("IA.x14_fall_speed_max", ["0.3", "2.0"], "f")],
       icd=False, tables=[("it_803F7E28", 5)])

_KLAP_NORM_C = """    {
        Item* its[2]; int k2, i2;
        its[0] = &IT; its[1] = &IT2;
        for (k2 = 0; k2 < 2; k2++) {
            u32 id = PI(its[k2]->xDD4_itemVar.klap.x20);
            unsigned char* b = (unsigned char*) &its[k2]->xDD4_itemVar.klap.x20;
            for (i2 = 0; i2 < 8; i2++) b[i2] = 0;
            *(u32*) b = id;
        }
    }
"""
_KLAP_NORM_RS = """    {
        let its: [*mut Item; 2] = [&raw mut IT, &raw mut IT2];
        for k2 in 0..2 {
            let id = pi((*its[k2]).xDD4_itemVar.klap.x20);
            let b = &raw mut (*its[k2]).xDD4_itemVar.klap.x20 as *mut u8;
            for i2 in 0..8 { *b.add(i2) = 0; }
            *(b as *mut u32) = id;
        }
    }
"""
_hots3("itklap", [("IT.msid", ["0", "1"], "i"), ("IA.x10_fall_speed", ["0.1", "0.5"], "f"),
                  ("IT.xDD4_itemVar.klap.x24", ["0.0", "0.5", "-0.5", "7.0", "-7.0"], "f"),
                  ("IT.xDD4_itemVar.klap.x28", ["0.0", "0.5", "-0.5", "7.0", "-7.0"], "f"),
                  ("JO[0].rotate.x", ["0.0", "1.5", "1.6", "1.55", "3.0"], "f"),
                  ("JO[0].rotate.y", ["0.0", "6.0", "-6.0", "6.3", "-6.3"], "f"),
                  ("JO[0].rotate.z", ["0.0", "6.0", "-6.0", "6.3", "-6.3"], "f")],
       icd=False, tables=[("it_803F8940", 5)],
       pre_hash_c=_KLAP_NORM_C, pre_hash_rs=_KLAP_NORM_RS,
       ops={"it_802E2330": ("it_802E2330(&G, &V1, (lbColl_80008D30_arg1*) &V2, rw());", "m::it_802E2330(&raw mut G, &raw mut V1, &raw mut V2 as *mut lbColl_80008D30_arg1, rw());")})
CFG["itklap"]["setup_c"] += "    IT.xDD4_itemVar.klap.x20 = (rnd() & 1) ? &G2 : 0; IT2.xDD4_itemVar.klap.x20 = 0;\n"
CFG["itklap"]["setup_rs"] += "    IT.xDD4_itemVar.klap.x20 = if rnd() & 1 != 0 { &raw mut G2 } else { null_mut() }; IT2.xDD4_itemVar.klap.x20 = null_mut();\n"

_hots3("itkoopaflame", [("IT.kind", ["It_Kind_Koopa_Flame", "It_Kind_Kirby_KoopaFlame", "5"], "i"),
                        ("IT2.kind", ["It_Kind_Koopa_Flame", "It_Kind_Kirby_KoopaFlame", "5"], "i"),
                        ("IT.x378_itemColl.env_flags", ["0", "1", "0x40", "0x2000", "0x8000", "0x8001", "0x2041", "0xA041", "0x10000"], "i"),
                        ("IT.x5D4_hitboxes[0].hit.state", ["0", "1"], "i"),
                        ("IT.xDD4_itemVar.koopaflame.x34_base_scale", ["0.0", "1.0", "2.0"], "f"),
                        ("IT.xDD4_itemVar.koopaflame.x44_spawned", ["0", "1"], "i"),
                        ("IT.xDD4_itemVar.koopaflame.x40_frame_counter", ["0", "10", "19", "20", "21"], "i"),
                        ("IT.xDD4_itemVar.koopaflame.x24_angle", ["0.0", "1.0", "-1.0", "3.0", "-3.0", "4.0", "7.0"], "f"),
                        ("IT.facing_dir", ["1.0", "-1.0"], "f")],
       icd=False, tables=[("ItemStateTable_KoopaFlame", 1)],
       ops={"itKoopaFlame_Spawn": ("ret = PI(itKoopaFlame_Spawn(&OWN, &V1, rw(), fv(), (s32) fv(), (s32) (fv() % 9), (s32) (fv() % 9), (s32) (fv() % 3 ? 5 : 6)));",
                                   "ret = pi(m::itKoopaFlame_Spawn(&raw mut OWN, &raw mut V1, rw(), fv(), fv() as i32, (fv() % 9) as i32, (fv() % 9) as i32, if fv() % 3 != 0 { 5 } else { 6 }));"),
            "itKoopaFlame_Update_Direction": ("itKoopaFlame_Update_Direction(&G, (int) (rnd() % 16));", "m::itKoopaFlame_Update_Direction(&raw mut G, (rnd() % 16) as i32);"),
            "itKoopaFlame_Update_Angle": ("itKoopaFlame_Update_Angle(&G, (int) (rnd() % 16));", "m::itKoopaFlame_Update_Angle(&raw mut G, (rnd() % 16) as i32);")})

_c_, _r_ = hotf(1, ["20.0", "0.5", "10.0"])
# the attribute floats feed the angle wrap loops: keep them finite (rg: -1..1, 0, NaN)
CFG["itkoopaflame"]["setup_c"] += "    for (i = 0; i < 6; i++) SA[i] = B(rg());" + chr(10)
CFG["itkoopaflame"]["setup_rs"] += "    for i in 0..6 { SA[i] = rg().to_bits(); }" + chr(10)
CFG["itkoopaflame"]["setup_c"] += _c_
CFG["itkoopaflame"]["setup_rs"] += _r_

_hots3("itkyasarin", [("IT.msid", ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10"], "i"),
                      ("IT.facing_dir", ["1.0", "-1.0"], "f"),
                      ("IT.xDD4_itemVar.kyasarin.x24", ["0.0", "0.5", "1.0", "-0.5"], "f"),
                      ("IT.xDD4_itemVar.kyasarin.x28", ["0.0", "0.5", "1.0", "-0.5"], "f"),
                      ("IT.xDD4_itemVar.kyasarin.x2C", ["0.0", "0.5", "1.0", "-0.5"], "f"),
                      ("IT.xDD4_itemVar.kyasarin.x30", ["1.0", "-1.0", "0.5"], "f"),
                      ("IT.xDD4_itemVar.kyasarin.x34", ["-1", "0", "1", "2", "5"], "i"),
                      ("IT.xDD4_itemVar.kyasarin.x38", ["0", "10", "100", "1000", "-5"], "i"),
                      ("IT.xDD4_itemVar.kyasarin.x3C", ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"], "i"),
                      ("IT.xCA0", ["0", "1", "5", "50", "500"], "i"),
                      ("IT.x5D0_animFrameSpeed", ["0.0", "0.5", "1.0", "2.0"], "f"),
                      ("IT.pos.x", ["-50.0", "-10.0", "0.0", "10.0", "50.0"], "f")],
       icd=False, tables=[("it_803F90D0", 11)],
       skip=["itKyasarin_Randi", "itKyasarin_TurnAround", "itKyasarin_UnkMotion8_case3", "itKyasarin_UnkMotion8_case8", "itKyasarin_SetFallCollPos", "itKyasarin_GetCollData", "itKyasarin_FlipAndFall", "itKyasarin_SaveStateAndStop"])
CFG["itkyasarin"]["c_decl"] = "static it_804D6D40_t D40;\n"
CFG["itkyasarin"]["rs_decl"] = "static mut D40: it_804D6D40_t = unsafe { core::mem::zeroed() };\n"
CFG["itkyasarin"]["setup_c"] += "    zero(&D40, sizeof D40); it_804D6D40 = &D40;\n"
CFG["itkyasarin"]["setup_rs"] += "    D40 = core::mem::zeroed(); m::ext::it_804D6D40 = &raw mut D40;\n"
for _p_, _v_ in ((5, ["20.0", "3.0", "0.5", "-1.0"]), (7, ["20.0", "3.0", "0.5", "-1.0"]), (11, ["-20.0", "0.0", "20.0"]), (10, ["-20.0", "0.0", "20.0"]),
                 (13, ["-20.0", "0.0", "20.0"]), (12, ["-20.0", "0.0", "20.0"])):
    _c_, _r_ = hotf(_p_, _v_)
    CFG["itkyasarin"]["setup_c"] += _c_
    CFG["itkyasarin"]["setup_rs"] += _r_
for _p_ in ("SA[16]", "SA[17]"):
    _c_, _r_ = hot(_p_, ["0", "10", "100", "1000"])
    CFG["itkyasarin"]["setup_c"] += _c_
    CFG["itkyasarin"]["setup_rs"] += _r_
_c_, _r_ = hot("D40.x8", ["0.0", "0.5", "1.0"], "f")
CFG["itkyasarin"]["setup_c"] += _c_
CFG["itkyasarin"]["setup_rs"] += _r_

_hots3("itkusudama", [("IT.xDD4_itemVar.kusudama.x0", ["0", "1"], "i"), ("IT.xDD4_itemVar.kusudama.x4", ["-1", "0", "1", "5"], "i"),
                      ("IT.xDB4_itcmd_var2", ["0", "1", "2"], "i"), ("IT.xDB0_itcmd_var1", ["0", "1"], "i"), ("IT.xDAC_itcmd_var0", ["0", "1"], "i"),
                      ("IT.facing_dir", ["1.0", "-1.0"], "f"), ("IT.xC9C", ["0", "5", "50"], "i"),
                      ("IT.x378_itemColl.env_flags", ["0", "0x2000", "0x4000", "0x6000", "1"], "i"),
                      ("IT.x40_vel.y", ["0.0", "-3.0", "3.0", "-0.5"], "f"),
                      ("JO[0].rotate.x", ["0.0", "1.5", "-1.5", "3.0", "-3.0", "0.1"], "f"),
                      ("JO[0].rotate.y", ["0.0", "1.6", "-1.6", "4.7", "-4.7", "2.5"], "f"),
                      ("IA.x10_fall_speed", ["0.1", "0.5"], "f"), ("IA.x14_fall_speed_max", ["0.3", "2.0"], "f"),
                      ("SA[0]", ["0", "1", "5"], "i"), ("SA[1]", ["0", "1", "5"], "i"), ("SA[2]", ["0", "1", "5"], "i"), ("SA[3]", ["0", "1", "5"], "i"),
                      ("SA[4]", ["34", "18", "5", "0"], "i"), ("SA[5]", ["0", "1", "3", "15"], "i"),
                      ("SA[16]", ["0"], "i")],
       icd=True, tables=[("it_803F59F8", 9)],
       ops={"it_80289BE8": ("it_80289BE8(&G, (s32) (rnd() % 4), (s32) (rnd() % 4), (s32) (rnd() % 4));", "m::it_80289BE8(&raw mut G, (rnd() % 4) as i32, (rnd() % 4) as i32, (rnd() % 4) as i32);")})
for _i_, _v_ in ((6, ["0.0", "3.0", "-3.0"]), (7, ["0.0", "30.0", "-30.0"]), (10, ["0.0", "0.1", "0.5", "3.0"]), (11, ["0.0", "0.5", "3.0"]), (8, ["0.0", "5.0", "50.0"])):
    _c_, _r_ = hotf(_i_, _v_)
    CFG["itkusudama"]["setup_c"] += _c_
    CFG["itkusudama"]["setup_rs"] += _r_
CFG["itkusudama"]["skip"] = ["it_8028A544_inline", "it_80289BE8_inline", "it_80289BE8_spawn_random", "it_80289BE8_spawn", "it_8028A190_inline",
                             "itKusudama_UnkMotion3_inline", "itKusudama_UnkMotion5_Coll_inline", "itKusudama_UnkMotion6_Coll_inline2",
                             "itKusudama_UnkMotion6_Coll_inline", "itKusudama_UnkMotion8_Anim_inline", "itKusudama_Logic4_DmgDealt_inline",
                             "itKusudama_Logic4_Clanked_inline"]
# Item_ClampAngle spins forever on a huge or infinite angle: the 1/4 of cases the hot-value draw leaves random
CFG["itkoopaflame"]["setup_c"] += "    if (!(IT.xDD4_itemVar.koopaflame.x24_angle > -50.0f && IT.xDD4_itemVar.koopaflame.x24_angle < 50.0f)) IT.xDD4_itemVar.koopaflame.x24_angle = 0.5f;" + chr(10)
CFG["itkoopaflame"]["setup_rs"] += "    if !(IT.xDD4_itemVar.koopaflame.x24_angle > -50.0 && IT.xDD4_itemVar.koopaflame.x24_angle < 50.0) { IT.xDD4_itemVar.koopaflame.x24_angle = 0.5; }" + chr(10)
# the burst loops index a 15-entry array by attr->x14 (SA[5]): keep it in range
CFG["itkusudama"]["setup_c"] += "    if ((int) SA[5] > 15) SA[5] &= 15;" + chr(10)
CFG["itkusudama"]["setup_rs"] += "    if (SA[5] as i32) > 15 { SA[5] &= 15; }" + chr(10)
# kyasarin.x20 is an 8-byte pointer inside the raw itemVar bytes that are hashed: replace it by its fixture id
CFG["itkyasarin"]["pre_hash_c"] = _KLAP_NORM_C.replace("klap.x20", "kyasarin.x20")
CFG["itkyasarin"]["pre_hash_rs"] = _KLAP_NORM_RS.replace("klap.x20", "kyasarin.x20")
