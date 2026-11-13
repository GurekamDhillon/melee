//! The look of the Subspace enemies (`subspace.rs`), loaded at runtime from the owner's own extracted Brawl files.
//!
//! Presentation only, exactly as `petey_model.rs` (whose readers it uses: `load_rigs`, `Rig`, `draw_tris`): nothing
//! here feeds the sim. Enemy behaviour, hurt shapes and hitboxes are hand-written parameters in `subspace.rs`, so a
//! run is the same with or without the files, and the same whether a model has finished loading or not.
//!
//! Loading never blocks the game thread (the zero-hitch rule): `request(kind)` starts one helper thread per archive,
//! `get(kind)` answers `None` until that thread has stored its result, and the draw shows plain capsules meanwhile.
//! The helper thread also makes every buffer a draw needs, so drawing allocates nothing inside the game scope.
#![allow(clippy::missing_safety_doc)]

use crate::tu::petey_model as pm;
use std::sync::atomic::{AtomicBool, Ordering::Relaxed};
use std::sync::{Mutex, OnceLock};

/// Where one enemy kind's rig is: archive path parts under `disc/files`, the MDL0 name, the CHR0 name prefix, the
/// animation whose first frame gives the extent, and extra models (thrown weapons) from the same archive.
pub struct Source {
    pub rel: &'static [&'static str],
    pub model: &'static str,
    pub prefix: &'static str,
    pub idle: &'static str,
    pub extra: &'static [&'static str],
}

pub struct Model {
    pub rig: pm::Rig,
    pub extra: Vec<Option<pm::Rig>>,
    /// One skinning scratch per enemy slot (and per extra model), made on the loader thread.
    pub bufs: Mutex<Vec<pm::DrawBufs>>,
    pub extra_bufs: Mutex<Vec<pm::DrawBufs>>,
}

const KINDS: usize = crate::tu::subspace::KINDS;

/// The loader's state. It must survive every rollback load: a snapshot rewrites the executable's statics and the
/// game arena, so it lives in one block leaked on the system heap (made on a helper thread), found through a
/// thread-local of the game thread (live TLS is outside the snapshot). A loader thread is only ever started from a
/// short-lived helper thread, so its handle, name and result packet are system-heap allocations too (spawning it
/// straight from the game thread put them in the arena, and a load then freed them twice).
struct State {
    slots: [OnceLock<Option<Model>>; KINDS],
    asked: [AtomicBool; KINDS],
    told: [AtomicBool; KINDS],
    msg: [Mutex<String>; KINDS],
}

std::thread_local! {
    static ST: std::cell::Cell<Option<&'static State>> = const { std::cell::Cell::new(None) };
}

fn state() -> &'static State {
    ST.with(|c| {
        if let Some(s) = c.get() {
            return s;
        }
        let s: &'static State = std::thread::spawn(|| {
            &*Box::leak(Box::new(State {
                slots: [const { OnceLock::new() }; KINDS],
                asked: [const { AtomicBool::new(false) }; KINDS],
                told: [const { AtomicBool::new(false) }; KINDS],
                msg: [const { Mutex::new(String::new()) }; KINDS],
            }))
        })
        .join()
        .expect("subspace model state");
        c.set(Some(s));
        s
    })
}

fn off() -> bool {
    static V: OnceLock<bool> = OnceLock::new();
    *V.get_or_init(|| std::env::var_os("MELEE_SUBSPACE_NO_MODEL").is_some())
}

/// Start loading `kind`'s rig if nobody has yet. Returns at once (the helper only starts the loader).
pub fn request(kind: usize) {
    if kind >= KINDS || off() {
        return;
    }
    let st = state();
    if st.asked[kind].swap(true, Relaxed) {
        return;
    }
    let src = &crate::tu::subspace::DEFS[kind].src;
    let _ = std::thread::spawn(move || {
        let _ = std::thread::Builder::new().name("subspace-model".into()).spawn(move || {
            let mut wants = vec![(src.model, src.prefix, src.idle)];
            for e in src.extra {
                wants.push((e, e, ""));
            }
            let r = pm::load_rigs(src.rel, &wants).and_then(|mut v| {
                let rig = v.remove(0).ok_or("model not found")?;
                let n = crate::tu::subspace::MAX;
                let bufs = (0..n).map(|_| rig.draw_bufs()).collect();
                let mut eb = vec![];
                for x in v.iter().flatten() {
                    for _ in 0..n {
                        eb.push(x.draw_bufs());
                    }
                }
                Ok(Model { rig, extra: v, bufs: Mutex::new(bufs), extra_bufs: Mutex::new(eb) })
            });
            // told by the game thread (`get`): printing from here raced the snapshot's rewrite of std's statics
            *st.msg[kind].lock().unwrap_or_else(|e| e.into_inner()) = match &r {
                Ok(m) => format!("subspace: {} model loaded ({} animations){}", src.model, m.rig.anims.len(), m.rig.motion_log),
                Err(e) => format!("subspace: no {} model ({e}); placeholder look", src.model),
            };
            let _ = st.slots[kind].set(r.ok());
        });
    })
    .join();
}

fn tell(st: &State, kind: usize) {
    if !st.told[kind].swap(true, Relaxed) {
        eprintln!("{}", st.msg[kind].lock().unwrap_or_else(|e| e.into_inner()));
    }
}

/// `kind`'s model once its loader has finished (never waits).
pub fn get(kind: usize) -> Option<&'static Model> {
    if kind >= KINDS {
        return None;
    }
    let st = state();
    let m = st.slots[kind].get()?;
    tell(st, kind);
    m.as_ref()
}

/// Whether `kind`'s load has finished, with or without a model (the checks wait for it before a screenshot).
pub fn settled(kind: usize) -> bool {
    kind >= KINDS || off() || state().slots[kind].get().is_some()
}
