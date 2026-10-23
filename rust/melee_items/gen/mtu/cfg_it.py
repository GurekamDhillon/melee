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
