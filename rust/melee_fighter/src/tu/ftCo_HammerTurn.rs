#![allow(non_snake_case, non_upper_case_globals, unused_unsafe)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_HammerTurn.c. Every function of the TU; calls out go
//! through `ext`. Checked against the C by tests/tu_ftCo_HammerTurn.rs.

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
        ftCo_800C54C4(fp: &mut Fighter) -> u32 = |fp: &mut Fighter| -> u32 { unsafe { crate::tu::ftCo_HammerWait::ftCo_800C54C4(fp as _) as _ } };
        ftCo_800C548C(fp: &mut Fighter) -> f32 = |fp: &mut Fighter| -> f32 { unsafe { crate::tu::ftCo_HammerWait::ftCo_800C548C(fp as _) as _ } };
        ftCo_800C4E94(fp: &mut Fighter) = |fp: &mut Fighter| { unsafe { crate::tu::ftCo_HammerWait::ftCo_800C4E94(fp as _); } };
        ftCo_800C4F64(g: G) = |g: G| { unsafe { crate::tu::ftCo_HammerWait::ftCo_800C4F64(g as _); } };
        ftCo_800C97A8(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_Turn::ftCo_800C97A8(g as _) as _ } };
        ftCo_Turn_Enter(g: G, msid: i32, flags: u32, a: f32, b: f32, c: f32) = |g: G, msid: i32, flags: u32, a: f32, b: f32, c: f32| { unsafe { crate::tu::ftCo_Turn::ftCo_Turn_Enter(g as _, msid as _, flags as _, a as _, b as _, c as _); } };
        ftCo_800C4ED8(g: G) = |g: G| { unsafe { crate::tu::ftCo_HammerWait::ftCo_800C4ED8(g as _); } };
        ftParts_GetBoneIndex(fp: &mut Fighter, part: u32) -> u32 = |a0, a1| crate::ftdmgfall::ftparts_get_bone_index(unsafe { &*(a0 as *const Fighter) }, a1 as _) as _;
        ftPartSetRotY(fp: &mut Fighter, part: i32, r: f32) = |fp: &mut Fighter, part: i32, r: f32| { unsafe { crate::tu::ftparts::ftPartSetRotY(fp as _, part as _, r as _); } };
        HSD_JObjSetMtxDirtySub(j: *mut HSD_JObj) = |a0| unsafe { melee_runtime::tu::jobj_ptr::HSD_JObjSetMtxDirtySub(a0 as _) };
        ftCo_800C5A50(g: G) -> bool = crate::tu::ftCo_HammerJump::ftCo_800C5A50;
        ft_80084F3C(g: G) = crate::ft084e::ft_80084f3c;
        ftCo_HammerWait_Coll(g: G) = |g: G| { unsafe { crate::tu::ftCo_HammerWait::ftCo_HammerWait_Coll(g as _); } };
    }
}

/// `HSD_JObjSetRotationY` (jobj.h inline; the asserts are debug-only).
fn hsd_jobj_set_rotation_y(j: *mut HSD_JObj, y: f32) {
    unsafe {
        (*j).rotate.y = y;
        if (*j).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
            let fl = (*j).flags;
            let dirty = fl & JOBJ_USER_DEF_MTX as u32 == 0 && fl & JOBJ_MTX_DIRTY as u32 != 0;
            if !dirty {
                ext::HSD_JObjSetMtxDirtySub(j);
            }
        }
    }
}

pub fn ftCo_800C576C(g: G) -> bool {
    melee_core::ct!("ftCo_800C576C");
    if ext::ftCo_800C97A8(g) {
        ftCo_800C57B4(g);
        return true;
    }
    false
}

pub fn ftCo_800C57B4(g: G) {
    melee_core::ct!("ftCo_800C57B4");
    let flags = ext::ftCo_800C54C4(fp(g));
    let c = ext::ftCo_800C548C(fp(g));
    ext::ftCo_Turn_Enter(g, ftCo_MS_HammerTurn as i32, flags, 0.0, cd().x6B0 as f32, c);
    ext::ftCo_800C4E94(fp(g));
}

pub fn ftCo_HammerTurn_Anim(g: G) {
    melee_core::ct!("ftCo_HammerTurn_Anim");
    ext::ftCo_800C4F64(g);
    unsafe {
        if fp(g).mv.co.walk.fast_anim_frame != 0.0 {
            let part = ext::ftParts_GetBoneIndex(fp(g), FtPart_TransN);
            let jobj = (*fp(g).parts.add(part as usize)).x4_jobj2;
            let f = fp(g);
            f.mv.co.walk.fast_anim_frame -= 1.0;
            if f.x2222_b6() == 0 {
                let n = cd().x6B0 as f32;
                let rot_y = (core::f64::consts::PI * ((n - f.mv.co.walk.fast_anim_frame) / n) as f64) as f32;
                ext::ftPartSetRotY(fp(g), part as i32, rot_y);
                hsd_jobj_set_rotation_y(jobj, rot_y);
            }
            let f = fp(g);
            if f.mv.co.walk.fast_anim_frame <= 0.0 {
                f.facing_dir = -f.facing_dir;
                ext::ftCo_800C4ED8(g);
            }
        }
    }
}

pub fn ftCo_HammerTurn_IASA(g: G) {
    melee_core::ct!("ftCo_HammerTurn_IASA");
    if ext::ftCo_800C5A50(g) {
        return;
    }
}

pub fn ftCo_HammerTurn_Phys(g: G) {
    melee_core::ct!("ftCo_HammerTurn_Phys");
    ext::ft_80084F3C(g);
}

pub fn ftCo_HammerTurn_Coll(g: G) {
    melee_core::ct!("ftCo_HammerTurn_Coll");
    ext::ftCo_HammerWait_Coll(g);
}
