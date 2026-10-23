//! Port of src/melee/it/itmaplib.c: the item-vs-map collision helpers (ECB set-up, wall/floor/ceiling queries,
//! bounce and slide). Callees of other TUs go through `ext`.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_assignments, unused_variables, unused_mut, clippy::missing_safety_doc)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;

// BEGIN GENERATED EXT (rust/melee_items/gen/mtu/gen_tu.py; the callees of the C TU)
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
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj);
    fn PSVECAdd(a0: *mut Vec, a1: *mut Vec, a2: *mut Vec);
    fn it_8026BB68(a0: *mut HSD_GObj, a1: *mut Vec);
    fn it_8026BB88(a0: *mut HSD_GObj, a1: *mut Vec);
    fn it_8026BC90(a0: *mut HSD_GObj, a1: *mut Vec);
    fn it_8026C100(a0: *mut HSD_GObj) -> *mut CollData;
    fn it_80272CC0(a0: *mut HSD_GObj, a1: i32) -> *mut HSD_JObj;
    fn it_80272D40(a0: *mut HSD_GObj) -> i32;
    fn it_8027321C(a0: *mut HSD_GObj);
    fn it_80274990(a0: *mut HSD_GObj) -> f32;
    fn it_80274A64(a0: *mut HSD_GObj);
    fn it_80275640(a0: *mut HSD_GObj, a1: f32);
    fn it_80278800(a0: *mut HSD_GObj, a1: i32, a2: i32, a3: *mut Vec, a4: *mut Vec, a5: i32, a6: f32);
    fn lbVector_Add_xy(a0: *mut Vec, a1: *mut Vec) -> *mut Vec;
    fn lbVector_Angle(a0: *mut Vec, a1: *mut Vec) -> f32;
    fn lbVector_AngleXY(a0: *mut Vec, a1: *mut Vec) -> f32;
    fn lbVector_CrossprodNormalized(a0: *mut Vec, a1: *mut Vec, a2: *mut Vec) -> *mut Vec;
    fn lbVector_Mirror(a0: *mut Vec, a1: *mut Vec);
    fn lbVector_NormalizeXY(a0: *mut Vec) -> f32;
    fn mpCheckMultiple(a0: f32, a1: f32, a2: f32, a3: f32, a4: *mut Vec, a5: *mut i32, a6: *mut u32, a7: *mut Vec, a8: u32, a9: i32, a10: i32) -> i32;
    fn mpCollGetSpeedCeiling(a0: *mut CollData, a1: *mut Vec) -> i32;
    fn mpCollGetSpeedFloor(a0: *mut CollData, a1: *mut Vec) -> i32;
    fn mpCollGetSpeedLeftWall(a0: *mut CollData, a1: *mut Vec) -> i32;
    fn mpCollGetSpeedRightWall(a0: *mut CollData, a1: *mut Vec) -> i32;
    fn mpCollSetFacingDir(a0: *mut CollData, a1: i32);
    fn mpColl_80041EE4(a0: *mut CollData);
    fn mpColl_80043670(a0: *mut CollData);
    fn mpColl_80043680(a0: *mut CollData, a1: *mut Vec);
    fn mpColl_800436E4(a0: *mut CollData, a1: f32);
    fn mpColl_800471F8(a0: *mut CollData) -> i32;
    fn mpColl_SetECBSource_Fixed(a0: *mut CollData, a1: *mut HSD_GObj, a2: f32, a3: f32, a4: f32, a5: f32);
    fn mpLib_8004DD90_Floor(a0: i32, a1: *mut Vec, a2: *mut f32, a3: *mut u32, a4: *mut Vec) -> i32;
    fn mpLib_8005199C_Floor(a0: *mut Vec, a1: i32, a2: i32) -> i32;
    fn mpLib_80054ED8(a0: i32) -> i32;
    fn mpLineGetNormal(a0: i32, a1: *mut Vec) -> *mut Vec;
    fn mpLinesConnected(a0: i32, a1: i32) -> i32;
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
    }
    // Data defined in other TUs (the tests set them).
    pub static mut it_804D6D28: *mut ItemCommonData = unsafe { core::mem::zeroed() };
    pub static mut result: i32 = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT

pub static it_803B8570: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
pub static it_803B857C: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

#[inline(always)]
unsafe fn item(gobj: *mut HSD_GObj) -> *mut Item {
    (*gobj).user_data as *mut Item
}

#[inline(always)]
fn abs_c(x: f32) -> f32 {
    if x < 0.0 { -x } else { x }
}

/// `HSD_JObjMtxIsDirty` (header inline).
#[inline(always)]
unsafe fn HSD_JObjMtxIsDirty(jobj: *mut HSD_JObj) -> bool {
    assert!(!jobj.is_null(), "HSD_ASSERT(564, jobj)");
    (*jobj).flags & JOBJ_USER_DEF_MTX as u32 == 0 && (*jobj).flags & JOBJ_MTX_DIRTY as u32 != 0
}

/// `HSD_JObjSetMtxDirty` (macro).
#[inline(always)]
unsafe fn HSD_JObjSetMtxDirty(jobj: *mut HSD_JObj) {
    if !jobj.is_null() && !HSD_JObjMtxIsDirty(jobj) {
        ext::HSD_JObjSetMtxDirtySub(jobj);
    }
}

