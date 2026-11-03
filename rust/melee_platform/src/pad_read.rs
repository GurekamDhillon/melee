//! `PADRead` as the game sees it: port of `gw_PADRead_profile_body` (`pc/platform/shim_pad.c`).
//!
//! Order, as in the C:
//!  1. the Aurora layer ([`aurora::aurora_read`]: keyboard bindings + gilrs gamepads);
//!  2. input mode NONE, or the adapter suspended (window in the background): every port reads
//!     disconnected;
//!  3. F9 (focused, adapter present) -> adapter recalibration on the next poll;
//!  4. the raw GameCube adapter: a port with a controller in the adapter replaces the gamepad's;
//!  5. input mode NONE: port 0 is a controller at rest;
//!  6. the txt pad script overrides the channels it drives;
//!  7. the L+R+Y+X+Start release combo, which calls `PADRecalibrate(0xF0000000)`: in the C
//!     that is Aurora's no-op `PADRecalibrate` (it does NOT reach the adapter), kept as such.
//!
//! Not ported (C features layered on top, no effect on PADStatus unless configured): local
//! control remapping (`ctl_poll`, gw_controls_runtime.inc), the per-port stick dead-zone setting,
//! Lua/live/bot pads, mirroring, menu button masks, diagnostics/overlay mirrors, profiling.
//! The C's `button` byte swap (game memory is big-endian) has no counterpart: native order.
use crate::adapter::Adapter;
use crate::aurora::{self, DeadZones, KeyboardBindings, RawGamepad};
use crate::gamepads::Gamepads;
use crate::pad::*;
use crate::plog;
use crate::script::PadScript;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use winit::keyboard::KeyCode;

/// `GW_INPUT_GC` / `GW_INPUT_NONE`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputMode {
    /// Every controller (the default).
    Devices,
    /// No devices (turbo / headless / `MELEE_INPUT=none`): port 0 is a controller at rest.
    None,
}

/// Where a port's value came from (`GW_SRC_*`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    None,
    Adapter,
    Gamepad,
    Idle,
    Script,
}

/// L R Y X Start (`GW_PAD_RESET_COMBO`).
pub const GW_PAD_RESET_COMBO: u16 = PAD_TRIGGER_L | PAD_TRIGGER_R | PAD_BUTTON_Y | PAD_BUTTON_X | PAD_BUTTON_START;

/// Where the gamepad layer's raw states come from.
pub enum GamepadSource {
    Gilrs(Gamepads),
    /// Fixed raw states (tests).
    Fixed([Option<RawGamepad>; PAD_CHANMAX]),
}

pub struct PadSystem {
    pub mode: InputMode,
    pub adapter: Option<Adapter>,
    pub gamepads: GamepadSource,
    pub keyboard: [KeyboardBindings; PAD_CHANMAX],
    pub keys_down: HashSet<KeyCode>,
    pub dead_zones: DeadZones,
    /// Remapped gilrs pads / adapter controllers per port (`bindings`, the launcher's tables); None = default.
    pub pad_maps: [Option<crate::bindings::PadMap>; PAD_CHANMAX],
    pub gc_maps: [Option<crate::bindings::PadMap>; PAD_CHANMAX],
    /// Keep the adapter read while the window is in the background (`backgroundinput 1`).
    pub background_input: bool,
    pub script: Option<PadScript>,
    pub focused: bool,
    pub sources: [Source; PAD_CHANMAX],
    ignore_adapter: bool,
    f9_was_down: bool,
    reset_armed: bool,
    /// Times the reset combo fired (`PADRecalibrate` calls).
    pub reset_combo_fired: u32,
    /// The keys held and the focus, for a [`PadSampler`] on another thread.
    shared_keys: Arc<Mutex<(HashSet<KeyCode>, bool)>>,
}

/// The controllers read off the game thread (early inputs' sampler, rust/melee/src/np_early.rs): the same pipeline as
/// [`PadSystem::read`] (keyboard, gamepads with their maps, the adapter) without the pad script, the reset combo or
/// the F9 recalibration. Its gamepad layer is its own gilrs instance, opened on first read (on the reading thread).
pub struct PadSampler {
    mode: InputMode,
    adapter: Option<crate::adapter::AdapterPeek>,
    ignore_adapter: bool,
    gamepads: Option<Gamepads>,
    use_gilrs: bool,
    keyboard: [KeyboardBindings; PAD_CHANMAX],
    dead_zones: DeadZones,
    pad_maps: [Option<crate::bindings::PadMap>; PAD_CHANMAX],
    gc_maps: [Option<crate::bindings::PadMap>; PAD_CHANMAX],
    keys: Arc<Mutex<(HashSet<KeyCode>, bool)>>,
}

