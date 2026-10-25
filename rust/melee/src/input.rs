//! Controller input for the native golden run: the scenario's pad script, the HSD pad layer
//! (sysdolphin controller.c) and the fighter's input step (fighter.c
//! `Fighter_Spaghetti_8006AD10`, up to its `input_cb` call), plus the HUD "Ready" timer that
//! unlocks input at GO (if_2F72.c `if_802F73C4` -> gmvs.c `fn_8016B7F8` -> `ftLib_800868A4`).
//!
//! Pad source: `tools/rust-port/golden/pad_fixed.lua`'s cycles (the Lua harness, not game code).
//! Its `on_frame_pre` runs after the frame's PADRead with `gd.input(port, sample, 2)`, so golden
//! frame `g` reads sample `n = g - 1` (n counts from 1); checked against the C build's per-frame
//! `HSD_PadGameStatus` (temporary MELEE_ECB_LOG instrumentation). The PC pad shim hands the raw
//! signed stick bytes through unchanged (no SDK PADClamp: a 100 stick reads 80/80 = 1.0).
//!
//! FMA: HSD_PadClampCheck3 fuses its two `x*x + y*y`; vec2DSqDist its one; lb_8000D148 five
//! (each `a*b + c*d` and `a*b - c` with the left multiply fused). Nothing else here fuses.

use melee_core::fp::fmadds;
use melee_runtime::gekko::sqrtf;
use melee_types::Fighter;

pub const HSD_PAD_A: u32 = 0x100;
pub const HSD_PAD_B: u32 = 0x200;
pub const HSD_PAD_X: u32 = 0x400;
pub const HSD_PAD_Y: u32 = 0x800;
pub const HSD_PAD_XY: u32 = HSD_PAD_X | HSD_PAD_Y;
pub const HSD_PAD_START: u32 = 0x1000;
pub const HSD_PAD_Z: u32 = 0x10;
pub const HSD_PAD_R: u32 = 0x20;
pub const HSD_PAD_L: u32 = 0x40;
pub const HSD_PAD_DPADUP: u32 = 0x8;
pub const HSD_PAD_DPADDOWN: u32 = 0x4;
pub const HSD_PAD_LR: u32 = 0x8000_0000;

/// One raw controller sample (what PADRead hands HSD: buttons and signed stick bytes).
#[derive(Clone, Copy, Debug, Default)]
pub struct RawPad {
    pub button: u32,
    pub stick_x: i8,
    pub stick_y: i8,
    pub sub_x: i8,
    pub sub_y: i8,
    pub analog_l: u8,
    pub analog_r: u8,
}

/// pad_fixed.lua's two cycles: (frames, buttons, x, y).
const CYCLE1: &[(u32, u32, i8, i8)] = &[
    (40, 0, 100, 0), (3, HSD_PAD_X, 100, 0), (30, 0, 100, 0), (4, HSD_PAD_A, 0, 0), (25, 0, -100, 0),
    (6, HSD_PAD_B, 0, -127), (50, 0, 0, 0), (3, HSD_PAD_X, -100, 0), (20, HSD_PAD_A, -60, 0), (60, 0, 0, 0),
    (8, HSD_PAD_R, 0, 0), (30, 0, 0, 127), (5, HSD_PAD_A, 100, 0),
];
const CYCLE2: &[(u32, u32, i8, i8)] = &[
    (30, 0, -100, 0), (4, HSD_PAD_X, 0, 0), (20, HSD_PAD_B, 0, 0), (50, 0, 80, 0), (5, HSD_PAD_A, 0, 0),
    (40, 0, 0, 0), (6, HSD_PAD_Z, 0, 0), (60, 0, -50, 0), (12, HSD_PAD_L, 0, 0), (7, HSD_PAD_B, 0, 127),
];

fn at(cycle: &[(u32, u32, i8, i8)], frame: u32) -> RawPad {
    let total: u32 = cycle.iter().map(|p| p.0).sum();
    let mut f = frame % total;
    for &(n, b, x, y) in cycle {
        if f < n {
            return RawPad { button: b, stick_x: x, stick_y: y, ..Default::default() };
        }
        f -= n;
    }
    unreachable!()
}

