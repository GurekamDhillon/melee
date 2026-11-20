//! Union-drift checker (feature `mvshadow`, off in every normal build; tools/rust-port/mvshadow/instrument.py).
//!
//! `Fighter.mv` overlays every motion state's variables. A view holding a pointer is 8 bytes natively and 4 on the
//! GameCube, so later leaves move: a read can see a different write natively than the GameCube would
//! (tools/rust-port/mv_layout_audit.py lists the pairs). This keeps, per Fighter's `mv`, two shadows stamped with a
//! write sequence number: by native byte and by GameCube byte (each leaf's GC offset from the layout table). A read
//! whose bytes have different last writers in the two shadows reads a value the GameCube would not: it is logged
//! once per (caller, path) on stderr as `MVSHADOW <file:line> <path> ...`. The game itself is unchanged.
//!
//! The instrumented build wraps every `X.mv.<path>` the transform resolves: reads as `*r(addr_of!(..), ID)`, writes as
//! `*w(addr_of_mut!(..), ID) = ..` (Rust evaluates the assigned value first, so its reads see the shadow before the
//! write), compound assigns as `rw`. `ID` indexes `PATHS` (the generated table: the path's native offset in `mv` and
//! its leaves as (native offset, GC offset, native size, GC size): a pointer is 8 and 4, stamped on both sides, never
//! checked). `reset` stamps a whole `mv` (Fighter_Create's zeroing).

#[cfg(feature = "mvshadow")]
mod imp {
    use std::collections::{HashMap, HashSet};
    use std::sync::Mutex;

    pub use super::table::{MV_GC_SIZE, MV_NATIVE_SIZE, PATHS};

    struct Shadow {
        nat: Vec<u32>,
        gc: Vec<u32>,
        // the path id of each byte's last writer, to name it in the log
        nat_w: Vec<u32>,
        gc_w: Vec<u32>,
    }
    struct State {
        seq: u32,
        mv: HashMap<usize, Shadow>,
        seen: HashSet<(&'static str, u32, u32)>,
    }
    static ST: Mutex<Option<State>> = Mutex::new(None);
    /// The engine frame (golden row) being simulated: bumped after each frame's NHV1 row, so a hit logged with frame
    /// F happened while the row with index F was computed (compare_golden's frame numbers).
    static FRAME: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    pub fn tick() {
        FRAME.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    fn with<R>(f: impl FnOnce(&mut State) -> R) -> R {
        let mut g = ST.lock().unwrap_or_else(|e| e.into_inner());
        f(g.get_or_insert_with(|| State { seq: 0, mv: HashMap::new(), seen: HashSet::new() }))
    }
    fn shadow(st: &mut State, base: usize) -> &mut Shadow {
        st.mv.entry(base).or_insert_with(|| Shadow {
            nat: vec![0; MV_NATIVE_SIZE],
            gc: vec![0; MV_GC_SIZE],
            nat_w: vec![u32::MAX; MV_NATIVE_SIZE],
            gc_w: vec![u32::MAX; MV_GC_SIZE],
        })
    }
    fn base(p: usize, id: u32) -> usize {
        p - PATHS[id as usize].1 as usize
    }
    fn stamp(base: usize, id: u32) {
        with(|st| {
            st.seq = st.seq.wrapping_add(1);
            let s = st.seq;
            let sh = shadow(st, base);
            for &(n, g, nz, gz) in PATHS[id as usize].2 {
                sh.nat[n as usize..(n + nz) as usize].fill(s);
                sh.gc[g as usize..(g + gz) as usize].fill(s);
                sh.nat_w[n as usize..(n + nz) as usize].fill(id);
                sh.gc_w[g as usize..(g + gz) as usize].fill(id);
            }
        })
    }
    fn who(id: u32) -> &'static str {
        if id == u32::MAX { "(reset)" } else { PATHS[id as usize].0 }
    }
    fn check(base: usize, id: u32, loc: &'static core::panic::Location<'static>) {
        with(|st| {
            let sh = shadow(st, base);
            let mut bad = None;
            for &(n, g, z, gz) in PATHS[id as usize].2 {
                if z != gz {
                    continue; // a pointer: no GameCube value to compare
                }
                for k in 0..z as usize {
                    // the native bytes of a little-endian scalar hold the GameCube bytes in reverse order
                    let (ni, gi) = (n as usize + k, g as usize + z as usize - 1 - k);
                    if sh.nat[ni] != sh.gc[gi] {
                        bad = Some((n, g, sh.nat[ni], sh.gc[gi], sh.nat_w[ni], sh.gc_w[gi]));
                    }
                }
            }
            if let Some((n, g, a, b, aw, bw)) = bad {
                if st.seen.insert((loc.file(), loc.line(), id)) {
                    eprintln!("MVSHADOW {}:{} {} native+{} gc+{} last write native #{} {} gc #{} {} frame {}", loc.file(),
                              loc.line(), PATHS[id as usize].0, n, g, a, who(aw), b, who(bw),
                              FRAME.load(std::sync::atomic::Ordering::Relaxed));
                }
            }
        })
    }

    #[track_caller]
    pub fn r<T>(p: *const T, id: u32) -> &'static T {
        check(base(p as usize, id), id, core::panic::Location::caller());
        unsafe { &*p }
    }
    #[track_caller]
    pub fn rp<T>(p: *const T, id: u32) -> *const T {
        check(base(p as usize, id), id, core::panic::Location::caller());
        p
    }
    pub fn w<T>(p: *mut T, id: u32) -> &'static mut T {
        stamp(base(p as usize, id), id);
        unsafe { &mut *p }
    }
    pub fn wp<T>(p: *mut T, id: u32) -> *mut T {
        stamp(base(p as usize, id), id);
        p
    }
    #[track_caller]
    pub fn rw<T>(p: *mut T, id: u32) -> &'static mut T {
        check(base(p as usize, id), id, core::panic::Location::caller());
        w(p, id)
    }
    /// The whole `mv` at `base` written at once (zeroed or copied): both shadows agree from here.
    pub fn reset(base: usize) {
        with(|st| {
            st.seq = st.seq.wrapping_add(1);
            let s = st.seq;
            let sh = shadow(st, base);
            sh.nat.fill(s);
            sh.gc.fill(s);
            sh.nat_w.fill(u32::MAX);
            sh.gc_w.fill(u32::MAX);
        })
    }
}

#[cfg(feature = "mvshadow")]
#[path = "mvshadow_table.rs"]
mod table;

#[cfg(feature = "mvshadow")]
pub use imp::*;
