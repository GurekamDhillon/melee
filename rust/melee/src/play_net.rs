//! `melee play` netplay, replays and an in-play SyncTest (docs/rust-port/notes/net.md "netplay in melee play").
//!
//! * [`ModesGame`]: the retail mode loop's engine under the rollback session (`melee_net::session::RbGame`). The session
//!   governs the VS match scene only; the frame counter is the scene's (`Engine::frames`, 0 at the scene's start), so
//!   `frame()` is `FIRST - 1` when the match scene begins, as `gw_Replay_Frame` once the match is armed.
//! * [`Replay`]: the native replay format (`.mrpl`): the play options that set the boot up, then one record per engine
//!   tick from the boot on (the four pads the engine read, and the neutral hash after the tick when it was a match
//!   frame). A playback feeds the same pads to the same boot and compares every hash.
//! * [`NetDriver`]: one peer of a 2-player rollback match over UDP (`melee_net::app::room` direct connect, the same
//!   HELLO / ACCEPT / scene blob as `melee netplay` and the C build's gw_netplay.c), wired into the retail flow.
//!
//! Memory rule of a rollback run: a snapshot load rewrites the game arena, so nothing play keeps across ticks may live
//! there. The newest frame's draw list is copied to the system heap after each frame and the arena original freed inside
//! a `GameScope`; the arena trim and the GX decode thread are off (`prepare_rollback_run`).
use melee_engine::snapshot::{Base, SnapRef};
use melee_engine::Engine;
use melee_net::pad::RbInput;
use melee_net::session::{RbGame, FIRST, SLOTS};
use melee_types::PADStatus;
use std::cell::RefCell;

fn log(msg: &str) {
    eprintln!("play: {msg}");
}

/// The scene kind of the VS match (`GS_VS`, scene_name "vs").
pub const SCENE_VS: u8 = 2;

/// Allocator / renderer modes a run that snapshots the engine needs (and a replay of such a run repeats).
pub fn prepare_rollback_run() {
    melee_engine::arena::set_trim(false);
    // the GX decode thread would hold arena blocks across a load
    std::env::set_var("MELEE_GX_SYNC", "1");
    // the sound is game state (a static, its buffers in the arena) a load restores: the mix stays on the game thread
    // (the mixer thread would hold the sound's lock while a snapshot copies it)
    std::env::set_var("MELEE_AUDIO_GAME_THREAD", "1");
}

// ------------------------------------------------------------------------------------------------ frames

/// A copy of a drawn frame that shares nothing with the game arena: every Vec on the system heap and every `Arc` (the
/// pipeline configs, texture / TLUT / array data) re-created, so the presenter's clones and drops never touch an
/// ArcInner a snapshot load rewrites. Call outside the game scope; the original is freed inside it.
pub fn detach_frame(f: Vec<melee_gx::draw::DrawCmd>) -> Vec<melee_gx::draw::DrawCmd> {
    Detacher::default().detach(f)
}

/// [`detach_frame`] with the copies kept from frame to frame, by content. The renderer caches its decoded GPU
/// textures by the data `Arc`'s address: fresh copies every frame made every texture a cache miss, ~100 new GPU
/// textures a frame kept alive until the 600-frame eviction (60k textures, ~2.7 GB of video memory per windowed
/// rollback peer: issue #42, the device was lost with a few games open). Identical bytes now detach to the same
/// `Arc`, so the texture cache hits as it does offline.
///
/// Rollback rule: this cache must be owned by a value on the system heap (it is a field of [`ModesGame`] / the
/// retail flow), never a static: a snapshot load rewrites every static, so a static map's header would be put back
/// pointing at buckets the system heap has since freed or reused.
#[derive(Default)]
pub struct Detacher {
    datas: std::collections::HashMap<(u64, u64, usize), (melee_gx::resources::GxData, u64)>,
    frame: u64,
}

/// A 128-bit content hash (two independent multiply-xorshift lanes over 8-byte words).
fn hash128(b: &[u8]) -> (u64, u64) {
    let (mut h1, mut h2) = (0x9E37_79B9_7F4A_7C15u64 ^ b.len() as u64, 0xC2B2_AE3D_27D4_EB4Fu64);
    let mut ch = b.chunks_exact(8);
    for c in &mut ch {
        let w = u64::from_le_bytes(c.try_into().unwrap());
        h1 = (h1 ^ w).wrapping_mul(0xFF51_AFD7_ED55_8CCD).rotate_left(29);
        h2 = (h2 ^ w.rotate_left(32)).wrapping_mul(0xC4CE_B9FE_1A85_EC53).rotate_left(31);
    }
    let mut t = [0u8; 8];
    t[..ch.remainder().len()].copy_from_slice(ch.remainder());
    let w = u64::from_le_bytes(t);
    h1 = (h1 ^ w).wrapping_mul(0xFF51_AFD7_ED55_8CCD);
    h2 = (h2 ^ w).wrapping_mul(0xC4CE_B9FE_1A85_EC53);
    (h1 ^ (h1 >> 33), h2 ^ (h2 >> 29))
}

/// Detached copies not used for this many frames are dropped.
const DETACH_KEEP: u64 = 120;

impl Detacher {
    /// [`detach_frame`]. Call outside the game scope; the original is freed inside it.
    pub fn detach(&mut self, f: Vec<melee_gx::draw::DrawCmd>) -> Vec<melee_gx::draw::DrawCmd> {
        use std::collections::HashMap;
        use std::sync::Arc;
        debug_assert!(!melee_engine::arena::contains(self as *const Self as *const u8), "Detacher must not live in the arena");
        self.frame += 1;
        let now = self.frame;
        let mut cfgs: HashMap<usize, Arc<melee_gx::shader::PipelineConfig>> = HashMap::new();
        // per frame by the arena Arc's address (stable within the frame), across frames by content
        let mut seen: HashMap<usize, melee_gx::resources::GxData> = HashMap::new();
        let datas = &mut self.datas;
        let mut data = |d: &melee_gx::resources::GxData| -> melee_gx::resources::GxData {
            seen.entry(Arc::as_ptr(d) as *const u8 as usize)
                .or_insert_with(|| {
                    let (a, b) = hash128(d);
                    let e = datas.entry((a, b, d.len())).or_insert_with(|| (Arc::from(&d[..]), now));
                    e.1 = now;
                    e.0.clone()
                })
                .clone()
        };
        let out: Vec<melee_gx::draw::DrawCmd> = f
            .iter()
            .map(|c| {
                let mut n = c.clone();
                n.config = cfgs.entry(Arc::as_ptr(&c.config) as usize).or_insert_with(|| Arc::new((*c.config).clone())).clone();
                for a in n.arrays.iter_mut().flatten() {
                    a.data = data(&a.data);
                }
                for t in n.textures.iter_mut() {
                    if let Some(d) = &t.data {
                        t.data = Some(data(d));
                    }
                }
                for t in n.tluts.iter_mut() {
                    if let Some(d) = &t.data {
                        t.data = Some(data(d));
                    }
                }
                n
            })
            .collect();
        if now % DETACH_KEEP == 0 {
            self.datas.retain(|_, e| now - e.1 < DETACH_KEEP);
        }
        let _s = melee_engine::arena::GameScope::enter();
        drop(f);
        drop(melee_engine::gxbridge::shim().gx.packed.take());
        out
    }
}

// ------------------------------------------------------------------------------------------------ pads

