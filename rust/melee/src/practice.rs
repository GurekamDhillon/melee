//! Practice aids for the live game (`melee play`), all OFF by default:
//!
//! * `lcancel_indicator` (F5): per player, whether the last aerial landing was L-cancelled and the timing (frames from
//!   the trigger press to the landing, or how late the press came when it missed). Reads the fighters only.
//! * `input_display` (F6): per connected port, the stick, C-stick, analog triggers and buttons the game read this frame
//!   (`HSD_PadMasterStatus`, after the game's own clamp). Reads the pad status only.
//! * `no_screen_shake` (F7): the screen shake (camera quake) left out of the rendered camera pose
//!   (`melee_world::tu::camera::NO_SCREEN_SHAKE`); the simulated camera is untouched.
//! * `lcancel_helper` (NO key; settings.cfg / `--lcancel-helper` only): a GAMEPLAY CHANGE. It presses analog L on
//!   alternate frames while a human player's aerial is in the air, so every aerial landing is L-cancelled. Offline only:
//!   it is refused under netplay, synctest and replay playback (a netplay peer could not reproduce it), and it acts on
//!   the inputs (before `set_pads`), so a `--record`ed replay carries the presses and plays back identically.
//!
//! The first three are presentation only: the overlay is extra GX draws appended to the finished frame, and nothing is
//! written into game memory. `melee golden` takes the same settings, so the goldens can be (and were) re-run with them on.
//!
//! Settings: `MELEE_<KEY>` / `<key>=1` in settings.cfg (crate::cfg), or `--lcancel-indicator`, `--input-display`,
//! `--no-screen-shake`, `--lcancel-helper` on the command line.
//!
//! Reference (reimplemented, not copied): Hero's melee-unlocked `port/runtime/host/lcancel.cpp`. The L-cancel rule it
//! documents is the game's own: `fighter.c` resets `x67F` on a fresh L/R/Z/analog press and counts it up each frame;
//! `ftCo_LandingAir` halves the landing lag when `x67F < ftCommonData.xE4` (7 on a stock PlCo).
use crate::ui::{self, Ui};
use melee_gx::draw::DrawCmd;
use melee_gx::shim::GxShim;
use std::sync::atomic::Ordering;

const ATTACK_AIR_N: i32 = 65;
const ATTACK_AIR_LW: i32 = 69;
const LANDING_AIR_N: i32 = 70;
const LANDING_AIR_LW: i32 = 74;
/// `ftCommonData.xE4` on the retail PlCo.dat: the L-cancel window.
const LC_WINDOW: u8 = 7;
/// How long a result stays on screen (frames).
const SHOW_FRAMES: u32 = 120;

const PAD_A: u32 = 0x100;
const PAD_B: u32 = 0x200;
const PAD_X: u32 = 0x400;
const PAD_Y: u32 = 0x800;
const PAD_START: u32 = 0x1000;
const PAD_Z: u32 = 0x10;
const PAD_R: u32 = 0x20;
const PAD_L: u32 = 0x40;

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Opts {
    pub lcancel_indicator: bool,
    pub input_display: bool,
    pub no_screen_shake: bool,
    pub lcancel_helper: bool,
}

impl Opts {
    pub fn from_args(args: &[String]) -> Opts {
        let has = |k: &str| args.iter().any(|a| a == k);
        Opts {
            lcancel_indicator: has("--lcancel-indicator") || crate::cfg::flag("lcancel_indicator", false),
            input_display: has("--input-display") || crate::cfg::flag("input_display", false),
            no_screen_shake: has("--no-screen-shake") || crate::cfg::flag("no_screen_shake", false),
            lcancel_helper: has("--lcancel-helper") || crate::cfg::flag("lcancel_helper", false),
        }
    }
    pub fn overlay(&self) -> bool {
        self.lcancel_indicator || self.input_display || self.lcancel_helper
    }
}

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct LcResult {
    pub success: bool,
    /// Frames from the last trigger press to the landing (`x67F` at the landing; 255 = no recent press).
    pub frames_before: u8,
    /// A miss whose press came during the landing lag: frames after the landing.
    pub late_by: Option<u32>,
}

#[derive(Clone, Copy, Default)]
struct Slot {
    last_motion: i32,
    result: Option<LcResult>,
    age: u32,
    /// Frames since the landing of a missed L-cancel (watching for a late press).
    since_landing: Option<u32>,
    hits: u32,
    total: u32,
}

