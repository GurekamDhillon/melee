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
//! Box mode, the stick modifier, per-binding levels, and the DualShock / Switch Pro / HID tables are
//! not read: those devices come through gilrs as plain gamepads and keep the `pad<n>` (or default)
//! layout. With no file nothing here runs and input is exactly the default.
use crate::aurora::{KeyboardBindings, NativeButton, PadAxis, RawGamepad};
use crate::pad::*;
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

/// Every table read from one settings file.
#[derive(Clone, Debug, Default)]
pub struct Bindings {
    pub keyboard: Option<KeyboardBindings>,
    pub xinput: [Option<PadMap>; PAD_CHANMAX],
    pub gc: [Option<PadMap>; PAD_CHANMAX],
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
        if let Some(a) = k.strip_prefix("key_").and_then(action_index) {
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
        } else if let Some((n, a)) = indexed(k, "gc") {
            if let Ok(m) = v.parse::<u16>() {
                b.gc[n].get_or_insert_with(PadMap::default).mask[a] = m;
                b.lines += 1;
            }
        }
    }
    if any_key {
        b.keyboard = Some(keyboard(&keys));
    }
    for g in b.gc.iter_mut() {
        if g.as_ref().is_some_and(is_gc_identity) {
            *g = None;
        }
    }
    b
}

/// The adapter table that changes nothing (his default): each button on itself.
fn is_gc_identity(m: &PadMap) -> bool {
    (0..12).all(|i| m.mask[i] == ACTION_BIT[i]) && (12..N).all(|i| m.mask[i] == 0 || matches!(i, 20 | 21))
}

fn keyboard(vk: &[i32; N]) -> KeyboardBindings {
    let mut kb = KeyboardBindings::default();
    for (i, &code) in vk.iter().enumerate() {
        let Some(key) = vk_to_keycode(code) else { continue };
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
            _ => {} // box keys and the stick modifier: not read
        }
    }
    kb
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
pub fn apply_xinput(map: &PadMap, p: &RawGamepad, s: &mut PADStatus) {
    apply(map, xinput_buttons(p), trig8(p.trigger_l), trig8(p.trigger_r), XI_LT, XI_RT, s);
}

/// The same over a GameCube adapter controller (its own PAD bits and trigger values).
pub fn apply_gc(map: &PadMap, s: &mut PADStatus) {
    let (raw, l, r) = (s.button, s.triggerLeft, s.triggerRight);
    apply(map, raw, l, r, PAD_TRIGGER_L, PAD_TRIGGER_R, s);
}

fn apply(map: &PadMap, buttons: u16, left: u8, right: u8, lt_bit: u16, rt_bit: u16, s: &mut PADStatus) {
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
    s.button &= !ACTION_BIT.iter().fold(0, |a, b| a | b);
    for i in 0..12 {
        if on[i] {
            s.button |= ACTION_BIT[i];
        }
    }
    let mut tl = travel(map.mask[20]);
    let mut tr = travel(map.mask[21]);
    if on[6] {
        tl = 255;
    }
    if on[7] {
        tr = 255;
    }
    s.triggerLeft = tl;
    s.triggerRight = tr;
    let push = |pos: bool, neg: bool, cur: i8| -> i8 {
        if pos != neg {
            if pos { 127 } else { -127 }
        } else {
            cur
        }
    };
    s.substickY = push(on[12], on[13], s.substickY);
    s.substickX = push(on[15], on[14], s.substickX);
    s.stickY = push(on[16], on[17], s.stickY);
    s.stickX = push(on[19], on[18], s.stickX);
}

#[cfg(test)]
mod tests {
    use super::*;

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
        apply_xinput(&map, &p, &mut s);
        assert_eq!(s.button, PAD_BUTTON_A);
        assert_eq!(s.substickY, 127);
        assert_eq!(s.triggerLeft, trig8(16000));
        p.trigger_l = 32767;
        apply_xinput(&map, &p, &mut s);
        assert_eq!(s.button & PAD_TRIGGER_L, PAD_TRIGGER_L);
        assert_eq!(s.triggerLeft, 255);
    }

    #[test]
    fn vk_codes() {
        assert_eq!(vk_to_keycode(0x41), Some(KeyCode::KeyA));
        assert_eq!(vk_to_keycode(0x26), Some(KeyCode::ArrowUp));
        assert_eq!(vk_to_keycode(0x0D), Some(KeyCode::Enter));
        assert_eq!(vk_to_keycode(0), None);
    }
}
