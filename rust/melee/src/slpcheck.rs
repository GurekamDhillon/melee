//! `melee slpcheck --slp FILE [--frames N]`: a Slippi replay as a direct oracle. The engine runs the replay's pre-frame
//! inputs (the existing `Replay_*` hooks), and after every frame the fighter state is compared with the console's own
//! Post-Frame Update (0x38) of that frame. No C run is involved. Also holds the Game Start (0x36) -> scene conversion
//! that `golden --slp` uses.
//!
//! The reader is `melee_data::slp` (written from the public spec); the comparison list and the Slippi version rules
//! (a field is checked only when the file's payload carries it) are in notes/slpcheck.md.

use melee_data::slp::{self, GameInfo, Slp};
use melee_engine::scenario::{PlayerSpec, Scenario};

/// A Game Start block turned into the engine's scene, plus what could not be applied (recorded, not hidden).
pub struct Converted {
    pub scn: Scenario,
    pub notes: Vec<String>,
}

/// Game Start -> `Scenario`: stage, ports and player types, characters, costumes, stocks, timer, teams, items setting.
/// Ports keep the file's player index (the engine's hash/fighter slot is the port). A CPU slot is run as a human: the
/// recorded pre-frame inputs of a CPU already are the AI's output, so replaying them reproduces it.
/// UCF toggle words are only recorded (see [`ucf_summary`]); the vanilla sim stays default.
pub fn convert(s: &Slp) -> Result<Converted, String> {
    let info: &GameInfo = s.info.as_ref().ok_or("the replay has no Game Start block")?;
    let mut notes = Vec::new();
    let mut scn = Scenario::parse("")?;
    scn.stage = info.stage() as i32;
    let mut max_stocks = 0;
    for i in 0..4 {
        let p = info.player(i);
        match p.ptype {
            0 | 1 => {}
            2 => {
                notes.push(format!("p{} demo slot ignored", i + 1));
                continue;
            }
            _ => continue,
        }
        if p.ptype == 1 {
            notes.push(format!("p{} is a CPU (level {}); run as a human on its recorded inputs", i + 1, p.cpu_level));
        }
        if p.ckind > 25 {
            return Err(format!("p{}: character id {} is outside the 26-character roster", i + 1, p.ckind));
        }
        if p.offense != 1.0 || p.defense != 1.0 || p.scale != 1.0 {
            notes.push(format!("p{} offense/defense/scale {}/{}/{} not applied", i + 1, p.offense, p.defense, p.scale));
        }
        if p.damage_start != 0 || p.damage_spawn != 0 {
            notes.push(format!("p{} start/spawn damage {}/{} not applied", i + 1, p.damage_start, p.damage_spawn));
        }
        if p.bitfield & 0x7F != 0 {
            notes.push(format!("p{} player bitfield {:#04x} (stamina/silent/low gravity/invisible/metal...) not applied", i + 1, p.bitfield));
        }
        max_stocks = max_stocks.max(p.stocks as i32);
        scn.players[i] = Some(PlayerSpec {
            ckind: p.ckind as i32,
            slot_type: 0,
            cpu_level: -1,
            color: p.costume as i32,
            team: if info.is_teams() { p.team as i32 } else { -1 },
            stocks: p.stocks as i32,
            handicap: p.handicap as i32,
            cpu_idle: false,
        });
    }
    if scn.players.iter().flatten().count() < 2 {
        return Err("fewer than two fighters".into());
    }
    // Game Bitfield 1: mode (0 time, 1 stock), timer behaviour; the timer is in seconds
    scn.rule_match = match info.game_mode() {
        0 => 0,
        1 => 1,
        m => return Err(format!("game mode {m} (coin/bonus) is not a VS match")),
    };
    match info.timer_behavior() {
        2 => {
            scn.time_limit = info.timer_secs() as i32;
            scn.rule_minutes = info.timer_secs().div_ceil(60) as i32;
        }
        0 => {
            scn.time_limit = 0;
            scn.rule_minutes = 0;
        }
        b => {
            scn.time_limit = 0;
            scn.rule_minutes = 0;
            notes.push(format!("timer behaviour {b} (count up) run as no timer"));
        }
    }
    scn.rule_stocks = max_stocks;
    scn.teams = info.is_teams() as i32;
    scn.item_freq = if info.item_spawn() < 0 { -2 } else { info.item_spawn() as i32 };
    if info.item_spawn() >= 0 {
        notes.push(format!("items on (frequency {}), item mask {:02x?}: item spawns are not driven by the file", info.item_spawn(), info.item_bits()));
    }
    if info.damage_ratio() != 1.0 {
        notes.push(format!("damage ratio {} not applied", info.damage_ratio()));
    }
    if info.bomb_rain() != 0 {
        notes.push("bomb rain on".into());
    }
    if s.pal {
        notes.push("PAL on (not applied)".into());
    }
    if s.frozen_ps {
        notes.push("frozen Pokemon Stadium on".into());
    }
    Ok(Converted { scn, notes })
}

