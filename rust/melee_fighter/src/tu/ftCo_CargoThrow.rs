#![allow(non_snake_case, non_upper_case_globals)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_CargoThrow.c. Every function of the TU; calls out of it go
//! through `ext` (unported ones panic until rewired). Checked against the C (whole TU, shipping
//! contraction) by tests/tu_ftCo_CargoThrow.rs.
use crate::types::*;

pub type G = *mut HSD_GObj;

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}

macro_rules! ext_fns {
    ($($name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? = $def:expr;)*) => {
        pub mod hook {
            #[allow(unused_imports)] use super::*;
            $( #[allow(non_upper_case_globals)] pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
        }
        $( #[allow(non_snake_case)] pub fn $name($($a: $t),*) $(-> $r)? {
            match unsafe { hook::$name } { Some(f) => f($($a),*), None => ($def)($($a),*) }
        } )*
    };
}

#[inline(always)]
fn x2cc<'a>(f: &Fighter) -> &'a _ftDonkeyAttributes {
    unsafe { &*f.x2CC }
}


/// Calls out of the TU. `hook::<name>` overrides; unset = the port, or a TODO panic.
pub mod ext {
    use super::*;
    ext_fns! {
        ftCo_8008F744(g: G) = |_| todo!("TODO(port) ftCo_8008F744 (ftCo_Damage.c)");
        ftCo_8009BC58(g: G) = crate::tu::ftCo_CargoFall::ftCo_8009BC58;
        ftCo_8009B518(g: G) = crate::tu::ftCo_CargoWait::ftCo_8009B518;
        ftCo_Damage_Phys(g: G) = |_| todo!("TODO(port) ftCo_Damage_Phys (ftCo_Damage.c)");
        ft_80082708(g: G) -> u32 = |_| todo!("TODO(port) ft_80082708 (ft_081B.c)");
        ft_80081D0C(g: G) -> u32 = |_| todo!("TODO(port) ft_80081D0C (ft_081B.c)");
        ftCommon_8007D5D4(f: &mut Fighter) = crate::ftcommon2::ftcommon_8007d5d4;
        ftCommon_8007D7FC(f: &mut Fighter) = crate::ftcommon3::ftcommon_8007d7fc;
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |_, _, _, _, _, _, _| todo!("TODO(port) Fighter_ChangeMotionState (fighter.c)");
        ftAnim_8006EBA4(g: G) = |_| todo!("TODO(port) ftAnim_8006EBA4 (ftanim.c)");
        ftCommon_8007E2F4(f: &mut Fighter, v: i16) = crate::ftcommon2::ftcommon_8007e2f4;
        ftCo_800DE3FC(g: G, msid: i32, speed: f32) = |_, _, _| todo!("TODO(port) ftCo_800DE3FC (ftCo_Thrown.c)");
        ftCo_800DD724(g: G) = |_| todo!("TODO(port) ftCo_800DD724 (ftCo_Throw.c)");
        ftAnim_IsFramesRemaining(g: G) -> bool = crate::hooks::ftanim_is_frames_remaining;
        ftCommon_8007D92C(g: G) = crate::ftcommon3::ftcommon_8007d92c;
        ftCo_ThrowF_IASA(g: G) = |_| todo!("TODO(port) ftCo_ThrowF_IASA (ftCo_Throw.c)");
        ftCo_ThrowF_Phys(g: G) = |_| todo!("TODO(port) ftCo_ThrowF_Phys (ftCo_Throw.c)");
        ftCo_ThrowB_Anim(g: G) = |_| todo!("TODO(port) ftCo_ThrowB_Anim (ftCo_Throw.c)");
        ftCo_ThrowB_IASA(g: G) = |_| todo!("TODO(port) ftCo_ThrowB_IASA (ftCo_Throw.c)");
        ftCo_ThrowB_Phys(g: G) = |_| todo!("TODO(port) ftCo_ThrowB_Phys (ftCo_Throw.c)");
        ftCo_ThrowHi_Anim(g: G) = |_| todo!("TODO(port) ftCo_ThrowHi_Anim (ftCo_Throw.c)");
        ftCo_ThrowHi_IASA(g: G) = |_| todo!("TODO(port) ftCo_ThrowHi_IASA (ftCo_Throw.c)");
        ftCo_ThrowHi_Phys(g: G) = |_| todo!("TODO(port) ftCo_ThrowHi_Phys (ftCo_Throw.c)");
        ftCo_ThrowLw_Anim(g: G) = |_| todo!("TODO(port) ftCo_ThrowLw_Anim (ftCo_Throw.c)");
        ftCo_ThrowLw_IASA(g: G) = |_| todo!("TODO(port) ftCo_ThrowLw_IASA (ftCo_Throw.c)");
        ftCo_ThrowLw_Phys(g: G) = |_| todo!("TODO(port) ftCo_ThrowLw_Phys (ftCo_Throw.c)");
        ft_8008403C(g: G, cb: HSD_GObjEvent) = |_, _| todo!("TODO(port) ft_8008403C (ft_081B.c)");
        ft_80084DB0(g: G) = crate::ft084e::ft_80084db0;
        ft_80082C74(g: G, cb: HSD_GObjEvent) = |_, _| todo!("TODO(port) ft_80082C74 (ft_081B.c)");
    }
}

