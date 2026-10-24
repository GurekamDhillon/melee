//! What the scripting engine needs from the platform around it (the `gw_*` services `gw_script.c` calls:
//! the log, clocks, the keyboard, rollback/netplay state). The game itself is reached through
//! [`crate::game::ScriptGame`], the port of the `gw_ScriptGame_*` scalar interface.

pub trait Host {
    /// `gw_log`: the port log (one line, no newline).
    fn log(&mut self, msg: &str);
    /// `gs_now_ms`: a monotonic millisecond clock.
    fn now_ms(&self) -> f64;
    /// `gw_RB_Enabled() || gw_Netplay_Enabled()`: a rollback or netplay session is running, so gameplay
    /// writes are refused.
    fn session(&self) -> bool {
        false
    }
    /// `gw_turbo_enabled`: rendering is uncapped, so the wall-clock backstop of the call budget is off.
    fn turbo(&self) -> bool {
        false
    }
    /// `GetAsyncKeyState(vk) & 0x8000` while the game window has the keyboard.
    fn key_down(&self, _vk: usize) -> bool {
        false
    }
    /// `gw_Console_ScriptWidth`: the overlay canvas width (640 at 4:3).
    fn canvas_width(&self) -> f32 {
        640.0
    }
    /// `gw_Mouse_ScriptRead`: (x, y, buttons, wheel) in the 640x480 script space; (-1000, -1000) outside the picture.
    fn mouse(&self) -> (f32, f32, i32, f32) {
        (-1000.0, -1000.0, 0, 0.0)
    }
    /// `gs_request_quit`: close the window the way the user would.
    fn request_quit(&mut self) {}
    /// `gw_Overlay_SetRunLabel`
    fn set_run_label(&mut self, _text: &str) {}
    /// `getenv`
    fn getenv(&self, key: &str) -> Option<String> {
        std::env::var(key).ok()
    }
    /// `GetLocalTime`: (year, month, day, hour, minute, second)
    fn local_time(&self) -> (u32, u32, u32, u32, u32, u32) {
        (1970, 1, 1, 0, 0, 0)
    }
    /// `localtime` + `strftime("%Y-%m-%d %H:%M")` of a unix time (UTC by default)
    fn format_time(&self, unix: i64) -> String {
        let days = unix.div_euclid(86400);
        let rem = unix.rem_euclid(86400);
        let z = days + 719468;
        let era = z.div_euclid(146097);
        let doe = z.rem_euclid(146097);
        let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let d = doy - (153 * mp + 2) / 5 + 1;
        let m = if mp < 10 { mp + 3 } else { mp - 9 };
        let y = yoe + era * 400 + (m <= 2) as i64;
        format!("{y:04}-{m:02}-{d:02} {:02}:{:02}", rem / 3600, rem % 3600 / 60)
    }
    /// `gw_SceneReport_SceneName`
    fn scene_name(&self, kind: i32) -> String {
        format!("scene {kind}")
    }
    /// `gw_SceneReport_ModeName`
    fn mode_name(&self, mode: i32) -> String {
        format!("mode {mode}")
    }
}

/// A host that keeps its log in memory; used by tests and the `gdscript` runner.
pub struct MemHost {
    pub lines: Vec<String>,
    pub echo: bool,
    pub quit: bool,
    pub label: String,
    pub keys: [bool; 256],
    pub start: std::time::Instant,
    pub session: bool,
}

impl Default for MemHost {
    fn default() -> Self {
        MemHost {
            lines: Vec::new(),
            echo: false,
            quit: false,
            label: String::new(),
            keys: [false; 256],
            start: std::time::Instant::now(),
            session: false,
        }
    }
}

impl Host for MemHost {
    fn log(&mut self, msg: &str) {
        if self.echo {
            println!("{msg}");
        }
        self.lines.push(msg.to_string());
    }
    fn now_ms(&self) -> f64 {
        self.start.elapsed().as_secs_f64() * 1000.0
    }
    fn session(&self) -> bool {
        self.session
    }
    fn key_down(&self, vk: usize) -> bool {
        self.keys.get(vk).copied().unwrap_or(false)
    }
    fn request_quit(&mut self) {
        self.quit = true;
    }
    fn set_run_label(&mut self, text: &str) {
        self.label = text.to_string();
    }
}
