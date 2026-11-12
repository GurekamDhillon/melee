//! Port of src/melee/lb/lbdvd.c (the DAT preload cache: 80 `PreloadEntry` slots, the scene
//! description they are planned from, and the PC-only six-slot admission budget).
//!
//! Native notes: `PreloadEntry.size`, the `HSD_AllocEntry*` handles and `uintptr_t` arguments are
//! native-width; the GameCube `sizeof(HSD_Archive)` (0x44) is spelled as a constant wherever the C
//! allocates or budgets an archive. `memcmp` of the scene structs is a field-wise compare (the
//! structs carry padding that is never read).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_variables)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;

pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    use core::ffi::c_void;
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? $(= $def:expr)?;)*) => {
            pub mod hook {
                use super::*;
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => crate::tu::hookmac::fallback!(stringify!($name) $(, $def)?),
                }
            } )*
        };
    }
    ext_fns! {
    fn DVDConvertPathToEntrynum(path: *const i8) -> i32;
    fn OSDisableInterrupts() -> i32;
    fn OSRestoreInterrupts(level: i32) -> i32;
    fn OSReport(msg: &str);
    fn Player_80031CB0(char_id: u32, color: u8);
    fn Player_80031D2C(char_id: u32, color: u8);
    fn Stage_802251B4(stkind: u32);
    fn efAsync_OnLoad(archive: *mut HSD_Archive, data: *mut u8, length: u32, effect_index: i32);
    fn gm_801B23F0();
    fn grDatFiles_801C5FC0(archive: *mut HSD_Archive, data: *mut c_void, length: usize);
    fn lbArchive_InitializeDAT(archive: *mut HSD_Archive, data: *mut c_void, length: usize) = crate::tu::lbarchive::lbArchive_InitializeDAT(archive, data, length);
    fn lbFileGetFullName(basename: *const i8) -> *mut i8 = crate::tu::lbfile::lbFileGetFullName(basename as _) as _;
    fn lbFile_800161C4(file: i32, src: usize, dest: usize, size: usize, ty: i32, pri: i32) = crate::tu::lbfile::lbFile_800161C4(file, src, dest, size, ty, pri);
    fn lbFile_8001634C(fileno: i32) -> usize = crate::tu::lbfile::lbFile_8001634C(fileno);
    fn lbFile_800164A4(file: i32, dest: usize, size: *mut usize, pri: i32, callback: Option<unsafe extern "C" fn(i32, usize, *mut c_void, bool)>, args: usize);
    fn lbHeap_800158D0(a0: i32, a1: i32) = crate::tu::lbheap::lbHeap_800158D0(a0, a1);
    fn lbHeap_800158E8(a0: i32) -> i32 = crate::tu::lbheap::lbHeap_800158E8(a0);
    fn lbHeap_80015900() = crate::tu::lbheap::lbHeap_80015900();
    fn lbHeap_80015BB8(heap: i32) -> u32;
    fn lbHeap_80015BD0(heap_id: i32, size: usize) -> *mut HSD_AllocEntry = crate::tu::lbheap::lbHeap_80015BD0(heap_id, size) as _;
    fn lbHeap_80015CA8(heap: i32, ptr: *mut c_void) = crate::tu::lbheap::lbHeap_80015CA8(heap, ptr);
    fn lbHeap_80015D6C(heap0: u32, cb: Option<unsafe extern "C" fn(u32)>, heap1: u32) -> i32 = crate::tu::lbheap::lbHeap_80015D6C(heap0, cb, heap1);
    fn lbHeap_Capacity(heap: i32) -> u32 = crate::tu::lbheap::lbHeap_Capacity(heap);
    fn lbHeap_Free(heap: i32) -> u32 = crate::tu::lbheap::lbHeap_Free(heap);
    fn lb_800195D0();
    }
    // Data defined in other TUs (the tests set them).
    pub use crate::shared::DbLevel;
}

const SIZEOF_HSD_ARCHIVE: usize = 0x44;
const ENTRIES: usize = 80;

/// 3BA638
const preload_cache_scene: PreloadedGameModeState = PreloadedGameModeState {
    is_heap_persistent: [0; 2],
    game_cache: GameCache {
        mode_kind: GM_COUNT as u8,
        x1: 0,
        x2: 0,
        x3: 0,
        stkind: 0x148,
        entries: [PreloadCacheSceneEntry { char_id: ChKind_None as i32, color: 0, x5: 1 }; 8],
    },
    mode_scene_changes: 0,
};

/// 3BA68C
const lbDvd_803BA68C: PreloadEntry = PreloadEntry {
    state: 0,
    r#type: 0,
    heap: 0,
    load_state: 0,
    unknown004: 0xFF,
    field5_0x5: 0,
    entry_num: -1,
    load_score: 0,
    field8_0xa: 0,
    field9_0xb: 0,
    size: 0,
    raw_data: null_mut(),
    archive: null_mut(),
    effect_index: 0,
};

/// 432078
pub static mut preloadCache: PreloadCache = unsafe { core::mem::zeroed() };

