//! Replay playback controls for `melee play --replay` / `--spectate` (docs/rust-port/notes/spectate-replay.md):
//! pause, frame step, speed, seek (snapshot-assisted), camera.
//!
//! Sources, all applied at the start of a render tick:
//! * options: `--replay-paused`, `--replay-speed X` (0.25..8), `--replay-seek T` (feed tick, or `mN` = match frame N),
//!   `--replay-camera game|fixed|standard`, `--replay-snap-every N` (ticks between seek snapshots, default 300);
//! * a control script (tests, no OS input): `--replay-ctl "120:pause,130:step,140:step,200:seek=60,260:speed=2,300:play"`
//!   or `MELEE_REPLAY_CTL`, keyed by render tick;
//! * the keyboard in a window (only while no UI mod holds it): space pause, `.` step, `,` step back, `+` / `-` speed,
//!   left / right seek -/+ 10 s, `c` camera, Home back to the match's first snapshot.
//!
//! Seeking back loads the nearest snapshot at or before the target (snapshots are taken in the VS match every N ticks
//! once the run is in the rollback allocator modes, which a seekable playback turns on) and plays forward unpaced; seeking
//! forward plays forward unpaced. Re-played ticks are hash-checked again (`ReplayFeed::rehashed`), which is the check that
//! a seek is deterministic.

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Camera {
    Game,
    Fixed,
    Standard,
}

impl Camera {
    fn parse(s: &str) -> Option<Camera> {
        match s {
            "game" | "default" => Some(Camera::Game),
            "fixed" => Some(Camera::Fixed),
            "standard" | "follow" => Some(Camera::Standard),
            _ => None,
        }
    }
}

pub enum Seek {
    Tick(usize),
    MatchFrame(u32),
    Rel(i64),
    First,
}

pub struct Ctl {
    pub paused: bool,
    pub speed: f32,
    acc: f32,
    pub step: u32,
    pub seek: Option<Seek>,
    /// a seek being played forward: run unpaced to this feed tick
    pub ff_to: Option<usize>,
    pub camera: Camera,
    pub snap_every: usize,
    script: Vec<(u64, String)>,
    pub seekable: bool,
}

fn log(m: &str) {
    eprintln!("play: replay ctl: {m}");
}

impl Ctl {
    /// None when the command line asks for no control (plain playback stays exactly as before).
    pub fn from_args(args: &[String]) -> Option<Ctl> {
        let get = |k: &str| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned();
        let has = |k: &str| args.iter().any(|a| a == k);
        let script_s = get("--replay-ctl").or_else(|| std::env::var("MELEE_REPLAY_CTL").ok()).unwrap_or_default();
        let any = has("--replay-paused") || has("--replay-seekable") || ["--replay-speed", "--replay-seek", "--replay-camera", "--replay-snap-every"].iter().any(|k| get(k).is_some()) || !script_s.is_empty();
        if !any {
            return None;
        }
        let mut c = Ctl {
            paused: has("--replay-paused"),
            speed: get("--replay-speed").and_then(|v| v.parse().ok()).unwrap_or(1.0f32).clamp(0.25, 8.0),
            acc: 0.0,
            step: 0,
            seek: None,
            ff_to: None,
            camera: get("--replay-camera").and_then(|v| Camera::parse(&v)).unwrap_or(Camera::Game),
            snap_every: get("--replay-snap-every").and_then(|v| v.parse().ok()).unwrap_or(300usize).max(1),
            script: Vec::new(),
            seekable: true,
        };
        if let Some(s) = get("--replay-seek") {
            c.cmd(&format!("seek={s}"));
        }
        for part in script_s.split(',').filter(|p| !p.trim().is_empty()) {
            match part.split_once(':').and_then(|(t, cmd)| t.trim().parse::<u64>().ok().map(|t| (t, cmd.trim().to_string()))) {
                Some(x) => c.script.push(x),
                None => log(&format!("bad control '{part}' (TICK:CMD)")),
            }
        }
        c.script.sort_by_key(|x| x.0);
        Some(c)
    }