#[inline(always)]
unsafe fn HSD_JObjSetRotationX(jobj: *mut HSD_JObj, x: f32) {
    assert!(!jobj.is_null(), "HSD_ASSERT(639, jobj)");
    assert!((*jobj).flags & JOBJ_USE_QUATERNION as u32 == 0, "HSD_ASSERT(640, !(jobj->flags & JOBJ_USE_QUATERNION))");
    (*jobj).rotate.x = x;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
        HSD_JObjSetMtxDirty(jobj);
    }
}
#[inline(always)]
unsafe fn HSD_JObjSetRotationY(jobj: *mut HSD_JObj, y: f32) {
    assert!(!jobj.is_null(), "HSD_ASSERT(660, jobj)");
    assert!((*jobj).flags & JOBJ_USE_QUATERNION as u32 == 0, "HSD_ASSERT(661, !(jobj->flags & JOBJ_USE_QUATERNION))");
    (*jobj).rotate.y = y;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
        HSD_JObjSetMtxDirty(jobj);
    }
}
#[inline(always)]
unsafe fn HSD_JObjSetRotationZ(jobj: *mut HSD_JObj, z: f32) {
    assert!(!jobj.is_null(), "HSD_ASSERT(681, jobj)");
    assert!((*jobj).flags & JOBJ_USE_QUATERNION as u32 == 0, "HSD_ASSERT(682, !(jobj->flags & JOBJ_USE_QUATERNION))");
    (*jobj).rotate.z = z;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
        HSD_JObjSetMtxDirty(jobj);
    }
}

/// `sqrtf_accurate_store` / `sqrtf_accurate_sp18` / `sqrtf_accurate_local` (static inlines; the volatile slot is
/// only a store/load of the same f32): frsqrte plus four Newton steps in double. `3.0 - g*g*x` contracts to an fma.
#[inline(always)]
fn sqrtf_accurate(x: f32) -> f32 {
    if x > 0.0 {
        let xd = x as f64;
        let mut guess = melee_core::fp::frsqrte(xd);
        for _ in 0..4 {
            guess = 0.5 * guess * ((-(guess * guess)).mul_add(xd, 3.0));
        }
        (xd * guess) as f32
    } else {
        x
    }
}

pub extern "C" fn sqrtf_accurate_local(x: f32) -> f32 {
    sqrtf_accurate(x)
}

pub extern "C" fn sqrtf_accurate_sp18(x: f32) -> f32 {
    sqrtf_accurate(x)
}

pub unsafe extern "C" fn sqrtf_accurate_store(x: f32, y: *mut f32) -> f32 {
    if x > 0.0 {
        let v = sqrtf_accurate(x);
        *y = v;
        return v;
    }
    x
}

/// `product_xy`: `a->x * b->x + a->y * b->y` (one fused site).
#[inline(always)]
pub unsafe extern "C" fn product_xy(a: *mut Vec3, b: *mut Vec3) -> f32 {
    (*a).x.mul_add((*b).x, (*a).y * (*b).y)
}

#[inline(always)]
pub unsafe extern "C" fn return_sqrt_value(v: *mut Vec3) -> f32 {
    sqrtf_accurate(product_xy(v, v))
}

#[inline(always)]
fn sq_sum(x: f32, y: f32) -> f32 {
    x.mul_add(x, y * y)
}

pub unsafe extern "C" fn it_802759DC(item_gobj1: *mut HSD_GObj, item_gobj2: *mut HSD_GObj) {
    let mut sp44: Vec3 = core::mem::zeroed();
    let mut sp40: f32 = 0.0;
    let mut sp34: Vec3 = core::mem::zeroed();
    let mut sp1C: Vec3 = core::mem::zeroed();
    let mut chk = false;
    let item1 = item(item_gobj1);
    sp44 = (*item1).pos;
    let coll2 = ext::it_8026C100(item_gobj2);
    let coll1 = &raw mut (*item1).x378_itemColl;
    if !coll2.is_null() {
        if ext::mpLib_80054ED8((*coll2).floor.index) != 0 {
            let temp_r3_2 = ext::mpLib_8005199C_Floor(&raw mut sp44, -1, -1);
            if temp_r3_2 != -1 && ext::mpLinesConnected(temp_r3_2, (*coll2).floor.index) != 0 {
                (*coll1).floor.index = temp_r3_2;
                ext::mpLib_8004DD90_Floor(temp_r3_2, &raw mut sp44, &raw mut sp40, null_mut(), null_mut());
                if sp40 >= (*ext::it_804D6D28).xF0 {
                    (*item1).pos.x = sp44.x;
                    (*item1).pos.y = sp44.y + sp40;
                    (*item1).pos.z = sp44.z;
                    ext::mpColl_80043680(coll1, &raw mut (*item1).pos);
                    chk = true;
                    it_802762B0(item1);
                }
            }
        }
        if !chk {
            let temp_f31 = 0.5 * ((*coll2).ecb.top.y + (*coll2).ecb.bottom.y);
            ext::it_8026BC90(item_gobj2, &raw mut sp1C);
            let zero = 0.0f32;
            sp34.x = sp1C.x + zero;
            sp34.y = sp1C.y + temp_f31;
            sp34.z = sp1C.z + zero;
            (*coll1).last_pos = sp34;
            ext::mpColl_80043670(coll1);
            (*coll1).cur_pos = sp44;
            if ext::mpColl_800471F8(coll1) != 0 {
                it_802762B0(item1);
                (*item1).pos = (*coll1).cur_pos;
                return;
            }
            it_802762BC(item1);
            (*item1).pos = sp44;
        }
    }
}

