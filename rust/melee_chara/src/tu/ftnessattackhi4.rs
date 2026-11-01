//! src/melee/ft/kinds/ftNess/ftnessattackhi4.c: Ness's Up Smash and the Yo-Yo helpers shared with
//! his Down Smash (ftnessattacklw4.c).
//!
//! Fused sites (tools/rust-port/fma_sites.py, all f32 -> [`fmadds`]): the three
//! `(a * w) + (b * (1 - w))` lerps of `YoyoSetHitPosUnk` / `AttackHi4Release_Phys`
//! (`fmuladd(a, w, b * (1 - w))`), `sp14.y += -1.0f * scale.y` in `YoyoCheckNoObstruct` and
//! `AttackHi4_Anim`, and the two-step `(1/256 * mul - 1) * ratio + 1` damage scale in
//! `YoyoSetChargeDamage` / `YoyoApplyDamage`.
//! Pure parts and the callbacks are checked by tests/tu_ftnessattackhi4.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::chars::da;
#[allow(unused_imports)]
use melee_types::{ftNessAttributes, Fighter};
#[allow(unused_imports)]
use melee_types::{HSD_GObj};
use crate::stubs as st;
use melee_core::fp::fmadds;
use melee_runtime::lbtrigf::atan2f;
use melee_runtime::lbvector;
use melee_types::*;

/// Already ported (rounds 2/4): re-exported under this TU.



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
    pub static mut LBCOLL_80008440: Option<fn(*mut HitCapsule) > = None;
            #[allow(non_snake_case)]
            pub fn lbColl_80008440(hit: *mut HitCapsule)  {
                match unsafe { LBCOLL_80008440 } {
                    Some(f) => f(hit),
                    None => { unsafe { melee_runtime::tu::lbcollision::lbColl_80008440(hit as _); } return; },
                }
            }
    pub static mut LBCOLL_80008434: Option<fn(*mut HitCapsule) > = None;
            #[allow(non_snake_case)]
            pub fn lbColl_80008434(hit: *mut HitCapsule)  {
                match unsafe { LBCOLL_80008434 } {
                    Some(f) => f(hit),
                    None => { unsafe { melee_runtime::tu::lbcollision::lbColl_80008434(hit as _); } return; },
                }
            }
    crate::port_hook!(FTCOLL_8007ABD0, ftColl_8007ABD0, crate::adapt::ftColl_8007ABD0, (hit: *mut HitCapsule, damage: u32, gobj: *mut HSD_GObj));
    pub static mut FTCO_800BFFD0: Option<fn(*mut Fighter, u32, u32)> = None;
    #[allow(non_snake_case)]
    pub fn ftCo_800BFFD0(fp: *mut Fighter, a: u32, b: u32) {
        match unsafe { FTCO_800BFFD0 } {
            Some(f) => f(fp, a, b),
            None => { unsafe { melee_fighter::tu::ftcolanim::ftco_800bffd0(&mut *(fp as *mut _), a as _, b as _); } },
        }
    }
    pub static mut LB_8000B1CC: Option<fn(*mut HSD_JObj, *const Vec3, *mut Vec3) > = None;
            #[allow(non_snake_case)]
            pub fn lb_8000B1CC(jobj: *mut HSD_JObj, off: *const Vec3, out: *mut Vec3)  {
                match unsafe { LB_8000B1CC } {
                    Some(f) => f(jobj, off, out),
                    None => { unsafe { melee_runtime::tu::lb_00B0::lb_8000B1CC(jobj as _, off as _, out as _); } return; },
                }
            }
    pub static mut MPCOLL_80041EE4: Option<fn(*mut CollData)> = None;
    #[allow(non_snake_case)]
    pub fn mpColl_80041EE4(coll: *mut CollData) {
        match unsafe { MPCOLL_80041EE4 } {
            Some(f) => f(coll),
            None => { unsafe { melee_world::tu::mpcoll::mpColl_80041EE4(coll as _); } },
        }
    }
    pub static mut MPCOLL_8004730C: Option<fn(*mut CollData, *mut ftCollisionBox) -> i32> = None;
    #[allow(non_snake_case)]
    pub fn mpColl_8004730C(coll: *mut CollData, ecb: *mut ftCollisionBox) -> i32 {
        match unsafe { MPCOLL_8004730C } {
            Some(f) => f(coll, ecb),
            None => { unsafe { melee_world::tu::mpcoll::mpColl_8004730C(coll as _, ecb as _) as _ } },
        }
    }
    pub static mut IT_802BFE5C: Option<fn(*mut HSD_GObj, *mut Vec3, f32) > = None;
            #[allow(non_snake_case)]
            pub fn it_802BFE5C(gobj: *mut HSD_GObj, vel: *mut Vec3, x: f32)  {
                match unsafe { IT_802BFE5C } {
                    Some(f) => f(gobj, vel, x),
                    None => { unsafe { melee_items::tu::itnessyoyo::it_802BFE5C(gobj as _, vel as _, x as _); } return; },
                }
            }
    pub static mut IT_802C0010: Option<fn(*mut HSD_GObj, *mut Vec3) > = None;
            #[allow(non_snake_case)]
            pub fn it_802C0010(gobj: *mut HSD_GObj, pos: *mut Vec3)  {
                match unsafe { IT_802C0010 } {
                    Some(f) => f(gobj, pos),
                    None => { unsafe { melee_items::tu::itnessyoyo::it_802C0010(gobj as _, pos as _); } return; },
                }
            }
    pub static mut IT_802BE5D8: Option<fn(*mut HSD_GObj, f32) > = None;
            #[allow(non_snake_case)]
            pub fn it_802BE5D8(gobj: *mut HSD_GObj, v: f32)  {
                match unsafe { IT_802BE5D8 } {
                    Some(f) => f(gobj, v),
                    None => { unsafe { melee_items::tu::itnessyoyo::it_802BE5D8(gobj as _, v as _); } return; },
                }
            }
    pub static mut IT_802BFEC4: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn it_802BFEC4(gobj: *mut HSD_GObj)  {
                match unsafe { IT_802BFEC4 } {
                    Some(f) => f(gobj),
                    None => { unsafe { melee_items::tu::itnessyoyo::it_802BFEC4(gobj as _); } return; },
                }
            }
    pub static mut IT_802BE958: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn it_802BE958(gobj: *mut HSD_GObj)  {
                match unsafe { IT_802BE958 } {
                    Some(f) => f(gobj),
                    None => { unsafe { melee_items::tu::itnessyoyo::it_802BE958(gobj as _); } return; },
                }
            }
    pub static mut IT_802BE9D8: Option<fn(*mut HSD_GObj, f32, *mut Vec3, i32) -> *mut HSD_GObj> = None;
            #[allow(non_snake_case)]
            pub fn it_802BE9D8(owner: *mut HSD_GObj, facing: f32, pos: *mut Vec3, motion_id: i32) -> *mut HSD_GObj {
                match unsafe { IT_802BE9D8 } {
                    Some(f) => f(owner, facing, pos, motion_id),
                    None => { return unsafe { melee_items::tu::itnessyoyo::it_802BE9D8(owner as _, facing as _, pos as _, motion_id as _) as _ }; },
                }
            }
    pub static mut IT_802BE598: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn it_802BE598(gobj: *mut HSD_GObj)  {
                match unsafe { IT_802BE598 } {
                    Some(f) => f(gobj),
                    None => { unsafe { melee_items::tu::itnessyoyo::it_802BE598(gobj as _); } return; },
                }
            }
    pub static mut IT_802BE5B8: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn it_802BE5B8(gobj: *mut HSD_GObj)  {
                match unsafe { IT_802BE5B8 } {
                    Some(f) => f(gobj),
                    None => { unsafe { melee_items::tu::itnessyoyo::it_802BE5B8(gobj as _); } return; },
                }
            }
}

