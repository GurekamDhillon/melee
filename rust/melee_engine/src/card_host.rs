//! Core v1 (feature `corev1`): the host's memory card (`MuHostCoreV1.card_*`, which keep the meaning of the console's
//! CARD calls) behind the same hooks as `card::install_at`. The host owns the folder and its files; our save bytes go
//! to it unchanged (GameCube byte order, `lbcardgame::GC_IMAGE`).
use super::card::{defer, info_from, info_to, pump, Cb};
use core::ffi::{c_char, c_void};
use melee_types::{CARDFileInfo, CARDStat};

/// `MuHostCoreV1`'s 14 card calls, in its order.
#[repr(C)]
pub struct HostCard {
    pub probe: extern "C" fn(i32, *mut i32, *mut i32) -> i32,
    pub mount: extern "C" fn(i32) -> i32,
    pub unmount: extern "C" fn(i32) -> i32,
    pub open: extern "C" fn(i32, *const c_char, *mut i32, *mut u32) -> i32,
    pub close: extern "C" fn(i32, i32) -> i32,
    pub create: extern "C" fn(i32, *const c_char, u32, *mut i32) -> i32,
    pub delete: extern "C" fn(i32, *const c_char) -> i32,
    pub rename: extern "C" fn(i32, *const c_char, *const c_char) -> i32,
    pub read: extern "C" fn(i32, i32, *mut c_void, u32, u32) -> i32,
    pub write: extern "C" fn(i32, i32, *const c_void, u32, u32) -> i32,
    pub stat: extern "C" fn(i32, i32, *mut c_void, u32) -> i32,
    pub set_stat: extern "C" fn(i32, i32, *const c_void, u32) -> i32,
    pub free_blocks: extern "C" fn(i32, *mut i32, *mut i32) -> i32,
    pub format: extern "C" fn(i32) -> i32,
}
const _: () = assert!(core::mem::size_of::<CARDStat>() == 0x6C);
const _: () = assert!(core::mem::size_of::<HostCard>() == 14 * 8);

/// Set by the core v1 entry; `card::install` uses it when the card is on.
pub static HOST: std::sync::OnceLock<&'static HostCard> = std::sync::OnceLock::new();
static XFER: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(0);

fn h() -> &'static HostCard {
    HOST.get().expect("core v1 card")
}
fn log(name: &str, r: i32) -> i32 {
    if super::card::diag() {
        eprintln!("gw: card (host): {name} -> {r}");
    }
    r
}
fn probe(chan: i32) -> i32 {
    probe_ex(chan, core::ptr::null_mut(), core::ptr::null_mut())
}
fn probe_ex(chan: i32, mem: *mut i32, sector: *mut i32) -> i32 {
    let (mut m, mut s) = (0, 0);
    let r = (h().probe)(chan, &mut m, &mut s);
    unsafe {
        if !mem.is_null() {
            *mem = m;
        }
        if !sector.is_null() {
            *sector = s;
        }
    }
    log("probe", r)
}
fn fill(info: *mut CARDFileInfo, chan: i32, no: i32, len: u32) {
    let mut i = info_from(info);
    i.chan = chan;
    i.file_no = no;
    i.offset = 0;
    i.length = len as i32;
    i.i_block = 0;
    info_to(info, &i);
}
unsafe fn open(chan: i32, name: *const i8, info: *mut CARDFileInfo) -> i32 {
    let (mut no, mut len) = (-1, 0u32);
    let r = (h().open)(chan, name, &mut no, &mut len);
    if r == 0 {
        fill(info, chan, no, len);
    }
    log("open", r)
}
fn fast_open(chan: i32, no: i32, info: *mut CARDFileInfo) -> i32 {
    let mut st = CARDStat::zeroed();
    let r = (h().stat)(chan, no, (&mut st as *mut CARDStat).cast(), 0x6C);
    if r == 0 {
        fill(info, chan, no, st.length);
    }
    log("fast_open", r)
}
fn close(info: *mut CARDFileInfo) -> i32 {
    let i = info_from(info);
    log("close", (h().close)(i.chan, i.file_no))
}
fn read(info: *mut CARDFileInfo, b: *mut c_void, len: i32, ofs: i32) -> i32 {
    let i = info_from(info);
    let r = (h().read)(i.chan, i.file_no, b, len.max(0) as u32, ofs.max(0) as u32);
    XFER.store(len, std::sync::atomic::Ordering::Relaxed);
    log("read", r)
}
fn write(info: *mut CARDFileInfo, b: *mut c_void, len: i32, ofs: i32) -> i32 {
    let i = info_from(info);
    let r = (h().write)(i.chan, i.file_no, b, len.max(0) as u32, ofs.max(0) as u32);
    XFER.store(len, std::sync::atomic::Ordering::Relaxed);
    log("write", r)
}
unsafe fn create(chan: i32, name: *const i8, size: u32, info: *mut CARDFileInfo, cb: Option<Cb>) -> i32 {
    let mut no = -1;
    let r = (h().create)(chan, name, size, &mut no);
    if r == 0 {
        fill(info, chan, no, size);
    }
    defer(cb, chan, r);
    log("create", r)
}
fn get_status(chan: i32, no: i32, stat: *mut CARDStat) -> i32 {
    if stat.is_null() {
        return -128;
    }
    log("stat", (h().stat)(chan, no, stat.cast(), 0x6C))
}
fn set_status(chan: i32, no: i32, stat: *mut CARDStat, cb: Option<Cb>) -> i32 {
    let r = (h().set_stat)(chan, no, stat as *const c_void, 0x6C);
    defer(cb, chan, r);
    log("set_stat", r)
}

