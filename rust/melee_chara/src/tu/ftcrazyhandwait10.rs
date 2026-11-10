//! src/melee/ft/kinds/ftCrazyHand/ftcrazyhandwait10.c: Crazy Hand idle (Wait1_0) and the
//! attack chooser. Float math: `da->x90 * rand * 2 + da->x88_pos.x - da->x90` is one fused
//! site (`fmuladd(x90*rand, 2, x88)` = `fmadds`, then a plain subtract); the `--x223C < 0`
//! countdown is a float decrement. Smoke-tested by tests/tu_ftcrazyhandwait10.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types::{Fighter};
use crate::stubs as st;
use melee_core::fp::fmadds;
use melee_types::*;

/// Already ported (rounds 2/4/8/9): re-exported under this TU.





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
                    None => melee_core::unported!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            }
        };
    }
    pub static mut FTCH_GRABUNK1_8015B174: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn ftCh_GrabUnk1_8015B174(gobj: *mut HSD_GObj)  {
                match unsafe { FTCH_GRABUNK1_8015B174 } {
                    Some(f) => f(gobj),
                    None => { unsafe { crate::tu::ftcrazyhandgrabunk1b174::ftCh_GrabUnk1_8015B174(gobj as _); } return; },
                }
            }
    // `ftCh_GrabUnk1_8015BA34(gobj, cb, pos)` (ftcrazyhandtagcancel.c).
    pub static mut FTCH_GRABUNK1_8015BA34: Option<fn(*mut HSD_GObj, HSD_GObjEvent, *const Vec3) > = None;
            #[allow(non_snake_case)]
            pub fn ftCh_GrabUnk1_8015BA34(gobj: *mut HSD_GObj, cb: HSD_GObjEvent, pos: *const Vec3)  {
                match unsafe { FTCH_GRABUNK1_8015BA34 } {
                    Some(f) => f(gobj, cb, pos),
                    None => { unsafe { crate::tu::ftcrazyhandtagcancel::ftCh_GrabUnk1_8015BA34(gobj as _, cb as _, pos as _); } return; },
                }
            }
    pub static mut FTCH_INIT_80157DF8: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn ftCh_Init_80157DF8(gobj: *mut HSD_GObj)  {
                match unsafe { FTCH_INIT_80157DF8 } {
                    Some(f) => f(gobj),
                    None => { unsafe { crate::tu::ftcrazyhandrockcrush0::ftCh_Init_80157DF8(gobj as _); } return; },
                }
            }
    crate::port_hook!(FTCH_INIT_80158B3C, ftCh_Init_80158B3C, crate::tu::ftcrazyhandfingerbeam::ftCh_Init_80158B3C, (gobj: *mut HSD_GObj));
    // `Stage_80225194()` (gr/stage.c).
    pub static mut STAGE_80225194: Option<fn() -> i32> = None;
    #[allow(non_snake_case)]
    pub fn Stage_80225194() -> i32 {
        match unsafe { STAGE_80225194 } {
            Some(f) => f(),
            None => { unsafe { melee_world::tu::stage::Stage_80225194() as _ } },
        }
    }
    // `mpFloorGetLeft(line, out)` (mp/mplib.c).
    pub static mut MPFLOORGETLEFT: Option<fn(i32, *mut Vec3)> = None;
    #[allow(non_snake_case)]
    pub fn mpFloorGetLeft(line: i32, out: *mut Vec3) {
        match unsafe { MPFLOORGETLEFT } {
            Some(f) => f(line, out),
            None => { unsafe { melee_world::tu::mplib::mpFloorGetLeft(line as _, out as _); } },
        }
    }
    // `HSD_PadMasterStatus[port].button` (baselib/controller.c).
    crate::port_hook!(HSD_PADMASTERSTATUS_BUTTON, HSD_PadMasterStatus_button, crate::adapt::HSD_PadMasterStatus_button, (port: i32) -> u32);
    crate::port_hook!(HSD_RANDI, HSD_Randi, crate::adapt::HSD_Randi, (max: i32) -> i32);
    crate::port_hook!(HSD_RANDF, HSD_Randf, crate::adapt::HSD_Randf, () -> f32);
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
/// `ftCh_GrabUnk1_8015BA34`, one per target (each is the C function of that name).
pub mod cbs {
    use melee_types::HSD_GObj;
    cb_wrap!(fn_80157080, crate::chars::r2::ftCrazyHand::ftcrazyhanddamage0::fn_80157080);
    cb_wrap!(fn_80156F6C, crate::chars::r9::ftCrazyHand::ftcrazyhandentry::fn_80156F6C);
    cb_wrap!(fn_8015746C, crate::chars::r2::ftCrazyHand::ftcrazyhandsweep::fn_8015746C);
    cb_wrap!(fn_8015755C, crate::chars::r2::ftCrazyHand::ftcrazyhandsweepwait::fn_8015755C);
    cb_wrap!(fn_801578E8, crate::chars::r2::ftCrazyHand::ftcrazyhandwalk::fn_801578E8);
    cb_wrap!(fn_80157C50, crate::chars::r3::ftCrazyHand::ftcrazyhanddrill::fn_80157C50);
    cb_wrap!(fn_80158144, crate::chars::r3::ftCrazyHand::ftcrazyhandrockcrush1::fn_80158144);
    cb_wrap!(fn_801582D8, crate::chars::r3::ftCrazyHand::ftcrazyhandpoke::fn_801582D8);
    cb_wrap!(fn_801587B0, crate::chars::r4::ftCrazyHand::ftcrazyhandfingerbeam0::fn_801587B0);
    cb_wrap!(fn_8015AAC8, crate::chars::r2::ftCrazyHand::ftcrazyhandtagrockpaper::fn_8015AAC8);
    cb_wrap!(fn_80159908, crate::tu::ftcrazyhandbackdisappear::fn_80159908);
    cb_wrap!(fn_80159AA4, crate::tu::ftcrazyhandwait11::fn_80159AA4);
}

