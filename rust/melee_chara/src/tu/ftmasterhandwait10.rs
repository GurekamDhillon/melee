//! src/melee/ft/kinds/ftMasterHand/ftmasterhandwait10.c: Master Hand idle (Wait1_0) and the
//! attack chooser. Float math: `da->xCC_pos.x * rand * 2 + da->xC4_pos.x - da->xCC_pos.x` is one
//! fused site (`fmuladd(xCC*rand, 2, xC4)` = `fmadds`, then a plain subtract); the
//! `--x223C < 0` countdown is a float decrement. Smoke-tested by tests/tu_ftmasterhandwait10.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_assignments)]

use crate::stubs as st;
use melee_core::fp::fmadds;
use melee_types::*;

/// Already ported (rounds 2/4/8/10): re-exported under this TU.
pub use crate::chars::r10::ftMasterHand::ftmasterhandwait10::ifStage251;
pub use crate::chars::r2::ftMasterHand::ftmasterhandwait10::ftMh_Wait1_0_Phys;
pub use crate::chars::r4::ftMasterHand::ftmasterhandwait10::{
    doAnim0, doAnim1, ftMh_MS_341_8014FE5C, ftMh_MS_341_8014FF1C, ftMh_MS_341_8014FFDC,
};
pub use crate::chars::r8::ftMasterHand::ftmasterhandwait10::ftMh_Wait1_0_IASA;

