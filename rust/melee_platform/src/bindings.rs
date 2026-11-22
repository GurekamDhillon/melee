//! Remapped controls in the Melee Unlocked launcher's format (`port-settings.ini`, written by its
//! settings panel), so a player who set their buttons there gets the same buttons in the Rust engine.
//!
//! The file holds one table per device family, `<family><n>_<Action> <mask>` (n = 0..3) and
//! `key_<Action> <Windows virtual-key code>`. The actions, in his order: A B X Y Z Start L R, the
//! D-pad, the C-stick and control-stick directions, the L/R analog travel, the box-controller keys
//! (ModX ModY LightShield MidShield) and the stick modifier. Read here:
//!
//! * `key_*` -> port 1's [`KeyboardBindings`] (buttons, both sticks, the triggers);
//! * `pad<n>_*` (XInput masks: the XINPUT_GAMEPAD bits, plus 0x0400 / 0x0800 for LT / RT past the
//!   click threshold) -> the gilrs gamepad on port n, which gilrs reports in the Xbox layout;
//! * `gc<n>_*` (GameCube adapter masks, the PAD bits) -> the adapter controller on port n, when the
//!   table is not the identity.
//!
//! * `ds4<n>_*` / `swpro<n>_*` (his DualShock 4 and Switch Pro masks) -> a gilrs pad on port n that
//!   is that family (profiles::Family, from its USB ids): gilrs reports both in the Xbox layout by
//!   position, so each mask is turned into the XInput one of the same physical buttons (Cross / B
//!   bottom = A, Circle / A right = B, Square / Y left = X, Triangle / X top = Y, L2 / ZL = LT ...)
//!   and goes through the same path as `pad<n>`. A DualSense counts as a DualShock 4. Without its
//!   own table such a pad keeps the `pad<n>` table (or the default layout).
//!
//! * `hid<n>_*` (his generic HID pads: B0XX / Frame1 / vJoy and any pad without a reader of its own; 32-bit
//!   masks, bit N = the device's button N+1, bits 28..31 the hat Up / Right / Down / Left) -> the gilrs pad on
//!   port n that is a generic or box family device ([`RawGamepad::hid_buttons`]); `hid<n>_invert` (stick
//!   reversal bits) likewise;
//! * box mode, as in his launcher: a keyboard or HID table with Mod X or Mod Y bound turns its button-made stick and
//!   C-stick into box coordinates. The coordinates and the opposing-direction rule are the port's own
//!   (`profiles::box_sticks`: `box_values`, SOCD 2IP without reactivation), not his full angle table; Mod X + Mod Y
//!   makes the C buttons the D-pad, the light / mid shield buttons press R that deep;
//! * `stick_modifier_percent` + a `StickMod` binding: while held, a stick made of buttons pushes that percent of a
//!   full push (his default 50), on the keyboard and on pad / HID tables.
//!
//! Per-binding levels are not read (every analog source clicks at 200 / 255).
//! With no file nothing here runs and input is exactly the default.
use crate::aurora::{KeyboardBindings, NativeButton, PadAxis, RawGamepad};
use crate::pad::*;
use crate::profiles::{box_sticks, PortState};
use crate::socd::SocdMode;
use std::collections::HashSet;
use winit::keyboard::KeyCode;

/// His `BindAction` order.
pub const ACTIONS: [&str; 27] = [
    "A", "B", "X", "Y", "Z", "Start", "L", "R", "DUp", "DDown", "DLeft", "DRight", "CUp", "CDown", "CLeft", "CRight", "SUp",
    "SDown", "SLeft", "SRight", "LAnalog", "RAnalog", "ModX", "ModY", "LightShield", "MidShield", "StickMod",
];
const N: usize = ACTIONS.len();
/// The GameCube button each of the first twelve actions presses.
const ACTION_BIT: [u16; 12] = [
    PAD_BUTTON_A, PAD_BUTTON_B, PAD_BUTTON_X, PAD_BUTTON_Y, PAD_TRIGGER_Z, PAD_BUTTON_START, PAD_TRIGGER_L, PAD_TRIGGER_R,
    PAD_BUTTON_UP, PAD_BUTTON_DOWN, PAD_BUTTON_LEFT, PAD_BUTTON_RIGHT,
];
const XI_LT: u16 = 0x0400;
const XI_RT: u16 = 0x0800;
/// A trigger past this (0..255) counts as pressed for a binding on it (his default click level).
const TRIGGER_CLICK: u8 = 200;
/// Travel given by a button bound to L / R analog (his default analog depth).
const BUTTON_DEPTH: u8 = 80;

/// One device's table: the source mask per action (0 = unbound).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PadMap {
    pub mask: [u16; N],
}

impl Default for PadMap {
    fn default() -> Self {
        PadMap { mask: [0; N] }
    }
}

/// His hat switch bits, above the device's own 28 buttons.
pub const HID_HAT_UP: u32 = 1 << 28;
pub const HID_HAT_RIGHT: u32 = 1 << 29;
pub const HID_HAT_DOWN: u32 = 1 << 30;
pub const HID_HAT_LEFT: u32 = 1 << 31;
/// `hid<n>_invert` bits.
pub const HID_INVERT_STICK_X: u8 = 1;
pub const HID_INVERT_STICK_Y: u8 = 2;
pub const HID_INVERT_CSTICK_X: u8 = 4;
pub const HID_INVERT_CSTICK_Y: u8 = 8;
/// His default `stick_modifier_percent`.
pub const STICK_MOD_DEFAULT: u8 = 50;