const SFX_VOLUME_MAX: u8 = 127;
const SFX_PAN_MID: u8 = 64;
const Collide_FloorMask: i32 = Collide_FloorPush | Collide_FloorHug;
const Collide_CeilingMask: i32 = Collide_CeilingPush | Collide_CeilingHug;
/// `Ft_MF_SkipItemVis`.
const MF_SKIP_ITEM_VIS: u32 = Ft_MF_SkipItemVis;

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

#[inline(always)]
unsafe fn attrs(fp: *mut Fighter) -> *const ftNessAttributes {
    (*fp).dat_attrs as *const ftNessAttributes
}

/// `((Item*) gobj->user_data)->xC4_article_data->x4_specialAttributes`.
#[inline(always)]
unsafe fn yoyo_attr(yoyo: *mut HSD_GObj) -> *const itYoyoAttributes {
    let item = (*yoyo).user_data as *mut Item;
    (*(*item).xC4_article_data).x4_specialAttributes as *const itYoyoAttributes
}

#[inline(always)]
unsafe fn joint(fp: *mut Fighter, part: usize) -> *mut HSD_JObj {
    (*(*fp).parts.add(part)).joint as *mut HSD_JObj
}

/// C-ABI entries for the callbacks this TU stores in Fighter (`HSD_GObjEvent`).
pub unsafe extern "C" fn cb_ftNs_AttackHi4_YoyoStartTimedRehit(g: *mut HSD_GObj) {
    melee_core::ct!("cb_ftNs_AttackHi4_YoyoStartTimedRehit");
    ftNs_AttackHi4_YoyoStartTimedRehit(&mut *fpp(g))
}
pub unsafe extern "C" fn cb_ftNs_AttackHi4_YoyoUpdateHitPos(g: *mut HSD_GObj) {
    melee_core::ct!("cb_ftNs_AttackHi4_YoyoUpdateHitPos");
    ftNs_AttackHi4_YoyoUpdateHitPos(g)
}
pub unsafe extern "C" fn cb_ftNs_Init_OnDamage(g: *mut HSD_GObj) {
    melee_core::ct!("cb_ftNs_Init_OnDamage");
    crate::tu::ftness::ftNs_Init_OnDamage(g)
}
pub unsafe extern "C" fn cb_ftNs_AttackHi4_YoyoItemSetUnk(g: *mut HSD_GObj) {
    melee_core::ct!("cb_ftNs_AttackHi4_YoyoItemSetUnk");
    ftNs_AttackHi4_YoyoItemSetUnk(g)
}
pub unsafe extern "C" fn cb_ftNs_AttackHi4_YoyoItemSetUnk2(g: *mut HSD_GObj) {
    melee_core::ct!("cb_ftNs_AttackHi4_YoyoItemSetUnk2");
    ftNs_AttackHi4_YoyoItemSetUnk2(g)
}

