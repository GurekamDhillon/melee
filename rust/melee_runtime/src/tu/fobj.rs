//! `src/sysdolphin/baselib/fobj.c` on the GameCube-layout record `melee_types::HSD_FObj`: allocator
//! pool, Remove / RemoveAll / Alloc / Free / LoadDesc, and the keyframe interpreter (`parseFloat`,
//! `FObjLoadData`, `FObjUpdateAnim`, `HSD_FObjInterpretAnim[All]`, `StopAnim[All]`, `ReqAnim[All]`)
//! over raw pointers. The owned-model twin is `crate::fobj` (oracle `tests/oracle_anim.rs`).
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, clippy::missing_safety_doc)]
pub use crate::fobj::{load_desc_list as HSD_FObjLoadDescList, HsdFObj};
use crate::cabi::f2i16;
use melee_core::fp::fmadds;
use core::ptr::{null_mut, write_bytes};
use melee_types::*;

/// Callees in unported TUs (objalloc.c).
pub mod ext {
    use crate::tu::hookmac::hook;
    use melee_types::*;
    pub static mut HSD_OBJALLOCINIT: Option<fn(*mut HSD_ObjAllocData, u32, u32)> = None;
    #[allow(non_snake_case)]
    pub fn HSD_ObjAllocInit(data: *mut HSD_ObjAllocData, size: u32, align: u32) {
        match unsafe { HSD_OBJALLOCINIT } { Some(f) => f(data, size, align), None => { unsafe { crate::tu::objalloc::HSD_ObjAllocInit(data as _, size as _, align as _); } return; } }
    }
    pub static mut HSD_OBJALLOC: Option<fn(*mut HSD_ObjAllocData) -> *mut core::ffi::c_void> = None;
    #[allow(non_snake_case)]
    pub fn HSD_ObjAlloc(data: *mut HSD_ObjAllocData) -> *mut core::ffi::c_void {
        match unsafe { HSD_OBJALLOC } { Some(f) => f(data), None => { return unsafe { crate::tu::objalloc::HSD_ObjAlloc(data as _) as _ }; } }
    }
    pub static mut HSD_OBJFREE: Option<fn(*mut HSD_ObjAllocData, *mut core::ffi::c_void)> = None;
    #[allow(non_snake_case)]
    pub fn HSD_ObjFree(data: *mut HSD_ObjAllocData, obj: *mut core::ffi::c_void) {
        match unsafe { HSD_OBJFREE } { Some(f) => f(data, obj), None => { unsafe { crate::tu::objalloc::HSD_ObjFree(data as _, obj as _); } return; } }
    }
}

/// `fobj_alloc_data`.
pub static mut fobj_alloc_data: core::mem::MaybeUninit<HSD_ObjAllocData> = core::mem::MaybeUninit::zeroed();

/// `HSD_FObjGetAllocData`.
pub unsafe fn HSD_FObjGetAllocData() -> *mut HSD_ObjAllocData {
    melee_core::ct!("HSD_FObjGetAllocData");
    fobj_alloc_data.as_mut_ptr()
}

/// `HSD_FObjInitAllocData`: pool of `sizeof(HSD_FObj)`, align 4. The native record is 0x40 bytes (8-byte
/// pointers; 0x30 on the GameCube), and `HSD_FObjAlloc` zeroes the whole native record.
pub unsafe fn HSD_FObjInitAllocData() {
    melee_core::ct!("HSD_FObjInitAllocData");
    ext::HSD_ObjAllocInit(HSD_FObjGetAllocData(), core::mem::size_of::<HSD_FObj>() as u32, 4);
}

/// `HSD_FObjSetState`.
pub unsafe fn HSD_FObjSetState(fobj: *mut HSD_FObj, state: u32) -> u32 {
    melee_core::ct!("HSD_FObjSetState");
    if !fobj.is_null() {
        (*fobj).flags = (state & 0xF) as u8 | ((*fobj).flags & 0xF0);
    }
    state
}

