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
//! Core v1.1 (the same tables, appended to; the header's `MU_CORE_V1_1_*_SIZE`): with a v1.1 host the mix goes to
//! `audio_out` (32 kHz, captured per tick), and the game fills `state_save` / `state_load` / `state_drop` /
//! `state_hash` / `state_frame` / `state_input` (a VS match runs under retail_flow's `Hosted` session). Either side may
//! be v1: a v1 host gets a silent game whose states are never asked for; a v1 game simply leaves the fields zero.
//! The host's memory card (`card_*`) backs the CARD calls when `MELEE_CARD` is not 0; `stop` ends the run.
//!
//! Not offered: the settings overlay and UI mods (they draw with our own renderer). `MELEE_COREV1_FORCE_V1=1`
//! makes the library answer as a v1 game (tests). Extra arguments for `melee play` come from
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
    card: melee_engine::card_host::HostCard,
    native_alloc: extern "C" fn(u32) -> *mut c_void,
    native_free: extern "C" fn(*mut c_void),
    // core v1.1: read only when the host's struct_size reaches it
    audio_out: Option<extern "C" fn(*const i16, u32, u32)>,
}

#[repr(C)]
pub struct MuGameCoreV1 {
    abi: u32,
    struct_size: u32,
    run: Option<extern "C" fn() -> i32>,
    retrace: Option<extern "C" fn()>,
    fire_alarms: Option<extern "C" fn(u64)>,
    ai_dma_done: Option<extern "C" fn()>,
    // core v1.1
    state_save: Option<extern "C" fn(i32) -> i32>,
    state_load: Option<extern "C" fn(i32) -> i32>,
    state_drop: Option<extern "C" fn(i32)>,
    state_hash: Option<extern "C" fn() -> u32>,
    state_frame: Option<extern "C" fn() -> i32>,
    state_input: Option<extern "C" fn(i32, *const MuCoreV1Pad)>,
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
    assert!(std::mem::offset_of!(MuHostCoreV1, audio_out) == V1_HOST);
    assert!(std::mem::size_of::<MuHostCoreV1>() == V11_HOST);
    assert!(std::mem::offset_of!(MuGameCoreV1, run) == 8);
    assert!(std::mem::offset_of!(MuGameCoreV1, ai_dma_done) == 32);
    assert!(std::mem::offset_of!(MuGameCoreV1, state_save) == V1_GAME);
    assert!(std::mem::offset_of!(MuGameCoreV1, state_input) == 80);
    assert!(std::mem::size_of::<MuGameCoreV1>() == V11_GAME);
};
/// The table sizes of core v1 and v1.1 (`MU_CORE_V1_HOST_SIZE` ...).
const V1_HOST: usize = 240;
const V1_GAME: usize = 40;
const V11_HOST: usize = 248;
const V11_GAME: usize = 88;