pub unsafe extern "C" fn it_80275BC8(item_gobj: *mut HSD_GObj, arg_gobj: *mut HSD_GObj) {
    let ip = item(item_gobj);
    let mut sp14: Vec3 = core::mem::zeroed();
    let mut sp24: itECB = (*ip).xBFC;
    it_802762BC(ip);
    (*ip).x378_itemColl.cur_pos = (*ip).pos;
    if !arg_gobj.is_null() {
        match ext::it_80272D40(arg_gobj) {
            0 => ext::it_8026BB68(arg_gobj, &raw mut sp14),
            1 => ext::it_8026BB88(arg_gobj, &raw mut sp14),
            _ => panic!("couldn't get Owner_GObj_Kind!!"),
        }
    } else {
        sp14 = (*ip).pos;
    }
    let e8 = (*ext::it_804D6D28).xE8;
    sp24.top *= e8;
    sp24.bottom *= e8;
    sp24.right *= e8;
    sp24.left *= e8;
    (*ip).xC0C = (*ip).xC1C;
    if (*ip).xDCE_flag.anon_0.b7() == 1 {
        it_80275D5C(item_gobj, &raw mut sp24);
    }
    it_80276100(item_gobj, &raw mut sp14);
}

#[inline(always)]
unsafe fn set_ecb_source(item_gobj: *mut HSD_GObj, it: *mut Item, scale: f32) {
    ext::mpColl_SetECBSource_Fixed(&raw mut (*it).x378_itemColl, item_gobj, (*it).xC1C.top * scale, (*it).xC1C.bottom * scale,
                                   (*it).xC1C.right * scale, (*it).xC1C.left * scale);
    let int_dir = if -1.0 == (*it).facing_dir { -1 } else { 1 };
    ext::mpCollSetFacingDir(&raw mut (*it).x378_itemColl, int_dir);
}

pub unsafe extern "C" fn it_80275D5C(item_gobj: *mut HSD_GObj, arg_ecb: *mut itECB) {
    let it = item(item_gobj);
    let scale = (*it).scl;
    (*it).xC1C = *arg_ecb;
    set_ecb_source(item_gobj, it, scale);
}

pub unsafe extern "C" fn it_80275DFC(item_gobj: *mut HSD_GObj) {
    let it = (*item_gobj).user_data as *mut Item;
    let scale = (*it).scl;
    (*it).xC1C = (*it).xC0C;
    set_ecb_source(item_gobj, it, scale);
}

pub unsafe extern "C" fn it_80275E98(item_gobj: *mut HSD_GObj, spawn: *mut SpawnItem) {
    let item1 = (*item_gobj).user_data as *mut Item;
    let attr = (*item1).xCC_item_attr;
    let coll = &raw mut (*item1).x378_itemColl;
    (*item1).x378_itemColl.cur_pos = (*item1).pos;
    ext::mpColl_80041EE4(coll);
    let kind = (*item1).kind;
    let v: Option<u8> = if kind < It_Kind_L_Gun_Ray { Some(2) }
        else if kind < It_Kind_Kuriboh { Some(4) }
        else if kind < It_Kind_Octarock_Stone { Some(3) }
        else if kind < It_Kind_Mario_Fire { Some(4) }
        else if kind < It_Kind_Unk4 { Some(5) }
        else if kind < It_PKind_Random { Some(5) }
        else if kind < It_PKind_Start { Some(5) }
        else if kind < It_PKind_Terminate { Some(5) }
        else if kind < It_Kind_Old_Kuri { Some(4) }
        else if kind < It_Kind_Arwing_Laser { Some(3) }
        else if kind < 238 { Some(4) }
        else { None };
    if let Some(v) = v {
        (*coll).x34_flags.set_b1234(v);
    }
    let item2 = (*item_gobj).user_data as *mut Item;
    let scale = (*item2).scl;
    (*item2).xC1C = (*item1).xC1C;
    set_ecb_source(item_gobj, item2, scale);
    (*coll).x50 = (*attr).x1C_damage_mul;
    if (*spawn).x44_flag.anon_0.b0() == 1 {
        if (*spawn).x48_ground_or_air == GA_Air {
            it_80276100(item_gobj, &raw mut (*spawn).pos);
            return;
        }
        it_80276174(item_gobj, &raw mut (*spawn).pos);
        return;
    }
    (*coll).last_pos = (*spawn).pos;
    ext::mpColl_80043670(coll);
}

pub unsafe extern "C" fn it_80276100(item_gobj: *mut HSD_GObj, pos: *mut Vec3) {
    let it = item(item_gobj);
    let coll = &raw mut (*it).x378_itemColl;
    (*coll).last_pos = *pos;
    ext::mpColl_80043670(coll);
    ext::mpColl_800471F8(coll);
    (*it).pos = (*coll).cur_pos;
}

