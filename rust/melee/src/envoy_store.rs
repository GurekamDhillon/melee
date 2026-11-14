//! The Envoy run record on disk (envoy-redesign.md 1.3): the host keeps the last record the sim mod published
//! (`melee_modes::tu::envoy::VIEW[REC_AT..]`) in `<exe dir>/melee_data/envoy/run.bin` (or `MELEE_ENVOY_DIR`), so the
//! camp's Continue works across sessions. Offline only; menu-time state (the sim reads it once, at arm time, through
//! `gd_envoy_record`). A file whose digest does not match is refused with a notice.

use melee_modes::tu::envoy as ev;
use std::path::PathBuf;

fn path() -> PathBuf {
    let dir = match std::env::var_os("MELEE_ENVOY_DIR").filter(|d| !d.is_empty()) {
        Some(d) => PathBuf::from(d),
        None => std::env::current_exe().ok().and_then(|e| e.parent().map(|p| p.join("melee_data").join("envoy"))).unwrap_or_else(|| PathBuf::from("melee_data/envoy")),
    };
    dir.join("run.bin")
}

/// Load the saved run once (at the first offline tick).
fn load() {
    let p = path();
    let Ok(b) = std::fs::read(&p) else { return };
    if b.len() != ev::REC_WORDS * 4 {
        eprintln!("envoy: the saved run is the wrong size; ignored");
        return;
    }
    let mut r = [0i32; ev::REC_WORDS];
    for (i, w) in r.iter_mut().enumerate() {
        *w = i32::from_le_bytes(b[i * 4..i * 4 + 4].try_into().unwrap());
    }
    if !ev::rec_valid(&r) {
        eprintln!("envoy: the saved run fails its check (stale or edited); ignored");
        return;
    }
    unsafe {
        ev::SAVED = r;
        ev::SAVED_LEN = ev::REC_WORDS as i32;
    }
    eprintln!("envoy: saved run loaded (loop {}, fight {}, pool {})", r[46] & 0xFF, r[4] + 1, r[5]);
}

fn write(r: &[i32; ev::REC_WORDS]) {
    let p = path();
    if let Some(d) = p.parent() {
        let _ = std::fs::create_dir_all(d);
    }
    let b: Vec<u8> = r.iter().flat_map(|w| w.to_le_bytes()).collect();
    let tmp = p.with_extension("tmp");
    if std::fs::write(&tmp, &b).and_then(|_| std::fs::rename(&tmp, &p)).is_err() {
        eprintln!("envoy: could not save the run record");
    }
}

/// Endless loops: the best loop reached on this save (`best.bin` beside the run, one LE word).
fn load_best() {
    if let Ok(b) = std::fs::read(path().with_file_name("best.bin")) {
        if b.len() == 4 {
            unsafe { ev::BEST_LOOP = i32::from_le_bytes(b[..4].try_into().unwrap()).clamp(0, 999) };
        }
    }
}

fn write_best(n: i32) {
    let p = path().with_file_name("best.bin");
    if let Some(d) = p.parent() {
        let _ = std::fs::create_dir_all(d);
    }
    if std::fs::write(&p, n.to_le_bytes()).is_err() {
        eprintln!("envoy: could not save the best loop");
    }
}

fn drop_file() {
    let _ = std::fs::remove_file(path());
}

/// Once per offline tick: load on the first call, apply the camp's "forget", copy a changed live record to disk, and
/// forget the record when the run has ended.
pub fn sync() {
    static LOADED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    if !LOADED.swap(true, std::sync::atomic::Ordering::Relaxed) {
        load();
        load_best();
    }
    unsafe {
        if ev::SAVED_DROP != 0 {
            ev::SAVED_DROP = 0;
            drop_file();
        }
        // an Item Lab session never touches the saved run, the record or the best loop
        if ev::lab()[0] != 0 {
            return;
        }
        let v = ev::VIEW;
        // the V3 view (envoy_data view.rs: V_MAGIC "ENV3", V_STATE 1, V_ARMED 20): v[20] says the sim has taken the arm
        // (its record is this run's, not the one before a New run)
        if ev::ARMED == 0 || v[0] != 0x454E_5633 || v[20] != 1 {
            return;
        }
        // V_LOOP (31): the loop the run is in; a live or ended run past the best raises it
        if matches!(v[1], 1 | 3) && v[31] > ev::BEST_LOOP && v[31] <= 999 {
            ev::BEST_LOOP = v[31];
            write_best(v[31]);
        }
        match v[1] {
            // RUN_LIVE: the record words as published
            1 => {
                let mut r = [0i32; ev::REC_WORDS];
                r.copy_from_slice(&v[ev::REC_AT..ev::REC_AT + ev::REC_WORDS]);
                if ev::rec_valid(&r) && (ev::SAVED_LEN == 0 || ev::SAVED != r) {
                    ev::SAVED = r;
                    ev::SAVED_LEN = ev::REC_WORDS as i32;
                    write(&r);
                }
            }
            // RUN_CLEAR / RUN_OVER: nothing to continue
            2 | 3 => {
                if ev::SAVED_LEN != 0 {
                    ev::SAVED_LEN = 0;
                    drop_file();
                }
            }
            _ => {}
        }
    }
}
