//! Port of src/sysdolphin/baselib/devcom.c: the device-communication request queue that streams
//! data between the DVD, main memory and ARAM through two 16 KiB relay buffers. Requests are
//! `HSD_DevCom` nodes on four priority lists (`devComStatus[0..4]`); `HSD_DevComDVDWakeUp` /
//! `HSD_DevComARAMWakeUp` start the head request of the DVD / ARAM side and the ARQ / DVD
//! callbacks advance it.
//!
//! Native shape: the request fields `src` / `dest` / `args` are the GameCube-width `uintptr_t`
//! (u32) of `melee_types::HSD_DevCom`, so a host pointer cannot round-trip through them. The two
//! conversions the C does with casts are [`addr_of`] (relay buffer -> `uintptr_t`, host address
//! truncated; the platform layer maps it) and [`ptr_of`] (`(void*) uintptr_t`). Both are the
//! identity on the console. Callees of unported TUs (OS interrupts, ARQ, DVD, DC cache calls, the
//! audio allocator) are the hooks in [`ext`].
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, clippy::missing_safety_doc)]

use super::debug::__assert;
use core::ffi::{c_char, c_void};
use core::ptr::null_mut;
use melee_types::{ARQCallback, ARQRequest, DVDCallback, DVDFileInfo, HSD_DevCom, HSD_DevComCallback};

pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
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
    fn OSDisableInterrupts() -> u32;
    fn OSRestoreInterrupts(level: u32);
    fn OSReport(s: &str);
    fn DCStoreRange(addr: *mut c_void, nbytes: u32);
    fn DCInvalidateRange(addr: *mut c_void, nbytes: u32);
    fn ARQPostRequest(req: *mut ARQRequest, owner: u32, type_: u32, priority: u32, source: u32, dest: u32, length: u32, callback: ARQCallback);
    fn DVDFastOpen(entrynum: i32, file_info: *mut DVDFileInfo) -> i32;
    fn DVDReadAsyncPrio(file_info: *mut DVDFileInfo, addr: *mut c_void, length: i32, offset: i32, callback: DVDCallback, prio: i32) -> i32;
    fn HSD_AudioMalloc(size: u32) -> *mut c_void;
    }
}

const DEVCOMDEST_SBUF: i32 = 2;
const DEVCOM_BUF_SIZE: u32 = 0x4000;
const INIT_N_DEVCOMS: usize = 16;

macro_rules! hsd_assert {
    ($line:expr, $cond:expr, $text:literal) => {
        if !($cond) {
            __assert(c"devcom.c".as_ptr(), $line, concat!($text, "\0").as_ptr() as *const c_char);
        }
    };
}

static mut devComARQR: [[ARQRequest; 2]; 2] = unsafe { core::mem::zeroed() };
static mut devComStatus: [*mut HSD_DevCom; 4] = [null_mut(); 4];
static mut HSD_DevCom_804C6330: [*mut HSD_DevCom; 4] = [null_mut(); 4];
static mut HSD_DevCom_804C6330_bufs: [[i32; (DEVCOM_BUF_SIZE / 4) as usize]; 2] = [[0; (DEVCOM_BUF_SIZE / 4) as usize]; 2];
static mut fileinfo: DVDFileInfo = unsafe { core::mem::zeroed() };

static mut HSD_DevCom_804D77F0: *mut HSD_DevCom = null_mut();
static mut aramstate: u8 = 0;
static mut HSD_DevCom_804D77F5: u8 = 0;
static mut HSD_DevCom_804D77F6: u8 = 0;
static mut HSD_DevCom_804D77F7: u8 = 0;
static mut dvdDC: *mut HSD_DevCom = null_mut();
static mut HSD_DevCom_804D77FC: [*mut HSD_DevCom; 2] = [null_mut(); 2];
static mut HSD_DevCom_804D7804: i32 = 0;
static mut aramDC: *mut HSD_DevCom = null_mut();
static mut devComRelayBufFlag: [bool; 2] = [false; 2];
static mut HSD_DevCom_804D6050: i32 = 4;

/// `(uintptr_t) ptr`.
#[inline]
fn addr_of<T>(p: *const T) -> u32 {
    p as usize as u32
}

