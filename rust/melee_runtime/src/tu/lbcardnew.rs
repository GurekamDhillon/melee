//! Port of src/melee/lb/lbcardnew.c (the task queue over the memory-card SDK and baselib's card
//! layer: mount, check, open, create, read, write, rename, delete, snapshot listing). Verified
//! against the C (whole TU, shipping contraction) by tests/tu_lbcardnew.rs
//! (`python rust/melee_runtime/gen/mtu/run_tu.py lbcardnew`).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::cabi;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;

// BEGIN GENERATED EXT (rust/melee_runtime/gen/mtu/gen_tu.py; the callees of the C TU)
pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    use core::ffi::c_void;
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)?;)*) => {
            pub mod hook {
                use super::*;
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => panic!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            } )*
        };
    }
    ext_fns! {
    fn CARDCheckAsync(a0: i32, a1: Option<unsafe extern "C" fn(i32, i32)>) -> i32;
    fn CARDClose(a0: *mut CARDFileInfo) -> i32;
    fn CARDDeleteAsync(a0: i32, a1: *mut i8, a2: Option<unsafe extern "C" fn(i32, i32)>) -> i32;
    fn CARDFormatAsync(a0: i32, a1: Option<unsafe extern "C" fn(i32, i32)>) -> i32;
    fn CARDFreeBlocks(a0: i32, a1: *mut i32, a2: *mut i32) -> i32;
    fn CARDGetStatus(a0: i32, a1: i32, a2: *mut CARDStat) -> i32;
    fn CARDMountAsync(a0: i32, a1: *mut c_void, a2: Option<unsafe extern "C" fn(i32, i32)>, a3: Option<unsafe extern "C" fn(i32, i32)>) -> i32;
    fn CARDOpen(a0: i32, a1: *const i8, a2: *mut CARDFileInfo) -> i32;
    fn CARDProbeEx(a0: i32, a1: *mut i32, a2: *mut i32) -> i32;
    fn CARDRenameAsync(a0: i32, a1: *const i8, a2: *const i8, a3: Option<unsafe extern "C" fn(i32, i32)>) -> i32;
    fn CARDUnmount(a0: i32) -> i32;
    fn DVDGetCurrentDiskID() -> *mut DVDDiskID;
    fn HSD_Free(a0: *mut c_void);
    fn HSD_MemAlloc(a0: i32) -> *mut c_void;
    fn OSDisableInterrupts() -> i32;
    fn OSRestoreInterrupts(a0: i32) -> i32;
    fn hsd_803AAA48();
    fn hsd_803AC3E0(a0: *mut CardState, a1: i32, a2: i32, a3: i32, a4: *mut u8);
    fn hsd_803B2374();
    fn hsd_803B24E4(a0: *mut CardState, a1: i32, a2: i32, a3: *mut c_void);
    fn hsd_803B2550(a0: *mut CardState, a1: *const i8, a2: Option<unsafe extern "C" fn(i32, i32)>) -> i32;
    fn hsd_803B2674(a0: *mut CardState) -> i32;
    fn hsd_803B27F4(a0: *mut CardState, a1: *mut c_void, a2: *mut c_void, a3: *mut c_void, a4: Option<unsafe extern "C" fn(i32, i32)>) -> i32;
    fn hsd_803B286C(a0: *mut CardState, a1: *const i8, a2: *const i8, a3: *mut c_void, a4: *mut c_void, a5: Option<unsafe extern "C" fn(i32, i32)>) -> i32;
    fn hsd_803B2928(a0: *mut CardState, a1: *const i8, a2: *mut c_void, a3: *mut c_void, a4: Option<unsafe extern "C" fn(i32, i32)>) -> i32;
    fn hsd_803B29D8(a0: *mut CardState, a1: i32, a2: *mut u8, a3: Option<unsafe extern "C" fn(i32, i32)>) -> i32;
    fn hsd_803B2A4C(a0: *mut CardState, a1: i32, a2: *mut u8, a3: Option<unsafe extern "C" fn(i32, i32)>) -> i32;
    fn hsd_SetCardIconInfo(a0: *mut CardState, a1: *mut CardIconInfo) -> i32;
    fn wait_idle();
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

pub const LbCardResult_Ready: u32 = 0;
pub const LbCardResult_2: u32 = 2;
pub const LbCardResult_NoFile: u32 = 4;
pub const LbCardResult_NullFilename: u32 = 7;
pub const LbCardResult_Malformed: u32 = 9;
pub const LbCardResult_10: u32 = 10;
pub const LbCardResult_Busy: u32 = 11;
pub const LbCardResult_BadSectorSize: u32 = 12;
pub const LbCardResult_Invalid: u32 = 13;
pub const LbCardResult_FatalError: u32 = 14;
pub const LbCardResult_DeviceError: u32 = 15;
pub const LbCardResult_16: u32 = 16;

const SECTOR_SIZE: i32 = 0x2000;
const CARD_FILENAME_MAX: usize = 32;
const TASK_MAX: usize = LbCardNewTaskArray_Max as usize;

pub const resultMask_Ready: i32 = 1 << LbCardResult_Ready;
pub const resultMask_Malformed: i32 = 1 << LbCardResult_Malformed;
pub const resultMask_NullFilename: i32 = 1 << LbCardResult_NullFilename;
pub const resultMask_16: i32 = 1 << LbCardResult_16;

pub const LbCardNewTask_Mount: u32 = 0;
pub const LbCardNewTask_Check: u32 = 1;
pub const LbCardNewTask_Open: u32 = 2;
pub const LbCardNewTask_Unk3: u32 = 3;
pub const LbCardNewTask_Format: u32 = 4;
pub const LbCardNewTask_Delete: u32 = 5;
pub const LbCardNewTask_Rename: u32 = 6;
pub const LbCardNewTask_Create: u32 = 7;
pub const LbCardNewTask_Read: u32 = 8;
pub const LbCardNewTask_Write: u32 = 9;
pub const LbCardNewTask_SetStatus: u32 = 10;
pub const LbCardNewTask_ReadHeader: u32 = 11;
pub const LbCardNewTask_ListSnapshots: u32 = 12;
pub const LbCardNewTask_FindFile: u32 = 13;
pub const LbCardNewTask_None: u32 = 14;

/// `LbCardEntry` (lbcardnew.h): one logical file of the card (size, flags, buffer).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct LbCardEntry {
    pub file_size: i32,
    pub file_flags: i32,
    pub data: *mut c_void,
}
melee_core::gc_layout!(LbCardEntry, "LbCardEntry", size 0xC, align 4, [
    file_size @ 0x0: 4 "int",
    file_flags @ 0x4: 4 "int",
    data @ 0x8: 4 "void*",
]);

