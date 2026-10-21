//! Dolphin SDK PAD: `PADStatus`, its constants, and `PADClamp` / `PADClampCircle`.
//!
//! `PADStatus` is `melee_types::PADStatus` (generated from `libs/dolphin/include/dolphin/pad.h`
//! with `-DTARGET_PC`, GC size 0x10 incl. `extButton`, with its `GcLayout` table). Native code
//! here stores it little-endian and by value: the C shim's `gw_w16`/`bswap16` of `button` exists
//! only because the C game's memory is big-endian, and has no counterpart in the native port.
//!
//! Clamp functions are ports of Aurora's (`extern/aurora/lib/dolphin/pad/pad.cpp` ClampTrigger,
//! ClampStick, ClampCircle, PADClamp, PADClampCircle) with its `ClampRegion` constants; those are
//! what the C build links (the shim forwards `gw_PADClamp` to Aurora).

pub use melee_types::PADStatus;

pub const PAD_CHANMAX: usize = 4;

pub const PAD_ERR_NONE: i8 = 0;
pub const PAD_ERR_NO_CONTROLLER: i8 = -1;
pub const PAD_ERR_NOT_READY: i8 = -2;
pub const PAD_ERR_TRANSFER: i8 = -3;

pub const PAD_BUTTON_LEFT: u16 = 0x0001;
pub const PAD_BUTTON_RIGHT: u16 = 0x0002;
pub const PAD_BUTTON_DOWN: u16 = 0x0004;
pub const PAD_BUTTON_UP: u16 = 0x0008;
pub const PAD_TRIGGER_Z: u16 = 0x0010;
pub const PAD_TRIGGER_R: u16 = 0x0020;
pub const PAD_TRIGGER_L: u16 = 0x0040;
pub const PAD_BUTTON_A: u16 = 0x0100;
pub const PAD_BUTTON_B: u16 = 0x0200;
pub const PAD_BUTTON_X: u16 = 0x0400;
pub const PAD_BUTTON_Y: u16 = 0x0800;
pub const PAD_BUTTON_START: u16 = 0x1000;
pub const PAD_BUTTON_MENU: u16 = 0x1000;

pub const PAD_CHAN0_BIT: u32 = 0x8000_0000;

pub const PAD_MOTOR_STOP: u32 = 0;
pub const PAD_MOTOR_RUMBLE: u32 = 1;
pub const PAD_MOTOR_STOP_HARD: u32 = 2;

/// `PADButtonDown(last, now)`.
#[inline]
pub const fn pad_button_down(last: u16, now: u16) -> u16 {
    (last ^ now) & now
}
/// `PADButtonUp(last, now)`.
#[inline]
pub const fn pad_button_up(last: u16, now: u16) -> u16 {
    (last ^ now) & last
}

/// A zeroed status (`memset(&st, 0, sizeof st)`): connected (err 0), neutral.
#[inline]
pub fn pad_zero() -> PADStatus {
    PADStatus::zeroed()
}

/// A zeroed status with `err = PAD_ERR_NO_CONTROLLER`.
#[inline]
pub fn pad_disconnected() -> PADStatus {
    let mut s = PADStatus::zeroed();
    s.err = PAD_ERR_NO_CONTROLLER;
    s
}

/// Field-by-field equality (PADStatus is generated without PartialEq).
pub fn pad_eq(a: &PADStatus, b: &PADStatus) -> bool {
    a.button == b.button
        && a.stickX == b.stickX
        && a.stickY == b.stickY
        && a.substickX == b.substickX
        && a.substickY == b.substickY
        && a.triggerLeft == b.triggerLeft
        && a.triggerRight == b.triggerRight
        && a.analogA == b.analogA
        && a.analogB == b.analogB
        && a.err == b.err
        && a.extButton == b.extButton
}

/// Aurora's `ClampRegion`.
pub mod clamp_region {
    pub const MIN_TRIGGER: u8 = 30;
    pub const MAX_TRIGGER: u8 = 180;
    pub const MIN_STICK: i8 = 15;
    pub const MAX_STICK: i8 = 72;
    pub const XY_STICK: i8 = 40;
    pub const MIN_SUBSTICK: i8 = 15;
    pub const MAX_SUBSTICK: i8 = 59;
    pub const XY_SUBSTICK: i8 = 31;
    pub const RAD_STICK: i8 = 56;
    pub const RAD_SUBSTICK: i8 = 44;
}