pub fn ftNs_AttackHi4_YoyoUpdateHitPos(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_AttackHi4_YoyoUpdateHitPos");
    unsafe {
        let fp = fpp(gobj);
        let hitbox = &mut (*fp).x914[0] as *mut HitCapsule;
        if (*hitbox).state != HitCapsule_Disabled {
            if (*fp).u.ns.yoyo_hitbox_pos.x != 0.0 || (*fp).u.ns.yoyo_hitbox_pos.y != 0.0 {
                let pos = (*fp).u.ns.yoyo_hitbox_pos;
                melee_fighter::ft0df::ftcoll_8007b8a8(&mut (*fp).x914[0], &pos);
            }
        }
    }
}

pub fn ftNs_AttackHi4_YoyoCheckTimedRehit(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_AttackHi4_YoyoCheckTimedRehit");
    unsafe {
        let fp = fpp(gobj);
        if (*fp).cmd_vars[0] == 0 && (*fp).mv.ns.attackhi4.yoyoRehitTimer > 0 {
            (*fp).mv.ns.attackhi4.yoyoRehitTimer -= 1;
            if (*fp).mv.ns.attackhi4.yoyoRehitTimer == 0 {
                ext::lbColl_80008440(&mut (*fp).x914[0]);
                ext::lbColl_80008434(&mut (*fp).x914[0]);
            }
        }
    }
}

/// `ftNs_AttackHi4_YoyoApplyDamage` (static): scale the Yo-Yo hitbox damage by the charge.
pub fn ftNs_AttackHi4_YoyoApplyDamage(unk_float: f32, gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_AttackHi4_YoyoApplyDamage");
    unsafe {
        let fp = fpp(gobj);
        let a = attrs(fp);
        if unk_float != 0.0 && (*fp).x914[0].state == HitCapsule_Enabled {
            // 0.0039059999398887157f: "likely 1/256 but won't match".
            let mul: f32 = 0.0039059999398887157_f32;
            let charge_duration = (*a).xAC_YOYO_CHARGE_DURATION;
            let charge_duration2 = unk_float / charge_duration;
            let damage_mul = fmadds((*a).xB0_YOYO_DAMAGE_MUL, mul, -1.0);
            let final_damage = fmadds(damage_mul, charge_duration2, 1.0);
            let final_damage = (*fp).x914[0].damage * final_damage;
            // u32 parameter: C float->unsigned conversion (saturating in the shipping build).
            ext::ftColl_8007ABD0(&mut (*fp).x914[0], final_damage as u32, gobj);
        }
    }
}

/// `push_ecb` (static inline): last = cur, cur = new.
fn push_ecb(coll: &mut CollData, b: &Vec3) {
    coll.last_pos = coll.cur_pos;
    coll.cur_pos = *b;
}

/// Returns the environment-hit bit set of a swept Yo-Yo probe (see the C `retval`).
pub fn ftNs_AttackHi4_YoyoCheckEnvColl(gobj: *mut HSD_GObj, ecb_unk: &Vec3, ecb_unk2: &Vec3, float_unk: f32) -> i32 {
    melee_core::ct!("ftNs_AttackHi4_YoyoCheckEnvColl");
    unsafe {
        let fp = fpp(gobj);
        let mut coll: Box<CollData> = Box::new(CollData::zeroed());
        let mut ecb: ftCollisionBox = ftCollisionBox::zeroed();
        let y_scale = float_unk * (*fp).x34_scale.y;

        ecb.top = y_scale;
        ecb.right.x = y_scale;
        ecb.bottom = -y_scale;
        ecb.left.x = -y_scale;
        ecb.right.y = 0.0;
        ecb.left.y = 0.0;

        ext::mpColl_80041EE4(&mut *coll);
        push_ecb(&mut coll, ecb_unk);
        push_ecb(&mut coll, ecb_unk2);
        coll.x34_flags.set_b1234(5);
        ext::mpColl_8004730C(&mut *coll, &mut ecb);

        let mut retval = 0;
        if coll.env_flags & Collide_FloorMask != 0 {
            retval |= 32768;
        }
        if coll.env_flags & Collide_LeftWallMask != 0 {
            retval |= 1;
        }
        if coll.env_flags & Collide_RightWallMask != 0 {
            retval |= 64;
        }
        if coll.env_flags & Collide_CeilingMask != 0 {
            retval |= 8192;
        }
        retval
    }
}

/// `lb_8000B1CC(parts[61].joint) -> floor-rotated about XRotN` (shared by SetUnkPos / SetHitPos /
/// SetHitPosUnk): the Yo-Yo hand position, optionally rotated by the floor normal.
unsafe fn hand_pos(fp: *mut Fighter) -> Vec3 {
    let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    if (*fp).mv.ns.attackhi4.isPosUpdateMod == 0 {
        ext::lb_8000B1CC(joint(fp, 61), core::ptr::null(), &mut pos);
        return pos;
    }
    let mut hand = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut origin = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    ext::lb_8000B1CC(joint(fp, 61), core::ptr::null(), &mut hand);
    ext::lb_8000B1CC(joint(fp, FtPart_XRotN as usize), core::ptr::null(), &mut origin);
    pos = hand;
    // `melee_types::Vec` and `melee_core::vec::Vec3` are the same three f32s.
    let cv = |v: &mut Vec3| &mut *(v as *mut Vec3 as *mut melee_core::vec::Vec3);
    let co = &*(&origin as *const Vec3 as *const melee_core::vec::Vec3);
    lbvector::sub(cv(&mut pos), co);
    let n = &(*fp).coll_data.floor.normal;
    lbvector::rotate(cv(&mut pos), 4, -atan2f(n.x, n.y));
    lbvector::add(cv(&mut pos), co);
    pos
}

