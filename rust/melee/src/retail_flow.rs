//! `melee play` on the retail scenes: the game's own mode loop (`gm_801A4510` -> `runGameMode` -> `gm_801A4014`,
//! `Engine::boot_modes` / `Engine::tick_modes`) runs from the boot on, so the opening movie, the title, the main menu,
//! the VS character select (`mnCharSel`), the stage select (`mnStageSel`), the match, the results (`gmResult`) and the
//! rules / name entry screens are the retail scenes, in the retail order. `MELEE_STANDIN_MENUS=1` keeps the older
//! stand-in flow (play.rs).
//!
//! Presets: `--stocks N` / `--time-secs N` / `--items F` are written into the saved VS rules (`GameRules` /
//! `GamePrefs`) the rules menu edits; `--match a,b/cpuN,..[,STAGE]` seeds the VS mode the way the C build's scene
//! launcher does (`SceneLaunch_SeedVs`: the first match starts at once), and after its results the retail CSS follows.
use melee_platform::gfx::{write_png, Presenter};
use melee_platform::pad::*;
use melee_platform::{Game, PadSystem};
use std::path::PathBuf;
use std::sync::Arc;
use winit::window::Window;

/// The match scene's frames run before the session takes the engine.
const ARM_FRAMES: u32 = 1;

#[path = "browser.rs"]
mod browser;

fn log(msg: &str) {
    eprintln!("play: {msg}");
}

pub fn scene_name(kind: u8) -> &'static str {
    match kind as u32 {
        0 => "title",
        1 => "menu",
        2 => "vs",
        3 => "suddendeath",
        5 => "results",
        8 => "css",
        9 => "sss",
        28 => "opening",
        40 => "progscan",
        42 => "memcard",
        _ => "scene",
    }
}

pub struct Presets {
    pub stocks: Option<i32>,
    pub time_secs: Option<i32>,
    /// 0 off, 1..5 very low..very high
    pub items: Option<i32>,
    pub scenario: Option<melee_engine::scenario::Scenario>,
}

pub struct RetailPlay {
    /// The practice aids (crate::practice), all off by default.
    pub practice: Option<crate::practice::Practice>,
    /// The in-game settings overlay (settings_overlay.rs; harness-owned, made on the first tick).
    pub settings: Option<crate::settings_overlay::Overlay>,
    hud_request: Option<bool>,
    /// Live video changes from the settings overlay: the pacer's (taken by the main loop) and the presenter's
    /// (applied at the next present). Presentation only.
    pacing: Option<melee_platform::main_loop::Pacing>,
    video: Vec<(String, String)>,
    iso: String,
    mods_dir: Option<String>,
    presets: Presets,
    e: Option<melee_engine::Engine>,
    frame: Option<Vec<melee_gx::draw::DrawCmd>>,
    /// The decode thread's packed vertex/index data for `frame` (async GX, see `boot`).
    packed: Option<melee_gx::render::Packed>,
    presenter: Option<Presenter>,
    /// The content aspect of `frame` (melee_engine view::aspect when it was taken).
    frame_aspect: f32,
    headless: bool,
    tick_no: u64,
    age: u32,
    menu_at: Option<(u8, u16)>,
    /// (mode, scene kind) of the current scene
    cur: (u8, u8),
    /// scenes entered so far (the shot index)
    scene_no: u32,
    matches: u32,
    quit_after: Option<u32>,
    pub(crate) max_frames: Option<u64>,
    shot_dir: Option<PathBuf>,
    shot_ages: Vec<u32>,
    shot_pending: Option<String>,
    quit: bool,
    pub failed: Option<String>,
    interp: bool,
    pub skip_intro: bool,
    /// Sound on (no --no-audio, audio.cfg on): melee_audio's sim, as the match (port/sfxparity), the menus' calls routed by name.
    pub audio_on: bool,
    audio: Option<crate::audio::Audio>,
    /// `--record FILE`: the replay being written (path, contents).
    pub rec: Option<(String, crate::play_net::Replay)>,
    /// `--replay FILE`: the replay being played back.
    pub feed: Option<crate::play_net::ReplayFeed>,
    /// `--netplay host|join ADDR`: the rollback peer.
    pub net: Option<crate::play_net::NetDriver>,
    /// `MELEE_PLAY_SYNCTEST=K`: SyncTest on every match frame.
    pub sync: Option<crate::play_net::PlaySync>,
    /// Savestates offered to a host (core v1.1 `state_*`): the match runs under a [`Hosted`] session.
    pub hosted: bool,
    pub hosted_g: Option<Hosted>,
    /// The run snapshots the engine (netplay, synctest, or a replay of such a run): frames are copied off the arena.
    pub rollback_mode: bool,
    /// Detached frame copies kept by content (see `play_net::Detacher`).
    detach: crate::play_net::Detacher,
    /// Offline input delay (practice parity, input_delay.rs): matches only, never under netplay / a replay feed.
    pub delay_line: crate::input_delay::DelayLine,
    hud_rates: crate::np_hud::HudRates,
    /// A match scene was entered: the session takes the engine at the next tick.
    arm: bool,
    /// gd-ui-v1 presentation mods (MELEE_WASM_MODS, modules importing gd_ui): harness-owned, never in the arena, so a
    /// savestate load or rollback never rewinds them. None with no UI mod: nothing runs and nothing is drawn.
    pub ui: Option<melee_modhost::ui::PresHost>,
    /// gd-lab-v1: the lab match (the engine while a Lab mod runs a match), its overlay painter, the clock of the
    /// offline commands and this tick's commands.
    pub lab: Option<crate::lab::LabRun>,
    lab_gfx: Option<(crate::ui::Ui, melee_gx::shim::GxShim)>,
    lab_clock: i32,
    lab_cmds: Vec<melee_modhost::lab::LabCmd>,
    /// How many of `lab_cmds` are the host's (they come first); the rest are the guest's.
    lab_cmds_host: usize,
    /// The lockstep frame of this tick's pads (online lab: the desync check's key).
    lab_frame: Option<u32>,
    /// The lab RNG's seed (the agreed netplay seed, else the test seed).
    pub lab_seed: u32,
    /// Launched as the LAB (`MELEE_SCENE=mode=lab..`, the browser's Modes / LAB relaunch): only then does a loaded Lab
    /// mod take the engine offline. Any other match with lab.wasm loaded is ordinary play (START pauses, no HUD).
    pub lab_launch: bool,
    /// MELEE_PRESENT_DUMP's count so far.
    dumped: u64,
    dump_buf: Vec<(std::path::PathBuf, (u32, u32, Vec<u8>))>,
    /// `--np-spectate`: the match host's spectate service (spectate.rs).
    pub spec_server: Option<crate::spectate::SpecServer>,
    /// `--spectate`: this run is a spectator; the feed's records arrive from the host.
    pub spec_client: Option<crate::spectate::SpecClient>,
    /// Replay playback controls (replay_ctl.rs).
    pub ctl: Option<crate::replay_ctl::Ctl>,
    rticks: u64,
    seek_base: Option<melee_engine::snapshot::Base>,
    seek_snaps: Vec<(usize, melee_engine::snapshot::SnapRef, u32, bool)>,
    /// gd-ui-v1.2 host side of the in-game browser (browser.rs).
    browser: browser::State,
    /// The settings overlay's SESSION exit picked this tick (true: main menu, false: quit), run after the step.
    exit_req: Option<bool>,
    /// MELEE_BOOT_MENU=1 (a "Return to main menu" relaunch): leave the boot's opening / title for the main menu.
    boot_menu: bool,
}

impl RetailPlay {
    pub fn new(iso: String, mods_dir: Option<String>, presets: Presets, headless: bool, shot_dir: Option<PathBuf>, shot_ages: Vec<u32>, quit_after: Option<u32>, max_frames: Option<u64>) -> RetailPlay {
        RetailPlay {
            iso, mods_dir, presets, e: None, frame: None, packed: None, presenter: None, frame_aspect: 4.0 / 3.0, headless, tick_no: 0, age: 0, menu_at: None, cur: (0xFF, 0xFF), scene_no: 0,
            matches: 0, quit_after, max_frames, shot_dir,
            shot_ages: if shot_ages.is_empty() { vec![90] } else { shot_ages },
            shot_pending: None, quit: false, failed: None, interp: false, skip_intro: false, audio_on: false, audio: None,
            rec: None, feed: None, net: None, sync: None, hosted: false, hosted_g: None, rollback_mode: false, detach: Default::default(), delay_line: crate::input_delay::DelayLine::new(0), hud_rates: Default::default(), arm: false, practice: None, settings: None, hud_request: None, pacing: None, video: Vec::new(),
            lab: None, lab_gfx: None, lab_clock: 0, lab_cmds: Vec::new(), lab_cmds_host: 0, lab_frame: None, lab_seed: 12345, lab_launch: false, dumped: 0, dump_buf: Vec::new(),
            spec_server: None, spec_client: None, ctl: None, rticks: 0, seek_base: None, seek_snaps: Vec::new(), browser: Default::default(), exit_req: None, boot_menu: std::env::var("MELEE_BOOT_MENU").is_ok_and(|v| v == "1"),
            ui: match melee_modhost::ui::PresHost::from_env() {
                Ok(h) => h,
                Err(e) => {
                    log(&format!("ui mods: {e}"));
                    None
                }
            },
        }
    }

