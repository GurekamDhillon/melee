//! Port of src/melee/it/itgroundcoll.c: the item-vs-ground collision wrappers (each runs `it_80276214` and one
//! mpColl pass, copies the collision position back, then dispatches on the wall / ceiling / floor result bits) and
//! the landing helpers (`it_8026DDFC` throw/land counters, `it_8026DC24` rest check, `it_8026DD5C` landing).
//! `bool` is the MSL `int`, so results keep their full value where the C ORs them. Callees of other TUs go
//! through `ext`.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_mut, unused_assignments, clippy::missing_safety_doc)]

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
    fn HSD_Randi(a0: i32) -> i32;
    fn Item_8026A8EC(a0: *mut HSD_GObj);
    fn Item_8026ADC0(a0: *mut HSD_GObj);
    fn it_802725D4(a0: *mut HSD_GObj);
    fn it_80272F7C(a0: *mut HSD_JObj, a1: f32);
    fn it_802734B4(a0: *mut HSD_GObj);
    fn it_80274658(a0: *mut HSD_GObj, a1: f32) -> i32;
    fn it_80274740(a0: *mut HSD_GObj);
    fn it_80275DFC(a0: *mut HSD_GObj);
    fn it_80276214(a0: *mut HSD_GObj);
    fn it_802762B0(a0: *mut Item);
    fn it_802762BC(a0: *mut Item);
    fn it_802762D8(a0: *mut HSD_GObj) -> i32;
    fn it_80276308(a0: *mut HSD_GObj) -> i32;
    fn it_802763E0(a0: *mut HSD_GObj) -> i32;
    fn it_80276CEC(a0: *mut HSD_GObj);
    fn it_80276D9C(a0: *mut HSD_GObj, a1: i32) -> i32;
    fn it_80276FC4(a0: *mut HSD_GObj, a1: i32);
    fn it_80277040(a0: *mut HSD_GObj) -> i32;
    fn it_80277544(a0: *mut HSD_GObj) -> i32;
    fn mpCheckAllRemap(a0: *mut Vec, a1: *mut i32, a2: *mut u32, a3: *mut Vec, a4: i32, a5: i32, a6: f32, a7: f32, a8: f32, a9: f32) -> i32;
    fn mpColl_800471F8(a0: *mut CollData) -> i32;
    fn mpColl_800477E0(a0: *mut CollData) -> i32;
    fn mpColl_8004B108(a0: *mut CollData) -> i32;
    fn mpColl_8004B2DC(a0: *mut CollData) -> i32;
    fn mpLib_80054ED8(a0: i32) -> i32;
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
    pub static mut it_804D6D28: *mut ItemCommonData = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT

#[inline(always)]
unsafe fn item(gobj: *mut HSD_GObj) -> *mut Item {
    (*gobj).user_data as *mut Item
}

#[inline(always)]
fn abs_c(x: f32) -> f32 {
    if x < 0.0 { -x } else { x }
}

/// `itResetVelocity` (header inline)
unsafe fn itResetVelocity(ip: *mut Item) {
    (*ip).x40_vel.z = 0.0;
    (*ip).x40_vel.y = 0.0;
    (*ip).x40_vel.x = 0.0;
}

#[inline(always)]
unsafe fn ev(f: HSD_GObjEvent, gobj: *mut HSD_GObj) {
    (f.expect("HSD_GObjEvent is null"))(gobj)
}

pub unsafe extern "C" fn it_8026D564(item_gobj: *mut HSD_GObj) -> i32 {
    let ip = item(item_gobj);
    let coll = &raw mut (*ip).x378_itemColl;
    ext::it_80276214(item_gobj);
    let test = ext::mpColl_8004B108(coll);
    (*ip).pos = (*coll).cur_pos;
    if test != 0 {
        (*ip).xC30 = (*coll).floor.index;
    }
    test
}

