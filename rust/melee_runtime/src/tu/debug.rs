//! Port of src/sysdolphin/baselib/debug.c: the log hook (`HSD_LogInit` swaps the C library's
//! stdout write procedure for [`report_func`], which forwards to a report callback first),
//! `__assert` / `HSD_Panic` (save the context, report, call the panic callback, `OSPanic`) and the
//! two callback setters. Callees of unported TUs are the hooks in [`ext`].
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs)]

use core::ffi::{c_char, CStr};
use core::ptr::addr_of_mut;
use melee_types::{_IO_FILE, OSContext, PanicCallback, ReportCallback, __file_handle, __idle_proc, __io_proc};
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
    fn OSSaveContext(a0: *mut OSContext) -> u32;
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
    pub static mut __files: [_IO_FILE; 3] = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT

/// `struct DebugContext` (the saved context plus 0x10 bytes of tail).
#[repr(C)]
pub struct DebugContext {
    pub context: OSContext,
    pub unk: [u8; 0x10],
}

/// `HSD_Debug_804C2608`.
pub static mut HSD_Debug_804C2608: DebugContext = unsafe { core::mem::zeroed() };

pub static mut reportCallback: ReportCallback = None;
pub static mut panicCallback: PanicCallback = None;
pub static mut logFunc: __io_proc = None;

/// The `%s` argument as text.
unsafe fn cstr<'a>(p: *const c_char) -> std::borrow::Cow<'a, str> {
    if p.is_null() {
        return "(null)".into();
    }
    CStr::from_ptr(p).to_string_lossy()
}

/// `report_func` (static): the replacement stdout write procedure.
///
/// # Safety
/// `arg1` points at `*arg2` bytes; `logFunc` was set by [`HSD_LogInit`].
pub unsafe extern "C" fn report_func(
    arg0: __file_handle,
    arg1: *mut u8,
    arg2: *mut usize,
    arg3: __idle_proc,
) -> i32 {
    melee_core::ct!("report_func");
    if let Some(cb) = reportCallback {
        cb(arg1, *arg2);
    }
    logFunc.expect("report_func: logFunc is NULL")(arg0, arg1, arg2, arg3);
    0
}

/// `HSD_LogInit`.
pub fn HSD_LogInit() {
    melee_core::ct!("HSD_LogInit");
    unsafe {
        let out = addr_of_mut!(ext::__files).cast::<_IO_FILE>().add(1); // stdout
        if logFunc.is_none() {
            logFunc = (*out).write_proc;
        }
        (*out).write_proc = Some(report_func);
        (*out).state.error = 0;
    }
}

/// `OSPanic(file, line, msg)` (variadic, no return): settable hook, then the process panic.
pub static mut H_OSPanic: Option<fn(&str, i32, &str)> = None;
fn os_panic(file: &str, line: i32, msg: &str) -> ! {
    if let Some(f) = unsafe { H_OSPanic } {
        f(file, line, msg);
    }
    panic!("OSPanic: {file}:{line}: {msg}")
}

/// `__assert`.
///
/// # Safety
/// `str` and `arg2` are NUL-terminated (or NULL).
pub unsafe fn __assert(str: *const c_char, arg1: u32, arg2: *const c_char) {
    melee_core::ct!("__assert");
    crate::tu::lbext::OSReport("assertion \"%s\" failed", &[crate::tu::lbext::OsArg::S(arg2)]);
    HSD_Panic(str, arg1, c"".as_ptr());
}

/// `HSD_Panic`.
///
/// # Safety
/// `arg0` and `arg2` are NUL-terminated (or NULL).
pub unsafe fn HSD_Panic(arg0: *const c_char, line: u32, arg2: *const c_char) {
    melee_core::ct!("HSD_Panic");
    if let Some(cb) = panicCallback {
        ext::OSSaveContext(&mut HSD_Debug_804C2608.context);
        crate::tu::lbext::OSReport("%s in %s on line %d.
", &[crate::tu::lbext::OsArg::S(arg2), crate::tu::lbext::OsArg::S(arg0), crate::tu::lbext::OsArg::I(line as i32)]);
        cb(&mut HSD_Debug_804C2608.context);
    }
    os_panic(&cstr(arg0), line as i32, &cstr(arg2));
}

/// `HSD_SetReportCallback`.
pub fn HSD_SetReportCallback(cb: ReportCallback) {
    melee_core::ct!("HSD_SetReportCallback");
    unsafe { reportCallback = cb };
}

/// `HSD_SetPanicCallback`.
pub fn HSD_SetPanicCallback(cb: PanicCallback) {
    melee_core::ct!("HSD_SetPanicCallback");
    unsafe { panicCallback = cb };
}
