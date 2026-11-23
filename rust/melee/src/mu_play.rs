//! `play --p2p-*`: a P2P Direct match (Hero's launcher lobby, mu_net) in the window. A child module of `play_net`
//! (it builds and drives a [`NetDriver`] like the Slippi-wire match, `connect_slippi` / `tick_slippi`): the same
//! Slippi-online-rules simulation, the [`melee_net::mu_net::link::Link`] instead of the Slippi client.
//!
//! What the launcher gets back (his `source_p2p.cpp`): exit 2 when no game was played (connect or handshake failed),
//! 0 after the game; the signed result file at `--p2p-result` (one per game).

use super::*;
use crate::mu_np::{establish, scene_of, P2pOptions};
use melee_net::mu_net::link::Link;
use melee_net::mu_net::result::{path_for_game, GameResult, PlayerResult};

/// The match's network half, driven one render tick at a time by the retail flow.
pub struct MuRun {
    pub link: Link,
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
        let run = MuRun {
            link,
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
        mr.link.poll();
        for (f, pad) in mr.link.input.take_fresh() {
            s.submit_remote_input(rp * 2, sess(f), &pad_input(&pad));
        }
        let mut gone: Option<String> = None;
        if mr.done_at.is_none() {
            if let Some(w) = mr.link.down() {
                gone = Some(w);
            } else if mr.link.input_timed_out() {
                gone = Some("no input from the other player for 7 s".into());
            }
        }
        if self.started_at.is_none() && mr.link.input.remote_latest() > 0 {
            self.started_at = Some(Instant::now());
            mr.started = true;
            log(&format!("netplay {tag}: P2P Direct match started (delay {})", mr.delay));
        }
        let f = s.current_frame();
        // start in step: online frame 2 waits for the peer's first frame (a peer still loading the stage)
        s.start_hold = !mr.ts.old && mr.link.input.remote_latest() == 0;
        let step = {
            let input = &mr.link.input;
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
                mr.link.commit(online(ff), pad_bytes(&i));
            }
            mr.last_sent = ff;
        }
        if let Some((&ckf, &ck)) = mr.finals.iter().next_back() {
            mr.link.input.set_checksum(ckf, ck);
        }
        mr.link.send_commit();
        if mr.done_at.is_none() && gone.is_none() {
            run_tick(&mut s, &mut **g, None);
            // behind the peer: one extra frame this tick (only when this tick ran its new frame)
            if extra > 0 && s.current_frame() > f {
                run_tick(&mut s, &mut **g, None);
            }
        }
        let conf = s.confirmed_frame();
        mr.link.input.set_floor(online(conf));
        mr.link.advance_transcript();
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
        for (&fr, &rv) in mr.link.remote_checksums.range((mr.compared_upto + 1).max(2)..) {
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
            mr.link.report_end();
        }
        // the end: the peer's RESULT compared (or 1.5 s for it), or the session gone
        let end_now = mr.done_at.is_some_and(|t| (mr.link.peer_agreement().is_some() && t.elapsed() > Duration::from_millis(300)) || t.elapsed() > Duration::from_millis(1500));
        let cur = s.current_frame();
        drop(s);
        if !end_now && gone.is_none() {
            return true;
        }
        let mr = self.mu.as_mut().unwrap();
        let has_outcome = mr.done_at.is_some() && gone.is_none();
        if let Some(w) = &gone {
            mr.disconnected = true;
            log(&format!("netplay {tag}: P2P: {w} at online frame {}", online(cur)));
        }
        let won = if has_outcome && mr.hard == 0 { match_winner() } else { -1 };
        LAST_WINNER.store(won, std::sync::atomic::Ordering::Relaxed);
        mr.write_result(has_outcome, won, online(cur));
        let agree = mr.link.peer_agreement();
        log(&format!(
            "netplay {tag}: P2P: {} final checksums, {} compared with the peer: {} soft, {} HARD desyncs; transcript {} frames, peer {}",
            mr.finals.len(),
            mr.compared,
            mr.soft,
            mr.hard,
            mr.link.transcript.as_ref().map_or(0, |t| t.frames),
            match agree {
                Some(true) => "agrees",
                Some(false) => "DISAGREES",
                None => "not compared",
            }
        ));
        mr.link.close();
        // a game that was played is exit 0 for the launcher, even one the other player left (the result file says it)
        self.end_frame = Some(cur);
        self.linger = 0;
        true
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
        let l = &self.link;
        let d = l.conn.descriptor();
        let mut r = GameResult {
            route: d.route,
            build_id: d.build_id.clone(),
            descriptor_digest: l.conn.digest().unwrap_or_default(),
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