/// The raw sample port `port` (0-based) reads on golden frame `g` (pad_fixed.lua, 1800 frames).
pub fn pad_fixed(port: usize, g: i32) -> RawPad {
    let n = g - 1;
    if !(1..=1800).contains(&n) {
        return RawPad::default();
    }
    match port {
        0 => at(CYCLE1, n as u32),
        1 => at(CYCLE2, n as u32),
        _ => RawPad::default(),
    }
}

/// `HSD_PadStatus` (the fields the game reads) for one port.
#[derive(Clone, Copy, Debug, Default)]
pub struct PadStatus {
    pub button: u32,
    pub last_button: u32,
    pub trigger: u32,
    pub release: u32,
    pub stick_x: i8,
    pub stick_y: i8,
    pub sub_x: i8,
    pub sub_y: i8,
    pub analog_l: u8,
    pub analog_r: u8,
    pub nml_stick_x: f32,
    pub nml_stick_y: f32,
    pub nml_sub_x: f32,
    pub nml_sub_y: f32,
    pub nml_analog_l: f32,
    pub nml_analog_r: f32,
}

/// `HSD_PadLibData` after `gmMain_8015FD24` (default_libinfo_data + the game's clamp/scale).
const CLAMP_STICK_SHIFT: u8 = 1;
const CLAMP_STICK_MAX: i8 = 80;
const CLAMP_STICK_MIN: i8 = 0;
const SCALE_STICK: i8 = 80;
const CLAMP_LR_SHIFT: u8 = 1;
const CLAMP_LR_MAX: u8 = 140;
const CLAMP_LR_MIN: u8 = 0;
const SCALE_LR: u8 = 140;
const ADC_TH: i8 = 0x1E;
const ADC_ANGLE: f32 = 0.0;

/// C `(s8) f` through gwtool's float->int rule (saturate to i32, truncate).
fn f2s8(f: f32) -> i8 {
    melee_runtime::cabi::f2i8(f as f64)
}

/// `HSD_PadClampCheck1`.
fn clamp_check1(val: &mut u8, shift: u8, min: u8, max: u8) {
    if *val < min {
        *val = 0;
        return;
    }
    if *val > max {
        *val = max;
    }
    if shift != 1 {
        return;
    }
    *val -= min;
}

/// `HSD_PadClampCheck3` (fused `x*x + y*y`, MSL sqrtf).
fn clamp_check3(x: &mut i8, y: &mut i8, shift: u8, min: i8, max: i8) {
    let (fx, fy) = (*x as f32, *y as f32);
    let mut r = sqrtf(fmadds(fx, fx, fy * fy));
    if r < min as f32 {
        *y = 0;
        *x = 0;
        return;
    }
    if r > max as f32 {
        *x = f2s8((*x as f32 * max as f32) / r);
        *y = f2s8((*y as f32 * max as f32) / r);
        let (fx, fy) = (*x as f32, *y as f32);
        r = sqrtf(fmadds(fx, fx, fy * fy));
    }
    if shift == 1 && r > 1.000000013351432e-10f32 {
        *x = f2s8(*x as f32 - ((*x as f32 * min as f32) / r));
        *y = f2s8(*y as f32 - ((*y as f32 * min as f32) / r));
    }
}

/// `HSD_PadADConvertCheck1`: the stick as digital direction bits.
fn adconvert_check1(button: &mut u32, x: i8, y: i8, up: u32, down: u32, left: u32, right: u32) {
    let (fx, fy) = (x as f32, y as f32);
    // vec2Dlen: sqrtf(vec2DSqDist) with its one fused site.
    let r = sqrtf(fmadds(fx, fx, fy * fy));
    let a: f32 = if (fx as f64).abs() == 0.0 {
        if y >= 0 { 1.5707963267948966f64 as f32 } else { -1.5707963267948966f64 as f32 }
    } else {
        melee_runtime::lbtrigf::atan2f(fy, fx)
    };
    let ha = 0.5f32 * ADC_ANGLE;
    if r < ADC_TH as f32 {
        return;
    }
    let (a, ha) = (a as f64, ha as f64);
    if a < -2.356194490192345 + ha {
        *button |= left;
    }
    if a >= -2.356194490192345 - ha && a <= -0.7853981633974483 + ha {
        *button |= down;
    }
    if a > -0.7853981633974483 - ha && a < 0.7853981633974483 + ha {
        *button |= right;
    }
    if a >= 0.7853981633974483 - ha && a <= 2.356194490192345 + ha {
        *button |= up;
    }
    if a > 2.356194490192345 - ha {
        *button |= left;
    }
}

