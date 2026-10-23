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