/// `dashback/shield-drop` toggle words of the ports in use: 0 off, 1 UCF, 2 Dween.
pub fn ucf_summary(s: &Slp) -> String {
    let mut parts = Vec::new();
    for i in 0..4 {
        if s.info.as_ref().is_some_and(|g| g.player(i).ptype < 2) {
            parts.push(format!("{}{}", s.dashback[i], s.shield_drop[i]));
        }
    }
    parts.join("/")
}

/// Short fixed-width stage / roster summary for the CSV.
fn roster(s: &Slp) -> String {
    let Some(g) = s.info.as_ref() else { return String::new() };
    (0..4).filter(|&i| g.player(i).ptype < 2).map(|i| format!("p{}:{}", i + 1, g.player(i).ckind)).collect::<Vec<_>>().join("+")
}

// ---- capture of the engine's post-frame state ---------------------------------------------------------------------

#[derive(Clone, Copy, Default)]
struct Cap {
    action: u16,
    x: f32,
    y: f32,
    facing: f32,
    percent: f32,
    stocks: u8,
    state_frame: f32,
    shield: f32,
    last_attack: u8,
    combo: u8,
    hitlag: f32,
    airborne: u8,
    cand: [i64; 6],
}

static mut CAP: [Option<Cap>; 4] = [None; 4];
/// Slippi 1.x/2.x recorded post-frame at `Fighter_procMap`, 3.x at `Fighter_UnkCallCameraCallback` (gw_Replay_TraceAtProcMap)
static mut AT_PROC_MAP: bool = false;

/// Route the fighter code's trace hook (`ftReplay_TraceFighter`, run at the end of `Fighter_procMap`, the end of the
/// frame's collision work as Slippi's post-frame is) into [`CAP`]. Before this only the replay input hooks were set.
fn install_capture(at_proc_map: bool) {
    use melee_fighter::platform::hook as h;
    unsafe {
        AT_PROC_MAP = at_proc_map;
        h::Replay_Tracing = Some(|| 1);
        h::Replay_TraceAtProcMap = Some(|| unsafe { AT_PROC_MAP } as i32);
        h::Replay_TraceFighter = Some(|port, fol, _ck, action, x, y, facing, pct, stocks, _ax, _ay, _kx, _ky, _gx| {
            if fol != 0 || !(0..4).contains(&port) {
                return;
            }
            let g = melee_fighter::player::get_entity(port);
            if g.is_null() || (*g).user_data.is_null() {
                return;
            }
            let fp = &*((*g).user_data as *const melee_types::Fighter);
            CAP[port as usize] = Some(Cap {
                action: action as u16,
                x,
                y,
                facing,
                percent: pct,
                stocks: stocks as u8,
                state_frame: fp.cur_anim_frame,
                shield: fp.shield_health,
                last_attack: fp.x208C as u8,
                combo: fp.x2090 as u8,
                hitlag: fp.dmg.x195c_hitlag_frames,
                airborne: (fp.ground_or_air != melee_types::GA_Ground) as u8,
                cand: [fp.x2068_attackID as i64, fp.x208C as i64, fp.x2090 as i64, fp.x2092 as i64, fp.x2098 as i64, fp.x206C_attack_instance as i64],
            });
        });
    }
}

