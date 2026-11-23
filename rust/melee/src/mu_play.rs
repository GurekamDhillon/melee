//! `play --p2p-*`: a P2P Direct match (Hero's launcher lobby, mu_net) in the window. A child module of `play_net`
//! (it builds and drives a [`NetDriver`] like the Slippi-wire match, `connect_slippi` / `tick_slippi`): the same
//! Slippi-online-rules simulation, the [`melee_net::mu_net::link::Link`] instead of the Slippi client.
//!
//! What the launcher gets back (his `source_p2p.cpp`): exit 2 when no game was played (connect or handshake failed),
//! 0 after the game; the signed result file at `--p2p-result` (one per game).

use super::*;
use crate::mu_np::{establish, scene_of, P2pOptions};
use melee_net::mu_net::link::{Link, NextGame};
use melee_net::mu_net::result::{path_for_game, GameResult, PlayerResult};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

/// The session's link, shared by the game thread (each tick), the pump thread (every millisecond, as his session's
/// pump does: arrivals are read and answered when they land, not at the next frame) and the early-input sampler
/// thread (a SAMPLE goes out the moment the controller changes).
pub struct LinkCell(pub Link);
// SAFETY: the Link owns its socket transport (a `Box<dyn Transport>`, not marked Send: UdpTransport or the test's
// simulated network over it) and plain data; it is only reached through the mutex, so one thread at a time.
unsafe impl Send for LinkCell {}
pub type SharedLink = Arc<Mutex<LinkCell>>;

pub fn lock(l: &SharedLink) -> MutexGuard<'_, LinkCell> {
    l.lock().unwrap_or_else(|e| e.into_inner())
}

