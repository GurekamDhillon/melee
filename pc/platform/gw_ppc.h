/* gw_ppc.h - a Gekko/PowerPC subset interpreter (Phase 1 of the m-ex PPC blob executor).
 *
 * This is NATIVE platform code (compiled directly with the i686 clang, NOT through the
 * gwtool retarget pipe). It reads/writes guest memory through the big-endian accessors
 * (gw_r32, gw_w32, ...) in gw.h, exactly like the other shims.
 *
 * Phase 1 scope: execute a hand-built PPC blob (fighter-callback-shaped) end to end, with a
 * single bridged call into a native helper. The full blob loader (Reloc/Overload), the build-time
 * guest->native table, and fighter integration are later phases.
 *
 * Design notes (see _research/mex-ppc-interpreter.md):
 *   - Guest MEM1 is mapped at native 0x80000000, so a guest address IS a native pointer; only
 *     multi-byte field access needs big-endian swapping (gw_r32/gw_w32/...).
 *   - r1 = guest stack pointer, r2 = guest rtoc (both caller-supplied).
 *   - FPRs are modeled as a 64-bit union so the paired-single ps_* ops can split one later.
 *   - CR is one 32-bit word of eight 4-bit fields; CR0 is the most-significant field.
 *   - Reentrant: gw_ppc_call saves/restores the whole machine, so native code reached through
 *     the bridge can call back in.
 */
#ifndef GW_PPC_H
#define GW_PPC_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* One floating-point register: a 64-bit container that doubles (for the single/double ops) and
 * a word/float pair (for the future paired-single ps_* split). */
typedef union gw_ppc_fpr {
    double d;        /* double-precision view (fadd/fsub/... ) */
    uint64_t u64;    /* raw 64-bit bit pattern (fctiwz result, lfd/stfd) */
    uint32_t u32[2]; /* two 32-bit halves */
    float f32[2];    /* two single-precision floats (ps_* later) */
} gw_ppc_fpr;

/* Guest CPU context. r0-r31 GPRs, f0-f31 FPRs, and the special-purpose registers. */
typedef struct gw_ppc_ctx {
    uint32_t gpr[32];
    gw_ppc_fpr fpr[32];
    uint32_t lr;  /* link register */
    uint32_t ctr; /* count register */
    uint32_t xer; /* fixed-point exception register (SO/OV/CA + byte count) */
    uint32_t cr;  /* condition register: 8 x 4-bit fields, CR0 = MSB field */
    uint32_t pc;  /* next instruction address */
} gw_ppc_ctx;

/* ---- native-call bridge seam ---------------------------------------------------------
 * When the interpreter branches/calls a target outside the blob's code range, it asks the
 * resolver for a native function pointer plus the target's calling signature. Integer arguments
 * come from r3..r10 in order; float arguments come from f1..f8, interleaved per the PowerPC ABI
 * (a float parameter consumes the next FPR, an integer/pointer parameter the next GPR). The
 * callee's returned word is stored back into r3; a float return (f1) is captured into FPR 1.
 * struct/sret returns ride the existing integer path: the guest already places the hidden sret
 * pointer in r3, so it is passed as argument 0 and the callee fills it big-endian. */

/* A native target, in the fixed callable shape the bridge marshals into. On i686 cdecl both a
 * float argument and a uint32 occupy four bytes on the stack, so a float argument's IEEE-754 bit
 * pattern passed as a uint32 lands in the callee's float slot verbatim. */
typedef uint32_t (*gw_ppc_native_fn)(uint32_t a0, uint32_t a1, uint32_t a2, uint32_t a3,
                                     uint32_t a4, uint32_t a5, uint32_t a6, uint32_t a7);

/* Set in `float_args` to mark the target VARIADIC. Argument slots are 0..7, so the mask's high
 * bits are free, and using one keeps the wire format of every existing signature table byte-for-
 * byte unchanged - a table that knows nothing about varargs keeps working untouched. With it set,
 * `n_args` counts only the FIXED parameters and the variadic tail is marshalled as described at
 * gw_ppc_bridge_call. */
#define GW_PPC_SIG_VARARGS 0x80000000u
#define GW_PPC_SIG_SLOT_MASK 0xFFu

/* Calling signature of a bridged target. `float_args` is a bitmask: bit i set => native argument
 * slot i is a float sourced from the next FPR (f1..f8) rather than the next GPR (r3..r10), plus
 * the GW_PPC_SIG_VARARGS flag above. `n_args` bounds the walk (0..8; 0 for a no-argument function
 * like HSD_Randf, and for a variadic target the count of FIXED parameters); `ret_float` set => the
 * callee returns a float, captured into FPR 1 instead of the word return in r3. */
