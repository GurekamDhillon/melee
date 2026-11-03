//! The CARD API shim (port of pc/platform/shim_card.c) over `melee_data::card` (the port of Aurora's GCI folder).
//!
//! The card is a folder of `.gci` files, `<base>/USA/Card A` (the C build's layout and naming, so a save made by one
//! build loads in the other). `<base>` is `MELEE_CARD_PATH`, else `card/` beside the executable, as `gw_CARDInit`.
//! `MELEE_CARD=0` keeps the no-card behaviour. Every operation completes inline; the game's completion callback is
//! queued and run from `pump` (the C's `gw_defer`, pumped from `gw_wait_idle` / `gw_frame_tick`), because lbcardnew
//! sets its own state right after starting a task and would lose a callback delivered inside the call.
//!
//! Never installed for golden / synctest / diff runs: those keep the seams' NOCARD defaults (the C test harness runs
//! save-less too), so a save can never change a sim result there.
#![allow(static_mut_refs)]

use core::ffi::c_void;
use melee_data::card::{res, CardSystem, FileInfo, Stat};
use melee_types::{CARDFileInfo, CARDStat};
use std::collections::VecDeque;
use std::path::{Path, PathBuf};

type Cb = unsafe extern "C" fn(i32, i32);

static mut CARD: Option<CardSystem> = None;
static mut QUEUE: VecDeque<(Cb, i32, i32)> = VecDeque::new();
static mut BASE: Option<PathBuf> = None;

/// The card folder in use (`gw_CardBasePath`), None while the card is off.
pub fn base_path() -> Option<PathBuf> {
    unsafe { BASE.clone() }
}

pub fn enabled() -> bool {
    unsafe { CARD.is_some() }
}

fn defer(cb: Option<Cb>, chan: i32, result: i32) {
    if let Some(cb) = cb {
        unsafe { QUEUE.push_back((cb, chan, result)) };
    }
}

/// `gw_defer`'s pump for the card completions: runs every queued callback (and any they queue) in order.
pub fn pump() {
    loop {
        let Some((cb, chan, r)) = (unsafe { QUEUE.pop_front() }) else { break };
        unsafe { cb(chan, r) };
    }
}

fn diag() -> bool {
    static D: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *D.get_or_init(|| std::env::var("MELEE_CARD_DIAG").map(|v| v.starts_with('1')).unwrap_or(false))
}

macro_rules! logr {
    ($name:literal, $r:expr) => {{
        let r = $r;
        if diag() {
            eprintln!("gw: card: {} -> {}", $name, r);
        }
        r
    }};
}

fn card() -> &'static mut CardSystem {
    unsafe { CARD.as_mut().expect("card shim not installed") }
}

/// The default base: `card` beside the executable (created on demand), as `gw_CARDInit`.
pub fn default_base() -> PathBuf {
    let dir = std::env::current_exe().ok().and_then(|p| p.parent().map(Path::to_path_buf)).unwrap_or_else(|| PathBuf::from("."));
    dir.join("card")
}

/// Where the card would live from the environment, or None when `MELEE_CARD=0`.
pub fn configured_base() -> Option<PathBuf> {
    if std::env::var("MELEE_CARD").map(|v| v.starts_with('0')).unwrap_or(false) {
        return None;
    }
    match std::env::var_os("MELEE_CARD_PATH") {
        Some(p) if !p.is_empty() => Some(PathBuf::from(p)),
        _ => Some(default_base()),
    }
}

unsafe fn cstr<'a>(p: *const i8) -> &'a [u8] {
    if p.is_null() {
        return &[];
    }
    core::ffi::CStr::from_ptr(p).to_bytes()
}

fn info_from(p: *const CARDFileInfo) -> FileInfo {
    if p.is_null() {
        return FileInfo::default();
    }
    let g = unsafe { &*p };
    FileInfo { chan: g.chan, file_no: g.fileNo, offset: g.offset, length: g.length, i_block: g.iBlock }
}

fn info_to(p: *mut CARDFileInfo, i: &FileInfo) {
    if p.is_null() {
        return;
    }
    let g = unsafe { &mut *p };
    g.chan = i.chan;
    g.fileNo = i.file_no;
    g.offset = i.offset;
    g.length = i.length;
    g.iBlock = i.i_block;
}