pub fn pad_of(i: &Option<RbInput>, port: usize) -> PADStatus {
    let mut st = PADStatus::zeroed();
    match i {
        Some(i) => {
            st.button = i.buttons as u16;
            st.stickX = i.raw[0];
            st.stickY = i.raw[1];
            st.substickX = i.raw[2];
            st.substickY = i.raw[3];
            st.triggerLeft = i.pad_l;
            st.triggerRight = i.pad_r;
            st.analogA = i.pad_a;
            st.analogB = i.pad_b;
        }
        // the netplay ports are 1 and 2; 3 and 4 are unplugged
        None => st.err = if port < 2 { 0 } else { -1 },
    }
    st
}

pub fn input_of(p: &PADStatus) -> RbInput {
    RbInput {
        present: 1,
        is_raw: 1,
        confirmed: 1,
        buttons: p.button as u32,
        raw: [p.stickX, p.stickY, p.substickX, p.substickY],
        pad_l: p.triggerLeft,
        pad_r: p.triggerRight,
        pad_a: p.analogA,
        pad_b: p.analogB,
        ..RbInput::default()
    }
}

/// The pads a netplay run feeds outside the session (before the match scene): neutral, ports 1-2 plugged.
pub fn neutral_pads() -> [PADStatus; 4] {
    let mut p = [PADStatus::zeroed(); 4];
    p[2].err = -1;
    p[3].err = -1;
    p
}

// ------------------------------------------------------------------------------------------------ the game

pub struct ModesGame {
    pub e: Engine,
    base: RefCell<Option<Base>>,
    /// `Engine::frames` when the session took the engine (its frame `FIRST - 1`)
    f0: u32,
    /// the newest drawn frame, still in the arena; taken (detached) before the next snapshot load can happen
    raw: Option<Vec<melee_gx::draw::DrawCmd>>,
    detach: Detacher,
}

#[derive(Clone)]
pub struct ModesSnap {
    mem: SnapRef,
    frames: u32,
    ended: bool,
}

impl ModesGame {
    pub fn new(e: Engine) -> Self {
        ModesGame { f0: e.frames, e, base: RefCell::new(None), raw: None, detach: Detacher::default() }
    }
    fn with_base<R>(&self, f: impl FnOnce(&Base) -> R) -> R {
        if self.base.borrow().is_none() {
            *self.base.borrow_mut() = Some(Base::capture());
        }
        f(self.base.borrow().as_ref().unwrap())
    }
    /// Give the engine back (the snapshot base is released: its copy-on-write watch ends).
    pub fn into_engine(mut self) -> (Engine, Option<Vec<melee_gx::draw::DrawCmd>>) {
        let f = self.take_frame();
        drop(self.base.into_inner());
        (self.e, f)
    }
    /// One engine tick with these pads; the frame it drew is kept on the system heap.
    pub fn step(&mut self, pads: [PADStatus; 4]) {
        self.e.set_pads(Some(pads));
        self.e.tick();
        let f = {
            let _s = melee_engine::arena::GameScope::enter();
            melee_engine::gxbridge::take_frame()
        };
        if let Some(f) = f {
            // only the newest frame of a tick is shown: detach (copy) it lazily, at the end of the tick
            if let Some(old) = self.raw.replace(f) {
                let _s = melee_engine::arena::GameScope::enter();
                drop(old);
            }
        }
    }

    /// The tick's newest frame as a detached copy (after the session's frames of this tick).
    pub fn take_frame(&mut self) -> Option<Vec<melee_gx::draw::DrawCmd>> {
        self.raw.take().map(|f| self.detach.detach(f))
    }
}

impl RbGame for ModesGame {
    type Snapshot = ModesSnap;
    fn frame(&self) -> i32 {
        self.e.frames as i32 - self.f0 as i32 - 1 + FIRST
    }
    fn live(&self) -> bool {
        true
    }
    fn port_human(&self, port: usize) -> bool {
        port < 2
    }
    fn peek_input(&self, _slot: usize, _frame: i32) -> Option<RbInput> {
        None
    }
    fn save(&self) -> ModesSnap {
        ModesSnap { mem: self.with_base(|b| b.save()), frames: self.e.frames, ended: self.e.ended }
    }
    fn load(&mut self, s: &ModesSnap) {
        // the held frame's blocks belong to the timeline being discarded: the load decides what they are now
        std::mem::forget(self.raw.take());
        self.with_base(|b| b.load(&s.mem));
        self.e.frames = s.frames;
        self.e.ended = s.ended;
    }
    fn state_hash(&self) -> u32 {
        self.with_base(|b| b.page_hash())
    }
    fn game_hash(&self) -> u32 {
        self.e.neutral_row()[0]
    }
    fn run_frame(&mut self, inputs: &[Option<RbInput>; SLOTS], resim: bool) {
        if self.e.ended {
            return;
        }
        let mut p = [PADStatus::zeroed(); 4];
        for (port, slot) in p.iter_mut().enumerate() {
            *slot = pad_of(&inputs[port * 2], port);
        }
        // a resimulated frame is never shown: its render pass runs (the game state it touches stays the same as a
        // shown frame's) but the display lists are not decoded (gw_gx_suppress_draws, as the C build's resim)
        melee_engine::gxbridge::shim().set_suppress_draws(resim);
        // MELEE_NP_TEST_DESYNC=<game frame>: this peer's logic RNG is perturbed before that frame (the desync check's test)
        static TEST_DESYNC: std::sync::OnceLock<Option<i32>> = std::sync::OnceLock::new();
        if let Some(f) = *TEST_DESYNC.get_or_init(|| std::env::var("MELEE_NP_TEST_DESYNC").ok().and_then(|v| v.parse().ok())) {
            if self.frame() + 1 - FIRST == f {
                let rs = melee_runtime::tu::random::global::state();
                rs.set_logic_seed(0x0BAD_5EED);
            }
        }
        self.step(p);
        melee_engine::gxbridge::shim().set_suppress_draws(false);
    }
}

// ------------------------------------------------------------------------------------------------ replay

/// The native replay file (`.mrpl`), little-endian:
/// ```text
/// "MRPL" u16 version (1) u16 flags (bit 0: rollback allocator modes)
/// u32 header length, header: UTF-8 lines "key=value" (args = the play options, one per line as "arg=<word>";
///     seed, build, note)
/// u32 record count, then per engine tick: 4 x pad (u16 button, i8 stick x/y, i8 c-stick x/y, u8 L, u8 R,
///     u8 analog A, u8 analog B, i8 err, u8 0, u32 extButton) = 64 bytes, then u32 hash (neutral hash | 1 after a
///     match frame, 0 for a tick outside the match scene)
/// ```
#[derive(Default, Clone)]
pub struct Replay {
    pub flags: u16,
    pub args: Vec<String>,
    pub meta: Vec<(String, String)>,
    pub ticks: Vec<([PADStatus; 4], u32)>,
}

const PAD_BYTES: usize = 16;
const REC_BYTES: usize = 4 * PAD_BYTES + 4;

