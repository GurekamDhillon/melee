//! Slippi `.slp` export of the matches `melee play` runs (`--slp FILE|auto`).
//!
//! Written from the public spec (project-slippi/slippi-wiki SPEC.md), Slippi 3.18.0 layout: Event Payloads, Game Start
//! (0x36, the Game Info Block encoded GameCube-style from the native `StartMeleeData`), per frame Frame Start (0x3A),
//! one Pre-Frame (0x37) and one Post-Frame (0x38) per fighter, Frame Bookend (0x3C), then Game End (0x39) and the
//! UBJSON `metadata` element. Which fields are faithful is listed in docs/rust-port/notes/net.md ("Slippi export").
//!
//! The events come from the fighter code's own replay hooks (`Replay_RecordInput` at the pre-frame point,
//! `Replay_TraceFighter` at the 3.x post-frame point) and the VS mode's `Replay_RecordMatch`; `begin_tick` /
//! `end_tick` around each engine tick group them into frames (the first tick with fighter input is frame -123). The
//! hooks see every simulated frame, so a run that rolls back must not record live: netplay exports from its `.mrpl`
//! playback instead (`play_net`).

use std::ffi::c_void;

const V: [u8; 4] = [3, 18, 0, 0];
const SZ_START: usize = 0x2F8;
const SZ_PRE: usize = 0x42;
const SZ_POST: usize = 0x54;
const SZ_FSTART: usize = 0xC;
const SZ_BOOK: usize = 8;
const SZ_END: usize = 6;

struct Pre {
    port: u8,
    fol: bool,
    ev: Vec<u8>,
}

struct Match {
    raw: Vec<u8>,
    frame: i32,
    frames: u32,
    started: bool,
    seed_at_tick: u32,
    pads: [melee_types::PADStatus; 4],
    pre: Vec<Pre>,
    post: Vec<Vec<u8>>,
    /// per port: (internal char id -> frames) and the last stocks / percent seen
    chars: [Vec<(u8, u32)>; 4],
    last: [Option<(u8, f32)>; 4],
    present: [bool; 4],
    timer_secs: u32,
    timed: bool,
    start_at: String,
}

pub struct Exporter {
    /// `auto` (`MELEE_SLP_DIR`, default Replays, /Game_<time>.slp) or a path (a second match gets `-2` before the extension, and so on)
    spec: String,
    played_on: String,
    names: [String; 4],
    m: Option<Match>,
    count: u32,
    pub written: Vec<String>,
}

static mut EXP: Option<Exporter> = None;

#[allow(static_mut_refs)]
fn exp() -> Option<&'static mut Exporter> {
    unsafe { EXP.as_mut() }
}

fn seed_now() -> u32 {
    let st = melee_runtime::tu::random::global::state();
    unsafe { *melee_runtime::tu::random::HSD_RandCurrentSeed(st) }
}

fn put32(v: &mut [u8], o: usize, x: u32) {
    v[o..o + 4].copy_from_slice(&x.to_be_bytes());
}
fn putf(v: &mut [u8], o: usize, x: f32) {
    put32(v, o, x.to_bits());
}

/// The Game Info Block (312 bytes, GameCube layout) from the native `StartMeleeData`.
pub fn game_info(sm: &melee_types::StartMeleeData) -> Vec<u8> {
    let r = &sm.rules;
    let mut g = vec![0u8; 0x138];
    g[0..6].copy_from_slice(&r.bf_0); // Game Bitfields 1-4 and the two bytes after them, already in GC order
    g[6] = r.x6;
    g[7] = r.x7;
    g[8] = r.is_teams;
    g[9] = r.x9;
    g[0xA] = r.xA;
    g[0xB] = r.item_freq as u8;
    g[0xC] = r.sd_penalty as u8;
    g[0xD] = r.xD;
    g[0xE..0x10].copy_from_slice(&r.stkind.to_be_bytes());
    put32(&mut g, 0x10, r.time_limit);
    g[0x14] = r.x14;
    put32(&mut g, 0x18, r.x18);
    g[0x20..0x28].copy_from_slice(&r.x20.to_be_bytes()); // Item Spawn Bitfields 1-5 are bytes 0x23..0x27
    put32(&mut g, 0x28, r.it_kind as u32);
    putf(&mut g, 0x2C, r.x2C);
    putf(&mut g, 0x30, r.x30);
    putf(&mut g, 0x34, r.game_speed);
    for (i, p) in sm.players.iter().enumerate() {
        let b = 0x60 + 0x24 * i;
        let head = [p.ckind as u8, p.slot_type, p.stocks as u8, p.color, p.slot, p.spawn_pos as u8, p.spawn_dir as u8, p.sub_color, p.handicap as u8, p.team, p.nametag, p.xB];
        g[b..b + 12].copy_from_slice(&head);
        g[b + 0xC..b + 0xE].copy_from_slice(&p.bf_c);
        g[b + 0xE] = p.cpu_kind;
        g[b + 0xF] = p.cpu_level;
        g[b + 0x10..b + 0x12].copy_from_slice(&p.damage.to_be_bytes());
        g[b + 0x12..b + 0x14].copy_from_slice(&p.damage1.to_be_bytes());
        g[b + 0x14..b + 0x16].copy_from_slice(&p.hp.to_be_bytes());
        putf(&mut g, b + 0x18, p.attack_ratio);
        putf(&mut g, b + 0x1C, p.defense_ratio);
        putf(&mut g, b + 0x20, p.model_scale);
    }
    g
}

