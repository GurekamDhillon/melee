//! Platform log (stands in for `gw_log`). Lines go to a process-wide sink: stderr by default,
//! or a capture buffer (tests). `MELEE_PLATFORM_QUIET=1` drops them.
use std::sync::atomic::{AtomicBool, Ordering::Relaxed};
use std::sync::Mutex;

static CAPTURE: Mutex<Option<Vec<String>>> = Mutex::new(None);
/// Capture is a test mode: outside it no line takes CAPTURE's lock. A rollback load rewrites the statics, a std Mutex
/// among them; one held by a worker thread (adapter, YouGame) at a snapshot came back held with no owner.
static CAPTURING: AtomicBool = AtomicBool::new(false);

pub fn log(line: String) {
    if !CAPTURING.load(Relaxed) {
        // the common path: no lock
    } else if let Ok(mut c) = CAPTURE.lock() {
        if let Some(buf) = c.as_mut() {
            buf.push(line);
            return;
        }
    }
    if std::env::var("MELEE_PLATFORM_QUIET").map(|v| v == "1").unwrap_or(false) {
        return;
    }
    eprintln!("{line}");
}

/// Start capturing log lines instead of printing them.
pub fn capture_start() {
    *CAPTURE.lock().unwrap() = Some(Vec::new());
    CAPTURING.store(true, Relaxed);
}
/// Stop capturing; return what was captured.
pub fn capture_take() -> Vec<String> {
    CAPTURING.store(false, Relaxed);
    CAPTURE.lock().unwrap().take().unwrap_or_default()
}

#[macro_export]
macro_rules! plog {
    ($($t:tt)*) => { $crate::log::log(format!($($t)*)) };
}