#[derive(Clone, Copy)]
pub struct U8Pair {
    pub a: u8,
    pub b: u8,
}

#[allow(non_camel_case_types)]
/// `struct ftCh_Init_803D4878_t`: the attack-chooser tables.
pub struct ftCh_Init_803D4878_t {
    pub states: [u32; 18],
    pub x48: [u8; 20],
    pub x5C: [U8Pair; 6],
    pub x68: [u8; 32],
}

pub static ftCh_Init_803D4878: ftCh_Init_803D4878_t = ftCh_Init_803D4878_t {
    states: [
        ftMh_MS_Entry, ftMh_MS_Damage2, ftMh_MS_SweepLoop, ftMh_MS_Slap,
        ftMh_MS_WalkLoop, ftMh_MS_Drill, ftMh_MS_RockCrushUp,
        ftMh_MS_RockCrushDown, ftMh_MS_BackDisappear, ftMh_MS_Wait1_1,
        ftMh_MS_Grab, ftMh_MS_Poke1, ftMh_MS_FingerBeamStart,
        ftMh_MS_BackAirplane2, ftMh_MS_BackAirplane3, ftMh_MS_Squeezing1,
        ftMh_MS_Squeeze, ftMh_MS_Throw,
    ],
    x48: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 0, 0],
    x5C: [
        U8Pair { a: 0, b: 4 }, U8Pair { a: 4, b: 4 }, U8Pair { a: 8, b: 3 },
        U8Pair { a: 11, b: 2 }, U8Pair { a: 13, b: 2 }, U8Pair { a: 15, b: 3 },
    ],
    x68: [
        1, 2, 3, 4, 5, 0, 2, 3, 4, 5, 0, 1, 3, 4, 5, 0,
        1, 2, 4, 5, 0, 1, 2, 3, 5, 0, 1, 2, 3, 4, 0, 0,
    ],
};