pub type LbCardOnFinishedCallback = Option<unsafe extern "C" fn(i32)>;

/// `CardTask` (0x54 bytes on the console).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct CardTask {
    /// +0x00
    pub r#type: u32,
    /// +0x04: bit i permits this task to run after result i
    pub result_mask: i32,
    /// +0x08
    pub file_entries: *mut c_void,
    /// +0x0C
    pub filename_ptr: *mut i8,
    /// +0x10
    pub filename: [i8; CARD_FILENAME_MAX + 1],
    /// +0x31
    pub new_filename: [i8; CARD_FILENAME_MAX + 1],
}
melee_core::gc_layout!(CardTask, "CardTask", size 0x54, align 4, [
    r#type @ 0x0: 4 "CardTaskType",
    result_mask @ 0x4: 4 "int",
    file_entries @ 0x8: 4 "void*",
    filename_ptr @ 0xC: 4 "char*",
    filename @ 0x10: 33 "char[33]",
    new_filename @ 0x31: 33 "char[33]",
]);

/// The file-static state record at 432A68 (`state`).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct LbCardNewState {
    pub work_area: *mut c_void,
    pub lib_area: *mut c_void,
    pub chan: i32,
    pub save_data: *mut c_void,
    pub status: *mut i32,
    pub comment: *mut i8,
    pub banner: *mut c_void,
    pub icons: *mut c_void,
    pub snapshot_entries: *mut lbCardNew_SnapshotEntry,
    pub free_blocks: *mut i32,
    pub free_files: *mut i32,
    pub company: [i8; 2],
    pub x2E: i8,
    pub game_name: [i8; 4],
    pub saved_error: i32,
    pub unk_38: [lb_80432A68_38_t; 9],
    pub mounted: i32,
    pub memsize: i32,
    pub sectorsize: i32,
    pub unused_bytes: i32,
    pub unused_files: i32,
    pub file_info: CARDFileInfo,
    pub card_state: CardState,
    pub on_finished: LbCardOnFinishedCallback,
    pub task_array: [CardTask; TASK_MAX],
    pub tasks_remaining: i32,
}
melee_core::gc_layout!(LbCardNewState, "lbcardnew_state", size 0x8B0, align 4, [
    work_area @ 0x0: 4 "void*",
    lib_area @ 0x4: 4 "void*",
    chan @ 0x8: 4 "int",
    save_data @ 0xC: 4 "void*",
    status @ 0x10: 4 "int*",
    comment @ 0x14: 4 "char*",
    banner @ 0x18: 4 "void*",
    icons @ 0x1C: 4 "void*",
    snapshot_entries @ 0x20: 4 "lbCardNew_SnapshotEntry*",
    free_blocks @ 0x24: 4 "int*",
    free_files @ 0x28: 4 "int*",
    company @ 0x2C: 2 "char[2]",
    x2E @ 0x2E: 1 "char",
    game_name @ 0x2F: 4 "char[4]",
    saved_error @ 0x34: 4 "s32",
    unk_38 @ 0x38: 72 "lb_80432A68_38_t[9]",
    mounted @ 0x80: 4 "bool",
    memsize @ 0x84: 4 "s32",
    sectorsize @ 0x88: 4 "s32",
    unused_bytes @ 0x8C: 4 "s32",
    unused_files @ 0x90: 4 "s32",
    file_info @ 0x94: 20 "CARDFileInfo",
    card_state @ 0xA8: 1124 "CardState",
    on_finished @ 0x50C: 4 "LbCardOnFinishedCallback",
    task_array @ 0x510: 924 "CardTask[11]",
    tasks_remaining @ 0x8AC: 4 "int",
]);

/// 432A68
pub static mut state: LbCardNewState = unsafe { core::mem::zeroed() };

/// libc `strncpy`: copies at most `n` bytes and zero-pads the rest.
unsafe fn strncpy(dst: *mut i8, src: *const i8, n: usize) {
    let mut i = 0;
    while i < n && *src.add(i) != 0 {
        *dst.add(i) = *src.add(i);
        i += 1;
    }
    while i < n {
        *dst.add(i) = 0;
        i += 1;
    }
}

/// libc `strncmp` (bytes compared as unsigned char).
unsafe fn strncmp(a: *const i8, b: *const i8, n: usize) -> i32 {
    for i in 0..n {
        let (x, y) = (*a.add(i) as u8, *b.add(i) as u8);
        if x != y {
            return x as i32 - y as i32;
        }
        if x == 0 {
            return 0;
        }
    }
    0
}

/// libc `strcmp`.
unsafe fn strcmp(a: *const i8, b: *const i8) -> i32 {
    let mut i = 0;
    loop {
        let (x, y) = (*a.add(i) as u8, *b.add(i) as u8);
        if x != y {
            return x as i32 - y as i32;
        }
        if x == 0 {
            return 0;
        }
        i += 1;
    }
}

/// MSL `isdigit`: `__ctype_map[(unsigned char) c] & __digit`; the map flags exactly '0'..='9'.
fn isdigit(c: i32) -> i32 {
    ((c as u8).is_ascii_digit()) as i32
}

