//! src/melee/ft/kinds/ftKirby/ftkirbyspecialfox.c: Kirby with Fox's / Falco's hat, neutral B
//! (Blaster), plus Ness's PK Flash helpers that the C file also holds. No fused sites
//! (tools/rust-port/fma_sites.py). The motion-id mappings are checked against the C
//! (tests/tu_ftkirbyspecialfox.rs); the callbacks are smoke-tested through recording hooks.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use melee_types::*;

/// Already ported (rounds 2, 6): re-exported under this TU.
pub use crate::chars::r2::ftKirby::ftkirbyspecialfox::{
    ftKb_FxSpecialAirNEnd_Phys, ftKb_FxSpecialAirNLoop_Phys, ftKb_FxSpecialAirNStart_Phys,
    ftKb_FxSpecialNEnd_Coll, ftKb_FxSpecialNEnd_Phys, ftKb_FxSpecialNLoop_Coll,
    ftKb_FxSpecialNLoop_Phys, ftKb_FxSpecialNStart_Coll, ftKb_FxSpecialNStart_Phys,
};
pub use crate::chars::r6::ftKirby::ftkirbyspecialfox::{
    ftKb_FxSpecialAirNEnd_Coll, ftKb_FxSpecialAirNLoop_Coll, ftKb_FxSpecialAirNStart_Coll,
};

