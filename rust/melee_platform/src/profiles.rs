//! Per-controller profiles: deadzones, analog-to-trigger, button remaps and box (digital stick)
//! mode with SOCD, read from a small config file (`controllers.ini`) and CLI flags.
//!
//! A profile turns one gamepad's raw state ([`RawGamepad`], SDL conventions, from gilrs) into a
//! `PADStatus`, replacing Aurora's arithmetic for that port only. Its output goes down the same
//! `PADRead` path as every other source (adapter override, pad script, reset combo after it), so
//! recordings, replays and netplay see plain PAD bytes and the sim stays deterministic. With no
//! config file and no flags nothing here runs and input is exactly the Aurora default.
//!
//! The file (`#` or `;` comments; keys are case-insensitive):
//!
//! ```text
//! [profile mypro]            ; a named profile
//! match = switch_pro         ; auto-apply to a family (xinput switch_pro ds4 ds5 phob box generic)
//! match_name = 8BitDo        ; ... or to devices whose name contains this (case-insensitive)
//! mode = pad                 ; pad | box
//! stick_deadzone = 0.10      ; radial, fraction of full travel; the rest is rescaled to full range
//! cstick_deadzone = 0.10
//! trigger_deadzone = 0.05    ; analog L/R below this read 0
//! trigger_threshold = 0.85   ; analog L/R at or past this also press digital L/R (0 = never)
//! remap = South:B, East:A, RightShoulder:Z, Back:none
//! socd = 2ip_noreact         ; box mode: neutral | 2ip | 2ip_noreact | first
//! box = DpadLeft:Left, DpadRight:Right, LeftShoulder:ModX ...   ; box mode bindings
//!
//! [port 2]
//! profile = mypro            ; force a profile on a port, whatever is plugged in
//! ```
//!
//! CLI: `--controllers <file|default>` (also `MELEE_CONTROLLERS`), `--pad-profile <port>=<name>`,
//! `--socd <mode>` (overrides every box profile). `default` loads only the built-in family profiles.
use crate::aurora::{NativeButton, RawGamepad};
use crate::pad::*;
use crate::socd::{Socd, SocdMode};

/// What kind of device a gamepad is, from its USB ids (and name for the ones without fixed ids).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Hash)]
pub enum Family {
    #[default]
    Generic,
    XInput,
    SwitchPro,
    Ds4,
    Ds5,
    /// PhOB (open-source GameCube controller) in USB HID mode; through the GC adapter it is a GC controller.
    Phob,
    /// Box / all-button controllers (B0XX, Frame1, GP2040-CE, Haybox and similar).
    Box,
}

impl Family {
    pub const ALL: [Family; 7] = [Family::Generic, Family::XInput, Family::SwitchPro, Family::Ds4, Family::Ds5, Family::Phob, Family::Box];

    pub fn detect(vendor: Option<u16>, product: Option<u16>, name: &str) -> Family {
        let n = name.to_ascii_lowercase();
        // name checks first: boxes and the PhOB often reuse a console vendor's ids
        if n.contains("phob") {
            return Family::Phob;
        }
        if ["b0xx", "frame1", "gp2040", "haybox", "hitbox", "box controller"].iter().any(|k| n.contains(k)) {
            return Family::Box;
        }
        match (vendor, product) {
            (Some(0x057e), Some(0x2009)) => Family::SwitchPro,
            (Some(0x054c), Some(0x05c4 | 0x09cc | 0x0ba0)) => Family::Ds4,
            (Some(0x054c), Some(0x0ce6 | 0x0df2)) => Family::Ds5,
            (Some(0x045e), _) => Family::XInput,
            _ if n.contains("xbox") || n.contains("xinput") => Family::XInput,
            _ if n.contains("pro controller") => Family::SwitchPro,
            _ if n.contains("dualsense") => Family::Ds5,
            _ if n.contains("dualshock") || n.contains("ps4") => Family::Ds4,
            _ => Family::Generic,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Family::Generic => "generic",
            Family::XInput => "xinput",
            Family::SwitchPro => "switch_pro",
            Family::Ds4 => "ds4",
            Family::Ds5 => "ds5",
            Family::Phob => "phob",
            Family::Box => "box",
        }
    }

