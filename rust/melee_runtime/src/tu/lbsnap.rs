//! Port of src/melee/lb/lbsnap.c: memory-card snapshot slots (directory reads, delete/copy/save through
//! lbcardnew), the 448x204 -> 64x32 RGB565 -> RGB5A3 banner scaler and the save header. Callees of
//! unported TUs go through `ext`. State types and GC layouts come from melee_types. Checked
//! against the C (whole TU, shipping contraction) by tests/lbtu_lbsnap.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables, unused_mut, dead_code)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;

// BEGIN GENERATED EXT (rust/melee_runtime/gen/lbtu/gen_tu.py; the callees of the C TU)
pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code, unreachable_code)]
    use super::*;
    use core::ffi::c_void;
    pub mod hook {
        use super::*;
        pub static mut CARDProbe: Option<fn(i32) -> i32> = None;
        pub static mut DCFlushRange: Option<fn(*mut c_void, u32)> = None;
        pub static mut OSGetTime: Option<fn() -> i64> = None;
        pub static mut OSTicksToCalendarTime: Option<fn(i64, *mut OSCalendarTime)> = None;
        pub static mut ft_GetFtKindMask: Option<fn() -> i32> = None;
        pub static mut gm_GetStKind: Option<fn() -> u16> = None;
        pub static mut hsd_803B51C8: Option<fn(i32, i32, i32, *mut i8, i32) -> i32> = None;
        pub static mut hsd_803B5C2C: Option<fn(i32)> = None;
        pub static mut hsd_803B6BE4: Option<fn(*mut i8, i32, *mut c_void) -> i32> = None;
        pub static mut it_8026C47C: Option<fn(*mut it_8026C47C_arg0_t)> = None;
        pub static mut lbArchive_80016DBC: Option<fn(*const i8, *mut c_void, *const i8, i32) -> *mut HSD_Archive> = None;
        pub static mut lbCardNew_DeleteSnap: Option<fn(i32, *const i8, *mut c_void) -> i32> = None;
        pub static mut lbLang_IsSettingJP: Option<fn() -> i32> = None;
        pub static mut lb_8001BB48: Option<fn(i32, *mut i8, *mut c_void, *mut c_void, *mut i8, *mut c_void, *mut c_void, *mut c_void) -> i32> = None;
        pub static mut lb_8001BF04: Option<fn(i32, *mut i8, *mut c_void, *mut i8, *mut c_void, *mut c_void, *mut c_void) -> i32> = None;
        pub static mut lb_8001BFD8: Option<fn(i32, *mut lbCardNew_SnapshotEntry, *mut i32, *mut i32) -> i32> = None;
        pub static mut lb_8001C0F4: Option<fn(i32, *const i8, *const i8, *const i8, *mut c_void) -> i32> = None;
        pub static mut lb_8001C4A8: Option<fn(*mut c_void, *mut c_void) -> i32> = None;
    }
    pub fn CARDProbe(a0: i32) -> i32 {
        match unsafe { hook::CARDProbe } { Some(f) => f(a0), None => { panic!("TODO: CARDProbe is not ported") } }
    }
    pub fn DCFlushRange(a0: *mut c_void, a1: u32) {
        match unsafe { hook::DCFlushRange } { Some(f) => f(a0, a1), None => { panic!("TODO: DCFlushRange is not ported") } }
    }
    pub fn OSGetTime() -> i64 {
        match unsafe { hook::OSGetTime } { Some(f) => f(), None => { panic!("TODO: OSGetTime is not ported") } }
    }
    pub fn OSTicksToCalendarTime(a0: i64, a1: *mut OSCalendarTime) {
        match unsafe { hook::OSTicksToCalendarTime } { Some(f) => f(a0, a1), None => { panic!("TODO: OSTicksToCalendarTime is not ported") } }
    }
    pub fn ft_GetFtKindMask() -> i32 {
        match unsafe { hook::ft_GetFtKindMask } { Some(f) => f(), None => { panic!("TODO: ft_GetFtKindMask is not ported") } }
    }
    pub fn gm_GetStKind() -> u16 {
        match unsafe { hook::gm_GetStKind } { Some(f) => f(), None => { panic!("TODO: gm_GetStKind is not ported") } }
    }
    pub fn hsd_803B51C8(a0: i32, a1: i32, a2: i32, a3: *mut i8, a4: i32) -> i32 {
        match unsafe { hook::hsd_803B51C8 } { Some(f) => f(a0, a1, a2, a3, a4), None => { { return unsafe { crate::tu::hsd_3B34::hsd_803B51C8(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _) as _ }; } } }
    }
    pub fn hsd_803B5C2C(a0: i32) {
        match unsafe { hook::hsd_803B5C2C } { Some(f) => f(a0), None => { { unsafe { let _ = crate::tu::hsd_3B34::hsd_803B5C2C(a0 as _); } return; } } }
    }
    pub fn hsd_803B6BE4(a0: *mut i8, a1: i32, a2: *mut c_void) -> i32 {
        match unsafe { hook::hsd_803B6BE4 } { Some(f) => f(a0, a1, a2), None => { { return unsafe { crate::tu::hsd_3B5C::hsd_803B6BE4(a0 as _, a1 as _, a2 as _) as _ }; } } }
    }
    pub fn it_8026C47C(a0: *mut it_8026C47C_arg0_t) {
        match unsafe { hook::it_8026C47C } { Some(f) => f(a0), None => { panic!("TODO: it_8026C47C is not ported") } }
    }
    pub fn lbArchive_80016DBC(a0: *const i8, a1: *mut c_void, a2: *const i8, a3: i32) -> *mut HSD_Archive {
        match unsafe { hook::lbArchive_80016DBC } { Some(f) => f(a0, a1, a2, a3), None => { panic!("TODO: lbArchive_80016DBC is not ported") } }
    }
    pub fn lbCardNew_DeleteSnap(a0: i32, a1: *const i8, a2: *mut c_void) -> i32 {
        match unsafe { hook::lbCardNew_DeleteSnap } { Some(f) => f(a0, a1, a2), None => { { return unsafe { crate::tu::lbcardnew::lbCardNew_DeleteSnap(a0 as _, a1 as _, a2 as _) as _ }; } } }
    }
    pub fn lbLang_IsSettingJP() -> i32 {
        match unsafe { hook::lbLang_IsSettingJP } { Some(f) => f(), None => { { return unsafe { crate::tu::lblanguage::lbLang_IsSettingJP() as _ }; } } }
    }
    pub fn lb_8001BB48(a0: i32, a1: *mut i8, a2: *mut c_void, a3: *mut c_void, a4: *mut i8, a5: *mut c_void, a6: *mut c_void, a7: *mut c_void) -> i32 {
        match unsafe { hook::lb_8001BB48 } { Some(f) => f(a0, a1, a2, a3, a4, a5, a6, a7), None => { { return unsafe { crate::tu::lbcardnew::lb_8001BB48(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _, a5 as _, a6 as _, a7 as _) as _ }; } } }
    }
    pub fn lb_8001BF04(a0: i32, a1: *mut i8, a2: *mut c_void, a3: *mut i8, a4: *mut c_void, a5: *mut c_void, a6: *mut c_void) -> i32 {
        match unsafe { hook::lb_8001BF04 } { Some(f) => f(a0, a1, a2, a3, a4, a5, a6), None => { { return unsafe { crate::tu::lbcardnew::lb_8001BF04(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _, a5 as _, a6 as _) as _ }; } } }
    }
    pub fn lb_8001BFD8(a0: i32, a1: *mut lbCardNew_SnapshotEntry, a2: *mut i32, a3: *mut i32) -> i32 {
        match unsafe { hook::lb_8001BFD8 } { Some(f) => f(a0, a1, a2, a3), None => { { return unsafe { crate::tu::lbcardnew::lb_8001BFD8(a0 as _, a1 as _, a2 as _, a3 as _) as _ }; } } }
    }
    pub fn lb_8001C0F4(a0: i32, a1: *const i8, a2: *const i8, a3: *const i8, a4: *mut c_void) -> i32 {
        match unsafe { hook::lb_8001C0F4 } { Some(f) => f(a0, a1, a2, a3, a4), None => { { return unsafe { crate::tu::lbcardnew::lb_8001C0F4(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _) as _ }; } } }
    }
    pub fn lb_8001C4A8(a0: *mut c_void, a1: *mut c_void) -> i32 {
        match unsafe { hook::lb_8001C4A8 } { Some(f) => f(a0, a1), None => { { return unsafe { crate::tu::lbcardnew::lb_8001C4A8(a0 as _, a1 as _) as _ }; } } }
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
    pub static mut __OSBusClock: u32 = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT

// C-local records are extracted verbatim by the ONE melee_types generator.
pub use melee_types::{LbCardEntry, LbMcSnapMemSnapIconData, Unk803BACC8,
                      Unk80433380, Unk80433380_0, Unk80433380_48};

/// `static struct Unk80433380 lbSnap_80433380;`
pub static mut lbSnap_80433380: Unk80433380 = unsafe { core::mem::zeroed() };

/// `static struct Unk803BACC8 lbSnap_803BACC8` (initialised data).
pub static mut lbSnap_803BACC8: Unk803BACC8 = Unk803BACC8 {
    icon: [2, 0, 1, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    entries: [
        LbCardEntry { file_size: 0, file_flags: 3, data: null_mut() },
        LbCardEntry { file_size: -1, file_flags: 0, data: null_mut() },
    ],
};

/// `OS_TIMER_CLOCK` = `__OSBusClock / 4` (u32).
#[inline]
unsafe fn os_timer_clock() -> u32 {
    ext::__OSBusClock / 4
}

#[inline]
unsafe fn slot(chan: i32) -> *mut Unk80433380_48 {
    lbSnap_80433380.slot.offset(chan as isize)
}

/// `sprintf(dst, "%s", s)` of a byte string (no bounds check, like the C).
#[inline]
unsafe fn put_str(dst: *mut u8, s: &[u8]) {
    for (i, b) in s.iter().enumerate() {
        *dst.add(i) = *b;
    }
    *dst.add(s.len()) = 0;
}

#[inline]
unsafe fn hsd_assert(cond: bool, what: &str) {
    if !cond {
        panic!("HSD_ASSERT {what}");
    }
}

pub unsafe fn lbSnap_8001D2BC() {
    for i in 0..2usize {
        let prev = lbSnap_80433380.card_state[i];
        lbSnap_80433380.card_state[i] = ext::CARDProbe(i as i32);
        if lbSnap_80433380.card_state[i] != prev {
            lbSnap_80433380.state_changed[i] = 1;
        }
    }
}

pub unsafe fn lbSnap_8001D338(arg0: i32) -> i32 {
    lbSnap_80433380.state_changed[arg0 as usize]
}

pub unsafe fn lbSnap_8001D350(chan: i32) -> i32 {
    let ptr = slot(chan);
    if (*ptr).card_result == 0 && lbSnap_80433380.state_changed[chan as usize] != 0 {
        (*ptr).card_result = 8;
    }
    (*ptr).card_result
}

pub unsafe fn lbSnap_8001D394(chan: i32) -> i32 {
    (*slot(chan)).num
}

pub unsafe fn lbSnap_8001D3B0(chan: i32) -> i32 {
    (*slot(chan)).free_blocks
}

pub unsafe fn lbSnap_8001D3CC(chan: i32) -> i32 {
    (*slot(chan)).free_files
}

pub unsafe fn lbSnap_8001D3E8(chan: i32, index: i32) -> i32 {
    (*slot(chan)).entries[index as usize].blocks as i32
}

pub unsafe fn lbSnap_8001D40C(chan: i32) -> i32 {
    let ptr = slot(chan);
    lbSnap_80433380.state_changed[chan as usize] = 0;
    (*ptr).card_result = ext::lb_8001BFD8(
        chan,
        (*ptr).entries.as_mut_ptr(),
        &raw mut (*ptr).free_blocks,
        &raw mut (*ptr).free_files,
    );
    if (*ptr).card_result == 0 {
        let mut i = 0usize;
        while i < 0x7F {
            if (*ptr).entries[i].file_no == -1 {
                break;
            }
            i += 1;
        }
        (*ptr).num = i as i32;
    }
    (*ptr).card_result
}

pub unsafe fn lbSnap_8001D4A4(chan: i32, arg1: *mut i8) {
    let ptr = slot(chan);
    let time2 = ext::OSGetTime();
    // `OSTicksToSeconds(time2)`: s64 / u32 (promoted to s64), stored in an unsigned int
    let mut time = (time2 / os_timer_clock() as i64) as u32;
    let new_var = (*ptr).num;
    let mut i;
    loop {
        i = 0;
        while i < new_var {
            if time == (*ptr).entries[i as usize].time {
                time = time.wrapping_add(1);
                break;
            }
            i += 1;
        }
        if i == (*ptr).num {
            break;
        }
    }
    for i in 0..0x21usize {
        *arg1.add(i) = 0;
    }
    put_str(arg1 as *mut u8, time.to_string().as_bytes());
}

/// C static inline: clear the filename plus its terminating byte.
#[inline]
pub unsafe fn lbSnap_ClearText(text: *mut i8) {
    for i in 0..0x21usize {
        *text.add(i) = 0;
    }
}

#[inline]
pub unsafe fn lbSnap_FormatTime(chan: i32, index: i32, text: *mut i8) {
    hsd_assert(index < (*slot(chan)).num, "index < _p(slot)[chan].num");
    lbSnap_ClearText(text);
    put_str(text as *mut u8, (*slot(chan)).entries[index as usize].time.to_string().as_bytes());
}

pub unsafe fn lbSnap_8001D5FC(chan: i32, index: i32) -> i32 {
    let ptr = slot(chan);
    let mut text = [0i8; 0x21];
    if (*ptr).card_result == 0 && lbSnap_80433380.state_changed[chan as usize] != 0 {
        (*ptr).card_result = 8;
    }
    let mut ret = (*ptr).card_result;
    if ret == 0 {
        lbSnap_FormatTime(chan, index, text.as_mut_ptr());
        (*slot(chan)).card_result = 8;
        ret = ext::lbCardNew_DeleteSnap(chan, text.as_ptr(), null_mut());
    }
    ret
}

pub unsafe fn lbSnap_8001D7B0(chan: i32, index: i32, jndex: i32) -> i32 {
    let ptr = slot(chan);
    let mut text1 = [0i8; 0x21];
    let mut text2 = [0i8; 0x21];
    let mut text3 = [0i8; 0x21];
    if (*ptr).card_result == 0 && lbSnap_80433380.state_changed[chan as usize] != 0 {
        (*ptr).card_result = 8;
    }
    let mut ret = (*ptr).card_result;
    if ret == 0 {
        lbSnap_FormatTime(chan, index, text1.as_mut_ptr());
        lbSnap_FormatTime(chan, jndex, text2.as_mut_ptr());
        lbSnap_8001D4A4(chan, text3.as_mut_ptr());
        (*slot(chan)).card_result = 8;
        ret = ext::lb_8001C0F4(chan, text1.as_ptr(), text2.as_ptr(), text3.as_ptr(), null_mut());
    }
    ret
}

const RGB5A3_MASK_A: u16 = 0x8000;
const RGB5A3_MASK_R: u16 = 0x7C00;
const RGB5A3_MASK_G: u16 = 0x03E0;
const RGB5A3_MASK_B: u16 = 0x001F;

#[inline]
pub fn RGB565_TO_RGB5A3(pixel: u16) -> u16 {
    let mut result = (pixel >> 1) & (RGB5A3_MASK_R | RGB5A3_MASK_G);
    result |= pixel & RGB5A3_MASK_B;
    result | RGB5A3_MASK_A
}

#[inline]
pub unsafe fn lbSnap_GetMemSnapIconData() -> *mut u8 {
    (*lbSnap_80433380.icon_data).ptr
}

/// Scale the 448x204 snapshot region to 64x32, centered in the 96x32 banner.
pub unsafe fn lbSnap_8001DA5C(src: *const u8) {
    let banner = lbSnap_GetMemSnapIconData();
    let mut src_row_accum: i32 = 0;
    for row in 0..32i32 {
        let src_row_base = src_row_accum / 32;
        let dst_row = banner.offset(((row % 4) * 8) as isize);
        let mut dst_tile_row = row / 4;
        dst_tile_row *= 24;
        let mut column: i32 = 0;
        let mut src_column_accum: i32 = 0;
        for _pair in 0..32 {
            let src_row = src_row_base + 138;
            let src_tile_row = (src_row / 4) * 160;
            let mut src_tile = src_column_accum / 64;
            let mut pixel_column = src_tile + 96;
            src_tile = pixel_column / 4;
            src_tile += src_tile_row;
            let src_row_in_tile = src_row % 4;
            let offset_base = (src_tile << 5) + (src_row_in_tile << 3);
            let src_column_in_tile = pixel_column % 4;
            let mut offset = offset_base + (src_column_in_tile << 1);
            pixel_column = column + 16;
            let rgb565 = (src.offset(offset as isize) as *const u16).read_unaligned();
            let mut rgb5a3 = rgb565 & RGB5A3_MASK_B;
            let mut offset_base = pixel_column / 4;
            offset = pixel_column % 4;
            src_column_accum += 448;
            src_tile = src_column_accum / 64;
            pixel_column = src_tile + 96;
            src_tile = pixel_column / 4;
            let src_row_in_tile = src_row % 4;
            let src_column_in_tile = pixel_column % 4;
            rgb5a3 |= ((rgb565 >> 1) & (RGB5A3_MASK_R | RGB5A3_MASK_G)) | RGB5A3_MASK_A;
            offset_base += dst_tile_row;
            src_tile += src_tile_row;
            offset = (offset_base << 5) + (offset << 1);
            (dst_row.offset(offset as isize) as *mut u16).write_unaligned(rgb5a3);
            pixel_column = column + 17;
            offset_base = src_tile << 5;
            offset = src_row_in_tile << 3;
            offset_base += offset;
            offset = offset_base + (src_column_in_tile << 1);
            let rgb565 = (src.offset(offset as isize) as *const u16).read_unaligned();
            offset_base = pixel_column / 4;
            offset_base += dst_tile_row;
            let rgb5a3 = RGB565_TO_RGB5A3(rgb565);
            offset = (offset_base << 5) + ((pixel_column % 4) << 1);
            (dst_row.offset(offset as isize) as *mut u16).write_unaligned(rgb5a3);
            src_column_accum += 448;
            column += 2;
        }
        src_row_accum += 204;
    }
}

pub unsafe fn lbSnap_8001DC0C(image: *mut u8) -> i32 {
    let mut ret = 0;
    let snap = lbSnap_80433380.snap;
    (*snap).x0 = 4;
    (*snap).width = 640;
    (*snap).height = 480;
    (*snap).stkind = ext::gm_GetStKind();
    ext::it_8026C47C(&raw mut (*snap).x14);
    (*snap).x34 = ext::ft_GetFtKindMask();
    (*snap).x8 = 3;
    ext::hsd_803B5C2C((*snap).x8 as i32);
    (*snap).xC = ext::hsd_803B51C8(
        image as isize as i32,
        (*snap).width as i32,
        (*snap).height as i32,
        (*snap).x38.as_mut_ptr(),
        256000,
    );
    if (*snap).xC != 0 {
        ret = 1;
    }
    lbSnap_8001DA5C(image);
    let ticks = ext::OSGetTime();
    let seconds = (ticks / os_timer_clock() as i64) as u32;
    let mut time: OSCalendarTime = core::mem::zeroed();
    ext::OSTicksToCalendarTime((seconds as u64).wrapping_mul(os_timer_clock() as u64) as i64, &raw mut time);
    let fname = lbSnap_80433380.filename.as_mut_ptr();
    for i in 0..64usize {
        *fname.add(i) = 0;
    }
    let text: &str = if ext::lbLang_IsSettingJP() != 0 {
        "大乱闘スマッシュブラザーズＤＸ  写真データ"
    } else {
        "Super Smash Bros. Melee         Snapshot"
    };
    // sprintf(filename, "%s %02d/%02d %02d:%02d:%02d", ...): not bounds checked in the C either
    let s = format!(
        "{} {:02}/{:02} {:02}:{:02}:{:02}",
        text,
        time.mon.wrapping_add(1),
        time.mday,
        time.hour,
        time.min,
        time.sec
    );
    put_str(fname as *mut u8, s.as_bytes());
    ret
}

pub unsafe fn lbSnap_8001DE8C(arg0: *mut c_void) -> i32 {
    let mut ret = 0;
    let snap = lbSnap_80433380.snap;
    if (*snap).x0 == 4 {
        let temp = ext::hsd_803B6BE4((*snap).x38.as_mut_ptr(), (*snap).xC, arg0);
        ext::DCFlushRange(
            arg0,
            ((*snap).width as i32).wrapping_mul((*snap).height as i32).wrapping_mul(2) as u32,
        );
        if temp != 0 {
            ret = 1;
        }
    }
    ret
}

#[inline]
pub unsafe fn lbSnap_GetSaveDataOffset(snap: *mut Unk80433380_0) -> i32 {
    (*snap).xC.wrapping_add(core::mem::offset_of!(Unk80433380_0, x38) as i32)
}

pub unsafe fn lbSnap_8001DF20() -> i32 {
    let snap = lbSnap_80433380.snap;
    lbSnap_803BACC8.entries[0].file_size = lbSnap_GetSaveDataOffset(snap);
    lbSnap_803BACC8.entries[0].data = snap as *mut c_void;
    ext::lb_8001C4A8(
        lbSnap_803BACC8.entries.as_mut_ptr() as *mut c_void,
        &raw mut lbSnap_803BACC8 as *mut c_void,
    )
}

pub unsafe fn lbSnap_8001DF6C(chan: i32) -> i32 {
    let ptr = slot(chan);
    let mut text = [0i8; 0x21];
    if (*ptr).card_result == 0 && lbSnap_80433380.state_changed[chan as usize] != 0 {
        (*ptr).card_result = 8;
    }
    let mut ret = (*ptr).card_result;
    if ret == 0 {
        let desc = &raw mut lbSnap_803BACC8;
        (*slot(chan)).card_result = 8;
        lbSnap_8001D4A4(chan, text.as_mut_ptr());
        (*desc).entries[0].file_size = lbSnap_GetSaveDataOffset(lbSnap_80433380.snap);
        (*desc).entries[0].data = lbSnap_80433380.snap as *mut c_void;
        ret = ext::lb_8001BB48(
            chan,
            text.as_mut_ptr(),
            (*desc).entries.as_mut_ptr() as *mut c_void,
            desc as *mut c_void,
            lbSnap_80433380.filename.as_mut_ptr(),
            (*lbSnap_80433380.icon_data).ptr as *mut c_void,
            (*lbSnap_80433380.icon_data.add(1)).ptr as *mut c_void,
            null_mut(),
        );
    }
    ret
}

pub unsafe fn lbSnap_8001E058(chan: i32, index: i32) -> i32 {
    let ptr = slot(chan);
    let mut text = [0i8; 0x21];
    if (*ptr).card_result == 0 && lbSnap_80433380.state_changed[chan as usize] != 0 {
        (*ptr).card_result = 8;
    }
    let mut ret = (*ptr).card_result;
    if ret == 0 {
        lbSnap_FormatTime(chan, index, text.as_mut_ptr());
        lbSnap_803BACC8.entries[0].data = lbSnap_80433380.snap as *mut c_void;
        ret = ext::lb_8001BF04(
            chan,
            text.as_mut_ptr(),
            lbSnap_803BACC8.entries.as_mut_ptr() as *mut c_void,
            lbSnap_80433380.filename.as_mut_ptr(),
            (*lbSnap_80433380.icon_data).ptr as *mut c_void,
            (*lbSnap_80433380.icon_data.add(1)).ptr as *mut c_void,
            null_mut(),
        );
    }
    ret
}

pub fn lbSnap_8001E204() -> i32 {
    0x3E840
}

pub fn lbSnap_8001E210() -> i32 {
    0x840
}

pub unsafe fn lbSnap_8001E218(snap: *mut c_void, slot_: *mut Unk80433380_48) {
    lbSnap_80433380.snap = snap as *mut Unk80433380_0;
    lbSnap_80433380.slot = slot_;
    (*lbSnap_80433380.slot).card_result = 8;
    (*lbSnap_80433380.slot.add(1)).card_result = 8;
    ext::lbArchive_80016DBC(
        c"LbMcSnap.".as_ptr(),
        &raw mut lbSnap_80433380.icon_data as *mut c_void,
        c"MemSnapIconData".as_ptr(),
        0,
    );
}

pub unsafe fn lbSnap_8001E27C() {
    lbSnap_80433380.snap = null_mut();
    lbSnap_80433380.slot = null_mut();
}

pub unsafe fn lbSnap_8001E290() {
    lbSnap_80433380.icon_data = null_mut();
    for chan in 0..2usize {
        lbSnap_80433380.card_state[chan] = ext::CARDProbe(chan as i32);
        lbSnap_80433380.state_changed[chan] = 0;
    }
}
