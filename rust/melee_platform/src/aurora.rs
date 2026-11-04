//! The layer under the C shim's `gw_PADRead`: Aurora's `PADRead`
//! (`extern/aurora/lib/dolphin/pad/pad.cpp`), which builds each port from its keyboard bindings
//! and its SDL gamepad. Here the gamepad comes from gilrs; the per-port arithmetic is Aurora's,
//! applied to SDL-convention raw values ([`RawGamepad`]: i16 axes, SDL's +Y = down, triggers
//! 0..32767), with Aurora's default "standard" button map, default axis map and default dead
//! zones (`lib/input.hpp`: stick/substick 8000, trigger activation 31150, emulateTriggers on).
//!
//! Not ported: Aurora's per-controller mapping files, the per-type default maps (Xbox/PS/Switch
//! tables differ from "standard" only in which native button is which), virtual pads,
//! `g_blockPAD` / held-input suppression (both inactive in the C build's normal flow), LEDs.
use crate::pad::*;
use std::collections::HashSet;
use winit::keyboard::KeyCode;

/// SDL gamepad buttons this layer reads (the subset Aurora's standard map + ext map name).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NativeButton {
    South,
    East,
    West,
    North,
    Start,
    Back,
    Guide,
    LeftShoulder,
    RightShoulder,
    LeftStick,
    RightStick,
    DpadUp,
    DpadDown,
    DpadLeft,
    DpadRight,
    /// Digital ZL / ZR / L2 / R2 (pads whose triggers are buttons, and the trigger click of analog ones).
    LeftTrigger2,
    RightTrigger2,
    /// Extra face buttons some HID pads and boxes report (gilrs `C` / `Z`).
    C,
    Z,
}

/// One gamepad's raw state, SDL conventions.
#[derive(Clone, Debug, Default)]
pub struct RawGamepad {
    pub buttons: HashSet<NativeButton>,
    pub left_x: i16,
    /// SDL: positive is down.
    pub left_y: i16,
    pub right_x: i16,
    pub right_y: i16,
    /// 0..32767
    pub trigger_l: i16,
    pub trigger_r: i16,
    pub has_rumble: bool,
    /// What the device is (profiles.rs), from its USB ids and name; read only by controller profiles.
    pub family: crate::profiles::Family,
    pub name: String,
}

/// `g_defaultButtonsStandard`.
pub const STANDARD_BUTTONS: [(NativeButton, u16); 10] = [
    (NativeButton::South, PAD_BUTTON_A),
    (NativeButton::East, PAD_BUTTON_B),
    (NativeButton::West, PAD_BUTTON_X),
    (NativeButton::North, PAD_BUTTON_Y),
    (NativeButton::Start, PAD_BUTTON_START),
    (NativeButton::RightShoulder, PAD_TRIGGER_Z),
    (NativeButton::DpadUp, PAD_BUTTON_UP),
    (NativeButton::DpadDown, PAD_BUTTON_DOWN),
    (NativeButton::DpadLeft, PAD_BUTTON_LEFT),
    (NativeButton::DpadRight, PAD_BUTTON_RIGHT),
];
// PAD_TRIGGER_L / R map to PAD_NATIVE_BUTTON_INVALID in the standard table: no digital button,
// so leftTriggerSet / rightTriggerSet stay false and the analog trigger emulates them.

pub const PAD_BUTTON_BACK: u32 = 0x0002000;
pub const PAD_BUTTON_GUIDE: u32 = 0x0004000;
pub const PAD_BUTTON_RIGHT_STICK: u32 = 0x2000000;
pub const PAD_BUTTON_LEFT_STICK: u32 = 0x4000000;

/// `kExtButtonMappings` (the subset gilrs can report).
const EXT_BUTTONS: [(NativeButton, u32); 4] = [
    (NativeButton::Back, PAD_BUTTON_BACK),
    (NativeButton::Guide, PAD_BUTTON_GUIDE),
    (NativeButton::RightStick, PAD_BUTTON_RIGHT_STICK),
    (NativeButton::LeftStick, PAD_BUTTON_LEFT_STICK),
];

/// `PADDeadZones` defaults (`lib/input.hpp`).
#[derive(Clone, Copy, Debug)]
pub struct DeadZones {
    pub emulate_triggers: bool,
    pub use_deadzones: bool,
    pub stick: u16,
    pub substick: u16,
    pub left_trigger_activation: u16,
    pub right_trigger_activation: u16,
}
impl Default for DeadZones {
    fn default() -> Self {
        Self {
            emulate_triggers: true,
            use_deadzones: true,
            stick: 8000,
            substick: 8000,
            left_trigger_activation: 31150,
            right_trigger_activation: 31150,
        }
    }
}

