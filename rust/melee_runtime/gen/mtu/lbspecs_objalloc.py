"""objalloc.c spec (see lbspecs.py)."""
from lbspecs import add

POOLS = [("HSD_ObjAllocData", "OD", 4)]
OWN = ["alloc_datas"]
_SETUP_C = (
    "{ int q, k, i; alloc_datas = (fv() & 1) ? &OD[fv() % 4] : 0; obj_heap.top = 0; obj_heap.curr = 0; obj_heap.size = (u32) -1; obj_heap.remain = fv(); "
    "for (i = 0; i < 16384; i++) POOLMEM[i] = (unsigned char) (i * 31 + 7); for (i = 0; i < 256; i++) LST[i >> 6][i & 63] = (unsigned char) (i * 13 + 1); "
    "for (q = 0; q < 4; q++) { OD[q].size = 8 + 8 * (fv() % 4); OD[q].align = 7; OD[q].next = (q < 3 && (fv() % 4)) ? &OD[q + 1] : 0; "
    "k = fv() % 3; for (i = 0; i < k; i++) *(void**) (LST[q] + 32 * i) = (i + 1 < k) ? (void*) (LST[q] + 32 * (i + 1)) : 0; "
    "OD[q].freehead = k ? (HSD_ObjAllocLink*) LST[q] : 0; OD[q].free = k; OD[q].used = fv() % 5; OD[q].peak = OD[q].used + fv() % 3; "
    "OD[q].num_limit = fv() % 8; OD[q].heap_limit_size = fv() % 1000; OD[q].heap_limit_num = (fv() & 1) ? (u32) -1 : fv() % 8; } }")
_SETUP_R = (
    "{ m::alloc_datas = if fv() & 1 != 0 { &raw mut OD[(fv() % 4) as usize] } else { null_mut() }; m::obj_heap.top = 0; m::obj_heap.curr = 0; m::obj_heap.size = u32::MAX as _; m::obj_heap.remain = fv() as _; "
    "for i in 0..16384usize { POOLMEM[i] = (i as u32).wrapping_mul(31).wrapping_add(7) as u8; } for i in 0..256usize { LST[i >> 6][i & 63] = (i as u32).wrapping_mul(13).wrapping_add(1) as u8; } "
    "for q in 0..4usize { OD[q].size = 8 + 8 * (fv() % 4); OD[q].align = 7; OD[q].next = if q < 3 && fv() % 4 != 0 { &raw mut OD[q + 1] } else { null_mut() }; "
    "let k = (fv() % 3) as usize; for i in 0..k { *((&raw mut LST[q] as *mut u8).add(32 * i) as *mut *mut c_void) = if i + 1 < k { (&raw mut LST[q] as *mut u8).add(32 * (i + 1)) as *mut c_void } else { null_mut() }; } "
    "OD[q].freehead = if k != 0 { (&raw mut LST[q]) as *mut HSD_ObjAllocLink } else { null_mut() }; OD[q].free = k as u32; OD[q].used = fv() % 5; OD[q].peak = OD[q].used + fv() % 3; "
    "OD[q].num_limit = fv() % 8; OD[q].heap_limit_size = fv() % 1000; OD[q].heap_limit_num = if fv() & 1 != 0 { u32::MAX } else { fv() % 8 }; } }")
_HASH_C = "{ hw(POOLMEM, 16384); hw(LST[0], 256); }"
_HASH_R = "{ hw(&raw const POOLMEM as *const u8, 16384); hw(&raw const LST as *const u8, 256); }"
add("objalloc", "src/sysdolphin/baselib/objalloc.c", dict(
    pub_only=True, pools=POOLS, setup=(_SETUP_C, _SETUP_R), hash=(_HASH_C, _HASH_R), ncases=400, skip=["HSD_ObjSetHeap"],
    c_decl="static unsigned char POOLMEM[16384]; static unsigned char LST[4][64]; static void hw(const unsigned char* b, int n) { int i; for (i = 0; i < n; i += 8) { unsigned long long v = *(const unsigned long long*) (b + i); u32 p = PI((const void*) v); if (p != 0xFFFFFFFFu) HM(p); else { HM((u32) v); HM((u32) (v >> 32)); } } }",
    rs_extra="static mut POOLMEM: [u8; 16384] = [0; 16384]; static mut LST: [[u8; 64]; 4] = [[0; 64]; 4]; unsafe fn hw(b: *const u8, n: usize) { let mut i = 0; while i < n { let v = (b.add(i) as *const u64).read_unaligned(); let p = pi(v as usize as *const u8); if p != 0xFFFF_FFFF { hm(p); } else { hm(v as u32); hm((v >> 32) as u32); } i += 8; } }",
    pregs=("preg(POOLMEM, sizeof POOLMEM); preg(LST, sizeof LST);",
           "preg(&raw const POOLMEM, 16384); preg(&raw const LST, 256); melee_runtime::tu::lbext::H_OSReport = Some(|_f, _a| ());"),
    ov={"HSD_ObjAllocAddFree": {1: ("(u32) (1 + fv() % 8)", "1 + fv() % 8")},
        "HSD_ObjFree": {1: ("(void*) (LST[0] + 8 * (fv() % 32))", "(&raw mut LST as *mut u8).add(8 * (fv() % 32) as usize) as *mut c_void")},
        "HSD_ObjAllocInit": {1: ("(size_t) (1 + fv() % 40)", "(1 + fv() % 40) as usize"), 2: ("(u32) 8", "8u32")}}),
    pub_only=True, types=[p[0] for p in POOLS], pools=[p[0] for p in POOLS], skip_stubs=["memset"],
    ret={"HSD_MemAlloc": ("((fv() & 7) ? (void*) POOLMEM : 0)", "if fv() & 7 != 0 { (&raw mut POOLMEM) as *mut c_void } else { null_mut() }"),
         "Snap_RenderPoolCount": ("(int) (fv() % 3)", "(fv() % 3) as i32"),
         "Snap_RenderPoolAt": ("(void*) &OD[fv() % 4]", "(&raw mut OD[(fv() % 4) as usize]) as *mut c_void")},
    own_fill=[], own_hash=OWN)