/// `HSD_FObjGetState`.
pub unsafe fn HSD_FObjGetState(fobj: *mut HSD_FObj) -> u32 {
    melee_core::ct!("HSD_FObjGetState");
    if fobj.is_null() {
        return 0;
    }
    ((*fobj).flags & 0xF) as u32
}

/// `HSD_FObjRemove`.
pub unsafe fn HSD_FObjRemove(fobj: *mut HSD_FObj) {
    melee_core::ct!("HSD_FObjRemove");
    if fobj.is_null() {
        return;
    }
    HSD_FObjFree(fobj);
}

/// `HSD_FObjRemoveAll`: frees the tail first, as the C recursion does.
pub unsafe fn HSD_FObjRemoveAll(fobj: *mut HSD_FObj) {
    melee_core::ct!("HSD_FObjRemoveAll");
    if fobj.is_null() {
        return;
    }
    HSD_FObjRemoveAll((*fobj).next);
    HSD_FObjRemove(fobj);
}

/// `HSD_FObjAlloc`: pool alloc, zeroed.
pub unsafe fn HSD_FObjAlloc() -> *mut HSD_FObj {
    melee_core::ct!("HSD_FObjAlloc");
    let new = ext::HSD_ObjAlloc(HSD_FObjGetAllocData()) as *mut HSD_FObj;
    assert!(!new.is_null(), "fobj.c:0x2F3 new");
    write_bytes(new, 0, 1);
    new
}

/// `HSD_FObjFree`.
pub unsafe fn HSD_FObjFree(fobj: *mut HSD_FObj) {
    melee_core::ct!("HSD_FObjFree");
    ext::HSD_ObjFree(HSD_FObjGetAllocData(), fobj as *mut core::ffi::c_void);
}

/// `HSD_FObjLoadDesc`: builds the list from a descriptor chain (`next` loaded first, as the C).
pub unsafe fn HSD_FObjLoadDesc(desc: *mut HSD_FObjDesc) -> *mut HSD_FObj {
    melee_core::ct!("HSD_FObjLoadDesc");
    if desc.is_null() {
        return null_mut();
    }
    let fobj = HSD_FObjAlloc();
    (*fobj).next = HSD_FObjLoadDesc((*desc).next);
    (*fobj).startframe = f2i16((*desc).startframe as f64);
    (*fobj).obj_type = (*desc).r#type;
    (*fobj).frac_value = (*desc).frac_value;
    (*fobj).frac_slope = (*desc).frac_slope;
    (*fobj).ad_head = (*desc).ad;
    (*fobj).length = (*desc).length;
    (*fobj).flags = 0;
    fobj
}

const HSD_A_OP_CON: u8 = 1;
const HSD_A_OP_LIN: u8 = 2;
const HSD_A_OP_SPL0: u8 = 3;
const HSD_A_OP_SPL: u8 = 4;
const HSD_A_OP_SLP: u8 = 5;
const HSD_A_OP_KEY: u8 = 6;
const HSD_A_FRAC_FLOAT: u8 = 0 << 5;
const HSD_A_FRAC_S16: u8 = 1 << 5;
const HSD_A_FRAC_U16: u8 = 2 << 5;
const HSD_A_FRAC_S8: u8 = 3 << 5;
const HSD_A_FRAC_U8: u8 = 4 << 5;
const FOBJ_LOAD_DATA0: u32 = 1;
const FOBJ_LOAD_DATA: u32 = 2;
const FOBJ_LOAD_WAIT: u32 = 3;

macro_rules! hsd_assert {
    ($line:expr, $cond:expr, $text:literal) => {
        if !($cond) {
            super::debug::__assert(c"fobj.c".as_ptr(), $line, concat!($text, "\0").as_ptr() as *const core::ffi::c_char);
        }
    };
}

