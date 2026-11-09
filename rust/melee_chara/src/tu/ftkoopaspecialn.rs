//! src/melee/ft/kinds/ftKoopa/ftkoopaspecialn.c: Bowser's fire breath (neutral B) and the
//! shared flame/hurtbox helpers the up B reuses.
//! Fused sites (tools/rust-port/fma_sites.py): `ftKp_SpecialLw_80134ACC` has 2 (`v.x += scale.y *
//! (x24 * facing)`, `v.y += x28 * scale.y`), both `fmadds` here. Smoke-tested by
//! tests/tu_ftkoopaspecialn.rs (the flame offset is checked against hand-computed fmadds).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types::Fighter;
#[allow(unused_imports)]
use melee_types::{HSD_GObj};
use crate::stubs as st;
use melee_core::fp::fmadds;
use melee_fighter::hooks as fh;
use melee_types::*;

/// Already ported (round 2 / hand-written): re-exported under this TU.



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
    pub static mut LB_8000B1CC: Option<fn(*mut HSD_JObj, *const Vec3, *mut Vec3) > = None;
            #[allow(non_snake_case)]
            pub fn lb_8000B1CC(jobj: *mut HSD_JObj, ofs: *const Vec3, out: *mut Vec3)  {
                match unsafe { LB_8000B1CC } {
                    Some(f) => f(jobj, ofs, out),
                    None => { unsafe { melee_runtime::tu::lb_00B0::lb_8000B1CC(jobj as _, ofs as _, out as _); } return; },
                }
            }
    pub static mut ITKOOPAFLAME_SPAWN: Option<fn(*mut HSD_GObj, *mut Vec3, f32, u32, i32, i32, i32, i32) -> *mut HSD_GObj> = None;
            #[allow(non_snake_case)]
            pub fn itKoopaFlame_Spawn(parent: *mut HSD_GObj, pos: *mut Vec3, facing_dir: f32, unk: u32, gfx: i32, base_speed: i32, scale: i32, kind: i32) -> *mut HSD_GObj {
                match unsafe { ITKOOPAFLAME_SPAWN } {
                    Some(f) => f(parent, pos, facing_dir, unk, gfx, base_speed, scale, kind),
                    None => { return unsafe { melee_items::tu::itkoopaflame::itKoopaFlame_Spawn(parent as _, pos as _, facing_dir as _, unk as _, gfx as _, base_speed as _, scale as _, kind as _) as _ }; },
                }
            }
    pub static mut ITEM_8026AE60: Option<fn() -> u32> = None;
            #[allow(non_snake_case)]
            pub fn Item_8026AE60() -> u32 {
                match unsafe { ITEM_8026AE60 } {
                    Some(f) => f(),
                    None => { return unsafe { melee_items::tu::item::Item_8026AE60() as _ }; },
                }
            }
    pub static mut FT_80089824: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn ft_80089824(gobj: *mut HSD_GObj)  {
                match unsafe { FT_80089824 } {
                    Some(f) => f(gobj),
                    None => { unsafe { let _ = melee_fighter::ft0892::ft_80089824(gobj as _); } return; },
                }
            }
    crate::port_hook!(FT_800892A0, ft_800892A0, crate::adapt::ft_800892A0, (gobj: *mut HSD_GObj));
    crate::port_hook!(CAMERA_REQUESTQUAKE, Camera_RequestQuake, crate::adapt::Camera_RequestQuake, (kind: CmQuakeKind, pos: *mut Vec3));
    crate::port_hook!(FTCOLL_8007B128, ftColl_8007B128, crate::adapt::ftColl_8007B128, (gobj: *mut HSD_GObj, bone_id: i32, state: i32));
    crate::port_hook!(FTCOLL_8007B0C0, ftColl_8007B0C0, crate::adapt::ftColl_8007B0C0, (gobj: *mut HSD_GObj, state: i32));
    pub static mut FTCOMMON_8007F5CC: Option<fn(*mut HSD_GObj, isize) > = None;
            #[allow(non_snake_case)]
            pub fn ftCommon_8007F5CC(gobj: *mut HSD_GObj, arg: i32)  {
                match unsafe { FTCOMMON_8007F5CC } {
                    Some(f) => f(gobj, arg as isize),
                    None => { unsafe { let _ = melee_fighter::ftcommon3::ftcommon_8007f5cc(gobj as _, arg as _); } return; },
                }
            }
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}
#[inline(always)]
unsafe fn da(fp: *mut Fighter) -> *mut ftKoopaAttributes {
    (*fp).dat_attrs as *mut ftKoopaAttributes
}

