"""pobj spec (autospec, then hand edited)."""
from lbspecs import add

POOLS = [('HSD_PObj', 'P0', 3), ('HSD_ShapeAnim', 'P1', 3), ('HSD_ShapeSet', 'P2', 3), ('HSD_ObjData', 'P3', 3), ('HSD_Envelope', 'P4', 3), ('HSD_SList', 'P5', 3), ('HSD_ShapeSetDesc', 'P6', 3), ('HSD_PObjDesc', 'P7', 3), ('HSD_PObjInfo', 'P8', 3), ('HSD_VtxDescList', 'P9', 3), ('HSD_JObj', 'P10', 3), ('HSD_Class', 'P11', 3), ('HSD_ClassInfo', 'P12', 3)]
add("pobj", "src/sysdolphin/baselib/pobj.c", dict(pub_only=True, pools=POOLS),
    pub_only=True, types=[p[0] for p in POOLS], pools=[p[0] for p in POOLS])