    /// One command: pause, play, toggle, step, back, speed=X, faster, slower, seek=T|mN|+N|-N|first, camera=NAME.
    pub fn cmd(&mut self, c: &str) {
        let (k, v) = c.split_once('=').unwrap_or((c, ""));
        match k {
            "pause" => self.paused = true,
            "play" => self.paused = false,
            "toggle" => self.paused = !self.paused,
            "step" => {
                self.paused = true;
                self.step += v.parse::<u32>().unwrap_or(1);
            }
            "back" => {
                self.paused = true;
                self.seek = Some(Seek::Rel(-(v.parse::<i64>().unwrap_or(1))));
            }
            "speed" => self.speed = v.parse().unwrap_or(1.0f32).clamp(0.25, 8.0),
            "faster" => self.speed = (self.speed * 2.0).min(8.0),
            "slower" => self.speed = (self.speed / 2.0).max(0.25),
            "seek" => {
                self.seek = if v == "first" {
                    Some(Seek::First)
                } else if let Some(m) = v.strip_prefix('m') {
                    m.parse().ok().map(Seek::MatchFrame)
                } else if v.starts_with('+') || v.starts_with('-') {
                    v.parse().ok().map(Seek::Rel)
                } else {
                    v.parse().ok().map(Seek::Tick)
                };
                if self.seek.is_none() {
                    log(&format!("bad seek '{v}'"));
                }
            }
            "camera" => match Camera::parse(v) {
                Some(cam) => self.camera = cam,
                None => {
                    self.camera = match self.camera {
                        Camera::Game => Camera::Fixed,
                        Camera::Fixed => Camera::Standard,
                        Camera::Standard => Camera::Game,
                    }
                }
            },
            _ => {
                log(&format!("unknown command '{c}'"));
                return;
            }
        }
        log(&format!("{c} -> {}", self.state()));
    }

    pub fn state(&self) -> String {
        format!("{} speed {}x camera {:?}", if self.paused { "paused" } else { "playing" }, self.speed, self.camera)
    }

    /// The script's commands due at this render tick.
    pub fn on_tick(&mut self, tick: u64) {
        while self.script.first().is_some_and(|x| x.0 <= tick) {
            let (_, c) = self.script.remove(0);
            self.cmd(&c);
        }
    }

    /// Window keys (melee_platform's UI key events: 3<<24|char, 1<<24|code down).
    pub fn keys(&mut self, q: &[u32]) {
        for &k in q {
            let (kind, v) = (k >> 24, k & 0xFF_FFFF);
            let c = match (kind, v) {
                (3, 0x20) => "toggle",
                (3, 0x2E) => "step",
                (3, 0x2C) => "back",
                (3, 0x2B) | (3, 0x3D) => "faster",
                (3, 0x2D) => "slower",
                (3, 0x63) | (3, 0x43) => "camera",
                (1, 6) => "seek=-600",
                (1, 7) => "seek=+600",
                _ => continue,
            };
            self.cmd(c);
        }
    }

    /// How many feed ticks to run this render tick (0 = hold the frame). A pending fast-forward is not counted here.
    pub fn steps(&mut self) -> usize {
        if self.paused {
            let n = self.step.min(1) as usize;
            self.step -= n as u32;
            return n;
        }
        self.acc += self.speed;
        let n = self.acc.floor();
        self.acc -= n;
        n as usize
    }

    /// The game camera mode the option asks for, applied in the match before each tick (a presentation choice: the
    /// neutral hash does not include the camera, and the playback's hash check says whether a mode changed gameplay).
    pub fn apply_camera(&self) {
        use melee_world::tu::camera as cm;
        unsafe {
            match self.camera {
                Camera::Game => {}
                Camera::Fixed => cm::Camera_SetModeToFixed(),
                Camera::Standard => cm::Camera_SetModeToStandard(),
            }
        }
    }
}
