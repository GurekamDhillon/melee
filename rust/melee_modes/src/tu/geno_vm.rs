//! Geno bosses and enemies run by the enemy-command VM (`brawl_boss::vm`): the scripts come from the player's own
//! Brawl files at load time (the boss's stage archive or `adventure_common.arc`, plus the shared script file), the IR
//! only supplies the parameter map. The actor lives at the end of `geno_boss::State` (`S.vm`), so the memory
//! snapshots cover it and `geno_boss::state_hash` folds it into NHV1 while it is live.
//!
//! Loading reads and compiles on a helper thread (allocations stay off the game heap and out of the frame); the
//! compiled program is kept per name for the process. Never prints a path.

use brawl_boss::file::BossFile;
use brawl_boss::vm::{self, Input, Program};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

static PROGRAMS: OnceLock<Mutex<HashMap<String, &'static Program>>> = OnceLock::new();

fn local_root() -> Option<PathBuf> {
    crate::tu::petey_model::local_root()
}

fn ir_text(root: &Path, stem: &str) -> Option<String> {
    let mut dirs = vec![];
    if let Some(p) = std::env::var_os("MELEE_BOSS_IR_DIR") {
        dirs.push(PathBuf::from(p));
    }
    dirs.push(root.join("boss-ir"));
    if let Some(p) = root.parent() {
        dirs.push(p.join("boss-ir"));
    }
    dirs.iter().find_map(|d| std::fs::read_to_string(d.join(format!("{stem}.brawl.ir.json"))).ok())
}

fn local_text(root: &Path, file: &str) -> Option<String> {
    let mut dirs = vec![];
    if let Some(p) = std::env::var_os("MELEE_BOSS_IR_DIR") {
        dirs.push(PathBuf::from(p));
    }
    dirs.push(root.join("boss-ir"));
    if let Some(p) = root.parent() {
        dirs.push(p.join("boss-ir"));
    }
    dirs.iter().find_map(|d| std::fs::read_to_string(d.join(file)).ok())
}

fn acc_text(root: &Path) -> Option<String> {
    let mut dirs = vec![];
    if let Some(p) = std::env::var_os("MELEE_BOSS_IR_DIR") {
        dirs.push(PathBuf::from(p));
    }
    dirs.push(root.join("boss-ir"));
    if let Some(p) = root.parent() {
        dirs.push(p.join("boss-ir"));
    }
    dirs.iter().find_map(|d| std::fs::read_to_string(d.join("enemies").join("raw").join("dec").join("accmap.json")).ok())
}

fn read(name: &str) -> Result<Program, String> {
    if crate::tu::petey_model::brawl_env().absent {
        return Err("disabled".into());
    }
    let &(_, stem, arc, h) = vm::FILES.iter().find(|f| f.0 == name).ok_or("not a VM boss / enemy")?;
    let root = local_root().ok_or("no local Brawl folder")?;
    let adv = root.join("disc").join("files").join("stage").join("adventure");
    let leaf_of = |arc: &str, h: u64| -> Result<Vec<u8>, String> {
        let raw = std::fs::read(adv.join(arc)).map_err(|e| format!("archive not readable ({})", e.kind()))?;
        let mut ls = Vec::new();
        vm::leaves(&raw, &mut ls, 0);
        ls.into_iter()
            .find(|l| if h == 0 { brawl_boss::archive::looks_like_boss_file(l, None) } else { vm::fnv(l) == h })
            .ok_or_else(|| "file not found in its archive".to_string())
    };
    let common = BossFile::parse_raw(&leaf_of("adventure_common.arc", vm::COMMON_FNV)?).map_err(|e| format!("{e:?}"))?;
    let main = BossFile::parse_raw(&leaf_of(arc, h)?).map_err(|e| format!("{e:?}"))?;
    let pm = ir_text(&root, stem).map(|t| vm::param_map(&t)).unwrap_or_default();
    // the accessor map fills the ids the IR leaves out (local, `<ir dir>/enemies/raw/dec/accmap.json`)
    let acc = acc_text(&root).map(|j| vm::acc_map(&j, vm::class_of(name))).unwrap_or_default();
    let pm = vm::merged_map(&pm, &acc);
    let mut p = Program::build(&main, Some(&common), &pm).map_err(|e| e.to_string())?;
    p.inits = ir_text(&root, stem).map(|t| vm::var_inits(&t)).unwrap_or_default();
    match local_text(&root, "vm-values.txt") {
        Some(t) => p.block_values = vm::block_values(&t),
        None => note(format!("geno_vm: WARNING: {name}: the local value table is missing; engine values 23000+ read 0")),
    }
    let left = p.unknown();
    if !left.is_empty() {
        note(format!("geno_vm: {name}: {} command kinds not run by the VM: {}", left.len(), left.join(", ")));
    }
    Ok(p)
}