    fn boot(&mut self) -> Result<melee_engine::Engine, String> {
        let mut margs: Vec<String> = self.mods_dir.iter().flat_map(|d| ["--mods".to_string(), d.clone()]).collect();
        margs.extend(["--iso".to_string(), self.iso.clone()]);
        crate::set_engine_mods(&margs)?;
        // HSD_SisLib_FontAtlas (main.dol) is loaded by Engine::boot_host.
        let t_boot = std::time::Instant::now();
        let disc = crate::engine_disc(&self.iso)?;
        let t_disc = t_boot.elapsed();
        unsafe {
            melee_engine::engine::RENDER = !self.headless;
            // a scene's first frame runs in the tick that enters it, as the C build's scene loop (frame-exact vs its shots)
            melee_engine::engine::ENTRY_RUNS_FRAME = true;
        }
        if self.audio_on {
            match crate::audio::Audio::boot(&self.iso, self.headless) {
                Ok(au) => {
                    // the TU hooks (post-link) and the scene's per-frame update, before the engine links
                    crate::engine_audio::attach(false);
                    unsafe { melee_modes::tu::ext_hook::AUDIO_ROUTE = Some(crate::engine_audio::route_by_name) };
                    self.audio = Some(au);
                }
                Err(e) => log(&format!("audio off: {e}")),
            }
        }
        // The boot mode (gmboot.c) is the memory-card scene, which the port skips with no card (the C build's unattended
        // runs: SceneLaunch_SkipMemcard); its bootOnLoad then picks the mode: the opening movie, or the title when the
        // intro is skipped. MELEE_RETAIL_FIRST=<GameModeKind> starts elsewhere.
        let boot_pick = if self.skip_intro { melee_types::GM_TITLE } else { melee_types::GM_OPENING_MV } as u8;
        let mut first = std::env::var("MELEE_RETAIL_FIRST").ok().and_then(|v| v.parse::<u8>().ok()).unwrap_or(boot_pick);
        // MELEE_SCENE="mode=classic;p1=fox;step=N;difficulty=D" (the C build's 1P scene launch, gw_runtime.c): the mode's
        // OnLoad skips its character select and starts at stage N
        if self.presets.scenario.is_none() {
            if let Ok(v) = std::env::var("MELEE_SCENE") {
                match melee_engine::scenario::Scenario::parse(&v) {
                    Ok(s) if s.onep_mode >= 0 => self.presets.scenario = Some(s),
                    // Geno's LAB (gw_sl_modes "lab": GM_LAB, its own mode row over VS's machinery). Its select screens
                    // are the port's kit screens, which the retail flow does not run: the scene launches its match only.
                    Ok(s) if s.game_mode == melee_modes::tu::geno_lab_mode::GM_LAB as i32 && matches!(s.entry_state, -1 | 2) => {
                        log("MELEE_SCENE: Geno LAB match (mode=lab)");
                        self.lab_launch = v.split(';').any(|kv| kv.trim().eq_ignore_ascii_case("mode=lab"));
                        self.presets.scenario = Some(s)
                    }
                    Ok(s) if s.game_mode == melee_modes::tu::geno_lab_mode::GM_LAB as i32 => log("MELEE_SCENE: mode=lab launches the LAB match only (its CSS/SSS are kit screens, not in the retail flow)"),
                    // a VS match straight from the boot (gw_runtime.c's VS scene launch, at=match): fighters by retail name,
                    // ck:, or an m-ex disc's own names / mexint:N / mexext:N (`p1=ultimatekirby`), resolved once the
                    // disc's MxDt.dat is read. Its CSS/SSS (at=css|sss) stay with the retail menus, so only the match.
                    Ok(s) if matches!(s.game_mode, -1 | 2) && matches!(s.entry_state, -1 | 2) => {
                        log("MELEE_SCENE: VS match (no menus)");
                        self.presets.scenario = Some(s)
                    }
                    Ok(_) => log("MELEE_SCENE: at=css|sss is the retail menus' job: only the match (at=match) launches this way"),
                    Err(e) => log(&format!("MELEE_SCENE: {e}")),
                }
            }
        }
        // The memory card (MELEE_CARD=0 turns it off; MELEE_CARD_PATH moves it): with it the boot mode is the real
        // memory-card scene (gmboot.c / gmscmemcard.c: load the save, or offer to create one), which then hands over
        // to the opening or the title. A scenario launch stays save-less (the C's scene launches skip the card).
        let mut card = false;
        if let Some(scn) = self.presets.scenario.take() {
            first = melee_engine::scenario::boot_mode(&scn);
            // `skipmemcard=0` opts a scene launch back into the card (gw_SceneLaunch_SkipMemcard)
            let skip = std::env::var("MELEE_SCENE").map(|v| melee_engine::scenario::skip_memcard(&v)).unwrap_or(-1);
            unsafe { melee_engine::engine::SKIP_MEMCARD = skip };
            card = skip == 0 && scn.onep_mode >= 0 && melee_engine::card::configured_base().is_some();
            if scn.onep_mode >= 0 {
                // as the C: the boot (memory-card) scene runs, skips its prompt and hands over to the mode
                unsafe { melee_engine::engine::BOOT_GAME_MODE = first as i32 };
                first = melee_types::GM_BOOT as u8;
            }
            melee_engine::scenario::install(scn);
        } else if melee_engine::card::configured_base().is_some() {
            card = true;
            if std::env::var_os("MELEE_RETAIL_FIRST").is_none() {
                first = melee_types::GM_BOOT as u8;
            }
        }
        unsafe {
            melee_engine::engine::CARD_ON = card;
            melee_engine::engine::BOOT_SKIP_INTRO = self.skip_intro;
        }
        let t_pre = t_boot.elapsed();
        let e = melee_engine::Engine::boot_modes(disc, 64 << 20, PadSystem::headless(), first)?;
        if std::env::var_os("MELEE_BOOT_TIMES").is_some() {
            let pn = melee_engine::archive::PARSE_NS.load(std::sync::atomic::Ordering::Relaxed);
            log(&format!("boot times: disc {:.1} ms, to engine {:.1} ms, engine boot {:.1} ms ({:.1} ms archive parse so far)", t_disc.as_secs_f64() * 1e3, t_pre.as_secs_f64() * 1e3, (t_boot.elapsed() - t_pre).as_secs_f64() * 1e3, pn as f64 / 1e6));
        }
        if self.audio.is_some() {
            // the menu TUs' sound hooks, over the seams' silent defaults (which the link above installs)
            crate::engine_audio_gen::install_menus();
        }
        self.apply_rules();
        // GX decode on its own thread, as play.rs does for its engine match (port/memfix): the windowed retail flow
        // never snapshots the arena (play turns trim on), so the thread's frames may hold its blocks. Without this the
        // FIFO decode and draw packing ran inside every tick (Temple 4P: tick p50 5.7 ms instead of ~2).
        // MELEE_GX_SYNC=1 keeps decoding in the tick; headless runs (golden, gate) never decode on a thread.
        if !self.headless && melee_engine::arena::trim() && std::env::var_os("MELEE_GX_SYNC").is_none() {
            melee_engine::gxbridge::shim().gx.enable_async(melee_engine::arena::mark_foreign_thread);
            // screenshots show exactly their tick: take each frame when it is decoded
            melee_engine::gxbridge::shim().wait_frame = self.shot_dir.is_some() && std::env::var_os("MELEE_SHOT_NOWAIT").is_none();
        }
        Ok(e)
    }

    /// The mix captured since the last call (a capture-mode audio: headless, or a core v1.1 host's).
    pub fn take_audio(&mut self) -> Vec<i16> {
        self.audio.as_mut().and_then(|a| a.capture.as_mut()).map(std::mem::take).unwrap_or_default()
    }

    /// The engine wherever it is (the retail flow's, or under the netplay / synctest session).
    fn engine(&self) -> Option<&melee_engine::Engine> {
        use crate::play_net::NetPhase;
        if let Some(e) = self.e.as_ref() {
            return Some(e);
        }
        if let Some(NetPhase::InMatch(g)) = self.net.as_ref().map(|n| &n.phase) {
            return Some(&g.e);
        }
        if let Some(l) = self.lab.as_ref() {
            return Some(&l.g.e);
        }
        if let Some(h) = self.hosted_g.as_ref() {
            return Some(&h.g.e);
        }
        self.sync.as_ref().and_then(|s| s.g.as_ref()).map(|g| &g.e)
    }