pub unsafe extern "C" fn it_80276174(item_gobj: *mut HSD_GObj, pos: *mut Vec3) {
    let mut temp_pos: Vec3 = core::mem::zeroed();
    let it = item(item_gobj);
    (*it).x378_itemColl.last_pos = *pos;
    ext::mpColl_80043670(&raw mut (*it).x378_itemColl);
    if ext::mpCheckMultiple((*it).pos.x, (*it).pos.y, (*it).pos.x, (*it).pos.y - (*ext::it_804D6D28).xF4, &raw mut temp_pos,
                            null_mut(), null_mut(), null_mut(), 1, -1, -1) != 0
    {
        (*it).pos = temp_pos;
    }
}

pub unsafe extern "C" fn it_80276214(item_gobj: *mut HSD_GObj) {
    let it = (*item_gobj).user_data as *mut Item;
    (*it).x378_itemColl.last_pos = (*it).x378_itemColl.cur_pos;
    (*it).x378_itemColl.cur_pos = (*it).pos;
    if (*it).xDCE_flag.anon_0.b7() == 1 {
        it_80276278(item_gobj);
    }
}

pub unsafe extern "C" fn it_80276278(item_gobj: *mut HSD_GObj) {
    let it = item(item_gobj);
    ext::mpColl_800436E4(&raw mut (*it).x378_itemColl, ext::it_80274990(item_gobj));
}

pub unsafe extern "C" fn it_802762B0(it: *mut Item) {
    (*it).ground_or_air = GA_Ground;
}

pub unsafe extern "C" fn it_802762BC(it: *mut Item) {
    (*it).ground_or_air = GA_Air;
    (*it).xDC8_word.flags.set_x1F(0);
}

const WALL_L: i32 = Collide_LeftWallMask;
const WALL_R: i32 = Collide_RightWallMask;
const CEIL: i32 = Collide_CeilingPush | Collide_CeilingHug;
const FLOOR: i32 = Collide_FloorPush | Collide_FloorHug;

pub unsafe extern "C" fn it_802762D8(item_gobj: *mut HSD_GObj) -> bool {
    let coll = &raw mut (*((*item_gobj).user_data as *mut Item)).x378_itemColl;
    let mut chk = false;
    if (*coll).env_flags & Collide_LeftEdge != 0 {
        chk = true;
    }
    if (*coll).env_flags & Collide_RightEdge != 0 {
        return true;
    }
    chk
}

pub unsafe extern "C" fn it_80276308(item_gobj: *mut HSD_GObj) -> i32 {
    let it = (*item_gobj).user_data as *mut Item;
    let mut ret_val = 0;
    let coll = &raw mut (*it).x378_itemColl;
    if (*coll).env_flags & WALL_L != 0 {
        ret_val = 8;
        (*it).xC30 = (*coll).left_facing_wall.index;
    }
    if (*coll).env_flags & WALL_R != 0 {
        (*it).xC30 = (*coll).right_facing_wall.index;
        ret_val = 4;
    }
    ret_val
}

pub unsafe extern "C" fn it_80276348(item_gobj: *mut HSD_GObj, vec: *mut Vec3) -> i32 {
    let it = (*item_gobj).user_data as *mut Item;
    let mut ret_val = 0;
    let coll = &raw mut (*it).x378_itemColl;
    if (*coll).env_flags & WALL_L != 0 {
        ret_val = 8;
        (*it).xC30 = (*coll).left_facing_wall.index;
        *vec = (*coll).left_facing_wall.normal;
    }
    if (*coll).env_flags & WALL_R != 0 {
        (*it).xC30 = (*coll).right_facing_wall.index;
        *vec = (*coll).right_facing_wall.normal;
        ret_val = 4;
    }
    ret_val
}

pub unsafe extern "C" fn it_802763B8(item_gobj: *mut HSD_GObj) -> bool {
    let it = (*item_gobj).user_data as *mut Item;
    let coll = &raw mut (*it).x378_itemColl;
    let mut chk = false;
    if (*coll).env_flags & FLOOR != 0 {
        (*it).xC30 = (*coll).floor.index;
        chk = true;
    }
    chk
}

pub unsafe extern "C" fn it_802763E0(item_gobj: *mut HSD_GObj) -> i32 {
    let mut result = 0;
    let it = item(item_gobj);
    let coll = &raw mut (*it).x378_itemColl;
    if (*coll).env_flags & CEIL != 0 {
        (*it).xC30 = (*coll).ceiling.index;
        result = 2;
    }
    result
}

pub unsafe extern "C" fn it_80276408(_item_gobj: *mut HSD_GObj, coll: *mut CollData, vec: *mut Vec3) {
    if (*coll).env_flags & WALL_L != 0 {
        *vec = (*coll).left_facing_wall.normal;
    }
    if (*coll).env_flags & WALL_R != 0 {
        *vec = (*coll).right_facing_wall.normal;
    }
    if (*coll).env_flags & CEIL != 0 {
        *vec = (*coll).ceiling.normal;
    }
    if (*coll).env_flags & FLOOR != 0 {
        *vec = (*coll).floor.normal;
    }
}