pub unsafe extern "C" fn it_8026D5CC(item_gobj: *mut HSD_GObj) -> i32 {
    let ip = item(item_gobj);
    let coll = &raw mut (*ip).x378_itemColl;
    ext::it_80276214(item_gobj);
    ext::mpColl_8004B108(coll)
}

pub unsafe extern "C" fn it_8026D604(item_gobj: *mut HSD_GObj) -> i32 {
    let ip = item(item_gobj);
    ext::mpLib_80054ED8((*ip).xC30)
}

pub unsafe extern "C" fn it_8026D62C(gobj: *mut HSD_GObj, arg1: HSD_GObjEvent) {
    let ip = (*gobj).user_data as *mut Item;
    let coll = &raw mut (*ip).x378_itemColl;
    ext::it_80276214(gobj);
    let test = ext::mpColl_8004B108(coll);
    (*ip).pos = (*coll).cur_pos;
    if test != 0 {
        (*ip).xC30 = (*coll).floor.index;
    }
    if test == 0 {
        ext::it_802762BC(ip);
        ev(arg1, gobj);
        return;
    }
    if ext::it_80277544(gobj) != 0 && (*ip).xDCD_flag.anon_0.b3() == 0 {
        ext::Item_8026ADC0(gobj);
    }
}

pub unsafe extern "C" fn it_8026D6F4(gobj: *mut HSD_GObj, arg1: HSD_GObjEvent) {
    let ip = (*gobj).user_data as *mut Item;
    let coll = &raw mut (*ip).x378_itemColl;
    ext::it_80276214(gobj);
    let test = ext::mpColl_8004B108(coll);
    (*ip).pos = (*coll).cur_pos;
    if test != 0 {
        (*ip).xC30 = (*coll).floor.index;
    }
    if test == 0 {
        ev(arg1, gobj);
    }
}

pub unsafe extern "C" fn it_8026D78C(gobj: *mut HSD_GObj, arg1: HSD_GObjEvent) {
    let ip = (*gobj).user_data as *mut Item;
    let coll = &raw mut (*ip).x378_itemColl;
    ext::it_80276214(gobj);
    let test = ext::mpColl_8004B108(coll);
    (*ip).pos = (*coll).cur_pos;
    if test != 0 {
        (*ip).xC30 = (*coll).floor.index;
    }
    if test == 0 {
        ev(arg1, gobj);
        ext::it_802762BC(ip);
    }
}

pub unsafe extern "C" fn it_8026D82C(gobj: *mut HSD_GObj) {
    let ip = (*gobj).user_data as *mut Item;
    let coll = &raw mut (*ip).x378_itemColl;
    ext::it_80276214(gobj);
    let test = ext::mpColl_8004B108(coll);
    (*ip).pos = (*coll).cur_pos;
    if test != 0 {
        (*ip).xC30 = (*coll).floor.index;
    }
    if test == 0 {
        ext::it_802762BC(ip);
    }
}

pub unsafe extern "C" fn it_8026D8A4(gobj: *mut HSD_GObj, arg1: HSD_GObjEvent) -> i32 {
    ext::it_80276214(gobj);
    let test;
    {
        let ip = item(gobj);
        let coll = &raw mut (*ip).x378_itemColl;
        ext::it_80276214(gobj);
        test = ext::mpColl_8004B2DC(coll);
        (*ip).pos = (*coll).cur_pos;
        if test != 0 {
            (*ip).xC30 = (*coll).floor.index;
        }
    }
    if ext::it_802762D8(gobj) != 0 {
        ev(arg1, gobj);
    }
    test
}

pub unsafe extern "C" fn it_8026D938(gobj: *mut HSD_GObj, arg1: HSD_GObjEvent) {
    let ip = item(gobj);
    if ext::mpLib_80054ED8((*ip).xC30) == 0 {
        (*ip).xC30 = -1;
        ev(arg1, gobj);
    }
}

