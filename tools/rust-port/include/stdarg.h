/* Transpile-only override of src/MSL/stdarg.h (tools/rust-port/gen_cc.py puts this dir first).
 * MSL's TARGET_PC va_arg goes through the gwtool __va_arg shim and __builtin_types_compatible_p,
 * which c2rust 0.22 cannot translate ("no entry found for key"). Natively the plain builtins are
 * right: c2rust turns them into Rust C-variadics (VaList::arg). */
#ifndef _STDARG_H_
#define _STDARG_H_
typedef __builtin_va_list va_list;
#define va_start(ap, fmt) __builtin_va_start(ap, fmt)
#define va_end(ap) __builtin_va_end(ap)
#define va_arg(ap, t) __builtin_va_arg(ap, t)
#define va_copy(d, s) __builtin_va_copy(d, s)
#endif
