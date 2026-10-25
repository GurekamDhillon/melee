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
    /// `gw_hang_script_state`: pending and expired script deadlines, and the last expired name
    fn hang_script_state(&mut self, _pending: i32, _expired: i32, _last: &str) {}
    /// `gw_perf_snapshot` + `gw_surface_stats` + `gw_Shader_Perf`: host frame diagnostics (never game state).
    fn perf_snapshot(&mut self, _want: usize) -> PerfSnapshot {
        PerfSnapshot::default()
    }
    /// `gw_prof_*` readers for the profiler block of `gd.perf`.
    fn prof_snapshot(&mut self) -> ProfSnapshot {
        ProfSnapshot::default()
    }
    /// `gw_prof_command`: (ok, message)
    fn prof_command(&mut self, _args: &str) -> (bool, String) {
        (false, "prof on|off|reset|report|perf|summary on|off|trace <frames>|hitch <ms>".into())
    }
}

/// `GW_PERF_HISTORY`
pub const GW_PERF_HISTORY: usize = 240;

/// `GwPerfFrame`
#[derive(Clone, Copy, Default, Debug)]
pub struct PerfFrame {
    pub total_ms: f32,
    pub logic_ms: f32,
    pub gx_ms: f32,
    pub submit_ms: f32,
    pub worker_ms: f32,
    pub draw_calls: u32,
    pub vertices: u32,
    pub fx_particles: u32,
}

/// `GwShaderPerf`
#[derive(Clone, Copy, Default, Debug)]
pub struct ShaderPerf {
    pub compiles: i64,
    pub cache_hits: i64,
    pub errors: i64,
    pub reloads: i64,
    pub draws: i64,
    pub passes: i64,
    pub resolves: i64,
    pub skipped: i64,
    pub pixels: f64,
    pub uniform_bytes: f64,
    pub vertex_bytes: f64,
    pub record_ms: f64,
}

#[derive(Clone, Default, Debug)]
pub struct PerfSnapshot {
    pub fps: f32,
    pub target: i32,
    /// load calls, fifo commands, selections
    pub surface: [u64; 3],
    /// oldest to newest
    pub frames: Vec<PerfFrame>,
    pub shaders: ShaderPerf,
}

/// `GwProfStats`
#[derive(Clone, Copy, Default, Debug)]
pub struct ProfStats {
    pub count: u64,
    pub mean: f64,
    pub p50: f64,
    pub p95: f64,
    pub p99: f64,
    pub max: f64,
}

/// `GwProfDetailStats` (`port_kind` is set for the fighter zones, which carry port and kind in `detail`)
#[derive(Clone, Debug, Default)]
pub struct ProfDetail {
    pub zone: String,
    pub name: String,
    pub detail: u32,
    pub stats: ProfStats,
    pub port_kind: bool,
}

#[derive(Clone, Default, Debug)]
pub struct ProfSnapshot {
    pub enabled: bool,
    pub frames: u64,
    pub zones: Vec<(String, ProfStats)>,
    pub counters: Vec<(String, ProfStats)>,
    pub details: Vec<ProfDetail>,
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