    /// One engine tick with these pads: the mode loop, or the match frame(s) under the netplay / synctest session.
    /// Returns whether a new scene was entered.
    fn step(&mut self, mut four: [melee_types::PADStatus; 4]) -> bool {
        use crate::play_net::*;
        if let Some(nd) = self.net.as_mut() {
            match nd.phase {
                NetPhase::InMatch(_) => {
                    // test pad scripts (css_pair.sh) are timed in render ticks; a START meant for the results that lands in
                    // the next match would pause it, so MELEE_NP_SCRIPT_NO_START=1 drops START in a match
                    if std::env::var("MELEE_NP_SCRIPT_NO_START").map_or(false, |v| v == "1") {
                        four[0].button &= !0x1000;
                    }
                    nd.tick_match(&four[0], self.rec.as_mut().map(|r| &mut r.1));
                    if let NetPhase::InMatch(g) = &mut nd.phase {
                        if let Some(f) = g.take_frame() {
                            self.frame = Some(f);
                            self.frame_aspect = melee_engine::view::aspect();
                        }
                    }
                    if let Some(err) = nd.error.clone() {
                        log(&nd.summary());
                        nd.write_hashes();
                        self.failed = Some(err);
                        self.quit = true;
                    } else if nd.end_frame.is_some() && !nd.lingering() {
                        log(&format!("netplay {}: match over at game frame {}", nd.tag(), nd.end_frame.unwrap() - melee_net::session::FIRST));
                        log(&nd.summary());
                        nd.write_hashes();
                        // --np-css: back to the menus in lockstep (results -> CSS -> the next match); else one match
                        let next = if nd.css() { NetPhase::PreMatch } else { NetPhase::Done };
                        if let Some(ls) = nd.ls.as_mut() {
                            ls.resume();
                        }
                        if let NetPhase::InMatch(g) = std::mem::replace(&mut nd.phase, next) {
                            let (e, _) = g.into_engine();
                            log(&format!("netplay {}: final hash {:08X}", nd.tag(), e.neutral_row()[0] | 1));
                            if let Some(r) = self.rec.as_mut() {
                                nd.flush_rec(&mut r.1, e.neutral_row()[0] | 1);
                            }
                            self.e = Some(e);
                        }
                        if !nd.css() {
                            self.quit = true;
                        }
                    }
                    return false;
                }
                NetPhase::PreMatch if nd.css() => {
                    // the menus in lockstep: this tick's local pad is scheduled; a frame runs once the peer's is in
                    let host = nd.host;
                    let lab_on = nd.lab;
                    let ls = nd.ls.as_mut().unwrap();
                    let ready = if lab_on {
                        // GAP-13: this peer's queued lab commands ride with this tick's pad (they apply on its frame)
                        let lab_sh = self.ui.as_ref().and_then(|u| u.lab.clone());
                        let mine: Vec<[u8; 16]> = lab_sh.as_ref().map_or(Vec::new(), |sh| {
                            let mut s = sh.borrow_mut();
                            let n = s.outbox.len().min(melee_modhost::lab::LAB_CMDS_PER_FRAME);
                            s.outbox.drain(..n).map(|c| c.to_bytes()).collect()
                        });
                        ls.push_local_cmds(&four[0], mine);
                        if let Some(sh) = &lab_sh {
                            let mut s = sh.borrow_mut();
                            s.apply_frame = ls.next_push_frame() as i32;
                            s.issuer = if host { 0 } else { 1 };
                            s.delay = ls.delay();
                        }
                        ls.poll();
                        ls.ready_lab().map(|(l, r, lc, rc, fr)| {
                            // the host's commands first on both peers
                            let (hc, gc) = if host { (lc, rc) } else { (rc, lc) };
                            self.lab_cmds_host = hc.iter().filter(|b| melee_modhost::lab::LabCmd::from_bytes(&b[..]).is_some()).count();
                            self.lab_cmds = hc.iter().chain(gc.iter()).filter_map(|b| melee_modhost::lab::LabCmd::from_bytes(b)).collect();
                            self.lab_frame = Some(fr);
                            (l, r)
                        })
                    } else {
                        ls.push_local(&four[0]);
                        ls.poll();
                        ls.ready()
                    };
                    match ready {
                        Some((l, r)) => {
                            four = neutral_pads();
                            four[0] = if host { l } else { r };
                            four[1] = if host { r } else { l };
                            four[0].err = 0;
                            four[1].err = 0;
                        }
                        None => {
                            if ls.silent_secs() > 20 {
                                self.failed = Some("netplay: connection lost: no packets from the other player for 20 s in the menus (they quit, or the network dropped)".into());
                                self.quit = true;
                            }
                            return false;
                        }
                    }
                }
                NetPhase::PreMatch => four = neutral_pads(),
                NetPhase::Done => {}
            }
        }
        if self.lab.is_some() {
            self.lab_step(four);
            return false;
        }
        if let Some(ps) = self.sync.as_mut() {
            if ps.g.is_some() {
                ps.tick(four);
                let g = ps.g.as_mut().unwrap();
                if let Some(f) = g.take_frame() {
                    self.frame = Some(f);
                    self.frame_aspect = melee_engine::view::aspect();
                }
                if let Some(r) = self.rec.as_mut() {
                    r.1.ticks.push((four, g.e.neutral_row()[0] | 1));
                }
                if self.tick_no % 600 == 0 {
                    let st = &ps.st;
                    let bad = st.mismatches.iter().filter(|m| m.3 != m.4).count();
                    log(&format!("synctest depth {} so far (match frame {}): {} passes, {} hash checks, {bad} neutral mismatches", st.k, g.e.frames, st.passes, st.checked));
                }
                if g.e.ended {
                    let st = &ps.st;
                    // the whole-state hash differs on a windowed run (a resimulated frame's display lists are not decoded, so
                    // the render pass leaves other bytes behind); the neutral (gameplay) hash is the one that must agree
                    let bad: Vec<_> = st.mismatches.iter().filter(|m| m.3 != m.4).collect();
                    log(&format!("synctest depth {}: {} passes, {} hash checks, {} NEUTRAL MISMATCHES ({} whole-state differences)", st.k, st.passes, st.checked, bad.len(), st.mismatches.len()));
                    for m in bad.iter().take(5) {
                        log(&format!("synctest:   mismatch frame {} full {:08x}/{:08x} neutral {:08x}/{:08x}", m.0, m.1, m.2, m.3, m.4));
                    }
                    if !bad.is_empty() {
                        self.failed = Some("synctest mismatches".into());
                    }
                    let (e, _) = ps.g.take().unwrap().into_engine();
                    self.e = Some(e);
                }
                return false;
            }
        }
        if let Some(h) = self.hosted_g.as_mut() {
            use melee_net::session::RbGame;
            // a frame run again after a load takes the pads it last ran with, unless the host gave others
            let next = h.g.frame() + 1;
            let pads = *h.inputs.entry(next).or_insert(four);
            h.g.step(pads);
            if let Some(f) = h.g.take_frame() {
                self.frame = Some(f);
                self.frame_aspect = melee_engine::view::aspect();
            }
            // each match tick once, as the plain path logs it (a re-run frame is not logged again)
            if h.g.e.frames > h.logged {
                h.logged = h.g.e.frames;
                hash_log(h.g.e.frames as u64, h.g.e.neutral_row()[0] | 1);
            }
            if h.g.e.ended {
                let h = self.hosted_g.take().unwrap();
                log(&format!("hosted savestates: match over, {} states kept", h.keep.len()));
                let (e, _) = h.g.into_engine();
                self.e = Some(e);
            }
            return false;
        }
        let e = self.e.as_mut().unwrap();
        // the first frame of a match scene: the session takes the engine from here on
        // (after the scene's first frames: the scene's first frame re-creates the host heap block, which a snapshot base
        // must not straddle)
        // (engine frames, not render ticks: a lockstep stall must not move the hand-over)
        if self.arm && !e.ended && e.mode_scene().1 == SCENE_VS && e.frames >= ARM_FRAMES {
            self.arm = false;
            if let Some(nd) = self.net.as_mut() {
                if matches!(nd.phase, NetPhase::PreMatch) && nd.room.is_none() {
                    if let Err(er) = nd.next_room() {
                        log(&format!("netplay: {er}"));
                        self.failed = Some(er);
                        self.quit = true;
                        return false;
                    }
                }
                if matches!(nd.phase, NetPhase::PreMatch) && nd.lab {
                    // LabNetplay: the match stays in lockstep; the lab takes the engine on both peers
                    if let Some(sh) = self.ui.as_ref().and_then(|u| u.lab.clone()) {
                        self.lab = Some(crate::lab::LabRun::new(ModesGame::new(self.e.take().unwrap()), sh, self.lab_seed));
                        log("lab: online lab match: the lab takes the engine (lockstep)");
                        self.lab_step(four);
                        return false;
                    }
                }
                if matches!(nd.phase, NetPhase::PreMatch) && !nd.lab {
                    nd.begin_match(self.e.take().unwrap());
                    return self.step(four);
                }
            } else if let Some(ps) = self.sync.as_mut() {
                ps.g = Some(Box::new(ModesGame::new(self.e.take().unwrap())));
                return self.step(four);
            } else if self.hosted && self.feed.is_none() {
                self.hosted_g = Some(Hosted { g: Box::new(ModesGame::new(self.e.take().unwrap())), keep: Default::default(), inputs: Default::default(), logged: 0 });
                log("hosted savestates: the match runs under the host's state calls");
                return self.step(four);
            } else if self.feed.is_none() && self.rollback_mode && self.lab_launch {
                if let Some(sh) = self.ui.as_ref().and_then(|u| u.lab.clone()) {
                    // offline Lab: the lab takes the engine for the match (snapshots need the rollback run modes)
                    self.lab = Some(crate::lab::LabRun::new(ModesGame::new(self.e.take().unwrap()), sh, self.lab_seed));
                    log("lab: lab match: the lab takes the engine");
                    self.lab_step(four);
                    return false;
                }
            }
        }
        let e = self.e.as_mut().unwrap();
        let vs_tick = !e.ended && e.mode_scene().1 == SCENE_VS;
        e.set_pads(Some(four));
        crate::slp_export::begin_tick(&four);
        let entered = e.tick_modes();
        crate::slp_export::end_tick(!e.ended && e.mode_scene().1 == SCENE_VS);
        let hash = if vs_tick { e.neutral_row()[0] | 1 } else { 0 };
        if vs_tick {
            hash_log(e.frames as u64, hash);
        }
        if let Some(r) = self.rec.as_mut() {
            r.1.ticks.push((four, hash));
        }
        if let Some(f) = self.feed.as_mut() {
            let i = f.at;
            f.check(i, hash);
            f.at += 1;
        }
        entered
    }