pub fn ftCh_Init_80155FCC(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_Init_80155FCC");
    if ext::Stage_80225194() == 0xFB {
        ftCh_Init_80156310(gobj);
    } else {
        ext::ftCh_GrabUnk1_8015B174(gobj);
    }
}

pub fn ftCh_Init_80156014(_gobj: *mut HSD_GObj) { melee_core::ct!("ftCh_Init_80156014");}

/// `doAnim0(gobj)` (static inline).
unsafe fn doAnim0(gobj: *mut HSD_GObj) {
    let fp = (*gobj).user_data as *mut Fighter;
    if (*fp).motion_id as i32 == 0x155 || (*fp).motion_id as i32 == 0x156 {
        (*fp).cur_pos = (*fp).u.mh.x2240_pos;
    } else {
        (*fp).u.mh.x2240_pos = (*fp).cur_pos;
    }
    if (*fp).u.mh.x2258 == 0x185 {
        st::Fighter_ChangeMotionState(gobj, 0x156, 0, (*fp).cur_anim_frame, 1.0, 0.0, core::ptr::null_mut());
    } else {
        st::Fighter_ChangeMotionState(gobj, 0x156, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
    }
    (*fp).u.mh.x2258 = 0x156;
}

/// `doAnim1(gobj)` (static inline).
unsafe fn doAnim1(gobj: *mut HSD_GObj) {
    let fp = (*gobj).user_data as *mut Fighter;
    (*fp).cmd_vars[1] = 0;
    if (*fp).u.mh.x2258 == 0x185 {
        ftCh_Init_801560D8(gobj);
    } else {
        ftCh_Init_80156018(gobj);
    }
}

#[inline(always)]
unsafe fn start(gobj: *mut HSD_GObj, cb: unsafe extern "C" fn(*mut HSD_GObj), v: &Vec3) {
    ext::ftCh_GrabUnk1_8015BA34(gobj, Some(cb), v);
}

pub fn ftCh_Wait1_0_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_Wait1_0_Anim");
    use melee_fighter::tu::ftbosslib as bl;
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let data = &ftCh_Init_803D4878;
        let slot_type = st::Player_GetPlayerSlotType((*fp).player_idx as i32);

        if (*fp).cmd_vars[1] != 0 {
            bl::ftBossLib_8015C5F8(gobj);
            (*fp).cmd_vars[1] = 0;
        }
        if slot_type == 0 {
            ftCh_Init_80156AD8(gobj);
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
                let da = (*(*fp).ft_data).ext_attr as *const ftCrazyHand_DatAttrs;
                let mut vec = Vec3::zeroed();

                bl::ftBossLib_8015BD24(
                    (*fp).cpu.level,
                    &mut (*fp).u.mh.x223C,
                    (*fp).u.ch.x2238,
                    (*da).x0,
                    (*da).x8,
                    (*da).x4,
                );
                let tmp: i32;
                if (*fp).u.ch.x2254 == (*da).xC {
                    (*fp).u.ch.x2254 = 0;
                    tmp = 2;
                } else {
                    let base = (*fp).u.ch.x224C as i32 * 5;
                    if bl::ftBossLib_8015C2E0() {
                        tmp = data.x68[(base + ext::HSD_Randi(4)) as usize] as i32;
                    } else {
                        tmp = data.x68[(base + ext::HSD_Randi(5)) as usize] as i32;
                    }
                    if tmp == 2 {
                        (*fp).u.ch.x2254 = 0;
                    } else {
                        (*fp).u.ch.x2254 += 1;
                    }
                }
                let qwe = data.x5C[tmp as usize];
                let mut tmp2 = data.states[data.x48[(qwe.a as i32 + ext::HSD_Randi(qwe.b as i32)) as usize] as usize];

                (*fp).u.ch.x224C = tmp as _;
                (*fp).u.ch.x2250 = tmp2 as _;
                if (*fp).input_disabled() != 0 {
                    tmp2 = ftMh_MS_Wait1_0;
                }
                match tmp2 {
                    ftMh_MS_Damage2 => {
                        ext::mpFloorGetLeft(0, &mut vec);
                        vec.y = (*da).x24;
                        start(gobj, cbs::fn_80157080, &vec);
                    }
                    ftMh_MS_Entry => {
                        ext::mpFloorGetLeft(0, &mut vec);
                        vec.y = (*da).x20;
                        start(gobj, cbs::fn_80156F6C, &vec);
                    }
                    ftMh_MS_SweepLoop => {
                        vec.x = (*da).x2C;
                        vec.y = (*da).x30_pos2.x;
                        vec.z = 0.0;
                        start(gobj, cbs::fn_8015746C, &vec);
                    }
                    ftMh_MS_Slap => {
                        vec.x = (*da).x30_pos2.y;
                        vec.y = (*da).x38;
                        vec.z = 0.0;
                        start(gobj, cbs::fn_8015755C, &vec);
                    }
                    ftMh_MS_WalkLoop => {
                        vec.x = (*da).x6C;
                        vec.y = (*da).x70;
                        vec.z = 0.0;
                        start(gobj, cbs::fn_801578E8, &vec);
                    }
                    ftMh_MS_Drill => {
                        vec.x = (*da).x80;
                        vec.y = (*da).x84;
                        vec.z = 0.0;
                        start(gobj, cbs::fn_80157C50, &vec);
                    }
                    ftMh_MS_RockCrushUp => {
                        ext::ftCh_Init_80157DF8(gobj);
                    }
                    ftMh_MS_RockCrushDown => {
                        vec.x = (*da).x4C;
                        vec.y = (*da).x50;
                        vec.z = 0.0;
                        start(gobj, cbs::fn_80158144, &vec);
                    }
                    ftMh_MS_Poke1 => {
                        let mut rand = ext::HSD_Randf();
                        vec.x = fmadds((*da).x90 * rand, 2.0, (*da).x88_pos.x) - (*da).x90;
                        rand = ext::HSD_Randf();
                        vec.y = fmadds((*da).x94 * rand, 2.0, (*da).x88_pos.y) - (*da).x94;
                        vec.z = 0.0;
                        start(gobj, cbs::fn_801582D8, &vec);
                    }
                    ftMh_MS_FingerBeamStart => {
                        vec.x = (*da).x98;
                        vec.y = (*da).x9C;
                        vec.z = 0.0;
                        start(gobj, cbs::fn_801587B0, &vec);
                    }
                    ftMh_MS_BackAirplane2 => {
                        ext::ftCh_Init_80158B3C(gobj);
                    }
                    ftMh_MS_BackAirplane3 => {
                        ext::ftCh_Init_80158B3C(gobj);
                    }
                    ftMh_MS_BackDisappear => {
                        crate::tu::ftcrazyhandbackcrush::ftCh_Init_801597F0(gobj, Some(cbs::fn_80159908));
                    }
                    ftMh_MS_Wait1_1 => {
                        crate::tu::ftcrazyhandbackcrush::ftCh_Init_801597F0(gobj, Some(cbs::fn_80159AA4));
                    }
                    ftMh_MS_Grab => {
                        crate::chars::r2::ftCrazyHand::ftcrazyhandgrab::ftCh_Init_80159F40(gobj);
                    }
                    ftMh_MS_Squeezing1 => {
                        vec.x = (*da).x104;
                        vec.y = (*da).x108;
                        vec.z = 0.0;
                        start(gobj, cbs::fn_8015AAC8, &vec);
                    }
                    ftMh_MS_Squeeze => {
                        vec.x = (*da).x10C;
                        vec.y = (*da).x110_pos.x;
                        vec.z = 0.0;
                        start(gobj, cbs::fn_8015AAC8, &vec);
                    }
                    ftMh_MS_Throw => {
                        vec.x = (*da).x110_pos.y;
                        vec.y = (*da).x118_pos.x;
                        vec.z = 0.0;
                        start(gobj, cbs::fn_8015AAC8, &vec);
                    }
                    ftMh_MS_Slam => {
                        vec.x = (*da).x118_pos.y;
                        vec.y = (*da).x120;
                        vec.z = 0.0;
                    }
                    _ => {}
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

pub fn ftCh_Wait1_0_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftCh_Wait1_0_Coll");}

/// Debug pad chooser (static): the port-4 pad picks an attack directly.
pub fn ftCh_Init_80156AD8(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_Init_80156AD8");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let da = (*(*fp).ft_data).ext_attr as *const ftCrazyHand_DatAttrs;
        let b = ext::HSD_PadMasterStatus_button(3) as i32;
        let mut pos = Vec3::zeroed();
        let has = |m: i32| b & m != 0;

        if has(HSD_PAD_L) && has(HSD_PAD_DPADUP) {
            ext::mpFloorGetLeft(0, &mut pos);
            pos.y = (*da).x24;
            start(gobj, cbs::fn_80157080, &pos);
        } else if has(HSD_PAD_L) && has(HSD_PAD_DPADRIGHT) {
            ext::mpFloorGetLeft(0, &mut pos);
            pos.y = (*da).x20;
            start(gobj, cbs::fn_80156F6C, &pos);
        } else if has(HSD_PAD_L) && has(HSD_PAD_DPADDOWN) {
            pos.x = (*da).x2C;
            pos.y = (*da).x30_pos2.x;
            pos.z = 0.0;
            start(gobj, cbs::fn_8015746C, &pos);
        } else if has(HSD_PAD_L) && has(HSD_PAD_DPADLEFT) {
            pos.x = (*da).x30_pos2.y;
            pos.y = (*da).x38;
            pos.z = 0.0;
            start(gobj, cbs::fn_8015755C, &pos);
        } else if has(HSD_PAD_R) && has(HSD_PAD_DPADUP) {
            pos.x = (*da).x6C;
            pos.y = (*da).x70;
            pos.z = 0.0;
            start(gobj, cbs::fn_801578E8, &pos);
        } else if has(HSD_PAD_R) && has(HSD_PAD_DPADRIGHT) {
            pos.x = (*da).x80;
            pos.y = (*da).x84;
            pos.z = 0.0;
            start(gobj, cbs::fn_80157C50, &pos);
        } else if has(HSD_PAD_R) && has(HSD_PAD_DPADDOWN) {
            ext::ftCh_Init_80157DF8(gobj);
        } else if has(1 << 5) && has(1 << 0) {
            pos.x = (*da).x4C;
            pos.y = (*da).x50;
            pos.z = 0.0;
            start(gobj, cbs::fn_80158144, &pos);
        } else if has(HSD_PAD_B) && has(HSD_PAD_DPADUP) {
            let mut rand = ext::HSD_Randf();
            pos.x = fmadds((*da).x90 * rand, 2.0, (*da).x88_pos.x) - (*da).x90;
            rand = ext::HSD_Randf();
            pos.y = fmadds((*da).x94 * rand, 2.0, (*da).x88_pos.y) - (*da).x94;
            pos.z = 0.0;
            start(gobj, cbs::fn_801582D8, &pos);
        } else if has(HSD_PAD_B) && has(HSD_PAD_DPADRIGHT) {
            pos.x = (*da).x98;
            pos.y = (*da).x9C;
            pos.z = 0.0;
            start(gobj, cbs::fn_801587B0, &pos);
        } else if has(HSD_PAD_Z) && has(HSD_PAD_DPADUP) {
            (*fp).u.ch.x2250 = ftMh_MS_BackAirplane2 as _;
            ext::ftCh_Init_80158B3C(gobj);
        } else if has(HSD_PAD_Z) && has(HSD_PAD_DPADRIGHT) {
            (*fp).u.ch.x2250 = ftMh_MS_BackAirplane3 as _;
            ext::ftCh_Init_80158B3C(gobj);
        } else if has(HSD_PAD_A) && has(HSD_PAD_DPADUP) {
            crate::tu::ftcrazyhandbackcrush::ftCh_Init_801597F0(gobj, Some(cbs::fn_80159908));
        } else if has(HSD_PAD_A) && has(HSD_PAD_DPADRIGHT) {
            crate::tu::ftcrazyhandbackcrush::ftCh_Init_801597F0(gobj, Some(cbs::fn_80159AA4));
        } else if has(HSD_PAD_A) && has(HSD_PAD_DPADDOWN) {
            crate::chars::r2::ftCrazyHand::ftcrazyhandgrab::ftCh_Init_80159F40(gobj);
        } else if has(HSD_PAD_Y) && has(HSD_PAD_DPADUP) {
            pos.x = (*da).x104;
            pos.y = (*da).x108;
            pos.z = 0.0;
            (*fp).u.ch.x2250 = ftMh_MS_Squeezing1 as _;
            start(gobj, cbs::fn_8015AAC8, &pos);
        } else if has(HSD_PAD_Y) && has(HSD_PAD_DPADRIGHT) {
            pos.x = (*da).x10C;
            pos.y = (*da).x110_pos.x;
            pos.z = 0.0;
            (*fp).u.ch.x2250 = ftMh_MS_Squeeze as _;
            start(gobj, cbs::fn_8015AAC8, &pos);
        } else if has(HSD_PAD_Y) && has(HSD_PAD_DPADDOWN) {
            pos.x = (*da).x110_pos.y;
            pos.y = (*da).x118_pos.x;
            pos.z = 0.0;
            (*fp).u.ch.x2250 = ftMh_MS_Throw as _;
            start(gobj, cbs::fn_8015AAC8, &pos);
        } else if has(HSD_PAD_Y) && has(HSD_PAD_DPADLEFT) {
            pos.x = (*da).x118_pos.y;
            pos.y = (*da).x120;
            pos.z = 0.0;
            (*fp).u.ch.x2250 = ftMh_MS_Slam as _;
            start(gobj, cbs::fn_8015AAC8, &pos);
        }
    }
}

// ---- folded from chars (consolidation step 20/21) ----
/// `ftCh_Wait1_0_Phys` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandwait10.c).
pub fn ftCh_Wait1_0_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_Wait1_0_Phys");
    st::ft_80085134(gobj);
}

/// `ftCh_Init_80156018` (static) (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandwait10.c).
pub fn ftCh_Init_80156018(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_Init_80156018");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if ((((*fp).motion_id as i32) == 341i32 /* ftMh_MS_Wait1_0 */) || (((*fp).motion_id as i32) == 342i32 /* ftMh_MS_Wait2_0 */)) {
            (*fp).cur_pos = (*fp).u.mh.x2240_pos;
        } else {
            (*fp).u.mh.x2240_pos = (*fp).cur_pos;
        }
        if (((*fp).u.mh.x2258 as i32) == 388i32 /* 0x184 */) {
            st::Fighter_ChangeMotionState(gobj, 341 /* ftMh_MS_Wait1_0 */, 0u32, (*fp).cur_anim_frame, 1.0_f32, 0.0_f32, core::ptr::null_mut());
        } else {
            st::Fighter_ChangeMotionState(gobj, 341 /* ftMh_MS_Wait1_0 */, 0u32, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
        }
        (*fp).u.mh.x2258 = (341_i32 /* ftMh_MS_Wait1_0 */);
    }
}

