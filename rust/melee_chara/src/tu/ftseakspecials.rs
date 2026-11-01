//! src/melee/ft/kinds/ftSeak/ftseakspecials.c: Sheik's Chain (side B): stick-driven aim, the chain
//! item and its four hitboxes, and the start / loop / end motions. Fused sites
//! (tools/rust-port/fma_sites.py, IR checked): `ftSk_SpecialS_80110490` 3 (`v5 * x44C + x18`,
//! `lx*lx + ly*ly` as `fma(lx, lx, ly*ly)`, `x44C * (v8 - x14) + x14`) and `ftSk_SpecialS_80110610`
//! 1 (`0.0556F * x18 + 4`); each is `fmadds`. The `sqrtf` inside 80110490 is the game's
//! `math_ppc.h` one (melee_runtime::gekko::sqrtf); `atan2f` is melee_runtime::lbtrigf. Not yet tested: no C oracle and no smoke test (unverified).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_fighter::ftcommon as co;
#[allow(unused_imports)]
use melee_types::Fighter;
#[allow(unused_imports)]
use melee_types::{HSD_GObj};
use crate::stubs as st;
use melee_core::fp::fmadds;
use melee_runtime::gekko::sqrtf;
use melee_runtime::lbtrigf::atan2f;
use melee_types::*;

