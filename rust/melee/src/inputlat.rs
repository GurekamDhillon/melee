//! Input-latency probe for the online paths (`play --slippi-mm`, `play --p2p-*`): `MELEE_INPUTLAT_CSV=<file>` writes
//! one row per event so two peers' files can be joined (tools/rust-port/inputlat/join.py). Times are wall-clock
//! microseconds (`SystemTime`), shared by two processes on one machine.
//!
//! Rows `kind,frame,a,b`:
//! - `T,f,sample,sim`: the tick that runs online frame `f`: when this tick read the controllers, when its first
//!   session frame started (sample-to-sim age = sim - sample);
//! - `S,f,sample,sent`: our input of online frame `f` went on the wire (flushed) at `sent`; its pad was read at `sample`;
//! - `R,f,used,cur`: the peer's input of online frame `f` reached our session at `used` (the tick that took it), while
//!   our session was at online frame `cur` (lead = f - cur frames).
//!
//! Test instrument only: the row buffer is reserved when the match connects and written when the driver is dropped,
//! so nothing is allocated or freed per frame.

use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Copy)]
struct Row {
    kind: u8,
    frame: i32,
    a: i64,
    b: i64,
}

pub struct LatProbe {
    rows: Vec<Row>,
    path: String,
    /// when the current tick read the controllers
    pub sample_us: i64,
}

/// Rows kept (about 20 minutes of a match; later rows are dropped).
const CAP: usize = 1 << 18;

impl LatProbe {
    pub fn from_env() -> Option<Box<LatProbe>> {
        let path = std::env::var("MELEE_INPUTLAT_CSV").ok().filter(|p| !p.is_empty())?;
        Some(Box::new(LatProbe { rows: Vec::with_capacity(CAP), path, sample_us: now_us() }))
    }

    pub fn push(&mut self, kind: u8, frame: i32, a: i64, b: i64) {
        if self.rows.len() < self.rows.capacity() {
            self.rows.push(Row { kind, frame, a, b });
        }
    }
}

pub fn now_us() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_micros() as i64)
}

impl Drop for LatProbe {
    fn drop(&mut self) {
        let mut t = String::with_capacity(self.rows.len() * 32 + 32);
        t.push_str("kind,frame,a,b\n");
        for r in &self.rows {
            t.push_str(&format!("{},{},{},{}\n", r.kind as char, r.frame, r.a, r.b));
        }
        let _ = std::fs::write(&self.path, t);
    }
}