static lbDvd_804D37F4: [i32; 2] = [4, 5];

pub static mut six_budget_check: i32 = 0;
pub static mut six_budget_ok: i32 = 0;
pub static mut six_budget_measure: i32 = 0;
pub static mut six_budget_measure_cache: GameCache = unsafe { core::mem::zeroed() };

fn scene_eq(a: &PreloadedGameModeState, b: &PreloadedGameModeState) -> bool {
    a.is_heap_persistent == b.is_heap_persistent && game_cache_eq(&a.game_cache, &b.game_cache) && a.mode_scene_changes == b.mode_scene_changes
}

fn game_cache_eq(a: &GameCache, b: &GameCache) -> bool {
    a.mode_kind == b.mode_kind
        && a.x1 == b.x1
        && a.x2 == b.x2
        && a.x3 == b.x3
        && a.stkind == b.stkind
        && a.entries.iter().zip(b.entries.iter()).all(|(x, y)| x.char_id == y.char_id && x.color == y.color && x.x5 == y.x5)
}

unsafe fn entry_at(i: usize) -> *mut PreloadEntry {
    (&raw mut preloadCache.entries as *mut PreloadEntry).add(i)
}

// ---- TARGET_PC six-slot budget -------------------------------------------------------------

pub unsafe extern "C" fn lbDvd_IsPlanningSixSlot() -> i32 {
    melee_core::ct!("lbDvd_IsPlanningSixSlot");
    six_budget_check
}

/// lbsixbudget.h
fn lbSixBudget_Add(used: &mut u32, capacity: u32, bytes: u32, archive_bytes: u32) -> bool {
    if bytes == 0 || bytes > 0x7FFF_FF00 {
        return false;
    }
    let cost = (bytes.wrapping_add(31) & !31u32).wrapping_add(archive_bytes.wrapping_add(31) & !31u32).wrapping_add(64);
    if *used > capacity || cost > capacity - *used {
        return false;
    }
    *used = used.wrapping_add(cost);
    true
}

unsafe fn lbDvd_SixBudget() -> i32 {
    let mut used = [0u32; 7];
    let mut counts = [0u32; 7];
    let mut files = 0i32;
    let mut ok = six_budget_ok;
    for i in 0..ENTRIES {
        let e = entry_at(i);
        if (*e).state == 0 || (*e).load_score <= 0 {
            continue;
        }
        files += 1;
        let h = (*e).heap as i32;
        if h < 2 || h > 6 || (*e).entry_num < 0 {
            ok = 0;
            continue;
        }
        counts[h as usize] += 1;
        let bytes: u32 = if (*e).size != 0 { (*e).size as u32 } else { ext::lbFile_8001634C((*e).entry_num as i32) as u32 };
        let mut capacity = ext::lbHeap_Capacity(h);
        if h == 4 || h == 5 {
            capacity = if capacity > 0x100000 { capacity - 0x100000 } else { 0 };
        }
        let ty = (*e).r#type;
        if !lbSixBudget_Add(&mut used[h as usize], capacity, bytes, if (2..=4).contains(&ty) { SIZEOF_HSD_ARCHIVE as u32 } else { 0 }) {
            ext::OSReport(&format!(
                "gw: scene: six-slot file rejected entry={} heap={} bytes={} admitted={} budget={}\n",
                (*e).entry_num as i32, h, bytes, used[h as usize], capacity
            ));
            ok = 0;
        }
    }
    {
        let main_free = ext::lbHeap_Free(0);
        ext::OSReport(&format!("gw: scene: six-slot main heap headroom={} admission_floor={}\n", main_free, 0x400000));
        if main_free < 0x400000 {
            ok = 0;
        }
    }
    for h in 3..=5usize {
        // the C passes lbHeap_Capacity(h) twice among OSReport's arguments (evaluated left to right)
        let cap = ext::lbHeap_Capacity(h as i32);
        let cap2 = ext::lbHeap_Capacity(h as i32);
        ext::OSReport(&format!(
            "gw: scene: six-slot planned heap {} requests={} bytes={} capacity={} headroom={} reserve={}\n",
            h,
            counts[h],
            used[h],
            cap,
            (cap2 as i32).wrapping_sub(used[h] as i32),
            if h == 4 || h == 5 { 0x100000 } else { 0 }
        ));
    }
    if files == 0 || (used[4] == 0 && used[5] == 0) {
        ok = 0;
    }
    if ok == 0 {
        ext::OSReport("gw: scene: six-slot preload refused: missing file, cache capacity or heap budget exceeded\n");
    }
    ok
}