    /// One command-clock frame of the lab match (offline: this tick; online: the lockstep frame that just became ready).
    fn lab_step(&mut self, four: [melee_types::PADStatus; 4]) {
        let online = self.net.as_ref().is_some_and(|n| n.lab);
        let cmds: Vec<melee_modhost::lab::LabCmd> = if online {
            std::mem::take(&mut self.lab_cmds)
        } else {
            let sh = self.lab.as_ref().unwrap().sh.clone();
            let mut s = sh.borrow_mut();
            let n = s.outbox.len().min(melee_modhost::lab::LAB_CMDS_PER_FRAME);
            let v = s.outbox.drain(..n).collect();
            self.lab_clock += 1;
            s.apply_frame = self.lab_clock + 1;
            v
        };
        let lab = self.lab.as_mut().unwrap();
        {
            // who issued what this frame (gd_lab_applied: the other player's toasts); offline every command is ours
            let n_host = if online { self.lab_cmds_host.min(cmds.len()) } else { cmds.len() };
            let mut s = lab.sh.borrow_mut();
            s.applied.clear();
            s.applied.extend(cmds.iter().enumerate().map(|(i, c)| ((i >= n_host) as u8, *c)));
        }
        let ran = lab.tick(four, &cmds);
        let hash = if ran { lab.frame_hash() } else { 0 };
        if let (true, Some(fr)) = (online, self.lab_frame.take()) {
            if let Some(ls) = self.net.as_mut().and_then(|n| n.ls.as_mut()) {
                ls.record_hash(fr, hash);
                if fr % 600 == 0 && hash != 0 {
                    log(&format!("lab netplay: lockstep frame {fr} match frame {} hash {hash:08X} ({} checked, {} desyncs)", lab.f, ls.checked, ls.desyncs));
                }
            }
        }
        if let Some(r) = self.rec.as_mut() {
            r.1.ticks.push((four, hash));
        }
        if let Some(f) = lab.g.take_frame() {
            let mut f = f;
            let gfx = self.lab_gfx.get_or_insert_with(|| (crate::ui::Ui::new(), melee_gx::shim::GxShim::new()));
            gfx.0.clear();
            if lab.overlay(&mut gfx.0) && gfx.0.len() > 0 {
                gfx.0.flush(&mut gfx.1);
                gfx.1.copy_disp(true);
                if let Some(d) = gfx.1.take_frame() {
                    f.extend(d);
                }
            }
            self.frame = Some(f);
            self.frame_aspect = melee_engine::view::aspect();
        }
        if lab.g.e.ended {
            log(&lab.perf_line());
            if let Some(ls) = self.net.as_ref().and_then(|n| n.ls.as_ref()) {
                log(&format!("lab netplay: match over: {} hash checks, {} desyncs{}", ls.checked, ls.desyncs, ls.first_desync.map_or(String::new(), |f| format!(" (first at lockstep frame {f})"))));
                if ls.desyncs > 0 {
                    self.failed = Some("lab netplay desync".into());
                }
            }
            let lab = self.lab.take().unwrap();
            let (e, _) = lab.finish().into_engine();
            log(&format!("lab: match over, final hash {:08X}", e.neutral_row()[0] | 1));
            self.e = Some(e);
        }
    }