/// ISO 8601 UTC from the system clock.
fn iso_now() -> String {
    let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs() as i64);
    let (days, s) = (t.div_euclid(86400), t.rem_euclid(86400));
    // civil from days (H. Hinnant)
    let z = days + 719468;
    let era = z.div_euclid(146097);
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + (m <= 2) as i64;
    format!("{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z", s / 3600, s / 60 % 60, s % 60)
}

fn ub_key(o: &mut Vec<u8>, k: &str) {
    o.push(b'U');
    o.push(k.len() as u8);
    o.extend_from_slice(k.as_bytes());
}
fn ub_str(o: &mut Vec<u8>, s: &str) {
    o.extend_from_slice(b"SU");
    o.push(s.len().min(255) as u8);
    o.extend_from_slice(&s.as_bytes()[..s.len().min(255)]);
}
fn ub_i32(o: &mut Vec<u8>, v: i32) {
    o.push(b'l');
    o.extend_from_slice(&v.to_be_bytes());
}

impl Exporter {
    fn path_for(&mut self) -> String {
        self.count += 1;
        if self.spec == "auto" {
            let t = iso_now().replace([':', '-'], "");
            let t = t.trim_end_matches('Z');
            let dir = std::env::var("MELEE_SLP_DIR").ok().filter(|d| !d.is_empty()).unwrap_or_else(|| "Replays".into());
            let base = format!("{dir}/Game_{t}");
            let mut p = format!("{base}.slp");
            let mut k = 2;
            while std::path::Path::new(&p).exists() {
                p = format!("{base}-{k}.slp");
                k += 1;
            }
            p
        } else if self.count == 1 {
            self.spec.clone()
        } else {
            let p = std::path::Path::new(&self.spec);
            let stem = p.file_stem().map_or("game".into(), |s| s.to_string_lossy().to_string());
            let ext = p.extension().map_or("slp".into(), |s| s.to_string_lossy().to_string());
            p.with_file_name(format!("{stem}-{}.{ext}", self.count)).to_string_lossy().to_string()
        }
    }

    fn start(&mut self, sm: &melee_types::StartMeleeData, seed: u32) {
        let mut raw = vec![0x35, 1 + 3 * 6];
        for (c, s) in [(0x36u8, SZ_START), (0x37, SZ_PRE), (0x38, SZ_POST), (0x39, SZ_END), (0x3A, SZ_FSTART), (0x3C, SZ_BOOK)] {
            raw.push(c);
            raw.extend_from_slice(&(s as u16).to_be_bytes());
        }
        let mut ev = vec![0u8; 1 + SZ_START];
        ev[0] = 0x36;
        ev[1..5].copy_from_slice(&V);
        let gi = game_info(sm);
        ev[5..5 + 0x138].copy_from_slice(&gi);
        put32(&mut ev, 0x13D, seed);
        // controller fixes (dashback / shield drop words per port, 1 = UCF) and Frozen PS (0x1A2): what this engine
        // plays with, i.e. the UCF mode (`--codes ucf`, else the `ucf` setting) and the frozen-stage codes
        let ucf = (melee_fighter::platform::Ucf_OfflineMode() == 84) as u32;
        for p in 0..4 {
            put32(&mut ev, 0x141 + 8 * p, ucf);
            put32(&mut ev, 0x145 + 8 * p, ucf);
        }
        ev[0x1A2] = melee_core::codes::on(melee_core::codes::FROZEN_PS | melee_core::codes::FROZEN_STAGES) as u8;
        ev[0x1A3] = 2; // minor scene
        ev[0x1A4] = 2; // major scene: VS mode
        ev[0x2BD] = 1; // English
        raw.extend_from_slice(&ev);
        let mut present = [false; 4];
        for (i, p) in present.iter_mut().enumerate() {
            *p = gi[0x61 + 0x24 * i] != 3;
        }
        let timed = gi[0] & 3 == 2;
        self.m = Some(Match { raw, frame: -124, present, timer_secs: sm.rules.time_limit, timed, start_at: iso_now(), started: false, seed_at_tick: 0, pads: [melee_types::PADStatus::zeroed(); 4], pre: Vec::new(), post: Vec::new(), chars: Default::default(), last: [None; 4], frames: 0 });
    }