pub unsafe extern "C" fn lbDvd_TrySetupSixSlotCache() -> i32 {
    melee_core::ct!("lbDvd_TrySetupSixSlotCache");
    let enabled = ext::OSDisableInterrupts(); // C `bool` is `int` (MSL stdbool.h)
    let saved = preloadCache;
    for i in 0..ENTRIES {
        *entry_at(i) = lbDvd_803BA68C;
    }
    preloadCache.persistent_heaps = 3;
    preloadCache.scene.is_heap_persistent[0] = 1;
    preloadCache.scene.is_heap_persistent[1] = 1;
    preloadCache.new_scene = preload_cache_scene;
    preloadCache.scene.mode_scene_changes += 1;
    six_budget_check = 1;
    six_budget_ok = 1;
    lbDvd_80018254();
    let ok = six_budget_ok;
    preloadCache = saved;
    six_budget_check = 0;
    six_budget_measure = ok;
    if ok != 0 {
        six_budget_measure_cache = saved.scene.game_cache;
    }
    ext::OSRestoreInterrupts(enabled);
    ok
}

pub unsafe extern "C" fn lbDvd_SixBudgetTest() -> i32 {
    melee_core::ct!("lbDvd_SixBudgetTest");
    let saved = preloadCache;
    let saved_ok = six_budget_ok;
    let cap = ext::lbHeap_Capacity(4);
    core::ptr::write_bytes(entry_at(0), 0, ENTRIES);
    (*entry_at(0)).state = 1;
    (*entry_at(0)).load_score = 8;
    (*entry_at(0)).heap = 4;
    (*entry_at(0)).entry_num = 1;
    (*entry_at(0)).size = cap.wrapping_add(1) as usize;
    six_budget_ok = 1;
    let mut rc = (lbDvd_SixBudget() != 0) as i32;
    rc |= ((*entry_at(0)).state != 1 || preloadCache.persistent_heap != saved.persistent_heap) as i32;
    core::ptr::write_bytes(entry_at(0), 0, ENTRIES);
    rc |= (lbDvd_SixBudget() != 0) as i32;
    preloadCache = saved;
    six_budget_ok = saved_ok;
    rc
}

// ---- the cache -----------------------------------------------------------------------------

pub unsafe extern "C" fn lbDvd_SetupVsPreloadCache() {
    melee_core::ct!("lbDvd_SetupVsPreloadCache");
    lbDvd_80018C6C();
    lbDvd_80018254();
    if six_budget_check != 0 && six_budget_ok == 0 {
        return;
    }
    lbDvd_80017700(4);
}

pub unsafe extern "C" fn lbDvd_800174E8(index: i32) {
    melee_core::ct!("lbDvd_800174E8");
    let entry = entry_at(index as usize);
    if !(*entry).archive.is_null() {
        ext::lbHeap_80015CA8((*entry).heap as i32, (*(*entry).archive).addr);
    }
    if !(*entry).raw_data.is_null() {
        ext::lbHeap_80015CA8((*entry).heap as i32, (*(*entry).raw_data).addr);
    }
    *entry = lbDvd_803BA68C;
}

pub unsafe extern "C" fn lbDvd_80017598(heap: i32) -> bool {
    melee_core::ct!("lbDvd_80017598");
    let mut no_change = false;
    let enabled = ext::OSDisableInterrupts();
    if preloadCache.persistent_heap == heap {
        no_change = true;
    } else {
        for i in 0..ENTRIES {
            let entry = entry_at(i);
            if (*entry).state != 0 && (*entry).heap as i32 == heap {
                if (*entry).state == 2 {
                    no_change = true;
                } else {
                    lbDvd_800174E8(i as i32);
                }
            }
        }
    }
    ext::OSRestoreInterrupts(enabled);
    no_change
}

pub unsafe fn lbDvd_80017644(heap: i32) -> bool {
    melee_core::ct!("lbDvd_80017644");
    let mut no_change = false;
    let enabled = ext::OSDisableInterrupts();
    if preloadCache.persistent_heap == heap {
        no_change = true;
    } else {
        for i in 0..ENTRIES {
            let entry = entry_at(i);
            if ((*entry).state == 3 || (*entry).state == 4) && (*entry).heap as i32 == heap && (*entry).load_score < 0 && (*entry).load_state == 2 {
                lbDvd_800174E8(i as i32);
            }
        }
    }
    ext::OSRestoreInterrupts(enabled);
    no_change
}

pub unsafe extern "C" fn lbDvd_80017700(arg0: i32) {
    melee_core::ct!("lbDvd_80017700");
    while lbDvd_80017644(arg0) {
        ext::lb_800195D0();
    }
}

/// The engine's asset service: told every disc entry the preload cache registers (a hint, never sim-visible).
pub static mut ASSET_HINT: Option<fn(i32, i32)> = None;