pub fn ftNs_AttackHi4_YoyoSetUnkPos(gobj: *mut HSD_GObj, pos: &mut Vec3) {
    melee_core::ct!("ftNs_AttackHi4_YoyoSetUnkPos");
    unsafe {
        *pos = hand_pos(fpp(gobj));
    }
}

pub fn ftNs_AttackHi4_YoyoSetHitPos(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_AttackHi4_YoyoSetHitPos");
    unsafe {
        let fp = fpp(gobj);
        (*fp).u.ns.yoyo_hitbox_pos = hand_pos(fp);
    }
}

/// Blend `target` into `cur` by `w`: `target * w + cur * (1 - w)` per axis (one fmadds each).
#[inline(always)]
pub fn blend_hit_pos(target: &Vec3, cur: &Vec3, w: f32) -> Vec3 {
    melee_core::ct!("blend_hit_pos");
    let inv = 1.0 - w;
    Vec3 {
        x: fmadds(target.x, w, cur.x * inv),
        y: fmadds(target.y, w, cur.y * inv),
        z: fmadds(target.z, w, cur.z * inv),
    }
}

pub fn ftNs_AttackHi4_YoyoSetHitPosUnk(gobj: *mut HSD_GObj, pos_unk: f32) {
    melee_core::ct!("ftNs_AttackHi4_YoyoSetHitPosUnk");
    unsafe {
        let fp = fpp(gobj);
        let sp3c = hand_pos(fp);
        let sp30 = (*fp).u.ns.yoyo_hitbox_pos;
        (*fp).u.ns.yoyo_hitbox_pos = blend_hit_pos(&sp3c, &sp30, pos_unk);
    }
}

/// C `bool`: the Yo-Yo can be charged (nothing in the way and floor under the string).
pub fn ftNs_AttackHi4_YoyoCheckNoObstruct(gobj: *mut HSD_GObj) -> bool {
    melee_core::ct!("ftNs_AttackHi4_YoyoCheckNoObstruct");
    unsafe {
        let fp = fpp(gobj);
        let ecb_x = 0.0_f32;
        let ecb_mul_y = 0.5_f32;
        let mut sp20 = Vec3 {
            x: ecb_x,
            y: ecb_mul_y * ((*fp).coll_data.ecb.top.y + (*fp).coll_data.ecb.bottom.y),
            z: ecb_x,
        };
        sp20.x += (*fp).cur_pos.x;
        sp20.y += (*fp).cur_pos.y;
        sp20.z += (*fp).cur_pos.z;
        let mut sp14 = (*fp).u.ns.yoyo_hitbox_pos;

        if ftNs_AttackHi4_YoyoCheckEnvColl(gobj, &sp20, &sp14, ecb_mul_y) == 0 {
            sp14 = (*fp).u.ns.yoyo_hitbox_pos;
            sp20 = sp14;
            sp20.y += (*fp).x34_scale.y;
            sp14.y = fmadds(-1.0, (*fp).x34_scale.y, sp14.y);
            if ftNs_AttackHi4_YoyoCheckEnvColl(gobj, &sp20, &sp14, 1.5) & Collide_FloorPush != 0 {
                return true;
            }
        }
        false
    }
}

pub fn ftNs_AttackHi4_YoyoApplySmash(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_AttackHi4_YoyoApplySmash");
    unsafe {
        let fp = fpp(gobj);
        (*fp).mv.ns.attackhi4.yoyoCurrentFrame = 0;
        let temp_yoyo = (*fp).u.ns.yoyo_gobj;
        if !temp_yoyo.is_null() {
            let ya = yoyo_attr(temp_yoyo);
            let mut sp18 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
            sp18.x = (*ya).x24_YOYO_RELEASE_VEL_X;
            let pos_x = sp18.x;
            sp18.x = pos_x * (*fp).facing_dir;
            sp18.y = (*ya).x30_YOYO_RELEASE_VEL_Y;
            ext::it_802BFE5C(temp_yoyo, &mut sp18, pos_x);
        }

        let col_anim_id: i32 = 119;
        (*fp).smash_attrs.state = 4;
        (*fp).smash_attrs.x2124_frameSpeedMul = (*fp).frame_speed_mul;
        (*fp).smash_attrs.x2118_frames = 0.0;
        (*fp).smash_attrs.x211C_holdFrame = 60.0;
        (*fp).smash_attrs.x212C = 0;
        (*fp).smash_attrs.x212D = melee_fighter::hooks::smash_charge_shake_table().x4 as u8;
        (*fp).smash_attrs.x2128 = col_anim_id;
        if (*fp).smash_attrs.x2128 != 123 {
            ext::ftCo_800BFFD0(fp, (*fp).smash_attrs.x2128 as u32, 0);
        }
        st::ftCommon_8007EBAC(&mut *fp, 36, 0);
        melee_fighter::ft0881::ft_80088510(&mut *fp, 210087, SFX_VOLUME_MAX, SFX_PAN_MID);
    }
}