/// `ftKp_Init_803CF2A0`: the direction table the flame picks its gfx slot from (96 entries).
pub const ftKp_Init_803CF2A0: [i32; 96] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 0, 0, 0, 0, 0, 0, 0, 0, 3, 3, 3, 3, 3, 3, 3, 3, 0, 0, 0, 0, 0, 0, 0, 0, 3, 3, 3, 3, 3, 3,
    3, 3, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2,
    2, 2, 2,
];

/// `ftKp_SpecialN_80134ACC_inline` (static inline): pick the next flame direction.
pub fn ftKp_SpecialN_80134ACC_inline(gobj: *mut HSD_GObj, dirs: &[i32; 96]) -> i32 {
    melee_core::ct!("ftKp_SpecialN_80134ACC_inline");
    unsafe {
        let fp = fpp(gobj);
        let new_facing = match (*fp).mv.kp.specialn.facing_dir {
            1 | 2 => dirs[(fh::hsd_randi(0x20) + 0x20) as usize],
            _ => {
                if dirs[fh::hsd_randi(0x20) as usize] == 0 {
                    dirs[(fh::hsd_randi(0x20) + 0x20) as usize]
                } else {
                    dirs[(fh::hsd_randi(0x20) + 0x40) as usize]
                }
            }
        };
        (*fp).mv.kp.specialn.facing_dir = new_facing;
        new_facing
    }
}

/// Which of the three flame sfx tiers the breath uses (`f < 0.3333f`, `f < 0.6666f`, else).
pub fn flame_sfx(kind: FighterKind, f: f32) -> i32 {
    melee_core::ct!("flame_sfx");
    let koopa = kind == Ft_Kind_Koopa;
    if f < 0.3333_f32 {
        if koopa { 0x24A2B } else { 0x1D4F3 }
    } else if f < 0.6666_f32 {
        if koopa { 0x24A28 } else { 0x1D4F0 }
    } else if koopa {
        0x24A25
    } else {
        0x1D4ED
    }
}

pub fn ftKp_SpecialLw_80134ACC(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKp_SpecialLw_80134ACC");
    unsafe {
        let fp = fpp(gobj);
        let da = &*da(fp);
        let dirs = &ftKp_Init_803CF2A0;
        let mut v = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        ext::lb_8000B1CC((*(*fp).parts.add(48)).joint, core::ptr::null(), &mut v);
        // v.x += scale.y * (x24 * facing); v.y += x28 * scale.y   (both fmuladd in the shipping IR)
        v.x = fmadds((*fp).x34_scale.y, da.x24 * (*fp).facing_dir, v.x);
        v.y = fmadds(da.x28, (*fp).x34_scale.y, v.y);
        // Argument order of the C call: parent, pos, facing_dir, x4, <inline>, x222C, x2230, kind.
        let x4 = (*fp).mv.kp.specialn.x4;
        let gfx = ftKp_SpecialN_80134ACC_inline(gobj, dirs);
        ext::itKoopaFlame_Spawn(
            gobj,
            &mut v,
            (*fp).facing_dir,
            x4,
            gfx,
            melee_core::ppc::fctiwz_f32((*fp).u.kp.x222C),
            melee_core::ppc::fctiwz_f32((*fp).u.kp.x2230),
            It_Kind_Koopa_Flame as i32,
        );
        if (*fp).mv.kp.specialn.x14 == 0 {
            (*fp).mv.kp.specialn.x4 = ext::Item_8026AE60();
            ext::ft_80089824(gobj);
            ext::ft_800892A0(gobj);
        }
        if (*fp).mv.kp.specialn.x14 % 3 == 0 {
            let f = ((*fp).u.kp.x2230 - da.x1C) / (da.x18 - da.x1C);
            let sfx = flame_sfx((*fp).kind as FighterKind, f);
            melee_fighter::hooks::play_sfx(&mut *fp, sfx, 0x7F, 0x40);
        }
        (*fp).mv.kp.specialn.x14 += 1;
        (*fp).mv.kp.specialn.x14 %= 0xC;
    }
}

