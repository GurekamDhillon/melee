/* gw_ppc.c - Gekko/PowerPC subset interpreter (Phase 1). See gw_ppc.h for the contract.
 *
 * Instruction words are fetched from guest memory big-endian (gw_r32), so a hand-built blob
 * stored big-endian in MEM1 decodes correctly. Every interpreted load/store is bounds-checked
 * against MEM1 and raises a clean panic (guest PC + effective address) rather than a native
 * fault. Anything outside the implemented subset raises a clean "unimplemented opcode" panic
 * with the guest PC and raw word - never silently continues.
 *
 * The decode tables below follow the PowerPC ISA bit layout. Where a field is split across the
 * instruction word (the mfspr/mtspr SPR number, the CR fields) or a value is sign-extended from a
 * sub-word field (the b/bc branch displacements), the encoding is spelled out next to the code.
 */
#include "gw.h"
#include "gw_test.h"
#include "gw_ppc.h"
#include "gw_mex_bridge.h"

#include <math.h>
#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/* Guest MEM1 base - the same reservation gw_mem_init makes in gw_runtime.c. */
#define GW_PPC_MEM1_BASE 0x80000000u

/* ---- machine state -------------------------------------------------------------------- */

typedef struct gw_ppc_machine {
    gw_ppc_ctx cpu;
    /* bridge seam (Phase 1: supplied by the test; later: the build-time guest->native table) */
    gw_ppc_resolver_fn resolve;
    void *bridge_ctx;
    uint32_t code_lo; /* blob code range [code_lo, code_hi) */
    uint32_t code_hi;
} gw_ppc_machine;

/* The active machine. gw_ppc_call saves/restores it on the native stack, so interpreted code that
 * bridges into native code can be re-entered (native -> gw_ppc_call -> ... ) without clobbering. */
static gw_ppc_machine gw_ppc_m;

/* ---- re-entry bound ---------------------------------------------------------------------
 * A guest override that calls the very engine function it was hooked from re-enters the
 * interpreter through the bridge (guest -> native -> hook dispatch -> gw_ppc_call -> guest ...).
 * The semantic fix for that cycle lives in the hook dispatcher (gw_runtime.c reruns vanilla for a
 * hook already on the stack); this is the backstop for any cycle that fix does not cover. Without
 * it the recursion consumes the NATIVE stack and dies as 0xC00000FD (STACK_OVERFLOW) at
 * gw_ppc_call+0x3, with no indication of which guest functions formed the loop.
 *
 * At the cap the call is refused (r3 = 0) and the whole guest chain is logged once, so a new cycle
 * shows up as a diagnosable log line instead of an unrecoverable crash. The cap is far above any
 * legitimate nesting: real fighter callbacks bridge out and back at most a few levels deep. */
#define GW_PPC_MAX_DEPTH 16

/* Instruction budget for one gw_ppc_call. Guest code that loops forever (e.g. a search over an
 * m-ex list whose shims return nothing, so the terminating condition never holds) otherwise
 * freezes the game thread inside the interpreter with no diagnostic - the watchdog just reports
 * "at gw_ppc_execute_fp" over and over. The budget turns that into a clean panic naming the guest
 * PC and the range it was looping in. It is far above any legitimate callback: a per-frame
 * fighter callback runs thousands of instructions, not tens of millions. */
#define GW_PPC_MAX_INSNS 50000000u

/* Gap left below the interrupted frame's r1 when a nested run derives its own guest stack. Big
 * enough for the PowerPC linkage area plus the leaf slack a caller may still be using. */
#define GW_PPC_NEST_GAP 0x40u

static int gw_ppc_depth;                          /* active gw_ppc_call nesting level */
static uint32_t gw_ppc_entry[GW_PPC_MAX_DEPTH];   /* entry address per level, for the cycle log */
static int gw_ppc_depth_logged;                   /* the cap log fires once per process */

/* MELEE_PPC_TRACE_FP=1 logs every bridged call that the signature table marks as float-returning,
 * with its marshalled arguments and result. Diagnostic only: a wrong float signature is silent
 * garbage, so seeing the actual values is the only way to tell a bad signature from a bad input. */
static int gw_ppc_trace_fp;

static gw_ppc_symbolizer_fn gw_ppc_symbolizer;

void gw_ppc_set_symbolizer(gw_ppc_symbolizer_fn fn) { gw_ppc_symbolizer = fn; }

const char *gw_ppc_describe(uint32_t guest_addr) {
    /* Rotating buffers so a single log call can describe more than one address. */
    static char buf[4][80];
    static int next;
    char *b = buf[next];
    const char *name = (gw_ppc_symbolizer != NULL) ? gw_ppc_symbolizer(guest_addr) : NULL;
    next = (next + 1) & 3;
    if (name != NULL) {
        snprintf(b, sizeof buf[0], "0x%08X (%s)", guest_addr, name);
    } else {
        snprintf(b, sizeof buf[0], "0x%08X", guest_addr);
    }
    return b;
}

void gw_ppc_set_bridge(gw_ppc_resolver_fn resolve, void *ctx, uint32_t code_lo, uint32_t code_hi) {
    const char *t = getenv("MELEE_PPC_TRACE_FP");
    gw_ppc_trace_fp = (t != NULL && t[0] == '1');
    gw_ppc_m.resolve = resolve;
    gw_ppc_m.bridge_ctx = ctx;
    gw_ppc_m.code_lo = code_lo;
    gw_ppc_m.code_hi = code_hi;
}

/* ---- guest memory access (bounds-checked, big-endian) ---------------------------------- */

/* The game's static globals (.data/.bss/.sbss) live in the NATIVE exe (the retargeted x86 image),
 * not in guest MEM1, even though their guest addresses (0x803B7280..0x804DEA98 in this build) fall
 * numerically inside the MEM1 window. Heap data (fighter/HSD objects, the blob's own stack) is
 * allocated from gw_mem1, so its guest address IS its native address. Only bridge-table data
 * objects (kind 0) are therefore re-routed to native storage; every other address stays a direct
 * MEM1 access. The value stored at a static's native address is big-endian (gwtool byte-swaps every
 * game access), so it is read/written with the same gw_rN/gw_wN accessors as MEM1. */
static uint32_t gw_ppc_static_native(uint32_t ea) {
    int kind = 0;
    uint32_t native;
    /* All bridge data objects sit at/above 0x803B7280 (the decomp .data base); the interpreter's
     * heap/stack/code all live below 0x80300000. Gate the lookup so the hot heap path skips the
     * binary search. The bound is deliberately loose: a heap address above it simply misses the
     * exact-match lookup and falls through to MEM1 unchanged. */
    if (ea < 0x80300000u) {
        return 0;
    }
    native = gw_mex_bridge_lookup(ea, &kind);
    if (native != 0 && kind == 0) {
        return native;
    }
    return 0;
}

static int gw_ppc_ea_ok(uint32_t ea, uint32_t size) {
    /* Every interpreted access must land wholly in MEM1. Use 64-bit to avoid wraparound. */
    uint64_t end = (uint64_t)ea + (uint64_t)size;
    if (ea < GW_PPC_MEM1_BASE) {
        return 0;
    }
    if (end > (uint64_t)GW_PPC_MEM1_BASE + (uint64_t)gw_mem1_size) {
        return 0;
    }
    return 1;
}

/* ---- fault dump ------------------------------------------------------------------------
 * A guest access violation is almost always "a register holds the wrong thing", and the two
 * numbers the panic used to carry - the guest pc and the effective address - name only the
 * symptom: which instruction dereferenced a bad value, never which register held it or what the
 * value actually points at. Reproducing an m-ex fault costs a whole windowed run, so the one dump
 * has to answer the next question as well. It prints the register file, the instruction stream
 * around the fault, the interpreted call chain, and a short big-endian hexdump at every distinct
 * register value that looks like a guest pointer - which is what identifies the object a bad
 * pointer really names (a Fighter, a GObj, an Item, a float array, ...).
 *
 * It must be safe on the panic path: every guest read is bounds-checked here, never through the
 * ld helpers (those panic again, which would recurse). */

#define GW_PPC_DUMP_PTRS 10 /* distinct pointer-looking registers to hexdump */

static int gw_ppc_looks_guest(uint32_t v) {
    /* Plausible-pointer test, deliberately loose: word-aligned, inside MEM1, and with room for
     * the 0x20 bytes the hexdump reads. A false positive costs one extra line. */
    return (v & 3u) == 0u && gw_ppc_ea_ok(v, 0x20u);
}

static void gw_ppc_dump_state(const gw_ppc_machine *m, uint32_t ip) {
    const gw_ppc_ctx *c = &m->cpu;
    uint32_t seen[GW_PPC_DUMP_PTRS];
    int n_seen = 0;
    int i;
    int j;

    gw_log("ppc: --- interpreter state at the fault ---");
    gw_log("ppc:   ip  = %s", gw_ppc_describe(ip));
    gw_log("ppc:   pc  = 0x%08X  lr = %s", c->pc, gw_ppc_describe(c->lr));
    gw_log("ppc:   ctr = 0x%08X  cr = 0x%08X  xer = 0x%08X", c->ctr, c->cr, c->xer);
    gw_log("ppc:   blob code range [0x%08X,0x%08X)  depth = %d", m->code_lo, m->code_hi,
           gw_ppc_depth);

    /* The instruction stream: four words before the faulting one and four after. The faulting
     * word is marked, so the raw encoding can be pasted straight into tools/mex_port/ppc_disasm.py
     * without a second dump of the blob. */
    for (i = -4; i <= 4; ++i) {
        uint32_t a = (uint32_t)((int32_t)ip + 4 * i);
        if (!gw_ppc_ea_ok(a, 4)) {
            continue;
        }
        gw_log("ppc:   %s 0x%08X  %08X", (i == 0) ? "->" : "  ", a,
               gw_r32((const void *)(uintptr_t)a));
    }

    for (i = 0; i < 32; i += 4) {
        gw_log("ppc:   r%-2d 0x%08X  r%-2d 0x%08X  r%-2d 0x%08X  r%-2d 0x%08X", i, c->gpr[i],
               i + 1, c->gpr[i + 1], i + 2, c->gpr[i + 2], i + 3, c->gpr[i + 3]);
    }
    /* f0..f13 only: the PowerPC ABI passes and returns floats in f1..f8 and f0/f9..f13 are the
     * scratch a callback actually uses, so the high FPRs are noise on this path. Both the double
     * value and the raw halves are printed - a single-precision bit pattern that has landed in a
     * GPR (the shape this dump exists to catch) is recognisable only in the raw word. */
    for (i = 0; i <= 13; i += 2) {
        gw_log("ppc:   f%-2d %-16g 0x%08X%08X   f%-2d %-16g 0x%08X%08X", i, c->fpr[i].d,
               c->fpr[i].u32[0], c->fpr[i].u32[1], i + 1, c->fpr[i + 1].d, c->fpr[i + 1].u32[0],
               c->fpr[i + 1].u32[1]);
    }

    for (i = gw_ppc_depth - 1; i >= 0; --i) {
        gw_log("ppc:   called from entry[%d] = %s", i, gw_ppc_describe(gw_ppc_entry[i]));
    }

    /* Hexdump at each distinct pointer-looking register. gw_r32 is used directly (not the ld
     * helpers) because a second access violation on the panic path would recurse. */
    for (i = 0; i < 32 && n_seen < GW_PPC_DUMP_PTRS; ++i) {
        uint32_t v = c->gpr[i];
        int dup = 0;
        if (!gw_ppc_looks_guest(v)) {
            continue;
        }
        for (j = 0; j < n_seen; ++j) {
            if (seen[j] == v) {
                dup = 1;
            }
        }
        if (dup) {
            continue;
        }
        seen[n_seen++] = v;
        gw_log("ppc:   [r%d] 0x%08X: %08X %08X %08X %08X %08X %08X %08X %08X", i, v,
               gw_r32((const void *)(uintptr_t)(v + 0x00)),
               gw_r32((const void *)(uintptr_t)(v + 0x04)),
               gw_r32((const void *)(uintptr_t)(v + 0x08)),
               gw_r32((const void *)(uintptr_t)(v + 0x0C)),
               gw_r32((const void *)(uintptr_t)(v + 0x10)),
               gw_r32((const void *)(uintptr_t)(v + 0x14)),
               gw_r32((const void *)(uintptr_t)(v + 0x18)),
               gw_r32((const void *)(uintptr_t)(v + 0x1C)));
    }
    gw_log("ppc: --- end of interpreter state ---");
}

static void gw_ppc_access_violation(const gw_ppc_machine *m, uint32_t ip, uint32_t ea) {
    gw_ppc_dump_state(m, ip);
    gw_panic("ppc: guest access violation at ip=%s ea=0x%08X", gw_ppc_describe(ip), ea);
}