/// Damage scale of a charged Yo-Yo smash from the charge frames: the shared fused expression of
/// `YoyoSetChargeDamage`: `((1/256 * mul) - 1) * (frames / duration) + 1`.
#[inline(always)]
pub fn charge_damage_scale(mul: f32, frames: f32, duration: f32) -> f32 {
    melee_core::ct!("charge_damage_scale");
    let damage_mul = fmadds(0.0039059999398887157_f32, mul, -1.0);
    fmadds(damage_mul, frames / duration, 1.0)
}

pub fn ftNs_AttackHi4_YoyoSetChargeDamage(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_AttackHi4_YoyoSetChargeDamage");
    unsafe {
        let fp = fpp(gobj);
        (*fp).mv.ns.attackhi4.yoyoCurrentFrame = 14;
        let yoyo_gobj = (*fp).u.ns.yoyo_gobj;
        if !yoyo_gobj.is_null() {
            let ya = yoyo_attr(yoyo_gobj);
            let mut sp30 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
            ext::it_802C0010(yoyo_gobj, &mut sp30);
            (*fp).u.ns.x223C = (*ya).x18_SPIN_TEXANIM_SPEED;
        }
        melee_fighter::ft0881::ft_80088510(&mut *fp, 210090, SFX_VOLUME_MAX, SFX_PAN_MID);
        let smash_charge_frames = (*fp).smash_attrs.x2118_frames;
        let a = attrs(fp);
        if 0.0 != smash_charge_frames && (*fp).x914[0].state == HitCapsule_Enabled {
            let scale = charge_damage_scale((*a).xB0_YOYO_DAMAGE_MUL, smash_charge_frames, (*a).xAC_YOYO_CHARGE_DURATION);
            ext::ftColl_8007ABD0(&mut (*fp).x914[0], ((*fp).x914[0].damage * scale) as u32, gobj);
        }
    }
}

/// C `bool`: the Yo-Yo item failed to spawn and Ness dropped back to Wait.
pub fn ftNs_AttackHi4_YoyoThink_IsRemove(gobj: *mut HSD_GObj) -> bool {
    melee_core::ct!("ftNs_AttackHi4_YoyoThink_IsRemove");
    unsafe {
        let fp = fpp(gobj);
        if (*fp).cmd_vars[0] != 0 {
            (*fp).mv.ns.attackhi4.isPosUpdateMod = 0;
        }
        let mut msid = (*fp).motion_id as i32;
        let in_up_smash = |m: i32| m >= ftNs_MS_AttackHi4 as i32 && m <= ftNs_MS_AttackHi4Release as i32;
        let yoyo_spawn_frame = 2;
        let yoyo_despawn_frame = if in_up_smash(msid) { 49 } else { 60 };
        let cur = (*fp).mv.ns.attackhi4.yoyoCurrentFrame;
        if cur > yoyo_spawn_frame && cur <= yoyo_despawn_frame {
            let yoyo_gobj = (*fp).u.ns.yoyo_gobj;
            let ya = yoyo_attr(yoyo_gobj);
            if (*fp).cmd_vars[1] == 1 {
                (*fp).cmd_vars[1] = 0;
                ext::it_802BE5D8(yoyo_gobj, 1.0);
                (*fp).u.ns.x223C = (*ya).x18_SPIN_TEXANIM_SPEED;
            }
            if (*fp).cmd_vars[1] == 2 {
                (*fp).cmd_vars[1] = 0;
                ext::it_802BE5D8(yoyo_gobj, 0.0);
                (*fp).u.ns.x223C = 0.0;
                melee_fighter::ft0881::ft_80088510(&mut *fp, 210096, SFX_VOLUME_MAX, SFX_PAN_MID);
            }
            msid = (*fp).motion_id as i32;
            let (yoyo_rot_frame, yoyo_nudge_frame) = if in_up_smash(msid) {
                ((*ya).x40_UPSMASH_YOYO_ROT_FRAME, (*ya).x44_UPSMASH_YOYO_NUDGE_FRAME)
            } else {
                ((*ya).x48_DOWNSMASH_YOYO_ROT_FRAME, (*ya).x4C_DOWNSMASH_YOYO_NUDGE_FRAME)
            };
            let cur2 = (*fp).mv.ns.attackhi4.yoyoCurrentFrame;
            if cur2 == yoyo_rot_frame {
                let mut sp14 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
                ext::it_802C0010(yoyo_gobj, &mut sp14);
                msid = (*fp).motion_id as i32;
                if in_up_smash(msid) {
                    melee_fighter::ft0881::ft_80088510(&mut *fp, 210099, SFX_VOLUME_MAX, SFX_PAN_MID);
                } else {
                    melee_fighter::ft0881::ft_80088510(&mut *fp, 210093, SFX_VOLUME_MAX, SFX_PAN_MID);
                }
            } else if cur2 == yoyo_nudge_frame {
                ext::it_802BFEC4(yoyo_gobj);
            }
            if (*fp).mv.ns.attackhi4.yoyoCurrentFrame == yoyo_despawn_frame {
                ext::it_802BE958((*fp).u.ns.yoyo_gobj);
            }
        } else if cur == yoyo_spawn_frame {
            ftNs_AttackHi4_YoyoCreateItem(gobj);
            if (*fp).u.ns.yoyo_gobj.is_null() {
                st::ft_8008A2BC(gobj);
                return true;
            }
        }
        false
    }
}

