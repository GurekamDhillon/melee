"""lbdvd spec (see lbspecs.py)."""
from lbspecs import add

OWN = ["preloadCache", "six_budget_check", "six_budget_ok", "six_budget_measure", "six_budget_measure_cache"]
BOOL = ("(fv() & 1)", "fv() & 1 != 0")
HEAP = ("(int) (fv() % 8)", "(fv() % 8) as i32")
IDX = ("(int) (fv() % 80)", "(fv() % 80) as i32")
add("lbdvd", "src/melee/lb/lbdvd.c", dict(
    pub_only=True,
    setup=("", "m::ext::hook::OSReport = Some(|_| {});"),
    stmt={"lbDvd_80017740": (
        "{ PreloadEntry* p = (PreloadEntry*) {call}; ret = p ? (u32) (p - preloadCache.entries) + 1 : 0; }",
        "{ let p = {call} as *mut PreloadEntry; ret = if p.is_null() { 0 } else { p.offset_from(m::preloadCache.entries.as_ptr()) as u32 + 1 }; }")},
    pools=[("HSD_AllocEntry", "AE", 8)], skip=["lbDvd_8001819C"],
    ov={"lbDvd_800174E8": {0: IDX}, "lbDvd_CachePreloadedFile": {0: IDX}, "lbDvd_80017598": {0: HEAP}, "lbDvd_80017644": {0: HEAP},
        "lbDvd_80017700": {0: HEAP}, "lbDvd_80017E64": {3: BOOL, 1: ("(uintptr_t) (fv() % 80)", "(fv() % 80) as usize")},
        "lbDvd_GetPreloadedArchive": {0: ("(ssize_t) (fv() % 90) - 5", "(fv() % 90) as isize - 5")},
        "lbDvd_80018F58": {0: BOOL},
        "lbDvd_800187F4": {0: ("(int) (fv() % 90) - 5", "(fv() % 90) as i32 - 5")}}),
    pub_only=True, skip_stubs=["memset", "memcmp", "strncmp", "strncpy"], types=["HSD_AllocEntry"], pools=["HSD_AllocEntry"], own_fill=OWN, own_hash=OWN, own_files=["lbdvd.static.h"],
    skip_data=["preload_cache_scene", "lbDvd_803BA68C"])