static uint8_t gw_ppc_ld8(gw_ppc_machine *m, uint32_t ea) {
    uint32_t native = gw_ppc_static_native(ea);
    if (native != 0) {
        return gw_r8((const void *)(uintptr_t)native);
    }
    if (!gw_ppc_ea_ok(ea, 1)) {
        gw_ppc_access_violation(m, m->cpu.pc - 4, ea);
    }
    return gw_r8((const void *)(uintptr_t)ea);
}
static uint16_t gw_ppc_ld16(gw_ppc_machine *m, uint32_t ea) {
    uint32_t native = gw_ppc_static_native(ea);
    if (native != 0) {
        return gw_r16((const void *)(uintptr_t)native);
    }
    if (!gw_ppc_ea_ok(ea, 2)) {
        gw_ppc_access_violation(m, m->cpu.pc - 4, ea);
    }
    return gw_r16((const void *)(uintptr_t)ea);
}
static uint32_t gw_ppc_ld32(gw_ppc_machine *m, uint32_t ea) {
    uint32_t native = gw_ppc_static_native(ea);
    if (native != 0) {
        return gw_r32((const void *)(uintptr_t)native);
    }
    if (!gw_ppc_ea_ok(ea, 4)) {
        gw_ppc_access_violation(m, m->cpu.pc - 4, ea);
    }
    return gw_r32((const void *)(uintptr_t)ea);
}
static uint64_t gw_ppc_ld64(gw_ppc_machine *m, uint32_t ea) {
    uint32_t native = gw_ppc_static_native(ea);
    if (native != 0) {
        return gw_r64((const void *)(uintptr_t)native);
    }
    if (!gw_ppc_ea_ok(ea, 8)) {
        gw_ppc_access_violation(m, m->cpu.pc - 4, ea);
    }
    return gw_r64((const void *)(uintptr_t)ea);
}
static void gw_ppc_st8(gw_ppc_machine *m, uint32_t ea, uint8_t v) {
    uint32_t native = gw_ppc_static_native(ea);
    if (native != 0) {
        gw_w8((void *)(uintptr_t)native, v);
        return;
    }
    if (!gw_ppc_ea_ok(ea, 1)) {
        gw_ppc_access_violation(m, m->cpu.pc - 4, ea);
    }
    gw_w8((void *)(uintptr_t)ea, v);
}
static void gw_ppc_st16(gw_ppc_machine *m, uint32_t ea, uint16_t v) {
    uint32_t native = gw_ppc_static_native(ea);
    if (native != 0) {
        gw_w16((void *)(uintptr_t)native, v);
        return;
    }
    if (!gw_ppc_ea_ok(ea, 2)) {
        gw_ppc_access_violation(m, m->cpu.pc - 4, ea);
    }
    gw_w16((void *)(uintptr_t)ea, v);
}
static void gw_ppc_st32(gw_ppc_machine *m, uint32_t ea, uint32_t v) {
    uint32_t native = gw_ppc_static_native(ea);
    if (native != 0) {
        gw_w32((void *)(uintptr_t)native, v);
        return;
    }
    if (!gw_ppc_ea_ok(ea, 4)) {
        gw_ppc_access_violation(m, m->cpu.pc - 4, ea);
    }
    gw_w32((void *)(uintptr_t)ea, v);
}
static void gw_ppc_st64(gw_ppc_machine *m, uint32_t ea, uint64_t v) {
    uint32_t native = gw_ppc_static_native(ea);
    if (native != 0) {
        gw_w64((void *)(uintptr_t)native, v);
        return;
    }
    if (!gw_ppc_ea_ok(ea, 8)) {
        gw_ppc_access_violation(m, m->cpu.pc - 4, ea);
    }
    gw_w64((void *)(uintptr_t)ea, v);
}

/* ---- panics --------------------------------------------------------------------------- */

static void gw_ppc_bad_opcode(const gw_ppc_machine *m, uint32_t ip, uint32_t insn) {
    /* Same reasoning as the access violation: an unimplemented opcode is cheap to add, but only
     * if the one run that hit it says what the surrounding code was doing. */
    gw_ppc_dump_state(m, ip);
    gw_panic("ppc: unimplemented opcode at ip=%s word=0x%08X", gw_ppc_describe(ip), insn);
}

/* ---- CR helpers -----------------------------------------------------------------------
 * The 32-bit CR is eight 4-bit fields; field n lives in bits (28-4n)..(31-4n) of the word, and
 * within a field the bits are LT, GT, EQ, SO from most- to least-significant. BI (a 5-bit field
 * in branch instructions) names a single CR bit with bit 0 = CR0.LT. */

static uint32_t gw_ppc_cr_bit(const gw_ppc_ctx *c, uint32_t bi) {
    return (c->cr >> (31 - bi)) & 1u;
}

static void gw_ppc_cr_set_bit(gw_ppc_ctx *c, uint32_t bi, uint32_t v) {
    uint32_t mask = 1u << (31 - bi);
    c->cr = (c->cr & ~mask) | ((v & 1u) << (31 - bi));
}

static void gw_ppc_cr_set_field(gw_ppc_ctx *c, uint32_t crfd, uint32_t field) {
    uint32_t shift = 28 - 4 * crfd;
    c->cr = (c->cr & ~(0xFu << shift)) | ((field & 0xFu) << shift);
}

static uint32_t gw_ppc_cr_get_field(const gw_ppc_ctx *c, uint32_t crfd) {
    return (c->cr >> (28 - 4 * crfd)) & 0xFu;
}

/* XER[CA] (bit 29, i.e. 0x20000000) - the carry flag the add/sub-with-carry family uses. */
static uint32_t gw_ppc_xer_ca(const gw_ppc_ctx *c) {
    return (c->xer >> 29) & 1u;
}

static void gw_ppc_xer_set_ca(gw_ppc_ctx *c, uint32_t v) {
    c->xer = (c->xer & ~0x20000000u) | ((v & 1u) << 29);
}

/* Set CR[crfd] from a signed comparison of `res` against zero (LT/GT/EQ), copying XER[SO]. */
static void gw_ppc_cr0_cmp(gw_ppc_ctx *c, uint32_t res) {
    uint32_t lt = (res >> 31) & 1u;
    uint32_t gt = (res != 0) && !lt;
    uint32_t eq = (res == 0);
    uint32_t so = (c->xer >> 31) & 1u;
    gw_ppc_cr_set_field(c, 0, (lt << 3) | (gt << 2) | (eq << 1) | so);
}

/* Signed/unsigned compare of a vs b, writing the LT/GT/EQ field to CR[crfd]. */
static void gw_ppc_cmp(gw_ppc_ctx *c, uint32_t crfd, uint32_t a, uint32_t b, int is_unsigned) {
    uint32_t lt, gt, eq, so;
    if (is_unsigned) {
        lt = (a < b);
        gt = (a > b);
    } else {
        lt = ((int32_t)a < (int32_t)b);
        gt = ((int32_t)a > (int32_t)b);
    }
    eq = (a == b);
    so = (c->xer >> 31) & 1u;
    gw_ppc_cr_set_field(c, crfd, (lt << 3) | (gt << 2) | (eq << 1) | so);
}

/* ---- branch helpers ------------------------------------------------------------------- */

static void gw_ppc_bridge_call(gw_ppc_machine *m, uint32_t guest_addr) {
    gw_ppc_ctx *c = &m->cpu;
    gw_ppc_native_fn fn;
    gw_ppc_sig sig;
    uint32_t args[8];
    int fpr_i = 1; /* float args start at f1 */
    int gpr_i = 3; /* integer args start at r3 */
    uint32_t i;
    uint32_t fmask;
    int variadic;

    if (m->resolve == NULL) {
        gw_panic("ppc: branch to 0x%08X outside blob with no resolver (ip=0x%08X)", guest_addr,
                 c->pc - 4);
    }
    sig.float_args = 0;
    sig.n_args = 8;
    sig.ret_float = 0;
    fn = m->resolve(guest_addr, m->bridge_ctx, &sig);
    if (fn == NULL) {
        gw_panic("ppc: resolver returned NULL for guest address 0x%08X", guest_addr);
    }
    if (sig.n_args > 8) {
        gw_panic("ppc: resolver reported %u args (>8) for guest address 0x%08X", sig.n_args,
                 guest_addr);
    }
    fmask = sig.float_args & GW_PPC_SIG_SLOT_MASK;
    variadic = (sig.float_args & GW_PPC_SIG_VARARGS) != 0;
    /* Marshal each native argument from its PowerPC register: a float argument takes the next
     * FPR (f1..f8), an integer/pointer argument the next GPR (r3..r10). The float's IEEE-754
     * bits are passed in the uint32 slot, which on i686 cdecl lands verbatim in the callee's
     * float slot. */
    for (i = 0; i < 8; ++i) {
        args[i] = 0;
    }
    for (i = 0; i < sig.n_args; ++i) {
        if (fmask & (1u << i)) {
            float f = (float)c->fpr[fpr_i].d;
            memcpy(&args[i], &f, 4);
            ++fpr_i;
        } else {
            args[i] = c->gpr[gpr_i];
            ++gpr_i;
        }
    }
    /* ---- the variadic tail -------------------------------------------------------------
     * A variadic callee does not read its arguments from a signature, so neither can the
     * bridge: the number and the types of the values past `n_args` are a property of the CALL
     * SITE, not of the function. The PowerPC EABI hands us exactly the one bit that matters.
     * A variadic caller must record in CR bit 6 whether it passed any argument in an FPR -
     * `creqv 6,6,6` (crset) when it did, `crxor 6,6,6` (crclr) when it did not - and MWCC emits
     * that instruction immediately before every variadic `bl`, so it is still live here.
     *
     * The native callee is i686 cdecl and walks its varargs along a flat stack, where the
     * default argument promotions have already turned a float into an EIGHT-byte double. So:
     *
     *   CR6 clear -> every variadic value is a word: the remaining GPRs are already in the
     *                right order, which is why all-pointer varargs (efSync_Spawn) happened to
     *                work under the old integer default.
     *   CR6 set   -> the first variadic value is a double in the next FPR: it occupies TWO
     *                native slots, and the words that follow come from the GPRs after it.
     *
     * Only the FIRST variadic value's class is knowable this way. A call passing a float
     * vararg followed by more values of mixed classes cannot be expressed, because the PowerPC
     * register assignment has already lost their relative order; such a target needs a
     * hand-written adapter. No call site in the shipped content does that today. */
    if (variadic) {
        if (gw_ppc_cr_bit(c, 6)) {
            double d = c->fpr[fpr_i].d;
            if (sig.n_args + 2u > 8u) {
                gw_panic("ppc: variadic call to 0x%08X has %u fixed args - no room for the "
                         "8-byte float vararg (ip=0x%08X)",
                         guest_addr, sig.n_args, c->pc - 4);
            }
            memcpy(&args[sig.n_args], &d, sizeof d);
            ++fpr_i;
            i = sig.n_args + 2u;
            if (gw_ppc_trace_fp) {
                gw_log("ppc: variadic %s: %u fixed args + float vararg %.6f",
                       gw_ppc_describe(guest_addr), sig.n_args, d);
            }
        }
        for (; i < 8u && gpr_i <= 10; ++i) {
            args[i] = c->gpr[gpr_i];
            ++gpr_i;
        }
    }
    if (sig.ret_float) {
        /* The callee returns in x87 ST(0); call it in its float shape and capture into f1. */
        float (*ffn)(uint32_t, uint32_t, uint32_t, uint32_t, uint32_t, uint32_t, uint32_t,
                     uint32_t) = (void *)fn;
        float r = ffn(args[0], args[1], args[2], args[3], args[4], args[5], args[6], args[7]);
        if (gw_ppc_trace_fp) {
            float a0, a1;
            memcpy(&a0, &args[0], 4);
            memcpy(&a1, &args[1], 4);
            gw_log("ppc: fp-call %s(%.6f, %.6f) -> %.6f  [nargs=%u mask=0x%X]",
                   gw_ppc_describe(guest_addr),
                   (double) a0, (double) a1, (double) r, sig.n_args, fmask);
        }
        c->fpr[1].d = (double)r;
    } else {
        c->gpr[3] = fn(args[0], args[1], args[2], args[3], args[4], args[5], args[6], args[7]);
    }
}

/* Evaluate a branch condition. Follows the ISA BO field:
 *   BO[4] (0x10): 1 => "branch always" (b) when BO[2] is set, else the CTR family (bdnz/bdz)
 *   BO[3] (0x08): for CR tests, 1 => branch-if-true, 0 => branch-if-false
 *   BO[2] (0x04): 1 => test CR bit BI, 0 => test CTR
 *   BO[1] (0x02): for CTR tests, 1 => branch-if-CTR==0, 0 => branch-if-CTR!=0
 * CTR branches decrement CTR first. The combined CTR+CR forms (BO[4]=0, BO[2]=0) are rare; we
 * approximate them as (CTR condition) AND (CR condition). */
static int gw_ppc_cond(gw_ppc_machine *m, uint32_t bo, uint32_t bi) {
    gw_ppc_ctx *c = &m->cpu;
    uint32_t crbit = gw_ppc_cr_bit(c, bi);
    if (bo & 0x10) {
        if (bo & 0x04) {
            return 1; /* b */
        }
        c->ctr = c->ctr - 1;
        return ((c->ctr != 0) ^ ((bo & 0x02) != 0)) != 0;
    }
    if (bo & 0x04) {
        return (crbit ^ ((bo & 0x08) == 0)) != 0;
    }
    c->ctr = c->ctr - 1;
    return (((c->ctr != 0) ^ ((bo & 0x02) != 0)) & (crbit ^ ((bo & 0x08) == 0))) != 0;
}

