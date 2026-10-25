"""psdisptev.c spec: particle TEV state selection (the cached key, the TEV stage / order / op setup per particle kind)."""
from lbspecs import add

POOLS = [('HSD_Particle', 'P0', 3)]
OWN = ["prevTev"]
add("psdisptev", "src/sysdolphin/baselib/psdisptev.c", dict(
    pub_only=True, pools=POOLS, ncases=600,
    setup=("{ int q; for (q = 0; q < 3; q++) P0[q].kind = fv() ^ (fv() << 11) ^ (fv() << 22) ^ (fv() << 28); }",
           "{ for q in 0..3usize { P0[q].kind = fv() ^ (fv() << 11) ^ (fv() << 22) ^ (fv() << 28); } }"),
    pregs=("", "melee_runtime::tu::lbext::H_OSReport = Some(|_f, _a| ());")),
    pub_only=True, types=[p[0] for p in POOLS], pools=[p[0] for p in POOLS], own_fill=OWN, own_hash=OWN,
    skip_stubs=["memcpy", "memset"])
