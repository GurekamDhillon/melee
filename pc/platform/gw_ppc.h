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
 * resolver for a native function pointer. Phase 1 marshals integer arguments only: the guest
 * argument registers r3..r10 are passed to the native function in order, and its returned word
 * is stored back into r3. Float args (f1..f8), float returns, and struct/sret returns are NOT
 * marshalled in Phase 1 - later phases widen this without changing the resolver contract. */

/* A native target, in the fixed callable shape the bridge marshals into. */
typedef uint32_t (*gw_ppc_native_fn)(uint32_t a0, uint32_t a1, uint32_t a2, uint32_t a3,
                                     uint32_t a4, uint32_t a5, uint32_t a6, uint32_t a7);

/* Resolver: guest target address -> native function pointer, or NULL (which panics). `ctx` is
 * the opaque pointer supplied to gw_ppc_set_bridge. */
typedef gw_ppc_native_fn (*gw_ppc_resolver_fn)(uint32_t guest_addr, void *ctx);

/* Set the process-global bridge: the resolver + its opaque context, and the blob's code range
 * [code_lo, code_hi). A branch/call whose target falls outside this range is resolved through
 * the bridge instead of being interpreted. */
void gw_ppc_set_bridge(gw_ppc_resolver_fn resolve, void *ctx, uint32_t code_lo, uint32_t code_hi);

/* Run guest_fn (a guest address) until it returns via blr, yielding r3. gpr_args[0..nargs-1] are
 * placed in r3, r4, ...; rtoc -> r2; sp -> r1. Reentrant: the active machine is saved/restored. */
uint32_t gw_ppc_call(uint32_t guest_fn, const uint32_t *gpr_args, int nargs, uint32_t rtoc,
                     uint32_t sp);

/* Registers the interpreter's self-contained end-to-end test with the in-engine suite. */
void gw_ppc_tests_register(void);

#ifdef __cplusplus
}
#endif
#endif /* GW_PPC_H */
