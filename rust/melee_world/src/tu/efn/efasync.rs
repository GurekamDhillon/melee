//! Mechanical native port of src/melee/ef/efasync.c (async effect loading, the spawn dispatch and the deferred queue,
//! TARGET_PC variant), generated from the C by `gen/mtu/c2rs.py` (libclang AST: every
//! implicit conversion is the C one, `-ffp-contract=on` multiply-adds become `fmadds`). Verified against
//! the C compiled natively by the state oracle `tests/tu_ef.rs` (`python rust/melee_world/gen/mtu/mk_ef.py`).
//! Callees of other TUs are `ext` hooks named like the C (unset: panic "TODO", or the native port
//! `gen/rewire/` wired). `bool` is MSL `int`. The hand port `tu::eflib` (EfHost-free, ext stubs) predates this.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, dead_code, unused_assignments, unused_variables, unused_mut, unused_parens, unused_labels, unused_comparisons, clippy::all)]

use core::ffi::{c_char, c_void};
use core::ptr::null_mut;

use melee_core::fp::fmadds;
use melee_types::*;

pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
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
    fn Camera_RequestQuake(a0: CmQuakeKind, a1: *mut Vec) = |a0: CmQuakeKind, a1: *mut Vec| unsafe { crate::tu::camera::Camera_RequestQuake(a0 as _, a1 as _) };
    fn HSD_ArchiveGetPublicAddress(a0: *mut HSD_Archive, a1: *const i8) -> *mut c_void = |a0: *mut HSD_Archive, a1: *const i8| unsafe { melee_runtime::tu::archive::HSD_ArchiveGetPublicAddress(a0 as _, a1 as _) as _ };
    fn HSD_JObjAnimAll(a0: *mut HSD_JObj);
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) = |a0: *mut HSD_JObj| { unsafe { melee_runtime::tu::jobj_world::HSD_JObjSetMtxDirtySub(a0 as _); } };
    fn HSD_ObjAlloc(a0: *mut _HSD_ObjAllocData) -> *mut c_void = |a0: *mut _HSD_ObjAllocData| unsafe { melee_runtime::tu::objalloc::HSD_ObjAlloc(a0 as _) as _ };
    fn HSD_ObjAllocInit(a0: *mut _HSD_ObjAllocData, a1: u32, a2: u32) = |a0: *mut _HSD_ObjAllocData, a1: u32, a2: u32| { unsafe { melee_runtime::tu::objalloc::HSD_ObjAllocInit(a0 as _, a1 as _, a2 as _); } };
    fn HSD_ObjFree(a0: *mut _HSD_ObjAllocData, a1: *mut c_void) = |a0: *mut _HSD_ObjAllocData, a1: *mut c_void| { unsafe { melee_runtime::tu::objalloc::HSD_ObjFree(a0 as _, a1 as _); } };
    fn HSD_Randf() -> f32 = || melee_runtime::tu::random::HSD_Randf(melee_runtime::tu::globals::rand());
    fn HSD_Randi(a0: i32) -> i32 = |a0: i32| melee_runtime::tu::random::HSD_Randi(melee_runtime::tu::globals::rand(), a0);
    fn Mex_EffectCount() -> i32;
    fn Mex_EffectString(a0: i32, a1: i32) -> *const i8;
    fn efSync_Spawn(a0: i32, a1: *mut HSD_GObj, a2: &[*const c_void]) -> *mut c_void;
    fn ftKb_MexKirbyLike(a0: i32) -> i32;
    fn hsd_8039EFAC(a0: i32, a1: i32, a2: i32, a3: *mut HSD_JObj) -> *mut HSD_Generator;
    fn lbArchive_80017040(a0: *mut *mut HSD_Archive, a1: *const i8, a2: *mut c_void, a3: &[*const c_void]) -> i32;
    fn lbArchive_InitializeDAT(a0: *mut HSD_Archive, a1: *mut c_void, a2: u32);
    fn lbDvd_800178E8(a0: i32, a1: *const i8, a2: i32, a3: i32, a4: i32, a5: i32, a6: i32, a7: u8, a8: i32) = |a0: i32, a1: *const i8, a2: i32, a3: i32, a4: i32, a5: i32, a6: i32, a7: u8, a8: i32| { unsafe { melee_runtime::tu::lbdvd::lbDvd_800178E8(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _, a5 as _, a6 as _, a7 as _, a8 as _); } };
    fn lb_8000B1CC(a0: *mut HSD_JObj, a1: *mut Vec, a2: *mut Vec) = |a0: *mut HSD_JObj, a1: *mut Vec, a2: *mut Vec| { unsafe { melee_runtime::tu::lb_00B0::lb_8000B1CC(a0 as _, a1 as _, a2 as _); } };
    fn psInitDataBank(a0: i32, a1: *mut i32, a2: *mut i32, a3: *mut u32, a4: *mut i32);
    fn psInitDataBankLoad(a0: i32, a1: *const i32, a2: *const i32, a3: *const u32, a4: *const i32) = |a0: i32, a1: *const i32, a2: *const i32, a3: *const u32, a4: *const i32| { unsafe { melee_runtime::tu::particle::psInitDataBankLoad(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _); } };
    fn psInitDataBankLocate(a0: *mut HSD_Archive, a1: *mut HSD_Archive, a2: *mut i32);
    fn sprintf(a0: *mut i8, a1: *const i8, a2: &[*const c_void]) -> i32;
    fn OSReport(a0: &str) = |a0: &str| melee_runtime::tu::os_error::OSReport_str(a0);
    }
    pub use melee_runtime::tu::gobj_native::HSD_GObj_CurrentInvokedProc;
    pub use melee_runtime::shared::ftData_UnkBytePerCharacter;
}

static mut efAsync_MexMiss_seen: [i32; 256] = [0; 256];
static mut efAsync_MexMiss_nseen: i32 = 0;
static mut efAsync_MexResolve_buf: [i8; 128] = [0; 128];
#[repr(C)]
#[derive(Clone, Copy)]
pub struct EfMexBehavior {
    pub mdl_num: i32,
    pub mdl_type: *mut u8,
    pub ptcl_num: i32,
    pub ptcl_type: *mut u8,
}

melee_core::gc_layout!(EfMexBehavior, "EfMexBehavior", size 0x10, align 4, [
    mdl_num @ 0x0: 4 "s32", mdl_type @ 0x4: 4 "u8 *", ptcl_num @ 0x8: 4 "s32", ptcl_type @ 0xc: 4 "u8 *",
]);