#define GW_PPC_MAX_CODE_RANGES 256
static uint32_t gw_ppc_range_lo[GW_PPC_MAX_CODE_RANGES];
static uint32_t gw_ppc_range_hi[GW_PPC_MAX_CODE_RANGES];
static int gw_ppc_range_count;

void gw_ppc_add_code_range(uint32_t lo, uint32_t hi) {
    int i;
    for (i = 0; i < gw_ppc_range_count; ++i) {
        if (gw_ppc_range_lo[i] == lo && gw_ppc_range_hi[i] == hi) {
            return; /* idempotent: a respawn may load the same article again */
        }
    }
    if (gw_ppc_range_count >= GW_PPC_MAX_CODE_RANGES) {
        gw_panic("ppc: too many guest code ranges (%d)", GW_PPC_MAX_CODE_RANGES);
    }
    gw_ppc_range_lo[gw_ppc_range_count] = lo;
    gw_ppc_range_hi[gw_ppc_range_count] = hi;
    ++gw_ppc_range_count;
}

void gw_ppc_remove_code_range(uint32_t lo, uint32_t hi) {
    int i;
    for (i = 0; i < gw_ppc_range_count; ++i) {
        if (gw_ppc_range_lo[i] == lo && gw_ppc_range_hi[i] == hi) {
            --gw_ppc_range_count;
            gw_ppc_range_lo[i] = gw_ppc_range_lo[gw_ppc_range_count];
            gw_ppc_range_hi[i] = gw_ppc_range_hi[gw_ppc_range_count];
            return;
        }
    }
}

static int gw_ppc_in_extra_range(uint32_t a) {
    int i;
    for (i = 0; i < gw_ppc_range_count; ++i) {
        if (a >= gw_ppc_range_lo[i] && a < gw_ppc_range_hi[i]) {
            return 1;
        }
    }
    return 0;
}

int gw_ppc_is_guest_code(uint32_t a) {
    return (a >= gw_ppc_m.code_lo && a < gw_ppc_m.code_hi) || gw_ppc_in_extra_range(a);
}

static int gw_ppc_in_blob(const gw_ppc_machine *m, uint32_t addr) {
    if (gw_ppc_in_extra_range(addr)) {
        return 1;
    }
    return addr >= m->code_lo && addr < m->code_hi;
}

/* ---- instruction fetch ----------------------------------------------------------------- */

static uint32_t gw_ppc_fetch(gw_ppc_machine *m, uint32_t ip) {
    if (!gw_ppc_in_blob(m, ip)) {
        gw_panic("ppc: pc=0x%08X outside blob code range [0x%08X,0x%08X)", ip, m->code_lo,
                 m->code_hi);
    }
    /* Blob code always lives in guest MEM1 (allocated from the fighter heap), so instruction
     * fetch is a direct big-endian read - never a bridged static. */
    return gw_r32((const void *)(uintptr_t)ip);
}

/* ---- rotation mask (rlwinm/rlwimi) ---------------------------------------------------- */

static uint32_t gw_ppc_rotl(uint32_t v, uint32_t sh) {
    sh &= 31;
    return (v << sh) | (v >> ((32 - sh) & 31));
}

static uint32_t gw_ppc_mask(uint32_t mb, uint32_t me) {
    if (mb <= me) {
        return (0xFFFFFFFFu << (31 - me)) & (0xFFFFFFFFu >> mb);
    }
    return (0xFFFFFFFFu << (31 - me)) | (0xFFFFFFFFu >> mb);
}

/* ---- forward declarations for the sub-dispatch ---------------------------------------- */

static int gw_ppc_execute_x(gw_ppc_machine *m, uint32_t insn);          /* opcode 31 */
static int gw_ppc_execute_fp(gw_ppc_machine *m, uint32_t insn, int single); /* 59 / 63 */
static int gw_ppc_execute_cr(gw_ppc_machine *m, uint32_t insn);         /* CR-logical / mcrf */

/* ---- main dispatch --------------------------------------------------------------------
 * Returns 1 when the guest function returns to the native caller (a blr to an out-of-blob LR);
 * the run loop then yields r3. */

static int gw_ppc_execute(gw_ppc_machine *m, uint32_t insn) {
    gw_ppc_ctx *c = &m->cpu;
    uint32_t op = insn >> 26;
    uint32_t ip = c->pc - 4; /* address of the instruction just fetched */
    uint32_t rd, ra, rb, rs;
    int32_t imm;

    switch (op) {
    /* ---- integer add/sub, immediate ------------------------------------------------ */
    case 14: /* addi rD, rA, SIMM */
        rd = (insn >> 21) & 0x1F;
        ra = (insn >> 16) & 0x1F;
        imm = (int32_t)(int16_t)(insn & 0xFFFF);
        c->gpr[rd] = (ra == 0 ? 0 : c->gpr[ra]) + (uint32_t)imm;
        break;
    case 15: /* addis rD, rA, SIMM */
        rd = (insn >> 21) & 0x1F;
        ra = (insn >> 16) & 0x1F;
        imm = (int32_t)(int16_t)(insn & 0xFFFF);
        c->gpr[rd] = (ra == 0 ? 0 : c->gpr[ra]) + ((uint32_t)imm << 16);
        break;
    case 7: /* mulli rD, rA, SIMM */
        rd = (insn >> 21) & 0x1F;
        ra = (insn >> 16) & 0x1F;
        imm = (int32_t)(int16_t)(insn & 0xFFFF);
        c->gpr[rd] = (uint32_t)((int32_t)c->gpr[ra] * imm);
        break;
    case 8: /* subfic rD, rA, SIMM: rD = SIMM - rA, XER[CA] = no borrow (rA=0 => literal 0) */
        rd = (insn >> 21) & 0x1F;
        ra = (insn >> 16) & 0x1F;
        imm = (int32_t)(int16_t)(insn & 0xFFFF);
        {
            uint32_t b = (ra == 0 ? 0 : c->gpr[ra]);
            uint32_t iv = (uint32_t)imm;
            gw_ppc_xer_set_ca(c, iv >= b);
            c->gpr[rd] = iv - b;
        }
        break;
    case 12: /* addic rD, rA, SIMM */
    case 13: /* addic. rD, rA, SIMM (records CR0; Rc is the low opcode bit) */
        rd = (insn >> 21) & 0x1F;
        ra = (insn >> 16) & 0x1F;
        imm = (int32_t)(int16_t)(insn & 0xFFFF);
        {
            uint32_t b = (ra == 0 ? 0 : c->gpr[ra]);
            uint32_t iv = (uint32_t)imm;
            uint32_t sum = b + iv;
            gw_ppc_xer_set_ca(c, (uint32_t)(((uint64_t)b + (uint64_t)iv) >> 32));
            c->gpr[rd] = sum;
            if (insn & 1) {
                gw_ppc_cr0_cmp(c, sum);
            }
        }
        break;

    /* ---- logic immediate ------------------------------------------------------------ */
    case 24: /* ori rA, rS, UIMM */
        ra = (insn >> 16) & 0x1F;
        rs = (insn >> 21) & 0x1F;
        c->gpr[ra] = c->gpr[rs] | (insn & 0xFFFF);
        break;
    case 25: /* oris */
        ra = (insn >> 16) & 0x1F;
        rs = (insn >> 21) & 0x1F;
        c->gpr[ra] = c->gpr[rs] | ((insn & 0xFFFF) << 16);
        break;
    case 26: /* xori */
        ra = (insn >> 16) & 0x1F;
        rs = (insn >> 21) & 0x1F;
        c->gpr[ra] = c->gpr[rs] ^ (insn & 0xFFFF);
        break;
    case 27: /* xoris rA, rS, UIMM: rA = rS ^ (UIMM << 16). No CR update.
              * Compilers emit `xoris rX,rX,0x8000` for int -> float conversion: flipping the sign
              * bit biases a signed int so it can be stored under the 0x43300000 exponent word and
              * loaded as a double, then the bias subtracted. It was the last instruction class
              * reachable in Sonic's code that the interpreter lacked (8 sites, 5 functions). */
        ra = (insn >> 16) & 0x1F;
        rs = (insn >> 21) & 0x1F;
        c->gpr[ra] = c->gpr[rs] ^ ((insn & 0xFFFF) << 16);
        break;
    case 28: /* andi. rA, rS, UIMM (always records) */
        ra = (insn >> 16) & 0x1F;
        rs = (insn >> 21) & 0x1F;
        c->gpr[ra] = c->gpr[rs] & (insn & 0xFFFF);
        gw_ppc_cr0_cmp(c, c->gpr[ra]);
        break;
    case 29: /* andis. */
        ra = (insn >> 16) & 0x1F;
        rs = (insn >> 21) & 0x1F;
        c->gpr[ra] = c->gpr[rs] & ((insn & 0xFFFF) << 16);
        gw_ppc_cr0_cmp(c, c->gpr[ra]);
        break;

    /* ---- rotate/shift immediate ----------------------------------------------------- */
    case 21: /* rlwinm rA, rS, SH, MB, ME (slwi/srwi/clrlwi/clrrwi are aliases) */
        ra = (insn >> 16) & 0x1F;
        rs = (insn >> 21) & 0x1F;
        c->gpr[ra] = gw_ppc_rotl(c->gpr[rs], (insn >> 11) & 0x1F) &
                     gw_ppc_mask((insn >> 6) & 0x1F, (insn >> 1) & 0x1F);
        if (insn & 1) {
            gw_ppc_cr0_cmp(c, c->gpr[ra]);
        }
        break;
    case 20: /* rlwimi rA, rS, SH, MB, ME (inserts into the existing rA) */
    {
        uint32_t m = gw_ppc_mask((insn >> 6) & 0x1F, (insn >> 1) & 0x1F);
        ra = (insn >> 16) & 0x1F;
        rs = (insn >> 21) & 0x1F;
        c->gpr[ra] = (gw_ppc_rotl(c->gpr[rs], (insn >> 11) & 0x1F) & m) | (c->gpr[ra] & ~m);
        if (insn & 1) {
            gw_ppc_cr0_cmp(c, c->gpr[ra]);
        }
        break;
    }
    case 23: /* rlwnm rA, rS, rB, MB, ME (rotate by register) */
        ra = (insn >> 16) & 0x1F;
        rs = (insn >> 21) & 0x1F;
        c->gpr[ra] = gw_ppc_rotl(c->gpr[rs], c->gpr[(insn >> 11) & 0x1F]) &
                     gw_ppc_mask((insn >> 6) & 0x1F, (insn >> 1) & 0x1F);
        if (insn & 1) {
            gw_ppc_cr0_cmp(c, c->gpr[ra]);
        }
        break;

    /* ---- compare immediate ---------------------------------------------------------- */
    case 11: /* cmpwi (L=0) / cmpi (L=1, 64-bit - unimplemented) */
        if ((insn >> 21) & 1) {
            gw_ppc_bad_opcode(m, ip, insn);
        }
        ra = (insn >> 16) & 0x1F;
        imm = (int32_t)(int16_t)(insn & 0xFFFF);
        gw_ppc_cmp(c, (insn >> 23) & 7, c->gpr[ra], (uint32_t)imm, 0);
        break;
    case 10: /* cmplwi */
        if ((insn >> 21) & 1) {
            gw_ppc_bad_opcode(m, ip, insn);
        }
        ra = (insn >> 16) & 0x1F;
        imm = (uint32_t)(insn & 0xFFFF);
        gw_ppc_cmp(c, (insn >> 23) & 7, c->gpr[ra], imm, 1);
        break;

    /* ---- loads ---------------------------------------------------------------------- */
    case 32: /* lwz rD, d(rA) */
        rd = (insn >> 21) & 0x1F;
        ra = (insn >> 16) & 0x1F;
        imm = (int32_t)(int16_t)(insn & 0xFFFF);
        c->gpr[rd] = gw_ppc_ld32(m, (ra == 0 ? 0 : c->gpr[ra]) + (uint32_t)imm);
        break;
    case 33: /* lwzu rD, d(rA) - rA is updated */
        rd = (insn >> 21) & 0x1F;
        ra = (insn >> 16) & 0x1F;
        imm = (int32_t)(int16_t)(insn & 0xFFFF);
        {
            uint32_t ea = c->gpr[ra] + (uint32_t)imm;
            c->gpr[rd] = gw_ppc_ld32(m, ea);
            c->gpr[ra] = ea;
        }
        break;
    case 40: /* lhz */
        rd = (insn >> 21) & 0x1F;
        ra = (insn >> 16) & 0x1F;
        imm = (int32_t)(int16_t)(insn & 0xFFFF);
        c->gpr[rd] = gw_ppc_ld16(m, (ra == 0 ? 0 : c->gpr[ra]) + (uint32_t)imm);
        break;
    case 42: /* lha */
        rd = (insn >> 21) & 0x1F;
        ra = (insn >> 16) & 0x1F;
        imm = (int32_t)(int16_t)(insn & 0xFFFF);
        c->gpr[rd] = (uint32_t)(int32_t)(int16_t)gw_ppc_ld16(m, (ra == 0 ? 0 : c->gpr[ra]) +
                                                               (uint32_t)imm);
        break;
    case 34: /* lbz */
        rd = (insn >> 21) & 0x1F;
        ra = (insn >> 16) & 0x1F;
        imm = (int32_t)(int16_t)(insn & 0xFFFF);
        c->gpr[rd] = gw_ppc_ld8(m, (ra == 0 ? 0 : c->gpr[ra]) + (uint32_t)imm);
        break;
    case 48: /* lfs fD, d(rA) */
        rd = (insn >> 21) & 0x1F;
        ra = (insn >> 16) & 0x1F;
        imm = (int32_t)(int16_t)(insn & 0xFFFF);
        {
            uint32_t bits = gw_ppc_ld32(m, (ra == 0 ? 0 : c->gpr[ra]) + (uint32_t)imm);
            float f;
            memcpy(&f, &bits, 4);
            c->fpr[rd].d = (double)f;
        }
        break;
    case 50: /* lfd fD, d(rA) */
        rd = (insn >> 21) & 0x1F;
        ra = (insn >> 16) & 0x1F;
        imm = (int32_t)(int16_t)(insn & 0xFFFF);
        c->fpr[rd].u64 = gw_ppc_ld64(m, (ra == 0 ? 0 : c->gpr[ra]) + (uint32_t)imm);
        break;

    /* ---- stores --------------------------------------------------------------------- */
    case 36: /* stw rS, d(rA) */
        rs = (insn >> 21) & 0x1F;
        ra = (insn >> 16) & 0x1F;
        imm = (int32_t)(int16_t)(insn & 0xFFFF);
        gw_ppc_st32(m, (ra == 0 ? 0 : c->gpr[ra]) + (uint32_t)imm, c->gpr[rs]);
        break;
    case 37: /* stwu rS, d(rA) */
        rs = (insn >> 21) & 0x1F;
        ra = (insn >> 16) & 0x1F;
        imm = (int32_t)(int16_t)(insn & 0xFFFF);
        {
            uint32_t ea = c->gpr[ra] + (uint32_t)imm;
            gw_ppc_st32(m, ea, c->gpr[rs]);
            c->gpr[ra] = ea;
        }
        break;
    case 44: /* sth */
        rs = (insn >> 21) & 0x1F;
        ra = (insn >> 16) & 0x1F;
        imm = (int32_t)(int16_t)(insn & 0xFFFF);
        gw_ppc_st16(m, (ra == 0 ? 0 : c->gpr[ra]) + (uint32_t)imm, (uint16_t)c->gpr[rs]);
        break;
    case 38: /* stb */
        rs = (insn >> 21) & 0x1F;
        ra = (insn >> 16) & 0x1F;
        imm = (int32_t)(int16_t)(insn & 0xFFFF);
        gw_ppc_st8(m, (ra == 0 ? 0 : c->gpr[ra]) + (uint32_t)imm, (uint8_t)c->gpr[rs]);
        break;
    case 52: /* stfs fS, d(rA) */
        rs = (insn >> 21) & 0x1F;
        ra = (insn >> 16) & 0x1F;
        imm = (int32_t)(int16_t)(insn & 0xFFFF);
        {
            float f = (float)c->fpr[rs].d;
            uint32_t bits;
            memcpy(&bits, &f, 4);
            gw_ppc_st32(m, (ra == 0 ? 0 : c->gpr[ra]) + (uint32_t)imm, bits);
        }
        break;
    case 54: /* stfd fS, d(rA) */
        rs = (insn >> 21) & 0x1F;
        ra = (insn >> 16) & 0x1F;
        imm = (int32_t)(int16_t)(insn & 0xFFFF);
        gw_ppc_st64(m, (ra == 0 ? 0 : c->gpr[ra]) + (uint32_t)imm, c->fpr[rs].u64);
        break;

    /* ---- multiple load/store -------------------------------------------------------- */
    case 46: /* lmw rD, d(rA): load rD..r31 */
        rd = (insn >> 21) & 0x1F;
        ra = (insn >> 16) & 0x1F;
        imm = (int32_t)(int16_t)(insn & 0xFFFF);
        {
            uint32_t ea = (ra == 0 ? 0 : c->gpr[ra]) + (uint32_t)imm;
            uint32_t i;
            for (i = rd; i <= 31; ++i) {
                c->gpr[i] = gw_ppc_ld32(m, ea + 4 * (i - rd));
            }
        }
        break;
    case 47: /* stmw rS, d(rA): store rS..r31 */
        rs = (insn >> 21) & 0x1F;
        ra = (insn >> 16) & 0x1F;
        imm = (int32_t)(int16_t)(insn & 0xFFFF);
        {
            uint32_t ea = (ra == 0 ? 0 : c->gpr[ra]) + (uint32_t)imm;
            uint32_t i;
            for (i = rs; i <= 31; ++i) {
                gw_ppc_st32(m, ea + 4 * (i - rs), c->gpr[i]);
            }
        }
        break;

    /* ---- branches ------------------------------------------------------------------- */
    case 18: /* b / bl */
    {
        int aa = (insn >> 1) & 1;
        int lk = insn & 1;
        int32_t li = ((int32_t)(insn << 6)) >> 8; /* sign-extend 24-bit LI (bits 6..29) */
        uint32_t target = (aa ? 0 : ip) + (uint32_t)(li << 2);
        if (lk) {
            c->lr = c->pc;
        }
        if (gw_ppc_in_blob(m, target)) {
            c->pc = target;
        } else {
            gw_ppc_bridge_call(m, target);
        }
        break;
    }
    case 16: /* bc (beq/bne/blt/bge/bgt/ble aliases) */
    {
        uint32_t bo = (insn >> 21) & 0x1F;
        uint32_t bi = (insn >> 16) & 0x1F;
        int aa = (insn >> 1) & 1;
        int lk = insn & 1;
        if (gw_ppc_cond(m, bo, bi)) {
            int32_t bd = ((int32_t)(insn << 16)) >> 18; /* sign-extend 14-bit BD (bits 16..29) */
            uint32_t target = (aa ? 0 : ip) + (uint32_t)(bd << 2);
            if (lk) {
                c->lr = c->pc;
            }
            if (gw_ppc_in_blob(m, target)) {
                c->pc = target;
            } else {
                gw_ppc_bridge_call(m, target);
            }
        }
        break;
    }
    case 19: /* bclr / bcctr (blr, blrl, bctr, bctrl) + the CR-logical family */
    {
        uint32_t xo = (insn >> 1) & 0x3FF;
        if (xo != 16 && xo != 528) { /* 16 = bclr, 528 = bcctr */
            return gw_ppc_execute_cr(m, insn);
        }
        {
            uint32_t bo = (insn >> 21) & 0x1F;
            uint32_t bi = (insn >> 16) & 0x1F;
            int is_ctr = (insn & 0x400) != 0;
            int lk = insn & 1;
            if (gw_ppc_cond(m, bo, bi)) {
                uint32_t target = is_ctr ? c->ctr : c->lr;
                if (lk) {
                    c->lr = c->pc;
                }
                if (gw_ppc_in_blob(m, target)) {
                    c->pc = target;
                } else if (!is_ctr && !lk) {
                    return 1; /* blr to an out-of-blob LR: return to the native caller */
                } else {
                    gw_ppc_bridge_call(m, target);
                }
            }
        }
        break;
    }

    /* ---- sub-dispatches ------------------------------------------------------------- */
    case 31:
        return gw_ppc_execute_x(m, insn);
    case 59:
        return gw_ppc_execute_fp(m, insn, 1);
    case 63:
        return gw_ppc_execute_fp(m, insn, 0);

    default:
        gw_ppc_bad_opcode(m, ip, insn);
    }
    return 0;
}