pub unsafe extern "C" fn it_8026D9A0(gobj: *mut HSD_GObj) -> i32 {
    let it = item(gobj);
    let coll_data = &raw mut (*it).x378_itemColl;
    ext::it_80276214(gobj);
    let temp = ext::mpColl_800477E0(coll_data);
    (*it).pos = (*coll_data).cur_pos;
    if temp != 0 {
        (*it).xC30 = (*coll_data).floor.index;
    }
    temp
}

pub unsafe extern "C" fn it_8026DA08(gobj: *mut HSD_GObj) -> i32 {
    let ip = (*gobj).user_data as *mut Item;
    let coll = &raw mut (*ip).x378_itemColl;
    ext::it_80276214(gobj);
    let test = ext::mpColl_800471F8(coll);
    (*ip).pos = (*coll).cur_pos;
    if test != 0 {
        (*ip).xC30 = (*coll).floor.index;
    }
    test
}

pub unsafe extern "C" fn it_8026DA70(gobj: *mut HSD_GObj) -> i32 {
    let it = item(gobj);
    let coll = &raw mut (*it).x378_itemColl;
    ext::it_80276214(gobj);
    ext::mpColl_800471F8(coll)
}

pub unsafe extern "C" fn it_8026DAA8(gobj: *mut HSD_GObj) -> i32 {
    let ip = item(gobj);
    let coll = &raw mut (*ip).x378_itemColl;
    ext::it_80276214(gobj);
    let mut test = ext::mpColl_800471F8(coll);
    (*ip).pos = (*coll).cur_pos;
    if test != 0 {
        (*ip).xC30 = (*coll).floor.index;
    }
    test |= ext::it_80276308(gobj);
    test |= ext::it_802763E0(gobj);
    test
}

pub unsafe extern "C" fn it_8026DB40(gobj: *mut HSD_GObj) -> i32 {
    let ip = item(gobj);
    let coll = &raw mut (*ip).x378_itemColl;
    ext::it_80276214(gobj);
    let mut test = ext::mpColl_800471F8(coll);
    (*ip).pos = (*coll).cur_pos;
    if test != 0 {
        (*ip).xC30 = (*coll).floor.index;
    }
    test |= ext::it_80276308(gobj);
    test
}

pub unsafe extern "C" fn it_8026DBC8(gobj: *mut HSD_GObj) -> i32 {
    if it_8026DDFC(gobj) != 0 && it_8026DC24(gobj) != 0 {
        ext::it_802725D4(gobj);
        return it_8026DD5C(gobj);
    }
    0
}

pub unsafe extern "C" fn it_8026DC24(gobj: *mut HSD_GObj) -> i32 {
    let ip = item(gobj);
    let attr = (*ip).xCC_item_attr;
    if (*ip).xD50_landNum <= 1 {
        ext::it_80274658(gobj, (*ext::it_804D6D28).x74_float);
        ext::it_80275DFC(gobj);
    }
    if abs_c((*ip).x40_vel.x) <= 0.00001 {
        (*ip).x40_vel.x = 0.0;
    }
    if abs_c((*ip).x40_vel.y) <= 0.00001 {
        (*ip).x40_vel.y = 0.0;
    }
    let mut rest = false;
    if abs_c((*ip).x40_vel.x) <= (*attr).x5c && abs_c((*ip).x40_vel.y) <= (*attr).x5c {
        rest = true;
    }
    if rest || (*ip).xDCD_flag.anon_0.b4() != 0 || (*attr).x58 == 0.0 {
        itResetVelocity(ip);
        return 1;
    }
    0
}

pub unsafe extern "C" fn it_8026DD5C(gobj: *mut HSD_GObj) -> i32 {
    let ip = item(gobj);
    (*ip).xD50_landNum = 0;
    ext::it_802762B0(ip);
    if ext::it_80277040(gobj) == 0 || (*ip).xDCD_flag.anon_0.b3() != 0 {
        ext::it_80274740(gobj);
        ext::it_80276CEC(gobj);
    } else {
        if !(*(*ip).xB8_itemLogicTable).entered_air.is_none() {
            ext::Item_8026ADC0(gobj);
            return 0;
        }
        ext::it_802734B4(gobj);
    }
    1
}