/// EXPERIMENTAL (env SLPCHECK_SEEDS=1; unexplained, see notes/slpcheck.md): first fighter whose RNG state at its pre-frame point differs from the Random Seed its Pre-Frame Update (0x37) recorded:
/// (frame, port, engine seed, recorded seed). The pre-frame point is `Fighter_Spaghetti_8006AD10` (Slippi hooks 0x8006B0E0, the
/// `Replay_HasInput` call), so a mismatch means the engine drew a different number of random values earlier in that frame.
static mut SEED_FIRST: Option<(i32, i32, u32, u32)> = None;
static mut SEED_PROBE: bool = false;
static mut RESEED_PRE: bool = false;
static mut SLP_SEED: u32 = 0;
static mut SEEDLOG_LAST: u32 = 0;

pub(crate) fn seed_probe(port: i32, fol: i32) {
    unsafe {
        if std::env::var_os("SLPCHECK_SEEDLOG").is_some() && fol == 0 {
            let st = melee_runtime::tu::random::global::state();
            let eng = *melee_runtime::tu::random::HSD_RandCurrentSeed(st);
            let rec = crate::r_in().input(port, false).map(|i| i.seed).unwrap_or(0);
            let f = crate::r_in().frame();
            let steps = |a: u32, b: u32| -> i32 {
                let mut x = a;
                for k in 0..20000 {
                    if x == b {
                        return k;
                    }
                    x = x.wrapping_mul(214013).wrapping_add(2531011);
                }
                -1
            };
            eprintln!("SEEDLOG f{f} p{port} prev_to_rec={} prev_to_eng={} eng_to_rec={}", steps(SEEDLOG_LAST, rec), steps(SEEDLOG_LAST, eng), steps(eng, rec));
            SEEDLOG_LAST = rec;
        }
        if RESEED_PRE && fol == 0 && crate::r_in().frame() > -123 {
            if let Some(rec) = crate::r_in().input(port, false).map(|i| i.seed) {
                if rec != 0 {
                    melee_runtime::tu::random::global::state().set_logic_seed(rec);
                }
            }
        }
        if !SEED_PROBE || fol != 0 || SEED_FIRST.is_some() || crate::r_in().frame() <= -123 {
            return;
        }
        let pl = crate::r_in();
        let Some(rec) = pl.input(port, false).map(|i| i.seed) else { return };
        let st = melee_runtime::tu::random::global::state();
        let eng = *melee_runtime::tu::random::HSD_RandCurrentSeed(st);
        if eng != rec && rec != 0 {
            SEED_FIRST = Some((pl.frame(), port, eng, rec));
        }
    }
}

const FIELDS: [&str; 12] =
    ["x", "y", "facing", "percent", "action", "state_frame", "shield", "stocks", "last_attack", "combo", "hitlag", "airborne"];

fn fbits(a: f32, b: f32) -> bool {
    a.to_bits() == b.to_bits()
}

/// Compare one fighter with its Post-Frame row. Fields the file does not carry are skipped.
/// Returns (field index, engine value, replay value) for every differing field.
fn compare(c: &Cap, p: &slp::Post) -> Vec<(usize, String, String)> {
    let mut d = Vec::new();
    let mut f = |k: usize, e: String, r: String| d.push((k, e, r));
    if !fbits(c.x, p.x) {
        f(0, format!("{:?}", c.x), format!("{:?}", p.x));
    }
    if !fbits(c.y, p.y) {
        f(1, format!("{:?}", c.y), format!("{:?}", p.y));
    }
    if !fbits(c.facing, p.facing) {
        f(2, format!("{:?}", c.facing), format!("{:?}", p.facing));
    }
    if !fbits(c.percent, p.percent) {
        f(3, format!("{:?}", c.percent), format!("{:?}", p.percent));
    }
    if c.action != p.action {
        f(4, c.action.to_string(), p.action.to_string());
    }
    if let Some(r) = p.state_frame {
        if !fbits(c.state_frame, r) {
            f(5, format!("{:?}", c.state_frame), format!("{:?}", r));
        }
    }
    if !fbits(c.shield, p.shield) {
        f(6, format!("{:?}", c.shield), format!("{:?}", p.shield));
    }
    if c.stocks != p.stocks {
        f(7, c.stocks.to_string(), p.stocks.to_string());
    }
    if c.last_attack != p.last_attack {
        f(8, c.last_attack.to_string(), p.last_attack.to_string());
    }
    if c.combo != p.combo {
        f(9, c.combo.to_string(), p.combo.to_string());
    }
    if let Some(r) = p.hitlag {
        if !fbits(c.hitlag, r) {
            f(10, format!("{:?}", c.hitlag), format!("{:?}", r));
        }
    }
    if let Some(r) = p.airborne {
        if c.airborne != r {
            f(11, c.airborne.to_string(), r.to_string());
        }
    }
    d
}