/// `HSD_FObjReqAnim` (static inline).
pub unsafe fn HSD_FObjReqAnim(fobj: *mut HSD_FObj, startframe: f32) {
    melee_core::ct!("HSD_FObjReqAnim");
    if fobj.is_null() {
        return;
    }
    (*fobj).ad = (*fobj).ad_head;
    (*fobj).time = (*fobj).startframe as f32 + startframe;
    (*fobj).op = 0;
    (*fobj).op_intrp = 0;
    (*fobj).flags &= !0x40;
    (*fobj).nb_pack = 0;
    (*fobj).fterm = 0;
    (*fobj).p0 = 0.0;
    (*fobj).p1 = 0.0;
    (*fobj).d0 = 0.0;
    (*fobj).d1 = 0.0;
    HSD_FObjSetState(fobj, 1);
}

/// `HSD_FObjReqAnimAll`.
pub unsafe fn HSD_FObjReqAnimAll(fobj: *mut HSD_FObj, startframe: f32) {
    melee_core::ct!("HSD_FObjReqAnimAll");
    if fobj.is_null() {
        return;
    }
    let mut fp = fobj;
    while !fp.is_null() {
        HSD_FObjReqAnim(fp, startframe);
        fp = (*fp).next;
    }
}

/// `HSD_FObjStopAnim`: flushes a pending key (`FObj_FlushKeyData`), then stops.
pub unsafe fn HSD_FObjStopAnim(fobj: *mut HSD_FObj, obj: *mut core::ffi::c_void, obj_update: HSD_ObjUpdateFunc, rate: f32) {
    melee_core::ct!("HSD_FObjStopAnim");
    if fobj.is_null() {
        return;
    }
    if (*fobj).op_intrp == HSD_A_OP_KEY {
        HSD_FObjInterpretAnim(fobj, obj, obj_update, rate);
    }
    HSD_FObjSetState(fobj, 0);
}

/// `HSD_FObjStopAnimAll`.
pub unsafe fn HSD_FObjStopAnimAll(mut fobj: *mut HSD_FObj, obj: *mut core::ffi::c_void, obj_update: HSD_ObjUpdateFunc, rate: f32) {
    melee_core::ct!("HSD_FObjStopAnimAll");
    while !fobj.is_null() {
        HSD_FObjStopAnim(fobj, obj, obj_update, rate);
        fobj = (*fobj).next;
    }
}

/// `parseFloat(&pos, frac)` (static).
pub unsafe fn parseFloat(pos: *mut *mut u8, frac: u8) -> f32 {
    melee_core::ct!("parseFloat");
    unsafe fn rd(pos: *mut *mut u8) -> u8 {
        let b = **pos;
        *pos = (*pos).add(1);
        b
    }
    if frac == HSD_A_FRAC_FLOAT {
        let mut d = rd(pos) as u32;
        d |= (rd(pos) as u32) << 8;
        d |= (rd(pos) as u32) << 16;
        d |= (rd(pos) as u32) << 24;
        return f32::from_bits(d);
    }
    let denom: i32 = 1i32 << (frac & 0x1F);
    let numer: f32 = match frac & 0xE0 {
        HSD_A_FRAC_S8 => {
            let n = (*pos).read() as i8 as f32;
            *pos = (*pos).add(1);
            n
        }
        HSD_A_FRAC_U8 => {
            let n = (*pos).read() as f32;
            *pos = (*pos).add(1);
            n
        }
        HSD_A_FRAC_S16 => {
            let n = ((((*pos).add(1).read() as i8 as i32) << 8) | (*pos).read() as i32) as f32;
            *pos = (*pos).add(2);
            n
        }
        HSD_A_FRAC_U16 => {
            let n = ((((*pos).add(1).read() as i32) << 8) | (*pos).read() as i32) as f32;
            *pos = (*pos).add(2);
            n
        }
        _ => return 0.0,
    };
    numer / denom as f32
}

/// `parseOpCode` (static).
unsafe fn parseOpCode(curr_parse: *mut *mut u8) -> u8 {
    **curr_parse & 0xF
}

