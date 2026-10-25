"""perf.c spec (see lbspecs.py)."""
from lbspecs import add

OWN = ["start_time", "HSD_PerfLastStat", "HSD_PerfCurrentStat"]
add("perf", "src/sysdolphin/baselib/perf.c", dict(
    pub_only=True, ncases=300, c_decl="u32 OS_BusClock;",
    setup=("OS_BusClock = (fv() & 1) ? 162000000u : 4u + fv() % 100000u;",
           "m::__OSBusClock = if fv() & 1 != 0 { 162000000u32 } else { 4u32 + fv() % 100000u32 };"),
    ov={"HSD_PerfCountEnvelopeBlending": {0: ("(int) (fv() % 40)", "(fv() % 40) as i32")}}),
    pub_only=True, own_fill=OWN, own_hash=OWN, skip_stubs=["memset"], skip_data=["OS_BusClock"],
    patch=[("#include <dolphin/os.h>\n", "#include <dolphin/os.h>\n#undef __OSBusClock\nextern u32 OS_BusClock;\n#define __OSBusClock OS_BusClock\n")])