/// Already ported (rounds 2/12/14 and the hand module): re-exported under this TU. The hand
/// module's `ZeroHitboxPositions`, `80110F58`, `80110F64` and `AirSStart_Phys` take the fighter
/// (`&mut Fighter` / `&Fighter`) where the C takes the gobj.





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
    /// Like `hook!`, but an unset hook runs the native port `$port` (a ported TU elsewhere).
    macro_rules! hook_or {
        ($hook:ident, $name:ident, ($($a:ident: $t:ty),*) $(-> $r:ty)?, $port:expr) => {
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
    hook_or!(FT_PLAYSFX, ft_PlaySFX, (fp: *mut Fighter, sfx: i32, vol: i32, pan: i32),
        |fp: *mut Fighter, sfx: i32, vol: i32, pan: i32| unsafe { melee_fighter::hooks::play_sfx(&mut *fp, sfx, vol, pan) });
    pub static mut LB_8000B1CC: Option<fn(*mut HSD_JObj, *const Vec3, *mut Vec3) > = None;
            #[allow(non_snake_case)]
            pub fn lb_8000B1CC(joint: *mut HSD_JObj, off: *const Vec3, out: *mut Vec3)  {
                match unsafe { LB_8000B1CC } {
                    Some(f) => f(joint, off, out),
                    None => { unsafe { melee_runtime::tu::lb_00B0::lb_8000B1CC(joint as _, off as _, out as _); } return; },
                }
            }
    hook_or!(FTDATA_80085E50, ftData_80085E50, (fp: *mut Fighter, msid: i32) -> *mut FigaTree,
        melee_fighter::tu::ftdata::ftData_80085E50);
    pub static mut FTANIM_8006F4C8: Option<fn(*mut Fighter, bool, *mut FigaTree)> = None;
    #[allow(non_snake_case)]
    pub fn ftAnim_8006F4C8(fp: *mut Fighter, arg1: bool, tree: *mut FigaTree) {
        match unsafe { FTANIM_8006F4C8 } {
            Some(f) => f(fp, arg1, tree),
            None => { unsafe { melee_fighter::tu::ftanim::ftAnim_8006F4C8(&mut *(fp as *mut _), arg1 as _, tree as _); } },
        }
    }
    pub static mut FTANIM_80070710: Option<fn(*mut HSD_JObj, f32) > = None;
            #[allow(non_snake_case)]
            pub fn ftAnim_80070710(jobj: *mut HSD_JObj, frame: f32)  {
                match unsafe { FTANIM_80070710 } {
                    Some(f) => f(jobj, frame),
                    None => { unsafe { melee_fighter::tu::ftanim::ftAnim_80070710(jobj as _, frame as _); } return; },
                }
            }
    pub static mut FTANIM_8006FB88: Option<fn(*mut Fighter, Fighter_Part, *mut HSD_Joint)> = None;
    #[allow(non_snake_case)]
    pub fn ftAnim_8006FB88(fp: *mut Fighter, part: Fighter_Part, joint: *mut HSD_Joint) {
        match unsafe { FTANIM_8006FB88 } {
            Some(f) => f(fp, part, joint),
            None => { unsafe { melee_fighter::tu::ftanim::ftAnim_8006FB88(&mut *(fp as *mut _), part as _, joint as _); } },
        }
    }
    crate::port_hook!(HSD_JOBJANIMALL, HSD_JObjAnimAll, crate::adapt::HSD_JObjAnimAll, (jobj: *mut HSD_JObj));
    pub static mut FTANIM_80070108: Option<fn(*mut Fighter, Fighter_Part, f32, f32, *mut HSD_Joint)> = None;
    #[allow(non_snake_case)]
    pub fn ftAnim_80070108(fp: *mut Fighter, part: Fighter_Part, a: f32, b: f32, joint: *mut HSD_Joint) {
        match unsafe { FTANIM_80070108 } {
            Some(f) => f(fp, part, a, b, joint),
            None => { unsafe { melee_fighter::tu::ftanim::ftAnim_80070108(&mut *(fp as *mut _), part as _, a as _, b as _, joint as _); } },
        }
    }
    pub static mut FTANIM_8006FE9C: Option<fn(*mut Fighter, Fighter_Part, f32, f32)> = None;
    #[allow(non_snake_case)]
    pub fn ftAnim_8006FE9C(fp: *mut Fighter, part: Fighter_Part, a: f32, b: f32) {
        match unsafe { FTANIM_8006FE9C } {
            Some(f) => f(fp, part, a, b),
            None => { unsafe { melee_fighter::tu::ftanim::ftAnim_8006FE9C(&mut *(fp as *mut _), part as _, a as _, b as _); } },
        }
    }
    pub static mut FTANIM_8006FF74: Option<fn(*mut Fighter, Fighter_Part)> = None;
    #[allow(non_snake_case)]
    pub fn ftAnim_8006FF74(fp: *mut Fighter, part: Fighter_Part) {
        match unsafe { FTANIM_8006FF74 } {
            Some(f) => f(fp, part),
            None => { unsafe { melee_fighter::tu::ftanim::ftAnim_8006FF74(&mut *(fp as *mut _), part as _); } },
        }
    }
    pub static mut FTANIM_80070010: Option<fn(*mut Fighter, Fighter_Part, f32, f32, *mut HSD_Joint)> = None;
    #[allow(non_snake_case)]
    pub fn ftAnim_80070010(fp: *mut Fighter, part: Fighter_Part, a: f32, b: f32, joint: *mut HSD_Joint) {
        match unsafe { FTANIM_80070010 } {
            Some(f) => f(fp, part, a, b, joint),
            None => { unsafe { melee_fighter::tu::ftanim::ftAnim_80070010(&mut *(fp as *mut _), part as _, a as _, b as _, joint as _); } },
        }
    }
    pub static mut FTANIM_8006FA58: Option<fn(*mut Fighter, Fighter_Part, *mut HSD_Joint)> = None;
    #[allow(non_snake_case)]
    pub fn ftAnim_8006FA58(fp: *mut Fighter, part: Fighter_Part, joint: *mut HSD_Joint) {
        match unsafe { FTANIM_8006FA58 } {
            Some(f) => f(fp, part, joint),
            None => { unsafe { melee_fighter::tu::ftanim::ftAnim_8006FA58(&mut *(fp as *mut _), part as _, joint as _); } },
        }
    }
    hook_or!(FTCOLL_8007B8A8, ftColl_8007B8A8, (hit: *mut HitCapsule, v: *mut Vec3),
        |hit: *mut HitCapsule, v: *mut Vec3| unsafe { melee_fighter::ft0df::ftcoll_8007b8a8(&mut *hit, &*v) });
    pub static mut LBCOLL_80008428: Option<fn(*mut HitCapsule)> = None;
    #[allow(non_snake_case)]
    pub fn lbColl_80008428(hit: *mut HitCapsule) {
        match unsafe { LBCOLL_80008428 } {
            Some(f) => f(hit),
            None => { unsafe { melee_runtime::tu::lbcollision::lbColl_80008428(hit as _); } },
        }
    }
    pub static mut LBCOLL_80008434: Option<fn(*mut HitCapsule)> = None;
    #[allow(non_snake_case)]
    pub fn lbColl_80008434(hit: *mut HitCapsule) {
        match unsafe { LBCOLL_80008434 } {
            Some(f) => f(hit),
            None => { unsafe { melee_runtime::tu::lbcollision::lbColl_80008434(hit as _); } },
        }
    }
    pub static mut LBCOLL_80008440: Option<fn(*mut HitCapsule)> = None;
    #[allow(non_snake_case)]
    pub fn lbColl_80008440(hit: *mut HitCapsule) {
        match unsafe { LBCOLL_80008440 } {
            Some(f) => f(hit),
            None => { unsafe { melee_runtime::tu::lbcollision::lbColl_80008440(hit as _); } },
        }
    }
    pub static mut FTNS_ATTACKHI4_YOYOCHECKENVCOLL: Option<fn(*mut HSD_GObj, *mut Vec3, *mut Vec3, f32) -> i32> = None;
    #[allow(non_snake_case)]
    pub fn ftNs_AttackHi4_YoyoCheckEnvColl(gobj: *mut HSD_GObj, ecb_unk: *mut Vec3, pos: *mut Vec3, arg: f32) -> i32 {
        match unsafe { FTNS_ATTACKHI4_YOYOCHECKENVCOLL } {
            Some(f) => f(gobj, ecb_unk, pos, arg),
            None => { unsafe { crate::tu::ftnessattackhi4::ftNs_AttackHi4_YoyoCheckEnvColl(gobj as _, &*(ecb_unk as *const _), &*(pos as *const _), arg as _) as _ } },
        }
    }
    // Items (it/kinds/itseakchain.c)
    pub static mut IT_802BB20C: Option<fn(*mut HSD_GObj)> = None;
    #[allow(non_snake_case)]
    pub fn it_802BB20C(item: *mut HSD_GObj) {
        match unsafe { IT_802BB20C } {
            Some(f) => f(item),
            None => { unsafe { melee_items::tu::itseakchain::it_802BB20C(item as _); } },
        }
    }
    pub static mut IT_802BAEEC: Option<fn(*mut HSD_GObj)> = None;
    #[allow(non_snake_case)]
    pub fn it_802BAEEC(item: *mut HSD_GObj) {
        match unsafe { IT_802BAEEC } {
            Some(f) => f(item),
            None => { unsafe { melee_items::tu::itseakchain::it_802BAEEC(item as _); } },
        }
    }
    pub static mut IT_802BAF0C: Option<fn(*mut HSD_GObj)> = None;
    #[allow(non_snake_case)]
    pub fn it_802BAF0C(item: *mut HSD_GObj) {
        match unsafe { IT_802BAF0C } {
            Some(f) => f(item),
            None => { unsafe { melee_items::tu::itseakchain::it_802BAF0C(item as _); } },
        }
    }
    pub static mut ITSEAKCHAIN_SPAWN: Option<fn(*mut HSD_GObj, *mut Vec3, f32) -> *mut HSD_GObj> = None;
    #[allow(non_snake_case)]
    pub fn itSeakChain_Spawn(owner: *mut HSD_GObj, pos: *mut Vec3, facing_dir: f32) -> *mut HSD_GObj {
        match unsafe { ITSEAKCHAIN_SPAWN } {
            Some(f) => f(owner, pos, facing_dir),
            None => { unsafe { melee_items::tu::itseakchain::itSeakChain_Spawn(owner as _, pos as _, facing_dir as _) as _ } },
        }
    }
    pub static mut IT_802BCFC4: Option<fn(*mut HSD_GObj, *mut Vec3)> = None;
    #[allow(non_snake_case)]
    pub fn it_802BCFC4(item: *mut HSD_GObj, vel: *mut Vec3) {
        match unsafe { IT_802BCFC4 } {
            Some(f) => f(item, vel),
            None => { unsafe { melee_items::tu::itseakchain::it_802BCFC4(item as _, vel as _); } },
        }
    }
    pub static mut IT_802BCF84: Option<fn(*mut HSD_GObj)> = None;
    #[allow(non_snake_case)]
    pub fn it_802BCF84(item: *mut HSD_GObj) {
        match unsafe { IT_802BCF84 } {
            Some(f) => f(item),
            None => { unsafe { melee_items::tu::itseakchain::it_802BCF84(item as _); } },
        }
    }
}

/// `transition_flags` (static const): SkipHit | SkipMatAnim | SkipColAnim | UpdateCmd | SkipItemVis
/// | Unk19 | SkipModelPartVis | SkipModelFlags | Unk27.
pub const transition_flags: u32 = Ft_MF_SkipHit
    | Ft_MF_SkipMatAnim
    | Ft_MF_SkipColAnim
    | Ft_MF_UpdateCmd
    | Ft_MF_SkipItemVis
    | Ft_MF_Unk19
    | Ft_MF_SkipModelPartVis
    | Ft_MF_SkipModelFlags
    | Ft_MF_Unk27;

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}
#[inline(always)]
unsafe fn da(fp: *mut Fighter) -> *const ftSeakAttributes {
    (*fp).dat_attrs as *const ftSeakAttributes
}

