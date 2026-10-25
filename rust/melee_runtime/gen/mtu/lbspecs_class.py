"""class.c spec (see lbspecs.py): the class tree (CI), instances (CL), the size-class pool over a bump arena."""
from lbspecs import add

POOLS = [("HSD_ClassInfo", "CI", 4), ("HSD_Class", "CL", 3)]
C_DECL = r'''
static unsigned char ARENA[65536]; static unsigned ARENA_P; static int FAIL1; static int PK;
#define PNEXT() (PIECES + 256 * (PK++ & 15))
static unsigned char PIECES[8192];
static void* arena_alloc(int n) { void* p; if (n <= 0 || ARENA_P + ((n + 7) & ~7) > sizeof ARENA) return 0; p = ARENA + ARENA_P; ARENA_P += (n + 7) & ~7; return p; }
static void cb_amn(HSD_ClassInfo* c) { T(8201); T(PI(c)); }
static int cb_init(HSD_Class* c) { T(8202); T(PI(c)); return (int) (fv() % 3) - 1; }
static void cb_rel(HSD_Class* c) { T(8203); T(PI(c)); }
static void cb_dest(HSD_Class* c) { T(8204); T(PI(c)); }
static HSD_Class* cb_alloc(HSD_ClassInfo* c) { T(8205); T(PI(c)); return (fv() % 5) ? (HSD_Class*) arena_alloc(c->head.obj_size) : 0; }
static void ii0(void) { T(8210); CI[0].head.flags |= 1; }
static void ii1(void) { T(8211); CI[1].head.flags |= 1; }
static void ii2(void) { T(8212); CI[2].head.flags |= 1; }
static void ii3(void) { T(8213); CI[3].head.flags |= 1; }
'''
RS_EXTRA = r'''
static mut ARENA: [u8; 65536] = [0; 65536]; static mut ARENA_P: usize = 0; static mut FAIL1: bool = false; static mut PK: usize = 0;
unsafe fn pnext() -> *mut c_void { let p = (&raw mut PIECES as *mut u8).add(256 * (PK & 15)); PK += 1; p as *mut c_void }
static mut PIECES: [u8; 8192] = [0; 8192];
unsafe fn arena_alloc(n: i32) -> *mut c_void { if n <= 0 || ARENA_P + ((n as usize + 7) & !7) > 65536 { return null_mut(); } let p = (&raw mut ARENA as *mut u8).add(ARENA_P); ARENA_P += (n as usize + 7) & !7; p as *mut c_void }
unsafe extern "C" fn cb_amn(c: *mut HSD_ClassInfo) { t(8201); t(pi(c)); }
unsafe extern "C" fn cb_init(c: *mut HSD_Class) -> i32 { t(8202); t(pi(c)); (fv() % 3) as i32 - 1 }
unsafe extern "C" fn cb_rel(c: *mut HSD_Class) { t(8203); t(pi(c)); }
unsafe extern "C" fn cb_dest(c: *mut HSD_Class) { t(8204); t(pi(c)); }
unsafe extern "C" fn cb_alloc(c: *mut HSD_ClassInfo) -> *mut HSD_Class { t(8205); t(pi(c)); if fv() % 5 != 0 { arena_alloc((*c).head.obj_size as i32) as *mut HSD_Class } else { null_mut() } }
unsafe extern "C" fn ii0() { t(8210); CI[0].head.flags |= 1; }
unsafe extern "C" fn ii1() { t(8211); CI[1].head.flags |= 1; }
unsafe extern "C" fn ii2() { t(8212); CI[2].head.flags |= 1; }
unsafe extern "C" fn ii3() { t(8213); CI[3].head.flags |= 1; }
'''
SETUP_C = r'''{ int q; FAIL1 = 0; PK = 0; memset(ARENA, 0, sizeof ARENA); ARENA_P = 0; memset(PIECES, 0, sizeof PIECES);
 for (q = 0; q < 4; q++) { HSD_ClassInfo* c = &CI[q];
  c->head.info_init = q == 0 ? ii0 : q == 1 ? ii1 : q == 2 ? ii2 : ii3; c->head.flags = fv() & 1;
  c->head.library_name = STRS[fv() % 4]; c->head.class_name = STRS[fv() % 4];
  c->head.obj_size = (s16) (8 + 8 * (fv() % 8)); c->head.info_size = (s16) (sizeof(HSD_ClassInfoHead) + 8 * (fv() % 6));
  c->head.parent = (q > 0 && (fv() & 1)) ? &CI[fv() % q] : 0;
  c->head.child = (q < 3 && (fv() & 1)) ? &CI[q + 1 + fv() % (3 - q)] : 0;
  c->head.next = (q < 3 && (fv() & 1)) ? &CI[q + 1 + fv() % (3 - q)] : 0;
  c->head.nb_exist = fv() % 50; c->head.nb_peak = c->head.nb_exist + fv() % 20;
  c->alloc = (q & 1) ? _hsdClassAlloc : cb_alloc; c->init = (fv() & 1) ? cb_init : _hsdClassInit; c->release = cb_rel; c->destroy = (fv() & 1) ? cb_dest : _hsdClassDestroy;
  c->amnesia = (fv() & 1) ? cb_amn : _hsdClassAmnesia; }
 for (q = 0; q < 3; q++) CL[q].class_info = &CI[fv() % 4];
 hsdClass.head.info_init = _hsdClassInfoInit; hsdClass.head.flags = fv() & 1; hsdClass.head.library_name = STRS[fv() % 4]; hsdClass.head.class_name = STRS[fv() % 4];
 hsdClass.head.obj_size = (s16) (8 + 8 * (fv() % 8)); hsdClass.head.info_size = (s16) sizeof(HSD_ClassInfoHead); hsdClass.head.parent = 0;
 hsdClass.head.child = (fv() & 1) ? &CI[fv() % 4] : 0; hsdClass.head.next = 0; hsdClass.head.nb_exist = fv() % 50; hsdClass.head.nb_peak = fv() % 70;
 hsdClass.alloc = _hsdClassAlloc; hsdClass.init = _hsdClassInit; hsdClass.release = _hsdClassRelease; hsdClass.destroy = _hsdClassDestroy; hsdClass.amnesia = _hsdClassAmnesia;
 memory_list = 0; nb_memory_list = 0; current_hash = (fv() & 1) ? (HSD_Hash*) BUF : 0;
 if (fv() & 1) { int w = fv() % 5; while (w--) { GetMemoryEntry(fv() % 100); } w = fv() % 3; while (w--) { hsdFreeMemPiece(PNEXT(), 1 + fv() % 200); } }
 FAIL1 = (op == OP_GetMemoryEntry || op == OP_hsdAllocMemPiece || op == OP__hsdClassAlloc) && ((fv() & 3) == 0);
}'''
SETUP_R = r'''{ FAIL1 = false; PK = 0; ARENA = [0; 65536]; ARENA_P = 0; PIECES = [0; 8192];
 for q in 0..4usize { let c = &raw mut CI[q];
  (*c).head.info_init = Some(match q { 0 => ii0 as unsafe extern "C" fn(), 1 => ii1, 2 => ii2, _ => ii3 }); (*c).head.flags = fv() & 1;
  (*c).head.library_name = (&raw mut STRS[(fv() % 4) as usize]) as *mut i8; (*c).head.class_name = (&raw mut STRS[(fv() % 4) as usize]) as *mut i8;
  (*c).head.obj_size = (8 + 8 * (fv() % 8)) as i16; (*c).head.info_size = (core::mem::size_of::<_HSD_ClassInfoHead>() as u32 + 8 * (fv() % 6)) as i16;
  (*c).head.parent = if q > 0 && fv() & 1 != 0 { &raw mut CI[(fv() % q as u32) as usize] } else { null_mut() };
  (*c).head.child = if q < 3 && fv() & 1 != 0 { &raw mut CI[q + 1 + (fv() % (3 - q) as u32) as usize] } else { null_mut() };
  (*c).head.next = if q < 3 && fv() & 1 != 0 { &raw mut CI[q + 1 + (fv() % (3 - q) as u32) as usize] } else { null_mut() };
  (*c).head.nb_exist = fv() % 50; (*c).head.nb_peak = (*c).head.nb_exist + fv() % 20;
  (*c).alloc = if q & 1 != 0 { Some(m::_hsdClassAlloc) } else { Some(cb_alloc) }; (*c).init = if fv() & 1 != 0 { Some(cb_init) } else { Some(m::_hsdClassInit) }; (*c).release = Some(cb_rel); (*c).destroy = if fv() & 1 != 0 { Some(cb_dest) } else { Some(m::_hsdClassDestroy) };
  (*c).amnesia = if fv() & 1 != 0 { Some(cb_amn) } else { Some(m::_hsdClassAmnesia) }; }
 for q in 0..3usize { CL[q].class_info = &raw mut CI[(fv() % 4) as usize]; }
 m::hsdClass.head.info_init = Some(m::_hsdClassInfoInit); m::hsdClass.head.flags = fv() & 1; m::hsdClass.head.library_name = (&raw mut STRS[(fv() % 4) as usize]) as *mut i8; m::hsdClass.head.class_name = (&raw mut STRS[(fv() % 4) as usize]) as *mut i8;
 m::hsdClass.head.obj_size = (8 + 8 * (fv() % 8)) as i16; m::hsdClass.head.info_size = core::mem::size_of::<_HSD_ClassInfoHead>() as i16; m::hsdClass.head.parent = null_mut();
 m::hsdClass.head.child = if fv() & 1 != 0 { &raw mut CI[(fv() % 4) as usize] } else { null_mut() }; m::hsdClass.head.next = null_mut(); m::hsdClass.head.nb_exist = fv() % 50; m::hsdClass.head.nb_peak = fv() % 70;
 m::hsdClass.alloc = Some(m::_hsdClassAlloc); m::hsdClass.init = Some(m::_hsdClassInit); m::hsdClass.release = Some(m::_hsdClassRelease); m::hsdClass.destroy = Some(m::_hsdClassDestroy); m::hsdClass.amnesia = Some(m::_hsdClassAmnesia);
 m::memory_list = null_mut(); m::nb_memory_list = 0; m::current_hash = if fv() & 1 != 0 { (&raw mut BUF) as *mut HSD_Hash } else { null_mut() };
 if fv() & 1 != 0 { let mut w = fv() % 5; while w > 0 { w -= 1; m::GetMemoryEntry((fv() % 100) as i32); } w = fv() % 3; while w > 0 { w -= 1; m::hsdFreeMemPiece(pnext(), (1 + fv() % 200) as i32); } }
 FAIL1 = (op == OP_GetMemoryEntry || op == OP_hsdAllocMemPiece || op == OP__hsdClassAlloc) && (fv() & 3) == 0;
}'''
HASH_C = r'''{ int i, n; hash_HSD_ClassInfo(&hsdClass); HM(PI(memory_list)); HM((u32) nb_memory_list); HM(PI(current_hash));
 for (i = 0; i < nb_memory_list; i++) { HSD_MemoryEntry* e = memory_list[i]; HM(PI(e)); if (e) { HSD_FreeList* f; HM(e->size); HM(e->nb_alloc); HM(e->nb_free); HM(PI(e->next)); HM(PI(e->free_list));
   for (f = e->free_list, n = 0; f && n < 64; f = f->next, n++) HM(PI(f)); } } }'''