impl PadStatus {
    /// `HSD_PadRenewMasterStatus` for one sample (err 0): clamp, AD convert, scale, cross dir
    /// (cross_dir 0), then the edge bits. Game status is a copy of master (HSD_PadRenewGameStatus).
    pub fn renew(&mut self, raw: &RawPad) {
        self.last_button = self.button;
        self.button = raw.button;
        self.stick_x = raw.stick_x;
        self.stick_y = raw.stick_y;
        self.sub_x = raw.sub_x;
        self.sub_y = raw.sub_y;
        self.analog_l = raw.analog_l;
        self.analog_r = raw.analog_r;
        // HSD_PadClamp (clamp_stickType 0; the A/B analog lines touch nothing the game reads).
        clamp_check3(&mut self.stick_x, &mut self.stick_y, CLAMP_STICK_SHIFT, CLAMP_STICK_MIN, CLAMP_STICK_MAX);
        clamp_check3(&mut self.sub_x, &mut self.sub_y, CLAMP_STICK_SHIFT, CLAMP_STICK_MIN, CLAMP_STICK_MAX);
        clamp_check1(&mut self.analog_l, CLAMP_LR_SHIFT, CLAMP_LR_MIN, CLAMP_LR_MAX);
        clamp_check1(&mut self.analog_r, CLAMP_LR_SHIFT, CLAMP_LR_MIN, CLAMP_LR_MAX);
        // HSD_PadADConvert (adc_type 0).
        let mut b = self.button;
        adconvert_check1(&mut b, self.stick_x, self.stick_y, 0x10000, 0x20000, 0x40000, 0x80000);
        adconvert_check1(&mut b, self.sub_x, self.sub_y, 0x100000, 0x200000, 0x400000, 0x800000);
        self.button = b;
        // HSD_PadScale.
        self.nml_stick_x = self.stick_x as f32 / SCALE_STICK as f32;
        self.nml_stick_y = self.stick_y as f32 / SCALE_STICK as f32;
        self.nml_sub_x = self.sub_x as f32 / SCALE_STICK as f32;
        self.nml_sub_y = self.sub_y as f32 / SCALE_STICK as f32;
        self.nml_analog_l = self.analog_l as f32 / SCALE_LR as f32;
        self.nml_analog_r = self.analog_r as f32 / SCALE_LR as f32;
        self.trigger = self.button & (self.last_button ^ self.button);
        self.release = self.last_button & (self.last_button ^ self.button);
    }
}

/// `lb_8000D148`: did the stick move from point0 to point1 across the circle of radius
/// `threshold` around point2 (five fused sites)?
pub fn lb_8000d148(p0x: f32, p0y: f32, p1x: f32, p1y: f32, p2x: f32, p2y: f32, threshold: f32) -> bool {
    let dx = p1x - p0x;
    let dy = p0y - p1y;
    let cross = p0y * p1x;
    let dist_01 = fmadds(dx, dx, dy * dy);
    let cross = fmadds(p0x, p1y, -cross);
    if dist_01 < 0.00001 {
        return false;
    }
    let dist_01 = sqrtf(dist_01);
    let mut var_f0 = cross + fmadds(dx, p2x, dy * p2y);
    if var_f0 < 0.0 {
        var_f0 = -var_f0;
    }
    if var_f0 / dist_01 <= threshold {
        let a = p0x - p2x;
        let b = p0y - p2y;
        let d02 = fmadds(a, a, b * b);
        let ts = threshold * threshold;
        let c = p1x - p2x;
        let e = p1y - p2y;
        let d12 = fmadds(c, c, e * e);
        if d02 < ts {
            if d12 > ts {
                return true;
            }
            if d12 < ts {
                return false;
            }
            return true;
        }
        if d02 > ts {
            if d12 > ts {
                return (p0x > p2x && p1x < p2x) || (p0x < p2x && p1x > p2x) || (p0y > p2y && p1y < p2y) || (p0y < p2y && p1y > p2y);
            }
            return true;
        }
        return true;
    }
    false
}