/// `_get_axis_value` for an axis mapping `(value, sign)`: `min(value * sign, SDL_JOYSTICK_AXIS_MAX)`.
#[inline]
fn axis_value(v: i16, sign: i32) -> i16 {
    (v as i32 * sign).min(32767) as i16
}

/// Aurora's `Sint16 y = -(y + 1u) / 256u` (unsigned arithmetic, then truncated to Sint16).
#[inline]
fn aurora_y(y: i16) -> i16 {
    let u = (y as i32 as u32).wrapping_add(1);
    (u.wrapping_neg() / 256) as i16
}

/// One stick, Aurora's arithmetic. `x`, `y` are SDL raw (y down). Returns (GC x, GC y).
fn stick(x: i16, y: i16, use_dz: bool, dz: u16) -> (i8, i8) {
    let x_pos = axis_value(x, 1);
    let x_neg = axis_value(x, -1);
    let y_pos = axis_value(y, -1); // LEFTY NEGATIVE -> PAD_AXIS_*_Y_POS
    let y_neg = axis_value(y, 1);
    let mut xl = ((x_pos as i32 + -(x_neg as i32)) / 2) as i16;
    let mut yl = ((-(y_pos as i32) + y_neg as i32) / 2) as i16;
    if use_dz {
        xl = if (xl as i32).abs() > dz as i32 { xl / 256 } else { 0 };
        yl = if (yl as i32).abs() > dz as i32 { aurora_y(yl) } else { 0 };
    } else {
        xl /= 256;
        yl = aurora_y(yl);
    }
    (xl as i8, yl as i8)
}

/// Aurora `PAD_AXIS_*` for keyboard axis bindings.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PadAxis {
    LeftXPos,
    LeftXNeg,
    LeftYPos,
    LeftYNeg,
    RightXPos,
    RightXNeg,
    RightYPos,
    RightYNeg,
    TriggerL,
    TriggerR,
}

/// One port's keyboard bindings (`g_keyboardBindings[i]`). Empty = `m_mappingsSet` false.
/// The C build sets none (its keyboard is hotkeys only), so the default is empty;
/// [`KeyboardBindings::preset_default`] is a port-side convenience, not from the C.
#[derive(Clone, Debug, Default)]
pub struct KeyboardBindings {
    pub buttons: Vec<(KeyCode, u16)>,
    pub axes: Vec<(KeyCode, PadAxis)>,
}

impl KeyboardBindings {
    pub fn is_set(&self) -> bool {
        !self.buttons.is_empty() || !self.axes.is_empty()
    }
    /// WASD stick, IJKL C-stick, arrows d-pad, Space A, LShift B, F/G X/Y, Q/E L/R (digital +
    /// full analog), Z, Enter Start. Port-side choice; nothing in the C defines a keyboard map.
    pub fn preset_default() -> Self {
        use KeyCode::*;
        Self {
            buttons: vec![
                (Space, PAD_BUTTON_A),
                (ShiftLeft, PAD_BUTTON_B),
                (KeyF, PAD_BUTTON_X),
                (KeyG, PAD_BUTTON_Y),
                (KeyZ, PAD_TRIGGER_Z),
                (KeyQ, PAD_TRIGGER_L),
                (KeyE, PAD_TRIGGER_R),
                (Enter, PAD_BUTTON_START),
                (ArrowUp, PAD_BUTTON_UP),
                (ArrowDown, PAD_BUTTON_DOWN),
                (ArrowLeft, PAD_BUTTON_LEFT),
                (ArrowRight, PAD_BUTTON_RIGHT),
            ],
            axes: vec![
                (KeyD, PadAxis::LeftXPos),
                (KeyA, PadAxis::LeftXNeg),
                (KeyW, PadAxis::LeftYPos),
                (KeyS, PadAxis::LeftYNeg),
                (KeyL, PadAxis::RightXPos),
                (KeyJ, PadAxis::RightXNeg),
                (KeyI, PadAxis::RightYPos),
                (KeyK, PadAxis::RightYNeg),
                (KeyQ, PadAxis::TriggerL),
                (KeyE, PadAxis::TriggerR),
            ],
        }
    }
}