/// libc `strtoul(s, NULL, 10)` on the digit strings the card file names hold (saturates at ULONG_MAX).
unsafe fn strtoul10(s: *const i8) -> u32 {
    let mut i = 0;
    while matches!(*s.add(i) as u8, b' ' | b'\t' | b'\n' | 0x0B | 0x0C | b'\r') {
        i += 1;
    }
    let mut neg = false;
    if *s.add(i) as u8 == b'+' || *s.add(i) as u8 == b'-' {
        neg = *s.add(i) as u8 == b'-';
        i += 1;
    }
    let mut v: u64 = 0;
    let mut over = false;
    while (*s.add(i) as u8).is_ascii_digit() {
        v = v * 10 + (*s.add(i) as u8 - b'0') as u64;
        if v > u32::MAX as u64 {
            over = true;
            v = u32::MAX as u64;
        }
        i += 1;
    }
    if over {
        return u32::MAX;
    }
    if neg {
        (v as u32).wrapping_neg()
    } else {
        v as u32
    }
}

/// `convertSdkResult`
pub unsafe extern "C" fn convertSdkResult(sdk_result: i32) -> u32 {
    match sdk_result {
        -1 | -2 | -3 => LbCardResult_DeviceError,
        -4 => LbCardResult_NoFile,
        0 => LbCardResult_Ready,
        -5 | -128 => LbCardResult_FatalError,
        -6 | -13 => LbCardResult_Malformed,
        _ => LbCardResult_Invalid,
    }
}

/// `lb_80019C38_noinline`
pub unsafe extern "C" fn lb_80019C38_noinline() -> *mut CardTask {
    getNewTask()
}

/// 019C38
pub unsafe extern "C" fn getNewTask() -> *mut CardTask {
    let mut i = 0;
    let mut result = &raw mut state.task_array[0];
    while i < TASK_MAX {
        result = &raw mut state.task_array[i];
        if (*result).r#type == LbCardNewTask_None {
            break;
        }
        i += 1;
    }
    assert!(i != TASK_MAX, "lbcardnew.c:340 i != LbCardNewTaskArray_Max");
    result
}

/// `resetTaskArray`
pub unsafe extern "C" fn resetTaskArray() {
    for i in 0..TASK_MAX {
        state.task_array[i].r#type = LbCardNewTask_None;
    }
}

/// 019CB0
pub unsafe extern "C" fn executeNextTask(mut result: u32) -> i32 {
    loop {
        let mut i = 0;
        let mut task: *mut CardTask = null_mut();
        while i < TASK_MAX {
            task = &raw mut state.task_array[i];
            if state.task_array[i].r#type != LbCardNewTask_None {
                break;
            }
            i += 1;
        }
        if i != TASK_MAX {
            if (*task).result_mask & (cabi::shl(1, result) as i32) == 0 {
                resetTaskArray();
            } else {
                match (*task).r#type {
                    LbCardNewTask_Mount => result = taskMount() as u32,
                    LbCardNewTask_Check => result = taskCheck() as u32,
                    LbCardNewTask_Open => result = taskOpen((*task).filename_ptr, (*task).file_entries as *mut LbCardEntry) as u32,
                    LbCardNewTask_Unk3 => result = taskUnk3() as u32,
                    LbCardNewTask_Format => result = taskFormat() as u32,
                    LbCardNewTask_Delete => result = taskDelete((*task).filename.as_mut_ptr()) as u32,
                    LbCardNewTask_Rename => result = taskRename((*task).filename.as_ptr(), (*task).new_filename.as_ptr()) as u32,
                    LbCardNewTask_Create => result = taskCreate((*task).filename.as_ptr()) as u32,
                    LbCardNewTask_Read => result = taskRead((*task).file_entries as *mut LbCardEntry) as u32,
                    LbCardNewTask_Write => result = taskWrite((*task).file_entries as *mut LbCardEntry) as u32,
                    LbCardNewTask_SetStatus => result = taskSetStatus() as u32,
                    LbCardNewTask_ReadHeader => result = taskReadHeader() as u32,
                    LbCardNewTask_ListSnapshots => result = taskListSnapshots() as u32,
                    LbCardNewTask_FindFile => result = taskFindFile((*task).filename.as_ptr()) as u32,
                    _ => {}
                }
                (*task).r#type = LbCardNewTask_None;
                if result != LbCardResult_Busy {
                    continue;
                }
            }
        }
        break;
    }
    if result != LbCardResult_Busy {
        if let Some(cb) = state.on_finished {
            cb(result as i32);
            state.on_finished = None;
        }
    }
    if result != LbCardResult_Busy && state.mounted != 0 {
        ext::CARDUnmount(state.chan);
        state.mounted = 0;
    }
    result as i32
}

/// `resetState`
pub unsafe extern "C" fn resetState(chan: i32, save_data: *mut c_void, status_out: *mut i32, callback: LbCardOnFinishedCallback) {
    state.chan = chan;
    state.save_data = save_data;
    state.status = status_out;
    state.comment = null_mut();
    state.banner = null_mut();
    state.icons = null_mut();
    state.snapshot_entries = null_mut();
    state.free_blocks = null_mut();
    state.free_files = null_mut();
    state.saved_error = LbCardResult_16 as i32;
    for i in 0..9 {
        state.unk_38[i].lb_error = LbCardResult_16 as i32;
        state.unk_38[i].hsd_result = -1;
    }
    state.mounted = 0;
    state.on_finished = callback;
    resetTaskArray();
}

/// `onCardComplete`
pub unsafe extern "C" fn onCardComplete(_chan: i32, result: i32) {
    let error = convertSdkResult(result) as i32;
    if error != 0 {
        state.saved_error = error;
    }
    state.tasks_remaining -= 1;
}

/// `convertHsdResult`
pub unsafe extern "C" fn convertHsdResult(hsd_result: i32) -> i32 {
    (match hsd_result {
        0 | 1 => LbCardResult_Ready,
        -261 | -260 | -259 | -258 => LbCardResult_2,
        -263 | -262 | -257 => 3,
        -264 | -265 | -266 | -267 => LbCardResult_10,
        _ => convertSdkResult(hsd_result),
    }) as i32
}

/// 01A0B0
pub unsafe extern "C" fn fn_8001A0B0(file_idx: i32, hsd_error: i32) {
    let error = convertHsdResult(hsd_error);
    state.unk_38[file_idx as usize].lb_error = error;
    state.unk_38[file_idx as usize].hsd_result = hsd_error;
    if error != 0 {
        state.saved_error = error;
    }
    state.tasks_remaining -= 1;
}

