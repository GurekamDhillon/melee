//! Port of src/melee/gr/grfzerocar.c (the F-Zero car crowd: one jobj per car slot, each with up to
//! four car models hung on it). Checked against the C (whole TU, shipping contraction) by
//! tests/tu_grfzerocar.rs. Only float arithmetic: `scl.x *= ground_scale` (plain f32 products).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::inline::*;
use melee_types::*;

// BEGIN GENERATED EXT (rust/melee_world/gen/cref/gen_tu.py; the callees of the C TU)
pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    use core::ffi::c_void;
    macro_rules! ext_none {
        ($name:ident, ($($a:ident),*)) => { panic!(concat!("TODO: ", stringify!($name), " is not ported")) };
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
    fn Ground_801C0498() -> f32 = || unsafe { crate::tu::ground::Ground_801C0498() as _ };
    fn Ground_801C13D0(a0: i32, a1: i32) -> *mut HSD_JObj = |a0: i32, a1: i32| unsafe { crate::tu::ground::Ground_801C13D0(a0 as _, a1 as _) as _ };
    fn Ground_801C4100(a0: *mut HSD_JObj) -> *mut HSD_JObj = |a0: *mut HSD_JObj| unsafe { crate::tu::ground::Ground_801C4100(a0 as _) as _ };
    fn HSD_AObjSetRate(a0: *mut HSD_AObj, a1: f32) = |a0: *mut HSD_AObj, a1: f32| { unsafe { melee_runtime::tu::aobj::HSD_AObjSetRate(a0 as _, a1 as _); } };
    fn HSD_ForeachAnim(a0: *mut c_void, a1: i32, a2: u32, a3: *mut c_void, a4: i32, a5: f64);
    fn HSD_JObjAddChild(a0: *mut HSD_JObj, a1: *mut HSD_JObj) = |a0: *mut HSD_JObj, a1: *mut HSD_JObj| { unsafe { melee_runtime::tu::jobj_world::HSD_JObjAddChild(a0 as _, a1 as _); } };
    fn HSD_JObjAddNext(a0: *mut HSD_JObj, a1: *mut HSD_JObj) = |a0: *mut HSD_JObj, a1: *mut HSD_JObj| { unsafe { melee_runtime::tu::jobj_world::HSD_JObjAddNext(a0 as _, a1 as _); } };
    fn HSD_JObjAlloc() -> *mut HSD_JObj = || unsafe { melee_runtime::tu::jobj::HSD_JObjAlloc() as _ };
    fn HSD_JObjReqAnimAllByFlags(a0: *mut HSD_JObj, a1: u32, a2: f32);
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) = |a0: *mut HSD_JObj| { unsafe { melee_runtime::tu::jobj_world::HSD_JObjSetMtxDirtySub(a0 as _); } };
    fn PSMTXIdentity(a0: *mut c_void) = |a0: *mut c_void| { unsafe { melee_runtime::tu::mtx_pc::raw::PSMTXIdentity(a0 as _); } };
    fn grAnime_801C6C0C(a0: *mut HSD_JObj, a1: *mut HSD_AnimJoint, a2: *mut HSD_MatAnimJoint, a3: *mut HSD_ShapeAnimJoint) = |a0: *mut HSD_JObj, a1: *mut HSD_AnimJoint, a2: *mut HSD_MatAnimJoint, a3: *mut HSD_ShapeAnimJoint| { unsafe { crate::tu::granime::grAnime_801C6C0C(a0 as _, a1 as _, a2 as _, a3 as _); } };
    fn grDatFiles_801C6330(a0: i32) -> *mut UnkArchiveStruct = |a0: i32| unsafe { crate::tu::grdatfiles::grDatFiles_801C6330(a0 as _) as _ };
    fn lb_8000F9F8(a0: *mut HSD_JObj) = |a0: *mut HSD_JObj| { unsafe { melee_runtime::tu::lb_00F9::lb_8000F9F8(a0 as _); } };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_AObjSetRate(a0: *mut HSD_AObj, a1: f32) { super::HSD_AObjSetRate(a0, a1) }
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