/// `ftKp_SpecialLw_80134D78` (`ftKp_Init_UnkMotionStates3` calls it): refill the flame meters.
pub fn ftKp_SpecialLw_80134D78(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKp_SpecialLw_80134D78");
    unsafe {
        let fp = fpp(gobj);
        let da = &*da(fp);
        if (*fp).motion_id >= 0x15B || (*fp).motion_id < 0x155 {
            (*fp).u.kp.x222C = (*fp).u.kp.x222C + da.x8;
            if (*fp).u.kp.x222C > da.x10 {
                (*fp).u.kp.x222C = da.x10;
            }
            (*fp).u.kp.x2230 = (*fp).u.kp.x2230 + da.xC;
            if (*fp).u.kp.x2230 > da.x18 {
                (*fp).u.kp.x2230 = da.x18;
            }
        }
    }
}

pub fn ftKp_SpecialLw_80134DE0(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("ftKp_SpecialLw_80134DE0");
    unsafe {
        if !gobj.is_null() {
            let fp = fpp(gobj);
            if !fp.is_null() {
                return melee_core::ppc::fctiwz_f32((*da(fp)).x10);
            }
        }
        1
    }
}

pub fn ftKp_SpecialLw_80134E1C(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("ftKp_SpecialLw_80134E1C");
    unsafe {
        if !gobj.is_null() {
            let fp = fpp(gobj);
            if !fp.is_null() {
                return melee_core::ppc::fctiwz_f32((*da(fp)).x18);
            }
        }
        2
    }
}

/// `ftKp_SpecialN_Enter_inline` (static).
pub fn ftKp_SpecialN_Enter_inline(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKp_SpecialN_Enter_inline");
    unsafe {
        let fp = fpp(gobj);
        (*fp).mv.kp.specialn.x0 = 0;
        (*fp).mv.kp.specialn.x4 = ext::Item_8026AE60();
        (*fp).mv.kp.specialn.facing_dir = 0;
        (*fp).mv.kp.specialn.x10 = 1;
        (*fp).mv.kp.specialn.x14 = 0;
        (*fp).mv.kp.specialn.x18 = 0;
        (*fp).mv.kp.specialn.flame_timer = 0;
    }
}

pub fn ftKp_SpecialN_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKp_SpecialN_Enter");
    st::Fighter_ChangeMotionState(gobj, 0x155, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
    ftKp_SpecialN_Enter_inline(gobj);
}

pub fn ftKp_SpecialAirN_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKp_SpecialAirN_Enter");
    st::Fighter_ChangeMotionState(gobj, 0x158, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
    ftKp_SpecialN_Enter_inline(gobj);
}

/// `ftKp_SpecialN_Anim_inline_2` (static): quake on every `x20`th frame.
pub fn ftKp_SpecialN_Anim_inline_2(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKp_SpecialN_Anim_inline_2");
    unsafe {
        let fp = fpp(gobj);
        let da = &*da(fp);
        if (*fp).mv.kp.specialn.x18 == 0 {
            ext::Camera_RequestQuake(QuakeKind_Small, &mut (*fp).cur_pos);
        }
        (*fp).mv.kp.specialn.x18 += 1;
        // C `%` on s32: x20 == 0 is UB in C; the gwtool contract defines it as 0.
        (*fp).mv.kp.specialn.x18 = if da.x20 == 0 { 0 } else { (*fp).mv.kp.specialn.x18.wrapping_rem(da.x20) };
    }
}

