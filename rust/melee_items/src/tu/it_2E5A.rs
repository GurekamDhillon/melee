//! Port of src/melee/it/kinds/it_2E5A.c: the unnamed `It_Kind_Unk4` item (the coin: bronze / silver / gold tiers,
//! `it_2E5A.x4` is the tier, `x8` = "collectible", `x0` the tier's value; `sub` tracks a bone for the pickup
//! range). `sqrtf_store` / `sqrtf_accurate_store` are the file's own copies of the MSL square roots
//! (`melee_runtime::gekko`); the volatile slots are only a store and a load of the same value.
//! Callees of unported TUs go through `ext` (unset: panics "TODO"). Checked against the C (whole TU, shipping
//! contraction) by tests/kinds_it_2E5A.rs.
//!
//! GameCube layout trap: `it_802E6380_tier` reads `tiers[1].threshold` as `((s32*) &tiers[0].ecb)[9]` (the
//! comment in the C says why: writing it plainly changed the PPC asm). That aliasing only holds with 4-byte
//! pointers, so the port reads `tiers[1].threshold` and the oracle compiles the C with the same substitution.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables, unused_mut, dead_code)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_core::fp::fmadds;
use melee_runtime::cabi::f2i32;
use melee_runtime::gekko;
use melee_types::*;

use super::kinds_inl::*;

// BEGIN GENERATED EXT (rust/melee_items/gen/kinds/gen_tu.py; the callees of the C TU)
pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    use core::ffi::c_void;
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)?;)*) => {
            pub mod hook {
                use super::*;
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => panic!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            } )*
        };
    }
    ext_fns! {
    fn HSD_JObjAnimAll(a0: *mut HSD_JObj);
    fn HSD_Randf() -> f32;
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj;
    fn Item_80268D34(a0: *mut HSD_GObj, a1: *mut ItemStateDesc);
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32);
    fn Item_8026A8EC(a0: *mut HSD_GObj);
    fn db_80225DD8(a0: *mut HSD_GObj, a1: *mut HSD_GObj);
    fn db_ShowCoinPickupRange() -> u32;
    fn gm_8016C6C0(a0: *mut HSD_GObj) -> i32;
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32;
    fn it_80272828(a0: i32) -> *mut HSD_GObj;
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32);
    fn it_8027321C(a0: *mut HSD_GObj);
    fn it_80273318(a0: *mut HSD_GObj, a1: *mut HSD_Joint);
    fn it_802750F8(a0: *mut HSD_GObj);
    fn it_80275158(a0: *mut HSD_GObj, a1: f32);
    fn it_80275444(a0: *mut HSD_GObj);
    fn it_80275640(a0: *mut HSD_GObj, a1: f32);
    fn it_80275D5C(a0: *mut HSD_GObj, a1: *mut _itECB);
    fn it_80276214(a0: *mut HSD_GObj);
    fn it_80276D9C(a0: *mut HSD_GObj, a1: i32) -> i32;
    fn it_80277C40(a0: *mut HSD_GObj, a1: i32);
    fn lbAudioAx_80023870(a0: i32, a1: i32, a2: i32, a3: i32) -> i32;
    fn lbVector_Add_xy(a0: *mut Vec, a1: *mut Vec) -> *mut Vec;
    fn lbVector_Mirror(a0: *mut Vec, a1: *mut Vec);
    fn lbVector_NormalizeXY(a0: *mut Vec) -> f32;
    fn lb_8000B1CC(a0: *mut HSD_JObj, a1: *mut Vec, a2: *mut Vec);
    fn mpCollGetSpeedFloor(a0: *mut CollData, a1: *mut Vec) -> i32;
    fn mpColl_80048844(a0: *mut CollData) -> i32;
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
    pub static mut Item_804A0C64: HSD_ObjAllocUnk = unsafe { core::mem::zeroed() };
    pub static mut it_804D6D28: *mut ItemCommonData = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT

const M_PI: f64 = 3.14159265358979323846;
const M_PI_2: f64 = M_PI / 2.0;
const M_TAU: f64 = 6.283185307179586;