/// Aurora `PADRead`, one port. `pad` = the gamepad assigned to this port, if any.
/// Returns the status and whether the port reports rumble support.
pub fn aurora_port(
    keys: &HashSet<KeyCode>,
    kb: &KeyboardBindings,
    pad: Option<&RawGamepad>,
    dz: &DeadZones,
) -> (PADStatus, bool) {
    let mut s = PADStatus::zeroed();
    if pad.is_none() && !kb.is_set() {
        s.err = PAD_ERR_NO_CONTROLLER;
        return (s, false);
    }
    s.err = PAD_ERR_NONE;
    if kb.is_set() {
        for (k, b) in &kb.buttons {
            if keys.contains(k) {
                s.button |= *b;
            }
        }
        let (mut lx, mut ly, mut rx, mut ry, mut tl, mut tr) = (0i32, 0i32, 0i32, 0i32, 0i32, 0i32);
        for (k, a) in &kb.axes {
            if !keys.contains(k) {
                continue;
            }
            match a {
                PadAxis::LeftXPos => lx += 127,
                PadAxis::LeftXNeg => lx -= 127,
                PadAxis::LeftYPos => ly += 127,
                PadAxis::LeftYNeg => ly -= 127,
                PadAxis::RightXPos => rx += 127,
                PadAxis::RightXNeg => rx -= 127,
                PadAxis::RightYPos => ry += 127,
                PadAxis::RightYNeg => ry -= 127,
                PadAxis::TriggerL => tl += 255,
                PadAxis::TriggerR => tr += 255,
            }
        }
        s.stickX = (s.stickX as i32 + lx).clamp(-127, 127) as i8;
        s.stickY = (s.stickY as i32 + ly).clamp(-127, 127) as i8;
        s.substickX = (s.substickX as i32 + rx).clamp(-127, 127) as i8;
        s.substickY = (s.substickY as i32 + ry).clamp(-127, 127) as i8;
        s.triggerLeft = (s.triggerLeft as i32 + tl).min(255) as u8;
        s.triggerRight = (s.triggerRight as i32 + tr).min(255) as u8;
    }
    let mut rumble = false;
    if let Some(p) = pad {
        for (nb, b) in STANDARD_BUTTONS {
            if p.buttons.contains(&nb) {
                s.button |= b;
            }
        }
        let (left_set, right_set) = (false, false);
        for (nb, b) in EXT_BUTTONS {
            if p.buttons.contains(&nb) {
                s.extButton |= b;
            }
        }
        let (x, y) = stick(p.left_x, p.left_y, dz.use_deadzones, dz.stick);
        s.stickX = x;
        s.stickY = y;
        let (x, y) = stick(p.right_x, p.right_y, dz.use_deadzones, dz.substick);
        s.substickX = x;
        s.substickY = y;
        let mut tl = axis_value(p.trigger_l, 1).max(0);
        let mut tr = axis_value(p.trigger_r, 1).max(0);
        if dz.emulate_triggers {
            if !left_set && tl as i32 > dz.left_trigger_activation as i32 {
                s.button |= PAD_TRIGGER_L;
            }
            if !right_set && tr as i32 > dz.right_trigger_activation as i32 {
                s.button |= PAD_TRIGGER_R;
            }
        }
        tl /= 128;
        tr /= 128;
        s.triggerLeft = tl as i8 as u8;
        s.triggerRight = tr as i8 as u8;
        if s.button & PAD_TRIGGER_L != 0 {
            s.triggerLeft = 180;
        }
        if s.button & PAD_TRIGGER_R != 0 {
            s.triggerRight = 180;
        }
        rumble = p.has_rumble;
    }
    (s, rumble)
}

/// Aurora `PADRead` over all four ports. Returns the rumble-support mask.
pub fn aurora_read(
    st: &mut [PADStatus; PAD_CHANMAX],
    keys: &HashSet<KeyCode>,
    kb: &[KeyboardBindings; PAD_CHANMAX],
    pads: &[Option<RawGamepad>; PAD_CHANMAX],
    dz: &DeadZones,
) -> u32 {
    let mut mask = 0;
    for i in 0..PAD_CHANMAX {
        let (s, r) = aurora_port(keys, &kb[i], pads[i].as_ref(), dz);
        st[i] = s;
        if r {
            mask |= PAD_CHAN0_BIT >> i;
        }
    }
    mask
}