/// 01A184
pub unsafe extern "C" fn taskMount() -> i32 {
    let mut did_disable = false;
    let mut enabled = 0;
    state.tasks_remaining = 0;
    let probe_result = ext::CARDProbeEx(state.chan, &raw mut state.memsize, &raw mut state.sectorsize);
    state.saved_error = convertSdkResult(probe_result) as i32;
    if state.saved_error == 0 {
        if !state.status.is_null() {
            *state.status = 0;
        }
        assert!(!state.work_area.is_null(), "lbcardnew.c:575 work_area");
        enabled = ext::OSDisableInterrupts();
        did_disable = true;
        let mount_result = ext::CARDMountAsync(state.chan, state.work_area, None, Some(onCardComplete));
        state.saved_error = convertSdkResult(mount_result) as i32;
        if mount_result == 0 || mount_result == -6 || mount_result == -0xD {
            state.mounted = 1;
        }
        if state.saved_error == 0 {
            state.tasks_remaining += 1;
        }
    }
    let pending_ops = state.tasks_remaining;
    let saved_error = state.saved_error;
    if did_disable {
        ext::OSRestoreInterrupts(enabled);
    }
    if pending_ops != 0 {
        return LbCardResult_Busy as i32;
    }
    saved_error
}

/// 01A3A4
pub unsafe extern "C" fn taskCheck() -> i32 {
    state.tasks_remaining = 0;
    let enabled = ext::OSDisableInterrupts();
    let check_result = ext::CARDCheckAsync(state.chan, Some(onCardComplete));
    state.saved_error = convertSdkResult(check_result) as i32;
    if state.saved_error == LbCardResult_Ready as i32 {
        state.tasks_remaining += 1;
    }
    let pending_ops = state.tasks_remaining;
    let saved_error = state.saved_error;
    ext::OSRestoreInterrupts(enabled);
    if pending_ops != 0 {
        return LbCardResult_Busy as i32;
    }
    saved_error
}

/// `setTaskFilename`
pub unsafe extern "C" fn setTaskFilename(filename: *const i8, file_entries: *mut c_void) {
    let task = getNewTask();
    (*task).r#type = LbCardNewTask_Open;
    (*task).result_mask = 1;
    if !filename.is_null() {
        (*task).filename_ptr = (*task).filename.as_mut_ptr();
        strncpy((*task).filename_ptr, filename, CARD_FILENAME_MAX);
    } else {
        (*task).filename_ptr = null_mut();
    }
    (*task).file_entries = file_entries;
}

/// `struct SnapshotNode`
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SnapshotNode {
    pub next: *mut SnapshotNode,
    pub time: u32,
    pub file_no: i16,
    pub blocks: u16,
}

/// `setupCardEntries` (static inline in the C)
pub unsafe extern "C" fn setupCardEntries(ctx: *mut CardState, icon: *mut c_void, mut entry: *mut LbCardEntry) {
    ext::hsd_SetCardIconInfo(ctx, icon as *mut CardIconInfo);
    let mut i = 0;
    while (*entry).file_size != -1 {
        if (*entry).file_size != 0 {
            ext::hsd_803AC3E0(ctx, i, (*entry).file_size, (*entry).file_flags, (*entry).data as *mut u8);
        }
        i += 1;
        entry = entry.add(1);
    }
}

/// 01A594
pub unsafe extern "C" fn taskOpen(filename: *const i8, entries: *mut LbCardEntry) -> i32 {
    state.tasks_remaining = 0;
    if state.sectorsize != SECTOR_SIZE {
        state.saved_error = LbCardResult_BadSectorSize as i32;
    } else {
        let free_result = ext::CARDFreeBlocks(state.chan, &raw mut state.unused_bytes, &raw mut state.unused_files);
        state.saved_error = convertSdkResult(free_result) as i32;
        if state.saved_error == 0 {
            if filename.is_null() {
                state.saved_error = LbCardResult_NullFilename as i32;
            } else {
                let open_result = ext::CARDOpen(state.chan, filename, &raw mut state.file_info);
                ext::CARDClose(&raw mut state.file_info);
                assert!(!state.lib_area.is_null(), "lbcardnew.c:712 lib_area");
                ext::hsd_803B24E4(&raw mut state.card_state, state.chan, SECTOR_SIZE, state.lib_area);
                if open_result == LbCardResult_Ready as i32 {
                    let hsd_result = ext::hsd_803B2550(&raw mut state.card_state, filename, Some(fn_8001A0B0));
                    state.saved_error = convertHsdResult(hsd_result);
                    if state.saved_error == LbCardResult_Ready as i32 {
                        state.tasks_remaining += 1;
                    }
                } else if entries.is_null() {
                    state.saved_error = 4;
                } else if state.unused_files == 0 {
                    state.saved_error = 6;
                } else {
                    setupCardEntries(&raw mut state.card_state, state.save_data, entries);
                    if state.unused_bytes < ext::hsd_803B2674(&raw mut state.card_state).wrapping_shl(13) {
                        state.saved_error = 5;
                    } else {
                        state.saved_error = 4;
                    }
                }
            }
        }
    }
    if state.tasks_remaining != 0 {
        return LbCardResult_Busy as i32;
    }
    state.saved_error
}

/// 01A860
pub unsafe extern "C" fn taskUnk3() -> i32 {
    state.tasks_remaining = 0;
    match state.saved_error {
        0 | 2 => state.saved_error = 1,
        _ => {}
    }
    state.saved_error
}

/// 01A8A4
pub unsafe extern "C" fn taskFormat() -> i32 {
    let mut did_disable = 0;
    let mut enabled = 0;
    state.tasks_remaining = 0;
    if state.mounted != 0 {
        enabled = ext::OSDisableInterrupts();
        did_disable = 1;
        let format_result = ext::CARDFormatAsync(state.chan, Some(onCardComplete));
        state.saved_error = convertSdkResult(format_result) as i32;
        if state.saved_error == 0 {
            state.tasks_remaining += 1;
        }
    }
    let pending_ops = state.tasks_remaining;
    let saved_error = state.saved_error;
    if did_disable != 0 {
        ext::OSRestoreInterrupts(enabled);
    }
    if pending_ops != 0 {
        return LbCardResult_Busy as i32;
    }
    saved_error
}