/// `const Vec3 it_803B8718`
pub static it_803B8718: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
/// `const Vec3 it_803B8724`
pub static it_803B8724: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

/// `ItemStateTable it_803F8BD8[]`
pub static mut it_803F8BD8: [ItemStateTable; 3] = [
    st_row(0, pred_b(it_2E5A_UnkMotion1_Anim), Some(it_2E5A_UnkMotion1_Phys), pred_b(it_2E5A_UnkMotion0_Coll)),
    st_row(-1, pred_b(it_2E5A_UnkMotion1_Anim), Some(it_2E5A_UnkMotion1_Phys), None),
    st_row(0, pred_b(it_2E5A_UnkMotion2_Anim), Some(it_2E5A_UnkMotion2_Phys), pred_b(it_2E5A_UnkMotion2_Coll)),
];

#[inline]
fn ABS(x: f32) -> f32 {
    if x < 0.0 { -x } else { x }
}

/// `sqrtf_store` (static inline): MSL `sqrtf` (3 Newton steps) with the result also stored to `*y`.
#[inline]
unsafe fn sqrtf_store(x: f32, y: *mut f32) -> f32 {
    if x > 0.0 {
        *y = gekko::sqrtf(x);
        return *y;
    }
    x
}

/// `sqrtf_accurate_store` (static inline): as `sqrtf_store` with the extra (4th) Newton step.
#[inline]
unsafe fn sqrtf_accurate_store(x: f32, y: *mut f32) -> f32 {
    if x > 0.0 {
        *y = gekko::sqrtf_accurate(x);
        return *y;
    }
    x
}

/// 2E5AC4: collision for the coin; bounces (or comes to rest) on the floor. Returns whether it landed
/// (`false` again after a bounce).
pub unsafe extern "C" fn it_802E5AC4(item_gobj: *mut Item_GObj, arg_check: bool) -> bool {
    let mut sp40: Vec3 = core::mem::zeroed();
    let mut sp34: Vec3 = core::mem::zeroed();
    let mut sp28: Vec3 = core::mem::zeroed();
    let mut sqrt_0: f32 = 0.0;
    let mut sqrt_1: f32 = 0.0;
    let mut var_f31: f32 = 0.0;
    let mut check2: bool;

    let item = GET_ITEM(item_gobj);
    let coll_data = &raw mut (*item).x378_itemColl;
    let comm_attr = (*item).xCC_item_attr;
    let spec_attr = (*(*item).xC4_article_data).x4_specialAttributes as *mut it_2E5A_Attrs;
    ext::it_80276214(item_gobj);
    (*coll_data).cur_pos.y -= (*item).xC1C.bottom;
    let mut check1 = ext::mpColl_80048844(coll_data) != 0;
    (*coll_data).cur_pos.y += (*item).xC1C.bottom;
    (*item).pos = (*coll_data).cur_pos;

    // Rounds item velocities to 0 if they are close to it
    if check1 && arg_check {
        if ABS((*item).x40_vel.x) <= 0.00001 {
            (*item).x40_vel.x = 0.0;
        }
        if ABS((*item).x40_vel.y) <= 0.00001 {
            (*item).x40_vel.y = 0.0;
        }
        // block_18 (reset) when both components are at most x5c; block_16 otherwise
        let at_rest = ABS((*item).x40_vel.x) <= (*comm_attr).x5c && ABS((*item).x40_vel.y) <= (*comm_attr).x5c;
        if at_rest || (*item).xDCD_flag.anon_0.b4() != 0 || (*comm_attr).x58 == 0.0 {
            itResetVelocity(item);
            (*item).xD50_landNum = 0;
        } else {
            check2 = false;
            sp34 = it_803B8718;
            check1 = false;
            sp28 = it_803B8724;
            var_f31 = sqrtf_accurate_store(
                fmadds((*item).x40_vel.x, (*item).x40_vel.x, (*item).x40_vel.y * (*item).x40_vel.y),
                &mut sqrt_0,
            );
            if fmadds((*item).x40_vel.x, (*coll_data).floor.normal.x, (*item).x40_vel.y * (*coll_data).floor.normal.y) < 0.0 {
                sp40 = (*item).x40_vel;
                ext::lbVector_Mirror(&mut sp40, &mut (*coll_data).floor.normal);
                ext::lbVector_Add_xy(&mut sp28, &mut sp40);
                if ext::mpCollGetSpeedFloor(coll_data, &mut sp40) != 0 {
                    ext::lbVector_Add_xy(&mut sp34, &mut sp40);
                }
                check2 = true;
            }
            if check2 {
                ext::lbVector_NormalizeXY(&mut sp28);
                if sqrtf_store(fmadds(sp28.z, sp28.z, fmadds(sp28.x, sp28.x, sp28.y * sp28.y)), &mut sqrt_1) < 0.01 {
                    sp28.x = (*item).x40_vel.x;
                    sp28.y = -1.0 * (*item).x40_vel.y;
                }
                let temp_f1_10 = var_f31 * (*comm_attr).x58;
                sp28.x *= temp_f1_10;
                sp28.y *= temp_f1_10;
                (*item).x64_vec_unk2 = sp34;
                (*item).x40_vel = sp28;
            }
            if ext::it_80276D9C(item_gobj, 1) != 0 {
                ext::it_8027321C(item_gobj);
                ext::it_80277C40(item_gobj, 0);
                ext::it_80275640(item_gobj, (*comm_attr).x58);
            }
            if (*item).xD50_landNum == 1 {
                (*item).xD84 = (*spec_attr).tiers[0].xD84_value;
            }
            (*item).x40_vel.x *= (*spec_attr).x14;
            (*item).xD50_landNum = (*item).xD50_landNum.wrapping_add(1);
            if (*item).xD50_landNum >= 3 {
                if (*item).xDCF_flag.anon_0.b0() == 0 {
                    (*item).xDCF_flag.anon_0.set_b0(1);
                }
            }
        }
    }
    check1
}