/// `COMMON_DATA_F32[275]` (`p_ftCommonData` as `float*`): the common-data float at offset 0x44C.
#[inline]
fn common_f32_275() -> f32 {
    melee_fighter::hooks::common_data().x44C
}

/// `(float) M_TAU`.
const TAU_F32: f32 = f32::from_bits(0x40C90FDB);
/// `MTXRadToDeg(a)` is `(a) * 57.29577951f`.
const RAD_TO_DEG: f32 = f32::from_bits(0x42652EE1);

/// The take-damage / death2 callback pair every Chain state installs.
unsafe fn set_chain_damage_callbacks(fp: *mut Fighter) {
    (*fp).death2_cb = Some(crate::tu::ftseak::cb_ftSk_Init_80110198);
    (*fp).take_dmg_cb = Some(crate::tu::ftseak::cb_ftSk_Init_80110198);
}

/// The accessory / hitlag callbacks every Chain state installs.
unsafe fn set_chain_callbacks(fp: *mut Fighter) {
    (*fp).accessory4_cb = Some(cb_ftSk_SpecialS_8011097C);
    (*fp).pre_hitlag_cb = Some(cb_ftSk_SpecialS_80110EE8);
    (*fp).post_hitlag_cb = Some(cb_ftSk_SpecialS_ChainSomething);
}

pub unsafe extern "C" fn cb_ftSk_SpecialS_8011097C(g: *mut HSD_GObj) {
    melee_core::ct!("cb_ftSk_SpecialS_8011097C");
    ftSk_SpecialS_8011097C(g);
}
pub unsafe extern "C" fn cb_ftSk_SpecialS_80110EE8(g: *mut HSD_GObj) {
    melee_core::ct!("cb_ftSk_SpecialS_80110EE8");
    ftSk_SpecialS_80110EE8(g);
}
pub unsafe extern "C" fn cb_ftSk_SpecialS_ChainSomething(g: *mut HSD_GObj) {
    melee_core::ct!("cb_ftSk_SpecialS_ChainSomething");
    ftSk_SpecialS_ChainSomething(g);
}

/// Stick angle (degrees, smoothed) and magnitude tracking for the chain's aim.
pub fn ftSk_SpecialS_80110490(fp: *mut Fighter) {
    melee_core::ct!("ftSk_SpecialS_80110490");
    unsafe {
        let lx = (*fp).input.lstick[0].x;
        let ly = (*fp).input.lstick[0].y;

        let mut v2 = atan2f(ly, lx * (*fp).facing_dir);

        if v2 < 0.0 {
            v2 += TAU_F32;
        }

        let mut v3 = v2 * RAD_TO_DEG;

        if v3 < 0.0 {
            v3 = 0.0;
        }

        if v3 > 359.0 {
            v3 = 359.0;
        }

        let mut v5 = v3 - (*fp).mv.sk.specials.x18;

        if v5 > 180.0 {
            v5 -= 360.0;
        } else if v5 < -180.0 {
            v5 += 360.0;
        }

        let mut v6 = fmadds(v5, common_f32_275(), (*fp).mv.sk.specials.x18);

        if v6 > 360.0 {
            v6 -= 360.0;
        } else if v6 < 0.0 {
            v6 += 360.0;
        }

        (*fp).mv.sk.specials.x18 = v6;

        // sqrtf(x*x + y*y): the sum is fma(lx, lx, ly*ly).
        let mut v8 = sqrtf(fmadds(lx, lx, ly * ly));

        if v8 > 1.0 {
            v8 = 1.0;
        }

        let x14 = (*fp).mv.sk.specials.x14;
        (*fp).mv.sk.specials.x14 = fmadds(common_f32_275(), v8 - x14, x14);
    }
}

/// Blend the chain's aim animation (`arg1` = 305 ground / else air) over the base pose.
pub fn ftSk_SpecialS_80110610(gobj: *mut HSD_GObj, arg1: i32, arg2: f32) {
    melee_core::ct!("ftSk_SpecialS_80110610");
    unsafe {
        let fp = fpp(gobj);
        let items = (*(*fp).ft_data).x48_items;

        let item: *mut *mut HSD_Joint = if arg1 == 305 {
            *items.add(4) as *mut *mut HSD_Joint
        } else {
            *items.add(5) as *mut *mut HSD_Joint
        };

        ftSk_SpecialS_80110490(fp);

        {
            let f = fmadds(0.0556_f32, (*fp).mv.sk.specials.x18, 4.0);

            if (*fp).mv.sk.specials.x14 != 0.0 {
                let bone = (*fp).x8AC_animSkeleton;
                ext::ftAnim_8006F4C8(fp, true, ext::ftData_80085E50(fp, arg1));
                ext::ftAnim_80070710(bone, f);
                ext::ftAnim_8006FB88(fp, FtPart_TransN, (*(*fp).x108_costume_joint).child);
                ext::HSD_JObjAnimAll(bone);

                if (*fp).mv.sk.specials.x14 < 1.0 {
                    ext::ftAnim_80070108(
                        fp,
                        FtPart_TransN,
                        1.0 - (*fp).mv.sk.specials.x14,
                        (*fp).mv.sk.specials.x14,
                        *item.add(2),
                    );
                }

                if arg2 < 1.0 {
                    ext::ftAnim_8006FE9C(fp, FtPart_TransN, arg2, 1.0 - arg2);
                    return;
                }

                ext::ftAnim_8006FF74(fp, FtPart_TransN);
                return;
            }
        }

        if arg2 < 1.0 {
            ext::ftAnim_80070010(fp, FtPart_TransN, arg2, 1.0 - arg2, *item.add(2));
            return;
        }

        ext::ftAnim_8006FA58(fp, FtPart_TransN, *item.add(2));
    }
}