/// `(void*) addr`.
#[inline]
fn ptr_of(a: u32) -> *mut c_void {
    a as usize as *mut c_void
}

unsafe fn buf_ptr(i: usize) -> *mut c_void {
    HSD_DevCom_804C6330_bufs[i].as_mut_ptr() as *mut c_void
}

/// `HSD_DevComIsBusy`.
pub unsafe fn HSD_DevComIsBusy(idx: i32) -> bool {
    !devComStatus[idx as usize].is_null()
}

/// `HSD_DevComUnlink` (static).
unsafe fn HSD_DevComUnlink(dc: *mut HSD_DevCom) {
    let enabled = ext::OSDisableInterrupts();
    let i = ((*dc).dcReq & 3) as usize;

    if devComStatus[i] == dc {
        devComStatus[i] = (*dc).next;
        if HSD_DevCom_804C6330[i] == dc {
            HSD_DevCom_804C6330[i] = null_mut();
        }
        ext::OSRestoreInterrupts(enabled);
        return;
    }

    let mut curr = devComStatus[i];
    while !(*curr).next.is_null() {
        if (*curr).next == dc {
            (*curr).next = (*dc).next;
            if HSD_DevCom_804C6330[i] == dc {
                HSD_DevCom_804C6330[i] = curr;
            }
            ext::OSRestoreInterrupts(enabled);
            return;
        }
        curr = (*curr).next;
    }
    hsd_assert!(0x6E, false, "0");
    ext::OSRestoreInterrupts(enabled);
}

/// Which relay slot an ARQ request belongs to (`&devComARQR[i][0]`); asserts at `line` otherwise.
unsafe fn request_slot(request: *mut ARQRequest, line: u32) -> usize {
    if request == devComARQR[0].as_mut_ptr() {
        0
    } else if request == devComARQR[1].as_mut_ptr() {
        1
    } else {
        hsd_assert!(line, false, "0");
        0
    }
}

/// `HSD_DevComStdCallback` (static).
unsafe extern "C" fn HSD_DevComStdCallback(request: *mut ARQRequest) {
    let i = request_slot(request, 0xA5);
    aramstate = 0;
    devComRelayBufFlag[i] = false;
    HSD_DevComDVDWakeUp();
    HSD_DevComARAMWakeUp();
}

/// `HSD_DevComARAMCallback_inline` (static inline): return `devcom` to the free list.
unsafe fn HSD_DevComARAMCallback_inline(devcom: *mut HSD_DevCom) {
    let enabled = ext::OSDisableInterrupts();
    (*devcom).next = HSD_DevCom_804D77F0;
    HSD_DevCom_804D77F0 = devcom;
    ext::OSRestoreInterrupts(enabled);
}

/// `HSD_DevComARAMCallback` (static).
unsafe extern "C" fn HSD_DevComARAMCallback(request: *mut ARQRequest) {
    let buf: *mut c_void;
    if (*aramDC).r#type == 0x1A {
        let i = if request == devComARQR[0].as_mut_ptr() { 0 } else { 1 };
        buf = buf_ptr(i);
    } else {
        buf = null_mut();
    }

    if let Some(cb) = (*aramDC).callback {
        cb((*aramDC).dcReq, (*aramDC).args, buf, (*aramDC).cancelflag as i32);
    }

    HSD_DevComUnlink(aramDC);
    HSD_DevComARAMCallback_inline(aramDC);
    HSD_DevComStdCallback(request);
}

/// `getRelayBufIdx` (static inline).
unsafe fn getRelayBufIdx() -> i32 {
    for i in 0..2 {
        if !devComRelayBufFlag[i] {
            devComRelayBufFlag[i] = true;
            return i as i32;
        }
    }
    -1
}

