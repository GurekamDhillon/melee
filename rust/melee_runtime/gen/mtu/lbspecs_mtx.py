"""mtx.c spec (see lbspecs.py): the HSD_ObjAlloc pool wrappers (the matrix arithmetic is checked by the oracle_* / pure_* tests)."""
from lbspecs import add

OWN = ["HSD_Mtx_804C2310", "HSD_Mtx_804C233C"]
add("mtx", "src/sysdolphin/baselib/mtx.c", dict(pub_only=True, ncases=300, skip=["HSD_MkRotationMtx", "HSD_MtxInverseTranspose", "HSD_MtxGetTranslate", "HSD_MtxScaledAdd", "HSD_MtxSRTQuat"]),
    pub_only=True, own_fill=OWN, own_hash=OWN, skip_stubs=["__frsqrte", "sinf", "cosf", "memset", "memcpy"],
    ret={"HSD_ObjAlloc": ("((fv() & 3) ? (void*) BUF : 0)", "if fv() & 3 != 0 { (&raw mut BUF) as *mut c_void } else { null_mut() }")})