pub struct Practice {
    pub opts: Opts,
    slots: [Slot; 6],
    keys_prev: [bool; 3],
    helper_phase: bool,
    helper_allowed: bool,
    ui: Ui,
    shim: GxShim,
    /// Every L-cancel result in order (slot, result): the log and the tests read it.
    pub log: Vec<(usize, LcResult)>,
}

/// The fighter of player slot `slot`, if one is in the match.
unsafe fn fighter(slot: i32) -> Option<&'static melee_types::Fighter> {
    let g = melee_fighter::player::get_entity(slot);
    if g.is_null() || (*g).user_data.is_null() {
        return None;
    }
    Some(&*((*g).user_data as *const melee_types::Fighter))
}

fn is_aerial(m: i32) -> bool {
    (ATTACK_AIR_N..=ATTACK_AIR_LW).contains(&m)
}
fn is_landing_air(m: i32) -> bool {
    (LANDING_AIR_N..=LANDING_AIR_LW).contains(&m)
}

impl Practice {
    pub fn new(opts: Opts) -> Practice {
        let p = Practice { opts, slots: [Slot { last_motion: -1, ..Default::default() }; 6], keys_prev: [false; 3], helper_phase: false, helper_allowed: true, ui: Ui::new(), shim: GxShim::new(), log: Vec::new() };
        p.apply();
        if opts != Opts::default() {
            eprintln!("practice: {:?}", opts);
        }
        p
    }

    /// The settings overlay's live change of the presentation options (the helper is not offered there).
    pub fn set_opts(&mut self, o: Opts) {
        self.opts = Opts { lcancel_helper: self.opts.lcancel_helper, ..o };
        self.apply();
    }

    fn apply(&self) {
        melee_world::tu::camera::NO_SCREEN_SHAKE.store(self.opts.no_screen_shake, Ordering::Relaxed);
    }

    /// Netplay, synctest or a replay playback: the helper (the one gameplay change) is refused.
    pub fn forbid_helper(&mut self, why: &str) {
        if self.opts.lcancel_helper && self.helper_allowed {
            eprintln!("practice: lcancel_helper refused ({why}): it changes gameplay, so it is offline only");
        }
        self.helper_allowed = false;
    }

    /// F5 / F6 / F7 toggle the presentation options (keyboard of the focused window; never the helper).
    pub fn keys(&mut self, pads: &melee_platform::PadSystem) {
        use winit::keyboard::KeyCode;
        let down = [KeyCode::F5, KeyCode::F6, KeyCode::F7].map(|k| pads.focused && pads.keys_down.contains(&k));
        for i in 0..3 {
            if down[i] && !self.keys_prev[i] {
                let (name, v) = match i {
                    0 => ("lcancel_indicator", &mut self.opts.lcancel_indicator),
                    1 => ("input_display", &mut self.opts.input_display),
                    _ => ("no_screen_shake", &mut self.opts.no_screen_shake),
                };
                *v = !*v;
                eprintln!("practice: {name} {}", if *v { "on" } else { "off" });
            }
        }
        self.keys_prev = down;
        self.apply();
    }

    /// The helper (gameplay change, opt-in): analog L on alternate frames for every human port whose fighter is in an
    /// aerial in the air. `slot_of_port[i]` is the player slot port `i` drives. Skips frames where A or Z is newly
    /// pressed (held L/R + A is a grab / tether). Analog only: the digital L bit (air dodge, tech) is never set.
    pub fn auto_lcancel(&mut self, pads: &mut [melee_types::PADStatus; 4], slot_of_port: impl Fn(usize) -> Option<i32>) {
        if !self.opts.lcancel_helper || !self.helper_allowed {
            return;
        }
        self.helper_phase = !self.helper_phase;
        if !self.helper_phase {
            return;
        }
        for (port, st) in pads.iter_mut().enumerate() {
            let Some(slot) = slot_of_port(port) else { continue };
            let Some(fp) = (unsafe { fighter(slot) }) else { continue };
            if !is_aerial(fp.motion_id) || fp.ground_or_air != melee_types::GA_Air {
                continue;
            }
            let held = unsafe { melee_runtime::tu::controller::HSD_PadMasterStatus[port].button };
            let new = st.button as u32 & !held;
            if new & (PAD_A | PAD_Z) != 0 {
                continue;
            }
            // 0.3 deadzone after the clamp (min 30, scale 150): 142 reads as 0.75, well short of the digital click
            st.triggerLeft = st.triggerLeft.max(142);
        }
    }