/// `HSD_DevComARAMWakeUp`.
pub unsafe fn HSD_DevComARAMWakeUp() {
    let enabled = ext::OSDisableInterrupts();
    if aramstate != 0 {
        ext::OSRestoreInterrupts(enabled);
        return;
    }
    aramDC = devComStatus[3];
    if !devComStatus[3].is_null() {
        if (*aramDC).cancelflag != 0 {
            if let Some(cb) = (*aramDC).callback {
                cb((*aramDC).dcReq, (*aramDC).args, null_mut(), 1);
            }
            HSD_DevComUnlink(aramDC);
            ext::OSRestoreInterrupts(enabled);
            HSD_DevComARAMWakeUp();
            return;
        }
        let req_idx = getRelayBufIdx();
        if req_idx >= 0 {
            let ri = req_idx as usize;
            let t = (*aramDC).r#type;
            if t == 3 {
                let arq_callback: ARQCallback;
                let xfer_size: u32;
                if (*aramDC).size as u32 > DEVCOM_BUF_SIZE {
                    arq_callback = Some(HSD_DevComStdCallback);
                    xfer_size = DEVCOM_BUF_SIZE;
                } else {
                    arq_callback = Some(HSD_DevComARAMCallback);
                    xfer_size = (*aramDC).size as u32;
                }
                for w in HSD_DevCom_804C6330_bufs[ri].iter_mut() {
                    *w = 0;
                }
                ext::DCStoreRange(buf_ptr(ri), DEVCOM_BUF_SIZE);
                ext::ARQPostRequest(
                    devComARQR[ri].as_mut_ptr(), 0, 0, 1, addr_of(buf_ptr(ri)), (*aramDC).dest as u32, xfer_size, arq_callback,
                );
                (*aramDC).dest = (*aramDC).dest.wrapping_add(xfer_size as usize);
                (*aramDC).size = (*aramDC).size.wrapping_sub(xfer_size as usize);
                aramstate = 1;
            } else if t == 0xB {
                ext::DCStoreRange(ptr_of((*aramDC).src as u32), (*aramDC).size as u32);
                ext::ARQPostRequest(
                    devComARQR[ri].as_mut_ptr(), 0, 0, 1, (*aramDC).src as u32, (*aramDC).dest as u32, (*aramDC).size as u32,
                    Some(HSD_DevComARAMCallback),
                );
                aramstate = 1;
            } else if t == 0x19 {
                ext::DCInvalidateRange(ptr_of((*aramDC).dest as u32), (*aramDC).size as u32);
                ext::ARQPostRequest(
                    devComARQR[ri].as_mut_ptr(), 0, 1, 1, (*aramDC).src as u32, (*aramDC).dest as u32, (*aramDC).size as u32,
                    Some(HSD_DevComARAMCallback),
                );
                aramstate = 1;
            } else if t == 0x1A {
                ext::DCInvalidateRange(buf_ptr(ri), DEVCOM_BUF_SIZE);
                ext::ARQPostRequest(
                    devComARQR[ri].as_mut_ptr(), 0, 1, 1, (*aramDC).src as u32, addr_of(buf_ptr(ri)), (*aramDC).size as u32,
                    Some(HSD_DevComARAMCallback),
                );
                aramstate = 1;
            } else if t == 0x1B {
                ext::DCInvalidateRange(buf_ptr(ri), DEVCOM_BUF_SIZE);
                let arq_callback2: ARQCallback;
                let xfer_size2: u32;
                if (*aramDC).size as u32 > DEVCOM_BUF_SIZE {
                    arq_callback2 = Some(HSD_DevComStdCallback);
                    xfer_size2 = DEVCOM_BUF_SIZE;
                } else {
                    arq_callback2 = Some(HSD_DevComARAMCallback);
                    xfer_size2 = (*aramDC).size as u32;
                }
                ext::ARQPostRequest(
                    devComARQR[ri].as_mut_ptr().add(1), 0, 1, 1, (*aramDC).src as u32, addr_of(buf_ptr(ri)), xfer_size2, None,
                );
                ext::ARQPostRequest(
                    devComARQR[ri].as_mut_ptr(), 0, 0, 1, addr_of(buf_ptr(ri)), (*aramDC).dest as u32, xfer_size2, arq_callback2,
                );
                (*aramDC).src = (*aramDC).src.wrapping_add(xfer_size2 as usize);
                (*aramDC).dest = (*aramDC).dest.wrapping_add(xfer_size2 as usize);
                (*aramDC).size = (*aramDC).size.wrapping_sub(xfer_size2 as usize);
                aramstate = 1;
            }
        }
    }
    ext::OSRestoreInterrupts(enabled);
}

/// `HSD_DevComDVDStdCallback` (static).
unsafe extern "C" fn HSD_DevComDVDStdCallback(request: *mut ARQRequest) {
    let i = request_slot(request, 0x158);
    devComRelayBufFlag[i] = false;
    HSD_DevComDVDWakeUp();
    HSD_DevComARAMWakeUp();
}