/// A generic HID pad's table: the source mask per action (0 = unbound), 32 bits ([`HID_HAT_UP`] ...).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HidMap {
    pub mask: [u32; N],
}

impl Default for HidMap {
    fn default() -> Self {
        HidMap { mask: [0; N] }
    }
}

/// The keyboard's keys for the actions the Aurora layer's button / axis lists do not carry (box mode, the stick
/// modifier). Port 1 only, as `key_*` is.
#[derive(Clone, Debug, Default)]
pub struct KeyboardExtra {
    /// Mod X or Mod Y is bound: the stick and C-stick keys are read here (in box coordinates), not as full pushes.
    pub box_mode: bool,
    pub keys: [Option<KeyCode>; N],
    /// The stick modifier's push, percent.
    pub pct: u8,
}

/// Every table read from one settings file.
#[derive(Clone, Debug, Default)]
pub struct Bindings {
    pub keyboard: Option<KeyboardBindings>,
    pub xinput: [Option<PadMap>; PAD_CHANMAX],
    pub gc: [Option<PadMap>; PAD_CHANMAX],
    /// `ds4<n>_*` / `swpro<n>_*`, already turned into XInput masks ([`family_to_xinput`]).
    pub ds4: [Option<PadMap>; PAD_CHANMAX],
    pub swpro: [Option<PadMap>; PAD_CHANMAX],
    /// `hid<n>_*` (32-bit masks) and `hid<n>_invert`.
    pub hid: [Option<HidMap>; PAD_CHANMAX],
    pub hid_invert: [u8; PAD_CHANMAX],
    /// Box mode and the stick modifier of the keyboard (port 1).
    pub keyboard_extra: Option<KeyboardExtra>,
    /// `stick_modifier_percent` (10..100); None = his default 50.
    pub stick_mod_pct: Option<u8>,
    /// `rumble 0|1` (his Rumble switch); None when the file does not say.
    pub rumble: Option<bool>,
    /// `key value` lines read, for the log.
    pub lines: usize,
}

fn action_index(name: &str) -> Option<usize> {
    ACTIONS.iter().position(|a| *a == name)
}

/// `<prefix><digit>_<Action>` -> (port, action).
fn indexed(key: &str, prefix: &str) -> Option<(usize, usize)> {
    let rest = key.strip_prefix(prefix)?;
    let (n, action) = rest.split_once('_')?;
    let n: usize = n.parse().ok()?;
    (n < PAD_CHANMAX).then_some(())?;
    Some((n, action_index(action)?))
}

/// Parse the binding lines of a `port-settings.ini` (`key value` per line). Other keys are ignored.
pub fn parse(text: &str) -> Bindings {
    let mut b = Bindings::default();
    let mut keys: [i32; N] = [0; N];
    let mut any_key = false;
    for line in text.lines() {
        let Some((k, v)) = line.trim().split_once(char::is_whitespace) else { continue };
        let v = v.trim();
        if k == "rumble" {
            b.rumble = Some(v != "0");
        } else if k == "stick_modifier_percent" {
            if let Ok(p) = v.parse::<i64>() {
                b.stick_mod_pct = Some(p.clamp(10, 100) as u8);
                b.lines += 1;
            }
        } else if let Some(n) = k.strip_prefix("hid").and_then(|r| r.strip_suffix("_invert")).and_then(|n| n.parse::<usize>().ok()).filter(|&n| n < PAD_CHANMAX) {
            if let Ok(m) = v.parse::<i64>() {
                b.hid_invert[n] = (m & 15) as u8;
                b.lines += 1;
            }
        } else if let Some((n, a)) = indexed(k, "hid") {
            // his masks are unsigned 32-bit (std::stoul); a signed spelling of bit 31 is taken back as unsigned
            if let Ok(m) = v.parse::<i64>() {
                b.hid[n].get_or_insert_with(HidMap::default).mask[a] = m as u32;
                b.lines += 1;
            }
        } else if let Some(a) = k.strip_prefix("key_").and_then(action_index) {
            if let Ok(vk) = v.parse::<i32>() {
                keys[a] = vk;
                any_key = true;
                b.lines += 1;
            }
        } else if let Some((n, a)) = indexed(k, "pad") {
            if let Ok(m) = v.parse::<u16>() {
                b.xinput[n].get_or_insert_with(PadMap::default).mask[a] = m;
                b.lines += 1;
            }
        } else if let Some((n, a, fam)) = indexed(k, "ds4").map(|(n, a)| (n, a, Fam::Ds4)).or_else(|| indexed(k, "swpro").map(|(n, a)| (n, a, Fam::SwPro))) {
            if let Ok(m) = v.parse::<u16>() {
                let t = if fam == Fam::Ds4 { &mut b.ds4 } else { &mut b.swpro };
                t[n].get_or_insert_with(PadMap::default).mask[a] = family_to_xinput(fam, m);
                b.lines += 1;
            }
        } else if let Some((n, a)) = indexed(k, "gc") {
            if let Ok(m) = v.parse::<u16>() {
                b.gc[n].get_or_insert_with(PadMap::default).mask[a] = m;
                b.lines += 1;
            }
        }
    }
    if any_key {
        let pct = b.stick_mod_pct.unwrap_or(STICK_MOD_DEFAULT);
        let (kb, extra) = keyboard(&keys, pct);
        b.keyboard = Some(kb);
        b.keyboard_extra = Some(extra);
    }
    for g in b.gc.iter_mut() {
        if g.as_ref().is_some_and(is_gc_identity) {
            *g = None;
        }
    }
    b
}

