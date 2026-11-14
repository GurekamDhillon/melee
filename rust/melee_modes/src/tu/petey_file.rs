//! Petey's numbers from his boss parameter file on the player's own Brawl disc (`brawl_boss`, clean-room decoder;
//! notes/brawl-boss-file.md). Loaded and decoded once, at boss setup, on a helper thread (its allocations stay on the
//! system heap, outside the game arena), then kept as plain values. Missing or undecodable files fall back to the
//! port's measured constants with one logged warning. Never prints a path.

use std::path::{Path, PathBuf};

/// The values `petey.rs` reads. Floats are the file's own f32 words (bit-exact), so a run is identical on every
/// machine with the same disc, and identical to the fallback where the two agree.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tuning {
    pub from_file: bool,
    pub cage_hp: i32,
    pub death_pct: f32,
    pub phase1_pct: f32,
    pub phase2_pct: f32,
    /// strengths for phases 0, 1, 2 (placement 1, then 24001 / 24002)
    pub strength: [usize; 3],
    /// 44003 / 44004: lo, lo per L, hi, hi per L
    pub wait_a: [f32; 4],
    pub wait_b: [f32; 4],
    /// 44005: T = a + b L
    pub hover: [f32; 2],
    /// 44006 / 44007 first floats (side 1 / side 0)
    pub goal: [f32; 2],
    /// tuning-block target-slot weights P1-P4
    pub slot_w: [i32; 4],
    /// the repeat cuts, per mille: the slot draw (tuning +0x74) and the weighted lists (param 4000)
    pub slot_cut_pm: i32,
    pub repeat_pm: i32,
    /// weighted lists: (base, slope per L)
    pub side: [(i32, f32); 2],
    pub menu: [(i32, f32); 3],
    pub after_angry: [(i32, f32); 2],
    pub wait_pick: [(i32, f32); 2],
    /// kind 16: start, accel, accel per L, max, max per L
    pub dive: [f32; 5],
    /// kinds 14 / 15: decay, decay per L on landing (the stored values are negative)
    pub decay: f32,
    pub land_decay_l: f32,
}

/// The port's measured constants (petey5 / petey6), used when the file is absent.
pub const FALLBACK: Tuning = Tuning {
    from_file: false,
    cage_hp: 999,
    death_pct: 10.0,
    phase1_pct: 50.0,
    phase2_pct: 25.0,
    strength: [1, 2, 3],
    wait_a: [100.0, -9.0, 150.0, -12.0],
    wait_b: [2.0, -0.18, 20.0, -1.75],
    hover: [60.0, -5.4],
    goal: [-68.0, 68.0],
    slot_w: [25, 10, 20, 8],
    slot_cut_pm: 300,
    repeat_pm: 300,
    side: [(50, 0.0), (50, 0.0)],
    menu: [(20, 0.0), (20, 0.0), (60, 2.0)],
    after_angry: [(20, 0.0), (80, 0.0)],
    wait_pick: [(75, 0.0), (25, 0.0)],
    dive: [0.2, 0.015, 0.003, 1.4, 0.03],
    decay: 0.06,
    land_decay_l: 0.005,
};

static T: std::sync::OnceLock<Tuning> = std::sync::OnceLock::new();

/// The tuning (loads on first use; call `load()` at boss setup so no game frame pays for it).
pub fn get() -> &'static Tuning {
    T.get_or_init(|| {
        let r = std::thread::spawn(read).join().unwrap_or_else(|_| Err("decoder panicked".into()));
        match r {
            Ok(t) => {
                eprintln!("petey: boss parameters read from the player's Brawl files");
                t
            }
            Err(e) => {
                eprintln!("petey: WARNING: boss parameters not read ({e}); using the built-in values");
                FALLBACK
            }
        }
    })
}

/// Read the boss parameters in the background ahead of the first fight (once; the Envoy plan's publication calls it
/// from mod load on). The first `get()` used to read and decode the Brawl files inside the first Envoy fight's first
/// tick (~5.7 ms, envoy_sim `fight_setup` -> `gd_envoy_boss_dmg`). The result is a function of the files alone, so
/// when it is read never reaches the simulation.
pub fn prefetch() {
    static STARTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    if T.get().is_none() && !STARTED.swap(true, std::sync::atomic::Ordering::Relaxed) {
        let _ = std::thread::Builder::new().name("petey prefetch".into()).spawn(|| {
            let _ = get();
        });
    }
}

