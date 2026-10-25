"""debug.c spec: the report / panic callbacks (the log hook goes through the MSL file table and is not driven)."""
from lbspecs import add

POOLS = [('OSContext', 'P0', 2)]
C_DECL = r'''
static void cb_rep(const u8* b, size_t n) { T(8301); T(PI(b)); T((u32) n); }
static void cb_pan(OSContext* c) { T(8302); T(PI(c)); }
'''
RS_EXTRA = r'''
unsafe extern "C" fn cb_rep(b: *const u8, n: usize) { t(8301); t(pi(b)); t(n as u32); }
unsafe extern "C" fn cb_pan(c: *mut OSContext) { t(8302); t(pi(c)); }
'''
add("debug", "src/sysdolphin/baselib/debug.c", dict(
    pub_only=True, pools=POOLS, ncases=300, c_decl=C_DECL, rs_extra=RS_EXTRA,
    only=["HSD_SetReportCallback"],
    ov={"HSD_SetReportCallback": {0: ("(fv() & 1) ? cb_rep : 0", "if fv() & 1 != 0 { Some(cb_rep as unsafe extern \"C\" fn(*const u8, usize)) } else { None }")},
        "HSD_SetPanicCallback": {0: ("(fv() & 1) ? cb_pan : 0", "if fv() & 1 != 0 { Some(cb_pan) } else { None }")}},
    pregs=("freg((const void*) cb_rep); freg((const void*) cb_pan);", "freg(cb_rep as usize); freg(cb_pan as usize);")),
    # debug.c defines __assert / HSD_Panic, which the driver's common header also defines: rename the TU's
    patch=[('#include "debug.h"\n', '#include "debug.h"\nvoid HSD_Panic_dbg(const char*, u32, const char*);\n')],
    sub=[(r"\b__assert\b", "__assert_dbg"), (r"\bHSD_Panic\b", "HSD_Panic_dbg")],
    pub_only=True, types=[p[0] for p in POOLS], pools=[p[0] for p in POOLS], own_fill=[], own_hash=["reportCallback"],
    own_only=["HSD_SetReportCallback"], skip_stubs=["memcpy", "memset"])