pub fn ftKp_SpecialN_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKp_SpecialN_Anim");
    unsafe {
        ftKp_SpecialN_Anim_inline_1(&mut *fpp(gobj));
    }
    ftKp_SpecialN_Anim_inline_2(gobj);
}

pub fn ftKp_SpecialAirN_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKp_SpecialAirN_Anim");
    unsafe {
        ftKp_SpecialN_Anim_inline_1(&mut *fpp(gobj));
    }
    ftKp_SpecialN_Anim_inline_2(gobj);
}

pub fn ftKp_SpecialNStart_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftKp_SpecialNStart_IASA");}
pub fn ftKp_SpecialNEnd_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftKp_SpecialNEnd_IASA");}
pub fn ftKp_SpecialAirNStart_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftKp_SpecialAirNStart_IASA");}
pub fn ftKp_SpecialAirNEnd_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftKp_SpecialAirNEnd_IASA");}

/// Shared body of the two loop IASAs; `end_msid` is the End state entered when B is released
/// (0x157 on the ground, 0x15A in the air).
unsafe fn n_iasa(gobj: *mut HSD_GObj, end_msid: i32) {
    let fp = fpp(gobj);
    let da = &*da(fp);
    if (*fp).mv.kp.specialn.flame_timer >= da.flame_limit {
        if (*fp).input.held_buttons[0] as i32 & HSD_PAD_B != 0 {
            if (*fp).mv.kp.specialn.x0 == 0 {
                ftKp_SpecialLw_80134ACC(gobj);
            }
        } else if (*fp).mv.kp.specialn.x10 != 0 {
            if (*fp).mv.kp.specialn.x0 == 0 {
                ftKp_SpecialLw_80134ACC(gobj);
            }
        } else {
            st::Fighter_ChangeMotionState(gobj, end_msid, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        }
    } else if (*fp).mv.kp.specialn.x0 == 0 {
        ftKp_SpecialLw_80134ACC(gobj);
    }
    (*fp).mv.kp.specialn.x0 += 1;
    if (*fp).mv.kp.specialn.x0 >= 3 {
        (*fp).mv.kp.specialn.x0 = 0;
    }
    (*fp).u.kp.x222C -= 1.0;
    if (*fp).u.kp.x222C < da.x14 {
        (*fp).u.kp.x222C = da.x14;
    }
    (*fp).u.kp.x2230 -= 1.0;
    if (*fp).u.kp.x2230 < da.x1C {
        (*fp).u.kp.x2230 = da.x1C;
    }
    (*fp).mv.kp.specialn.flame_timer += 1;
    if (*fp).mv.kp.specialn.flame_timer > da.flame_limit {
        (*fp).mv.kp.specialn.flame_timer = da.flame_limit;
    }
}

pub fn ftKp_SpecialN_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKp_SpecialN_IASA");
    unsafe { n_iasa(gobj, 0x157) }
}

pub fn ftKp_SpecialAirN_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKp_SpecialAirN_IASA");
    unsafe { n_iasa(gobj, 0x15A) }
}

/// `ftKp_SpecialN_80135780_inline` (static): intangible body while the script bit is set.
pub fn ftKp_SpecialN_80135780_inline(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKp_SpecialN_80135780_inline");
    st::ftParts_80074B0C(gobj, 0, 1);
    for bone in [0x37, 0x30, 0x3C, 0x21, 0x3D, 0x22, 0xF, 6, 0x10, 7, 0x13, 0xA] {
        ext::ftColl_8007B128(gobj, bone, HurtCapsule_Intangible as i32);
    }
    ext::ftCommon_8007F5CC(gobj, 0);
}

/// `ftKp_SpecialN_80135780_inline_2` (static).
pub fn ftKp_SpecialN_80135780_inline_2(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKp_SpecialN_80135780_inline_2");
    st::ftParts_80074B0C(gobj, 0, 0);
    ext::ftColl_8007B0C0(gobj, HurtCapsule_Enabled as i32);
    ext::ftCommon_8007F5CC(gobj, 1);
}