/// `ftCh_Init_801560D8` (static) (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandwait10.c).
pub fn ftCh_Init_801560D8(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_Init_801560D8");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if ((((*fp).motion_id as i32) == 341i32 /* 0x155 */) || (((*fp).motion_id as i32) == 342i32 /* 0x156 */)) {
            (*fp).cur_pos = (*fp).u.mh.x2240_pos;
        } else {
            (*fp).u.mh.x2240_pos = (*fp).cur_pos;
        }
        if (((*fp).u.mh.x2258 as i32) == 389i32 /* 0x185 */) {
            st::Fighter_ChangeMotionState(gobj, 342 /* 0x156 */, 0u32, (*fp).cur_anim_frame, 1.0_f32, 0.0_f32, core::ptr::null_mut());
        } else {
            st::Fighter_ChangeMotionState(gobj, 342 /* 0x156 */, 0u32, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
        }
        (*fp).u.mh.x2258 = (342_i32 /* 0x156 */);
    }
}

/// `ftCh_Init_80156198` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandwait10.c).
pub fn ftCh_Init_80156198(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_Init_80156198");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        (*fp).cmd_vars[1] = 0_u32;
        if (((*fp).u.mh.x2258 as i32) == 389i32 /* 0x185 */) {
            crate::chars::r4::ftCrazyHand::ftcrazyhandwait10::ftCh_Init_801560D8(gobj);
        } else {
            crate::chars::r4::ftCrazyHand::ftcrazyhandwait10::ftCh_Init_80156018(gobj);
        }
    }
}