/* ---- opcode 31 (X-form integer/logical/system) --------------------------------------- */

static int gw_ppc_execute_x(gw_ppc_machine *m, uint32_t insn) {
    gw_ppc_ctx *c = &m->cpu;
    uint32_t xo = (insn >> 1) & 0x3FF;
    uint32_t ip = c->pc - 4;
    uint32_t rd = (insn >> 21) & 0x1F;
    uint32_t ra = (insn >> 16) & 0x1F;
    uint32_t rb = (insn >> 11) & 0x1F;
    uint32_t rs = (insn >> 21) & 0x1F;

    switch (xo) {
    case 0:  /* cmp / cmpw (L=0); L=1 is 64-bit, unimplemented */
        if ((insn >> 21) & 1) {
            gw_ppc_bad_opcode(m, ip, insn);
        }
        gw_ppc_cmp(c, (insn >> 23) & 7, c->gpr[ra], c->gpr[rb], 0);
        break;
    case 32: /* cmpl / cmplw */
        if ((insn >> 21) & 1) {
            gw_ppc_bad_opcode(m, ip, insn);
        }
        gw_ppc_cmp(c, (insn >> 23) & 7, c->gpr[ra], c->gpr[rb], 1);
        break;

    case 266: /* add */
        c->gpr[rd] = c->gpr[ra] + c->gpr[rb];
        if (insn & 1) {
            gw_ppc_cr0_cmp(c, c->gpr[rd]);
        }
        break;
    case 40: /* subf: rD = rB - rA */
        c->gpr[rd] = c->gpr[rb] - c->gpr[ra];
        if (insn & 1) {
            gw_ppc_cr0_cmp(c, c->gpr[rd]);
        }
        break;
    case 104: /* neg: rD = -rA */
        c->gpr[rd] = 0u - c->gpr[ra];
        if (insn & 1) {
            gw_ppc_cr0_cmp(c, c->gpr[rd]);
        }
        break;
    case 235: /* mullw */
        c->gpr[rd] = (uint32_t)((int32_t)c->gpr[ra] * (int32_t)c->gpr[rb]);
        if (insn & 1) {
            gw_ppc_cr0_cmp(c, c->gpr[rd]);
        }
        break;
    case 491: /* divw: signed divide; division by zero / INT_MIN/-1 handled like the hardware */
        if (c->gpr[rb] == 0) {
            c->gpr[rd] = 0;
        } else if (c->gpr[ra] == 0x80000000u && c->gpr[rb] == 0xFFFFFFFFu) {
            c->gpr[rd] = 0x80000000u;
        } else {
            c->gpr[rd] = (uint32_t)((int32_t)c->gpr[ra] / (int32_t)c->gpr[rb]);
        }
        if (insn & 1) {
            gw_ppc_cr0_cmp(c, c->gpr[rd]);
        }
        break;

    case 28: /* and */
        c->gpr[ra] = c->gpr[rs] & c->gpr[rb];
        if (insn & 1) {
            gw_ppc_cr0_cmp(c, c->gpr[ra]);
        }
        break;
    case 444: /* or (mr) */
        c->gpr[ra] = c->gpr[rs] | c->gpr[rb];
        if (insn & 1) {
            gw_ppc_cr0_cmp(c, c->gpr[ra]);
        }
        break;
    case 316: /* xor */
        c->gpr[ra] = c->gpr[rs] ^ c->gpr[rb];
        if (insn & 1) {
            gw_ppc_cr0_cmp(c, c->gpr[ra]);
        }
        break;
    case 124: /* nor */
        c->gpr[ra] = ~(c->gpr[rs] | c->gpr[rb]);
        if (insn & 1) {
            gw_ppc_cr0_cmp(c, c->gpr[ra]);
        }
        break;

    case 24: /* slw: rA = rS << (rB & 0x1F) */
        c->gpr[ra] = c->gpr[rs] << (c->gpr[rb] & 0x1F);
        if (insn & 1) {
            gw_ppc_cr0_cmp(c, c->gpr[ra]);
        }
        break;
    case 536: /* srw */
        c->gpr[ra] = c->gpr[rs] >> (c->gpr[rb] & 0x1F);
        if (insn & 1) {
            gw_ppc_cr0_cmp(c, c->gpr[ra]);
        }
        break;
    case 792: /* sraw: arithmetic shift by (rB & 0x3F); sets XER[CA] */
    {
        uint32_t sh = c->gpr[rb] & 0x3F;
        uint32_t src = c->gpr[rs];
        uint32_t res;
        uint32_t ca = 0;
        if (sh == 0) {
            res = src;
        } else if (sh >= 32) {
            res = (src & 0x80000000u) ? 0xFFFFFFFFu : 0;
            ca = (src & 0x80000000u) != 0;
        } else {
            res = (uint32_t)((int32_t)src >> sh);
            if ((src & 0x80000000u) && (src & ((1u << sh) - 1))) {
                ca = 1;
            }
        }
        c->gpr[ra] = res;
        c->xer = (c->xer & ~0x20000000u) | (ca ? 0x20000000u : 0);
        if (insn & 1) {
            gw_ppc_cr0_cmp(c, c->gpr[ra]);
        }
        break;
    }
    case 824: /* srawi: arithmetic shift by SH (bits 11..15); sets XER[CA] */
    {
        uint32_t sh = (insn >> 11) & 0x1F;
        uint32_t src = c->gpr[rs];
        uint32_t res;
        uint32_t ca = 0;
        if (sh == 0) {
            res = src;
        } else {
            res = (uint32_t)((int32_t)src >> sh);
            if ((src & 0x80000000u) && (src & ((1u << sh) - 1))) {
                ca = 1;
            }
        }
        c->gpr[ra] = res;
        c->xer = (c->xer & ~0x20000000u) | (ca ? 0x20000000u : 0);
        if (insn & 1) {
            gw_ppc_cr0_cmp(c, c->gpr[ra]);
        }
        break;
    }
    case 26: /* cntlzw */
        {
            uint32_t v = c->gpr[rs];
            uint32_t n = 0;
            if (v == 0) {
                n = 32;
            } else {
                while (!(v & 0x80000000u)) {
                    v <<= 1;
                    ++n;
                }
            }
            c->gpr[ra] = n;
            if (insn & 1) {
                gw_ppc_cr0_cmp(c, c->gpr[ra]);
            }
        }
        break;
    case 954: /* extsb */
        c->gpr[ra] = (uint32_t)(int32_t)(int8_t)c->gpr[rs];
        if (insn & 1) {
            gw_ppc_cr0_cmp(c, c->gpr[ra]);
        }
        break;
    case 922: /* extsh */
        c->gpr[ra] = (uint32_t)(int32_t)(int16_t)c->gpr[rs];
        if (insn & 1) {
            gw_ppc_cr0_cmp(c, c->gpr[ra]);
        }
        break;

    case 23: /* lwzx */
        c->gpr[rd] = gw_ppc_ld32(m, (ra == 0 ? 0 : c->gpr[ra]) + c->gpr[rb]);
        break;
    case 151: /* stwx */
        gw_ppc_st32(m, (ra == 0 ? 0 : c->gpr[ra]) + c->gpr[rb], c->gpr[rs]);
        break;
    case 790: /* lhbrx: load halfword byte-reversed (i.e. little-endian) */
        c->gpr[rd] = gw_bswap16(gw_ppc_ld16(m, (ra == 0 ? 0 : c->gpr[ra]) + c->gpr[rb]));
        break;
    case 918: /* stwbrx: store word byte-reversed */
    {
        uint32_t ea = (ra == 0 ? 0 : c->gpr[ra]) + c->gpr[rb];
        uint32_t v = c->gpr[rs];
        if (!gw_ppc_ea_ok(ea, 4)) {
            gw_ppc_access_violation(m, ip, ea);
        }
        {
            uint8_t *p = (uint8_t *)(uintptr_t)ea;
            p[0] = (uint8_t)v;
            p[1] = (uint8_t)(v >> 8);
            p[2] = (uint8_t)(v >> 16);
            p[3] = (uint8_t)(v >> 24);
        }
        break;
    }

    case 19: /* mfcr */
        c->gpr[rd] = c->cr;
        break;
    case 144: /* mtcrf: move to CR under the 8-bit CRM field mask (bits 12..19) */
    {
        uint32_t crm = (insn >> 12) & 0xFF;
        uint32_t mask = 0;
        int i;
        for (i = 0; i < 8; ++i) {
            if (crm & (0x80u >> i)) {
                mask |= 0xFu << (28 - 4 * i);
            }
        }
        c->cr = (c->cr & ~mask) | (c->gpr[rs] & mask);
        break;
    }

    case 339: /* mfspr rD, SPR (SPR split across bits 11..15 = low 5, 16..20 = high 5) */
    {
        uint32_t spr = ((insn >> 16) & 0x1F) | (((insn >> 11) & 0x1F) << 5);
        switch (spr) {
        case 1: /* XER */
            c->gpr[rd] = c->xer;
            break;
        case 8: /* LR */
            c->gpr[rd] = c->lr;
            break;
        case 9: /* CTR */
            c->gpr[rd] = c->ctr;
            break;
        default:
            gw_panic("ppc: unimplemented mfspr %u at ip=0x%08X", spr, ip);
        }
        break;
    }
    case 467: /* mtspr SPR, rS */
    {
        uint32_t spr = ((insn >> 16) & 0x1F) | (((insn >> 11) & 0x1F) << 5);
        switch (spr) {
        case 1: /* XER */
            c->xer = c->gpr[rs];
            break;
        case 8: /* LR */
            c->lr = c->gpr[rs];
            break;
        case 9: /* CTR */
            c->ctr = c->gpr[rs];
            break;
        default:
            gw_panic("ppc: unimplemented mtspr %u at ip=0x%08X", spr, ip);
        }
        break;
    }

    /* add/sub with carry (the 64-bit arithmetic idioms m-ex code emits) */
    case 8: /* subfc rD, rA, rB: rD = rB - rA, CA = no borrow */
        gw_ppc_xer_set_ca(c, c->gpr[rb] >= c->gpr[ra]);
        c->gpr[rd] = c->gpr[rb] - c->gpr[ra];
        if (insn & 1) {
            gw_ppc_cr0_cmp(c, c->gpr[rd]);
        }
        break;
    case 136: /* subfe rD, rA, rB: rD = ~rA + rB + CA */
    {
        uint32_t ca = gw_ppc_xer_ca(c);
        uint32_t res = (~c->gpr[ra]) + c->gpr[rb] + ca;
        gw_ppc_xer_set_ca(c, (uint64_t)(~c->gpr[ra]) + c->gpr[rb] + ca >= 0x100000000u);
        c->gpr[rd] = res;
        if (insn & 1) {
            gw_ppc_cr0_cmp(c, c->gpr[rd]);
        }
        break;
    }
    case 200: /* subfze rD, rA: rD = ~rA + CA */
    {
        uint32_t ca = gw_ppc_xer_ca(c);
        uint32_t res = (~c->gpr[ra]) + ca;
        gw_ppc_xer_set_ca(c, (uint64_t)(~c->gpr[ra]) + ca >= 0x100000000u);
        c->gpr[rd] = res;
        if (insn & 1) {
            gw_ppc_cr0_cmp(c, c->gpr[rd]);
        }
        break;
    }
    case 232: /* subfme rD, rA: rD = ~rA + CA - 1 */
    {
        uint32_t ca = gw_ppc_xer_ca(c);
        uint32_t res = (~c->gpr[ra]) + ca - 1;
        gw_ppc_xer_set_ca(c, (uint64_t)(~c->gpr[ra]) + ca >= 1);
        c->gpr[rd] = res;
        if (insn & 1) {
            gw_ppc_cr0_cmp(c, c->gpr[rd]);
        }
        break;
    }
    case 202: /* addze rD, rA: rD = rA + CA */
    {
        uint32_t ca = gw_ppc_xer_ca(c);
        uint32_t res = c->gpr[ra] + ca;
        gw_ppc_xer_set_ca(c, (uint64_t)c->gpr[ra] + ca >= 0x100000000u);
        c->gpr[rd] = res;
        if (insn & 1) {
            gw_ppc_cr0_cmp(c, c->gpr[rd]);
        }
        break;
    }
    case 234: /* addme rD, rA: rD = rA + CA - 1 */
    {
        uint32_t ca = gw_ppc_xer_ca(c);
        uint32_t res = c->gpr[ra] + ca - 1;
        gw_ppc_xer_set_ca(c, (uint64_t)c->gpr[ra] + ca >= 1);
        c->gpr[rd] = res;
        if (insn & 1) {
            gw_ppc_cr0_cmp(c, c->gpr[rd]);
        }
        break;
    }

    case 983: /* stfiwx fS, rA, rB: store the low 32 bits of fS as a word */
        gw_ppc_st32(m, (ra == 0 ? 0 : c->gpr[ra]) + c->gpr[rb], c->fpr[rs].u32[1]);
        break;

    /* cache / synchronisation instructions are no-ops in the interpreter (single-threaded);
     * dcbz zeroes its 32-byte block so struct-clearing code behaves */
    case 54: /* dcbst */
    case 86: /* dcbf */
    case 150: /* isync */
    case 246: /* dcbtst */
    case 278: /* dcbt */
    case 470: /* dcbi */
    case 598: /* sync */
    case 854: /* eieio */
    case 982: /* icbi */
        break;
    case 1014: /* dcbz */
    case 1015: /* dcbz_l */
    {
        uint32_t ea = (ra == 0 ? 0 : c->gpr[ra]) + c->gpr[rb];
        uint32_t i;
        for (i = 0; i < 32; i += 4) {
            gw_ppc_st32(m, ea + i, 0);
        }
        break;
    }

    default:
        gw_ppc_bad_opcode(m, ip, insn);
    }
    return 0;
}