impl PadSampler {
    pub fn read(&mut self, st: &mut [PADStatus; PAD_CHANMAX]) {
        let pads = if self.use_gilrs {
            self.gamepads.get_or_insert_with(Gamepads::new).poll()
        } else {
            Default::default()
        };
        let keys = {
            let k = self.keys.lock().unwrap();
            if k.1 { k.0.clone() } else { HashSet::new() }
        };
        aurora::aurora_read(st, &keys, &self.keyboard, &pads, &self.dead_zones);
        for i in 0..PAD_CHANMAX {
            if let (Some(m), Some(p)) = (self.pad_maps[i].as_ref(), pads[i].as_ref()) {
                crate::bindings::apply_xinput(m, p, &mut st[i]);
            }
        }
        if self.mode == InputMode::None {
            for s in st.iter_mut() {
                *s = pad_disconnected();
            }
        }
        if !self.ignore_adapter {
            if let Some(a) = self.adapter.as_mut() {
                let mut adp = [pad_disconnected(); PAD_CHANMAX];
                a.read(&mut adp);
                for i in 0..PAD_CHANMAX {
                    if adp[i].err == 0 {
                        if let Some(m) = self.gc_maps[i].as_ref() {
                            crate::bindings::apply_gc(m, &mut adp[i]);
                        }
                        st[i] = adp[i];
                    }
                }
            }
        }
        if self.mode == InputMode::None {
            st[0] = pad_zero();
        }
    }
}

impl PadSystem {
    /// Interactive: gilrs + the system adapter (unless forbidden by env) + script from env.
    /// `MELEE_INPUT=n*` or `k*` (without '+') selects mode NONE, as in the C.
    pub fn interactive() -> Self {
        let v = std::env::var("MELEE_INPUT").unwrap_or_default();
        let none = matches!(v.bytes().next(), Some(b'n' | b'N')) || (matches!(v.bytes().next(), Some(b'k' | b'K')) && !v.contains('+'));
        let mode = if none { InputMode::None } else { InputMode::Devices };
        let mut adapter = None;
        if mode == InputMode::Devices && !crate::adapter::forbidden() {
            let mut a = Adapter::new_system();
            a.init();
            a.start_hotplug();
            adapter = Some(a);
        }
        plog!(
            "gw: input: {}",
            if mode == InputMode::Devices { "every controller; the keyboard is hotkeys only unless bound" } else { "no devices: port 1 is a controller at rest" }
        );
        Self::build(mode, adapter, GamepadSource::Gilrs(Gamepads::new()), PadScript::from_env())
    }

    /// Headless (the C's turbo): no devices, port 0 at rest, plus the env pad script if any.
    pub fn headless() -> Self {
        Self::build(InputMode::None, None, GamepadSource::Fixed(Default::default()), PadScript::from_env())
    }

    pub fn build(mode: InputMode, adapter: Option<Adapter>, gamepads: GamepadSource, script: Option<PadScript>) -> Self {
        let ignore_adapter = crate::adapter::forbidden() || mode == InputMode::None;
        Self {
            mode,
            adapter,
            gamepads,
            keyboard: Default::default(),
            keys_down: HashSet::new(),
            dead_zones: DeadZones::default(),
            pad_maps: Default::default(),
            gc_maps: Default::default(),
            background_input: false,
            script,
            focused: true,
            sources: [Source::None; PAD_CHANMAX],
            ignore_adapter,
            f9_was_down: false,
            reset_armed: false,
            reset_combo_fired: 0,
            shared_keys: Arc::new(Mutex::new((HashSet::new(), true))),
        }
    }

    /// A reader of the same controllers for another thread (see [`PadSampler`]).
    pub fn sampler(&self) -> PadSampler {
        self.sync_keys();
        PadSampler {
            mode: self.mode,
            adapter: self.adapter.as_ref().map(|a| a.peeker()),
            ignore_adapter: self.ignore_adapter,
            gamepads: None,
            use_gilrs: matches!(self.gamepads, GamepadSource::Gilrs(_)),
            keyboard: self.keyboard.clone(),
            dead_zones: self.dead_zones,
            pad_maps: self.pad_maps.clone(),
            gc_maps: self.gc_maps.clone(),
            keys: self.shared_keys.clone(),
        }
    }

    fn sync_keys(&self) {
        if let Ok(mut k) = self.shared_keys.lock() {
            k.0 = self.keys_down.clone();
            k.1 = self.focused;
        }
    }

    /// Force the adapter layer on/off regardless of env (tests).
    pub fn set_ignore_adapter(&mut self, v: bool) {
        self.ignore_adapter = v;
    }