fn stat_to(p: *mut CARDStat, s: &Stat) {
    let g = unsafe { &mut *p };
    for i in 0..32 {
        g.fileName[i] = s.file_name[i] as i8;
    }
    g.length = s.length;
    g.time = s.time;
    g.gameName = s.game_name;
    g.company = s.company;
    g.bannerFormat = s.banner_format;
    g.iconAddr = s.icon_addr;
    g.iconFormat = s.icon_format;
    g.iconSpeed = s.icon_speed;
    g.commentAddr = s.comment_addr;
    g.offsetBanner = s.offset_banner;
    g.offsetBannerTlut = s.offset_banner_tlut;
    g.offsetIcon = s.offset_icon;
    g.offsetIconTlut = s.offset_icon_tlut;
    g.offsetData = s.offset_data;
}

fn stat_from(p: *const CARDStat) -> Stat {
    let g = unsafe { &*p };
    let mut s = Stat::zeroed();
    for i in 0..32 {
        s.file_name[i] = g.fileName[i] as u8;
    }
    s.length = g.length;
    s.time = g.time;
    s.game_name = g.gameName;
    s.company = g.company;
    s.banner_format = g.bannerFormat;
    s.icon_addr = g.iconAddr;
    s.icon_format = g.iconFormat;
    s.icon_speed = g.iconSpeed;
    s.comment_addr = g.commentAddr;
    s.offset_banner = g.offsetBanner;
    s.offset_banner_tlut = g.offsetBannerTlut;
    s.offset_icon = g.offsetIcon;
    s.offset_icon_tlut = g.offsetIconTlut;
    s.offset_data = g.offsetData;
    s
}

fn buf<'a>(p: *mut c_void, len: i32) -> &'a mut [u8] {
    if p.is_null() || len <= 0 {
        return &mut [];
    }
    unsafe { core::slice::from_raw_parts_mut(p as *mut u8, len as usize) }
}

// ---- the entry points (gw_CARD*) ----

pub fn probe(chan: i32) -> i32 {
    logr!("probe", card().probe(chan))
}
pub fn probe_ex(chan: i32, mem: *mut i32, sector: *mut i32) -> i32 {
    let (r, m, s) = card().probe_ex(chan);
    unsafe {
        if !mem.is_null() {
            *mem = m;
        }
        if !sector.is_null() {
            *sector = s;
        }
    }
    logr!("probe_ex", r)
}
pub fn mount_async(chan: i32, attach: Option<Cb>) -> i32 {
    let r = card().mount(chan);
    defer(attach, chan, r);
    logr!("mount_async", r)
}
pub fn unmount(chan: i32) -> i32 {
    card().unmount(chan)
}
pub fn check_async(chan: i32, cb: Option<Cb>) -> i32 {
    let r = card().check(chan);
    defer(cb, chan, r);
    logr!("check_async", r)
}
pub fn format_async(chan: i32, cb: Option<Cb>) -> i32 {
    let r = card().format(chan);
    defer(cb, chan, r);
    logr!("format_async", r)
}
pub unsafe fn open(chan: i32, name: *const i8, info: *mut CARDFileInfo) -> i32 {
    let mut i = FileInfo::default();
    let r = card().open(chan, cstr(name), &mut i);
    info_to(info, &i);
    logr!("open", r)
}
pub fn fast_open(chan: i32, no: i32, info: *mut CARDFileInfo) -> i32 {
    let mut i = FileInfo::default();
    let r = card().fast_open(chan, no, &mut i);
    info_to(info, &i);
    logr!("fast_open", r)
}
pub fn close(info: *mut CARDFileInfo) -> i32 {
    let mut i = info_from(info);
    let r = card().close(&mut i);
    info_to(info, &i);
    logr!("close", r)
}
pub fn read(info: *mut CARDFileInfo, b: *mut c_void, len: i32, ofs: i32) -> i32 {
    let i = info_from(info);
    let r = card().read(&i, buf(b, len), len, ofs);
    info_to(info, &i);
    logr!("read", r)
}
pub fn read_async(info: *mut CARDFileInfo, b: *mut c_void, len: i32, ofs: i32, cb: Option<Cb>) -> i32 {
    let r = read(info, b, len, ofs);
    defer(cb, info_from(info).chan, r);
    logr!("read_async", r)
}
pub fn write(info: *mut CARDFileInfo, b: *mut c_void, len: i32, ofs: i32) -> i32 {
    let i = info_from(info);
    let r = card().write(&i, buf(b, len), len, ofs);
    info_to(info, &i);
    logr!("write", r)
}
pub fn write_async(info: *mut CARDFileInfo, b: *mut c_void, len: i32, ofs: i32, cb: Option<Cb>) -> i32 {
    let r = write(info, b, len, ofs);
    defer(cb, info_from(info).chan, r);
    logr!("write_async", r)
}
pub unsafe fn create_async(chan: i32, name: *const i8, size: u32, info: *mut CARDFileInfo, cb: Option<Cb>) -> i32 {
    let mut i = FileInfo::default();
    let r = card().create(chan, cstr(name), size, &mut i);
    info_to(info, &i);
    defer(cb, chan, r);
    logr!("create_async", r)
}
pub unsafe fn delete_async(chan: i32, name: *const i8, cb: Option<Cb>) -> i32 {
    let r = card().delete(chan, cstr(name));
    defer(cb, chan, r);
    logr!("delete_async", r)
}
pub unsafe fn rename_async(chan: i32, old: *const i8, new: *const i8, cb: Option<Cb>) -> i32 {
    let r = card().rename(chan, cstr(old), cstr(new));
    defer(cb, chan, r);
    logr!("rename_async", r)
}
pub fn get_status(chan: i32, no: i32, stat: *mut CARDStat) -> i32 {
    let mut s = Stat::zeroed();
    let r = card().get_status(chan, no, &mut s);
    if r == res::READY && !stat.is_null() {
        stat_to(stat, &s);
    }
    logr!("get_status", r)
}
pub fn set_status_async(chan: i32, no: i32, stat: *mut CARDStat, cb: Option<Cb>) -> i32 {
    let s = stat_from(stat);
    let r = card().set_status(chan, no, &s);
    defer(cb, chan, r);
    logr!("set_status_async", r)
}
pub fn free_blocks(chan: i32, bytes: *mut i32, files: *mut i32) -> i32 {
    let (r, b, f) = card().free_blocks(chan);
    unsafe {
        if !bytes.is_null() {
            *bytes = b;
        }
        if !files.is_null() {
            *files = f;
        }
    }
    logr!("free_blocks", r)
}
pub fn xferred_bytes(chan: i32) -> i32 {
    card().xferred_bytes(chan)
}