struct Dirty;
impl JObjDirty for Dirty {
    fn set_mtx_dirty(&mut self, _j: &mut HSD_JObj) { unreachable!("HSD_JObjSetMtxDirty (out of line) is not called by this TU") }
    fn set_mtx_dirty_sub(&mut self, j: &mut HSD_JObj) { ext::HSD_JObjSetMtxDirtySub(j) }
}

/// `grFZeroCarEntry` (file-local in the C).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct grFZeroCarEntry {
    pub unk0: i16,
    pub unk2: i16,
    pub children: [i16; 4],
}

/// `struct grFZeroCar_RST` (file-local in the C).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct grFZeroCar_RST {
    pub scale: Vec3,
    pub translate: Vec3,
    pub rotate: Quaternion,
}

/// `grFZeroCar_803B7E50` (3B7E50)
pub static grFZeroCar_803B7E50: grFZeroCar_RST = grFZeroCar_RST {
    scale: Vec3 { x: 1.0, y: 1.0, z: 1.0 },
    translate: Vec3 { x: 0.0, y: 0.0, z: 0.0 },
    rotate: Quaternion { x: 0.0, y: 0.0, z: 0.0, w: 1.0 },
};

const fn e(unk2: i16, c: [i16; 4]) -> grFZeroCarEntry {
    grFZeroCarEntry { unk0: 0, unk2, children: c }
}

/// `grFZeroCar_803E0BD8` (3E0BD8)
pub static mut grFZeroCar_803E0BD8: [grFZeroCarEntry; 30] = [
    e(0, [1, 3, -1, -1]), e(1, [1, 3, 2, 4]),
    e(2, [1, 2, 3, -1]), e(3, [1, 2, -1, -1]),
    e(4, [1, 3, 2, 4]), e(5, [1, -1, -1, -1]),
    e(6, [1, 2, -1, -1]), e(7, [1, 2, -1, -1]),
    e(8, [1, 3, 2, 4]), e(9, [1, 2, -1, -1]),
    e(10, [1, 2, -1, -1]), e(11, [1, -1, -1, -1]),
    e(12, [1, 2, -1, -1]), e(13, [1, -1, -1, -1]),
    e(14, [1, 2, -1, -1]), e(15, [1, 2, 3, -1]),
    e(16, [1, 2, -1, -1]), e(17, [1, 2, -1, -1]),
    e(18, [2, 3, 1, -1]), e(19, [1, 2, -1, -1]),
    e(20, [1, 3, 2, 4]), e(21, [1, 2, 3, -1]),
    e(22, [1, -1, -1, -1]), e(23, [1, 2, 3, -1]),
    e(24, [1, 3, 2, 4]), e(25, [1, 2, 3, -1]),
    e(26, [1, 2, 3, -1]), e(27, [1, -1, -1, -1]),
    e(28, [1, -1, -1, -1]), e(29, [1, 3, 2, 4]),
];

/// `static char grFZeroCar_804D4598[8] = "archive"`
pub static grFZeroCar_804D4598: [u8; 8] = *b"archive\0";

/// `static inline setup_car_child`
pub unsafe fn setup_car_child(parent: *mut HSD_JObj, ext_count: i16, offset: i32, scale_factor: f32) {
    melee_core::ct!("setup_car_child");
    let archive = ext::grDatFiles_801C6330(ext_count as i32);
    assert!(!archive.is_null(), "archive"); // HSD_ASSERTMSG(95, archive, grFZeroCar_804D4598)

    let jobj = ext::Ground_801C13D0(ext_count as i32, 0);
    if !jobj.is_null() {
        // (u8*) offset + (uintptr_t) archive->unk4->unk8
        let entry = (offset as isize as usize).wrapping_add((*(*archive).unk4).unk8 as usize) as *mut DynamicModelDesc;
        if !(*entry).anims.is_null() {
            if !(*entry).matanims.is_null() {
                ext::grAnime_801C6C0C(jobj, *(*entry).anims, *(*entry).matanims, null_mut());
                ext::HSD_JObjReqAnimAllByFlags(jobj, 0x497, 0.0);
                ext::HSD_ForeachAnim(jobj as *mut c_void, JOBJ_TYPE as i32, 0x76a4, ext::cb::HSD_AObjSetRate as *mut c_void,
                                     AOBJ_ARG_AF as i32, 1.0);
            }
        }
        ext::HSD_JObjAddChild(parent, jobj);
    }

    HSD_JObjSetScaleX(&mut *parent, scale_factor, &mut Dirty);
    HSD_JObjSetScaleY(&mut *parent, scale_factor, &mut Dirty);
}