/// Stick-flick SFX (the chain's swish) and the stick delta used to wiggle the chain.
pub fn ftSk_SpecialS_80110788(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_SpecialS_80110788");
    unsafe {
        let fp = gobj_fighter(gobj);
        (*fp).u.sk.lstick_delta.x = (*fp).input.lstick[0].x - (*fp).input.lstick[1].x;
        (*fp).u.sk.lstick_delta.y = (*fp).input.lstick[0].y - (*fp).input.lstick[1].y;

        {
            let state_var3 = (*fp).mv.sk.specials.x8;

            if state_var3 > 0 {
                (*fp).mv.sk.specials.x8 = state_var3 - 1;
            } else {
                const FLAGS: i32 = (1 << 3) | (1 << 6) | (1 << 8) | (1 << 9) | (1 << 10) | (1 << 11) | (1 << 12) | (1 << 18);

                if ((*fp).facing_dir == 1.0 && (*fp).u.sk.lstick_delta.x > 0.3_f32)
                    || ((*fp).facing_dir == -1.0 && (*fp).u.sk.lstick_delta.x < -0.3_f32)
                {
                    ext::ft_PlaySFX(fp, FLAGS, 127, 64);
                    (*fp).mv.sk.specials.x8 = 6;
                } else if (*fp).u.sk.lstick_delta.y > 0.5_f32 {
                    ext::ft_PlaySFX(fp, FLAGS, 127, 64);
                    (*fp).mv.sk.specials.x8 = 12;
                }
            }
        }

        {
            let state_var4 = (*fp).mv.sk.specials.xC;
            if state_var4 > 0 {
                (*fp).mv.sk.specials.xC = state_var4 - 1;
            } else {
                const FLAGS: i32 = (1 << 0)
                    | (1 << 1)
                    | (1 << 3)
                    | (1 << 6)
                    | (1 << 8)
                    | (1 << 9)
                    | (1 << 10)
                    | (1 << 11)
                    | (1 << 12)
                    | (1 << 18);

                if ((*fp).facing_dir == 1.0 && (*fp).u.sk.lstick_delta.x < -0.3_f32 && (*fp).input.lstick[0].x < 0.0)
                    || ((*fp).facing_dir == -1.0 && (*fp).u.sk.lstick_delta.x > 0.3_f32 && (*fp).input.lstick[0].x > 0.0)
                {
                    ext::ft_PlaySFX(fp, FLAGS, 127, 64);
                    (*fp).mv.sk.specials.xC = 6;
                }
            }
        }

        {
            let item_gobj = (*fp).u.sk.x8;

            if item_gobj.is_null() {
                return;
            }

            {
                let mut left_stick_x = (*fp).input.lstick[0].x;
                let item_data = (*item_gobj).user_data as *mut Item;
                let article = (*item_data).xC4_article_data;
                let chain_segment = (*article).x4_specialAttributes as *mut itChainSegment;

                if left_stick_x < 0.0 {
                    left_stick_x = -left_stick_x;
                }

                if left_stick_x < (*chain_segment).x48 {
                    let mut left_stick_y = (*fp).input.lstick[0].y;

                    if (*fp).input.lstick[0].y < 0.0 {
                        left_stick_y = -left_stick_y;
                    }

                    if left_stick_y < (*chain_segment).x48 {
                        let mul = 0.5_f32;
                        (*fp).u.sk.lstick_delta.x *= mul;
                        (*fp).u.sk.lstick_delta.y *= mul;
                    }
                }
            }
        }
    }
}

#[inline(always)]
unsafe fn gobj_fighter(gobj: *mut HSD_GObj) -> *mut Fighter {
    fpp(gobj)
}

/// Set hitbox `hitbox_id` of the chain to `new_pos` (the chain item calls this each frame).
pub fn ftSk_SpecialS_UpdateHitboxes(gobj: *mut HSD_GObj, new_pos: *mut Vec3, hitbox_id: i32) {
    melee_core::ct!("ftSk_SpecialS_UpdateHitboxes");
    unsafe {
        if gobj.is_null() {
            return;
        }

        let fp = fpp(gobj);

        if (*fp).cmd_vars[0] == 0 {
            return;
        }

        (*fp).u.sk.xC[hitbox_id as usize] = *new_pos;

        if (*new_pos).x != 0.0 || (*new_pos).y != 0.0 {
            let hit = &mut (*fp).x914[hitbox_id as usize] as *mut HitCapsule;
            ext::ftColl_8007B8A8(hit, new_pos);

            (*hit).x4C = *new_pos;

            if (*hit).x58.x == 0.0 && (*hit).x58.y == 0.0 && (*hit).x58.z == 0.0 {
                (*hit).x58 = *new_pos;
            }
        }
    }
}

pub fn ftSk_SpecialS_80110AEC(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_SpecialS_80110AEC");
    unsafe {
        let fp = fpp(gobj);

        for i in 0..4 {
            ext::lbColl_80008434(&mut (*fp).x914[i]);
        }

        // The C repeats the zeroing inline here (a "todo: should be ZeroHitboxPositions").
        ftSk_SpecialS_ZeroHitboxPositions(&mut *fp);

        (*fp).set_x2219_b3(1);
    }
}

/// `ftSeakSpecialS_LoopChainHitCollisions` (static inline).
fn ftSeakSpecialS_LoopChainHitCollisions(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);

        for i in 0..4 {
            ext::lbColl_80008440(&mut (*fp).x914[i]);
            ext::lbColl_80008428(&mut (*fp).x914[i]);
        }

        ftSk_SpecialS_ZeroHitboxPositions(&mut *fp);
    }
}

/// `ftSeakSpecialS_LoopChainHitActivate` (static inline).
fn ftSeakSpecialS_LoopChainHitActivate(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);

        for i in 0..4 {
            ext::lbColl_80008434(&mut (*fp).x914[i]);
        }

        ftSk_SpecialS_ZeroHitboxPositions(&mut *fp);
        (*fp).set_x2219_b3(1);
    }
}

/// `sumOfSquares` (static inline): separate statements, so no contraction.
#[inline(always)]
fn sumOfSquares(a: f32, b: f32) -> f32 {
    let a = a * a;
    let c = b * b;
    a + c
}