/* ---- opcode 19 CR-logical family (and mcrf) -------------------------------------------
 * `cr<op> crD, crA, crB` names three single CR bits (BT/BA/BB); the destination bit is the
 * boolean op of the two source bits. mcrf copies a whole 4-bit field. These appear in
 * m-ex-generated code (the `crxor 6,6,6` "clear a scratch CR field" idiom). */

static int gw_ppc_execute_cr(gw_ppc_machine *m, uint32_t insn) {
    gw_ppc_ctx *c = &m->cpu;
    uint32_t xo = (insn >> 1) & 0x3FF;
    uint32_t ip = c->pc - 4;
    uint32_t bt = (insn >> 21) & 0x1F;
    uint32_t ba = (insn >> 16) & 0x1F;
    uint32_t bb = (insn >> 11) & 0x1F;
    uint32_t a, b, r;

    switch (xo) {
    case 0: /* mcrf crfD, crfS */
        gw_ppc_cr_set_field(c, (insn >> 23) & 7, gw_ppc_cr_get_field(c, (insn >> 18) & 7));
        break;
    case 33: /* crnor */
    case 129: /* crandc */
    case 193: /* crxor */
    case 225: /* crnand */
    case 257: /* crand */
    case 289: /* creqv */
    case 417: /* crorc */
    case 449: /* cror */
        a = gw_ppc_cr_bit(c, ba);
        b = gw_ppc_cr_bit(c, bb);
        switch (xo) {
        case 33:
            r = !(a | b);
            break;
        case 129:
            r = a & !b;
            break;
        case 193:
            r = a ^ b;
            break;
        case 225:
            r = !(a & b);
            break;
        case 257:
            r = a & b;
            break;
        case 289:
            r = !(a ^ b);
            break;
        case 417:
            r = a | !b;
            break;
        default: /* 449 */
            r = a | b;
            break;
        }
        gw_ppc_cr_set_bit(c, bt, r);
        break;
    default:
        gw_ppc_bad_opcode(m, ip, insn);
    }
    return 0;
}

/* ---- opcode 59 (single-precision) and 63 (double-precision) floating point ------------
 * Single ops compute in double then round to single (frsp semantics); Gekko's 25-bit-mantissa
 * rounding is not modeled in Phase 1. NaN/exception behavior is approximated. */

/* fsqrt/frsqrte operand helper. Gekko's fsqrt is an estimate; computing it exactly is closer to
 * the hardware than refusing the instruction, and the port is not cycle- or bit-exact anyway. */
static double gw_ppc_sqrt(double x) { return sqrt(x); }