/// `HSD_DevComDVDARAMEndCallback` (static).
unsafe extern "C" fn HSD_DevComDVDARAMEndCallback(request: *mut ARQRequest) {
    HSD_DevComDVDStdCallback(request);

    let i = if request == devComARQR[0].as_mut_ptr() { 0 } else { 1 };

    let dc = HSD_DevCom_804D77FC[i];
    if let Some(cb) = (*dc).callback {
        if HSD_DevCom_804D7804 == 0 {
            cb((*dc).dcReq, (*dc).args, null_mut(), (*dc).cancelflag as i32);
        }
    }
    HSD_DevComARAMCallback_inline(HSD_DevCom_804D77FC[i]);
    HSD_DevCom_804D77FC[i] = null_mut();
}

/// `HSD_DevComDVDMemCallback` (static).
unsafe extern "C" fn HSD_DevComDVDMemCallback(result: i32, _unused: *mut DVDFileInfo) {
    if result == -1 {
        HSD_DevCom_804D7804 = 1;
    }
    if (*dvdDC).size > 0x80000 {
        (*dvdDC).src = (*dvdDC).src.wrapping_add(0x80000);
        (*dvdDC).dest = (*dvdDC).dest.wrapping_add(0x80000);
        (*dvdDC).size -= 0x80000;
        HSD_DevCom_804D77F5 = 0;
        HSD_DevComDVDWakeUp();
        return;
    }
    if let Some(cb) = (*dvdDC).callback {
        if HSD_DevCom_804D7804 == 0 {
            cb((*dvdDC).dcReq, (*dvdDC).args, null_mut(), (*dvdDC).cancelflag as i32);
        }
    }
    HSD_DevComUnlink(dvdDC);
    let dc = dvdDC;
    let enabled = ext::OSDisableInterrupts();
    (*dc).next = HSD_DevCom_804D77F0;
    HSD_DevCom_804D77F0 = dc;
    ext::OSRestoreInterrupts(enabled);
    HSD_DevCom_804D77F5 = 0;
    HSD_DevComDVDWakeUp();
}

/// `HSD_DevComDVDCallback` (static).
unsafe extern "C" fn HSD_DevComDVDCallback(result: i32, _unused: *mut DVDFileInfo) {
    if result == -1 {
        HSD_DevCom_804D7804 = 1;
    }
    let type_ = (*dvdDC).r#type;
    if type_ == 0x22 {
        hsd_assert!(0x18C, (*dvdDC).size as u32 <= DEVCOM_BUF_SIZE, "dvdDC->size <= DEVCOM_BUF_SIZE");
        hsd_assert!(0x18D, (*dvdDC).callback.is_some(), "dvdDC->callback");
        if HSD_DevCom_804D7804 == 0 {
            ((*dvdDC).callback.unwrap())(
                (*dvdDC).dcReq,
                (*dvdDC).args,
                buf_ptr(HSD_DevCom_804D77F6 as usize),
                (*dvdDC).cancelflag as i32,
            );
        }
        HSD_DevComUnlink(dvdDC);
        let dc = dvdDC;
        let enabled = ext::OSDisableInterrupts();
        (*dc).next = HSD_DevCom_804D77F0;
        HSD_DevCom_804D77F0 = dc;
        ext::OSRestoreInterrupts(enabled);
        HSD_DevCom_804D77F5 = 0;
        devComRelayBufFlag[HSD_DevCom_804D77F6 as usize] = false;
        HSD_DevComDVDWakeUp();
        HSD_DevComARAMWakeUp();
    } else if type_ == 0x23 {
        HSD_DevCom_804D77F7 = HSD_DevCom_804D77F6;
        let k = HSD_DevCom_804D77F7 as usize;
        if (*dvdDC).size as u32 > DEVCOM_BUF_SIZE {
            ext::ARQPostRequest(
                devComARQR[k].as_mut_ptr(), 0, 0, 1, addr_of(buf_ptr(k)), (*dvdDC).dest as u32, DEVCOM_BUF_SIZE,
                Some(HSD_DevComDVDStdCallback),
            );
            (*dvdDC).src = (*dvdDC).src.wrapping_add(DEVCOM_BUF_SIZE as usize);
            (*dvdDC).dest = (*dvdDC).dest.wrapping_add(DEVCOM_BUF_SIZE as usize);
            (*dvdDC).size -= DEVCOM_BUF_SIZE as usize;
            HSD_DevCom_804D77F5 = 0;
            HSD_DevComDVDWakeUp();
        } else {
            HSD_DevCom_804D77FC[k] = dvdDC;
            ext::ARQPostRequest(
                devComARQR[k].as_mut_ptr(), 0, 0, 1, addr_of(buf_ptr(k)), (*dvdDC).dest as u32, (*dvdDC).size as u32,
                Some(HSD_DevComDVDARAMEndCallback),
            );
            HSD_DevComUnlink(dvdDC);
            HSD_DevCom_804D77F5 = 0;
            HSD_DevComDVDWakeUp();
        }
    }
}