pub unsafe extern "C" fn it_8026DDFC(gobj: *mut HSD_GObj) -> i32 {
    let ip = item(gobj);
    (*ip).xD50_landNum += 1;
    if (*ip).xD50_landNum == 1 {
        if (*ip).xD54_throwNum != 0 {
            let tmp = (*ext::it_804D6D28).x48_byte;
            if (*ip).xD54_throwNum as u32 == ((tmp >> 4) & 0xF) as u32 || ext::HSD_Randi((tmp & 0xF) as i32) == 0 {
                (*ip).destroy_type = 1;
                ext::Item_8026A8EC(gobj);
                return 0;
            }
        }
    }
    1
}

pub unsafe extern "C" fn it_8026DE98(gobj: *mut HSD_GObj) -> i32 {
    it_8026DDFC(gobj)
}

pub unsafe extern "C" fn it_8026DF34(gobj: *mut HSD_GObj) -> i32 {
    let ip = item(gobj);
    let coll = &raw mut (*ip).x378_itemColl;
    ext::it_80276214(gobj);
    let cond = ext::mpColl_800471F8(coll);
    (*ip).pos = (*coll).cur_pos;
    if cond != 0 {
        (*ip).xC30 = (*coll).floor.index;
    }
    if cond != 0 {
        return 1;
    }
    0
}

pub unsafe extern "C" fn it_8026DFB0(gobj: *mut HSD_GObj) -> i32 {
    let ip = item(gobj);
    let coll = &raw mut (*ip).x378_itemColl;
    ext::it_80276214(gobj);
    let mut cond = ext::mpColl_800471F8(coll);
    (*ip).pos = (*coll).cur_pos;
    if cond != 0 {
        (*ip).xC30 = (*coll).floor.index;
    }
    cond |= ext::it_80276308(gobj);
    cond |= ext::it_802763E0(gobj);
    if cond & 0xF != 0 {
        return 1;
    }
    0
}

pub unsafe extern "C" fn it_8026E058(gobj: *mut HSD_GObj) -> i32 {
    let ip = item(gobj);
    let coll = &raw mut (*ip).x378_itemColl;
    ext::it_80276214(gobj);
    let mut cond = ext::mpColl_800471F8(coll);
    (*ip).pos = (*coll).cur_pos;
    if cond != 0 {
        (*ip).xC30 = (*coll).floor.index;
    }
    cond |= ext::it_80276308(gobj);
    if cond & 0xD != 0 {
        return 1;
    }
    0
}

pub unsafe extern "C" fn it_8026E0F4(gobj: *mut HSD_GObj) {
    let ip = item(gobj);
    let coll = &raw mut (*ip).x378_itemColl;
    ext::it_80276214(gobj);
    let cond = ext::mpColl_800477E0(coll);
    (*ip).pos = (*coll).cur_pos;
    if cond != 0 {
        (*ip).xC30 = (*coll).floor.index;
    }
}

/// `it_8026E_inline` (static inline)
pub unsafe extern "C" fn it_8026E_inline(gobj: *mut HSD_GObj) -> i32 {
    let ip = item(gobj);
    let coll = &raw mut (*ip).x378_itemColl;
    ext::it_80276214(gobj);
    let mut cond = ext::mpColl_800471F8(coll);
    (*ip).pos = (*coll).cur_pos;
    if cond != 0 {
        (*ip).xC30 = (*coll).floor.index;
    }
    let mut cond2 = cond;
    cond2 |= ext::it_80276308(gobj);
    cond2 |= ext::it_802763E0(gobj);
    cond = cond2;
    cond
}