/// The two pad families with tables of their own in his file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fam {
    Ds4,
    SwPro,
}

/// His DualShock 4 / Switch Pro mask (input_bindings.h: D-pad bits 0..3, then the four face buttons
/// left / bottom / right / top as he numbers them, L1 R1 L2 R2, Share/Minus, Options/Plus, L3, R3) as
/// the XInput mask of the button in the same place on an Xbox pad, which is how gilrs reports them.
pub fn family_to_xinput(fam: Fam, m: u16) -> u16 {
    // XInput: DUp 1, DDown 2, DLeft 4, DRight 8, Start 0x10, Back 0x20, LThumb 0x40, RThumb 0x80, LB 0x100, RB 0x200,
    // A 0x1000, B 0x2000, X 0x4000, Y 0x8000; his LT / RT bits 0x400 / 0x800
    let face: [u16; 4] = match fam {
        // Square, Cross, Circle, Triangle
        Fam::Ds4 => [0x4000, 0x1000, 0x2000, 0x8000],
        // B (bottom), A (right), Y (left), X (top)
        Fam::SwPro => [0x1000, 0x2000, 0x4000, 0x8000],
    };
    let map: [u16; 16] = [
        0x0001, 0x0002, 0x0004, 0x0008, face[0], face[1], face[2], face[3], 0x0100, 0x0200, XI_LT, XI_RT, 0x0020, 0x0010, 0x0040, 0x0080,
    ];
    (0..16).filter(|i| m & (1 << i) != 0).fold(0, |a, i| a | map[i])
}

/// One port's gamepad tables; [`PortMaps::apply`] picks the one for the pad plugged in there.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PortMaps {
    pub xinput: Option<PadMap>,
    pub ds4: Option<PadMap>,
    pub swpro: Option<PadMap>,
    pub hid: Option<HidMap>,
    pub hid_invert: u8,
}

impl PortMaps {
    /// Every port's tables from a parsed file.
    pub fn from_bindings(b: &Bindings) -> [PortMaps; PAD_CHANMAX] {
        std::array::from_fn(|i| PortMaps {
            xinput: b.xinput[i].clone(),
            ds4: b.ds4[i].clone(),
            swpro: b.swpro[i].clone(),
            hid: b.hid[i].clone(),
            hid_invert: b.hid_invert[i],
        })
    }
    pub fn for_family(&self, f: crate::profiles::Family) -> Option<&PadMap> {
        use crate::profiles::Family;
        let own = match f {
            Family::Ds4 | Family::Ds5 => self.ds4.as_ref(),
            Family::SwitchPro => self.swpro.as_ref(),
            _ => None,
        };
        own.or(self.xinput.as_ref())
    }
    /// Rebuild `s` (read through the default layout) from the table for the pad `p`; `state` is the port's box-mode
    /// history, `pct` the stick modifier's push.
    pub fn apply(&self, p: &RawGamepad, state: &mut PortState, pct: u8, s: &mut PADStatus) {
        if p.hid_raw {
            if let Some(h) = self.hid.as_ref() {
                return apply_hid(h, p, state, self.hid_invert, pct, s);
            }
        }
        if let Some(m) = self.for_family(p.family) {
            apply_xinput(m, p, pct, s);
        }
    }
}

/// The adapter table that changes nothing (his default): each button on itself.
fn is_gc_identity(m: &PadMap) -> bool {
    (0..12).all(|i| m.mask[i] == ACTION_BIT[i]) && (12..N).all(|i| m.mask[i] == 0 || matches!(i, 20 | 21))
}

fn keyboard(vk: &[i32; N], pct: u8) -> (KeyboardBindings, KeyboardExtra) {
    let mut kb = KeyboardBindings::default();
    // box mode (Mod X or Mod Y bound) takes the stick and C-stick keys: they go to KeyboardExtra, as box coordinates
    let box_mode = vk[22] != 0 || vk[23] != 0;
    let mut extra = KeyboardExtra { box_mode, keys: [None; N], pct };
    for (i, &code) in vk.iter().enumerate() {
        let Some(key) = vk_to_keycode(code) else { continue };
        if i >= 12 {
            extra.keys[i] = Some(key);
        }
        match i {
            0..=11 => {
                kb.buttons.push((key, ACTION_BIT[i]));
                // a digital L / R is the full press, as on the default keyboard
                if i == 6 {
                    kb.axes.push((key, PadAxis::TriggerL));
                } else if i == 7 {
                    kb.axes.push((key, PadAxis::TriggerR));
                }
            }
            12..=19 if box_mode => {}
            12 => kb.axes.push((key, PadAxis::RightYPos)),
            13 => kb.axes.push((key, PadAxis::RightYNeg)),
            14 => kb.axes.push((key, PadAxis::RightXNeg)),
            15 => kb.axes.push((key, PadAxis::RightXPos)),
            16 => kb.axes.push((key, PadAxis::LeftYPos)),
            17 => kb.axes.push((key, PadAxis::LeftYNeg)),
            18 => kb.axes.push((key, PadAxis::LeftXNeg)),
            19 => kb.axes.push((key, PadAxis::LeftXPos)),
            20 => kb.axes.push((key, PadAxis::TriggerL)),
            21 => kb.axes.push((key, PadAxis::TriggerR)),
            _ => {} // box keys and the stick modifier: KeyboardExtra
        }
    }
    (kb, extra)
}

