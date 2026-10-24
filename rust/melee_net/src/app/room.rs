//! The room flow of gw_netplay.c: `np_start_session`, `np_poll`, `np_cb_guest_hello`,
//! `np_cb_lobby`, `lb_go`, `np_lobby_enter`, `np_lobby_tick`, `np_arm`. A room is one connection
//! between two players: the host's scene is agreed in the handshake (`use_lobby == false`, the
//! scripted `MELEE_NETPLAY=host|join:` path) or in the pick/ban lobby (`use_lobby`).
//!
//! The game side supplies [`LobbyEnv`] (identities) and the inner [`Callbacks`] (the rollback
//! session's input / checksum glue); this module adds the guest-hello and lobby callbacks, which
//! cannot touch the `Net` they run inside, so replies go to an outbox drained after each poll.

use super::lobby::*;
use super::scene::*;
use crate::net::{Callbacks, Config, Event, Net, State};
use crate::transport::{Addr, Transport};
use crate::{LOBBY_MAX, MAX_BLOB};
use std::cell::RefCell;
use std::rc::Rc;

pub const NP_FIRST_FRAME: i32 = -123;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Idle,
    Working,
    /// The match is agreed and armed.
    Connected,
    Failed,
    Lobby,
}

/// The identities of this install: fighter and stage tokens. `RoomEnv: LobbyEnv` adds the stage queries.
pub trait RoomEnv: LobbyEnv {
    fn token_for_ck(&self, ck: i32) -> String;
    /// A scene / hello token ("id:<hex>", "ck:<n>") to this install's CharacterKind, -1 if absent.
    fn ck_for_token(&self, tok: &str) -> i32;
    fn global_hash(&self) -> u64;
    fn global_describe(&self) -> String;
    /// Whether this install can play the scene: Err names the missing content (`gw_MexId_SceneCheck`).
    fn scene_check(&self, _scene: &ParsedScene) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct RoomSettings {
    pub host: bool,
    pub ck: i32,
    pub color: i32,
    pub stocks: i32,
    pub minutes: i32,
    pub delay: i32,
    pub stage_ext: i32,
    pub use_lobby: bool,
    /// Guest: where the host is.
    pub peer: Option<Addr>,
    pub name: String,
    pub exe_hash: u64,
    pub stage_mode: i32,
    pub seed: Option<u32>,
    pub controls: u32,
}

impl Default for RoomSettings {
    fn default() -> Self {
        RoomSettings { host: true, ck: 2, color: 0, stocks: 4, minutes: 8, delay: 2, stage_ext: 31, use_lobby: false, peer: None, name: String::new(), exe_hash: 0, stage_mode: 0, seed: None, controls: 0 }
    }
}

pub fn controls_word_valid(w: u32) -> bool {
    w & !0xff3d == 0
}

pub struct Shared {
    pub s: RoomSettings,
    pub phase: Phase,
    pub status: String,
    pub scene: String,
    pub seed: u32,
    pub controls: [u32; 2],
    pub peer_name: String,
    pub lb: Lobby,
    pub outbox: Vec<Vec<u8>>,
    pub started: bool,
    pub dead: bool,
    pub accepted: bool,
    pub peer_gone: bool,
    pub env: Box<dyn RoomEnv>,
    pub log: Vec<String>,
    xtras: String,
    t0: std::time::Instant,
}

impl Shared {
    fn now_ms(&self) -> u32 {
        self.t0.elapsed().as_millis() as u32
    }
    fn status(&mut self, s: impl Into<String>) {
        let s = s.into();
        self.log.push(s.clone());
        self.status = s;
    }
    fn send(&mut self, m: String) {
        self.outbox.push(m.into_bytes());
    }
    fn controls_send(&mut self) {
        let m = if self.s.host { format!("J {} {}", self.controls[0], self.controls[1]) } else { format!("J {}", self.controls[1]) };
        self.send(m);
    }
    fn broadcast(&mut self) {
        let m = self.lb.encode();
        self.send(m);
    }
    fn build_scene_for(&self, hck: i32, hc: i32, gck: i32, gc: i32) -> String {
        build_scene(&self.env.token_for_ck(hck), hc, &self.env.token_for_ck(gck), gc, &self.env.token_for_ext(self.s.stage_ext), self.s.stocks, self.s.minutes, 0)
    }