/// `HSD_DevComDVDWakeUp`.
pub unsafe fn HSD_DevComDVDWakeUp() {
    let enabled = ext::OSDisableInterrupts();

    if HSD_DevCom_804D77F5 != 0 {
        ext::OSRestoreInterrupts(enabled);
        return;
    }
    for i in 0..3 {
        dvdDC = devComStatus[i];
        if !dvdDC.is_null() {
            if (*dvdDC).cancelflag != 0 {
                if let Some(cb) = (*dvdDC).callback {
                    cb((*dvdDC).dcReq, (*dvdDC).args, null_mut(), 1);
                }
                HSD_DevComUnlink(dvdDC);
                ext::OSRestoreInterrupts(enabled);
                HSD_DevComDVDWakeUp();
                return;
            }
            ext::DVDFastOpen((*dvdDC).file, &mut fileinfo);
            if (*dvdDC).r#type == 0x21 {
                ext::DVDReadAsyncPrio(
                    &mut fileinfo,
                    ptr_of((*dvdDC).dest as u32),
                    ((*dvdDC).size as u32).min(0x80000) as i32,
                    (*dvdDC).src as i32,
                    Some(HSD_DevComDVDMemCallback),
                    2,
                );
                HSD_DevCom_804D77F5 = 1;
                ext::OSRestoreInterrupts(enabled);
                return;
            }
            let buf_idx = getRelayBufIdx();
            if buf_idx >= 0 {
                HSD_DevCom_804D77F6 = buf_idx as u8;
                ext::DVDReadAsyncPrio(
                    &mut fileinfo,
                    buf_ptr(buf_idx as usize),
                    ((*dvdDC).size as u32).min(DEVCOM_BUF_SIZE) as i32,
                    (*dvdDC).src as i32,
                    Some(HSD_DevComDVDCallback),
                    2,
                );
                HSD_DevCom_804D77F5 = 1;
                ext::OSRestoreInterrupts(enabled);
                return;
            }
        }
    }
    ext::OSRestoreInterrupts(enabled);
}

/// `HSD_DevComGetDestType` (static inline).
#[inline]
fn HSD_DevComGetDestType(type_: i32) -> i32 {
    type_ & 7
}

/// `DevComLinkNext` (static inline): chain `dc[1..15]`.
unsafe fn DevComLinkNext(dc: *mut HSD_DevCom) {
    let mut i = 1;
    while i < INIT_N_DEVCOMS - 1 {
        (*dc.add(i)).next = dc.add(i + 1);
        i += 1;
    }
    (*dc.add(i)).next = null_mut();
}

