"""Shared pieces of the gr/if specs."""
# Mex_GrBind(fn): the native pointer passes straight through (an m-ex guest address would be bound)
MEX_GRBIND = {
    "c": 'void* Mex_GrBind(void* a0) { tuo_call_begin("Mex_GrBind"); tuo_arg_p(a0); tuo_call_end(); return a0; }',
    "rust": 'unsafe extern "C" fn s_Mex_GrBind(a0: *mut core::ffi::c_void) -> *mut core::ffi::c_void { let v = vv(); v.call_begin("Mex_GrBind"); v.arg_p(a0 as usize); v.call_end(); a0 }',
    "ext": ('    pub static mut H_Mex_GrBind: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> *mut core::ffi::c_void> = None;\n'
            '    pub unsafe extern "C" fn Mex_GrBind(a0: *mut core::ffi::c_void) -> *mut core::ffi::c_void { match H_Mex_GrBind { Some(f) => f(a0), None => a0 } }'),
}