/// 2E5EF4
pub unsafe extern "C" fn it_802E5EF4(item_gobj: *mut Item_GObj) -> i32 {
    (*GET_ITEM(item_gobj)).xDD4_itemVar.it_2E5A.x0
}

/// 2E5F00: spawns coins worth `arg3` at `pos` with velocity `vel`.
pub unsafe extern "C" fn it_802E5F00(item_gobj: *mut Item_GObj, pos: *mut Vec3, vel: *mut Vec3, arg3: i32) {
    let mut vars: it_802E5FXX_struct = core::mem::zeroed();
    let mut sp1C: SpawnItem = core::mem::zeroed();

    it_802E614C(null_mut(), item_gobj, &mut sp1C, pos, vel);
    vars.x0 = item_gobj;
    sp1C.facing_dir = -1.0;
    vars.x4 = 0.0;
    vars.x8 = 0.0;
    vars.xC = arg3;
    vars.x10 = 0;
    vars.x14 = 1;
    vars.x18 = 0;
    it_802E609C(&mut vars, &mut sp1C);
}

/// 2E5F8C: spawns coins worth `arg2`, flung at angle `arg4` with speed `arg5`; returns the remaining value.
pub unsafe extern "C" fn it_802E5F8C(item_gobj: *mut Item_GObj, pos: *mut Vec3, arg2: i32, arg3: i32, arg4: f32, arg5: f32) -> i32 {
    let mut vars: it_802E5FXX_struct = core::mem::zeroed();
    let mut sp2C: SpawnItem = core::mem::zeroed();
    let mut vel: Vec3 = core::mem::zeroed();

    vel.x = 0.0;
    vel.y = 0.0;
    vel.z = 0.0;
    it_802E614C(item_gobj, item_gobj, &mut sp2C, pos, &mut vel);
    let mut angle: f32 = (M_PI_2 + arg4 as f64) as f32;
    while (angle as f64) < 0.0 {
        angle = (angle as f64 + M_TAU) as f32;
    }
    while angle as f64 > M_TAU {
        angle = (angle as f64 - M_TAU) as f32;
    }
    let dir: f32 = if angle as f64 > M_PI { -1.0 } else { 1.0 };
    vars.x0 = item_gobj;
    vars.x4 = angle;
    vars.x8 = arg5;
    vars.xC = arg2;
    vars.x10 = 1;
    sp2C.facing_dir = dir;
    vars.x14 = arg3;
    vars.x18 = ext::gm_8016C6C0(item_gobj);
    it_802E609C(&mut vars, &mut sp2C)
}

