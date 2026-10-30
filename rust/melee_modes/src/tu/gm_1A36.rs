//! src/melee/gm/gm_1A36.c: the scene-level controller map (`controller_map`: buttons, triggers, repeats and
//! releases per port plus the merged slot 4, with the menu-style virtual buttons PAD_CONFIRM, PAD_CANCEL,
//! PAD_LR_START, PAD_ANY_UP...). All 10 functions are checked bit for bit against the C by a state
//! oracle (tests/tu_gm_1A36.rs over gen/turef/tu_gm_1A36_ref.c): the whole `controller_map` is compared
//! after every call.
#![allow(non_snake_case, non_upper_case_globals, dead_code, static_mut_refs)]

use melee_types::*;

pub const PAD_MAX_CONTROLLERS: usize = 4;
pub const PAD_BUTTON_LEFT: u64 = 1 << 0;
pub const PAD_BUTTON_RIGHT: u64 = 1 << 1;
pub const PAD_BUTTON_DOWN: u64 = 1 << 2;
pub const PAD_BUTTON_UP: u64 = 1 << 3;
pub const PAD_TRIGGER_R: u64 = 1 << 5;
pub const PAD_TRIGGER_L: u64 = 1 << 6;
pub const PAD_BUTTON_A: u64 = 1 << 8;
pub const PAD_BUTTON_B: u64 = 1 << 9;
pub const PAD_BUTTON_START: u64 = 1 << 12;
pub const PAD_STICK_UP: u64 = 1 << 16;
pub const PAD_STICK_DOWN: u64 = 1 << 17;
pub const PAD_STICK_LEFT: u64 = 1 << 18;
pub const PAD_STICK_RIGHT: u64 = 1 << 19;
pub const PAD_CONFIRM: u64 = 1 << 32;
pub const PAD_CANCEL: u64 = 1 << 33;
pub const PAD_LR_START: u64 = 1 << 34;
pub const PAD_LRA_START: u64 = 1 << 35;
pub const PAD_ANY_UP: u64 = 1 << 36;
pub const PAD_ANY_DOWN: u64 = 1 << 37;
pub const PAD_ANY_LEFT: u64 = 1 << 38;
pub const PAD_ANY_RIGHT: u64 = 1 << 39;

/// `struct gm_controller_map` (gm_1A36.static.h), GameCube size 0x30.
#[repr(C)]
#[derive(Clone, Copy, Default, PartialEq, Debug)]
pub struct gm_controller_map {
    pub button: u64,
    /// Buttons pressed this frame.
    pub trigger: u64,
    pub repeat: u64,
    pub release: u64,
    pub repeat2: u64,
    pub repeat_timer: i32,
    pub x2C: i32,
}

melee_core::gc_layout!(gm_controller_map, "gm_controller_map", size 0x30, align 8, [
    button @ 0x0: 8 "u64",
    trigger @ 0x8: 8 "u64",
    repeat @ 0x10: 8 "u64",
    release @ 0x18: 8 "u64",
    repeat2 @ 0x20: 8 "u64",
    repeat_timer @ 0x28: 4 "s32",
    x2C @ 0x2C: 4 "s32",
]);

/// `static struct controller_map` (gm_1A36.static.h), GameCube size 0x100.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct controller_map_t {
    pub x0: [gm_controller_map; PAD_MAX_CONTROLLERS + 1],
    pub xF0: Option<fn(i32)>,
    pub xF4: u16,
    pub xF6: u8,
    pub xF8: u16,
    pub xFA: u8,
    pub xFC: u16,
    pub xFE: u8,
}

melee_core::gc_layout!(controller_map_t, "controller_map", size 0x100, align 8, [
    x0 @ 0x0: 0xF0 "struct gm_controller_map[5]",
    xF0 @ 0xF0: 4 "void (*)(int)",
    xF4 @ 0xF4: 2 "u16",
    xF6 @ 0xF6: 1 "u8",
    xF8 @ 0xF8: 2 "u16",
    xFA @ 0xFA: 1 "u8",
    xFC @ 0xFC: 2 "u16",
    xFE @ 0xFE: 1 "u8",
]);

/// `static struct controller_map controller_map;` (zeroed at boot).
pub static mut controller_map: controller_map_t = controller_map_t {
    x0: [gm_controller_map { button: 0, trigger: 0, repeat: 0, release: 0, repeat2: 0, repeat_timer: 0, x2C: 0 };
        PAD_MAX_CONTROLLERS + 1],
    xF0: None,
    xF4: 0,
    xF6: 0,
    xF8: 0,
    xFA: 0,
    xFC: 0,
    xFE: 0,
};