static int gw_ppc_execute_fp(gw_ppc_machine *m, uint32_t insn, int single) {
    gw_ppc_ctx *c = &m->cpu;
    /* Opcodes 59/63 carry TWO instruction forms with DIFFERENT XO widths:
     *   A-form  (fadd/fsub/fmul/fdiv/fmadd/fsel/...): XO is 5 bits, bits 26..30.
     *   X-form  (fmr/fneg/fabs/frsp/fctiwz/fcmpu/...): XO is 10 bits, bits 21..30.
     * Decoding everything with the 10-bit mask (as this did) only works for an A-form
     * instruction whose frC field happens to be zero, because frC occupies the upper five bits
     * of that mask. `fmuls f12,f12,f12` (0xED8C0332) therefore decoded as xo=409 and raised
     * "unimplemented opcode" mid-move. So: try the 5-bit A-form XO first, fall back to X-form. */
    uint32_t xo5 = (insn >> 1) & 0x1F;
    uint32_t xo10 = (insn >> 1) & 0x3FF;
    uint32_t ip = c->pc - 4;
    uint32_t fd = (insn >> 21) & 0x1F;
    uint32_t fa = (insn >> 16) & 0x1F;
    uint32_t fb = (insn >> 11) & 0x1F;
    uint32_t fc = (insn >> 6) & 0x1F;
    double a = c->fpr[fa].d;
    double b = c->fpr[fb].d;
    double cc = c->fpr[fc].d;
    double r;
    int have = 1;

    /* ---- A-form (5-bit XO). Note the operand pattern: the multiply family reads frA and
     * frC (NOT frB), and the multiply-add family is frA*frC +/- frB. */
    switch (xo5) {
    case 18: /* fdiv / fdivs:   fD = fA / fB */
        r = a / b;
        break;
    case 20: /* fsub / fsubs:   fD = fA - fB */
        r = a - b;
        break;
    case 21: /* fadd / fadds:   fD = fA + fB */
        r = a + b;
        break;
    case 22: /* fsqrt / fsqrts: fD = sqrt(fB) */
        r = gw_ppc_sqrt(b);
        break;
    case 24: /* fres:           fD = 1 / fB (estimate; computed exactly) */
        r = 1.0 / b;
        break;
    case 25: /* fmul / fmuls:   fD = fA * fC  <- frC, not frB */
        r = a * cc;
        break;
    case 26: /* frsqrte:        fD = 1 / sqrt(fB) (estimate; computed exactly) */
        r = 1.0 / gw_ppc_sqrt(b);
        break;
    case 28: /* fmsub / fmsubs:   fD =  (fA * fC) - fB */
        r = (a * cc) - b;
        break;
    case 29: /* fmadd / fmadds:   fD =  (fA * fC) + fB */
        r = (a * cc) + b;
        break;
    case 30: /* fnmsub / fnmsubs: fD = -((fA * fC) - fB) */
        r = -((a * cc) - b);
        break;
    case 31: /* fnmadd / fnmadds: fD = -((fA * fC) + fB) */
        r = -((a * cc) + b);
        break;
    case 23: /* fsel: fD = (fA >= 0 or NaN) ? fC : fB. Never rounded to single. */
        c->fpr[fd].d = (a >= 0.0 || a != a) ? cc : b;
        return 0;
    default:
        have = 0;
        break;
    }
    if (have) {
        /* Single-precision forms round the double result to float (frsp semantics). Gekko's
         * 25-bit-mantissa intermediate rounding is not modeled. */
        c->fpr[fd].d = single ? (double) (float) r : r;
        return 0;
    }

    /* ---- X-form (10-bit XO), opcode 63. */
    switch (xo10) {
    case 72: /* fmr: fD = fB */
        c->fpr[fd].d = b;
        break;
    case 40: /* fneg */
        c->fpr[fd].d = -b;
        break;
    case 264: /* fabs */
        c->fpr[fd].d = b < 0 ? -b : b;
        break;
    case 136: /* fnabs */
        c->fpr[fd].d = b < 0 ? b : -b;
        break;
    case 12: /* frsp: round to single */
        c->fpr[fd].d = (double) (float) b;
        break;
    case 15: /* fctiwz: convert to 32-bit integer (truncate), sign-extended in the FPR */
    {
        int32_t iv = (int32_t) b;
        c->fpr[fd].u64 = (uint64_t) (int64_t) iv;
        break;
    }
    case 14: /* fctiw (round to nearest) */
    {
        int32_t iv = (int32_t) (b >= 0 ? b + 0.5 : b - 0.5);
        c->fpr[fd].u64 = (uint64_t) (int64_t) iv;
        break;
    }

    case 0:  /* fcmpu */
    case 32: /* fcmpo (FP exception not modeled) */
    {
        uint32_t crfd = (insn >> 23) & 7;
        uint32_t so = (c->xer >> 31) & 1u;
        uint32_t field;
        if (a != a || b != b) { /* NaN: unordered */
            field = so;         /* LT=GT=EQ=0 */
        } else if (a > b) {
            /* A CR field is [LT, GT, EQ, SO] with LT the MSB, so GT is bit 2 - (1u << 2).
             * This previously read (2u << 2) = 0b1000, which is the LT bit: every float
             * compare that should have said "greater" reported "less". A `blt` after a float
             * compare therefore branched exactly when it should not, which hung Sonic's
             * neutral special in `while (angle < 0) angle += 2*PI;` - the angle went positive
             * immediately and the loop kept going anyway (observed: f31 had reached 8.9e7). */
            field = (1u << 2) | so; /* GT */
        } else if (a < b) {
            field = (1u << 3) | so; /* LT */
        } else {
            field = (1u << 1) | so; /* EQ */
        }
        gw_ppc_cr_set_field(c, crfd, field);
        break;
    }

    default:
        gw_ppc_bad_opcode(m, ip, insn);
    }
    return 0;
}

/* ---- run loop ------------------------------------------------------------------------ */

static uint32_t gw_ppc_run(gw_ppc_machine *m) {
    uint32_t budget = GW_PPC_MAX_INSNS;
    uint32_t lo = 0xFFFFFFFFu, hi = 0u; /* PC range visited, to describe a runaway loop */
    for (;;) {
        uint32_t ip = m->cpu.pc;
        uint32_t insn;
        if (ip < lo) {
            lo = ip;
        }
        if (ip > hi) {
            hi = ip;
        }
        if (budget-- == 0u) {
            /* Dump the FPRs/GPRs a float loop would be turning on, before panicking: a runaway
             * numeric loop is almost always one operand being wrong (a zero step, an infinity),
             * and the values are the only way to tell which. */
            int k;
            gw_log("ppc: instruction budget exhausted at ip=%s, PC range 0x%08X..0x%08X - "
                   "guest register state follows:", gw_ppc_describe(ip), lo, hi);
            for (k = 0; k < 32; ++k) {
                if (m->cpu.fpr[k].d != 0.0) {
                    gw_log("ppc:   f%-2d = %.9g  (bits 0x%08X%08X)", k, m->cpu.fpr[k].d,
                           (unsigned) (m->cpu.fpr[k].u64 >> 32),
                           (unsigned) (m->cpu.fpr[k].u64 & 0xFFFFFFFFu));
                }
            }
            for (k = 0; k < 32; ++k) {
                if (m->cpu.gpr[k] != 0u) {
                    gw_log("ppc:   r%-2d = 0x%08X", k, m->cpu.gpr[k]);
                }
            }
            gw_panic("ppc: instruction budget (%u) exhausted at ip=%s - guest code is "
                     "looping (PC range %s..0x%08X). This is an infinite loop in the "
                     "interpreted blob, not an interpreter fault.",
                     GW_PPC_MAX_INSNS, gw_ppc_describe(ip), gw_ppc_describe(lo), hi);
        }
        insn = gw_ppc_fetch(m, ip);
        m->cpu.pc = ip + 4;
        if (gw_ppc_execute(m, insn)) {
            return m->cpu.gpr[3];
        }
    }
}

uint32_t gw_ppc_call(uint32_t guest_fn, const uint32_t *gpr_args, int nargs, uint32_t rtoc,
                     uint32_t sp) {
    gw_ppc_machine saved = gw_ppc_m; /* full reentrant save (cpu + bridge) */
    gw_ppc_ctx *c = &gw_ppc_m.cpu;
    uint32_t r3;
    int i;

    /* Refuse to recurse past the cap, and log the guest chain that got here. */
    if (gw_ppc_depth >= GW_PPC_MAX_DEPTH) {
        if (!gw_ppc_depth_logged) {
            gw_ppc_depth_logged = 1;
            gw_log("ppc: call depth cap (%d) hit entering guest 0x%08X - refusing, r3=0. "
                   "Guest chain follows (innermost last):",
                   GW_PPC_MAX_DEPTH, guest_fn);
            for (i = 0; i < GW_PPC_MAX_DEPTH; ++i) {
                gw_log("ppc:   depth %2d: guest %s", i, gw_ppc_describe(gw_ppc_entry[i]));
            }
        }
        return 0;
    }

    /* A nested run must not restart on the caller-supplied stack top: the interrupted guest frame
     * is still live there, and reusing it silently corrupts the outer call's locals and saved
     * registers. Continue below the interrupted frame's own r1 instead. `sp` is still used for the
     * outermost run, and as the fallback if the saved r1 is not a sane MEM1 address. */
    if (gw_ppc_depth > 0) {
        uint32_t outer_sp = saved.cpu.gpr[1];
        if (outer_sp >= GW_PPC_MEM1_BASE + GW_PPC_NEST_GAP &&
            (uint64_t) outer_sp < (uint64_t) GW_PPC_MEM1_BASE + (uint64_t) gw_mem1_size) {
            sp = outer_sp - GW_PPC_NEST_GAP;
        }
    }
    gw_ppc_entry[gw_ppc_depth] = guest_fn;
    ++gw_ppc_depth;

    memset(c, 0, sizeof *c);
    for (i = 0; i < 32; ++i) {
        c->fpr[i].u64 = 0;
    }
    c->gpr[1] = sp;
    c->gpr[2] = rtoc;
    c->lr = 0; /* sentinel: a blr to this returns to the native caller */
    c->pc = guest_fn;
    if (gpr_args != NULL) {
        for (i = 0; i < nargs && i < 8; ++i) {
            c->gpr[3 + i] = gpr_args[i];
        }
    }

    r3 = gw_ppc_run(&gw_ppc_m);
    --gw_ppc_depth;
    gw_ppc_m = saved;
    return r3;
}

/* ---- end-to-end test -------------------------------------------------------------------
 * A hand-assembled PPC blob, encoded big-endian into guest memory, that (a) takes an argument in
 * r3, (b) calls a bridged native helper, (c) stores the helper's return to a guest address given
 * in r4, (d) returns. It uses the standard mflr/mtlr prologue so the final blr returns to the
 * native caller (gw_ppc_call sets LR = 0), exercising mflr/mtlr alongside the bridge and a store.
 *
 * Blob (guest code base GW_PPC_TEST_CODE):
 *   word 0: mflr r0           ; save the native return address (0) into r0
 *   word 1: bl  HELPER_GUEST  ; r3 = helper(r3), LR = +8
 *   word 2: stw r3, 0(r4)     ; *r4 = r3 (big-endian)
 *   word 3: mtlr r0           ; restore LR = 0
 *   word 4: blr               ; return to native caller
 */

#define GW_PPC_TEST_CODE 0x80300000u   /* guest code base (inside MEM1) */
#define GW_PPC_TEST_RESULT 0x80300040u /* guest scratch word the blob writes */
#define GW_PPC_TEST_STACK 0x80400000u  /* guest stack (unused by this leaf) */
#define GW_PPC_TEST_HELPER_GUEST 0x80380358u /* fake guest address of the native helper */

static uint32_t gw_ppc_test_helper(uint32_t x, uint32_t a1, uint32_t a2, uint32_t a3,
                                   uint32_t a4, uint32_t a5, uint32_t a6, uint32_t a7) {
    (void)a1;
    (void)a2;
    (void)a3;
    (void)a4;
    (void)a5;
    (void)a6;
    (void)a7;
    return x * 3u + 7u;
}

static gw_ppc_native_fn gw_ppc_test_resolve(uint32_t guest_addr, void *ctx, gw_ppc_sig *sig) {
    (void)ctx;
    (void)sig;
    if (guest_addr == GW_PPC_TEST_HELPER_GUEST) {
        return gw_ppc_test_helper;
    }
    return NULL;
}

static int test_ppc_call_bridged_helper(void) {
    /* word 1 is `bl helper`: opcode 18, disp = target - pc (byte offset, low 2 bits zero), LK=1 */
    static const uint32_t blob[] = {
        0x7C0802A6u, /* mflr r0 */
        0x48000000u |
            ((GW_PPC_TEST_HELPER_GUEST - (GW_PPC_TEST_CODE + 4)) & 0x03FFFFFCu) | 1u, /* bl */
        0x90640000u, /* stw r3, 0(r4) */
        0x7C0803A6u, /* mtlr r0 */
        0x4E800020u, /* blr */
    };
    const uint32_t input = 42u;
    const uint32_t expected = input * 3u + 7u; /* 133 */
    uint32_t args[2];
    uint32_t r3;
    unsigned i;

    /* Encode the blob big-endian into guest memory. */
    for (i = 0; i < sizeof blob / sizeof blob[0]; ++i) {
        gw_w32((void *)(uintptr_t)(GW_PPC_TEST_CODE + 4 * i), blob[i]);
    }
    gw_w32((void *)(uintptr_t)GW_PPC_TEST_RESULT, 0u);

    args[0] = input;             /* -> r3 */
    args[1] = GW_PPC_TEST_RESULT; /* -> r4 */

    gw_ppc_set_bridge(gw_ppc_test_resolve, NULL, GW_PPC_TEST_CODE,
                      GW_PPC_TEST_CODE + (uint32_t)sizeof blob);

    r3 = gw_ppc_call(GW_PPC_TEST_CODE, args, 2, 0 /* rtoc */, GW_PPC_TEST_STACK);

    if (r3 != expected) {
        gw_test_fail("ppc_call returned r3=0x%08X, expected 0x%08X", r3, expected);
        return 1;
    }
    if (gw_r32((const void *)(uintptr_t)GW_PPC_TEST_RESULT) != expected) {
        gw_test_fail("blob stored 0x%08X at 0x%08X, expected 0x%08X",
                     gw_r32((const void *)(uintptr_t)GW_PPC_TEST_RESULT), GW_PPC_TEST_RESULT,
                     expected);
        return 1;
    }
    return 0;
}

/* ---- float bridge test ------------------------------------------------------------------
 * Exercises the float-argument and float-return marshalling: a blob loads two float constants
 * into f1/f2, passes an integer in r3, calls a bridged helper that takes (word, float, float)
 * and returns a float, then stores the returned f1 back to guest memory. */

#define GW_PPC_TEST_FCODE 0x80300100u        /* guest code base */
#define GW_PPC_TEST_FDATA 0x80300140u        /* float constants + result scratch (r9+0x140..) */
#define GW_PPC_TEST_FHELPER_GUEST 0x80380360u /* fake guest address of the float helper */

static float gw_ppc_test_fhelper(uint32_t x, float a, float b, uint32_t c, uint32_t d,
                                 uint32_t e, uint32_t f, uint32_t g) {
    (void)c;
    (void)d;
    (void)e;
    (void)f;
    (void)g;
    return (float)x + 2.0f * a + b;
}

static gw_ppc_native_fn gw_ppc_test_fresolve(uint32_t guest_addr, void *ctx, gw_ppc_sig *sig) {
    (void)ctx;
    if (guest_addr == GW_PPC_TEST_FHELPER_GUEST) {
        sig->float_args = (1u << 1) | (1u << 2); /* args 1 and 2 are floats (f1, f2) */
        sig->n_args = 3;
        sig->ret_float = 1;
        return (gw_ppc_native_fn)(uintptr_t)gw_ppc_test_fhelper;
    }
    return NULL;
}

