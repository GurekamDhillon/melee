"""Shared pieces of the gr/if specs."""
# Mex_GrBind(fn): the native pointer passes straight through (an m-ex guest address would be bound)
MEX_GRBIND = {
    "c": 'void* Mex_GrBind(void* a0) { tuo_call_begin("Mex_GrBind"); tuo_arg_p(a0); tuo_call_end(); return a0; }',
    "rust": 'unsafe extern "C" fn s_Mex_GrBind(a0: *mut core::ffi::c_void) -> *mut core::ffi::c_void { let v = vv(); v.call_begin("Mex_GrBind"); v.arg_p(a0 as usize); v.call_end(); a0 }',
    "ext": ('    pub static mut H_Mex_GrBind: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> *mut core::ffi::c_void> = None;\n'
            '    pub unsafe extern "C" fn Mex_GrBind(a0: *mut core::ffi::c_void) -> *mut core::ffi::c_void { match H_Mex_GrBind { Some(f) => f(a0), None => a0 } }'),
}

# HSD_ForeachAnim(obj, type, mask, func, arg_type, ...): the variadic tail is a u32 for AOBJ_ARG_AU (3) and a
# double for AOBJ_ARG_AF (1); the port calls ext::HSD_ForeachAnim(obj, type, mask, func, arg_type, tail_bits).
HSD_FOREACHANIM = {
    "c": ('void HSD_ForeachAnim(void* a0, int a1, unsigned a2, void* a3, int a4, ...) { __builtin_va_list ap; __builtin_va_start(ap, a4); '
          'tuo_call_begin("HSD_ForeachAnim"); tuo_arg_p(a0); tuo_arg_i(a1); tuo_arg_u(a2); tuo_arg_p(a3); tuo_arg_i(a4); '
          'if (a4 == 1) { double d = __builtin_va_arg(ap, double); unsigned long long b; memcpy(&b, &d, 8); tuo_arg_f64(b); } '
          'else { tuo_arg_u(__builtin_va_arg(ap, unsigned)); } tuo_call_end(); __builtin_va_end(ap); }'),
    "rust": ('unsafe extern "C" fn s_HSD_ForeachAnim(a0: *mut core::ffi::c_void, a1: i32, a2: u32, a3: *mut core::ffi::c_void, a4: i32, a5: u64) { '
             'let v = vv(); v.call_begin("HSD_ForeachAnim"); v.arg_p(a0 as usize); v.arg_i(a1 as i64); v.arg_u(a2 as u64); v.arg_p(a3 as usize); '
             'v.arg_i(a4 as i64); if a4 == 1 { v.arg_f64(a5); } else { v.arg_u(a5 as u32 as u64); } v.call_end(); }'),
    "ext": ('    pub static mut H_HSD_ForeachAnim: Option<unsafe extern "C" fn(*mut core::ffi::c_void, i32, u32, *mut core::ffi::c_void, i32, u64)> = None;\n'
            '    pub unsafe extern "C" fn HSD_ForeachAnim(a0: *mut core::ffi::c_void, a1: i32, a2: u32, a3: *mut core::ffi::c_void, a4: i32, a5: u64) { match H_HSD_ForeachAnim { Some(f) => f(a0, a1, a2, a3, a4, a5), None => panic!("TODO: HSD_ForeachAnim is not ported") } }'),
}