/// `Fighter_Spaghetti_8006AD10_Inner1`: pressed / released edges (not in hitlag here).
fn inner1(fp: &mut Fighter) {
    let h = fp.input.held_buttons;
    let pressed = h[0] & (h[1] ^ h[0]);
    let released = h[1] & (h[1] ^ h[0]);
    if fp.x2219_b5() != 0 {
        fp.input.pressed_buttons |= pressed;
        fp.input.released_buttons |= released;
    } else {
        fp.input.pressed_buttons = pressed;
        fp.input.released_buttons = released;
    }
}

/// `Fighter_UnkInitLoad_80068914_Inner1`: clear the input and the timers (input locked).
pub fn clear_input(fp: &mut Fighter) {
    let i = &mut fp.input;
    i.triggers[0] = 0.0;
    i.triggers[1] = 0.0;
    i.cstick[0].x = 0.0;
    i.cstick[0].y = 0.0;
    i.cstick[1].x = 0.0;
    i.cstick[1].y = 0.0;
    i.lstick[0].x = 0.0;
    i.lstick[0].y = 0.0;
    i.lstick[1].x = 0.0;
    i.lstick[1].y = 0.0;
    i.held_buttons[1] = 0;
    i.released_buttons = 0;
    i.pressed_buttons = 0;
    i.held_buttons[0] = 0;
    for t in [&mut fp.activity_timer, &mut fp.active_duration, &mut fp.active_sticky, &mut fp.active_timer] {
        t.lstick.x = 254;
        t.lstick.y = 254;
        t.trigger = 254;
    }
    for v in [
        &mut fp.x67C, &mut fp.x67D, &mut fp.x67E, &mut fp.x681, &mut fp.x682, &mut fp.x67F, &mut fp.x680, &mut fp.x683,
        &mut fp.x684, &mut fp.x685, &mut fp.x686, &mut fp.x687, &mut fp.x688, &mut fp.x689, &mut fp.x68A, &mut fp.x68B,
    ] {
        *v = 255;
    }
}

/// `Fighter_ResetInputData_80068854` (sets the same as `clear_input` here).
pub fn reset_input(fp: &mut Fighter) {
    clear_input(fp);
}

#[inline]
fn abs(x: f32) -> f32 {
    if x < 0.0 {
        -x
    } else {
        x
    }
}

#[inline]
fn inc254(v: &mut u8) {
    *v = v.wrapping_add(1);
    if *v > 254 {
        *v = 254;
    }
}