pub unsafe extern "C" fn it_8027649C(item_gobj: *mut HSD_GObj) -> f32 {
    let mut sp20: Vec3 = core::mem::zeroed();
    let mut sp14: Vec3 = core::mem::zeroed();
    let it = item(item_gobj);
    let coll = &raw mut (*it).x378_itemColl;
    sp14.z = 0.0;
    sp14.x = 0.0;
    sp14.y = 1.0;
    it_80276408(item_gobj, coll, &raw mut sp20);
    let angle = ext::lbVector_Angle(&raw mut sp20, &raw mut sp14);
    let int_dir: i32 = if sp20.x < 0.0 { -1 } else { 1 };
    (*it).facing_dir * (angle * int_dir as f32)
}

/// `it_8027649C_ref` (static inline).
pub unsafe extern "C" fn it_8027649C_ref(item_gobj: *mut HSD_GObj, sp14: *mut Vec3, sp20: *mut Vec3) -> f32 {
    let it = item(item_gobj);
    let coll = &raw mut (*it).x378_itemColl;
    (*sp14).z = 0.0;
    (*sp14).x = 0.0;
    (*sp14).y = 1.0;
    it_80276408(item_gobj, coll, sp20);
    let angle = ext::lbVector_Angle(sp20, sp14);
    let int_dir: i32 = if (*sp20).x < 0.0 { -1 } else { 1 };
    (*it).facing_dir * (angle * int_dir as f32)
}

pub unsafe extern "C" fn it_802765BC(item_gobj: *mut HSD_GObj, arg1: i32) {
    let mut v00: Vec3 = core::mem::zeroed();
    let mut v01: Vec3 = core::mem::zeroed();
    let ip = (*item_gobj).user_data as *mut Item;
    let jobj = ext::it_80272CC0(item_gobj, arg1);
    let bit_chk = (*ip).xDC8_word.flags.x17();
    if bit_chk == 0 {
        let mut angle = it_8027649C_ref(item_gobj, &raw mut v00, &raw mut v01);
        angle = -(*ip).facing_dir * angle;
        HSD_JObjSetRotationZ(jobj, angle);
    } else if bit_chk == 1 {
        let angle = it_8027649C_ref(item_gobj, &raw mut v00, &raw mut v01);
        HSD_JObjSetRotationX(jobj, angle);
    } else {
        let angle = it_8027649C_ref(item_gobj, &raw mut v00, &raw mut v01);
        HSD_JObjSetRotationY(jobj, angle);
    }
}

pub unsafe extern "C" fn it_80276934(item_gobj: *mut HSD_GObj, arg1: i32) {
    let mut sp80: Vec3 = core::mem::zeroed();
    let mut sp74: Vec3 = core::mem::zeroed();
    let item1 = (*item_gobj).user_data as *mut Item;
    let jobj = ext::it_80272CC0(item_gobj, arg1);
    let bit_chk = (*item1).xDC8_word.flags.x17();
    let item2 = (*item_gobj).user_data as *mut Item;
    sp80.z = 0.0;
    sp80.x = 0.0;
    sp80.y = 1.0;
    ext::mpLineGetNormal((*item2).xC30, &raw mut sp74);
    let angle = ext::lbVector_Angle(&raw mut sp74, &raw mut sp80);
    let dir: i32 = if sp74.x < 0.0 { -1 } else { 1 };
    let rot = angle * dir as f32;
    if bit_chk == 0 {
        let a1 = (*item2).facing_dir * rot;
        let r = -(*item1).facing_dir * a1;
        HSD_JObjSetRotationZ(jobj, r);
    } else if bit_chk == 1 {
        HSD_JObjSetRotationX(jobj, (*item2).facing_dir * rot);
    } else {
        HSD_JObjSetRotationY(jobj, (*item2).facing_dir * rot);
    }
}

pub unsafe extern "C" fn it_80276CB8(item_gobj: *mut HSD_GObj) {
    let it = (*item_gobj).user_data as *mut Item;
    if (*it).x378_itemColl.env_flags & FLOOR != 0 {
        it_802765BC(item_gobj, 0);
    }
}

pub unsafe extern "C" fn it_80276CEC(item_gobj: *mut HSD_GObj) {
    let it = (*item_gobj).user_data as *mut Item;
    let coll = &raw mut (*it).x378_itemColl;
    let mut coll_index: u32 = 0;
    if (*coll).env_flags & WALL_L != 0 {
        (*it).xAC_unk = (*coll).left_facing_wall.normal;
        coll_index = (*coll).left_facing_wall.index as u32;
    }
    if (*coll).env_flags & WALL_R != 0 {
        (*it).xAC_unk = (*coll).right_facing_wall.normal;
        coll_index = (*coll).right_facing_wall.index as u32;
    }
    if (*coll).env_flags & CEIL != 0 {
        (*it).xAC_unk = (*coll).ceiling.normal;
        coll_index = (*coll).ceiling.index as u32;
    }
    if (*coll).env_flags & FLOOR != 0 {
        (*it).xAC_unk = (*coll).floor.normal;
        coll_index = (*coll).floor.index as u32;
    }
    (*it).xC2C = coll_index as _;
}