/// His modifier push: `pct` percent of a full 127, rounded to nearest.
pub fn modifier_push(pct: u8) -> i8 {
    ((127 * pct.clamp(0, 100) as i32 + 50) / 100) as i8
}

/// The keyboard's box mode and stick modifier, on top of what the Aurora layer read for port 1 (its button and axis
/// lists leave the stick keys out in box mode). `keys` are the keys held (empty while the window is not focused).
pub fn apply_keyboard(e: &KeyboardExtra, keys: &HashSet<KeyCode>, state: &mut PortState, s: &mut PADStatus) {
    let mut on = [false; N];
    for (i, o) in on.iter_mut().enumerate().skip(12) {
        *o = e.keys[i].is_some_and(|k| keys.contains(&k));
    }
    // the plain stick keys are the Aurora layer's (full pushes); only the modifier rescales them
    if !e.box_mode {
        scale_stick_keys(&on, e.pct, s);
        return;
    }
    let mut tr = s.triggerRight;
    sticks(&on, &mut tr, Some(state), e.pct, s);
    s.triggerRight = tr;
}

/// The stick modifier on a keyboard whose stick keys the Aurora layer already read at full push: with the modifier
/// held, each axis that has a direction key down takes the reduced push.
fn scale_stick_keys(on: &[bool; N], pct: u8, s: &mut PADStatus) {
    if !on[26] {
        return;
    }
    let push = modifier_push(pct);
    let (up, down, left, right) = (on[16], on[17], on[18], on[19]);
    if up != down {
        s.stickY = if up { push } else { -push };
    }
    if left != right {
        s.stickX = if right { push } else { -push };
    }
}

/// A Windows virtual-key code as the winit key in the same place on a US layout.
pub fn vk_to_keycode(vk: i32) -> Option<KeyCode> {
    use KeyCode::*;
    const LETTERS: [KeyCode; 26] = [
        KeyA, KeyB, KeyC, KeyD, KeyE, KeyF, KeyG, KeyH, KeyI, KeyJ, KeyK, KeyL, KeyM, KeyN, KeyO, KeyP, KeyQ, KeyR, KeyS, KeyT,
        KeyU, KeyV, KeyW, KeyX, KeyY, KeyZ,
    ];
    const DIGITS: [KeyCode; 10] = [Digit0, Digit1, Digit2, Digit3, Digit4, Digit5, Digit6, Digit7, Digit8, Digit9];
    const NUMPAD: [KeyCode; 10] = [Numpad0, Numpad1, Numpad2, Numpad3, Numpad4, Numpad5, Numpad6, Numpad7, Numpad8, Numpad9];
    const FKEYS: [KeyCode; 12] = [F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12];
    Some(match vk {
        0x41..=0x5A => LETTERS[(vk - 0x41) as usize],
        0x30..=0x39 => DIGITS[(vk - 0x30) as usize],
        0x60..=0x69 => NUMPAD[(vk - 0x60) as usize],
        0x70..=0x7B => FKEYS[(vk - 0x70) as usize],
        0x08 => Backspace,
        0x09 => Tab,
        0x0D => Enter,
        0x10 | 0xA0 => ShiftLeft,
        0xA1 => ShiftRight,
        0x11 | 0xA2 => ControlLeft,
        0xA3 => ControlRight,
        0x12 | 0xA4 => AltLeft,
        0xA5 => AltRight,
        0x14 => CapsLock,
        0x1B => Escape,
        0x20 => Space,
        0x21 => PageUp,
        0x22 => PageDown,
        0x23 => End,
        0x24 => Home,
        0x25 => ArrowLeft,
        0x26 => ArrowUp,
        0x27 => ArrowRight,
        0x28 => ArrowDown,
        0x2D => Insert,
        0x2E => Delete,
        0x6A => NumpadMultiply,
        0x6B => NumpadAdd,
        0x6D => NumpadSubtract,
        0x6E => NumpadDecimal,
        0x6F => NumpadDivide,
        0xBA => Semicolon,
        0xBB => Equal,
        0xBC => Comma,
        0xBD => Minus,
        0xBE => Period,
        0xBF => Slash,
        0xC0 => Backquote,
        0xDB => BracketLeft,
        0xDC => Backslash,
        0xDD => BracketRight,
        0xDE => Quote,
        _ => return None,
    })
}