pub unsafe extern "C" fn it_8026E15C_inline1(gobj: *mut HSD_GObj) -> i32 {
    it_8026DDFC(gobj)
}

pub unsafe extern "C" fn it_8026E15C_inline2(gobj: *mut HSD_GObj) -> i32 {
    it_8026DD5C(gobj)
}

pub unsafe extern "C" fn it_8026E15C(gobj: *mut HSD_GObj, arg1: HSD_GObjEvent) {
    let res = it_8026E_inline(gobj);
    if res & 0xF != 0 {
        ext::it_80276FC4(gobj, res);
        if res & 1 != 0 {
            let res2;
            if it_8026E15C_inline1(gobj) != 0 && it_8026DC24(gobj) != 0 {
                ext::it_802725D4(gobj);
                res2 = it_8026E15C_inline2(gobj);
            } else {
                res2 = 0;
            }
            if res2 != 0 {
                ev(arg1, gobj);
            }
        }
    }
}

pub unsafe extern "C" fn it_8026E248_inline(gobj: *mut HSD_GObj) -> i32 {
    it_8026DD5C(gobj)
}

/// `it_8026DE98_outline` .. `_outline4` (static): the same call through a chain of wrappers.
pub unsafe extern "C" fn it_8026DE98_outline(gobj: *mut HSD_GObj) -> i32 {
    it_8026DE98(gobj)
}
pub unsafe extern "C" fn it_8026DE98_outline2(gobj: *mut HSD_GObj) -> i32 {
    it_8026DE98_outline(gobj)
}
pub unsafe extern "C" fn it_8026DE98_outline3(gobj: *mut HSD_GObj) -> i32 {
    it_8026DE98_outline2(gobj)
}
pub unsafe extern "C" fn it_8026DE98_outline4(gobj: *mut HSD_GObj) -> i32 {
    it_8026DE98_outline3(gobj)
}

pub unsafe extern "C" fn it_8026E248(gobj: *mut HSD_GObj, arg1: HSD_GObjEvent) {
    let res = it_8026E_inline(gobj);
    if res & 0xF != 0 {
        ext::it_80276FC4(gobj, res);
        if res & 1 != 0 {
            let res2;
            if it_8026DE98_outline4(gobj) != 0 && it_8026DC24(gobj) != 0 {
                res2 = it_8026E248_inline(gobj);
            } else {
                res2 = 0;
            }
            if res2 != 0 {
                ev(arg1, gobj);
            }
        }
    }
}

pub unsafe extern "C" fn it_8026E32C_inline(gobj: *mut HSD_GObj) -> i32 {
    let ip = item(gobj);
    (*ip).xD50_landNum += 1;
    if it_8026DC24(gobj) != 0 {
        return it_8026DD5C(gobj);
    }
    0
}

pub unsafe extern "C" fn it_8026E32C(item_gobj: *mut HSD_GObj, arg1: HSD_GObjEvent) -> i32 {
    let res = it_8026E_inline(item_gobj);
    if res & 0xF != 0 {
        ext::it_80276FC4(item_gobj, res);
        if res & 1 != 0 {
            if it_8026E32C_inline(item_gobj) != 0 {
                ev(arg1, item_gobj);
            }
        }
    }
    res
}

pub unsafe extern "C" fn it_8026E414(item_gobj: *mut HSD_GObj, arg1: HSD_GObjEvent) {
    let it = (*item_gobj).user_data as *mut Item;
    let coll = &raw mut (*it).x378_itemColl;
    ext::it_80276214(item_gobj);
    let mut chk = ext::mpColl_800471F8(coll);
    (*it).pos = (*coll).cur_pos;
    if chk != 0 {
        (*it).xC30 = (*coll).floor.index;
    }
    chk |= ext::it_80276308(item_gobj);
    chk |= ext::it_802763E0(item_gobj);
    let new_var = chk;
    if new_var & 0xE != 0 {
        ext::it_80276FC4(item_gobj, new_var);
    }
    if new_var & 1 != 0 {
        ext::it_80275DFC(item_gobj);
        ext::it_802762B0(it);
        ev(arg1, item_gobj);
    }
}