    /// np_cb_guest_hello.
    fn guest_hello(&mut self, info: &[u8], blob: &mut [u8], blob_len: &mut u16) {
        let s: String = String::from_utf8_lossy(&info[..info.len().min(64)]).into_owned();
        let mut gck = 9;
        let mut gc = 0;
        let head = s.split('/').next().unwrap_or("");
        let k = self.env.ck_for_token(head);
        if (head.starts_with("id:") || head.starts_with("ck:")) && k >= 0 && k != 0x21 && k <= 0x7F {
            gck = k;
        }
        for part in s.split('/').skip(1) {
            if let Some(c) = part.strip_prefix('c') {
                gc = c.parse().unwrap_or(0);
            }
        }
        if gc < 0 || (gc > 254 && gc & 0x4000_0000 == 0) {
            gc = 0;
        }
        let mut word = 0u32;
        if let Some(j) = s.split('/').find_map(|p| p.strip_prefix('j')) {
            word = j.parse().unwrap_or(0);
        }
        self.controls[0] = self.s.controls;
        self.controls[1] = if controls_word_valid(word) { word } else { 0 };
        self.scene = self.build_scene_for(self.s.ck, self.s.color, gck, gc);
        let out = format!("J {} {} {}", self.controls[0], self.controls[1], self.scene);
        if out.len() + 1 > blob.len() || out.len() + 1 > MAX_BLOB {
            *blob_len = (MAX_BLOB + 1) as u16;
            self.log.push("refusing oversize fighter-controls match setup".into());
            return;
        }
        blob[..out.len()].copy_from_slice(out.as_bytes());
        blob[out.len()] = 0;
        *blob_len = (out.len() + 1) as u16;
        self.log.push(format!("the guest picked \"{s}\" - match \"{}\"", self.scene));
    }

    fn lb_action(&mut self, act: &str, a: i32, b: i32) {
        if self.s.host {
            if self.lb.apply(0, act, a, b, false) {
                self.lb.seq += 1;
                self.broadcast();
            }
        } else {
            self.send(format!("A {act} {a} {b}"));
        }
    }

    /// np_cb_lobby.
    fn lobby_msg(&mut self, data: &[u8]) {
        let m = String::from_utf8_lossy(&data[..data.len().min(LOBBY_MAX)]).into_owned();
        let m = m.trim_end_matches('\0');
        let b = m.as_bytes();
        if b.len() < 2 {
            return;
        }
        let host = self.s.host;
        if m.starts_with("J ") {
            let w: Vec<u32> = m[2..].split_whitespace().filter_map(|t| t.parse().ok()).collect();
            if w.len() < if host { 1 } else { 2 } || !w.iter().all(|&x| controls_word_valid(x)) {
                self.log.push("refused malformed lobby fighter controls".into());
                return;
            }
            if host {
                if self.controls[1] != w[0] {
                    self.controls[1] = w[0];
                    self.lb.ready[1] = false;
                    self.lb.countdown = 0;
                    self.lb.seq += 1;
                    self.controls_send();
                }
            } else {
                self.controls = [w[0], w[1]];
            }
        } else if m.starts_with("N ") {
            self.peer_name = m[2..].chars().filter(|c| (' '..'\x7f').contains(c)).take(23).collect();
            self.lb.seq += 1;
        } else if host && b[0] == b'A' {
            let mut it = m[2..].split_whitespace();
            let act = it.next().unwrap_or("").to_string();
            let a: i32 = it.next().and_then(|t| t.parse().ok()).unwrap_or(0);
            let bb: i32 = it.next().and_then(|t| t.parse().ok()).unwrap_or(0);
            if !act.is_empty() && self.lb.apply(1, &act, a, bb, false) {
                self.lb.seq += 1;
                self.broadcast();
            }
        } else if !host && b[0] == b'L' {
            if self.lb.list_rx(m, &*self.env) {
                self.lb.seq += 1;
            }
        } else if !host && b[0] == b'S' {
            self.lb.decode(m);
        } else if !host && b[0] == b'C' {
            self.xtras = m.get(2..).unwrap_or("").to_string();
        } else if !host && b[0] == b'G' {
            if let Some((sd, scene)) = m[2..].split_once(' ') {
                if let Ok(seed) = sd.parse::<u32>() {
                    self.seed = seed;
                    self.scene = format!("{scene}{}", self.xtras);
                    self.xtras.clear();
                    self.lb.phase = LB_GO;
                    self.lb.seq += 1;
                }
            }
        }
    }

    /// lb_go (host): the countdown ran out; send the match.
    fn lb_go(&mut self) {
        self.s.stage_ext = self.lb.stage_ext[self.lb.chosen.max(0) as usize];
        self.scene = self.build_scene_for(self.lb.ck[0], self.lb.color[0], self.lb.ck[1], self.lb.color[1]);
        self.lb.phase = LB_GO;
        self.controls_send();
        let g = format!("G {} {}", self.seed, self.scene);
        self.send(g);
        self.lb.seq += 1;
    }