    fn end_match(&mut self, quit: bool) {
        let Some(mut m) = self.m.take() else { return };
        if !m.started {
            return;
        }
        // Game End: GAME! when one fighter (or none) has stocks left, TIME! when a timed match ran its clock, else no contest
        let alive: Vec<usize> = (0..4).filter(|&p| m.present[p] && m.last[p].map_or(false, |l| l.0 > 0)).collect();
        let method = if alive.len() <= 1 {
            2
        } else if !quit && m.timed && m.frames as u64 >= (m.timer_secs as u64) * 60 + 123 {
            1
        } else {
            7
        };
        let mut order: Vec<usize> = (0..4).filter(|&p| m.present[p]).collect();
        order.sort_by(|&a, &b| {
            let (sa, pa) = m.last[a].unwrap_or((0, 0.0));
            let (sb, pb) = m.last[b].unwrap_or((0, 0.0));
            sb.cmp(&sa).then(pa.partial_cmp(&pb).unwrap_or(std::cmp::Ordering::Equal))
        });
        let mut place = [-1i8; 4];
        for (k, &p) in order.iter().enumerate() {
            place[p] = k as i8;
        }
        m.raw.extend_from_slice(&[0x39, method, 0xFF, place[0] as u8, place[1] as u8, place[2] as u8, place[3] as u8]);
        let mut o = b"{U\x03raw[$U#l".to_vec();
        o.extend_from_slice(&(m.raw.len() as u32).to_be_bytes());
        o.extend_from_slice(&m.raw);
        ub_key(&mut o, "metadata");
        o.push(b'{');
        ub_key(&mut o, "startAt");
        ub_str(&mut o, &m.start_at);
        ub_key(&mut o, "lastFrame");
        ub_i32(&mut o, m.frame);
        ub_key(&mut o, "players");
        o.push(b'{');
        for p in 0..4 {
            if !m.present[p] {
                continue;
            }
            ub_key(&mut o, &p.to_string());
            o.push(b'{');
            ub_key(&mut o, "characters");
            o.push(b'{');
            for (c, n) in &m.chars[p] {
                ub_key(&mut o, &c.to_string());
                ub_i32(&mut o, *n as i32);
            }
            o.push(b'}');
            ub_key(&mut o, "names");
            o.push(b'{');
            ub_key(&mut o, "netplay");
            ub_str(&mut o, &self.names[p]);
            o.push(b'}');
            o.push(b'}');
        }
        o.push(b'}');
        // the opt-in gameplay codes (melee_core::codes): word (hex, `:limit` with LGL) and names; our own key
        let (cw, cl) = (melee_core::codes::word(), melee_core::codes::lgl_limit());
        ub_key(&mut o, "gameplayCodes");
        ub_str(&mut o, &melee_core::codes::encode(cw, cl));
        ub_key(&mut o, "gameplayCodesNames");
        ub_str(&mut o, &melee_core::codes::describe(cw, cl));
        ub_key(&mut o, "playedOn");
        ub_str(&mut o, &self.played_on);
        o.push(b'}');
        o.push(b'}');
        let path = self.path_for();
        if let Some(d) = std::path::Path::new(&path).parent().filter(|d| !d.as_os_str().is_empty()) {
            let _ = std::fs::create_dir_all(d);
        }
        match std::fs::write(&path, &o) {
            Ok(()) => {
                eprintln!("gw: slp: {} frames ({}..{}) written to {path}", m.frames, -123, m.frame);
                self.written.push(path);
            }
            Err(e) => eprintln!("gw: slp: {path}: {e}"),
        }
    }
}