static int test_ppc_float_bridge(void) {
    static const uint32_t blob[] = {
        0x7C0802A6u, /* mflr r0 */
        0x3D208030u, /* lis r9, 0x8030 */
        0xC0290140u, /* lfs f1, 0x140(r9) */
        0xC0490144u, /* lfs f2, 0x144(r9) */
        0x3860000Au, /* li r3, 10 */
        0x48000000u |
            ((GW_PPC_TEST_FHELPER_GUEST - (GW_PPC_TEST_FCODE + 20)) & 0x03FFFFFCu) | 1u, /* bl */
        0x3D208030u, /* lis r9, 0x8030 */
        0xD0290148u, /* stfs f1, 0x148(r9) */
        0x7C0803A6u, /* mtlr r0 */
        0x4E800020u, /* blr */
    };
    const float a = 1.5f, b = 2.25f;
    const float expected = 10.0f + 2.0f * a + b; /* 15.25 */
    uint32_t args[1];
    unsigned i;

    for (i = 0; i < sizeof blob / sizeof blob[0]; ++i) {
        gw_w32((void *)(uintptr_t)(GW_PPC_TEST_FCODE + 4 * i), blob[i]);
    }
    gw_wf32((void *)(uintptr_t)GW_PPC_TEST_FDATA, a);
    gw_wf32((void *)(uintptr_t)(GW_PPC_TEST_FDATA + 4), b);
    gw_wf32((void *)(uintptr_t)(GW_PPC_TEST_FDATA + 8), 0.0f);

    args[0] = 10u;

    gw_ppc_set_bridge(gw_ppc_test_fresolve, NULL, GW_PPC_TEST_FCODE,
                      GW_PPC_TEST_FCODE + (uint32_t)sizeof blob);

    gw_ppc_call(GW_PPC_TEST_FCODE, args, 1, 0 /* rtoc */, GW_PPC_TEST_STACK);

    {
        float got = gw_rf32((const void *)(uintptr_t)(GW_PPC_TEST_FDATA + 8));
        if (got != expected) {
            gw_test_fail("float bridge stored %.6f, expected %.6f", got, expected);
            return 1;
        }
    }
    return 0;
}

/* ---- variadic bridge test -----------------------------------------------------------------
 * A variadic callee reads its arguments from no signature, so neither can the bridge: what the
 * tail contains belongs to the CALL SITE. The PowerPC EABI makes the caller say so out loud -
 * `creqv 6,6,6` before the bl when an argument went into an FPR, `crxor 6,6,6` when none did -
 * and gw_ppc_bridge_call marshals from that bit, because the native i686 callee walks a flat
 * stack on which a float vararg has already been promoted to an eight-byte double.
 *
 * Both halves matter and both are checked here. The crclr half is the behaviour that the old
 * integer default happened to get right (all-pointer varargs, e.g. efSync_Spawn) and must not
 * regress; the crset half is the one it silently dropped, which is what made
 * HSD_ForeachAnim(..., AOBJ_ARG_AF, frame) request a garbage animation frame.
 *
 * The helper is genuinely variadic and is called through the bridge's fixed-arity cdecl pointer,
 * exactly as every real vararg target is: on i686 cdecl the caller builds and cleans the stack,
 * so the shapes agree. */

#define GW_PPC_TEST_VCODE_F 0x80300200u       /* blob: float vararg (crset) */
#define GW_PPC_TEST_VCODE_W 0x80300240u       /* blob: word vararg (crclr) */
#define GW_PPC_TEST_VHELPER_GUEST 0x80380368u /* fake guest address of the variadic helper */

static uint32_t gw_ppc_test_vhelper(uint32_t kind, uint32_t unused, ...) {
    va_list ap;
    uint32_t r;
    (void)unused;
    va_start(ap, unused);
    if (kind == 1u) {
        /* The default argument promotions make a float vararg a double. */
        double d = va_arg(ap, double);
        r = (uint32_t)(int32_t)(d * 100.0);
    } else {
        r = va_arg(ap, uint32_t);
    }
    va_end(ap);
    return r;
}

static gw_ppc_native_fn gw_ppc_test_vresolve(uint32_t guest_addr, void *ctx, gw_ppc_sig *sig) {
    (void)ctx;
    if (guest_addr == GW_PPC_TEST_VHELPER_GUEST) {
        sig->float_args = GW_PPC_SIG_VARARGS; /* no fixed float; variadic tail */
        sig->n_args = 2;                      /* kind, unused */
        sig->ret_float = 0;
        return (gw_ppc_native_fn)(uintptr_t)gw_ppc_test_vhelper;
    }
    return NULL;
}

static int test_ppc_varargs_bridge(void) {
    /* mflr r0; lis r9,0x8030; lfs f1,0x150(r9); li r3,1; li r4,0; creqv 6,6,6; bl helper;
     * lis r9,0x8030; stw r3,0x154(r9); mtlr r0; blr */
    static const uint32_t blob_f[] = {
        0x7C0802A6u, 0x3D208030u, 0xC0290150u, 0x38600001u, 0x38800000u, 0x4CC63242u,
        0x48000000u | ((GW_PPC_TEST_VHELPER_GUEST - (GW_PPC_TEST_VCODE_F + 24)) & 0x03FFFFFCu) |
            1u,
        0x3D208030u, 0x90690154u, 0x7C0803A6u, 0x4E800020u,
    };
    /* mflr r0; li r3,2; li r4,0; lis r5,0x00C0; ori r5,r5,0xFFEE; crxor 6,6,6; bl helper;
     * lis r9,0x8030; stw r3,0x158(r9); mtlr r0; blr */
    static const uint32_t blob_w[] = {
        0x7C0802A6u, 0x38600002u, 0x38800000u, 0x3CA000C0u, 0x60A5FFEEu, 0x4CC63182u,
        0x48000000u | ((GW_PPC_TEST_VHELPER_GUEST - (GW_PPC_TEST_VCODE_W + 24)) & 0x03FFFFFCu) |
            1u,
        0x3D208030u, 0x90690158u, 0x7C0803A6u, 0x4E800020u,
    };
    const float f = 3.5f;
    unsigned i;
    uint32_t got;

    for (i = 0; i < sizeof blob_f / sizeof blob_f[0]; ++i) {
        gw_w32((void *)(uintptr_t)(GW_PPC_TEST_VCODE_F + 4 * i), blob_f[i]);
        gw_w32((void *)(uintptr_t)(GW_PPC_TEST_VCODE_W + 4 * i), blob_w[i]);
    }
    gw_wf32((void *)(uintptr_t)0x80300150u, f);
    gw_w32((void *)(uintptr_t)0x80300154u, 0u);
    gw_w32((void *)(uintptr_t)0x80300158u, 0u);

    gw_ppc_set_bridge(gw_ppc_test_vresolve, NULL, GW_PPC_TEST_VCODE_F,
                      GW_PPC_TEST_VCODE_F + (uint32_t)sizeof blob_f);
    gw_ppc_call(GW_PPC_TEST_VCODE_F, NULL, 0, 0, GW_PPC_TEST_STACK);
    got = gw_r32((const void *)(uintptr_t)0x80300154u);
    if (got != 350u) {
        gw_test_fail("float vararg (crset) arrived as %u, expected 350 (3.5 * 100)", got);
        return 1;
    }

    gw_ppc_set_bridge(gw_ppc_test_vresolve, NULL, GW_PPC_TEST_VCODE_W,
                      GW_PPC_TEST_VCODE_W + (uint32_t)sizeof blob_w);
    gw_ppc_call(GW_PPC_TEST_VCODE_W, NULL, 0, 0, GW_PPC_TEST_STACK);
    got = gw_r32((const void *)(uintptr_t)0x80300158u);
    if (got != 0x00C0FFEEu) {
        gw_test_fail("word vararg (crclr) arrived as 0x%08X, expected 0x00C0FFEE", got);
        return 1;
    }
    return 0;
}

/* ---- static-global bridge test -----------------------------------------------------------
 * The interpreter must route a load/store of a bridge-table data object (kind 0) through the
 * NATIVE storage, not raw MEM1. ftData_803C52A0 (guest 0x803C52A0) is a game .data table whose
 * bytes are big-endian (gw_apply_fixups has run); a blob loading it must see the same value as a
 * direct native read, and a blob store must land in native storage. This is the interpreter-level
 * proof of the guest->native static bridge, independent of any in-match game state. */

#define GW_PPC_TEST_STATIC_GUEST 0x803C52A0u /* ftData_803C52A0 (kind-0 .data object) */

static int test_ppc_static_bridge(void) {
    /* lis r3,0x803C ; lwz r3,0x52A0(r3) ; blr  -> return *(0x803C52A0) via the bridge */
    static const uint32_t load_blob[] = {
        0x3C60803Cu, /* lis r3, 0x803C */
        0x806352A0u, /* lwz r3, 0x52A0(r3) */
        0x4E800020u, /* blr */
    };
    /* lis r4,0x803C ; lis r3,0xC0FF ; ori r3,r3,0xEE11 ; stw r3,0x52A0(r4) ; blr */
    static const uint32_t store_blob[] = {
        0x3C80803Cu, /* lis r4, 0x803C */
        0x3C60C0FFu, /* lis r3, 0xC0FF */
        0x6063EE11u, /* ori r3, r3, 0xEE11 */
        0x906452A0u, /* stw r3, 0x52A0(r4) */
        0x4E800020u, /* blr */
    };
    const uint32_t magic = 0xC0FFEE11u;
    int kind = -1;
    uint32_t native = gw_mex_bridge_lookup(GW_PPC_TEST_STATIC_GUEST, &kind);
    uint32_t saved, expected, got;
    unsigned i;

    if (native == 0 || kind != 0) {
        gw_test_fail("bridge lookup of ftData_803C52A0 (0x803C52A0) failed: native=0x%08X kind=%d",
                     native, kind);
        return 1;
    }

    /* 1. load: the blob returns *(static) and must equal a direct native read. */
    for (i = 0; i < sizeof load_blob / sizeof load_blob[0]; ++i) {
        gw_w32((void *)(uintptr_t)(GW_PPC_TEST_CODE + 4 * i), load_blob[i]);
    }
    gw_ppc_set_bridge(gw_ppc_test_resolve, NULL, GW_PPC_TEST_CODE,
                      GW_PPC_TEST_CODE + (uint32_t)sizeof load_blob);
    expected = gw_r32((const void *)(uintptr_t)native);
    got = gw_ppc_call(GW_PPC_TEST_CODE, NULL, 0, 0, GW_PPC_TEST_STACK);
    if (got != expected) {
        gw_test_fail("static load returned 0x%08X, expected native value 0x%08X", got, expected);
        return 1;
    }

    /* 2. store: the blob writes MAGIC to the static; native storage must reflect it (then restore). */
    saved = gw_r32((const void *)(uintptr_t)native);
    for (i = 0; i < sizeof store_blob / sizeof store_blob[0]; ++i) {
        gw_w32((void *)(uintptr_t)(GW_PPC_TEST_CODE + 4 * i), store_blob[i]);
    }
    gw_ppc_set_bridge(gw_ppc_test_resolve, NULL, GW_PPC_TEST_CODE,
                      GW_PPC_TEST_CODE + (uint32_t)sizeof store_blob);
    gw_ppc_call(GW_PPC_TEST_CODE, NULL, 0, 0, GW_PPC_TEST_STACK);
    got = gw_r32((const void *)(uintptr_t)native);
    gw_w32((void *)(uintptr_t)native, saved); /* restore the table slot */
    if (got != magic) {
        gw_test_fail("static store left 0x%08X, expected 0x%08X", got, magic);
        return 1;
    }

    return 0;
}

/* ---- re-entry cap test -------------------------------------------------------------------
 * Regression test for the double-jump crash: Sonic's onDoubleJump override calls the very engine
 * function whose dispatch site invoked it, so the interpreter re-entered itself without bound and
 * the process died on a blown native stack (last log line: "onDoubleJump ... invocation 1
 * running", no matching "ran"). The semantic fix is in gw_Mex_GObjDispatch; GW_PPC_MAX_DEPTH is
 * the interpreter's backstop for any cycle that misses.
 *
 * Here a bridged helper re-enters gw_ppc_call on the same blob, i.e. unbounded recursion by
 * construction. The cap must stop it: the call returns normally, the nesting never exceeds
 * GW_PPC_MAX_DEPTH, and the refused innermost call yields 0. Without the cap this test does not
 * fail - it takes the process down, which is precisely the bug. */

#define GW_PPC_TEST_RECURSE_GUEST 0x80380360u /* fake guest address of the recursing helper */

static int gw_ppc_test_recurse_max; /* deepest gw_ppc_depth observed inside the helper */

static gw_ppc_native_fn gw_ppc_test_recurse_resolve(uint32_t guest_addr, void *ctx,
                                                    gw_ppc_sig *sig);

static uint32_t gw_ppc_test_recurse_helper(uint32_t a0, uint32_t a1, uint32_t a2, uint32_t a3,
                                           uint32_t a4, uint32_t a5, uint32_t a6, uint32_t a7) {
    (void)a0; (void)a1; (void)a2; (void)a3; (void)a4; (void)a5; (void)a6; (void)a7;
    if (gw_ppc_depth > gw_ppc_test_recurse_max) {
        gw_ppc_test_recurse_max = gw_ppc_depth;
    }
    /* Re-enter the interpreter on the same blob - the cycle the cap exists to bound. */
    return gw_ppc_call(GW_PPC_TEST_CODE, NULL, 0, 0, GW_PPC_TEST_STACK);
}