impl Replay {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut h = String::new();
        for a in &self.args {
            h += &format!("arg={a}\n");
        }
        for (k, v) in &self.meta {
            h += &format!("{k}={v}\n");
        }
        let mut o = Vec::with_capacity(16 + h.len() + self.ticks.len() * REC_BYTES);
        o.extend_from_slice(b"MRPL");
        o.extend_from_slice(&1u16.to_le_bytes());
        o.extend_from_slice(&self.flags.to_le_bytes());
        o.extend_from_slice(&(h.len() as u32).to_le_bytes());
        o.extend_from_slice(h.as_bytes());
        o.extend_from_slice(&(self.ticks.len() as u32).to_le_bytes());
        for (pads, hash) in &self.ticks {
            for p in pads {
                o.extend_from_slice(&p.button.to_le_bytes());
                o.extend_from_slice(&[p.stickX as u8, p.stickY as u8, p.substickX as u8, p.substickY as u8, p.triggerLeft, p.triggerRight, p.analogA, p.analogB, p.err as u8, 0]);
                o.extend_from_slice(&p.extButton.to_le_bytes());
            }
            o.extend_from_slice(&hash.to_le_bytes());
        }
        o
    }

    pub fn parse(d: &[u8]) -> Result<Replay, String> {
        let u16at = |o: usize| d.get(o..o + 2).map(|b| u16::from_le_bytes([b[0], b[1]])).ok_or("truncated");
        let u32at = |o: usize| d.get(o..o + 4).map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]])).ok_or("truncated");
        if d.get(..4) != Some(b"MRPL") {
            return Err("not a melee replay (MRPL)".into());
        }
        let ver = u16at(4)?;
        if ver != 1 {
            return Err(format!("replay version {ver} (this build reads 1)"));
        }
        let mut r = Replay { flags: u16at(6)?, ..Default::default() };
        let hl = u32at(8)? as usize;
        let h = std::str::from_utf8(d.get(12..12 + hl).ok_or("truncated header")?).map_err(|_| "header is not UTF-8")?;
        for l in h.lines() {
            if let Some((k, v)) = l.split_once('=') {
                if k == "arg" {
                    r.args.push(v.to_string());
                } else {
                    r.meta.push((k.to_string(), v.to_string()));
                }
            }
        }
        let mut o = 12 + hl;
        let n = u32at(o)? as usize;
        o += 4;
        if d.len() < o + n * REC_BYTES {
            return Err(format!("replay truncated: {n} records announced, {} present", (d.len() - o) / REC_BYTES));
        }
        r.ticks.reserve(n);
        for _ in 0..n {
            let mut pads = [PADStatus::zeroed(); 4];
            for p in pads.iter_mut() {
                let b = &d[o..o + PAD_BYTES];
                p.button = u16::from_le_bytes([b[0], b[1]]);
                p.stickX = b[2] as i8;
                p.stickY = b[3] as i8;
                p.substickX = b[4] as i8;
                p.substickY = b[5] as i8;
                p.triggerLeft = b[6];
                p.triggerRight = b[7];
                p.analogA = b[8];
                p.analogB = b[9];
                p.err = b[10] as i8;
                p.extButton = u32::from_le_bytes([b[12], b[13], b[14], b[15]]);
                o += PAD_BYTES;
            }
            let hash = u32::from_le_bytes([d[o], d[o + 1], d[o + 2], d[o + 3]]);
            o += 4;
            r.ticks.push((pads, hash));
        }
        Ok(r)
    }

    pub fn meta(&self, k: &str) -> Option<&str> {
        self.meta.iter().find(|m| m.0 == k).map(|m| m.1.as_str())
    }

    pub fn write(&self, path: &str) -> Result<(), String> {
        if let Some(d) = std::path::Path::new(path).parent().filter(|d| !d.as_os_str().is_empty()) {
            std::fs::create_dir_all(d).map_err(|e| format!("{}: {e}", d.display()))?;
        }
        std::fs::write(path, self.to_bytes()).map_err(|e| format!("{path}: {e}"))
    }
}

thread_local! {
    /// `--slp` on a run that rolls back (netplay): the .slp is made from the `--record` file once it is written.
    pub static SLP_AFTER: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
}

/// A rollback run's `.slp`: play its `.mrpl` back headless in a child process (the playback runs linearly, so the
/// exporter's hooks see each frame once) with `--slp`.
pub fn slp_from_replay(mrpl: &str, iso: &str) {
    let Some(spec) = SLP_AFTER.with(|c| c.borrow_mut().take()) else { return };
    let Ok(exe) = std::env::current_exe() else { return };
    log(&format!("slp: exporting {mrpl} to {spec} (headless playback)"));
    let st = std::process::Command::new(exe)
        .args(["play", "--iso", iso, "--replay", mrpl, "--headless", "--no-audio", "--slp", &spec])
        .env("MELEE_WINDOW_UNFOCUSED", "1")
        .status();
    match st {
        Ok(s) if s.success() => log("slp: export done"),
        Ok(s) => log(&format!("slp: export playback failed ({s})")),
        Err(e) => log(&format!("slp: {e}")),
    }
}

/// The play options a replay keeps (they decide the boot: intro, presets, the seeded match).
pub fn replay_args(args: &[String]) -> Vec<String> {
    // a netplay match is set up by its agreed scene (the np_scene header line), not by the peer's own --match
    let netplay = args.iter().any(|a| a == "--netplay");
    let mut out = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let a = &args[i];
        match a.as_str() {
            "--skip-intro" | "--teams" => out.push(a.clone()),
            "--match" if netplay => i += 1,
            "--stocks" | "--time-secs" | "--items" | "--match" | "--mods" => {
                if let Some(v) = args.get(i + 1) {
                    out.push(a.clone());
                    out.push(v.clone());
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }
    out
}

/// Playback state: the file's records, fed one per engine tick.
pub struct ReplayFeed {
    pub r: Replay,
    pub at: usize,
    pub hashed: u32,
    pub bad: u32,
    pub first_bad: Option<(usize, u32, u32)>,
    pub last_hash: u32,
}

impl ReplayFeed {
    pub fn new(r: Replay) -> Self {
        ReplayFeed { r, at: 0, hashed: 0, bad: 0, first_bad: None, last_hash: 0 }
    }
    pub fn check(&mut self, i: usize, got: u32) {
        let want = self.r.ticks[i].1;
        if want != 0 || got != 0 {
            self.hashed += 1;
            self.last_hash = got;
            if want != got {
                self.bad += 1;
                if self.first_bad.is_none() {
                    self.first_bad = Some((i, want, got));
                    log(&format!("replay: hash mismatch at tick {i}: file {want:08X}, playback {got:08X}"));
                }
            }
        }
    }
    pub fn summary(&self) -> String {
        format!(
            "replay: {} ticks played, {} match frames hashed, {} mismatches{}, final hash {:08X} (file {:08X})",
            self.at,
            self.hashed,
            self.bad,
            self.first_bad.map_or(String::new(), |b| format!(" (first at tick {})", b.0)),
            self.last_hash,
            self.r.ticks.iter().rev().find(|t| t.1 != 0).map_or(0, |t| t.1)
        )
    }
}

// ------------------------------------------------------------------------------------------------ netplay

use melee_net::app::room::{Phase, Room, RoomEnv, RoomSettings};
use melee_net::app::scene::{parse_scene, ParsedScene};
use melee_net::net::Net;
use melee_net::rollback::{NetAdapter, NetLog};
use melee_net::session::{run_tick, InputSource, RbConfig, RbSession, SyncTest};
use melee_net::transport::{Transport, UdpTransport};
use std::rc::Rc;
use std::time::{Duration, Instant};

/// This install's identities for the direct room: every VS fighter (`ck:N` = the external character id the
/// scenario launcher takes) and every VS stage (`ext:N`). The handshake's "global data" word is the mounted mod set
/// (`mods_hash`, described by `mods_desc`); the engine build is the separate exe word (`build_hash`).
#[derive(Clone, Default)]
pub struct PlayEnv {
    pub mods_hash: u64,
    pub mods_desc: String,
}

/// The engine build both peers must share (the sim is compared bit for bit): the git commit this exe was built from.
pub fn build_hash() -> u64 {
    if let Some(h) = std::env::var("MELEE_NP_BUILD_HASH").ok().and_then(|v| u64::from_str_radix(&v, 16).ok()) {
        return h; // test hook: pretend to be another build
    }
    let h = u64::from_str_radix(&env!("MELEE_GIT_HASH").chars().filter(|c| c.is_ascii_hexdigit()).take(15).collect::<String>(), 16).unwrap_or(0);
    if h == 0 {
        1
    } else {
        h
    }
}

/// The mounted mod set (`--mods DIR [--enable ..]`, else settings / MELEE_MODS_DIR) as the handshake's digest and
/// description: 0 / "no mods" for vanilla.
pub fn mods_identity(args: &[String]) -> Result<(u64, String), String> {
    let get = |k: &str| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned();
    let m = match get("--mods") {
        Some(dir) => {
            let en: Option<Vec<String>> = get("--enable").map(|s| s.split(',').map(str::to_string).collect());
            crate::mods::Mods::open(std::path::Path::new(&dir), en.as_deref()).map(Some)?
        }
        None => crate::mods::Mods::from_env()?,
    };
    let wasm = wasm_mods_identity();
    let Some(m) = m.filter(|m| !m.active.is_empty()) else {
        return Ok(match wasm {
            None => (0, "no mods".into()),
            Some(w) => (melee_net::hash64(format!("|wasm {w}").as_bytes(), 0), format!("code mods {w}")),
        });
    };
    let ids: Vec<&str> = m.active.iter().map(|m| m.id.as_str()).collect();
    let fp = m.set.fingerprint();
    let w = wasm.map(|w| format!("|wasm {w}")).unwrap_or_default();
    let h = melee_net::hash64(format!("{}|{fp:016x}{w}", ids.join(",")).as_bytes(), 0);
    Ok((h, format!("mods {} ({fp:016x}){}", ids.join(", "), w.replace("|wasm ", "; code mods "))))
}

/// The wasm code mods `MELEE_WASM_MODS` will load (notes/modhost.md), as "file#fnv64, ..." sorted by file name; None when
/// off. Both peers must run the same code mods, so they enter the handshake digest (read from the files before boot).
fn wasm_mods_identity() -> Option<String> {
    let dir = melee_modhost::dir_from_env()?;
    let mut v: Vec<String> = std::fs::read_dir(&dir).ok()?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x.eq_ignore_ascii_case("wasm")))
        .filter_map(|p| {
            let b = std::fs::read(&p).ok()?;
            let h = b.iter().fold(0xcbf2_9ce4_8422_2325u64, |h, &x| (h ^ x as u64).wrapping_mul(0x100_0000_01b3));
            Some(format!("{}#{:08x}", p.file_stem()?.to_string_lossy(), h as u32))
        })
        .collect();
    v.sort();
    (!v.is_empty()).then(|| v.join(", "))
}

