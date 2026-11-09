//! The `OSReport(fmt, ...)` hooks that spell the C varargs as fixed parameters (sislib, sobjlib, synth) must format, never
//! panic: a plain `cast` of the pointer into `os_error::OSReport`'s `&[usize]` slice was a size mismatch that crashed the
//! Envoy demo (link cast site 79202, a sislib error report). gen_link.py `osreport_adapter` builds the slice instead.
use std::sync::Mutex;

static OUT: Mutex<Vec<u8>> = Mutex::new(Vec::new());

fn sink(b: &[u8]) {
    OUT.lock().unwrap().extend_from_slice(b);
}

#[test]
fn varargs_osreport_hooks_format_and_do_not_panic() {
    melee_link::install();
    unsafe { melee_runtime::tu::os_error::OSREPORT_SINK = Some(sink) };
    let name = b"SdMenu.dat\0";
    melee_runtime::tu::sislib::ext::OSReport(b"Cannot open archive %s.\n\0".as_ptr() as *mut _, name.as_ptr() as *mut _);
    melee_runtime::tu::sislib::ext::OSReport(b"unknow camera\n\0".as_ptr() as *mut _, core::ptr::null_mut());
    melee_runtime::tu::sobjlib::ext::OSReport(b"sobj %s\n\0".as_ptr() as *mut _, name.as_ptr() as *mut _);
    melee_runtime::tu::synth::ext::OSReport("[NODEIDX] bad index=%d\n", -3);
    let out = String::from_utf8(OUT.lock().unwrap().clone()).unwrap();
    assert_eq!(out, "Cannot open archive SdMenu.dat.\nunknow camera\nsobj SdMenu.dat\n[NODEIDX] bad index=-3\n");
}