/// 2E609C: spawns coins until `vars.xC` (the value left) is used up; returns the first coin's re-rolled total.
pub unsafe extern "C" fn it_802E609C(vars: *mut it_802E5FXX_struct, spawn: *mut SpawnItem) -> i32 {
    // `var_r30` is uninitialised in the C when no coin is rolled (the C build zero-initialises locals)
    let mut var_r30: i32 = 0;
    let mut var_r29: i32 = 0;

    while (*vars).xC != 0 {
        it_802E6658();
        let spawn_gobj = ext::Item_80268B18(spawn);
        if !spawn_gobj.is_null() {
            if (*vars).x10 == 1 && var_r29 == 0 {
                let temp_r3_2 = it_802E61C4(spawn_gobj, (*vars).xC, (*vars).x18);
                (*vars).xC = temp_r3_2;
                var_r30 = temp_r3_2;
                var_r29 = 1;
            }
            it_802E6380(spawn_gobj, vars);
            it_802E66A0(spawn_gobj);
            ext::db_80225DD8(spawn_gobj, (*vars).x0);
            ext::it_802750F8(spawn_gobj);
        }
    }
    var_r30
}

/// `static inline it_802E614C`: the coin's `SpawnItem` setup.
#[inline]
pub unsafe extern "C" fn it_802E614C(parent_gobj1: *mut Item_GObj, parent_gobj2: *mut Item_GObj, spawn: *mut SpawnItem, pos: *mut Vec3, vel: *mut Vec3) {
    (*spawn).kind = It_Kind_Unk4;
    (*spawn).prev_pos = *pos;
    (*spawn).pos = (*spawn).prev_pos;
    (*spawn).x3C_damage = 0;
    (*spawn).vel = *vel;
    (*spawn).x0_parent_gobj = parent_gobj1;
    (*spawn).x4_parent_gobj2 = parent_gobj2;
    (*spawn).x44_flag.anon_0.set_b0(1);
    (*spawn).x40 = 0;
}

/// 2E61C4: re-rolls the total value `arg1` (per-player multiplier `x2C[arg2]`, random +-`x24` percent).
pub unsafe extern "C" fn it_802E61C4(item_gobj: *mut Item_GObj, arg1: i32, arg2: i32) -> i32 {
    let item = GET_ITEM(item_gobj);
    let attr = (*(*item).xC4_article_data).x4_specialAttributes as *mut it_2E5A_Attrs;
    let mut var_r29: i32 = f2i32((arg1 as f32 * *(*attr).x2C.as_ptr().offset(arg2 as isize)) as f64);
    let new_var = ext::HSD_Randf();
    let temp_f1 = (*attr).x24 * new_var;
    var_r29 = f2i32((var_r29 as f32 * fmadds(0.01, temp_f1, 1.0)) as f64);
    if var_r29 < 1 {
        var_r29 = 1;
    }
    var_r29
}

/// 2E628C: random launch velocity.
pub unsafe extern "C" fn it_802E628C(item_gobj: *mut Item_GObj, arg8: f32, arg9: f32) {
    let item = GET_ITEM(item_gobj);
    let attr = (*(*item).xC4_article_data).x4_specialAttributes as *mut it_2E5A_Attrs;

    let temp_f31 = ext::HSD_Randf();
    let temp_f2 = (*attr).xC;
    let temp_f1 = (arg8 as f64 - M_PI_2) as f32;
    let temp_f0 = M_PI as f32;
    let mut var_f30 = temp_f31 * temp_f2;
    var_f30 *= (*item).facing_dir;
    if ((arg8 >= (temp_f0 / 3.0)) && (arg8 <= (2.0 * (temp_f0 / 3.0)))) || ((arg8 >= (4.0 * (temp_f0 / 3.0))) && (arg8 <= (5.0 * (temp_f0 / 3.0)))) {
        var_f30 = fmadds((*attr).x10, gekko::cosf(temp_f1), var_f30);
    }
    (*item).x40_vel.x = var_f30;
    let temp_f0 = (*attr).x20;
    let temp_f1 = (*attr).x18;
    let mut temp_f0 = temp_f31 * temp_f0;
    temp_f0 += arg9;
    let temp_f1 = temp_f1 + temp_f0;
    (*item).x40_vel.y = temp_f1;
    (*item).x40_vel.z = 0.0;
}

