//! Port of src/melee/it/iteffect.c: loading ItCo's data tables (`it_8027870C`) and spawning an item script's effect
//! (`it_80278800`: the effect id picks the spawn kind and which extra arguments the effect takes). Callees of other
//! TUs go through `ext`; `efSync_Spawn` / `efAsync_Spawn` are C varargs, so the port passes the extra arguments as
//! typed [`EfArg`]s.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, clippy::missing_safety_doc)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::itkinds7::bisect_value;
#[allow(unused_imports)]
use crate::layout::itEgg_ItemVars;
#[allow(unused_imports)]
use crate::layout::itLizardonAttributes;
#[allow(unused_imports)]
use crate::layout::itPeachTurnipAttributes;
#[allow(unused_imports)]
use crate::layout::itTaruAttributes;
#[allow(unused_imports)]
use crate::layout::it_2E5A_Attrs;
#[allow(unused_imports)]
use crate::layout::Item;
#[allow(unused_imports)]
use crate::layout::ItemCommonData;
#[allow(unused_imports)]
use crate::layout::ItemPickTable;
#[allow(unused_imports)]
use crate::layout::Vec as GcVec;
#[allow(unused_imports)]
use melee_core::fp::frsqrte;
#[allow(unused_imports)]
use melee_runtime::random::HsdRand;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_core::fp::fmadds;
use melee_types::*;

/// One extra argument of a variadic effect spawn (`efSync_Spawn` / `efAsync_Spawn` are C varargs; the port keeps the
/// C type of each extra argument so the eventual implementation can read it back).
#[derive(Clone, Copy)]
pub enum EfArg {
    Int(i32),
    Ptr(*mut c_void),
    F32P(*const f32),
    Vec3P(*const Vec3),
}