/// The compiled program for a boss / enemy (read once, on a helper thread; None when the files are absent).
/// Messages from the loader, printed by the next `program` call on the game thread (no stdio from helper threads).
static NOTES: Mutex<Vec<String>> = Mutex::new(Vec::new());
fn note(m: String) {
    if let Ok(mut v) = NOTES.lock() {
        v.push(m);
    }
}

pub fn program(name: &str) -> Option<&'static Program> {
    let p = load(name);
    if let Ok(mut v) = NOTES.lock() {
        for m in v.drain(..) {
            eprintln!("{m}");
        }
    }
    p
}

/// `program` without printing, for a helper thread that compiles ahead of the match (its notes wait for `program`).
pub fn prefetch(name: &str) {
    let _ = load(name);
}

fn load(name: &str) -> Option<&'static Program> {
    static LOCK: Mutex<()> = Mutex::new(());
    let _g = LOCK.lock().ok()?;
    let m = PROGRAMS.get_or_init(|| Mutex::new(HashMap::new()));
    if let Some(p) = m.lock().ok()?.get(name) {
        return Some(*p);
    }
    let n = name.to_string();
    let r = std::thread::spawn(move || read(&n)).join().unwrap_or_else(|_| Err("decoder panicked".into()));
    match r {
        Ok(p) => {
            note(format!("geno_vm: {name}: scripts compiled from the player's Brawl files ({} instructions)", p.code.len()));
            let p: &'static Program = Box::leak(Box::new(p));
            m.lock().ok()?.insert(name.to_string(), p);
            Some(p)
        }
        Err(e) => {
            note(format!("geno_vm: WARNING: {name} not loaded ({e})"));
            None
        }
    }
}

/// Set the motion lengths (frames) from the host's animation data; the VM's motion-end requirement reads them.
pub fn set_motion_lengths(name: &str, lens: &[(String, u16)]) {
    let Some(p) = program(name) else { return };
    let mut q = p.clone();
    for (n, l) in lens {
        if let Some(i) = q.motion_names.iter().position(|m| m == n) {
            q.motion_len[i] = (*l).max(1);
        }
    }
    if let Some(m) = PROGRAMS.get() {
        if let Ok(mut g) = m.lock() {
            g.insert(name.to_string(), Box::leak(Box::new(q)));
        }
    }
}

/// Start the VM actor (the core's own define-driven state stays off).
pub fn begin(p: &Program, strength: i32, difficulty: i32, seed: u32, start_status: i32, x: f32) {
    unsafe {
        let s = &mut *core::ptr::addr_of_mut!(crate::tu::geno_boss::S);
        vm::begin(p, &mut s.vm, strength, difficulty.clamp(0, 14), seed, start_status, true);
        s.vm.pos[0] = x;
    }
}

/// One frame.
pub fn tick(p: &Program, i: &Input) {
    unsafe { vm::step(p, &mut *core::ptr::addr_of_mut!(crate::tu::geno_boss::S.vm), i) }
}

pub fn actor() -> &'static vm::Actor {
    unsafe { &*core::ptr::addr_of!(crate::tu::geno_boss::S.vm) }
}

pub fn stop() {
    unsafe { crate::tu::geno_boss::S.vm = vm::OFF };
}

#[cfg(test)]
mod tests {
    #[test]
    fn rayquaza_vm_in_state() {
        let Some(p) = super::program("rayquaza") else {
            eprintln!("geno_vm: no local files, skipped");
            return;
        };
        // status 2: Rayquaza's Hub (status 0 is DyingNoEffect)
        super::begin(p, 1, 5, 7, 2, 0.0);
        let h0 = crate::tu::geno_boss::state_hash().expect("vm live");
        let mut i = brawl_boss::vm::Input { ground: true, ..Default::default() };
        i.targets[0] = Some([40.0, 0.0]);
        for _ in 0..600 {
            super::tick(p, &i);
        }
        assert_ne!(h0, crate::tu::geno_boss::state_hash().unwrap());
        super::stop();
        assert!(crate::tu::geno_boss::state_hash().is_none());
    }
}