/// `static inline it_802E6380_inline`: sets up the pickup-range bone tracking.
#[inline]
unsafe fn it_802E6380_inline(item_gobj: *mut Item_GObj) {
    let item = GET_ITEM(item_gobj);
    (*item).xDD4_itemVar.it_2E5A.x18.anon_0.set_b1(0);
    (*item).xDD4_itemVar.it_2E5A.x18.anon_0.set_b0(0);

    if (*item).xDD4_itemVar.it_2E5A.x8 != 0 {
        let attr = (*(*item).xC4_article_data).x4_specialAttributes as *mut it_2E5A_Attrs;
        let sub = &raw mut (*item).xDD4_itemVar.it_2E5A.sub;
        (*sub).x4 = GET_JOBJ(item_gobj);
        (*sub).x0 = (0.003906 * (*attr).x28) / (*item).scl;
        (*item).x3C = (*sub).x0;
        ext::lb_8000B1CC((*sub).x4, null_mut(), &mut (*sub).x8);
        (*sub).x14 = (*sub).x8;
        if ext::db_ShowCoinPickupRange() != 0 {
            (*item).anon_daa.xDAA_flag.anon_0.set_b0(1);
        }
    }
}

/// `static inline it_802E6380_tier`: which tier the remaining value `arg1.xC` is spawned as.
#[inline]
unsafe fn it_802E6380_tier(item_gobj: *mut Item_GObj, attr: *mut it_2E5A_Attrs, arg1: *mut it_802E5FXX_struct) -> i32 {
    let mut off = 2;
    if (*arg1).xC < (*attr).tiers[2].threshold {
        off = 1;
        if (*arg1).xC < (*attr).tiers[1].threshold {
            off = 0;
        }
    }
    off
}

/// 2E6380: initialises one coin of the batch (tier, value, velocity, state); returns the value left.
pub unsafe extern "C" fn it_802E6380(item_gobj: *mut Item_GObj, arg1: *mut it_802E5FXX_struct) -> i32 {
    let item = GET_ITEM(item_gobj);
    let attr = (*(*item).xC4_article_data).x4_specialAttributes as *mut it_2E5A_Attrs;
    let off = it_802E6380_tier(item_gobj, attr, arg1);

    (*item).xDD4_itemVar.it_2E5A.x4 = off;

    (*arg1).xC = (*arg1).xC.wrapping_sub((*attr).tiers[off as usize].threshold);
    (*item).xDD4_itemVar.it_2E5A.x0 = (*attr).tiers[off as usize].threshold;

    if (*arg1).x10 == 1 {
        (*arg1).x8 *= (*attr).x1C;
        it_802E628C(item_gobj, (*arg1).x4, (*arg1).x8);
    }
    ext::it_80275158(item_gobj, (*attr).x0);
    (*item).xDD4_itemVar.it_2E5A.x10 = (*attr).x4;

    (*item).xDD4_itemVar.it_2E5A.x14 = (*attr).x8;
    (*item).xDD4_itemVar.it_2E5A.x8 = (*arg1).x14;
    (*item).xDCD_flag.anon_0.set_b6(1);

    {
        let tier_idx = off as usize;
        (*item).xD84 = (*attr).tiers[tier_idx].xD84_value;
        (*item).scl = (*attr).tiers[tier_idx].scale;
        ext::it_80273318(item_gobj, (*attr).tiers[tier_idx].joint);
        ext::it_80275D5C(item_gobj, &mut (*attr).tiers[tier_idx].ecb);
    }

    (*item).xC0C = (*item).xC1C;
    (*item).xC1C.bottom *= (*item).scl;
    (*item).xDD4_itemVar.it_2E5A.xC = 0;

    it_802E6380_inline(item_gobj);
    (*arg1).xC
}

