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
                plog!("gw: input: gamepad \"{}\" -> port {}", gp.name(), port + 1);
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
            const MAP: [(Button, NativeButton); 15] = [
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