/// `HSD_DevComRequest`.
pub unsafe fn HSD_DevComRequest(
    file: i32,
    src: u32,
    dest: u32,
    size: usize,
    type_: i32,
    mut pri: i32,
    cb: HSD_DevComCallback,
    args: u32,
) -> i32 {
    let enabled = ext::OSDisableInterrupts();

    let mut dc = HSD_DevCom_804D77F0;
    if !dc.is_null() {
        HSD_DevCom_804D77F0 = (*dc).next;
        ext::OSRestoreInterrupts(enabled);
    } else {
        dc = ext::HSD_AudioMalloc((core::mem::size_of::<HSD_DevCom>() * INIT_N_DEVCOMS) as u32) as *mut HSD_DevCom;
        DevComLinkNext(dc);

        HSD_DevCom_804D77F0 = dc.add(1);
        ext::OSRestoreInterrupts(enabled);
    }

    hsd_assert!(0x1ED, !dc.is_null(), "dc");
    hsd_assert!(
        0x1EE,
        !(HSD_DevComGetDestType(type_) == DEVCOMDEST_SBUF && size as u32 > DEVCOM_BUF_SIZE),
        "!(HSD_DevComGetDestType(type) == DEVCOMDEST_SBUF && size > DEVCOM_BUF_SIZE)"
    );

    // TARGET_PC: report the operands before the alignment asserts die.
    if src % 32 != 0 || dest % 32 != 0 || size % 32 != 0 || size == 0 {
        ext::OSReport(&format!(
            "HSD_DevComRequest: file={} src=0x{:08X} dest=0x{:08X} size=0x{:X} type=0x{:X} pri={} cb={:p} (src%32={} dest%32={} size%32={})\n",
            file,
            src,
            dest,
            size as u32,
            type_,
            pri,
            cb.map_or(core::ptr::null(), |f| f as *const c_void),
            (src % 32) as i32,
            (dest % 32) as i32,
            (size % 32) as i32
        ));
    }

    hsd_assert!(0x1EF, src % 32 == 0, "src % 32 == 0");
    hsd_assert!(0x1F0, dest % 32 == 0, "dest % 32 == 0");
    hsd_assert!(0x1F1, size % 32 == 0, "size % 32 == 0");
    hsd_assert!(0x1F2, size != 0, "size != 0");

    pri = if (type_ & 0x38) == 0x20 { pri } else { 3 };

    (*dc).file = file;
    (*dc).src = src as usize;
    (*dc).dest = dest as usize;
    (*dc).size = size;
    (*dc).r#type = type_ as u16;
    (*dc).cancelflag = 0;
    (*dc).callback = cb;
    (*dc).args = args as usize;

    let enabled = ext::OSDisableInterrupts();
    let result = HSD_DevCom_804D6050 + pri;
    (*dc).dcReq = result;
    HSD_DevCom_804D6050 += 4;
    let pu = pri as usize;
    if !HSD_DevCom_804C6330[pu].is_null() {
        (*HSD_DevCom_804C6330[pu]).next = dc;
    }
    HSD_DevCom_804C6330[pu] = dc;
    (*dc).next = null_mut();
    if devComStatus[pu].is_null() {
        devComStatus[pu] = dc;
        HSD_DevComDVDWakeUp();
        HSD_DevComARAMWakeUp();
    }
    ext::OSRestoreInterrupts(enabled);

    result
}

/// `HSD_DevComCancelEx_inline` (static inline).
unsafe fn HSD_DevComCancelEx_inline(dc_req: i32) -> *mut HSD_DevCom {
    let mut cur = devComStatus[(dc_req & 3) as usize];
    while !cur.is_null() {
        if (*cur).dcReq == dc_req {
            return cur;
        }
        cur = (*cur).next;
    }
    null_mut()
}

/// `HSD_DevComCancelEx`.
pub unsafe fn HSD_DevComCancelEx(dc_req: i32, flags: u32, cb: HSD_DevComCallback, args: u32) -> i32 {
    let enabled = ext::OSDisableInterrupts();

    let dc = HSD_DevComCancelEx_inline(dc_req);
    if !dc.is_null() {
        let tmp = (dc_req & 3) as usize;
        if flags & 1 != 0 {
            (*dc).callback = cb;
        }
        if flags & 2 != 0 {
            (*dc).args = args as usize;
        }
        if devComStatus[tmp] == dc {
            (*dc).cancelflag = 1;
        } else {
            if let Some(f) = (*dc).callback {
                f((*dc).dcReq, (*dc).args, null_mut(), 1);
            }
            HSD_DevComUnlink(dc);
        }
    } else {
        for i in 0..2 {
            let dc = HSD_DevCom_804D77FC[i];
            if !dc.is_null() && (*dc).dcReq == dc_req {
                (*dc).callback = cb;
                (*dc).args = args as usize;
                (*HSD_DevCom_804D77FC[i]).cancelflag = 1;
            }
        }
    }
    ext::OSRestoreInterrupts(enabled);
    0
}