pub fn ftNs_AttackHi4_YoyoSetUnkRate(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_AttackHi4_YoyoSetUnkRate");
    unsafe {
        let fp = fpp(gobj);
        let a = attrs(fp);
        let yoyo_gobj = (*fp).u.ns.yoyo_gobj;
        if !yoyo_gobj.is_null() {
            let ya = yoyo_attr(yoyo_gobj);
            let texanim_unk = (*ya).x20_UNK_TEXANIM_MOD;
            let mut yoyo_float = texanim_unk - (*ya).x1C_UNK_TEXANIM_SPEED;
            yoyo_float *= (*fp).mv.ns.attackhi4.yoyoCurrentFrame as f32 / (*a).xAC_YOYO_CHARGE_DURATION;
            (*fp).u.ns.x223C = texanim_unk - yoyo_float;
        }
    }
}

pub fn ftNs_AttackHi4_YoyoCreateItem(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_AttackHi4_YoyoCreateItem");
    unsafe {
        let fp = fpp(gobj);
        let mut sp10 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        ext::lb_8000B1CC(joint(fp, FtPart_R2ndNa as usize), core::ptr::null(), &mut sp10);
        (*fp).u.ns.yoyo_gobj = ext::it_802BE9D8(gobj, (*fp).facing_dir, &mut sp10, (*fp).motion_id as i32);
        (*fp).x1984_heldItemSpec = (*fp).u.ns.yoyo_gobj;
        if !(*fp).u.ns.yoyo_gobj.is_null() {
            if (*fp).death2_cb.is_none() {
                (*fp).death2_cb = Some(cb_ftNs_Init_OnDamage);
            }
            if (*fp).take_dmg_cb.is_none() {
                (*fp).take_dmg_cb = Some(cb_ftNs_Init_OnDamage);
            }
        }
        (*fp).pre_hitlag_cb = Some(cb_ftNs_AttackHi4_YoyoItemSetUnk);
        (*fp).post_hitlag_cb = Some(cb_ftNs_AttackHi4_YoyoItemSetUnk2);
    }
}

pub fn ftNs_AttackHi4_YoyoItemSetFlag(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_AttackHi4_YoyoItemSetFlag");
    unsafe {
        let fp = fpp(gobj);
        ftNs_AttackHi4_YoyoItemSetUnk2(gobj);
        (*fp).u.ns.yoyo_gobj = core::ptr::null_mut();
    }
}

pub fn ftNs_AttackHi4_YoyoItemDespawn(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_AttackHi4_YoyoItemDespawn");
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.ns.yoyo_gobj.is_null() {
            ext::it_802BE958((*fp).u.ns.yoyo_gobj);
            ftNs_AttackHi4_YoyoItemSetUnk2(gobj);
            (*fp).u.ns.yoyo_gobj = core::ptr::null_mut();
        }
    }
}

pub fn ftNs_AttackHi4_YoyoItemSetUnk(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_AttackHi4_YoyoItemSetUnk");
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.ns.yoyo_gobj.is_null() {
            ext::it_802BE598((*fp).u.ns.yoyo_gobj);
        }
    }
}

pub fn ftNs_AttackHi4_YoyoItemSetUnk2(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_AttackHi4_YoyoItemSetUnk2");
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.ns.yoyo_gobj.is_null() {
            ext::it_802BE5B8((*fp).u.ns.yoyo_gobj);
        }
    }
}

/// Arm the Yo-Yo callbacks every smash entry sets last (`x2222_b2 = 1`, deal-damage rehit timer,
/// hit-position update).
pub unsafe fn set_yoyo_callbacks(fp: *mut Fighter) {
    melee_core::ct!("set_yoyo_callbacks");
    (*fp).set_x2222_b2(1);
    (*fp).deal_dmg_cb = Some(cb_ftNs_AttackHi4_YoyoStartTimedRehit);
    (*fp).accessory4_cb = Some(cb_ftNs_AttackHi4_YoyoUpdateHitPos);
}

pub fn ftNs_AttackHi4_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_AttackHi4_Enter");
    unsafe {
        let fp = fpp(gobj);
        (*fp).set_allow_interrupt(0);
        (*fp).mv.ns.attackhi4.isChargeDisable = 0;
        // Inlined `ftNs_AttackHi4_YoyoSetVarAll`.
        (*fp).cmd_vars[1] = 0;
        (*fp).cmd_vars[0] = 0;
        (*fp).mv.ns.attackhi4.yoyoCurrentFrame = 1;
        (*fp).mv.ns.attackhi4.yoyoRehitTimer = 0;
        (*fp).mv.ns.attackhi4.isPosUpdateMod = 1;
        (*fp).u.ns.yoyo_hitbox_pos.z = 0.0;
        (*fp).u.ns.yoyo_hitbox_pos.y = 0.0;
        (*fp).u.ns.yoyo_hitbox_pos.x = 0.0;
        (*fp).u.ns.x223C = 0.0;

        st::Fighter_ChangeMotionState(gobj, ftNs_MS_AttackHi4 as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        set_yoyo_callbacks(fp);
    }
}