/// 01A9CC
pub unsafe extern "C" fn taskDelete(filename: *mut i8) -> i32 {
    state.tasks_remaining = 0;
    let enabled = ext::OSDisableInterrupts();
    let delete_result = ext::CARDDeleteAsync(state.chan, filename, Some(onCardComplete));
    state.saved_error = convertSdkResult(delete_result) as i32;
    if state.saved_error == 0 {
        state.tasks_remaining += 1;
    }
    let pending_ops = state.tasks_remaining;
    let saved_error = state.saved_error;
    ext::OSRestoreInterrupts(enabled);
    if pending_ops != 0 {
        return LbCardResult_Busy as i32;
    }
    saved_error
}

/// 01AAE4
pub unsafe extern "C" fn taskRename(old_name: *const i8, new_name: *const i8) -> i32 {
    state.tasks_remaining = 0;
    let enabled = ext::OSDisableInterrupts();
    let rename_result = ext::CARDRenameAsync(state.chan, old_name, new_name, Some(onCardComplete));
    state.saved_error = convertSdkResult(rename_result) as i32;
    if state.saved_error == 0 {
        state.tasks_remaining += 1;
    }
    let pending_ops = state.tasks_remaining;
    let saved_error = state.saved_error;
    ext::OSRestoreInterrupts(enabled);
    if pending_ops != 0 {
        return LbCardResult_Busy as i32;
    }
    saved_error
}

/// 01AC04
pub unsafe extern "C" fn taskCreate(filename: *const i8) -> i32 {
    let hsd_result = ext::hsd_803B286C(
        &raw mut state.card_state,
        filename,
        state.comment,
        state.banner,
        state.icons,
        Some(fn_8001A0B0),
    );
    state.saved_error = convertHsdResult(hsd_result);
    if state.saved_error == 0 {
        state.tasks_remaining += 1;
    }
    if state.tasks_remaining != 0 {
        return LbCardResult_Busy as i32;
    }
    state.saved_error
}

/// `readCardFileSize` (volatile read)
pub unsafe extern "C" fn readCardFileSize(file_size: *const i32) -> i32 {
    core::ptr::read_volatile(file_size)
}

unsafe fn taskReadWrite(entries: *mut LbCardEntry, write: bool) -> i32 {
    state.saved_error = LbCardResult_Ready as i32;
    for i in 0..HSD_CARD_MAX_FILES as usize {
        let _cached_size = readCardFileSize(&raw const state.card_state.file_sizes[i]);
        let _cached_data = core::ptr::read_volatile(&raw const state.card_state.file_flags[i]);
        if readCardFileSize(&raw const state.card_state.file_sizes[i]) != 0 {
            let data = (*entries.add(i)).data as *mut u8;
            let hsd_result = if write {
                ext::hsd_803B2A4C(&raw mut state.card_state, i as i32, data, Some(fn_8001A0B0))
            } else {
                ext::hsd_803B29D8(&raw mut state.card_state, i as i32, data, Some(fn_8001A0B0))
            };
            state.unk_38[i].lb_error = convertHsdResult(hsd_result);
            state.unk_38[i].hsd_result = hsd_result;
            let file_error = state.unk_38[i].lb_error;
            if file_error == LbCardResult_Ready as i32 {
                state.tasks_remaining += 1;
            } else {
                state.saved_error = file_error;
            }
        }
    }
    if state.tasks_remaining != 0 {
        return LbCardResult_Busy as i32;
    }
    state.saved_error
}

/// 01ACEC
pub unsafe extern "C" fn taskRead(entries: *mut LbCardEntry) -> i32 {
    taskReadWrite(entries, false)
}

/// 01AE38
pub unsafe extern "C" fn taskWrite(entries: *mut LbCardEntry) -> i32 {
    taskReadWrite(entries, true)
}

/// 01AF84
pub unsafe extern "C" fn taskSetStatus() -> i32 {
    let hsd_result = ext::hsd_803B2928(&raw mut state.card_state, state.comment, state.banner, state.icons, Some(fn_8001A0B0));
    state.saved_error = convertHsdResult(hsd_result);
    if state.saved_error == 0 {
        state.tasks_remaining += 1;
    }
    if state.tasks_remaining != 0 {
        return LbCardResult_Busy as i32;
    }
    state.saved_error
}

/// 01B068
pub unsafe extern "C" fn taskReadHeader() -> i32 {
    let hsd_result = ext::hsd_803B27F4(&raw mut state.card_state, state.comment as *mut c_void, state.banner, state.icons, Some(fn_8001A0B0));
    state.saved_error = convertHsdResult(hsd_result);
    if state.saved_error == 0 {
        state.tasks_remaining += 1;
    }
    if state.tasks_remaining != 0 {
        return LbCardResult_Busy as i32;
    }
    state.saved_error
}

/// 01B14C
pub unsafe extern "C" fn taskListSnapshots() -> i32 {
    const NODES_LEN: usize = 127;
    let mut head: *mut SnapshotNode = null_mut();
    let mut stat: CARDStat = core::mem::zeroed();
    let disk_id = ext::DVDGetCurrentDiskID();
    state.tasks_remaining = 0;
    if !state.free_blocks.is_null() {
        *state.free_blocks = state.unused_bytes / SECTOR_SIZE;
    }
    if !state.free_files.is_null() {
        *state.free_files = state.unused_files;
    }
    let nodes = ext::HSD_MemAlloc((core::mem::size_of::<SnapshotNode>() * NODES_LEN) as i32) as *mut SnapshotNode;
    let mut node = nodes;
    let company = stat.company.as_mut_ptr() as *mut i8;
    let game = stat.gameName.as_mut_ptr() as *mut i8;
    let mut count = 0;
    for file_no in 0..0x7F {
        if ext::CARDGetStatus(state.chan, file_no, &mut stat) == 0
            && strncmp(company, (*disk_id).company.as_ptr() as *const i8, 2) == 0
            && strncmp(game, (*disk_id).gameName.as_ptr() as *const i8, 4) == 0
            && isdigit(stat.fileName[0] as i32) != 0
        {
            (*node).time = strtoul10(stat.fileName.as_ptr() as *const i8);
            (*node).file_no = file_no as i16;
            (*node).blocks = (stat.length >> 13) as u16;
            let mut scan: *mut *mut SnapshotNode = &mut head;
            while !(*scan).is_null() && (**scan).time > (*node).time {
                scan = &raw mut (**scan).next;
            }
            (*node).next = *scan;
            count += 1;
            *scan = node;
            node = node.add(1);
        }
    }
    let mut scan: *mut *mut SnapshotNode = &mut head;
    for file_no in 0..count {
        let e = state.snapshot_entries.add(file_no);
        (*e).time = (**scan).time;
        (*e).file_no = (**scan).file_no;
        (*e).blocks = (**scan).blocks;
        scan = &raw mut (**scan).next;
    }
    for i in count..0x7F {
        (*state.snapshot_entries.add(i)).file_no = -1;
    }
    ext::HSD_Free(nodes as *mut c_void);
    state.saved_error = 0;
    state.saved_error
}