/// `parsePackInfo` (static).
unsafe fn parsePackInfo(adp: *mut *mut u8) -> u32 {
    let mut d = **adp;
    *adp = (*adp).add(1);
    let mut nb_pack: u32 = (((d >> 4) & 7) + 1) as u32;
    let mut shift: i32 = 3;
    if d & 0x80 == 0 {
        return nb_pack;
    }
    loop {
        d = **adp;
        *adp = (*adp).add(1);
        nb_pack = nb_pack.wrapping_add(crate::cabi::shl((d & 0x7F) as u32, shift as u32));
        shift += 7;
        if d & 0x80 == 0 {
            return nb_pack;
        }
    }
}

/// `FObjLaunchKeyData` (static).
unsafe fn FObjLaunchKeyData(fobj: *mut HSD_FObj) {
    if (*fobj).flags & 0x40 != 0 {
        (*fobj).op_intrp = (*fobj).op;
        (*fobj).flags &= !0x40;
        (*fobj).flags |= 0x80;
        (*fobj).p0 = (*fobj).p1;
    }
}

/// `parseWait` (static).
unsafe fn parseWait(adp: *mut *mut u8) -> i32 {
    let mut wait: i32 = 0;
    let mut shift: i32 = 0;
    loop {
        let d = **adp;
        *adp = (*adp).add(1);
        wait |= crate::cabi::shl((d & 0x7f) as u32, shift as u32) as i32;
        shift += 7;
        if d & 0x80 == 0 {
            return wait;
        }
    }
}

/// `FObjLoadWait` (static).
unsafe fn FObjLoadWait(fobj: *mut HSD_FObj) -> u32 {
    let st = HSD_FObjGetState(fobj);
    hsd_assert!(0x16C, st == FOBJ_LOAD_WAIT, "st == FOBJ_LOAD_WAIT");
    if ((*fobj).ad.offset_from((*fobj).ad_head) as u32) >= (*fobj).length {
        6
    } else {
        (*fobj).fterm = parseWait(&mut (*fobj).ad) as u16;
        (*fobj).flags |= 0x20;
        HSD_FObjSetState(fobj, 2)
    }
}

/// `FObjAnimCON` / `FObjAnimLinear` (identical bodies in the C, differing only in the assert line).
unsafe fn FObj_AnimConLin(fobj: *mut HSD_FObj, line: u32) -> u32 {
    let st = HSD_FObjGetState(fobj);
    hsd_assert!(line, st == FOBJ_LOAD_DATA0 || st == FOBJ_LOAD_DATA, "st == FOBJ_LOAD_DATA0 || st == FOBJ_LOAD_DATA");
    (*fobj).p0 = (*fobj).p1;
    (*fobj).p1 = parseFloat(&mut (*fobj).ad, (*fobj).frac_value);
    if (*fobj).op_intrp != 5 {
        (*fobj).d0 = (*fobj).d1;
        (*fobj).d1 = 0.0;
    }
    HSD_FObjSetState(fobj, if st == FOBJ_LOAD_DATA0 { 3 } else { 4 })
}

/// `FObjAnimCON` (static).
unsafe fn FObjAnimCON(fobj: *mut HSD_FObj) -> u32 {
    FObj_AnimConLin(fobj, 0x17F)
}

/// `FObjAnimLinear` (static).
unsafe fn FObjAnimLinear(fobj: *mut HSD_FObj) -> u32 {
    FObj_AnimConLin(fobj, 0x193)
}

/// `FObjAnimSPL0` (static).
unsafe fn FObjAnimSPL0(fobj: *mut HSD_FObj) -> u32 {
    let st = HSD_FObjGetState(fobj);
    hsd_assert!(0x1A7, st == FOBJ_LOAD_DATA0 || st == FOBJ_LOAD_DATA, "st == FOBJ_LOAD_DATA0 || st == FOBJ_LOAD_DATA");
    (*fobj).p0 = (*fobj).p1;
    (*fobj).d0 = (*fobj).d1;
    (*fobj).p1 = parseFloat(&mut (*fobj).ad, (*fobj).frac_value);
    (*fobj).d1 = 0.0;
    HSD_FObjSetState(fobj, if st == FOBJ_LOAD_DATA0 { 3 } else { 4 })
}

