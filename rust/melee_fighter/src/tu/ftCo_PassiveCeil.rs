#![allow(non_snake_case, unused_unsafe)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_PassiveCeil.c (every function of the TU). Calls out of it
//! go through `ext`. Checked against the C (whole TU, shipping contraction) by tests/tu_ftCo_PassiveCeil.rs.
use crate::types::*;

pub type G = *mut HSD_GObj;

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}
#[inline(always)]
fn cd() -> &'static ftCommonData {
    crate::hooks::common_data()
}

/// Calls out of the TU. `hook::<name>` overrides; unset = the port, or a TODO panic.
pub mod ext {
    use super::*;
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
    ext_fns! {
        ftCo_800986B0(g: G) -> bool = crate::tu::ftCo_DownAttack::ftCo_800986B0;
        ftCommon_8007E2FC(g: G) = |a0| crate::tu::ftcommon::ftcommon_8007e2fc(unsafe { &mut *((*(a0 as *mut HSD_GObj)).user_data as *mut Fighter) });
        ftKb_SpecialN_800F1F1C(g: G, ofs: &mut Vec3) = |_, _| todo!("TODO(port) ftKb_SpecialN_800F1F1C (ftKb_SpecialN.c)");
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        efAsync_Spawn(g: G, queue: *mut core::ffi::c_void, kind: u32, gfx: u32, jobj: *mut HSD_JObj, pos: *mut Vec3) = |a0, a1, a2, a3, a4, a5| unsafe { let _ = melee_world::tu::efn::efasync::efAsync_Spawn(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _, &[a5 as *const core::ffi::c_void]); };
        ftCo_80090574(g: G) = |a0| crate::tu::ftCo_DamageFall::ftco_80090574(a0);
        ft_800881D8(fp: &mut Fighter, sfx: i32, vol: u8, pan: u8) = |a0, a1, a2, a3| crate::tu::ft_0881::ft_800881d8(a0, a1, a2, a3);
        ft_PlaySFX(fp: &mut Fighter, id: i32, vol: u8, pan: u8) = |_, _, _, _| todo!("ft_PlaySFX: installed by melee_link");
        ftCommon_8007EBAC(fp: &mut Fighter, a: u32, b: u32) = |a0, a1, a2| crate::tu::ftcommon::ftcommon_8007ebac(a0, a1, a2);
        ftCo_800BFFD0(fp: &mut Fighter, a: i32, b: bool) -> bool = |fp: &mut Fighter, a: i32, b: bool| -> bool { unsafe { crate::tu::ftcolanim::ftco_800bffd0(fp as _, a as _, b as _) as _ } };
        ftAnim_IsFramesRemaining(g: G) -> bool = crate::hooks::ftanim_is_frames_remaining;
        ftCo_Fall_Enter(g: G) = crate::tu::ftCo_Fall::ftCo_Fall_Enter;
        ft_80084DB0(g: G) = crate::ft084e::ft_80084db0;
        ft_800831CC(g: G, a: fn(G, i32) -> bool, b: fn(G)) = crate::tu::ft_081b::ft_800831cc;
        ftCo_80096CC8(g: G, line_id: i32) -> bool = |g: G, line_id: i32| -> bool { unsafe { crate::tu::ftCo_FallSpecial::ftCo_80096CC8(g as _, line_id as _) as _ } };
        ft_80082B1C(g: G) = crate::tu::ft_081b::ft_80082b1c;
    }
}

fn cb_80096CC8(g: G, line_id: i32) -> bool {
    ext::ftCo_80096CC8(g, line_id)
}
fn cb_80082B1C(g: G) {
    ext::ft_80082B1C(g)
}

pub fn ftCo_800C23A0(g: G) -> bool {
    melee_core::ct!("ftCo_800C23A0");
    if fp(g).coll_data.env_flags & Collide_CeilingHug != 0 && ext::ftCo_800986B0(g) {
        ftCo_800C23FC(g);
        return true;
    }
    false
}

pub fn ftCo_800C23FC(g: G) {
    melee_core::ct!("ftCo_800C23FC");
    ext::ftCommon_8007E2FC(g);
    let f = fp(g);
    f.anon_2210.throw_flags = 0;
    let mut ef_offset = Vec3 { x: 0.0, y: f.coll_data.ecb.top.y, z: 0.0 };
    ext::ftKb_SpecialN_800F1F1C(g, &mut ef_offset);
    ext::Fighter_ChangeMotionState(g, ftCo_MS_PassiveCeil as i32, Ft_MF_None as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
    let f = fp(g);
    let pos_y_offset = f.cur_pos.y + ef_offset.y;
    let mut ef_pos = Vec3 { x: f.cur_pos.x, y: pos_y_offset, z: f.cur_pos.z };
    ext::efAsync_Spawn(g, &mut f.x60C as *mut _ as *mut core::ffi::c_void, 2, 0x41D, core::ptr::null_mut(), &mut ef_pos);
    let f = fp(g);
    f.cur_pos.y = pos_y_offset + f.x68C_transNPos.y;
    ext::ftCo_80090574(g);
    let f = fp(g);
    let sfx = unsafe { (*(*f.ft_data).x4C_sfx).x24 };
    ext::ft_800881D8(f, sfx, 0x7F, 0x40);
    ext::ft_PlaySFX(fp(g), 3, 0x7F, 0x40);
    ext::ftCommon_8007EBAC(fp(g), 12, 0);
    ext::ftCo_800BFFD0(fp(g), 120, false);
}

pub fn ftCo_PassiveCeil_Anim(g: G) {
    melee_core::ct!("ftCo_PassiveCeil_Anim");
    let f = fp(g);
    // ftCheckThrowB3 (ft/inlines.h)
    let b3 = unsafe {
        if f.anon_2210.anon_0.throw_flags_b3() != 0 {
            f.anon_2210.anon_0.set_throw_flags_b3(0);
            true
        } else {
            false
        }
    };
    if b3 {
        f.self_vel.x = f.input.lstick[0].x * f.co_attrs.passiveceil_vel_x;
    }
    if !ext::ftAnim_IsFramesRemaining(g) {
        ext::ftCo_Fall_Enter(g);
    }
}

pub fn ftCo_PassiveCeil_IASA(_g: G) { melee_core::ct!("ftCo_PassiveCeil_IASA");}

pub fn ftCo_PassiveCeil_Phys(g: G) {
    melee_core::ct!("ftCo_PassiveCeil_Phys");
    ext::ft_80084DB0(g);
}

pub fn ftCo_PassiveCeil_Coll(g: G) {
    melee_core::ct!("ftCo_PassiveCeil_Coll");
    ext::ft_800831CC(g, cb_80096CC8, cb_80082B1C);
}