/// Calls into TUs that are not ported yet. Each is a settable hook; an unset hook panics.
pub mod ext {
    use super::*;
    macro_rules! hook {
        ($hook:ident, $name:ident, ($($a:ident: $t:ty),*) $(-> $r:ty)?, $port:path) => {
            pub static mut $hook: Option<fn($($t),*) $(-> $r)?> = None;
            #[allow(non_snake_case)]
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $hook } {
                    Some(f) => f($($a),*),
                    None => $port($($a),*),
                }
            }
        };
        ($hook:ident, $name:ident, ($($a:ident: $t:ty),*) $(-> $r:ty)?) => {
            pub static mut $hook: Option<fn($($t),*) $(-> $r)?> = None;
            #[allow(non_snake_case)]
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $hook } {
                    Some(f) => f($($a),*),
                    None => panic!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            }
        };
    }
    /// `HSD_PadCopyStatus` (sysdolphin/baselib/controller.c): the 4-port copy array.
    hook!(HSD_PADCOPYSTATUS, HSD_PadCopyStatus, () -> *mut HSD_PadStatus);
    /// `gm_GetAllGameModes()` (gm/gm_unsplit.c): the mode table, terminated by `kind == GM_COUNT`.
    pub static mut GM_GETALLGAMEMODES: Option<fn() -> *mut GameMode> = None;
            #[allow(non_snake_case)]
            pub fn gm_GetAllGameModes() -> *mut GameMode {
                match unsafe { GM_GETALLGAMEMODES } {
                    Some(f) => f(),
                    None => { return unsafe { crate::tu::gmscdata::gm_GetAllGameModes() as _ }; },
                }
            }
}

/// `gm_GetButtonsPressed(idx)`.
pub fn gm_GetButtonsPressed(idx: u8) -> u64 {
    melee_core::ct!("gm_GetButtonsPressed");
    unsafe { controller_map.x0[idx as usize].button }
}

/// `gm_GetButtonsTriggered(idx)`.
pub fn gm_GetButtonsTriggered(idx: u8) -> u64 {
    melee_core::ct!("gm_GetButtonsTriggered");
    unsafe { controller_map.x0[idx as usize].trigger }
}

/// `gm_801A36C0(idx)`: the repeat-filtered buttons.
pub fn gm_801A36C0(idx: u8) -> u64 {
    melee_core::ct!("gm_801A36C0");
    unsafe { controller_map.x0[idx as usize].repeat2 }
}

/// `gm_801A36E0(idx, arg1)`: set the repeat timer of one port, or of all four when `idx == 4`.
pub fn gm_801A36E0(idx: i32, arg1: i32) {
    melee_core::ct!("gm_801A36E0");
    unsafe {
        if idx == PAD_MAX_CONTROLLERS as i32 {
            for i in 0..PAD_MAX_CONTROLLERS {
                controller_map.x0[i].repeat_timer = arg1;
            }
            return;
        }
        controller_map.x0[idx as usize].repeat_timer = arg1;
    }
}

/// `gm_801A3714(idx, arg1, arg2)`: where any of the `arg1` buttons is set, also set `arg2`.
pub fn gm_801A3714(idx: i32, arg1: u64, arg2: u64) {
    melee_core::ct!("gm_801A3714");
    unsafe {
        let c = &mut controller_map.x0[idx as usize];
        if arg1 & c.button != 0 {
            c.button |= arg2;
        }
        if arg1 & c.repeat != 0 {
            c.repeat |= arg2;
        }
        if arg1 & c.trigger != 0 {
            c.trigger |= arg2;
        }
        if arg1 & c.release != 0 {
            c.release |= arg2;
        }
    }
}

/// `static gm_801A3820(idx, arg1, arg2)`: like gm_801A3714 but needs every `arg1` button held.
pub fn gm_801A3820(idx: i32, arg1: u64, arg2: u64) {
    melee_core::ct!("gm_801A3820");
    unsafe {
        let c = &mut controller_map.x0[idx as usize];
        if arg1 == (arg1 & c.button) {
            c.button |= arg2;
        }
        if arg1 == (arg1 & c.repeat) {
            c.repeat |= arg2;
        }
        if arg1 == (arg1 & c.button) {
            if arg1 & c.trigger != 0 {
                c.trigger |= arg2;
            }
        }
        if arg1 == (arg1 & c.button) {
            if arg1 & c.release != 0 {
                c.release |= arg2;
            }
        }
    }
}

