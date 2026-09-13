#ifndef _STDARG_H_
#define _STDARG_H_

#ifdef TARGET_PC

/* The port cannot use MSL's va_list. Its layout is the PowerPC one (register counters plus two
 * area pointers), filled by __builtin_va_info, and gwtool retargets the code to x86 long after
 * the frontend has committed to that shape -- so the counters describe a register save area that
 * does not exist and the game reads zeros.
 *
 * What does survive the retarget is llvm.va_start: it is an intrinsic, so the *x86* backend
 * lowers it, and it writes the address of the first variadic argument into the va_list object
 * (verified: `leal 20(%esp), %eax; movl %eax, (%esp)`). __builtin_va_arg is no use -- the PPC
 * frontend expands that inline before gwtool ever sees it -- so the stepping is done by the
 * native __va_arg shim instead, which reads the pointer at offset 0 natively.
 *
 * Nothing in game code may touch the va_list itself: the backend writes it natively and gwtool
 * would byte-swap any load of it. Same hazard as jmp_buf. Only __va_arg looks inside. */
typedef __builtin_va_list va_list;

/* Low 16 bits: the argument's size in bytes. Above that, how to interpret it -- the caller
 * promoted floats to double per the usual variadic rules, so asking for an f32 has to read eight
 * bytes and narrow, which sizeof alone cannot express. */
#define _GW_VA_FLOAT 0x10000u

void* __va_arg(va_list v_list, unsigned info);

#define va_start(ap, fmt) __builtin_va_start(ap, fmt)
#define va_end(ap) __builtin_va_end(ap)
#define va_arg(ap, t)                                                         \
    (*((t*) __va_arg(ap, (unsigned) sizeof(t) |                               \
                             (__builtin_types_compatible_p(t, float) ||       \
                                      __builtin_types_compatible_p(t, double) \
                                  ? _GW_VA_FLOAT                              \
                                  : 0u))))

#else

typedef struct {
    char gpr;
    char fpr;
    char reserved[2];
    char* input_arg_area;
    char* reg_save_area;
} __va_list[1];
typedef __va_list va_list;

extern void __builtin_va_info(void*);

void* __va_arg(va_list v_list, unsigned char type);

#ifndef __MWERKS__
#define _var_arg_typeof(e) 0
#endif

#define va_start(ap, fmt) ((void) fmt, __builtin_va_info(&ap))
#define va_arg(ap, t) (*((t*) __va_arg(ap, _var_arg_typeof(t))))
#define va_end(ap) (void) 0

#endif

#endif