/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {
    use melee_types::{HSD_GObj, HSD_JObj, Item_GObj, Vec3};
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
    pub static mut LB_8000B1CC: Option<fn(*mut HSD_JObj, *const Vec3, *mut Vec3) > = None;
            #[allow(non_snake_case)]
            pub fn lb_8000B1CC(jobj: *mut HSD_JObj, offset: *const Vec3, out: *mut Vec3)  {
                match unsafe { LB_8000B1CC } {
                    Some(f) => f(jobj, offset, out),
                    None => { unsafe { melee_runtime::tu::lb_00B0::lb_8000B1CC(jobj as _, offset as _, out as _); } return; },
                }
            }
    crate::port_hook!(FT_800892A0, ft_800892A0, crate::adapt::ft_800892A0, (gobj: *mut HSD_GObj));
    pub static mut IT_802AEAB4: Option<fn(*mut Item_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn it_802AEAB4(item: *mut Item_GObj)  {
                match unsafe { IT_802AEAB4 } {
                    Some(f) => f(item),
                    None => { unsafe { melee_items::tu::itfoxblaster::it_802AEAB4(item as _); } return; },
                }
            }
    pub static mut IT_8029C6A4: Option<fn(f64, f32, *mut HSD_GObj, *mut Vec3, u32) > = None;
            #[allow(non_snake_case)]
            pub fn it_8029C6A4(angle: f64, speed: f32, gobj: *mut HSD_GObj, pos: *mut Vec3, id: u32)  {
                match unsafe { IT_8029C6A4 } {
                    Some(f) => f(angle, speed, gobj, pos, id),
                    None => { unsafe { melee_items::tu::itfoxlaser::it_8029C6A4(angle as _, speed as _, gobj as _, pos as _, id as _); } return; },
                }
            }
    pub static mut IT_802AE1D0: Option<fn(*mut Item_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn it_802AE1D0(item: *mut Item_GObj)  {
                match unsafe { IT_802AE1D0 } {
                    Some(f) => f(item),
                    None => { unsafe { melee_items::tu::itfoxblaster::it_802AE1D0(item as _); } return; },
                }
            }
    pub static mut IT_802AE8A8: Option<fn(f32, *mut HSD_GObj, *mut Vec3, i32, u32) -> *mut HSD_GObj> = None;
            #[allow(non_snake_case)]
            pub fn it_802AE8A8(dir: f32, gobj: *mut HSD_GObj, pos: *mut Vec3, part: i32, id: u32) -> *mut HSD_GObj {
                match unsafe { IT_802AE8A8 } {
                    Some(f) => f(dir, gobj, pos, part, id),
                    None => { return unsafe { melee_items::tu::itfoxblaster::it_802AE8A8(dir as _, gobj as _, pos as _, part as _, id as _) as _ }; },
                }
            }
    pub static mut IT_802ADDD0: Option<fn(*mut Item_GObj, i32) > = None;
            #[allow(non_snake_case)]
            pub fn it_802ADDD0(item: *mut Item_GObj, state: i32)  {
                match unsafe { IT_802ADDD0 } {
                    Some(f) => f(item, state),
                    None => { unsafe { melee_items::tu::itfoxblaster::it_802ADDD0(item as _, state as _); } return; },
                }
            }
    pub static mut IT_802AE538: Option<fn(*mut Item_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn it_802AE538(item: *mut Item_GObj)  {
                match unsafe { IT_802AE538 } {
                    Some(f) => f(item),
                    None => { unsafe { melee_items::tu::itfoxblaster::it_802AE538(item as _); } return; },
                }
            }
    pub static mut IT_802AE608: Option<fn(*mut Item_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn it_802AE608(item: *mut Item_GObj)  {
                match unsafe { IT_802AE608 } {
                    Some(f) => f(item),
                    None => { unsafe { melee_items::tu::itfoxblaster::it_802AE608(item as _); } return; },
                }
            }
    pub static mut IT_802AAA50: Option<fn(*mut Item_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn it_802AAA50(item: *mut Item_GObj)  {
                match unsafe { IT_802AAA50 } {
                    Some(f) => f(item),
                    None => { unsafe { melee_items::tu::itnesspkflash::it_802AAA50(item as _); } return; },
                }
            }
    pub static mut FTFOX_SPECIALN_CHECKLOOPINPUT: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn ftFox_SpecialN_CheckLoopInput(gobj: *mut HSD_GObj)  {
                match unsafe { FTFOX_SPECIALN_CHECKLOOPINPUT } {
                    Some(f) => f(gobj),
                    None => { unsafe { crate::tu::ftfoxspecialn::ftFox_SpecialN_CheckLoopInput(gobj as _); } return; },
                }
            }

    /// `ftpickupitem_80094818(gobj, arg)`: group 3's port when the hook is unset.
    pub static mut FTPICKUPITEM_80094818: Option<fn(*mut HSD_GObj, bool)> = None;
    #[allow(non_snake_case)]
    pub fn ftpickupitem_80094818(gobj: *mut HSD_GObj, arg: bool) {
        match unsafe { FTPICKUPITEM_80094818 } {
            Some(f) => f(gobj, arg),
            None => melee_fighter::tu::ftpickupitem::ftpickupitem_80094818(gobj, arg),
        }
    }
}

const THROW_B: u32 = ftCo_MS_ThrowB as u32;
const THROW_HI: u32 = ftCo_MS_ThrowHi as u32;
const THROW_LW: u32 = ftCo_MS_ThrowLw as u32;

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

#[inline(always)]
fn clear_cmd_vars(fp: &mut Fighter) {
    fp.cmd_vars[3] = 0;
    fp.cmd_vars[2] = 0;
    fp.cmd_vars[1] = 0;
    fp.cmd_vars[0] = 0;
}

/// `ftKb_Init_804D3DB8`: Fox's blaster-shot voice, [right, left].
pub static ftKb_Init_804D3DB8: [u32; 2] = [0x0001AE17, 0x0001AE1A];
/// `ftKb_Init_804D3DC0`: Falco's blaster-shot voice, [right, left].
pub static ftKb_Init_804D3DC0: [u32; 2] = [0x00018703, 0x00018706];

/// `ftKb_SpecialN_set_cbs` (ftKirby/inlines.h).
#[inline(always)]
unsafe fn set_cbs(gobj: *mut HSD_GObj) {
    let fp = fpp(gobj);
    (*fp).death2_cb = Some(crate::chars::cb::ftkirby__ftKb_Init_800EE74C);
    (*fp).take_dmg_cb = Some(crate::chars::cb::ftkirby__ftKb_Init_800EE7B8);
}

/// `ftKb_SpecialNFx_OnChangeAction` as the `HSD_GObjEvent` stored in `x21EC`.
pub unsafe extern "C" fn cb_ftKb_SpecialNFx_OnChangeAction(g: *mut HSD_GObj) {
    melee_core::ct!("cb_ftKb_SpecialNFx_OnChangeAction");
    ftKb_SpecialNFx_OnChangeAction(g);
}

/// `ftKb_SpecialNFx_CreateBlasterShot` as the `HSD_GObjEvent` stored in `accessory4_cb`.
pub unsafe extern "C" fn cb_ftKb_SpecialNFx_CreateBlasterShot(g: *mut HSD_GObj) {
    melee_core::ct!("cb_ftKb_SpecialNFx_CreateBlasterShot");
    ftKb_SpecialNFx_CreateBlasterShot(g);
}

/// `ftKb_SpecialNFx_800FDC00`: world position of the blaster muzzle (part 44, offset (0, 1.45, 5.016)).
pub fn ftKb_SpecialNFx_800FDC00(gobj: *mut HSD_GObj, vec: *mut Vec3) {
    melee_core::ct!("ftKb_SpecialNFx_800FDC00");
    unsafe {
        if !gobj.is_null() {
            let fp = fpp(gobj);
            if !fp.is_null() {
                let offset = Vec3 { x: 0.0, y: 1.45, z: 5.016 };
                ext::lb_8000B1CC((*(*fp).parts.add(44)).joint, &offset, vec);
                return;
            }
        }
        (*vec).z = 0.0;
        (*vec).y = 0.0;
        (*vec).x = 0.0;
    }
}

/// `ftKb_SpecialNFx_800FDC70`: the same for the hand (FtPart_R3rdNa), offset (0, 1.45, 0.016).
pub fn ftKb_SpecialNFx_800FDC70(gobj: *mut HSD_GObj, vec: *mut Vec3) {
    melee_core::ct!("ftKb_SpecialNFx_800FDC70");
    unsafe {
        if !gobj.is_null() {
            let fp = fpp(gobj);
            if !fp.is_null() {
                let pos1 = Vec3 { x: 0.0, y: 1.45, z: 0.016 };
                ext::lb_8000B1CC((*(*fp).parts.add(FtPart_R3rdNa as usize)).joint, &pos1, vec);
                return;
            }
        }
        (*vec).z = 0.0;
        (*vec).y = 0.0;
        (*vec).x = 0.0;
    }
}

pub fn ftKb_SpecialNFx_OnChangeAction(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_SpecialNFx_OnChangeAction");
    ext::ft_800892A0(gobj);
    melee_fighter::ft0892::ft_80089824(gobj);
}

/// `ftKb_SpecialNFx_800FDD14`: true when no blaster item is attached (or no fighter).
pub fn ftKb_SpecialNFx_800FDD14(gobj: *mut HSD_GObj) -> bool {
    melee_core::ct!("ftKb_SpecialNFx_800FDD14");
    unsafe {
        if !gobj.is_null() {
            let ft = fpp(gobj);
            if !ft.is_null() {
                if (*ft).u.kb.xB0.is_null() {
                    return true;
                }
                return false;
            }
        }
        true
    }
}

/// `ftKb_SpecialNFx_800FDD4C`: the ftFox SpecialN index (0..) matching Kirby's current motion, 9 if none.
pub fn ftKb_SpecialNFx_800FDD4C(gobj: *mut HSD_GObj) -> ftFx_SpecialNIndex {
    melee_core::ct!("ftKb_SpecialNFx_800FDD4C");
    unsafe {
        let mut msid: u32 = 9;
        if !gobj.is_null() {
            let fp = fpp(gobj);
            if !fp.is_null() {
                match (*fp).u.kb.hat.kind {
                    Ft_Kind_Fox => {
                        let motion_id = (*fp).motion_id as u32;
                        match motion_id {
                            ftKb_MS_FxSpecialNStart
                            | ftKb_MS_FxSpecialNLoop
                            | ftKb_MS_FxSpecialNEnd
                            | ftKb_MS_FxSpecialAirNStart
                            | ftKb_MS_FxSpecialAirNLoop
                            | ftKb_MS_FxSpecialAirNEnd => msid = motion_id.wrapping_sub(0x1A7),
                            THROW_B | THROW_HI | THROW_LW => msid = motion_id.wrapping_sub(0xD6),
                            _ => {}
                        }
                    }
                    Ft_Kind_Falco => {
                        let motion_id = (*fp).motion_id as u32;
                        match motion_id {
                            ftKb_MS_FcSpecialNStart
                            | ftKb_MS_FcSpecialNLoop
                            | ftKb_MS_FcSpecialNEnd
                            | ftKb_MS_FcSpecialAirNStart
                            | ftKb_MS_FcSpecialAirNLoop
                            | ftKb_MS_FcSpecialAirNEnd => msid = motion_id.wrapping_sub(0x208),
                            THROW_B | THROW_HI | THROW_LW => msid = motion_id.wrapping_sub(0xD6),
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        }
        msid
    }
}

/// `ftKb_SpecialNFx_800FDDF4`: x2071_b6 while in a blaster/throw motion, else true.
pub fn ftKb_SpecialNFx_800FDDF4(gobj: *mut HSD_GObj) -> bool {
    melee_core::ct!("ftKb_SpecialNFx_800FDDF4");
    unsafe {
        if !gobj.is_null() {
            let fp = fpp(gobj);
            if !fp.is_null() {
                match (*fp).u.kb.hat.kind {
                    Ft_Kind_Fox => {
                        let motion_id = (*fp).motion_id as u32;
                        match motion_id {
                            ftKb_MS_FxSpecialNStart
                            | ftKb_MS_FxSpecialNLoop
                            | ftKb_MS_FxSpecialNEnd
                            | ftKb_MS_FxSpecialAirNStart
                            | ftKb_MS_FxSpecialAirNLoop
                            | ftKb_MS_FxSpecialAirNEnd
                            | THROW_B
                            | THROW_HI
                            | THROW_LW => {
                                if (*fp).x2070.anon_0.x2071_b6() != 0 {
                                    return true;
                                }
                                return false;
                            }
                            _ => {}
                        }
                    }
                    Ft_Kind_Falco => {
                        let motion_id = (*fp).motion_id as u32;
                        match motion_id {
                            ftKb_MS_FcSpecialNStart
                            | ftKb_MS_FcSpecialNLoop
                            | ftKb_MS_FcSpecialNEnd
                            | ftKb_MS_FcSpecialAirNStart
                            | ftKb_MS_FcSpecialAirNLoop
                            | ftKb_MS_FcSpecialAirNEnd
                            | THROW_B
                            | THROW_HI
                            | THROW_LW => {
                                if (*fp).x2070.anon_0.x2071_b6() != 0 {
                                    return true;
                                }
                                return false;
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        }
        true
    }
}

/// `ftKb_SpecialNFx_800FDEB4`: forget the blaster item (without destroying it).
pub fn ftKb_SpecialNFx_800FDEB4(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_SpecialNFx_800FDEB4");
    unsafe {
        if !gobj.is_null() {
            let ft = fpp(gobj);
            if !ft.is_null() && !(*ft).u.kb.xB0.is_null() {
                (*ft).u.kb.xB0 = core::ptr::null_mut();
            }
        }
    }
}

/// `ftKb_SpecialNFx_800FDEE0`: destroy the blaster item.
pub fn ftKb_SpecialNFx_800FDEE0(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_SpecialNFx_800FDEE0");
    unsafe {
        if !gobj.is_null() {
            let fp = fpp(gobj);
            if !fp.is_null() && !(*fp).u.kb.xB0.is_null() {
                ext::it_802AEAB4((*fp).u.kb.xB0 as *mut Item_GObj);
                (*fp).u.kb.xB0 = core::ptr::null_mut();
            }
        }
    }
}

/// `ftKb_SpecialNFx_800FDF30`: on the script's cmd_vars[2] pulse, fire the blaster shot.
pub fn ftKb_SpecialNFx_800FDF30(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_SpecialNFx_800FDF30");
    unsafe {
        let fp = fpp(gobj);
        let da = (*fp).dat_attrs as *const ftKb_DatAttrs;
        if (*fp).cmd_vars[2] != 0 {
            (*fp).cmd_vars[2] = 0;
            let mut sp2c = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
            let mut fp2 = core::ptr::null_mut::<Fighter>();
            if !gobj.is_null() {
                fp2 = fpp(gobj);
            }
            if !gobj.is_null() && !fp2.is_null() {
                let pos1 = Vec3 { x: 0.0, y: 1.45, z: 5.016 };
                ext::lb_8000B1CC((*(*fp2).parts.add(FtPart_R3rdNa as usize)).joint, &pos1, &mut sp2c);
            } else {
                sp2c.z = 0.0;
                sp2c.y = 0.0;
                sp2c.x = 0.0;
            }
            sp2c.z = 0.0;
            let left = ((*fp).facing_dir == -1.0) as usize;
            match (*fp).u.kb.hat.kind {
                Ft_Kind_Fox => {
                    let launch_angle: f64 = if 1.0 == (*fp).facing_dir {
                        (*da).specialn_fx_launch_angle as f64
                    } else {
                        core::f64::consts::PI - (*da).specialn_fx_launch_angle as f64
                    };
                    ext::it_8029C6A4(launch_angle, (*da).specialn_fx_launch_speed, gobj, &mut sp2c, (*da).specialn_fx_blaster_projectile_id);
                    ext::it_802AE1D0((*fp).u.kb.xB0 as *mut Item_GObj);
                    melee_fighter::hooks::play_sfx(&mut *fp, ftKb_Init_804D3DB8[left] as i32, 0x7F, 0x40);
                }
                Ft_Kind_Falco => {
                    let launch_angle: f64 = if 1.0 == (*fp).facing_dir {
                        (*da).specialn_fc_launch_angle as f64
                    } else {
                        core::f64::consts::PI - (*da).specialn_fc_launch_angle as f64
                    };
                    ext::it_8029C6A4(launch_angle, (*da).specialn_fc_launch_speed, gobj, &mut sp2c, (*da).specialn_fc_blaster_projectile_id);
                    ext::it_802AE1D0((*fp).u.kb.xB0 as *mut Item_GObj);
                    melee_fighter::hooks::play_sfx(&mut *fp, ftKb_Init_804D3DC0[left] as i32, 0x7F, 0x40);
                }
                _ => {}
            }
        }
    }
}

pub fn ftKb_SpecialNFx_CreateBlasterShot(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_SpecialNFx_CreateBlasterShot");
    ftKb_SpecialNFx_800FDF30(gobj);
}

/// `ftKbGetStartMotionId` (static inline).
pub fn ftKbGetStartMotionId(gobj: *mut HSD_GObj) -> u32 {
    melee_core::ct!("ftKbGetStartMotionId");
    unsafe {
        let fp = fpp(gobj);
        let mut msid = ftKb_MS_FxSpecialNStart;
        match (*fp).u.kb.hat.kind {
            Ft_Kind_Falco => msid = ftKb_MS_FcSpecialNStart,
            Ft_Kind_Fox => msid = ftKb_MS_FxSpecialNStart,
            _ => {}
        }
        msid
    }
}

/// The blaster item id for the hat in `fp` (0 for any other kind: the C local is zero-initialised).
unsafe fn blaster_item_id(fp: *mut Fighter, da: *const ftKb_DatAttrs) -> u32 {
    match (*fp).u.kb.hat.kind {
        Ft_Kind_Fox => (*da).specialn_fx_blaster_item_id,
        Ft_Kind_Falco => (*da).specialn_fc_blaster_item_id,
        _ => 0,
    }
}

/// The failure path of the blaster-spawn: `OSReport("ftToSpecialNFox::Caution!!!\n"); HSD_ASSERT(line, 0)`.
fn blaster_assert(line: u32) -> ! {
    panic!("ftToSpecialNFox::Caution!!! (HSD_ASSERT ftkirbyspecialfox.c:{line})");
}

/// `ftKb_SpecialNFx_800FE100`: ground entry; spawns the blaster item.
pub fn ftKb_SpecialNFx_800FE100(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_SpecialNFx_800FE100");
    unsafe {
        let fp = fpp(gobj);
        let da = (*fp).dat_attrs as *const ftKb_DatAttrs;
        st::ftCommon_8007D7FC(&mut *fp);
        st::Fighter_ChangeMotionState(gobj, ftKbGetStartMotionId(gobj) as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        clear_cmd_vars(&mut *fp);
        st::ftAnim_8006EBA4(gobj);
        (*fp).gr_vel = 0.0;
        (*fp).self_vel.z = 0.0;
        (*fp).self_vel.y = 0.0;
        (*fp).self_vel.x = 0.0;
        (*fp).mv.fx.SpecialN.isBlasterLoop = 0;
        let blaster_id = blaster_item_id(fp, da);
        let blaster = ext::it_802AE8A8((*fp).facing_dir, gobj, &mut (*fp).cur_pos, FtPart_R3rdNa as i32, blaster_id);
        (*fp).u.kb.xB0 = blaster;
        if !blaster.is_null() {
            set_cbs(gobj);
            return;
        }
        blaster_assert(429);
    }
}

/// `ftKbGetAirStartMotionId` (static inline).
pub fn ftKbGetAirStartMotionId(gobj: *mut HSD_GObj) -> u32 {
    melee_core::ct!("ftKbGetAirStartMotionId");
    unsafe {
        let fp = fpp(gobj);
        let mut msid = ftKb_MS_FxSpecialAirNStart;
        match (*fp).u.kb.hat.kind {
            Ft_Kind_Falco => msid = ftKb_MS_FcSpecialAirNStart,
            Ft_Kind_Fox => msid = ftKb_MS_FxSpecialAirNStart,
            _ => {}
        }
        msid
    }
}

/// `ftKb_SpecialNFx_800FE240`: aerial entry.
pub fn ftKb_SpecialNFx_800FE240(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_SpecialNFx_800FE240");
    unsafe {
        let fp = fpp(gobj);
        let da = (*fp).dat_attrs as *const ftKb_DatAttrs;
        st::Fighter_ChangeMotionState(gobj, ftKbGetAirStartMotionId(gobj) as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        clear_cmd_vars(&mut *fp);
        st::ftAnim_8006EBA4(gobj);
        (*fp).mv.fx.SpecialN.isBlasterLoop = 0;
        let blaster_id = blaster_item_id(fp, da);
        let blaster = ext::it_802AE8A8((*fp).facing_dir, gobj, &mut (*fp).cur_pos, FtPart_R3rdNa as i32, blaster_id);
        (*fp).u.kb.xB0 = blaster;
        if !blaster.is_null() {
            set_cbs(gobj);
            return;
        }
        blaster_assert(465);
    }
}

/// `ftKbGetLoopMotionId` (static inline).
pub fn ftKbGetLoopMotionId(gobj: *mut HSD_GObj) -> u32 {
    melee_core::ct!("ftKbGetLoopMotionId");
    unsafe {
        let fp = fpp(gobj);
        let mut msid = ftKb_MS_FxSpecialNLoop;
        match (*fp).u.kb.hat.kind {
            Ft_Kind_Falco => msid = ftKb_MS_FcSpecialNLoop,
            Ft_Kind_Fox => msid = ftKb_MS_FxSpecialNLoop,
            _ => {}
        }
        msid
    }
}

/// The per-frame blaster-item pose and the cmd_vars[3] == 1 trigger common to the Start and Loop anims.
unsafe fn blaster_pose(fp: *mut Fighter) {
    ext::it_802ADDD0((*fp).u.kb.xB0 as *mut Item_GObj, 1);
    if (*fp).cmd_vars[3] == 1 && !(*fp).u.kb.xB0.is_null() {
        (*fp).cmd_vars[3] = 0;
        ext::it_802AE538((*fp).u.kb.xB0 as *mut Item_GObj);
    }
}

pub fn ftKb_FxSpecialNStart_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_FxSpecialNStart_Anim");
    unsafe {
        let fp = fpp(gobj);
        blaster_pose(fp);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::Fighter_ChangeMotionState(gobj, ftKbGetLoopMotionId(gobj) as i32, Ft_MF_SkipModel | Ft_MF_KeepGfx, 0.0, 1.0, 0.0, core::ptr::null_mut());
            set_cbs(gobj);
            (*fp).accessory4_cb = Some(cb_ftKb_SpecialNFx_CreateBlasterShot);
            ext::it_802ADDD0((*fp).u.kb.xB0 as *mut Item_GObj, 1);
        }
    }
}

/// `ftKbGetEndMotionId` (static inline).
pub fn ftKbGetEndMotionId(gobj: *mut HSD_GObj) -> u32 {
    melee_core::ct!("ftKbGetEndMotionId");
    unsafe {
        let fp = fpp(gobj);
        let mut msid = ftKb_MS_FxSpecialNEnd;
        match (*fp).u.kb.hat.kind {
            Ft_Kind_Falco => msid = ftKb_MS_FcSpecialNEnd,
            Ft_Kind_Fox => msid = ftKb_MS_FxSpecialNEnd,
            _ => {}
        }
        msid
    }
}

pub fn ftKb_FxSpecialNLoop_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_FxSpecialNLoop_Anim");
    unsafe {
        let fp = fpp(gobj);
        blaster_pose(fp);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            if (*fp).mv.fx.SpecialN.isBlasterLoop == 1 {
                (*fp).x21EC = Some(cb_ftKb_SpecialNFx_OnChangeAction);
                st::Fighter_ChangeMotionState(
                    gobj,
                    ftKbGetLoopMotionId(gobj) as i32,
                    Ft_MF_SkipAttackCount | Ft_MF_SkipModel | Ft_MF_KeepGfx,
                    0.0,
                    1.0,
                    0.0,
                    core::ptr::null_mut(),
                );
                (*fp).accessory4_cb = Some(cb_ftKb_SpecialNFx_CreateBlasterShot);
                (*fp).mv.fx.SpecialN.isBlasterLoop = 0;
                ext::it_802ADDD0((*fp).u.kb.xB0 as *mut Item_GObj, 1);
            } else {
                st::Fighter_ChangeMotionState(gobj, ftKbGetEndMotionId(gobj) as i32, Ft_MF_SkipModel | Ft_MF_KeepGfx, 0.0, 1.0, 0.0, core::ptr::null_mut());
                let temp = (*fp).u.kb.xB0;
                (*fp).cmd_vars[1] = 1;
                ext::it_802ADDD0(temp as *mut Item_GObj, 1);
            }
            set_cbs(gobj);
        }
        ftKb_SpecialNFx_800FDF30(gobj);
    }
}

/// The End anims' common head: pose by cmd_vars[1], pick up the item on 2, release it on cmd_vars[3] == 2.
unsafe fn blaster_end_pose(gobj: *mut HSD_GObj, fp: *mut Fighter) {
    ext::it_802ADDD0((*fp).u.kb.xB0 as *mut Item_GObj, (*fp).cmd_vars[1] as i32);
    if (*fp).cmd_vars[1] == 2 {
        ext::ftpickupitem_80094818(gobj, false);
    }
    if (*fp).cmd_vars[3] == 2 && !(*fp).u.kb.xB0.is_null() {
        (*fp).cmd_vars[3] = 0;
        ext::it_802AE608((*fp).u.kb.xB0 as *mut Item_GObj);
    }
}

pub fn ftKb_FxSpecialNEnd_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_FxSpecialNEnd_Anim");
    unsafe {
        let fp = fpp(gobj);
        blaster_end_pose(gobj, fp);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            ftKb_SpecialNFx_800FDEB4(gobj);
            st::ft_8008A2BC(gobj);
        }
    }
}

/// `ftKbGetAirLoopMotionId` (static inline).
pub fn ftKbGetAirLoopMotionId(gobj: *mut HSD_GObj) -> u32 {
    melee_core::ct!("ftKbGetAirLoopMotionId");
    unsafe {
        let fp = fpp(gobj);
        let mut msid = ftKb_MS_FxSpecialAirNLoop;
        match (*fp).u.kb.hat.kind {
            Ft_Kind_Falco => msid = ftKb_MS_FcSpecialAirNLoop,
            Ft_Kind_Fox => msid = ftKb_MS_FxSpecialAirNLoop,
            _ => {}
        }
        msid
    }
}

pub fn ftKb_FxSpecialAirNStart_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_FxSpecialAirNStart_Anim");
    unsafe {
        let fp = fpp(gobj);
        blaster_pose(fp);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::Fighter_ChangeMotionState(gobj, ftKbGetAirLoopMotionId(gobj) as i32, Ft_MF_SkipModel | Ft_MF_KeepGfx, 0.0, 1.0, 0.0, core::ptr::null_mut());
            set_cbs(gobj);
            (*fp).accessory4_cb = Some(cb_ftKb_SpecialNFx_CreateBlasterShot);
            ext::it_802ADDD0((*fp).u.kb.xB0 as *mut Item_GObj, 1);
        }
    }
}

/// `ftKbGetAirEndMotionId` (static inline).
pub fn ftKbGetAirEndMotionId(gobj: *mut HSD_GObj) -> u32 {
    melee_core::ct!("ftKbGetAirEndMotionId");
    unsafe {
        let fp = fpp(gobj);
        let mut msid = ftKb_MS_FxSpecialAirNEnd;
        match (*fp).u.kb.hat.kind {
            Ft_Kind_Falco => msid = ftKb_MS_FcSpecialAirNEnd,
            Ft_Kind_Fox => msid = ftKb_MS_FxSpecialAirNEnd,
            _ => {}
        }
        msid
    }
}

pub fn ftKb_FxSpecialAirNLoop_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_FxSpecialAirNLoop_Anim");
    unsafe {
        let fp = fpp(gobj);
        blaster_pose(fp);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            if (*fp).mv.fx.SpecialN.isBlasterLoop == 1 {
                (*fp).x21EC = Some(cb_ftKb_SpecialNFx_OnChangeAction);
                st::Fighter_ChangeMotionState(
                    gobj,
                    ftKbGetAirLoopMotionId(gobj) as i32,
                    Ft_MF_SkipAttackCount | Ft_MF_SkipModel | Ft_MF_KeepGfx,
                    0.0,
                    1.0,
                    0.0,
                    core::ptr::null_mut(),
                );
                set_cbs(gobj);
                (*fp).accessory4_cb = Some(cb_ftKb_SpecialNFx_CreateBlasterShot);
                (*fp).mv.fx.SpecialN.isBlasterLoop = 0;
                ext::it_802ADDD0((*fp).u.kb.xB0 as *mut Item_GObj, 1);
            } else {
                st::Fighter_ChangeMotionState(gobj, ftKbGetAirEndMotionId(gobj) as i32, Ft_MF_SkipModel | Ft_MF_KeepGfx, 0.0, 1.0, 0.0, core::ptr::null_mut());
                set_cbs(gobj);
                let temp = (*fp).u.kb.xB0;
                (*fp).cmd_vars[1] = 1;
                ext::it_802ADDD0(temp as *mut Item_GObj, 1);
            }
            set_cbs(gobj);
        }
        ftKb_SpecialNFx_800FDF30(gobj);
    }
}

pub fn ftKb_FxSpecialAirNEnd_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_FxSpecialAirNEnd_Anim");
    unsafe {
        let fp = fpp(gobj);
        let da = (*fp).dat_attrs as *const ftKb_DatAttrs;
        blaster_end_pose(gobj, fp);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            ftKb_SpecialNFx_800FDEB4(gobj);
            let mut landing_lag: f32 = 0.0;
            match (*fp).u.kb.hat.kind {
                Ft_Kind_Fox => landing_lag = (*da).specialn_fx_freefall_toggle,
                Ft_Kind_Falco => landing_lag = (*da).specialn_fc_freefall_toggle,
                _ => {}
            }
            if landing_lag == 0.0 {
                st::ftCo_Fall_Enter(gobj);
                return;
            }
            st::ftCo_80096900(gobj, 1, 0, 1, 1.0, landing_lag);
        }
    }
}

pub fn ftKb_FxSpecialNStart_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_FxSpecialNStart_IASA");
    ext::ftFox_SpecialN_CheckLoopInput(gobj);
}
pub fn ftKb_FxSpecialNLoop_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_FxSpecialNLoop_IASA");
    ext::ftFox_SpecialN_CheckLoopInput(gobj);
}
pub fn ftKb_FxSpecialNEnd_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftKb_FxSpecialNEnd_IASA");}
pub fn ftKb_FxSpecialAirNStart_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_FxSpecialAirNStart_IASA");
    ext::ftFox_SpecialN_CheckLoopInput(gobj);
}
pub fn ftKb_FxSpecialAirNLoop_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_FxSpecialAirNLoop_IASA");
    ext::ftFox_SpecialN_CheckLoopInput(gobj);
}
pub fn ftKb_FxSpecialAirNEnd_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftKb_FxSpecialAirNEnd_IASA");}