impl melee_net::app::lobby::LobbyEnv for PlayEnv {
    fn stage_available(&self, ext: i32) -> bool {
        (1..=32).contains(&ext)
    }
    fn all_stages(&self) -> Vec<i32> {
        vec![31, 32]
    }
    fn peer_ready(&self) -> bool {
        true
    }
    fn token_for_ext(&self, ext: i32) -> String {
        format!("ext:{ext}")
    }
    fn ext_for_token(&self, t: &str) -> i32 {
        t.strip_prefix("ext:").and_then(|n| n.parse().ok()).unwrap_or(-1)
    }
}

impl RoomEnv for PlayEnv {
    fn token_for_ck(&self, ck: i32) -> String {
        format!("ck:{ck}")
    }
    fn ck_for_token(&self, tok: &str) -> i32 {
        tok.strip_prefix("ck:").and_then(|n| n.parse().ok()).unwrap_or(-1)
    }
    fn global_hash(&self) -> u64 {
        self.mods_hash
    }
    fn global_describe(&self) -> String {
        self.mods_desc.clone()
    }
    fn scene_check(&self, p: &ParsedScene) -> Result<(), String> {
        for t in [&p.host_tok, &p.guest_tok] {
            if !(0..=25).contains(&self.ck_for_token(t)) {
                return Err(format!("fighter {t} is not a VS fighter"));
            }
        }
        if !(1..=32).contains(&melee_net::app::lobby::LobbyEnv::ext_for_token(self, &p.stage_tok)) {
            return Err(format!("stage {} is not a VS stage", p.stage_tok));
        }
        Ok(())
    }
}

/// The host's rules as the agreed scene carries them (`;np_rules=s<stocks>,t<secs>,i<items>`, `-` = the default):
/// the guest adopts these, so the two need not pass the same options.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HostRules {
    pub stocks: Option<i32>,
    pub time_secs: Option<i32>,
    /// the `--items` text (`_` for spaces)
    pub items: Option<String>,
}

impl HostRules {
    pub fn from_args(args: &[String]) -> HostRules {
        HostRules {
            stocks: arg(args, "--stocks").and_then(|s| s.parse().ok()),
            time_secs: arg(args, "--time-secs").and_then(|s| s.parse().ok()),
            items: arg(args, "--items").map(|s| s.trim().replace(' ', "_")),
        }
    }
    pub fn encode(&self) -> String {
        let o = |v: &Option<String>| v.clone().unwrap_or_else(|| "-".into());
        format!(";np_rules=s{},t{},i{}", o(&self.stocks.map(|x| x.to_string())), o(&self.time_secs.map(|x| x.to_string())), o(&self.items))
    }
    pub fn from_scene(p: &ParsedScene) -> Option<HostRules> {
        let v = p.extras.iter().find_map(|e| e.strip_prefix("np_rules="))?;
        let mut r = HostRules::default();
        for f in v.split(',') {
            let (k, x) = f.split_at(1.min(f.len()));
            if x == "-" || x.is_empty() {
                continue;
            }
            match k {
                "s" => r.stocks = x.parse().ok(),
                "t" => r.time_secs = x.parse().ok(),
                "i" => r.items = Some(x.to_string()),
                _ => {}
            }
        }
        Some(r)
    }
    pub fn describe(&self) -> String {
        format!(
            "stocks {}, time {}, items {}",
            self.stocks.map_or("default".into(), |x| x.to_string()),
            self.time_secs.map_or("default".into(), |x| format!("{x} s")),
            self.items.clone().unwrap_or_else(|| "default".into())
        )
    }
}

/// A netem-like simulated network on the inbound side of the socket (`--np-lag-ms`, `--np-jitter-ms`,
/// `--np-loss-pct`, `--np-dup-pct`, `--np-reorder-pct`): each datagram is dropped, delayed by lag + a random part of
/// the jitter, held back an extra lag/2 + 10 ms (reordered behind its successors) or delivered twice.
pub struct Degraded {
    pub inner: Box<dyn Transport>,
    pub lag: Duration,
    pub jitter: Duration,
    pub loss_pct: u32,
    pub dup_pct: u32,
    pub reorder_pct: u32,
    pub rng: u32,
    pub q: Vec<(Instant, melee_net::transport::Addr, Vec<u8>)>,
    pub stats: [u64; 4],
}

impl Degraded {
    pub fn new(inner: Box<dyn Transport>, n: &Netem, seed: u32) -> Degraded {
        Degraded { inner, lag: n.lag, jitter: n.jitter, loss_pct: n.loss, dup_pct: n.dup, reorder_pct: n.reorder, rng: seed | 1, q: Vec::new(), stats: [0; 4] }
    }
    fn roll(&mut self) -> u32 {
        self.rng = self.rng.wrapping_mul(1664525).wrapping_add(1013904223);
        self.rng >> 8
    }
    fn delay(&mut self) -> Duration {
        let j = self.jitter.as_micros() as u32;
        let extra = if j > 0 { self.roll() % (j + 1) } else { 0 };
        self.lag + Duration::from_micros(extra as u64)
    }
}