/// The input half of `Fighter_Spaghetti_8006AD10` for a human fighter (x221F_b3 and x2224_b2
/// clear): the stick/trigger/button history, the pad read with its deadzones, the edges, the
/// smash/hold timers and the press counters; then the input lock (`x221D_b4`) copy-and-clear.
/// `input_locked` is `x221D_b4`; `x221d_b3` the "history primed" bit.
pub fn fighter_input(fp: &mut Fighter, pad: &PadStatus, x221d_b3: &mut bool, input_locked: bool, cpu: bool, replay: Option<&melee_net::pad::RbInput>) {
    let cd = melee_fighter::hooks::common_data();
    if !*x221d_b3 {
        fp.input.lstick[1] = fp.input.lstick[2];
        fp.input.cstick[1] = fp.input.cstick[2];
        fp.input.triggers[1] = fp.input.triggers[2];
        fp.input.held_buttons[1] = fp.input.held_buttons[2];
        *x221d_b3 = true;
    } else {
        fp.input.lstick[1] = fp.input.lstick[0];
        fp.input.cstick[1] = fp.input.cstick[0];
        fp.input.triggers[1] = fp.input.triggers[0];
        fp.input.held_buttons[1] = fp.input.held_buttons[0];
    }
    if cpu {
        // ftCo_IsCpuControlled: the CPU's virtual pad (Fighter_Spaghetti_8006AD10).
        use melee_fighter::tu::ftCo_0A01 as c;
        fp.input.lstick[0].x = c::ftCo_GetCpuLStickX(fp);
        fp.input.lstick[0].y = c::ftCo_GetCpuLStickY(fp);
        fp.input.cstick[0].x = c::ftCo_GetCpuCStickX(fp);
        fp.input.cstick[0].y = c::ftCo_GetCpuCStickY(fp);
        let (l, r) = (c::ftCo_GetCpuLTrigger(fp), c::ftCo_GetCpuRTrigger(fp));
        fp.input.triggers[0] = if l > r { l } else { r };
    } else {
    // Human: HSD_PadGameStatus (DbLevel < DebugRom and not 1P: the C-stick is routed).
    fp.input.lstick[0].x = pad.nml_stick_x;
    fp.input.lstick[0].y = pad.nml_stick_y;
    fp.input.cstick[0].x = pad.nml_sub_x;
    fp.input.cstick[0].y = pad.nml_sub_y;
    let (t1, t0) = (pad.nml_analog_r, pad.nml_analog_l);
    fp.input.triggers[0] = if t0 > t1 { t0 } else { t1 };
    }
    if abs(fp.input.lstick[0].x) <= cd.horizontal_stick_deadzone {
        fp.input.lstick[0].x = 0.0;
    }
    if abs(fp.input.lstick[0].y) <= cd.vertical_stick_deadzone {
        fp.input.lstick[0].y = 0.0;
    }
    if abs(fp.input.cstick[0].x) <= cd.horizontal_stick_deadzone {
        fp.input.cstick[0].x = 0.0;
    }
    if abs(fp.input.cstick[0].y) <= cd.vertical_stick_deadzone {
        fp.input.cstick[0].y = 0.0;
    }
    if fp.input.triggers[0] <= cd.analog_shoulder_deadzone {
        fp.input.triggers[0] = 0.0;
    }
    fp.input.held_buttons[0] = if cpu { melee_fighter::tu::ftCo_0A01::ftCo_GetCpuButtons(fp) } else { pad.button };
    // gm_8016B0FC (single-button mode) is off in a plain VS match.
    if fp.input.held_buttons[0] & (HSD_PAD_L | HSD_PAD_R) != 0 {
        fp.input.held_buttons[0] |= HSD_PAD_LR;
        fp.input.triggers[0] = 1.0;
    } else if fp.input.triggers[0] != 0.0 {
        fp.input.held_buttons[0] |= HSD_PAD_LR;
    }
    // gm_GetDbPauseFlag(0) is clear.
    if fp.input.held_buttons[0] & HSD_PAD_Z != 0 {
        fp.input.held_buttons[0] |= HSD_PAD_LR | HSD_PAD_A;
        fp.input.triggers[0] = cd.z_press_analog_value;
    }
    // MELEE_SLP playback (fighter.c, TARGET_PC): this frame's recorded inputs replace what the pad
    // gave, at the point Slippi's RestoreGameFrame.asm writes them (fp+0x620..0x65C).
    if let Some(r) = replay {
        fp.input.lstick[0].x = r.lx;
        fp.input.lstick[0].y = r.ly;
        fp.input.cstick[0].x = r.cx;
        fp.input.cstick[0].y = r.cy;
        fp.input.triggers[0] = r.trigger;
        fp.input.held_buttons[0] = r.buttons;
    }
    inner1(fp);
    // The stick smash timers (Fighter_ClampSpecificValue / Fighter_ClampThreeValues).
    let (hs, vs) = (cd.horizontal_stick_smash_deadzone, cd.vertical_stick_smash_deadzone);
    inc254(&mut fp.active_duration.lstick.x);
    let (x0, x1) = (fp.input.lstick[0].x, fp.input.lstick[1].x);
    if x0 >= hs {
        if x1 >= hs {
            inc254(&mut fp.active_timer.lstick.x);
            inc254(&mut fp.active_sticky.lstick.x);
            inc254(&mut fp.activity_timer.lstick.x);
        } else {
            fp.active_duration.lstick.x = 0;
            fp.active_sticky.lstick.x = 0;
            fp.active_timer.lstick.x = 0;
            fp.set_x2228_b7(1);
        }
    } else if x0 <= -hs {
        if x1 <= -hs {
            inc254(&mut fp.active_timer.lstick.x);
            inc254(&mut fp.active_sticky.lstick.x);
            inc254(&mut fp.activity_timer.lstick.x);
        } else {
            fp.active_duration.lstick.x = 0;
            fp.active_sticky.lstick.x = 0;
            fp.active_timer.lstick.x = 0;
            fp.set_x2228_b7(0);
        }
    } else {
        fp.activity_timer.lstick.x = 254;
        fp.active_sticky.lstick.x = 254;
        fp.active_timer.lstick.x = 254;
    }
    inc254(&mut fp.active_duration.lstick.y);
    let (y0, y1) = (fp.input.lstick[0].y, fp.input.lstick[1].y);
    if y0 >= vs {
        if y1 >= vs {
            inc254(&mut fp.active_timer.lstick.y);
            inc254(&mut fp.active_sticky.lstick.y);
            inc254(&mut fp.activity_timer.lstick.y);
        } else {
            fp.active_duration.lstick.y = 0;
            fp.active_sticky.lstick.y = 0;
            fp.active_timer.lstick.y = 0;
            fp.set_x2229_b0(0);
        }
    } else if y0 <= -vs {
        if y1 <= -vs {
            inc254(&mut fp.active_timer.lstick.y);
            inc254(&mut fp.active_sticky.lstick.y);
            inc254(&mut fp.activity_timer.lstick.y);
        } else {
            fp.active_duration.lstick.y = 0;
            fp.active_sticky.lstick.y = 0;
            fp.active_timer.lstick.y = 0;
            fp.set_x2229_b0(1);
        }
    } else {
        fp.activity_timer.lstick.y = 254;
        fp.active_sticky.lstick.y = 254;
        fp.active_timer.lstick.y = 254;
    }
    if lb_8000d148(fp.input.lstick[1].x, fp.input.lstick[1].y, fp.input.lstick[0].x, fp.input.lstick[0].y, 0.0, 0.0, hs) {
        fp.activity_timer.lstick.y = 0;
        fp.activity_timer.lstick.x = 0;
    }
    // TARGET_PC UCF cardinals: only for MELEE_SLP playback (Replay_UcfCardinals is 0 here).
    inc254(&mut fp.active_duration.trigger);
    if fp.input.triggers[0] >= cd.shield_press_threshold {
        if fp.input.triggers[1] >= cd.shield_press_threshold {
            inc254(&mut fp.active_timer.trigger);
            inc254(&mut fp.active_sticky.trigger);
            inc254(&mut fp.activity_timer.trigger);
        } else {
            fp.activity_timer.trigger = 0;
            fp.active_duration.trigger = 0;
            fp.active_sticky.trigger = 0;
            fp.active_timer.trigger = 0;
        }
    } else {
        fp.activity_timer.trigger = 254;
        fp.active_sticky.trigger = 254;
        fp.active_timer.trigger = 254;
    }
    let pr = fp.input.pressed_buttons;
    if pr & HSD_PAD_A != 0 {
        fp.x683 = fp.x67C;
        fp.x67C = 0;
    } else if fp.x67C < 255 {
        fp.x67C += 1;
    }
    let mut cnt = |v: &mut u8, bit: u32| {
        if pr & bit != 0 {
            *v = 0;
        } else if *v < 255 {
            *v += 1;
        }
    };
    cnt(&mut fp.x67D, HSD_PAD_B);
    cnt(&mut fp.x67E, HSD_PAD_XY);
    cnt(&mut fp.x681, HSD_PAD_DPADUP);
    cnt(&mut fp.x682, HSD_PAD_DPADDOWN);
    cnt(&mut fp.x67F, HSD_PAD_LR);
    if pr & (HSD_PAD_L | HSD_PAD_R) != 0 {
        fp.x684 = fp.x680;
        fp.x680 = 0;
    } else if fp.x680 < 255 {
        fp.x680 += 1;
    }
    // x221D_b4 (input locked until GO), x2224_b2, gm_GetDbPauseFlag(2).
    if input_locked {
        fp.input.lstick[2] = fp.input.lstick[0];
        fp.input.cstick[2] = fp.input.cstick[0];
        fp.input.triggers[2] = fp.input.triggers[0];
        fp.input.held_buttons[2] = fp.input.held_buttons[0];
        *x221d_b3 = false;
        clear_input(fp);
    }
}