pub fn ftKp_SpecialN_80135780(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKp_SpecialN_80135780");
    unsafe {
        let fp = fpp(gobj);
        let bit = if (*fp).ground_or_air == GA_Ground { (*fp).cmd_vars[2] } else { (*fp).cmd_vars[1] };
        if bit & 1 != 0 {
            ftKp_SpecialN_80135780_inline(gobj);
            (*fp).set_x221E_b4(0);
        } else {
            ftKp_SpecialN_80135780_inline_2(gobj);
            (*fp).set_x221E_b4(1);
        }
    }
}

// ---- folded from chars (consolidation step 20/21) ----
/// `ftKp_SpecialN_Anim_inline_1` (static).
pub fn ftKp_SpecialN_Anim_inline_1(fp: &mut Fighter) {
    melee_core::ct!("ftKp_SpecialN_Anim_inline_1");
    if fp.cur_anim_frame == 0.0 {
        unsafe {
            fp.mv.kp.specialn.x10 -= 1;
            if fp.mv.kp.specialn.x10 <= 0 {
                fp.mv.kp.specialn.x10 = 0;
            }
        }
    }
}

/// `ftKp_SpecialNStart_Anim` (src/melee/ft/kinds/ftKoopa/ftkoopaspecialn.c).
pub fn ftKp_SpecialNStart_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKp_SpecialNStart_Anim");
    let fp = unsafe { &mut *((*gobj).user_data as *mut Fighter) };
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        st::Fighter_ChangeMotionState(gobj, 342 /* ftKp_MS_SpecialN */, 0x0u32 /* Ft_MF_None */, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
    }
}

/// `ftKp_SpecialNEnd_Anim` (src/melee/ft/kinds/ftKoopa/ftkoopaspecialn.c).
pub fn ftKp_SpecialNEnd_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKp_SpecialNEnd_Anim");
    let fp = unsafe { &mut *((*gobj).user_data as *mut Fighter) };
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        st::ft_8008A2BC(gobj);
    }
}

/// `ftKp_SpecialAirNStart_Anim` (src/melee/ft/kinds/ftKoopa/ftkoopaspecialn.c).
pub fn ftKp_SpecialAirNStart_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKp_SpecialAirNStart_Anim");
    let fp = unsafe { &mut *((*gobj).user_data as *mut Fighter) };
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        st::Fighter_ChangeMotionState(gobj, 345 /* ftKp_MS_SpecialAirN */, 0x0u32 /* Ft_MF_None */, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
    }
}

/// `ftKp_SpecialAirNEnd_Anim` (src/melee/ft/kinds/ftKoopa/ftkoopaspecialn.c).
pub fn ftKp_SpecialAirNEnd_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKp_SpecialAirNEnd_Anim");
    let fp = unsafe { &mut *((*gobj).user_data as *mut Fighter) };
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        st::ftCo_Fall_Enter(gobj);
    }
}

/// `ftKp_SpecialNStart_Phys` (src/melee/ft/kinds/ftKoopa/ftkoopaspecialn.c).
pub fn ftKp_SpecialNStart_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKp_SpecialNStart_Phys");
    st::ft_80084F3C(gobj);
}

/// `ftKp_SpecialN_Phys` (src/melee/ft/kinds/ftKoopa/ftkoopaspecialn.c).
pub fn ftKp_SpecialN_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKp_SpecialN_Phys");
    st::ft_80084F3C(gobj);
}

/// `ftKp_SpecialNEnd_Phys` (src/melee/ft/kinds/ftKoopa/ftkoopaspecialn.c).
pub fn ftKp_SpecialNEnd_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKp_SpecialNEnd_Phys");
    st::ft_80084F3C(gobj);
}

/// `ftKp_SpecialAirNStart_Phys` (src/melee/ft/kinds/ftKoopa/ftkoopaspecialn.c).
pub fn ftKp_SpecialAirNStart_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKp_SpecialAirNStart_Phys");
    st::ft_80084DB0(gobj);
}

