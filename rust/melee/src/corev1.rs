//! Core v1 (feature `corev1`, built only as the `melee_corev1` cdylib): this engine as a game library for a host
//! that speaks Hero88go's frozen game/host boundary, `mu_core_v1.h` (melee-unlocked `port/runtime/abi`). Written from
//! that header alone: the two tables below are its `MuHostCoreV1` / `MuGameCoreV1`, field for field.
//!
//! The host loads the library at 0x82800000 (melee_corev1's link arguments), calls `mu_game_entry_core_v1` with its
//! table, then `run()` on its game thread. `run` is `melee play` (the retail flow) with:
//! * the disc read through the host's `disc_read` (melee_engine `gcm::HOST_READ`; the `--iso` path is a placeholder),
//! * pads from the host's `pad_read` (melee_platform `HOST_PADS`) unless `MELEE_PAD_SCRIPT` drives them,
//! * every frame's GX stream transcoded to the console's (melee_gx `corev1`) and handed to `gx_fifo`, with the data
//!   it names staged in the host's MEM1,
//! * one `vi_wait_retrace()` per tick: the host paces the game, as the console's retrace did.
//!
//! Not offered over core v1: audio (the boundary has none), savestates / rollback (no state hooks), the settings
//! overlay and UI mods (they draw with our own renderer). Extra arguments for `melee play` come from
//! `MELEE_COREV1_ARGS` (whitespace-separated; default `--skip-intro`).
#![allow(non_snake_case, static_mut_refs)]

use std::ffi::{c_char, c_void, CString};

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct MuCoreV1Pad {
    button: u16,
    stick_x: i8,
    stick_y: i8,
    sub_x: i8,
    sub_y: i8,
    trigger_l: u8,
    trigger_r: u8,
    analog_a: u8,
    analog_b: u8,
    err: i8,
}

type DiscDone = extern "C" fn(i32, *mut c_void);

#[repr(C)]
pub struct MuHostCoreV1 {
    abi: u32,
    struct_size: u32,
    host_version: u32,
    reserved0: u32,
    log: extern "C" fn(*const c_char),
    poll: extern "C" fn(),
    ticks: extern "C" fn() -> u64,
    stop: extern "C" fn(i32, i32),
    gx_fifo: extern "C" fn(*const u8, u32),
    vi_wait_retrace: extern "C" fn(),
    mem1_base: extern "C" fn() -> *mut c_void,
    mem1_size: extern "C" fn() -> u32,
    pad_read: extern "C" fn(*mut MuCoreV1Pad),
    disc_entrynum: extern "C" fn(*const c_char) -> i32,
    disc_file: extern "C" fn(i32, *mut u32, *mut u32) -> i32,
    disc_read: extern "C" fn(u32, *mut c_void, u32, DiscDone, *mut c_void),
    card: [usize; 14],
    native_alloc: extern "C" fn(u32) -> *mut c_void,
    native_free: extern "C" fn(*mut c_void),
}

#[repr(C)]
pub struct MuGameCoreV1 {
    abi: u32,
    struct_size: u32,
    run: Option<extern "C" fn() -> i32>,
    retrace: Option<extern "C" fn()>,
    fire_alarms: Option<extern "C" fn(u64)>,
    ai_dma_done: Option<extern "C" fn()>,
}

// The header pins these for 64-bit targets.
const _: () = {
    assert!(std::mem::size_of::<MuCoreV1Pad>() == 12);
    assert!(std::mem::offset_of!(MuHostCoreV1, log) == 16);
    assert!(std::mem::offset_of!(MuHostCoreV1, gx_fifo) == 48);
    assert!(std::mem::offset_of!(MuHostCoreV1, mem1_base) == 64);
    assert!(std::mem::offset_of!(MuHostCoreV1, pad_read) == 80);
    assert!(std::mem::offset_of!(MuHostCoreV1, disc_read) == 104);
    assert!(std::mem::offset_of!(MuHostCoreV1, card) == 112);
    assert!(std::mem::offset_of!(MuHostCoreV1, native_alloc) == 224);
    assert!(std::mem::size_of::<MuHostCoreV1>() == 240);
    assert!(std::mem::offset_of!(MuGameCoreV1, run) == 8);
    assert!(std::mem::offset_of!(MuGameCoreV1, ai_dma_done) == 32);
    assert!(std::mem::size_of::<MuGameCoreV1>() == 40);
};