    fn tick_once(&mut self, pads: &mut PadSystem) {
        let _sampled = crate::prof::Gate::enter();
        if self.quit {
            return;
        }
        if self.engine().is_none() {
            match self.boot() {
                Ok(e) => self.e = Some(e),
                Err(err) => {
                    log(&format!("retail boot failed: {err}"));
                    self.failed = Some(err);
                    self.quit = true;
                    return;
                }
            }
        }
        let mut st = [pad_disconnected(); PAD_CHANMAX];
        pads.read(&mut st);
        if let Some(nd) = self.net.as_mut() {
            nd.pre_tick(pads);
        }
        let mut four = [melee_types::PADStatus::zeroed(); 4];
        four.copy_from_slice(&st[..4]);
        let live_pads = four;
        if let Some(f) = self.feed.as_mut() {
            if f.at >= f.r.ticks.len() {
                log(&f.summary());
                if f.bad > 0 || f.rebad > 0 {
                    self.failed = Some(format!("replay diverged at tick {}", f.first_bad.map_or(0, |b| b.0)));
                }
                self.quit = true;
                self.on_quit();
                return;
            }
            four = f.r.ticks[f.at].0;
        }
        let gameplay = matches!(self.cur.1, 2 | 3);
        four = self.delay_line.apply(four, gameplay && self.net.is_none() && self.feed.is_none() && self.sync.is_none());
        let in_vs = self.engine().is_some_and(|e| !e.ended && e.mode_scene().1 == crate::play_net::SCENE_VS);
        if let Some(pr) = self.practice.as_mut() {
            let _h = SysHeap::enter();
            pr.keys(pads);
            if self.net.is_some() || self.sync.is_some() || self.feed.is_some() {
                pr.forbid_helper("netplay / synctest / replay playback");
            }
            // the VS match only: player slot i is driven by port i in the retail VS mode
            if in_vs {
                pr.auto_lcancel(&mut four, |p| Some(p as i32));
            }
        }
        // gd-ui-v1 input boundary (A4): the same place as lcancel_helper, before set_pads and the rollback submit
        let pres_frame = melee_modhost::pres::PresFrame {
            session: if self.net.as_ref().is_some_and(|n| n.lab) { melee_modhost::pres::Session::LabNetplay } else if self.net.is_some() { melee_modhost::pres::Session::Netplay } else if self.sync.is_some() { melee_modhost::pres::Session::Synctest } else if self.feed.is_some() { melee_modhost::pres::Session::Replay } else { melee_modhost::pres::Session::Offline },
            scene: (self.cur.0 as i32, self.cur.1 as i32),
            match_frame: self.age as i32,
        };
        let overlay_keys = self.settings_input(pads, &live_pads, &mut four);
        if let Some(u) = self.ui.as_mut() {
            u.set_native_modal(self.settings.as_ref().is_some_and(|o| o.open));
            u.set_session(pres_frame.session);
            u.retail_scene(self.cur.0 as i32, self.cur.1 as i32, gameplay);
            // GAP-14: the keyboard goes to a KEYBOARD capture instead of the pads while one exists
            match (u.keyboard_wanted(), pads.ui_keys.as_mut()) {
                (true, Some(q)) => {
                    u.push_keys(q);
                    q.clear();
                }
                (true, None) => pads.ui_keys = Some(Vec::with_capacity(256)),
                (false, _) => {
                    u.push_keys(&[]);
                    if !overlay_keys {
                        pads.ui_keys = None;
                    }
                }
            }
            u.input(&mut four, &pres_frame);
        }
        let t_step = std::time::Instant::now();
        let entered = self.step(four);
        self.session_exit();
        if let Some(pr) = self.practice.as_mut() {
            let _h = SysHeap::enter();
            pr.observe();
        }
        let t_step = t_step.elapsed();
        if self.quit {
            self.on_quit();
            return;
        }
        // MELEE_MEM_EVERY=<ticks>: the play `mem after` line (and a heap-sampler section) every N ticks, for creep checks
        static MEM_EVERY: std::sync::OnceLock<Option<u64>> = std::sync::OnceLock::new();
        if let Some(n) = *MEM_EVERY.get_or_init(|| std::env::var("MELEE_MEM_EVERY").ok().and_then(|v| v.parse::<u64>().ok()).filter(|&n| n > 0)) {
            if self.tick_no % n == 0 {
                crate::play::mem_line(self.matches, &format!("tick {}", self.tick_no));
            }
        }
        if let Some(au) = self.audio.as_mut() {
            // a rollback run: the sound state is a static the load restores, so what it allocates must be in the arena too
            let _s = self.rollback_mode.then(melee_engine::arena::GameScope::enter);
            let t = std::time::Instant::now();
            au.tick();
            if std::env::var_os("MELEE_BOOT_TIMES").is_some() && (t_step + t.elapsed()).as_millis() >= 20 {
                log(&format!("slow tick {}: step {:.1} ms, audio {:.1} ms", self.tick_no, t_step.as_secs_f64() * 1e3, t.elapsed().as_secs_f64() * 1e3));
            }
        }
        if let Some(f) = melee_engine::gxbridge::take_frame() {
            // a frame no present showed (several ticks ran first): its EFB copies still have to happen
            if let (Some(old), Some(p)) = (self.frame.take(), self.presenter.as_mut()) {
                p.render_copies(&melee_engine::gxbridge::shim().gx.state, &old, self.packed.take());
            }
            if self.rollback_mode {
                // a snapshot load rewrites the arena: keep a detached copy (no Vec or Arc shared with the arena)
                self.frame = Some(self.detach.detach(f));
            } else {
                self.frame = Some(f);
                self.packed = melee_engine::gxbridge::shim().gx.packed.take();
            }
            if let (Some(pr), Some(f)) = (self.practice.as_mut(), self.frame.as_mut()) {
                let _h = SysHeap::enter();
                if pr.decorate(f) {
                    self.packed = None; // the decode thread's packing was for the undecorated draws
                }
            }
            // the view the frame was drawn for (its cameras' projection and the target shape must agree)
            self.frame_aspect = melee_engine::view::aspect();
        }
        // gd-ui-v1 present (A3 order: events, gd_ui_frame, node trees), after the logic frame
        match (self.ui.as_mut(), self.settings.as_mut()) {
            (Some(u), Some(o)) => u.present_with(&pres_frame, |b, a| o.draw(b, a)),
            (Some(u), None) => u.present(&pres_frame),
            (None, Some(o)) if self.headless && o.open => {
                // headless: build the frame anyway (the quad count, and MELEE_OVERLAY_SHOT's CPU raster)
                if let Some((f, a)) = o.own_frame() {
                    if let Ok(path) = std::env::var("MELEE_OVERLAY_SHOT") {
                        let (w, h, px) = crate::settings_overlay::raster(f, a);
                        if write_png(std::path::Path::new(&path), w, h, &px).is_ok() {
                            log(&format!("settings overlay: shot {path} ({} quads)", f.quads.len()));
                        }
                    }
                }
            }
            _ => {}
        }
        self.browser_tick();
        // MELEE_RETAIL_CSSDBG=N: every N ticks on the CSS, port 1's cursor (world units) and the four CPU levels
        if let Some(n) = std::env::var("MELEE_RETAIL_CSSDBG").ok().and_then(|v| v.parse::<u64>().ok()) {
            if self.cur.1 == melee_types::GS_CSS as u8 && n > 0 && self.tick_no % n == 0 {
                unsafe {
                    let c = melee_modes::tu::mncharsel::mnCharSel_804A0BC0[0];
                    let d = melee_modes::tu::mncharsel::mnCharSel_804D6CB0;
                    if !c.is_null() && !d.is_null() {
                        let lv: Vec<String> = (0..4).map(|i| format!("{}/{}/{}", (*d).vs.start.players[i].slot_type, (*d).vs.start.players[i].ckind, (*d).vs.start.players[i].cpu_level)).collect();
                        let mut knobs = Vec::new();
                        for i in 0..4 {
                            let mut j: *mut melee_types::HSD_JObj = core::ptr::null_mut();
                            let jt = melee_modes::tu::mncharsel::mnCharSel_803F0DFC.doors[i].cpuslider_joint as i32;
                            melee_runtime::tu::lbspdisplay::lb_80011E24(melee_modes::tu::mncharsel::mnCharSel_804D6CC0, &mut j, &[jt, -1]);
                            if !j.is_null() {
                                let mut p = melee_types::Vec3 { x: 0.0, y: 0.0, z: 0.0 };
                                melee_runtime::tu::lb_00B0::lb_8000B1CC(j as _, core::ptr::null_mut(), &mut p as *mut _ as _);
                                knobs.push(format!("{:.2},{:.2}", -2.9 + p.x, 1.7 + p.y));
                            }
                        }
                        log(&format!("css knobs {}", knobs.join(" ")));
                        log(&format!("css age {} cursor {:.2},{:.2} x5 {} x6 {} slots {}", self.age, (*c).xC, (*c).x10, (*c).x5, (*c).x6, lv.join(" ")));
                    }
                }
            }
        }
        // a once-a-second heartbeat (tick, scene, draw commands of the last frame): a stall shows as a stopped tick,
        // a render-only fault as a running tick with no draws
        if self.tick_no % 60 == 0 {
            log(&format!("heartbeat tick {} scene {} age {} draws {}", self.tick_no, scene_name(self.cur.1), self.age, self.frame.as_ref().map_or(0, |f| f.len())));
            if std::env::var_os("MELEE_RETAIL_DRAWDBG").is_some() {
                if let Some(f) = self.frame.as_ref() {
                    for (i, d) in f.iter().take(6).enumerate() {
                        log(&format!("  draw {i}: prim {:#x} verts {} vp {:?} sc {:?} pn {}", d.prim, d.vtx_count, d.render_viewport, d.render_scissor, d.current_pn_mtx));
                    }
                }
            }
        }
        self.tick_no += 1;
        self.age += 1;
        let Some(e) = self.engine() else { return };
        let now = e.mode_scene();
        if now.1 == 1 && std::env::var_os("MELEE_MENU_LOG").is_some() {
            // the main menu's cursor (mn_804A04F0): which list and entry a scripted run is on
            let m = unsafe { (melee_modes::tu::mnmain::mn_804A04F0.cur_menu, melee_modes::tu::mnmain::mn_804A04F0.hovered_selection) };
            if self.menu_at != Some(m) { self.menu_at = Some(m); log(&format!("tick {}: menu {} entry {}", self.tick_no, m.0, m.1)); }
        }
        if entered || now != self.cur {
            let prev = self.cur;
            self.cur = now;
            self.age = 0;
            self.scene_no += 1;
            // the 1P modes: the player's stocks left as the mode's record holds them (Classic keeps its record in the All-Star one)
            let onep = unsafe {
                match now.0 as u32 {
                    melee_types::GM_CLASSIC => Some((*melee_modes::tu::gm_17eb::gm_GetAllStarData()).x0.x0.stocks),
                    melee_types::GM_ADVENTURE => Some((*melee_modes::tu::gm_17e4::gm_GetAdventureData()).x0.x0.stocks),
                    melee_types::GM_ALLSTAR => Some((*core::ptr::addr_of!(melee_runtime::shared::gm_80473A18)).x0.x0.stocks),
                    _ => None,
                }
            };
            let onep = onep.map_or(String::new(), |n| format!(", 1P stocks {n}"));
            log(&format!("tick {}: scene {} (mode {}, scene kind {}, state {}{onep})", self.tick_no, scene_name(now.1), now.0, now.1, unsafe { melee_modes::tu::gm_1A3F::state_machine.routing.curr_state_id }));
            if now.0 == melee_types::GM_EVENT as u8 && std::env::var_os("MELEE_MENU_LOG").is_some() {
                // Event Match: which event (gmevent.c's level index, 0-based) the scene plays
                log(&format!("tick {}: event {}", self.tick_no, unsafe { (*melee_modes::tu::gmmain_lib::gmMainLib_804D3EE0).vs.unk_530.unk_535 }));
            }
            self.arm = now.1 == crate::play_net::SCENE_VS;
            if prev.1 == melee_types::GS_RESULTS as u8 {
                self.matches += 1;
                if self.quit_after.is_some_and(|n| self.matches >= n) {
                    log(&format!("quit after {} matches", self.matches));
                    self.quit = true;
                }
            }
        }
        if self.max_frames.is_some_and(|n| self.tick_no >= n) {
            log(&format!("max frames reached in {}", scene_name(self.cur.1)));
            self.quit = true;
        }
        self.on_quit();
        // MELEE_RETAIL_SHOT_TICKS="a-b,c": timeline shots (tick numbers from the first tick), for the transition captures
        if self.shot_dir.is_some() {
            if let Ok(v) = std::env::var("MELEE_RETAIL_SHOT_TICKS") {
                let t = self.tick_no as u64;
                let hit = v.split(',').any(|r| {
                    let mut it = r.split('-').map(|x| x.trim().parse::<u64>().unwrap_or(u64::MAX));
                    let a = it.next().unwrap_or(u64::MAX);
                    let b = it.next().unwrap_or(a);
                    t >= a && t <= b
                });
                if hit {
                    self.shot_pending = Some(format!("tl_{:05}", t));
                }
            }
        }
        if self.shot_dir.is_some() && self.shot_ages.contains(&self.age) {
            self.shot_pending = Some(format!("{:02}_{}_{:04}", self.scene_no, scene_name(self.cur.1), self.age));
        }
    }


    /// The settings overlay at the input boundary: open / navigate / mask, then the live changes. Returns whether it
    /// holds the keyboard (gd-ui key events).
    fn settings_input(&mut self, pads: &mut PadSystem, live: &[melee_types::PADStatus; 4], four: &mut [melee_types::PADStatus; 4]) -> bool {
        // harness state: everything the overlay allocates must be on the system heap, never in the arena a rollback
        // load rewrites (net.md's statics rule); the rollback tick may be inside the game scope here
        let _h = SysHeap::enter();
        self.settings_input_sys(pads, live, four)
    }