    pub fn parse(s: &str) -> Option<Family> {
        Family::ALL.into_iter().find(|f| f.name() == s.trim().to_ascii_lowercase())
    }
}

/// What a source button does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    None,
    /// GameCube buttons (`PAD_BUTTON_*` / `PAD_TRIGGER_*`).
    Pad(u16),
    /// Box mode only.
    Box(BoxAction),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoxAction {
    Left,
    Right,
    Up,
    Down,
    CLeft,
    CRight,
    CUp,
    CDown,
    ModX,
    ModY,
    LightShield,
    MidShield,
}

const NATIVE: [(&str, NativeButton); 19] = [
    ("south", NativeButton::South),
    ("east", NativeButton::East),
    ("west", NativeButton::West),
    ("north", NativeButton::North),
    ("start", NativeButton::Start),
    ("back", NativeButton::Back),
    ("guide", NativeButton::Guide),
    ("leftshoulder", NativeButton::LeftShoulder),
    ("rightshoulder", NativeButton::RightShoulder),
    ("leftstick", NativeButton::LeftStick),
    ("rightstick", NativeButton::RightStick),
    ("dpadup", NativeButton::DpadUp),
    ("dpaddown", NativeButton::DpadDown),
    ("dpadleft", NativeButton::DpadLeft),
    ("dpadright", NativeButton::DpadRight),
    ("lefttrigger2", NativeButton::LeftTrigger2),
    ("righttrigger2", NativeButton::RightTrigger2),
    ("c", NativeButton::C),
    ("z", NativeButton::Z),
];

fn parse_native(s: &str) -> Option<NativeButton> {
    let s = s.trim().to_ascii_lowercase();
    NATIVE.iter().find(|(n, _)| *n == s).map(|(_, b)| *b)
}