/// `gw_CARDInit` + the hook install: every CARD callee of the card TUs (lbcardgame, lbcardnew, baselib card.c,
/// lbsnap) goes through this shim, and lbcardgame keeps its save files in GameCube byte order on the card
/// (`lbcardgame::gc_image`). Returns false (card off, NOCARD defaults kept) when `MELEE_CARD=0`.
pub fn install() -> bool {
    let Some(base) = configured_base() else {
        eprintln!("gw: card: disabled (MELEE_CARD=0); reporting no card");
        return false;
    };
    install_at(&base);
    true
}

pub fn install_at(base: &Path) {
    let _ = std::fs::create_dir_all(base);
    let mut c = CardSystem::with_system_clock();
    c.init(base);
    eprintln!("gw: card: initialised (GCI folder) at {}", base.display());
    unsafe {
        CARD = Some(c);
        BASE = Some(base.to_path_buf());
        QUEUE.clear();
        use melee_runtime::tu::card::ext as bc;
        use melee_runtime::tu::lbcardgame::ext::hook as cg;
        use melee_runtime::tu::lbcardnew::ext::hook as cn;
        use melee_runtime::tu::lbsnap::ext::hook as sn;
        cg::CARDProbe = Some(probe);
        sn::CARDProbe = Some(probe);
        cn::CARDProbeEx = Some(probe_ex);
        cn::CARDMountAsync = Some(|chan, _work, _detach, attach| mount_async(chan, attach));
        cn::CARDUnmount = Some(unmount);
        cn::CARDCheckAsync = Some(check_async);
        cn::CARDFormatAsync = Some(format_async);
        cn::CARDOpen = Some(|c, n, i| unsafe { open(c, n, i) });
        cn::CARDClose = Some(close);
        cn::CARDDeleteAsync = Some(|c, n, cb| unsafe { delete_async(c, n, cb) });
        cn::CARDRenameAsync = Some(|c, o, n, cb| unsafe { rename_async(c, o, n, cb) });
        cn::CARDGetStatus = Some(get_status);
        cn::CARDFreeBlocks = Some(free_blocks);
        bc::CARDCreateAsync_hook = Some(melee_link::cabi::c5(|c, n, s, i, cb| unsafe { create_async(c, n, s, i, cb) }));
        bc::CARDFastOpen_hook = Some(melee_link::cabi::c3(fast_open));
        bc::CARDOpen_hook = Some(melee_link::cabi::c3(|c, n, i| unsafe { open(c, n, i) }));
        bc::CARDClose_hook = Some(melee_link::cabi::c1(close));
        bc::CARDGetXferredBytes_hook = Some(melee_link::cabi::c1(xferred_bytes));
        bc::CARDReadAsync_hook = Some(melee_link::cabi::c5(read_async));
        bc::CARDRead_hook = Some(melee_link::cabi::c4(read));
        bc::CARDGetStatus_hook = Some(melee_link::cabi::c3(get_status));
        bc::CARDSetStatusAsync_hook = Some(melee_link::cabi::c4(set_status_async));
        bc::CARDWriteAsync_hook = Some(melee_link::cabi::c5(write_async));
        bc::CARDWrite_hook = Some(melee_link::cabi::c4(write));
        // gw_wait_idle pumps the deferred queue
        melee_runtime::tu::lbcardgame::ext::hook::wait_idle = Some(pump);
        melee_runtime::tu::lbcardnew::ext::hook::wait_idle = Some(pump);
        melee_runtime::tu::lbcardgame::GC_IMAGE = true;
    }
}

