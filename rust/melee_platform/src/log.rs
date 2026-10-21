//! Platform log (stands in for `gw_log`). Lines go to a process-wide sink: stderr by default,
//! or a capture buffer (tests). `MELEE_PLATFORM_QUIET=1` drops them.
use std::sync::Mutex;

static CAPTURE: Mutex<Option<Vec<String>>> = Mutex::new(None);

pub fn log(line: String) {
    if let Ok(mut c) = CAPTURE.lock() {
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
}
/// Stop capturing; return what was captured.
pub fn capture_take() -> Vec<String> {
    CAPTURE.lock().unwrap().take().unwrap_or_default()
}

#[macro_export]
macro_rules! plog {
    ($($t:tt)*) => { $crate::log::log(format!($($t)*)) };
}