    fn list_pump(&mut self) {
        while self.lb.list_tx >= 0 && self.lb.list_tx < self.lb.nstages {
            let (m, end) = self.lb.list_encode(self.lb.list_tx, &*self.env);
            self.send(m);
            self.lb.list_tx = end;
        }
        if self.lb.list_tx >= self.lb.nstages {
            self.lb.list_tx = -1;
        }
    }
}

struct RoomCb {
    sh: Rc<RefCell<Shared>>,
    inner: Box<dyn Callbacks>,
}

impl Callbacks for RoomCb {
    fn remote_input(&mut self, frame: i32, slot: usize, payload: &[u8]) {
        self.inner.remote_input(frame, slot, payload)
    }
    fn has_local_input(&self) -> bool {
        self.inner.has_local_input()
    }
    fn local_input(&mut self, frame: i32, slot: usize, out: &mut [u8]) -> bool {
        self.inner.local_input(frame, slot, out)
    }
    fn has_checksum(&self) -> bool {
        self.inner.has_checksum()
    }
    fn checksum(&mut self, frame: i32) -> Option<u32> {
        self.inner.checksum(frame)
    }
    fn event(&mut self, ev: Event, msg: Option<&str>) {
        {
            let mut s = self.sh.borrow_mut();
            s.log.push(format!("{ev:?}{}", msg.map(|m| format!(" - {m}")).unwrap_or_default()));
            match ev {
                Event::Started => s.started = true,
                Event::Refused | Event::Disconnected => {
                    s.dead = true;
                    let m = format!("{}: {}", if ev == Event::Refused { "Refused" } else { "Disconnected" }, msg.unwrap_or("the other player left"));
                    s.status(m);
                }
                _ => {}
            }
        }
        self.inner.event(ev, msg)
    }
    fn desync(&mut self, frame: i32, local: u32, remote: u32) {
        self.inner.desync(frame, local, remote)
    }
    fn guest_hello(&mut self, info: &[u8], blob: &mut [u8], blob_len: &mut u16) {
        self.sh.borrow_mut().guest_hello(info, blob, blob_len)
    }
    fn has_lobby(&self) -> bool {
        true
    }
    fn lobby_msg(&mut self, data: &[u8]) {
        self.sh.borrow_mut().lobby_msg(data)
    }
    fn now_ms(&self) -> Option<u32> {
        self.inner.now_ms()
    }
}

/// One room (`np`).
pub struct Room {
    pub sh: Rc<RefCell<Shared>>,
    pub net: Option<Net>,
}

impl Room {
    /// `np_begin` for a direct connection: the transport is already open (and wrapped by the
    /// caller for a server / network simulation). `inner` is the rollback session's callbacks.
    pub fn begin(s: RoomSettings, env: Box<dyn RoomEnv>, inner: Box<dyn Callbacks>, tp: Box<dyn Transport>) -> Result<Room, String> {
        let t0 = std::time::Instant::now();
        let seed = s.seed.unwrap_or_else(|| {
            let n = (t0.elapsed().as_nanos() as u32 ^ std::process::id()).wrapping_mul(2654435761);
            if n == 0 {
                1
            } else {
                n
            }
        });
        let sh = Rc::new(RefCell::new(Shared {
            controls: [s.controls, 0],
            s: s.clone(),
            phase: Phase::Working,
            status: String::new(),
            scene: String::new(),
            seed,
            peer_name: String::new(),
            lb: Lobby::new(),
            outbox: Vec::new(),
            started: false,
            dead: false,
            accepted: false,
            peer_gone: false,
            env,
            log: Vec::new(),
            xtras: String::new(),
            t0,
        }));
        let cb = Box::new(RoomCb { sh: sh.clone(), inner });
        let mut cfg = Config::default();
        {
            let b = sh.borrow();
            cfg.exe_hash = s.exe_hash;
            cfg.iso_hash = 0;
            cfg.mods_hash = b.env.global_hash();
            cfg.mods_desc = Some(b.env.global_describe());
            cfg.mods_soft = s.host && s.use_lobby;
            cfg.first_frame = NP_FIRST_FRAME;
            cfg.payload_bytes = crate::NP_PAYLOAD as u8;
            cfg.hold_start = true;
            cfg.handshake_timeout_ms = 180_000;
            cfg.frame_us = 16667;
        }
        let net = if s.host {
            cfg.seed = seed;
            cfg.input_delay = s.delay as u8;
            cfg.host_slots = 0x01;
            cfg.guest_slots = 0x04;
            let scene = sh.borrow().build_scene_for(s.ck, s.color, 9, 0); // until the guest says
            cfg.match_blob = scene.clone().into_bytes();
            cfg.match_blob.push(0);
            sh.borrow_mut().scene = scene;
            Net::host(&cfg, cb, tp)
        } else {
            let info = format!("{}/c{}/j{}", sh.borrow().env.token_for_ck(s.ck), s.color, s.controls);
            cfg.guest_info = info.into_bytes();
            let Some(peer) = s.peer else { return Err("no host address".into()) };
            Net::join(&cfg, cb, tp, peer)
        };
        let Some(net) = net else { return Err("could not start the session".into()) };
        Ok(Room { sh, net: Some(net) })
    }