const ABI: u32 = 1;
static mut HOST: Option<&'static MuHostCoreV1> = None;

fn host() -> &'static MuHostCoreV1 {
    unsafe { HOST.expect("core v1: no host") }
}

/// A line in the host's log (and ours).
pub fn hlog(msg: &str) {
    eprintln!("corev1: {msg}");
    if let Ok(c) = CString::new(format!("rust core v1: {msg}")) {
        (host().log)(c.as_ptr());
    }
}

/// Is this process running the engine under a core v1 host?
pub fn active() -> bool {
    unsafe { HOST.is_some() }
}

#[no_mangle]
pub unsafe extern "C" fn mu_game_entry_core_v1(h: *const MuHostCoreV1, g: *mut MuGameCoreV1) -> i32 {
    let (Some(h), Some(g)) = (h.as_ref(), g.as_mut()) else { return 1 };
    // a host older than this table (a field we call lies past its struct_size) is refused
    if h.abi != ABI || (h.struct_size as usize) < std::mem::size_of::<MuHostCoreV1>() {
        return 1;
    }
    if (g.struct_size as usize) < std::mem::offset_of!(MuGameCoreV1, ai_dma_done) + 8 {
        return 1;
    }
    HOST = Some(&*(h as *const MuHostCoreV1));
    g.abi = ABI;
    g.run = Some(run);
    g.retrace = Some(retrace);
    g.fire_alarms = Some(fire_alarms);
    g.ai_dma_done = None;
    g.struct_size = std::mem::size_of::<MuGameCoreV1>() as u32;
    0
}

static RETRACES: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
extern "C" fn retrace() {
    RETRACES.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
}
/// The engine sets no OS alarms.
extern "C" fn fire_alarms(_now: u64) {}

struct Done {
    result: i32,
    done: bool,
}
extern "C" fn disc_done(result: i32, user: *mut c_void) {
    let d = unsafe { &mut *(user as *mut Done) };
    d.result = result;
    d.done = true;
}

/// `gcm::HOST_READ`: a raw disc read, waited for by letting the host's time pass.
fn host_read(off: u64, buf: &mut [u8]) -> Result<(), String> {
    let h = host();
    let mut at = 0usize;
    // in pieces: the console's reads were bounded, and a host may be too
    while at < buf.len() {
        let n = (buf.len() - at).min(8 << 20);
        let mut d = Done { result: 0, done: false };
        (h.disc_read)((off + at as u64) as u32, buf[at..].as_mut_ptr().cast(), n as u32, disc_done, (&mut d as *mut Done).cast());
        while !unsafe { std::ptr::read_volatile(&d.done) } {
            (h.poll)();
        }
        if d.result < 0 || (d.result as usize) < n {
            return Err(format!("core v1 disc_read @{:#x}+{n:#x}: {}", off + at as u64, d.result));
        }
        at += n;
    }
    Ok(())
}

/// `melee_platform::pad_read::HOST_PADS`: ports 1-4 from the host, the rest unplugged.
fn host_pads(st: &mut [melee_types::PADStatus; melee_platform::pad::PAD_CHANMAX]) {
    let mut p = [MuCoreV1Pad::default(); 4];
    (host().pad_read)(p.as_mut_ptr());
    for (i, s) in st.iter_mut().enumerate() {
        match p.get(i) {
            Some(q) => {
                s.button = q.button;
                s.stickX = q.stick_x;
                s.stickY = q.stick_y;
                s.substickX = q.sub_x;
                s.substickY = q.sub_y;
                s.triggerLeft = q.trigger_l;
                s.triggerRight = q.trigger_r;
                s.analogA = q.analog_a;
                s.analogB = q.analog_b;
                s.err = q.err;
            }
            None => s.err = -1,
        }
    }
}