/// `ftCh_Wait1_0_IASA` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandwait10.c).
pub fn ftCh_Wait1_0_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_Wait1_0_IASA");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::Player_GetPlayerSlotType(((*fp).player_idx as i32)) == 0 /* Gm_PKind_Human */) {
            st::ftBossLib_8015BD20(gobj);
        }
    }
}

/// `ftCh_Init_80156310` (static) (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandwait10.c).
pub fn ftCh_Init_80156310(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_Init_80156310");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        (*fp).cmd_vars[1] = 0_u32;
        (*fp).u.mh.x2258 = (341_i32 /* ftMh_MS_Wait1_0 */);
        (*fp).motion_id = (387_i32 /* 0x183 */);
        (*fp).cur_pos.x = (*((*(*fp).ft_data).ext_attr as *const melee_types::ftCrazyHand_DatAttrs)).x18;
        (*fp).cur_pos.y = (*((*(*fp).ft_data).ext_attr as *const melee_types::ftCrazyHand_DatAttrs)).x1C;
        crate::chars::r4::ftCrazyHand::ftcrazyhandwait10::ftCh_Init_80156018(gobj);
    }
}

// ---- folded from chars blocks (consolidation step 20) ----
/// `doAnim0_fp` (static) (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandwait10.c).
pub fn doAnim0_fp(gobj: *mut HSD_GObj) {
    melee_core::ct!("doAnim0_fp");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if ((((*fp).motion_id as i32) == 341i32 /* 0x155 */) || (((*fp).motion_id as i32) == 342i32 /* 0x156 */)) {
            (*fp).cur_pos = (*fp).u.mh.x2240_pos;
        } else {
            (*fp).u.mh.x2240_pos = (*fp).cur_pos;
        }
        if (((*fp).u.mh.x2258 as i32) == 389i32 /* 0x185 */) {
            st::Fighter_ChangeMotionState(gobj, 342 /* 0x156 */, 0u32, (*fp).cur_anim_frame, 1.0_f32, 0.0_f32, core::ptr::null_mut());
        } else {
            st::Fighter_ChangeMotionState(gobj, 342 /* 0x156 */, 0u32, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
        }
        (*fp).u.mh.x2258 = (342_i32 /* 0x156 */);
    }
}

/// `doAnim1_fp` (static) (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandwait10.c).
pub fn doAnim1_fp(gobj: *mut HSD_GObj) {
    melee_core::ct!("doAnim1_fp");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        (*fp).cmd_vars[1] = 0_u32;
        if (((*fp).u.mh.x2258 as i32) == 389i32 /* 0x185 */) {
            crate::chars::r4::ftCrazyHand::ftcrazyhandwait10::ftCh_Init_801560D8(gobj);
        } else {
            crate::chars::r4::ftCrazyHand::ftcrazyhandwait10::ftCh_Init_80156018(gobj);
        }
    }
}