/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {
    use melee_types::*;
    macro_rules! hook {
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
    // `Stage_80225194()` (gr/stage.c).
    pub static mut STAGE_80225194: Option<fn() -> i32> = None;
    #[allow(non_snake_case)]
    pub fn Stage_80225194() -> i32 {
        match unsafe { STAGE_80225194 } {
            Some(f) => f(),
            None => { unsafe { melee_world::tu::stage::Stage_80225194() as _ } },
        }
    }
    // `mpFloorGetRight(line, out)` (mp/mplib.c).
    pub static mut MPFLOORGETRIGHT: Option<fn(i32, *mut Vec3)> = None;
    #[allow(non_snake_case)]
    pub fn mpFloorGetRight(line: i32, out: *mut Vec3) {
        match unsafe { MPFLOORGETRIGHT } {
            Some(f) => f(line, out),
            None => { unsafe { melee_world::tu::mplib::mpFloorGetRight(line as _, out as _); } },
        }
    }
    // `HSD_PadMasterStatus[port].button` (baselib/controller.c).
    crate::port_hook!(HSD_PADMASTERSTATUS_BUTTON, HSD_PadMasterStatus_button, crate::adapt::HSD_PadMasterStatus_button, (port: i32) -> u32);
    crate::port_hook!(HSD_RANDI, HSD_Randi, crate::adapt::HSD_Randi, (max: i32) -> i32);
    crate::port_hook!(HSD_RANDF, HSD_Randf, crate::adapt::HSD_Randf, () -> f32);
    // ftmasterhandentry.c
    pub static mut FTMH_MS_343_801510B0: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn ftMh_MS_343_801510B0(gobj: *mut HSD_GObj)  {
                match unsafe { FTMH_MS_343_801510B0 } {
                    Some(f) => f(gobj),
                    None => { unsafe { crate::tu::ftmasterhandentry::ftMh_MS_343_801510B0(gobj as _); } return; },
                }
            }
    // ftmasterhandpoke.c
    pub static mut FTMH_MS_358_80152880: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn ftMh_MS_358_80152880(gobj: *mut HSD_GObj)  {
                match unsafe { FTMH_MS_358_80152880 } {
                    Some(f) => f(gobj),
                    None => { unsafe { crate::tu::ftmasterhandpoke::ftMh_MS_358_80152880(gobj as _); } return; },
                }
            }
    // ftmasterhandfingergun.c
    pub static mut FTMH_MS_363_801530A4: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn ftMh_MS_363_801530A4(gobj: *mut HSD_GObj)  {
                match unsafe { FTMH_MS_363_801530A4 } {
                    Some(f) => f(gobj),
                    None => { unsafe { crate::tu::ftmasterhandfingergun::ftMh_MS_363_801530A4(gobj as _); } return; },
                }
            }
    // ftmasterhandbackcrush*.c
    pub static mut FTMH_MS_371_801541C8: Option<fn(*mut HSD_GObj, HSD_GObjEvent) > = None;
            #[allow(non_snake_case)]
            pub fn ftMh_MS_371_801541C8(gobj: *mut HSD_GObj, cb: HSD_GObjEvent)  {
                match unsafe { FTMH_MS_371_801541C8 } {
                    Some(f) => f(gobj, cb),
                    None => { unsafe { crate::tu::ftmasterhandbackcrush1::ftMh_MS_371_801541C8(gobj as _, cb as _); } return; },
                }
            }
    pub static mut FTMH_MS_369_80153B90: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn ftMh_MS_369_80153B90(gobj: *mut HSD_GObj)  {
                match unsafe { FTMH_MS_369_80153B90 } {
                    Some(f) => f(gobj),
                    None => { unsafe { crate::tu::ftmasterhandbackairplane3::ftMh_MS_369_80153B90(gobj as _); } return; },
                }
            }
    pub static mut FTMH_MS_370_80153D2C: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn ftMh_MS_370_80153D2C(gobj: *mut HSD_GObj)  {
                match unsafe { FTMH_MS_370_80153D2C } {
                    Some(f) => f(gobj),
                    None => { unsafe { crate::tu::ftmasterhandbackcrush0::ftMh_MS_370_80153D2C(gobj as _); } return; },
                }
            }
    // ftmasterhandgrab / throw start
    pub static mut FTMH_MS_372_801542E0: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn ftMh_MS_372_801542E0(gobj: *mut HSD_GObj)  {
                match unsafe { FTMH_MS_372_801542E0 } {
                    Some(f) => f(gobj),
                    None => { unsafe { crate::tu::ftmasterhandbackdisappear::ftMh_MS_372_801542E0(gobj as _); } return; },
                }
            }
    // ftmasterhandslam.c / ftmasterhandwait11-12 combos
    pub static mut FTMH_MS_380_80155194: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn ftMh_MS_380_80155194(gobj: *mut HSD_GObj)  {
                match unsafe { FTMH_MS_380_80155194 } {
                    Some(f) => f(gobj),
                    None => { unsafe { crate::tu::ftmasterhandslam::ftMh_MS_380_80155194(gobj as _); } return; },
                }
            }
    pub static mut FTMH_MS_383_80155484: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn ftMh_MS_383_80155484(gobj: *mut HSD_GObj)  {
                match unsafe { FTMH_MS_383_80155484 } {
                    Some(f) => f(gobj),
                    None => { unsafe { crate::tu::ftmasterhandtagapplaud::ftMh_MS_383_80155484(gobj as _); } return; },
                }
            }
}

macro_rules! cb_wrap {
    ($(#[$m:meta])* $cb:ident, $path:path) => {
        $(#[$m])*
        pub unsafe extern "C" fn $cb(g: *mut HSD_GObj) {
            $path(g)
        }
    };
}
/// Fighter-callback entries of the attack-start functions the chooser hands to
/// `ftMh_MS_389_80150DC4` / `ftMh_MS_371_801541C8` (each is the C function of that name).
pub mod cbs {
    use melee_types::HSD_GObj;
    cb_wrap!(ftMh_MS_344_80151828, crate::chars::r2::ftMasterHand::ftmasterhandsweep::ftMh_MS_344_80151828);
    cb_wrap!(ftMh_MS_348_80151BB8, crate::chars::r2::ftMasterHand::ftmasterhandsweepwait::ftMh_MS_348_80151BB8);
    cb_wrap!(ftMh_MS_353_80151DC4, crate::chars::r2::ftMasterHand::ftmasterhandwalk::ftMh_MS_353_80151DC4);
    cb_wrap!(ftMh_MS_352_801521DC, crate::chars::r3::ftMasterHand::ftmasterhanddrill::ftMh_MS_352_801521DC);
    cb_wrap!(ftMh_MS_354_80152370, crate::chars::r2::ftMasterHand::ftmasterhandrockcrush::ftMh_MS_354_80152370);
    cb_wrap!(ftMh_MS_357_801526D8, crate::chars::r3::ftMasterHand::ftmasterhandpapercrush::ftMh_MS_357_801526D8);
    cb_wrap!(ftMh_MS_359_80152BCC, crate::chars::r3::ftMasterHand::ftmasterhandfingerbeam::ftMh_MS_359_80152BCC);
    cb_wrap!(ftMh_MS_380_80155194, super::ext::ftMh_MS_380_80155194);
    cb_wrap!(ftMh_MS_382_801552F8, crate::tu::ftmasterhandtagcrush::ftMh_MS_382_801552F8);
    cb_wrap!(ftMh_MS_383_80155484, super::ext::ftMh_MS_383_80155484);
    cb_wrap!(ftMh_MS_369_80153B90, super::ext::ftMh_MS_369_80153B90);
    cb_wrap!(ftMh_MS_370_80153D2C, super::ext::ftMh_MS_370_80153D2C);
}

#[derive(Clone, Copy)]
pub struct U8Pair {
    pub a: u8,
    pub b: u8,
}

/// `struct MasterHandDataStuff`: the attack-chooser tables (states, x38, x48, x54).
pub struct MasterHandDataStuff {
    pub states: [u32; 14],
    pub x38: [u8; 16],
    pub x48: [U8Pair; 6],
    pub x54: [u8; 30],
}
impl MasterHandDataStuff {
    /// GameCube field offsets (ppc32 layout of the C struct).
    pub const GC_FIELDS: &'static [(&'static str, u32)] =
        &[("states", 0x0), ("x38", 0x38), ("x48", 0x48), ("x54", 0x54)];
    pub const GC_SIZE: u32 = 0x74;
}

pub static ftMh_Init_803D40D0: MasterHandDataStuff = MasterHandDataStuff {
    states: [
        ftMh_MS_Slap, ftMh_MS_SweepLoop, ftMh_MS_WalkLoop, ftMh_MS_RockCrushUp,
        ftMh_MS_PaperCrush, ftMh_MS_Poke1, ftMh_MS_Drill, ftMh_MS_BackPunch,
        ftMh_MS_BackCrush, ftMh_MS_BackAirplane1, ftMh_MS_FingerBeamLoop,
        ftMh_MS_FingerGun2, ftMh_MS_Throw, ftMh_MS_Slam,
    ],
    x38: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 0, 0],
    x48: [
        U8Pair { a: 0, b: 3 }, U8Pair { a: 3, b: 4 }, U8Pair { a: 7, b: 3 },
        U8Pair { a: 10, b: 2 }, U8Pair { a: 12, b: 2 }, U8Pair { a: 0, b: 0 },
    ],
    x54: [
        1, 2, 3, 4, 0, 0, 2, 3, 4, 0, 0, 1, 3, 4, 0,
        0, 1, 2, 4, 0, 0, 1, 2, 3, 0, 0, 0, 0, 0, 0,
    ],
};