pub unsafe extern "C" fn it_80276D9C(item_gobj: *mut HSD_GObj, arg1: u32) -> bool {
    let mut ret_val = true;
    let it = (*item_gobj).user_data as *mut Item;
    let coll = &raw mut (*it).x378_itemColl;
    let flags = (*coll).env_flags;
    let dir1 = if flags & WALL_L != 0 && flags & WALL_R != 0 { 1 } else { 0 };
    if dir1 != 0 {
        let mut sp34: itECB = (*it).xC1C;
        if (*coll).facing_dir == 1 {
            sp34.right = abs_c((*coll).ecb.right.x - 1.0);
            sp34.left = abs_c(1.0 + (*coll).ecb.left.x);
        } else {
            sp34.right = abs_c(1.0 + (*coll).ecb.left.x);
            sp34.left = abs_c((*coll).ecb.right.x - 1.0);
        }
        (*it).xC1C = sp34;
        let top = (*it).xC1C.top;
        let scale = 1.0f32;
        ext::mpColl_SetECBSource_Fixed(&raw mut (*it).x378_itemColl, item_gobj, top * scale, (*it).xC1C.bottom * scale,
                                       (*it).xC1C.right * scale, (*it).xC1C.left * scale);
        let dir2 = if -1.0 == (*it).facing_dir { -1 } else { 1 };
        ext::mpCollSetFacingDir(&raw mut (*it).x378_itemColl, dir2);
        return false;
    }
    if arg1 & 4 != 0 && (*coll).prev_env_flags & WALL_R != 0 {
        ret_val = false;
        (*it).pos.x += 1.5;
    }
    if arg1 & 8 != 0 && (*coll).prev_env_flags & WALL_L != 0 {
        ret_val = false;
        (*it).pos.x -= 1.5;
    }
    if arg1 & 2 != 0 && (*coll).prev_env_flags & CEIL != 0 {
        ret_val = false;
        (*it).pos.y -= 1.5;
    }
    if arg1 & 1 != 0 && (*coll).prev_env_flags & FLOOR != 0 {
        ret_val = false;
        (*it).pos.y += 1.5;
    }
    ret_val
}

pub unsafe extern "C" fn it_80276FC4(item_gobj: *mut HSD_GObj, arg1: i32) {
    let it = (*item_gobj).user_data as *mut Item;
    let attr = (*it).xCC_item_attr;
    it_8027781C(item_gobj);
    if it_80276D9C(item_gobj, arg1 as u32) {
        ext::it_8027321C(item_gobj);
        it_80277C40(item_gobj, arg1);
        ext::it_80275640(item_gobj, (*attr).x58);
    }
}

pub unsafe extern "C" fn it_80277040(item_gobj: *mut HSD_GObj) -> bool {
    let mut sp5C: Vec3 = core::mem::zeroed();
    let mut sp50: Vec3 = core::mem::zeroed();
    let mut sp44: Vec3 = core::mem::zeroed();
    let mut sp38: Vec3 = core::mem::zeroed();
    let mut sp20: Vec3 = core::mem::zeroed();
    let mut sp14: Vec3 = core::mem::zeroed();
    let mut ret_val = true;
    let item1 = (*item_gobj).user_data as *mut Item;
    let coll = &raw mut (*item1).x378_itemColl;
    if abs_c((*(*item1).xCC_item_attr).x50) < 0.00001 {
        (*item1).x88.y = 0.0;
        (*item1).x88.x = 0.0;
        return false;
    }
    sp20.z = 0.0;
    sp20.x = 0.0;
    sp20.y = 1.0;
    if (*coll).env_flags & WALL_L != 0 {
        sp5C = (*coll).left_facing_wall.normal;
    }
    if (*coll).env_flags & WALL_R != 0 {
        sp5C = (*coll).right_facing_wall.normal;
    }
    if (*coll).env_flags & CEIL != 0 {
        sp5C = (*coll).ceiling.normal;
    }
    if (*coll).env_flags & FLOOR != 0 {
        sp5C = (*coll).floor.normal;
    }
    (*item1).x94 = (*item1).x88;
    let mut angle1 = ext::lbVector_AngleXY(&raw mut sp5C, &raw mut sp20);
    if angle1 < 0.0 {
        angle1 = -angle1;
    }
    if angle1 >= (*ext::it_804D6D28).xC0 {
        sp50.z = 0.0;
        sp50.x = 0.0;
        sp50.y = -1.0;
        ext::lbVector_CrossprodNormalized(&raw mut sp5C, &raw mut sp50, &raw mut sp44);
        ext::lbVector_CrossprodNormalized(&raw mut sp44, &raw mut sp5C, &raw mut sp38);
        let temp_f3 = (*(*item1).xCC_item_attr).x50;
        let temp_f31 = (*item1).x94.x / temp_f3;
        let temp_f30 = (*item1).x94.y / temp_f3;
        (*item1).x88.x = sp38.x * temp_f3;
        (*item1).x88.y = sp38.y * (*(*item1).xCC_item_attr).x50;
        let item2 = (*item_gobj).user_data as *mut Item;
        it_80276408(item_gobj, &raw mut (*item2).x378_itemColl, &raw mut sp14);
        let mut angle2 = ext::lbVector_Angle(&raw mut sp14, &raw mut (*item2).xAC_unk);
        if angle2 < 0.0 {
            angle2 = -angle2;
        }
        let chk = angle2 >= (*ext::it_804D6D28).xC4 && (*item2).xC30 == (*item2).xC2C;
        if !chk {
            let z = 0.0f32;
            if (*item1).x94.x != z || (*item1).x94.y != z {
                let int_dir1 = if sp38.x < 0.0 { -1 } else { 1 };
                let int_dir2 = if temp_f31 < 0.0 { -1 } else { 1 };
                let mut block38 = false;
                if int_dir2 == int_dir1 {
                    let int_dir3 = if sp38.y < 0.0 { -1 } else { 1 };
                    let int_dir4 = if temp_f30 < 0.0 { -1 } else { 1 };
                    if int_dir4 != int_dir3 {
                        block38 = true;
                    }
                } else {
                    block38 = true;
                }
                if block38 {
                    (*item1).xD58 += 1;
                    if (*item1).xD58 >= (*ext::it_804D6D28).x38_float {
                        (*item1).xD58 = 0;
                        ret_val = false;
                        (*item1).x88.y = 0.0;
                        (*item1).x88.x = 0.0;
                    }
                }
            }
        } else {
            (*item1).xD5C = 2;
        }
    } else {
        ret_val = false;
        (*item1).x88.y = 0.0;
        (*item1).x88.x = 0.0;
    }
    ret_val
}