const ABI: u32 = 1;
static mut HOST: Option<&'static MuHostCoreV1> = None;
/// The host filled the v1.1 fields (`audio_out`).
static mut HOST_V11: bool = false;
/// We filled the v1.1 fields (the state calls).
static mut GAME_V11: bool = false;
/// The retail flow while `drive` runs it: the state calls act on its hosted match between ticks.
static mut RP: *mut crate::retail_flow::RetailPlay = std::ptr::null_mut();

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
    if h.abi != ABI || (h.struct_size as usize) < V1_HOST {
        return 1;
    }
    if (g.struct_size as usize) < V1_GAME {
        return 1;
    }
    let force_v1 = std::env::var("MELEE_COREV1_FORCE_V1").is_ok_and(|v| v == "1");
    HOST = Some(&*(h as *const MuHostCoreV1));
    HOST_V11 = !force_v1 && h.struct_size as usize >= V11_HOST && h.audio_out.is_some();
    g.abi = ABI;
    g.run = Some(run);
    g.retrace = Some(retrace);
    g.fire_alarms = Some(fire_alarms);
    g.ai_dma_done = None;
    // the v1.1 calls only into the room the host gave (its struct_size in)
    let room = g.struct_size as usize;
    g.struct_size = V1_GAME as u32;
    if !force_v1 && room >= V11_GAME {
        g.state_save = Some(state_save);
        g.state_load = Some(state_load);
        g.state_drop = Some(state_drop);
        g.state_hash = Some(state_hash);
        g.state_frame = Some(state_frame);
        g.state_input = Some(state_input);
        g.struct_size = V11_GAME as u32;
        GAME_V11 = true;
    }
    let _ = melee_engine::card_host::HOST.set(&*(&h.card as *const melee_engine::card_host::HostCard));
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
    let (hv11, gv11) = unsafe { (HOST_V11, GAME_V11) };
    hlog(&format!(
        "engine {} ({}), host API {}, core v1 host table {} bytes ({}), game table {}",
        env!("MELEE_GIT_HASH"),
        env!("MELEE_GIT_WHERE"),
        h.host_version,
        h.struct_size,
        if hv11 { "v1.1: audio_out" } else { "v1" },
        if gv11 { "v1.1: savestates" } else { "v1" }
    ));
    if hv11 {
        crate::audio::HOST_MIX.store(true, std::sync::atomic::Ordering::Relaxed);
    }
    if gv11 && std::env::var_os("MELEE_PLAY_SYNCTEST").is_none() {
        crate::retail_flow::HOSTED_STATES.store(true, std::sync::atomic::Ordering::Relaxed);
    }
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
    // the transcoder's allocations stay off the game arena (snapshot loads rewrite it)
    let snapshots = (gv11 && std::env::var_os("MELEE_COREV1_FORCE_V1").is_none()) || std::env::var_os("MELEE_PLAY_SYNCTEST").is_some();
    // a run that loads snapshots: the transcoder holds none of the game's buffers (melee_gx corev1 ROLLBACK)
    melee_gx::corev1::ROLLBACK.store(snapshots, std::sync::atomic::Ordering::Relaxed);
    let _ = melee_gx::corev1::HOST_HEAP.set(|on| {
        let prev = melee_engine::arena::active();
        melee_engine::arena::set_active(on);
        prev
    });
    let t = unsafe { melee_gx::corev1::Transcoder::new(mem, len, Box::new(move |b: &[u8]| submit(b.as_ptr(), b.len() as u32))) };
    melee_gx::corev1::install(t);
    hlog(&format!("MEM1 {} MB at {mem:p}", len >> 20));
    if std::env::var("MELEE_COREV1_CARD_TEST").is_ok_and(|v| v == "1") {
        hlog(&card_test(&h.card));
    }
    let extra = std::env::var("MELEE_COREV1_ARGS").unwrap_or_else(|_| "--skip-intro".into());
    let mut args: Vec<String> = vec!["--iso".into(), "core-v1-host-disc".into()];
    if !hv11 {
        // a v1 host has nowhere to play sound
        args.push("--no-audio".into());
    }
    args.extend(extra.split_whitespace().map(str::to_string));
    hlog(&format!("melee play {}", args[2..].join(" ")));
    let code = crate::play::play(&args);
    if let Some(s) = melee_gx::corev1::stats_line() {
        hlog(&s);
    }
    let ok = code == std::process::ExitCode::SUCCESS;
    hlog(&format!("play returned {}", if ok { "success" } else { "failure" }));
    // MU_STOP_EXIT with the exit code: the host ends the program as the console's OSResetSystem path would
    (h.stop)(0, i32::from(!ok));
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
    let audio_out = if unsafe { HOST_V11 } { h.audio_out } else { None };
    // MELEE_COREV1_WAV=<path>: what went to audio_out, as a WAV (the mix check against our own headless capture)
    let wav_path = std::env::var("MELEE_COREV1_WAV").ok();
    let mut wav: Vec<i16> = Vec::new();
    let (mut sent, tb0) = (0u64, (h.ticks)());
    unsafe { RP = rp as *mut _ };
    while n < limit && !rp.should_quit() {
        let t = std::time::Instant::now();
        rp.tick(&mut pads);
        tick_ns += t.elapsed().as_nanos();
        n += 1;
        if let Some(out) = audio_out {
            let mix = rp.take_audio();
            if !mix.is_empty() {
                out(mix.as_ptr(), (mix.len() / 2) as u32, 32000);
                sent += mix.len() as u64 / 2;
                if wav_path.is_some() {
                    wav.extend_from_slice(&mix);
                }
            }
        }
        // the state calls (inside vi_wait_retrace) act between ticks
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
    unsafe { RP = std::ptr::null_mut() };
    let s = t0.elapsed().as_secs_f64();
    // the host's clock (the console timebase, 40.5 MHz) over the same run
    let host_s = ((h.ticks)() - tb0) as f64 / 40_500_000.0;
    hlog(&format!("{n} ticks in {s:.1} s ({:.1} fps; host clock {host_s:.1} s), engine tick avg {:.2} ms", n as f64 / s, tick_ns as f64 / 1e6 / n.max(1) as f64));
    if audio_out.is_some() {
        hlog(&format!("audio_out: {sent} frames at 32 kHz ({:.1} s)", sent as f64 / 32000.0));
    }
    if let Some(p) = wav_path {
        if let Err(e) = crate::write_wav(&p, &wav) {
            hlog(&format!("{p}: {e}"));
        }
    }
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

// ------------------------------------------------------------------------------------------------ core v1.1 states
// The host calls these on the game thread from inside vi_wait_retrace, between two ticks of `drive`.

fn hosted() -> Option<&'static mut crate::retail_flow::Hosted> {
    unsafe { RP.as_mut().and_then(|rp| rp.hosted_g.as_mut()) }
}
extern "C" fn state_save(key: i32) -> i32 {
    hosted().map_or(0, |h| i32::from(h.save(key)))
}
extern "C" fn state_load(key: i32) -> i32 {
    hosted().map_or(0, |h| i32::from(h.load(key)))
}
extern "C" fn state_drop(key: i32) {
    if let Some(h) = hosted() {
        h.drop_key(key);
    }
}
extern "C" fn state_hash() -> u32 {
    hosted().map_or(0, |h| h.hash())
}
extern "C" fn state_frame() -> i32 {
    hosted().map_or(-1, |h| h.next_frame())
}
extern "C" fn state_input(frame: i32, pads: *const MuCoreV1Pad) {
    let (Some(h), Some(p)) = (hosted(), unsafe { pads.cast::<[MuCoreV1Pad; 4]>().as_ref() }) else { return };
    let mut four = [melee_types::PADStatus::zeroed(); 4];
    for (s, q) in four.iter_mut().zip(p) {
        *s = pad_status(q);
    }
    h.set_input(frame, four);
}

