//! gd-ui-v1, the UI layer (docs/rust-port/notes/wasm-ui-lab-api.md part A, with the 2-player amendment and the
//! lab-ui-spec.md gaps GAP-0 and GAP-14).
//!
//! * A presentation-only wasm mod (it imports from `gd_ui`, declares UI feature bits and no hooks) gets its own wasmi
//!   `Store` here, owned by the play flow, never by the game arena: a savestate load or a rollback never rewinds a
//!   menu. The sim host skips such modules, so they are sim-inert.
//! * Per present ([`PresHost::present`], after the logic frame): pending scene changes, navigation from the captured
//!   pads, `gd_ui_event`s, `gd_ui_frame`, then the node trees; the result is one sorted [`melee_gx::ui::UiFrame`] that
//!   the presenter draws in the blit pass (one draw call).
//! * Input ([`PresHost::input`], before `set_pads` and the rollback submit): exclusive captures zero the captured ports
//!   for the game, injections replace a port (offline only), so a `--record` replay and a netplay peer see the result.
//! * With no UI mod nothing here is created ([`PresHost::from_env`] returns None).

pub mod cmd;
pub mod nodes;
pub mod theme;

use crate::pres::*;
use cmd::Batch;
use melee_gx::ui::{Atlas, AtlasRect};
use melee_types::PADStatus;
use std::cell::RefCell;
use std::rc::Rc;
use wasmi::{Caller, Engine, Extern, Linker, Memory, Module, Store, TypedFunc};

/// Fuel per presentation call (a trap or running out disables the mod's presentation instance).
pub const UI_FUEL: u64 = 4_000_000;
pub const MAX_UI_MODS: usize = 8;

pub const SESSION_OFFLINE: i32 = 0;
pub const SESSION_NETPLAY: i32 = 1;
pub const SESSION_SYNCTEST: i32 = 2;
pub const SESSION_REPLAY: i32 = 3;
pub const SESSION_LAB_NETPLAY: i32 = 4;

pub const CAP_EXCLUSIVE: u32 = 1;
pub const CAP_SHARED: u32 = 2;
pub const CAP_KEYBOARD: u32 = 4;
pub const CAP_INJECT: u32 = 8;

/// Keyboard events (`gd_ui_keys`): kind << 24 | value.
pub const KEY_DOWN: u32 = 1;
pub const KEY_UP: u32 = 2;
pub const KEY_CHAR: u32 = 3;
pub const KEY_ENTER: u32 = 1;
pub const KEY_ESCAPE: u32 = 2;
pub const KEY_BACKSPACE: u32 = 3;
pub const KEY_UP_ARROW: u32 = 4;
pub const KEY_DOWN_ARROW: u32 = 5;
pub const KEY_LEFT_ARROW: u32 = 6;
pub const KEY_RIGHT_ARROW: u32 = 7;
pub const KEY_TAB: u32 = 8;

/// `gd_ui_scene_goto` argument for a retail scene (`RETAIL | major << 8 | minor`).
pub const SCENE_RETAIL: u32 = 0x8000_0000;
/// `gd_ui_scene_replace_when` condition: an Envoy run is armed and has fought since arming.
pub const REPLACE_WHEN_ENVOY: u32 = 1;