    fn settings_input_sys(&mut self, pads: &mut PadSystem, live: &[melee_types::PADStatus; 4], four: &mut [melee_types::PADStatus; 4]) -> bool {
        if self.settings.is_none() {
            log(&format!("settings overlay: ready (F1, or hold D-pad Down + Start){}", if melee_engine::arena::active() { " [arena!]" } else { "" }));
            if !crate::cfg::flag("settings_overlay", true) {
                return false;
            }
            let vol = self.audio.as_mut().map_or_else(melee_audio::output::load_volume, |a| a.volume());
            let mut o = crate::settings_overlay::Overlay::new(vol);
            if let Some(pr) = self.practice.as_ref() {
                o.set_value("input_display", pr.opts.input_display);
                o.set_value("lcancel_indicator", pr.opts.lcancel_indicator);
                o.set_value("no_screen_shake", pr.opts.no_screen_shake);
            }
            if let Some(sh) = crate::cosmetics::booted() {
                for (slot, v) in sh.lock().unwrap().verdict.iter() {
                    o.set_note(&format!("skin_{slot}"), match v { Ok(n) => format!("mounted: {n} joints match"), Err(e) => format!("refused: {e}") });
                }
            }
            self.settings = Some(o);
        }
        let o = self.settings.as_mut().unwrap();
        let f1 = pads.focused && pads.keys_down.contains(&winit::keyboard::KeyCode::F1);
        let keys: Vec<u32> = if o.open { pads.ui_keys.as_mut().map(std::mem::take).unwrap_or_default() } else { Vec::new() };
        let held = o.input(live, four, self.feed.is_none(), f1, &keys);
        if held && pads.ui_keys.is_none() {
            pads.ui_keys = Some(Vec::with_capacity(64));
        } else if !held && self.ui.as_ref().is_none_or(|u| !u.keyboard_wanted()) {
            pads.ui_keys = None;
        }
        let changes = o.take_changes();
        let offline = self.net.is_none() && !self.rollback_mode;
        for c in changes.into_iter().filter(|c| c.live) {
            let on = c.value == "1";
            match c.key.as_str() {
                "volume" => {
                    let v: i32 = c.value.parse().unwrap_or(100);
                    if let Some(a) = self.audio.as_mut() {
                        a.set_volume(v);
                    }
                    if std::env::var_os("MELEE_VOLUME").is_none() {
                        let _ = melee_audio::output::save_volume(v);
                    }
                }
                "action:menu" => self.exit_req = Some(true),
                "action:quit" => self.exit_req = Some(false),
                "overlay" => self.hud_request = Some(on),
                "vsync" | "fps_cap" | "interp" => {
                    let p = self.pacing.get_or_insert_with(Default::default);
                    match c.key.as_str() {
                        "vsync" => {
                            p.vsync = Some(on);
                            self.video.push((c.key.clone(), c.value.clone()));
                        }
                        "fps_cap" => p.fps_cap = Some(c.value.parse().unwrap_or(0.0)),
                        _ => p.interp = Some(match c.value.as_str() { "on" => Some(true), "off" => Some(false), _ => None }),
                    }
                }
                "interp_mode" | "render_scale" => self.video.push((c.key.clone(), c.value.clone())),
                "widescreen" => {
                    // renderer-side widescreen only (the camera mode is fixed for the run): the game's view hooks keep
                    // answering 4:3, the presenter widens, the next frame takes the new content aspect
                    melee_engine::view::set_widescreen(on);
                    self.video.push((c.key.clone(), c.value.clone()));
                }
                "input_display" | "lcancel_indicator" | "no_screen_shake" => {
                    if let Some(pr) = self.practice.as_mut() {
                        let mut op = pr.opts;
                        match c.key.as_str() {
                            "input_display" => op.input_display = on,
                            "lcancel_indicator" => op.lcancel_indicator = on,
                            _ => op.no_screen_shake = on,
                        }
                        pr.set_opts(op);
                    }
                }
                k if k.starts_with("portrait_") => {
                    // render-only (texture overrides): live everywhere, online included
                    let disc = &mut melee_engine::host::Host::get().disc;
                    let r = crate::cosmetics_import::apply_pictures(&mut |f| disc.read_vanilla(f).ok());
                    log(&format!("cosmetics: {k} = {} ({r})", c.value));
                    self.settings.as_mut().unwrap().set_note(k, r);
                }
                k if k.starts_with("skin_") => {
                    let slot = k[5..].to_string();
                    let o = self.settings.as_mut().unwrap();
                    // live only offline and only when the boot mounted the cosmetics overlay (it answers the slots)
                    let Some(sh) = crate::cosmetics::booted().filter(|_| offline) else {
                        o.set_note(k, "saved: applies at the next launch".into());
                        continue;
                    };
                    let path = if c.value == "none" { None } else { crate::cosmetics::list(&crate::cosmetics::dir()).into_iter().find(|s| s.slot == slot && s.name == c.value).map(|s| s.path) };
                    let disc = &mut melee_engine::host::Host::get().disc;
                    let r = crate::cosmetics::mount(&sh, &mut |f| disc.read_vanilla(f).ok(), &slot, path.as_deref());
                    disc.forget_replaced(&crate::cosmetics::slot_file(&slot));
                    let note = match (&r, path.is_some()) {
                        (_, false) => "vanilla from the next match".to_string(),
                        (Ok(n), _) => format!("mounted ({n} joints match): next match"),
                        (Err(e), _) => format!("refused: {e}"),
                    };
                    log(&format!("cosmetics: {} -> {} ({note})", crate::cosmetics::slot_file(&slot), c.value));
                    let pr = crate::cosmetics_import::apply_pictures(&mut |f| disc.read_vanilla(f).ok());
                    log(&format!("cosmetics: pictures: {pr}"));
                    o.set_note(k, note);
                }
                _ => {}
            }
        }
        held
    }

    /// The match host's spectate service: this tick's confirmed records out.
    fn spec_serve(&mut self) {
        if let (Some(ss), Some(r)) = (self.spec_server.as_mut(), self.rec.as_ref()) {
            let done = self.quit || self.net.as_ref().is_some_and(|n| matches!(n.phase, crate::play_net::NetPhase::Done));
            ss.update(&r.1, done);
        }
    }

    /// A replay / spectator render tick: the stream in, the controls, then 0..n feed ticks.
    fn tick_playback(&mut self, pads: &mut PadSystem) {
        if self.quit {
            return;
        }
        if let (Some(sc), Some(f)) = (self.spec_client.as_mut(), self.feed.as_mut()) {
            sc.poll(&mut f.r.ticks);
            if sc.lost {
                log(&f.summary());
                self.failed = Some("spectate: the stream stopped".into());
                self.quit = true;
                self.on_quit();
                return;
            }
        }
        self.rticks += 1;
        let mut n = 1usize;
        let mut ff = false;
        if let Some(c) = self.ctl.as_mut() {
            c.on_tick(self.rticks);
            if self.ui.is_none() && !self.headless {
                match pads.ui_keys.as_mut() {
                    Some(q) => {
                        c.keys(q);
                        q.clear();
                    }
                    None => pads.ui_keys = Some(Vec::with_capacity(64)),
                }
            }
        }
        self.do_seek();
        if let Some(c) = self.ctl.as_mut() {
            if c.ff_to.is_some() {
                ff = true;
                n = if self.headless { usize::MAX } else { 240 };
            } else {
                n = c.steps();
            }
        }
        if let (Some(sc), Some(f)) = (self.spec_client.as_ref(), self.feed.as_ref()) {
            if !sc.ended() {
                // the delay buffer: never closer than `delay` ticks to the newest record
                let ready = f.r.ticks.len().saturating_sub(sc.delay);
                let behind = ready.saturating_sub(f.at);
                n = if behind > 2 { n.max(behind.min(sc.catch_up)) } else { n.min(behind) };
                if behind > 2 * sc.catch_up && self.rticks % 60 == 0 {
                    log(&format!("spectate: catching up, {behind} ticks behind"));
                }
            }
        }
        let mut k = 0;
        while k < n {
            if let (true, Some(c), Some(f)) = (ff, self.ctl.as_mut(), self.feed.as_ref()) {
                if c.ff_to.is_some_and(|t| f.at >= t || f.at >= f.r.ticks.len()) {
                    c.ff_to = None;
                    log(&format!("replay ctl: at tick {} ({})", f.at, c.state()));
                    break;
                }
            }
            if let Some(c) = self.ctl.as_ref() {
                if self.engine().is_some_and(|e| !e.ended && e.mode_scene().1 == crate::play_net::SCENE_VS) {
                    c.apply_camera();
                }
            }
            self.tick_once(pads);
            self.seek_snapshot();
            k += 1;
            if self.quit {
                break;
            }
        }
    }

    /// A seekable playback: a snapshot of the VS match every `snap_every` feed ticks (the base lives for one match).
    fn seek_snapshot(&mut self) {
        let Some(c) = self.ctl.as_ref() else { return };
        if !self.rollback_mode {
            return;
        }
        let every = c.snap_every;
        let in_vs = self.e.as_ref().is_some_and(|e| !e.ended && e.mode_scene().1 == crate::play_net::SCENE_VS && e.frames > ARM_FRAMES);
        if !in_vs {
            if self.seek_base.is_some() {
                self.seek_snaps.clear();
                self.seek_base = None;
            }
            return;
        }
        let Some(f) = self.feed.as_ref() else { return };
        let at = f.at;
        let due = self.seek_snaps.is_empty() || (at % every == 0 && !self.seek_snaps.iter().any(|s| s.0 == at));
        if !due {
            return;
        }
        let base = self.seek_base.get_or_insert_with(melee_engine::snapshot::Base::capture);
        let e = self.e.as_ref().unwrap();
        let snap = base.save();
        self.seek_snaps.push((at, snap, e.frames, e.ended));
        self.seek_snaps.sort_by_key(|s| s.0);
    }

