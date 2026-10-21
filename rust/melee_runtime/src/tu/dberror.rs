//! Port of src/melee/db/dberror.c (FPU exception clear and the crash handlers that print the
//! build stamp and a stack trace).
//!
//! The TU has no globals; calls out of it go through [`DbErrorHost`], whose default
//! implementation [`ext::Ext`] is the unported-callee stub set (panics, named like the C).
//! The handlers the C registers by address are passed to the host as Rust fn items
//! ([`fn_HSDPanicHandler`], [`fn_OSErrorHandler`]).
#![allow(non_snake_case)]

use core::ffi::c_void;
use melee_types::OSContext;

use super::dbinit::DB_BUILD_TIMESTAMP;

/// The handler signatures the host registers.
pub type DbPanicHandler = fn(&mut dyn DbErrorHost, *mut OSContext);
pub type DbErrorHandler = fn(&mut dyn DbErrorHost, u16, *mut OSContext, i32, i32);

/// Callees of dberror.c.
pub trait DbErrorHost {
    fn PPCMfmsr(&mut self) -> u32;
    fn PPCMtmsr(&mut self, v: u32);
    fn OSGetCurrentContext(&mut self) -> *mut OSContext;
    fn OSSaveFPUContext(&mut self, ctx: *mut OSContext);
    fn OSLoadFPUContext(&mut self, ctx: *mut OSContext);
    fn HSD_VISetUserPreRetraceCallback(&mut self);
    fn HSD_VISetUserPostRetraceCallback(&mut self);
    fn lb_80019A48(&mut self);
    fn OSReport(&mut self, s: &str);
    fn Exception_ReportStackTrace(&mut self, ctx: *mut OSContext, depth: i32);
    fn Exception_ReportCodeline(&mut self, error: u16, dsisr: i32, dar: i32, ctx: *mut OSContext);
    fn hsd_80397DFC(&mut self, v: i32);
    fn Exception_StoreDebugLevel(&mut self, level: i32);
    fn hsd_80397DA4(&mut self, ctx: *mut OSContext);
    fn DbLevel(&mut self) -> i32;
    fn DBIsDebuggerPresent(&mut self) -> i32;
    fn OSAllocFromArenaLo(&mut self, size: u32, align: u32) -> *mut c_void;
    fn hsd_80393DA0(&mut self, mem: *mut c_void, size: u32);
    fn HSD_SetPanicCallback(&mut self, cb: DbPanicHandler);
    fn OSSetErrorHandler(&mut self, error: u16, cb: DbErrorHandler);
}

pub mod ext {
    use super::*;
    /// Unported callees (TODO: rewire once their TUs are ported).
    pub struct Ext;
    impl DbErrorHost for Ext {
        fn PPCMfmsr(&mut self) -> u32 { todo!("PPCMfmsr (pc/platform)") }
        fn PPCMtmsr(&mut self, _: u32) { todo!("PPCMtmsr (pc/platform)") }
        fn OSGetCurrentContext(&mut self) -> *mut OSContext { todo!("OSGetCurrentContext (pc/platform)") }
        fn OSSaveFPUContext(&mut self, _: *mut OSContext) { todo!("OSSaveFPUContext (pc/platform)") }
        fn OSLoadFPUContext(&mut self, _: *mut OSContext) { todo!("OSLoadFPUContext (pc/platform)") }
        fn HSD_VISetUserPreRetraceCallback(&mut self) { todo!("HSD_VISetUserPreRetraceCallback (baselib/video.c)") }
        fn HSD_VISetUserPostRetraceCallback(&mut self) { todo!("HSD_VISetUserPostRetraceCallback (baselib/video.c)") }
        fn lb_80019A48(&mut self) { todo!("lb_80019A48 (lb/lb_0195.c)") }
        fn OSReport(&mut self, _: &str) { todo!("OSReport (pc/platform)") }
        fn Exception_ReportStackTrace(&mut self, _: *mut OSContext, _: i32) { todo!("Exception_ReportStackTrace (baselib/debugconsole_main.c)") }
        fn Exception_ReportCodeline(&mut self, _: u16, _: i32, _: i32, _: *mut OSContext) { todo!("Exception_ReportCodeline (baselib/debugconsole_main.c)") }
        fn hsd_80397DFC(&mut self, _: i32) { todo!("hsd_80397DFC (baselib)") }
        fn Exception_StoreDebugLevel(&mut self, _: i32) { todo!("Exception_StoreDebugLevel (baselib)") }
        fn hsd_80397DA4(&mut self, _: *mut OSContext) { todo!("hsd_80397DA4 (baselib)") }
        fn DbLevel(&mut self) -> i32 { todo!("DbLevel (db/dbinit.c)") }
        fn DBIsDebuggerPresent(&mut self) -> i32 { todo!("DBIsDebuggerPresent (pc/platform)") }
        fn OSAllocFromArenaLo(&mut self, _: u32, _: u32) -> *mut c_void { todo!("OSAllocFromArenaLo (pc/platform)") }
        fn hsd_80393DA0(&mut self, _: *mut c_void, _: u32) { todo!("hsd_80393DA0 (baselib/hsd_393C.c)") }
        fn HSD_SetPanicCallback(&mut self, _: DbPanicHandler) { todo!("HSD_SetPanicCallback (baselib/debug.c)") }
        fn OSSetErrorHandler(&mut self, _: u16, _: DbErrorHandler) { todo!("OSSetErrorHandler (pc/platform)") }
    }
}