pub fn session_code(s: Session) -> i32 {
    match s {
        Session::Offline => SESSION_OFFLINE,
        Session::Netplay => SESSION_NETPLAY,
        Session::Synctest => SESSION_SYNCTEST,
        Session::Replay => SESSION_REPLAY,
        Session::LabNetplay => SESSION_LAB_NETPLAY,
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Capture {
    pub token: u32,
    pub module: u16,
    pub ports: u32,
    pub flags: u32,
    /// Released: dropped at the next frame boundary.
    pub released: bool,
}

pub struct SceneDef {
    pub module: u16,
    pub name: String,
}

/// Host state the `gd_ui` imports of every presentation store share.
pub struct Shared {
    /// A host-native modal (the settings overlay) owns the pads: gd_ui_pad answers GD_ESTATE.
    pub native_modal: bool,
    /// Nanoseconds spent in gd_ui_submit (parse + encode) this present (the stage timers).
    pub submit_ns: u64,
    pub atlas: Atlas,
    pub batch: Batch,
    pub tree: nodes::Tree,
    pub theme: i32,
    pub session: Session,
    /// (mode, scene kind) of the retail scene, and whether it is a match.
    pub retail: (i32, i32),
    pub in_match: bool,
    pub caps: Vec<Capture>,
    next_token: u32,
    /// This present's pads before any mask (the game's clamp already applied by PADRead), and the new presses.
    pub pads: [PADStatus; 4],
    pub prev_buttons: [u16; 4],
    pub edges: [u16; 4],
    pub inject: [Option<PADStatus>; 4],
    /// Keyboard events of this present (only filled while a KEYBOARD capture exists).
    pub keys: Vec<u32>,
    pub scenes: Vec<SceneDef>,
    pub replace: Vec<((i32, i32), u32)>,
    /// `gd_ui_scene_replace_when` rows: (retail scene, custom scene, condition); entered only while the condition holds.
    pub replace_when: Vec<((i32, i32), u32, u32)>,
    /// The Envoy camp scene (`gd_ui_envoy_door`): opened by the main menu's Envoy row and the browser's Envoy mode.
    pub door: u32,
    /// Custom scenes on top of the retail flow, innermost last.
    pub stack: Vec<u32>,
    /// A goto / back for the next frame boundary: Some(0) pops everything (back to retail), Some(-1) pops one.
    pending: Option<i64>,
    /// Per module: loaded textures (handle h = index + 1) and the package dir.
    pub texs: Vec<Vec<AtlasRect>>,
    /// gd_ui_model_tex: (module, model, handle) already rasterized.
    pub model_texs: Vec<(usize, i32, u32)>,
    pub dirs: Vec<std::path::PathBuf>,
    pub features: Vec<u32>,
    /// Window size and the 4:3 band rect in window pixels (GAP-0: `gd_ui_band_rect`).
    pub window: (u32, u32),
    pub present_no: u32,
    pub log: Vec<String>,
    /// gd-ui-v1.2 (UI_HOST): the host catalogs the play flow fills (records of NUL-separated UTF-8 fields) and
    /// their generation (bumped on every replace, so a mod re-reads only after a change).
    pub lists: [Vec<Vec<u8>>; LIST_KINDS],
    pub list_gen: [u16; LIST_KINDS],
    /// Requests of this present for the play flow (`gd_ui_request`), drained by [`PresHost::take_requests`].
    pub requests: Vec<Request>,
    /// gd-ui-v1.3 host pictures (`gd_ui_host_pic`): a fixed pool of atlas slots, recycled least recently used.
    pub pics: Pics,
}

/// gd-ui-v1.3 host pictures: PIC_SLOTS fixed atlas rects of PIC_W x PIC_H (allocated at the first ask, never freed:
/// the atlas has no free), the key each holds, decoded RGBA the play flow offered (bounded), and this present's
/// asks for keys not on hand (the play flow fetches those off the game thread).
#[derive(Default)]
pub struct Pics {
    pub slots: Vec<(AtlasRect, u32, u32)>,
    pub base: Vec<(u16, u32)>,
    pub ready: std::collections::VecDeque<(u32, std::sync::Arc<Vec<u8>>)>,
    pub wants: Vec<u32>,
    pub uploads: u32,
}
pub const PIC_W: u16 = 96;
pub const PIC_H: u16 = 72;
pub const PIC_SLOTS: usize = 16;
pub const PIC_READY_MAX: usize = 48;
pub const PIC_UPLOADS_PER_PRESENT: u32 = 2;

/// gd-ui-v1.2 catalog kinds (`GD_UI_LIST_*`).
pub const LIST_MODES: usize = 0;
pub const LIST_MODS: usize = 1;
pub const LIST_REPLAYS: usize = 2;
pub const LIST_SKINS: usize = 3;
pub const LIST_STATUS: usize = 4;
pub const LIST_ONLINE: usize = 5;
/// gd-ui-v1.3: the SSBM Nucleus browser (search results, the download queue, its state line).
pub const LIST_NC: usize = 6;
pub const LIST_NC_QUEUE: usize = 7;
pub const LIST_NC_STATE: usize = 8;
pub const LIST_KINDS: usize = 9;
/// gd-ui-v1.2 request ops (`GD_UI_REQ_*`).
pub const REQ_LAUNCH: i32 = 1;
pub const REQ_MOD_SET: i32 = 2;
pub const REQ_REPLAY_PLAY: i32 = 3;
pub const REQ_SKIN_SET: i32 = 4;
pub const REQ_NET_HOST: i32 = 5;
pub const REQ_NET_JOIN: i32 = 6;
pub const REQ_NET_SPECTATE: i32 = 7;
pub const REQ_REFRESH: i32 = 8;
/// gd-ui-v1.3 Nucleus requests: search ("text\0tab\0sort"), queue toggle (post id), process ("1" start / "0" pause),
/// sync (a press of Refresh: the 10-minute rule is skipped, never the one-minute floor or the back-off).
pub const REQ_NC_SEARCH: i32 = 9;
pub const REQ_NC_QUEUE: i32 = 10;
pub const REQ_NC_PROCESS: i32 = 11;
pub const REQ_NC_SYNC: i32 = 12;
pub const REQ_MAX_ARG: u32 = 256;
pub const REQ_MAX_PENDING: usize = 8;

/// One `gd_ui_request`: the issuing module, the op and its argument (NUL-separated UTF-8 fields).
#[derive(Clone, Debug)]
pub struct Request {
    pub module: u16,
    pub op: i32,
    pub arg: String,
}

impl Shared {
    fn new() -> Shared {
        Shared {
            native_modal: false,
            submit_ns: 0,
            atlas: Atlas::new(),
            batch: Batch::with_capacity(8192),
            tree: nodes::Tree::default(),
            theme: theme::THEME_A,
            session: Session::Offline,
            retail: (-1, -1),
            in_match: false,
            caps: Vec::with_capacity(16),
            next_token: 1,
            pads: [PADStatus::zeroed(); 4],
            prev_buttons: [0; 4],
            edges: [0; 4],
            inject: [None; 4],
            keys: Vec::with_capacity(64),
            scenes: Vec::new(),
            replace: Vec::new(),
            replace_when: Vec::new(),
            door: 0,
            stack: Vec::new(),
            pending: None,
            texs: Vec::new(),
            model_texs: Vec::new(),
            dirs: Vec::new(),
            features: Vec::new(),
            window: (1280, 720),
            present_no: 0,
            log: Vec::new(),
            lists: Default::default(),
            list_gen: [0; LIST_KINDS],
            requests: Vec::with_capacity(REQ_MAX_PENDING),
            pics: Pics::default(),
        }
    }

    /// The band rect (x, y, w, h) in window pixels: the 4:3 area centred in the window (letterboxed).
    pub fn band_rect(&self) -> [f32; 4] {
        let (w, h) = (self.window.0.max(1) as f32, self.window.1.max(1) as f32);
        let (bw, bh) = if w / h > 4.0 / 3.0 { (h * 4.0 / 3.0, h) } else { (w, w * 0.75) };
        [((w - bw) * 0.5).floor(), ((h - bh) * 0.5).floor(), bw, bh]
    }

    fn online(&self) -> bool {
        matches!(self.session, Session::Netplay | Session::Synctest | Session::LabNetplay)
    }

    /// `gd_ui_capture`'s session rules (A4.4, GAP-14).
    pub fn capture(&mut self, module: u16, ports: u32, flags: u32) -> Result<u32, i32> {
        let feats = self.features[module as usize];
        if flags & !(CAP_EXCLUSIVE | CAP_SHARED | CAP_KEYBOARD | CAP_INJECT) != 0 || flags & (CAP_EXCLUSIVE | CAP_SHARED) == (CAP_EXCLUSIVE | CAP_SHARED) || ports & !0xF != 0 {
            return Err(GD_EINVAL);
        }
        if flags & CAP_INJECT != 0 && feats & FEATURE_UI_SIM_INPUT == 0 {
            return Err(GD_ECAP);
        }
        // INJECT is offline only. KEYBOARD is local-only input, so it is also allowed in an online Lab session
        // (GAP-14: the command bar works in LabNetplay); normal netplay, synctest and replays refuse it.
        if flags & CAP_INJECT != 0 && self.session != Session::Offline {
            return Err(GD_EOFFLINE);
        }
        if flags & CAP_KEYBOARD != 0 && !matches!(self.session, Session::Offline | Session::LabNetplay) {
            return Err(GD_EOFFLINE);
        }
        if flags & CAP_EXCLUSIVE != 0 && self.online() && self.in_match && self.session != Session::LabNetplay {
            return Err(GD_EOFFLINE);
        }
        if self.caps.len() >= 64 {
            return Err(GD_EFULL);
        }
        let token = self.next_token;
        self.next_token += 1;
        self.caps.push(Capture { token, module, ports, flags, released: false });
        Ok(token)
    }

    pub fn keyboard_wanted(&self) -> bool {
        self.caps.iter().any(|c| !c.released && c.flags & CAP_KEYBOARD != 0)
    }

    /// Ports the game must not see this frame: exclusive captures (local ports only, not in an online match) and a
    /// custom scene on top.
    pub fn masked_ports(&self) -> u32 {
        let online_match = self.online() && self.in_match;
        let mut m = 0;
        for c in &self.caps {
            if c.flags & CAP_EXCLUSIVE != 0 && !online_match {
                m |= c.ports;
            }
        }
        if !self.stack.is_empty() && !online_match {
            m |= 0xF;
        }
        m
    }

    fn release_module(&mut self, module: u16) {
        self.caps.retain(|c| c.module != module);
        self.tree.drop_module(module);
        let scenes = &self.scenes;
        self.stack.retain(|&s| scenes[s as usize - 1].module != module);
    }
}

pub struct PresMod {
    pub id: String,
    pub name: String,
    pub digest: u64,
    pub features: u32,
    pub disabled: bool,
    store: Store<PresData>,
    frame: Option<TypedFunc<u32, ()>>,
    event: Option<TypedFunc<(u32, i32, i32), ()>>,
    enter: Option<TypedFunc<u32, ()>>,
    leave: Option<TypedFunc<u32, ()>>,
    /// The frame profiler's slot for this mod's time (melee_core::frameprof).
    prof: u8,
}

/// Timing of the presents (ns), a fixed ring: p50 / p99 for the stats line and the bench.
pub struct Timings {
    pub ns: Box<[u32; 4096]>,
    pub n: usize,
}

impl Timings {
    pub fn push(&mut self, v: u32) {
        let i = self.n % self.ns.len();
        self.ns[i] = v;
        self.n += 1;
    }
    /// (p50, p99, max) in µs over the ring.
    pub fn pct(&self) -> (f64, f64, f64) {
        let k = self.n.min(self.ns.len());
        if k == 0 {
            return (0.0, 0.0, 0.0);
        }
        let mut v = self.ns[..k].to_vec();
        v.sort_unstable();
        let q = |f: f64| v[((k - 1) as f64 * f) as usize] as f64 / 1e3;
        (q(0.5), q(0.99), v[k - 1] as f64 / 1e3)
    }
}

pub struct PresHost {
    engine: Engine,
    pub mods: Vec<PresMod>,
    pub shared: Rc<RefCell<Shared>>,
    pub exts: Vec<Box<dyn PresExtension>>,
    pub timings: Timings,
    /// Per-stage times of each present: navigation + events, the mods' gd_ui_frame (wasm, without submit), submit
    /// (parse + encode), the Lab extensions' after-logic, node layout + draw + sort; the renderer's set_ui (instance
    /// and atlas upload, pushed by the play flow).
    pub stages: [Timings; 6],
    /// Quads of the last present.
    pub last_quads: usize,
    /// The Lab extension's shared state (gd-lab-v1), when a loaded mod imports `gd_lab`.
    pub lab: Option<Rc<RefCell<crate::lab::LabShared>>>,
}

fn mem(c: &Caller<'_, PresData>) -> Option<Memory> {
    c.data().mem
}

fn read_bytes(c: &Caller<'_, PresData>, ptr: u32, len: u32) -> Option<Vec<u8>> {
    let m = mem(c)?;
    let d = m.data(c);
    d.get(ptr as usize..(ptr as usize).checked_add(len as usize)?).map(|s| s.to_vec())
}

fn read_str(c: &Caller<'_, PresData>, ptr: u32, len: u32) -> Option<String> {
    String::from_utf8(read_bytes(c, ptr, len.min(4096))?).ok()
}

fn write_bytes(c: &mut Caller<'_, PresData>, ptr: u32, b: &[u8]) -> bool {
    let Some(m) = mem(c) else { return false };
    let d = m.data_mut(c);
    match d.get_mut(ptr as usize..ptr as usize + b.len()) {
        Some(s) => {
            s.copy_from_slice(b);
            true
        }
        None => false,
    }
}

fn feat(c: &Caller<'_, PresData>, f: u32) -> bool {
    c.data().features & f != 0
}

fn pad_bytes(p: &PADStatus, edges: u16) -> [u8; 16] {
    let mut o = [0u8; 16];
    o[0..2].copy_from_slice(&p.button.to_le_bytes());
    o[2] = p.stickX as u8;
    o[3] = p.stickY as u8;
    o[4] = p.substickX as u8;
    o[5] = p.substickY as u8;
    o[6] = p.triggerLeft;
    o[7] = p.triggerRight;
    o[8] = p.analogA;
    o[9] = p.analogB;
    o[10] = p.err as u8;
    o[12..14].copy_from_slice(&edges.to_le_bytes());
    o
}

fn linker(engine: &Engine, exts: &[Box<dyn PresExtension>]) -> Result<Linker<PresData>, String> {
    let mut l: Linker<PresData> = Linker::new(engine);
    macro_rules! f {
        ($name:literal, $f:expr) => {
            l.func_wrap("gd_ui", $name, $f).map_err(|e| format!("linker {}: {e}", $name))?;
        };
    }
    // ---- A1 draw
    f!("gd_ui_submit", |c: Caller<'_, PresData>, ptr: u32, len: u32| -> i32 {
        if !feat(&c, FEATURE_UI_DRAW) {
            return GD_ECAP;
        }
        let Some(m) = mem(&c) else { return GD_EINVAL };
        let d = c.data();
        let mut sh = d.ui().borrow_mut();
        let sh = &mut *sh;
        let Some(buf) = m.data(&c).get(ptr as usize..(ptr as usize).saturating_add(len as usize)) else { return GD_EINVAL };
        let module = d.module;
        let vector = d.features & FEATURE_UI_VECTOR != 0;
        let t = std::time::Instant::now();
        let r = sh.batch.submit(buf, module, &sh.atlas, &sh.texs[module as usize], vector);
        sh.submit_ns += t.elapsed().as_nanos() as u64;
        match r {
            Ok(()) => GD_OK,
            Err(e) => {
                sh.log.push(format!("gd_ui_submit refused at byte {}: {}", e.at, e.what));
                e.code
            }
        }
    });
    f!("gd_ui_measure", |c: Caller<'_, PresData>, s: u32, len: u32, role: i32| -> f32 {
        // per-frame safe: reads the mod's bytes in place (no copy, no allocation)
        let Some(m) = mem(&c) else { return 0.0 };
        let Some(b) = m.data(&c).get(s as usize..(s as usize).saturating_add(len as usize)) else { return 0.0 };
        let Ok(t) = core::str::from_utf8(b) else { return 0.0 };
        c.data().ui().borrow().atlas.measure(t, cmd::style(role))
    });
    // a line in the game's log (headless checks of what a mod showed); at most 16 lines of 160 bytes per present
    f!("gd_ui_log", |c: Caller<'_, PresData>, s: u32, len: u32| -> i32 {
        // off unless MELEE_UI_LOG=1 (a console write is ~0.1 ms: never in a normal present)
        static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        if !*ON.get_or_init(|| std::env::var("MELEE_UI_LOG").is_ok_and(|v| v == "1")) {
            return GD_OK;
        }
        let Some(t) = read_str(&c, s, len.min(160)) else { return GD_EINVAL };
        let ui = c.data().ui();
        let mut sh = ui.borrow_mut();
        if sh.log.len() >= 16 { return GD_EFULL }
        sh.log.push(format!("log: {t}"));
        GD_OK
    });
    f!("gd_ui_kit_id", |c: Caller<'_, PresData>, s: u32, len: u32| -> u32 {
        let Some(t) = read_str(&c, s, len) else { return 0 };
        melee_gx::ui::KIT_PIECES.iter().position(|p| !p.0.is_empty() && p.0 == t).unwrap_or(0) as u32
    });
    f!("gd_ui_tex_load", |c: Caller<'_, PresData>, s: u32, len: u32| -> u32 {
        if !feat(&c, FEATURE_UI_DRAW) {
            return 0;
        }
        let Some(p) = read_str(&c, s, len) else { return 0 };
        // the mod's package dir only: a relative path, no `..`, no drive or root
        if p.is_empty() || p.contains("..") || p.contains(':') || p.starts_with('/') || p.starts_with('\\') || !p.to_ascii_lowercase().ends_with(".png") {
            return 0;
        }
        let module = c.data().module as usize;
        let mut sh = c.data().ui().borrow_mut();
        let path = sh.dirs[module].join(&p);
        let Some((w, h, px)) = decode_png(&path) else {
            sh.log.push(format!("gd_ui_tex_load {p}: not a readable RGBA/RGB png"));
            return 0;
        };
        let Some(r) = sh.atlas.alloc(w as u16, h as u16) else {
            sh.log.push(format!("gd_ui_tex_load {p}: the UI atlas is full"));
            return 0;
        };
        sh.atlas.put(r, &px);
        sh.texs[module].push(r);
        sh.texs[module].len() as u32
    });
    // ---- Envoy drives (envoy-core-v2.md 3.1): presentation only, never read back by a sim
    // a drive model in the world (melee_modes::tu::drive_draw): model id, centre, yaw, diameter, glow 0..1, alpha 0..1
    f!("gd_ui_world_model", |c: Caller<'_, PresData>, model: i32, x: f32, y: f32, z: f32, yaw: f32, scale: f32, glow: f32, alpha: f32| -> i32 {
        if !feat(&c, FEATURE_UI_DRAW) || !(0..melee_modes::tu::drive_draw::MODEL_COUNT as i32).contains(&model) {
            return GD_EINVAL;
        }
        if ![x, y, z, yaw, scale, glow, alpha].iter().all(|v| v.is_finite()) {
            return GD_EINVAL;
        }
        let i = melee_modes::tu::drive_draw::Inst::Model { model: model as u8, at: [x, y, z], yaw, scale: scale.clamp(0.0, 200.0), alpha: alpha.clamp(0.0, 1.0), mat: 0, p: [glow.clamp(0.0, 2.0), 0.0, 0.0, 0.0], rgba: 0xFFFF_FFFF };
        if melee_modes::tu::drive_draw::push(i) { GD_OK } else { GD_EFULL }
    });
    // envoy-fx.md: a drive model with a built-in material (melee_gx::fx::MAT_*: 0 body, 1 glass, 2 glow, 3 foil,
    // 4 dissolve, 5 unlit additive; | 0x80 a light trail, | 0x40 the ink edge on a drive body), its parameters p0..p3 (glow 0..2, charge 0..1, the wearer's
    // Heat state 0..3, extra: foil rarity 1 Epic / 2 Unique, dissolve 0 whole .. 1 gone) and colour RGBA. Same cap as
    // gd_ui_world_model; host-clamped; presentation only.
    f!("gd_ui_world_model_ex", |c: Caller<'_, PresData>, model: i32, x: f32, y: f32, z: f32, yaw: f32, scale: f32, alpha: f32, material: i32, p0: f32, p1: f32, p2: f32, p3: f32, rgba: u32| -> i32 {
        if !feat(&c, FEATURE_UI_DRAW) || !(0..melee_modes::tu::drive_draw::MODEL_COUNT as i32).contains(&model) {
            return GD_EINVAL;
        }
        let m = material as u32;
        if m & !0xFF != 0 || (m & melee_gx::fx::MAT_ID as u32) >= melee_gx::fx::MAT_COUNT as u32 || ![x, y, z, yaw, scale, alpha, p0, p1, p2, p3].iter().all(|v| v.is_finite()) {
            return GD_EINVAL;
        }
        let p = [p0.clamp(0.0, 2.0), p1.clamp(0.0, 1.0), p2.clamp(0.0, 3.0), p3.clamp(0.0, 2.0)];
        let i = melee_modes::tu::drive_draw::Inst::Model { model: model as u8, at: [x, y, z], yaw, scale: scale.clamp(0.0, 200.0), alpha: alpha.clamp(0.0, 1.0), mat: m as u8, p, rgba };
        if melee_modes::tu::drive_draw::push(i) { GD_OK } else { GD_EFULL }
    });
    // envoy-fx.md: a full-screen post effect this present (melee_gx::fx::POST_*: 1 vignette, 2 ember tint, 3 heat-haze
    // with embers (held: re-post every present), 4 flash, 5 radial ring, 6 colour split, 7 shockwave (one-shots: post
    // once); | 0x100: (x, y) is a world position, else 640x480 canvas units), strength 0..1, colour RGBA. At most 4 a
    // present (GD_EFULL), clamped, never over the centre of the picture; flashes rate-capped; off at intensity Off.
    f!("gd_fx_post", |c: Caller<'_, PresData>, effect: u32, strength: f32, x: f32, y: f32, rgba: u32| -> i32 {
        if !feat(&c, FEATURE_UI_DRAW) {
            return GD_ECAP;
        }
        let kind = effect & 0xFF;
        if !(1..melee_gx::fx::POST_COUNT).contains(&kind) || effect & !(0xFF | melee_gx::fx::POST_WORLD) != 0 || ![strength, x, y].iter().all(|v| v.is_finite()) {
            return GD_EINVAL;
        }
        if melee_gx::fx::post(effect, strength, x, y, rgba) { GD_OK } else { GD_EFULL }
    });
    // an additive effect shape in the world: kind (drive_draw::FX_*), centre, three numbers, colour RGBA
    f!("gd_ui_world_fx", |c: Caller<'_, PresData>, kind: i32, x: f32, y: f32, z: f32, a: f32, b: f32, cc: f32, rgba: u32| -> i32 {
        if !feat(&c, FEATURE_UI_DRAW) || !(1..=6).contains(&kind) || ![x, y, z, a, b, cc].iter().all(|v| v.is_finite() && v.abs() < 1.0e5) {
            return GD_EINVAL;
        }
        let i = melee_modes::tu::drive_draw::Inst::Fx { kind: kind as u8, at: [x, y, z], a, b, c: cc, rgba };
        if melee_modes::tu::drive_draw::push(i) { GD_OK } else { GD_EFULL }
    });
    // a drive model as a turning sprite sheet for 2D cards: a texture handle (SPRITE_FRAMES cells side by side), 0 if
    // the atlas is full. Rasterized once per module and model.
    f!("gd_ui_model_tex", |c: Caller<'_, PresData>, model: i32| -> u32 {
        if !feat(&c, FEATURE_UI_DRAW) || !(0..melee_modes::tu::drive_draw::MODEL_COUNT as i32).contains(&model) {
            return 0;
        }
        let module = c.data().module as usize;
        let mut sh = c.data().ui().borrow_mut();
        let sh = &mut *sh;
        if let Some(&(_, _, h)) = sh.model_texs.iter().find(|t| t.0 == module && t.1 == model) {
            return h;
        }
        let Some((w, h, px)) = melee_modes::tu::drive_draw::sprite_sheet(model as usize) else { return 0 };
        let Some(r) = sh.atlas.alloc(w as u16, h as u16) else { return 0 };
        sh.atlas.put(r, &px);
        sh.texs[module].push(r);
        let hd = sh.texs[module].len() as u32;
        sh.model_texs.push((module, model, hd));
        hd
    });
    // a Brawl sound effect from the player's own disc (melee_audio::brawl): fire and forget, presentation only
    f!("gd_ui_brawl_sfx", |c: Caller<'_, PresData>, s: u32, len: u32| -> i32 {
        let Some(name) = read_str(&c, s, len.min(64)) else { return GD_EINVAL };
        melee_audio::brawl::play_sfx(&name);
        GD_OK
    });
    // GAP-0: the band rect and the window size, for window-space layouts: out = f32 x, y, w, h, win_w, win_h
    f!("gd_ui_band_rect", |mut c: Caller<'_, PresData>, out: u32| -> i32 {
        let (b, win) = { let sh = c.data().ui().borrow(); (sh.band_rect(), sh.window) };
        let mut o = [0u8; 24];
        for (i, v) in [b[0], b[1], b[2], b[3], win.0 as f32, win.1 as f32].iter().enumerate() {
            o[i * 4..i * 4 + 4].copy_from_slice(&v.to_le_bytes());
        }
        if write_bytes(&mut c, out, &o) { GD_OK } else { GD_EINVAL }
    });
    f!("gd_ui_theme", |c: Caller<'_, PresData>, id: i32| -> i32 {
        if !(0..theme::THEMES.len() as i32).contains(&id) {
            return GD_EINVAL;
        }
        c.data().ui().borrow_mut().theme = id;
        GD_OK
    });
    f!("gd_ui_token", |c: Caller<'_, PresData>, tok: i32| -> u32 { theme::token(c.data().ui().borrow().theme, tok) });
    f!("gd_ui_session", |c: Caller<'_, PresData>| -> i32 { session_code(c.data().ui().borrow().session) });
    f!("gd_ui_scene", |c: Caller<'_, PresData>| -> i32 { c.data().ui().borrow().stack.last().copied().unwrap_or(0) as i32 });
    f!("gd_ui_retail_scene", |c: Caller<'_, PresData>| -> i32 {
        let r = c.data().ui().borrow().retail;
        (r.0 << 8) | (r.1 & 0xFF)
    });
    // ---- A2 nodes
    macro_rules! menu {
        ($c:expr) => {
            if !feat(&$c, FEATURE_UI_MENU) {
                return GD_ECAP;
            }
        };
    }
    f!("gd_ui_node", |c: Caller<'_, PresData>, parent: u32, kind: i32, id: u32| -> u32 {
        if !feat(&c, FEATURE_UI_MENU) {
            return 0;
        }
        let m = c.data().module;
        c.data().ui().borrow_mut().tree.create(m, parent, kind, id)
    });
    f!("gd_ui_node_label", |c: Caller<'_, PresData>, n: u32, s: u32, len: u32| -> i32 {
        menu!(c);
        let Some(t) = read_str(&c, s, len) else { return GD_EINVAL };
        let m = c.data().module;
        match c.data().ui().borrow_mut().tree.get_mut(n, m) { Some(x) => { x.label = t; GD_OK } None => GD_EINVAL }
    });
    f!("gd_ui_node_value", |c: Caller<'_, PresData>, n: u32, v: i32| -> i32 {
        menu!(c);
        let m = c.data().module;
        match c.data().ui().borrow_mut().tree.get_mut(n, m) { Some(x) => { x.value = v; GD_OK } None => GD_EINVAL }
    });
    f!("gd_ui_node_get", |c: Caller<'_, PresData>, n: u32| -> i32 {
        let m = c.data().module;
        c.data().ui().borrow().tree.get(n, m).map_or(0, |x| x.value)
    });
    f!("gd_ui_node_range", |c: Caller<'_, PresData>, n: u32, lo: i32, hi: i32, step: i32| -> i32 {
        menu!(c);
        if hi < lo || step <= 0 {
            return GD_EINVAL;
        }
        let m = c.data().module;
        match c.data().ui().borrow_mut().tree.get_mut(n, m) { Some(x) => { x.range = (lo, hi, step); x.value = x.value.clamp(lo, hi); GD_OK } None => GD_EINVAL }
    });
    f!("gd_ui_node_choices", |c: Caller<'_, PresData>, n: u32, s: u32, len: u32| -> i32 {
        menu!(c);
        let Some(t) = read_str(&c, s, len) else { return GD_EINVAL };
        let m = c.data().module;
        match c.data().ui().borrow_mut().tree.get_mut(n, m) {
            Some(x) => {
                x.choices = t.split('\0').filter(|s| !s.is_empty()).map(String::from).collect();
                GD_OK
            }
            None => GD_EINVAL,
        }
    });
    f!("gd_ui_node_flags", |c: Caller<'_, PresData>, n: u32, fl: u32| -> i32 {
        menu!(c);
        let m = c.data().module;
        match c.data().ui().borrow_mut().tree.get_mut(n, m) { Some(x) => { x.flags = fl & 7; GD_OK } None => GD_EINVAL }
    });
    f!("gd_ui_node_rect", |c: Caller<'_, PresData>, n: u32, x: f32, y: f32, w: f32, h: f32| -> i32 {
        menu!(c);
        if ![x, y, w, h].iter().all(|v| v.is_finite()) {
            return GD_EINVAL;
        }
        let m = c.data().module;
        match c.data().ui().borrow_mut().tree.get_mut(n, m) { Some(nd) => { nd.place = Some([x, y, w, h]); GD_OK } None => GD_EINVAL }
    });
    // envoy-fx.md: a built-in quad effect for a node's quads (melee_gx::ui::FX_*: 0 none, 1 foil, 2 holo, 3 glow),
    // colour RGBA; presentation only. Immediate-mode drawing sets the same with the OP_EFFECT record.
    f!("gd_ui_node_effect", |c: Caller<'_, PresData>, n: u32, effect: u32, rgba: u32| -> i32 {
        menu!(c);
        if effect >= melee_gx::ui::FX_COUNT {
            return GD_EINVAL;
        }
        let m = c.data().module;
        match c.data().ui().borrow_mut().tree.get_mut(n, m) { Some(x) => { x.fx = [effect, rgba]; GD_OK } None => GD_EINVAL }
    });
    f!("gd_ui_node_free", |c: Caller<'_, PresData>, n: u32| {
        let m = c.data().module;
        c.data().ui().borrow_mut().tree.free_subtree(n, m);
    });
    f!("gd_ui_focus", |c: Caller<'_, PresData>, n: u32| -> i32 {
        menu!(c);
        let m = c.data().module;
        let mut sh = c.data().ui().borrow_mut();
        if sh.tree.get(n, m).is_none() {
            return GD_EINVAL;
        }
        sh.tree.nav_mut(m).focus = n;
        sh.tree.dirty = true;
        GD_OK
    });
    f!("gd_ui_focused", |c: Caller<'_, PresData>| -> u32 {
        let m = c.data().module;
        c.data().ui().borrow_mut().tree.nav_mut(m).focus
    });
    f!("gd_ui_show", |c: Caller<'_, PresData>, root: u32, layer: i32, z: i32| -> i32 {
        menu!(c);
        if !(cmd::LAYER_SCENE..=cmd::LAYER_DEBUG).contains(&layer) {
            return GD_EINVAL;
        }
        let m = c.data().module;
        if c.data().ui().borrow_mut().tree.show(m, root, layer, z) { GD_OK } else { GD_EINVAL }
    });
    f!("gd_ui_hide", |c: Caller<'_, PresData>, root: u32| {
        let m = c.data().module;
        c.data().ui().borrow_mut().tree.hide(m, root);
    });
    // ---- A4 input
    f!("gd_ui_capture", |c: Caller<'_, PresData>, ports: u32, flags: u32| -> u32 {
        if !feat(&c, FEATURE_UI_INPUT) {
            return 0;
        }
        let m = c.data().module;
        let mut sh = c.data().ui().borrow_mut();
        match sh.capture(m, ports, flags) {
            Ok(t) => t,
            Err(e) => {
                sh.log.push(format!("gd_ui_capture(ports {ports:#x}, flags {flags:#x}) refused: {e}"));
                0
            }
        }
    });
    f!("gd_ui_release", |c: Caller<'_, PresData>, token: u32| {
        let m = c.data().module;
        for cap in c.data().ui().borrow_mut().caps.iter_mut().filter(|x| x.token == token && x.module == m) {
            cap.released = true;
        }
    });
    // whether a controller is plugged into a port (its last read had no PAD_ERR_NO_CONTROLLER); read-only, no capture
    // needed (the Envoy co-op HUD's "connect a second controller" note)
    f!("gd_ui_pad_present", |c: Caller<'_, PresData>, port: u32| -> i32 {
        if port > 3 {
            return GD_EINVAL;
        }
        (c.data().ui().borrow().pads[port as usize].err != -1) as i32
    });
    f!("gd_ui_pad", |mut c: Caller<'_, PresData>, port: u32, out: u32| -> i32 {
        if c.data().ui().borrow().native_modal {
            return GD_ESTATE;
        }
        if !feat(&c, FEATURE_UI_INPUT) {
            return GD_ECAP;
        }
        let m = c.data().module;
        let b = {
            let sh = c.data().ui().borrow();
            if port > 3 || !sh.caps.iter().any(|x| x.module == m && !x.released && x.ports >> port & 1 != 0) {
                return GD_ESTATE;
            }
            pad_bytes(&sh.pads[port as usize], sh.edges[port as usize])
        };
        if write_bytes(&mut c, out, &b) { GD_OK } else { GD_EINVAL }
    });
    // read-only view of the sim transition (melee_modes::tu::tag_swap; the tag cinematic): 9 i32 words, phase, kind,
    // slot, frame, len, frames since the last release (saturating), release count, from kind, to kind; then skipped
    f!("gd_ui_transition", |mut c: Caller<'_, PresData>, out: u32| -> i32 {
        let t = unsafe { melee_modes::tu::tag_swap::TRANSITION };
        let since = unsafe { melee_modes::tu::tag_swap::LOGIC_FRAMES.wrapping_sub(t.released_at) };
        let w = [t.phase as i32, t.kind as i32, t.slot as i32, t.frame as i32, t.len as i32,
                 if t.count == 0 { i32::MAX } else { since.min(i32::MAX as u32) as i32 }, t.count as i32,
                 t.from_kind as i32, t.to_kind as i32, t.skipped as i32];
        let mut b = [0u8; 40];
        for (i, x) in w.iter().enumerate() { b[i * 4..i * 4 + 4].copy_from_slice(&x.to_le_bytes()); }
        if write_bytes(&mut c, out, &b) { GD_OK } else { GD_EINVAL }
    });
    // read-only view of the Envoy run (melee_modes::tu::envoy::VIEW, published by the envoy sim mod): n i32 words
    f!("gd_ui_envoy", |mut c: Caller<'_, PresData>, out: u32, n: u32| -> i32 {
        let v = unsafe { melee_modes::tu::envoy::VIEW };
        let n = (n as usize).min(v.len());
        let mut b = [0u8; 4 * melee_modes::tu::envoy::VIEW_WORDS];
        for (i, x) in v.iter().take(n).enumerate() { b[i * 4..i * 4 + 4].copy_from_slice(&x.to_le_bytes()); }
        if write_bytes(&mut c, out, &b[..n * 4]) { GD_OK } else { GD_EINVAL }
    });
    // read-only view of the Petey boss (melee_modes::tu::petey::view): n i32 words (notes/petey-boss.md)
    f!("gd_ui_fight_hud", |mut c: Caller<'_, PresData>, out: u32, n: u32| -> i32 {
        let v = unsafe { melee_modes::tu::envoy::hud_words() };
        let n = (n as usize).min(v.len());
        let mut b = [0u8; 64];
        for (i, x) in v.iter().take(n).enumerate() { b[i * 4..i * 4 + 4].copy_from_slice(&x.to_le_bytes()); }
        if write_bytes(&mut c, out, &b[..n * 4]) { GD_OK } else { GD_EINVAL }
    });
    f!("gd_ui_petey", |mut c: Caller<'_, PresData>, out: u32, n: u32| -> i32 {
        let v = melee_modes::tu::petey::view();
        let n = (n as usize).min(v.len());
        let mut b = [0u8; 48];
        for (i, x) in v.iter().take(n).enumerate() { b[i * 4..i * 4 + 4].copy_from_slice(&x.to_le_bytes()); }
        if write_bytes(&mut c, out, &b[..n * 4]) { GD_OK } else { GD_EINVAL }
    });
    f!("gd_ui_inject", |c: Caller<'_, PresData>, port: u32, pad: u32| -> i32 {
        if !feat(&c, FEATURE_UI_SIM_INPUT) {
            return GD_ECAP;
        }
        let Some(b) = read_bytes(&c, pad, 12) else { return GD_EINVAL };
        let m = c.data().module;
        let mut sh = c.data().ui().borrow_mut();
        if sh.session != Session::Offline {
            return GD_EOFFLINE;
        }
        if port > 3 || !sh.caps.iter().any(|x| x.module == m && !x.released && x.flags & CAP_INJECT != 0 && x.ports >> port & 1 != 0) {
            return GD_ESTATE;
        }
        let mut p = PADStatus::zeroed();
        p.button = u16::from_le_bytes([b[0], b[1]]);
        p.stickX = b[2] as i8;
        p.stickY = b[3] as i8;
        p.substickX = b[4] as i8;
        p.substickY = b[5] as i8;
        p.triggerLeft = b[6];
        p.triggerRight = b[7];
        p.analogA = b[8];
        p.analogB = b[9];
        sh.inject[port as usize] = Some(p);
        GD_OK
    });
    // GAP-14: keyboard events of this present for a KEYBOARD capture (Offline or LabNetplay): u32 kind<<24 | value
    f!("gd_ui_keys", |mut c: Caller<'_, PresData>, out: u32, max: u32| -> i32 {
        let m = c.data().module;
        let words: Vec<u8> = {
            let sh = c.data().ui().borrow();
            if !sh.caps.iter().any(|x| x.module == m && !x.released && x.flags & CAP_KEYBOARD != 0) {
                return GD_ESTATE;
            }
            sh.keys.iter().take(max as usize).flat_map(|k| k.to_le_bytes()).collect()
        };
        if write_bytes(&mut c, out, &words) { (words.len() / 4) as i32 } else { GD_EINVAL }
    });
    // ---- A5 scenes
    macro_rules! scene {
        ($c:expr) => {
            if !feat(&$c, FEATURE_UI_SCENE) {
                return GD_ECAP;
            }
        };
    }
    f!("gd_ui_scene_register", |c: Caller<'_, PresData>, s: u32, len: u32| -> u32 {
        if !feat(&c, FEATURE_UI_SCENE) {
            return 0;
        }
        let Some(name) = read_str(&c, s, len) else { return 0 };
        let m = c.data().module;
        let mut sh = c.data().ui().borrow_mut();
        sh.scenes.push(SceneDef { module: m, name });
        sh.scenes.len() as u32
    });
    f!("gd_ui_scene_replace", |c: Caller<'_, PresData>, major: i32, minor: i32, scene: u32| -> i32 {
        scene!(c);
        let m = c.data().module;
        let mut sh = c.data().ui().borrow_mut();
        if scene == 0 || sh.scenes.get(scene as usize - 1).is_none_or(|s| s.module != m) {
            return GD_EINVAL;
        }
        if sh.replace.iter().any(|r| r.0 == (major, minor)) {
            sh.log.push(format!("gd_ui_scene_replace({major}, {minor}): that retail scene already has a replacer"));
            return GD_ESTATE;
        }
        sh.replace.push(((major, minor), scene));
        GD_OK
    });
    // replace a retail scene only while a host condition holds (1: an Envoy run is armed and has fought since arming,
    // so the VS CSS after its results is the camp, and every other VS CSS stays retail with no custom frame)
    f!("gd_ui_scene_replace_when", |c: Caller<'_, PresData>, major: i32, minor: i32, scene: u32, cond: u32| -> i32 {
        scene!(c);
        let m = c.data().module;
        let mut sh = c.data().ui().borrow_mut();
        if scene == 0 || cond != REPLACE_WHEN_ENVOY || sh.scenes.get(scene as usize - 1).is_none_or(|s| s.module != m) {
            return GD_EINVAL;
        }
        if sh.replace.iter().any(|r| r.0 == (major, minor)) || sh.replace_when.iter().any(|r| r.0 == (major, minor)) {
            sh.log.push(format!("gd_ui_scene_replace_when({major}, {minor}): that retail scene already has a replacer"));
            return GD_ESTATE;
        }
        sh.replace_when.push(((major, minor), scene, cond));
        GD_OK
    });
    // ---- the Envoy door (envoy-redesign.md 1.2-1.3; melee_modes::tu::envoy). Offline only.
    // register the camp scene: the main menu grows its Envoy row, and the row / the browser's Envoy mode open the scene
    f!("gd_ui_envoy_door", |c: Caller<'_, PresData>, scene: u32| -> i32 {
        scene!(c);
        let m = c.data().module;
        let mut sh = c.data().ui().borrow_mut();
        if scene == 0 || sh.scenes.get(scene as usize - 1).is_none_or(|s| s.module != m) {
            return GD_EINVAL;
        }
        sh.door = scene;
        unsafe { melee_modes::tu::envoy::DOOR_ON = 1 };
        GD_OK
    });
    // bits: 1 armed, 2 the main menu is up with no custom scene on it (its Envoy row is shown, envoy-v3-ui.md), 4 its
    // cursor is on the Envoy row, 8 a run record is saved, 16 a fight was played since arming, 32 the player's Brawl
    // files are missing (Envoy is blocked), 64 the Envoy row was chosen without them (show the message)
    f!("gd_ui_envoy_state", |c: Caller<'_, PresData>| -> i32 {
        use melee_modes::tu::envoy as ev;
        let sh = c.data().ui().borrow();
        let main = sh.retail == (melee_types::GM_MENU as i32, 1)
            && sh.stack.is_empty()
            && ev::DOOR_ON_now()
            && unsafe { melee_modes::tu::mnmain::mn_804A04F0.cur_menu } as i32 == melee_types::MENU_KIND_MAIN as i32;
        let hover = main && unsafe { melee_modes::tu::mnmain::mn_804A04F0.hovered_selection } as i32 == melee_modes::tu::mnmain::ENVOY_SEL;
        unsafe {
            if !hover {
                ev::DOOR_NO_DISC = 0;
            } else if ev::DOOR_NO_DISC > 0 {
                ev::DOOR_NO_DISC -= 1;
            }
            (ev::ARMED != 0) as i32 | (main as i32) << 1 | (hover as i32) << 2 | ((ev::SAVED_LEN > 0) as i32) << 3 | ((ev::CAMP_DUE != 0) as i32) << 4
                | (!melee_modes::tu::petey_model::brawl_present() as i32) << 5 | ((hover && ev::DOOR_NO_DISC > 0) as i32) << 6
                | ((ev::BYPASSED && !ev::SEL_HOLD) as i32) << 7
                | ((main && melee_modes::tu::mnmain::ENVOY_ROW_OK != 0) as i32) << 8
        }
    });
    // Envoy V3: player 2 joined (1) or left (0) at the camp; a new run started after this is co-op. Offline only.
    f!("gd_ui_envoy_coop", |c: Caller<'_, PresData>, on: i32| -> i32 {
        if c.data().ui().borrow().session != Session::Offline {
            return GD_EOFFLINE;
        }
        unsafe { melee_modes::tu::envoy::COOP_JOIN = (on != 0) as i32 };
        GD_OK
    });
    // the saved run record (melee_modes::tu::envoy::SAVED): the words written, 0 = none
    f!("gd_ui_envoy_saved", |mut c: Caller<'_, PresData>, out: u32, n: u32| -> i32 {
        let (rec, len) = unsafe { (melee_modes::tu::envoy::SAVED, melee_modes::tu::envoy::SAVED_LEN) };
        let n = (n as usize).min(len.max(0) as usize).min(rec.len());
        let mut b = [0u8; 4 * melee_modes::tu::envoy::REC_WORDS];
        for (i, x) in rec.iter().take(n).enumerate() { b[i * 4..i * 4 + 4].copy_from_slice(&x.to_le_bytes()); }
        if write_bytes(&mut c, out, &b[..n * 4]) { n as i32 } else { GD_EINVAL }
    });
    // Envoy endless loops: the best loop any run on this save reached (0: none past loop 1 yet)
    f!("gd_ui_envoy_best", |_c: Caller<'_, PresData>| -> i32 { unsafe { melee_modes::tu::envoy::BEST_LOOP } });
    // arm / disarm the run: op 0 disarm, 1 new run (forgets the saved run), 2 continue, 3 abandon (forget + disarm);
    // | 0x10: start the VS mode now (the camp on the main menu; the first CSS is the party pick, not the camp)
    // the camp's tag-button row: 0 Y, 1 X, 2 D-pad down (classic); returns the value now
    f!("gd_ui_envoy_set_tag_button", |_c: Caller<'_, PresData>, b: i32| -> i32 {
        if (0..=2).contains(&b) {
            unsafe { melee_modes::tu::envoy::TAG_BUTTON = b };
        }
        melee_modes::tu::envoy::tag_button()
    });
    f!("gd_ui_envoy_arm", |c: Caller<'_, PresData>, op: i32| -> i32 {
        use melee_modes::tu::envoy as ev;
        let sh = c.data().ui().borrow();
        if sh.session != Session::Offline {
            return GD_EOFFLINE;
        }
        let launch = op & 0x10 != 0;
        // no Brawl files, no Envoy run (full-game plan Q3): arming is refused, disarm / abandon still work
        if matches!(op & 0xF, 1 | 2) && !melee_modes::tu::petey_model::brawl_present() {
            return GD_ESTATE;
        }
        if launch && sh.retail.0 != melee_types::GM_MENU as i32 {
            return GD_ESTATE;
        }
        unsafe {
            match op & 0xF {
                0 => ev::ARMED = 0,
                1 => {
                    ev::SAVED_LEN = 0;
                    ev::SAVED_DROP = 1;
                    ev::ARMED = 1;
                }
                2 => {
                    ev::ARMED = 1;
                    // the road after a fight: on to the next fight without the character and stage selects
                    if !launch && ev::skip_select_on() {
                        ev::SKIP_SELECT = 1;
                    }
                }
                3 => {
                    ev::SAVED_LEN = 0;
                    ev::SAVED_DROP = 1;
                    ev::ARMED = 0;
                }
                // the run is over: back to the camp (through the main menu), never the character select
                4 => {
                    ev::ARMED = 0;
                    ev::BACK_TO_CAMP = 1;
                }
                _ => return GD_EINVAL,
            }
            if launch {
                ev::CAMP_DUE = 0;
                ev::LAUNCH_VS = 1;
                // the camp's picker set the party (or a saved run has its own): no character or stage select
                if ev::skip_select_on() && matches!(op & 0xF, 1 | 2) && (ev::PICK[0] >= 0 || op & 0xF == 2) {
                    ev::SKIP_SELECT = 3;
                    ev::SKIP_WAIT = 0;
                }
            }
        }
        GD_OK
    });
    // the camp's fighter picker: the party for a new run (melee_modes::tu::envoy::PICK; -1 none)
    f!("gd_ui_envoy_party", |_c: Caller<'_, PresData>, m1: i32, p1: i32, m2: i32, p2: i32| -> i32 {
        let ok = |k: i32| k == -1 || melee_modes::tu::envoy::taggable(k);
        if !(ok(m1) && ok(p1) && ok(m2) && ok(p2)) {
            return GD_EINVAL;
        }
        unsafe { melee_modes::tu::envoy::PICK = [m1, p1, m2, p2] };
        GD_OK
    });
    // the picker's roster: every fighter the build can load as one half of a tag pair, retail (0-25) then m-ex
    // added slots (CK_MEX0 +): one word each, ckind | state << 16 (state 1 offered, 2 locked). Unlocks: everything
    // the build can load is offered for now (gm_IsCKindUnlocked is not applied; owner, Nov 11). Returns the count.
    f!("gd_ui_envoy_roster", |mut c: Caller<'_, PresData>, out: u32, max: i32| -> i32 {
        use melee_modes::tu::envoy as ev;
        let mut b: Vec<u8> = Vec::new();
        for k in (0..26).chain(ev::CK_MEX0..ev::CK_MEX0 + ev::MEX_SLOTS) {
            if (b.len() / 4) as i32 >= max {
                break;
            }
            if ev::taggable(k) {
                b.extend_from_slice(&(k | 1 << 16).to_le_bytes());
            }
        }
        if write_bytes(&mut c, out, &b) { (b.len() / 4) as i32 } else { GD_EINVAL }
    });
    // an m-ex fighter's name into `out` (at most `cap` bytes); returns the length, 0 for a retail fighter
    f!("gd_ui_envoy_fighter_name", |mut c: Caller<'_, PresData>, ck: i32, out: u32, cap: i32| -> i32 {
        let p = unsafe { melee_modes::tu::envoy::mex_name(ck) };
        if p.is_null() {
            return 0;
        }
        let n = unsafe { core::ffi::CStr::from_ptr(p as *const core::ffi::c_char) }.to_bytes();
        let n = &n[..n.len().min(cap.max(0) as usize)];
        if write_bytes(&mut c, out, n) { n.len() as i32 } else { GD_EINVAL }
    });
    f!("gd_ui_scene_goto", |c: Caller<'_, PresData>, to: u32| -> i32 {
        scene!(c);
        let mut sh = c.data().ui().borrow_mut();
        if to & SCENE_RETAIL != 0 {
            // back to the retail flow: only the scene under the custom ones (the port does not drive the retail
            // scene machine from the UI yet)
            let (ma, mi) = (((to >> 8) & 0xFF) as i32, (to & 0xFF) as i32);
            if (ma, mi) != sh.retail {
                return GD_ESTATE;
            }
            sh.pending = Some(0);
            return GD_OK;
        }
        if to == 0 || to as usize > sh.scenes.len() {
            return GD_EINVAL;
        }
        sh.pending = Some(to as i64);
        GD_OK
    });
    f!("gd_ui_scene_back", |c: Caller<'_, PresData>| -> i32 {
        scene!(c);
        let mut sh = c.data().ui().borrow_mut();
        if sh.stack.is_empty() {
            return GD_ESTATE;
        }
        sh.pending = Some(-1);
        GD_OK
    });
    f!("gd_ui_match_start", |c: Caller<'_, PresData>, _p: u32, _l: u32| -> i32 {
        scene!(c);
        // gd_match_setup is not defined yet (A5): refused until the match setup lands
        GD_ESTATE
    });
    // ---- gd-ui-v1.2 host catalogs and requests (UI_HOST): the in-game browser
    // generation << 16 | record count of a catalog (one call per section per present at most)
    f!("gd_ui_list_info", |c: Caller<'_, PresData>, kind: u32| -> i32 {
        if !feat(&c, FEATURE_UI_HOST) {
            return GD_ECAP;
        }
        let sh = c.data().ui().borrow();
        match sh.lists.get(kind as usize) {
            Some(l) => ((sh.list_gen[kind as usize] as i32 & 0x7FFF) << 16) | l.len().min(0xFFFF) as i32,
            None => GD_EINVAL,
        }
    });
    // copy record `i` of a catalog into the mod's memory (truncated to cap): the bytes written; no allocation
    f!("gd_ui_list_get", |mut c: Caller<'_, PresData>, kind: u32, i: u32, out: u32, cap: u32| -> i32 {
        if !feat(&c, FEATURE_UI_HOST) {
            return GD_ECAP;
        }
        let Some(m) = mem(&c) else { return GD_EINVAL };
        let ui = c.data().ui().clone();
        let sh = ui.borrow();
        let Some(rec) = sh.lists.get(kind as usize).and_then(|l| l.get(i as usize)) else { return GD_EINVAL };
        let n = rec.len().min(cap as usize);
        match m.data_mut(&mut c).get_mut(out as usize..out as usize + n) {
            Some(d) => {
                d.copy_from_slice(&rec[..n]);
                n as i32
            }
            None => GD_EINVAL,
        }
    });
    // queue a request for the play flow (applied after this present); at most 8 per present, args <= 256 bytes
    f!("gd_ui_request", |c: Caller<'_, PresData>, op: i32, s: u32, len: u32| -> i32 {
        if !feat(&c, FEATURE_UI_HOST) {
            return GD_ECAP;
        }
        if !(REQ_LAUNCH..=REQ_NC_SYNC).contains(&op) || len > REQ_MAX_ARG {
            return GD_EINVAL;
        }
        let Some(arg) = read_str(&c, s, len) else { return GD_EINVAL };
        let module = c.data().module;
        let mut sh = c.data().ui().borrow_mut();
        if sh.requests.len() >= REQ_MAX_PENDING {
            return GD_EFULL;
        }
        sh.requests.push(Request { module, op, arg });
        GD_OK
    });
    // gd-ui-v1.3: the texture handle of host picture `key` (a record field), 0 while it is not on hand. Asking renews
    // it; an unknown key is passed to the play flow, which fetches it off the game thread. At most 2 uploads a present.
    f!("gd_ui_host_pic", |c: Caller<'_, PresData>, key: u32| -> u32 {
        if !feat(&c, FEATURE_UI_HOST) || !feat(&c, FEATURE_UI_DRAW) || key == 0 {
            return 0;
        }
        let module = c.data().module;
        let mut g = c.data().ui().borrow_mut();
        let sh = &mut *g;
        let pno = sh.present_no;
        let p = &mut sh.pics;
        if p.slots.is_empty() {
            for _ in 0..PIC_SLOTS {
                match sh.atlas.alloc(PIC_W, PIC_H) {
                    Some(r) => p.slots.push((r, 0, 0)),
                    None => break,
                }
            }
        }
        // this module's handles of the slots (pushed once into its texture table)
        let base = match p.base.iter().find(|b| b.0 == module) {
            Some(b) => b.1,
            None => {
                let t = &mut sh.texs[module as usize];
                let b = t.len() as u32 + 1;
                t.extend(p.slots.iter().map(|s| s.0));
                p.base.push((module, b));
                b
            }
        };
        if let Some(i) = p.slots.iter().position(|s| s.1 == key) {
            p.slots[i].2 = pno;
            return base + i as u32;
        }
        let Some(r) = p.ready.iter().position(|r| r.0 == key) else {
            if p.wants.len() < 64 && !p.wants.contains(&key) {
                p.wants.push(key);
            }
            return 0;
        };
        if p.uploads >= PIC_UPLOADS_PER_PRESENT {
            return 0;
        }
        // the least recently used slot not drawn this present
        let Some(i) = (0..p.slots.len()).filter(|&i| p.slots[i].2 != pno).min_by_key(|&i| p.slots[i].2) else { return 0 };
        let px = p.ready[r].1.clone();
        if px.len() != PIC_W as usize * PIC_H as usize * 4 {
            return 0;
        }
        p.uploads += 1;
        p.slots[i] = (p.slots[i].0, key, pno);
        let rect = p.slots[i].0;
        sh.atlas.put(rect, &px);
        base + i as u32
    });
    // gd_lab: the Lab half links its own imports (B) on the same linker
    for e in exts {
        e.link(&mut l)?;
    }
    Ok(l)
}

