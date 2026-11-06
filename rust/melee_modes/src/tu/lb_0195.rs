//! Port of src/melee/lb/lb_0195.c (the pad sampling-rate / game-tick timer: the periodic alarm
//! that renews the raw pad status, the per-slot "tick due" flags, and the reset-switch handler).
//! Checked against the C (whole TU, shipping contraction) by tests/tu_lb_0195.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use core::ffi::c_void;
use melee_types::*;

// BEGIN GENERATED EXT (rust/melee_modes/gen/cref/gen_tu.py; the callees of the C TU)
pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    use core::ffi::c_void;
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? $(= $def:path)?;)*) => {
            pub mod hook {
                use super::*;
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => { $crate::ext_dflt!([$($def)?] ($($a),*) $name) }
                }
            } )*
        };
    }
    ext_fns! {
    fn HSD_PadGetRawQueueCount() -> u8 = crate::tu::adapt::HSD_PadGetRawQueueCount__b9c435;
    fn HSD_PadGetResetSwitch() -> i32 = crate::tu::adapt::HSD_PadGetResetSwitch__f7c5ee;
    fn HSD_PadRenewCopyStatus() = crate::tu::adapt::HSD_PadRenewCopyStatus__300d10;
    fn HSD_PadRenewGameStatus() = crate::tu::adapt::HSD_PadRenewGameStatus__5366e1;
    fn HSD_PadRenewMasterStatus() = crate::tu::adapt::HSD_PadRenewMasterStatus__03a172;
    fn HSD_PadRenewRawStatus(a0: i32) = crate::tu::adapt::HSD_PadRenewRawStatus__ba4cff;
    fn OSCancelAlarm(a0: *mut OSAlarm);
    fn OSCreateAlarm(a0: *mut OSAlarm);
    fn OSDisableInterrupts() -> i32;
    fn OSResetSystem(a0: i32, a1: u32, a2: i32);
    fn OSRestoreInterrupts(a0: i32) -> i32;
    fn OSSetPeriodicAlarm(a0: *mut OSAlarm, a1: i64, a2: i64, a3: Option<unsafe extern "C" fn(*mut OSAlarm, *mut OSContext)>);
    fn PADSetSamplingRate(a0: u32);
    fn VIFlush();
    fn VISetBlack(a0: i32);
    fn VISetPostRetraceCallback(a0: Option<unsafe extern "C" fn(u32)>) -> Option<unsafe extern "C" fn(u32)>;
    fn VISetPreRetraceCallback(a0: Option<unsafe extern "C" fn(u32)>) -> Option<unsafe extern "C" fn(u32)>;
    fn VIWaitForRetrace();
    fn lbAudioAx_80027DBC() = crate::tu::adapt::lbAudioAx_80027DBC__2be6f4;
    fn lbCardNew_CompleteNextTask() -> i32 = crate::tu::adapt::lbCardNew_CompleteNextTask__92582d;
    fn lbSnap_8001D2BC() = crate::tu::adapt::lbSnap_8001D2BC__4abedd;
    fn lb_800192A8(a0: Option<unsafe extern "C" fn()>) = crate::tu::adapt::lb_800192A8__02b4e0;
    fn lb_8001C600() = crate::tu::adapt::lb_8001C600__203725;
    fn lb_8001CC84() = crate::tu::adapt::lb_8001CC84__76fb76;
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

/// `Event` (lb_0195.h): `void (*)(void)`.
pub type Event = Option<unsafe extern "C" fn()>;

/// `__OSBusClock`: the C reads guest memory `0x800000F8` (OSInit stores the 162 MHz bus clock
/// there; `OS_TIMER_CLOCK` is a quarter of it). Settable so the integrating crate can bind it.
pub static mut OS_BUS_CLOCK: u32 = 162_000_000;

#[inline(always)]
unsafe fn os_timer_clock() -> u32 {
    OS_BUS_CLOCK / 4
}

/// `struct UnkArrElem` (0x18)
#[repr(C)]
#[derive(Clone, Copy)]
pub struct UnkArrElem {
    pub x0: i64,
    pub x8: i64,
    pub x10: i32,
}
melee_core::gc_layout!(UnkArrElem, "UnkArrElem", size 0x18, align 8, [
    x0 @ 0x0: 8 "s64",
    x8 @ 0x8: 8 "s64",
    x10 @ 0x10: 4 "int",
]);

/// `struct lb_804329F0_t` (0x78 on the GC)
#[repr(C)]
#[derive(Clone, Copy)]
pub struct lb_804329F0_t {
    pub x0: [UnkArrElem; 2],
    pub x4: u32,
    pub x38: u64,
    pub x40: i64,
    pub x48: i32,
    pub alarm: OSAlarm,
}
melee_core::gc_layout!(lb_804329F0_t, "lb_804329F0_t", size 0x78, align 8, [
    x0 @ 0x0: 0x30 "UnkArrElem[2]",
    x4 @ 0x30: 4 "u32",
    x38 @ 0x38: 8 "u64",
    x40 @ 0x40: 8 "OSTime",
    x48 @ 0x48: 4 "int",
    alarm @ 0x50: 0x28 "OSAlarm",
]);

/// 4329F0 (static)
pub static mut lb_804329F0: lb_804329F0_t = unsafe { core::mem::zeroed() };

/// 01955C
pub unsafe extern "C" fn lb_8001955C() {
    melee_core::ct!("lb_8001955C");
    if ext::HSD_PadGetResetSwitch() != 0 {
        ext::lbAudioAx_80027DBC();
        while ext::lbCardNew_CompleteNextTask() == 11 {}
        ext::VISetPostRetraceCallback(None);
        ext::VISetPreRetraceCallback(None);
        ext::VISetBlack(1);
        ext::VIFlush();
        ext::VIWaitForRetrace();
        ext::VIWaitForRetrace();
        ext::OSResetSystem(0, 0, 0);
    }
    ext::lbCardNew_CompleteNextTask();
    ext::lb_8001CC84();
}

/// 0195D0
pub unsafe fn lb_800195D0() {
    melee_core::ct!("lb_800195D0");
    ext::lb_800192A8(Some(lb_8001955C));
    ext::lb_8001CC84();
}

/// 0195FC
pub unsafe extern "C" fn fn_800195FC() {
    melee_core::ct!("fn_800195FC");
    ext::HSD_PadRenewRawStatus(0);
    ext::lb_8001C600();
    ext::lbSnap_8001D2BC();
}

/// 019628: recomputes the sampling period from the two slot periods and (re)arms the alarm.
pub unsafe fn lb_80019628() {
    melee_core::ct!("lb_80019628");
    let new_val: i64 = lb_804329F0.x38 as i64;
    let tmp = &raw mut lb_804329F0.x0[0];

    if new_val == (*tmp).x0 {
        return;
    }

    (*tmp).x0 = new_val;

    if (*tmp).x8 >= (*tmp).x0 {
        (*tmp).x8 = 0;
    }

    // OSSecondsToTicks(1) is an unsigned int product; the (f32) cast rounds it
    let mut period: i64 = melee_core::ppc::fcvt_i64((os_timer_clock() as f32));

    for i in 0..2 {
        if lb_804329F0.x0[i].x0 < period {
            period = lb_804329F0.x0[i].x0;
        }
    }

    // OSSecondsToTicks(1.0F / 60) is a float product
    let frame_ticks: f32 = (1.0f32 / 60.0f32) * (os_timer_clock() as f32);
    if period >= melee_core::ppc::fcvt_i64(frame_ticks) {
        period = melee_core::ppc::fcvt_i64(frame_ticks);
    }

    if lb_804329F0.x40 == period {
        return;
    }

    lb_804329F0.x40 = period;

    {
        // OSTicksToMilliseconds: s64 / u32 (the u32 widens to s64)
        let mut rate: u32 = (<i64 as melee_core::ppc::Ppc>::pdiv(lb_804329F0.x40, (os_timer_clock() / 1000) as i64)) as u32;
        if rate > 11 {
            rate = 11;
        }
        if lb_804329F0.x4 != rate {
            ext::PADSetSamplingRate(rate);
            lb_804329F0.x4 = rate;
        }
    }

    if lb_804329F0.x48 != 0 {
        ext::OSCancelAlarm(&raw mut lb_804329F0.alarm);
    }
    ext::OSCreateAlarm(&raw mut lb_804329F0.alarm);
    ext::OSSetPeriodicAlarm(
        &raw mut lb_804329F0.alarm,
        lb_804329F0.x40,
        lb_804329F0.x40,
        // (OSAlarmHandler) fn_800195FC: a handler that ignores its arguments
        Some(core::mem::transmute::<unsafe extern "C" fn(), unsafe extern "C" fn(*mut OSAlarm, *mut OSContext)>(fn_800195FC)),
    );
    lb_804329F0.x48 = 1;
}

/// 019880
pub unsafe fn lb_80019880(arg0: u64) {
    melee_core::ct!("lb_80019880");
    lb_804329F0.x38 = arg0;
}

/// 019894
pub unsafe fn lb_80019894() -> u8 {
    melee_core::ct!("lb_80019894");
    let enabled = ext::OSDisableInterrupts();
    let count = ext::HSD_PadGetRawQueueCount();
    lb_80019628();
    ext::OSRestoreInterrupts(enabled);
    count
}

/// 0198E0
pub unsafe fn lb_800198E0() {
    melee_core::ct!("lb_800198E0");
    ext::HSD_PadRenewMasterStatus();
}

/// 019900
pub unsafe fn lb_80019900() {
    melee_core::ct!("lb_80019900");
    for i in 0..2 {
        lb_804329F0.x0[i].x8 = lb_804329F0.x0[i].x8.wrapping_add(lb_804329F0.x40);
        if lb_804329F0.x0[i].x8 >= lb_804329F0.x0[i].x0 {
            lb_804329F0.x0[i].x8 = lb_804329F0.x0[i].x8.wrapping_sub(lb_804329F0.x0[i].x0);
            lb_804329F0.x0[i].x10 = 1;
        } else {
            lb_804329F0.x0[i].x10 = 0;
        }
    }

    if lb_80019A30(0) {
        ext::HSD_PadRenewGameStatus();
    }
    if lb_80019A30(0) {
        ext::HSD_PadRenewCopyStatus();
    }
}

/// 019A30
pub unsafe fn lb_80019A30(index: i32) -> bool {
    melee_core::ct!("lb_80019A30");
    lb_804329F0.x0[index as usize].x10 != 0
}

/// 019A48
pub unsafe fn lb_80019A48() {
    melee_core::ct!("lb_80019A48");
    let enabled = ext::OSDisableInterrupts();

    if lb_804329F0.x48 != 0 {
        ext::OSCancelAlarm(&raw mut lb_804329F0.alarm);
        lb_804329F0.x48 = 0;
    }
    ext::OSRestoreInterrupts(enabled);
}

/// 019AAC
pub unsafe fn lb_80019AAC(arg0: Event) {
    melee_core::ct!("lb_80019AAC");
    (arg0.unwrap())();

    let frame_ticks: f32 = (1.0f32 / 60.0f32) * (os_timer_clock() as f32);
    for i in 0..2 {
        let cur = &raw mut lb_804329F0.x0[i];
        (*cur).x0 = melee_core::ppc::fcvt_i64(frame_ticks);
        (*cur).x8 = 0;
        (*cur).x10 = 0;
    }

    lb_804329F0.x4 = 0;
    lb_804329F0.x48 = 0;
    lb_804329F0.x0[0].x0 = 0; // huh? overwritten?
    lb_804329F0.x40 = 0;
    lb_804329F0.x38 = melee_core::ppc::fcvt_u64(frame_ticks);

    lb_80019628();
}