/// `Fighter_UnkIncrementCounters_8006ABEC` with the input predicates it calls.
pub fn increment_counters(fp: &mut Fighter, can_jump_input: bool) {
    let cd = melee_fighter::hooks::common_data();
    let pr = fp.input.pressed_buttons;
    let (lx, ly) = (fp.input.lstick[0].x, fp.input.lstick[0].y);
    let mut tick = |v: &mut u8, hit: bool| {
        if hit {
            *v = 0;
        } else if *v < 255 {
            *v += 1;
        }
    };
    // ftCo_Jump_GetInput (computed by the caller).
    if can_jump_input {
        fp.x68A = fp.x685;
        fp.x685 = 0;
    } else if fp.x685 < 255 {
        fp.x685 += 1;
    }
    // ftCo_800D6928: B with the stick up.
    let up_b = pr & HSD_PAD_B != 0 && ly >= cd.x21C;
    if up_b {
        fp.x68B = fp.x686;
        fp.x686 = 0;
    } else if fp.x686 < 255 {
        fp.x686 += 1;
    }
    // ftCo_800D688C: B with the stick down.
    tick(&mut fp.x687, pr & HSD_PAD_B != 0 && ly < -cd.x21C);
    // ftCo_SpecialS_HasInput: B with the stick sideways.
    tick(&mut fp.x688, pr & HSD_PAD_B != 0 && abs(lx) >= cd.x218);
    // ftCo_800D67C4: B with the stick neutral.
    tick(&mut fp.x689, pr & HSD_PAD_B != 0 && abs(lx) < cd.x218 && abs(ly) < cd.x21C);
}

