//! object.c class initialization, over the generated HSD class records.
#![allow(non_snake_case, non_upper_case_globals)]
#[allow(unused_imports)]
use melee_types::*;
/// Tests (C oracle) run on the `ext::hsdClass` copy; the engine uses the one `class::hsdClass`.
pub static mut use_ext_data: bool = false;
unsafe fn hsd_class_ptr() -> *mut HSD_ClassInfo { if use_ext_data { core::ptr::addr_of_mut!(ext::hsdClass) } else { core::ptr::addr_of_mut!(crate::tu::class::hsdClass) } }
pub static mut hsdObj: HSD_ClassInfo = unsafe {
    let mut info: HSD_ClassInfo = core::mem::zeroed();
    info.head.info_init = Some(ObjInfoInit);
    info
};
// BEGIN GENERATED EXT (rust/melee_runtime/gen/mtu/gen_tu.py; the callees of the C TU)
pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    use core::ffi::c_void;
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? $(= $def:expr)?;)*) => {
            pub mod hook {
                use super::*;
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => crate::tu::hookmac::fallback!(stringify!($name) $(, $def)?),
                }
            } )*
        };
    }
    ext_fns! {
    fn hsdInitClassInfo(a0: *mut _HSD_ClassInfo, a1: *mut _HSD_ClassInfo, a2: *mut i8, a3: *mut i8, a4: i32, a5: i32);
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
    pub static mut hsdClass: HSD_ClassInfo = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT
pub unsafe extern "C" fn ObjInfoInit() {
    melee_core::ct!("ObjInfoInit");
    ext::hsdInitClassInfo(core::ptr::addr_of_mut!(hsdObj), hsd_class_ptr(),
        b"sysdolphin_base_library\0".as_ptr().cast_mut().cast(), b"hsd_obj\0".as_ptr().cast_mut().cast(),
        core::mem::size_of::<HSD_ObjInfo>() as i32, core::mem::size_of::<HSD_Obj>() as i32);
}
