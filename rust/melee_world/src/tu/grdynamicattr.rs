//! Port of src/melee/gr/grdynamicattr.c: the four-slot pool of "dynamic attribute" discs (a
//! centre, a radius, a floor line id and a countdown) that stage code drops onto the map. Slot
//! 0..3 are linked into a free list (`804D6964`) and an active list (`804D6960`). Checked against
//! the C (whole TU, shipping contraction) by tests/tu_grdynamicattr.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use core::ptr::null_mut;
use melee_types::*;

// BEGIN GENERATED EXT (rust/melee_world/gen/mtu/gen_tu.py; the callees of the C TU)
pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    use core::ffi::c_void;
    macro_rules! ext_none {
        ($name:ident, ($($a:ident),*)) => { melee_core::unported!(concat!("TODO: ", stringify!($name), " is not ported")) };
        ($name:ident, ($($a:ident),*), $def:expr) => { ($def)($($a),*) };
    }
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? $(= $def:expr)?;)*) => {
            pub mod hook {
                use super::*;
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => ext_none!($name, ($($a),*) $(, $def)?),
                }
            } )*
        };
    }
    ext_fns! {
    fn mpLinesConnected(a0: i32, a1: i32) -> i32 = |a0: i32, a1: i32| unsafe { crate::tu::mplib::mpLinesConnected(a0 as _, a1 as _) as _ };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

/// `static const s32 GC_Id_None`
const GC_Id_None: i32 = -1;

/// `static grDynamicAttr_UnkStruct grDynamicAttr_8049EFA0[4]` (49EFA0).
pub static mut grDynamicAttr_8049EFA0: [grDynamicAttr_UnkStruct; 4] = unsafe { core::mem::zeroed() };
/// `static grDynamicAttr_UnkStruct* grDynamicAttr_804D6960` (4D6960): the active list.
pub static mut grDynamicAttr_804D6960: *mut grDynamicAttr_UnkStruct = null_mut();
/// `static grDynamicAttr_UnkStruct* grDynamicAttr_804D6964` (4D6964): the free list.
pub static mut grDynamicAttr_804D6964: *mut grDynamicAttr_UnkStruct = null_mut();

/// 1CA0B4
pub unsafe extern "C" fn grDynamicAttr_801CA0B4() {
    melee_core::ct!("grDynamicAttr_801CA0B4");
    let mut prev: *mut grDynamicAttr_UnkStruct = null_mut();
    grDynamicAttr_804D6960 = null_mut();
    grDynamicAttr_804D6964 = null_mut();
    for i in 0..4 {
        let slot = &raw mut grDynamicAttr_8049EFA0[i];
        (*slot).next = prev;
        grDynamicAttr_804D6964 = slot;
        prev = slot;
    }
}

/// 1CA0F8
pub unsafe extern "C" fn grDynamicAttr_801CA0F8(
    arg0: i32,
    v: *mut Vec3,
    floor_id: u32,
    f: f32,
    arg3: i32,
) -> *mut grDynamicAttr_UnkStruct {
    melee_core::ct!("grDynamicAttr_801CA0F8");
    // HSD_ASSERT(55, floor_id != GC_Id_None)
    if floor_id as i32 == GC_Id_None {
        panic!("grdynamicattr.c:55 floor_id!=GC_Id_None");
    }
    let tmp = grDynamicAttr_804D6964;
    if !tmp.is_null() {
        grDynamicAttr_804D6964 = (*tmp).next;
        (*tmp).next = grDynamicAttr_804D6960;
        grDynamicAttr_804D6960 = tmp;
        (*tmp).unk4 = arg0;
        (*tmp).unk8 = *v;
        (*tmp).unk14 = floor_id as i32;
        (*tmp).unk18 = f;
        (*tmp).unk1C = arg3;
        return tmp;
    }
    null_mut()
}

/// 1CA1C0
pub unsafe extern "C" fn grDynamicAttr_801CA1C0(arg: *mut grDynamicAttr_UnkStruct) {
    melee_core::ct!("grDynamicAttr_801CA1C0");
    if arg.is_null() {
        return;
    }
    let mut insert = false;
    if grDynamicAttr_804D6960 == arg {
        grDynamicAttr_804D6960 = (*arg).next;
        insert = true;
    } else {
        let mut cur = grDynamicAttr_804D6960;
        while !cur.is_null() {
            if (*cur).next == arg {
                (*cur).next = (*arg).next;
                break;
            }
            cur = (*cur).next;
        }
        if cur.is_null() {
            insert = true;
        }
    }
    if insert {
        (*arg).next = grDynamicAttr_804D6964;
        grDynamicAttr_804D6964 = arg;
    }
}

/// 1CA224
pub unsafe extern "C" fn grDynamicAttr_801CA224() {
    melee_core::ct!("grDynamicAttr_801CA224");
    let mut cur = grDynamicAttr_804D6960;
    while !cur.is_null() {
        let next = (*cur).next;
        if (*cur).unk1C > 0 {
            (*cur).unk1C -= 1;
            if (*cur).unk1C <= 0 {
                grDynamicAttr_801CA1C0(cur);
            }
        }
        cur = next;
    }
}

/// `static inline f32 do_sqrtf(f32)`: MSL's `sqrtf` (math_ppc.h, Newton-Raphson on `__frsqrte`).
#[inline]
pub fn do_sqrtf(x: f32) -> f32 {
    melee_core::ct!("do_sqrtf");
    melee_runtime::gekko::sqrtf(x)
}

/// 1CA284
pub unsafe extern "C" fn grDynamicAttr_801CA284(v: *mut Vec3, arg1: i32) -> i32 {
    melee_core::ct!("grDynamicAttr_801CA284");
    let mut cur = grDynamicAttr_804D6960;
    while !cur.is_null() {
        if ext::mpLinesConnected(arg1, (*cur).unk14) != 0 {
            let dx = (*v).x - (*cur).unk8.x;
            let dy = (*v).y - (*cur).unk8.y;
            let dx2 = dx * dx;
            let dy2 = dy * dy;
            if do_sqrtf(dx2 + dy2) < (*cur).unk18 {
                return (*cur).unk4;
            }
        }
        cur = (*cur).next;
    }
    0
}