pub unsafe extern "C" fn it_8027737C(item_gobj: *mut HSD_GObj, pos: *mut Vec3) {
    let mut sp1C: Vec3 = core::mem::zeroed();
    let mut sqrt_0: f32 = 0.0;
    let it = item(item_gobj);
    if (*it).xDC8_word.flags.x1F() == 0 || (*it).xD5C == 0 {
        return;
    }
    ext::PSVECAdd(&raw mut (*it).x88, &raw mut (*it).x7C, &raw mut (*it).x7C);
    (*it).x7C.x *= (*(*it).xCC_item_attr).x54;
    (*it).x7C.y *= (*(*it).xCC_item_attr).x54;
    if sqrtf_accurate_store(sq_sum((*it).x7C.x, (*it).x7C.y), &raw mut sqrt_0) > (*ext::it_804D6D28).xC8 {
        ext::lbVector_NormalizeXY(&raw mut (*it).x7C);
        let temp_f1 = (*ext::it_804D6D28).xC8;
        (*it).x7C.x *= temp_f1;
        (*it).x7C.y *= temp_f1;
    }
    if sq_sum((*it).x7C.x, (*it).x7C.y) < (*ext::it_804D6D28).xCC {
        (*it).xDC8_word.flags.set_x1F(0);
        (*it).x7C.y = 0.0;
        (*it).x7C.x = 0.0;
        return;
    }
    it_802775F0(item_gobj, &raw mut (*it).x7C);
    ext::it_80274A64(item_gobj);
    ext::PSVECAdd(&raw mut (*it).x7C, pos, &raw mut sp1C);
    *pos = sp1C;
}

pub unsafe extern "C" fn it_80277544(item_gobj: *mut HSD_GObj) -> bool {
    let mut sp10: Vec3 = core::mem::zeroed();
    let mut ret_val = false;
    let it = item(item_gobj);
    if (*it).xD5C != 0 && it_80277040(item_gobj) {
        ext::PSVECAdd(&raw mut (*it).x40_vel, &raw mut (*it).x70_nudge, &raw mut sp10);
        if sq_sum(sp10.x, sp10.y) >= (*ext::it_804D6D28).xCC || (*it).xD5C == 2 {
            (*it).xDC8_word.flags.set_x1F(1);
            ret_val = true;
            (*it).xD5C = 1;
        }
    }
    ret_val
}

pub unsafe extern "C" fn it_802775F0(item_gobj: *mut HSD_GObj, arg1: *mut Vec3) {
    let it = item(item_gobj);
    let mut temp_sqrt = 0.0f32;
    if (*it).spin_spd != temp_sqrt {
        temp_sqrt = sqrtf_accurate(sq_sum((*arg1).x, (*arg1).y));
        (*it).xD3C_spinSpeed = 0.85 * (temp_sqrt / (*it).xC1C.bottom);
        if (if (*arg1).x < 0.0 { -1 } else { 1 }) != (if (*it).facing_dir < 0.0 { -1 } else { 1 }) {
            (*it).xD3C_spinSpeed = -(*it).xD3C_spinSpeed;
        }
        if (*it).xDC8_word.flags.x19() != 1 {
            (*it).xD3C_spinSpeed *= -(*it).facing_dir;
        }
    }
}

pub unsafe extern "C" fn it_8027770C(item_gobj: *mut HSD_GObj) -> bool {
    let mut sp38: Vec3 = core::mem::zeroed();
    let mut var_r5 = 0;
    let it = (*item_gobj).user_data as *mut Item;
    let mut ret_val = false;
    let coll = &raw mut (*it).x378_itemColl;
    let attr = (*it).xCC_item_attr;
    if (*coll).env_flags & WALL_L != 0 {
        var_r5 = 8;
        (*it).xC30 = (*coll).left_facing_wall.index;
        sp38 = (*coll).left_facing_wall.normal;
    }
    if (*coll).env_flags & WALL_R != 0 {
        var_r5 = 4;
        (*it).xC30 = (*coll).right_facing_wall.index;
        sp38 = (*coll).right_facing_wall.normal;
    }
    if var_r5 != 0 {
        let dot = (*it).x40_vel.z.mul_add(sp38.z, (*it).x40_vel.x.mul_add(sp38.x, (*it).x40_vel.y * sp38.y));
        if dot < 0.0 {
            ext::lbVector_Mirror(&raw mut (*it).x40_vel, &raw mut sp38);
        }
        let mult = (*attr).x58;
        ret_val = true;
        (*it).x40_vel.x *= mult;
        (*it).x40_vel.y *= mult;
        (*it).x40_vel.z *= mult;
    }
    ret_val
}