fn decode_png(path: &std::path::Path) -> Option<(u32, u32, Vec<u8>)> {
    let f = std::fs::File::open(path).ok()?;
    let mut d = png::Decoder::new(std::io::BufReader::new(f));
    d.set_transformations(png::Transformations::EXPAND);
    let mut r = d.read_info().ok()?;
    let mut buf = vec![0; r.output_buffer_size()?];
    let info = r.next_frame(&mut buf).ok()?;
    let (w, h) = (info.width, info.height);
    if w > 1024 || h > 1024 || info.bit_depth != png::BitDepth::Eight {
        return None;
    }
    let px = match info.color_type {
        png::ColorType::Rgba => buf[..(w * h * 4) as usize].to_vec(),
        png::ColorType::Rgb => buf[..(w * h * 3) as usize].chunks(3).flat_map(|p| [p[0], p[1], p[2], 255]).collect(),
        _ => return None,
    };
    Some((w, h, px))
}

/// True when the module imports from `gd_ui` (a presentation mod: the sim host skips it).
pub fn is_presentation(bytes: &[u8]) -> bool {
    let engine = Engine::new(&crate::engine_config());
    Module::new(&engine, bytes).is_ok_and(|m| m.imports().any(|i| i.module() == "gd_ui"))
}

fn rd_u32(d: &[u8], o: usize) -> Result<u32, String> {
    d.get(o..o + 4).map(|b| u32::from_le_bytes(b.try_into().unwrap())).ok_or_else(|| format!("read at {o:#x}: out of memory bounds"))
}