pub fn ftNs_AttackHi4_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_AttackHi4_Anim");
    unsafe {
        let fp = fpp(gobj);
        (*fp).mv.ns.attackhi4.yoyoCurrentFrame += 1;
        if !ftNs_AttackHi4_YoyoThink_IsRemove(gobj) {
            ftNs_AttackHi4_YoyoCheckTimedRehit(gobj);
            if (*fp).mv.ns.attackhi4.yoyoCurrentFrame == 13
                && (*fp).mv.ns.attackhi4.isChargeDisable == 0
                && ftNs_AttackHi4_YoyoCheckNoObstruct(gobj)
            {
                ftNs_AttackHi4Charge_Enter(gobj);
            }
            if st::ftAnim_IsFramesRemaining(gobj) == 0 {
                st::ft_8008A2BC(gobj);
            }
        }
    }
}

pub fn ftNs_AttackHi4_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_AttackHi4_IASA");
    unsafe {
        let fp = fpp(gobj);
        if (*fp).input.held_buttons[0] as i32 & HSD_PAD_A == 0 {
            // No Yo-Yo charge until the next Up Smash.
            (*fp).mv.ns.attackhi4.isChargeDisable = 1;
        }
        if (*fp).allow_interrupt() != 0 {
            st::ftCo_Wait_IASA(gobj);
        }
    }
}

pub fn ftNs_AttackHi4_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_AttackHi4_Phys");
    unsafe {
        st::ft_80084F3C(gobj);
        let fp = fpp(gobj);
        let mut sp10 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        ftNs_AttackHi4_YoyoSetUnkPos(gobj, &mut sp10);
        (*fp).u.ns.yoyo_hitbox_pos = sp10;
    }
}

/// Shared air tail of the three `*_Coll` callbacks: in the air the Yo-Yo is despawned.
unsafe fn despawn_in_air(gobj: *mut HSD_GObj, fp: *mut Fighter) {
    if (*fp).ground_or_air as i32 == GA_Air as i32 && !(*fp).u.ns.yoyo_gobj.is_null() {
        ext::it_802BE958((*fp).u.ns.yoyo_gobj);
        let fp = fpp(gobj);
        if !(*fp).u.ns.yoyo_gobj.is_null() {
            ext::it_802BE5B8((*fp).u.ns.yoyo_gobj);
        }
        (*fp).u.ns.yoyo_gobj = core::ptr::null_mut();
    }
}

pub fn ftNs_AttackHi4_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_AttackHi4_Coll");
    unsafe {
        let fp = fpp(gobj);
        st::ft_80084104(gobj);
        despawn_in_air(gobj, fp);
    }
}

pub fn ftNs_AttackHi4Charge_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_AttackHi4Charge_Anim");
    unsafe {
        let fp = fpp(gobj);
        let a = attrs(fp);
        (*fp).mv.ns.attackhi4.yoyoCurrentFrame += 1;
        let yoyo_gobj = (*fp).u.ns.yoyo_gobj;
        if !yoyo_gobj.is_null() {
            let ya = yoyo_attr(yoyo_gobj);
            let mut unk_float = (*ya).x20_UNK_TEXANIM_MOD - (*ya).x1C_UNK_TEXANIM_SPEED;
            unk_float *= (*fp).mv.ns.attackhi4.yoyoCurrentFrame as f32 / (*a).xAC_YOYO_CHARGE_DURATION;
            (*fp).u.ns.x223C = (*ya).x20_UNK_TEXANIM_MOD - unk_float;
        }
        ftNs_AttackHi4_YoyoCheckTimedRehit(gobj);
        if (*fp).mv.ns.attackhi4.yoyoCurrentFrame as f32 >= (*a).xAC_YOYO_CHARGE_DURATION {
            ftNs_AttackHi4Release_Enter(gobj);
        }
    }
}

pub fn ftNs_AttackHi4Charge_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_AttackHi4Charge_IASA");
    unsafe {
        let fp = fpp(gobj);
        if (*fp).input.held_buttons[0] as i32 & HSD_PAD_A == 0 {
            ftNs_AttackHi4Release_Enter(gobj);
        }
    }
}

pub fn ftNs_AttackHi4Charge_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_AttackHi4Charge_Phys");
    st::ft_80084F3C(gobj);
}

pub fn ftNs_AttackHi4Charge_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_AttackHi4Charge_Coll");
    unsafe {
        let fp = fpp(gobj);
        st::ft_80084104(gobj);
        despawn_in_air(gobj, fp);
    }
}

pub fn ftNs_AttackHi4Charge_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_AttackHi4Charge_Enter");
    unsafe {
        let fp = fpp(gobj);
        st::Fighter_ChangeMotionState(
            gobj,
            ftNs_MS_AttackHi4Charge as i32,
            MF_SKIP_ITEM_VIS,
            12.0,
            1.0,
            0.0,
            core::ptr::null_mut(),
        );
        st::ftAnim_8006EBA4(gobj);
        st::ftAnim_SetAnimRate(gobj, 0.0);
        ftNs_AttackHi4_YoyoApplySmash(gobj);
        set_yoyo_callbacks(fp);
    }
}