pub fn ftCo_CargoWait2_Anim(g: G) {
    ext::ftCo_8008F744(g);
    let f = fp(g);
    if f.x221C_b6() == 0 {
        if f.ground_or_air == GA_Air {
            ext::ftCo_8009BC58(g);
        } else {
            ext::ftCo_8009B518(g);
        }
    }
}

pub fn ftCo_CargoWait2_Phys(g: G) {
    ext::ftCo_Damage_Phys(g);
}

pub fn ftCo_CargoWait2_Coll(g: G) {
    if fp(g).ground_or_air == GA_Ground {
        if ext::ft_80082708(g) == GA_Ground {
            ext::ftCommon_8007D5D4(fp(g));
        }
    } else if ext::ft_80081D0C(g) != GA_Ground {
        ext::ftCommon_8007D7FC(fp(g));
    }
}

/// `ABS` (C macro, `x < 0 ? -x : x`).
#[inline(always)]
fn abs_c(x: f32) -> f32 {
    if x < 0.0 { -x } else { x }
}

/// `inlineA0`: the throw motion picked by A/B + stick direction (unchanged = no throw).
fn inline_a0(f: &Fighter) -> i32 {
    let mut msid = f.motion_id;
    let cd = crate::hooks::common_data();
    if f.input.pressed_buttons & (HSD_PAD_A | HSD_PAD_B) as u32 != 0 {
        if abs_c(f.input.lstick[0].x) >= cd.x98 {
            if f.input.lstick[0].x * f.facing_dir > 0.0 {
                msid = x2cc(f).x4_motion_state.wrapping_add(10);
            } else {
                msid = x2cc(f).x4_motion_state.wrapping_add(11);
            }
        } else if f.input.lstick[0].y >= cd.attackhi3_stick_threshold_y {
            msid = x2cc(f).x4_motion_state.wrapping_add(12);
        } else if f.input.lstick[0].y <= cd.xB0 {
            msid = x2cc(f).x4_motion_state.wrapping_add(13);
        }
    }
    msid
}

pub fn ftCo_8009BF3C(g: G) -> bool {
    let msid = inline_a0(fp(g));
    if msid != fp(g).motion_id {
        ftCo_8009C02C(g, msid);
        return true;
    }
    false
}

pub fn ftCo_8009C02C(g: G, msid: i32) {
    let mf: u32 = if fp(g).ground_or_air == GA_Air { 1 } else { 0 };
    let msid1 = if fp(g).ground_or_air == GA_Air { msid.wrapping_add(4) } else { msid };
    ext::Fighter_ChangeMotionState(g, msid1, mf, 0.0, 1.0, 0.0, core::ptr::null_mut());
    ext::ftAnim_8006EBA4(g);
    ext::ftCommon_8007E2F4(fp(g), 0x1FF);
    let f = fp(g);
    let msid1 = msid
        .wrapping_sub(x2cc(f).x4_motion_state.wrapping_add(10))
        .wrapping_add(ftCo_MS_ThrownFF as i32);
    ext::ftCo_800DE3FC(f.victim_gobj, msid1, 1.0);
}

pub fn ftCo_CargoThrowF_Anim(g: G) {
    ext::ftCo_800DD724(g);
    if !ext::ftAnim_IsFramesRemaining(g) {
        ext::ftCommon_8007D92C(g);
    }
}

pub fn ftCo_CargoThrowF_IASA(g: G) {
    ext::ftCo_ThrowF_IASA(g);
}