typedef struct gw_ppc_sig {
    uint32_t float_args;
    uint32_t n_args;
    int ret_float;
    /* EXTENDED signature, for what the three fields above cannot say. NULL (the default the
     * bridge sets before resolving) keeps the word-slot path, so every existing signature is
     * untouched. Otherwise a string "<ret>:<args>", one class per C parameter:
     *   i       a word from the next GPR (r3..r10), then from the caller's stack parameter area
     *           at r1+8, as the PowerPC EABI spills a ninth integer argument
     *   f       a float from the next FPR (f1..f8), one native word
     *   d       a double from the next FPR, TWO native words (i686 cdecl passes it inline)
     *   a<N>    an N-byte aggregate BY VALUE: PowerPC passes a pointer to a copy in the next GPR,
     *           i686 cdecl passes the bytes themselves inline, rounded up to words. The bytes are
     *           copied raw - gwtool's callers materialise that copy big-endian, so the callee
     *           expects exactly the guest's bytes (docs/DEVLOG.md, "Struct-by-value ABI").
     * and <ret> is i (word, pointer, void, or an aggregate of at most 4 bytes - PowerPC returns
     * that in r3 and the retargeted callee in EAX as the same big-endian word), f (float) or d
     * (double), the latter two captured into f1. Up to GW_PPC_EXT_MAX_WORDS native words. */
    const char *ext;
} gw_ppc_sig;

#define GW_PPC_EXT_MAX_WORDS 16u

/* Resolver: guest target address -> native function pointer, or NULL (which panics). When the
 * function is resolved, *sig is filled with its calling signature. `ctx` is the opaque pointer
 * supplied to gw_ppc_set_bridge. */
typedef gw_ppc_native_fn (*gw_ppc_resolver_fn)(uint32_t guest_addr, void *ctx, gw_ppc_sig *sig);

/* Set the process-global bridge: the resolver + its opaque context, and the blob's code range
 * [code_lo, code_hi). A branch/call whose target falls outside this range is resolved through
 * the bridge instead of being interpreted. */
void gw_ppc_set_bridge(gw_ppc_resolver_fn resolve, void *ctx, uint32_t code_lo, uint32_t code_hi);

/* Guest code is a SET of ranges, not one. The bridge's [code_lo, code_hi) is the fighter's
 * ftFunction; each item article (itFunction) is loaded into its own region and registered here.
 * Without this, a `bl` from one article function to another is mistaken for a native call. */
void gw_ppc_add_code_range(uint32_t lo, uint32_t hi);

/* Forget a range added with gw_ppc_add_code_range (exact lo/hi). No-op if absent. Used when a
 * fighter's file is unloaded and its code is reinstalled elsewhere on the next load. */
void gw_ppc_remove_code_range(uint32_t lo, uint32_t hi);

/* True if `a` is guest code: the bridge range or any registered range. Used by the interpreter's
 * interpret-vs-bridge decision and fetch guard, and by the m-ex runtime's execute trap. */
int gw_ppc_is_guest_code(uint32_t a);

/* Blob return address of the innermost guest->native call, or 0 outside guest code. */
uint32_t gw_ppc_guest_lr(void);

/* Run guest_fn (a guest address) until it returns via blr, yielding r3. gpr_args[0..nargs-1] are
 * placed in r3, r4, ...; rtoc -> r2; sp -> r1. Reentrant: the active machine is saved/restored. */
uint32_t gw_ppc_call(uint32_t guest_fn, const uint32_t *gpr_args, int nargs, uint32_t rtoc,
                     uint32_t sp);

/* ---- symbolizer -------------------------------------------------------------------------
 * Optional: resolves a guest code address to a function name for panics and traces. The m-ex
 * runtime installs one backed by the blob's own MEXDebugSymbol table. The interpreter keeps no
 * knowledge of where names come from, exactly like the bridge resolver above. Unset (or a NULL
 * return) simply means addresses print bare. */
typedef const char *(*gw_ppc_symbolizer_fn)(uint32_t guest_addr);
void gw_ppc_set_symbolizer(gw_ppc_symbolizer_fn fn);

/* "0x807FA130 (SpawnTrailEffect)" or "0x807FA130" - never NULL, safe on any path. Returns a
 * pointer to one of a few rotating static buffers, so several calls in one printf are fine. */
const char *gw_ppc_describe(uint32_t guest_addr);

/* Registers the interpreter's self-contained end-to-end test with the in-engine suite. */
void gw_ppc_tests_register(void);

#ifdef __cplusplus
}
#endif
#endif /* GW_PPC_H */