#[derive(Clone, Debug)]
struct First {
    frame: i32,
    port: usize,
    field: &'static str,
    engine: String,
    replay: String,
}

pub fn slpcheck(args: &[String]) -> std::process::ExitCode {
    use std::process::ExitCode;
    let get = |k: &str| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned();
    let Some(path) = get("--slp") else {
        eprintln!("usage: melee slpcheck --slp FILE [--frames N] [--iso DISC.iso] [--csv SUMMARY.csv] [--no-resync] [--gecko]");
        return ExitCode::from(2);
    };
    let iso = get("--iso").or_else(|| std::env::var("GW_ISO_VANILLA").ok());
    let Some(iso) = iso else {
        eprintln!("melee slpcheck: no disc (--iso or GW_ISO_VANILLA)");
        return ExitCode::from(2);
    };
    let name = std::path::Path::new(&path).file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
    let data = match std::fs::read(&path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("melee slpcheck: {name}: {e}");
            return ExitCode::from(2);
        }
    };
    let file = match slp::parse(&data, true) {
        Ok(s) => s,
        Err(e) => {
            return finish(args, &name, &Summary { status: format!("parse-error: {e}"), ..Default::default() });
        }
    };
    let mut sm = Summary {
        version: file.version_str(),
        stage: file.info.as_ref().map_or(0, |g| g.stage() as i32),
        roster: roster(&file),
        ucf: ucf_summary(&file),
        frozen_ps: file.frozen_ps,
        pal: file.pal,
        gecko_codes: slp::gecko_codes(&file.gecko).len(),
        gecko_hash: gecko_hash(&file.gecko),
        replay_frames: (file.last_frame - file.first_frame + 1).max(0),
        ..Default::default()
    };
    if args.iter().any(|a| a == "--gecko") {
        for c in slp::gecko_codes(&file.gecko) {
            println!("GECKO {:02X} {:08X} lines={} hash={:016x}", c.kind, c.addr, c.lines, c.hash);
        }
    }
    let conv = match convert(&file) {
        Ok(c) => c,
        Err(e) => {
            sm.status = format!("refused: {e}");
            return finish(args, &name, &sm);
        }
    };
    sm.notes = conv.notes.join("; ");
    let mut pl = melee_data::player::Player::new(melee_data::player::Env { resync: !args.iter().any(|a| a == "--no-resync"), identity: None, ..Default::default() });
    pl.load_slp(&path, &data, false);
    for l in pl.log.drain(..) {
        eprintln!("melee: {l}");
    }
    if !pl.active {
        sm.status = "refused: the replay player refused the file".into();
        return finish(args, &name, &sm);
    }
    pl.frame = -124; // gw_Replay_ApplyMatch arms the counter: the first Replay_Tick makes it -123
    let seed = file.seed;
    // gw_Replay_ApplyMatch returns r.seed as the match's RNG seed: the Game Start block's Random Seed
    pl.r.seed = seed;
    // The engine's seam answers 0 for Replay_ApplyMatch (the RNG was zeroed at the match start); answer the file's seed.
    unsafe {
        SLP_SEED = seed;
        melee_modes::tu::gmvs::ext::hook::Replay_ApplyMatch = Some(|_| unsafe { SLP_SEED });
    }
    // SAFETY: single-threaded start-up, before the engine reads them.
    unsafe {
        std::env::set_var("MELEE_TEST_SEED", seed.to_string());
        if std::env::var_os("MELEE_ENGINE_PRE_DRAWS").is_none() {
            // the console's stage load draws once more for the stages with several BGM tracks (Pokemon Stadium, Battlefield,
            // Final Destination): measured on 125 replays, the engine is exactly one draw short before the first pre-frame there
            let pre = if matches!(sm.stage, 3 | 31 | 32) { "1" } else { "0" };
            std::env::set_var("MELEE_ENGINE_PRE_DRAWS", pre);
        }
    }
    // SLPCHECK_VANILLA_SPAWN=1: no Slippi Neutral Spawn (the port's own .slp exports, played without it)
    if std::env::var_os("SLPCHECK_VANILLA_SPAWN").is_none() {
        crate::replay_modes_active();
    }
    // the replay's own widescreen camera code (if its Gecko list has one) changes the logic's off-screen test
    let ws = slp::widescreen_code(&file.gecko);
    melee_core::codes::set_replay_widescreen(ws.map(|w| (w.left, w.right, w.num, w.den)));
    if let Some(w) = ws {
        sm.notes = format!("{}{}widescreen code {}..{} x{}/{}", sm.notes, if sm.notes.is_empty() { "" } else { "; " }, w.left, w.right, w.num, w.den);
    }
    let booted = crate::engine_disc(&iso).and_then(|disc| melee_engine::Engine::boot_vs(disc, 64 << 20, melee_platform::PadSystem::headless(), conv.scn.clone()));
    let mut e = match booted {
        Ok(e) => e,
        Err(er) => {
            sm.status = format!("boot-error: {}", er.replace(&iso, "<disc>"));
            return finish(args, &name, &sm);
        }
    };
    crate::install_replay(pl);
    install_capture(file.version[0] < 3);
    unsafe { RESEED_PRE = !args.iter().any(|a| a == "--no-pre-reseed"); SEED_PROBE = std::env::var_os("SLPCHECK_SEEDS").is_some(); SEED_FIRST = None };
    let ports: Vec<usize> = (0..4).filter(|&i| conv.scn.players[i].is_some()).collect();
    let posts: Vec<Vec<Option<slp::Post>>> = (0..4).map(|p| if ports.contains(&p) { file.post_by_port(p) } else { Vec::new() }).collect();
    let max_ticks: usize = get("--frames").and_then(|s| s.parse().ok()).unwrap_or(usize::MAX);
    let dump = std::env::var_os("SLPCHECK_DUMP").is_some();
    let mut first: Option<First> = None;
    let mut per_field: [Option<i32>; 12] = [None; 12];
    let mut compared = 0i64;
    let mut last_frame = file.first_frame - 1;
    let mut panic_msg: Option<String> = None;
    // the Dolphin oracle's comparison outputs (notes/dolphin-oracle.md): `--nhv1-out CSV` the NHV1 rows
    // (frame = slippi frame + 123), `--wide-out TXT [--wide]` every named Fighter/Item field per frame
    // (+ game_camera and stage_info with --wide), the same lines `melee dolphin-wide` writes
    let mut nh_csv = get("--nhv1-out").and_then(|p| std::fs::File::create(p).ok());
    if let Some(f) = nh_csv.as_mut() {
        use std::io::Write;
        let _ = f.write_all(b"frame,hash,seed,items,itemsum,s0,s1,s2,s3,s4,s5,s6,s7,s8,s9,s10,s11\n");
    }
    let mut wide_out = get("--wide-out").and_then(|p| std::fs::File::create(p).ok().map(std::io::BufWriter::new));
    let wide_all = args.iter().any(|a| a == "--wide");
    // `--wide-from F --wide-to F`: only those slippi frames (a whole match is ~2 GB of lines)
    let wide_from: i32 = get("--wide-from").and_then(|v| v.parse().ok()).unwrap_or(i32::MIN);
    let wide_to: i32 = get("--wide-to").and_then(|v| v.parse().ok()).unwrap_or(i32::MAX);
    let run = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut ticks = 0usize;
        loop {
            if ticks >= max_ticks {
                break;
            }
            let pl = unsafe { crate::r_in() };
            pl.tick();
            let fr = pl.frame();
            if fr > file.last_frame {
                break;
            }
            let sd = pl.resync_seed();
            if std::env::var_os("SLPCHECK_SEEDLOG").is_some() {
                if fr == -123 { let rec0 = pl.input(ports[0] as i32, false).map(|i| i.seed).unwrap_or(0); let mut x = seed; let mut k = -1i32; for i in 0..100000 { if x == rec0 { k = i; break; } x = x.wrapping_mul(214013).wrapping_add(2531011); } unsafe { SEEDLOG_LAST = seed; } eprintln!("SEEDLOG-START game_seed={seed:08x} first_pre_rec={rec0:08x} steps={k}"); }
            unsafe { eprintln!("SEEDLOG-FS f{fr} sd={sd:08x} cur={:08x} last={SEEDLOG_LAST:08x}", *melee_runtime::tu::random::HSD_RandCurrentSeed(melee_runtime::tu::random::global::state())); }
            }
            if sd != 0 && std::env::var_os("SLPCHECK_FS_RESYNC").is_some() {
                melee_runtime::tu::random::global::state().set_logic_seed(sd);
            }
            unsafe { CAP = [None; 4] };
            if !e.tick() {
                break;
            }
            ticks += 1;
            last_frame = fr;
            if let Some(f) = nh_csv.as_mut() {
                use std::io::Write;
                let r = e.neutral_row();
                let mut l = format!("{},{:08X},{:08X},{},{:08X}", fr + 123, r[0], r[1], r[2], r[3]);
                for w in &r[4..16] {
                    l.push_str(&format!(",{w:08X}"));
                }
                l.push('\n');
                let _ = f.write_all(l.as_bytes());
            }
            if let Some(f) = wide_out.as_mut().filter(|_| fr >= wide_from && fr <= wide_to) {
                use std::io::Write;
                let mut t = String::new();
                crate::dolphin_oracle::engine_frame(&e, fr, wide_all, &mut t);
                let _ = f.write_all(t.as_bytes());
            }
            for &port in &ports {
                let Some(Some(p)) = posts[port].get((fr - file.first_frame) as usize) else { continue };
                let cap = unsafe { CAP[port] };
                let Some(c) = cap else {
                    // no engine fighter reached the trace this frame
                    if first.as_ref().is_none_or(|f| (fr, port) < (f.frame, f.port)) {
                        first = Some(First { frame: fr, port, field: "absent", engine: "none".into(), replay: format!("action {}", p.action) });
                    }
                    continue;
                };
                compared += 1;
                if dump && std::env::var_os("SLPCHECK_ATK").is_some() {
                    eprintln!("ATK f{fr} p{} slp la={} combo={} | eng {:?}", port + 1, p.last_attack, p.combo, c.cand);
                } else if dump {
                    if let Some(i) = unsafe { crate::r_in() }.input(port as i32, false) { eprintln!("PRE f{fr} p{} lx={:?} ly={:?} cx={:?} cy={:?} raw={:?} btn={:x}", port + 1, i.lx, i.ly, i.cx, i.cy, i.raw, i.buttons); }
                    eprintln!("DUMP f{fr} p{} eng x={:?} y={:?} act={} sf={:?} pct={:?} hl={:?} la={} sh={:?} | slp x={:?} y={:?} act={} sf={:?} pct={:?} hl={:?} la={} sh={:?} misc={:?}", port + 1, c.x, c.y, c.action, c.state_frame, c.percent, c.hitlag, c.last_attack, c.shield, p.x, p.y, p.action, p.state_frame, p.percent, p.hitlag, p.last_attack, p.shield, p.misc_as);
                }
                for (k, ev, rv) in compare(&c, p) {
                    if per_field[k].is_none() {
                        per_field[k] = Some(fr);
                    }
                    if first.as_ref().is_none_or(|f| (fr, port) < (f.frame, f.port)) {
                        first = Some(First { frame: fr, port, field: FIELDS[k], engine: ev, replay: rv });
                    }
                }
            }
            // the first divergent frame is the answer; keep going only to fill the per-field table when asked
            if first.is_some() && !dump && !args.iter().any(|a| a == "--full") {
                break;
            }
        }
    }));
    if let Err(p) = run {
        panic_msg = Some(if let Some(s) = p.downcast_ref::<&str>() { (*s).to_string() } else if let Some(s) = p.downcast_ref::<String>() { s.clone() } else { "panic".into() });
    }
    sm.seed_first = unsafe { SEED_FIRST };
    sm.frames_compared = compared;
    sm.last_frame = last_frame;
    if let Some(m) = panic_msg {
        sm.status = format!("panic@{}", last_frame + 1);
        sm.panic = m.replace(&iso, "<disc>");
    } else {
        sm.status = if first.is_some() { "diverged".into() } else { "match".into() };
    }
    if let Some(f) = &first {
        sm.first = Some(f.clone());
    }
    sm.per_field = per_field;
    finish(args, &name, &sm)
}