/// 01B614
pub unsafe extern "C" fn taskFindFile(filename: *const i8) -> i32 {
    let mut card_stat: CARDStat = core::mem::zeroed();
    let mut fileno = 0;
    state.tasks_remaining = 0;
    loop {
        if ext::CARDGetStatus(state.chan, fileno, &mut card_stat) == 0
            && strncmp(card_stat.company.as_ptr() as *const i8, state.company.as_ptr(), 2) == 0
            && strncmp(card_stat.gameName.as_ptr() as *const i8, state.game_name.as_ptr(), 4) == 0
            && strcmp(card_stat.fileName.as_ptr() as *const i8, filename) == 0
        {
            state.saved_error = LbCardResult_Ready as i32;
            break;
        } else {
            fileno += 1;
            if fileno >= 0x7F {
                state.saved_error = LbCardResult_Invalid as i32;
                break;
            }
        }
    }
    state.saved_error
}

/// 01B6E0
pub unsafe extern "C" fn lb_8001B6E0(file_idx: i32) -> i32 {
    state.unk_38[file_idx as usize].lb_error
}

/// 01B6F8
pub unsafe extern "C" fn lbCardNew_CompleteNextTask() -> i32 {
    ext::hsd_803AAA48();
    let enabled = ext::OSDisableInterrupts();
    let mut result = if state.tasks_remaining != 0 { LbCardResult_Busy as i32 } else { state.saved_error };
    ext::OSRestoreInterrupts(enabled);
    if result != LbCardResult_Busy as i32 {
        result = executeNextTask(result as u32);
    }
    result
}

/// 01B760
pub unsafe extern "C" fn lbCardNew_CompleteAllTasks(mut result: i32) -> i32 {
    if result == LbCardResult_Busy as i32 {
        loop {
            result = lbCardNew_CompleteNextTask();
            if result != LbCardResult_Busy as i32 {
                break;
            }
            ext::wait_idle();
        }
    }
    result
}

/// `setupTask` (static inline in the C)
pub unsafe extern "C" fn setupTask(r#type: u32, result_mask: i32) -> *mut CardTask {
    let task = getNewTask();
    (*task).r#type = r#type;
    (*task).result_mask = result_mask;
    task
}

/// `lb_8001A4CC_dontinline`
pub unsafe extern "C" fn lb_8001A4CC_dontinline(filename: *const i8, file_entries: *mut c_void) {
    setTaskFilename(filename, file_entries);
}

/// Runs the queued tasks to completion (the `while (... == LbCardResult_Busy) wait_idle()` tails).
unsafe fn finish(mut result: i32) -> i32 {
    if result == LbCardResult_Busy as i32 {
        loop {
            result = lbCardNew_CompleteNextTask();
            if result != LbCardResult_Busy as i32 {
                break;
            }
            ext::wait_idle();
        }
    }
    result
}

/// 01B7E0
pub unsafe extern "C" fn lb_8001B7E0(chan: i32, filename: *mut i8, file_entries: *mut c_void, save_data: *mut c_void, status_out: *mut i32) -> u32 {
    resetState(chan, save_data, status_out, None);
    setupTask(LbCardNewTask_Mount, resultMask_16);
    setupTask(LbCardNewTask_Check, resultMask_Ready | resultMask_Malformed);
    lb_8001A4CC_dontinline(filename, file_entries);
    setupTask(LbCardNewTask_Unk3, -1);
    let result = executeNextTask(LbCardResult_16);
    finish(result) as u32
}

/// 01B8C8
pub unsafe extern "C" fn lb_8001B8C8(chan: i32) -> i32 {
    resetState(chan, null_mut(), null_mut(), None);
    setupTask(LbCardNewTask_Mount, resultMask_16);
    setupTask(LbCardNewTask_Check, resultMask_Ready | resultMask_Malformed);
    setupTask(LbCardNewTask_Format, -1);
    let result = executeNextTask(LbCardResult_16);
    finish(result)
}

/// `lbCardNew_DeleteSnap`
pub unsafe extern "C" fn lbCardNew_DeleteSnap(chan: i32, filename: *const i8, status_out: *mut c_void) -> i32 {
    resetState(chan, null_mut(), status_out as *mut i32, None);
    setupTask(LbCardNewTask_Mount, resultMask_16);
    setupTask(LbCardNewTask_Check, resultMask_Ready | resultMask_Malformed);
    lb_8001A4CC_dontinline(filename, null_mut());
    setupTask(LbCardNewTask_Unk3, -1);
    let result = LbCardResult_16;
    strncpy((*setupTask(LbCardNewTask_Delete, 0xE)).filename.as_mut_ptr(), filename, CARD_FILENAME_MAX);
    executeNextTask(result)
}

/// 01BA44
pub unsafe extern "C" fn lb_8001BA44(chan: i32, filename: *const i8, status_out: *mut c_void) -> i32 {
    resetState(chan, null_mut(), status_out as *mut i32, None);
    setupTask(LbCardNewTask_Mount, 0x10000);
    setupTask(LbCardNewTask_Check, 0x201);
    lb_8001A4CC_dontinline(filename, null_mut());
    setupTask(LbCardNewTask_Unk3, -1);
    strncpy((*setupTask(LbCardNewTask_Delete, 0xE)).filename.as_mut_ptr(), filename, CARD_FILENAME_MAX);
    let result = executeNextTask(0x10);
    finish(result)
}

