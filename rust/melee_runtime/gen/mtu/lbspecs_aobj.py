"""aobj spec (autospec, then hand edited)."""
from lbspecs import add

POOLS = [('HSD_AObj', 'P0', 3), ('HSD_FObj', 'P1', 3), ('HSD_AObjDesc', 'P2', 3)]
add("aobj", "src/sysdolphin/baselib/aobj.c", dict(pub_only=True, pools=POOLS),
    pub_only=True, types=[p[0] for p in POOLS], pools=[p[0] for p in POOLS])