    /// After each engine tick: watch the fighters for aerial landings.
    pub fn observe(&mut self) {
        if !self.opts.lcancel_indicator && !self.opts.lcancel_helper {
            return;
        }
        for (i, s) in self.slots.iter_mut().enumerate() {
            s.age = s.age.saturating_add(1);
            let Some(fp) = (unsafe { fighter(i as i32) }) else {
                s.last_motion = -1;
                continue;
            };
            let m = fp.motion_id;
            if is_landing_air(m) && is_aerial(s.last_motion) {
                let r = LcResult { success: fp.x67F < LC_WINDOW, frames_before: fp.x67F, late_by: None };
                s.result = Some(r);
                s.age = 0;
                s.total += 1;
                s.hits += r.success as u32;
                s.since_landing = (!r.success).then_some(0);
                self.log.push((i, r));
                eprintln!("practice: P{} L-cancel {} ({} f before landing)", i + 1, if r.success { "OK" } else { "MISS" }, fp.x67F);
            } else if let Some(n) = s.since_landing {
                if is_landing_air(m) {
                    let n = n + 1;
                    s.since_landing = Some(n);
                    if fp.x67F == 0 {
                        if let Some(r) = s.result.as_mut() {
                            r.late_by = Some(n);
                        }
                        if let Some(l) = self.log.iter_mut().rev().find(|l| l.0 == i) {
                            l.1.late_by = Some(n);
                        }
                        s.since_landing = None;
                    }
                } else {
                    s.since_landing = None;
                }
            }
            s.last_motion = m;
        }
    }

    /// The overlay's draws (alpha-blended quads in 640x480 EFB coordinates), to append to a finished frame.
    pub fn draws(&mut self) -> Option<Vec<DrawCmd>> {
        if !self.opts.overlay() {
            return None;
        }
        self.ui.clear();
        let mut col = 0usize;
        for port in 0..4 {
            let st = unsafe { melee_runtime::tu::controller::HSD_PadMasterStatus[port] };
            let s = self.slots[port];
            let lc = (self.opts.lcancel_indicator || self.opts.lcancel_helper) && s.total > 0;
            if st.err != 0 && !lc {
                continue;
            }
            let x = 8.0 + col as f32 * 158.0;
            col += 1;
            let mut y = 8.0;
            if self.opts.input_display && st.err == 0 {
                draw_pad(&mut self.ui, x, y, port, &st);
                y += 62.0;
            }
            if lc {
                let r = s.result.unwrap();
                let fade = if s.age < SHOW_FRAMES { 230 } else { 120 };
                let (msg, c) = if r.success {
                    (format!("L-CANCEL OK -{}F", r.frames_before), [80, 230, 110, fade])
                } else if let Some(l) = r.late_by {
                    (format!("L-CANCEL MISS +{l}F"), [240, 80, 60, fade])
                } else if r.frames_before == 255 {
                    ("L-CANCEL MISS".to_string(), [240, 80, 60, fade])
                } else {
                    (format!("L-CANCEL MISS -{}F", r.frames_before), [240, 80, 60, fade])
                };
                self.ui.rect(x, y, 150.0, 24.0, [0, 0, 0, 150]);
                self.ui.text(x + 4.0, y + 3.0, 1.0, &msg, c);
                let pct = s.hits * 100 / s.total.max(1);
                self.ui.text(x + 4.0, y + 13.0, 1.0, &format!("P{} {}/{} {}%{}", port + 1, s.hits, s.total, pct, if self.opts.lcancel_helper { " AUTO" } else { "" }), ui::GREY);
            }
        }
        if self.opts.lcancel_helper {
            self.ui.rect(8.0, 462.0, 214.0, 12.0, [0, 0, 0, 160]);
            self.ui.text(10.0, 464.0, 1.0, "AUTO L-CANCEL (GAMEPLAY MOD)", ui::YELLOW);
        }
        if self.ui.len() == 0 {
            return None;
        }
        self.ui.flush(&mut self.shim);
        self.shim.copy_disp(true);
        self.shim.take_frame()
    }