pub unsafe extern "C" fn lbDvd_80017740(
    ty: i32,
    entry_num: i32,
    transient_heap: i32,
    heap: i32,
    size: u32,
    load_state: i32,
    load_score: i32,
    arg7: u8,
    effect_index: i32,
) -> *mut c_void {
    melee_core::ct!("lbDvd_80017740");
    if let Some(h) = ASSET_HINT {
        h(entry_num, heap);
    }
    let mut free_index: i32 = -1;

    for i in 0..ENTRIES {
        let entry = entry_at(i);
        if (*entry).state == 0 {
            if free_index == -1 {
                free_index = i as i32;
            }
        } else if (*entry).entry_num as i32 == entry_num && (*entry).heap as i32 == transient_heap {
            if (*entry).state == 1 {
                if (*entry).load_score < 0 {
                    (*entry).load_score = (*entry).load_score.wrapping_mul(-1);
                    if (*entry).load_score <= 0x2314 {
                        (*entry).load_score = (*entry).load_score.wrapping_add(10);
                    }
                }
            } else {
                (*entry).load_score = 0x270F;
            }
            return entry as *mut c_void;
        }
    }

    if free_index == -1 && six_budget_check != 0 {
        six_budget_ok = 0;
        return null_mut();
    }
    assert!(free_index != -1, "lbdvd.c:0x1C1 free_index != -1");
    let entry = entry_at(free_index as usize);
    (*entry).state = 1;
    (*entry).r#type = ty as i8;
    (*entry).entry_num = entry_num as i16;
    if ext::lbHeap_80015BB8(heap) != 0 && six_budget_check == 0 {
        panic!("lbdvd.c:0x1CB {}, {}", heap, entry_num);
    }
    (*entry).heap = heap as i8;
    (*entry).size = size as usize;
    (*entry).archive = null_mut();
    (*entry).raw_data = null_mut();
    (*entry).load_state = load_state as i8;
    (*entry).load_score = load_score as i16;
    (*entry).unknown004 = arg7;
    (*entry).effect_index = effect_index;
    entry as *mut c_void
}

pub unsafe extern "C" fn lbDvd_800178E8(arg0: i32, name: *const i8, arg2: i32, arg3: i32, arg4: i32, arg5: i32, arg6: i32, arg7: u8, arg8: i32) {
    melee_core::ct!("lbDvd_800178E8");
    let entry = ext::DVDConvertPathToEntrynum(ext::lbFileGetFullName(name));
    lbDvd_80017740(arg0, entry, arg2, arg3, arg4 as u32, arg5, arg6, arg7, arg8);
}

pub unsafe extern "C" fn lbDvd_80017960() {
    melee_core::ct!("lbDvd_80017960");
    let game_cache = &raw mut preloadCache.new_scene.game_cache;

    if (*game_cache).mode_kind as u32 != GM_COUNT {
        if (*game_cache).mode_kind as u32 == GM_CAMERA_MODE {
            ext::gm_801B23F0();
        }
    }

    if (*game_cache).stkind != 0x148 {
        ext::Stage_802251B4((*game_cache).stkind);
    }

    for i in 0..8usize {
        let ei = (*game_cache).entries[i];
        if ei.char_id as u32 != ChKind_None {
            ext::Player_80031CB0(ei.char_id as u32, ei.color);
        }
        if ei.char_id as u32 == CKind_Kirby {
            if ei.x5 == 0 {
                for kind in 0..ChKind_Max {
                    ext::Player_80031D2C(kind, ei.color);
                }
            } else {
                for j in 0..8usize {
                    let ej = (*game_cache).entries[j];
                    if ej.char_id as u32 != ChKind_None {
                        if ej.char_id as u32 == CKind_Kirby && ej.x5 == 0 {
                            for kind in 0..ChKind_Max {
                                ext::Player_80031D2C(kind, ei.color);
                            }
                        }
                        ext::Player_80031D2C(ej.char_id as u32, ei.color);
                    }
                }
            }
        }
    }
}

unsafe extern "C" fn lbDvd_80017A80(_unused: u32) {
    preloadCache.persistent_heap = 6;
    lbDvd_80017CC4();
}

unsafe fn lbDvd_CleanupPreloadHeap(heap: i32) -> i32 {
    for i in 0..ENTRIES {
        let entry = entry_at(i);
        if (*entry).state == 3 && (*entry).heap as i32 == heap && (*entry).load_score < 0 {
            if !(*entry).archive.is_null() {
                ext::lbHeap_80015CA8((*entry).heap as i32, (*(*entry).archive).addr);
            }
            if !(*entry).raw_data.is_null() {
                ext::lbHeap_80015CA8((*entry).heap as i32, (*(*entry).raw_data).addr);
            }
            *entry = lbDvd_803BA68C;
        }
    }
    preloadCache.persistent_heap = heap;
    ext::lbHeap_80015D6C(heap as u32, Some(lbDvd_80017A80), heap as u32)
}

pub unsafe extern "C" fn lbDvd_CachePreloadedFile(index: i32) {
    melee_core::ct!("lbDvd_CachePreloadedFile");
    let pe = entry_at(index as usize);

    let heap = lbDvd_CleanupPreloadHeap((*pe).heap as i32);

    if heap == 0 {
        preloadCache.persistent_heap = 6;
    }

    if heap == 0 {
        if (*pe).size == 0 {
            (*pe).size = ext::lbFile_8001634C((*pe).entry_num as i32);
        }
        (*pe).raw_data = ext::lbHeap_80015BD0((*pe).heap as i32, ((*pe).size + 31) & !31usize);
        if (*pe).r#type == 2 || (*pe).r#type == 3 || (*pe).r#type == 4 {
            (*pe).archive = ext::lbHeap_80015BD0((*pe).heap as i32, core::mem::size_of::<HSD_Archive>());
        }
        if (*pe).raw_data.is_null() {
            lbDvd_80017E64(0, index as usize, null_mut(), true);
        } else if (*pe).r#type == 0 {
            lbDvd_80017E64(0, index as usize, null_mut(), false);
        } else {
            (*pe).state = 2;
            (*pe).load_score = 9999;
            ext::lbFile_800164A4(
                (*pe).entry_num as i32,
                (*(*pe).raw_data).addr as usize,
                &raw mut (*pe).size,
                2,
                Some(lbDvd_80017E64),
                index as usize,
            );
        }
    }
}