impl PresHost {
    pub fn new() -> PresHost {
        PresHost {
            engine: Engine::new(&crate::engine_config()),
            mods: Vec::new(),
            shared: Rc::new(RefCell::new(Shared::new())),
            exts: Vec::new(),
            timings: Timings { ns: Box::new([0; 4096]), n: 0 },
            stages: std::array::from_fn(|_| Timings { ns: Box::new([0; 4096]), n: 0 }),
            last_quads: 0,
            lab: None,
        }
    }

    /// `MELEE_WASM_MODS`: every presentation mod of the dir (None when off or there is none: zero cost).
    pub fn from_env() -> Result<Option<PresHost>, String> {
        let Some(dir) = crate::dir_from_env() else { return Ok(None) };
        let mut files: Vec<_> = std::fs::read_dir(&dir).map_err(|e| format!("{}: {e}", dir.display()))?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().is_some_and(|x| x.eq_ignore_ascii_case("wasm")))
            .collect();
        files.sort();
        let mut h = PresHost::new();
        for f in &files {
            let b = std::fs::read(f).map_err(|e| format!("{}: {e}", f.display()))?;
            if !is_presentation(&b) {
                continue;
            }
            if h.lab.is_none() && crate::lab::imports_lab(&b) {
                h.install_lab();
            }
            // the package dir: <dir>/<file stem>/ (textures), else the mods dir itself
            let stem = dir.join(f.file_stem().unwrap_or_default());
            let pkg = if stem.is_dir() { stem } else { dir.clone() };
            // one refused mod must not take the others down with it (a mods folder holds several: browser + lab)
            let i = match h.load(&b, pkg) {
                Ok(i) => i,
                Err(e) => {
                    eprintln!("modhost: ui: {} refused: {e} (build the mods with tools/rust-port/modhost/build_mods.sh, or cargo build --release inside rust/mods/<mod>)", f.display());
                    continue;
                }
            };
            let m = &h.mods[i];
            eprintln!("modhost: ui: loaded {} \"{}\" fnv64 {:016x} (features {:#x})", m.id, m.name, m.digest, m.features);
        }
        Ok((!h.mods.is_empty()).then_some(h))
    }

    /// Install the gd-lab-v1 extension (before the Lab mods load): the `gd_lab` imports and the shared Lab state.
    pub fn install_lab(&mut self) -> Rc<RefCell<crate::lab::LabShared>> {
        if let Some(l) = &self.lab {
            return l.clone();
        }
        let e = crate::lab::LabExt::new();
        let s = e.shared.clone();
        self.exts.push(Box::new(e));
        self.lab = Some(s.clone());
        s
    }

    /// Validate and instantiate one presentation module.
    pub fn load(&mut self, bytes: &[u8], pkg_dir: std::path::PathBuf) -> Result<usize, String> {
        if self.mods.len() >= MAX_UI_MODS {
            return Err(format!("ui mods loaded: limit {MAX_UI_MODS}"));
        }
        if crate::has_start_section(bytes) {
            return Err("start function: present (refuse)".into());
        }
        let module = Module::new(&self.engine, bytes).map_err(|e| format!("valid wasm: {e}"))?;
        let ext_mods: Vec<&str> = if self.exts.is_empty() { vec![] } else { vec!["gd_lab"] };
        for imp in module.imports() {
            if imp.module() != "gd_ui" && !ext_mods.contains(&imp.module()) {
                return Err(format!("import {}::{}: a presentation mod imports from gd_ui only", imp.module(), imp.name()));
            }
            if imp.ty().func().is_none() {
                return Err(format!("import gd_ui::{}: only functions may be imported", imp.name()));
            }
        }
        let mi = self.mods.len() as u16;
        let data = PresData { module: mi, ui: Some(self.shared.clone()), ext: self.exts.iter().find_map(|e| e.store_ext()), ..Default::default() };
        let mut store = Store::new(&self.engine, data);
        store.set_fuel(UI_FUEL).map_err(|e| e.to_string())?;
        let inst = linker(&self.engine, &self.exts)?
            .instantiate_and_start(&mut store, &module)
            .map_err(|e| format!("imports / instantiate: {e} (an import gd-ui-lab-v1.h declares but this host lacks is refused)"))?;
        let mem = inst.get_memory(&store, "memory").ok_or("memory: no exported `memory`")?;
        let mt = mem.ty(&store);
        if mt.maximum() != Some(mt.minimum()) || mt.minimum() > crate::MAX_PAGES {
            return Err(format!("memory: must be fixed (min = max) and at most {} pages", crate::MAX_PAGES));
        }
        let addr = |name: &str| -> Result<usize, String> {
            let g = inst.get_global(&store, name).ok_or_else(|| format!("export {name}: missing"))?;
            g.get(&store).i32().map(|v| v as u32 as usize).ok_or_else(|| format!("export {name}: not an i32 global"))
        };
        let d = mem.data(&store);
        let man = addr("gd_mod_manifest")?;
        let f = |i: usize| rd_u32(d, man + i * 4);
        if f(0)? != crate::MANIFEST_MAGIC || f(1)? != crate::MANIFEST_SIZE {
            return Err("manifest: magic / struct_size".into());
        }
        let (req, opt, count) = (f(6)?, f(7)?, f(9)?);
        if count != 0 {
            return Err("manifest hook_count: a gd_ui presentation mod has no sim hooks in v1 (load the hooks as a separate mod)".into());
        }
        let all = req | opt;
        if all & FEATURE_UI_HTML != 0 && req & FEATURE_UI_HTML != 0 {
            return Err("required_features: UI_HTML is reserved (refused by v1)".into());
        }
        if req & FEATURE_UI_VECTOR != 0 {
            return Err("required_features: UI_VECTOR needs Vello, which is not in this build".into());
        }
        if req & FEATURES_LAB != 0 && self.exts.is_empty() {
            return Err("required_features: LAB bits, but no Lab extension is installed".into());
        }
        if all & FEATURES_LAB != 0 {
            // B6: a LAB mod is presentation-only (hook_count 0 is checked above) and declares gdmod_lab_abi
            if all & FEATURES_LAB & !FEATURE_LAB != 0 && all & FEATURE_LAB == 0 {
                return Err("features: a LAB_* sub-area needs LAB".into());
            }
            let g = inst.get_global(&store, "gdmod_lab_abi").ok_or("export gdmod_lab_abi: missing (required with LAB bits)")?;
            let v = g.get(&store).i32().map(|v| v as u32).unwrap_or(0);
            let v = if v >> 16 == LAB_ABI_VERSION >> 16 { v } else { rd_u32(d, v as usize).unwrap_or(0) };
            if v >> 16 != LAB_ABI_VERSION >> 16 || v > LAB_ABI_VERSION {
                return Err(format!("gdmod_lab_abi: {v:#x} not satisfied by host {LAB_ABI_VERSION:#x}"));
            }
        }
        let abi_at = addr("gdmod_ui_abi").map_err(|e| format!("{e} (refuse)"))?;
        let abi = inst.get_global(&store, "gdmod_ui_abi").and_then(|g| g.get(&store).i32()).map(|v| v as u32).unwrap_or(rd_u32(d, abi_at).unwrap_or(0));
        // the global holds the version (or, from a C/Rust static, its address: then the version is in memory)
        let abi = if abi >> 16 == UI_ABI_VERSION >> 16 { abi } else { rd_u32(d, abi_at).unwrap_or(0) };
        if abi >> 16 != UI_ABI_VERSION >> 16 || abi > UI_ABI_VERSION {
            return Err(format!("gdmod_ui_abi: {abi:#x} not satisfied by host {UI_ABI_VERSION:#x}"));
        }
        let cstr = |o: usize, max: usize| -> String {
            let b = &d[o.min(d.len())..(o + max).min(d.len())];
            String::from_utf8_lossy(&b[..b.iter().position(|&c| c == 0).unwrap_or(b.len())]).into_owned()
        };
        let (id, name) = (cstr(man + 44, 32), cstr(man + 76, 48));
        if id.is_empty() || self.mods.iter().any(|m| m.id == id) {
            return Err(format!("manifest mod_id: {id:?} empty or already loaded"));
        }
        if all & FEATURE_UI_DRAW != 0 && inst.get_typed_func::<u32, ()>(&store, "gd_ui_frame").is_err() {
            return Err("export gd_ui_frame(u32): required with UI_DRAW".into());
        }
        let features = all & (FEATURES_UI | FEATURES_LAB) & !FEATURE_UI_HTML & !FEATURE_UI_VECTOR;
        store.data_mut().mem = Some(mem);
        store.data_mut().features = features;
        {
            let mut sh = self.shared.borrow_mut();
            sh.texs.push(Vec::new());
            sh.dirs.push(pkg_dir);
            sh.features.push(features);
        }
        let prof = if melee_core::frameprof::on() { melee_core::frameprof::mod_slot(&id) } else { 0 };
        let pm = PresMod {
            id,
            name,
            digest: crate::fnv64(bytes),
            features,
            disabled: false,
            frame: inst.get_typed_func(&store, "gd_ui_frame").ok(),
            event: inst.get_typed_func(&store, "gd_ui_event").ok(),
            enter: inst.get_typed_func(&store, "gd_ui_scene_enter").ok(),
            leave: inst.get_typed_func(&store, "gd_ui_scene_leave").ok(),
            prof,
            store,
        };
        let init: Option<TypedFunc<(), i32>> = inst.get_typed_func(&pm.store, "gd_ui_init").ok();
        self.mods.push(pm);
        let i = self.mods.len() - 1;
        if let Some(init) = init {
            let r = self.call(i, |s| init.call(s, ()));
            if r.is_some_and(|v| v != 0) {
                return Err(format!("gd_ui_init returned {}", r.unwrap()));
            }
        }
        self.drain_log(i);
        Ok(i)
    }

    /// Run a presentation call with a fresh fuel budget; a trap disables the mod (releases, pops, drops its nodes).
    fn call<R>(&mut self, i: usize, f: impl FnOnce(&mut Store<PresData>) -> Result<R, wasmi::Error>) -> Option<R> {
        let m = &mut self.mods[i];
        if m.disabled {
            return None;
        }
        let _ = m.store.set_fuel(UI_FUEL);
        let prof = melee_core::frameprof::on().then(std::time::Instant::now);
        let r = f(&mut m.store);
        if let Some(t) = prof {
            melee_core::frameprof::mod_ns(m.prof, t.elapsed().as_nanos() as u64);
        }
        match r {
            Ok(r) => Some(r),
            Err(e) => {
                eprintln!("modhost: ui: {}: trapped ({e}); its presentation instance is disabled, captures released, scenes popped", m.id);
                m.disabled = true;
                self.shared.borrow_mut().release_module(i as u16);
                None
            }
        }
    }

    fn drain_log(&mut self, i: usize) {
        let mut sh = self.shared.borrow_mut();
        for l in sh.log.drain(..) {
            eprintln!("modhost: ui: {}: {l}", self.mods[i].id);
        }
    }

    pub fn set_session(&mut self, s: Session) {
        let changed = { let mut sh = self.shared.borrow_mut(); let c = sh.session != s; sh.session = s; c };
        if changed {
            // a session change drops captures the new session forbids (INJECT and KEYBOARD outside the allowed ones)
            let mut sh = self.shared.borrow_mut();
            let keep_kb = matches!(s, Session::Offline | Session::LabNetplay);
            sh.caps.retain(|c| !(c.flags & CAP_INJECT != 0 && s != Session::Offline) && !(c.flags & CAP_KEYBOARD != 0 && !keep_kb));
            sh.inject = [None; 4];
            drop(sh);
            for e in &mut self.exts {
                e.on_session(s);
            }
        }
    }

    pub fn set_window(&mut self, w: u32, h: u32) {
        self.shared.borrow_mut().window = (w, h);
    }

    /// Whether a KEYBOARD capture is active (the platform then routes key events here instead of to the pads).
    /// A host overlay holds the pads (gd_ui_pad answers GD_ESTATE to every mod meanwhile).
    pub fn set_native_modal(&mut self, on: bool) {
        self.shared.borrow_mut().native_modal = on;
    }

    pub fn keyboard_wanted(&self) -> bool {
        self.shared.borrow().keyboard_wanted()
    }

    /// Key events from the platform for this present (kind << 24 | value).
    pub fn push_keys(&mut self, keys: &[u32]) {
        let mut sh = self.shared.borrow_mut();
        sh.keys.clear();
        if sh.keyboard_wanted() {
            sh.keys.extend_from_slice(keys);
        }
    }

    /// The retail scene the game is in (`mode`, scene kind) and whether it is a match: a scene with a replacer enters
    /// the custom scene instead (drawn on SCENE, retail GX off, input held from the game).
    pub fn retail_scene(&mut self, major: i32, minor: i32, in_match: bool) {
        let enter = {
            let mut sh = self.shared.borrow_mut();
            sh.in_match = in_match;
            if sh.retail == (major, minor) {
                return;
            }
            sh.retail = (major, minor);
            sh.replace.iter().find(|r| r.0 == (major, minor)).map(|r| r.1).or_else(|| {
                let envoy = unsafe { melee_modes::tu::envoy::ARMED != 0 && melee_modes::tu::envoy::CAMP_DUE != 0 };
                sh.replace_when.iter().find(|r| r.0 == (major, minor) && (r.2 == REPLACE_WHEN_ENVOY && envoy)).map(|r| r.1)
            })
        };
        if std::env::var_os("MELEE_UI_LOG").is_some() {
            let (a, d) = unsafe { (melee_modes::tu::envoy::ARMED, melee_modes::tu::envoy::CAMP_DUE) };
            eprintln!("gd_ui: retail scene ({major}, {minor}) -> custom {enter:?} (envoy armed {a} camp due {d}, stack {:?})", self.shared.borrow().stack);
        }
        if let Some(s) = enter {
            self.shared.borrow_mut().pending = Some(s as i64);
        }
    }

    /// The input boundary (A4): before `set_pads` / the rollback submit. Captured pads are recorded, injections
    /// applied (offline), exclusive ports zeroed for the game, then the Lab extensions run.
    pub fn input(&mut self, pads: &mut [PADStatus; 4], frame: &PresFrame) {
        {
            let mut sh = self.shared.borrow_mut();
            sh.caps.retain(|c| !c.released);
            for p in 0..4 {
                let b = pads[p].button;
                sh.edges[p] = b & !sh.prev_buttons[p];
                sh.prev_buttons[p] = b;
                sh.pads[p] = pads[p];
            }
            if sh.session == Session::Offline {
                for p in 0..4 {
                    if let Some(i) = sh.inject[p].take() {
                        let err = pads[p].err;
                        pads[p] = i;
                        pads[p].err = err;
                    }
                }
            } else {
                sh.inject = [None; 4];
            }
            let m = sh.masked_ports();
            for p in 0..4 {
                if m >> p & 1 != 0 {
                    let err = pads[p].err;
                    pads[p] = PADStatus::zeroed();
                    pads[p].err = err;
                }
            }
        }
        if !self.exts.is_empty() {
            let session = self.shared.borrow().session;
            let mut io = Boundary { pads, session };
            for e in &mut self.exts {
                e.before_logic(frame, &mut io);
            }
        }
    }

    fn scene_switch(&mut self) {
        // the Envoy door: the main menu's row or the browser chose Envoy (melee_modes::tu::envoy::DOOR_OPEN)
        unsafe {
            if melee_modes::tu::envoy::DOOR_OPEN != 0 {
                melee_modes::tu::envoy::DOOR_OPEN = 0;
                let mut sh = self.shared.borrow_mut();
                if sh.door != 0 && sh.pending.is_none() && !sh.stack.contains(&sh.door) {
                    sh.pending = Some(sh.door as i64);
                    eprintln!("envoy: camp opened");
                }
            }
        }
        let Some(p) = self.shared.borrow_mut().pending.take() else { return };
        let leave: Vec<u32> = {
            let sh = self.shared.borrow();
            match p {
                0 => sh.stack.iter().rev().copied().collect(),
                -1 => sh.stack.last().copied().into_iter().collect(),
                _ => vec![],
            }
        };
        for s in leave {
            if s == self.shared.borrow().door {
                eprintln!("envoy: camp closed");
            }
            let m = self.shared.borrow().scenes[s as usize - 1].module as usize;
            self.shared.borrow_mut().stack.pop();
            if let Some(f) = self.mods[m].leave {
                self.call(m, |st| f.call(st, s));
            }
        }
        if p > 0 {
            let s = p as u32;
            let m = self.shared.borrow().scenes[s as usize - 1].module as usize;
            if self.mods[m].disabled {
                return;
            }
            self.shared.borrow_mut().stack.push(s);
            if let Some(f) = self.mods[m].enter {
                self.call(m, |st| f.call(st, s));
            }
        }
    }

    /// One present (A3 call order): scene changes at the frame boundary, navigation and events, `gd_ui_frame`, the node
    /// trees; then the frame is sorted for the renderer ([`PresHost::frame`]).
    pub fn present(&mut self, frame: &PresFrame) {
        self.present_with(frame, |_, _| {});
    }

    /// [`PresHost::present`] plus host-native drawing (the port's own overlays, e.g. the settings overlay) into the
    /// same batch, after every mod and the node trees and before the sort: one frame, one draw call.
    pub fn present_with(&mut self, frame: &PresFrame, native: impl FnOnce(&mut Batch, &Atlas)) {
        let t0 = std::time::Instant::now();
        self.scene_switch();
        let n = self.mods.len();
        {
            let mut sh = self.shared.borrow_mut();
            sh.present_no = sh.present_no.wrapping_add(1);
            sh.batch.begin();
            // navigation from each mod's captured ports (OR of the pads it holds)
            for m in 0..n as u16 {
                let (mut btn, mut ed, mut sx, mut sy) = (0u16, 0u16, 0i8, 0i8);
                let mut any = false;
                for c in sh.caps.iter().filter(|c| c.module == m && !c.released) {
                    for p in 0..4 {
                        if c.ports >> p & 1 != 0 {
                            any = true;
                            btn |= sh.pads[p].button;
                            ed |= sh.edges[p];
                            if sh.pads[p].stickX.unsigned_abs() > sx.unsigned_abs() { sx = sh.pads[p].stickX }
                            if sh.pads[p].stickY.unsigned_abs() > sy.unsigned_abs() { sy = sh.pads[p].stickY }
                        }
                    }
                }
                if any {
                    sh.tree.navigate(m, btn, ed, sx, sy);
                }
            }
        }
        // events, then the frame
        let mut k = 0;
        loop {
            let ev = { let mut sh = self.shared.borrow_mut(); if k < sh.tree.events.len() { Some(sh.tree.events[k]) } else { sh.tree.events.clear(); None } };
            let Some((m, node, e, v)) = ev else { break };
            k += 1;
            if let Some(f) = self.mods[m as usize].event {
                self.call(m as usize, |st| f.call(st, (node, e, v)));
            }
        }
        let pno = self.shared.borrow().present_no;
        let t1 = std::time::Instant::now();
        melee_modes::tu::drive_draw::begin();
        self.shared.borrow_mut().submit_ns = 0;
        for i in 0..n {
            if let Some(f) = self.mods[i].frame {
                self.call(i, |st| f.call(st, pno));
            }
            if !self.shared.borrow().log.is_empty() {
                self.drain_log(i);
            }
        }
        melee_modes::tu::drive_draw::commit();
        let t2 = std::time::Instant::now();
        for e in &mut self.exts {
            e.after_logic(frame);
        }
        let t3 = std::time::Instant::now();
        {
            let mut sh = self.shared.borrow_mut();
            let sh = &mut *sh;
            sh.tree.layout();
            sh.tree.draw(&mut sh.batch, &sh.atlas, sh.theme);
            native(&mut sh.batch, &sh.atlas);
            let hide = !sh.stack.is_empty();
            sh.batch.finish(hide);
            self.last_quads = sh.batch.frame.quads.len();
        }
        let t4 = std::time::Instant::now();
        let ns = |a: std::time::Instant, b: std::time::Instant| (b - a).as_nanos().min(u32::MAX as u128) as u32;
        let sub = self.shared.borrow().submit_ns.min(u32::MAX as u64) as u32;
        self.stages[0].push(ns(t0, t1));
        self.stages[1].push(ns(t1, t2).saturating_sub(sub));
        self.stages[2].push(sub);
        self.stages[3].push(ns(t2, t3));
        self.stages[4].push(ns(t3, t4));
        self.timings.push(ns(t0, t4));
    }

    /// The sorted frame and the atlas, for `Renderer::set_ui`.
    pub fn with_frame<R>(&self, f: impl FnOnce(&melee_gx::ui::UiFrame, &mut Atlas) -> R) -> R {
        let mut sh = self.shared.borrow_mut();
        let sh = &mut *sh;
        f(&sh.batch.frame, &mut sh.atlas)
    }

    /// The Lab extensions' B4 overlays for this frame (GX draws appended to the game's frame).
    pub fn overlays(&mut self, frame: &PresFrame, out: &mut Vec<melee_gx::draw::DrawCmd>) {
        for e in &mut self.exts {
            e.overlay(frame, out);
        }
    }

    /// gd-ui-v1.2: whether a loaded (and live) mod declared UI_HOST, i.e. whether the flow needs to fill catalogs.
    pub fn wants_host(&self) -> bool {
        self.mods.iter().any(|m| !m.disabled && m.features & FEATURE_UI_HOST != 0)
    }

    /// gd-ui-v1.2: replace a catalog (records of NUL-separated fields); bumps its generation.
    pub fn set_list(&mut self, kind: usize, records: Vec<Vec<u8>>) {
        let mut sh = self.shared.borrow_mut();
        if kind < LIST_KINDS {
            sh.lists[kind] = records;
            sh.list_gen[kind] = sh.list_gen[kind].wrapping_add(1) & 0x7FFF;
        }
    }

    /// gd-ui-v1.3: picture keys the mods asked for this present that are not on hand (drained).
    pub fn take_pic_wants(&mut self) -> Vec<u32> {
        let mut sh = self.shared.borrow_mut();
        sh.pics.uploads = 0;
        std::mem::take(&mut sh.pics.wants)
    }

    /// gd-ui-v1.3: decoded RGBA (PIC_W x PIC_H) for a picture key; the oldest offer goes when PIC_READY_MAX are held.
    pub fn offer_pic(&mut self, key: u32, rgba: std::sync::Arc<Vec<u8>>) {
        let mut sh = self.shared.borrow_mut();
        let p = &mut sh.pics;
        p.ready.retain(|r| r.0 != key);
        if p.ready.len() >= PIC_READY_MAX {
            p.ready.pop_front();
        }
        p.ready.push_back((key, rgba));
    }

    /// gd-ui-v1.2: the requests the mods queued since the last call.
    pub fn take_requests(&mut self) -> Vec<Request> {
        let mut sh = self.shared.borrow_mut();
        if sh.requests.is_empty() {
            return Vec::new();
        }
        std::mem::take(&mut sh.requests)
    }

    /// The stats line: p50 / p99 / max µs per present over the last presents, quads, mods.
    pub fn stats_line(&self) -> String {
        let (a, b, c) = self.timings.pct();
        let mut st = String::new();
        for (name, t) in ["nav+events", "mod frame", "submit", "lab after", "layout+sort", "set_ui upload"].iter().zip(&self.stages) {
            let (a, b, c) = t.pct();
            st.push_str(&format!("; {name} p50 {a:.1} p99 {b:.1} max {c:.1}"));
        }
        format!("gd_ui: present p50 {a:.1} us p99 {b:.1} us max {c:.1} us over {} presents, {} quads last, {} mods (us{st})", self.timings.n.min(4096), self.last_quads, self.mods.len())
    }
}

impl Default for PresHost {
    fn default() -> Self {
        PresHost::new()
    }
}

/// [`InputBoundary`] over this frame's pads (A implements, B calls).
pub struct Boundary<'a> {
    pub pads: &'a mut [PADStatus; 4],
    pub session: Session,
}

impl InputBoundary for Boundary<'_> {
    fn read(&self, port: usize) -> PADStatus {
        self.pads.get(port).copied().unwrap_or_else(PADStatus::zeroed)
    }
    fn set(&mut self, port: usize, pad: PADStatus, _src: u32) {
        // offline, or an online Lab session where the caller has made it a synced lab command
        if port < 4 && self.session.lab_allowed() {
            self.pads[port] = pad;
        }
    }
}

#[cfg(test)]
mod tests;