/// Boss setup: decode now (on a helper thread).
pub fn load() {
    let _ = get();
}

fn local_root() -> Option<PathBuf> {
    if let Some(p) = std::env::var_os("MELEE_BRAWL_LOCAL") {
        return Some(PathBuf::from(p));
    }
    let mut starts = vec![];
    if let Ok(c) = std::env::current_dir() {
        starts.push(c);
    }
    if let Ok(e) = std::env::current_exe() {
        if let Some(d) = e.parent() {
            starts.push(d.to_path_buf());
        }
    }
    for s in starts {
        let mut d: Option<&Path> = Some(&s);
        while let Some(p) = d {
            let c = p.join("_build").join("brawl");
            if c.join("disc").join("files").is_dir() {
                return Some(c);
            }
            d = p.parent();
        }
    }
    None
}

fn read() -> Result<Tuning, String> {
    if std::env::var_os("MELEE_PETEY_NO_FILE").is_some() || std::env::var_os("MELEE_BRAWL_ABSENT").is_some() {
        return Err("disabled".into());
    }
    let root = local_root().ok_or("no local Brawl folder")?;
    let f = root.join("disc").join("files").join("stage").join("adventure").join("030101.pac");
    let raw = std::fs::read(&f).map_err(|e| format!("boss archive not readable ({})", e.kind()))?;
    let bytes = brawl_boss::archive::find_petey_file(&raw).map_err(|e| format!("not found: {e}"))?;
    let bf = brawl_boss::BossFile::parse(&bytes).map_err(|e| format!("decode: {e}"))?;
    from_file(&bf).ok_or_else(|| "a value is missing".into())
}

fn bits(w: u32) -> f32 {
    f32::from_bits(w)
}
fn fx_f32(x: brawl_boss::Fx) -> f32 {
    x.raw() as f32 / 60000.0
}
fn per_mille(x: brawl_boss::Fx) -> i32 {
    (x.raw() / 60) as i32
}

/// The tuning from a decoded file (None if a value is missing).
pub fn from_file(bf: &brawl_boss::BossFile) -> Option<Tuning> {
    let p = &bf.params;
    let f4 = |id| -> Option<[f32; 4]> {
        let r = p.raw(id)?;
        Some([bits(*r.first()?), bits(*r.get(1)?), bits(*r.get(2)?), bits(*r.get(3)?)])
    };
    let list = |id, n| -> Option<Vec<(i32, f32)>> {
        Some(p.weighted(id, n).ok()?.iter().map(|e| (e.base, fx_f32(e.slope))).collect())
    };
    let k16 = p.kinetic_raw(16)?;
    let k14 = p.kinetic_raw(14)?;
    let k15 = p.kinetic_raw(15)?;
    let proc_ = p.cage_proc(0)?;
    let hover = p.raw(44005)?;
    let side = list(44008, 2)?;
    let menu = list(44009, 3)?;
    let aa = list(44010, 2)?;
    let wp = list(44011, 2)?;
    let sw = bf.slot_weights();
    Some(Tuning {
        from_file: true,
        cage_hp: p.cage_hp(0)?,
        death_pct: bits(bf.general[0x9C / 4]),
        phase1_pct: proc_.phase1_pct as f32,
        phase2_pct: proc_.phase2_pct as f32,
        strength: [1, p.int(24001)?.clamp(1, 5) as usize, p.int(24002)?.clamp(1, 5) as usize],
        wait_a: f4(44003)?,
        wait_b: f4(44004)?,
        hover: [bits(*hover.first()?), bits(*hover.get(1)?)],
        goal: [bits(*p.raw(44006)?.first()?), bits(*p.raw(44007)?.first()?)],
        slot_w: [sw[0].trunc() as i32, sw[1].trunc() as i32, sw[2].trunc() as i32, sw[3].trunc() as i32],
        slot_cut_pm: per_mille(bf.slot_repeat_factor()),
        repeat_pm: per_mille(p.scalar(4000)?),
        side: [side[0], side[1]],
        menu: [menu[0], menu[1], menu[2]],
        after_angry: [aa[0], aa[1]],
        wait_pick: [wp[0], wp[1]],
        dive: [bits(k16[1]), bits(k16[3]), bits(k16[4]), bits(k16[7]), bits(k16[8])],
        decay: -bits(k14[3]),
        land_decay_l: -bits(k15[4]),
    })
}