    /// A pending seek: forward = play on unpaced; back = load the nearest snapshot at or before it, then forward.
    fn do_seek(&mut self) {
        use crate::replay_ctl::Seek;
        let (Some(c), Some(f)) = (self.ctl.as_mut(), self.feed.as_mut()) else { return };
        let Some(s) = c.seek.take() else { return };
        let at = f.at;
        let len = f.r.ticks.len();
        let target = match s {
            Seek::Tick(t) => t,
            Seek::Rel(d) => (at as i64 + d).max(0) as usize,
            Seek::First => self.seek_snaps.first().map_or(at, |x| x.0),
            Seek::MatchFrame(m) => match f.r.ticks.iter().enumerate().filter(|t| t.1 .1 != 0).nth(m as usize) {
                Some((i, _)) => i + 1,
                None => len,
            },
        }
        .min(len);
        if target >= at {
            c.ff_to = Some(target);
            log(&format!("replay ctl: seek to tick {target} (forward from {at})"));
            return;
        }
        let Some(sn) = self.seek_snaps.iter().rev().find(|x| x.0 <= target).cloned() else {
            let why = if !self.rollback_mode {
                "this playback is not seekable".to_string()
            } else if self.seek_snaps.is_empty() {
                "snapshots cover the VS match only".to_string()
            } else {
                format!("this match's snapshots start at tick {}", self.seek_snaps[0].0)
            };
            log(&format!("replay ctl: seek to tick {target}: no snapshot at or before it ({why}); staying at {at}"));
            return;
        };
        let (Some(base), Some(e)) = (self.seek_base.as_ref(), self.e.as_mut()) else { return };
        base.load(&sn.1);
        e.frames = sn.2;
        e.ended = sn.3;
        f.at = sn.0;
        c.ff_to = Some(target);
        log(&format!("replay ctl: seek to tick {target}: snapshot of tick {} loaded (from {at})", sn.0));
    }

    /// The run ends: the replay is written, a failure is noted for the exit code.
    fn on_quit(&mut self) {
        if !self.quit {
            return;
        }
        if let (Some(ss), Some(r)) = (self.spec_server.as_mut(), self.rec.as_ref()) {
            ss.drain(&r.1, 8);
        }
        if let Some(sc) = self.spec_client.as_ref() {
            sc.bye();
        }
        self.save_replay();
        crate::slp_export::finish();
        if let Some(f) = &self.failed {
            log(&format!("failed: {f}"));
            crate::play_net::RUN_FAILED.store(true, std::sync::atomic::Ordering::Relaxed);
        }
    }

    /// Write the `--record` replay (at the end of the run).
    pub fn save_replay(&mut self) {
        if let Some((p, r)) = self.rec.take() {
            if p.is_empty() {
                return; // kept only for the spectators
            }
            match r.write(&p) {
                Ok(()) => {
                    log(&format!("replay: {} ticks written to {p}", r.ticks.len()));
                    crate::play_net::slp_from_replay(&p, &self.iso);
                }
                Err(e) => log(&format!("replay: {e}")),
            }
        }
    }

    /// The presets into the saved VS rules (what the rules menu edits and `gm_InitVsMode` reads).
    fn apply_rules(&self) {
        let p = &self.presets;
        unsafe {
            let gr = melee_modes::tu::gmmain_lib::gmMainLib_GetGameRules();
            let gp = melee_modes::tu::gmmain_lib::gmMainLib_GetGamePrefs();
            if let Some(n) = p.stocks {
                (*gr).mode = 1;
                (*gr).stock_count = n.clamp(1, 99) as u8;
            }
            if let Some(s) = p.time_secs {
                if p.stocks.is_none() {
                    (*gr).mode = 0;
                }
                let minutes = ((s.max(1) + 59) / 60) as u8;
                (*gr).time_limit = minutes;
                (*gr).stock_time_limit = minutes;
            }
            if let Some(i) = p.items {
                (*gp).item_freq = if i == 0 { 0xFF } else { (i - 1) as u8 };
            }
        }
    }

    fn emit_shot(&mut self) {
        let (Some(name), Some(dir), Some(p)) = (self.shot_pending.take(), self.shot_dir.as_ref(), self.presenter.as_mut()) else { return };
        // with UI mods the shot is the window as shown (window-space panels sit outside the EFB)
        let (w, h, px) = if self.ui.is_some() { p.screenshot_window() } else { (p.renderer.width, p.renderer.height, p.screenshot()) };
        let path = dir.join(format!("retail_{name}.png"));
        match write_png(&path, w, h, &px) {
            Ok(()) => log(&format!("screenshot {}", path.display())),
            Err(e) => log(&format!("screenshot: {e}")),
        }
    }
}

impl RetailPlay {
    fn present_inner(&mut self, window: &Arc<Window>, alpha: f64) {
        if self.presenter.is_none() {
            match Presenter::new(window.clone()) {
                Ok(mut p) => {
                    melee_engine::gxbridge::shim().gx.set_pipe_factory(p.renderer.pipe_factory());
                    p.offthread_drop = Some(|q| !melee_engine::arena::contains(q));
                    let want = melee_gx::render::Renderer::env_msaa();
                    if p.renderer.samples != want {
                        log(&format!("video: MSAA {want}x is not supported for the EFB formats on this GPU: clamped to {}x", p.renderer.samples));
                    }
                    if let Some(o) = self.settings.as_mut() {
                        o.refresh_msaa();
                    }
                    self.presenter = Some(p)
                }
                Err(e) => {
                    self.failed = Some(e);
                    self.quit = true;
                    return;
                }
            }
        }
        // A pending screenshot is of the newest sim frame, never an interpolated one (unless MELEE_SHOT_PHASE asks for
        // an in-between: the new frame drawn at that phase between it and the previous tick's)
        let shot_phase = std::env::var("MELEE_SHOT_PHASE").ok().and_then(|v| v.parse::<f64>().ok());
        let a = if self.shot_pending.is_some() { shot_phase.filter(|_| self.interp) } else { self.interp.then_some(alpha) };
        if let Some(p) = self.presenter.as_mut() {
            p.force_phase = if self.shot_pending.is_some() { a } else { None };
            p.shot = self.shot_pending.is_some();
        }
        let frame = self.frame.take();
        let mut pose = if frame.is_some() { melee_engine::gxbridge::shim().gx.pose.take() } else { None };
        if self.rollback_mode {
            // the pose record lives in the arena: a snapshot load (lab load / rewind, rollback) rewrites it while the
            // interpolator still holds the previous one, so the presenter gets a system-heap copy (as frames are
            // detached, play_net::Detacher) and the arena original is freed inside the game scope
            pose = pose.map(|a| {
                let c = std::sync::Arc::new((*a).clone());
                let _s = melee_engine::arena::GameScope::enter();
                drop(a);
                c
            });
        }
        let p = self.presenter.as_mut().unwrap();
        for (k, v) in self.video.drain(..) {
            match k.as_str() {
                "vsync" => p.set_vsync(v == "1"),
                "render_scale" => p.set_render_scale(v.parse().ok()),
                "interp_mode" => p.set_interp_mode(melee_gx::interp::Mode::parse(&v).unwrap_or_default()),
                "widescreen" => p.set_true_widescreen(v == "1"),
                _ => {}
            }
            log(&format!("video: {k} = {v} (live)"));
        }
        p.set_match_scene(melee_engine::view::scene_wide(melee_engine::view::scene()));
        if let Some(pk) = self.packed.take() {
            p.renderer.set_packed(pk);
        }
        p.set_content_aspect(self.frame_aspect);
        if let Some(u) = self.ui.as_mut() {
            let (w, h) = p.size();
            u.set_window(w, h);
            let t = std::time::Instant::now();
            u.with_frame(|f, a| p.renderer.set_ui(Some(f), a));
            u.stages[5].push(t.elapsed().as_nanos().min(u32::MAX as u128) as u32);
        } else if let Some(o) = self.settings.as_mut() {
            // the overlay alone: its own batch (nothing at all while it stays closed; one empty frame after it closes)
            if let Some((f, a)) = o.own_frame() {
                p.renderer.set_ui(Some(f), a);
            }
        }
        p.present_posed(&melee_engine::gxbridge::shim().gx.state, frame, pose, a);
        // MELEE_PRESENT_DUMP="TICK,N": the window as shown for N consecutive presents from tick TICK, in-between
        // (interpolated) presents included: present_<tick>_<k>_<alpha>.png in the shot dir (interpolation evidence)
        if let (Some(dir), Ok(v)) = (self.shot_dir.clone(), std::env::var("MELEE_PRESENT_DUMP")) {
            let mut it = v.split(',').filter_map(|x| x.trim().parse::<u64>().ok());
            if let (Some(t0), Some(n)) = (it.next(), it.next()) {
                if self.tick_no as u64 >= t0 && self.dumped < n {
                    // kept in memory until the last one (a PNG per present would stretch the presents apart)
                    let shot = p.screenshot_window();
                    let path = dir.join(format!("present_{:05}_{:04}_{:.3}.png", self.tick_no, self.dumped, a.unwrap_or(-1.0)));
                    self.dump_buf.push((path, shot));
                    self.dumped += 1;
                    if self.dumped == n {
                        for (path, (w, h, px)) in self.dump_buf.drain(..) {
                            let _ = write_png(&path, w, h, &px);
                        }
                    }
                }
            }
        }
        self.emit_shot();
        if let Some(e) = self.presenter.as_ref().and_then(|p| p.renderer.gpu_fault()) {
            self.failed = Some(format!("GPU error, stopping: {e}"));
            self.quit = true;
        }
    }
}

impl Game for RetailPlay {
    fn tick(&mut self, pads: &mut PadSystem) {
        if self.feed.is_none() {
            self.tick_once(pads);
            self.spec_serve();
            return;
        }
        self.tick_playback(pads);
    }

    fn set_interp(&mut self, on: bool) {
        self.interp = on;
        melee_runtime::pose_capture::ON.store(on, std::sync::atomic::Ordering::Relaxed);
    }