// BEGIN GENERATED EXT (rust/melee_items/gen/mtu/gen_tu.py; the callees of the C TU)
pub mod ext {
    #![allow(unreachable_code, non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    use core::ffi::c_void;
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? $(=> $fb:expr)?;)*) => {
            pub mod hook {
                use super::*;
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => { $( return $fb; )? melee_core::unported!(concat!("TODO: ", stringify!($name), " is not ported")) }
                }
            } )*
        };
    }
    ext_fns! {
    fn HSD_Randf() -> f32 => unsafe { melee_runtime::tu::random::global::HSD_Randf() };
    fn atan2f(a0: f32, a1: f32) -> f32 => unsafe { melee_runtime::lbtrigf::atan2f(a0, a1) };
    fn efAsync_MexSpawn(a0: *mut HSD_GObj, a1: *mut c_void, a2: i32, a3: *mut HSD_JObj, a4: *mut Vec, a5: f32, a6: f32);
    fn efAsync_Spawn(a0: *mut HSD_GObj, a1: *mut c_void, a2: u32, a3: u32, a4: *mut HSD_JObj, a5: &[EfArg]);
    fn efSync_Spawn(a0: i32, a1: *mut HSD_GObj, a2: &[EfArg]) -> *mut c_void;
    fn it_80272CC0(a0: *mut HSD_GObj, a1: i32) -> *mut HSD_JObj => unsafe { crate::itkinds11::it_80272cc0(a0, a1) };
    fn lbArchive_80017040(a0: *mut *mut HSD_Archive, a1: *const i8, a2: *mut c_void, a3: *const i8, a4: i32) -> i32;
    fn lbLang_IsSettingUS() -> i32 => unsafe { melee_runtime::tu::lblanguage::lbLang_IsSettingUS() as _ };
    fn lb_8000B1CC(a0: *mut HSD_JObj, a1: *mut Vec, a2: *mut Vec) => unsafe { melee_runtime::tu::lb_00B0::lb_8000B1CC(a0, a1, a2) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
    pub static mut it_803F1ED8: [i8; 16] = unsafe { core::mem::zeroed() };
    pub static mut it_803F1EE4: [i8; 16] = unsafe { core::mem::zeroed() };
    pub static mut it_803F1EF0: [i8; 16] = unsafe { core::mem::zeroed() };
    pub use crate::tu::it_3F14::it_804D6D04;
    pub use crate::tu::it_3F14::it_804D6D20;
    pub use crate::tu::it_3F14::it_804D6D24;
    pub use crate::tu::it_3F14::it_804D6D28;
    pub use crate::tu::it_3F14::it_804D6D30;
    pub use crate::tu::it_3F14::it_804D6D38;
    pub use crate::tu::it_3F14::it_804D6D40;
}
// END GENERATED EXT

pub unsafe extern "C" fn it_8027870C(_arg0: i32) {
    melee_core::ct!("it_8027870C");
    if ext::lbLang_IsSettingUS() != 0 {
        ext::lbArchive_80017040(null_mut(), ext::it_803F1EE4.as_ptr(), (&raw mut ext::it_804D6D20) as *mut c_void, ext::it_803F1EF0.as_ptr(), 0);
    } else {
        ext::lbArchive_80017040(null_mut(), ext::it_803F1ED8.as_ptr(), (&raw mut ext::it_804D6D20) as *mut c_void, ext::it_803F1EF0.as_ptr(), 0);
    }
    ext::it_804D6D28 = (*ext::it_804D6D20).x0;
    ext::it_804D6D24 = (*ext::it_804D6D20).x4;
    ext::it_804D6D38 = (*ext::it_804D6D20).x8;
    ext::it_804D6D30 = (*ext::it_804D6D20).xC;
    ext::it_804D6D40 = (*ext::it_804D6D20).x10;
    ext::it_804D6D04 = (*ext::it_804D6D20).x14;
}

pub unsafe extern "C" fn it_802787B4(item_gobj: *mut HSD_GObj, arg1: i32) {
    melee_core::ct!("it_802787B4");
    let mut sp1C = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut sp10 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    it_80278800(item_gobj, arg1, 0, &raw mut sp1C, &raw mut sp10, 1, 0.0);
}

/// `it_80278800_rand_vec` (static inline): `out += 2 * v * (rand - 0.5)` per axis; each `+=` is a fused multiply-add
/// in the shipping build.
pub unsafe extern "C" fn it_80278800_rand_vec(out: *mut Vec3, v: *mut Vec3) {
    melee_core::ct!("it_80278800_rand_vec");
    let mut f = ext::HSD_Randf();
    (*out).x = fmadds(2.0 * (*v).x, f - 0.5, (*out).x);
    f = ext::HSD_Randf();
    (*out).y = fmadds(2.0 * (*v).y, f - 0.5, (*out).y);
    f = ext::HSD_Randf();
    (*out).z = fmadds(2.0 * (*v).z, f - 0.5, (*out).z);
}

/// Where the id-range decision tree after the first `switch` ends (the C's `goto` labels).
#[derive(Clone, Copy, PartialEq)]
enum Blk {
    Ret,
    B4A8,
    B4FC,
    B558,
    B59C,
    B61C,
    B680,
    B6B4,
    B6E8,
}

/// The range ladder of `it_80278800` (default case, `ef_id >= 0x250`), transcribed branch for branch.
fn blk_of(ef_id: i32) -> Blk {
    use Blk::*;
    if ef_id == 0x41E {
        return Ret;
    } else if ef_id < 0x41E {
        if ef_id == 0x3FF {
            return B558;
        } else if ef_id < 0x3FF {
            if ef_id == 0x3F5 {
                return B4FC;
            } else if ef_id < 0x3F5 {
                if ef_id == 0x3EE {
                    return B4A8;
                } else if ef_id < 0x3EE {
                    if ef_id >= 0x3ED {
                        return B4FC;
                    } else if ef_id >= 0x3E9 {
                        return B4A8;
                    } else {
                        return Ret;
                    }
                }
                if ef_id >= 0x3F3 {
                    return B4A8;
                } else {
                    return B4FC;
                }
            } else {
                if ef_id == 0x3FD {
                    return B558;
                } else if ef_id >= 0x3FD {
                    return B4FC;
                }
                if ef_id >= 0x3FA {
                    return B4A8;
                } else if ef_id >= 0x3F7 {
                    return B558;
                } else {
                    return B4A8;
                }
            }
        }
        if ef_id == 0x408 {
            return B61C;
        } else if ef_id < 0x408 {
            if ef_id == 0x405 {
                return B4A8;
            } else if ef_id < 0x405 {
                if ef_id >= 0x404 {
                    return B61C;
                }
                if ef_id >= 0x402 {
                    return Ret;
                }
                return B4FC;
            }
            if ef_id >= 0x407 {
                return B4A8;
            } else {
                return B61C;
            }
        }
        if ef_id == 0x416 {
            return B4A8;
        } else if ef_id < 0x416 {
            if ef_id >= 0x412 {
                return Ret;
            }
            if ef_id >= 0x40C {
                return B4A8;
            } else {
                return Ret;
            }
        } else if ef_id >= 0x41C {
            return B4A8;
        } else {
            return Ret;
        }
    } else if ef_id == 0x440 {
        return B59C;
    } else if ef_id < 0x440 {
        if ef_id < 0x432 {
            if ef_id < 0x428 {
                if ef_id >= 0x425 {
                    return B4A8;
                }
                if ef_id >= 0x422 {
                    return Ret;
                }
                return B4A8;
            }
            if ef_id == 0x42D {
                return B4A8;
            } else if ef_id >= 0x42D {
                return B4FC;
            } else {
                return Ret;
            }
        }
        if ef_id < 0x438 {
            if ef_id == 0x434 {
                return B4A8;
            } else if ef_id >= 0x434 {
                return B59C;
            } else {
                return Ret;
            }
        }
        if ef_id >= 0x43D {
            return Ret;
        } else if ef_id >= 0x43A {
            return B4FC;
        } else {
            return Ret;
        }
    } else if ef_id == 0x4D0 {
        return B4A8;
    } else if ef_id < 0x4D0 {
        if ef_id < 0x446 {
            if ef_id == 0x443 {
                return B4FC;
            } else {
                return B4A8;
            }
        } else if ef_id >= 0x44D {
            return Ret;
        } else if ef_id >= 0x44A {
            return B4A8;
        } else {
            return Ret;
        }
    } else if ef_id == 0x514 {
        return B6B4;
    } else if ef_id < 0x514 {
        if ef_id >= 0x513 {
            return B680;
        } else {
            return Ret;
        }
    } else {
        if ef_id >= 0x516 {
            return Ret;
        }
        return B6E8;
    }
}

pub unsafe extern "C" fn it_80278800(item_gobj: *mut HSD_GObj, ef_id: i32, arg2: i32, arg3: *mut Vec3, arg4: *mut Vec3, arg5: i32, _arg6: f32) {
    melee_core::ct!("it_80278800");
    let item = (*item_gobj).user_data as *mut Item;
    let xbc0 = &raw mut (*item).xBC0 as *mut c_void;
    let sp88: f32;
    let sp84: f32;
    let sp80: f32;
    let mut sp74: Vec3;
    let mut sp68: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let sp64: f32;
    let sp60: f32;
    let sp5C: f32;
    let mut jobj: *mut HSD_JObj;

    match ef_id {
        0x402 | 0x403 | 0x409 | 0x412 | 0x413 | 0x414 | 0x41B | 0x422 | 0x433 | 0x449 | 0x473 => {
            if arg5 == 1 {
                jobj = ext::it_80272CC0(item_gobj, arg2);
                ext::efSync_Spawn(ef_id, item_gobj, &[EfArg::Ptr(jobj as *mut c_void)]);
                return;
            }
            jobj = ext::it_80272CC0(item_gobj, arg2);
            ext::efAsync_Spawn(item_gobj, xbc0, 0, ef_id as u32, jobj, &[]);
            return;
        }
        0x446 | 0x448 => {
            if arg5 == 1 {
                jobj = ext::it_80272CC0(item_gobj, arg2);
                ext::efSync_Spawn(ef_id, item_gobj, &[EfArg::Ptr(jobj as *mut c_void), EfArg::Vec3P(arg3)]);
                return;
            }
            jobj = ext::it_80272CC0(item_gobj, arg2);
            ext::efAsync_Spawn(item_gobj, xbc0, 7, ef_id as u32, jobj, &[EfArg::Vec3P(arg3)]);
            return;
        }
        0x417 | 0x418 | 0x419 | 0x41A => {
            jobj = ext::it_80272CC0(item_gobj, arg2);
            ext::efSync_Spawn(ef_id, item_gobj, &[EfArg::Ptr(jobj as *mut c_void), EfArg::Int(0xF25959)]);
            return;
        }
        0x415 | 0x41E | 0x428 | 0x429 | 0x42A | 0x438 | 0x439 | 0x43E | 0x44D | 0x45B | 0x45C | 0x45F | 0x460 | 0x461 | 0x462 | 0x463
        | 0x464 | 0x465 | 0x466 | 0x467 | 0x468 | 0x469 | 0x46A | 0x46B | 0x46C | 0x46D | 0x46E | 0x46F | 0x470 | 0x471 | 0x472 => {
            sp88 = 1.0;
            if arg5 == 1 {
                jobj = ext::it_80272CC0(item_gobj, arg2);
                ext::efSync_Spawn(ef_id, item_gobj, &[EfArg::Ptr(jobj as *mut c_void), EfArg::F32P(&sp88)]);
                return;
            }
            jobj = ext::it_80272CC0(item_gobj, arg2);
            ext::efAsync_Spawn(item_gobj, xbc0, 3, ef_id as u32, jobj, &[EfArg::F32P(&sp88)]);
            return;
        }
        0x44E | 0x44F | 0x450 | 0x451 | 0x452 | 0x457 | 0x458 | 0x459 | 0x45A => {
            sp84 = 1.0;
            jobj = ext::it_80272CC0(item_gobj, arg2);
            ext::efSync_Spawn(ef_id, item_gobj, &[EfArg::Ptr(jobj as *mut c_void), EfArg::F32P(&(*item).facing_dir), EfArg::F32P(&sp84)]);
            return;
        }
        0x423 | 0x424 => {
            sp80 = 0.0;
            jobj = ext::it_80272CC0(item_gobj, arg2);
            ext::efAsync_Spawn(item_gobj, xbc0, 3, ef_id as u32, jobj, &[EfArg::F32P(&sp80)]);
            return;
        }
        _ => {}
    }
    // default:
    sp74 = *arg3;
    it_80278800_rand_vec(&raw mut sp74, arg4);
    ext::lb_8000B1CC(ext::it_80272CC0(item_gobj, arg2), &raw mut sp74, &raw mut sp68);
    // TARGET_PC: an m-ex custom effect id (5000..8999) is one of the original owner's own effects
    if (5000..9000).contains(&ef_id) {
        let mut orientation = 0.0f32;
        if (*item).ground_or_air == GA_Ground {
            orientation = ext::atan2f(-(*item).x378_itemColl.floor.normal.x, (*item).x378_itemColl.floor.normal.y);
        }
        ext::efAsync_MexSpawn(item_gobj, xbc0, ef_id, ext::it_80272CC0(item_gobj, arg2), &raw mut sp74, (*item).facing_dir, orientation);
        return;
    }
    if ef_id < 0x250 {
        if arg5 == 1 {
            ext::efSync_Spawn(ef_id, item_gobj, &[EfArg::Vec3P(&sp68)]);
            return;
        }
        jobj = ext::it_80272CC0(item_gobj, arg2);
        ext::efAsync_Spawn(item_gobj, xbc0, 2, ef_id as u32, jobj, &[EfArg::Vec3P(&sp74)]);
        return;
    }
    match blk_of(ef_id) {
        Blk::Ret => {}
        Blk::B4A8 => {
            if arg5 == 1 {
                ext::efSync_Spawn(ef_id, item_gobj, &[EfArg::Vec3P(&sp68)]);
                return;
            }
            jobj = ext::it_80272CC0(item_gobj, arg2);
            ext::efAsync_Spawn(item_gobj, xbc0, 2, ef_id as u32, jobj, &[EfArg::Vec3P(&sp74)]);
        }
        Blk::B4FC => {
            if arg5 == 1 {
                ext::efSync_Spawn(ef_id, item_gobj, &[EfArg::Vec3P(&sp68), EfArg::F32P(&(*item).facing_dir)]);
                return;
            }
            jobj = ext::it_80272CC0(item_gobj, arg2);
            ext::efAsync_Spawn(item_gobj, xbc0, 5, ef_id as u32, jobj, &[EfArg::Vec3P(&sp74), EfArg::F32P(&(*item).facing_dir)]);
        }
        Blk::B558 => {
            sp64 = 0.0;
            jobj = ext::it_80272CC0(item_gobj, arg2);
            ext::efAsync_Spawn(
                item_gobj,
                xbc0,
                6,
                ef_id as u32,
                jobj,
                &[EfArg::Vec3P(&sp74), EfArg::F32P(&(*item).facing_dir), EfArg::F32P(&sp64)],
            );
        }
        Blk::B59C => {
            let var_f0: f64 = if (*item).facing_dir < 0.0 { core::f64::consts::PI } else { 0.0 };
            sp60 = var_f0 as f32;
            if arg5 == 1 {
                ext::efSync_Spawn(ef_id, item_gobj, &[EfArg::Vec3P(&sp68), EfArg::F32P(&sp60)]);
                return;
            }
            jobj = ext::it_80272CC0(item_gobj, arg2);
            ext::efAsync_Spawn(item_gobj, xbc0, 5, ef_id as u32, jobj, &[EfArg::Vec3P(&sp74), EfArg::F32P(&sp60)]);
        }
        Blk::B61C => {
            sp5C = 0.0;
            if arg5 == 1 {
                ext::efSync_Spawn(ef_id, item_gobj, &[EfArg::Vec3P(&sp68), EfArg::F32P(&sp5C)]);
                return;
            }
            jobj = ext::it_80272CC0(item_gobj, arg2);
            ext::efAsync_Spawn(item_gobj, xbc0, 5, ef_id as u32, jobj, &[EfArg::Vec3P(&sp74), EfArg::F32P(&sp5C)]);
        }
        Blk::B680 => {
            jobj = ext::it_80272CC0(item_gobj, arg2);
            ext::efAsync_Spawn(item_gobj, xbc0, 8, 2, jobj, &[EfArg::Vec3P(&sp74)]);
        }
        Blk::B6B4 => {
            jobj = ext::it_80272CC0(item_gobj, arg2);
            ext::efAsync_Spawn(item_gobj, xbc0, 8, 3, jobj, &[EfArg::Vec3P(&sp74)]);
        }
        Blk::B6E8 => {
            jobj = ext::it_80272CC0(item_gobj, arg2);
            ext::efAsync_Spawn(item_gobj, xbc0, 8, 4, jobj, &[EfArg::Vec3P(&sp74)]);
        }
    }
}

// ---- folded from itkinds9.rs (consolidation steps 21/22) ----
/// `it_80278800_rand_vec_fp` (iteffect.c): `out += 2 v (r - 0.5)` per axis, one draw each.
pub fn it_80278800_rand_vec_fp(rand: &mut HsdRand, out: &mut GcVec, v: &GcVec) {
    melee_core::ct!("it_80278800_rand_vec_fp");
    let f = rand.randf();
    out.x = fmadds(2.0 * v.x, f - 0.5, out.x);
    let f = rand.randf();
    out.y = fmadds(2.0 * v.y, f - 0.5, out.y);
    let f = rand.randf();
    out.z = fmadds(2.0 * v.z, f - 0.5, out.z);
}
