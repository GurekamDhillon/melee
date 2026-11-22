//! gilrs gamepads -> [`RawGamepad`] in SDL conventions, assigned to ports in connection order
//! (Aurora assigns by SDL player index, which SDL hands out in connection order too).
use crate::aurora::{NativeButton, RawGamepad};
use crate::pad::PAD_CHANMAX;
use crate::plog;
use gilrs::{Axis, Button, GamepadId, Gilrs};

pub struct Gamepads {
    gilrs: Option<Gilrs>,
    ports: [Option<GamepadId>; PAD_CHANMAX],
}

/// gilrs axis value (-1..1, +Y up) -> SDL Sint16.
fn to_sdl(v: f32) -> i16 {
    (v * 32767.0).round().clamp(-32768.0, 32767.0) as i16
}

/// A raw controller's buttons and hat as his `hid<n>` mask reads them. Button N+1 is bit N: the WGI backend numbers
/// its raw buttons from 0 (kind 0 in the code's high half); elsewhere the n-th button the device has. The hat is a
/// switch (kind 2) as two axes, x then y, where gilrs gives WGI's y as down-positive. Not tried on a box.
pub fn hid_mask(state: &gilrs::ev::state::GamepadState) -> u32 {
    let mut m = 0u32;
    for (n, (code, d)) in state.buttons().enumerate() {
        if !d.is_pressed() {
            continue;
        }
        let c = code.into_u32();
        let idx = if cfg!(windows) {
            if c >> 16 != 0 {
                continue;
            }
            c as usize
        } else {
            n
        };
        if idx < 28 {
            m |= 1 << idx;
        }
    }
    let (mut hx, mut hy) = (0.0f32, 0.0f32);
    for (code, a) in state.axes() {
        let c = code.into_u32();
        if cfg!(windows) && c >> 16 == 2 && c & 0xFFFF < 2 {
            if c & 1 == 0 {
                hx = a.value();
            } else {
                hy = a.value();
            }
        }
    }
    m | hat_bits(hx, hy)
}

/// The hat switch from its two axes (-1..1; y negative is Up).
pub fn hat_bits(x: f32, y: f32) -> u32 {
    use crate::bindings::{HID_HAT_DOWN, HID_HAT_LEFT, HID_HAT_RIGHT, HID_HAT_UP};
    let mut m = 0;
    if y < -0.5 {
        m |= HID_HAT_UP;
    }
    if y > 0.5 {
        m |= HID_HAT_DOWN;
    }
    if x < -0.5 {
        m |= HID_HAT_LEFT;
    }
    if x > 0.5 {
        m |= HID_HAT_RIGHT;
    }
    m
}

impl Gamepads {
    pub fn new() -> Self {
        let gilrs = match Gilrs::new() {
            Ok(g) => Some(g),
            Err(gilrs::Error::NotImplemented(g)) => {
                plog!("gw: input: gilrs has no backend on this platform");
                Some(g)
            }
            Err(e) => {
                plog!("gw: input: gilrs failed to start: {e}");
                None
            }
        };
        let mut me = Self { gilrs, ports: [None; PAD_CHANMAX] };
        me.reassign();
        me
    }

    /// No gamepads at all (headless).
    pub fn none() -> Self {
        Self { gilrs: None, ports: [None; PAD_CHANMAX] }
    }

    fn reassign(&mut self) {
        let Some(g) = self.gilrs.as_ref() else { return };
        for p in self.ports.iter_mut() {
            if let Some(id) = *p {
                if g.connected_gamepad(id).is_none() {
                    plog!("gw: input: gamepad {id} disconnected");
                    *p = None;
                }
            }
        }
        for (id, gp) in g.gamepads() {
            if self.ports.contains(&Some(id)) {
                continue;
            }
            if let Some(port) = self.ports.iter().position(|p| p.is_none()) {
                plog!(
                    "gw: input: gamepad \"{}\" ({}) -> port {}",
                    gp.name(),
                    crate::profiles::Family::detect(gp.vendor_id(), gp.product_id(), gp.name()).name(),
                    port + 1
                );
                self.ports[port] = Some(id);
            }
        }
    }

    /// Drain events (keeps gilrs' cached state current) and return each port's raw state.
    pub fn poll(&mut self) -> [Option<RawGamepad>; PAD_CHANMAX] {
        let mut changed = false;
        if let Some(g) = self.gilrs.as_mut() {
            while let Some(ev) = g.next_event() {
                if matches!(ev.event, gilrs::EventType::Connected | gilrs::EventType::Disconnected) {
                    changed = true;
                }
            }
        }
        if changed {
            self.reassign();
        }
        let mut out: [Option<RawGamepad>; PAD_CHANMAX] = Default::default();
        let Some(g) = self.gilrs.as_ref() else { return out };
        for (i, id) in self.ports.iter().enumerate() {
            let Some(gp) = id.and_then(|id| g.connected_gamepad(id)) else { continue };
            let mut r = RawGamepad::default();
            const MAP: [(Button, NativeButton); 19] = [
                (Button::South, NativeButton::South),
                (Button::East, NativeButton::East),
                (Button::West, NativeButton::West),
                (Button::North, NativeButton::North),
                (Button::Start, NativeButton::Start),
                (Button::Select, NativeButton::Back),
                (Button::Mode, NativeButton::Guide),
                (Button::LeftTrigger, NativeButton::LeftShoulder),
                (Button::RightTrigger, NativeButton::RightShoulder),
                (Button::LeftThumb, NativeButton::LeftStick),
                (Button::RightThumb, NativeButton::RightStick),
                (Button::DPadUp, NativeButton::DpadUp),
                (Button::DPadDown, NativeButton::DpadDown),
                (Button::DPadLeft, NativeButton::DpadLeft),
                (Button::DPadRight, NativeButton::DpadRight),
                (Button::LeftTrigger2, NativeButton::LeftTrigger2),
                (Button::RightTrigger2, NativeButton::RightTrigger2),
                (Button::C, NativeButton::C),
                (Button::Z, NativeButton::Z),
            ];
            for (b, nb) in MAP {
                if gp.is_pressed(b) {
                    r.buttons.insert(nb);
                }
            }
            r.left_x = to_sdl(gp.value(Axis::LeftStickX));
            r.left_y = to_sdl(-gp.value(Axis::LeftStickY));
            r.right_x = to_sdl(gp.value(Axis::RightStickX));
            r.right_y = to_sdl(-gp.value(Axis::RightStickY));
            let t = |b: Button| gp.button_data(b).map(|d| d.value()).unwrap_or(0.0);
            r.trigger_l = to_sdl(t(Button::LeftTrigger2).max(0.0));
            r.trigger_r = to_sdl(t(Button::RightTrigger2).max(0.0));
            r.has_rumble = gp.is_ff_supported();
            r.name = gp.name().to_string();
            r.family = crate::profiles::Family::detect(gp.vendor_id(), gp.product_id(), gp.name());
            // his "HID Pad": a generic / box device that is a raw controller, not an XInput-layout gamepad
            if matches!(r.family, crate::profiles::Family::Generic | crate::profiles::Family::Box) && gp.mapping_source() != gilrs::MappingSource::Driver {
                r.hid_raw = true;
                r.hid_buttons = hid_mask(gp.state());
            }
            out[i] = Some(r);
        }
        out
    }
}

impl Default for Gamepads {
    fn default() -> Self {
        Self::new()
    }
}