    fn present(&mut self, window: &Arc<Window>, alpha: f64) {
        let t_present = std::time::Instant::now();
        self.present_inner(window, alpha);
        // MELEE_PRESENT_SPIKE=<ms>: one line per present at least that long, with the presenter's stages (frame-time tail)
        static SPIKE: std::sync::OnceLock<Option<f64>> = std::sync::OnceLock::new();
        if let (Some(ms), Some(p)) = (*SPIKE.get_or_init(|| std::env::var("MELEE_PRESENT_SPIKE").ok().and_then(|v| v.parse().ok())), self.presenter.as_ref()) {
            let el = t_present.elapsed().as_secs_f64() * 1e3;
            if el >= ms {
                log(&format!("slow present at tick {}: {el:.2} ms (prepare {:.2} draw {:.2} swap {:.2} adopt {:.2} (drop {:.2}) ms, fresh {})", self.tick_no, p.last[0] as f64 / 1e3, p.last[1] as f64 / 1e3, p.last[2] as f64 / 1e3, p.adopt_us as f64 / 1e3, p.drop_us as f64 / 1e3, p.last_fresh));
            }
        }
    }
    fn resized(&mut self, w: u32, h: u32) {
        melee_engine::view::set_window(w, h);
        if let Some(p) = self.presenter.as_mut() {
            p.resize(w, h);
        }
    }

    fn should_quit(&self) -> bool {
        self.quit
    }

    fn present_now(&self) -> bool {
        self.shot_pending.is_some()
    }

    fn take_hud_request(&mut self) -> Option<bool> {
        self.hud_request.take()
    }

    fn take_pacing(&mut self) -> Option<melee_platform::main_loop::Pacing> {
        self.pacing.take()
    }

    fn status_line(&mut self) -> Option<String> {
        self.net.as_mut().map(|n| n.status())
    }

    fn display_stats(&mut self) -> Option<(u64, u64, f64)> {
        self.presenter.as_ref().and_then(|p| p.frame_stats())
    }

    fn set_hud(&mut self, lines: Option<Vec<melee_platform::overlay::Line>>) {
        if let Some(p) = self.presenter.as_mut() {
            p.hud = lines;
        }
    }

    fn overlay_lines(&mut self) -> Vec<melee_platform::overlay::Line> {
        let Some(n) = self.net.as_ref() else {
            let d = self.delay_line.frames;
            return vec![(if d > 0 { format!("offline input delay {d}f") } else { "offline".into() }, melee_platform::overlay::DIM)];
        };
        let h = n.hud();
        self.hud_rates.update(&h);
        crate::np_hud::lines(&h, &self.hud_rates)
    }
}

/// Harness code that allocates (the practice aids, the settings overlay) runs on the system heap: the rollback tick
/// can be inside the game scope, and a harness-owned value holding arena blocks is corrupted by the next snapshot
/// load (net.md's statics rule). Seen as `execute at 0x0` crashes of headless netplay peers with the input display on.
struct SysHeap(bool);

impl SysHeap {
    fn enter() -> SysHeap {
        let was = melee_engine::arena::active();
        if was {
            static ONCE: std::sync::Once = std::sync::Once::new();
            ONCE.call_once(|| eprintln!("play: harness code entered with the arena active; switched to the system heap"));
            melee_engine::arena::set_active(false);
        }
        SysHeap(was)
    }
}

impl Drop for SysHeap {
    fn drop(&mut self) {
        if self.0 {
            melee_engine::arena::set_active(true);
        }
    }
}

pub fn run(mut rp: RetailPlay, headless: bool) -> std::process::ExitCode {
    use std::process::ExitCode;
    #[cfg(feature = "corev1")]
    if crate::corev1::active() {
        // the host presents and paces (corev1.rs)
        crate::corev1::drive(&mut rp);
        return if rp.failed.is_some() { ExitCode::FAILURE } else { ExitCode::SUCCESS };
    }
    if headless {
        let mut pads = PadSystem::headless();
        let limit = rp.max_frames.unwrap_or(60 * 600);
        let ticks = if rp.net.is_some() || rp.spec_client.is_some() {
            // netplay is real time: one tick per 1/60 s
            let mut n = 0u64;
            let t0 = std::time::Instant::now();
            while n < limit && !rp.should_quit() {
                rp.tick(&mut pads);
                n += 1;
                let due = t0 + std::time::Duration::from_micros(n * 16_667);
                let now = std::time::Instant::now();
                if due > now {
                    std::thread::sleep(due - now);
                }
            }
            n
        } else {
            melee_platform::run_headless(&mut rp, &mut pads, limit)
        };
        rp.save_replay();
        log(&format!("headless: {ticks} ticks, {}", scene_name(rp.cur.1)));
        // MELEE_RETAIL_WAV=<path>: the headless mix as a WAV (the movie / menu sound checks against the C's MELEE_AUDIO_DUMP)
        if let (Ok(p), Some(cap)) = (std::env::var("MELEE_RETAIL_WAV"), rp.audio.as_ref().and_then(|a| a.capture.as_ref())) {
            if let Err(e) = crate::write_wav(&p, cap) {
                eprintln!("play: {p}: {e}");
            }
        }
        return if rp.failed.is_some() { ExitCode::FAILURE } else { ExitCode::SUCCESS };
    }
    let scripted = std::env::var_os("MELEE_PAD_SCRIPT").is_some_and(|v| !v.is_empty());
    // play.rs window_title: MELEE_WINDOW_TITLE, else the build id ("melee [where hash]") and "retail menus"
    let title = std::env::var("MELEE_WINDOW_TITLE").unwrap_or_else(|_| format!("{} retail menus", crate::play::window_title(None)));
    let cfg = crate::display::current().window_config(title, !scripted);
    if scripted {
        std::env::set_var("MELEE_PAD_IGNORE_ADAPTER", "1");
    }
    let mut pads = PadSystem::interactive();
    if scripted {
        pads.set_ignore_adapter(true);
        pads.gamepads = melee_platform::pad_read::GamepadSource::Fixed(Default::default());
    }
    if !scripted && std::env::var("MELEE_KEYBOARD").map_or(true, |v| v != "0") {
        pads.keyboard[0] = melee_platform::aurora::KeyboardBindings::preset_default();
    }
    // the launcher's own controls, when it started us (launcher_compat.rs)
    crate::launcher_compat::apply_input(&mut pads);
    match melee_platform::run_windowed(rp, pads, cfg) {
        Ok((ticks, _)) => {
            log(&format!("closed after {ticks} ticks"));
            if crate::play_net::RUN_FAILED.load(std::sync::atomic::Ordering::Relaxed) {
                return ExitCode::FAILURE;
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("play: event loop: {e}");
            ExitCode::FAILURE
        }
    }
}

impl Drop for RetailPlay {
    /// A window closed by hand still writes its `--record` replay.
    fn drop(&mut self) {
        if let Some(u) = self.ui.as_ref() {
            let (dc, fr) = self.presenter.as_ref().map_or((0, 0), |p| p.renderer.ui_stats());
            log(&format!("{}; renderer: {dc} UI draw calls over {fr} presents with UI", u.stats_line()));
        }
        self.save_replay();
    }
}

/// `MELEE_HASH_LOG=<file>`: one line per match tick, `frame hash` (the NHV1 neutral hash), to compare two windowed runs
/// (display options on / off must simulate the same game).
fn hash_log(frame: u64, hash: u32) {
    use std::io::Write;
    static F: std::sync::OnceLock<Option<std::sync::Mutex<std::fs::File>>> = std::sync::OnceLock::new();
    let f = F.get_or_init(|| std::env::var_os("MELEE_HASH_LOG").and_then(|p| std::fs::File::create(p).ok()).map(std::sync::Mutex::new));
    if let Some(f) = f {
        let _ = writeln!(f.lock().unwrap(), "{frame} {hash:08X}");
    }
}

/// Set before `melee play` by a core v1.1 entry whose host may call the state calls.
pub static HOSTED_STATES: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// A match whose states the host keeps by key (core v1.1 `state_save` / `state_load`, corev1.rs): the match engine
/// under the snapshot base, the kept states, and the pads each frame last ran with.
pub struct Hosted {
    pub g: Box<crate::play_net::ModesGame>,
    pub keep: std::collections::HashMap<i32, crate::play_net::ModesSnap>,
    pub inputs: std::collections::BTreeMap<i32, [melee_types::PADStatus; 4]>,
    logged: u32,
}

impl Hosted {
    /// The session frame the next game frame will have, counted from 0 at the hand-over.
    pub fn next_frame(&self) -> i32 {
        use melee_net::session::RbGame;
        self.g.frame() + 1 - melee_net::session::FIRST
    }
    pub fn save(&mut self, key: i32) -> bool {
        use melee_net::session::RbGame;
        let s = self.g.save();
        self.keep.insert(key, s);
        true
    }
    pub fn load(&mut self, key: i32) -> bool {
        use melee_net::session::RbGame;
        let Some(s) = self.keep.get(&key).cloned() else { return false };
        self.g.load(&s);
        true
    }
    pub fn drop_key(&mut self, key: i32) {
        self.keep.remove(&key);
        // the pads of the last minute are what a load can re-run (a rollback reaches back a few frames)
        use melee_net::session::RbGame;
        let keep_from = self.g.frame() - 3600;
        self.inputs = self.inputs.split_off(&keep_from);
    }
    pub fn hash(&self) -> u32 {
        self.g.e.neutral_row()[0] | 1
    }
    pub fn set_input(&mut self, frame: i32, pads: [melee_types::PADStatus; 4]) {
        self.inputs.insert(frame + melee_net::session::FIRST, pads);
    }
}