pub fn ftMh_MS_341_8014FE10(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_MS_341_8014FE10");
    if ext::Stage_80225194() == 251 {
        ifStage251(gobj);
    } else {
        ext::ftMh_MS_343_801510B0(gobj);
    }
}

pub fn ftMh_MS_341_8014FE58(_gobj: *mut HSD_GObj) { melee_core::ct!("ftMh_MS_341_8014FE58");}

#[inline(always)]
unsafe fn start(gobj: *mut HSD_GObj, cb: unsafe extern "C" fn(*mut HSD_GObj), v: &Vec3) {
    crate::tu::ftmasterhandwait12::ftMh_MS_389_80150DC4(gobj, Some(cb), v);
}

pub fn ftMh_Wait1_0_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_Wait1_0_Anim");
    use melee_fighter::tu::ftbosslib as bl;
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let data = &ftMh_Init_803D40D0;
        if st::Player_GetPlayerSlotType((*fp).player_id as i32) == 0 {
            ftMh_MS_341_80150894(gobj);
            if st::ftAnim_IsFramesRemaining(gobj) == 0 {
                if (*fp).motion_id as i32 == ftMh_MS_Wait1_0 as i32 {
                    doAnim0(gobj);
                } else if (*fp).motion_id as i32 == ftMh_MS_Wait2_0 as i32 {
                    doAnim1(gobj);
                }
            }
        } else {
            (*fp).u.mh.x223C -= 1.0;
            if (*fp).u.mh.x223C < 0.0 {
                let da = (*(*fp).ft_data).ext_attr as *const ftMasterHand_SpecialAttrs;
                let mut vec = Vec3::zeroed();

                bl::ftBossLib_8015BD24(
                    (*fp).cpu.level as i32,
                    &mut (*fp).u.mh.x223C,
                    (*fp).u.mh.x2238,
                    (*da).x18,
                    (*da).x20,
                    (*da).x1C,
                );
                if bl::ftBossLib_GetMotionId(Ft_Kind_CrezyH) == ftMh_MS_TagRockPaper as i32 {
                    // Crazy Hand Combo Attack
                    match bl::ftBossLib_8015C4C4() {
                        x if x == ftMh_MS_Squeezing1 as i32 => {
                            vec.x = (*da).x124_pos.x;
                            vec.y = (*da).x124_pos.y;
                            vec.z = 0.0;
                            start(gobj, cbs::ftMh_MS_380_80155194, &vec);
                        }
                        x if x == ftMh_MS_Squeeze as i32 => {
                            vec.x = (*da).x12C_pos.x;
                            vec.y = (*da).x12C_pos.y;
                            vec.z = 0.0;
                            start(gobj, cbs::ftMh_MS_382_801552F8, &vec);
                        }
                        x if x == ftMh_MS_Throw as i32 => {
                            vec.x = (*da).x134_pos.x;
                            vec.y = (*da).x134_pos.y;
                            vec.z = 0.0;
                            start(gobj, cbs::ftMh_MS_383_80155484, &vec);
                        }
                        x if x == ftMh_MS_Slam as i32 => {
                            vec.x = (*da).x13C_pos.x;
                            vec.y = (*da).x13C_pos.y;
                            vec.z = 0.0;
                        }
                        _ => {}
                    }
                } else {
                    let tmp: i32;
                    if (*fp).u.mh.x2254 == (*da).x24 {
                        (*fp).u.mh.x2254 = 0;
                        tmp = 2;
                    } else {
                        let qwe4 = data.x54[((*fp).u.mh.x224C as i32 * 5 + ext::HSD_Randi(5)) as usize] as i32;
                        tmp = qwe4;
                        if qwe4 == 2 {
                            (*fp).u.mh.x2254 = 0;
                        } else {
                            (*fp).u.mh.x2254 += 1;
                        }
                    }
                    let qwe = data.x48[tmp as usize];
                    let mut tmp2 = data.states[data.x38[(qwe.a as i32 + ext::HSD_Randi(qwe.b as i32)) as usize] as usize];

                    (*fp).u.mh.x224C = tmp as u32;
                    (*fp).u.mh.x2250 = tmp2 as i32;
                    if (*fp).x221D_b4() != 0 {
                        tmp2 = ftMh_MS_Wait1_0;
                    }
                    match tmp2 {
                        ftMh_MS_SweepLoop => {
                            // Stage-Wide Slap
                            ext::mpFloorGetRight(0, &mut vec);
                            vec.y = (*da).x38;
                            start(gobj, cbs::ftMh_MS_344_80151828, &vec);
                        }
                        ftMh_MS_Slap => {
                            // Sweep
                            ext::mpFloorGetRight(0, &mut vec);
                            vec.y = (*da).x50.y;
                            start(gobj, cbs::ftMh_MS_348_80151BB8, &vec);
                        }
                        ftMh_MS_WalkLoop => {
                            vec.x = (*da).x40_pos.x;
                            vec.y = (*da).x40_pos.y;
                            vec.z = 0.0;
                            start(gobj, cbs::ftMh_MS_353_80151DC4, &vec);
                        }
                        ftMh_MS_Drill => {
                            vec.x = (*da).x88_pos.x;
                            vec.y = (*da).x88_pos.y;
                            vec.z = 0.0;
                            start(gobj, cbs::ftMh_MS_352_801521DC, &vec);
                        }
                        ftMh_MS_RockCrushUp => {
                            // Punch
                            vec.x = (*da).xA8_pos.x;
                            vec.y = (*da).xA8_pos.y;
                            vec.z = 0.0;
                            start(gobj, cbs::ftMh_MS_354_80152370, &vec);
                        }
                        ftMh_MS_PaperCrush => {
                            // Ground Slap
                            vec.x = (*da).xBC_pos.x;
                            vec.y = (*da).xBC_pos.y;
                            vec.z = 0.0;
                            start(gobj, cbs::ftMh_MS_357_801526D8, &vec);
                        }
                        ftMh_MS_Poke1 => ext::ftMh_MS_358_80152880(gobj),
                        ftMh_MS_BackAirplane1 => {
                            crate::chars::r2::ftMasterHand::ftmasterhandfingergun3::ftMh_MS_366_80153820(gobj)
                        }
                        ftMh_MS_BackPunch => ext::ftMh_MS_371_801541C8(gobj, Some(cbs::ftMh_MS_369_80153B90)),
                        ftMh_MS_BackCrush => ext::ftMh_MS_371_801541C8(gobj, Some(cbs::ftMh_MS_370_80153D2C)),
                        ftMh_MS_FingerBeamLoop => {
                            // Lasers
                            let mut rand = ext::HSD_Randf();
                            vec.x = fmadds((*da).xCC_pos.x * rand, 2.0, (*da).xC4_pos.x) - (*da).xCC_pos.x;
                            rand = ext::HSD_Randf();
                            vec.y = fmadds((*da).xCC_pos.y * rand, 2.0, (*da).xC4_pos.y) - (*da).xCC_pos.y;
                            vec.z = 0.0;
                            start(gobj, cbs::ftMh_MS_359_80152BCC, &vec);
                        }
                        ftMh_MS_FingerGun2 => ext::ftMh_MS_363_801530A4(gobj),
                        ftMh_MS_Throw => ext::ftMh_MS_372_801542E0(gobj),
                        ftMh_MS_Slam => ext::ftMh_MS_372_801542E0(gobj),
                        _ => {}
                    }
                }
            } else if st::ftAnim_IsFramesRemaining(gobj) == 0 {
                if (*fp).motion_id as i32 == ftMh_MS_Wait1_0 as i32 {
                    doAnim0(gobj);
                } else if (*fp).motion_id as i32 == ftMh_MS_Wait2_0 as i32 {
                    doAnim1(gobj);
                }
            }
        }
    }
}