    pub fn phase(&self) -> Phase {
        self.sh.borrow().phase
    }

    fn fail(&mut self, why: &str) {
        let mut s = self.sh.borrow_mut();
        s.status(why.to_string());
        s.phase = Phase::Failed;
        drop(s);
        self.net = None;
    }

    fn drain_outbox(&mut self) {
        let Some(net) = self.net.as_mut() else { return };
        let mut s = self.sh.borrow_mut();
        let mut n = 0;
        for m in s.outbox.iter() {
            if net.lobby_send(m) != 0 {
                break;
            }
            n += 1;
        }
        s.outbox.drain(..n);
    }

    /// np_poll: one step of the connection. Returns the phase.
    pub fn poll(&mut self) -> Phase {
        let ph = self.sh.borrow().phase;
        if ph != Phase::Working {
            return ph;
        }
        let Some(net) = self.net.as_mut() else { return Phase::Failed };
        net.poll(NP_FIRST_FRAME);
        self.drain_outbox();
        if self.sh.borrow().dead {
            let why = self.sh.borrow().status.clone();
            self.fail(&why);
            return Phase::Failed;
        }
        if !self.sh.borrow().accepted {
            let st = self.net.as_ref().unwrap().state();
            if st >= State::Accepted && st <= State::Running {
                let host = self.sh.borrow().s.host;
                if !host {
                    let Some(hc) = self.net.as_ref().unwrap().remote_config() else { return Phase::Working };
                    if let Err(e) = self.guest_take(&hc) {
                        self.fail(&e);
                        return Phase::Failed;
                    }
                }
                let use_lobby = {
                    let mut s = self.sh.borrow_mut();
                    s.accepted = true;
                    s.s.use_lobby
                };
                if use_lobby {
                    self.lobby_enter();
                } else {
                    let mut s = self.sh.borrow_mut();
                    s.phase = Phase::Connected;
                    s.status("Connected! Starting the match...");
                }
            }
        }
        self.sh.borrow().phase
    }

    /// The guest's reading of the host's ACCEPT (the "J c0 c1 scene" blob).
    fn guest_take(&mut self, hc: &crate::net::RemoteConfig) -> Result<(), String> {
        let blob = String::from_utf8_lossy(&hc.match_blob).trim_end_matches('\0').to_string();
        let mut s = self.sh.borrow_mut();
        let mut it = blob.splitn(4, ' ');
        let (j, c0, c1, scene) = (it.next(), it.next().and_then(|t| t.parse::<u32>().ok()), it.next().and_then(|t| t.parse::<u32>().ok()), it.next());
        let (Some("J"), Some(c0), Some(c1), Some(scene)) = (j, c0, c1, scene) else {
            return Err("malformed fighter controls in ACCEPT; refusing match".into());
        };
        if !controls_word_valid(c0) || !controls_word_valid(c1) {
            return Err("malformed fighter controls in ACCEPT; refusing match".into());
        }
        s.controls = [c0, c1];
        s.scene = scene.to_string();
        let Some(p) = parse_scene(scene) else { return Err(format!("the host's match \"{scene}\" is not understood")) };
        s.env.scene_check(&p).map_err(|why| format!("The host's match uses content you don't have: {why}"))?;
        s.seed = hc.seed;
        s.s.delay = hc.input_delay as i32;
        if hc.rules != 0 || p.turbo != hc.rules {
            return Err("The host's match rules are not supported by this version".into());
        }
        if hc.envoy != 0 {
            return Err("The host's room uses Envoy or a stage run, which this build does not implement".into());
        }
        if !p.extras.is_empty() {
            s.log.push(format!("scene extras not implemented here: {:?}", p.extras));
        }
        Ok(())
    }