/// `FObjAnimSPL` (static).
unsafe fn FObjAnimSPL(fobj: *mut HSD_FObj) -> u32 {
    let st = HSD_FObjGetState(fobj);
    hsd_assert!(0x1B9, st == FOBJ_LOAD_DATA0 || st == FOBJ_LOAD_DATA, "st == FOBJ_LOAD_DATA0 || st == FOBJ_LOAD_DATA");
    (*fobj).p0 = (*fobj).p1;
    (*fobj).p1 = parseFloat(&mut (*fobj).ad, (*fobj).frac_value);
    (*fobj).d0 = (*fobj).d1;
    (*fobj).d1 = parseFloat(&mut (*fobj).ad, (*fobj).frac_slope);
    HSD_FObjSetState(fobj, if st == FOBJ_LOAD_DATA0 { 3 } else { 4 })
}

/// `FObjAnimSLP` (static).
unsafe fn FObjAnimSLP(fobj: *mut HSD_FObj) -> u32 {
    let st = HSD_FObjGetState(fobj);
    hsd_assert!(0x1CC, st == FOBJ_LOAD_DATA0 || st == FOBJ_LOAD_DATA, "st == FOBJ_LOAD_DATA0 || st == FOBJ_LOAD_DATA");
    (*fobj).d0 = (*fobj).d1;
    (*fobj).d1 = parseFloat(&mut (*fobj).ad, (*fobj).frac_slope);
    HSD_FObjGetState(fobj)
}

/// `FObjAnimKey` (static).
unsafe fn FObjAnimKey(fobj: *mut HSD_FObj) -> u32 {
    let st = HSD_FObjGetState(fobj);
    hsd_assert!(0x1E9, st == FOBJ_LOAD_DATA0 || st == FOBJ_LOAD_DATA, "st == FOBJ_LOAD_DATA0 || st == FOBJ_LOAD_DATA");
    FObjLaunchKeyData(fobj);
    (*fobj).p1 = parseFloat(&mut (*fobj).ad, (*fobj).frac_value);
    (*fobj).flags |= 0x40;
    HSD_FObjSetState(fobj, if st == FOBJ_LOAD_DATA0 { 3 } else { 4 })
}

/// `FObjLoadData` (static inline).
pub unsafe fn FObjLoadData(fobj: *mut HSD_FObj) -> u32 {
    melee_core::ct!("FObjLoadData");
    if ((*fobj).ad.offset_from((*fobj).ad_head) as u32) >= (*fobj).length {
        return 6;
    }
    (*fobj).op_intrp = (*fobj).op;
    if (*fobj).nb_pack == 0 {
        (*fobj).op = parseOpCode(&mut (*fobj).ad);
        (*fobj).nb_pack = parsePackInfo(&mut (*fobj).ad) as u16;
    }
    (*fobj).nb_pack = (*fobj).nb_pack.wrapping_sub(1);
    match (*fobj).op {
        HSD_A_OP_CON => FObjAnimCON(fobj),
        HSD_A_OP_LIN => FObjAnimLinear(fobj),
        HSD_A_OP_SPL0 => FObjAnimSPL0(fobj),
        HSD_A_OP_SPL => FObjAnimSPL(fobj),
        HSD_A_OP_SLP => FObjAnimSLP(fobj),
        HSD_A_OP_KEY => FObjAnimKey(fobj),
        _ => 0,
    }
}

