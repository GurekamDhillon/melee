"""lbcollision spec: the capsule-vs-capsule test lbColl_80006E58 and the shield-hit wrapper lbColl_80007BCC."""
from lbspecs import add

POOLS = [('HitCapsule', 'P0', 2), ('HitResult', 'P2', 2), ('HSD_JObj', 'P8', 2)]
ONLY = ["lbColl_80006E58", "lbColl_80007BCC"]
add("lbcollision", "src/melee/lb/lbcollision.c", dict(
    pub_only=True, pools=POOLS, only=ONLY, ncases=1500,
    setup=("{ int q; for (q = 0; q < 2; q++) { P2[q].bone = &P8[q]; P2[q].skip_update_pos = fv() & 1; } }",
           "{ for q in 0..2usize { P2[q].bone = &raw mut P8[q]; P2[q].set_skip_update_pos((fv() & 1) as _); } }"),
    ov={"lbColl_80007BCC": {2: ("((fv() & 1) ? (void*) BUF : 0)", "if fv() & 1 != 0 { (&raw mut BUF) as *const Mtx } else { core::ptr::null() }")}}),
    pub_only=True, types=[p[0] for p in POOLS], pools=[p[0] for p in POOLS], own_only=ONLY,
    skip_stubs=["memcpy", "memset", "sqrtf", "__frsqrte"])