/// `ftKp_SpecialAirN_Phys` (src/melee/ft/kinds/ftKoopa/ftkoopaspecialn.c).
pub fn ftKp_SpecialAirN_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKp_SpecialAirN_Phys");
    st::ft_80084DB0(gobj);
}

/// `ftKp_SpecialAirNEnd_Phys` (src/melee/ft/kinds/ftKoopa/ftkoopaspecialn.c).
pub fn ftKp_SpecialAirNEnd_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKp_SpecialAirNEnd_Phys");
    st::ft_80084DB0(gobj);
}

/// `ftKp_SpecialNStart_Coll` (src/melee/ft/kinds/ftKoopa/ftkoopaspecialn.c).
pub fn ftKp_SpecialNStart_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKp_SpecialNStart_Coll");
    let fp = unsafe { &mut *((*gobj).user_data as *mut Fighter) };
    if st::ft_80082708(gobj) == 0 /* GA_Ground */ {
        st::ftCommon_GroundToAirStateChange(gobj, fp, 344 /* ftKp_MS_SpecialAirNStart */, 0xC4C5080u32 /* ftCommon_GroundAirColl_MF */);
    }
}

/// `ftKp_SpecialN_Coll` (src/melee/ft/kinds/ftKoopa/ftkoopaspecialn.c).
pub fn ftKp_SpecialN_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKp_SpecialN_Coll");
    let fp = unsafe { &mut *((*gobj).user_data as *mut Fighter) };
    if st::ft_80082708(gobj) == 0 /* GA_Ground */ {
        st::ftCommon_GroundToAirStateChange(gobj, fp, 345 /* 0x159 */, 0xC4C5880u32 /* ftKp_MF_SpecialN_Coll */);
    }
}

/// `ftKp_SpecialNEnd_Coll` (src/melee/ft/kinds/ftKoopa/ftkoopaspecialn.c).
pub fn ftKp_SpecialNEnd_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKp_SpecialNEnd_Coll");
    let fp = unsafe { &mut *((*gobj).user_data as *mut Fighter) };
    if st::ft_80082708(gobj) == 0 /* GA_Ground */ {
        st::ftCommon_GroundToAirStateChange(gobj, fp, 346 /* 0x15A */, 0xC4C5080u32 /* ftCommon_GroundAirColl_MF */);
    }
}

/// `ftKp_SpecialAirNStart_Coll` (src/melee/ft/kinds/ftKoopa/ftkoopaspecialn.c).
pub fn ftKp_SpecialAirNStart_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKp_SpecialAirNStart_Coll");
    let fp = unsafe { &mut *((*gobj).user_data as *mut Fighter) };
    if st::ft_80081D0C(gobj) != 0 /* GA_Ground */ {
        st::ftCommon_AirToGroundStateChange(gobj, fp, 341 /* 0x155 */, 0xC4C5080u32 /* ftCommon_GroundAirColl_MF */);
    }
}

/// `ftKp_SpecialAirN_Coll` (src/melee/ft/kinds/ftKoopa/ftkoopaspecialn.c).
pub fn ftKp_SpecialAirN_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKp_SpecialAirN_Coll");
    let fp = unsafe { &mut *((*gobj).user_data as *mut Fighter) };
    if st::ft_80081D0C(gobj) != 0 /* GA_Ground */ {
        st::ftCommon_AirToGroundStateChange(gobj, fp, 342 /* 0x156 */, 0xC4C5880u32 /* ftKp_MF_SpecialN_Coll */);
    }
}

/// `ftKp_SpecialAirNEnd_Coll` (src/melee/ft/kinds/ftKoopa/ftkoopaspecialn.c).
pub fn ftKp_SpecialAirNEnd_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKp_SpecialAirNEnd_Coll");
    let fp = unsafe { &mut *((*gobj).user_data as *mut Fighter) };
    if st::ft_80081D0C(gobj) != 0 /* GA_Ground */ {
        st::ftCommon_AirToGroundStateChange(gobj, fp, 343 /* 0x157 */, 0xC4C5080u32 /* ftCommon_GroundAirColl_MF */);
    }
}
