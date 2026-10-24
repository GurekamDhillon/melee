//! Native port of baselib/rumble.c. Commands use generated bitfield accessors.
#![allow(non_snake_case, non_upper_case_globals)]
use core::ptr::{addr_of_mut, null_mut};
use melee_types::*;
pub static mut HSD_Rumble_804C22E0: [HSD_RumbleData; 4] =
    unsafe { core::mem::zeroed() };
pub static mut HSD_Rumble_80406DE0: HSD_RumbleData =
    unsafe { core::mem::zeroed() };
pub mod ext {
    use melee_types::*;
    pub static mut HSD_PadLibData: *mut PadLibData = core::ptr::null_mut();
    pub static mut OSDisableInterrupts: Option<unsafe fn() -> i32> = None;
    pub static mut OSRestoreInterrupts: Option<unsafe fn(i32) -> i32> = None;
    pub static mut PADControlMotor: Option<unsafe fn(i32, u32)> = None;
}
unsafe fn disable() -> i32 {
    (ext::OSDisableInterrupts.expect("TODO OSDisableInterrupts"))()
}
unsafe fn restore(v: i32) {
    (ext::OSRestoreInterrupts.expect("TODO OSRestoreInterrupts"))(v);
}
unsafe fn info() -> *mut RumbleInfo {
    assert!(!ext::HSD_PadLibData.is_null(), "TODO HSD_PadLibData");
    addr_of_mut!((*ext::HSD_PadLibData).rumble_info)
}
unsafe fn channel(n: u8) -> *mut HSD_RumbleData {
    addr_of_mut!(HSD_Rumble_804C22E0)
        .cast::<HSD_RumbleData>()
        .add(n as usize)
}
pub unsafe fn HSD_PadRumbleOn(n: u8) {
    let i = disable();
    (*channel(n)).direct_status = 1;
    restore(i);
}
pub unsafe fn HSD_PadRumbleOffN(n: u8) {
    let i = disable();
    (*channel(n)).direct_status = 0;
    restore(i);
}
pub unsafe fn HSD_PadRumbleFree(
    a: *mut HSD_RumbleData,
    b: *mut HSD_PadRumbleListData,
) {
    let r = info();
    let mut p = addr_of_mut!((*a).listdatap);
    while *p != b {
        p = addr_of_mut!((**p).next);
    }
    *p = (*b).next;
    (*a).nb_list = (*a).nb_list.wrapping_sub(1);
    (*b).next = (*r).listdatap;
    (*r).listdatap = b;
}
pub unsafe fn HSD_PadRumbleRemove(n: u8) {
    let a = channel(n);
    let i = disable();
    let mut b = (*a).listdatap;
    while !b.is_null() {
        let next = (*b).next;
        HSD_PadRumbleFree(a, b);
        b = next;
    }
    restore(i);
}
pub unsafe fn HSD_PadRumbleRemoveAll() {
    for n in 0..4 {
        HSD_PadRumbleRemove(n);
    }
}
pub unsafe fn HSD_PadRumbleRemoveId(n: u8, id: i32) {
    let a = channel(n);
    let i = disable();
    let mut b = (*a).listdatap;
    while !b.is_null() {
        let next = (*b).next;
        if (*b).id == id as u32 {
            HSD_PadRumbleFree(a, b);
        }
        b = next;
    }
    restore(i);
}
pub unsafe fn HSD_PadRumblePause(n: u8, status: i32) {
    let i = disable();
    let mut b = (*channel(n)).listdatap;
    while !b.is_null() {
        let next = (*b).next;
        (*b).pause = status as u8;
        b = next;
    }
    restore(i);
}
pub unsafe fn HSD_PadRumblePauseAll() {
    for n in 0..4 {
        HSD_PadRumblePause(n, 1);
    }
}
pub unsafe fn HSD_PadRumbleUnpauseAll() {
    for n in 0..4 {
        HSD_PadRumblePause(n, 0);
    }
}
pub unsafe fn func_80378430_inline(
    mut p: *mut *mut HSD_PadRumbleListData,
    b: *mut HSD_PadRumbleListData,
) {
    while !(*p).is_null() && (**p).pri <= (*b).pri {
        p = addr_of_mut!((**p).next);
    }
    (*b).next = *p;
    *p = b;
}
pub unsafe fn HSD_PadRumbleAdd(
    n: u8,
    id: i32,
    frame: i32,
    pri: i32,
    list: *mut core::ffi::c_void,
) -> i32 {
    let r = info();
    let a = channel(n);
    let mut result = 0;
    let i = disable();
    let b = (*r).listdatap;
    if !b.is_null() && (*a).nb_list < (*r).max_list {
        (*r).listdatap = (*b).next;
        (*b).id = id as u32;
        (*b).pause = 0;
        (*b).pri = pri as u8;
        (*b).status = 0;
        (*b).loop_count = 0;
        (*b).wait = 0;
        (*b).frame = frame;
        (*b).stack = null_mut();
        (*b).headp = list.cast();
        (*b).listp = list.cast();
        func_80378430_inline(addr_of_mut!((*a).listdatap), b);
        (*a).nb_list = (*a).nb_list.wrapping_add(1);
        result = 1;
    }
    restore(i);
    result
}
pub unsafe fn HSD_Rumble_80378524(a: i32) {
    let i = disable();
    (*info()).unk2 = a as u8;
    restore(i);
}
pub unsafe fn HSD_PadRumbleInterpret1(
    a: *mut HSD_PadRumbleListData,
    b: *mut u8,
) -> i32 {
    if (*a).pause == 1 {
        return 0;
    }
    while (*a).wait == 0 {
        let cmd = (*(*a).listp).command;
        match cmd.op() {
            0 => {
                if (*a).frame == -2 {
                    return 1;
                }
                (*a).listp = (*a).headp;
            }
            1 | 2 | 3 => {
                (*a).status = (3 - cmd.op()) as u8;
                (*a).wait = cmd.frame();
                (*a).listp = (*a).listp.add(1);
            }
            4 => {
                (*a).loop_count = cmd.frame();
                (*a).listp = (*a).listp.add(1);
                (*a).stack = (*a).listp;
            }
            5 => {
                (*a).loop_count = (*a).loop_count.wrapping_sub(1);
                if (*a).loop_count != 0 {
                    (*a).listp = (*a).stack;
                } else {
                    (*a).listp = (*a).listp.add(1);
                }
            }
            _ => {}
        }
    }
    *b = (*a).status;
    (*a).wait = (*a).wait.wrapping_sub(1);
    if (*a).frame != -1 && (*a).frame != -2 {
        (*a).frame = (*a).frame.wrapping_sub(1);
        if (*a).frame == 0 {
            return 1;
        }
    }
    0
}
pub unsafe fn HSD_PadRumbleInterpret() {
    let r = info();
    for n in 0..4 {
        let a = channel(n);
        (*a).status = 0;
        if (*r).unk2 == 0 {
            (*a).status = (*a).direct_status;
            let mut b = (*a).listdatap;
            while !b.is_null() {
                let next = (*b).next;
                if HSD_PadRumbleInterpret1(b, addr_of_mut!((*a).status)) != 0 {
                    HSD_PadRumbleFree(a, b);
                }
                b = next;
            }
        }
        if (*a).status != (*a).last_status {
            let cmd = match (*a).status {
                0 => Some(2),
                1 => Some(0),
                2 => Some(1),
                _ => None,
            };
            if let Some(cmd) = cmd {
                (ext::PADControlMotor.expect("TODO PADControlMotor"))(
                    n as i32, cmd,
                );
            }
            (*a).last_status = (*a).status;
        }
    }
}
pub unsafe fn HSD_PadRumbleInit(a: u16, b: *mut core::ffi::c_void) {
    let r = info();
    (*r).unk2 = 0;
    (*r).max_list = a;
    (*r).listdatap = b.cast();
    if a != 0 {
        for i in 0..a as usize - 1 {
            (*(*r).listdatap.add(i)).next = (*r).listdatap.add(i + 1);
        }
        (*(*r).listdatap.add(a as usize - 1)).next = null_mut();
    }
    for n in 0..4 {
        *channel(n) = HSD_Rumble_80406DE0;
    }
}
