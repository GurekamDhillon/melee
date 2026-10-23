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