pub fn ftCo_CargoThrowF_Phys(g: G) {
    ext::ftCo_ThrowF_Phys(g);
}

/// `inlineB0`: re-enter `msid` at the current frame with the pre-turn facing restored.
fn inline_b0(g: G, msid: i32) {
    let f = fp(g);
    let facing_dir = f.facing_dir;
    if facing_dir != f.facing_dir1 {
        f.facing_dir = f.facing_dir1;
    }
    let frame = f.cur_anim_frame;
    ext::Fighter_ChangeMotionState(
        g,
        msid,
        Ft_MF_SkipMatAnim | Ft_MF_SkipColAnim | Ft_MF_UpdateCmd | Ft_MF_SkipItemVis | Ft_MF_Unk19
            | Ft_MF_SkipModelPartVis | Ft_MF_SkipModelFlags | Ft_MF_Unk27,
        frame,
        1.0,
        0.0,
        core::ptr::null_mut(),
    );
    let f = fp(g);
    f.facing_dir = facing_dir;
    ext::ftCommon_8007E2F4(f, 0x1FF);
}

fn inline_b1(g: G, cb: fn(&mut Fighter), offset: i32) {
    cb(fp(g));
    let ms = fp(g).motion_id.wrapping_add(offset);
    inline_b0(g, ms);
}

pub fn ftCo_8009C170(g: G) {
    inline_b1(g, ext::ftCommon_8007D5D4, 4);
}

/// `ftCo_8009C170` as the C `HSD_GObjEvent` handed to `ft_8008403C`.
pub unsafe extern "C" fn ftCo_8009C170_cb(g: G) {
    ftCo_8009C170(g)
}

pub fn ftCo_CargoThrowF_Coll(g: G) {
    ext::ft_8008403C(g, Some(ftCo_8009C170_cb));
}

pub fn ftCo_CargoThrowB_Anim(g: G) {
    ext::ftCo_ThrowB_Anim(g);
}

pub fn ftCo_CargoThrowB_IASA(g: G) {
    ext::ftCo_ThrowB_IASA(g);
}

pub fn ftCo_CargoThrowB_Phys(g: G) {
    ext::ftCo_ThrowB_Phys(g);
}

pub fn ftCo_CargoThrowB_Coll(g: G) {
    ext::ft_8008403C(g, Some(ftCo_8009C170_cb));
}

pub fn ftCo_CargoThrowHi_Anim(g: G) {
    ext::ftCo_ThrowHi_Anim(g);
}

pub fn ftCo_CargoThrowHi_IASA(g: G) {
    ext::ftCo_ThrowHi_IASA(g);
}

pub fn ftCo_CargoThrowHi_Phys(g: G) {
    ext::ftCo_ThrowHi_Phys(g);
}

pub fn ftCo_CargoThrowHi_Coll(g: G) {
    ext::ft_8008403C(g, Some(ftCo_8009C170_cb));
}

pub fn ftCo_CargoThrowLw_Anim(g: G) {
    ext::ftCo_ThrowLw_Anim(g);
}

pub fn ftCo_CargoThrowLw_IASA(g: G) {
    ext::ftCo_ThrowLw_IASA(g);
}

pub fn ftCo_CargoThrowLw_Phys(g: G) {
    ext::ftCo_ThrowLw_Phys(g);
}

pub fn ftCo_CargoThrowLw_Coll(g: G) {
    ext::ft_8008403C(g, Some(ftCo_8009C170_cb));
}

pub fn ftCo_CargoThrowAir_Anim(g: G) {
    ext::ftCo_800DD724(g);
    if !ext::ftAnim_IsFramesRemaining(g) {
        ext::ftCommon_8007D92C(g);
    }
}

pub fn ftCo_CargoThrowAir_IASA(_g: G) {}

pub fn ftCo_CargoThrowAir_Phys(g: G) {
    ext::ft_80084DB0(g);
}

pub fn ftCo_CargoThrowAir_Coll(g: G) {
    ext::ft_80082C74(g, Some(ftCo_8009C45C_cb));
}

pub fn ftCo_8009C45C(g: G) {
    inline_b1(g, ext::ftCommon_8007D7FC, -4);
}

/// `ftCo_8009C45C` as the C `HSD_GObjEvent` handed to `ft_80082C74`.
pub unsafe extern "C" fn ftCo_8009C45C_cb(g: G) {
    ftCo_8009C45C(g)
}