/// A gilrs pad's state as XInput `wButtons` plus his LT / RT bits.
pub fn xinput_buttons(p: &RawGamepad) -> u16 {
    use NativeButton::*;
    const BITS: [(NativeButton, u16); 14] = [
        (DpadUp, 0x0001), (DpadDown, 0x0002), (DpadLeft, 0x0004), (DpadRight, 0x0008), (Start, 0x0010), (Back, 0x0020),
        (LeftStick, 0x0040), (RightStick, 0x0080), (LeftShoulder, 0x0100), (RightShoulder, 0x0200), (South, 0x1000),
        (East, 0x2000), (West, 0x4000), (North, 0x8000),
    ];
    let mut m = 0;
    for (b, bit) in BITS {
        if p.buttons.contains(&b) {
            m |= bit;
        }
    }
    m
}

fn trig8(v: i16) -> u8 {
    (v.max(0) as i32 * 255 / 32767) as u8
}

/// Rebuild `s` (already read through the default layout) from `map` over a gilrs pad: the buttons
/// and triggers come from the table, the analog sticks stay as read unless a direction is bound.
/// `pct` is the stick modifier's push (percent) for a stick made of buttons.
pub fn apply_xinput(map: &PadMap, p: &RawGamepad, pct: u8, s: &mut PADStatus) {
    apply(map, xinput_buttons(p), trig8(p.trigger_l), trig8(p.trigger_r), XI_LT, XI_RT, pct, s);
}

/// The same over a GameCube adapter controller (its own PAD bits and trigger values).
pub fn apply_gc(map: &PadMap, pct: u8, s: &mut PADStatus) {
    let (raw, l, r) = (s.button, s.triggerLeft, s.triggerRight);
    apply(map, raw, l, r, PAD_TRIGGER_L, PAD_TRIGGER_R, pct, s);
}

fn apply(map: &PadMap, buttons: u16, left: u8, right: u8, lt_bit: u16, rt_bit: u16, pct: u8, s: &mut PADStatus) {
    let analog = lt_bit == XI_LT; // the XInput table's trigger bits are not buttons
    let held = |m: u16| -> bool {
        if m == 0 {
            return false;
        }
        if analog && ((m & XI_LT != 0 && left > TRIGGER_CLICK) || (m & XI_RT != 0 && right > TRIGGER_CLICK)) {
            return true;
        }
        let plain = if analog { m & !(XI_LT | XI_RT) } else { m };
        buttons & plain != 0
    };
    let travel = |m: u16| -> u8 {
        let mut t = 0u8;
        if m & lt_bit != 0 {
            t = left;
        }
        if m & rt_bit != 0 {
            t = t.max(right);
        }
        let plain = if analog { m & !(XI_LT | XI_RT) } else { m & !(lt_bit | rt_bit) };
        if buttons & plain != 0 {
            t = t.max(BUTTON_DEPTH);
        }
        t
    };
    let on: Vec<bool> = map.mask.iter().map(|&m| held(m)).collect();
    let on: [bool; N] = std::array::from_fn(|i| on[i]);
    let (mut tl, mut tr) = (travel(map.mask[20]), travel(map.mask[21]));
    finish(&on, &mut tl, &mut tr, None, pct, s);
    s.triggerLeft = tl;
    s.triggerRight = tr;
}

/// A generic HID pad through its table: the device's own sticks and analog triggers (read by the Aurora layer when
/// gilrs maps them) stay unless a direction is bound or a stick key is held; `invert` flips them first (his
/// `hid<n>_invert`). A table with Mod X or Mod Y is in box mode and uses `state`.
pub fn apply_hid(map: &HidMap, p: &RawGamepad, state: &mut PortState, invert: u8, pct: u8, s: &mut PADStatus) {
    let flip = |v: i8, on: bool| if on { v.saturating_neg() } else { v };
    s.stickX = flip(s.stickX, invert & HID_INVERT_STICK_X != 0);
    s.stickY = flip(s.stickY, invert & HID_INVERT_STICK_Y != 0);
    s.substickX = flip(s.substickX, invert & HID_INVERT_CSTICK_X != 0);
    s.substickY = flip(s.substickY, invert & HID_INVERT_CSTICK_Y != 0);
    let on: [bool; N] = std::array::from_fn(|i| map.mask[i] & p.hid_buttons != 0);
    // the device's own trigger axes pass through; a button on L / R analog presses that deep on top of them
    let depth = |a: bool, axis: u8| if a { axis.max(BUTTON_DEPTH) } else { axis };
    let (mut tl, mut tr) = (depth(on[20], s.triggerLeft), depth(on[21], s.triggerRight));
    let box_mode = map.mask[22] != 0 || map.mask[23] != 0;
    finish(&on, &mut tl, &mut tr, box_mode.then_some(state), pct, s);
    s.triggerLeft = tl;
    s.triggerRight = tr;
}

/// Turn the pressed actions `on` into `s`: the twelve buttons, L / R (a digital press is a full pull), the C-stick
/// and control stick directions as full pushes (the modifier scales the stick's), or, with `box_state`, the box's
/// coordinates. `tl` / `tr` are the trigger travel so far.
fn finish(on: &[bool; N], tl: &mut u8, tr: &mut u8, box_state: Option<&mut PortState>, pct: u8, s: &mut PADStatus) {
    set_buttons(on, tl, tr, s);
    sticks(on, tr, box_state, pct, s);
}