/// `FObjUpdateAnim`. (The TARGET_PC guest-callee branch is not needed: a native callee gets the
/// host stack address directly.)
pub unsafe fn FObjUpdateAnim(fobj: *mut HSD_FObj, obj: *mut core::ffi::c_void, obj_update: HSD_ObjUpdateFunc) {
    melee_core::ct!("FObjUpdateAnim");
    let Some(obj_update) = obj_update else { return };
    let mut fobjdata = HSD_ObjData::zeroed();
    match (*fobj).op_intrp {
        HSD_A_OP_KEY => {
            if (*fobj).flags & 0x80 != 0 {
                fobjdata.fv = (*fobj).p0;
                (*fobj).flags &= 0x7F;
            } else {
                return;
            }
        }
        HSD_A_OP_CON => {
            fobjdata.fv = if (*fobj).time >= (*fobj).fterm as f32 { (*fobj).p1 } else { (*fobj).p0 };
        }
        HSD_A_OP_LIN => {
            if (*fobj).flags & 0x20 != 0 {
                (*fobj).flags &= 0xDF;
                if (*fobj).fterm != 0 {
                    (*fobj).d0 = ((*fobj).p1 - (*fobj).p0) / (*fobj).fterm as f32;
                } else {
                    (*fobj).d0 = 0.0;
                    (*fobj).p0 = (*fobj).p1;
                }
            }
            fobjdata.fv = fmadds((*fobj).d0, (*fobj).time, (*fobj).p0);
        }
        HSD_A_OP_SPL0 | HSD_A_OP_SPL | HSD_A_OP_SLP => {
            fobjdata.fv = if (*fobj).fterm != 0 {
                crate::spline::get_helmite(
                    (1.0f64 / (*fobj).fterm as f64) as f32,
                    (*fobj).time,
                    (*fobj).p0,
                    (*fobj).p1,
                    (*fobj).d0,
                    (*fobj).d1,
                )
            } else {
                (*fobj).p1
            };
        }
        _ => {}
    }
    obj_update(obj, (*fobj).obj_type as _, &mut fobjdata);
}

/// `HSD_FObjInterpretAnim`.
pub unsafe fn HSD_FObjInterpretAnim(fobj: *mut HSD_FObj, obj: *mut core::ffi::c_void, obj_update: HSD_ObjUpdateFunc, rate: f32) {
    melee_core::ct!("HSD_FObjInterpretAnim");
    let mut fterm: f32 = 0.0;
    let mut state = if !fobj.is_null() { HSD_FObjGetState(fobj) } else { 0 };
    if state == 0 {
        return;
    }
    (*fobj).time += rate;
    if ((*fobj).time as f64) < 0.0 {
        return;
    }
    loop {
        match state {
            6 => {
                (*fobj).time += fterm;
                FObjLaunchKeyData(fobj);
                FObjUpdateAnim(fobj, obj, obj_update);
                return;
            }
            1 | 2 => state = FObjLoadData(fobj),
            3 => {
                if (*fobj).flags & 0x80 != 0 {
                    FObjUpdateAnim(fobj, obj, obj_update);
                }
                state = FObjLoadWait(fobj);
            }
            4 => {
                if (*fobj).fterm as f32 <= (*fobj).time {
                    state = 3;
                    fterm = (*fobj).fterm as f32;
                    (*fobj).time -= (*fobj).fterm as f32;
                    HSD_FObjSetState(fobj, state);
                    continue;
                }
                FObjUpdateAnim(fobj, obj, obj_update);
                state = 5;
                HSD_FObjSetState(fobj, state);
                return;
            }
            5 => {
                state = 4;
                HSD_FObjSetState(fobj, state);
            }
            // 0, and 7..15: the C loops forever on those (only reachable through an external SetState).
            _ => return,
        }
    }
}

/// `HSD_FObjInterpretAnimAll`.
pub unsafe fn HSD_FObjInterpretAnimAll(fobj: *mut core::ffi::c_void, obj: *mut core::ffi::c_void, obj_update: HSD_ObjUpdateFunc, rate: f32) {
    melee_core::ct!("HSD_FObjInterpretAnimAll");
    let mut f = fobj as *mut HSD_FObj;
    while !f.is_null() {
        HSD_FObjInterpretAnim(f, obj, obj_update, rate);
        f = (*f).next;
    }
}