/// `--slp FILE|auto`: install the hooks (before the engine boots). `played_on` goes to the metadata (`playedOn`).
pub fn install(spec: &str, played_on: &str, names: [String; 4]) {
    use melee_fighter::platform::hook as h;
    unsafe {
        EXP = Some(Exporter { spec: spec.to_string(), played_on: played_on.to_string(), names, m: None, count: 0, written: Vec::new() });
        melee_modes::tu::gmvs::ext::hook::Replay_RecordMatch = Some(record_match);
        h::Replay_RecordInput = Some(record_input);
        h::Replay_Tracing = Some(|| exp().map_or(0, |x| x.m.is_some() as i32));
        h::Replay_TraceAtProcMap = Some(|| 0);
        h::Replay_TraceFighter = Some(trace_fighter);
    }
}

pub fn active() -> bool {
    exp().is_some()
}

fn record_match(sm: *mut c_void, seed: u32) {
    let Some(x) = exp() else { return };
    x.end_match(true);
    let sm = unsafe { &*(sm as *const melee_types::StartMeleeData) };
    x.start(sm, seed);
}

#[allow(clippy::too_many_arguments)]
fn record_input(port: i32, fol: i32, lx: f32, ly: f32, cx: f32, cy: f32, t: f32, b: u32, action: i32, x: f32, y: f32, facing: f32, pct: f32) {
    let Some(m) = exp().and_then(|e| e.m.as_mut()) else { return };
    if !(0..4).contains(&port) {
        return;
    }
    if m.pre.iter().any(|p| p.port == port as u8 && p.fol == (fol != 0)) {
        // this fighter's second pre-frame in one engine tick (the scene loop ran two logic frames): close the first
        flush_frame(m);
        m.seed_at_tick = seed_now();
    }
    let pad = m.pads[port as usize];
    let mut ev = vec![0u8; 1 + SZ_PRE];
    ev[0] = 0x37;
    ev[5] = port as u8;
    ev[6] = (fol != 0) as u8;
    put32(&mut ev, 0x7, seed_now());
    ev[0xB..0xD].copy_from_slice(&(action as u16).to_be_bytes());
    putf(&mut ev, 0xD, x);
    putf(&mut ev, 0x11, y);
    putf(&mut ev, 0x15, facing);
    putf(&mut ev, 0x19, lx);
    putf(&mut ev, 0x1D, ly);
    putf(&mut ev, 0x21, cx);
    putf(&mut ev, 0x25, cy);
    putf(&mut ev, 0x29, t);
    put32(&mut ev, 0x2D, b);
    // the controller this frame: GC pad button bits are the spec's Physical Buttons
    ev[0x31..0x33].copy_from_slice(&(pad.button & 0x1F7F).to_be_bytes());
    putf(&mut ev, 0x33, (pad.triggerLeft as f32 / 140.0).min(1.0));
    putf(&mut ev, 0x37, (pad.triggerRight as f32 / 140.0).min(1.0));
    ev[0x3B] = pad.stickX as u8;
    putf(&mut ev, 0x3C, pct);
    ev[0x40] = pad.stickY as u8;
    ev[0x41] = pad.substickX as u8;
    ev[0x42] = pad.substickY as u8;
    m.pre.push(Pre { port: port as u8, fol: fol != 0, ev });
}