pub static mut efAsync_AllocData: HSD_ObjAllocData = unsafe { core::mem::zeroed() };
pub static mut efAsync_803BFD68: [i8; 13] = [69, 102, 67, 111, 68, 97, 116, 97, 46, 100, 97, 116, 0];
pub static mut efAsync_803BFD78: [i8; 19] = [101, 102, 102, 67, 111, 109, 109, 111, 110, 68, 97, 116, 97, 84, 97, 98, 108, 101, 0];
pub static mut efAsync_803BFD8C: [i8; 13] = [69, 102, 77, 114, 68, 97, 116, 97, 46, 100, 97, 116, 0];
pub static mut efAsync_803BFD9C: [i8; 18] = [101, 102, 102, 77, 97, 114, 105, 111, 68, 97, 116, 97, 84, 97, 98, 108, 101, 0];
pub static mut efAsync_803BFDB0: [i8; 13] = [69, 102, 83, 115, 68, 97, 116, 97, 46, 100, 97, 116, 0];
pub static mut efAsync_803BFDC0: [i8; 18] = [101, 102, 102, 83, 97, 109, 117, 115, 68, 97, 116, 97, 84, 97, 98, 108, 101, 0];
pub static mut efAsync_803BFDD4: [i8; 13] = [69, 102, 70, 120, 68, 97, 116, 97, 46, 100, 97, 116, 0];
pub static mut efAsync_803BFDE4: [i8; 16] = [101, 102, 102, 70, 111, 120, 68, 97, 116, 97, 84, 97, 98, 108, 101, 0];
pub static mut efAsync_803BFDF4: [i8; 13] = [69, 102, 67, 97, 68, 97, 116, 97, 46, 100, 97, 116, 0];
pub static mut efAsync_803BFE04: [i8; 20] = [101, 102, 102, 67, 97, 112, 116, 97, 105, 110, 68, 97, 116, 97, 84, 97, 98, 108, 101, 0];
pub static mut efAsync_803BFE18: [i8; 13] = [69, 102, 75, 98, 68, 97, 116, 97, 46, 100, 97, 116, 0];
pub static mut efAsync_803BFE28: [i8; 18] = [101, 102, 102, 75, 105, 114, 98, 121, 68, 97, 116, 97, 84, 97, 98, 108, 101, 0];
pub static mut efAsync_803BFE3C: [i8; 13] = [69, 102, 76, 107, 68, 97, 116, 97, 46, 100, 97, 116, 0];
pub static mut efAsync_803BFE4C: [i8; 17] = [101, 102, 102, 76, 105, 110, 107, 68, 97, 116, 97, 84, 97, 98, 108, 101, 0];
pub static mut efAsync_803BFE60: [i8; 13] = [69, 102, 80, 107, 68, 97, 116, 97, 46, 100, 97, 116, 0];
pub static mut efAsync_803BFE70: [i8; 20] = [101, 102, 102, 80, 105, 107, 97, 99, 104, 117, 68, 97, 116, 97, 84, 97, 98, 108, 101, 0];
pub static mut efAsync_803BFE84: [i8; 13] = [69, 102, 68, 107, 68, 97, 116, 97, 46, 100, 97, 116, 0];
pub static mut efAsync_803BFE94: [i8; 19] = [101, 102, 102, 68, 111, 110, 107, 101, 121, 68, 97, 116, 97, 84, 97, 98, 108, 101, 0];
pub static mut efAsync_803BFEA8: [i8; 13] = [69, 102, 89, 115, 68, 97, 116, 97, 46, 100, 97, 116, 0];
pub static mut efAsync_803BFEB8: [i8; 18] = [101, 102, 102, 89, 111, 115, 104, 105, 68, 97, 116, 97, 84, 97, 98, 108, 101, 0];
pub static mut efAsync_803BFECC: [i8; 13] = [69, 102, 78, 115, 68, 97, 116, 97, 46, 100, 97, 116, 0];
pub static mut efAsync_803BFEDC: [i8; 17] = [101, 102, 102, 78, 101, 115, 115, 68, 97, 116, 97, 84, 97, 98, 108, 101, 0];
pub static mut efAsync_803BFEF0: [i8; 13] = [69, 102, 80, 114, 68, 97, 116, 97, 46, 100, 97, 116, 0];
pub static mut efAsync_803BFF00: [i8; 18] = [101, 102, 102, 80, 117, 114, 105, 110, 68, 97, 116, 97, 84, 97, 98, 108, 101, 0];
pub static mut efAsync_803BFF14: [i8; 13] = [69, 102, 75, 112, 68, 97, 116, 97, 46, 100, 97, 116, 0];
pub static mut efAsync_803BFF24: [i8; 18] = [101, 102, 102, 75, 111, 111, 112, 97, 68, 97, 116, 97, 84, 97, 98, 108, 101, 0];
pub static mut efAsync_803BFF38: [i8; 13] = [69, 102, 77, 116, 68, 97, 116, 97, 46, 100, 97, 116, 0];
pub static mut efAsync_803BFF48: [i8; 19] = [101, 102, 102, 77, 101, 119, 116, 119, 111, 68, 97, 116, 97, 84, 97, 98, 108, 101, 0];
pub static mut efAsync_803BFF5C: [i8; 13] = [69, 102, 73, 99, 68, 97, 116, 97, 46, 100, 97, 116, 0];
pub static mut efAsync_803BFF6C: [i8; 23] = [101, 102, 102, 73, 99, 101, 99, 108, 105, 109, 98, 101, 114, 68, 97, 116, 97, 84, 97, 98, 108, 101, 0];
pub static mut efAsync_803BFF84: [i8; 13] = [69, 102, 80, 101, 68, 97, 116, 97, 46, 100, 97, 116, 0];
pub static mut efAsync_803BFF94: [i8; 18] = [101, 102, 102, 80, 101, 97, 99, 104, 68, 97, 116, 97, 84, 97, 98, 108, 101, 0];
pub static mut efAsync_803BFFA8: [i8; 13] = [69, 102, 77, 115, 68, 97, 116, 97, 46, 100, 97, 116, 0];
pub static mut efAsync_803BFFB8: [i8; 17] = [101, 102, 102, 77, 97, 114, 115, 68, 97, 116, 97, 84, 97, 98, 108, 101, 0];
pub static mut efAsync_803BFFCC: [i8; 13] = [69, 102, 90, 100, 68, 97, 116, 97, 46, 100, 97, 116, 0];
pub static mut efAsync_803BFFDC: [i8; 18] = [101, 102, 102, 90, 101, 108, 100, 97, 68, 97, 116, 97, 84, 97, 98, 108, 101, 0];
pub static mut efAsync_803BFFF0: [i8; 13] = [69, 102, 76, 103, 68, 97, 116, 97, 46, 100, 97, 116, 0];
pub static mut efAsync_803C0000: [i8; 18] = [101, 102, 102, 76, 117, 105, 103, 105, 68, 97, 116, 97, 84, 97, 98, 108, 101, 0];
pub static mut efAsync_803C0014: [i8; 13] = [69, 102, 71, 110, 68, 97, 116, 97, 46, 100, 97, 116, 0];
pub static mut efAsync_803C0024: [i8; 18] = [101, 102, 102, 71, 97, 110, 111, 110, 68, 97, 116, 97, 84, 97, 98, 108, 101, 0];
pub static mut efAsync_803C0038: [i8; 11] = [69, 102, 75, 98, 77, 115, 46, 100, 97, 116, 0];
pub static mut efAsync_803C0044: [i8; 22] = [101, 102, 102, 75, 105, 114, 98, 121, 77, 97, 114, 115, 68, 97, 116, 97, 84, 97, 98, 108, 101, 0];
pub static mut efAsync_803C005C: [i8; 11] = [69, 102, 75, 98, 90, 100, 46, 100, 97, 116, 0];
pub static mut efAsync_803C0068: [i8; 23] = [101, 102, 102, 75, 105, 114, 98, 121, 90, 101, 108, 100, 97, 68, 97, 116, 97, 84, 97, 98, 108, 101, 0];
pub static mut efAsync_803C0080: [i8; 13] = [69, 102, 77, 110, 68, 97, 116, 97, 46, 100, 97, 116, 0];
pub static mut efAsync_803C0090: [i8; 17] = [101, 102, 102, 77, 101, 110, 117, 68, 97, 116, 97, 84, 97, 98, 108, 101, 0];
pub static mut efAsync_803C00A4: [i8; 11] = [69, 102, 75, 98, 77, 114, 46, 100, 97, 116, 0];
pub static mut efAsync_803C00B0: [i8; 23] = [101, 102, 102, 75, 105, 114, 98, 121, 77, 97, 114, 105, 111, 68, 97, 116, 97, 84, 97, 98, 108, 101, 0];
pub static mut efAsync_803C00C8: [i8; 11] = [69, 102, 75, 98, 70, 120, 46, 100, 97, 116, 0];
pub static mut efAsync_803C00D4: [i8; 21] = [101, 102, 102, 75, 105, 114, 98, 121, 70, 111, 120, 68, 97, 116, 97, 84, 97, 98, 108, 101, 0];
pub static mut efAsync_803C00EC: [i8; 11] = [69, 102, 75, 98, 83, 115, 46, 100, 97, 116, 0];
pub static mut efAsync_803C00F8: [i8; 23] = [101, 102, 102, 75, 105, 114, 98, 121, 83, 97, 109, 117, 115, 68, 97, 116, 97, 84, 97, 98, 108, 101, 0];
pub static mut efAsync_803C0110: [i8; 11] = [69, 102, 75, 98, 80, 107, 46, 100, 97, 116, 0];
pub static mut efAsync_803C011C: [i8; 25] = [101, 102, 102, 75, 105, 114, 98, 121, 80, 105, 107, 97, 99, 104, 117, 68, 97, 116, 97, 84, 97, 98, 108, 101, 0];
pub static mut efAsync_803C0138: [i8; 11] = [69, 102, 75, 98, 76, 103, 46, 100, 97, 116, 0];
pub static mut efAsync_803C0144: [i8; 23] = [101, 102, 102, 75, 105, 114, 98, 121, 76, 117, 105, 103, 105, 68, 97, 116, 97, 84, 97, 98, 108, 101, 0];
pub static mut efAsync_803C015C: [i8; 11] = [69, 102, 75, 98, 67, 97, 46, 100, 97, 116, 0];
pub static mut efAsync_803C0168: [i8; 25] = [101, 102, 102, 75, 105, 114, 98, 121, 67, 97, 112, 116, 97, 105, 110, 68, 97, 116, 97, 84, 97, 98, 108, 101, 0];
pub static mut efAsync_803C0184: [i8; 11] = [69, 102, 75, 98, 68, 107, 46, 100, 97, 116, 0];
pub static mut efAsync_803C0190: [i8; 24] = [101, 102, 102, 75, 105, 114, 98, 121, 68, 111, 110, 107, 101, 121, 68, 97, 116, 97, 84, 97, 98, 108, 101, 0];
pub static mut efAsync_803C01A8: [i8; 11] = [69, 102, 75, 98, 75, 112, 46, 100, 97, 116, 0];
pub static mut efAsync_803C01B4: [i8; 23] = [101, 102, 102, 75, 105, 114, 98, 121, 75, 111, 111, 112, 97, 68, 97, 116, 97, 84, 97, 98, 108, 101, 0];
pub static mut efAsync_803C01CC: [i8; 11] = [69, 102, 75, 98, 73, 99, 46, 100, 97, 116, 0];
pub static mut efAsync_803C01D8: [i8; 21] = [101, 102, 102, 75, 105, 114, 98, 121, 73, 99, 101, 68, 97, 116, 97, 84, 97, 98, 108, 101, 0];
pub static mut efAsync_803C01F0: [i8; 11] = [69, 102, 75, 98, 71, 110, 46, 100, 97, 116, 0];
pub static mut efAsync_803C01FC: [i8; 23] = [101, 102, 102, 75, 105, 114, 98, 121, 71, 97, 110, 111, 110, 68, 97, 116, 97, 84, 97, 98, 108, 101, 0];
pub static mut efAsync_803C0214: [i8; 11] = [69, 102, 75, 98, 70, 101, 46, 100, 97, 116, 0];
pub static mut efAsync_803C0220: [i8; 24] = [101, 102, 102, 75, 105, 114, 98, 121, 69, 109, 98, 108, 101, 109, 68, 97, 116, 97, 84, 97, 98, 108, 101, 0];
pub static mut efAsync_803C0238: [i8; 13] = [69, 102, 70, 101, 68, 97, 116, 97, 46, 100, 97, 116, 0];
pub static mut efAsync_803C0248: [i8; 19] = [101, 102, 102, 69, 109, 98, 108, 101, 109, 68, 97, 116, 97, 84, 97, 98, 108, 101, 0];
pub static mut efAsync_DatEntries: [EF_DAT_Entry; 65] = [EF_DAT_Entry { ef_DAT_file: core::ptr::addr_of_mut!(efAsync_803BFD68) as *mut i8, effDataTable_name: core::ptr::addr_of_mut!(efAsync_803BFD78) as *mut i8, data: null_mut() }, EF_DAT_Entry { ef_DAT_file: core::ptr::addr_of_mut!(efAsync_803BFD8C) as *mut i8, effDataTable_name: core::ptr::addr_of_mut!(efAsync_803BFD9C) as *mut i8, data: null_mut() }, EF_DAT_Entry { ef_DAT_file: core::ptr::addr_of_mut!(efAsync_803BFDB0) as *mut i8, effDataTable_name: core::ptr::addr_of_mut!(efAsync_803BFDC0) as *mut i8, data: null_mut() }, EF_DAT_Entry { ef_DAT_file: core::ptr::addr_of_mut!(efAsync_803BFDD4) as *mut i8, effDataTable_name: core::ptr::addr_of_mut!(efAsync_803BFDE4) as *mut i8, data: null_mut() }, EF_DAT_Entry { ef_DAT_file: core::ptr::addr_of_mut!(efAsync_803BFDF4) as *mut i8, effDataTable_name: core::ptr::addr_of_mut!(efAsync_803BFE04) as *mut i8, data: null_mut() }, EF_DAT_Entry { ef_DAT_file: core::ptr::addr_of_mut!(efAsync_803BFE18) as *mut i8, effDataTable_name: core::ptr::addr_of_mut!(efAsync_803BFE28) as *mut i8, data: null_mut() }, EF_DAT_Entry { ef_DAT_file: core::ptr::addr_of_mut!(efAsync_803BFE3C) as *mut i8, effDataTable_name: core::ptr::addr_of_mut!(efAsync_803BFE4C) as *mut i8, data: null_mut() }, EF_DAT_Entry { ef_DAT_file: core::ptr::addr_of_mut!(efAsync_803BFE60) as *mut i8, effDataTable_name: core::ptr::addr_of_mut!(efAsync_803BFE70) as *mut i8, data: null_mut() }, EF_DAT_Entry { ef_DAT_file: core::ptr::addr_of_mut!(efAsync_803BFE84) as *mut i8, effDataTable_name: core::ptr::addr_of_mut!(efAsync_803BFE94) as *mut i8, data: null_mut() }, EF_DAT_Entry { ef_DAT_file: core::ptr::addr_of_mut!(efAsync_803BFEA8) as *mut i8, effDataTable_name: core::ptr::addr_of_mut!(efAsync_803BFEB8) as *mut i8, data: null_mut() }, EF_DAT_Entry { ef_DAT_file: core::ptr::addr_of_mut!(efAsync_803BFECC) as *mut i8, effDataTable_name: core::ptr::addr_of_mut!(efAsync_803BFEDC) as *mut i8, data: null_mut() }, EF_DAT_Entry { ef_DAT_file: core::ptr::addr_of_mut!(efAsync_803BFEF0) as *mut i8, effDataTable_name: core::ptr::addr_of_mut!(efAsync_803BFF00) as *mut i8, data: null_mut() }, EF_DAT_Entry { ef_DAT_file: core::ptr::addr_of_mut!(efAsync_803BFF14) as *mut i8, effDataTable_name: core::ptr::addr_of_mut!(efAsync_803BFF24) as *mut i8, data: null_mut() }, EF_DAT_Entry { ef_DAT_file: core::ptr::addr_of_mut!(efAsync_803BFF38) as *mut i8, effDataTable_name: core::ptr::addr_of_mut!(efAsync_803BFF48) as *mut i8, data: null_mut() }, EF_DAT_Entry { ef_DAT_file: core::ptr::addr_of_mut!(efAsync_803BFF5C) as *mut i8, effDataTable_name: core::ptr::addr_of_mut!(efAsync_803BFF6C) as *mut i8, data: null_mut() }, EF_DAT_Entry { ef_DAT_file: core::ptr::addr_of_mut!(efAsync_803BFF84) as *mut i8, effDataTable_name: core::ptr::addr_of_mut!(efAsync_803BFF94) as *mut i8, data: null_mut() }, EF_DAT_Entry { ef_DAT_file: core::ptr::addr_of_mut!(efAsync_803BFFA8) as *mut i8, effDataTable_name: core::ptr::addr_of_mut!(efAsync_803BFFB8) as *mut i8, data: null_mut() }, EF_DAT_Entry { ef_DAT_file: core::ptr::addr_of_mut!(efAsync_803BFFCC) as *mut i8, effDataTable_name: core::ptr::addr_of_mut!(efAsync_803BFFDC) as *mut i8, data: null_mut() }, EF_DAT_Entry { ef_DAT_file: core::ptr::addr_of_mut!(efAsync_803BFFF0) as *mut i8, effDataTable_name: core::ptr::addr_of_mut!(efAsync_803C0000) as *mut i8, data: null_mut() }, EF_DAT_Entry { ef_DAT_file: core::ptr::addr_of_mut!(efAsync_803C0014) as *mut i8, effDataTable_name: core::ptr::addr_of_mut!(efAsync_803C0024) as *mut i8, data: null_mut() }, EF_DAT_Entry { ef_DAT_file: core::ptr::addr_of_mut!(efAsync_803C0038) as *mut i8, effDataTable_name: core::ptr::addr_of_mut!(efAsync_803C0044) as *mut i8, data: null_mut() }, EF_DAT_Entry { ef_DAT_file: core::ptr::addr_of_mut!(efAsync_803C005C) as *mut i8, effDataTable_name: core::ptr::addr_of_mut!(efAsync_803C0068) as *mut i8, data: null_mut() }, EF_DAT_Entry { ef_DAT_file: null_mut(), effDataTable_name: null_mut(), data: null_mut() }, EF_DAT_Entry { ef_DAT_file: null_mut(), effDataTable_name: null_mut(), data: null_mut() }, EF_DAT_Entry { ef_DAT_file: null_mut(), effDataTable_name: null_mut(), data: null_mut() }, EF_DAT_Entry { ef_DAT_file: null_mut(), effDataTable_name: null_mut(), data: null_mut() }, EF_DAT_Entry { ef_DAT_file: null_mut(), effDataTable_name: null_mut(), data: null_mut() }, EF_DAT_Entry { ef_DAT_file: null_mut(), effDataTable_name: null_mut(), data: null_mut() }, EF_DAT_Entry { ef_DAT_file: null_mut(), effDataTable_name: null_mut(), data: null_mut() }, EF_DAT_Entry { ef_DAT_file: null_mut(), effDataTable_name: null_mut(), data: null_mut() }, EF_DAT_Entry { ef_DAT_file: null_mut(), effDataTable_name: null_mut(), data: null_mut() }, EF_DAT_Entry { ef_DAT_file: core::ptr::addr_of_mut!(efAsync_803C0080) as *mut i8, effDataTable_name: core::ptr::addr_of_mut!(efAsync_803C0090) as *mut i8, data: null_mut() }, EF_DAT_Entry { ef_DAT_file: core::ptr::addr_of_mut!(efAsync_803C00A4) as *mut i8, effDataTable_name: core::ptr::addr_of_mut!(efAsync_803C00B0) as *mut i8, data: null_mut() }, EF_DAT_Entry { ef_DAT_file: core::ptr::addr_of_mut!(efAsync_803C00C8) as *mut i8, effDataTable_name: core::ptr::addr_of_mut!(efAsync_803C00D4) as *mut i8, data: null_mut() }, EF_DAT_Entry { ef_DAT_file: core::ptr::addr_of_mut!(efAsync_803C00EC) as *mut i8, effDataTable_name: core::ptr::addr_of_mut!(efAsync_803C00F8) as *mut i8, data: null_mut() }, EF_DAT_Entry { ef_DAT_file: null_mut(), effDataTable_name: null_mut(), data: null_mut() }, EF_DAT_Entry { ef_DAT_file: core::ptr::addr_of_mut!(efAsync_803C0110) as *mut i8, effDataTable_name: core::ptr::addr_of_mut!(efAsync_803C011C) as *mut i8, data: null_mut() }, EF_DAT_Entry { ef_DAT_file: core::ptr::addr_of_mut!(efAsync_803C0138) as *mut i8, effDataTable_name: core::ptr::addr_of_mut!(efAsync_803C0144) as *mut i8, data: null_mut() }, EF_DAT_Entry { ef_DAT_file: core::ptr::addr_of_mut!(efAsync_803C015C) as *mut i8, effDataTable_name: core::ptr::addr_of_mut!(efAsync_803C0168) as *mut i8, data: null_mut() }, EF_DAT_Entry { ef_DAT_file: core::ptr::addr_of_mut!(efAsync_803C0184) as *mut i8, effDataTable_name: core::ptr::addr_of_mut!(efAsync_803C0190) as *mut i8, data: null_mut() }, EF_DAT_Entry { ef_DAT_file: null_mut(), effDataTable_name: null_mut(), data: null_mut() }, EF_DAT_Entry { ef_DAT_file: core::ptr::addr_of_mut!(efAsync_803C01A8) as *mut i8, effDataTable_name: core::ptr::addr_of_mut!(efAsync_803C01B4) as *mut i8, data: null_mut() }, EF_DAT_Entry { ef_DAT_file: null_mut(), effDataTable_name: null_mut(), data: null_mut() }, EF_DAT_Entry { ef_DAT_file: null_mut(), effDataTable_name: null_mut(), data: null_mut() }, EF_DAT_Entry { ef_DAT_file: null_mut(), effDataTable_name: null_mut(), data: null_mut() }, EF_DAT_Entry { ef_DAT_file: null_mut(), effDataTable_name: null_mut(), data: null_mut() }, EF_DAT_Entry { ef_DAT_file: core::ptr::addr_of_mut!(efAsync_803C01CC) as *mut i8, effDataTable_name: core::ptr::addr_of_mut!(efAsync_803C01D8) as *mut i8, data: null_mut() }, EF_DAT_Entry { ef_DAT_file: core::ptr::addr_of_mut!(efAsync_803C01F0) as *mut i8, effDataTable_name: core::ptr::addr_of_mut!(efAsync_803C01FC) as *mut i8, data: null_mut() }, EF_DAT_Entry { ef_DAT_file: core::ptr::addr_of_mut!(efAsync_803C0214) as *mut i8, effDataTable_name: core::ptr::addr_of_mut!(efAsync_803C0220) as *mut i8, data: null_mut() }, EF_DAT_Entry { ef_DAT_file: core::ptr::addr_of_mut!(efAsync_803C0238) as *mut i8, effDataTable_name: core::ptr::addr_of_mut!(efAsync_803C0248) as *mut i8, data: null_mut() }, EF_DAT_Entry { ef_DAT_file: null_mut(), effDataTable_name: null_mut(), data: null_mut() }, unsafe { core::mem::zeroed() }, unsafe { core::mem::zeroed() }, unsafe { core::mem::zeroed() }, unsafe { core::mem::zeroed() }, unsafe { core::mem::zeroed() }, unsafe { core::mem::zeroed() }, unsafe { core::mem::zeroed() }, unsafe { core::mem::zeroed() }, unsafe { core::mem::zeroed() }, unsafe { core::mem::zeroed() }, unsafe { core::mem::zeroed() }, unsafe { core::mem::zeroed() }, unsafe { core::mem::zeroed() }, unsafe { core::mem::zeroed() }];
pub static mut efAsync_MexBhv: [*mut EfMexBehavior; 65] = [null_mut(); 65];
pub static mut efAsync_MexReady: i32 = 0;

/// `efAsync_GetEffectJObj`
pub unsafe extern "C" fn efAsync_GetEffectJObj(mut effect: *mut EF_Effect) -> *mut HSD_JObj {
    melee_core::ct!("efAsync_GetEffectJObj");
    return HSD_GObjGetHSDObj((*effect).gobj) as *mut HSD_JObj;
}

/// `efAsync_SetEffectRotationZFromPtr`
pub unsafe extern "C" fn efAsync_SetEffectRotationZFromPtr(mut effect: *mut EF_Effect, mut rotation: *const f32) {
    melee_core::ct!("efAsync_SetEffectRotationZFromPtr");
    HSD_JObjSetRotationZ(HSD_GObjGetHSDObj((*effect).gobj) as *mut HSD_JObj, *rotation);
}

/// `efAsync_SetEffectRandomRotationZ`
pub unsafe extern "C" fn efAsync_SetEffectRandomRotationZ(mut effect: *mut EF_Effect) {
    melee_core::ct!("efAsync_SetEffectRandomRotationZ");
    HSD_JObjSetRotationZ(HSD_GObjGetHSDObj((*effect).gobj) as *mut HSD_JObj, (6.283185307179586f64 * (ext::HSD_Randf() as f64)) as f32);
}