/// 01BB48
pub unsafe extern "C" fn lb_8001BB48(
    chan: i32,
    filename: *mut i8,
    file_entries: *mut c_void,
    save_data: *mut c_void,
    comment: *mut i8,
    banner: *mut c_void,
    icons: *mut c_void,
    status_out: *mut c_void,
) -> i32 {
    resetState(chan, save_data, status_out as *mut i32, None);
    let mut task = lb_80019C38_noinline();
    (*task).r#type = LbCardNewTask_Mount;
    (*task).result_mask = 0x10000;
    task = lb_80019C38_noinline();
    (*task).r#type = LbCardNewTask_Check;
    (*task).result_mask = 0x201;
    lb_8001A4CC_dontinline(filename, file_entries);
    task = lb_80019C38_noinline();
    (*task).r#type = LbCardNewTask_Unk3;
    (*task).result_mask = -1;
    task = lb_80019C38_noinline();
    (*task).r#type = LbCardNewTask_Create;
    (*task).result_mask = 0x10;
    core::ptr::copy_nonoverlapping(filename as *const u8, (*task).filename.as_mut_ptr() as *mut u8, 0x20);
    state.comment = comment;
    state.banner = banner;
    state.icons = icons;
    executeNextTask(0x10)
}

/// 01BC18
pub unsafe extern "C" fn lb_8001BC18(
    chan: i32,
    filename: *mut i8,
    file_entries: *mut *mut c_void,
    save_data: *mut c_void,
    comment: *mut i8,
    banner: *mut c_void,
    icons: *mut c_void,
    status_out: *mut i32,
) -> i32 {
    resetState(chan, save_data, status_out, None);
    setupTask(LbCardNewTask_Mount, 0x10000);
    let size = 0x20;
    setupTask(LbCardNewTask_Check, 0x201);
    lb_8001A4CC_dontinline(filename, file_entries as *mut c_void);
    setupTask(LbCardNewTask_Unk3, -1);
    core::ptr::copy_nonoverlapping(filename as *const u8, (*setupTask(LbCardNewTask_Create, 0x10)).filename.as_mut_ptr() as *mut u8, size);
    state.comment = comment;
    state.banner = banner;
    state.icons = icons;
    let result = executeNextTask(0x10);
    finish(result)
}

/// 01BD34
pub unsafe extern "C" fn lb_8001BD34(chan: i32, filename: *const i8, file_entries: *mut c_void, status_out: *mut c_void) -> i32 {
    resetState(chan, null_mut(), status_out as *mut i32, None);
    setupTask(LbCardNewTask_Mount, 0x10000);
    setupTask(LbCardNewTask_Check, 0x201);
    lb_8001A4CC_dontinline(filename, null_mut());
    setupTask(LbCardNewTask_Unk3, -1);
    (*setupTask(LbCardNewTask_Read, 3)).file_entries = file_entries;
    let result = executeNextTask(LbCardResult_16);
    finish(result)
}

/// 01BE30
pub unsafe extern "C" fn lb_8001BE30(
    chan: i32,
    filename: *const i8,
    file_entries: *mut c_void,
    comment: *mut i8,
    banner: *mut c_void,
    icons: *mut c_void,
    status_out: *mut c_void,
    callback: *mut c_void,
) -> i32 {
    resetState(chan, null_mut(), status_out as *mut i32, core::mem::transmute::<*mut c_void, LbCardOnFinishedCallback>(callback));
    let mut task = getNewTask();
    (*task).r#type = LbCardNewTask_Mount;
    (*task).result_mask = 0x10000;
    task = getNewTask();
    (*task).r#type = LbCardNewTask_Check;
    (*task).result_mask = 0x201;
    setTaskFilename(filename, null_mut());
    task = getNewTask();
    (*task).r#type = LbCardNewTask_Unk3;
    (*task).result_mask = -1;
    task = getNewTask();
    (*task).r#type = LbCardNewTask_SetStatus;
    (*task).result_mask = 2;
    state.comment = comment;
    state.banner = banner;
    state.icons = icons;
    task = getNewTask();
    (*task).r#type = LbCardNewTask_Write;
    (*task).result_mask = 3;
    (*task).file_entries = file_entries;
    executeNextTask(0x10)
}

/// 01BF04
pub unsafe extern "C" fn lb_8001BF04(
    chan: i32,
    filename: *mut i8,
    file_entries: *mut c_void,
    comment: *mut i8,
    banner: *mut c_void,
    icons: *mut c_void,
    status_out: *mut c_void,
) -> i32 {
    resetState(chan, null_mut(), status_out as *mut i32, None);
    let mut task = lb_80019C38_noinline();
    (*task).r#type = LbCardNewTask_Mount;
    (*task).result_mask = 0x10000;
    task = lb_80019C38_noinline();
    (*task).r#type = LbCardNewTask_Check;
    (*task).result_mask = resultMask_Ready | resultMask_Malformed;
    lb_8001A4CC_dontinline(filename, null_mut());
    task = lb_80019C38_noinline();
    (*task).r#type = LbCardNewTask_Unk3;
    (*task).result_mask = -1;
    task = lb_80019C38_noinline();
    (*task).r#type = LbCardNewTask_ReadHeader;
    (*task).result_mask = 2;
    state.comment = comment;
    state.banner = banner;
    state.icons = icons;
    task = lb_80019C38_noinline();
    (*task).r#type = LbCardNewTask_Read;
    (*task).result_mask = 3;
    (*task).file_entries = file_entries;
    executeNextTask(0x10)
}

/// 01BFD8
pub unsafe extern "C" fn lb_8001BFD8(chan: i32, snapshot_entries: *mut lbCardNew_SnapshotEntry, free_blocks: *mut i32, free_files: *mut i32) -> i32 {
    resetState(chan, null_mut(), null_mut(), None);
    setupTask(LbCardNewTask_Mount, resultMask_16);
    setupTask(LbCardNewTask_Check, resultMask_Ready | resultMask_Malformed);
    lb_8001A4CC_dontinline(core::ptr::null(), null_mut());
    setupTask(LbCardNewTask_Unk3, -1);
    setupTask(LbCardNewTask_ListSnapshots, resultMask_NullFilename);
    state.snapshot_entries = snapshot_entries;
    state.free_blocks = free_blocks;
    state.free_files = free_files;
    let result = executeNextTask(LbCardResult_16);
    finish(result)
}

