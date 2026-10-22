//! src/melee/ft/kinds/ftMasterHand/ftmasterhandtagcancel.c. No float math, no fused sites.
//! Smoke-tested by tests/tu_ftmasterhandtagcancel.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports)]

use crate::stubs as st;
use melee_types::*;

/// Calls into TUs outside this file. Each is a settable hook; unset it falls back to the
/// native port in `melee_fighter` where one exists.
pub mod ext {
    use melee_types::*;
    macro_rules! hook_or {
        ($hook:ident, $name:ident, $port:path, ($($a:ident: $t:ty),*) $(-> $r:ty)?) => {
            pub static mut $hook: Option<fn($($t),*) $(-> $r)?> = None;
            #[allow(non_snake_case)]
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $hook } {
                    Some(f) => f($($a),*),
                    None => $port($($a),*),
                }
            }
        };
    }
    hook_or!(FTCOMMON_8007DB58, ftCommon_8007DB58, melee_fighter::ftcommon3::ftcommon_8007db58, (gobj: *mut HSD_GObj));
    hook_or!(FTCO_8009750C, ftCo_8009750C, melee_fighter::tu::ftCo_Lift::ftCo_8009750C, (gobj: *mut HSD_GObj));
    hook_or!(FTCO_800DD168, ftCo_800DD168, melee_fighter::tu::ftCo_Throw::ftCo_800DD168, (gobj: *mut HSD_GObj));
    hook_or!(FTCOMMON_INITGRAB, ftCommon_InitGrab, melee_fighter::ftcommon2::init_grab, (fp: &mut Fighter, arg1: i32, timer: f32));
    hook_or!(FTCO_800DA824, ftCo_800DA824, melee_fighter::tu::ftCo_CapturePulled::ftCo_800DA824, (fp: &mut Fighter) -> f32);
    hook_or!(FTCO_800DB368, ftCo_800DB368, melee_fighter::tu::ftCo_CapturePulled::ftCo_800DB368, (att: &mut Fighter, fp: &mut Fighter));
}

pub fn ftMh_MS_388_80155A58(gobj_1: *mut HSD_GObj, gobj_2: *mut HSD_GObj) {
    unsafe {
        let ft_1 = (*gobj_1).user_data as *mut Fighter;
        let ft_2 = (*gobj_2).user_data as *mut Fighter;
        ext::ftCommon_8007DB58(gobj_1);
        ext::ftCo_8009750C(gobj_1);
        ext::ftCo_800DD168(gobj_1);
        (*ft_1).x1A5C = gobj_2;
        (*ft_1).victim_gobj = gobj_2;
        (*ft_1).anon_221b.set_x221B_b5(0);
        (*ft_1).anon_221b.set_x221B_b7(0);
        (*ft_1).facing_dir = (*ft_2).facing_dir;
        let t = ext::ftCo_800DA824(&mut *ft_2);
        ext::ftCommon_InitGrab(&mut *ft_1, 0, t);
        ext::ftCo_800DB368(&mut *ft_2, &mut *ft_1);
        (*ft_1).accessory1_cb = Some(melee_fighter::tu::ftCo_CapturePulled::ftCo_800DB464_cb);
        st::ftCommon_8007D5D4(&mut *ft_1);
        st::Fighter_ChangeMotionState(gobj_1, ftCo_MS_CaptureMasterHand as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        (*ft_1).set_invisible(1);
        (*ft_1).set_x2220_b3(1);
        st::ftCommon_8007E2F4(&mut *ft_1, 511);
        st::ftAnim_8006EBA4(gobj_1);
        st::ftCommon_8007E2FC(gobj_1);
    }
}