/// The CARD hooks on the host's calls.
pub fn install(_: &HostCard) {
    eprintln!("gw: card: the core v1 host's memory card");
    unsafe {
        super::card::clear_queue();
        use melee_runtime::tu::card::ext as bc;
        use melee_runtime::tu::lbcardgame::ext::hook as cg;
        use melee_runtime::tu::lbcardnew::ext::hook as cn;
        use melee_runtime::tu::lbsnap::ext::hook as sn;
        cg::CARDProbe = Some(probe);
        sn::CARDProbe = Some(probe);
        cn::CARDProbeEx = Some(probe_ex);
        cn::CARDMountAsync = Some(|chan, _work, _detach, attach| {
            let r = log("mount", (h().mount)(chan));
            defer(attach, chan, r);
            r
        });
        cn::CARDUnmount = Some(|chan| (h().unmount)(chan));
        // the host checks its files when it mounts them
        cn::CARDCheckAsync = Some(|chan, cb| {
            let r = probe(chan);
            defer(cb, chan, r);
            r
        });
        cn::CARDFormatAsync = Some(|chan, cb| {
            let r = log("format", (h().format)(chan));
            defer(cb, chan, r);
            r
        });
        cn::CARDOpen = Some(|c, n, i| open(c, n, i));
        cn::CARDClose = Some(close);
        cn::CARDDeleteAsync = Some(|c, n, cb| {
            let r = log("delete", (h().delete)(c, n));
            defer(cb, c, r);
            r
        });
        cn::CARDRenameAsync = Some(|c, o, n, cb| {
            let r = log("rename", (h().rename)(c, o, n));
            defer(cb, c, r);
            r
        });
        cn::CARDGetStatus = Some(get_status);
        cn::CARDFreeBlocks = Some(|c, b, f| log("free_blocks", (h().free_blocks)(c, b, f)));
        bc::CARDCreateAsync_hook = Some(melee_link::cabi::c5(|c, n, s, i, cb| create(c, n, s, i, cb)));
        bc::CARDFastOpen_hook = Some(melee_link::cabi::c3(fast_open));
        bc::CARDOpen_hook = Some(melee_link::cabi::c3(|c, n, i| open(c, n, i)));
        bc::CARDClose_hook = Some(melee_link::cabi::c1(close));
        bc::CARDGetXferredBytes_hook = Some(melee_link::cabi::c1(|_| XFER.load(std::sync::atomic::Ordering::Relaxed)));
        bc::CARDReadAsync_hook = Some(melee_link::cabi::c5(|i, b, l, o, cb| {
            let r = read(i, b, l, o);
            defer(cb, info_from(i).chan, r);
            r
        }));
        bc::CARDRead_hook = Some(melee_link::cabi::c4(read));
        bc::CARDGetStatus_hook = Some(melee_link::cabi::c3(get_status));
        bc::CARDSetStatusAsync_hook = Some(melee_link::cabi::c4(set_status));
        bc::CARDWriteAsync_hook = Some(melee_link::cabi::c5(|i, b, l, o, cb| {
            let r = write(i, b, l, o);
            defer(cb, info_from(i).chan, r);
            r
        }));
        bc::CARDWrite_hook = Some(melee_link::cabi::c4(write));
        melee_runtime::tu::lbcardgame::ext::hook::wait_idle = Some(pump);
        melee_runtime::tu::lbcardnew::ext::hook::wait_idle = Some(pump);
        melee_runtime::tu::lbcardgame::GC_IMAGE = true;
    }
}