pub unsafe extern "C" fn lbDvd_80017CC4() {
    melee_core::ct!("lbDvd_80017CC4");
    let mut file_to_cache: i32 = -1;
    let mut file_already_cached: i32 = -1;
    let mut max_load_score: i32 = 0;

    if six_budget_measure != 0
        && preloadCache.persistent_heaps == 3
        && game_cache_eq(&preloadCache.new_scene.game_cache, &six_budget_measure_cache)
    {
        let mut pending = 0;
        let mut loaded = 0;
        for i in 0..ENTRIES {
            let e = entry_at(i);
            if (*e).state != 0 && (*e).load_score > 0 {
                if (*e).state == 1 || (*e).state == 2 {
                    pending = 1;
                }
                if ((*e).heap == 4 || (*e).heap == 5) && (*e).state >= 3 {
                    loaded += 1;
                }
            }
        }
        if pending == 0 && loaded != 0 {
            six_budget_measure = 0;
            for h in 0..=5i32 {
                if h == 0 || h >= 3 {
                    ext::OSReport(&format!(
                        "gw: scene: six-slot after-load heap={} free={} capacity={} fighter_files={}\n",
                        h,
                        ext::lbHeap_Free(h),
                        ext::lbHeap_Capacity(h),
                        loaded
                    ));
                }
            }
        }
    }
    if preloadCache.persistent_heap == 6 {
        for i in 0..ENTRIES {
            let entry = entry_at(i);
            match (*entry).state {
                0 | 3 => {}
                2 => file_already_cached = i as i32,
                1 => {
                    if (*entry).load_score as i32 > max_load_score {
                        max_load_score = (*entry).load_score as i32;
                        file_to_cache = i as i32;
                    }
                }
                _ => {}
            }
        }

        if file_already_cached == -1 && file_to_cache != -1 {
            lbDvd_CachePreloadedFile(file_to_cache);
        }
    }
}

pub unsafe extern "C" fn lbDvd_80017E64(key: i32, index: usize, value: *mut c_void, cancelflag: bool) {
    melee_core::ct!("lbDvd_80017E64");
    let pe = entry_at(index);
    assert!(!cancelflag, "lbdvd.c:827 0");
    (*pe).state = 3;
    lbDvd_80017CC4();
}

