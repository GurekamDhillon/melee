//! object.c class initialization, over the generated HSD class records.
#![allow(non_snake_case, non_upper_case_globals)]
use melee_types::{HSD_ClassInfo, HSD_ObjInfo, HSD_Obj};
pub static mut hsdObj: HSD_ClassInfo = unsafe {
    let mut info: HSD_ClassInfo = core::mem::zeroed();
    info.head.info_init = Some(ObjInfoInit);
    info
};
pub mod ext {
    use super::*;
    pub static mut hsdClass: HSD_ClassInfo = unsafe { core::mem::zeroed() };
    pub static mut hsdInitClassInfo_hook: Option<unsafe fn(*mut HSD_ClassInfo, *mut HSD_ClassInfo, *const i8, *const i8, i32, i32)> = None;
    pub unsafe fn hsdInitClassInfo(c: *mut HSD_ClassInfo, p: *mut HSD_ClassInfo, lib: *const i8, name: *const i8, info: i32, obj: i32) {
        hsdInitClassInfo_hook.expect("TODO hsdInitClassInfo")(c, p, lib, name, info, obj);
    }
}
pub unsafe extern "C" fn ObjInfoInit() {
    ext::hsdInitClassInfo(core::ptr::addr_of_mut!(hsdObj), core::ptr::addr_of_mut!(ext::hsdClass),
        b"sysdolphin_base_library\0".as_ptr().cast(), b"hsd_obj\0".as_ptr().cast(),
        core::mem::size_of::<HSD_ObjInfo>() as i32, core::mem::size_of::<HSD_Obj>() as i32);
}