/// 1CAFBC
pub unsafe extern "C" fn grFZeroCar_801CAFBC(gobj: *mut HSD_GObj, data: *mut c_void, count: i32, mode: i32) {
    melee_core::ct!("grFZeroCar_801CAFBC");
    let root = (*gobj).hsd_obj as *mut HSD_JObj;
    let ground_scale = ext::Ground_801C0498();

    let scl_local = grFZeroCar_803B7E50.scale;
    HSD_JObjSetScale(&mut *root, &scl_local, &mut Dirty);

    let mut child = HSD_JObjGetChild(Some(&*root));

    if mode != 0 {
        ext::lb_8000F9F8(child);
    }

    let mut tra_jobj = ext::HSD_JObjAlloc();
    assert!(!tra_jobj.is_null(), "tra_jobj");
    ext::PSMTXIdentity((*tra_jobj).mtx.as_mut_ptr() as *mut c_void);
    (*tra_jobj).scl = null_mut();
    ext::HSD_JObjAddNext(child, tra_jobj);

    let data_ptr = data as *mut i16;

    for i in 0..30usize {
        if i != 0 {
            child = ext::Ground_801C13D0(*data_ptr.add(i) as i32, 0);
            if !child.is_null() {
                tra_jobj = ext::HSD_JObjAlloc();
                assert!(!tra_jobj.is_null(), "tra_jobj");
                ext::PSMTXIdentity((*tra_jobj).mtx.as_mut_ptr() as *mut c_void);
                (*tra_jobj).scl = null_mut();
                ext::HSD_JObjAddNext(child, tra_jobj);
                ext::HSD_JObjAddChild(root, tra_jobj);
                if mode != 0 {
                    ext::lb_8000F9F8(child);
                }
            } else {
                assert!(false, "0");
            }
        }

        let trans_local = grFZeroCar_803B7E50.translate;
        let rot_local = grFZeroCar_803B7E50.rotate;
        HSD_JObjSetTranslate(&mut *child, &trans_local, &mut Dirty);
        HSD_JObjSetRotation(&mut *child, &rot_local, &mut Dirty);

        {
            let mut scl = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
            HSD_JObjGetScale(&*tra_jobj, &mut scl);
            scl.x *= ground_scale;
            scl.y *= ground_scale;
            scl.z *= ground_scale;
            HSD_JObjSetScale(&mut *tra_jobj, &scl, &mut Dirty);
        }

        let ch = grFZeroCar_803E0BD8[i].children;
        let mut scale_factor = 1.0f32;
        let mut active = 0;

        if ch[0] != -1 {
            active = 1;
        }
        if ch[1] != -1 {
            active += 1;
        }
        if ch[2] != -1 {
            active += 1;
        }
        if ch[3] != -1 {
            active += 1;
        }

        if active == 1 {
            scale_factor = 1.1;
        } else if active == 2 {
            scale_factor = 1.0;
        } else if active == 3 {
            scale_factor = 1.0;
        } else if active == 4 {
            scale_factor = 0.9;
        }

        let mut cs: [*mut HSD_JObj; 4] = [null_mut(); 4];
        for (k, c) in cs.iter_mut().enumerate() {
            let mut idx = ch[k] as i32;
            if idx != -1 {
                let mut j = child;
                while !j.is_null() && idx != 0 {
                    idx -= 1;
                    j = ext::Ground_801C4100(j);
                }
                *c = j;
            }
        }

        for c in cs {
            if !c.is_null() {
                setup_car_child(c, count as i16, (count as i16 as i32).wrapping_mul(0x34), scale_factor);
            }
        }
    }
}