/// `db_ClearFPUExceptions` (0x80228A64): enable FP + FE0 in the MSR and clear the FPSCR's
/// exception/status bits (keep the low 20).
///
/// # Safety
/// `OSGetCurrentContext` returns a live context.
pub unsafe fn db_ClearFPUExceptions(h: &mut dyn DbErrorHost) {
    let m = h.PPCMfmsr();
    h.PPCMtmsr(m | 0x900);
    let ctx = h.OSGetCurrentContext();
    h.OSSaveFPUContext(ctx);
    (*ctx).fpscr &= 0xFFFFF;
    h.OSLoadFPUContext(ctx);
}

/// `fn_HSDPanicHandler` (0x80228AB4).
pub fn fn_HSDPanicHandler(h: &mut dyn DbErrorHost, ctx: *mut OSContext) {
    h.HSD_VISetUserPreRetraceCallback();
    h.HSD_VISetUserPostRetraceCallback();
    h.lb_80019A48();
    h.OSReport(&format!("{}\n", DB_BUILD_TIMESTAMP));
    h.Exception_ReportStackTrace(ctx, 0x10);
    h.hsd_80397DFC(0x1388);
    let lvl = h.DbLevel();
    h.Exception_StoreDebugLevel(lvl);
    h.hsd_80397DA4(ctx);
}

/// `fn_OSErrorHandler` (0x80228B28); the C's two varargs are `dsisr` and `dar`.
pub fn fn_OSErrorHandler(h: &mut dyn DbErrorHost, error: u16, ctx: *mut OSContext, dsisr: i32, dar: i32) {
    h.HSD_VISetUserPreRetraceCallback();
    h.HSD_VISetUserPostRetraceCallback();
    h.lb_80019A48();
    h.OSReport(&format!("{}\n", DB_BUILD_TIMESTAMP));
    h.Exception_ReportStackTrace(ctx, 0x10);
    h.Exception_ReportCodeline(error, dsisr, dar, ctx);
    h.hsd_80397DFC(0x1388);
    let lvl = h.DbLevel();
    h.Exception_StoreDebugLevel(lvl);
    h.hsd_80397DA4(ctx);
}

/// `db_SetupCrashHandler` (0x80228C4C): without a debugger, install the handlers for every OS
/// error except 4, 7, 8 and 9.
pub fn db_SetupCrashHandler(h: &mut dyn DbErrorHost) {
    if h.DBIsDebuggerPresent() == 0 {
        let mem = h.OSAllocFromArenaLo(0x2000, 4);
        h.hsd_80393DA0(mem, 0x2000);
        h.HSD_SetPanicCallback(fn_HSDPanicHandler);
        for x in 0u16..16 {
            match x {
                4 | 7 | 8 | 9 => {}
                _ => h.OSSetErrorHandler(x, fn_OSErrorHandler),
            }
        }
    }
}