/// The simulated network's settings.
#[derive(Clone, Copy, Debug, Default)]
pub struct Netem {
    pub lag: Duration,
    pub jitter: Duration,
    pub loss: u32,
    pub dup: u32,
    pub reorder: u32,
}

impl Netem {
    pub fn from_args(args: &[String]) -> Netem {
        let num = |k: &str| arg(args, k).and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);
        Netem {
            lag: Duration::from_millis(num("--np-lag-ms")),
            jitter: Duration::from_millis(num("--np-jitter-ms")),
            loss: num("--np-loss-pct").min(100) as u32,
            dup: num("--np-dup-pct").min(100) as u32,
            reorder: num("--np-reorder-pct").min(100) as u32,
        }
    }
}

impl Transport for Degraded {
    fn send(&mut self, to: &melee_net::transport::Addr, data: &[u8]) -> i32 {
        self.inner.send(to, data)
    }
    fn recv(&mut self, from: &mut melee_net::transport::Addr, buf: &mut [u8]) -> i32 {
        let mut tmp = [0u8; 2048];
        loop {
            let mut a = melee_net::transport::Addr::default();
            let n = self.inner.recv(&mut a, &mut tmp);
            if n <= 0 {
                break;
            }
            let d = tmp[..n as usize].to_vec();
            if self.loss_pct > 0 && self.roll() % 100 < self.loss_pct {
                self.stats[0] += 1;
                continue;
            }
            let mut at = Instant::now() + self.delay();
            if self.reorder_pct > 0 && self.roll() % 100 < self.reorder_pct {
                at += self.lag / 2 + Duration::from_millis(10);
                self.stats[1] += 1;
            }
            if self.dup_pct > 0 && self.roll() % 100 < self.dup_pct {
                let at2 = Instant::now() + self.delay();
                self.q.push((at2, a, d.clone()));
                self.stats[2] += 1;
            }
            self.q.push((at, a, d));
        }
        // the earliest due datagram (jitter and the reorder hold make the queue unordered)
        let now = Instant::now();
        let Some(i) = (0..self.q.len()).filter(|&i| self.q[i].0 <= now).min_by_key(|&i| self.q[i].0) else { return 0 };
        let (_, a, d) = self.q.remove(i);
        *from = a;
        let n = d.len().min(buf.len());
        buf[..n].copy_from_slice(&d[..n]);
        self.stats[3] += 1;
        n as i32
    }
    fn local_port(&self) -> u16 {
        self.inner.local_port()
    }
}

pub type Sess = RbSession<ModesGame>;

/// An agreed scene string as a scenario (port 1 the host's fighter, port 2 the guest's; a stock match with the time
/// limit of `minutes`).
pub fn scenario_of_scene(sc: &str) -> Result<melee_engine::scenario::Scenario, String> {
    use melee_engine::scenario::{PlayerSpec, Scenario};
    let p = parse_scene(sc).ok_or_else(|| format!("netplay scene does not parse: {sc}"))?;
    PlayEnv::default().scene_check(&p)?;
    let ck = |t: &str| PlayEnv::default().ck_for_token(t);
    let spec = |c: i32, color: i32| PlayerSpec { ckind: c, slot_type: 0, cpu_level: -1, color, team: -1, stocks: -1, handicap: -1, cpu_idle: false };
    let mut players = [None; 6];
    players[0] = Some(spec(ck(&p.host_tok), p.host_color));
    players[1] = Some(spec(ck(&p.guest_tok), p.guest_color));
    Ok(Scenario {
        stage: melee_net::app::lobby::LobbyEnv::ext_for_token(&PlayEnv::default(), &p.stage_tok),
        players,
        rule_match: 1,
        rule_minutes: -1,
        item_freq: -2,
        teams: 0,
        time_limit: HostRules::from_scene(&p).and_then(|r| r.time_secs).map_or(p.minutes * 60, |s| s.max(1)),
        rule_stocks: p.stocks,
        onep_mode: -1,
        onep_step: -1,
        onep_level: -1, game_mode: -1, entry_state: -1,
    })
}

/// The agreed match, as the room settled it.
pub struct Agreed {
    pub scene: ParsedScene,
    pub scene_str: String,
    pub seed: u32,
    pub delay: i32,
    pub host: bool,
}

pub enum NetPhase {
    /// Booting / loading: the engine runs with neutral pads until the match scene starts.
    PreMatch,
    /// The match scene under the session.
    InMatch(Box<ModesGame>),
    /// The match is over (or the session failed); the engine is back in `RetailPlay`.
    Done,
}

pub struct NetDriver {
    pub host: bool,
    pub sess: Rc<RefCell<Sess>>,
    pub log: Rc<RefCell<NetLog>>,
    pub room: Option<Room>,
    pub net: Option<Net>,
    pub agreed: Agreed,
    /// the host's rules (the guest adopts them)
    pub host_rules: HostRules,
    pub phase: NetPhase,
    pub started_at: Option<Instant>,
    /// final hashes by session frame (start-of-frame checksums, `RbSession::checksum`)
    pub finals: Vec<(i32, u32)>,
    pub hash_out: Option<String>,
    /// frames of the session already written to the replay
    pub rec_next: i32,
    /// the end of the match: when the scene ended (session frame) and since when we linger for the peer
    pub end_frame: Option<i32>,
    pub linger: u32,
    pub error: Option<String>,
    /// for the title line: rollback count a second ago, its delta, the deepest of the last second
    last_rb: (i32, i32, i32),
    pub ticks: u64,
    pub held_ticks: u64,
    /// `--np-css`: the menus' lockstep channel
    pub ls: Option<crate::np_lockstep::Lockstep>,
    params: RoomParams,
    /// matches played on this connection before the current one
    pub match_no: u32,
    /// the final hashes of the earlier matches
    pub all_finals: Vec<Vec<(i32, u32)>>,
}

/// What a room is opened with (kept for the rematches of `--np-css`).
pub struct RoomParams {
    host: bool,
    rcfg: RbConfig,
    timeout: u64,
    settings: RoomSettings,
    env: PlayEnv,
    /// the run's one socket (np_mux.rs): room 0, the lockstep, the rematch rooms
    mux: crate::np_mux::MuxRef,
    /// the rendezvous client below the mux (`--np-server`)
    rdv: Option<melee_net::app::rdv::RdvRef>,
    /// the guest's way to the host (direct address, or the server's introduction); the host learns the guest's
    peer: Option<melee_net::transport::Addr>,
}

