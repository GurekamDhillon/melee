//! Controller input of the scripted scenarios: the golden pad script (`pad_fixed`) and the HSD pad layer
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