fn gecko_hash(g: &[u8]) -> String {
    if g.is_empty() {
        return "none".into();
    }
    let mut h = 0xcbf29ce484222325u64;
    for b in g {
        h = (h ^ *b as u64).wrapping_mul(0x100000001b3);
    }
    format!("{:08x}", h as u32)
}

#[derive(Default)]
struct Summary {
    status: String,
    version: String,
    stage: i32,
    roster: String,
    ucf: String,
    frozen_ps: bool,
    pal: bool,
    gecko_codes: usize,
    gecko_hash: String,
    replay_frames: i32,
    frames_compared: i64,
    last_frame: i32,
    first: Option<First>,
    per_field: [Option<i32>; 12],
    panic: String,
    notes: String,
    seed_first: Option<(i32, i32, u32, u32)>,
}

fn finish(args: &[String], name: &str, s: &Summary) -> std::process::ExitCode {
    let csvq = |t: &str| t.replace(['\n', '\r', ','], " ");
    let (ff, fp, fl, fe, fr) = match &s.first {
        Some(f) => (f.frame.to_string(), (f.port + 1).to_string(), f.field.to_string(), f.engine.clone(), f.replay.clone()),
        None => (String::new(), String::new(), String::new(), String::new(), String::new()),
    };
    let pf: Vec<String> = s.per_field.iter().map(|v| v.map_or(String::new(), |x| x.to_string())).collect();
    let row = format!(
        "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
        name, csvq(&s.status), s.version, s.stage, s.roster, s.ucf, s.frozen_ps as i32, s.pal as i32, s.gecko_codes, s.gecko_hash,
        s.replay_frames, s.frames_compared, s.last_frame, ff, fp, fl, csvq(&fe), csvq(&fr), pf.join(","), csvq(&s.panic), csvq(&s.notes), s.seed_first.map_or(String::new(), |(f, p, e, r)| format!("{f}:p{}:{e:08x}:{r:08x}", p + 1))
    );
    let header = format!(
        "file,status,slippi,stage,roster,ucf_dash_shield,frozen_ps,pal,gecko_codes,gecko_hash,replay_frames,frames_compared,last_frame,first_frame,first_port,first_field,engine_value,replay_value,{},panic,notes,seed_first",
        FIELDS.iter().map(|f| format!("fd_{f}")).collect::<Vec<_>>().join(",")
    );
    if let Some(path) = args.iter().position(|a| a == "--csv").and_then(|i| args.get(i + 1)) {
        use std::io::Write;
        let fresh = std::fs::metadata(path).map_or(true, |m| m.len() == 0);
        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
            if fresh {
                let _ = writeln!(f, "{header}");
            }
            let _ = writeln!(f, "{row}");
        }
    }
    match &s.first {
        Some(f) => println!("SLPCHECK {name} {} first divergence: frame {} port {} field {} engine={} replay={} (compared {} fighter-frames)", s.status, f.frame, f.port + 1, f.field, f.engine, f.replay, s.frames_compared),
        None => println!("SLPCHECK {name} {} (compared {} fighter-frames, last frame {}) {}", s.status, s.frames_compared, s.last_frame, s.panic),
    }
    if s.status == "match" || s.status == "diverged" {
        std::process::ExitCode::SUCCESS
    } else {
        std::process::ExitCode::from(1)
    }
}