static gw_ppc_native_fn gw_ppc_test_recurse_resolve(uint32_t guest_addr, void *ctx,
                                                    gw_ppc_sig *sig) {
    (void)ctx;
    (void)sig;
    if (guest_addr == GW_PPC_TEST_RECURSE_GUEST) {
        return gw_ppc_test_recurse_helper;
    }
    return NULL;
}

static int test_ppc_reentry_cap(void) {
    /* mflr r0 ; bl RECURSE_HELPER ; mtlr r0 ; blr  -> r3 = helper() */
    static const uint32_t blob[] = {
        0x7C0802A6u, /* mflr r0 */
        0x48000000u |
            ((GW_PPC_TEST_RECURSE_GUEST - (GW_PPC_TEST_CODE + 4)) & 0x03FFFFFCu) | 1u, /* bl */
        0x7C0803A6u, /* mtlr r0 */
        0x4E800020u, /* blr */
    };
    int saved_logged = gw_ppc_depth_logged;
    uint32_t r3;
    unsigned i;

    for (i = 0; i < sizeof blob / sizeof blob[0]; ++i) {
        gw_w32((void *)(uintptr_t)(GW_PPC_TEST_CODE + 4 * i), blob[i]);
    }
    gw_ppc_test_recurse_max = 0;
    gw_ppc_depth_logged = 1; /* keep the 16-line cap dump out of the test log */

    gw_ppc_set_bridge(gw_ppc_test_recurse_resolve, NULL, GW_PPC_TEST_CODE,
                      GW_PPC_TEST_CODE + (uint32_t)sizeof blob);

    /* Reaching the next line at all is most of the point: unbounded, this never returns. */
    r3 = gw_ppc_call(GW_PPC_TEST_CODE, NULL, 0, 0, GW_PPC_TEST_STACK);

    gw_ppc_depth_logged = saved_logged; /* leave the one-shot for a real in-game cap hit */

    if (gw_ppc_test_recurse_max != GW_PPC_MAX_DEPTH) {
        gw_test_fail("re-entry reached depth %d, expected the cap %d", gw_ppc_test_recurse_max,
                     GW_PPC_MAX_DEPTH);
        return 1;
    }
    if (r3 != 0u) {
        gw_test_fail("the refused call should yield r3=0, got 0x%08X", r3);
        return 1;
    }
    if (gw_ppc_depth != 0) {
        gw_test_fail("gw_ppc_depth left at %d after unwinding, expected 0", gw_ppc_depth);
        return 1;
    }
    return 0;
}

/* ---- A-form FP decode test -----------------------------------------------------------
 * Regression test for the neutral-B crash. Two distinct bugs lived in the opcode 59/63 decode:
 *   1. XO was masked to 10 bits for every form, but an A-form op's XO is 5 bits (26..30); the
 *      upper five bits of the wide mask are frC. So any fmul with frC != 0 missed its case and
 *      raised "unimplemented opcode" - exactly what killed Sonic's neutral special
 *      (0xED8C0332 = fmuls f12,f12,f12 at guest 0x807F8490).
 *   2. fmul was computed as frA * frB. The ISA says frD = frA * frC.
 * Both are invisible unless frC is both non-zero and different from frB, so the test uses three
 * distinct registers with distinct values: only frA*frC gives 3.5 * 11.0.
 */
static int test_ppc_fp_aform_decode(void) {
    /* lfs f3,0(r3) ; lfs f4,4(r3) ; lfs f5,8(r3) ; fmuls f2,f3,f5 ; fmadds f6,f3,f5,f4 ;
     * stfs f2,12(r3) ; stfs f6,16(r3) ; blr
     * f3 = fA = 3.5, f4 = fB = 100.0, f5 = fC = 11.0. fmuls must yield fA*fC = 38.5 (NOT
     * fA*fB = 350.0), and fmadds must yield fA*fC + fB = 138.5. */
    static const uint32_t blob[] = {
        0xC0630000u, /* lfs  f3, 0(r3)  */
        0xC0830004u, /* lfs  f4, 4(r3)  */
        0xC0A30008u, /* lfs  f5, 8(r3)  */
        0xEC430172u, /* fmuls f2, f3, f5      (A-form XO=25, frC=5) */
        0xECC3217Au, /* fmadds f6, f3, f5, f4 (A-form XO=29, frC=5) */
        0xD043000Cu, /* stfs f2, 12(r3) */
        0xD0C30010u, /* stfs f6, 16(r3) */
        0x4E800020u, /* blr */
    };
    const float fa = 3.5f, fb = 100.0f, fc = 11.0f;
    const float want_mul = fa * fc;        /* 38.5  */
    const float want_madd = fa * fc + fb;  /* 138.5 */
    uint32_t args[1];
    float got_mul, got_madd;
    unsigned i;

    for (i = 0; i < sizeof blob / sizeof blob[0]; ++i) {
        gw_w32((void *) (uintptr_t) (GW_PPC_TEST_CODE + 4 * i), blob[i]);
    }
    gw_wf32((void *) (uintptr_t) (GW_PPC_TEST_FDATA + 0), fa);
    gw_wf32((void *) (uintptr_t) (GW_PPC_TEST_FDATA + 4), fb);
    gw_wf32((void *) (uintptr_t) (GW_PPC_TEST_FDATA + 8), fc);
    gw_wf32((void *) (uintptr_t) (GW_PPC_TEST_FDATA + 12), 0.0f);
    gw_wf32((void *) (uintptr_t) (GW_PPC_TEST_FDATA + 16), 0.0f);

    args[0] = GW_PPC_TEST_FDATA;
    gw_ppc_set_bridge(gw_ppc_test_resolve, NULL, GW_PPC_TEST_CODE,
                      GW_PPC_TEST_CODE + (uint32_t) sizeof blob);
    gw_ppc_call(GW_PPC_TEST_CODE, args, 1, 0, GW_PPC_TEST_STACK);

    got_mul = gw_rf32((const void *) (uintptr_t) (GW_PPC_TEST_FDATA + 12));
    got_madd = gw_rf32((const void *) (uintptr_t) (GW_PPC_TEST_FDATA + 16));
    if (got_mul != want_mul) {
        gw_test_fail("fmuls gave %.4f, expected fA*fC = %.4f (fA*fB would be %.4f)", got_mul,
                     want_mul, fa * fb);
        return 1;
    }
    if (got_madd != want_madd) {
        gw_test_fail("fmadds gave %.4f, expected fA*fC+fB = %.4f", got_madd, want_madd);
        return 1;
    }
    return 0;
}

/* ---- float compare test ---------------------------------------------------------------
 * Regression test for the neutral-B hang. fcmpu encoded GT as (2u << 2) = 0b1000, which is the
 * LT bit of the [LT, GT, EQ, SO] field - so a float compare reporting "greater" set "less", and
 * every blt/bgt after a float compare took the wrong path. Checks all three orderings, because
 * only GT was wrong and testing one direction would have missed it. */
static int test_ppc_fcmpu_orderings(void) {
    /* lfs f1,0(r3) ; lfs f2,4(r3) ; fcmpu cr0,f1,f2 ; (mfcr r4) ; stw r4,8(r3) ; blr */
    static const uint32_t blob[] = {
        0xC0230000u, /* lfs  f1, 0(r3) */
        0xC0430004u, /* lfs  f2, 4(r3) */
        0xFC011000u, /* fcmpu cr0, f1, f2 */
        0x7C800026u, /* mfcr r4 */
        0x90830008u, /* stw  r4, 8(r3) */
        0x4E800020u, /* blr */
    };
    /* CR0 occupies the top nibble of CR: LT=0x80000000, GT=0x40000000, EQ=0x20000000. */
    static const struct { float a, b; uint32_t want; const char *name; } cases[] = {
        {1.0f, 2.0f, 0x80000000u, "less"},
        {2.0f, 1.0f, 0x40000000u, "greater"},
        {1.5f, 1.5f, 0x20000000u, "equal"},
    };
    unsigned i;

    for (i = 0; i < sizeof blob / sizeof blob[0]; ++i) {
        gw_w32((void *) (uintptr_t) (GW_PPC_TEST_CODE + 4 * i), blob[i]);
    }
    gw_ppc_set_bridge(gw_ppc_test_resolve, NULL, GW_PPC_TEST_CODE,
                      GW_PPC_TEST_CODE + (uint32_t) sizeof blob);

    for (i = 0; i < sizeof cases / sizeof cases[0]; ++i) {
        uint32_t args[1];
        uint32_t got;
        gw_wf32((void *) (uintptr_t) (GW_PPC_TEST_FDATA + 0), cases[i].a);
        gw_wf32((void *) (uintptr_t) (GW_PPC_TEST_FDATA + 4), cases[i].b);
        gw_w32((void *) (uintptr_t) (GW_PPC_TEST_FDATA + 8), 0u);
        args[0] = GW_PPC_TEST_FDATA;
        gw_ppc_call(GW_PPC_TEST_CODE, args, 1, 0, GW_PPC_TEST_STACK);
        got = gw_r32((const void *) (uintptr_t) (GW_PPC_TEST_FDATA + 8)) & 0xF0000000u;
        if (got != cases[i].want) {
            gw_test_fail("fcmpu %.1f vs %.1f (%s): CR0 = 0x%08X, expected 0x%08X",
                         (double) cases[i].a, (double) cases[i].b, cases[i].name, got,
                         cases[i].want);
            return 1;
        }
    }
    return 0;
}

/* ---- xoris / int->float idiom test ------------------------------------------------------
 * Exercises xoris the way compilers actually use it - the classic int -> float conversion:
 *   xoris r4,r3,0x8000 ; stw r4,4(r5) ; lis r4,0x4330 ; stw r4,0(r5) ; lfd f1,0(r5)
 *   ; lfd f2,8(r5) ; fsub f1,f1,f2 ; stfs f1,16(r5)
 * where 8(r5) holds the bias double 0x4330000080000000. Checks negative, zero and positive ints,
 * because a wrong xoris (e.g. dropping the << 16) breaks the sign handling specifically. */
static int test_ppc_xoris_int_to_float(void) {
    static const uint32_t blob[] = {
        0x6C648000u, /* xoris r4, r3, 0x8000 */
        0x90850004u, /* stw   r4, 4(r5)      */
        0x3C804330u, /* lis   r4, 0x4330     */
        0x90850000u, /* stw   r4, 0(r5)      */
        0xC8250000u, /* lfd   f1, 0(r5)      */
        0xC8450008u, /* lfd   f2, 8(r5)      */
        0xFC211028u, /* fsub  f1, f1, f2     */
        0xD0250010u, /* stfs  f1, 16(r5)     */
        0x4E800020u, /* blr                  */
    };
    static const int32_t ins[] = {-7, 0, 12345};
    unsigned i;

    for (i = 0; i < sizeof blob / sizeof blob[0]; ++i) {
        gw_w32((void *) (uintptr_t) (GW_PPC_TEST_CODE + 4 * i), blob[i]);
    }
    gw_ppc_set_bridge(gw_ppc_test_resolve, NULL, GW_PPC_TEST_CODE,
                      GW_PPC_TEST_CODE + (uint32_t) sizeof blob);
    for (i = 0; i < sizeof ins / sizeof ins[0]; ++i) {
        uint32_t args[3];
        float got;
        gw_w32((void *) (uintptr_t) (GW_PPC_TEST_FDATA + 8), 0x43300000u);  /* bias hi */
        gw_w32((void *) (uintptr_t) (GW_PPC_TEST_FDATA + 12), 0x80000000u); /* bias lo */
        gw_wf32((void *) (uintptr_t) (GW_PPC_TEST_FDATA + 16), -999.0f);
        args[0] = (uint32_t) ins[i];      /* r3 */
        args[1] = 0u;                     /* r4 */
        args[2] = GW_PPC_TEST_FDATA;      /* r5 */
        gw_ppc_call(GW_PPC_TEST_CODE, args, 3, 0, GW_PPC_TEST_STACK);
        got = gw_rf32((const void *) (uintptr_t) (GW_PPC_TEST_FDATA + 16));
        if (got != (float) ins[i]) {
            gw_test_fail("int->float via xoris: %d came out as %.3f", ins[i], (double) got);
            return 1;
        }
    }
    return 0;
}

void gw_ppc_tests_register(void) {
    gw_test_register("ppc_call_bridged_helper", test_ppc_call_bridged_helper);
    gw_test_register("ppc_float_bridge", test_ppc_float_bridge);
    gw_test_register("ppc_varargs_bridge", test_ppc_varargs_bridge);
    gw_test_register("ppc_static_bridge", test_ppc_static_bridge);
    gw_test_register("ppc_reentry_cap", test_ppc_reentry_cap);
    gw_test_register("ppc_fp_aform_decode", test_ppc_fp_aform_decode);
    gw_test_register("ppc_fcmpu_orderings", test_ppc_fcmpu_orderings);
    gw_test_register("ppc_xoris_int_to_float", test_ppc_xoris_int_to_float);
}