    /// np_lobby_enter.
    fn lobby_enter(&mut self) {
        let mut s = self.sh.borrow_mut();
        s.phase = Phase::Lobby;
        s.controls_send();
        if !s.s.name.is_empty() {
            let n = format!("N {}", s.s.name);
            s.send(n);
        }
        let seed = s.seed;
        s.lb.seed = seed;
        let host = s.s.host;
        let (ck, mode) = (s.s.ck, s.s.stage_mode);
        let mut lb = std::mem::take(&mut s.lb);
        {
            let e: &dyn LobbyEnv = &*s.env;
            if host {
                if lb.game <= 0 {
                    lb.mode = mode;
                }
                lb.build_list(e);
            } else if !lb.have_state {
                lb.rx_id = -1;
                lb.list_ok = false;
            }
            if lb.nstages <= 0 || lb.stage_ext[0] == 0 {
                lb.default_stages(e);
            }
            if host || !lb.have_state {
                if lb.game <= 0 {
                    let (a, b) = if host { (ck, 9) } else { (2, ck) };
                    lb.new_set(a, b, e);
                }
                lb.reset_game(e);
            }
        }
        s.lb = lb;
        s.status("Opponent found! Opening the lobby...");
        if host {
            s.lb.list_tx = 0;
            s.list_pump();
            s.broadcast();
        }
        drop(s);
        self.drain_outbox();
    }

    /// np_lobby_tick: every frame while in the lobby.
    pub fn lobby_tick(&mut self) {
        if self.sh.borrow().phase != Phase::Lobby {
            return;
        }
        let Some(net) = self.net.as_mut() else { return };
        net.poll(NP_FIRST_FRAME);
        if {
            let s = self.sh.borrow();
            s.dead || s.peer_gone
        } {
            let mut s = self.sh.borrow_mut();
            s.phase = Phase::Failed;
            s.lb.game = 0;
            drop(s);
            self.net = None;
            return;
        }
        {
            let mut s = self.sh.borrow_mut();
            if s.s.host {
                s.list_pump();
            }
            let now = s.now_ms();
            if s.lb.countdown > 0 && s.lb.cd_t0.is_none() {
                let back = ((LB_COUNTDOWN - s.lb.countdown) as u32) * 1000 / 60;
                s.lb.cd_t0 = Some(now.wrapping_sub(back).max(1));
            } else if s.lb.countdown <= 0 {
                s.lb.cd_t0 = None;
            }
            if s.lb.countdown > 0 {
                let t0 = s.lb.cd_t0.unwrap_or(now);
                let was = (s.lb.countdown + 59) / 60;
                let elapsed = (now.wrapping_sub(t0) * 60 / 1000) as i32;
                if s.s.host {
                    if s.lb.countdown_at(elapsed) {
                        s.lb_go();
                    } else if s.lb.countdown == 0 || (s.lb.countdown + 59) / 60 != was {
                        s.lb.seq += 1;
                        s.broadcast();
                    }
                } else if s.lb.phase == LB_READY && elapsed < LB_COUNTDOWN - 1 {
                    s.lb.countdown = LB_COUNTDOWN - elapsed;
                    if (s.lb.countdown + 59) / 60 != was {
                        s.lb.seq += 1;
                    }
                }
            }
            if s.lb.phase == LB_GO && s.phase == Phase::Lobby {
                s.phase = Phase::Connected;
                s.status("Starting the match...");
            }
        }
        self.drain_outbox();
    }

    // ---- local lobby actions (the menu) ----
    pub fn lobby_char(&mut self, ck: i32, color: i32) {
        self.sh.borrow_mut().lb_action("CHAR", ck, color);
        self.drain_outbox();
    }
    pub fn lobby_stage_act(&mut self, i: i32) {
        let mut s = self.sh.borrow_mut();
        let act = match s.lb.phase {
            LB_STRIKE => "STRIKE",
            LB_BAN => "BAN",
            LB_PICK => "PICK",
            _ => return,
        };
        s.lb_action(act, i, 0);
        drop(s);
        self.drain_outbox();
    }
    pub fn lobby_ready(&mut self, on: bool) {
        self.sh.borrow_mut().lb_action("READY", on as i32, 0);
        self.drain_outbox();
    }

    /// Release the held start once the match is loaded (`gw_Netplay_Handshake`).
    pub fn release(&mut self) {
        if let Some(n) = self.net.as_mut() {
            n.release();
        }
    }
}