/// Per-frame chain hit tracking: re-arm the hitboxes while the chain tip moves fast enough.
pub fn ftSk_SpecialS_80110BCC(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_SpecialS_80110BCC");
    unsafe {
        let fp = fpp(gobj);
        let item_gobj = (*fp).u.sk.x8;
        let special_attributes = da(fp);

        if item_gobj.is_null() {
            return;
        }

        let item_data = (*item_gobj).user_data as *mut Item;
        let chain = (*(*item_data).xC4_article_data).x4_specialAttributes as *mut itChainSegment;

        let mut sums_of_squares = [0.0_f32; 4];
        for i in 0..4 {
            let x = (*fp).u.sk.xC[i].x - (*fp).u.sk.x3C[i].x;
            let y = (*fp).u.sk.xC[i].y - (*fp).u.sk.x3C[i].y;

            sums_of_squares[i] = sumOfSquares(x, y);

            (*fp).u.sk.x3C[i].x = (*fp).u.sk.xC[i].x;
            (*fp).u.sk.x3C[i].y = (*fp).u.sk.xC[i].y;
        }

        if (*fp).mv.sk.specials.x1C > 0 {
            (*fp).mv.sk.specials.x1C -= 1;

            if (*fp).mv.sk.specials.x1C == 0 {
                ftSeakSpecialS_LoopChainHitCollisions(gobj);
            }
        }

        let chain_val = (*chain).x4C;
        let chain_val_sq = chain_val * chain_val;

        if sums_of_squares[0] > chain_val_sq
            || sums_of_squares[1] > chain_val_sq
            || sums_of_squares[2] > chain_val_sq
            || sums_of_squares[3] > chain_val_sq
        {
            if (*fp).mv.sk.specials.x1C <= 0 {
                (*fp).mv.sk.specials.x1C = (*special_attributes).x18 as i32;
                ftSeakSpecialS_LoopChainHitActivate(gobj);
            }
        } else {
            let var = (*fp).mv.sk.specials.x20;
            if var > 0 {
                (*fp).mv.sk.specials.x20 -= 1;
            } else {
                (*fp).mv.sk.specials.x1C = 0;
                ftSeakSpecialS_LoopChainHitCollisions(gobj);
            }
        }
    }
}

pub fn ftSk_SpecialS_80110E4C(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_SpecialS_80110E4C");
    unsafe {
        let fp = fpp(gobj);

        ftSk_SpecialS_ChainSomething(gobj);

        (*fp).u.sk.x8 = core::ptr::null_mut();
        (*fp).death2_cb = None;
        (*fp).take_dmg_cb = None;
    }
}

pub fn ftSk_SpecialS_CheckAndDestroyChain(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_SpecialS_CheckAndDestroyChain");
    unsafe {
        let fp = fpp(gobj);

        if (*fp).u.sk.x8.is_null() {
            return;
        }

        ext::it_802BB20C((*fp).u.sk.x8);

        let fp = fpp(gobj);

        ftSk_SpecialS_ChainSomething(gobj);

        (*fp).u.sk.x8 = core::ptr::null_mut();
        (*fp).death2_cb = None;
        (*fp).take_dmg_cb = None;
    }
}

pub fn ftSk_SpecialS_80110EE8(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_SpecialS_80110EE8");
    unsafe {
        let fp = fpp(gobj);

        if !(*fp).u.sk.x8.is_null() {
            ext::it_802BAEEC((*fp).u.sk.x8);
        }
    }
}

pub fn ftSk_SpecialS_ChainSomething(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_SpecialS_ChainSomething");
    unsafe {
        let fp = fpp(gobj);

        if !(*fp).u.sk.x8.is_null() {
            ext::it_802BAF0C((*fp).u.sk.x8);
            (*fp).mv.sk.specials.x20 = 2;
        }
    }
}

pub fn ftSk_SpecialS_80110F70(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_SpecialS_80110F70");
    unsafe {
        let fp = fpp(gobj);
        (*fp).cmd_vars[0] = 0;

        (*fp).mv.sk.specials.x0 = 0;
        (*fp).mv.sk.specials.x4 = 0;
        (*fp).mv.sk.specials.x8 = 0;
        (*fp).mv.sk.specials.xC = 0;

        {
            let var: f32 = 0.0;

            (*fp).mv.sk.specials.x10 = var;
            (*fp).mv.sk.specials.x18 = 4.0;
            (*fp).mv.sk.specials.x14 = var;

            (*fp).mv.sk.specials.x1C = 0;
            (*fp).mv.sk.specials.x20 = 0;
            (*fp).u.sk.x8 = core::ptr::null_mut();

            for i in 0..4 {
                (*fp).u.sk.xC[i].z = var;
                (*fp).u.sk.xC[i].y = var;
                (*fp).u.sk.xC[i].x = var;

                (*fp).u.sk.x3C[i].z = var;
                (*fp).u.sk.x3C[i].y = var;
                (*fp).u.sk.x3C[i].x = var;
            }

            (*fp).u.sk.lstick_delta.z = var;
            (*fp).u.sk.lstick_delta.y = var;
            (*fp).u.sk.lstick_delta.x = var;
        }

        (*fp).set_x2222_b2(1);
        (*fp).accessory4_cb = Some(cb_ftSk_SpecialS_8011097C);
    }
}

pub fn ftSk_SpecialS_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_SpecialS_Enter");
    st::Fighter_ChangeMotionState(gobj, 349, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
    st::ftAnim_8006EBA4(gobj);
    ftSk_SpecialS_80110F70(gobj);
}

pub fn ftSk_SpecialAirS_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_SpecialAirS_Enter");
    unsafe {
        let fp = fpp(gobj);
        (*fp).self_vel.y = 0.0;
    }
    st::Fighter_ChangeMotionState(gobj, 352, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
    st::ftAnim_8006EBA4(gobj);
    ftSk_SpecialS_80110F70(gobj);
}

/// `ftSk_SpecialS_SpawnChain` (static inline): create the chain item at the left hand.
fn ftSk_SpecialS_SpawnChain(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

        ext::lb_8000B1CC((*(*fp).parts.add(FtPart_L3rdNa as usize)).joint, core::ptr::null(), &mut pos);
        (*fp).u.sk.x8 = ext::itSeakChain_Spawn(gobj, &mut pos, (*fp).facing_dir);
        (*fp).x1984_heldItemSpec = (*fp).u.sk.x8;

        if !(*fp).u.sk.x8.is_null() {
            set_chain_damage_callbacks(fp);
        }

        (*fp).pre_hitlag_cb = Some(cb_ftSk_SpecialS_80110EE8);
        (*fp).post_hitlag_cb = Some(cb_ftSk_SpecialS_ChainSomething);
    }
}