fn parse_target(s: &str) -> Option<Target> {
    use BoxAction::*;
    Some(match s.trim().to_ascii_lowercase().as_str() {
        "none" | "-" => Target::None,
        "a" => Target::Pad(PAD_BUTTON_A),
        "b" => Target::Pad(PAD_BUTTON_B),
        "x" => Target::Pad(PAD_BUTTON_X),
        "y" => Target::Pad(PAD_BUTTON_Y),
        "z" => Target::Pad(PAD_TRIGGER_Z),
        "l" => Target::Pad(PAD_TRIGGER_L),
        "r" => Target::Pad(PAD_TRIGGER_R),
        "start" => Target::Pad(PAD_BUTTON_START),
        "dup" => Target::Pad(PAD_BUTTON_UP),
        "ddown" => Target::Pad(PAD_BUTTON_DOWN),
        "dleft" => Target::Pad(PAD_BUTTON_LEFT),
        "dright" => Target::Pad(PAD_BUTTON_RIGHT),
        "left" => Target::Box(Left),
        "right" => Target::Box(Right),
        "up" => Target::Box(Up),
        "down" => Target::Box(Down),
        "cleft" => Target::Box(CLeft),
        "cright" => Target::Box(CRight),
        "cup" => Target::Box(CUp),
        "cdown" => Target::Box(CDown),
        "modx" => Target::Box(ModX),
        "mody" => Target::Box(ModY),
        "lightshield" => Target::Box(LightShield),
        "midshield" => Target::Box(MidShield),
        _ => return None,
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Mode {
    #[default]
    Pad,
    Box,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Profile {
    pub name: String,
    pub match_family: Option<Family>,
    pub match_name: Option<String>,
    pub mode: Mode,
    pub stick_deadzone: f32,
    pub cstick_deadzone: f32,
    pub trigger_deadzone: f32,
    /// 0 = analog never presses digital L/R.
    pub trigger_threshold: f32,
    /// Source button -> target; replaces the default for that source.
    pub remap: Vec<(NativeButton, Target)>,
    pub socd: SocdMode,
}

impl Profile {
    pub fn new(name: &str) -> Self {
        Profile {
            name: name.to_string(),
            match_family: None,
            match_name: None,
            mode: Mode::Pad,
            stick_deadzone: 0.10,
            cstick_deadzone: 0.10,
            trigger_deadzone: 0.0,
            trigger_threshold: 0.85,
            remap: Vec::new(),
            socd: SocdMode::default(),
        }
    }

    /// The built-in profile of a family. Port-side choices, documented in notes/controllers.md.
    pub fn builtin(f: Family) -> Self {
        let mut p = Profile::new(f.name());
        p.match_family = Some(f);
        match f {
            Family::XInput => p.stick_deadzone = 0.08,
            Family::Ds4 | Family::Ds5 => p.stick_deadzone = 0.06,
            // first-party Pro Controller sticks drift more; ZL/ZR are digital (read as full travel)
            Family::SwitchPro => p.stick_deadzone = 0.12,
            // the PhOB calibrates its sticks itself; add nothing on top
            Family::Phob => {
                p.stick_deadzone = 0.0;
                p.cstick_deadzone = 0.0;
            }
            Family::Box => p.mode = Mode::Box,
            Family::Generic => {}
        }
        p.cstick_deadzone = if f == Family::Phob { 0.0 } else { p.stick_deadzone.max(0.10) };
        p
    }

    fn target_of(&self, b: NativeButton) -> Target {
        if let Some((_, t)) = self.remap.iter().rev().find(|(s, _)| *s == b) {
            return *t;
        }
        default_target(self.mode, b)
    }

    fn matches(&self, pad: &RawGamepad) -> bool {
        self.match_family == Some(pad.family)
            || self.match_name.as_ref().is_some_and(|n| pad.name.to_ascii_lowercase().contains(&n.to_ascii_lowercase()))
    }
}

/// Aurora's standard table, plus the digital trigger buttons to L / R (pad mode), or the box layout.
fn default_target(mode: Mode, b: NativeButton) -> Target {
    use NativeButton as N;
    if mode == Mode::Box {
        let a = match b {
            N::DpadLeft => Some(BoxAction::Left),
            N::DpadRight => Some(BoxAction::Right),
            N::DpadUp => Some(BoxAction::Up),
            N::DpadDown => Some(BoxAction::Down),
            N::LeftShoulder => Some(BoxAction::ModX),
            N::Back => Some(BoxAction::ModY),
            _ => None,
        };
        if let Some(a) = a {
            return Target::Box(a);
        }
    }
    match b {
        N::South => Target::Pad(PAD_BUTTON_A),
        N::East => Target::Pad(PAD_BUTTON_B),
        N::West => Target::Pad(PAD_BUTTON_X),
        N::North => Target::Pad(PAD_BUTTON_Y),
        N::Start => Target::Pad(PAD_BUTTON_START),
        N::RightShoulder => Target::Pad(PAD_TRIGGER_Z),
        N::DpadUp => Target::Pad(PAD_BUTTON_UP),
        N::DpadDown => Target::Pad(PAD_BUTTON_DOWN),
        N::DpadLeft => Target::Pad(PAD_BUTTON_LEFT),
        N::DpadRight => Target::Pad(PAD_BUTTON_RIGHT),
        _ => Target::None,
    }
}

/// Raw SDL axis -> -1..1.
fn unit(v: i16) -> f32 {
    (v as f32 / 32767.0).clamp(-1.0, 1.0)
}

/// Radial deadzone, rescaled so travel just past it starts at 0 and the rim stays full. `y_up` in GC sense.
pub fn stick(x: f32, y_up: f32, dz: f32) -> (i8, i8) {
    let mut m = (x * x + y_up * y_up).sqrt();
    if m <= dz || m == 0.0 {
        return (0, 0);
    }
    let (nx, ny) = (x / m, y_up / m);
    m = m.min(1.0);
    let s = if dz >= 1.0 { 0.0 } else { (m - dz) / (1.0 - dz) };
    let q = |v: f32| (v * 127.0).round().clamp(-127.0, 127.0) as i8;
    (q(nx * s), q(ny * s))
}

/// Analog trigger (0..1) -> (analog byte, digital pressed).
pub fn trigger(t: f32, dz: f32, threshold: f32) -> (u8, bool) {
    let t = t.clamp(0.0, 1.0);
    let a = if t <= dz { 0 } else { (t * 255.0).round() as u8 };
    (a, threshold > 0.0 && t >= threshold)
}

/// Box-mode stick values, GameCube raw units (rim = 80, the reach of a real stick's gate after the game's clamp).
/// Port-side choices close to the commonly published box coordinates; documented in notes/controllers.md.
pub mod box_values {
    pub const FULL: i8 = 80;
    pub const DIAG: i8 = 56;
    pub const MODX_X: i8 = 53;
    pub const MODX_DIAG: (i8, i8) = (59, 25);
    pub const MODY_Y: i8 = 43;
    pub const MODY_DIAG: (i8, i8) = (25, 59);
    /// L analog for the light / mid shield buttons (no digital press).
    pub const LIGHT_SHIELD: u8 = 49;
    pub const MID_SHIELD: u8 = 94;
}

/// One port's running state (SOCD history for the four axes).
#[derive(Clone, Copy, Debug, Default)]
pub struct PortState {
    socd: [Socd; 4],
}

/// Map one gamepad through a profile.
pub fn map(p: &Profile, pad: &RawGamepad, state: &mut PortState) -> PADStatus {
    let mut s = PADStatus::zeroed();
    s.err = PAD_ERR_NONE;
    let mut held = [false; 12];
    for &b in &pad.buttons {
        match p.target_of(b) {
            Target::None => {}
            Target::Pad(bits) => s.button |= bits,
            Target::Box(a) => held[a as usize] = true,
        }
    }
    // digital triggers held: full travel
    if s.button & PAD_TRIGGER_L != 0 {
        s.triggerLeft = 255;
    }
    if s.button & PAD_TRIGGER_R != 0 {
        s.triggerRight = 255;
    }
    for (raw, digital, out) in [(pad.trigger_l, PAD_TRIGGER_L, 0usize), (pad.trigger_r, PAD_TRIGGER_R, 1)] {
        let (a, d) = trigger(unit(raw.max(0)), p.trigger_deadzone, p.trigger_threshold);
        let slot = if out == 0 { &mut s.triggerLeft } else { &mut s.triggerRight };
        *slot = (*slot).max(a);
        if d {
            s.button |= digital;
        }
    }
    match p.mode {
        Mode::Pad => {
            let (x, y) = stick(unit(pad.left_x), -unit(pad.left_y), p.stick_deadzone);
            s.stickX = x;
            s.stickY = y;
            let (x, y) = stick(unit(pad.right_x), -unit(pad.right_y), p.cstick_deadzone);
            s.substickX = x;
            s.substickY = y;
        }
        Mode::Box => {
            use box_values::*;
            use BoxAction as A;
            let h = |a: A| held[a as usize];
            let x = state.socd[0].resolve(p.socd, h(A::Left), h(A::Right));
            let y = state.socd[1].resolve(p.socd, h(A::Down), h(A::Up));
            let cx = state.socd[2].resolve(p.socd, h(A::CLeft), h(A::CRight));
            let cy = state.socd[3].resolve(p.socd, h(A::CDown), h(A::CUp));
            let (mx, my) = (h(A::ModX), h(A::ModY));
            let (ax, ay) = match (x != 0, y != 0, mx && !my, my && !mx) {
                (true, false, true, _) => (MODX_X, 0),
                (false, true, _, true) => (0, MODY_Y),
                (true, true, true, _) => MODX_DIAG,
                (true, true, _, true) => MODY_DIAG,
                (true, true, _, _) => (DIAG, DIAG),
                _ => (FULL, FULL),
            };
            s.stickX = x * ax;
            s.stickY = y * ay;
            s.substickX = cx * FULL;
            s.substickY = cy * FULL;
            if h(A::LightShield) {
                s.triggerRight = s.triggerRight.max(LIGHT_SHIELD);
            }
            if h(A::MidShield) {
                s.triggerRight = s.triggerRight.max(MID_SHIELD);
            }
        }
    }
    s
}

/// Every profile, the port overrides, and which ports they apply to.
#[derive(Clone, Debug, Default)]
pub struct ProfileSet {
    pub profiles: Vec<Profile>,
    pub ports: [Option<String>; PAD_CHANMAX],
    /// `--socd`: replaces every profile's SOCD mode.
    pub socd_override: Option<SocdMode>,
    pub warnings: Vec<String>,
}

impl ProfileSet {
    /// Only the built-in family profiles.
    pub fn builtin() -> Self {
        let mut s = ProfileSet::default();
        for f in Family::ALL {
            if f != Family::Generic {
                s.profiles.push(Profile::builtin(f));
            }
        }
        s
    }

    /// The config file on top of the built-ins (a file profile wins over a built-in for the same device).
    pub fn parse(text: &str) -> Self {
        let mut set = ProfileSet::builtin();
        let mut file: Vec<Profile> = Vec::new();
        enum Sec {
            None,
            Profile,
            Port(usize),
        }
        let mut sec = Sec::None;
        for (ln, raw) in text.lines().enumerate() {
            let line = raw.split(['#', ';']).next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }
            let warn = |set: &mut ProfileSet, m: &str| set.warnings.push(format!("line {}: {m}", ln + 1));
            if let Some(h) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
                let mut it = h.split_whitespace();
                match (it.next().map(|k| k.to_ascii_lowercase()), it.next()) {
                    (Some(k), Some(n)) if k == "profile" => {
                        file.retain(|p| p.name != n);
                        file.push(Profile::new(n));
                        sec = Sec::Profile;
                    }
                    (Some(k), Some(n)) if k == "port" => match n.parse::<usize>() {
                        Ok(n @ 1..=4) => sec = Sec::Port(n - 1),
                        _ => {
                            warn(&mut set, "port must be 1..4");
                            sec = Sec::None;
                        }
                    },
                    _ => {
                        warn(&mut set, "unknown section");
                        sec = Sec::None;
                    }
                }
                continue;
            }
            let Some((k, v)) = line.split_once('=') else {
                warn(&mut set, "expected key = value");
                continue;
            };
            let (k, v) = (k.trim().to_ascii_lowercase(), v.trim());
            match sec {
                Sec::None => warn(&mut set, "key outside a section"),
                Sec::Port(n) => {
                    if k == "profile" {
                        set.ports[n] = Some(v.to_string());
                    } else {
                        warn(&mut set, "unknown port key");
                    }
                }
                Sec::Profile => {
                    let p = file.last_mut().unwrap();
                    let num = |v: &str| v.parse::<f32>().ok().filter(|f| (0.0..=1.0).contains(f));
                    let ok = match k.as_str() {
                        "match" => Family::parse(v).map(|f| p.match_family = Some(f)).is_some(),
                        "match_name" => {
                            p.match_name = Some(v.to_string());
                            true
                        }
                        "mode" => match v.to_ascii_lowercase().as_str() {
                            "pad" => {
                                p.mode = Mode::Pad;
                                true
                            }
                            "box" => {
                                p.mode = Mode::Box;
                                true
                            }
                            _ => false,
                        },
                        "stick_deadzone" => num(v).map(|f| p.stick_deadzone = f).is_some(),
                        "cstick_deadzone" => num(v).map(|f| p.cstick_deadzone = f).is_some(),
                        "trigger_deadzone" => num(v).map(|f| p.trigger_deadzone = f).is_some(),
                        "trigger_threshold" => num(v).map(|f| p.trigger_threshold = f).is_some(),
                        "socd" => SocdMode::parse(v).map(|m| p.socd = m).is_some(),
                        "remap" | "box" => {
                            let mut all = true;
                            for pair in v.split(',').filter(|x| !x.trim().is_empty()) {
                                match pair.split_once(':').and_then(|(a, b)| Some((parse_native(a)?, parse_target(b)?))) {
                                    Some(e) => p.remap.push(e),
                                    None => all = false,
                                }
                            }
                            all
                        }
                        _ => false,
                    };
                    if !ok {
                        warn(&mut set, &format!("bad value for '{k}'"));
                    }
                }
            }
        }
        // file profiles first so they win the match
        file.append(&mut set.profiles);
        set.profiles = file;
        for n in set.ports.iter().flatten() {
            if !set.profiles.iter().any(|p| &p.name == n) {
                set.warnings.push(format!("port profile '{n}' is not defined"));
            }
        }
        set
    }

    /// `--pad-profile 2=name`.
    pub fn set_port(&mut self, spec: &str) -> Result<(), String> {
        let (p, n) = spec.split_once('=').ok_or("expected <port>=<profile>")?;
        let port: usize = p.trim().parse().ok().filter(|x| (1..=4).contains(x)).ok_or("port must be 1..4")?;
        let n = n.trim();
        if !self.profiles.iter().any(|x| x.name == n) {
            return Err(format!("no profile '{n}'"));
        }
        self.ports[port - 1] = Some(n.to_string());
        Ok(())
    }

    /// The profile for the gamepad on `port`, if any.
    pub fn for_port(&self, port: usize, pad: &RawGamepad) -> Option<&Profile> {
        if let Some(n) = &self.ports[port] {
            return self.profiles.iter().find(|p| &p.name == n);
        }
        self.profiles.iter().find(|p| p.matches(pad))
    }

    /// Map every port that has a gamepad and a profile; other ports keep their value.
    /// Returns the ports written (bit i = port i).
    pub fn apply(&self, pads: &[Option<RawGamepad>; PAD_CHANMAX], state: &mut [PortState; PAD_CHANMAX], st: &mut [PADStatus; PAD_CHANMAX]) -> u32 {
        let mut done = 0;
        for i in 0..PAD_CHANMAX {
            let Some(pad) = pads[i].as_ref() else {
                state[i] = PortState::default();
                continue;
            };
            let Some(p) = self.for_port(i, pad) else { continue };
            let mut p = p.clone();
            if let Some(m) = self.socd_override {
                p.socd = m;
            }
            let ext = st[i].extButton;
            st[i] = map(&p, pad, &mut state[i]);
            st[i].extButton = ext;
            done |= 1 << i;
        }
        done
    }
}

/// Build the set from CLI args (`--controllers`, `--pad-profile`, `--socd`) and `MELEE_CONTROLLERS`.
/// None when nothing asks for profiles.
pub fn from_args(args: &[String]) -> Result<Option<ProfileSet>, String> {
    let get = |k: &str| args.windows(2).filter(|w| w[0] == k).map(|w| w[1].clone()).collect::<Vec<_>>();
    let src = get("--controllers").pop().or_else(|| std::env::var("MELEE_CONTROLLERS").ok().filter(|v| !v.is_empty()));
    let ports = get("--pad-profile");
    let socd = get("--socd").pop();
    if src.is_none() && ports.is_empty() && socd.is_none() {
        return Ok(None);
    }
    let mut set = match src.as_deref() {
        None | Some("default") => ProfileSet::builtin(),
        Some(path) => ProfileSet::parse(&std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?),
    };
    for p in &ports {
        set.set_port(p)?;
    }
    if let Some(m) = socd {
        set.socd_override = Some(SocdMode::parse(&m).ok_or(format!("unknown SOCD mode '{m}'"))?);
    }
    Ok(Some(set))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pad(f: Family, buttons: &[NativeButton]) -> RawGamepad {
        RawGamepad { family: f, buttons: buttons.iter().copied().collect(), ..Default::default() }
    }

    #[test]
    fn family_detection() {
        assert_eq!(Family::detect(Some(0x057e), Some(0x2009), "Pro Controller"), Family::SwitchPro);
        assert_eq!(Family::detect(Some(0x054c), Some(0x09cc), "Wireless Controller"), Family::Ds4);
        assert_eq!(Family::detect(Some(0x054c), Some(0x0ce6), "DualSense"), Family::Ds5);
        assert_eq!(Family::detect(Some(0x045e), Some(0x028e), "Xbox 360 Controller"), Family::XInput);
        assert_eq!(Family::detect(Some(0x2e8a), Some(0x10dd), "GP2040-CE"), Family::Box);
        assert_eq!(Family::detect(Some(0x057e), Some(0x2009), "PhOB 2"), Family::Phob);
        assert_eq!(Family::detect(None, None, "Some Pad"), Family::Generic);
    }

    #[test]
    fn deadzone_radial_and_rescaled() {
        assert_eq!(stick(0.05, 0.05, 0.10), (0, 0));
        assert_eq!(stick(1.0, 0.0, 0.10), (127, 0));
        assert_eq!(stick(0.0, -1.0, 0.10), (0, -127));
        // just past the deadzone starts near zero, not at the deadzone's value
        let (x, _) = stick(0.11, 0.0, 0.10);
        assert!(x <= 2, "{x}");
        // halfway through the live range
        assert_eq!(stick(0.55, 0.0, 0.10).0, 64);
        // past the rim is clamped to the rim, direction kept
        let (x, y) = stick(1.0, 1.0, 0.0);
        assert_eq!((x, y), (90, 90));
        assert_eq!(stick(0.3, 0.0, 0.0).0, 38);
    }

    #[test]
    fn analog_to_trigger() {
        assert_eq!(trigger(0.03, 0.05, 0.85), (0, false));
        assert_eq!(trigger(0.5, 0.05, 0.85), (128, false));
        assert_eq!(trigger(0.9, 0.05, 0.85), (230, true));
        assert_eq!(trigger(1.0, 0.0, 0.0), (255, false));
        let p = Profile::builtin(Family::Ds4);
        let mut g = pad(Family::Ds4, &[]);
        g.trigger_l = 32767;
        g.trigger_r = 10000;
        let s = map(&p, &g, &mut PortState::default());
        assert_eq!(s.triggerLeft, 255);
        assert!(s.button & PAD_TRIGGER_L != 0);
        assert!(s.button & PAD_TRIGGER_R == 0);
        assert_eq!(s.triggerRight, 78);
    }

    #[test]
    fn switch_pro_digital_triggers_full_travel() {
        let p = Profile::builtin(Family::SwitchPro);
        let mut g = pad(Family::SwitchPro, &[NativeButton::LeftTrigger2]);
        g.trigger_l = 32767; // gilrs reports a digital ZL as value 1.0
        let s = map(&p, &g, &mut PortState::default());
        assert_eq!(s.triggerLeft, 255);
        assert!(s.button & PAD_TRIGGER_L != 0);
    }

    #[test]
    fn default_and_remapped_buttons() {
        let mut p = Profile::new("t");
        let g = pad(Family::Generic, &[NativeButton::South, NativeButton::RightShoulder]);
        let s = map(&p, &g, &mut PortState::default());
        assert_eq!(s.button, PAD_BUTTON_A | PAD_TRIGGER_Z);
        p.remap.push((NativeButton::South, Target::Pad(PAD_BUTTON_B)));
        p.remap.push((NativeButton::RightShoulder, Target::None));
        let s = map(&p, &g, &mut PortState::default());
        assert_eq!(s.button, PAD_BUTTON_B);
    }

    #[test]
    fn box_mode_values_and_socd() {
        use box_values::*;
        let p = Profile::builtin(Family::Box);
        let mut st = PortState::default();
        let s = map(&p, &pad(Family::Box, &[NativeButton::DpadRight]), &mut st);
        assert_eq!((s.stickX, s.stickY), (FULL, 0));
        let s = map(&p, &pad(Family::Box, &[NativeButton::DpadRight, NativeButton::LeftShoulder]), &mut st);
        assert_eq!((s.stickX, s.stickY), (MODX_X, 0));
        let s = map(&p, &pad(Family::Box, &[NativeButton::DpadRight, NativeButton::DpadUp]), &mut st);
        assert_eq!((s.stickX, s.stickY), (DIAG, DIAG));
        let s = map(&p, &pad(Family::Box, &[NativeButton::DpadDown, NativeButton::Back]), &mut st);
        assert_eq!((s.stickX, s.stickY), (0, -MODY_Y));
        // 2IP without reactivation: right held, left added -> left; left released -> neutral
        let mut st = PortState::default();
        let xs: Vec<i8> = [&[NativeButton::DpadRight][..], &[NativeButton::DpadRight, NativeButton::DpadLeft], &[NativeButton::DpadRight]]
            .iter()
            .map(|b| map(&p, &pad(Family::Box, b), &mut st).stickX)
            .collect();
        assert_eq!(xs, [FULL, -FULL, 0]);
    }

    #[test]
    fn same_input_same_output() {
        // determinism: two independent states fed the same frames give identical PAD bytes
        let p = Profile::builtin(Family::Box);
        let frames = [vec![NativeButton::DpadLeft], vec![NativeButton::DpadLeft, NativeButton::DpadRight], vec![], vec![NativeButton::DpadRight]];
        let (mut a, mut b) = (PortState::default(), PortState::default());
        for f in &frames {
            let (x, y) = (map(&p, &pad(Family::Box, f), &mut a), map(&p, &pad(Family::Box, f), &mut b));
            assert!(pad_eq(&x, &y));
        }
    }

    #[test]
    fn config_file() {
        let text = "\
# comment
[profile mine]
match_name = 8bitdo
stick_deadzone = 0.2
trigger_threshold = 0
remap = South:B, East:A, Back:none
[profile hitbox]
mode = box
socd = neutral
box = West:Left, C:CUp
[port 3]
profile = hitbox
[profile bad]
stick_deadzone = 7
";
        let set = ProfileSet::parse(text);
        assert_eq!(set.warnings.len(), 1, "{:?}", set.warnings);
        let m = set.profiles.iter().find(|p| p.name == "mine").unwrap();
        assert_eq!(m.stick_deadzone, 0.2);
        assert_eq!(m.remap.len(), 3);
        let h = set.profiles.iter().find(|p| p.name == "hitbox").unwrap();
        assert_eq!((h.mode, h.socd), (Mode::Box, SocdMode::Neutral));
        let mut g = pad(Family::XInput, &[]);
        g.name = "8BitDo Pro 2".into();
        assert_eq!(set.for_port(0, &g).unwrap().name, "mine");
        g.name = "Xbox Controller".into();
        assert_eq!(set.for_port(0, &g).unwrap().name, "xinput");
        assert_eq!(set.for_port(2, &g).unwrap().name, "hitbox");
        assert!(set.for_port(1, &pad(Family::Generic, &[])).is_none());
    }

    #[test]
    fn apply_only_touches_profiled_ports_and_resets_on_unplug() {
        let mut set = ProfileSet::builtin();
        set.socd_override = Some(SocdMode::Neutral);
        let mut pads: [Option<RawGamepad>; PAD_CHANMAX] = Default::default();
        pads[1] = Some(pad(Family::Box, &[NativeButton::DpadLeft, NativeButton::DpadRight]));
        pads[2] = Some(pad(Family::Generic, &[]));
        let mut st = [pad_disconnected(); PAD_CHANMAX];
        let mut state = [PortState::default(); PAD_CHANMAX];
        assert_eq!(set.apply(&pads, &mut state, &mut st), 0b10);
        assert_eq!(st[1].stickX, 0);
        assert_eq!(st[0].err, PAD_ERR_NO_CONTROLLER);
    }
}