/// The HUD "Ready" element (IfAll `ScInfCnt_scene_models[1]`, anims[0]): the frame its JObj
/// animations all end (`lb_8000B09C` false), which runs the end callback `fn_8016B7F8` (GO:
/// `ftLib_800868A4` unlocks every fighter's input). None: some track loops (never ends).
pub fn ready_hud_end_frame(ifall: &[u8]) -> Result<Option<f32>, String> {
    ready_hud_end_frame_of(ifall, 1)
}

/// `ScInfCnt_scene_models[idx]`'s end frame (see `ready_hud_end_frame`).
pub fn ready_hud_end_frame_of(ifall: &[u8], idx: u32) -> Result<Option<f32>, String> {
    let a = melee_runtime::dat::DatArchive::parse(ifall).map_err(|e| format!("IfAll: {e}"))?;
    let d = &a.data;
    let be32 = |at: u32| u32::from_be_bytes(d[at as usize..at as usize + 4].try_into().unwrap());
    let models = a.public_offset("ScInfCnt_scene_models").ok_or("IfAll: no ScInfCnt_scene_models")?;
    let m1 = be32(models + 4 * idx);
    let (joint, anims) = (be32(m1), be32(m1 + 4));
    if anims == 0 {
        return Ok(Some(0.0));
    }
    let aj = be32(anims);
    // HSD_JObjAddAnimAll pairing: child with child, next with next (no INSTANCE here).
    let mut end = 0.0f32;
    let mut stack = vec![(joint, aj)];
    while let Some((j, aj)) = stack.pop() {
        if j == 0 {
            continue;
        }
        if aj != 0 {
            let ad = be32(aj + 8);
            if ad != 0 {
                let flags = be32(ad);
                if flags & melee_runtime::aobj::AOBJ_LOOP != 0 {
                    return Ok(None);
                }
                let e = f32::from_bits(be32(ad + 4));
                if e > end {
                    end = e;
                }
            }
        }
        let (jc, jn) = (be32(j + 8), be32(j + 0xC));
        let (ac, an) = if aj != 0 { (be32(aj), be32(aj + 4)) } else { (0, 0) };
        stack.push((jn, an));
        stack.push((jc, ac));
    }
    Ok(Some(end))
}