pub unsafe extern "C" fn it_8026E4D0(item_gobj: *mut HSD_GObj, arg1: HSD_GObjEvent) {
    let it = (*item_gobj).user_data as *mut Item;
    let coll = &raw mut (*it).x378_itemColl;
    ext::it_80276214(item_gobj);
    let mut chk = ext::mpColl_800471F8(coll);
    (*it).pos = (*coll).cur_pos;
    if chk != 0 {
        (*it).xC30 = (*coll).floor.index;
    }
    chk |= ext::it_80276308(item_gobj);
    chk |= ext::it_802763E0(item_gobj);
    let new_var = chk;
    if new_var & 0xE != 0 {
        ext::it_80276FC4(item_gobj, new_var);
    }
    if new_var & 1 != 0 {
        let item_jobj = (*item_gobj).hsd_obj as *mut HSD_JObj;
        ext::it_80275DFC(item_gobj);
        ext::it_802762B0(it);
        ev(arg1, item_gobj);
        ext::it_80272F7C(item_jobj, (*(*it).xCC_item_attr).x60_scale);
    }
}

pub unsafe extern "C" fn it_8026E5A0(item_gobj: *mut HSD_GObj, arg1: HSD_GObjEvent) {
    let it = (*item_gobj).user_data as *mut Item;
    let coll = &raw mut (*it).x378_itemColl;
    ext::it_80276214(item_gobj);
    let mut chk = ext::mpColl_800471F8(coll);
    (*it).pos = (*coll).cur_pos;
    if chk != 0 {
        (*it).xC30 = (*coll).floor.index;
    }
    chk |= ext::it_80276308(item_gobj);
    chk |= ext::it_802763E0(item_gobj);
    let new_val = chk;
    if new_val & 0xF != 0 {
        if new_val & 0xE != 0 {
            ext::it_80276FC4(item_gobj, new_val);
        }
        if new_val & 1 != 0 {
            ext::it_80275DFC(item_gobj);
            ext::it_802762B0(it);
        }
        ev(arg1, item_gobj);
    }
}

pub unsafe extern "C" fn it_8026E664(item_gobj: *mut HSD_GObj, arg1: HSD_GObjEvent) {
    let it = (*item_gobj).user_data as *mut Item;
    let coll = &raw mut (*it).x378_itemColl;
    ext::it_80276214(item_gobj);
    let mut chk = ext::mpColl_800471F8(coll);
    (*it).pos = (*coll).cur_pos;
    if chk != 0 {
        (*it).xC30 = (*coll).floor.index;
    }
    chk |= ext::it_80276308(item_gobj);
    let new_var = chk;
    if new_var & 0xD != 0 {
        if new_var & 0xC != 0 {
            ext::it_80276FC4(item_gobj, new_var);
        }
        if new_var & 1 != 0 {
            ext::it_80275DFC(item_gobj);
            ext::it_802762B0(it);
        }
        ev(arg1, item_gobj);
    }
}

pub unsafe extern "C" fn it_8026E71C(item_gobj: *mut HSD_GObj, arg1: HSD_GObjEvent) {
    let it = (*item_gobj).user_data as *mut Item;
    let coll = &raw mut (*it).x378_itemColl;
    ext::it_80276214(item_gobj);
    let mut chk = ext::mpColl_800471F8(coll);
    (*it).pos = (*coll).cur_pos;
    if chk != 0 {
        (*it).xC30 = (*coll).floor.index;
    }
    chk |= ext::it_80276308(item_gobj);
    chk |= ext::it_802763E0(item_gobj);
    let new_var = chk;
    if new_var & 0xF != 0 {
        if new_var & 0xC != 0 {
            ext::it_80276D9C(item_gobj, new_var);
        }
        if new_var & 1 != 0 {
            ext::it_80275DFC(item_gobj);
            ext::it_802762B0(it);
        }
        ev(arg1, item_gobj);
    }
}