/// Advance the start timer, spawn the chain at frame `x1C`, launch it the frame after; true when
/// the start phase is over.
pub fn ftSk_SpecialS_CheckInitChain(gobj: *mut HSD_GObj) -> bool {
    melee_core::ct!("ftSk_SpecialS_CheckInitChain");
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);

        (*fp).mv.sk.specials.x0 += 1;

        if (*fp).mv.sk.specials.x0 as f32 == (*da).x1C {
            ftSk_SpecialS_SpawnChain(gobj);
            (*fp).mv.sk.specials.x1C = (*da).x18 as i32;

            if (*fp).u.sk.x8.is_null() {
                if (*fp).ground_or_air == GA_Air {
                    st::ftCo_Fall_Enter(gobj);
                } else {
                    st::ft_8008A2BC(gobj);
                }
            }
        }

        if (*fp).mv.sk.specials.x0 as f32 == (*da).x1C + 1.0 {
            let mut vel = Vec3 { x: 1.8_f32, y: 0.0, z: 0.0 };
            let item_gobj = (*fp).u.sk.x8;
            let ip = (*item_gobj).user_data as *mut Item;
            let segment = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itChainSegment;

            vel.x = (*segment).x50;
            vel.x *= (*ip).facing_dir;
            ext::it_802BCFC4(item_gobj, &mut vel);
        }

        (*fp).mv.sk.specials.x0 as f32 > (*da).x20
    }
}

pub fn ftSk_SpecialSStart_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_SpecialSStart_Anim");
    if ftSk_SpecialS_CheckInitChain(gobj) {
        ftSk_SpecialS_80111830(gobj);
    }
}

pub fn ftSk_SpecialAirSStart_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_SpecialAirSStart_Anim");
    if ftSk_SpecialS_CheckInitChain(gobj) {
        ftSk_SpecialS_80111988(gobj);
    }
}

/// The shared prologue of both S_Anim functions: true when the release latched in IASA has
/// passed the minimum duration.
unsafe fn s_anim_released(fp: *mut Fighter, special_attributes: *const ftSeakAttributes) -> bool {
    if (*fp).mv.sk.specials.x0 as f32 > (*special_attributes).x14 && (*fp).mv.sk.specials.x4 != 0 {
        (*fp).mv.sk.specials.x4 = 0;
        true
    } else {
        false
    }
}

pub fn ftSk_SpecialS_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_SpecialS_Anim");
    unsafe {
        let fp = fpp(gobj);
        let special_attributes = da(fp);

        ftSk_SpecialS_80110BCC(gobj);

        (*fp).mv.sk.specials.x0 += 1;

        if s_anim_released(fp, special_attributes) {
            ftSk_SpecialS_80111DF8(gobj);
            return;
        }

        ftSk_SpecialS_80110610(gobj, 305, 1.0);
    }
}

pub fn ftSk_SpecialAirS_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_SpecialAirS_Anim");
    unsafe {
        let fp = fpp(gobj);
        let special_attributes = da(fp);

        ftSk_SpecialS_80110BCC(gobj);
        (*fp).mv.sk.specials.x0 += 1;

        if s_anim_released(fp, special_attributes) {
            ftSk_SpecialS_80111EB4(gobj);
            return;
        }

        ftSk_SpecialS_80110610(gobj, 308, 1.0);
    }
}

pub fn ftSk_SpecialS_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_SpecialS_IASA");
    unsafe {
        let fp = fpp(gobj);

        if ((*fp).input.held_buttons[0] as i32 & HSD_PAD_B) == 0 {
            (*fp).mv.sk.specials.x4 = 1;
        }

        ftSk_SpecialS_80110788(gobj);
    }
}

pub fn ftSk_SpecialAirS_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_SpecialAirS_IASA");
    unsafe {
        let fp = fpp(gobj);

        if ((*fp).input.held_buttons[0] as i32 & HSD_PAD_B) == 0 {
            (*fp).mv.sk.specials.x4 = 1;
        }

        ftSk_SpecialS_80110788(gobj);
    }
}

pub fn ftSk_SpecialS_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_SpecialS_Coll");
    if st::ft_800827A0(gobj) == 0 {
        ftSk_SpecialS_80111DF8(gobj);
    }
}

pub fn ftSk_SpecialAirS_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_SpecialAirS_Coll");
    if st::ft_80081D0C(gobj) != 0 {
        ftSk_SpecialS_80111EB4(gobj);
    }
}

pub fn ftSk_SpecialS_80111830(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_SpecialS_80111830");
    unsafe {
        let fp = fpp(gobj);

        st::Fighter_ChangeMotionState(gobj, 350, 8, 0.0, 1.0, 0.0, core::ptr::null_mut());
        ftSk_SpecialS_80110610(gobj, 305, 0.0);

        let fp2 = fpp(gobj);
        (*fp2).mv.sk.specials.x0 = 0;

        ftSk_SpecialS_80110AEC(gobj);

        if !(*fp2).u.sk.x8.is_null() {
            set_chain_damage_callbacks(fp2);
        }

        (*fp2).set_x2222_b2(1);
        set_chain_callbacks(fp2);

        {
            let mut vec0 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
            let mut vec1 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
            vec0.x = 0.0;
            let ecb_top = (*fp).coll_data.ecb.top.y;
            let ecb_bot = (*fp).coll_data.ecb.bottom.y;
            vec0.y = 0.5_f32 * (ecb_top + ecb_bot);
            vec0.z = 0.0;
            vec0.x += (*fp).cur_pos.x;
            vec0.y += (*fp).cur_pos.y;
            vec0.z += (*fp).cur_pos.z;

            ext::lb_8000B1CC((*(*fp).parts.add(FtPart_L3rdNa as usize)).joint, core::ptr::null(), &mut vec1);

            let flags = ext::ftNs_AttackHi4_YoyoCheckEnvColl(gobj, &mut vec0, &mut vec1, 0.5_f32);

            if flags != 0 {
                ftSk_SpecialS_80111DF8(gobj);
            }
        }
    }
}

pub fn ftSk_SpecialS_80111988(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_SpecialS_80111988");
    unsafe {
        st::Fighter_ChangeMotionState(gobj, 353, 8, 0.0, 1.0, 0.0, core::ptr::null_mut());
        ftSk_SpecialS_80110610(gobj, 308, 0.0);

        let fp = fpp(gobj);
        (*fp).mv.sk.specials.x0 = 0;

        ftSk_SpecialS_80110AEC(gobj);

        if !(*fp).u.sk.x8.is_null() {
            set_chain_damage_callbacks(fp);
        }

        (*fp).set_x2222_b2(1);
        set_chain_callbacks(fp);
    }
}