fn pad_status(q: &MuCoreV1Pad) -> melee_types::PADStatus {
    let mut s = melee_types::PADStatus::zeroed();
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
    s
}

/// `MELEE_COREV1_CARD_TEST=1` (tests): the host's card calls as our CARD shim uses them, on slot A: probe, mount,
/// create, write, read back, stat, rename, delete, unmount. Run it with the host's card folder pointed at a scratch
/// folder (`--card-dir`): it creates and deletes a file there.
fn card_test(c: &melee_engine::card_host::HostCard) -> String {
    let name = CString::new("corev11_card_test").unwrap();
    let other = CString::new("corev11_card_test2").unwrap();
    let (mut mem, mut sector) = (0, 0);
    let probe = (c.probe)(0, &mut mem, &mut sector);
    let mount = (c.mount)(0);
    let mut no = -1;
    let create = (c.create)(0, name.as_ptr(), 0x4000, &mut no);
    let data: Vec<u8> = (0..0x4000u32).map(|i| (i * 7 + 3) as u8).collect();
    let write = (c.write)(0, no, data.as_ptr().cast(), data.len() as u32, 0);
    let mut back = vec![0u8; data.len()];
    let read = (c.read)(0, no, back.as_mut_ptr().cast(), back.len() as u32, 0);
    let mut st = melee_types::CARDStat::zeroed();
    let stat = (c.stat)(0, no, (&mut st as *mut melee_types::CARDStat).cast(), 0x6C);
    let close = (c.close)(0, no);
    let rename = (c.rename)(0, name.as_ptr(), other.as_ptr());
    let (mut no2, mut len2) = (-1, 0u32);
    let reopen = (c.open)(0, other.as_ptr(), &mut no2, &mut len2);
    let delete = (c.delete)(0, other.as_ptr());
    let gone = (c.open)(0, other.as_ptr(), &mut no2, &mut len2);
    let unmount = (c.unmount)(0);
    let ok = probe == 0 && mount == 0 && create == 0 && write == 0 && read == 0 && back == data && stat == 0 && st.length as usize >= data.len()
        && rename == 0 && reopen == 0 && delete == 0 && gone != 0 && unmount == 0;
    format!(
        "card test {}: probe {probe} ({mem} Mbit, sector {sector}), mount {mount}, create {create} (file {no}), write {write}, read {read} ({}), stat {stat} (length {}), close {close}, rename {rename}, reopen {reopen} (length {len2}), delete {delete}, open after delete {gone}, unmount {unmount}",
        if ok { "passed" } else { "FAILED" },
        if back == data { "same bytes" } else { "DIFFERENT bytes" },
        st.length
    )
}
