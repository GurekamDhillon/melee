//! Call tracer (consolidation step 1). With the `calltrace` feature every TU function that
//! `tools/rust-port/trace/insert_ct.py` instrumented logs its name on entry into a per-frame list;
//! without the feature `ct!` expands to nothing. `melee golden --calltrace OUT` writes
//! `frame,fn,seq` (seq = position within the frame). Frame numbers are set by the driver with
//! `begin_frame`; calls made before the first `begin_frame` are logged as frame -1.
//! `MELEE_CALLTRACE_FRAMES=a-b` (or the `--calltrace-frames` flag) limits what is kept.

#[cfg(feature = "calltrace")]
mod imp {
    use std::sync::Mutex;

    struct State {
        frame: i64,
        lo: i64,
        hi: i64,
        seq: u32,
        rows: Vec<(i64, u32, &'static str)>,
    }
    static ST: Mutex<State> = Mutex::new(State { frame: -1, lo: i64::MIN, hi: i64::MAX, seq: 0, rows: Vec::new() });

    pub fn enter(name: &'static str) {
        if let Ok(mut s) = ST.lock() {
            if s.frame >= s.lo && s.frame <= s.hi {
                let (f, q) = (s.frame, s.seq);
                s.rows.push((f, q, name));
            }
            s.seq += 1;
        }
    }
    pub fn begin_frame(f: i64) {
        if let Ok(mut s) = ST.lock() {
            s.frame = f;
            s.seq = 0;
        }
    }
    pub fn set_range(lo: i64, hi: i64) {
        if let Ok(mut s) = ST.lock() {
            s.lo = lo;
            s.hi = hi;
        }
    }
    pub fn write(path: &str) -> std::io::Result<usize> {
        use std::io::Write;
        let s = ST.lock().unwrap();
        let mut w = std::io::BufWriter::new(std::fs::File::create(path)?);
        writeln!(w, "frame,fn,seq")?;
        for (f, q, n) in &s.rows {
            writeln!(w, "{f},{n},{q}")?;
        }
        w.flush()?;
        Ok(s.rows.len())
    }
    pub const ENABLED: bool = true;
}

#[cfg(not(feature = "calltrace"))]
mod imp {
    #[inline(always)]
    pub fn enter(_: &'static str) {}
    #[inline(always)]
    pub fn begin_frame(_: i64) {}
    #[inline(always)]
    pub fn set_range(_: i64, _: i64) {}
    pub fn write(_: &str) -> std::io::Result<usize> {
        Err(std::io::Error::other("built without the melee_core `calltrace` feature (cargo build --features calltrace)"))
    }
    pub const ENABLED: bool = false;
}

pub use imp::*;

/// Entry marker for a ported C function: `melee_core::ct!("Fighter_8006A1BC");`. Nothing without the feature
/// (the cfg is on the definition, so callers need no feature of their own).
#[cfg(feature = "calltrace")]
#[macro_export]
macro_rules! ct {
    ($n:expr) => {
        $crate::ct::enter($n)
    };
}
#[cfg(not(feature = "calltrace"))]
#[macro_export]
macro_rules! ct {
    ($n:expr) => {};
}