/// The pump thread: polls the link about every millisecond until stopped (dropped).
struct Pump {
    run: Arc<AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl Pump {
    fn start(link: SharedLink) -> Pump {
        let run = Arc::new(AtomicBool::new(true));
        let r2 = run.clone();
        let thread = std::thread::Builder::new()
            .name("p2p-pump".into())
            .spawn(move || {
                // never game code: its allocations stay on the system heap whatever the game thread is doing
                melee_engine::arena::mark_foreign_thread();
                while r2.load(Ordering::Acquire) {
                    lock(&link).0.poll();
                    std::thread::sleep(Duration::from_millis(1));
                }
            })
            .ok();
        Pump { run, thread }
    }
}

impl Drop for Pump {
    fn drop(&mut self) {
        self.run.store(false, Ordering::Release);
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

/// The match's network half, driven one render tick at a time by the retail flow.
pub struct MuRun {
    pub shared: SharedLink,
    _pump: Pump,
    /// the early-input sampler of the current match (sends SAMPLEs to a revision-2 peer), and SAMPLEs sent
    sampler: Option<crate::np_early::EarlySender>,
    samples_sent: Arc<AtomicU64>,
    pub opts: P2pOptions,
    local: usize,
    remote: usize,
    delay: i32,
    chars: [u32; 4],
    last_sent: i32,
    last_local: i32,
    finals: std::collections::BTreeMap<i32, u32>,
    compared: u32,
    compared_upto: i32,
    pub soft: u32,
    pub hard: u32,
    /// time sync (melee_net::time_sync, the same numbers as mu_net's section 8): holds when ahead, extra frames
    /// when behind; local pacing only
    ts: melee_net::time_sync::TimeSync,
    started: bool,
    done_at: Option<Instant>,
    /// the session ended before the game reported its end (BYE, a lost connection, 7 s without input)
    disconnected: bool,
    written: bool,
    /// early inputs on for this run (np_early::enabled): the peer's SAMPLEs are used for prediction
    early: bool,
    /// test hooks: `--mu-test-frames N` ends the match after N final checksums (a scripted match that never reaches
    /// GAME!), `--mu-test-padgen SEED` plays the session's generated pads instead of the local controller
    frames: Option<i32>,
    padgen: Option<u32>,
}

impl NetDriver {
    /// `play --p2p-port P --p2p-peer A:P ...` (the launcher's line): connect before the window, then the match
    /// scenario the retail flow boots. An Err is "no game was played" (exit 2).
    pub fn connect_mu(args: &[String]) -> Result<(NetDriver, melee_engine::scenario::Scenario), String> {
        let o = P2pOptions::parse(args)?.ok_or("no --p2p-port / --p2p-peer")?;
        let link = establish(&o, if o.slot == 0 { "slot 0" } else { "slot 1" }, args)?;
        let d = link.conn.descriptor().clone();
        let scene = scene_of(&d)?;
        melee_core::slippi_online::enable(d.rng_seed);
        if std::env::var_os("MELEE_SLIPPI_NO_ONLINE_CODES").is_none() {
            crate::slippi_np::online_codes(d.alt_stage);
        }
        for (slot, p) in d.players.iter().enumerate() {
            melee_world::tu::ifnametag::set_online_tag(slot, if slot < 2 { &p.name } else { "" });
        }
        let mut chars = [u32::MAX; 4];
        for (i, p) in d.players.iter().take(2).enumerate() {
            chars[i] = p.character as u32;
        }
        if let Some(st) = melee_core::slippi_online::state() {
            st.pending = Default::default();
            for (p, c) in chars.iter().enumerate() {
                st.pending[p].ext_char = *c;
            }
        }
        let scn = melee_engine::scenario::Scenario::parse(&format!("mode=vs;at=match;{scene}"))?;
        let p = &d.players;
        let room_scene = melee_net::app::scene::build_scene(
            &format!("ck:{}", p[0].character),
            p[0].color as i32,
            &format!("ck:{}", p[1].character),
            p[1].color as i32,
            &format!("ext:{}", d.stage),
            d.stocks as i32,
            (d.timer_seconds / 60) as i32,
            0,
        );
        let scene_p = parse_scene(&room_scene).ok_or("p2p: the match scene does not parse")?;
        let delay = d.input_delay as i32;
        let decider = link.conn.decider();
        log(&format!("netplay {}: P2P Direct match: seed {:08x}, delay {delay}, scene {scene}", if decider { "host" } else { "guest" }, d.rng_seed));
        let local = o.slot;
        let rcfg = RbConfig { net: true, local_port: local, delay, src: InputSource::Live, ..RbConfig::default() };
        let sess: Rc<RefCell<Sess>> = Rc::new(RefCell::new(Sess::new(rcfg)));
        melee_data::netid::freeze();
        let val = |k: &str| crate::mu_np::arg_value(args, k);
        let shared: SharedLink = Arc::new(Mutex::new(LinkCell(link)));
        let run = MuRun {
            _pump: Pump::start(shared.clone()),
            shared,
            sampler: None,
            samples_sent: Arc::new(AtomicU64::new(0)),
            local,
            remote: 1 - local,
            delay,
            chars,
            last_sent: FIRST - 1,
            last_local: FIRST - 1,
            finals: Default::default(),
            compared: 0,
            compared_upto: 0,
            soft: 0,
            hard: 0,
            ts: melee_net::time_sync::TimeSync::new(melee_net::time_sync::old_rule()),
            started: false,
            done_at: None,
            disconnected: false,
            written: false,
            early: crate::np_early::enabled(args),
            frames: val("--mu-test-frames").and_then(|v| v.parse().ok()),
            padgen: val("--mu-test-padgen").and_then(|v| v.parse().ok()),
            opts: o,
        };
        let nd = NetDriver {
            _frozen: NetidThaw,
            // the slot-0 player is "host" in the logs and the HUD (port 1)
            host: local == 0,
            sess,
            log: Rc::new(RefCell::new(NetLog::default())),
            room: None,
            net: None,
            agreed: Agreed { scene: scene_p, scene_str: room_scene, seed: d.rng_seed, delay, host: local == 0 },
            host_rules: HostRules::default(),
            phase: NetPhase::PreMatch,
            started_at: None,
            finals: Vec::new(),
            hash_out: None,
            rec_next: FIRST,
            end_frame: None,
            linger: 0,
            error: None,
            last_rb: (0, 0, 0),
            ticks: 0,
            held_ticks: 0,
            ls: None,
            lab: false,
            params: None,
            slippi: None,
            mu: Some(Box::new(run)),
            match_no: 0,
            all_finals: Vec::new(),
            early_on: false,
            early: None,
            early_logged: false,
            late_send: false,
        };
        Ok((nd, scn))
    }

    /// The match scene began: the session's ports and delay, the Slippi-online checksum state.
    pub(super) fn begin_mu(&mut self, g: &mut ModesGame) {
        let mr = self.mu.as_mut().unwrap();
        let mut s = self.sess.borrow_mut();
        s.set_local_port(mr.local);
        s.set_delay(mr.delay);
        g.slippi = Some(Box::new(SlippiSim { chars: mr.chars, ..Default::default() }));
    }

    /// One render tick of the match (as `tick_slippi`): the link, the local pad committed `delay` frames ahead
    /// (online frames 1..=delay the zero pad), one COMMIT, the time-sync hold, the session's frames, the finalized
    /// checksums sent and compared, the transcript; at the end RESULT and the result file.
    pub(super) fn tick_mu(&mut self, local: &PADStatus) -> bool {
        use melee_core::slippi_online::{compare, Compare};
        use melee_net::slippi::{pad_bytes, pad_input};
        if self.end_frame.is_some() || self.error.is_some() {
            return false;
        }
        let tag = self.tag();
        let sess = |f: i32| f + FIRST - 1;
        let online = |s: i32| s - FIRST + 1;
        let NetPhase::InMatch(g) = &mut self.phase else { return false };
        let mr = self.mu.as_mut().unwrap();
        let mut s = self.sess.borrow_mut();
        let (lp, rp) = (mr.local, mr.remote);
        let shared = mr.shared.clone();
        let mut guard = lock(&shared);
        let link = &mut guard.0;
        link.poll();
        for (f, pad) in link.input.take_fresh() {
            s.submit_remote_input(rp * 2, sess(f), &pad_input(&pad));
        }
        // early inputs (rev 2): the peer's SAMPLEs predict its unconfirmed frames
        if !s.hints_on && mr.early {
            s.hints_on = true;
            link.input.samples_on = true;
        }
        for (t, pad) in link.input.take_samples() {
            s.submit_hint(rp * 2, sess(t), &crate::mu_np::hint_of(&pad));
        }
        let mut gone: Option<String> = None;
        if mr.done_at.is_none() {
            if let Some(w) = link.down() {
                gone = Some(w);
            } else if link.input_timed_out() {
                gone = Some("no input from the other player for 7 s".into());
            }
        }
        if self.started_at.is_none() && link.input.remote_latest() > 0 {
            self.started_at = Some(Instant::now());
            mr.started = true;
            log(&format!("netplay {tag}: P2P Direct match started (delay {})", mr.delay));
        }
        let f = s.current_frame();
        // start in step: online frame 2 waits for the peer's first frame (a peer still loading the stage)
        s.start_hold = !mr.ts.old && link.input.remote_latest() == 0;
        let step = {
            let input = &link.input;
            mr.ts.step(online(f), || input.time_offset_us())
        };
        if step.hold > 0 {
            s.request_wait(step.hold);
        }
        let extra = step.advance as i32;
        for ff in (mr.last_local + 1).max(f)..=f + mr.delay + extra {
            let i = if online(ff) <= mr.delay {
                pad_input(&[0; 8])
            } else {
                match mr.padgen {
                    Some(gs) => melee_net::session::rb_gen(gs, lp * 2, ff),
                    None => input_of(local),
                }
            };
            s.submit_local_input(lp * 2, ff, &i);
            mr.last_local = ff;
        }
        for ff in (mr.last_sent + 1)..=(f + mr.delay + extra) {
            let Some(i) = s.local_input_for_send(lp * 2, ff) else { break };
            if online(ff) >= 1 {
                link.commit(online(ff), pad_bytes(&i));
            }
            mr.last_sent = ff;
        }
        if let Some((&ckf, &ck)) = mr.finals.iter().next_back() {
            link.input.set_checksum(ckf, ck);
        }
        link.send_commit();
        // the link is free for the pump and the sampler while the frames run
        drop(guard);
        if mr.done_at.is_none() && gone.is_none() {
            run_tick(&mut s, &mut **g, None);
            // behind the peer: one extra frame this tick (only when this tick ran its new frame)
            if extra > 0 && s.current_frame() > f {
                run_tick(&mut s, &mut **g, None);
            }
        }
        if let Some(e) = mr.sampler.as_mut() {
            // the local frames now scheduled: the sampler's baseline (what a SAMPLE must differ from)
            let newest = s.current_frame() + s.cfg.delay + 1;
            let ss = &*s;
            e.after_tick(newest, &|fr| {
                ss.local_input_for_send(lp * 2, fr).map(|i| {
                    let mut b = [0u8; melee_net::NP_PAYLOAD];
                    melee_net::np_encode(&i, &mut b);
                    b
                })
            });
        }
        let mut guard = lock(&shared);
        let link = &mut guard.0;
        let conf = s.confirmed_frame();
        link.input.set_floor(online(conf));
        link.advance_transcript();
        let from = mr.finals.keys().next_back().map_or(2, |k| k + 1);
        {
            let sl = g.slippi.as_ref().unwrap();
            let _h = melee_engine::arena::SysScope::enter();
            for of in from..=online(conf) + 1 {
                match sl.ck.get(&of) {
                    Some(&v) if s.checksum(sess(of)) != 0 => {
                        mr.finals.insert(of, v);
                    }
                    _ => break,
                }
            }
        }
        for (&fr, &rv) in link.remote_checksums.range((mr.compared_upto + 1).max(2)..) {
            let Some(&lv) = mr.finals.get(&fr) else { break };
            mr.compared += 1;
            mr.compared_upto = fr;
            match compare(lv, rv) {
                Compare::Same => {}
                Compare::Soft => mr.soft += 1,
                Compare::Hard => {
                    mr.hard += 1;
                    if mr.hard < 4 {
                        log(&format!("netplay {tag}: DESYNC at online frame {fr}: local {lv:08x} remote {rv:08x} (the match goes on)"));
                    }
                }
            }
        }
        let reached = mr.frames.is_some_and(|n| mr.finals.keys().next_back().is_some_and(|&k| k >= n));
        if (reached || g.e.ended) && mr.done_at.is_none() {
            mr.done_at = Some(Instant::now());
            link.report_end();
            // several games (plan 2.12): NEXT when the game ended by itself with no checksum disagreement
            link.offer_next(true, mr.hard > 0);
        }
        // the end: the peer's RESULT compared (or 1.5 s for it) and the next game settled, or the session gone
        let end_now = link.next != NextGame::Waiting
            && mr.done_at.is_some_and(|t| (link.peer_agreement().is_some() && t.elapsed() > Duration::from_millis(300)) || t.elapsed() > Duration::from_millis(1500));
        let cur = s.current_frame();
        drop(s);
        drop(guard);
        if !end_now && gone.is_none() {
            return true;
        }
        let mr = self.mu.as_mut().unwrap();
        if let Some(mut e) = mr.sampler.take() {
            e.stop();
            let st = e.stats();
            log(&format!("netplay {tag}: P2P early inputs: {} SAMPLEs sent, sampler {st:?}", mr.samples_sent.load(Ordering::Relaxed)));
        }
        let has_outcome = mr.done_at.is_some() && gone.is_none();
        if let Some(w) = &gone {
            mr.disconnected = true;
            log(&format!("netplay {tag}: P2P: {w} at online frame {}", online(cur)));
        }
        let won = if has_outcome && mr.hard == 0 { match_winner() } else { -1 };
        LAST_WINNER.store(won, std::sync::atomic::Ordering::Relaxed);
        mr.write_result(has_outcome, won, online(cur));
        let mut guard = lock(&mr.shared);
        let link = &mut guard.0;
        let agree = link.peer_agreement();
        let (rx, ign) = (link.input.stats.samples_received, link.input.stats.samples_ignored);
        log(&format!(
            "netplay {tag}: P2P: {} final checksums, {} compared with the peer: {} soft, {} HARD desyncs; transcript {} frames, peer {}; SAMPLEs received {rx} ({ign} ignored)",
            mr.finals.len(),
            mr.compared,
            mr.soft,
            mr.hard,
            link.transcript.as_ref().map_or(0, |t| t.frames),
            match agree {
                Some(true) => "agrees",
                Some(false) => "DISAGREES",
                None => "not compared",
            }
        ));
        if link.next != NextGame::Agreed {
            link.close();
        }
        drop(guard);
        // a game that was played is exit 0 for the launcher, even one the other player left (the result file says it)
        self.end_frame = Some(cur);
        self.linger = 0;
        true
    }

    /// After a match whose next game both sides agreed (NEXT): the session moves to it (descriptor, frame base,
    /// transcript), this driver is reset for it, and the next match's scenario comes back for the engine
    /// (`Engine::next_vs`). None when the session is over.
    pub fn mu_next_match(&mut self) -> Option<melee_engine::scenario::Scenario> {
        let mr = self.mu.as_mut()?;
        let shared = mr.shared.clone();
        let mut guard = lock(&shared);
        let link = &mut guard.0;
        if link.next != NextGame::Agreed || !link.begin_next_game() {
            if let NextGame::Over(w) = &link.next {
                log(&format!("netplay {}: P2P: no further game: {w}", if self.host { "host" } else { "guest" }));
            }
            return None;
        }
        let d = link.descriptor()?.clone();
        let scene = scene_of(&d).ok()?;
        let scn = melee_engine::scenario::Scenario::parse(&format!("mode=vs;at=match;{scene}")).ok()?;
        melee_core::slippi_online::enable(d.rng_seed);
        if std::env::var_os("MELEE_SLIPPI_NO_ONLINE_CODES").is_none() {
            crate::slippi_np::online_codes(d.alt_stage);
        }
        if let Some(st) = melee_core::slippi_online::state() {
            st.pending = Default::default();
            for (p, c) in mr.chars.iter().enumerate() {
                st.pending[p].ext_char = *c;
            }
        }
        melee_modes::tu::envoy::set_net_salt(d.rng_seed);
        std::env::set_var("MELEE_TEST_SEED", d.rng_seed.to_string());
        mr.last_sent = FIRST - 1;
        mr.last_local = FIRST - 1;
        mr.finals.clear();
        mr.compared = 0;
        mr.compared_upto = 0;
        mr.soft = 0;
        mr.hard = 0;
        mr.ts = melee_net::time_sync::TimeSync::new(melee_net::time_sync::old_rule());
        mr.started = false;
        mr.done_at = None;
        mr.disconnected = false;
        mr.written = false;
        let (tag, game) = (if self.host { "host" } else { "guest" }, link.game);
        log(&format!("netplay {tag}: P2P: game {game} of the session: stage {}, seed {:08x}", d.stage, d.rng_seed));
        self.agreed.seed = d.rng_seed;
        self.end_frame = None;
        self.linger = 0;
        self.started_at = None;
        self.match_no += 1;
        self.phase = NetPhase::PreMatch;
        Some(scn)
    }

    /// Right after the game read the controllers (retail_flow `tick`): in a match with a revision-2 peer, start the
    /// early-input sampler (SAMPLEs through the locked link, from its own thread) and tell it about this poll.
    pub(super) fn pre_tick_mu(&mut self, pads: &melee_platform::PadSystem) {
        if !matches!(self.phase, NetPhase::InMatch(_)) || self.end_frame.is_some() {
            return; // the pump keeps the link serviced meanwhile
        }
        let tag = self.tag();
        let mr = self.mu.as_mut().unwrap();
        if mr.sampler.is_none() && mr.early && mr.done_at.is_none() {
            let ready = {
                let g = lock(&mr.shared);
                g.0.peer_takes_samples() && g.0.input.local_latest() > 0
            };
            if ready {
                let (sh, sent) = (mr.shared.clone(), mr.samples_sent.clone());
                mr.sampler = Some(crate::np_early::EarlySender::start_with(
                    pads.sampler(),
                    Box::new(move |t, pad| {
                        let mut p8 = [0u8; 8];
                        p8.copy_from_slice(&pad[..8]);
                        // session frame -> the game's frame (online frame 1 = session FIRST)
                        let ok = lock(&sh).0.send_sample(t - FIRST + 1, p8);
                        if ok {
                            sent.fetch_add(1, Ordering::Relaxed);
                        }
                        ok
                    }),
                ));
                log(&format!("netplay {tag}: P2P early inputs: SAMPLEs on (the other player is revision 2)"));
            }
        }
        let next = pads.script.as_ref().and_then(|s| s.peek_next()).map(|g| {
            let mut p = PADStatus::zeroed();
            p.button = g.buttons;
            p.stickX = g.sx;
            p.stickY = g.sy;
            p.triggerLeft = g.tl;
            p.triggerRight = g.tr;
            p
        });
        if let Some(e) = mr.sampler.as_mut() {
            e.note_poll(next);
        }
    }

    /// A P2P match's checksum comparisons: (soft, hard).
    pub(super) fn mu_desyncs(&self) -> Option<(u32, u32)> {
        self.mu.as_ref().map(|m| (m.soft, m.hard))
    }
}

impl MuRun {
    /// The signed result file of this game (`--p2p-result`), once.
    fn write_result(&mut self, has_outcome: bool, winner: i32, frames: i32) {
        let Some(base) = self.opts.result.clone() else { return };
        if self.written || !self.started {
            return;
        }
        self.written = true;
        let guard = lock(&self.shared);
        let l = &guard.0;
        // the game's own descriptor and digest (games after the first: the chained ones, not the handshake's)
        let Some(d) = l.descriptor() else { return };
        let mut r = GameResult {
            route: d.route,
            build_id: d.build_id.clone(),
            descriptor_digest: l.digest(),
            match_id: d.match_id,
            local_slot: self.local as u8,
            peer_identity: l.conn.peer_identity().unwrap_or_default(),
            transcript_digest: l.transcript.as_ref().map(|t| t.digest()).unwrap_or_default(),
            transcript_frames: l.transcript.as_ref().map_or(0, |t| t.frames),
            transcript_complete: true,
            peer_agreement: l.peer_agreement(),
            desync: self.hard > 0,
            disconnected: self.disconnected,
            has_outcome,
            frame_length: frames.max(0) as u32,
            game_index: l.game,
            winner,
            // Slippi's game end methods: 2 GAME (stocks), 1 TIME; 0 when the game did not end by itself
            end_method: if !has_outcome { 0 } else if winner >= 0 && (0..2).any(|p| melee_fighter::stale::stocks(p) == 0) { 2 } else { 1 },
            ..Default::default()
        };
        for (p, slot) in r.players.iter_mut().enumerate() {
            *slot = if p < 2 {
                let st = melee_fighter::stale::stocks(p as i32).clamp(0, 255) as u8;
                PlayerResult { slot_type: 0, stocks: st, synced_stocks: st, ..Default::default() }
            } else {
                // an empty slot (Melee's player kind 3, none)
                PlayerResult { slot_type: 3, ..Default::default() }
            };
        }
        let path = path_for_game(&base, l.game);
        match r.write(l.conn.identity(), &path) {
            Ok(()) => log(&format!("p2p: result file written: {path} (game {}, {} input frames, winner {winner})", l.game, r.transcript_frames)),
            Err(e) => log(&format!("p2p: result file not written: {e}")),
        }
    }
}