    /// Append the overlay to `frame` (a no-op with every option off).
    pub fn decorate(&mut self, frame: &mut Vec<DrawCmd>) -> bool {
        match self.draws() {
            Some(d) => {
                frame.extend(d);
                true
            }
            None => false,
        }
    }
}

fn draw_pad(u: &mut Ui, x: f32, y: f32, port: usize, st: &melee_types::HSD_PadStatus) {
    const PORT_COLS: [ui::Rgba; 4] = [ui::RED, ui::CYAN, ui::YELLOW, ui::GREEN];
    u.rect(x, y, 150.0, 58.0, [0, 0, 0, 150]);
    u.frame(x, y, 150.0, 58.0, 1.0, PORT_COLS[port]);
    u.text(x + 3.0, y + 3.0, 1.0, &format!("P{}", port + 1), PORT_COLS[port]);
    // main stick: a 40 px box, the raw value (-128..127) mapped onto it
    let stick = |u: &mut Ui, bx: f32, by: f32, size: f32, sx: i8, sy: i8, c: ui::Rgba| {
        u.rect(bx, by, size, size, [40, 40, 50, 200]);
        u.rect(bx + size / 2.0 - 0.5, by, 1.0, size, [80, 80, 90, 200]);
        u.rect(bx, by + size / 2.0 - 0.5, size, 1.0, [80, 80, 90, 200]);
        let px = bx + size / 2.0 + sx as f32 / 128.0 * (size / 2.0);
        let py = by + size / 2.0 - sy as f32 / 128.0 * (size / 2.0);
        u.rect(px - 3.0, py - 3.0, 6.0, 6.0, c);
    };
    stick(u, x + 20.0, y + 12.0, 40.0, st.stickX, st.stickY, ui::WHITE);
    stick(u, x + 64.0, y + 20.0, 30.0, st.subStickX, st.subStickY, ui::YELLOW);
    // analog triggers: vertical bars, full when the digital click is down
    let trig = |u: &mut Ui, bx: f32, v: u8, dig: bool| {
        u.rect(bx, y + 12.0, 8.0, 40.0, [40, 40, 50, 200]);
        let h = if dig { 40.0 } else { v as f32 / 255.0 * 40.0 };
        u.rect(bx, y + 52.0 - h, 8.0, h, if dig { ui::WHITE } else { [170, 170, 190, 255] });
    };
    trig(u, x + 4.0, st.analogL, st.button & PAD_L != 0);
    trig(u, x + 98.0, st.analogR, st.button & PAD_R != 0);
    // buttons
    let b = |u: &mut Ui, bx: f32, by: f32, w: f32, h: f32, on: bool, c: ui::Rgba, l: &str| {
        u.rect(bx, by, w, h, if on { c } else { [50, 50, 60, 220] });
        u.text(bx + (w - 8.0) / 2.0, by + (h - 8.0) / 2.0, 1.0, l, if on { [0, 0, 0, 255] } else { ui::DIM });
    };
    let bt = st.button;
    b(u, x + 112.0, y + 30.0, 14.0, 14.0, bt & PAD_A != 0, [60, 220, 140, 255], "A");
    b(u, x + 112.0, y + 46.0, 12.0, 10.0, bt & PAD_B != 0, [235, 70, 60, 255], "B");
    b(u, x + 130.0, y + 30.0, 14.0, 14.0, bt & PAD_X != 0, ui::WHITE, "X");
    b(u, x + 112.0, y + 14.0, 14.0, 14.0, bt & PAD_Y != 0, ui::WHITE, "Y");
    b(u, x + 130.0, y + 14.0, 14.0, 14.0, bt & PAD_Z != 0, [150, 90, 230, 255], "Z");
    b(u, x + 130.0, y + 46.0, 14.0, 10.0, bt & PAD_START != 0, ui::GREY, "S");
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn defaults_off() {
        let o = Opts { ..Default::default() };
        assert!(!o.overlay());
        let a: Vec<String> = ["--input-display", "--no-screen-shake"].iter().map(|s| s.to_string()).collect();
        let o = Opts::from_args(&a);
        assert!(o.input_display && o.no_screen_shake && !o.lcancel_helper);
    }
}