/// The shared middle of both End_Anim functions: `x0` counts frames; at `x24` the chain
/// retracts, at `x28` it is destroyed; otherwise the hit tracking runs.
unsafe fn end_anim_step(gobj: *mut HSD_GObj, fp: *mut Fighter, special_attributes: *const ftSeakAttributes) {
    (*fp).mv.sk.specials.x0 += 1;

    let temp_r3 = (*fp).mv.sk.specials.x0;
    let temp_f1 = (*special_attributes).x28;

    if (temp_r3 as f32) < temp_f1 {
        let item_gobj = (*fp).u.sk.x8;

        if temp_r3 as f32 == (*special_attributes).x24 {
            ext::it_802BCF84(item_gobj);
        }

        ftSk_SpecialS_80110BCC(gobj);
    } else if temp_r3 as f32 == temp_f1 {
        let item_gobj = (*fp).u.sk.x8;
        ext::it_802BB20C(item_gobj);
    } else {
        ftSk_SpecialS_80110BCC(gobj);
    }
}

pub fn ftSk_SpecialSEnd_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_SpecialSEnd_Anim");
    unsafe {
        let fp = fpp(gobj);
        let special_attributes = da(fp);

        end_anim_step(gobj, fp, special_attributes);

        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::ft_8008A2BC(gobj);
        }
    }
}

pub fn ftSk_SpecialAirSEnd_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_SpecialAirSEnd_Anim");
    unsafe {
        let fp = fpp(gobj);
        let special_attributes = da(fp);

        end_anim_step(gobj, fp, special_attributes);

        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::ftCo_Fall_Enter(gobj);
        }
    }
}

pub fn ftSk_SpecialSEnd_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_SpecialSEnd_Coll");
    if st::ft_800827A0(gobj) == 0 {
        ftSk_SpecialS_80111CB0(gobj);
    }
}

pub fn ftSk_SpecialS_80111CB0(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_SpecialS_80111CB0");
    unsafe {
        let fp = fpp(gobj);
        st::ftCommon_GroundToAirStateChange(gobj, &mut *fp, 354, transition_flags);

        let fp2 = fpp(gobj);

        if !(*fp2).u.sk.x8.is_null() {
            set_chain_damage_callbacks(fp2);
        }

        set_chain_callbacks(fp2);
    }
}

pub fn ftSk_SpecialS_80111DF8(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_SpecialS_80111DF8");
    unsafe {
        st::Fighter_ChangeMotionState(gobj, 351, 8, 0.0, 1.0, 0.0, core::ptr::null_mut());

        let fp = fpp(gobj);
        (*fp).mv.sk.specials.x0 = 0;

        if (*fp).mv.sk.specials.x1C != 0 {
            ftSk_SpecialS_80110AEC(gobj);
        }

        if !(*fp).u.sk.x8.is_null() {
            set_chain_damage_callbacks(fp);
        }

        (*fp).set_x2222_b2(1);
        set_chain_callbacks(fp);
    }
}

pub fn ftSk_SpecialS_80111EB4(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_SpecialS_80111EB4");
    unsafe {
        st::Fighter_ChangeMotionState(gobj, 354, 8, 0.0, 1.0, 0.0, core::ptr::null_mut());

        let fp = fpp(gobj);
        (*fp).mv.sk.specials.x0 = 0;

        if (*fp).mv.sk.specials.x1C != 0 {
            ftSk_SpecialS_80110AEC(gobj);
        }

        if !(*fp).u.sk.x8.is_null() {
            set_chain_damage_callbacks(fp);
        }

        (*fp).set_x2222_b2(1);
        set_chain_callbacks(fp);
    }
}

/// True unless a needle is held (`u.sk.x4 != 0`); true for a NULL gobj.
pub fn ftSk_SpecialS_80111F70(gobj: *mut HSD_GObj) -> bool {
    melee_core::ct!("ftSk_SpecialS_80111F70");
    unsafe {
        if !gobj.is_null() {
            let fp = fpp(gobj);
            if !(*fp).u.sk.x4.is_null() {
                return false;
            }
            return true;
        }

        true
    }
}

/// The needle count (`u.sk.x0`); 0 for a NULL gobj.
pub fn ftSk_SpecialS_80111FA0(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("ftSk_SpecialS_80111FA0");
    unsafe {
        if !gobj.is_null() {
            return (*fpp(gobj)).u.sk.x0;
        }

        0
    }
}

/// `ftSk_SpecialAirSStart_Phys(gobj)`: the retail motion-state callback; the body is the fighter-only port `chars::ftSeak::ftseakspecials::ftSk_SpecialAirSStart_Phys_fp`.
pub fn ftSk_SpecialAirSStart_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_SpecialAirSStart_Phys");
    unsafe { crate::chars::ftSeak::ftseakspecials::ftSk_SpecialAirSStart_Phys_fp(&mut *((*gobj).user_data as *mut Fighter)) }
}

// ---- folded from chars (consolidation step 20/21) ----
pub fn ftSk_SpecialS_80110F58(fp: &Fighter) -> f32 {
    melee_core::ct!("ftSk_SpecialS_80110F58");
    fp.input.lstick[0].x
}

pub fn ftSk_SpecialS_80110F64(fp: &Fighter) -> f32 {
    melee_core::ct!("ftSk_SpecialS_80110F64");
    fp.input.lstick[0].y
}

/// Zero the position and last position of the 4 hitboxes.
pub fn ftSk_SpecialS_ZeroHitboxPositions(fp: &mut Fighter) {
    melee_core::ct!("ftSk_SpecialS_ZeroHitboxPositions");
    for hit in fp.x914.iter_mut() {
        hit.x58.x = 0.0;
        hit.x58.y = 0.0;
        hit.x58.z = 0.0;
        hit.x4C.x = 0.0;
        hit.x4C.y = 0.0;
        hit.x4C.z = 0.0;
    }
}

/// `ftSk_SpecialS_8011097C` (src/melee/ft/kinds/ftSeak/ftseakspecials.c).
pub fn ftSk_SpecialS_8011097C(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_SpecialS_8011097C");
    unsafe {
        return;
    }
}

/// `ftSk_SpecialSStart_IASA` (src/melee/ft/kinds/ftSeak/ftseakspecials.c).
pub fn ftSk_SpecialSStart_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_SpecialSStart_IASA");
    unsafe {
        return;
    }
}

/// `ftSk_SpecialAirSStart_IASA` (src/melee/ft/kinds/ftSeak/ftseakspecials.c).
pub fn ftSk_SpecialAirSStart_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_SpecialAirSStart_IASA");
    unsafe {
        return;
    }
}

/// `ftSk_SpecialSEnd_IASA` (src/melee/ft/kinds/ftSeak/ftseakspecials.c).
pub fn ftSk_SpecialSEnd_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_SpecialSEnd_IASA");
    unsafe {
        return;
    }
}

