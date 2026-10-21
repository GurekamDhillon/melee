#![allow(non_snake_case)]
use core::ffi::c_void;
use melee_runtime::tu::dberror::{db_ClearFPUExceptions, db_SetupCrashHandler, DbErrorHandler, DbErrorHost, DbPanicHandler};
use melee_types::OSContext;

struct H { msr: u32, ctx: Box<OSContext>, log: Vec<String>, handlers: Vec<(u16, DbErrorHandler)>, panic: Option<DbPanicHandler>, dbg: i32 }
impl DbErrorHost for H {
    fn PPCMfmsr(&mut self) -> u32 { self.msr }
    fn PPCMtmsr(&mut self, v: u32) { self.msr = v }
    fn OSGetCurrentContext(&mut self) -> *mut OSContext { &mut *self.ctx }
    fn OSSaveFPUContext(&mut self, _: *mut OSContext) {}
    fn OSLoadFPUContext(&mut self, _: *mut OSContext) {}
    fn HSD_VISetUserPreRetraceCallback(&mut self) {}
    fn HSD_VISetUserPostRetraceCallback(&mut self) {}
    fn lb_80019A48(&mut self) {}
    fn OSReport(&mut self, s: &str) { self.log.push(s.into()) }
    fn Exception_ReportStackTrace(&mut self, _: *mut OSContext, d: i32) { self.log.push(format!("trace {d}")) }
    fn Exception_ReportCodeline(&mut self, e: u16, a: i32, b: i32, _: *mut OSContext) { self.log.push(format!("code {e} {a} {b}")) }
    fn hsd_80397DFC(&mut self, _: i32) {}
    fn Exception_StoreDebugLevel(&mut self, l: i32) { self.log.push(format!("lvl {l}")) }
    fn hsd_80397DA4(&mut self, _: *mut OSContext) {}
    fn DbLevel(&mut self) -> i32 { 1 }
    fn DBIsDebuggerPresent(&mut self) -> i32 { self.dbg }
    fn OSAllocFromArenaLo(&mut self, _: u32, _: u32) -> *mut c_void { core::ptr::null_mut() }
    fn hsd_80393DA0(&mut self, _: *mut c_void, s: u32) { assert_eq!(s, 0x2000) }
    fn HSD_SetPanicCallback(&mut self, cb: DbPanicHandler) { self.panic = Some(cb) }
    fn OSSetErrorHandler(&mut self, e: u16, cb: DbErrorHandler) { self.handlers.push((e, cb)) }
}

#[test]
fn clear_fpu_and_crash_handlers() {
    let mut c: OSContext = unsafe { core::mem::zeroed() };
    c.fpscr = 0xFFFF_FFFF;
    let mut h = H { msr: 0x30, ctx: Box::new(c), log: vec![], handlers: vec![], panic: None, dbg: 1 };
    unsafe { db_ClearFPUExceptions(&mut h) };
    assert_eq!(h.msr, 0x930);
    assert_eq!(h.ctx.fpscr, 0xFFFFF);
    db_SetupCrashHandler(&mut h);
    assert!(h.handlers.is_empty());
    h.dbg = 0;
    db_SetupCrashHandler(&mut h);
    let ids: Vec<u16> = h.handlers.iter().map(|x| x.0).collect();
    assert_eq!(ids, [0, 1, 2, 3, 5, 6, 10, 11, 12, 13, 14, 15]);
    let cb = h.handlers[0].1;
    cb(&mut h, 3, core::ptr::null_mut(), 7, 8);
    assert_eq!(h.log, ["DATE Feb 13 2002  TIME 22:06:27\n", "trace 16", "code 3 7 8", "lvl 1"]);
    (h.panic.unwrap())(&mut h, core::ptr::null_mut());
    assert_eq!(h.log.len(), 7);
}