HASH_R = r'''{ g::hash_HSD_ClassInfo(&m::hsdClass); hm(pi(m::memory_list)); hm(m::nb_memory_list as u32); hm(pi(m::current_hash));
 for i in 0..m::nb_memory_list as usize { let e = *m::memory_list.add(i); hm(pi(e)); if !e.is_null() { hm((*e).size); hm((*e).nb_alloc); hm((*e).nb_free); hm(pi((*e).next)); hm(pi((*e).free_list));
   let mut f = (*e).free_list; let mut n = 0; while !f.is_null() && n < 64 { hm(pi(f)); f = (*f).next; n += 1; } } } }'''
add("class", "src/sysdolphin/baselib/class.c", dict(
    pub_only=True, pools=POOLS, ncases=400, c_decl=C_DECL, rs_extra=RS_EXTRA, setup=(SETUP_C, SETUP_R), hash=(HASH_C, HASH_R),
    pregs=("preg(ARENA, sizeof ARENA); preg(PIECES, sizeof PIECES); freg((const void*) cb_amn); freg((const void*) cb_init); freg((const void*) cb_rel); freg((const void*) cb_dest); freg((const void*) cb_alloc); freg((const void*) ii0); freg((const void*) ii1); freg((const void*) ii2); freg((const void*) ii3);",
           "preg(&raw const ARENA, 65536); preg(&raw const PIECES, 8192); freg(cb_amn as usize); freg(cb_init as usize); freg(cb_rel as usize); freg(cb_dest as usize); freg(cb_alloc as usize); freg(ii0 as usize); freg(ii1 as usize); freg(ii2 as usize); freg(ii3 as usize); melee_runtime::tu::lbext::H_OSReport = Some(|_f, _a| ());"),
    ov={"GetMemoryEntry": {0: ("(s32) (((fv() % 32) == 0) ? -1 : (int) (fv() % 140))", "if fv() % 32 == 0 { -1 } else { (fv() % 140) as i32 }")},
        "hsdAllocMemPiece": {0: ("(s32) (1 + fv() % 1500)", "(1 + fv() % 1500) as i32")},
        "hsdFreeMemPiece": {0: ("(void*) ((fv() & 7) ? (void*) PNEXT() : 0)", "if fv() & 7 != 0 { pnext() } else { null_mut() }"),
                            1: ("(s32) (1 + fv() % 200)", "(1 + fv() % 200) as i32")},
        "hsdInitClassInfo": {1: ("picko_HSD_ClassInfo()", "picko_HSD_ClassInfo()"),
                             4: ("(s32) (sizeof(HSD_ClassInfoHead) + 8 * (fv() % 8))", "(core::mem::size_of::<_HSD_ClassInfoHead>() as u32 + 8 * (fv() % 8)) as i32"),
                             5: ("(s32) (8 + 8 * (fv() % 10))", "(8 + 8 * (fv() % 10)) as i32")},
        "OSReport_PrintSpaces": {0: ("(s32) (fv() % 5) - 1", "(fv() % 5) as i32 - 1")},
        "hsdChangeClass": {0: ("(void*) pick_HSD_Class()", "pick_HSD_Class() as *mut c_void"), 1: ("(void*) pick_HSD_ClassInfo()", "pick_HSD_ClassInfo() as *mut c_void")},
        "hsdIsDescendantOf": {0: ("(void*) picko_HSD_ClassInfo()", "picko_HSD_ClassInfo() as *mut c_void"), 1: ("(void*) picko_HSD_ClassInfo()", "picko_HSD_ClassInfo() as *mut c_void")},
        "hsdObjIsDescendantOf": {0: ("(HSD_Obj*) picko_HSD_Class()", "picko_HSD_Class() as *mut HSD_Obj")},
        "class_set_flags": {1: ("(s32) (fv() & 3)", "(fv() & 3) as i32"), 2: ("(s32) (fv() & 3)", "(fv() & 3) as i32")},
        "hsdDumpClassStat": {1: ("(fv() & 1)", "fv() & 1 != 0"), 2: ("(s32) (fv() % 4)", "(fv() % 4) as i32")},
        "DumpClassStat": {1: ("(s32) (fv() % 4)", "(fv() % 4) as i32")}}),
    pub_only=True, types=[p[0] for p in POOLS], pools=[p[0] for p in POOLS], skip_stubs=["memcpy", "memset", "strcmp"],
    skip_data=["hsdClass"], null_ret_ok=["HSD_HashSearch", "HSD_MemAlloc"],
    ret={"HSD_MemAlloc": ("(FAIL1 ? (FAIL1 = 0, (void*) 0) : arena_alloc(a0))", "if FAIL1 { FAIL1 = false; PK = 0; null_mut() } else { arena_alloc(a0) }"),
         "HSD_HashSearch": ("picko_HSD_ClassInfo()", "picko_HSD_ClassInfo() as *mut c_void")})