#[allow(clippy::too_many_arguments)]
fn trace_fighter(port: i32, fol: i32, ck: i32, action: i32, x: f32, y: f32, facing: f32, pct: f32, stocks: i32, ax: f32, ay: f32, kx: f32, ky: f32, gx: f32) {
    let Some(m) = exp().and_then(|e| e.m.as_mut()) else { return };
    if !(0..4).contains(&port) || m.pre.is_empty() {
        return;
    }
    let mut ev = vec![0u8; 1 + SZ_POST];
    ev[0] = 0x38;
    ev[5] = port as u8;
    ev[6] = (fol != 0) as u8;
    ev[7] = ck as u8;
    ev[8..10].copy_from_slice(&(action as u16).to_be_bytes());
    putf(&mut ev, 0xA, x);
    putf(&mut ev, 0xE, y);
    putf(&mut ev, 0x12, facing);
    putf(&mut ev, 0x16, pct);
    ev[0x20] = 6; // last hit by: none (overwritten below)
    ev[0x21] = stocks as u8;
    putf(&mut ev, 0x35, ax);
    putf(&mut ev, 0x39, ay);
    putf(&mut ev, 0x3D, kx);
    putf(&mut ev, 0x41, ky);
    putf(&mut ev, 0x45, gx);
    let g = if fol != 0 { melee_fighter::player::get_entity_at_index(port, 1) } else { melee_fighter::player::get_entity(port) };
    unsafe {
        if !g.is_null() && !(*g).user_data.is_null() {
            let fp = &*((*g).user_data as *const melee_types::Fighter);
            putf(&mut ev, 0x1A, fp.shield_health);
            ev[0x1E] = fp.x208C as u8;
            ev[0x1F] = fp.x2090 as u8;
            let src = fp.dmg.x18c4_source_ply;
            ev[0x20] = if (0..4).contains(&src) { src as u8 } else { 6 };
            putf(&mut ev, 0x22, fp.cur_anim_frame);
            putf(&mut ev, 0x2B, *(&fp.mv as *const _ as *const f32));
            ev[0x2F] = (fp.ground_or_air != melee_types::GA_Ground) as u8;
            ev[0x30..0x32].copy_from_slice(&(fp.coll_data.floor.index as u16).to_be_bytes());
            ev[0x32] = (fp.co_attrs.max_jumps - fp.x1968_jumpsUsed as i32).max(0) as u8;
            ev[0x34] = fp.x198C.clamp(0, 2) as u8;
            putf(&mut ev, 0x49, fp.dmg.x195c_hitlag_frames);
            put32(&mut ev, 0x4D, fp.anim_id as u32);
        }
    }
    if fol == 0 {
        m.last[port as usize] = Some((stocks as u8, pct));
        let ch = &mut m.chars[port as usize];
        match ch.iter_mut().find(|c| c.0 == ck as u8) {
            Some(c) => c.1 += 1,
            None => ch.push((ck as u8, 1)),
        }
    }
    m.post.push(ev);
}

/// Before an engine tick: the controllers and the RNG seed the tick starts from (the Frame Start seed).
pub fn begin_tick(pads: &[melee_types::PADStatus; 4]) {
    if let Some(m) = exp().and_then(|e| e.m.as_mut()) {
        m.pads = *pads;
        m.seed_at_tick = seed_now();
        m.pre.clear();
        m.post.clear();
    }
}

/// After an engine tick: a tick that ran the fighters is one Slippi frame; leaving the match scene ends the file.
pub fn end_tick(in_match: bool) {
    let Some(x) = exp() else { return };
    let Some(m) = x.m.as_mut() else { return };
    if m.pre.is_empty() {
        if m.started && !in_match {
            x.end_match(false);
        }
        return;
    }
    flush_frame(m);
}

/// The buffered events as one Slippi frame (Frame Start, pre-frames, post-frames, Bookend).
fn flush_frame(m: &mut Match) {
    if m.pre.is_empty() {
        return;
    }
    m.started = true;
    m.frame += 1;
    m.frames += 1;
    let f = m.frame as u32;
    let mut fs = vec![0u8; 1 + SZ_FSTART];
    fs[0] = 0x3A;
    put32(&mut fs, 1, f);
    put32(&mut fs, 5, m.seed_at_tick);
    put32(&mut fs, 9, m.frames - 1);
    m.raw.extend_from_slice(&fs);
    // the spec's order: every pre-frame (port order, leader before follower), then every post-frame
    let mut pre = std::mem::take(&mut m.pre);
    pre.sort_by_key(|p| (p.port, p.fol));
    for mut p in pre {
        put32(&mut p.ev, 1, f);
        m.raw.extend_from_slice(&p.ev);
    }
    let mut post = std::mem::take(&mut m.post);
    post.sort_by_key(|e| (e[5], e[6]));
    for mut e in post {
        put32(&mut e, 1, f);
        m.raw.extend_from_slice(&e);
    }
    let mut bk = [0u8; 1 + SZ_BOOK];
    bk[0] = 0x3C;
    put32(&mut bk, 1, f);
    put32(&mut bk, 5, f);
    m.raw.extend_from_slice(&bk);
}

/// The run ends: a match still open is written as a no contest.
pub fn finish() -> Vec<String> {
    match exp() {
        Some(x) => {
            x.end_match(true);
            x.written.clone()
        }
        None => Vec::new(),
    }
}