/// `static fn_801A396C(idx)`: the held-button repeat generator (`repeat2`, `repeat_timer`, `x2C`).
pub fn fn_801A396C(idx: i32) {
    melee_core::ct!("fn_801A396C");
    unsafe {
        let m = &mut controller_map;
        let i = idx as usize;
        if m.x0[i].trigger != 0 || m.x0[i].release != 0 {
            m.x0[i].repeat2 = m.x0[i].trigger;
            m.x0[i].repeat_timer = m.xF4 as i32;
            m.x0[i].x2C = 0;
            return;
        }
        if m.x0[i].x2C < m.xFC as i32 {
            m.x0[i].x2C = m.x0[i].x2C.wrapping_add(1);
        }

        if m.x0[i].repeat_timer != 0 {
            m.x0[i].repeat_timer = m.x0[i].repeat_timer.wrapping_sub(1);
            m.x0[i].repeat2 = 0;
            return;
        }

        m.x0[i].repeat2 = m.x0[i].button;

        if m.x0[i].x2C >= m.xFC as i32 {
            m.x0[i].repeat_timer = m.xFE as i32;
        } else if m.x0[i].x2C >= m.xF8 as i32 {
            m.x0[i].repeat_timer = m.xFA as i32;
        } else {
            m.x0[i].repeat_timer = m.xF6 as i32;
        }
    }
}

/// `gm_EvaluateAllControllerInputs()`: copy the pad status into the map, derive the virtual buttons,
/// run the repeat generator per port and merge all four ports into slot 4.
pub fn gm_EvaluateAllControllerInputs() {
    melee_core::ct!("gm_EvaluateAllControllerInputs");
    unsafe {
        let pads = ext::HSD_PadCopyStatus();
        for i in 0..PAD_MAX_CONTROLLERS {
            let p = &*pads.add(i);
            controller_map.x0[i].button = p.button as u64;
            controller_map.x0[i].trigger = p.trigger as u64;
            controller_map.x0[i].repeat = p.repeat as u64;
            controller_map.x0[i].release = p.release as u64;
            let ii = i as i32;
            gm_801A3714(ii, PAD_BUTTON_A | PAD_BUTTON_START, PAD_CONFIRM);
            gm_801A3714(ii, PAD_BUTTON_B, PAD_CANCEL);
            gm_801A3820(ii, PAD_TRIGGER_L | PAD_TRIGGER_R | PAD_BUTTON_START, PAD_LR_START);
            gm_801A3820(ii, PAD_TRIGGER_L | PAD_TRIGGER_R | PAD_BUTTON_A | PAD_BUTTON_START, PAD_LRA_START);
            gm_801A3714(ii, PAD_BUTTON_UP | PAD_STICK_UP, PAD_ANY_UP);
            gm_801A3714(ii, PAD_BUTTON_DOWN | PAD_STICK_DOWN, PAD_ANY_DOWN);
            gm_801A3714(ii, PAD_BUTTON_LEFT | PAD_STICK_LEFT, PAD_ANY_LEFT);
            gm_801A3714(ii, PAD_BUTTON_RIGHT | PAD_STICK_RIGHT, PAD_ANY_RIGHT);
            (controller_map.xF0.expect("controller_map.xF0 is NULL"))(ii);
        }
        let all = &mut controller_map.x0[PAD_MAX_CONTROLLERS];
        all.button = 0;
        all.trigger = 0;
        all.repeat = 0;
        all.release = 0;
        all.repeat2 = 0;

        for i in 0..PAD_MAX_CONTROLLERS {
            let c = controller_map.x0[i];
            let all = &mut controller_map.x0[PAD_MAX_CONTROLLERS];
            all.button |= c.button;
            all.trigger |= c.trigger;
            all.repeat |= c.repeat;
            all.release |= c.release;
            all.repeat2 |= c.repeat2;
        }
    }
}

/// `gm_801A3E88()`: reset the map to its defaults (repeat tuning 0x14/8/0x28/4/0x64/2) and install
/// the repeat generator.
pub fn gm_801A3E88() {
    melee_core::ct!("gm_801A3E88");
    unsafe {
        // static struct controller_map gm_803DA788 = { { 0 }, NULL, 0x14, 0x8, 0x28, 0x4, 0x64, 0x2 };
        controller_map = controller_map_t {
            x0: [gm_controller_map::default(); PAD_MAX_CONTROLLERS + 1],
            xF0: None,
            xF4: 0x14,
            xF6: 0x8,
            xF8: 0x28,
            xFA: 0x4,
            xFC: 0x64,
            xFE: 0x2,
        };
        for i in 0..=PAD_MAX_CONTROLLERS {
            controller_map.x0[i].repeat_timer = controller_map.xF4 as i32;
        }
        controller_map.xF0 = Some(fn_801A396C);
    }
}

/// `gm_801A3EF4()`: run every game mode's `on_init`.
pub fn gm_801A3EF4() {
    melee_core::ct!("gm_801A3EF4");
    unsafe {
        let mut scene = ext::gm_GetAllGameModes();
        while (*scene).kind != GM_COUNT as u8 {
            if let Some(f) = (*scene).on_init {
                f();
            }
            scene = scene.add(1);
        }
    }
}
