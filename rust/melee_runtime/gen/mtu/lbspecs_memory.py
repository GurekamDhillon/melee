"""memory.c spec (see lbspecs.py)."""
from lbspecs import add

OWN = ["hsd_trace", "hsd_allocs", "hsd_frees", "hsd_live", "hsd_big_size", "hsd_big_count", "hist_size", "hist_hits", "caller_pc", "caller_hits"]
add("memory", "src/sysdolphin/baselib/memory.c", dict(
    pub_only=True, ncases=400,
    pregs=("", "melee_runtime::tu::lbext::H_OSReport = Some(|_f, _a| ()); m::return_address_hook = Some(|_d| 0x4242usize as *mut c_void);"),
    ov={"HSD_MemAlloc": {0: ("(ssize_t) (int) fv()", "fv() as i32")}}),
    pub_only=True, own_fill=OWN, own_hash=OWN,
    patch=[('#include "memory.h"\n', '#include "memory.h"\n#define __builtin_return_address(x) ((void*) 0x4242)\n')],
    ret={"OSAllocFromHeap": ("((fv() & 3) ? (void*) BUF : 0)", "if fv() & 3 != 0 { (&raw mut BUF) as *mut c_void } else { null_mut() }")})