/// The memory-card scene's hooks (gmscmemcard.c, gm_1ADD.c, gmmain_lib.c), after the link installed its defaults.
pub fn install_scene_hooks() {
    unsafe {
        // gw_SceneReport_Memcard / _Cursor: the prompt's state in the log (how a headless run is checked)
        melee_modes::tu::gmscmemcard::ext::SCENEREPORT_MEMCARD = Some(|d, o| {
            static mut LAST: (i32, i32) = (-1, -1);
            if unsafe { LAST } == (d, o) {
                return;
            }
            unsafe { LAST = (d, o) };
            let wait = matches!(d, 2 | 3 | 5 | 11);
            eprintln!("scene: cursor memcard decision={d} option={o} ({}) -- {}", if o == 0 { "Yes/left" } else { "No/right" }, if wait { "WAITING FOR A BUTTON" } else { "working" });
        });
        // the memory-card scene's (gmscmemcard.c, gm_1ADD.c) PC-side callees
        {
            use melee_modes::tu::{gm_1ADD::ext as g1, gmscmemcard::ext as mc};
            use melee_runtime::tu::controller as pad;
            mc::HSD_PADCOPYSTATUS0_BUTTON = Some(|| unsafe { pad::HSD_PadCopyStatus[0].button });
            mc::HSD_PADCOPYSTATUS0_TRIGGER = Some(|| unsafe { pad::HSD_PadCopyStatus[0].trigger });
            // gw_Settings_Int: settings.cfg beside the exe, MELEE_<KEY> wins
            mc::SETTINGS_INT = Some(|k, d| melee_data::settings::Settings::open_default().get_int(k, d));
            g1::HSD_PADCOPYSTATUS_TRIGGER = Some(|i| unsafe { pad::HSD_PadCopyStatus[i].trigger });
            g1::HSD_PADMASTERSTATUS_ERR = Some(|i| unsafe { pad::HSD_PadMasterStatus[i].err });
            g1::HSD_GOBJ_JOBJKIND = Some(|| unsafe { melee_runtime::tu::gobj_native::HSD_GObj_JObjKind });
            g1::HSD_GOBJ_CAMERAKIND = Some(|| unsafe { melee_runtime::tu::gobj_native::HSD_GObj_CameraKind });
            // gmMainLib_8015FA34 (after the load): the debug-level branch is off at the retail DbLevel
            melee_modes::tu::gmmain_lib::ext::DBLKIND_DEBUGDEVELOP = Some(|| melee_types::DbLKind_DebugDevelop as i32);
            melee_modes::tu::gmmain_lib::ext::DB_804D6B20 = Some(|| false);
            g1::HSD_SISLIB_803A62A0 = Some(|a, file, sym| {
                let f = std::ffi::CString::new(file).unwrap();
                let s = std::ffi::CString::new(sym).unwrap();
                unsafe { melee_runtime::tu::sislib::HSD_SisLib_803A62A0(a, f.as_ptr() as _, s.as_ptr() as _) };
            });
            g1::LBARCHIVE_80016DBC = Some(|file, out, sym, _end| {
                let f = std::ffi::CString::new(file).unwrap();
                let s = std::ffi::CString::new(sym).unwrap();
                unsafe {
                    melee_runtime::tu::lbarchive::lbArchive_80016DBC(f.as_ptr(), &[out as *mut c_void, s.as_ptr() as *mut c_void, core::ptr::null_mut()]);
                }
            });
        }
        melee_modes::tu::gmscmemcard::ext::SCENEREPORT_CURSOR = Some(|w, a, b| eprintln!("scene: cursor {w} {a} {b}"));
    }
}