    /// Window focus change. The C acts on a loss after 100 ms; here the caller debounces.
    pub fn set_focus(&mut self, focused: bool) {
        self.focused = focused;
        self.sync_keys();
        if let Some(a) = self.adapter.as_ref() {
            a.set_focused(focused || self.background_input);
            if focused || self.background_input { a.resume() } else { a.suspend(); }
        }
    }

    pub fn key_event(&mut self, key: KeyCode, down: bool) {
        if down {
            self.keys_down.insert(key);
        } else {
            self.keys_down.remove(&key);
        }
        self.sync_keys();
    }

    /// `PADRead`. Returns Aurora's rumble-support mask, as the C does.
    pub fn read(&mut self, st: &mut [PADStatus; PAD_CHANMAX]) -> u32 {
        let pads = match &mut self.gamepads {
            GamepadSource::Gilrs(g) => g.poll(),
            GamepadSource::Fixed(f) => f.clone(),
        };
        let keys = if self.focused { self.keys_down.clone() } else { HashSet::new() };
        self.sync_keys();
        let ret = aurora::aurora_read(st, &keys, &self.keyboard, &pads, &self.dead_zones);
        for i in 0..PAD_CHANMAX {
            if let (Some(m), Some(p)) = (self.pad_maps[i].as_ref(), pads[i].as_ref()) {
                crate::bindings::apply_xinput(m, p, &mut st[i]);
            }
        }

        let suspended = self.adapter.as_ref().is_some_and(|a| a.suspended());
        if self.mode == InputMode::None || suspended {
            for s in st.iter_mut() {
                *s = pad_disconnected();
            }
        }
        let mut src = [Source::None; PAD_CHANMAX];
        for i in 0..PAD_CHANMAX {
            src[i] = if st[i].err == 0 { Source::Gamepad } else { Source::None };
        }

        if let Some(a) = self.adapter.as_ref() {
            if a.present() {
                let f9 = self.focused && self.keys_down.contains(&KeyCode::F9);
                if f9 && !self.f9_was_down {
                    a.recalibrate();
                }
                self.f9_was_down = f9;
            }
        }

        if !self.ignore_adapter {
            if let Some(a) = self.adapter.as_mut() {
                let mut adp = [pad_disconnected(); PAD_CHANMAX];
                a.read(&mut adp);
                for i in 0..PAD_CHANMAX {
                    if adp[i].err == 0 {
                        if let Some(m) = self.gc_maps[i].as_ref() {
                            crate::bindings::apply_gc(m, &mut adp[i]);
                        }
                        st[i] = adp[i];
                        src[i] = Source::Adapter;
                    }
                }
            }
        }

        if self.mode == InputMode::None {
            st[0] = pad_zero();
            src[0] = Source::Idle;
        }

        if let Some(s) = self.script.as_mut() {
            let driven = s.apply(st);
            for i in 0..PAD_CHANMAX {
                if driven & (1 << i) != 0 {
                    src[i] = if st[i].err == 0 { Source::Script } else { Source::None };
                }
            }
        }

        self.reset_combo(st);
        for i in 0..PAD_CHANMAX {
            self.sources[i] = if st[i].err == 0 || src[i] == Source::Idle { src[i] } else { Source::None };
        }
        ret
    }

    /// `gw_pad_reset_combo`.
    fn reset_combo(&mut self, st: &[PADStatus; PAD_CHANMAX]) {
        let held = st.iter().any(|s| s.err == 0 && s.button & GW_PAD_RESET_COMBO == GW_PAD_RESET_COMBO);
        if held {
            self.reset_armed = true;
            return;
        }
        if self.reset_armed {
            self.reset_armed = false;
            plog!("gw: pad: L+R+Y+X+Start released - recalibrating all four channels");
            pad_recalibrate(0xF000_0000);
            self.reset_combo_fired += 1;
        }
    }

    /// `PADControlMotor`: adapter ports rumble through the adapter; gamepad rumble is not
    /// implemented yet (Aurora drives SDL rumble).
    pub fn control_motor(&self, chan: usize, cmd: u32) {
        if chan < PAD_CHANMAX && self.sources[chan] == Source::Adapter {
            if let Some(a) = self.adapter.as_ref() {
                a.rumble(chan, cmd == PAD_MOTOR_RUMBLE);
            }
        }
    }
}

/// Aurora `PADRecalibrate`: returns true, does nothing.
pub fn pad_recalibrate(_mask: u32) -> bool {
    true
}
/// Aurora `PADReset`: returns true, does nothing.
pub fn pad_reset(_mask: u32) -> bool {
    true
}