/// `ftSk_SpecialAirSEnd_IASA` (src/melee/ft/kinds/ftSeak/ftseakspecials.c).
pub fn ftSk_SpecialAirSEnd_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_SpecialAirSEnd_IASA");
    unsafe {
        return;
    }
}

/// `ftSk_SpecialSStart_Coll` (src/melee/ft/kinds/ftSeak/ftseakspecials.c).
pub fn ftSk_SpecialSStart_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_SpecialSStart_Coll");
    unsafe {
        if (st::ft_800827A0(gobj) == 0) {
            crate::chars::r14::ftSeak::ftseakspecials::ftSk_SpecialS_80111440(gobj);
        }
    }
}

/// `ftSk_SpecialAirSStart_Coll` (src/melee/ft/kinds/ftSeak/ftseakspecials.c).
pub fn ftSk_SpecialAirSStart_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_SpecialAirSStart_Coll");
    unsafe {
        if (st::ft_80081D0C(gobj) != 0) {
            crate::chars::r14::ftSeak::ftseakspecials::ftSk_SpecialS_801114E4(gobj);
        }
    }
}

/// `ftSk_SpecialS_80111440` (src/melee/ft/kinds/ftSeak/ftseakspecials.c).
pub fn ftSk_SpecialS_80111440(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_SpecialS_80111440");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::ftCommon_8007D5D4(&mut *fp);
        st::Fighter_ChangeMotionState(gobj, 352, 0xC4C5088u32 /* transition_flags */, ((*fp).cur_anim_frame as f32), 1.0_f32, 0.0_f32, core::ptr::null_mut());
        if (!(*fp).u.sk.x8.is_null()) {
            (*fp).death2_cb = Some(crate::chars::cb::ftseak__ftSk_Init_80110198);
            (*fp).take_dmg_cb = Some(crate::chars::cb::ftseak__ftSk_Init_80110198);
        }
        (*fp).accessory4_cb = Some(crate::chars::cb::ftseakspecials__ftSk_SpecialS_8011097C);
        (*fp).pre_hitlag_cb = Some(crate::chars::cb::ftseakspecials__ftSk_SpecialS_80110EE8);
        (*fp).post_hitlag_cb = Some(crate::chars::cb::ftseakspecials__ftSk_SpecialS_ChainSomething);
    }
}

/// `ftSk_SpecialS_801114E4` (src/melee/ft/kinds/ftSeak/ftseakspecials.c).
pub fn ftSk_SpecialS_801114E4(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_SpecialS_801114E4");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::ftCommon_AirToGroundStateChange(gobj, &mut *fp, 349, 0xC4C5088u32 /* transition_flags */);
        if (!(*fp).u.sk.x8.is_null()) {
            (*fp).death2_cb = Some(crate::chars::cb::ftseak__ftSk_Init_80110198);
            (*fp).take_dmg_cb = Some(crate::chars::cb::ftseak__ftSk_Init_80110198);
        }
        (*fp).accessory4_cb = Some(crate::chars::cb::ftseakspecials__ftSk_SpecialS_8011097C);
        (*fp).pre_hitlag_cb = Some(crate::chars::cb::ftseakspecials__ftSk_SpecialS_80110EE8);
        (*fp).post_hitlag_cb = Some(crate::chars::cb::ftseakspecials__ftSk_SpecialS_ChainSomething);
    }
}

/// `ftSk_SpecialAirSEnd_Coll` (src/melee/ft/kinds/ftSeak/ftseakspecials.c).
pub fn ftSk_SpecialAirSEnd_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_SpecialAirSEnd_Coll");
    unsafe {
        if (st::ft_80081D0C(gobj) != 0) {
            crate::chars::r14::ftSeak::ftseakspecials::ftSk_SpecialS_80111D54(gobj);
        }
    }
}

/// `ftSk_SpecialS_80111D54` (src/melee/ft/kinds/ftSeak/ftseakspecials.c).
pub fn ftSk_SpecialS_80111D54(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_SpecialS_80111D54");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::ftCommon_AirToGroundStateChange(gobj, &mut *fp, 351, 0xC4C5088u32 /* transition_flags */);
        if (!(*fp).u.sk.x8.is_null()) {
            (*fp).death2_cb = Some(crate::chars::cb::ftseak__ftSk_Init_80110198);
            (*fp).take_dmg_cb = Some(crate::chars::cb::ftseak__ftSk_Init_80110198);
        }
        (*fp).accessory4_cb = Some(crate::chars::cb::ftseakspecials__ftSk_SpecialS_8011097C);
        (*fp).pre_hitlag_cb = Some(crate::chars::cb::ftseakspecials__ftSk_SpecialS_80110EE8);
        (*fp).post_hitlag_cb = Some(crate::chars::cb::ftseakspecials__ftSk_SpecialS_ChainSomething);
    }
}

/// `ftSk_SpecialSStart_Phys` (src/melee/ft/kinds/ftSeak/ftseakspecials.c).
pub fn ftSk_SpecialSStart_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_SpecialSStart_Phys");
    st::ft_80084F3C(gobj);
}

/// `ftSk_SpecialS_Phys` (src/melee/ft/kinds/ftSeak/ftseakspecials.c).
pub fn ftSk_SpecialS_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_SpecialS_Phys");
    st::ft_80084F3C(gobj);
}

/// `ftSk_SpecialAirS_Phys` (src/melee/ft/kinds/ftSeak/ftseakspecials.c).
pub fn ftSk_SpecialAirS_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_SpecialAirS_Phys");
    st::ft_80084EEC(gobj);
}

/// `ftSk_SpecialSEnd_Phys` (src/melee/ft/kinds/ftSeak/ftseakspecials.c).
pub fn ftSk_SpecialSEnd_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_SpecialSEnd_Phys");
    st::ft_80084F3C(gobj);
}

/// `ftSk_SpecialAirSEnd_Phys` (src/melee/ft/kinds/ftSeak/ftseakspecials.c).
pub fn ftSk_SpecialAirSEnd_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_SpecialAirSEnd_Phys");
    st::ft_80084EEC(gobj);
}

// ---- folded from chars blocks (consolidation step 20) ----
pub fn ftSk_SpecialAirSStart_Phys_fp(fp: &mut Fighter) {
    melee_core::ct!("ftSk_SpecialAirSStart_Phys_fp");
    if fp.cmd_vars[0] != 0 {
        co::fall(fp, fp.co_attrs.gravity, fp.co_attrs.terminal_velocity);
    }
    let f = fp.co_attrs.aerial_friction;
    co::calc_self_accel_deaccel(fp, f);
}
