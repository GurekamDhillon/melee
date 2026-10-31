#![allow(non_snake_case, non_upper_case_globals)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_HammerFall.c. Every function of the TU; calls out go
//! through `ext`. Checked against the C by tests/tu_ftCo_HammerFall.rs.

use crate::types::*;

pub type G = *mut HSD_GObj;

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}
#[allow(dead_code)]
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
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ftCo_800C54C4(fp: &mut Fighter) -> u32 = |fp: &mut Fighter| -> u32 { unsafe { crate::tu::ftCo_HammerWait::ftCo_800C54C4(fp as _) as _ } };
        ftCo_800C548C(fp: &mut Fighter) -> f32 = |fp: &mut Fighter| -> f32 { unsafe { crate::tu::ftCo_HammerWait::ftCo_800C548C(fp as _) as _ } };
        ftCo_800C4E94(fp: &mut Fighter) = |fp: &mut Fighter| { unsafe { crate::tu::ftCo_HammerWait::ftCo_800C4E94(fp as _); } };
        ftCo_800C4F64(g: G) = |g: G| { unsafe { crate::tu::ftCo_HammerWait::ftCo_800C4F64(g as _); } };
        ftCo_800C5240(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_HammerWait::ftCo_800C5240(g as _) as _ } };
        ftCo_800C544C(fp: &mut Fighter) = |fp: &mut Fighter| { unsafe { crate::tu::ftCo_HammerWait::ftCo_800C544C(fp as _); } };
        ftCommon_8007D5D4(fp: &mut Fighter) = crate::ftcommon2::ftcommon_8007d5d4;
        Item_8026ABD8(ig: *mut HSD_GObj, pos: &mut Vec3, v: f32) = |ig: *mut HSD_GObj, pos: &mut Vec3, v: f32| { unsafe { melee_items::tu::item::Item_8026ABD8(ig as _, pos as _, v as _); } };
        ftCo_Fall_Enter(g: G) = |g: G| { unsafe { crate::tu::ftCo_Fall::ftCo_Fall_Enter(g as _); } };
        pl_8003FDF4(slot: i32, sub: i32) = crate::plbonus::pl_8003fdf4;
        ft_80084DB0(g: G) = crate::ft084e::ft_80084db0;
        ft_80082C74(g: G, cb: fn(G)) = crate::tu::ft_081b::ft_80082c74;
        ftCo_HammerLanding_Enter(g: G) = crate::tu::ftCo_HammerLanding::ftCo_HammerLanding_Enter;
    }
}

const HSD_PAD_AB: u32 = (HSD_PAD_A | HSD_PAD_B) as u32;

pub fn ftCo_800C5CD4(g: G) -> bool {
    melee_core::ct!("ftCo_800C5CD4");
    if ext::ftCo_800C5240(g) && fp(g).input.pressed_buttons & HSD_PAD_AB != 0 {
        ftCo_800C5D34(g);
        return true;
    }
    false
}

pub fn ftCo_800C5D34(g: G) {
    melee_core::ct!("ftCo_800C5D34");
    let flags = ext::ftCo_800C54C4(fp(g));
    let start = ext::ftCo_800C548C(fp(g));
    ext::Fighter_ChangeMotionState(g, ftCo_MS_HammerFall as i32, flags, start, 1.0, 0.0, core::ptr::null_mut());
    if fp(g).ground_or_air == GA_Ground as _ {
        ext::ftCommon_8007D5D4(fp(g));
    }
    ext::ftCo_800C4E94(fp(g));
}

pub fn ftCo_HammerFall_Anim(g: G) {
    melee_core::ct!("ftCo_HammerFall_Anim");
    ext::ftCo_800C4F64(g);
}

/// `inlineA0` (static inline).
fn inline_a0(g: G) {
    let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    ext::Item_8026ABD8(fp(g).item_gobj as *mut HSD_GObj, &mut pos, 1.0);
    ext::ftCo_Fall_Enter(g);
    ext::ftCo_800C544C(fp(g));
    let f = fp(g);
    ext::pl_8003FDF4(f.player_id as i32, f.is_sub_fighter() as i32);
}

pub fn ftCo_800C5DDC(g: G) -> bool {
    melee_core::ct!("ftCo_800C5DDC");
    let f = fp(g);
    if f.x2338.y != 0
        && f.input.pressed_buttons & HSD_PAD_A as u32 != 0
        && f.input.held_buttons[0] & HSD_PAD_LR as u32 != 0
        && f.x683 as i32 >= cd().x1C
    {
        inline_a0(g);
        return true;
    }
    false
}

pub fn ftCo_HammerFall_IASA(g: G) {
    melee_core::ct!("ftCo_HammerFall_IASA");
    ftCo_800C5DDC(g);
}

pub fn ftCo_HammerFall_Phys(g: G) {
    melee_core::ct!("ftCo_HammerFall_Phys");
    ext::ft_80084DB0(g);
}

pub fn ftCo_HammerFall_Coll(g: G) {
    melee_core::ct!("ftCo_HammerFall_Coll");
    ext::ft_80082C74(g, ext::ftCo_HammerLanding_Enter);
}