pub unsafe extern "C" fn it_8027781C(item_gobj: *mut HSD_GObj) -> bool {
    let mut sp48: Vec3 = core::mem::zeroed();
    let it = item(item_gobj);
    let attr = (*it).xCC_item_attr;
    let coll = &raw mut (*it).x378_itemColl;
    let mut sp3C: Vec3 = it_803B8570;
    let mut chk = false;
    let mut sp30: Vec3 = it_803B857C;

    let mut speed = return_sqrt_value(&raw mut (*it).x40_vel);
    if (*coll).env_flags & WALL_L != 0 && product_xy(&raw mut (*it).x40_vel, &raw mut (*coll).left_facing_wall.normal) < 0.0 {
        sp48 = (*it).x40_vel;
        ext::lbVector_Mirror(&raw mut sp48, &raw mut (*coll).left_facing_wall.normal);
        ext::lbVector_Add_xy(&raw mut sp30, &raw mut sp48);
        if ext::mpCollGetSpeedLeftWall(coll, &raw mut sp48) != 0 {
            ext::lbVector_Add_xy(&raw mut sp3C, &raw mut sp48);
        }
        chk = true;
    }
    if (*coll).env_flags & WALL_R != 0 && product_xy(&raw mut (*it).x40_vel, &raw mut (*coll).right_facing_wall.normal) < 0.0 {
        sp48 = (*it).x40_vel;
        ext::lbVector_Mirror(&raw mut sp48, &raw mut (*coll).right_facing_wall.normal);
        ext::lbVector_Add_xy(&raw mut sp30, &raw mut sp48);
        if ext::mpCollGetSpeedRightWall(coll, &raw mut sp48) != 0 {
            ext::lbVector_Add_xy(&raw mut sp3C, &raw mut sp48);
        }
        chk = true;
    }
    if (*coll).env_flags & CEIL != 0 && product_xy(&raw mut (*it).x40_vel, &raw mut (*coll).ceiling.normal) < 0.0 {
        sp48 = (*it).x40_vel;
        ext::lbVector_Mirror(&raw mut sp48, &raw mut (*coll).ceiling.normal);
        ext::lbVector_Add_xy(&raw mut sp30, &raw mut sp48);
        if ext::mpCollGetSpeedCeiling(coll, &raw mut sp48) != 0 {
            ext::lbVector_Add_xy(&raw mut sp3C, &raw mut sp48);
        }
        chk = true;
    }
    if (*coll).env_flags & FLOOR != 0 && product_xy(&raw mut (*it).x40_vel, &raw mut (*coll).floor.normal) < 0.0 {
        sp48 = (*it).x40_vel;
        ext::lbVector_Mirror(&raw mut sp48, &raw mut (*coll).floor.normal);
        ext::lbVector_Add_xy(&raw mut sp30, &raw mut sp48);
        if ext::mpCollGetSpeedFloor(coll, &raw mut sp48) != 0 {
            ext::lbVector_Add_xy(&raw mut sp3C, &raw mut sp48);
        }
        chk = true;
    }
    if chk {
        if return_sqrt_value(&raw mut sp30) < 0.01 {
            sp30.x = (*it).x40_vel.x;
            sp30.y = -1.0 * (*it).x40_vel.y;
        }
        ext::lbVector_NormalizeXY(&raw mut sp30);
        speed *= (*attr).x58;
        sp30.x *= speed;
        sp30.y *= speed;
        (*it).x64_vec_unk2 = sp3C;
        (*it).x40_vel = sp30;
        return true;
    }
    false
}

pub unsafe extern "C" fn it_80277C40(item_gobj: *mut HSD_GObj, arg1: i32) {
    let mut sp20: Vec3 = core::mem::zeroed();
    let mut sp14: Vec3 = core::mem::zeroed();
    let it = item(item_gobj);
    let coll = &raw mut (*it).x378_itemColl;
    if arg1 & 8 != 0 {
        sp20.x = (*coll).ecb.right.x;
        sp20.y = (*coll).ecb.right.y;
    }
    if arg1 & 4 != 0 {
        sp20.x = (*coll).ecb.left.x;
        sp20.y = (*coll).ecb.left.y;
    }
    if arg1 & 2 != 0 {
        sp20.x = (*coll).ecb.top.x;
        sp20.y = (*coll).ecb.top.y;
    }
    if arg1 & 1 != 0 {
        sp20.x = (*coll).ecb.bottom.x;
        sp20.y = (*coll).ecb.bottom.y;
    }
    if (*it).xDCF_flag.anon_0.b0() == 0 {
        ext::it_80278800(item_gobj, 0x405, 0, &raw mut sp20, &raw mut sp14, 0, 0.0);
    }
}