/// Aurora `ClampTrigger`.
pub fn clamp_trigger(t: &mut u8, min: u8, max: u8) {
    if *t <= min {
        *t = 0;
    } else {
        if *t > max {
            *t = max;
        }
        *t -= min;
    }
}

/// Aurora `ClampCircle`. `std::sqrt(int)` is the double sqrt, truncated to int32.
pub fn clamp_circle(px: &mut i8, py: &mut i8, radius: i8, min: i8) {
    let (radius, min) = (radius as i32, min as i32);
    let mut x = *px as i32;
    let mut y = *py as i32;
    if -min < x && x < min {
        x = 0;
    } else if 0 < x {
        x -= min;
    } else {
        x += min;
    }
    if -min < y && y < min {
        y = 0;
    } else if 0 < y {
        y -= min;
    } else {
        y += min;
    }
    let squared = x * x + y * y;
    if radius * radius < squared {
        let length = (squared as f64).sqrt() as i32;
        x = x * radius / length;
        y = y * radius / length;
    }
    *px = x as i8;
    *py = y as i8;
}

/// Aurora `ClampStick`.
pub fn clamp_stick(px: &mut i8, py: &mut i8, max: i8, xy: i8, min: i8) {
    let (max, xy, min) = (max as i32, xy as i32, min as i32);
    let mut x = *px as i32;
    let mut y = *py as i32;
    let sign_x = if 0 <= x { 1 } else { x = -x; -1 };
    let sign_y = if 0 <= y { 1 } else { y = -y; -1 };
    x = if x <= min { 0 } else { x - min };
    y = if y <= min { 0 } else { y - min };
    if x == 0 && y == 0 {
        *px = 0;
        *py = 0;
        return;
    }
    x = x * max / (i8::MAX as i32 - min);
    y = y * max / (i8::MAX as i32 - min);
    if xy * y <= xy * x {
        let d = xy * x + (max - xy) * y;
        if xy * max < d {
            x = xy * max * x / d;
            y = xy * max * y / d;
        }
    } else {
        let d = xy * y + (max - xy) * x;
        if xy * max < d {
            x = xy * max * x / d;
            y = xy * max * y / d;
        }
    }
    *px = (sign_x * x) as i8;
    *py = (sign_y * y) as i8;
}

/// `PADClamp`: every connected channel (err == 0).
pub fn pad_clamp(status: &mut [PADStatus; PAD_CHANMAX]) {
    use clamp_region::*;
    for s in status.iter_mut() {
        if s.err != PAD_ERR_NONE {
            continue;
        }
        clamp_stick(&mut s.stickX, &mut s.stickY, MAX_STICK, XY_STICK, MIN_STICK);
        clamp_stick(&mut s.substickX, &mut s.substickY, MAX_SUBSTICK, XY_SUBSTICK, MIN_SUBSTICK);
        clamp_trigger(&mut s.triggerLeft, MIN_TRIGGER, MAX_TRIGGER);
        clamp_trigger(&mut s.triggerRight, MIN_TRIGGER, MAX_TRIGGER);
    }
}

/// `PADClampCircle`.
pub fn pad_clamp_circle(status: &mut [PADStatus; PAD_CHANMAX]) {
    use clamp_region::*;
    for s in status.iter_mut() {
        if s.err != PAD_ERR_NONE {
            continue;
        }
        clamp_circle(&mut s.stickX, &mut s.stickY, RAD_STICK, MIN_STICK);
        clamp_circle(&mut s.substickX, &mut s.substickY, RAD_SUBSTICK, MIN_SUBSTICK);
        clamp_trigger(&mut s.triggerLeft, MIN_TRIGGER, MAX_TRIGGER);
        clamp_trigger(&mut s.triggerRight, MIN_TRIGGER, MAX_TRIGGER);
    }
}