pub unsafe extern "C" fn it_8026E7E0(item_gobj: *mut HSD_GObj, arg1: HSD_GObjEvent) {
    let it = (*item_gobj).user_data as *mut Item;
    let coll = &raw mut (*it).x378_itemColl;
    ext::it_80276214(item_gobj);
    let mut chk = ext::mpColl_800471F8(coll);
    (*it).pos = (*coll).cur_pos;
    if chk != 0 {
        (*it).xC30 = (*coll).floor.index;
    }
    chk |= ext::it_80276308(item_gobj);
    chk |= ext::it_802763E0(item_gobj);
    let mut new_var = chk;
    if new_var & 0xC != 0 {
        ext::it_80276D9C(item_gobj, new_var);
    }
    if new_var & 1 != 0 {
        if it_8026DE98(item_gobj) != 0 && it_8026DC24(item_gobj) != 0 {
            new_var = it_8026DD5C(item_gobj);
        } else {
            new_var = 0;
        }
        if new_var != 0 {
            ev(arg1, item_gobj);
        }
    }
}

pub unsafe extern "C" fn it_8026E8C4(item_gobj: *mut HSD_GObj, arg1: HSD_GObjEvent, arg2: HSD_GObjEvent) {
    let it = (*item_gobj).user_data as *mut Item;
    let coll = &raw mut (*it).x378_itemColl;
    ext::it_80276214(item_gobj);
    let chk = ext::mpColl_8004B108(coll);
    (*it).pos = (*coll).cur_pos;
    if chk != 0 {
        (*it).xC30 = (*coll).floor.index;
    }
    if chk == 0 {
        ext::it_802762BC(it);
        ev(arg2, item_gobj);
        return;
    }
    ext::it_80277040(item_gobj);
    if ext::it_80276308(item_gobj) != 0 {
        (*it).xD5C = 0;
    }
    if ((*it).xDC8_word.flags.x1F() & 1) == 0 || (*it).xD5C == 0 {
        ext::it_80276CEC(item_gobj);
        ev(arg1, item_gobj);
    }
}

pub unsafe extern "C" fn it_8026E9A4(_gobj: *mut HSD_GObj, arg1: *mut Vec3, arg2: *mut Vec3, arg3: *mut Vec3) -> i32 {
    let mut p = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    if ext::mpCheckAllRemap(&raw mut p, null_mut(), null_mut(), arg3, -1, -1, (*arg1).x, (*arg1).y, (*arg2).x, (*arg2).y) == 1 {
        *arg2 = p;
        return 1;
    }
    0
}

pub unsafe extern "C" fn it_8026EA20(_gobj: *mut HSD_GObj, arg1: *mut Vec3, arg2: *mut Vec3, arg3: *mut Vec3, arg4: *mut i32, flags_out: *mut u32) -> i32 {
    let mut p = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    if ext::mpCheckAllRemap(&raw mut p, arg4, flags_out, arg3, -1, -1, (*arg1).x, (*arg1).y, (*arg2).x, (*arg2).y) == 1 {
        *arg2 = p;
        return 1;
    }
    0
}

pub unsafe extern "C" fn it_8026EA9C(_gobj: *mut HSD_GObj, arg1: *mut Vec3, arg2: *mut Vec3, arg3: *mut Vec3, arg4: *mut i32, flags_out: *mut u32, arg6: i32) -> i32 {
    let mut p = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    if ext::mpCheckAllRemap(&raw mut p, arg4, flags_out, arg3, -1, arg6, (*arg1).x, (*arg1).y, (*arg2).x, (*arg2).y) == 1 {
        *arg2 = p;
        return 1;
    }
    0
}