pub fn ftNs_AttackHi4Release_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_AttackHi4Release_Anim");
    unsafe {
        let fp = fpp(gobj);
        (*fp).mv.ns.attackhi4.yoyoCurrentFrame += 1;
        if !ftNs_AttackHi4_YoyoThink_IsRemove(gobj) {
            ftNs_AttackHi4_YoyoCheckTimedRehit(gobj);
            if st::ftAnim_IsFramesRemaining(gobj) == 0 {
                st::ft_8008A2BC(gobj);
            }
        }
    }
}

pub fn ftNs_AttackHi4Release_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_AttackHi4Release_IASA");
    unsafe {
        let fp = fpp(gobj);
        if (*fp).allow_interrupt() != 0 {
            st::ftCo_Wait_IASA(gobj);
        }
    }
}

pub fn ftNs_AttackHi4Release_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_AttackHi4Release_Phys");
    unsafe {
        let fp = fpp(gobj);
        st::ft_80084F3C(gobj);
        let cur = (*fp).mv.ns.attackhi4.yoyoCurrentFrame;
        if cur < 24 {
            let mut phi = 0.10000000149011612_f32 * (cur as f32 - 14.0);
            if phi >= 1.0 {
                phi = 1.0;
            } else if phi <= 0.0 {
                phi = 0.0;
            }
            let mut sp24 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
            ftNs_AttackHi4_YoyoSetUnkPos(gobj, &mut sp24);
            let sp30 = (*fp).u.ns.yoyo_hitbox_pos;
            (*fp).u.ns.yoyo_hitbox_pos = blend_hit_pos(&sp24, &sp30, phi);
            return;
        }
        let mut sp18 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        ftNs_AttackHi4_YoyoSetUnkPos(gobj, &mut sp18);
        (*fp).u.ns.yoyo_hitbox_pos = sp18;
    }
}

pub fn ftNs_AttackHi4Release_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_AttackHi4Release_Coll");
    unsafe {
        let fp = fpp(gobj);
        st::ft_80084104(gobj);
        despawn_in_air(gobj, fp);
    }
}

pub fn ftNs_AttackHi4Release_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_AttackHi4Release_Enter");
    unsafe {
        let fp = fpp(gobj);
        st::Fighter_ChangeMotionState(
            gobj,
            ftNs_MS_AttackHi4Release as i32,
            MF_SKIP_ITEM_VIS,
            13.0,
            1.0,
            0.0,
            core::ptr::null_mut(),
        );
        st::ftAnim_8006EBA4(gobj);

        (*fp).mv.ns.attackhi4.yoyoCurrentFrame = 14;
        let yoyo_gobj = (*fp).u.ns.yoyo_gobj;
        if !yoyo_gobj.is_null() {
            let ya = yoyo_attr(yoyo_gobj);
            let mut sp34 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
            ext::it_802C0010(yoyo_gobj, &mut sp34);
            (*fp).u.ns.x223C = (*ya).x18_SPIN_TEXANIM_SPEED;
        }
        melee_fighter::ft0881::ft_80088510(&mut *fp, 210090, SFX_VOLUME_MAX, SFX_PAN_MID);
        ftNs_AttackHi4_YoyoApplyDamage((*fp).smash_attrs.x2118_frames, gobj);
        set_yoyo_callbacks(fp);
    }
}

// ---- folded from chars (consolidation step 20/21) ----
/// `(s32) xB4_YOYO_REHIT_RATE`: C float->int truncation. The shipping build's gwtool makes
/// fptosi saturating (NaN -> 0), which is Rust's `as i32`.
pub fn ftNs_AttackHi4_YoyoStartTimedRehit(fp: &mut Fighter) {
    melee_core::ct!("ftNs_AttackHi4_YoyoStartTimedRehit");
    let a: &ftNessAttributes = da(fp);
    unsafe { fp.mv.ns.attackhi4.yoyoRehitTimer = a.xB4_YOYO_REHIT_RATE as i32 };
}

/// `ftNs_AttackHi4_YoyoSetVarAll` (src/melee/ft/kinds/ftNess/ftnessattackhi4.c).
pub fn ftNs_AttackHi4_YoyoSetVarAll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_AttackHi4_YoyoSetVarAll");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        (*fp).cmd_vars[1] = 0_u32;
        (*fp).cmd_vars[0] = 0_u32;
        (*fp).mv.ns.attackhi4.yoyoCurrentFrame = 1_i32;
        (*fp).mv.ns.attackhi4.yoyoRehitTimer = 0_i32;
        (*fp).mv.ns.attackhi4.isPosUpdateMod = (1_i32 /* true */);
        (*fp).u.ns.yoyo_hitbox_pos.z = f32::from_bits(0x00000000) /* 0.0f */;
        (*fp).u.ns.yoyo_hitbox_pos.y = f32::from_bits(0x00000000) /* 0.0f */;
        (*fp).u.ns.yoyo_hitbox_pos.x = f32::from_bits(0x00000000) /* 0.0f */;
        (*fp).u.ns.x223C = f32::from_bits(0x00000000) /* 0.0f */;
    }
}