/// `efAsync_SetEffectScaleXYZ`
pub unsafe extern "C" fn efAsync_SetEffectScaleXYZ(mut effect: *mut EF_Effect, mut scale: f32) {
    melee_core::ct!("efAsync_SetEffectScaleXYZ");
    let mut jobj: *mut HSD_JObj = null_mut();
    jobj = (*(*effect).gobj).hsd_obj as *mut HSD_JObj;
    ();
    jobj = HSD_GObjGetHSDObj((*effect).gobj) as *mut HSD_JObj;
    HSD_JObjSetScaleX(jobj, scale);
    jobj = HSD_GObjGetHSDObj((*effect).gobj) as *mut HSD_JObj;
    HSD_JObjSetScaleY(jobj, scale);
    jobj = HSD_GObjGetHSDObj((*effect).gobj) as *mut HSD_JObj;
    HSD_JObjSetScaleZ(jobj, scale);
}

/// `efAsync_SetEffectScale`
pub unsafe extern "C" fn efAsync_SetEffectScale(mut effect: *mut EF_Effect, mut scale: *mut Vec) {
    melee_core::ct!("efAsync_SetEffectScale");
    HSD_JObjSetScale(HSD_GObjGetHSDObj((*effect).gobj) as *mut HSD_JObj, scale);
}

/// `efAsync_SetEffectFacingDir`
pub unsafe extern "C" fn efAsync_SetEffectFacingDir(mut effect: *mut EF_Effect, mut facing_dir: f32) {
    melee_core::ct!("efAsync_SetEffectFacingDir");
    let mut rotation: f64 = 0.0;
    if facing_dir < 0.0f32 {
        rotation = -(3.141592653589793f64 / (2 as f64));
    } else {
        rotation = 3.141592653589793f64 / (2 as f64);
    }
    HSD_JObjSetRotationY(HSD_GObjGetHSDObj((*effect).gobj) as *mut HSD_JObj, rotation as f32);
}

