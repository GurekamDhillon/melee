//! Port of src/sysdolphin/baselib/debug.c: the log hook (`HSD_LogInit` swaps the C library's
//! stdout write procedure for [`report_func`], which forwards to a report callback first),
//! `__assert` / `HSD_Panic` (save the context, report, call the panic callback, `OSPanic`) and the
//! two callback setters. Callees of unported TUs are the hooks in [`ext`].
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs)]

use core::ffi::{c_char, CStr};
use melee_types::{OSContext, PanicCallback, ReportCallback, __file_handle, __idle_proc, __io_proc};

pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
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
    fn OSReport(s: &str);
    fn OSSaveContext(a0: *mut OSContext) -> i32;
    fn OSPanic(file: &str, line: i32, msg: &str);
    // `stdout->write_proc` (MSL ansi_files.c `stdout` FILE, read).
    fn stdout_write_proc() -> __io_proc;
    // `stdout->write_proc = p; stdout->state.error = 0;`
    fn stdout_set_write_proc_clear_error(p: __io_proc);
    }
}

/// `struct DebugContext` (the saved context plus 0x10 bytes of tail).
#[repr(C)]
pub struct DebugContext {
    pub context: OSContext,
    pub unk: [u8; 0x10],
}

/// `HSD_Debug_804C2608`.
pub static mut HSD_Debug_804C2608: DebugContext = unsafe { core::mem::zeroed() };

static mut reportCallback: ReportCallback = None;
static mut panicCallback: PanicCallback = None;
static mut logFunc: __io_proc = None;

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
    if let Some(cb) = reportCallback {
        cb(arg1, *arg2);
    }
    logFunc.expect("report_func: logFunc is NULL")(arg0, arg1, arg2, arg3);
    0
}

/// `HSD_LogInit`.
pub fn HSD_LogInit() {
    unsafe {
        if logFunc.is_none() {
            logFunc = ext::stdout_write_proc();
        }
        ext::stdout_set_write_proc_clear_error(Some(report_func));
    }
}

/// `__assert`.
///
/// # Safety
/// `str` and `arg2` are NUL-terminated (or NULL).
pub unsafe fn __assert(str: *const c_char, arg1: u32, arg2: *const c_char) {
    ext::OSReport(&format!("assertion \"{}\" failed", cstr(arg2)));
    HSD_Panic(str, arg1, c"".as_ptr());
}

/// `HSD_Panic`.
///
/// # Safety
/// `arg0` and `arg2` are NUL-terminated (or NULL).
pub unsafe fn HSD_Panic(arg0: *const c_char, line: u32, arg2: *const c_char) {
    if let Some(cb) = panicCallback {
        ext::OSSaveContext(&mut HSD_Debug_804C2608.context);
        ext::OSReport(&format!("{} in {} on line {}.\n", cstr(arg2), cstr(arg0), line as i32));
        cb(&mut HSD_Debug_804C2608.context);
    }
    ext::OSPanic(&cstr(arg0), line as i32, &cstr(arg2));
}

/// `HSD_SetReportCallback`.
pub fn HSD_SetReportCallback(cb: ReportCallback) {
    unsafe { reportCallback = cb };
}

/// `HSD_SetPanicCallback`.
pub fn HSD_SetPanicCallback(cb: PanicCallback) {
    unsafe { panicCallback = cb };
}