/// 01C0F4
pub unsafe extern "C" fn lb_8001C0F4(chan: i32, name_a: *const i8, name_b: *const i8, name_c: *const i8, status_out: *mut c_void) -> i32 {
    resetState(chan, null_mut(), status_out as *mut i32, None);
    let mut task = lb_80019C38_noinline();
    (*task).r#type = LbCardNewTask_Mount;
    (*task).result_mask = 0x10000;
    task = lb_80019C38_noinline();
    (*task).r#type = LbCardNewTask_Check;
    (*task).result_mask = 0x201;
    lb_8001A4CC_dontinline(name_a, null_mut());
    task = lb_80019C38_noinline();
    (*task).r#type = LbCardNewTask_Unk3;
    (*task).result_mask = -1;
    task = lb_80019C38_noinline();
    (*task).r#type = LbCardNewTask_Rename;
    (*task).result_mask = 14;
    strncpy((*task).filename.as_mut_ptr(), name_a, CARD_FILENAME_MAX);
    strncpy((*task).new_filename.as_mut_ptr(), name_c, CARD_FILENAME_MAX);
    task = lb_80019C38_noinline();
    (*task).r#type = LbCardNewTask_Open;
    (*task).result_mask = 1;
    if !name_b.is_null() {
        (*task).filename_ptr = (*task).filename.as_mut_ptr();
        strncpy((*task).filename.as_mut_ptr(), name_b, CARD_FILENAME_MAX);
    } else {
        (*task).filename_ptr = null_mut();
    }
    (*task).file_entries = null_mut();
    task = lb_80019C38_noinline();
    (*task).r#type = LbCardNewTask_Unk3;
    (*task).result_mask = -1;
    task = lb_80019C38_noinline();
    (*task).r#type = LbCardNewTask_Rename;
    (*task).result_mask = 14;
    strncpy((*task).filename.as_mut_ptr(), name_b, CARD_FILENAME_MAX);
    strncpy((*task).new_filename.as_mut_ptr(), name_a, CARD_FILENAME_MAX);
    task = lb_80019C38_noinline();
    (*task).r#type = LbCardNewTask_Open;
    (*task).result_mask = 1;
    if !name_c.is_null() {
        (*task).filename_ptr = (*task).filename.as_mut_ptr();
        strncpy((*task).filename.as_mut_ptr(), name_c, CARD_FILENAME_MAX);
    } else {
        (*task).filename_ptr = null_mut();
    }
    (*task).file_entries = null_mut();
    task = lb_80019C38_noinline();
    (*task).r#type = LbCardNewTask_Unk3;
    (*task).result_mask = -1;
    task = lb_80019C38_noinline();
    (*task).r#type = LbCardNewTask_Rename;
    (*task).result_mask = 14;
    strncpy((*task).filename.as_mut_ptr(), name_c, CARD_FILENAME_MAX);
    strncpy((*task).new_filename.as_mut_ptr(), name_b, CARD_FILENAME_MAX);
    executeNextTask(0x10)
}

/// 01C2D8
pub unsafe extern "C" fn lb_8001C2D8(chan: i32, company: *const i8, game_name: *const i8, filename: *const i8) -> i32 {
    resetState(chan, null_mut(), null_mut(), None);
    setupTask(LbCardNewTask_Mount, 0x10000);
    setupTask(LbCardNewTask_Check, 0x201);
    lb_8001A4CC_dontinline(core::ptr::null(), null_mut());
    setupTask(LbCardNewTask_Unk3, -1);
    let task = setupTask(LbCardNewTask_FindFile, 0x80);
    strncpy(state.company.as_mut_ptr(), company, 2);
    strncpy(state.game_name.as_mut_ptr(), game_name, 4);
    strncpy((*task).filename.as_mut_ptr(), filename, CARD_FILENAME_MAX);
    let result = executeNextTask(0x10);
    finish(result)
}

/// `lbCardNew_ProbeEx`
pub unsafe extern "C" fn lbCardNew_ProbeEx(chan: i32) -> i32 {
    let mut memsize: i32 = 0;
    let mut sectorsize: i32 = 0;
    convertSdkResult(ext::CARDProbeEx(chan, &mut memsize, &mut sectorsize)) as i32
}

/// 01C4A8
pub unsafe extern "C" fn lb_8001C4A8(file_entries: *mut c_void, icon_data: *mut c_void) -> i32 {
    let ctx = &raw mut state.card_state;
    let mut entry = file_entries as *mut LbCardEntry;
    ext::hsd_803B24E4(ctx, 0, SECTOR_SIZE, state.lib_area);
    ext::hsd_SetCardIconInfo(ctx, icon_data as *mut CardIconInfo);
    let mut i = 0;
    while (*entry).file_size != -1 {
        if (*entry).file_size != 0 {
            ext::hsd_803AC3E0(ctx, i, (*entry).file_size, (*entry).file_flags, (*entry).data as *mut u8);
        }
        i += 1;
        entry = entry.add(1);
    }
    ext::hsd_803B2674(ctx)
}

/// `lbCardNew_AllocWorkArea`
pub unsafe extern "C" fn lbCardNew_AllocWorkArea() {
    if state.work_area.is_null() {
        state.work_area = ext::HSD_MemAlloc(0xA000);
        state.lib_area = ext::HSD_MemAlloc(0x2000);
    }
}

/// `lbCardNew_ForgetMemory`
pub unsafe extern "C" fn lbCardNew_ForgetMemory() {
    state.lib_area = null_mut();
    state.work_area = null_mut();
}

/// `lbCardNew_Init`
pub unsafe extern "C" fn lbCardNew_Init() {
    ext::hsd_803B2374();
    resetState(0, null_mut(), null_mut(), None);
    state.tasks_remaining = 0;
}