/// Open a room on mux channel `ch` (HELLO / ACCEPT / the scene blob) and wait until it is connected. The first room
/// (`code`: Some for a match-code guest, "" for a match-code host) goes through the rendezvous server when there is one.
fn open_room(p: &mut RoomParams, ch: u32, code: Option<String>) -> Result<(Room, Rc<RefCell<Sess>>, Rc<RefCell<NetLog>>), String> {
    let sess: Rc<RefCell<Sess>> = Rc::new(RefCell::new(Sess::new(p.rcfg.clone())));
    let nlog = Rc::new(RefCell::new(NetLog::default()));
    let inner = Box::new(NetAdapter::new(sess.clone(), nlog.clone(), None));
    let tp: Box<dyn Transport> = Box::new(crate::np_mux::Chan { core: p.mux.clone(), ch });
    let host_port = tp.local_port();
    let mut settings = p.settings.clone();
    settings.peer = p.peer;
    let rdv = if code.is_some() { p.rdv.clone() } else { None };
    let room_code = code.filter(|c| !c.is_empty());
    let mut room = Room::begin(settings, Box::new(p.env.clone()), inner, tp, rdv, room_code, false, host_port)?;
    let tag = if p.host { "host" } else { "guest" };
    let t0 = Instant::now();
    let mut shown = String::new();
    loop {
        if t0.elapsed().as_secs() > p.timeout {
            let st = room.sh.borrow().status.clone();
            let hint = if p.host {
                "nobody connected; check that the guest has your address and port, that UDP to it is allowed through the firewall / forwarded by the router, or use a match code (--np-server)"
            } else {
                "the host did not answer; check the address and port, the host's firewall / port forward, or use a match code (--np-server)"
            };
            return Err(format!("netplay {tag}: no connection after {} s ({st}): {hint}", p.timeout));
        }
        match room.phase() {
            Phase::Working => {
                room.poll();
                let c = room.code();
                if p.host && !c.is_empty() && c != shown {
                    log(&format!("netplay host: MATCH CODE {c} (give it to the other player)"));
                    shown = c;
                }
                if let Some(r) = &p.rdv {
                    for m in r.borrow_mut().log.drain(..) {
                        log(&format!("netplay {tag}: {m}"));
                    }
                }
            }
            Phase::Connected => break,
            Phase::Lobby => return Err("netplay: the peer asked for a lobby (not offered by melee play)".into()),
            _ => return Err(format!("netplay {tag}: {}", room.sh.borrow().status)),
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    if let Some(r) = p.rdv.as_ref().filter(|_| ch == 0) {
        let r = r.borrow();
        if r.have_peer && p.peer.is_none() && !p.host {
            p.peer = Some(r.peer);
        }
        log(&format!("netplay {tag}: path {}", if r.relay { "RELAYED through the server" } else { "direct (hole punched)" }));
    }
    Ok((room, sess, nlog))
}

fn arg(args: &[String], k: &str) -> Option<String> {
    args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned()
}

/// A match code: 4 characters of the server's alphabet (dashes and case ignored).
fn looks_like_code(s: &str) -> bool {
    let c: String = s.chars().filter(|c| *c != '-').collect::<String>().to_ascii_uppercase();
    c.len() == 4 && c.chars().all(|ch| melee_net::app::rdv::NP_ALPHABET.contains(ch))
}

impl NetDriver {
    /// `--netplay host [--np-port P]` / `--netplay join ADDR[:PORT] | CODE`: connect (blocking, before the window) and
    /// agree the match. The host's `--match a,b,STAGE` gives its own fighter (the first) and the stage; the guest's
    /// `--match a,..` its own fighter (the first). The host's `--stocks N` / `--time-secs S` / `--items F` are the rules
    /// (the guest adopts them). `--np-server HOST[:PORT]` = match codes through a rendezvous server (melee_relay).
    pub fn connect(args: &[String], ck_of: &dyn Fn(&str) -> Option<i32>, stage_of: &dyn Fn(&str) -> Option<i32>) -> Result<NetDriver, String> {
        use melee_net::app::rdv::{Rdv, RdvTransport};
        let mode = arg(args, "--netplay").ok_or("--netplay host|join ADDR")?;
        let host = match mode.as_str() {
            "host" => true,
            "join" => false,
            m => return Err(format!("--netplay {m}: host or join")),
        };
        let target = if host { None } else { Some(args.iter().position(|a| a == "--netplay").and_then(|i| args.get(i + 2)).cloned().ok_or("--netplay join ADDR[:PORT] or CODE")?) };
        let num = |k: &str, d: i64| arg(args, k).and_then(|s| s.parse::<i64>().ok()).unwrap_or(d);
        let (delay, auto) = crate::np_hud::parse_delay(arg(args, "--np-delay").as_deref());
        crate::np_hud::AUTO_DELAY.store(auto, std::sync::atomic::Ordering::Relaxed);
        let maxb = num("--np-rollback", 7) as i32;
        let netem = Netem::from_args(args);
        let timeout = num("--np-timeout", 60) as u64;
        let server = arg(args, "--np-server").or_else(|| std::env::var("MELEE_NP_SERVER").ok().filter(|s| !s.is_empty()));
        // the own fighter: the first of --match (default Falcon / Fox); the host's stage: the --match stage (default FD)
        let spec = arg(args, "--match").unwrap_or_else(|| if host { "falcon,fox,32".into() } else { "fox".into() });
        let parts: Vec<&str> = spec.split(',').map(str::trim).collect();
        let ck = ck_of(parts[0]).ok_or_else(|| format!("--match: unknown fighter '{}'", parts[0]))?;
        let stage_ext = parts.iter().skip(1).find_map(|p| stage_of(p)).unwrap_or(32);
        let rules = HostRules::from_args(args);
        let stocks = rules.stocks.unwrap_or(4);
        let minutes = rules.time_secs.map_or(8, |s| (s.max(1) + 59) / 60);
        let local_port = if host { 0 } else { 1 };
        let tag = if host { "host" } else { "guest" };
        // where the guest goes: a match code (with a server) or an address (IPv4, [IPv6], host name; port 51500 default)
        let code = match (&target, &server) {
            (Some(t), Some(_)) if looks_like_code(t) => Some(t.chars().filter(|c| *c != '-').collect::<String>().to_ascii_uppercase()),
            (None, Some(_)) => Some(String::new()),
            _ => None,
        };
        let peer = match (&target, &code) {
            (Some(t), None) => Some(melee_net::addr_resolve(t, 51500, arg(args, "--np-bind").map_or(false, |b| b.contains(':'))).map_err(|e| format!("--netplay join {t}: {e}"))?),
            _ => None,
        };
        // the socket: all interfaces (IPv4 + IPv6) unless --np-bind; the host on --np-port (default 51500), the guest on
        // --np-port or an ephemeral port
        let bind_port = num("--np-port", if host { 51500 } else { 0 }) as u16;
        let udp = match arg(args, "--np-bind") {
            Some(b) => {
                let ip: std::net::IpAddr = b.trim_matches(['[', ']']).parse().map_err(|_| format!("--np-bind {b}: not an IP address"))?;
                UdpTransport::open_at(ip, bind_port).map_err(|e| format!("bind {b} port {bind_port}: {e}"))?
            }
            None => UdpTransport::open(0, bind_port).map_err(|e| format!("bind UDP port {bind_port}: {e} (is another copy running?)"))?,
        };
        let port = udp.local_port();
        let v6 = udp.has_v6();
        let deg: Box<dyn Transport> = Box::new(Degraded::new(Box::new(udp), &netem, 0x1234_5678 ^ local_port as u32));
        let (rdv, base): (Option<melee_net::app::rdv::RdvRef>, Box<dyn Transport>) = match &server {
            Some(sv) => {
                let r = Rdv::new(sv, deg)?;
                if arg(args, "--np-bind").map_or(false, |b| b.contains(':')) {
                    r.borrow_mut().lan_override = Some(0); // an IPv6-only socket: no IPv4 LAN address to offer
                }
                if let Some(l) = arg(args, "--np-lan") {
                    let a = melee_net::addr_parse(&l, 1).ok_or_else(|| format!("--np-lan {l}: an IPv4 address"))?;
                    r.borrow_mut().lan_override = Some(a.ip);
                }
                r.borrow_mut().force_relay = args.iter().any(|a| a == "--np-relay");
                log(&format!("netplay {tag}: rendezvous server {}", r.borrow().srv));
                (Some(r.clone()), Box::new(RdvTransport(r)))
            }
            None => (None, deg),
        };
        if host {
            log(&format!(
                "netplay: hosting on UDP port {port} ({}), waiting for the guest{}",
                match arg(args, "--np-bind") { Some(b) => format!("bound to {b}"), None => format!("all interfaces, {}", if v6 { "IPv4 and IPv6" } else { "IPv4" }) },
                if server.is_some() { " (match code from the server)" } else { "; the guest joins YOUR-PUBLIC-IP:PORT (UDP port forwarded) or YOUR-LAN-IP:PORT" }
            ));
        } else if let Some(p) = &peer {
            log(&format!("netplay: joining {p} from UDP port {port}"));
        } else {
            log(&format!("netplay: joining match code {} from UDP port {port}", code.clone().unwrap_or_default()));
        }
        let (mods_hash, mods_desc) = mods_identity(args)?;
        let env = PlayEnv { mods_hash, mods_desc: mods_desc.clone() };
        log(&format!("netplay {tag}: build {:015x}, {mods_desc}", build_hash()));
        let mut rp = RoomParams {
            host,
            rcfg: RbConfig { net: true, local_port, delay, maxb, src: InputSource::Live, ..RbConfig::default() },
            timeout,
            settings: RoomSettings {
                host,
                ck,
                color: num("--np-color", 0) as i32,
                stocks,
                minutes,
                delay,
                stage_ext,
                use_lobby: false,
                peer,
                name: arg(args, "--np-name").unwrap_or_default(),
                seed: arg(args, "--np-seed").and_then(|s| s.parse().ok()),
                exe_hash: build_hash(),
                scene_extra: if host { rules.encode() } else { String::new() },
                ..RoomSettings::default()
            },
            env,
            mux: crate::np_mux::new(base),
            rdv,
            peer,
        };
        let t0 = Instant::now();
        let (room, sess, nlog) = open_room(&mut rp, 0, code)?;
        let (scene_str, seed, adelay) = {
            let b = room.sh.borrow();
            (b.scene.clone(), b.seed, b.s.delay)
        };
        let scene = parse_scene(&scene_str).ok_or_else(|| format!("netplay: the agreed scene does not parse: {scene_str}"))?;
        log(&format!("netplay {tag}: match agreed after {:.2}s: \"{scene_str}\" (seed {seed}, delay {adelay}, rollback window {maxb})", t0.elapsed().as_secs_f32()));
        let host_rules = HostRules::from_scene(&scene).unwrap_or_default();
        if !host {
            if rules != HostRules::default() && rules != host_rules {
                log(&format!("netplay guest: the host's rules win: {} (yours: {})", host_rules.describe(), rules.describe()));
            } else {
                log(&format!("netplay guest: the host's rules: {}", host_rules.describe()));
            }
        }
        sess.borrow_mut().set_delay(adelay);
        // --np-css: the character / stage select and the results run in lockstep (mux channel 1), and every match after
        // the first is a new room on mux channel 2 + k
        let css = args.iter().any(|a| a == "--np-css");
        let ls = css.then(|| {
            let tp: Box<dyn Transport> = Box::new(crate::np_mux::Chan { core: rp.mux.clone(), ch: 1 });
            crate::np_lockstep::Lockstep::new(tp, rp.peer, host, adelay.max(1) as u32)
        });
        Ok(NetDriver {
            host,
            sess,
            log: nlog,
            room: Some(room),
            net: None,
            agreed: Agreed { scene, scene_str: scene_str.clone(), seed, delay: adelay, host },
            host_rules,
            phase: NetPhase::PreMatch,
            started_at: None,
            finals: Vec::new(),
            hash_out: arg(args, "--np-hash-out"),
            rec_next: FIRST,
            end_frame: None,
            linger: 0,
            error: None,
            last_rb: (0, 0, 0),
            ticks: 0,
            held_ticks: 0,
            ls,
            params: rp,
            match_no: 0,
            all_finals: Vec::new(),
        })
    }

    /// `--np-css`: the menus run in lockstep (np_lockstep.rs).
    pub fn css(&self) -> bool {
        self.ls.is_some()
    }

    /// The match after the first (`--np-css`): a new room (and session) on the next mux channel of the same socket; both
    /// peers get here on the same lockstep frame (the VS scene's first frame). Blocking, like the first connection.
    pub fn next_room(&mut self) -> Result<(), String> {
        self.match_no += 1;
        if self.match_no > 250 {
            return Err("netplay: too many matches on one connection".into());
        }
        if self.host && crate::np_hud::AUTO_DELAY.load(std::sync::atomic::Ordering::Relaxed) {
            // --np-delay auto: this match's delay from the ping of the last one (the guest adopts the host's)
            let d = crate::np_hud::auto_delay(self.net.as_ref().map_or(0, |n| n.rtt_ms()));
            log(&format!("netplay host: auto delay {d} for match {}", self.match_no + 1));
            self.params.settings.delay = d;
            self.params.rcfg.delay = d;
        }
        let t0 = Instant::now();
        let (room, sess, nlog) = open_room(&mut self.params, 2 + self.match_no, None)?;
        let adelay = room.sh.borrow().s.delay;
        sess.borrow_mut().set_delay(adelay);
        log(&format!("netplay {}: match {} connected after {:.2}s", self.tag(), self.match_no + 1, t0.elapsed().as_secs_f32()));
        self.all_finals.push(std::mem::take(&mut self.finals));
        self.sess = sess;
        self.log = nlog;
        self.room = Some(room);
        self.net = None;
        self.started_at = None;
        self.rec_next = FIRST;
        self.end_frame = None;
        self.linger = 0;
        self.last_rb = (0, 0, 0);
        Ok(())
    }

    pub fn scene_string(&self) -> String {
        self.agreed.scene_str.clone()
    }

    /// The match scene began: hand the session the game, release the room (loaded, ready).
    pub fn begin_match(&mut self, e: Engine) {
        self.sess.borrow_mut().scene_begin(SCENE_VS as i32);
        if let Some(mut room) = self.room.take() {
            room.release();
            self.net = room.net.take();
        }
        self.phase = NetPhase::InMatch(Box::new(ModesGame::new(e)));
        log(&format!("netplay {}: match scene, waiting for the peer", self.tag()));
    }

    pub fn tag(&self) -> &'static str {
        if self.host { "host" } else { "guest" }
    }

    fn fail(&mut self, msg: String) {
        if self.error.is_none() {
            log(&format!("netplay {}: ERROR: {msg}", self.tag()));
            self.error = Some(msg);
        }
    }

    /// One render tick in the match: the network, the local pad, the session's frames. Returns false while the session
    /// holds (not started yet).
    pub fn tick_match(&mut self, local: &PADStatus, rec: Option<&mut Replay>) -> bool {
        self.ticks += 1;
        let NetPhase::InMatch(g) = &mut self.phase else { return false };
        let Some(net) = self.net.as_mut() else {
            self.error = Some("no connection".into());
            return false;
        };
        NetAdapter::tick(net, &self.sess);
        if matches!(net.state(), melee_net::net::State::Refused | melee_net::net::State::Dead) {
            let r = net.last_reason().to_string();
            if self.end_frame.is_none() {
                self.fail(format!("connection lost: {r} (the other player quit, or the network dropped)"));
            }
            return false;
        }
        if !net.started() {
            self.held_ticks += 1;
            return false;
        }
        if self.started_at.is_none() {
            self.started_at = Some(Instant::now());
            if !self.host {
                if let Some(rc) = net.remote_config() {
                    self.sess.borrow_mut().set_delay(rc.input_delay as i32);
                }
            }
            log(&format!("netplay {}: started (delay {})", if self.host { "host" } else { "guest" }, self.sess.borrow().cfg.delay));
        }
        let mut s = self.sess.borrow_mut();
        let p = local;
        s.pad_latch(0, p.button as i32, p.stickX as i32, p.stickY as i32, p.substickX as i32, p.substickY as i32, p.triggerLeft as i32, p.triggerRight as i32, p.analogA as i32, p.analogB as i32, 0);
        run_tick(&mut s, g, None);
        // final checksums (start of frame f = the state after frame f - 1)
        let next = self.finals.last().map_or(FIRST, |x| x.0 + 1);
        for fr in next..=s.current_frame() {
            let h = s.checksum(fr);
            if h == 0 {
                break;
            }
            self.finals.push((fr, h));
        }
        // the replay: confirmed frames with their final inputs and the hash after them
        if let Some(rec) = rec {
            while self.rec_next < s.current_frame() {
                let fr = self.rec_next;
                let h = s.checksum(fr + 1);
                if h == 0 || fr > s.confirmed_frame() {
                    break;
                }
                let mut pads = [PADStatus::zeroed(); 4];
                for (port, pd) in pads.iter_mut().enumerate() {
                    *pd = pad_of(&if port < 2 { s.input_any(port, fr).copied() } else { None }, port);
                }
                rec.ticks.push((pads, h));
                self.rec_next += 1;
            }
        }
        let lg = self.log.borrow();
        if let Some(&(f, a, b)) = lg.desyncs.first() {
            let m = format!("DESYNC at frame {} (game frame {}): local hash {a:08X}, peer hash {b:08X}", f, f - FIRST);
            drop(lg);
            drop(s);
            self.fail(m);
            return true;
        }
        if s.stats.desyncs > 0 {
            drop(lg);
            drop(s);
            self.fail("DESYNC: a correction's snapshot was gone (rolled back past the window)".into());
            return true;
        }
        drop(lg);
        // the match's end: the scene ended on a frame every input of which is confirmed
        if g.e.ended {
            let endf = g.frame();
            if s.confirmed_frame() >= endf && self.finals.last().map_or(false, |x| x.0 >= endf) {
                self.end_frame.get_or_insert(endf);
            }
        }
        true
    }

    /// The match is over: the frames the replay has not taken yet, through the one the scene ended on (it has no
    /// start-of-next-frame checksum, so `tick_match` never records it), each with the hash after it; `last` is the
    /// engine's hash after the end frame.
    pub fn flush_rec(&mut self, rec: &mut Replay, last: u32) {
        let Some(endf) = self.end_frame else { return };
        let s = self.sess.borrow();
        while self.rec_next <= endf {
            let fr = self.rec_next;
            let h = if fr == endf { last } else { s.checksum(fr + 1) };
            let mut pads = [PADStatus::zeroed(); 4];
            for (port, pd) in pads.iter_mut().enumerate() {
                *pd = pad_of(&if port < 2 { s.input_any(port, fr).copied() } else { None }, port);
            }
            rec.ticks.push((pads, h));
            self.rec_next += 1;
        }
    }

    /// Ticks to keep polling after the end (the peer still needs our last inputs and checksums).
    pub fn lingering(&mut self) -> bool {
        if self.end_frame.is_some() {
            self.linger += 1;
            return self.linger < 90;
        }
        false
    }

    /// A read-only snapshot for the HUD overlay (np_hud.rs).
    pub fn hud(&self) -> crate::np_hud::NetHud {
        let s = self.sess.borrow();
        let st = s.stats;
        let net = self.net.as_ref().or_else(|| self.room.as_ref().and_then(|r| r.net.as_ref()));
        let phase = if self.error.is_some() {
            "error"
        } else if self.end_frame.is_some() {
            "match over"
        } else if self.started_at.is_some() {
            "match"
        } else if matches!(self.phase, NetPhase::InMatch(_)) {
            "connecting"
        } else {
            "menus"
        };
        crate::np_hud::NetHud {
            host: self.host,
            ping_ms: net.map_or(0, |n| n.rtt_ms()),
            silent_ms: net.map_or(0, |n| n.silent_ms()),
            delay: s.cfg.delay,
            auto: crate::np_hud::AUTO_DELAY.load(std::sync::atomic::Ordering::Relaxed),
            rollbacks: st.rollbacks,
            resim: st.resim,
            depth_max: st.depth_max,
            stall_ticks: st.stall_ticks,
            phase,
            frame: self.started_at.map(|_| s.current_frame() - FIRST),
            lockstep: self.ls.as_ref().filter(|_| phase == "menus").map(|l| (l.frame, l.stalls)),
            error: self.error.clone(),
        }
    }

    /// The MELEE_SHOW_FPS title line part.
    pub fn status(&mut self) -> String {
        let s = self.sess.borrow();
        let st = s.stats;
        let ping = self.net.as_ref().map_or(0, |n| n.rtt_ms());
        let d = st.rollbacks - self.last_rb.0;
        self.last_rb = (st.rollbacks, d, st.depth_max);
        let state = if let Some(e) = &self.error {
            format!("ERROR {}", e.chars().take(48).collect::<String>())
        } else if self.started_at.is_none() {
            "connecting".into()
        } else if self.end_frame.is_some() {
            "match over".into()
        } else {
            format!("frame {}", s.current_frame() - FIRST)
        };
        format!("netplay {} | ping {ping} ms | delay {} | rollbacks {d}/s (total {}, max {} frames) | {state}", self.tag(), s.cfg.delay, st.rollbacks, st.depth_max)
    }

    pub fn summary(&self) -> String {
        let s = self.sess.borrow();
        let st = s.stats;
        let lg = self.log.borrow();
        format!(
            "netplay {}: {} final hashes (through game frame {}), rollbacks {} (max depth {}, resim {}), stalls {}, waits {}, session desyncs {}, checksum desyncs {}, ping {} ms; net {:?}",
            self.tag(),
            self.finals.len(),
            self.finals.last().map_or(-1, |x| x.0 - 1 - FIRST),
            st.rollbacks,
            st.depth_max,
            st.resim,
            st.stall_ticks,
            st.waits,
            st.desyncs,
            lg.desyncs.len(),
            self.net.as_ref().map_or(0, |n| n.rtt_ms()),
            self.net.as_ref().map(|n| n.stats())
        )
    }

    pub fn write_hashes(&self) {
        if let Some(p) = &self.hash_out {
            let mut t = String::from("frame,hash\n");
            // match k > 0 of an --np-css connection: its frames are keyed k * 1000000 + frame
            for (k, fin) in self.all_finals.iter().chain(std::iter::once(&self.finals)).enumerate() {
                for &(f, h) in fin {
                    t += &format!("{},{:08X}\n", k as i64 * 1_000_000 + (f - 1 - FIRST) as i64, h);
                }
            }
            if let Err(e) = std::fs::write(p, t) {
                log(&format!("{p}: {e}"));
            }
        }
    }
}

// ------------------------------------------------------------------------------------------------ synctest

/// `MELEE_PLAY_SYNCTEST=K`: every match frame of `melee play` runs GGPO's SyncTest (save, step, roll back K frames,
/// resimulate, compare the whole-state and neutral hashes). The local pads drive the frame.
pub struct PlaySync {
    pub st: SyncTest<ModesGame>,
    pub g: Option<Box<ModesGame>>,
}

impl PlaySync {
    pub fn tick(&mut self, pads: [PADStatus; 4]) {
        let Some(g) = self.g.as_mut() else { return };
        if g.e.ended {
            return;
        }
        let mut a = [None; SLOTS];
        for p in 0..4 {
            if pads[p].err == 0 {
                a[p * 2] = Some(input_of(&pads[p]));
            }
        }
        self.st.tick(g, &mut |_| a);
    }
}

/// A failure of the run (netplay error, replay divergence) for the exit code after the window loop.
pub static RUN_FAILED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