extern "C" fn run() -> i32 {
    let h = host();
    hlog(&format!("engine {} ({}), host API {}", env!("MELEE_GIT_HASH"), env!("MELEE_GIT_WHERE"), h.host_version));
    // decode in the tick (the transcoder replaces the decoder) and no device of our own
    std::env::set_var("MELEE_GX_SYNC", "1");
    std::env::set_var("MELEE_PAD_IGNORE_ADAPTER", "1");
    let _ = melee_engine::gcm::HOST_READ.set(host_read);
    if std::env::var_os("MELEE_PAD_SCRIPT").map_or(true, |v| v.is_empty()) {
        let _ = melee_platform::pad_read::HOST_PADS.set(host_pads);
    } else {
        hlog("pads: MELEE_PAD_SCRIPT (the host's controllers are not read)");
    }
    let mem = (h.mem1_base)() as *mut u8;
    let len = (h.mem1_size)() as usize;
    let submit = h.gx_fifo;
    let t = unsafe { melee_gx::corev1::Transcoder::new(mem, len, Box::new(move |b: &[u8]| submit(b.as_ptr(), b.len() as u32))) };
    melee_gx::corev1::install(t);
    hlog(&format!("MEM1 {} MB at {mem:p}", len >> 20));
    let extra = std::env::var("MELEE_COREV1_ARGS").unwrap_or_else(|_| "--skip-intro".into());
    let mut args: Vec<String> = vec!["--iso".into(), "core-v1-host-disc".into(), "--no-audio".into()];
    args.extend(extra.split_whitespace().map(str::to_string));
    hlog(&format!("melee play {}", args[2..].join(" ")));
    let code = crate::play::play(&args);
    if let Some(s) = melee_gx::corev1::stats_line() {
        hlog(&s);
    }
    let ok = code == std::process::ExitCode::SUCCESS;
    hlog(&format!("play returned {}", if ok { "success" } else { "failure" }));
    i32::from(!ok)
}

/// `retail_flow::run` under a core v1 host: one tick, the frame to the host, one retrace, until the flow quits.
pub fn drive(rp: &mut crate::retail_flow::RetailPlay) {
    use melee_platform::Game;
    let h = host();
    let mut pads = melee_platform::pad_read::PadSystem::headless();
    let title = std::env::var("MELEE_WINDOW_TITLE").ok();
    let t0 = std::time::Instant::now();
    let (mut tick_ns, mut n, mut window_n) = (0u128, 0u64, std::time::Instant::now());
    let limit = rp.max_frames.unwrap_or(u64::MAX);
    while n < limit && !rp.should_quit() {
        let t = std::time::Instant::now();
        rp.tick(&mut pads);
        tick_ns += t.elapsed().as_nanos();
        n += 1;
        (h.vi_wait_retrace)();
        if let Some(tl) = title.as_deref() {
            if n % 60 == 1 {
                set_window_title(tl);
            }
        }
        if n % 600 == 0 {
            let w = window_n.elapsed().as_secs_f64();
            window_n = std::time::Instant::now();
            hlog(&format!("tick {n}: {:.1} fps over the last 600, engine tick avg {:.2} ms", 600.0 / w, tick_ns as f64 / 1e6 / n as f64));
        }
    }
    let s = t0.elapsed().as_secs_f64();
    hlog(&format!("{n} ticks in {s:.1} s ({:.1} fps), engine tick avg {:.2} ms", n as f64 / s, tick_ns as f64 / 1e6 / n.max(1) as f64));
}

/// The host owns the window; a test run's window still carries MELEE_WINDOW_TITLE (every visible window does).
fn set_window_title(title: &str) {
    #[cfg(windows)]
    unsafe {
        #[link(name = "user32")]
        extern "system" {
            fn EnumWindows(cb: extern "system" fn(isize, isize) -> i32, l: isize) -> i32;
            fn GetWindowThreadProcessId(w: isize, pid: *mut u32) -> u32;
            fn IsWindowVisible(w: isize) -> i32;
            fn SetWindowTextW(w: isize, s: *const u16) -> i32;
        }
        extern "system" fn each(w: isize, l: isize) -> i32 {
            let mut pid = 0u32;
            unsafe {
                GetWindowThreadProcessId(w, &mut pid);
                if pid == std::process::id() && IsWindowVisible(w) != 0 {
                    SetWindowTextW(w, l as *const u16);
                }
            }
            1
        }
        let wide: Vec<u16> = title.encode_utf16().chain(Some(0)).collect();
        EnumWindows(each, wide.as_ptr() as isize);
    }
}