/// `ftKb_SpecialNNs_800FEC78`: true while Kirby (Ness hat) holds PK Flash charge.
pub fn ftKb_SpecialNNs_800FEC78(gobj: *mut HSD_GObj) -> bool {
    melee_core::ct!("ftKb_SpecialNNs_800FEC78");
    unsafe {
        if !gobj.is_null() {
            let fp = fpp(gobj);
            if !fp.is_null()
                && !(*fp).u.kb.ns_flash_gobj.is_null()
                && ((*fp).motion_id as u32 == ftKb_MS_NsSpecialNHold0 || (*fp).motion_id as u32 == ftKb_MS_NsSpecialAirNHold0)
            {
                return true;
            }
        }
        false
    }
}

/// `ftKb_SpecialNNs_800FECBC`: forget the PK Flash item.
pub fn ftKb_SpecialNNs_800FECBC(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_SpecialNNs_800FECBC");
    unsafe {
        if !gobj.is_null() {
            let fp = fpp(gobj);
            if !fp.is_null() && !(*fp).u.kb.ns_flash_gobj.is_null() {
                (*fp).u.kb.ns_flash_gobj = core::ptr::null_mut();
            }
        }
    }
}

/// `ftKb_SpecialNNs_800FECE8`: destroy the PK Flash item.
pub fn ftKb_SpecialNNs_800FECE8(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_SpecialNNs_800FECE8");
    unsafe {
        if !gobj.is_null() {
            let fp = fpp(gobj);
            if !fp.is_null() && !(*fp).u.kb.ns_flash_gobj.is_null() {
                ext::it_802AAA50((*fp).u.kb.ns_flash_gobj);
                (*fp).u.kb.ns_flash_gobj = core::ptr::null_mut();
            }
        }
    }
}
