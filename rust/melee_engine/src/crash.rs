//! Debug aid: on an access violation print the faulting address and a symbolized backtrace (Windows).
//! Enabled with `MELEE_CRASH_TRACE=1` through `crash::install()` (called by `Host::boot`).

#[cfg(windows)]
mod sys {
    use core::ffi::c_void;
    #[repr(C)]
    struct ExceptionRecord {
        code: u32,
        flags: u32,
        record: *mut ExceptionRecord,
        address: *mut c_void,
        nparams: u32,
        info: [usize; 15],
    }
    #[repr(C)]
    struct ExceptionPointers {
        record: *mut ExceptionRecord,
        context: *mut c_void,
    }
    extern "system" {
        fn AddVectoredExceptionHandler(first: u32, h: unsafe extern "system" fn(*mut ExceptionPointers) -> i32) -> *mut c_void;
    }
    unsafe extern "system" fn handler(p: *mut ExceptionPointers) -> i32 {
        let r = &*(*p).record;
        if r.code == 0xC000_0005 || r.code == 0xC000_001D || r.code == 0xC000_00FD {
            eprintln!(
                "CRASH code {:#x} at {:p}, access kind {} addr {:#x}",
                r.code,
                r.address,
                r.info[0],
                r.info[1]
            );
            eprintln!("{}", std::backtrace::Backtrace::force_capture());
        }
        0 // EXCEPTION_CONTINUE_SEARCH
    }
    pub fn install() {
        unsafe {
            AddVectoredExceptionHandler(1, handler);
        }
    }
}

pub fn install() {
    #[cfg(windows)]
    if std::env::var_os("MELEE_CRASH_TRACE").is_some() {
        sys::install();
    }
}
