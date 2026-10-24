//! src/melee/ft/kinds/ftMario/ftmariospecialn.c: Fireball / Megavitamin (neutral B). No fused
//! sites (tools/rust-port/fma_sites.py). The Megavitamin colour pick is checked against the C
//! by tests/tu_ftmariospecialn.rs (both sides draw from the same recorded RNG); the rest is
//! smoke-tested through recording hooks.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use melee_fighter::hooks as fh;
use melee_types::*;

/// Already ported (rounds 2, 3, 5, 7): re-exported under this TU.
pub use crate::chars::r2::ftMario::ftmariospecialn::{
    ftMr_SpecialAirN_Anim, ftMr_SpecialAirN_Phys, ftMr_SpecialN_Anim, ftMr_SpecialN_Phys,
};
pub use crate::chars::r3::ftMario::ftmariospecialn::{
    ftMr_SpecialAirN_AirToGround, ftMr_SpecialAirN_Coll, ftMr_SpecialN_Coll, ftMr_SpecialN_GroundToAir,
};
pub use crate::chars::r5::ftMario::ftmariospecialn::ftMr_SpecialN_IASA;
pub use crate::chars::r7::ftMario::ftmariospecialn::{ftMr_SpecialAirN_Enter, ftMr_SpecialN_Enter};

/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {
    use core::ffi::c_void;
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
    // `lb_8000B1CC(joint, offset, out)` (lb/lb_00B0.c); the C passes a NULL offset here.
    pub static mut LB_8000B1CC: Option<fn(*mut HSD_JObj, *const Vec3, *mut Vec3) > = None;
            #[allow(non_snake_case)]
            pub fn lb_8000B1CC(joint: *mut HSD_JObj, off: *const Vec3, out: *mut Vec3)  {
                match unsafe { LB_8000B1CC } {
                    Some(f) => f(joint, off, out),
                    None => { unsafe { let _ = melee_runtime::tu::lb_00B0::lb_8000B1CC(joint as _, off as _, out as _); } return; },
                }
            }
    pub static mut IT_8029B6F8: Option<fn(*mut HSD_GObj, *mut Vec3, i32, f32) > = None;
            #[allow(non_snake_case)]
            pub fn it_8029B6F8(gobj: *mut HSD_GObj, pos: *mut Vec3, kind: i32, facing: f32)  {
                match unsafe { IT_8029B6F8 } {
                    Some(f) => f(gobj, pos, kind, facing),
                    None => { unsafe { let _ = melee_items::tu::itmariofireball::it_8029B6F8(gobj as _, pos as _, kind as _, facing as _); } return; },
                }
            }
    // `efSync_Spawn(gfx_id, gobj, ...)` (ef/efsync.c, variadic): the trailing arguments as a slice.
    hook!(EFSYNC_SPAWN, efSync_Spawn, (gfx_id: i32, gobj: *mut HSD_GObj, args: &[*mut c_void]) -> *mut c_void);
    hook!(ITDRMARIOPILL_SPAWN, itDrMarioPill_Spawn, (gobj: *mut HSD_GObj, pos: *mut Vec3, color: i32, kind: i32, facing: f32));
    // TARGET_PC `Geno_DefineBaseKind(kind)` (pc/geno).
    hook_or!(GENO_DEFINE_BASE_KIND, Geno_DefineBaseKind, melee_fighter::hooks::geno_define_base_kind, (kind: i32) -> i32);
    // `HSD_Randi(n)` (baselib/random.c).
    hook_or!(HSD_RANDI, HSD_Randi, melee_fighter::hooks::hsd_randi, (n: i32) -> i32);
    hook_or!(FTCO_FALL_IASA_INNER, ftCo_Fall_IASA_Inner, ftco_fall_iasa_inner_port, (gobj: *mut HSD_GObj));
    fn ftco_fall_iasa_inner_port(gobj: *mut HSD_GObj) {
        melee_fighter::tu::ftCo_Fall::ftCo_Fall_IASA_Inner(gobj);
    }
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

/// `ftMr_SpecialN_ItemFireSpawn` as a Fighter callback entry (`accessory4_cb`).
pub unsafe extern "C" fn cb_ftMr_SpecialN_ItemFireSpawn(g: *mut HSD_GObj) {
    ftMr_SpecialN_ItemFireSpawn(g);
}

/// `pickMegavitamin` (static): `arr[HSD_Randi(outpos)]`, shifting the current colour to previous.
pub fn pickMegavitamin(fp: &mut Fighter, arr: &[i32], outpos: i32) -> i32 {
    let result = arr[ext::HSD_Randi(outpos) as usize];
    unsafe {
        fp.u.mr.x2230_vitaminPrev = fp.u.mr.x222C_vitaminCurr;
        fp.u.mr.x222C_vitaminCurr = result;
    }
    result
}

/// Get random Megavitamin color combo for Dr. Mario.
pub fn ftMr_SpecialN_VitaminRandom(gobj: *mut HSD_GObj) -> i32 {
    unsafe {
        let fp = &mut *fpp(gobj);
        let mut arr = [0i32; 9];
        let mut r3: i32 = 0;
        for i in 0..9 {
            if i != fp.u.mr.x222C_vitaminCurr && i != fp.u.mr.x2230_vitaminPrev {
                arr[r3 as usize] = i;
                r3 += 1;
            }
        }
        pickMegavitamin(fp, &arr, r3)
    }
}

pub fn ftMr_SpecialN_ItemFireSpawn(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let flag_res = if (*fp).anon_2210.anon_0.throw_flags_b0() != 0 {
            (*fp).anon_2210.anon_0.set_throw_flags_b0(0);
            true
        } else {
            false
        };
        if flag_res {
            let mut coords = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
            let bone = fh::ftparts_get_bone_index(&*fp, FtPart_L1stNb as i32);
            ext::lb_8000B1CC((*(*fp).parts.offset(bone as isize)).joint, core::ptr::null(), &mut coords);
            if (*fp).kind == Ft_Kind_Mario as _ || ext::Geno_DefineBaseKind((*fp).kind as i32) == Ft_Kind_Mario as i32 {
                ext::it_8029B6F8(gobj, &mut coords, It_Kind_Mario_Fire, (*fp).facing_dir);
                let bone = fh::ftparts_get_bone_index(&*fp, FtPart_L1stNb as i32);
                let joint = (*(*fp).parts.offset(bone as isize)).joint;
                ext::efSync_Spawn(
                    1146,
                    gobj,
                    &[joint as *mut core::ffi::c_void, &mut (*fp).facing_dir as *mut f32 as *mut core::ffi::c_void],
                );
            } else {
                let rand_val_800e0d1c = ftMr_SpecialN_VitaminRandom(gobj);
                ext::itDrMarioPill_Spawn(gobj, &mut coords, rand_val_800e0d1c, It_Kind_DrMario_Vitamin, (*fp).facing_dir);
            }
        }
    }
}

pub fn ftMr_SpecialAirN_IASA(gobj: *mut HSD_GObj) {
    unsafe {
        if (*fpp(gobj)).cmd_vars[0] != 0 {
            ext::ftCo_Fall_IASA_Inner(gobj);
        }
    }
}