/// 2E657C: lifetime / pickup-delay timers; true once the coin is gone.
pub unsafe extern "C" fn it_802E657C(item_gobj: *mut Item_GObj) -> bool {
    let item = GET_ITEM(item_gobj);
    let timer = (*item).xD44_lifeTimer;
    if timer <= 0.0 {
        return true;
    }
    (*item).xD44_lifeTimer = timer - 1.0;
    if (*item).xDD4_itemVar.it_2E5A.x8 == 1 {
        let temp_f0_2 = (*item).xDD4_itemVar.it_2E5A.x10;
        if temp_f0_2 <= 0.0 {
            let temp_r4 = (*item).xDD4_itemVar.it_2E5A.x18.anon_0.b0();
            if temp_r4 == 0 {
                (*item).xDD4_itemVar.it_2E5A.x18.anon_0.set_b0(1);
            }
        } else {
            (*item).xDD4_itemVar.it_2E5A.x10 = temp_f0_2 - 1.0;
        }
        let temp_f1 = (*item).xDD4_itemVar.it_2E5A.x14;
        if temp_f1 <= 0.0 {
            if (*item).xDCD_flag.anon_0.b5() != 1 {
                ext::it_80275444(item_gobj);
                (*item).xDD4_itemVar.it_2E5A.x18.anon_0.set_b1(1);
            }
        } else {
            (*item).xDD4_itemVar.it_2E5A.x14 = temp_f1 - 1.0;
        }
    }
    false
}

/// 2E6658: when the coin pool is nearly full, recycles the oldest coin.
pub unsafe extern "C" fn it_802E6658() {
    if (ext::Item_804A0C64.x50 as u32) >= (*ext::it_804D6D28).x20.wrapping_sub(2) {
        ext::Item_8026A8EC(ext::it_80272828(It_Kind_Unk4));
    }
}

/// `static inline it_2E5A_ApplyStateDesc`
#[inline]
unsafe fn it_2E5A_ApplyStateDesc(item_gobj: *mut HSD_GObj, idx: i32) {
    let item = (*item_gobj).user_data as *mut Item;
    let item_jobj = (*item_gobj).hsd_obj as *mut HSD_JObj;
    let attr = (*(*item).xC4_article_data).x4_specialAttributes as *mut it_2E5A_Attrs;
    (*item).xD0_itemStateDesc = &raw mut (*attr).tiers[idx as usize].anim_joint as *mut ItemStateDesc;
    ext::Item_80268D34(item_gobj, (*item).xD0_itemStateDesc);
    ext::HSD_JObjAnimAll(item_jobj);
}

/// 2E66A0: enters the coin's idle (collectible) or falling state.
pub unsafe extern "C" fn it_802E66A0(item_gobj: *mut HSD_GObj) {
    let item = (*item_gobj).user_data as *mut Item;
    if (*item).xDD4_itemVar.it_2E5A.x8 == 1 {
        if (*item).xDD4_itemVar.it_2E5A.xC != 0 {
            ext::Item_80268E5C(item_gobj, 0, (ITEM_HIT_PRESERVE | ITEM_UNK_0x1) as u32);
        } else {
            ext::Item_80268E5C(item_gobj, 0, ITEM_ANIM_UPDATE as u32);
            (*item).xDD4_itemVar.it_2E5A.xC = 1;
            it_2E5A_ApplyStateDesc(item_gobj, (*item).xDD4_itemVar.it_2E5A.x4);
        }
        (*item).on_accessory = Some(it_802E6A74);
        return;
    }
    ext::Item_80268E5C(item_gobj, 1, ITEM_ANIM_UPDATE as u32);
    it_2E5A_ApplyStateDesc(item_gobj, (*item).xDD4_itemVar.it_2E5A.x4);
}

pub unsafe extern "C" fn it_2E5A_UnkMotion1_Anim(item_gobj: *mut HSD_GObj) -> bool {
    let item = GET_ITEM(item_gobj);
    // `(0.1 * ABS(vy)) + 1.0` in double: one fused multiply-add
    (*item).x5D0_animFrameSpeed = (0.1f64).mul_add(ABS((*item).x40_vel.y) as f64, 1.0) as f32;
    it_802E657C(item_gobj)
}