/// `efAsync_Dispatch`
pub unsafe extern "C" fn efAsync_Dispatch(mut gfx_id: i32, mut gobj: *mut HSD_GObj, mut vlist: *mut crate::tu::va::Va) -> *mut c_void {
    melee_core::ct!("efAsync_Dispatch");
    let mut translate: Vec3 = unsafe { core::mem::zeroed() };
    let mut scale: Vec3 = unsafe { core::mem::zeroed() };
    let mut generator: *mut HSD_Generator = null_mut();
    let mut psAppSRT: *mut HSD_psAppSRT = null_mut();
    let mut effect: *mut EF_Effect = null_mut();
    let mut va_pos: *mut c_void = null_mut();
    let mut va_jobj: *mut c_void = null_mut();
    let mut ret_obj: *mut c_void = null_mut();
    let mut rot_y: f64 = 0.0;
    let mut f32_2: f32 = 0.0;
    let mut f32_1: f32 = 0.0;
    let mut u32_2: i32 = 0;
    let mut u32_1: i32 = 0;
    let mut color: u32 = 0;
    let mut jobj_2: *mut HSD_JObj = null_mut();
    let mut jobj_1: *mut HSD_JObj = null_mut();
    let mut jobj_3: *mut HSD_JObj = null_mut();
    let mut va_vec3: *mut Vec = null_mut();
    let mut count: i32 = 0;
    #[derive(Clone, Copy)] #[repr(C)] struct Local1 { generator: *mut HSD_Generator }
    let mut state: Local1 = unsafe { core::mem::zeroed() };
    ret_obj = null_mut();
    's16: {
        let mut jl17 = false;
        let mut jl18 = false;
        match gfx_id {
            1000 => {
                if ext::HSD_Randi(8) == 0 {
                    ret_obj = super::eflib::efLib_Create_Attach_Pos(9 as u32, gobj, *((*vlist).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
                } else {
                    ret_obj = super::eflib::efLib_Create_Attach_Pos(10 as u32, gobj, *((*vlist).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
                }
                if !ret_obj.is_null() {
                    (*(ret_obj as *mut EF_Effect)).state_flags = (((*(ret_obj as *mut EF_Effect)).state_flags as i32) | 128) as u8;
                    f32_1 = fmadds(0.04f32, *(*((*vlist).arg(8_u32) as *mut *mut f32)), 0.3f32);
                    if f32_1 < 0.3f32 {
                        f32_1 = 0.3f32;
                    }
                    if f32_1 > 1.5f32 {
                        f32_1 = 1.5f32;
                    }
                    scale.x = { scale.y = { scale.z = f32_1; scale.z }; scale.y };
                    HSD_JObjSetScale(HSD_GObjGetHSDObj((*(ret_obj as *mut EF_Effect)).gobj) as *mut HSD_JObj, core::ptr::addr_of_mut!(scale) as *mut Vec);
                }
            }
            1001 => {
                ret_obj = super::eflib::efLib_CreateGenerator(12, *((*vlist).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
            }
            1002 => {
                ret_obj = super::eflib::efLib_CreateGenerator(20, *((*vlist).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
            }
            1003 => {
                ret_obj = super::eflib::efLib_Create_Attach_Pos(7 as u32, gobj, *((*vlist).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
                if !ret_obj.is_null() {
                    (*(ret_obj as *mut EF_Effect)).state_flags = (((*(ret_obj as *mut EF_Effect)).state_flags as i32) | 128) as u8;
                }
            }
            1004 => {
                ret_obj = super::eflib::efLib_Create_Attach_Pos(8 as u32, gobj, *((*vlist).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
                if !ret_obj.is_null() {
                    (*(ret_obj as *mut EF_Effect)).state_flags = (((*(ret_obj as *mut EF_Effect)).state_flags as i32) | 128) as u8;
                    efAsync_SetEffectRandomRotationZ(ret_obj as *mut EF_Effect);
                }
            }
            1005 => {
                va_vec3 = *((*vlist).arg(8_u32) as *mut *mut Vec);
                f32_1 = (*va_vec3).x;
                translate = *va_vec3;
                ret_obj = super::eflib::efLib_CreateGenerator(80, core::ptr::addr_of_mut!(translate) as *mut Vec) as *mut c_void;
                if !ret_obj.is_null() {
                    generator = super::eflib::efLib_CreateGenerator_AddAppSRT(84);
                    if !generator.is_null() {
                        psAppSRT = (*generator).appsrt;
                        (*psAppSRT).translate = translate;
                        if (*(*((*vlist).arg(8_u32) as *mut *mut f32))) < 0.0f32 {
                            rot_y = 0.0f64;
                        } else {
                            rot_y = -3.141592653589793f64;
                        }
                        psAppSRT = (*generator).appsrt;
                        (*psAppSRT).rot.y = rot_y as f32;
                    }
                }
            }
            1006 => {
                ret_obj = super::eflib::efLib_Create_Attach_Pos(39 as u32, gobj, *((*vlist).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
                if !ret_obj.is_null() {
                    efAsync_SetEffectRandomRotationZ(ret_obj as *mut EF_Effect);
                }
            }
            1007 | 1008 => {
                va_vec3 = *((*vlist).arg(8_u32) as *mut *mut Vec);
                translate = *va_vec3;
                if gfx_id == 1008 {
                    f32_1 = *(*((*vlist).arg(8_u32) as *mut *mut f32));
                } else {
                    f32_1 = -(*(*((*vlist).arg(8_u32) as *mut *mut f32)));
                }
                ret_obj = super::eflib::efLib_CreateGenerator_Translate_FacingDir(66, core::ptr::addr_of_mut!(translate) as *mut Vec, f32_1) as *mut c_void;
            }
            1009 | 1010 => {
                va_vec3 = *((*vlist).arg(8_u32) as *mut *mut Vec);
                translate = *va_vec3;
                if gfx_id == 1010 {
                    f32_1 = *(*((*vlist).arg(8_u32) as *mut *mut f32));
                } else {
                    f32_1 = -(*(*((*vlist).arg(8_u32) as *mut *mut f32)));
                }
                ret_obj = super::eflib::efLib_CreateGenerator_Translate_FacingDir(331, core::ptr::addr_of_mut!(translate) as *mut Vec, f32_1) as *mut c_void;
            }
            1011 => {
                ret_obj = super::eflib::efLib_CreateGenerator(11, *((*vlist).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
            }
            1012 => {
                ret_obj = super::eflib::efLib_CreateGenerator(72, *((*vlist).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
            }
            1013 => {
                ret_obj = super::eflib::efLib_Create_Attach_Pos(16 as u32, gobj, *((*vlist).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
                if !ret_obj.is_null() {
                    (*(ret_obj as *mut EF_Effect)).state_flags = (((*(ret_obj as *mut EF_Effect)).state_flags as i32) | 128) as u8;
                    efAsync_SetEffectFacingDir(ret_obj as *mut EF_Effect, *(*((*vlist).arg(8_u32) as *mut *mut f32)));
                }
            }
            1014 => {
                ret_obj = super::eflib::efLib_Create_Attach_Pos(17 as u32, gobj, *((*vlist).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
                if !ret_obj.is_null() {
                    (*(ret_obj as *mut EF_Effect)).state_flags = (((*(ret_obj as *mut EF_Effect)).state_flags as i32) | 128) as u8;
                }
            }
            1015 => {
                ret_obj = super::eflib::efLib_Create_Attach_Pos(18 as u32, gobj, *((*vlist).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
                if !ret_obj.is_null() {
                    (*(ret_obj as *mut EF_Effect)).state_flags = (((*(ret_obj as *mut EF_Effect)).state_flags as i32) | 128) as u8;
                    efAsync_SetEffectFacingDir(ret_obj as *mut EF_Effect, *(*((*vlist).arg(8_u32) as *mut *mut f32)));
                    efAsync_SetEffectRotationZFromPtr(ret_obj as *mut EF_Effect, (*((*vlist).arg(8_u32) as *mut *mut f32)) as *const f32);
                }
            }
            1016 => {
                ret_obj = super::eflib::efLib_Create_Attach_Pos(19 as u32, gobj, *((*vlist).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
                if !ret_obj.is_null() {
                    (*(ret_obj as *mut EF_Effect)).state_flags = (((*(ret_obj as *mut EF_Effect)).state_flags as i32) | 128) as u8;
                    efAsync_SetEffectFacingDir(ret_obj as *mut EF_Effect, *(*((*vlist).arg(8_u32) as *mut *mut f32)));
                    efAsync_SetEffectRotationZFromPtr(ret_obj as *mut EF_Effect, (*((*vlist).arg(8_u32) as *mut *mut f32)) as *const f32);
                }
            }
            1017 => {
                ret_obj = super::eflib::efLib_Create_Attach_Pos(20 as u32, gobj, *((*vlist).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
                if !ret_obj.is_null() {
                    (*(ret_obj as *mut EF_Effect)).state_flags = (((*(ret_obj as *mut EF_Effect)).state_flags as i32) | 128) as u8;
                    efAsync_SetEffectFacingDir(ret_obj as *mut EF_Effect, *(*((*vlist).arg(8_u32) as *mut *mut f32)));
                    efAsync_SetEffectRotationZFromPtr(ret_obj as *mut EF_Effect, (*((*vlist).arg(8_u32) as *mut *mut f32)) as *const f32);
                }
            }
            1018 => {
                ret_obj = super::eflib::efLib_Create_Attach_Pos(21 as u32, gobj, *((*vlist).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
                if !ret_obj.is_null() {
                    (*(ret_obj as *mut EF_Effect)).state_flags = (((*(ret_obj as *mut EF_Effect)).state_flags as i32) | 128) as u8;
                }
            }
            1019 => {
                ret_obj = super::eflib::efLib_Create_Attach_Pos(22 as u32, gobj, *((*vlist).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
                if !ret_obj.is_null() {
                    (*(ret_obj as *mut EF_Effect)).state_flags = (((*(ret_obj as *mut EF_Effect)).state_flags as i32) | 128) as u8;
                }
            }
            1020 => {
                ret_obj = super::eflib::efLib_Create_Attach_Pos(23 as u32, gobj, *((*vlist).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
                if !ret_obj.is_null() {
                    (*(ret_obj as *mut EF_Effect)).state_flags = (((*(ret_obj as *mut EF_Effect)).state_flags as i32) | 128) as u8;
                }
            }
            1021 => {
                ret_obj = super::eflib::efLib_Create_Attach_Pos(3 as u32, gobj, *((*vlist).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
                if !ret_obj.is_null() {
                    (*(ret_obj as *mut EF_Effect)).state_flags = (((*(ret_obj as *mut EF_Effect)).state_flags as i32) | 128) as u8;
                    efAsync_SetEffectFacingDir(ret_obj as *mut EF_Effect, *(*((*vlist).arg(8_u32) as *mut *mut f32)));
                    efAsync_SetEffectRotationZFromPtr(ret_obj as *mut EF_Effect, (*((*vlist).arg(8_u32) as *mut *mut f32)) as *const f32);
                }
            }
            1022 => {
                va_vec3 = *((*vlist).arg(8_u32) as *mut *mut Vec);
                translate = *va_vec3;
                f32_1 = *(*((*vlist).arg(8_u32) as *mut *mut f32));
                ret_obj = super::eflib::efLib_CreateGenerator_Translate_FacingDir(263, core::ptr::addr_of_mut!(translate) as *mut Vec, f32_1) as *mut c_void;
            }
            1023 => {
                ret_obj = super::eflib::efLib_Create_Attach_Pos(5 as u32, gobj, *((*vlist).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
                if !ret_obj.is_null() {
                    (*(ret_obj as *mut EF_Effect)).state_flags = (((*(ret_obj as *mut EF_Effect)).state_flags as i32) | 128) as u8;
                    efAsync_SetEffectFacingDir(ret_obj as *mut EF_Effect, *(*((*vlist).arg(8_u32) as *mut *mut f32)));
                    efAsync_SetEffectRotationZFromPtr(ret_obj as *mut EF_Effect, (*((*vlist).arg(8_u32) as *mut *mut f32)) as *const f32);
                }
            }
            1024 | 1025 => {
                va_vec3 = *((*vlist).arg(8_u32) as *mut *mut Vec);
                translate = *va_vec3;
                if gfx_id == 1025 {
                    f32_1 = *(*((*vlist).arg(8_u32) as *mut *mut f32));
                } else {
                    f32_1 = -(*(*((*vlist).arg(8_u32) as *mut *mut f32)));
                }
                ret_obj = super::eflib::efLib_CreateGenerator_Translate_FacingDir(90, core::ptr::addr_of_mut!(translate) as *mut Vec, f32_1) as *mut c_void;
            }
            1026 => {
                ret_obj = ext::hsd_8039EFAC(0, 0, 89, *((*vlist).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
            }
            1027 => {
                ret_obj = ext::hsd_8039EFAC(0, 0, 94, *((*vlist).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
            }
            1028 => {
                ret_obj = super::eflib::efLib_Create_Attach_Pos(24 as u32, gobj, *((*vlist).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
                if !ret_obj.is_null() {
                    (*(ret_obj as *mut EF_Effect)).state_flags = (((*(ret_obj as *mut EF_Effect)).state_flags as i32) | 128) as u8;
                    efAsync_SetEffectRotationZFromPtr(ret_obj as *mut EF_Effect, (*((*vlist).arg(8_u32) as *mut *mut f32)) as *const f32);
                }
            }
            1029 => {
                ret_obj = super::eflib::efLib_CreateGenerator(44, *((*vlist).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
            }
            1030 => {
                ret_obj = super::eflib::efLib_Create_Attach_Pos(4 as u32, gobj, *((*vlist).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
                if !ret_obj.is_null() {
                    (*(ret_obj as *mut EF_Effect)).state_flags = (((*(ret_obj as *mut EF_Effect)).state_flags as i32) | 128) as u8;
                    efAsync_SetEffectRotationZFromPtr(ret_obj as *mut EF_Effect, (*((*vlist).arg(8_u32) as *mut *mut f32)) as *const f32);
                }
            }
            1031 => {
                ret_obj = super::eflib::efLib_CreateGenerator(60, *((*vlist).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
            }
            1032 => {
                {
                    let mut result: *mut HSD_Generator = null_mut();
                    result = super::eflib::efLib_CreateGenerator_AddAppSRT(62);
                    if !result.is_null() {
                        va_vec3 = *((*vlist).arg(8_u32) as *mut *mut Vec);
                        psAppSRT = (*result).appsrt;
                        (*psAppSRT).translate = *va_vec3;
                        f32_1 = *(*((*vlist).arg(8_u32) as *mut *mut f32));
                        ret_obj = result as *mut c_void;
                        (*(*result).appsrt).rot.z = f32_1;
                    }
                    break 's16;
                }
            }
            1033 => {
                ret_obj = ext::hsd_8039EFAC(0, 0, 226, *((*vlist).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
            }
            1034 => {
                {
                    let mut result: *mut HSD_Generator = null_mut();
                    result = super::eflib::efLib_CreateGenerator_AddAppSRT(577);
                    if !result.is_null() {
                        va_vec3 = *((*vlist).arg(8_u32) as *mut *mut Vec);
                        psAppSRT = (*result).appsrt;
                        (*psAppSRT).translate = *va_vec3;
                        f32_1 = *(*((*vlist).arg(8_u32) as *mut *mut f32));
                        ret_obj = result as *mut c_void;
                        (*(*result).appsrt).rot.z = f32_1;
                    }
                    break 's16;
                }
            }
            1035 => {
                {
                    let mut result: *mut HSD_Generator = null_mut();
                    result = super::eflib::efLib_CreateGenerator_AddAppSRT(578);
                    if !result.is_null() {
                        va_vec3 = *((*vlist).arg(8_u32) as *mut *mut Vec);
                        psAppSRT = (*result).appsrt;
                        (*psAppSRT).translate = *va_vec3;
                        f32_1 = *(*((*vlist).arg(8_u32) as *mut *mut f32));
                        ret_obj = result as *mut c_void;
                        (*(*result).appsrt).rot.z = f32_1;
                    }
                    break 's16;
                }
            }
            1036 => {
                ret_obj = super::eflib::efLib_CreateGenerator(25, *((*vlist).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
            }
            1037 => {
                {
                    state.generator = super::eflib::efLib_CreateGenerator_AddAppSRT(25);
                    if !state.generator.is_null() {
                        va_vec3 = *((*vlist).arg(8_u32) as *mut *mut Vec);
                        psAppSRT = (*state.generator).appsrt;
                        (*psAppSRT).translate = *va_vec3;
                        jobj_1 = HSD_GObjGetHSDObj(gobj) as *mut HSD_JObj;
                        ();
                        HSD_JObjGetScale(jobj_1, core::ptr::addr_of_mut!(scale) as *mut Vec);
                        ret_obj = state.generator as *mut c_void;
                        (*(*state.generator).appsrt).scale.x = { (*(*state.generator).appsrt).scale.y = { (*(*state.generator).appsrt).scale.z = scale.y; (*(*state.generator).appsrt).scale.z }; (*(*state.generator).appsrt).scale.y };
                    }
                    break 's16;
                }
            }
            1038 => {
                ret_obj = super::eflib::efLib_CreateGenerator(67, *((*vlist).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
            }
            1039 => {
                ret_obj = super::eflib::efLib_CreateGenerator(227, *((*vlist).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
            }
            1040 => {
                ret_obj = super::eflib::efLib_CreateGenerator(554, *((*vlist).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
            }
            1041 => {
                ret_obj = super::eflib::efLib_CreateGenerator(75, *((*vlist).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
            }
            1042 => {
                ret_obj = ext::hsd_8039EFAC(0, 0, 19, *((*vlist).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
            }
            1043 => {
                ret_obj = ext::hsd_8039EFAC(0, 0, 55, *((*vlist).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
            }
            1044 => {
                ret_obj = ext::hsd_8039EFAC(0, 0, 225, *((*vlist).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
            }
            1045 => {
                ret_obj = super::eflib::efLib_Create_AttachChild(37 as u32, gobj, *((*vlist).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
                if !ret_obj.is_null() {
                    (*(ret_obj as *mut EF_Effect)).state_flags = (((*(ret_obj as *mut EF_Effect)).state_flags as i32) | 128) as u8;
                    f32_1 = *(*((*vlist).arg(8_u32) as *mut *mut f32));
                    scale.x = { scale.y = { scale.z = f32_1; scale.z }; scale.y };
                    efAsync_SetEffectScale(ret_obj as *mut EF_Effect, core::ptr::addr_of_mut!(scale) as *mut Vec);
                }
            }
            1046 => {
                ret_obj = super::eflib::efLib_CreateGenerator(406, *((*vlist).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
            }
            1047 => {
                super::efdata::efLib_LoadKind = 1 as u32;
                ret_obj = super::eflib::efLib_Create_Attach(11 as u32, gobj, *((*vlist).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
                if !ret_obj.is_null() {
                    jobj_1 = HSD_GObjGetHSDObj((*(ret_obj as *mut EF_Effect)).gobj) as *mut HSD_JObj;
                    u32_1 = (*((*vlist).arg(4_u32) as *mut u32)) as i32;
                    if jobj_1.is_null() {
                        jobj_2 = null_mut();
                    } else {
                        jobj_2 = (*jobj_1).child;
                    }
                    if ((u32_1 as u32) + 4288675840_u32) == 24672_u32 {
                        u32_2 = 8421504;
                    } else {
                        u32_2 = 16777215;
                    }
                    super::eflib::efLib_SetTevKonstColor(jobj_2, 1, u32_2 as u32, u32_1 as u32);
                    (*(ret_obj as *mut EF_Effect)).scale_flags = (((*(ret_obj as *mut EF_Effect)).scale_flags as i32) | 1) as u8;
                    super::eflib::efLib_SetParamGfxId(gobj, gfx_id);
                    (*(ret_obj as *mut EF_Effect)).update = Some(super::eflib::efLib_Cb_ApplyStoredAlpha);
                }
            }
            1048 => {
                super::efdata::efLib_LoadKind = 1 as u32;
                ret_obj = super::eflib::efLib_Create_Attach(12 as u32, gobj, *((*vlist).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
                if !ret_obj.is_null() {
                    jobj_1 = HSD_GObjGetHSDObj((*(ret_obj as *mut EF_Effect)).gobj) as *mut HSD_JObj;
                    u32_1 = (*((*vlist).arg(4_u32) as *mut u32)) as i32;
                    if jobj_1.is_null() {
                        jobj_2 = null_mut();
                    } else {
                        jobj_2 = (*jobj_1).child;
                    }
                    if ((u32_1 as u32) + 4288675840_u32) == 24672_u32 {
                        u32_2 = 8421504;
                    } else {
                        u32_2 = 16777215;
                    }
                    super::eflib::efLib_SetTevKonstColor(jobj_2, 0, u32_2 as u32, u32_1 as u32);
                    (*(ret_obj as *mut EF_Effect)).scale_flags = (((*(ret_obj as *mut EF_Effect)).scale_flags as i32) | 1) as u8;
                    super::eflib::efLib_SetParamGfxId(gobj, gfx_id);
                    (*(ret_obj as *mut EF_Effect)).update = Some(super::eflib::efLib_Cb_ApplyStoredAlpha);
                }
            }
            1049 => {
                {
                    let mut child: *mut HSD_JObj = null_mut();
                    super::efdata::efLib_LoadKind = 1 as u32;
                    ret_obj = super::eflib::efLib_Create_Attach(13 as u32, gobj, *((*vlist).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
                    if !ret_obj.is_null() {
                        jobj_1 = efAsync_GetEffectJObj(ret_obj as *mut EF_Effect);
                        u32_1 = (*((*vlist).arg(4_u32) as *mut u32)) as i32;
                        if ((u32_1 as u32) + 4288675840_u32) == 24672_u32 {
                            color = 8421504 as u32;
                        } else {
                            color = 16777215 as u32;
                        }
                        u32_2 = color as i32;
                        child = HSD_JObjGetChild(jobj_1);
                        super::eflib::efLib_SetTevKonstColor(child, 0, u32_2 as u32, u32_1 as u32);
                        jobj_1 = HSD_JObjGetNext(child);
                        super::eflib::efLib_SetTevKonstColor(jobj_1, 0, u32_2 as u32, u32_1 as u32);
                        jobj_1 = HSD_JObjGetNext(jobj_1);
                        super::eflib::efLib_SetTevKonstColor(jobj_1, 0, u32_2 as u32, u32_1 as u32);
                        jobj_1 = HSD_JObjGetNext(jobj_1);
                        super::eflib::efLib_SetTevKonstColor(jobj_1, 0, u32_2 as u32, u32_1 as u32);
                        jobj_1 = HSD_JObjGetNext(jobj_1);
                        super::eflib::efLib_SetTevKonstColor(jobj_1, 0, u32_2 as u32, u32_1 as u32);
                        jobj_1 = HSD_JObjGetNext(jobj_1);
                        super::eflib::efLib_SetTevKonstColor(jobj_1, 0, u32_2 as u32, u32_1 as u32);
                        jobj_1 = HSD_JObjGetNext(jobj_1);
                        super::eflib::efLib_SetTevKonstColor(jobj_1, 0, u32_2 as u32, u32_1 as u32);
                        (*(ret_obj as *mut EF_Effect)).scale_flags = (((*(ret_obj as *mut EF_Effect)).scale_flags as i32) | 1) as u8;
                        super::eflib::efLib_SetParamGfxId(gobj, gfx_id);
                        (*(ret_obj as *mut EF_Effect)).update = Some(super::eflib::efLib_Cb_ApplyStoredAlpha);
                    }
                    break 's16;
                }
            }
            1050 => {
                super::efdata::efLib_LoadKind = 1 as u32;
                ret_obj = super::eflib::efLib_Create_Attach(14 as u32, gobj, *((*vlist).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
                if !ret_obj.is_null() {
                    jobj_1 = HSD_GObjGetHSDObj((*(ret_obj as *mut EF_Effect)).gobj) as *mut HSD_JObj;
                    u32_1 = (*((*vlist).arg(4_u32) as *mut u32)) as i32;
                    if jobj_1.is_null() {
                        jobj_2 = null_mut();
                    } else {
                        jobj_2 = (*jobj_1).child;
                    }
                    if ((u32_1 as u32) + 4288675840_u32) == 24672_u32 {
                        u32_2 = 8421504;
                    } else {
                        u32_2 = 16777215;
                    }
                    super::eflib::efLib_SetTevKonstColor(jobj_2, 0, u32_2 as u32, u32_1 as u32);
                    (*(ret_obj as *mut EF_Effect)).scale_flags = (((*(ret_obj as *mut EF_Effect)).scale_flags as i32) | 1) as u8;
                    super::eflib::efLib_SetParamGfxId(gobj, gfx_id);
                    (*(ret_obj as *mut EF_Effect)).update = Some(super::eflib::efLib_Cb_ApplyStoredAlpha);
                }
            }
            1051 => {
                {
                    let mut result: *mut HSD_Generator = null_mut();
                    let mut jobj: *mut HSD_JObj = null_mut();
                    super::efdata::efLib_LoadKind = 1 as u32;
                    result = super::eflib::efLib_CreateGenerator_AddAppSRT(49);
                    if !result.is_null() {
                        jobj = *((*vlist).arg(8_u32) as *mut *mut HSD_JObj);
                        ext::lb_8000B1CC(jobj, null_mut(), core::ptr::addr_of_mut!((*(*result).appsrt).translate) as *mut Vec);
                        HSD_JObjGetScale(jobj, core::ptr::addr_of_mut!(scale) as *mut Vec);
                        (*(*result).appsrt).scale.x = { (*(*result).appsrt).scale.y = { (*(*result).appsrt).scale.z = scale.y; (*(*result).appsrt).scale.z }; (*(*result).appsrt).scale.y };
                    }
                    break 's16;
                }
            }
            1052 => {
                ret_obj = super::eflib::efLib_CreateGenerator(93, *((*vlist).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
            }
            1053 => {
                ret_obj = super::eflib::efLib_Create_Attach_Pos(15 as u32, gobj, *((*vlist).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
                if !ret_obj.is_null() {
                    (*(ret_obj as *mut EF_Effect)).state_flags = (((*(ret_obj as *mut EF_Effect)).state_flags as i32) | 128) as u8;
                }
            }
            1054 => {
                ret_obj = super::eflib::efLib_CreateGenerator_AppSRT_SetScale(85, vlist) as *mut c_void;
            }
            1055 => {
                ret_obj = super::eflib::efLib_CreateGenerator(92, *((*vlist).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
            }
            1056 => {
                ret_obj = super::eflib::efLib_CreateGenerator(345, *((*vlist).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
            }
            1057 => {
                ret_obj = super::eflib::efLib_CreateGenerator(63, *((*vlist).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
            }
            1058 => {
                ret_obj = ext::hsd_8039EFAC(0, 0, 91, *((*vlist).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
            }
            1059 => {
                ret_obj = super::eflib::efLib_Create_Attach(1 as u32, gobj, *((*vlist).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
                if !ret_obj.is_null() {
                    efAsync_SetEffectRotationZFromPtr(ret_obj as *mut EF_Effect, (*((*vlist).arg(8_u32) as *mut *mut f32)) as *const f32);
                }
            }
            1060 => {
                ret_obj = super::eflib::efLib_Create_Attach(2 as u32, gobj, *((*vlist).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
                if !ret_obj.is_null() {
                    efAsync_SetEffectRotationZFromPtr(ret_obj as *mut EF_Effect, (*((*vlist).arg(8_u32) as *mut *mut f32)) as *const f32);
                }
            }
            1061 => {
                ret_obj = super::eflib::efLib_CreateGenerator(126, *((*vlist).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
            }
            1062 => {
                ret_obj = super::eflib::efLib_CreateGenerator(127, *((*vlist).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
            }
            1063 => {
                {
                    let mut node: *mut EF_Effect = null_mut();
                    let mut i: i32 = 0;
                    va_vec3 = *((*vlist).arg(8_u32) as *mut *mut Vec);
                    translate = *va_vec3;
                    {
                        i = 0;
                        'l19: while i < 6 {
                            node = super::eflib::efLib_Create_Attach_Pos(27 as u32, gobj, core::ptr::addr_of_mut!(translate) as *mut Vec);
                            if node.is_null() {
                                break 'l19;
                            }
                            (*node).update = Some(super::eflib::efLib_Cb_Fall_FromParamY);
                            (*node).lifetime = 40 as u16;
                            (*node).params.y = fmadds(0.4f32, ext::HSD_Randf(), 2.8f32);
                            f32_1 = (6.283185307179586f64 * (ext::HSD_Randf() as f64)) as f32;
                            jobj_1 = HSD_GObjGetHSDObj((*node).gobj) as *mut HSD_JObj;
                            HSD_JObjSetRotationY(jobj_1, f32_1);
                            if i != 0 {
                                (*effect).next = node;
                                effect = (*effect).next;
                            } else {
                                ret_obj = { effect = node; effect } as *mut c_void;
                            }
                            i = i + 1;
                        }
                    }
                    break 's16;
                }
            }
            1064 => {
                ret_obj = super::eflib::efLib_CreateGenerator_AppSRT_SetScale(202, vlist) as *mut c_void;
            }
            1065 => {
                ret_obj = super::eflib::efLib_CreateGenerator_AppSRT_SetScale(206, vlist) as *mut c_void;
            }
            1066 => {
                ret_obj = super::eflib::efLib_CreateGenerator_AppSRT_SetScale(207, vlist) as *mut c_void;
            }
            1067 => {
                {
                    let mut setter_jobj: *mut HSD_JObj = null_mut();
                    super::efdata::efLib_LoadKind = 1 as u32;
                    ret_obj = super::eflib::efLib_Create_Attach_Pos(25 as u32, gobj, *((*vlist).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
                    if !ret_obj.is_null() {
                        jobj_1 = HSD_GObjGetHSDObj((*(ret_obj as *mut EF_Effect)).gobj) as *mut HSD_JObj;
                        setter_jobj = jobj_1;
                        f32_2 = *(*((*vlist).arg(8_u32) as *mut *mut f32));
                        HSD_JObjSetRotationZ(setter_jobj, f32_2);
                        f32_2 = *(*((*vlist).arg(8_u32) as *mut *mut f32));
                        scale.x = { scale.y = { scale.z = f32_2; scale.z }; scale.y };
                        HSD_JObjSetScale(setter_jobj, core::ptr::addr_of_mut!(scale) as *mut Vec);
                        if jobj_1.is_null() {
                            jobj_2 = null_mut();
                        } else {
                            jobj_2 = (*jobj_1).child;
                        }
                        // MWCC evaluates these two va_args right to left (DOL 0x800659A0: the first __va_arg feeds the
                        // 4th parameter): notes/semantics.md "evaluation order".
                        let tev0 = *((*vlist).arg(4_u32) as *mut u32);
                        let konst = *((*vlist).arg(4_u32) as *mut u32);
                        super::eflib::efLib_SetTevKonstColor(jobj_2, 0, konst, tev0);
                    }
                    break 's16;
                }
            }
            1068 => {
                super::efdata::efLib_LoadKind = 1 as u32;
                ret_obj = super::eflib::efLib_Create_Attach_Pos(26 as u32, gobj, *((*vlist).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
                if !ret_obj.is_null() {
                    jobj_2 = HSD_GObjGetHSDObj((*(ret_obj as *mut EF_Effect)).gobj) as *mut HSD_JObj;
                    f32_2 = *(*((*vlist).arg(8_u32) as *mut *mut f32));
                    HSD_JObjSetRotationZ(jobj_2, f32_2);
                }
            }
            1069 => {
                super::efdata::efLib_LoadKind = 1 as u32;
                ret_obj = super::eflib::efLib_CreateGenerator(289, *((*vlist).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
            }
            1070 => {
                va_vec3 = *((*vlist).arg(8_u32) as *mut *mut Vec);
                translate = *va_vec3;
                f32_1 = *(*((*vlist).arg(8_u32) as *mut *mut f32));
                ret_obj = super::eflib::efLib_CreateGenerator_Translate_FacingDir(316, core::ptr::addr_of_mut!(translate) as *mut Vec, f32_1) as *mut c_void;
            }
            1071 => {
                ret_obj = super::eflib::efLib_Create_Attach_Pos(32 as u32, gobj, *((*vlist).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
                if !ret_obj.is_null() {
                    if (*(*((*vlist).arg(8_u32) as *mut *mut f32))) < 0.0f32 {
                        rot_y = -(3.141592653589793f64 / (2 as f64));
                    } else {
                        rot_y = 3.141592653589793f64 / (2 as f64);
                    }
                    f32_2 = rot_y as f32;
                    jobj_2 = HSD_GObjGetHSDObj((*(ret_obj as *mut EF_Effect)).gobj) as *mut HSD_JObj;
                    HSD_JObjSetRotationY(jobj_2, f32_2);
                }
            }
            1072 => {
                va_vec3 = *((*vlist).arg(8_u32) as *mut *mut Vec);
                translate = *va_vec3;
                f32_1 = *(*((*vlist).arg(8_u32) as *mut *mut f32));
                ret_obj = super::eflib::efLib_CreateGenerator_Translate_FacingDir(320, core::ptr::addr_of_mut!(translate) as *mut Vec, f32_1) as *mut c_void;
            }
            1073 => {
                ret_obj = super::eflib::efLib_Create_Attach_Pos(33 as u32, gobj, *((*vlist).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
                if !ret_obj.is_null() {
                    if (*(*((*vlist).arg(8_u32) as *mut *mut f32))) < 0.0f32 {
                        rot_y = -(3.141592653589793f64 / (2 as f64));
                    } else {
                        rot_y = 3.141592653589793f64 / (2 as f64);
                    }
                    f32_2 = rot_y as f32;
                    jobj_2 = HSD_GObjGetHSDObj((*(ret_obj as *mut EF_Effect)).gobj) as *mut HSD_JObj;
                    HSD_JObjSetRotationY(jobj_2, f32_2);
                }
            }
            1074 => {
                {
                    let mut result: *mut HSD_Generator = null_mut();
                    result = super::eflib::efLib_CreateGenerator_AddAppSRT(325);
                    if !result.is_null() {
                        va_vec3 = *((*vlist).arg(8_u32) as *mut *mut Vec);
                        psAppSRT = (*result).appsrt;
                        (*psAppSRT).translate = *va_vec3;
                        f32_1 = *(*((*vlist).arg(8_u32) as *mut *mut f32));
                        (*(*result).appsrt).scale.x = { (*(*result).appsrt).scale.y = { (*(*result).appsrt).scale.z = f32_1; (*(*result).appsrt).scale.z }; (*(*result).appsrt).scale.y };
                    }
                    break 's16;
                }
            }
            1075 => {
                ret_obj = ext::hsd_8039EFAC(0, 0, 277, *((*vlist).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
            }
            1076 => {
                ret_obj = super::eflib::efLib_CreateGenerator(333, *((*vlist).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
            }
            1077 => {
                u32_1 = 334;
                jl17 = true;
            }
            1078 => {
                u32_1 = 339;
                jl17 = true;
            }
            1079 => {
                u32_1 = 342;
                jl17 = true;
            }
            1080 => {
                ret_obj = super::eflib::efLib_Create_Attach(34 as u32, gobj, *((*vlist).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
                if !ret_obj.is_null() {
                    jobj_3 = HSD_GObjGetHSDObj(gobj) as *mut HSD_JObj;
                    ();
                    let _ = HSD_JObjGetRotationY(jobj_3);
                    f32_2 = (*jobj_3).rotate.y;
                    jobj_2 = HSD_GObjGetHSDObj((*(ret_obj as *mut EF_Effect)).gobj) as *mut HSD_JObj;
                    HSD_JObjSetRotationY(jobj_2, f32_2);
                    (*(ret_obj as *mut EF_Effect)).update = Some(super::eflib::efLib_Cb_SetScale_FromParamX);
                    (*(ret_obj as *mut EF_Effect)).params.x = *(*((*vlist).arg(8_u32) as *mut *mut f32));
                }
            }
            1081 => {
                ret_obj = super::eflib::efLib_Create_Attach(35 as u32, gobj, *((*vlist).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
                if !ret_obj.is_null() {
                    f32_2 = *(*((*vlist).arg(8_u32) as *mut *mut f32));
                    efAsync_SetEffectScaleXYZ(ret_obj as *mut EF_Effect, f32_2);
                }
            }
            1082 => {
                ret_obj = super::eflib::efLib_CreateGenerator(403, *((*vlist).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
            }
            1083 => {
                ret_obj = super::eflib::efLib_CreateGenerator(402, *((*vlist).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
            }
            1084 => {
                ret_obj = super::eflib::efLib_CreateGenerator(416, *((*vlist).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
            }
            1085 => {
                ret_obj = ext::hsd_8039EFAC(0, 0, 431, *((*vlist).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
            }
            1086 => {
                super::efdata::efLib_LoadKind = 1 as u32;
                ret_obj = super::eflib::efLib_Create_Attach(36 as u32, gobj, *((*vlist).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
                if !ret_obj.is_null() {
                    f32_2 = *(*((*vlist).arg(8_u32) as *mut *mut f32));
                    efAsync_SetEffectScaleXYZ(ret_obj as *mut EF_Effect, f32_2);
                }
            }
            1087 => {
                {
                    let mut result: *mut HSD_Generator = null_mut();
                    super::efdata::efLib_LoadKind = 1 as u32;
                    result = super::eflib::efLib_CreateGenerator_AddAppSRT(202);
                    if !result.is_null() {
                        va_vec3 = *((*vlist).arg(8_u32) as *mut *mut Vec);
                        psAppSRT = (*result).appsrt;
                        (*psAppSRT).translate = *va_vec3;
                        f32_1 = *(*((*vlist).arg(8_u32) as *mut *mut f32));
                        (*(*result).appsrt).scale.x = { (*(*result).appsrt).scale.y = { (*(*result).appsrt).scale.z = f32_1; (*(*result).appsrt).scale.z }; (*(*result).appsrt).scale.y };
                    }
                    break 's16;
                }
            }
            1088 => {
                va_vec3 = *((*vlist).arg(8_u32) as *mut *mut Vec);
                translate = *va_vec3;
                f32_1 = *(*((*vlist).arg(8_u32) as *mut *mut f32));
                ret_obj = super::eflib::efLib_CreateGenerator_Translate_FacingDir(472, core::ptr::addr_of_mut!(translate) as *mut Vec, f32_1) as *mut c_void;
            }
            1089 => {
                ret_obj = super::eflib::efLib_CreateGenerator(507, *((*vlist).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
            }
            1090 => {
                ret_obj = super::eflib::efLib_CreateGenerator(476, *((*vlist).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
            }
            1091 => {
                {
                    let mut result: *mut HSD_Generator = null_mut();
                    result = super::eflib::efLib_CreateGenerator_AddAppSRT(497);
                    if !result.is_null() {
                        va_vec3 = *((*vlist).arg(8_u32) as *mut *mut Vec);
                        psAppSRT = (*result).appsrt;
                        (*psAppSRT).translate = *va_vec3;
                        f32_1 = *(*((*vlist).arg(8_u32) as *mut *mut f32));
                        (*(*result).appsrt).scale.x = { (*(*result).appsrt).scale.y = { (*(*result).appsrt).scale.z = f32_1; (*(*result).appsrt).scale.z }; (*(*result).appsrt).scale.y };
                    }
                    break 's16;
                }
            }
            1092 => {
                ret_obj = super::eflib::efLib_CreateGenerator(511, *((*vlist).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
            }
            1093 => {
                ret_obj = super::eflib::efLib_CreateGenerator(521, *((*vlist).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
            }
            1094 => {
                {
                    let mut source_jobj: *mut HSD_JObj = *((*vlist).arg(8_u32) as *mut *mut HSD_JObj);
                    va_vec3 = *((*vlist).arg(8_u32) as *mut *mut Vec);
                    ret_obj = super::eflib::efLib_CreateGenerator_AppSRT_SetPos(27, gobj, source_jobj, va_vec3) as *mut c_void;
                    if !ret_obj.is_null() {
                        (*(ret_obj as *mut EF_Effect)).lifetime = 22 as u16;
                    }
                    break 's16;
                }
            }
            1095 => {
                super::efdata::efLib_LoadKind = 1 as u32;
                ret_obj = super::eflib::efLib_Create_Attach_Pos(40 as u32, gobj, *((*vlist).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
                if !ret_obj.is_null() {
                    (*(ret_obj as *mut EF_Effect)).update = Some(super::eflib::efLib_Cb_ftCo_Bury);
                    if gfx_id == 1095 {
                        f32_1 = *(*((*vlist).arg(8_u32) as *mut *mut f32));
                        scale.x = { scale.y = { scale.z = f32_1; scale.z }; scale.y };
                        jobj_2 = HSD_GObjGetHSDObj((*(ret_obj as *mut EF_Effect)).gobj) as *mut HSD_JObj;
                        HSD_JObjSetScale(jobj_2, core::ptr::addr_of_mut!(scale) as *mut Vec);
                    }
                }
            }
            1096 => {
                va_jobj = *((*vlist).arg(8_u32) as *mut *mut c_void);
                va_pos = *((*vlist).arg(8_u32) as *mut *mut c_void);
                ret_obj = super::eflib::efLib_CreateGenerator_AppSRT_SetPos(146, gobj, va_jobj as *mut HSD_JObj, va_pos as *mut Vec) as *mut c_void;
                if !ret_obj.is_null() {
                    (*(ret_obj as *mut EF_Effect)).lifetime = 180 as u16;
                }
            }
            1097 => {
                ret_obj = super::eflib::efLib_Create_Attach(38 as u32, gobj, *((*vlist).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
            }
            1098 => {
                f32_2 = 0.5f32;
                jl18 = true;
            }
            1099 => {
                f32_2 = 1.0f32;
                jl18 = true;
            }
            1100 => {
                f32_2 = 2.0f32;
                jl18 = true;
            }
            1101 => {
                ret_obj = super::eflib::efLib_CreateGenerator_AppSRT_SetScale(72, vlist) as *mut c_void;
            }
            1102 | 1111 => {
                ret_obj = super::eflib::efLib_CreateGenerator_AppSRT_SetFacingDirScale(218, vlist) as *mut c_void;
            }
            1103 | 1112 => {
                ret_obj = super::eflib::efLib_CreateGenerator_AppSRT_SetFacingDirScale(219, vlist) as *mut c_void;
            }
            1104 | 1113 => {
                ret_obj = super::eflib::efLib_CreateGenerator_AppSRT_SetFacingDirScale(220, vlist) as *mut c_void;
            }
            1105 | 1114 => {
                ret_obj = super::eflib::efLib_CreateGenerator_AppSRT_SetFacingDirScale(221, vlist) as *mut c_void;
            }
            1107 => {
                {
                    let mut result: *mut HSD_Generator = null_mut();
                    ext::lb_8000B1CC(*((*vlist).arg(8_u32) as *mut *mut HSD_JObj), null_mut(), core::ptr::addr_of_mut!(translate) as *mut Vec);
                    result = super::eflib::efLib_CreateGenerator_Translate_FacingDir(564, core::ptr::addr_of_mut!(translate) as *mut Vec, *(*((*vlist).arg(8_u32) as *mut *mut f32)));
                    if !result.is_null() {
                        f32_1 = *(*((*vlist).arg(8_u32) as *mut *mut f32));
                        ret_obj = result as *mut c_void;
                        (*(*result).appsrt).scale.x = { (*(*result).appsrt).scale.y = { (*(*result).appsrt).scale.z = f32_1; (*(*result).appsrt).scale.z }; (*(*result).appsrt).scale.y };
                    }
                    break 's16;
                }
            }
            1108 => {
                {
                    let mut result: *mut HSD_Generator = null_mut();
                    ext::lb_8000B1CC(*((*vlist).arg(8_u32) as *mut *mut HSD_JObj), null_mut(), core::ptr::addr_of_mut!(translate) as *mut Vec);
                    result = super::eflib::efLib_CreateGenerator_Translate_FacingDir(565, core::ptr::addr_of_mut!(translate) as *mut Vec, *(*((*vlist).arg(8_u32) as *mut *mut f32)));
                    if !result.is_null() {
                        f32_1 = *(*((*vlist).arg(8_u32) as *mut *mut f32));
                        ret_obj = result as *mut c_void;
                        (*(*result).appsrt).scale.x = { (*(*result).appsrt).scale.y = { (*(*result).appsrt).scale.z = f32_1; (*(*result).appsrt).scale.z }; (*(*result).appsrt).scale.y };
                    }
                    break 's16;
                }
            }
            1109 => {
                {
                    let mut result: *mut HSD_Generator = null_mut();
                    ext::lb_8000B1CC(*((*vlist).arg(8_u32) as *mut *mut HSD_JObj), null_mut(), core::ptr::addr_of_mut!(translate) as *mut Vec);
                    result = super::eflib::efLib_CreateGenerator_Translate_FacingDir(566, core::ptr::addr_of_mut!(translate) as *mut Vec, *(*((*vlist).arg(8_u32) as *mut *mut f32)));
                    if !result.is_null() {
                        f32_1 = *(*((*vlist).arg(8_u32) as *mut *mut f32));
                        ret_obj = result as *mut c_void;
                        (*(*result).appsrt).scale.x = { (*(*result).appsrt).scale.y = { (*(*result).appsrt).scale.z = f32_1; (*(*result).appsrt).scale.z }; (*(*result).appsrt).scale.y };
                    }
                    break 's16;
                }
            }
            1110 => {
                {
                    let mut result: *mut HSD_Generator = null_mut();
                    ext::lb_8000B1CC(*((*vlist).arg(8_u32) as *mut *mut HSD_JObj), null_mut(), core::ptr::addr_of_mut!(translate) as *mut Vec);
                    result = super::eflib::efLib_CreateGenerator_Translate_FacingDir(573, core::ptr::addr_of_mut!(translate) as *mut Vec, *(*((*vlist).arg(8_u32) as *mut *mut f32)));
                    if !result.is_null() {
                        f32_1 = *(*((*vlist).arg(8_u32) as *mut *mut f32));
                        ret_obj = result as *mut c_void;
                        (*(*result).appsrt).scale.x = { (*(*result).appsrt).scale.y = { (*(*result).appsrt).scale.z = f32_1; (*(*result).appsrt).scale.z }; (*(*result).appsrt).scale.y };
                    }
                    break 's16;
                }
            }
            1106 => {
                {
                    let mut result: *mut HSD_Generator = null_mut();
                    result = super::eflib::efLib_CreateGenerator_AddAppSRT(542);
                    if !result.is_null() {
                        ext::lb_8000B1CC(*((*vlist).arg(8_u32) as *mut *mut HSD_JObj), null_mut(), core::ptr::addr_of_mut!((*(*result).appsrt).translate) as *mut Vec);
                        f32_1 = *(*((*vlist).arg(8_u32) as *mut *mut f32));
                        ret_obj = result as *mut c_void;
                        (*(*result).appsrt).scale.x = { (*(*result).appsrt).scale.y = { (*(*result).appsrt).scale.z = f32_1; (*(*result).appsrt).scale.z }; (*(*result).appsrt).scale.y };
                    }
                    break 's16;
                }
            }
            1115 => {
                ret_obj = super::eflib::efLib_CreateGenerator_AppSRT_SetScale(140, vlist) as *mut c_void;
            }
            1116 => {
                ret_obj = super::eflib::efLib_CreateGenerator_AppSRT_SetScale(141, vlist) as *mut c_void;
            }
            1117 => {
                {
                    let mut result: *mut HSD_Generator = null_mut();
                    result = super::eflib::efLib_CreateGenerator_AddAppSRT(575);
                    if !result.is_null() {
                        ext::lb_8000B1CC(*((*vlist).arg(8_u32) as *mut *mut HSD_JObj), null_mut(), core::ptr::addr_of_mut!((*(*result).appsrt).translate) as *mut Vec);
                        f32_1 = *(*((*vlist).arg(8_u32) as *mut *mut f32));
                        ret_obj = result as *mut c_void;
                        (*(*result).appsrt).scale.x = { (*(*result).appsrt).scale.y = { (*(*result).appsrt).scale.z = f32_1; (*(*result).appsrt).scale.z }; (*(*result).appsrt).scale.y };
                    }
                    break 's16;
                }
            }
            1118 => {
                {
                    let mut result: *mut HSD_Generator = null_mut();
                    result = super::eflib::efLib_CreateGenerator_AddAppSRT(576);
                    if !result.is_null() {
                        ext::lb_8000B1CC(*((*vlist).arg(8_u32) as *mut *mut HSD_JObj), null_mut(), core::ptr::addr_of_mut!((*(*result).appsrt).translate) as *mut Vec);
                        f32_1 = *(*((*vlist).arg(8_u32) as *mut *mut f32));
                        ret_obj = result as *mut c_void;
                        (*(*result).appsrt).scale.x = { (*(*result).appsrt).scale.y = { (*(*result).appsrt).scale.z = f32_1; (*(*result).appsrt).scale.z }; (*(*result).appsrt).scale.y };
                    }
                    break 's16;
                }
            }
            1119 => {
                ret_obj = super::eflib::efLib_CreateGenerator_AppSRT_SetScale(142, vlist) as *mut c_void;
            }
            1120 => {
                ret_obj = super::eflib::efLib_CreateGenerator_AppSRT_SetScale(153, vlist) as *mut c_void;
            }
            1121 => {
                ret_obj = super::eflib::efLib_CreateGenerator_AppSRT_SetScale(149, vlist) as *mut c_void;
            }
            1122 => {
                ret_obj = super::eflib::efLib_CreateGenerator_AppSRT_SetScale(537, vlist) as *mut c_void;
            }
            1123 => {
                ret_obj = super::eflib::efLib_Create_Attach(41 as u32, gobj, *((*vlist).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
                if !ret_obj.is_null() {
                    f32_1 = *(*((*vlist).arg(8_u32) as *mut *mut f32));
                    scale.x = { scale.y = { scale.z = f32_1; scale.z }; scale.y };
                    jobj_2 = HSD_GObjGetHSDObj((*(ret_obj as *mut EF_Effect)).gobj) as *mut HSD_JObj;
                    HSD_JObjSetScale(jobj_2, core::ptr::addr_of_mut!(scale) as *mut Vec);
                }
            }
            1124 => {
                ret_obj = super::eflib::efLib_Create_Attach(42 as u32, gobj, *((*vlist).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
                if !ret_obj.is_null() {
                    f32_1 = *(*((*vlist).arg(8_u32) as *mut *mut f32));
                    scale.x = { scale.y = { scale.z = f32_1; scale.z }; scale.y };
                    jobj_2 = HSD_GObjGetHSDObj((*(ret_obj as *mut EF_Effect)).gobj) as *mut HSD_JObj;
                    HSD_JObjSetScale(jobj_2, core::ptr::addr_of_mut!(scale) as *mut Vec);
                }
            }
            1125 => {
                ret_obj = super::eflib::efLib_CreateGenerator_AppSRT_SetScale(136, vlist) as *mut c_void;
            }
            1126 => {
                ret_obj = super::eflib::efLib_CreateGenerator_AppSRT_SetScale(137, vlist) as *mut c_void;
            }
            1127 => {
                ret_obj = super::eflib::efLib_CreateGenerator_AppSRT_SetScale(138, vlist) as *mut c_void;
            }
            1128 => {
                ret_obj = super::eflib::efLib_Create_Attach(44 as u32, gobj, *((*vlist).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
                if !ret_obj.is_null() {
                    f32_1 = *(*((*vlist).arg(8_u32) as *mut *mut f32));
                    scale.x = { scale.y = { scale.z = f32_1; scale.z }; scale.y };
                    jobj_2 = HSD_GObjGetHSDObj((*(ret_obj as *mut EF_Effect)).gobj) as *mut HSD_JObj;
                    HSD_JObjSetScale(jobj_2, core::ptr::addr_of_mut!(scale) as *mut Vec);
                }
            }
            1129 => {
                ret_obj = super::eflib::efLib_Create_Attach(46 as u32, gobj, *((*vlist).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
                if !ret_obj.is_null() {
                    f32_1 = *(*((*vlist).arg(8_u32) as *mut *mut f32));
                    scale.x = { scale.y = { scale.z = f32_1; scale.z }; scale.y };
                    jobj_2 = HSD_GObjGetHSDObj((*(ret_obj as *mut EF_Effect)).gobj) as *mut HSD_JObj;
                    HSD_JObjSetScale(jobj_2, core::ptr::addr_of_mut!(scale) as *mut Vec);
                }
            }
            1130 => {
                generator = super::eflib::efLib_CreateGenerator_AddAppSRT(172);
                if !generator.is_null() {
                    ext::lb_8000B1CC(*((*vlist).arg(8_u32) as *mut *mut HSD_JObj), null_mut(), core::ptr::addr_of_mut!((*(*generator).appsrt).translate) as *mut Vec);
                    f32_1 = *(*((*vlist).arg(8_u32) as *mut *mut f32));
                    (*(*generator).appsrt).scale.x = { (*(*generator).appsrt).scale.y = { (*(*generator).appsrt).scale.z = f32_1; (*(*generator).appsrt).scale.z }; (*(*generator).appsrt).scale.y };
                    jobj_3 = HSD_GObjGetHSDObj(gobj) as *mut HSD_JObj;
                    ();
                    f32_2 = HSD_JObjGetRotationY(jobj_3);
                    ret_obj = generator as *mut c_void;
                    (*(*generator).appsrt).rot.y = (*jobj_3).rotate.y;
                }
            }
            1131 => {
                ret_obj = super::eflib::efLib_Create_Attach(43 as u32, gobj, *((*vlist).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
                if !ret_obj.is_null() {
                    f32_1 = *(*((*vlist).arg(8_u32) as *mut *mut f32));
                    scale.x = { scale.y = { scale.z = f32_1; scale.z }; scale.y };
                    jobj_2 = HSD_GObjGetHSDObj((*(ret_obj as *mut EF_Effect)).gobj) as *mut HSD_JObj;
                    HSD_JObjSetScale(jobj_2, core::ptr::addr_of_mut!(scale) as *mut Vec);
                }
            }
            1132 => {
                generator = super::eflib::efLib_CreateGenerator_AddAppSRT(158);
                if !generator.is_null() {
                    ext::lb_8000B1CC(*((*vlist).arg(8_u32) as *mut *mut HSD_JObj), null_mut(), core::ptr::addr_of_mut!((*(*generator).appsrt).translate) as *mut Vec);
                    f32_1 = *(*((*vlist).arg(8_u32) as *mut *mut f32));
                    (*(*generator).appsrt).scale.x = { (*(*generator).appsrt).scale.y = { (*(*generator).appsrt).scale.z = f32_1; (*(*generator).appsrt).scale.z }; (*(*generator).appsrt).scale.y };
                    jobj_3 = (*gobj).hsd_obj as *mut HSD_JObj;
                    ();
                    f32_2 = HSD_JObjGetRotationY(jobj_3);
                    ret_obj = generator as *mut c_void;
                    (*(*generator).appsrt).rot.y = (*jobj_3).rotate.y;
                }
            }
            1133 => {
                ret_obj = super::eflib::efLib_Create_Attach(45 as u32, gobj, *((*vlist).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
                if !ret_obj.is_null() {
                    f32_1 = *(*((*vlist).arg(8_u32) as *mut *mut f32));
                    scale.x = { scale.y = { scale.z = f32_1; scale.z }; scale.y };
                    jobj_2 = HSD_GObjGetHSDObj((*(ret_obj as *mut EF_Effect)).gobj) as *mut HSD_JObj;
                    HSD_JObjSetScale(jobj_2, core::ptr::addr_of_mut!(scale) as *mut Vec);
                }
            }
            1134 => {
                ret_obj = super::eflib::efLib_CreateGenerator_AppSRT_SetScale(174, vlist) as *mut c_void;
            }
            1135 => {
                ret_obj = super::eflib::efLib_CreateGenerator_AppSRT_SetScale(160, vlist) as *mut c_void;
            }
            1136 => {
                ret_obj = super::eflib::efLib_CreateGenerator_AppSRT_SetScale(539, vlist) as *mut c_void;
            }
            1137 => {
                ret_obj = super::eflib::efLib_CreateGenerator_AppSRT_SetScale(544, vlist) as *mut c_void;
            }
            1138 => {
                ret_obj = super::eflib::efLib_CreateGenerator_AppSRT_SetScale(305, vlist) as *mut c_void;
            }
            1139 => {
                ret_obj = ext::hsd_8039EFAC(0, 0, 130, *((*vlist).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
            }
            1143 => {
                ret_obj = super::eflib::efLib_CreateGenerator(31000, *((*vlist).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
            }
            1140 => {
                ret_obj = super::eflib::efLib_CreateGenerator(247, *((*vlist).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
            }
            1141 => {
                ret_obj = super::eflib::efLib_CreateGenerator(252, *((*vlist).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
            }
            1142 => {
                ret_obj = super::eflib::efLib_CreateGenerator(255, *((*vlist).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
            }
            _ => {}
        }
        if jl17 {
            {
                let mut result: *mut HSD_Generator = null_mut();
                result = super::eflib::efLib_CreateGenerator_AddAppSRT(u32_1);
                if !result.is_null() {
                    va_vec3 = *((*vlist).arg(8_u32) as *mut *mut Vec);
                    psAppSRT = (*result).appsrt;
                    (*psAppSRT).translate = *va_vec3;
                    (*(*result).appsrt).rot.y = (3.141592653589793f64 / (2 as f64)) as f32;
                    f32_1 = *(*((*vlist).arg(8_u32) as *mut *mut f32));
                    ret_obj = result as *mut c_void;
                    (*(*result).appsrt).rot.z = f32_1;
                }
                break 's16;
            }
        }
        if jl18 {
            {
                let mut result: *mut HSD_Generator = null_mut();
                result = super::eflib::efLib_CreateGenerator_AddAppSRT(567);
                if !result.is_null() {
                    va_vec3 = *((*vlist).arg(8_u32) as *mut *mut Vec);
                    psAppSRT = (*result).appsrt;
                    (*psAppSRT).translate = *va_vec3;
                    (*(*result).appsrt).scale.x = { (*(*result).appsrt).scale.y = { (*(*result).appsrt).scale.z = f32_2; (*(*result).appsrt).scale.z }; (*(*result).appsrt).scale.y };
                }
                break 's16;
            }
        }
    }
    'l20: while super::efdata::efLib_AnimCount != 0 {
        count = super::efdata::efLib_AnimCount - 1;
        super::efdata::efLib_AnimCount = count;
        ext::HSD_JObjAnimAll(*((super::eflib::efLib_AnimQueue.as_mut_ptr()) as *mut *mut HSD_JObj).offset(count as isize));
    }
    return ret_obj;
}

/// `efAsync_LoadAsync`
pub unsafe extern "C" fn efAsync_LoadAsync(mut index: i32) {
    melee_core::ct!("efAsync_LoadAsync");
    let mut entry: *mut EF_DAT_Entry = core::ptr::addr_of_mut!(efAsync_DatEntries).cast::<EF_DAT_Entry>().wrapping_offset(index as isize) as *mut EF_DAT_Entry;
    efAsync_MexInit();
    if (index >= 65) || (index < 0) {
        return;
    }
    if (*entry).ef_DAT_file.is_null() {
        return;
    }
    ext::lbDvd_800178E8(3, (*entry).ef_DAT_file as *const i8, 4, 4, 0, 1_u32 as i32, 4, 4_u32 as u8, index);
}

/// `efAsync_OnLoad`
pub unsafe extern "C" fn efAsync_OnLoad(mut archive: *mut HSD_Archive, mut data: *mut u8, mut length: u32, mut index: i32) {
    melee_core::ct!("efAsync_OnLoad");
    let mut result: *mut EF_DAT_Entry = null_mut();
    ext::lbArchive_InitializeDAT(archive, data as *mut c_void, length);
    result = ext::HSD_ArchiveGetPublicAddress(archive, efAsync_DatEntries[index as usize].effDataTable_name as *const i8) as *mut EF_DAT_Entry;
    efAsync_MexNoteArchive(index, archive);
    if (((*result).ef_DAT_file as usize as u32) | ((*result).effDataTable_name as usize as u32)) != 0 {
        ext::psInitDataBankLocate((*result).ef_DAT_file as *mut HSD_Archive, (*result).effDataTable_name as *mut HSD_Archive, null_mut());
    }
}

/// `efAsync_LoadSync`
pub unsafe extern "C" fn efAsync_LoadSync(mut idx: i32) {
    melee_core::ct!("efAsync_LoadSync");
    let mut spC: *mut EF_DAT_Entry = null_mut();
    let mut lookup: *mut EF_DAT_Entry = null_mut();
    lookup = core::ptr::addr_of_mut!(efAsync_DatEntries).cast::<EF_DAT_Entry>().wrapping_offset(idx as isize) as *mut EF_DAT_Entry;
    efAsync_MexInit();
    if (idx >= 65) || (idx < 0) {
        return;
    }
    if !(!(*lookup).ef_DAT_file.is_null()) {
        return;
    }
    if !(*lookup).data.is_null() {
        return;
    }
    {
        let mut ef_archive: *mut HSD_Archive = null_mut();
        let mut chk: i32 = ext::lbArchive_80017040(core::ptr::addr_of_mut!(ef_archive) as *mut *mut HSD_Archive, (*lookup).ef_DAT_file as *const i8, (core::ptr::addr_of_mut!(spC) as *mut *mut EF_DAT_Entry) as *mut c_void, &[(*lookup).effDataTable_name as *const c_void, (0) as isize as *const c_void]);
        efAsync_MexNoteArchive(idx, ef_archive);
        if (((*spC).ef_DAT_file as usize as u32) | ((*spC).effDataTable_name as usize as u32)) != 0 {
            if chk != 0 {
                ext::psInitDataBankLoad(idx, ((*spC).ef_DAT_file as *mut c_void) as *const i32, ((*spC).effDataTable_name as *mut c_void) as *const i32, null_mut(), null_mut());
            } else {
                ext::psInitDataBank(idx, ((*spC).ef_DAT_file as *mut c_void) as *mut i32, ((*spC).effDataTable_name as *mut c_void) as *mut i32, null_mut(), null_mut());
            }
        }
        (*lookup).data = (core::ptr::addr_of_mut!((*spC).data) as *mut *mut c_void) as *mut EF_EffectDesc;
    }
}

/// `efAsync_QueueProcessDeferred`
pub unsafe extern "C" fn efAsync_QueueProcessDeferred(mut gobj: *mut HSD_GObj, mut queued_effect: *mut EF_QueuedEffect) {
    melee_core::ct!("efAsync_QueueProcessDeferred");
    let mut sp4C: Vec3 = unsafe { core::mem::zeroed() };
    let mut sp40: Vec3 = unsafe { core::mem::zeroed() };
    let mut sp34: Vec3 = unsafe { core::mem::zeroed() };
    let mut sp28: Vec3 = unsafe { core::mem::zeroed() };
    let mut sp1C: Vec3 = unsafe { core::mem::zeroed() };
    let mut sp10: Vec3 = unsafe { core::mem::zeroed() };
    let mut jobj: *mut HSD_JObj = null_mut();
    let mut gfx_id: u32 = 0;
    let mut spawn_kind: u8 = 0;
    spawn_kind = (*queued_effect).spawn_kind;
    gfx_id = (*queued_effect).gfx_id as u32;
    jobj = (*queued_effect).jobj;
    's21: {
        match spawn_kind as i32 {
            0 => {
                let _ = ext::efSync_Spawn(gfx_id as i32, gobj, &[jobj as *const c_void]);
            }
            1 => {
                ext::lb_8000B1CC(jobj, null_mut(), core::ptr::addr_of_mut!(sp4C) as *mut Vec);
                let _ = ext::efSync_Spawn(gfx_id as i32, gobj, &[core::ptr::addr_of_mut!(sp4C) as *mut Vec as *const c_void]);
            }
            2 => {
                ext::lb_8000B1CC(jobj, core::ptr::addr_of_mut!((*queued_effect).params) as *mut Vec, core::ptr::addr_of_mut!(sp40) as *mut Vec);
                let _ = ext::efSync_Spawn(gfx_id as i32, gobj, &[core::ptr::addr_of_mut!(sp40) as *mut Vec as *const c_void]);
            }
            3 => {
                let _ = ext::efSync_Spawn(gfx_id as i32, gobj, &[jobj as *const c_void, core::ptr::addr_of_mut!((*queued_effect).params) as *mut Vec as *const c_void]);
            }
            4 => {
                ext::lb_8000B1CC(jobj, null_mut(), core::ptr::addr_of_mut!(sp34) as *mut Vec);
                let _ = ext::efSync_Spawn(gfx_id as i32, gobj, &[core::ptr::addr_of_mut!(sp34) as *mut Vec as *const c_void, core::ptr::addr_of_mut!((*queued_effect).params) as *mut Vec as *const c_void]);
            }
            5 => {
                ext::lb_8000B1CC(jobj, core::ptr::addr_of_mut!((*queued_effect).params) as *mut Vec, core::ptr::addr_of_mut!(sp28) as *mut Vec);
                let _ = ext::efSync_Spawn(gfx_id as i32, gobj, &[core::ptr::addr_of_mut!(sp28) as *mut Vec as *const c_void, core::ptr::addr_of_mut!((*queued_effect).extra1) as *mut f32 as *const c_void]);
            }
            6 => {
                ext::lb_8000B1CC(jobj, core::ptr::addr_of_mut!((*queued_effect).params) as *mut Vec, core::ptr::addr_of_mut!(sp1C) as *mut Vec);
                let _ = ext::efSync_Spawn(gfx_id as i32, gobj, &[core::ptr::addr_of_mut!(sp1C) as *mut Vec as *const c_void, core::ptr::addr_of_mut!((*queued_effect).extra1) as *mut f32 as *const c_void, core::ptr::addr_of_mut!((*queued_effect).extra2) as *mut f32 as *const c_void]);
            }
            7 => {
                let _ = ext::efSync_Spawn(gfx_id as i32, gobj, &[jobj as *const c_void, core::ptr::addr_of_mut!((*queued_effect).params) as *mut Vec as *const c_void]);
            }
            8 => {
                ext::lb_8000B1CC(jobj, core::ptr::addr_of_mut!((*queued_effect).params) as *mut Vec, core::ptr::addr_of_mut!(sp10) as *mut Vec);
                ext::Camera_RequestQuake(gfx_id, core::ptr::addr_of_mut!(sp10) as *mut Vec);
            }
            9 => {
                efAsync_MexProcessDeferred(gobj, queued_effect);
            }
            _ => {
                if 0 != 0 { (); } else { ext::OSReport(&format!("[EfASync] unknown type {}\n", spawn_kind as i32)); panic!("efAsync_QueueProcessDeferred: assertion failed"); }
            }
        }
    }
    ext::HSD_ObjFree(core::ptr::addr_of_mut!(efAsync_AllocData) as *mut _HSD_ObjAllocData, queued_effect as *mut c_void);
}

/// `efAsync_QueueFlush`
pub unsafe extern "C" fn efAsync_QueueFlush(mut gobj: *mut HSD_GObj, mut arg_struct: *mut c_void) {
    melee_core::ct!("efAsync_QueueFlush");
    let mut temp_r31: *mut EF_QueuedEffect = null_mut();
    let mut var_r4: *mut EF_QueuedEffect = null_mut();
    var_r4 = (*(arg_struct as *mut EF_QueuedEffect)).next;
    'l22: while !var_r4.is_null() {
        temp_r31 = (*var_r4).next;
        efAsync_QueueProcessDeferred(gobj, var_r4);
        var_r4 = temp_r31;
    }
    (*(arg_struct as *mut EF_QueuedEffect)).next = null_mut();
}

/// `efAsync_QueueClear`
pub unsafe extern "C" fn efAsync_QueueClear(mut arg_struct: *mut c_void) {
    melee_core::ct!("efAsync_QueueClear");
    let mut temp_r30: *mut EF_QueuedEffect = null_mut();
    let mut var_r4: *mut EF_QueuedEffect = null_mut();
    var_r4 = (*(arg_struct as *mut EF_QueuedEffect)).next;
    'l23: while !var_r4.is_null() {
        temp_r30 = (*var_r4).next;
        ext::HSD_ObjFree(core::ptr::addr_of_mut!(efAsync_AllocData) as *mut _HSD_ObjAllocData, var_r4 as *mut c_void);
        var_r4 = temp_r30;
    }
    (*(arg_struct as *mut EF_QueuedEffect)).next = null_mut();
}

/// `efAsync_Spawn`
pub unsafe fn efAsync_Spawn(mut gobj: *mut HSD_GObj, mut queue_head: *mut c_void, mut spawn_kind: u32, mut gfx_id: u32, mut jobj: *mut HSD_JObj, args: &[*const c_void]) {
    melee_core::ct!("efAsync_Spawn");
    let mut va_store = crate::tu::va::Va::new(args);
    let mut vlist: *mut crate::tu::va::Va = null_mut();
    let mut va_vec3: *mut Vec = null_mut();
    let mut extra1: *mut f32 = null_mut();
    let mut extra2: *mut f32 = null_mut();
    let mut queued: *mut EF_QueuedEffect = null_mut();
    vlist = &raw mut va_store;
    queued = ext::HSD_ObjAlloc(core::ptr::addr_of_mut!(efAsync_AllocData) as *mut _HSD_ObjAllocData) as *mut EF_QueuedEffect;
    (*queued).spawn_kind = spawn_kind as u8;
    (*queued).gfx_id = gfx_id as i32;
    (*queued).jobj = jobj;
    's24: {
        match spawn_kind {
            0 | 1 => {
            }
            2 => {
                (*queued).params = *(*((*vlist).arg(8_u32) as *mut *mut Vec));
            }
            3 => {
                (*queued).params.x = *(*((*vlist).arg(8_u32) as *mut *mut f32));
            }
            4 => {
                (*queued).params.x = *(*((*vlist).arg(8_u32) as *mut *mut f32));
            }
            5 => {
                va_vec3 = *((*vlist).arg(8_u32) as *mut *mut Vec);
                extra2 = *((*vlist).arg(8_u32) as *mut *mut f32);
                (*queued).params = *va_vec3;
                (*queued).extra1 = *extra2;
            }
            6 => {
                va_vec3 = *((*vlist).arg(8_u32) as *mut *mut Vec);
                extra1 = *((*vlist).arg(8_u32) as *mut *mut f32);
                extra2 = *((*vlist).arg(8_u32) as *mut *mut f32);
                (*queued).params = *va_vec3;
                (*queued).extra1 = *extra1;
                (*queued).extra2 = *extra2;
            }
            7 => {
                (*queued).params = *(*((*vlist).arg(8_u32) as *mut *mut Vec));
            }
            8 => {
                (*queued).params = *(*((*vlist).arg(8_u32) as *mut *mut Vec));
            }
            _ => {
                if 0 != 0 { (); } else { ext::OSReport(&format!("[EfASync] unknown type {}\n", spawn_kind)); panic!("efAsync_Spawn: assertion failed"); }
            }
        }
    }
    ();
    if (!ext::HSD_GObj_CurrentInvokedProc.is_null()) && (((*ext::HSD_GObj_CurrentInvokedProc).s_link as u32) < 9_u32) {
        (*queued).next = (*(queue_head as *mut EF_QueuedEffect)).next;
        (*(queue_head as *mut EF_QueuedEffect)).next = queued;
        return;
    }
    efAsync_QueueProcessDeferred(gobj, queued);
}

/// `efAsync_QueueInit`
pub unsafe extern "C" fn efAsync_QueueInit() {
    melee_core::ct!("efAsync_QueueInit");
    ext::HSD_ObjAllocInit(core::ptr::addr_of_mut!(efAsync_AllocData) as *mut _HSD_ObjAllocData, 48_u64 as u32, 8_u64 as u32);
}

/// `efAsync_MexInit`
pub unsafe extern "C" fn efAsync_MexInit() {
    melee_core::ct!("efAsync_MexInit");
    let mut n: i32 = 0;
    let mut i: i32 = 0;
    if efAsync_MexReady != 0 {
        return;
    }
    efAsync_MexReady = 1;
    n = ext::Mex_EffectCount();
    if n > 65 {
        ext::OSReport(&format!("mexeffect: {} effect banks, the particle system holds {} - the rest are left out\n", n, 65));
        n = 65;
    }
    {
        i = 0;
        'l25: while i < n {
            'c25: {
                let mut file: *mut i8 = ext::Mex_EffectString(i, 0) as *mut i8;
                let mut sym: *mut i8 = ext::Mex_EffectString(i, 1) as *mut i8;
                if (file.is_null()) || (sym.is_null()) {
                    break 'c25;
                }
                efAsync_DatEntries[i as usize].ef_DAT_file = file;
                efAsync_DatEntries[i as usize].effDataTable_name = sym;
            }
            i = i + 1;
        }
    }
    if n > 0 {
        ext::OSReport(&format!("mexeffect: {} effect banks from MxDt.dat\n", n));
    }
}

/// `efAsync_MexNoteArchive`
pub unsafe extern "C" fn efAsync_MexNoteArchive(mut bank: i32, mut archive: *mut HSD_Archive) {
    melee_core::ct!("efAsync_MexNoteArchive");
    let mut bhv: *mut EfMexBehavior = null_mut();
    if ((bank < 0) || (bank >= 65)) || (archive.is_null()) {
        return;
    }
    bhv = ext::HSD_ArchiveGetPublicAddress(archive, (c"effBehaviorTable".as_ptr() as *mut i8) as *const i8) as *mut EfMexBehavior;
    if (bhv != efAsync_MexBhv[bank as usize]) && (!bhv.is_null()) {
        ext::OSReport(&format!("mexeffect: bank {} ({}): {} model(s), {} particle generator(s) with a behaviour\n", bank, if (efAsync_DatEntries[bank as usize].ef_DAT_file as *const core::ffi::c_char).is_null() { std::borrow::Cow::Borrowed("(null)") } else { core::ffi::CStr::from_ptr(efAsync_DatEntries[bank as usize].ef_DAT_file as *const core::ffi::c_char).to_string_lossy() }, (*bhv).mdl_num, (*bhv).ptcl_num));
    }
    efAsync_MexBhv[bank as usize] = bhv;
    // An m-ex effect file keeps the particle id base of the build it came from (Diddy's EfDdData.dat says 49000, Tails'
    // 22000, Charizard's 12000) while the resolver asks for bank * 1000 + n (efAsync_MexResolve, as m-ex numbers a
    // fighter's effects by the bank it loads into): the base is the bank's, so hsd_8039F05C reads the bank's own
    // n-th generator instead of one past the table (Diddy's gun item spawned from a garbage PSCmdList and crashed).
    if !bhv.is_null() && !efAsync_DatEntries[bank as usize].effDataTable_name.is_null() {
        let e = ext::HSD_ArchiveGetPublicAddress(archive, efAsync_DatEntries[bank as usize].effDataTable_name as *const i8) as *mut EF_DAT_Entry;
        if !e.is_null() && !(*e).ef_DAT_file.is_null() {
            let cmd = (*e).ef_DAT_file as *mut isize;
            if matches!(*cmd.cast::<u16>(), 0x40..=0x43) {
                *cmd.add(1) = bank as isize * 1000;
            }
        }
    }
}

/// `efAsync_MexMiss`
pub unsafe extern "C" fn efAsync_MexMiss(mut kind: i32, mut gfx_id: i32, mut why: *const i8) {
    melee_core::ct!("efAsync_MexMiss");
    let mut key: i32 = (((kind * 10000) + (gfx_id % 100000)) * 2) + ((gfx_id >= 100000) as i32);
    let mut i: i32 = 0;
    {
        i = 0;
        'l26: while i < efAsync_MexMiss_nseen {
            if efAsync_MexMiss_seen[i as usize] == key {
                return;
            }
            i = i + 1;
        }
    }
    if efAsync_MexMiss_nseen < (256_u64 as i32) {
        let __t28 = core::ptr::addr_of_mut!(efAsync_MexMiss_seen[{ let __t27 = efAsync_MexMiss_nseen; efAsync_MexMiss_nseen = efAsync_MexMiss_nseen + 1; __t27 } as usize]); *__t28 = key;
    }
    if gfx_id >= 100000 {
        ext::OSReport(&format!("mexeffect: fighter kind {} effect {} {}\n", kind, gfx_id - 100000, if (why as *const core::ffi::c_char).is_null() { std::borrow::Cow::Borrowed("(null)") } else { core::ffi::CStr::from_ptr(why as *const core::ffi::c_char).to_string_lossy() }));
        return;
    }
    ext::OSReport(&format!("mexeffect: fighter kind {} effect {} not drawn - {}\n", kind, gfx_id, if (why as *const core::ffi::c_char).is_null() { std::borrow::Cow::Borrowed("(null)") } else { core::ffi::CStr::from_ptr(why as *const core::ffi::c_char).to_string_lossy() }));
}

/// `efAsync_MexResolve`
pub unsafe extern "C" fn efAsync_MexResolve(mut gobj: *mut HSD_GObj, mut gfx_id: i32, mut final_id: *mut i32, mut is_ptcl: *mut i32, mut owner: *mut *mut HSD_GObj) -> i32 {
    melee_core::ct!("efAsync_MexResolve");
    let mut fp: *mut Fighter = null_mut();
    let mut kind: i32 = 0;
    let mut src_kind: i32 = 0;
    let mut bank: i32 = 0;
    let mut int_id: i32 = 0;
    let mut copy: i32 = 0;
    let mut r#type: i32 = 0;
    let mut bhv: *mut EfMexBehavior = null_mut();
    if (gobj.is_null()) || (!((gfx_id >= 5000) && (gfx_id < 9000))) {
        return (-1_i32);
    }
    if ((*gobj).classifier as i32) == 6 {
        gobj = (*((*gobj).user_data as *mut Item)).mex_original_owner;
        if (gobj.is_null()) || (((*gobj).classifier as i32) != 4) {
            return (-1_i32);
        }
    }
    if ((*gobj).classifier as i32) != 4 {
        return (-1_i32);
    }
    fp = HSD_GObjGetUserData(gobj) as *mut Fighter;
    kind = (*fp).kind as i32;
    if ((kind == (Ft_Kind_Kirby as i32)) || (((kind >= (Ft_Kind_Mex0 as i32)) && (kind < (Ft_Kind_Max as i32))) && (ext::ftKb_MexKirbyLike(kind) != 0))) && ((kind == (Ft_Kind_Kirby as i32)) || ((ext::ftData_UnkBytePerCharacter[kind as usize] as i32) == (ext::ftData_UnkBytePerCharacter[(Ft_Kind_Kirby as i32) as usize] as i32))) {
        if gfx_id >= 7000 {
            return (-1_i32);
        }
        copy = 1;
        *is_ptcl = (gfx_id >= 6000) as i32;
        int_id = gfx_id - (if (*is_ptcl) != 0 { 6000 } else { 5000 });
    } else {
        copy = (gfx_id >= 7000) as i32;
        *is_ptcl = (((gfx_id >= 6000) && (gfx_id < 7000)) || (gfx_id >= 8000)) as i32;
        int_id = gfx_id % 1000;
    }
    src_kind = if copy != 0 { (*fp).u.kb.hat.kind as i32 } else { kind };
    if (src_kind < 0) || (src_kind >= (Ft_Kind_Max as i32)) {
        return (-1_i32);
    }
    bank = ext::ftData_UnkBytePerCharacter[src_kind as usize] as i32;
    if ((bank < 0) || (bank >= 65)) || (bank == 255) {
        efAsync_MexMiss(kind, gfx_id, (c"no effect bank".as_ptr() as *mut i8) as *const i8);
        return (-1_i32);
    }
    bhv = efAsync_MexBhv[bank as usize];
    if (bhv.is_null()) || (efAsync_DatEntries[bank as usize].data.is_null()) {
        if !((kind == (Ft_Kind_Kirby as i32)) || (((kind >= (Ft_Kind_Mex0 as i32)) && (kind < (Ft_Kind_Max as i32))) && (ext::ftKb_MexKirbyLike(kind) != 0))) {
            efAsync_MexMiss(kind, gfx_id, (c"its effect bank has no effBehaviorTable (not loaded?)".as_ptr() as *mut i8) as *const i8);
        }
        return (-1_i32);
    }
    if int_id >= (if (*is_ptcl) != 0 { (*bhv).ptcl_num } else { (*bhv).mdl_num }) {
        efAsync_MexMiss(kind, gfx_id, (c"past the bank's behaviour table".as_ptr() as *mut i8) as *const i8);
        return (-1_i32);
    }
    r#type = (*(if (*is_ptcl) != 0 { (*bhv).ptcl_type } else { (*bhv).mdl_type }).offset(int_id as isize)) as i32;
    *final_id = (bank * 1000) + int_id;
    *owner = gobj;
    {
        // the C's sprintf of the diagnostic line (efAsync_MexMiss logs it once per key)
        let file = if efAsync_DatEntries[bank as usize].ef_DAT_file.is_null() { std::borrow::Cow::Borrowed("(null)") } else { core::ffi::CStr::from_ptr(efAsync_DatEntries[bank as usize].ef_DAT_file as *const core::ffi::c_char).to_string_lossy() };
        let line = format!("drawn as bank {} ({}) {} {}, behaviour {}, in motion {}", bank, file, if *is_ptcl != 0 { "generator" } else { "model" }, int_id, r#type, (*fp).motion_id);
        let n = line.len().min(127);
        for (k, b) in line.bytes().take(n).enumerate() { efAsync_MexResolve_buf[k] = b as i8; }
        efAsync_MexResolve_buf[n] = 0;
        efAsync_MexMiss(kind, gfx_id + 100000, (efAsync_MexResolve_buf.as_mut_ptr()) as *const i8);
    }
    return r#type;
}

/// `efAsync_MexSpawn`
pub unsafe extern "C" fn efAsync_MexSpawn(mut gobj: *mut HSD_GObj, mut queue_head: *mut c_void, mut gfx_id: i32, mut jobj: *mut HSD_JObj, mut offset: *mut Vec, mut facing: f32, mut orientation: f32) {
    melee_core::ct!("efAsync_MexSpawn");
    let mut q: *mut EF_QueuedEffect = null_mut();
    let mut final_id: i32 = 0;
    let mut is_ptcl: i32 = 0;
    let mut owner: *mut HSD_GObj = null_mut();
    if efAsync_MexResolve(gobj, gfx_id, core::ptr::addr_of_mut!(final_id) as *mut i32, core::ptr::addr_of_mut!(is_ptcl) as *mut i32, core::ptr::addr_of_mut!(owner) as *mut *mut HSD_GObj) < 0 {
        return;
    }
    q = ext::HSD_ObjAlloc(core::ptr::addr_of_mut!(efAsync_AllocData) as *mut _HSD_ObjAllocData) as *mut EF_QueuedEffect;
    if q.is_null() {
        return;
    }
    (*q).spawn_kind = 9 as u8;
    (*q).gfx_id = gfx_id;
    (*q).jobj = jobj;
    (*q).params = *offset;
    (*q).extra1 = facing;
    (*q).extra2 = orientation;
    if (!ext::HSD_GObj_CurrentInvokedProc.is_null()) && (((*ext::HSD_GObj_CurrentInvokedProc).s_link as u32) < 9_u32) {
        (*q).next = (*(queue_head as *mut EF_QueuedEffect)).next;
        (*(queue_head as *mut EF_QueuedEffect)).next = q;
        return;
    }
    efAsync_QueueProcessDeferred(gobj, q);
}

/// `efAsync_MexProcessDeferred`
pub unsafe extern "C" fn efAsync_MexProcessDeferred(mut gobj: *mut HSD_GObj, mut q: *mut EF_QueuedEffect) {
    melee_core::ct!("efAsync_MexProcessDeferred");
    let mut pos: Vec3 = unsafe { core::mem::zeroed() };
    let mut final_id: i32 = 0;
    let mut is_ptcl: i32 = 0;
    let mut r#type: i32 = 0;
    let mut owner: *mut HSD_GObj = null_mut();
    let mut id: i32 = (*q).gfx_id;
    r#type = efAsync_MexResolve(gobj, id, core::ptr::addr_of_mut!(final_id) as *mut i32, core::ptr::addr_of_mut!(is_ptcl) as *mut i32, core::ptr::addr_of_mut!(owner) as *mut *mut HSD_GObj);
    if r#type < 0 {
        return;
    }
    if !(is_ptcl != 0) {
        's29: {
            match r#type {
                0 | 1 | 2 | 3 | 4 => {
                    ext::lb_8000B1CC((*q).jobj, core::ptr::addr_of_mut!((*q).params) as *mut Vec, core::ptr::addr_of_mut!(pos) as *mut Vec);
                    let _ = ext::efSync_Spawn(id, gobj, &[core::ptr::addr_of_mut!(pos) as *mut Vec as *const c_void, core::ptr::addr_of_mut!((*q).extra1) as *mut f32 as *const c_void, core::ptr::addr_of_mut!((*q).extra2) as *mut f32 as *const c_void]);
                }
                5 | 6 => {
                    let _ = ext::efSync_Spawn(id, gobj, &[(*q).jobj as *const c_void]);
                }
                _ => {
                }
            }
        }
        return;
    }
    's30: {
        match r#type {
            0 => {
                ext::lb_8000B1CC((*q).jobj, core::ptr::addr_of_mut!((*q).params) as *mut Vec, core::ptr::addr_of_mut!(pos) as *mut Vec);
                let _ = ext::efSync_Spawn(id, gobj, &[core::ptr::addr_of_mut!(pos) as *mut Vec as *const c_void, core::ptr::addr_of_mut!((*q).extra1) as *mut f32 as *const c_void, core::ptr::addr_of_mut!((*q).extra2) as *mut f32 as *const c_void]);
            }
            3 => {
                ext::lb_8000B1CC((*q).jobj, core::ptr::addr_of_mut!((*q).params) as *mut Vec, core::ptr::addr_of_mut!(pos) as *mut Vec);
                let _ = ext::efSync_Spawn(id, gobj, &[core::ptr::addr_of_mut!(pos) as *mut Vec as *const c_void, core::ptr::addr_of_mut!((*q).extra1) as *mut f32 as *const c_void]);
            }
            5 | 7 => {
                let _ = ext::efSync_Spawn(id, gobj, &[(*q).jobj as *const c_void]);
            }
            6 => {
                let _ = ext::efSync_Spawn(id, gobj, &[(*q).jobj as *const c_void, core::ptr::addr_of_mut!((*q).extra1) as *mut f32 as *const c_void]);
            }
            _ => {
            }
        }
    }
}

/// `HSD_GObjGetHSDObj`
pub unsafe extern "C" fn HSD_GObjGetHSDObj(mut gobj: *mut HSD_GObj) -> *mut c_void {
    melee_core::ct!("HSD_GObjGetHSDObj");
    return (*gobj).hsd_obj;
}

/// `HSD_JObjSetRotationZ`
pub unsafe extern "C" fn HSD_JObjSetRotationZ(mut jobj: *mut HSD_JObj, mut z: f32) {
    melee_core::ct!("HSD_JObjSetRotationZ");
    if !jobj.is_null() { (); } else { panic!("HSD_JObjSetRotationZ: assertion failed"); }
    if !(((*jobj).flags & ((1 << 17) as u32)) != 0) { (); } else { panic!("HSD_JObjSetRotationZ: assertion failed"); }
    (*jobj).rotate.z = z;
    if !(((*jobj).flags & ((1 << 25) as u32)) != 0) {
        {
            if (!jobj.is_null()) && (!(HSD_JObjMtxIsDirty(jobj) != 0)) {
                ext::HSD_JObjSetMtxDirtySub(jobj);
            }
        }
    }
}

/// `HSD_JObjSetScaleX`
pub unsafe extern "C" fn HSD_JObjSetScaleX(mut jobj: *mut HSD_JObj, mut x: f32) {
    melee_core::ct!("HSD_JObjSetScaleX");
    if !jobj.is_null() { (); } else { panic!("HSD_JObjSetScaleX: assertion failed"); }
    (*jobj).scale.x = x;
    if !(((*jobj).flags & ((1 << 25) as u32)) != 0) {
        {
            if (!jobj.is_null()) && (!(HSD_JObjMtxIsDirty(jobj) != 0)) {
                ext::HSD_JObjSetMtxDirtySub(jobj);
            }
        }
    }
}

/// `HSD_JObjSetScaleY`
pub unsafe extern "C" fn HSD_JObjSetScaleY(mut jobj: *mut HSD_JObj, mut y: f32) {
    melee_core::ct!("HSD_JObjSetScaleY");
    if !jobj.is_null() { (); } else { panic!("HSD_JObjSetScaleY: assertion failed"); }
    (*jobj).scale.y = y;
    if !(((*jobj).flags & ((1 << 25) as u32)) != 0) {
        {
            if (!jobj.is_null()) && (!(HSD_JObjMtxIsDirty(jobj) != 0)) {
                ext::HSD_JObjSetMtxDirtySub(jobj);
            }
        }
    }
}

/// `HSD_JObjSetScaleZ`
pub unsafe extern "C" fn HSD_JObjSetScaleZ(mut jobj: *mut HSD_JObj, mut z: f32) {
    melee_core::ct!("HSD_JObjSetScaleZ");
    if !jobj.is_null() { (); } else { panic!("HSD_JObjSetScaleZ: assertion failed"); }
    (*jobj).scale.z = z;
    if !(((*jobj).flags & ((1 << 25) as u32)) != 0) {
        {
            if (!jobj.is_null()) && (!(HSD_JObjMtxIsDirty(jobj) != 0)) {
                ext::HSD_JObjSetMtxDirtySub(jobj);
            }
        }
    }
}

/// `HSD_JObjSetScale`
pub unsafe extern "C" fn HSD_JObjSetScale(mut jobj: *mut HSD_JObj, mut scale: *mut Vec) {
    melee_core::ct!("HSD_JObjSetScale");
    if !jobj.is_null() { (); } else { panic!("HSD_JObjSetScale: assertion failed"); }
    if !scale.is_null() { (); } else { panic!("HSD_JObjSetScale: assertion failed"); }
    (*jobj).scale = *scale;
    if !(((*jobj).flags & ((1 << 25) as u32)) != 0) {
        {
            if (!jobj.is_null()) && (!(HSD_JObjMtxIsDirty(jobj) != 0)) {
                ext::HSD_JObjSetMtxDirtySub(jobj);
            }
        }
    }
}

/// `HSD_JObjSetRotationY`
pub unsafe extern "C" fn HSD_JObjSetRotationY(mut jobj: *mut HSD_JObj, mut y: f32) {
    melee_core::ct!("HSD_JObjSetRotationY");
    if !jobj.is_null() { (); } else { panic!("HSD_JObjSetRotationY: assertion failed"); }
    if !(((*jobj).flags & ((1 << 17) as u32)) != 0) { (); } else { panic!("HSD_JObjSetRotationY: assertion failed"); }
    (*jobj).rotate.y = y;
    if !(((*jobj).flags & ((1 << 25) as u32)) != 0) {
        {
            if (!jobj.is_null()) && (!(HSD_JObjMtxIsDirty(jobj) != 0)) {
                ext::HSD_JObjSetMtxDirtySub(jobj);
            }
        }
    }
}

/// `HSD_JObjGetScale`
pub unsafe extern "C" fn HSD_JObjGetScale(mut jobj: *mut HSD_JObj, mut scale: *mut Vec) {
    melee_core::ct!("HSD_JObjGetScale");
    if !jobj.is_null() { (); } else { panic!("HSD_JObjGetScale: assertion failed"); }
    if !scale.is_null() { (); } else { panic!("HSD_JObjGetScale: assertion failed"); }
    *scale = (*jobj).scale;
}

/// `HSD_JObjGetChild`
pub unsafe extern "C" fn HSD_JObjGetChild(mut jobj: *mut HSD_JObj) -> *mut HSD_JObj {
    melee_core::ct!("HSD_JObjGetChild");
    if jobj.is_null() {
        return null_mut();
    } else {
        return (*jobj).child;
    }
}

/// `HSD_JObjGetNext`
pub unsafe extern "C" fn HSD_JObjGetNext(mut jobj: *mut HSD_JObj) -> *mut HSD_JObj {
    melee_core::ct!("HSD_JObjGetNext");
    if jobj.is_null() {
        return null_mut();
    } else {
        return (*jobj).next;
    }
}

/// `HSD_JObjGetRotationY`
pub unsafe extern "C" fn HSD_JObjGetRotationY(mut jobj: *mut HSD_JObj) -> f32 {
    melee_core::ct!("HSD_JObjGetRotationY");
    if !jobj.is_null() { (); } else { panic!("HSD_JObjGetRotationY: assertion failed"); }
    return (*jobj).rotate.y;
}

/// `HSD_GObjGetUserData`
pub unsafe extern "C" fn HSD_GObjGetUserData(mut gobj: *mut HSD_GObj) -> *mut c_void {
    melee_core::ct!("HSD_GObjGetUserData");
    return (*gobj).user_data;
}

/// `HSD_JObjMtxIsDirty`
pub unsafe extern "C" fn HSD_JObjMtxIsDirty(mut jobj: *mut HSD_JObj) -> i32 {
    melee_core::ct!("HSD_JObjMtxIsDirty");
    let mut result: i32 = 0;
    if !jobj.is_null() { (); } else { panic!("HSD_JObjMtxIsDirty: assertion failed"); }
    result = 0;
    if (!(((*jobj).flags & ((1 << 23) as u32)) != 0)) && (((*jobj).flags & ((1 << 6) as u32)) != 0) {
        result = 1;
    }
    return result;
}