pub fn ftMh_Wait1_0_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftMh_Wait1_0_Coll");}

/// Debug pad chooser (`buttons = HSD_PadMasterStatus[2].button`): port 3 picks an attack directly.
pub fn ftMh_MS_341_80150894(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_MS_341_80150894");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let da = (*(*fp).ft_data).ext_attr as *const ftMasterHand_SpecialAttrs;
        let b = ext::HSD_PadMasterStatus_button(2) as i32;
        let mut pos = Vec3::zeroed();
        let has = |m: i32| b & m != 0;
        let l_pressed = has(HSD_PAD_L);

        if l_pressed && has(HSD_PAD_DPADUP) {
            ext::mpFloorGetRight(0, &mut pos);
            pos.y = (*da).x38;
            start(gobj, cbs::ftMh_MS_344_80151828, &pos);
        } else if l_pressed && has(HSD_PAD_DPADRIGHT) {
            ext::mpFloorGetRight(0, &mut pos);
            pos.y = (*da).x50.y;
            start(gobj, cbs::ftMh_MS_348_80151BB8, &pos);
        } else if l_pressed && has(HSD_PAD_DPADDOWN) {
            pos.x = (*da).x40_pos.x;
            pos.y = (*da).x40_pos.y;
            pos.z = 0.0;
            start(gobj, cbs::ftMh_MS_353_80151DC4, &pos);
        } else if has(HSD_PAD_R) && has(HSD_PAD_DPADUP) {
            pos.x = (*da).x88_pos.x;
            pos.y = (*da).x88_pos.y;
            pos.z = 0.0;
            start(gobj, cbs::ftMh_MS_352_801521DC, &pos);
        } else if has(HSD_PAD_R) && has(HSD_PAD_DPADRIGHT) {
            pos.x = (*da).xA8_pos.x;
            pos.y = (*da).xA8_pos.y;
            pos.z = 0.0;
            start(gobj, cbs::ftMh_MS_354_80152370, &pos);
        } else if has(HSD_PAD_R) && has(HSD_PAD_DPADDOWN) {
            pos.x = (*da).xBC_pos.x;
            pos.y = (*da).xBC_pos.y;
            pos.z = 0.0;
            start(gobj, cbs::ftMh_MS_357_801526D8, &pos);
        } else if has(HSD_PAD_R) && has(HSD_PAD_DPADLEFT) {
            ext::ftMh_MS_358_80152880(gobj);
        } else if has(HSD_PAD_A) && has(HSD_PAD_DPADUP) {
            crate::chars::r2::ftMasterHand::ftmasterhandfingergun3::ftMh_MS_366_80153820(gobj);
        } else if has(HSD_PAD_A) && has(HSD_PAD_DPADRIGHT) {
            ext::ftMh_MS_371_801541C8(gobj, Some(cbs::ftMh_MS_369_80153B90));
        } else if has(HSD_PAD_A) && has(HSD_PAD_DPADDOWN) {
            ext::ftMh_MS_371_801541C8(gobj, Some(cbs::ftMh_MS_370_80153D2C));
        } else if has(HSD_PAD_B) && has(HSD_PAD_DPADUP) {
            let mut rand = ext::HSD_Randf();
            pos.x = fmadds((*da).xCC_pos.x * rand, 2.0, (*da).xC4_pos.x) - (*da).xCC_pos.x;
            rand = ext::HSD_Randf();
            pos.y = fmadds((*da).xCC_pos.y * rand, 2.0, (*da).xC4_pos.y) - (*da).xCC_pos.y;
            pos.z = 0.0;
            start(gobj, cbs::ftMh_MS_359_80152BCC, &pos);
        } else if has(HSD_PAD_B) && has(HSD_PAD_DPADRIGHT) {
            ext::ftMh_MS_363_801530A4(gobj);
        } else if has(HSD_PAD_Z) && has(HSD_PAD_DPADUP) {
            (*fp).u.mh.x2250 = ftMh_MS_Throw as i32;
            ext::ftMh_MS_372_801542E0(gobj);
        } else if has(HSD_PAD_Z) && has(HSD_PAD_DPADRIGHT) {
            (*fp).u.mh.x2250 = ftMh_MS_Slam as i32;
            ext::ftMh_MS_372_801542E0(gobj);
        } else if has(HSD_PAD_Y) && has(HSD_PAD_DPADUP) {
            // Crazy Hand Combo Attack
            match melee_fighter::tu::ftbosslib::ftBossLib_8015C4C4() {
                x if x == ftMh_MS_Squeezing1 as i32 => {
                    pos.x = (*da).x124_pos.x;
                    pos.y = (*da).x124_pos.y;
                    pos.z = 0.0;
                    start(gobj, cbs::ftMh_MS_380_80155194, &pos);
                }
                x if x == ftMh_MS_Squeeze as i32 => {
                    pos.x = (*da).x12C_pos.x;
                    pos.y = (*da).x12C_pos.y;
                    pos.z = 0.0;
                    start(gobj, cbs::ftMh_MS_382_801552F8, &pos);
                }
                x if x == ftMh_MS_Throw as i32 => {
                    pos.x = (*da).x134_pos.x;
                    pos.y = (*da).x134_pos.y;
                    pos.z = 0.0;
                    start(gobj, cbs::ftMh_MS_383_80155484, &pos);
                }
                x if x == ftMh_MS_Slam as i32 => {
                    pos.x = (*da).x13C_pos.x;
                    pos.y = (*da).x13C_pos.y;
                    pos.z = 0.0;
                }
                _ => {}
            }
        }
    }
}