pub unsafe extern "C" fn it_2E5A_UnkMotion1_Phys(item_gobj: *mut HSD_GObj) {
    let comm_attr = (*GET_ITEM(item_gobj)).xCC_item_attr;
    ext::it_80272860(item_gobj, (*comm_attr).x10_fall_speed, (*comm_attr).x14_fall_speed_max);
}

pub unsafe extern "C" fn it_2E5A_UnkMotion0_Coll(item_gobj: *mut HSD_GObj) -> bool {
    let item = GET_ITEM(item_gobj);
    if it_802E5AC4(item_gobj, true) {
        it_802E6888(item_gobj);
        (*item).xD50_landNum = 0;
        (*item).xDCF_flag.anon_0.set_b0(0);
    }
    false
}

/// 2E6888: lands (rest state).
pub unsafe extern "C" fn it_802E6888(item_gobj: *mut HSD_GObj) {
    let item = (*item_gobj).user_data as *mut Item;
    itResetVelocity(item);
    (*item).x5D0_animFrameSpeed = 1.0;
    ext::Item_80268E5C(item_gobj, 2, (ITEM_HIT_PRESERVE | ITEM_UNK_0x1) as u32);
    if (*item).xDD4_itemVar.it_2E5A.x8 == 1 {
        (*item).on_accessory = Some(it_802E6A74);
    }
}

pub unsafe extern "C" fn it_2E5A_UnkMotion2_Anim(item_gobj: *mut HSD_GObj) -> bool {
    it_802E657C(item_gobj)
}

pub unsafe extern "C" fn it_2E5A_UnkMotion2_Phys(item_gobj: *mut HSD_GObj) {}

pub unsafe extern "C" fn it_2E5A_UnkMotion2_Coll(item_gobj: *mut HSD_GObj) -> bool {
    let mut item = (*item_gobj).user_data as *mut Item;
    if ((*item).xDD4_itemVar.it_2E5A.x8 == 1) && !it_802E5AC4(item_gobj, false) {
        item = (*item_gobj).user_data as *mut Item;
        if (*item).xDD4_itemVar.it_2E5A.x8 == 1 {
            if (*item).xDD4_itemVar.it_2E5A.xC != 0 {
                ext::Item_80268E5C(item_gobj, 0, (ITEM_HIT_PRESERVE | ITEM_UNK_0x1) as u32);
            } else {
                ext::Item_80268E5C(item_gobj, 0, ITEM_ANIM_UPDATE as u32);
                (*item).xDD4_itemVar.it_2E5A.xC = 1;
                it_2E5A_ApplyStateDesc(item_gobj, (*item).xDD4_itemVar.it_2E5A.x4);
            }
            (*item).on_accessory = Some(it_802E6A74);
        } else {
            ext::Item_80268E5C(item_gobj, 1, ITEM_ANIM_UPDATE as u32);
            it_2E5A_ApplyStateDesc(item_gobj, (*item).xDD4_itemVar.it_2E5A.x4);
        }
    }
    false
}

pub unsafe extern "C" fn it_2E5A_Logic115_DmgDealt(item_gobj: *mut Item_GObj) -> bool {
    ext::lbAudioAx_80023870(0x93, 0x7F, 0x40, 0x1A);
    true
}

/// 2E6A74: remembers the bone position of the previous frame and reads the new one.
pub unsafe extern "C" fn it_802E6A74(gobj: *mut HSD_GObj) {
    let item = (*gobj).user_data as *mut Item;
    if (*item).xDD4_itemVar.it_2E5A.x8 != 0 {
        let sub = &raw mut (*item).xDD4_itemVar.it_2E5A.sub;
        (*sub).x14 = (*sub).x8;
        ext::lb_8000B1CC((*sub).x4, null_mut(), &mut (*sub).x8);
    }
}

pub unsafe extern "C" fn it_2E5A_Logic115_EvtUnk(item_gobj: *mut Item_GObj, ref_gobj: *mut HSD_GObj) {
    ext::it_8026B894(item_gobj, ref_gobj);
}