fn set_buttons(on: &[bool; N], tl: &mut u8, tr: &mut u8, s: &mut PADStatus) {
    let bits = ACTION_BIT.iter().fold(0, |a, b| a | b);
    s.button &= !bits;
    for i in 0..12 {
        if on[i] {
            s.button |= ACTION_BIT[i];
        }
    }
    if on[6] {
        *tl = 255;
    }
    if on[7] {
        *tr = 255;
    }
}

/// The stick half of [`finish`]: only the stick, C-stick, modifier and box actions of `on` are read.
fn sticks(on: &[bool; N], tr: &mut u8, box_state: Option<&mut PortState>, pct: u8, s: &mut PADStatus) {
    let Some(state) = box_state else {
        let push = |pos: bool, neg: bool, cur: i8, full: i8| -> i8 {
            if pos != neg {
                if pos { full } else { -full }
            } else {
                cur
            }
        };
        let sp = if on[26] { modifier_push(pct) } else { 127 };
        s.substickY = push(on[12], on[13], s.substickY, 127);
        s.substickX = push(on[15], on[14], s.substickX, 127);
        s.stickY = push(on[16], on[17], s.stickY, sp);
        s.stickX = push(on[19], on[18], s.stickX, sp);
        return;
    };
    use crate::profiles::BoxAction as A;
    let mut held = [false; 12];
    for (a, i) in [
        (A::Left, 18), (A::Right, 19), (A::Up, 16), (A::Down, 17), (A::CLeft, 14), (A::CRight, 15), (A::CUp, 12), (A::CDown, 13),
        (A::ModX, 22), (A::ModY, 23), (A::LightShield, 24), (A::MidShield, 25),
    ] {
        held[a as usize] = on[i];
    }
    let out = box_sticks(SocdMode::default(), &held, state);
    if out.any_stick {
        (s.stickX, s.stickY) = out.stick;
    }
    if on[22] && on[23] {
        // Mod X + Mod Y: the C buttons are the D-pad and the C-stick rests
        s.substickX = 0;
        s.substickY = 0;
        for (d, bit) in [(out.dir[3] > 0, PAD_BUTTON_UP), (out.dir[3] < 0, PAD_BUTTON_DOWN), (out.dir[2] < 0, PAD_BUTTON_LEFT), (out.dir[2] > 0, PAD_BUTTON_RIGHT)] {
            if d {
                s.button |= bit;
            }
        }
    } else if out.any_c {
        (s.substickX, s.substickY) = out.c;
    }
    *tr = (*tr).max(out.shield);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ds4_and_switch_pro_tables_become_xinput_by_position() {
        // his defaults: A on Cross (DS4) / on the bottom B (Switch Pro), Z on R1 / R, L on L2 / ZL, Start on Options / Plus
        let b = parse("ds40_A 32
ds40_Z 512
ds40_L 1024
ds40_Start 8192
swpro1_A 16
swpro1_B 32
swpro1_X 128
swpro1_DUp 1
");
        let d = b.ds4[0].as_ref().unwrap();
        assert_eq!((d.mask[0], d.mask[4], d.mask[6], d.mask[5]), (0x1000, 0x0200, XI_LT, 0x0010));
        let w = b.swpro[1].as_ref().unwrap();
        assert_eq!((w.mask[0], w.mask[1], w.mask[2], w.mask[8]), (0x1000, 0x2000, 0x8000, 0x0001));
        assert_eq!(b.lines, 8);
        assert_eq!(b.rumble, None);
        assert_eq!(parse("rumble 0
").rumble, Some(false));
        use crate::profiles::Family;
        let ports = PortMaps::from_bindings(&b);
        assert_eq!(ports[0].for_family(Family::Ds5), Some(d));
        assert_eq!(ports[0].for_family(Family::XInput), None);
        assert_eq!(ports[1].for_family(Family::SwitchPro), Some(w));
    }

    #[test]
    fn reads_keys_pads_and_skips_identity_gc() {
        let text = "volume 3\nkey_A 88\nkey_B 90\nkey_SLeft 65\nkey_L 81\npad0_A 4096\npad0_Z 512\npad0_L 1024\ngc0_A 256\ngc0_B 512\ngc0_X 1024\ngc0_Y 2048\ngc0_Z 16\ngc0_Start 4096\ngc0_L 64\ngc0_R 32\ngc0_DUp 8\ngc0_DDown 4\ngc0_DLeft 1\ngc0_DRight 2\ngc1_A 512\n";
        let b = parse(text);
        let kb = b.keyboard.unwrap();
        assert!(kb.buttons.contains(&(KeyCode::KeyX, PAD_BUTTON_A)));
        assert!(kb.buttons.contains(&(KeyCode::KeyZ, PAD_BUTTON_B)));
        assert!(kb.axes.contains(&(KeyCode::KeyA, PadAxis::LeftXNeg)));
        assert!(kb.axes.contains(&(KeyCode::KeyQ, PadAxis::TriggerL)));
        assert_eq!(b.xinput[0].as_ref().unwrap().mask[0], 4096);
        assert!(b.gc[0].is_none(), "the identity adapter table is not applied");
        assert_eq!(b.gc[1].as_ref().unwrap().mask[0], 512);
    }

    #[test]
    fn xinput_table_drives_buttons_and_triggers() {
        let mut map = PadMap::default();
        map.mask[0] = 0x2000; // A on the B button
        map.mask[4] = 0x0200; // Z on RB
        map.mask[6] = XI_LT; // L click on LT
        map.mask[20] = XI_LT; // L travel on LT
        map.mask[12] = 0x8000; // C-up on Y
        let mut p = RawGamepad::default();
        p.buttons.insert(NativeButton::East);
        p.buttons.insert(NativeButton::North);
        p.trigger_l = 16000;
        let mut s = PADStatus::zeroed();
        s.button = PAD_BUTTON_B; // what the default layout gave East
        apply_xinput(&map, &p, 50, &mut s);
        assert_eq!(s.button, PAD_BUTTON_A);
        assert_eq!(s.substickY, 127);
        assert_eq!(s.triggerLeft, trig8(16000));
        p.trigger_l = 32767;
        apply_xinput(&map, &p, 50, &mut s);
        assert_eq!(s.button & PAD_TRIGGER_L, PAD_TRIGGER_L);
        assert_eq!(s.triggerLeft, 255);
    }

    fn hid_pad(buttons: u32) -> RawGamepad {
        RawGamepad { hid_raw: true, hid_buttons: buttons, family: crate::profiles::Family::Generic, ..Default::default() }
    }

    #[test]
    fn hid_tables_parse_as_unsigned_32_bit() {
        // bit 31 (hat Left) arrives unsigned from his writer, or negative from a signed spelling
        let b = parse("hid0_A 2\nhid0_DLeft 2147483648\nhid0_DRight -1073741824\nhid2_B 1\nhid0_invert 6\nhid9_A 1\nhid0_Nope 1\nstick_modifier_percent 200\n");
        let h = b.hid[0].as_ref().unwrap();
        assert_eq!((h.mask[0], h.mask[10], h.mask[11]), (2, HID_HAT_LEFT, 0xC000_0000));
        assert_eq!(b.hid[2].as_ref().unwrap().mask[1], 1);
        assert!(b.hid[1].is_none() && b.hid[3].is_none());
        assert_eq!(b.hid_invert[0], 6);
        assert_eq!(b.stick_mod_pct, Some(100), "clamped like his 10..100");
        assert_eq!(parse("stick_modifier_percent 3\n").stick_mod_pct, Some(10));
        assert_eq!(PortMaps::from_bindings(&b)[0].hid_invert, 6);
    }

    #[test]
    fn hid_table_buttons_hat_and_inversion() {
        let mut map = HidMap::default();
        map.mask[0] = 1 << 1; // A on button 2
        map.mask[1] = 1 << 0; // B on button 1
        map.mask[8] = HID_HAT_UP;
        map.mask[7] = 1 << 5; // R on button 6: a digital press pulls the trigger all the way
        map.mask[21] = 1 << 6; // R analog on button 7: a light shield depth
        let mut s = PADStatus::zeroed();
        s.stickX = 40;
        s.stickY = -20;
        s.substickX = 5;
        let p = hid_pad((1 << 1) | HID_HAT_UP | (1 << 5));
        apply_hid(&map, &p, &mut PortState::default(), HID_INVERT_STICK_X | HID_INVERT_CSTICK_X, 50, &mut s);
        assert_eq!(s.button, PAD_BUTTON_A | PAD_BUTTON_UP | PAD_TRIGGER_R);
        assert_eq!(s.triggerRight, 255);
        assert_eq!((s.stickX, s.stickY, s.substickX), (-40, -20, -5));
        let mut s = PADStatus::zeroed();
        apply_hid(&map, &hid_pad(1 << 6), &mut PortState::default(), 0, 50, &mut s);
        assert_eq!((s.button, s.triggerRight), (0, BUTTON_DEPTH));
    }

    #[test]
    fn hid_pad_that_is_not_raw_keeps_the_pad_table() {
        let b = parse("hid0_A 1\npad0_A 8192\n");
        let maps = PortMaps::from_bindings(&b);
        let mut xi = RawGamepad::default();
        xi.buttons.insert(NativeButton::East);
        let mut s = PADStatus::zeroed();
        maps[0].apply(&xi, &mut PortState::default(), 50, &mut s);
        assert_eq!(s.button, PAD_BUTTON_A, "an XInput-layout pad reads pad0, not hid0");
        let mut s = PADStatus::zeroed();
        maps[0].apply(&hid_pad(1), &mut PortState::default(), 50, &mut s);
        assert_eq!(s.button, PAD_BUTTON_A);
    }

    #[test]
    fn box_mode_from_mod_x_and_mod_y() {
        use crate::profiles::box_values::*;
        let mut map = HidMap::default();
        map.mask[18] = 1 << 0; // stick left
        map.mask[19] = 1 << 1; // stick right
        map.mask[16] = 1 << 2; // up
        map.mask[14] = 1 << 3; // C left
        map.mask[12] = 1 << 4; // C up
        map.mask[22] = 1 << 8; // Mod X: box mode
        map.mask[23] = 1 << 9;
        map.mask[24] = 1 << 10; // light shield
        let mut st = PortState::default();
        let run = |st: &mut PortState, buttons: u32| {
            let mut s = PADStatus::zeroed();
            s.stickX = 11; // the device's own stick: left alone while no stick button is held
            apply_hid(&map, &hid_pad(buttons), st, 0, 50, &mut s);
            s
        };
        let s = run(&mut st, 0);
        assert_eq!((s.stickX, s.stickY), (11, 0));
        let s = run(&mut st, 1 << 1);
        assert_eq!((s.stickX, s.stickY), (FULL, 0));
        let s = run(&mut st, (1 << 1) | (1 << 8));
        assert_eq!(s.stickX, MODX_X);
        let s = run(&mut st, (1 << 1) | (1 << 2));
        assert_eq!((s.stickX, s.stickY), (DIAG, DIAG));
        let s = run(&mut st, 1 << 3);
        assert_eq!((s.substickX, s.substickY), (-FULL, 0));
        assert_eq!(run(&mut st, 1 << 10).triggerRight, LIGHT_SHIELD);
        // Mod X + Mod Y: the C buttons are the D-pad
        let s = run(&mut st, (1 << 8) | (1 << 9) | (1 << 4));
        assert_eq!(s.button, PAD_BUTTON_UP);
        assert_eq!((s.substickX, s.substickY), (0, 0));
        // 2IP without reactivation, on this path too: right held, left added -> left; left released -> neutral
        let mut st = PortState::default();
        let xs: Vec<i8> = [1u32 << 1, (1 << 1) | 1, 1 << 1].iter().map(|b| run(&mut st, *b).stickX).collect();
        assert_eq!(xs, [FULL, -FULL, 0]);
    }

    #[test]
    fn stick_modifier_scales_a_button_stick() {
        assert_eq!((modifier_push(50), modifier_push(100), modifier_push(10)), (64, 127, 13));
        // a pad table with the stick on buttons and the modifier held
        let mut map = PadMap::default();
        map.mask[19] = 0x0008; // stick right on D-pad right
        map.mask[16] = 0x0001; // stick up on D-pad up
        map.mask[26] = 0x0100; // modifier on LB
        let mut p = RawGamepad::default();
        p.buttons.insert(NativeButton::DpadRight);
        let mut s = PADStatus::zeroed();
        apply_xinput(&map, &p, 50, &mut s);
        assert_eq!(s.stickX, 127);
        p.buttons.insert(NativeButton::LeftShoulder);
        apply_xinput(&map, &p, 50, &mut s);
        assert_eq!((s.stickX, s.stickY), (64, 0));
        apply_xinput(&map, &p, 30, &mut s);
        assert_eq!(s.stickX, 38);
    }

    #[test]
    fn keyboard_box_mode_and_modifier() {
        use crate::profiles::box_values::*;
        // box keys bound: the stick keys leave the Aurora lists and go through the box rule
        let b = parse("key_A 90\nkey_SLeft 65\nkey_SRight 68\nkey_ModX 16\nkey_CUp 73\nstick_modifier_percent 40\n");
        let (kb, e) = (b.keyboard.unwrap(), b.keyboard_extra.unwrap());
        assert!(e.box_mode);
        assert!(kb.axes.is_empty(), "stick and C keys are the extra's in box mode");
        let keys: HashSet<KeyCode> = [KeyCode::KeyD, KeyCode::ShiftLeft].into_iter().collect();
        let mut s = PADStatus::zeroed();
        apply_keyboard(&e, &keys, &mut PortState::default(), &mut s);
        assert_eq!((s.stickX, s.stickY), (MODX_X, 0));
        // no box keys: plain full pushes, and the modifier key rescales them
        let b = parse("key_SLeft 65\nkey_SRight 68\nkey_StickMod 16\nstick_modifier_percent 40\n");
        let (kb, e) = (b.keyboard.unwrap(), b.keyboard_extra.unwrap());
        assert!(!e.box_mode);
        assert!(kb.axes.contains(&(KeyCode::KeyD, PadAxis::LeftXPos)));
        let mut s = PADStatus::zeroed();
        s.stickX = 127; // what the Aurora layer read for D
        apply_keyboard(&e, &[KeyCode::KeyD, KeyCode::ShiftLeft].into_iter().collect(), &mut PortState::default(), &mut s);
        assert_eq!(s.stickX, modifier_push(40));
        let mut s = PADStatus::zeroed();
        s.stickX = 127;
        apply_keyboard(&e, &[KeyCode::KeyD].into_iter().collect(), &mut PortState::default(), &mut s);
        assert_eq!(s.stickX, 127);
    }

    #[test]
    fn hat_axes_to_bits() {
        use crate::gamepads::hat_bits;
        assert_eq!(hat_bits(0.0, 0.0), 0);
        assert_eq!(hat_bits(0.0, -1.0), HID_HAT_UP);
        assert_eq!(hat_bits(1.0, 1.0), HID_HAT_RIGHT | HID_HAT_DOWN);
        assert_eq!(hat_bits(-1.0, -1.0), HID_HAT_LEFT | HID_HAT_UP);
    }

    #[test]
    fn vk_codes() {
        assert_eq!(vk_to_keycode(0x41), Some(KeyCode::KeyA));
        assert_eq!(vk_to_keycode(0x26), Some(KeyCode::ArrowUp));
        assert_eq!(vk_to_keycode(0x0D), Some(KeyCode::Enter));
        assert_eq!(vk_to_keycode(0), None);
    }
}