pub unsafe extern "C" fn lbDvd_GetPreloadedArchive(entry_num: isize) -> *mut c_void {
    melee_core::ct!("lbDvd_GetPreloadedArchive");
    let interrupt = ext::OSDisableInterrupts();

    let mut i = 0usize;
    while i < ENTRIES {
        let entry = entry_at(i);
        if (*entry).state != 0 && (*entry).load_score > 0 && (*entry).entry_num as isize == entry_num {
            break;
        }
        i += 1;
    }

    if i == 80 {
        ext::OSRestoreInterrupts(interrupt);
        return null_mut();
    }
    let entry = entry_at(i);

    if (*entry).state == 1 {
        (*entry).load_score = 8980;
    }

    ext::OSRestoreInterrupts(interrupt);
    lbDvd_800189EC(entry_num as i32);
    // TARGET_PC: a preloaded archive whose heap was compacted is re-read into the moved buffer.
    if (*entry).load_state == 2
        && ((*entry).r#type == 2 || (*entry).r#type == 3 || (*entry).r#type == 4)
        && !(*entry).archive.is_null()
        && !(*entry).raw_data.is_null()
        && (*((*(*entry).archive).addr as *mut HSD_Archive)).data != ((*(*entry).raw_data).addr as *mut u8).add(0x20)
    {
        let dst = (*(*entry).raw_data).addr as usize;
        ext::OSReport(&format!(
            "lbDvd: preloaded archive for entry {} moved (data {:p}, buffer {:p}): reloading it\n",
            entry_num as i32,
            (*((*(*entry).archive).addr as *mut HSD_Archive)).data,
            (*(*entry).raw_data).addr
        ));
        let length = ext::lbFile_8001634C(entry_num as i32);
        ext::lbFile_800161C4(entry_num as i32, 0, dst, (length + 31) & !31usize, if dst >= 0x8000_0000 { 0x21 } else { 0x23 }, 2);
        (*entry).size = length;
        (*entry).load_state = 1;
    }
    if (*entry).load_state == 1 {
        let ty = (*entry).r#type;
        match ty {
            2 => ext::lbArchive_InitializeDAT((*(*entry).archive).addr as *mut HSD_Archive, (*(*entry).raw_data).addr, (*entry).size),
            3 => ext::efAsync_OnLoad(
                (*(*entry).archive).addr as *mut HSD_Archive,
                (*(*entry).raw_data).addr as *mut u8,
                (*entry).size as u32,
                (*entry).effect_index,
            ),
            4 => ext::grDatFiles_801C5FC0((*(*entry).archive).addr as *mut HSD_Archive, (*(*entry).raw_data).addr, (*entry).size),
            _ => panic!("lbdvd.c:864 0"),
        }
        (*entry).load_state = 2;
    }
    if !(*entry).archive.is_null() {
        return (*(*entry).archive).addr;
    }
    (*(*entry).raw_data).addr
}

unsafe fn inline1_inner(data: &lbDvd_803B72C0_t) {
    let x4 = data.name;
    let x8 = data.effect_index;
    let tmp = data.r#type;
    let temp_r3_2 = ext::DVDConvertPathToEntrynum(ext::lbFileGetFullName(x4));
    lbDvd_80017740(tmp as i32, temp_r3_2, 2, 2, 0, 1, 9, 0x80, x8);
}

unsafe fn inline1() {
    let sp_a0 = lbDvd_803B72C0_t { r#type: 2, name: c"LbRb.dat".as_ptr() as *const i8, effect_index: 0 };
    if preloadCache.new_scene.is_heap_persistent[0] != 0 {
        inline1_inner(&sp_a0);
    }
}

unsafe fn inline2_inner(data: &lbDvd_803B72C0_t) {
    let filename = data.name;
    let effect_index = data.effect_index;
    let ty = data.r#type;
    let entry_num = ext::DVDConvertPathToEntrynum(ext::lbFileGetFullName(filename));
    lbDvd_80017740(ty as i32, entry_num, 3, 3, 0, 1, 8, 0x40, effect_index);
}

unsafe fn inline2() {
    let sp28 = [
        lbDvd_803B72C0_t { r#type: 3, name: c"EfMnData.dat".as_ptr() as *const i8, effect_index: 0x1F },
        lbDvd_803B72C0_t { r#type: 3, name: c"EfCoData.dat".as_ptr() as *const i8, effect_index: 0 },
        lbDvd_803B72C0_t { r#type: 2, name: c"ItCo.".as_ptr() as *const i8, effect_index: 0 },
        lbDvd_803B72C0_t { r#type: 2, name: c"IfAll".as_ptr() as *const i8, effect_index: 0 },
    ];
    if preloadCache.new_scene.is_heap_persistent[1] != 0 {
        for d in sp28.iter() {
            inline2_inner(d);
        }
    }
}

static mut warned: [[u8; 24]; 48] = [[0; 24]; 48];
static mut nwarned: i32 = 0;

unsafe fn cstr_bytes<'a>(p: *const i8) -> &'a [u8] {
    core::ffi::CStr::from_ptr(p).to_bytes()
}

pub unsafe extern "C" fn lbDvd_8001819C(basename: *const i8) -> *mut HSD_Archive {
    melee_core::ct!("lbDvd_8001819C");
    let filename = ext::lbFileGetFullName(basename);
    let archive = lbDvd_GetPreloadedArchive(ext::DVDConvertPathToEntrynum(filename) as isize) as *mut HSD_Archive;
    if ext::DbLevel != DbLKind_Master && preloadCache.preloaded != 0 && archive.is_null() {
        panic!("lbdvd.c:948 [LbDvd] {} is not PRELOADed.", String::from_utf8_lossy(cstr_bytes(filename)));
    }
    if preloadCache.preloaded != 0 && archive.is_null() {
        let name = cstr_bytes(basename);
        let key = &name[..name.len().min(23)];
        let mut seen = false;
        for i in 0..nwarned as usize {
            let w = &warned[i][..23];
            let wl = w.iter().position(|&c| c == 0).unwrap_or(23);
            if &w[..wl] == key {
                seen = true;
                break;
            }
        }
        if !seen {
            let entry = ext::DVDConvertPathToEntrynum(filename);
            if nwarned < 48 {
                let n = nwarned as usize;
                warned[n] = [0; 24];
                warned[n][..key.len()].copy_from_slice(key);
                nwarned += 1;
            }
            ext::OSReport(&format!(
                "lbDvd: {} loaded without a preload ({} bytes)\n",
                String::from_utf8_lossy(cstr_bytes(filename)),
                if entry >= 0 { ext::lbFile_8001634C(entry) as u32 } else { 0 }
            ));
        }
    }
    archive
}

pub unsafe extern "C" fn lbDvd_GetPreloadCacheScene() -> *mut PreloadedGameModeState {
    melee_core::ct!("lbDvd_GetPreloadCacheScene");
    &raw mut preloadCache.scene
}

pub unsafe extern "C" fn lbDvd_8001823C() {
    melee_core::ct!("lbDvd_8001823C");
    (*lbDvd_GetPreloadCacheScene()).mode_scene_changes = preloadCache.new_scene.mode_scene_changes.wrapping_add(1);
}

pub unsafe extern "C" fn lbDvd_80018254() {
    melee_core::ct!("lbDvd_80018254");
    if scene_eq(&preloadCache.new_scene, &preloadCache.scene) {
        return;
    }

    preloadCache.new_scene = preloadCache.scene;
    // inline_preload_entries
    let enabled = ext::OSDisableInterrupts();
    for i in 0..ENTRIES {
        let entry = entry_at(i);
        if (*entry).state != 0 && (*entry).load_score > 0 {
            (*entry).load_score = (*entry).load_score.wrapping_mul(-1);
        }
    }

    match preloadCache.persistent_heaps {
        0 => {}
        1 => inline1(),
        2 => {
            inline1();
            inline2();
        }
        3 => {
            inline1();
            inline2();
            lbDvd_80017960();
        }
        _ => {}
    }

    // inline_cleanup_entries
    for j in 0..ENTRIES {
        let ce = entry_at(j);
        if (*ce).load_score < 0 {
            if (*ce).state == 1 {
                if !(*ce).archive.is_null() {
                    ext::lbHeap_80015CA8((*ce).heap as i32, (*(*ce).archive).addr);
                }
                if !(*ce).raw_data.is_null() {
                    ext::lbHeap_80015CA8((*ce).heap as i32, (*(*ce).raw_data).addr);
                }
                *ce = lbDvd_803BA68C;
            } else if (*ce).state == 4 {
                (*ce).state = 3;
            }
        }
    }
    if six_budget_check != 0 {
        six_budget_ok = lbDvd_SixBudget();
        if six_budget_ok == 0 {
            for i in 0..ENTRIES {
                if (*entry_at(i)).state == 1 {
                    *entry_at(i) = lbDvd_803BA68C;
                }
            }
            preloadCache.new_scene = preload_cache_scene;
            ext::OSRestoreInterrupts(enabled);
            return;
        }
    }
    if six_budget_check == 0 {
        lbDvd_80017CC4();
    }
    ext::OSRestoreInterrupts(enabled);
}

unsafe fn inline3(entry: *mut PreloadEntry) -> (bool, bool) {
    let mut var_r10 = false;
    let mut var_r9 = false;
    for i in 0..ENTRIES {
        let other = entry_at(i);
        if (*other).state == 1 && (*other).heap == (*entry).heap && (*other).load_score > 0 {
            var_r9 = true;
        }
        if ((*other).state == 2 || (*other).state == 3) && (*other).heap == (*entry).heap && (*other).load_score < 0 {
            var_r10 = true;
        }
    }
    (var_r9, var_r10)
}

pub unsafe extern "C" fn lbDvd_800187F4(entry_num: i32) -> i32 {
    melee_core::ct!("lbDvd_800187F4");
    let mut result = 0;
    let enabled = ext::OSDisableInterrupts(); // C `bool` is `int` (MSL stdbool.h)
    let persistent_heap = preloadCache.persistent_heap;

    for i in 0..ENTRIES {
        let entry = entry_at(i);
        if (*entry).state == 0 {
            continue;
        }
        if (*entry).load_score <= 0 {
            continue;
        }
        if (*entry).entry_num as i32 != entry_num {
            continue;
        }
        if (*entry).heap as i32 == persistent_heap {
            result = 1;
            break;
        }
        match (*entry).state {
            1 | 2 => {
                result = 1;
                break;
            }
            3 => {
                let (r9, r10) = inline3(entry);
                if r9 && r10 {
                    result = 1;
                    break;
                }
                (*entry).state = 4;
                (*entry).load_score = 0x270F;
                result = 2;
                break;
            }
            4 => {
                result = 2;
                break;
            }
            _ => continue,
        }
    }

    ext::OSRestoreInterrupts(enabled);
    result
}

unsafe fn lbDvd_800189EC(entry_num: i32) {
    while lbDvd_800187F4(entry_num) == 1 {
        ext::lb_800195D0();
        // TARGET_PC: the alarm interrupt that steps a heap compaction (lbmemory.rs COPY_ALARM) fires during the wait
        crate::tu::lbmemory::lbMemory_FireCopyAlarm();
    }
}

pub unsafe extern "C" fn lbDvd_80018A2C(arg0: u8) -> i32 {
    melee_core::ct!("lbDvd_80018A2C");
    let mut result = 0;
    let enabled = ext::OSDisableInterrupts(); // C `bool` is `int` (MSL stdbool.h)

    for i in 0..0x50usize {
        let entry = entry_at(i);
        if (*entry).state == 0 {
            continue;
        }
        if (*entry).load_score <= 0 {
            continue;
        }
        if ((*entry).unknown004 & arg0) == 0 {
            continue;
        }
        if (*entry).heap as i32 == preloadCache.persistent_heap {
            result = 1;
            break;
        }
        match (*entry).state {
            1 | 2 => {
                result = 1;
                break;
            }
            3 => {
                let mut has_stale_entry = false;
                let mut has_pending_entry = false;
                for j in 0..ENTRIES {
                    let other = entry_at(j);
                    if (*other).state == 1 && (*other).heap == (*entry).heap && (*other).load_score > 0 {
                        has_pending_entry = true;
                    }
                    if ((*other).state == 2 || (*other).state == 3) && (*other).heap == (*entry).heap && (*other).load_score < 0 {
                        has_stale_entry = true;
                    }
                }
                if has_pending_entry && has_stale_entry {
                    result = 1;
                    break;
                }
                (*entry).state = 4;
                (*entry).load_score = 0x270F;
                result = 2;
                continue;
            }
            4 => {
                result = 2;
                continue;
            }
            _ => continue,
        }
    }

    ext::OSRestoreInterrupts(enabled);
    result
}

pub unsafe extern "C" fn lbDvd_80018C2C(arg0: u8) {
    melee_core::ct!("lbDvd_80018C2C");
    while lbDvd_80018A2C(arg0) == 1 {
        ext::lb_800195D0();
    }
}

pub unsafe extern "C" fn lbDvd_80018C6C() {
    melee_core::ct!("lbDvd_80018C6C");
    match preloadCache.persistent_heaps {
        1 => {
            (*lbDvd_GetPreloadCacheScene()).is_heap_persistent[0] = 1;
        }
        2 => {
            (*lbDvd_GetPreloadCacheScene()).is_heap_persistent[0] = 1;
            (*lbDvd_GetPreloadCacheScene()).is_heap_persistent[1] = 1;
        }
        3 => {
            (*lbDvd_GetPreloadCacheScene()).is_heap_persistent[0] = 1;
            (*lbDvd_GetPreloadCacheScene()).is_heap_persistent[1] = 1;
            (*lbDvd_GetPreloadCacheScene()).game_cache = preload_cache_scene.game_cache;
        }
        _ => {}
    }
}

unsafe fn inline0() {
    for i in 0..2usize {
        if ext::lbHeap_800158E8(lbDvd_804D37F4[i]) == 1 {
            let tmp = lbDvd_804D37F4[i];
            while lbDvd_80017598(tmp) {
                ext::lb_800195D0();
            }
        }
    }
}

pub unsafe extern "C" fn lbDvd_80018CF4(arg0: i32) {
    melee_core::ct!("lbDvd_80018CF4");
    if preloadCache.persistent_heaps != arg0 {
        (*lbDvd_GetPreloadCacheScene()).mode_scene_changes = preloadCache.new_scene.mode_scene_changes.wrapping_add(1);
    }
    preloadCache.persistent_heaps = arg0;
    ext::lbHeap_800158D0(2, 1);
    ext::lbHeap_800158D0(3, 1);

    for i in 0..2usize {
        ext::lbHeap_800158D0(lbDvd_804D37F4[i], 1);
    }
    match preloadCache.persistent_heaps {
        0 => {
            (*lbDvd_GetPreloadCacheScene()).is_heap_persistent[0] = preload_cache_scene.is_heap_persistent[0];
            (*lbDvd_GetPreloadCacheScene()).is_heap_persistent[1] = preload_cache_scene.is_heap_persistent[1];
            (*lbDvd_GetPreloadCacheScene()).game_cache = preload_cache_scene.game_cache;
        }
        1 => {
            ext::lbHeap_800158D0(2, 0);
            (*lbDvd_GetPreloadCacheScene()).is_heap_persistent[1] = preload_cache_scene.is_heap_persistent[1];
            (*lbDvd_GetPreloadCacheScene()).game_cache = preload_cache_scene.game_cache;
        }
        2 => {
            ext::lbHeap_800158D0(2, 0);
            ext::lbHeap_800158D0(3, 0);
            (*lbDvd_GetPreloadCacheScene()).game_cache = preload_cache_scene.game_cache;
        }
        3 => {
            ext::lbHeap_800158D0(2, 0);
            ext::lbHeap_800158D0(3, 0);
            for i in 0..2usize {
                ext::lbHeap_800158D0(lbDvd_804D37F4[i], 0);
            }
        }
        _ => {}
    }
    if ext::lbHeap_800158E8(2) == 1 {
        while lbDvd_80017598(2) {
            ext::lb_800195D0();
        }
    }
    if ext::lbHeap_800158E8(3) == 1 {
        while lbDvd_80017598(3) {
            ext::lb_800195D0();
        }
    }
    inline0();
    ext::lbHeap_80015900();
}

pub unsafe extern "C" fn lbDvd_80018F58(value: bool) {
    melee_core::ct!("lbDvd_80018F58");
    preloadCache.preloaded = value as i32;
}

pub unsafe extern "C" fn lbDvd_80018F68() {
    melee_core::ct!("lbDvd_80018F68");
    preloadCache.persistent_heaps = 0;
    *lbDvd_GetPreloadCacheScene() = preload_cache_scene;
    preloadCache.new_scene = preload_cache_scene;
    for i in 0..ENTRIES {
        *entry_at(i) = lbDvd_803BA68C;
    }
    preloadCache.persistent_heap = 6;
    preloadCache.preloaded = 0;
}
