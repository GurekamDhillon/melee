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
    /* INTERIOR ADDRESSES. The lookup above matches an object's BASE only, so `global[i]` for
     * every i != 0 missed it and fell through to MEM1 - where the object is not. That is a
     * silent wrong read on every interior access to a game global, and it is the forward
     * direction of exactly the problem gw_mex_bridge_is_native_data already solves for
     * pointers coming back the other way. Two of the stage sweep's resolver failures were
     * branches to gmResultCharacterData +0x20 and +0x34, which is what a garbage read out of
     * a global looks like once the value is used. */
    return gw_mex_bridge_guest_data(ea);
}

/* A word argument crossing into NATIVE code. The interpreter re-routes its own accesses to a game
 * static (above), but a static's guest address handed to a native function as a POINTER argument
 * used to arrive raw, and the native side then read MEM1 at that address - where the static is not.
 * GrSp.dat (ext:307) does `HSD_SetupTevStageAll(&tev$297)` with the vanilla static at guest
 * 0x80407370; the TEV descriptor read back as garbage and HSD_TevStage2Num asserted on a stage
 * number above 15. Heap and stack addresses miss the lookup and pass through unchanged. */
static uint32_t gw_ppc_arg_word(uint32_t v) {
    uint32_t native = gw_ppc_static_native(v);
    return native != 0u ? native : v;
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

/* Defined with the code-range table further down; declared here so the fault dump can name the
 * ranges. It has to: a blob installed through gw_ppc_add_code_range leaves the machine's own
 * [code_lo, code_hi) at [0, 0), so printing only that pair says "no code" about a blob that is
 * running perfectly well. The stage path does exactly this. */
static void gw_ppc_log_code_ranges(void);

/* What a register value looks like: 0 = not a pointer worth dumping, 1 = guest MEM1,
 * 2 = a NATIVE address inside one of the game's globals. Telling those two apart in the dump is
 * the whole diagnosis for a bridged function that returned a pointer to a global - it is what
 * turns "ea=0x107819A0" into "r3 is grDatFiles_8049EE10's native storage". */
static int gw_ppc_ptr_kind(uint32_t v) {
    if ((v & 3u) != 0u) {
        return 0;
    }
    if (gw_ppc_ea_ok(v, 0x40u)) {
        return 1;
    }
    /* Only 4 bytes need to be inside the object: the 0x40-byte hexdump may run past a small
     * global into whatever .data follows it, which is mapped image memory and cannot fault -
     * and seeing the neighbours is usually what identifies the object anyway. */
    if (gw_mex_bridge_is_native_data(v, 4)) {
        return 2;
    }
    return 0;
}

/* 0x40 bytes at a guest address, two lines of eight big-endian words. 0x40 rather than 0x20
 * because that is what tells the engine's structures apart by eye: an HSD_JObj's scale sits at
 * +0x2C..+0x34 and a Fighter's facing_dir at +0x2C, and a pointer holding the wrong KIND of
 * object is the usual reason an interpreted load faults. */
static void gw_ppc_dump_mem(const char *label, uint32_t v) {
    int k;
    for (k = 0; k < 2; ++k) {
        uint32_t a = v + (uint32_t)(0x20 * k);
        gw_log("ppc:   %s+0x%02X 0x%08X: %08X %08X %08X %08X %08X %08X %08X %08X", label,
               0x20 * k, a, gw_r32((const void *)(uintptr_t)(a + 0x00)),
               gw_r32((const void *)(uintptr_t)(a + 0x04)),
               gw_r32((const void *)(uintptr_t)(a + 0x08)),
               gw_r32((const void *)(uintptr_t)(a + 0x0C)),
               gw_r32((const void *)(uintptr_t)(a + 0x10)),
               gw_r32((const void *)(uintptr_t)(a + 0x14)),
               gw_r32((const void *)(uintptr_t)(a + 0x18)),
               gw_r32((const void *)(uintptr_t)(a + 0x1C)));
    }
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
    gw_log("ppc:   bridge code range [0x%08X,0x%08X)  depth = %d", m->code_lo, m->code_hi,
           gw_ppc_depth);
    gw_ppc_log_code_ranges();

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
        int kind = gw_ppc_ptr_kind(v);
        if (kind == 0) {
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
        {
            char label[24];
            snprintf(label, sizeof label, "r%d%s", i, (kind == 2) ? " NATIVE-GLOBAL" : "");
            gw_ppc_dump_mem(label, v);
        }
    }
    gw_log("ppc: --- end of interpreter state ---");
}

static void gw_ppc_access_violation(const gw_ppc_machine *m, uint32_t ip, uint32_t ea) {
    gw_ppc_dump_state(m, ip);
    gw_panic("ppc: guest access violation at ip=%s ea=0x%08X", gw_ppc_describe(ip), ea);
}

/* Resolve an interpreted effective address to the native address that really holds those `size`
 * bytes, or panic cleanly. Three cases, tried in order:
 *
 *   1. a GUEST address naming a game static -> its native storage, through the bridge's exact-base
 *      guest->native lookup: the statics live in this exe's .data, not in MEM1;
 *   2. a GUEST address inside MEM1          -> itself (heap, stack, blob code and blob data);
 *   3. an address that is ALREADY NATIVE    -> itself, when it lies wholly inside a game global.
 *
 * Case 3 is not a special case for one function; it is the missing half of case 1. The bridge
 * translates guest->native at the point of ACCESS, but an engine function reached over the bridge
 * can RETURN a pointer to a global - grDatFiles_801C6330 ends `return &grDatFiles_8049EE10[i]` -
 * and nothing translates a returned pointer. Testing the ADDRESS rather than its provenance is
 * what makes that work no matter how far the pointer travels first: guest code may store it into
 * a struct in MEM1 and dereference it many frames later, and the load still resolves.
 *
 * It is a RANGE test, not an exact-base one, because `&global[i]` is interior for every i != 0 -
 * an exact-base reverse lookup would miss those and succeed only for i == 0, which is the kind of
 * half-working that hides a bug rather than reporting it.
 *
 * All three cases go through the same big-endian gw_rN/gw_wN accessors: gwtool byte-swaps every
 * game memory access, so a static's native storage holds big-endian bytes exactly as MEM1 does.
 *
 * Note the asymmetry: case 1 cannot bound the access (a bridge entry carries no size), while
 * cases 2 and 3 do. Case 1 is reached only from an exact symbol base, so it is already as narrow
 * as the table can express. */
static uint32_t gw_ppc_resolve_ea(gw_ppc_machine *m, uint32_t ea, uint32_t size) {
    uint32_t native = gw_ppc_static_native(ea);
    if (native != 0) {
        return native;
    }
    if (gw_ppc_ea_ok(ea, size)) {
        return ea;
    }
    if (gw_mex_bridge_is_native_data(ea, size)) {
        return ea;
    }
    gw_ppc_access_violation(m, m->cpu.pc - 4, ea);
    return 0; /* not reached: gw_panic does not return */
}

static uint8_t gw_ppc_ld8(gw_ppc_machine *m, uint32_t ea) {
    return gw_r8((const void *)(uintptr_t)gw_ppc_resolve_ea(m, ea, 1));
}
static uint16_t gw_ppc_ld16(gw_ppc_machine *m, uint32_t ea) {
    return gw_r16((const void *)(uintptr_t)gw_ppc_resolve_ea(m, ea, 2));
}
static uint32_t gw_ppc_ld32(gw_ppc_machine *m, uint32_t ea) {
    return gw_r32((const void *)(uintptr_t)gw_ppc_resolve_ea(m, ea, 4));
}
static uint64_t gw_ppc_ld64(gw_ppc_machine *m, uint32_t ea) {
    return gw_r64((const void *)(uintptr_t)gw_ppc_resolve_ea(m, ea, 8));
}
/* The GX write-gather pipe. Guest code built against the SDK's GXVert.h writes vertex data with
 * plain stores to 0xCC008000 (GXPosition3f32 & co. are static inlines there); Akaneia's GrMVb.dat
 * (ext:294) draws its ball trail that way. Each store is forwarded to Aurora's FIFO, which takes
 * host-order values and appends them big-endian - the same bytes the pipe would have received. A
 * float store arrives here as its IEEE bits, which is exactly what the pipe carries. */
#define GW_PPC_GX_WGPIPE 0xCC008000u
extern void GXCmd1u8(unsigned char x);
extern void GXCmd1u16(unsigned short x);
extern void GXCmd1u32(unsigned int x);

static void gw_ppc_st8(gw_ppc_machine *m, uint32_t ea, uint8_t v) {
    if (ea == GW_PPC_GX_WGPIPE) {
        GXCmd1u8(v);
        return;
    }
    gw_w8((void *)(uintptr_t)gw_ppc_resolve_ea(m, ea, 1), v);
}
static void gw_ppc_st16(gw_ppc_machine *m, uint32_t ea, uint16_t v) {
    if (ea == GW_PPC_GX_WGPIPE) {
        GXCmd1u16(v);
        return;
    }
    gw_w16((void *)(uintptr_t)gw_ppc_resolve_ea(m, ea, 2), v);
}
static void gw_ppc_st32(gw_ppc_machine *m, uint32_t ea, uint32_t v) {
    if (ea == GW_PPC_GX_WGPIPE) {
        GXCmd1u32(v);
        return;
    }
    gw_w32((void *)(uintptr_t)gw_ppc_resolve_ea(m, ea, 4), v);
}
static void gw_ppc_st64(gw_ppc_machine *m, uint32_t ea, uint64_t v) {
    if (ea == GW_PPC_GX_WGPIPE) {
        GXCmd1u32((unsigned int)(v >> 32));
        GXCmd1u32((unsigned int)v);
        return;
    }
    gw_w64((void *)(uintptr_t)gw_ppc_resolve_ea(m, ea, 8), v);
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

/* The variadic tail of a printf-family call, in the order its format string names it.
 *
 * The CR6 rule below can only say whether the FIRST vararg is a float, so a call that passes a
 * word before a double - `OSReport("%s in %fms\n", name, ms)`, which GrGc.dat (ext:306) makes on
 * load - had the double's bits placed where the %s pointer belonged. Akaneia printed " in
 * -0.000ms"; ACE handed 0xA0000000 to %s and faulted inside ucrtbase. For printf-family targets the
 * call site's order is not lost at all: it is written in the format string, which is the last fixed
 * argument. So when that argument is a readable guest string containing a conversion, walk it and
 * take each value from the register file its class uses - a word from the next GPR, a double from
 * the next FPR (two i686 slots), a long long from the next odd-aligned GPR pair (two slots, low word
 * first). Returns 0 when the last fixed argument is not such a string, and the caller falls back to
 * the CR6 rule (efSync_Spawn and the other non-format varargs). */
static int gw_ppc_guest_byte(gw_ppc_machine *m, uint32_t a, uint8_t *out) {
    if (!gw_ppc_ea_ok(a, 1) && !gw_mex_bridge_is_native_data(a, 1)) {
        return 0;
    }
    *out = gw_ppc_ld8(m, a);
    return 1;
}

static int gw_ppc_varargs_from_format(gw_ppc_machine *m, uint32_t fmt, uint32_t *args,
                                      uint32_t slot, int *gpr_i, int *fpr_i) {
    gw_ppc_ctx *c = &m->cpu;
    uint32_t p;
    uint8_t ch;
    int any = 0;
    uint32_t n;

    /* A format string first: readable, NUL-terminated within a sane bound, and naming at least
     * one conversion. Anything else is not ours to interpret. */
    for (n = 0; n < 1024u; ++n) {
        if (!gw_ppc_guest_byte(m, fmt + n, &ch)) {
            return 0;
        }
        if (ch == 0) {
            break;
        }
        if (ch == '%') {
            any = 1;
        }
    }
    if (n == 1024u || !any) {
        return 0;
    }

    for (p = fmt; gw_ppc_guest_byte(m, p, &ch) && ch != 0; ++p) {
        int longs = 0;
        if (ch != '%') {
            continue;
        }
        ++p;
        (void)gw_ppc_guest_byte(m, p, &ch);
        if (ch == '%') {
            continue;
        }
        while (ch == '-' || ch == '+' || ch == ' ' || ch == '#' || ch == '0') {
            (void)gw_ppc_guest_byte(m, ++p, &ch);
        }
        /* '*' width/precision consume an int argument each */
        for (;;) {
            if (ch == '*') {
                if (slot < 8u && *gpr_i <= 10) {
                    args[slot++] = c->gpr[(*gpr_i)++];
                }
                (void)gw_ppc_guest_byte(m, ++p, &ch);
            } else if ((ch >= '0' && ch <= '9') || ch == '.') {
                (void)gw_ppc_guest_byte(m, ++p, &ch);
            } else {
                break;
            }
        }
        while (ch == 'h' || ch == 'l' || ch == 'L' || ch == 'q' || ch == 'j' || ch == 'z' ||
               ch == 't') {
            if (ch == 'l' || ch == 'q' || ch == 'L') {
                ++longs;
            }
            (void)gw_ppc_guest_byte(m, ++p, &ch);
        }
        if (ch == 0) {
            break;
        }
        switch (ch) {
        case 'f': case 'F': case 'e': case 'E': case 'g': case 'G': case 'a': case 'A':
            if (slot + 2u <= 8u && *fpr_i <= 8) {
                double d = c->fpr[(*fpr_i)++].d;
                memcpy(&args[slot], &d, sizeof d);
                slot += 2u;
            }
            break;
        default:
            if (longs >= 2 && ch != 's' && ch != 'c' && ch != 'p' && ch != 'n') {
                /* long long: an odd-aligned GPR pair, high word first on PowerPC */
                if (((*gpr_i - 3) & 1) != 0) {
                    ++*gpr_i;
                }
                if (slot + 2u <= 8u && *gpr_i + 1 <= 10) {
                    args[slot++] = c->gpr[*gpr_i + 1];
                    args[slot++] = c->gpr[*gpr_i];
                    *gpr_i += 2;
                }
            } else if (slot < 8u && *gpr_i <= 10) {
                args[slot++] = gw_ppc_arg_word(c->gpr[(*gpr_i)++]);
            }
            break;
        }
    }
    return 1;
}

/* The extended-signature call: see `ext` in gw_ppc_sig. Walks the C parameters in order, taking
 * each from where the PowerPC EABI put it, and lays them out as the i686 cdecl callee reads them.
 * A ninth integer argument is the case the word path could never express - PowerPC has only eight
 * GPR argument registers and spills the rest to the caller's parameter area at r1+8 (r1 is still
 * the caller's frame at the `bl`), while cdecl just keeps going along the stack. */
typedef uint32_t (*gw_ppc_ext_ifn)(uint32_t, uint32_t, uint32_t, uint32_t, uint32_t, uint32_t,
                                   uint32_t, uint32_t, uint32_t, uint32_t, uint32_t, uint32_t,
                                   uint32_t, uint32_t, uint32_t, uint32_t);
typedef float (*gw_ppc_ext_ffn)(uint32_t, uint32_t, uint32_t, uint32_t, uint32_t, uint32_t,
                                uint32_t, uint32_t, uint32_t, uint32_t, uint32_t, uint32_t,
                                uint32_t, uint32_t, uint32_t, uint32_t);
typedef double (*gw_ppc_ext_dfn)(uint32_t, uint32_t, uint32_t, uint32_t, uint32_t, uint32_t,
                                 uint32_t, uint32_t, uint32_t, uint32_t, uint32_t, uint32_t,
                                 uint32_t, uint32_t, uint32_t, uint32_t);

static void gw_ppc_bridge_call_ext(gw_ppc_machine *m, uint32_t guest_addr, gw_ppc_native_fn fn,
                                   const char *ext) {
    gw_ppc_ctx *c = &m->cpu;
    uint32_t w[GW_PPC_EXT_MAX_WORDS];
    uint32_t nw = 0, overflow = c->gpr[1] + 8u;
    int gpr_i = 3, fpr_i = 1;
    const char *p = ext + 2;
    char ret = ext[0];

    memset(w, 0, sizeof w);
    if (ext[0] == '\0' || ext[1] != ':') {
        gw_panic("ppc: malformed extended signature \"%s\" for 0x%08X", ext, guest_addr);
    }
    while (*p != '\0') {
        char cls = *p++;
        uint32_t need = 1, word = 0, size = 0;
        double d = 0.0;
        if (cls == 'd') {
            need = 2;
        } else if (cls == 'a') {
            while (*p >= '0' && *p <= '9') {
                size = size * 10u + (uint32_t) (*p++ - '0');
            }
            need = (size + 3u) / 4u;
        }
        if (nw + need > GW_PPC_EXT_MAX_WORDS) {
            gw_panic("ppc: extended signature \"%s\" for 0x%08X needs more than %u native words",
                     ext, guest_addr, GW_PPC_EXT_MAX_WORDS);
        }
        if (cls == 'f' || cls == 'd') {
            if (fpr_i > 8) {
                /* Past f8 the EABI spills to the parameter area; gen_sigs never emits one. */
                gw_panic("ppc: extended signature \"%s\" for 0x%08X has more than 8 FPR arguments",
                         ext, guest_addr);
            }
            d = c->fpr[fpr_i++].d;
        } else if (gpr_i <= 10) {
            word = c->gpr[gpr_i++];
        } else {
            word = gw_ppc_ld32(m, overflow);
            overflow += 4u;
        }
        switch (cls) {
        case 'i':
            w[nw] = gw_ppc_arg_word(word);
            break;
        case 'f': {
            float f = (float) d;
            memcpy(&w[nw], &f, 4);
            break;
        }
        case 'd':
            memcpy(&w[nw], &d, 8);
            break;
        case 'a':
            if (size != 0u) {
                memcpy(&w[nw],
                       (const void *) (uintptr_t) gw_ppc_resolve_ea(m, word, size),
                       size);
            }
            break;
        default:
            gw_panic("ppc: unknown class '%c' in extended signature \"%s\" for 0x%08X", cls, ext,
                     guest_addr);
        }
        nw += need;
    }
    {
        static int trace_bridge = -1;
        if (trace_bridge < 0) {
            const char *v = getenv("MELEE_PPC_TRACE_BRIDGE");
            trace_bridge = (v != NULL && v[0] == '1');
        }
        if (trace_bridge) {
            gw_log("ppc: bridge %s [%s] from ip=0x%08X (%08X %08X %08X %08X)",
                   gw_ppc_describe(guest_addr), ext, c->pc - 4, w[0], w[1], w[2], w[3]);
        }
    }
    if (ret == 'f') {
        c->fpr[1].d = (double) ((gw_ppc_ext_ffn) (void *) fn)(
            w[0], w[1], w[2], w[3], w[4], w[5], w[6], w[7], w[8], w[9], w[10], w[11], w[12], w[13],
            w[14], w[15]);
    } else if (ret == 'd') {
        c->fpr[1].d = ((gw_ppc_ext_dfn) (void *) fn)(w[0], w[1], w[2], w[3], w[4], w[5], w[6],
                                                     w[7], w[8], w[9], w[10], w[11], w[12], w[13],
                                                     w[14], w[15]);
    } else if (ret == 'i') {
        c->gpr[3] = ((gw_ppc_ext_ifn) (void *) fn)(w[0], w[1], w[2], w[3], w[4], w[5], w[6], w[7],
                                                   w[8], w[9], w[10], w[11], w[12], w[13], w[14],
                                                   w[15]);
    } else {
        gw_panic("ppc: unknown return class '%c' in extended signature \"%s\" for 0x%08X", ret, ext,
                 guest_addr);
    }
}

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
    sig.ext = NULL;
    fn = m->resolve(guest_addr, m->bridge_ctx, &sig);
    if (fn == NULL) {
        gw_panic("ppc: resolver returned NULL for guest address 0x%08X", guest_addr);
    }
    if (sig.ext != NULL) {
        gw_ppc_bridge_call_ext(m, guest_addr, fn, sig.ext);
        return;
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
            args[i] = gw_ppc_arg_word(c->gpr[gpr_i]);
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
    if (variadic && sig.n_args >= 1u && !(fmask & (1u << (sig.n_args - 1u))) &&
        gw_ppc_varargs_from_format(m, args[sig.n_args - 1u], args, sig.n_args, &gpr_i, &fpr_i)) {
        /* laid out in format order; see gw_ppc_varargs_from_format */
    } else if (variadic) {
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
            args[i] = gw_ppc_arg_word(c->gpr[gpr_i]);
            ++gpr_i;
        }
    }
    {   /* MELEE_PPC_TRACE_BRIDGE=1: every guest->native call, with its call site and first args.
         * The last line before a native fault names the function and arguments that faulted. */
        static int trace_bridge = -1;
        if (trace_bridge < 0) {
            const char *v = getenv("MELEE_PPC_TRACE_BRIDGE");
            trace_bridge = (v != NULL && v[0] == '1');
        }
        if (trace_bridge) {
            gw_log("ppc: bridge %s from ip=0x%08X (%08X %08X %08X %08X)",
                   gw_ppc_describe(guest_addr), c->pc - 4, args[0], args[1], args[2], args[3]);
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

/* Every blob currently installed. gw_ppc_set_bridge's own pair is [0, 0) for both the fighter and
 * the stage paths, which register their code here instead, so this is where a fault dump's "which
 * blob was running" actually lives. */
static void gw_ppc_log_code_ranges(void) {
    int i;
    for (i = 0; i < gw_ppc_range_count; ++i) {
        gw_log("ppc:   registered code range [0x%08X,0x%08X)", gw_ppc_range_lo[i],
               gw_ppc_range_hi[i]);
    }
}

/* A DEGENERATE RANGE MATCHES NOTHING. An entry that was never filled in, or was released, is
 * [0,0) - and a plain `a >= lo && a < hi` test on it makes address 0 look like guest code.
 * Address 0 is exactly the LR sentinel gw_ppc_call uses for "return to the native caller", so
 * one stale entry turns every tail call and every blr into a fall-through. It presented as
 * ppc_tail_branch_returns passing or failing by TEST ORDER (these ranges are platform state,
 * which the per-test MEM1 snapshot does not restore), and in the game as
 * "outside blob code range [0x00000000,0x00000000)" on Dedede's double jump. */
static int gw_ppc_in_extra_range(uint32_t a) {
    int i;
    for (i = 0; i < gw_ppc_range_count; ++i) {
        if (gw_ppc_range_lo[i] >= gw_ppc_range_hi[i]) {
            continue;
        }
        if (a >= gw_ppc_range_lo[i] && a < gw_ppc_range_hi[i]) {
            return 1;
        }
    }
    return 0;
}

/* The blob address that called into native code: LR at the moment of the bridge call. Zero when
 * no guest frame is executing, so a native function can tell "a blob asked for this" from "the
 * engine asked for this", and name the exact call site inside the blob. */
uint32_t gw_ppc_guest_lr(void) { return gw_ppc_m.cpu.lr; }

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

/* A branch with LK clear that leaves the blob is a TAIL CALL: the native callee's return goes to
 * OUR caller, not to the word after the branch. Bridging it and then falling through executes
 * whatever function happens to be laid out next - Akaneia's PlWf.dat onLoad ends `b
 * MEX_IndexFighterItem` and the fall-through ran onRespawn with r3 = 0, killing the port on
 * entering a VS match as Wolf. So after the call, return the way a blr would. */
static int gw_ppc_bridge_tail(gw_ppc_machine *m, uint32_t target) {
    gw_ppc_ctx *c = &m->cpu;
    gw_ppc_bridge_call(m, target);
    /* LR == 0 is gw_ppc_call's sentinel for "there is no guest caller". Check it explicitly
     * rather than leaning on the range test: it is the one value that must never be treated as
     * an address, whatever the ranges happen to say. */
    if (c->lr == 0u) {
        return 1;
    }
    if (gw_ppc_in_blob(m, c->lr)) {
        c->pc = c->lr;
        return 0;
    }
    return 1; /* an out-of-blob LR: return to the native caller, r3 already set */
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
    /* The update forms write the computed address back to rA, so rA is never treated as 0 here
     * the way it is in the non-update forms - `lbzu rD, d(0)` is invalid, not an absolute load. */
    case 35: /* lbzu */
        rd = (insn >> 21) & 0x1F;
        ra = (insn >> 16) & 0x1F;
        imm = (int32_t)(int16_t)(insn & 0xFFFF);
        {
            uint32_t ea = c->gpr[ra] + (uint32_t)imm;
            c->gpr[rd] = gw_ppc_ld8(m, ea);
            c->gpr[ra] = ea;
        }
        break;
    case 41: /* lhzu */
        rd = (insn >> 21) & 0x1F;
        ra = (insn >> 16) & 0x1F;
        imm = (int32_t)(int16_t)(insn & 0xFFFF);
        {
            uint32_t ea = c->gpr[ra] + (uint32_t)imm;
            c->gpr[rd] = gw_ppc_ld16(m, ea);
            c->gpr[ra] = ea;
        }
        break;
    case 43: /* lhau */
        rd = (insn >> 21) & 0x1F;
        ra = (insn >> 16) & 0x1F;
        imm = (int32_t)(int16_t)(insn & 0xFFFF);
        {
            uint32_t ea = c->gpr[ra] + (uint32_t)imm;
            c->gpr[rd] = (uint32_t)(int32_t)(int16_t)gw_ppc_ld16(m, ea);
            c->gpr[ra] = ea;
        }
        break;
    case 39: /* stbu */
        rs = (insn >> 21) & 0x1F;
        ra = (insn >> 16) & 0x1F;
        imm = (int32_t)(int16_t)(insn & 0xFFFF);
        {
            uint32_t ea = c->gpr[ra] + (uint32_t)imm;
            gw_ppc_st8(m, ea, (uint8_t)c->gpr[rs]);
            c->gpr[ra] = ea;
        }
        break;
    case 45: /* sthu */
        rs = (insn >> 21) & 0x1F;
        ra = (insn >> 16) & 0x1F;
        imm = (int32_t)(int16_t)(insn & 0xFFFF);
        {
            uint32_t ea = c->gpr[ra] + (uint32_t)imm;
            gw_ppc_st16(m, ea, (uint16_t)c->gpr[rs]);
            c->gpr[ra] = ea;
        }
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
    /* The update forms of the same two loads. lfs/lfd/stfs/stfsu/stfd were here and lfsu, lfdu
     * and stfdu were not - an asymmetry with no reason behind it, so it is closed here with the
     * indexed family rather than left for the next blob to find. */
    case 49: /* lfsu fD, d(rA) */
        rd = (insn >> 21) & 0x1F;
        ra = (insn >> 16) & 0x1F;
        imm = (int32_t)(int16_t)(insn & 0xFFFF);
        {
            uint32_t ea = c->gpr[ra] + (uint32_t)imm;
            uint32_t bits = gw_ppc_ld32(m, ea);
            float f;
            memcpy(&f, &bits, 4);
            c->fpr[rd].d = (double)f;
            c->gpr[ra] = ea;
        }
        break;
    case 51: /* lfdu fD, d(rA) */
        rd = (insn >> 21) & 0x1F;
        ra = (insn >> 16) & 0x1F;
        imm = (int32_t)(int16_t)(insn & 0xFFFF);
        {
            uint32_t ea = c->gpr[ra] + (uint32_t)imm;
            c->fpr[rd].u64 = gw_ppc_ld64(m, ea);
            c->gpr[ra] = ea;
        }
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
    case 53: /* stfsu fS, d(rA) */
        rs = (insn >> 21) & 0x1F;
        ra = (insn >> 16) & 0x1F;
        imm = (int32_t)(int16_t)(insn & 0xFFFF);
        {
            float f = (float)c->fpr[rs].d;
            uint32_t bits;
            uint32_t ea = c->gpr[ra] + (uint32_t)imm;
            memcpy(&bits, &f, 4);
            gw_ppc_st32(m, ea, bits);
            c->gpr[ra] = ea;
        }
        break;
    case 54: /* stfd fS, d(rA) */
        rs = (insn >> 21) & 0x1F;
        ra = (insn >> 16) & 0x1F;
        imm = (int32_t)(int16_t)(insn & 0xFFFF);
        gw_ppc_st64(m, (ra == 0 ? 0 : c->gpr[ra]) + (uint32_t)imm, c->fpr[rs].u64);
        break;
    case 55: /* stfdu fS, d(rA) */
        rs = (insn >> 21) & 0x1F;
        ra = (insn >> 16) & 0x1F;
        imm = (int32_t)(int16_t)(insn & 0xFFFF);
        {
            uint32_t ea = c->gpr[ra] + (uint32_t)imm;
            gw_ppc_st64(m, ea, c->fpr[rs].u64);
            c->gpr[ra] = ea;
        }
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
        } else if (lk) {
            gw_ppc_bridge_call(m, target);
        } else {
            return gw_ppc_bridge_tail(m, target);
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
            } else if (lk) {
                gw_ppc_bridge_call(m, target);
            } else {
                return gw_ppc_bridge_tail(m, target);
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
                } else if (lk) {
                    gw_ppc_bridge_call(m, target);
                } else {
                    /* bctr with LK clear: a tail call through a function pointer. */
                    return gw_ppc_bridge_tail(m, target);
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
    case 75: /* mulhw: the HIGH 32 bits of the signed product */
        c->gpr[rd] =
            (uint32_t)(((int64_t)(int32_t)c->gpr[ra] * (int64_t)(int32_t)c->gpr[rb]) >> 32);
        if (insn & 1) {
            gw_ppc_cr0_cmp(c, c->gpr[rd]);
        }
        break;
    case 11: /* mulhwu: the same, unsigned */
        c->gpr[rd] =
            (uint32_t)(((uint64_t)c->gpr[ra] * (uint64_t)c->gpr[rb]) >> 32);
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

    case 459: /* divwu: unsigned divide; x/0 is undefined on hardware, 0 here like divw */
        c->gpr[rd] = c->gpr[rb] == 0 ? 0 : c->gpr[ra] / c->gpr[rb];
        if (insn & 1) {
            gw_ppc_cr0_cmp(c, c->gpr[rd]);
        }
        break;

    case 60: /* andc */
        c->gpr[ra] = c->gpr[rs] & ~c->gpr[rb];
        if (insn & 1) {
            gw_ppc_cr0_cmp(c, c->gpr[ra]);
        }
        break;
    case 412: /* orc */
        c->gpr[ra] = c->gpr[rs] | ~c->gpr[rb];
        if (insn & 1) {
            gw_ppc_cr0_cmp(c, c->gpr[ra]);
        }
        break;
    case 476: /* nand */
        c->gpr[ra] = ~(c->gpr[rs] & c->gpr[rb]);
        if (insn & 1) {
            gw_ppc_cr0_cmp(c, c->gpr[ra]);
        }
        break;
    case 284: /* eqv */
        c->gpr[ra] = ~(c->gpr[rs] ^ c->gpr[rb]);
        if (insn & 1) {
            gw_ppc_cr0_cmp(c, c->gpr[ra]);
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
    /* The indexed byte/halfword forms, found by the moveset sweep: nine stage and fighter blobs
     * across both discs died on stbx alone. They are ordinary loads and stores - the interpreter
     * simply never needed them until custom stages started running. */
    case 87: /* lbzx */
        c->gpr[rd] = gw_ppc_ld8(m, (ra == 0 ? 0 : c->gpr[ra]) + c->gpr[rb]);
        break;
    case 215: /* stbx */
        gw_ppc_st8(m, (ra == 0 ? 0 : c->gpr[ra]) + c->gpr[rb], (uint8_t)c->gpr[rs]);
        break;
    case 279: /* lhzx */
        c->gpr[rd] = gw_ppc_ld16(m, (ra == 0 ? 0 : c->gpr[ra]) + c->gpr[rb]);
        break;
    case 407: /* sthx */
        gw_ppc_st16(m, (ra == 0 ? 0 : c->gpr[ra]) + c->gpr[rb], (uint16_t)c->gpr[rs]);
        break;
    case 343: /* lhax: like lhzx but SIGN-extended */
        c->gpr[rd] =
            (uint32_t)(int32_t)(int16_t)gw_ppc_ld16(m, (ra == 0 ? 0 : c->gpr[ra]) + c->gpr[rb]);
        break;
    case 151: /* stwx */
        gw_ppc_st32(m, (ra == 0 ? 0 : c->gpr[ra]) + c->gpr[rb], c->gpr[rs]);
        break;
    case 790: /* lhbrx: load halfword byte-reversed (i.e. little-endian) */
        c->gpr[rd] = gw_bswap16(gw_ppc_ld16(m, (ra == 0 ? 0 : c->gpr[ra]) + c->gpr[rb]));
        break;
    /* The byte-reversed family. Byte-reversing a big-endian access is the same as the ordinary
     * accessor followed by a host swap, so these go through gw_ppc_ld/st like everything else and
     * inherit its address translation. XO 918 used to be labelled stwbrx and stored four bytes;
     * 918 is sthbrx (two bytes) and stwbrx is 662, so a halfword store clobbered the next two
     * bytes. */
    case 534: /* lwbrx */
        c->gpr[rd] = gw_bswap32(gw_ppc_ld32(m, (ra == 0 ? 0 : c->gpr[ra]) + c->gpr[rb]));
        break;
    case 662: /* stwbrx */
        gw_ppc_st32(m, (ra == 0 ? 0 : c->gpr[ra]) + c->gpr[rb], gw_bswap32(c->gpr[rs]));
        break;
    case 918: /* sthbrx */
        gw_ppc_st16(m, (ra == 0 ? 0 : c->gpr[ra]) + c->gpr[rb],
                    gw_bswap16((uint16_t)c->gpr[rs]));
        break;

    /* Indexed-with-update integer loads and stores. ext:306 died on `stwux r10, r1, r9` - the
     * aligned-stack-frame prologue a compiler emits for a function with an over-aligned local
     * (r9 = -(frame size) rounded to the alignment). Treating it as a no-op left r1 unmoved, so
     * the callee's frame overlapped its caller's. As with the d-form update loads, rA is the base
     * and is written back; rA == 0 is an invalid form, not an absolute address. */
    case 55: /* lwzux */
    {
        uint32_t ea = c->gpr[ra] + c->gpr[rb];
        c->gpr[rd] = gw_ppc_ld32(m, ea);
        c->gpr[ra] = ea;
        break;
    }
    case 119: /* lbzux */
    {
        uint32_t ea = c->gpr[ra] + c->gpr[rb];
        c->gpr[rd] = gw_ppc_ld8(m, ea);
        c->gpr[ra] = ea;
        break;
    }
    case 311: /* lhzux */
    {
        uint32_t ea = c->gpr[ra] + c->gpr[rb];
        c->gpr[rd] = gw_ppc_ld16(m, ea);
        c->gpr[ra] = ea;
        break;
    }
    case 375: /* lhaux */
    {
        uint32_t ea = c->gpr[ra] + c->gpr[rb];
        c->gpr[rd] = (uint32_t)(int32_t)(int16_t)gw_ppc_ld16(m, ea);
        c->gpr[ra] = ea;
        break;
    }
    case 183: /* stwux */
    {
        uint32_t ea = c->gpr[ra] + c->gpr[rb];
        gw_ppc_st32(m, ea, c->gpr[rs]);
        c->gpr[ra] = ea;
        break;
    }
    case 247: /* stbux */
    {
        uint32_t ea = c->gpr[ra] + c->gpr[rb];
        gw_ppc_st8(m, ea, (uint8_t)c->gpr[rs]);
        c->gpr[ra] = ea;
        break;
    }
    case 439: /* sthux */
    {
        uint32_t ea = c->gpr[ra] + c->gpr[rb];
        gw_ppc_st16(m, ea, (uint16_t)c->gpr[rs]);
        c->gpr[ra] = ea;
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
    case 10: /* addc rD, rA, rB: CA = carry out */
    {
        uint64_t sum = (uint64_t)c->gpr[ra] + c->gpr[rb];
        c->gpr[rd] = (uint32_t)sum;
        gw_ppc_xer_set_ca(c, sum >= 0x100000000u);
        if (insn & 1) {
            gw_ppc_cr0_cmp(c, c->gpr[rd]);
        }
        break;
    }
    case 138: /* adde rD, rA, rB: rD = rA + rB + CA */
    {
        uint64_t sum = (uint64_t)c->gpr[ra] + c->gpr[rb] + gw_ppc_xer_ca(c);
        c->gpr[rd] = (uint32_t)sum;
        gw_ppc_xer_set_ca(c, sum >= 0x100000000u);
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

    /* ---- the X-form floating load/store family ----------------------------------------
     * ACE's external stage 357 died on `unimplemented opcode word=0x7D87FC2E`, which is
     * lfsx f12, r7, r31 (primary 31, extended 535). stfiwx below was the only member of this
     * family the interpreter had, because it is the one the float-to-int idiom needs and that
     * is the only one content had used. The other eight are the same two address modes
     * (indexed, indexed-with-update) over the same two widths (single, double) as the d-form
     * lfs/lfd/stfs/stfd already here, so all of them go in at once rather than waiting for
     * eight more stages to each find their own.
     *
     * The update forms write the effective address back to rA. rA == 0 is an invalid form for
     * them (there is no r0 base), so, unlike the non-update forms, they do not special-case it.
     * A single-precision load widens to the double the FPR model stores; a single store
     * narrows. That is the same conversion the d-form cases do, and it is what `frsp`-free
     * content depends on. */
    case 535: /* lfsx fD, rA, rB */
    {
        uint32_t bits = gw_ppc_ld32(m, (ra == 0 ? 0 : c->gpr[ra]) + c->gpr[rb]);
        float f;
        memcpy(&f, &bits, 4);
        c->fpr[rd].d = (double) f;
        break;
    }
    case 567: /* lfsux fD, rA, rB */
    {
        uint32_t ea = c->gpr[ra] + c->gpr[rb];
        uint32_t bits = gw_ppc_ld32(m, ea);
        float f;
        memcpy(&f, &bits, 4);
        c->fpr[rd].d = (double) f;
        c->gpr[ra] = ea;
        break;
    }
    case 599: /* lfdx fD, rA, rB */
        c->fpr[rd].u64 = gw_ppc_ld64(m, (ra == 0 ? 0 : c->gpr[ra]) + c->gpr[rb]);
        break;
    case 631: /* lfdux fD, rA, rB */
    {
        uint32_t ea = c->gpr[ra] + c->gpr[rb];
        c->fpr[rd].u64 = gw_ppc_ld64(m, ea);
        c->gpr[ra] = ea;
        break;
    }
    case 663: /* stfsx fS, rA, rB */
    {
        float f = (float) c->fpr[rs].d;
        uint32_t bits;
        memcpy(&bits, &f, 4);
        gw_ppc_st32(m, (ra == 0 ? 0 : c->gpr[ra]) + c->gpr[rb], bits);
        break;
    }
    case 695: /* stfsux fS, rA, rB */
    {
        uint32_t ea = c->gpr[ra] + c->gpr[rb];
        float f = (float) c->fpr[rs].d;
        uint32_t bits;
        memcpy(&bits, &f, 4);
        gw_ppc_st32(m, ea, bits);
        c->gpr[ra] = ea;
        break;
    }
    case 727: /* stfdx fS, rA, rB */
        gw_ppc_st64(m, (ra == 0 ? 0 : c->gpr[ra]) + c->gpr[rb], c->fpr[rs].u64);
        break;
    case 759: /* stfdux fS, rA, rB */
    {
        uint32_t ea = c->gpr[ra] + c->gpr[rb];
        gw_ppc_st64(m, ea, c->fpr[rs].u64);
        c->gpr[ra] = ea;
        break;
    }
    case 983: /* stfiwx fS, rA, rB: store the low 32 bits of fS as a word */
        /* u32[0], not u32[1]. On big-endian PowerPC the LOW half of a 64-bit FPR is word 1;
         * on this little-endian host it is word 0, and gw_ppc_fpr is a host-order union. The
         * pair `fctiwz fD,fS` + `stfiwx fD,rA,rB` is how every PowerPC compiler spells
         * (int)some_float, so reading the wrong half stored a constant 0 for any value whose
         * double bit pattern has a zero low word - which is every integer-valued float a
         * timer is ever set from. GrGh (ext:302) re-arms its background-model spawn timer
         * this way from a stage parameter of 900.0; the timer read back 0 every frame, so the
         * stage spawned a model on ~90% of frames instead of one per 900, and exhausted the
         * HSD heap at frame ~270. */
        gw_ppc_st32(m, (ra == 0 ? 0 : c->gpr[ra]) + c->gpr[rb], c->fpr[rs].u32[0]);
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

/* ---- host-stack arguments to interpreted code -------------------------------------------
 *
 * A native engine call site may hand a callback a pointer to one of ITS OWN STACK LOCALS as an
 * out-parameter. ftcoll.c does it twice:
 *
 *     ft_80459A8C[i].active_cb(ground, gobj, (Vec3*) &desc)   ftColl_8007BAC0
 *     ft_80459A68[i].active_cb(grp, fgp, &wind)               ftColl_GetWindOffsetVec
 *
 * On a GameCube that is unremarkable. Here the callback may be an m-ex stage's PPC blob running
 * on this interpreter, and every interpreted load and store is bounds-checked against guest
 * MEM1 - which the host stack is not part of. So the callback's `stw r9,0(r30)` dies as
 *
 *     ppc: guest access violation at ip=0x81155088 ea=0x001AFDA4
 *
 * and takes the stage down with it.
 *
 * THE MECHANISM LIVES HERE, not at the call sites, because gw_ppc_call is the one funnel every
 * native->guest entry in the port passes through: the thunk pool, the execute trap's
 * trampoline, the Arch_FighterFunc override slots, MoveLogic's four per-state callbacks, the
 * fighter callback fields and the stage callbacks all end up in this function. Patching the two
 * call sites we know about would leave the third to be rediscovered from a crash log. Any
 * argument that points into this thread's host stack, in a frame ABOVE our own, is mirrored
 * into a scratch block carved off the guest stack; the guest is handed that guest address, and
 * the bytes are copied back when it returns.
 *
 * BYTE ORDER: the copy is RAW in both directions, and that is deliberate, not an oversight.
 * gwtool byte-swaps EVERY memory access in a game TU, including accesses to that TU's own stack
 * locals, so a game-compiled function's locals are big-endian exactly like the heap is. From
 * llvm-objdump of this tree's own object for the wind site above:
 *
 *     6a36: 8b 44 24 08   movl 0x8(%esp), %eax     ; wind.z - a plain stack local
 *     6a3a: 0f c8         bswapl %eax              ; ...read big-endian
 *
 * The interpreter already stores big-endian into the scratch, so a raw copy back is exactly
 * what the caller then reads correctly. Swapping here would hand ftColl_8007BAC0 a byte-
 * reversed DynamicsDesc pointer, which is the same crash one indirection later.
 *
 * WINDOW: the call site knows the object's size and we do not, so a fixed window is mirrored.
 * That is safe precisely because it is a mirror - bytes the guest never writes are copied back
 * unchanged, and no frame above us can move while we are running inside it. The window is
 * clamped to the top of the stack so the copy cannot run off the end.
 *
 * FALSE POSITIVES: an integer argument that is not a pointer but happens to land in the live
 * stack range and be 4-byte aligned would be rewritten. Host stack addresses here are ~1.7 MB,
 * a magnitude engine callbacks do not pass as counts or flags, and every distinct one is logged
 * the first time, so a mistake shows up as a log line rather than as silence.
 */
#define GW_PPC_HOST_ARG_WINDOW 64u  /* bytes mirrored per host-stack argument */
#define GW_PPC_HOST_ARG_MAX 8       /* at most one per integer argument */
#define GW_PPC_HOST_ARG_GUARD 0x40u /* keep the guest's own r1 clear of the scratch */

typedef struct {
    int n;
    uint32_t host[GW_PPC_HOST_ARG_MAX];  /* the host address the caller passed */
    uint32_t guest[GW_PPC_HOST_ARG_MAX]; /* the guest scratch it was replaced with */
    uint32_t size[GW_PPC_HOST_ARG_MAX];
} gw_ppc_host_args;

/* Declared by hand rather than pulling <windows.h> into the interpreter: this file is otherwise
 * free of Windows headers and their macros. kernel32 is already linked. */
extern __declspec(dllimport) void __stdcall GetCurrentThreadStackLimits(uintptr_t *low,
                                                                        uintptr_t *high);

/* Mirror every host-stack argument into guest scratch. Returns the guest stack pointer the run
 * should use (lowered past the scratch when anything was mirrored). */
static uint32_t gw_ppc_host_args_in(gw_ppc_host_args *hb, uint32_t guest_fn, uint32_t *a, int n,
                                    uint32_t sp, uint32_t own_frame) {
    static int logged;
    uintptr_t slo_p = 0, shi_p = 0;
    uint32_t slo, shi, top;
    int i, j;

    hb->n = 0;
    if (n <= 0) {
        return sp;
    }
    /* The scratch comes off the guest stack, so there has to be a sane one. */
    if (sp < GW_PPC_MEM1_BASE + 0x1000u ||
        (uint64_t) sp > (uint64_t) GW_PPC_MEM1_BASE + (uint64_t) gw_mem1_size) {
        return sp;
    }
    /* AND it has to be PLAIN MEM1. gw_ppc_resolve_ea sends any guest address that falls inside
     * a game global's extent to that global's NATIVE storage instead (gw_ppc_static_native), so
     * a scratch carved out of such a range is written by the guest somewhere other than where
     * the copy-back reads it from - which comes back as an out-parameter full of zeros, not as
     * an error. The statics occupy 0x803B7280..0x804DEA9C and the m-ex guest stack lives at the
     * top of MEM1, far above them, so this rejects only a misconfigured stack; it says so once
     * rather than quietly handing the caller nothing. */
    {
        uint32_t need = (uint32_t) GW_PPC_HOST_ARG_MAX *
                        ((GW_PPC_HOST_ARG_WINDOW + 15u) & ~15u);
        if (sp < GW_PPC_MEM1_BASE + need || !gw_ppc_ea_ok(sp - need, need) ||
            gw_ppc_static_native(sp - need) != 0u || gw_ppc_static_native(sp - 4u) != 0u) {
            static int moaned;
            if (!moaned) {
                moaned = 1;
                gw_log("ppc: guest stack 0x%08X cannot hold the host-argument mirror (it is not "
                       "plain MEM1) - a host-stack out-parameter to guest code will fault",
                       sp);
            }
            return sp;
        }
    }
    GetCurrentThreadStackLimits(&slo_p, &shi_p);
    slo = (uint32_t) slo_p;
    shi = (uint32_t) shi_p;
    if (shi <= slo) {
        return sp;
    }
    /* Only frames ABOVE this one: our own frame and everything below it is scratch that no
     * caller can be pointing into, and rewriting an argument that names it would be wrong. */
    if (own_frame > slo) {
        slo = own_frame;
    }

    top = sp;
    for (i = 0; i < n && i < GW_PPC_HOST_ARG_MAX; ++i) {
        uint32_t v = a[i], sz;
        if (v < slo || v >= shi || (v & 3u) != 0u) {
            continue;
        }
        for (j = 0; j < hb->n; ++j) {
            if (hb->host[j] == v) {
                break; /* the same local passed twice: one mirror, one copy-back */
            }
        }
        if (j < hb->n) {
            a[i] = hb->guest[j];
            continue;
        }
        sz = GW_PPC_HOST_ARG_WINDOW;
        if (shi - v < sz) {
            sz = shi - v;
        }
        top -= (sz + 15u) & ~15u;
        memcpy((void *) (uintptr_t) top, (const void *) (uintptr_t) v, sz);
        hb->host[hb->n] = v;
        hb->guest[hb->n] = top;
        hb->size[hb->n] = sz;
        ++hb->n;
        a[i] = top;
        if (logged < 16) {
            ++logged;
            gw_log("ppc: host-stack arg %d (0x%08X) to guest %s mirrored into guest 0x%08X "
                   "(%u bytes, copied back on return)",
                   i, v, gw_ppc_describe(guest_fn), top, sz);
        }
    }
    if (hb->n == 0) {
        return sp;
    }
    /* The guest's prologue writes the back chain at r1+0 and the saved LR at r1+4, so leave the
     * PowerPC linkage area between its stack pointer and the scratch above it. */
    return top - GW_PPC_HOST_ARG_GUARD;
}

static void gw_ppc_host_args_out(const gw_ppc_host_args *hb) {
    int i;
    for (i = 0; i < hb->n; ++i) {
        memcpy((void *) (uintptr_t) hb->host[i], (const void *) (uintptr_t) hb->guest[i],
               hb->size[i]);
    }
}

uint32_t gw_ppc_call(uint32_t guest_fn, const uint32_t *gpr_args, int nargs, uint32_t rtoc,
                     uint32_t sp) {
    gw_ppc_machine saved = gw_ppc_m; /* full reentrant save (cpu + bridge) */
    gw_ppc_ctx *c = &gw_ppc_m.cpu;
    gw_ppc_host_args hb;
    uint32_t a[8];
    uint32_t r3;
    int i, na;

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

    /* Copy the arguments out before anything can rewrite one: an argument that points into the
     * host stack is unusable by interpreted code and gets mirrored into guest memory. */
    na = 0;
    if (gpr_args != NULL) {
        for (i = 0; i < nargs && i < 8; ++i) {
            a[i] = gpr_args[i];
        }
        na = nargs < 8 ? nargs : 8;
    }
    sp = gw_ppc_host_args_in(&hb, guest_fn, a, na, sp, (uint32_t) (uintptr_t) &saved);

    memset(c, 0, sizeof *c);
    for (i = 0; i < 32; ++i) {
        c->fpr[i].u64 = 0;
    }
    c->gpr[1] = sp;
    c->gpr[2] = rtoc;
    c->lr = 0; /* sentinel: a blr to this returns to the native caller */
    c->pc = guest_fn;
    for (i = 0; i < na; ++i) {
        c->gpr[3 + i] = a[i];
    }

    r3 = gw_ppc_run(&gw_ppc_m);
    gw_ppc_host_args_out(&hb);
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
/* A guest stack that really is MEM1: the game's statics span 0x803B7280..0x804DEA9C, so
 * GW_PPC_TEST_STACK above is inside one of them. Only tests that actually use the stack care. */
#define GW_PPC_TEST_STACK_MEM1 0x80380000u
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

/* ---- extended-signature bridge tests ------------------------------------------------------
 * The two things the word-slot path cannot express, each end to end through the interpreter:
 *   1. a ninth and tenth integer argument, which PowerPC spills to the caller's parameter area
 *      at r1+8 while cdecl just keeps going along the stack (mpCheckFloor, GXSetTevIndirect);
 *   2. by-value aggregates (PowerPC passes a pointer to a copy, cdecl the bytes inline), a double
 *      argument (two native words) and a double return (x87 -> f1).
 * Each helper weights every argument by its position, so a slot off by one word fails loudly. */

#define GW_PPC_TEST_XCODE_A 0x80300600u      /* blob: aggregates + doubles */
#define GW_PPC_TEST_XCODE_S 0x80300640u      /* blob: ten ints, two on the stack */
#define GW_PPC_TEST_XDATA 0x80300700u        /* colour, Vec3, double, float, results */
#define GW_PPC_TEST_XAGG_GUEST 0x80380380u   /* fake guest address of the aggregate helper */
#define GW_PPC_TEST_XSTACK_GUEST 0x80380384u /* fake guest address of the ten-int helper */
#define GW_PPC_TEST_BL(pc, target) (0x48000001u | (((target) - (pc)) & 0x03FFFFFCu))

typedef struct { uint8_t r, g, b, a; } gw_ppc_test_rgba;
typedef struct { float x, y, z; } gw_ppc_test_vec3;

static uint32_t gw_ppc_test_xstack(uint32_t a1, uint32_t a2, uint32_t a3, uint32_t a4, uint32_t a5,
                                   uint32_t a6, uint32_t a7, uint32_t a8, uint32_t a9,
                                   uint32_t a10) {
    return a1 * 1u + a2 * 2u + a3 * 3u + a4 * 4u + a5 * 5u + a6 * 6u + a7 * 7u + a8 * 8u +
           a9 * 9u + a10 * 10u;
}

/* The inline copy is the guest's raw big-endian bytes - what a gwtool-retargeted callee expects,
 * because its loads byte-swap - so the float fields are read the way that callee reads them. */
static double gw_ppc_test_xagg(uint32_t tag, gw_ppc_test_rgba c, gw_ppc_test_vec3 v, double d,
                               float f) {
    double x = gw_rf32(&v.x), y = gw_rf32(&v.y), z = gw_rf32(&v.z);
    return (double) tag + c.r + 2.0 * c.g + 3.0 * c.b + 4.0 * c.a + x + 10.0 * y + 100.0 * z +
           1000.0 * d + 10000.0 * f;
}

static gw_ppc_native_fn gw_ppc_test_xresolve(uint32_t guest_addr, void *ctx, gw_ppc_sig *sig) {
    (void) ctx;
    if (guest_addr == GW_PPC_TEST_XAGG_GUEST) {
        sig->ext = "d:ia4a12df";
        return (gw_ppc_native_fn) (void *) gw_ppc_test_xagg;
    }
    if (guest_addr == GW_PPC_TEST_XSTACK_GUEST) {
        sig->ext = "i:iiiiiiiiii";
        return (gw_ppc_native_fn) (void *) gw_ppc_test_xstack;
    }
    return NULL;
}

static int test_ppc_ext_stack_args(void) {
    static uint32_t blob[] = {
        0x9421FFE0u, /* stwu r1, -32(r1) */
        0x7C0802A6u, /* mflr r0 */
        0x90010024u, /* stw r0, 36(r1) */
        0x38000009u, /* li r0, 9 */
        0x90010008u, /* stw r0, 8(r1)   -- ninth argument, parameter area */
        0x3800000Au, /* li r0, 10 */
        0x9001000Cu, /* stw r0, 12(r1)  -- tenth */
        0x38600001u, 0x38800002u, 0x38A00003u, 0x38C00004u, /* li r3..r6, 1..4 */
        0x38E00005u, 0x39000006u, 0x39200007u, 0x39400008u, /* li r7..r10, 5..8 */
        0u,          /* bl helper (patched below) */
        0x3D208030u, /* lis r9, 0x8030 */
        0x90690780u, /* stw r3, 0x780(r9) */
        0x80010024u, /* lwz r0, 36(r1) */
        0x7C0803A6u, /* mtlr r0 */
        0x38210020u, /* addi r1, r1, 32 */
        0x4E800020u, /* blr */
    };
    const uint32_t expected = 1 + 4 + 9 + 16 + 25 + 36 + 49 + 64 + 81 + 100; /* 385 */
    unsigned i;
    uint32_t got;

    blob[15] = GW_PPC_TEST_BL(GW_PPC_TEST_XCODE_S + 15u * 4u, GW_PPC_TEST_XSTACK_GUEST);
    for (i = 0; i < sizeof blob / sizeof blob[0]; ++i) {
        gw_w32((void *) (uintptr_t) (GW_PPC_TEST_XCODE_S + 4 * i), blob[i]);
    }
    gw_w32((void *) (uintptr_t) (GW_PPC_TEST_XDATA + 0x80), 0u);
    gw_ppc_set_bridge(gw_ppc_test_xresolve, NULL, GW_PPC_TEST_XCODE_S,
                      GW_PPC_TEST_XCODE_S + (uint32_t) sizeof blob);
    gw_ppc_call(GW_PPC_TEST_XCODE_S, NULL, 0, 0 /* rtoc */, GW_PPC_TEST_STACK);
    got = gw_r32((const void *) (uintptr_t) (GW_PPC_TEST_XDATA + 0x80));
    if (got != expected) {
        gw_test_fail("ten-int bridge returned %u, expected %u (stack-spilled args misplaced)", got,
                     expected);
        return 1;
    }
    return 0;
}

static int test_ppc_ext_aggregate_double(void) {
    static uint32_t blob[] = {
        0x7C0802A6u, /* mflr r0 */
        0x3D208030u, /* lis r9, 0x8030 */
        0x38890700u, /* addi r4, r9, 0x700  -- &colour */
        0x38A90704u, /* addi r5, r9, 0x704  -- &Vec3 */
        0x38600007u, /* li r3, 7 */
        0xC8290710u, /* lfd f1, 0x710(r9)   -- double */
        0xC0490718u, /* lfs f2, 0x718(r9)   -- float */
        0u,          /* bl helper (patched below) */
        0x3D208030u, /* lis r9, 0x8030 */
        0xD8290720u, /* stfd f1, 0x720(r9) */
        0x7C0803A6u, /* mtlr r0 */
        0x4E800020u, /* blr */
    };
    const double d = 0.125, expected_d = 7.0 + 1 + 2.0 * 2 + 3.0 * 3 + 4.0 * 4 + 0.5 + 10.0 * 1.5 +
                                         100.0 * 2.5 + 1000.0 * d + 10000.0 * 0.25;
    uint64_t bits;
    double got;
    unsigned i;

    blob[7] = GW_PPC_TEST_BL(GW_PPC_TEST_XCODE_A + 7u * 4u, GW_PPC_TEST_XAGG_GUEST);
    for (i = 0; i < sizeof blob / sizeof blob[0]; ++i) {
        gw_w32((void *) (uintptr_t) (GW_PPC_TEST_XCODE_A + 4 * i), blob[i]);
    }
    gw_w8((void *) (uintptr_t) (GW_PPC_TEST_XDATA + 0), 1); /* GXColor r g b a */
    gw_w8((void *) (uintptr_t) (GW_PPC_TEST_XDATA + 1), 2);
    gw_w8((void *) (uintptr_t) (GW_PPC_TEST_XDATA + 2), 3);
    gw_w8((void *) (uintptr_t) (GW_PPC_TEST_XDATA + 3), 4);
    gw_wf32((void *) (uintptr_t) (GW_PPC_TEST_XDATA + 4), 0.5f); /* Vec3 */
    gw_wf32((void *) (uintptr_t) (GW_PPC_TEST_XDATA + 8), 1.5f);
    gw_wf32((void *) (uintptr_t) (GW_PPC_TEST_XDATA + 12), 2.5f);
    memcpy(&bits, &d, 8);
    gw_w64((void *) (uintptr_t) (GW_PPC_TEST_XDATA + 0x10), bits);
    gw_wf32((void *) (uintptr_t) (GW_PPC_TEST_XDATA + 0x18), 0.25f);
    gw_w64((void *) (uintptr_t) (GW_PPC_TEST_XDATA + 0x20), 0u);
    gw_ppc_set_bridge(gw_ppc_test_xresolve, NULL, GW_PPC_TEST_XCODE_A,
                      GW_PPC_TEST_XCODE_A + (uint32_t) sizeof blob);
    gw_ppc_call(GW_PPC_TEST_XCODE_A, NULL, 0, 0 /* rtoc */, GW_PPC_TEST_STACK);
    bits = gw_r64((const void *) (uintptr_t) (GW_PPC_TEST_XDATA + 0x20));
    memcpy(&got, &bits, 8);
    if (got != expected_d) {
        gw_test_fail("aggregate/double bridge returned %.9f, expected %.9f", got, expected_d);
        return 1;
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

#define GW_PPC_TEST_VCODE_S 0x80300280u       /* blob: printf-style, word then double */
#define GW_PPC_TEST_FMT_GUEST 0x8038036Cu     /* fake guest address of the format helper */
static char gw_ppc_test_fmt_out[64];

static uint32_t gw_ppc_test_fmthelper(const char *fmt, ...) {
    va_list ap;
    va_start(ap, fmt);
    vsnprintf(gw_ppc_test_fmt_out, sizeof gw_ppc_test_fmt_out, fmt, ap);
    va_end(ap);
    return (uint32_t)strlen(gw_ppc_test_fmt_out);
}

static gw_ppc_native_fn gw_ppc_test_vresolve(uint32_t guest_addr, void *ctx, gw_ppc_sig *sig) {
    (void)ctx;
    if (guest_addr == GW_PPC_TEST_FMT_GUEST) {
        sig->float_args = GW_PPC_SIG_VARARGS;
        sig->n_args = 1; /* fmt */
        sig->ret_float = 0;
        return (gw_ppc_native_fn)(uintptr_t)gw_ppc_test_fmthelper;
    }
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

/* A word BEFORE a double: `OSReport("%s in %.2fms", name, ms)`, the call GrGc.dat (ext:306) makes.
 * The CR6 rule alone put the double first and handed its bits to %s. */
static int test_ppc_varargs_format_order(void) {
    /* mflr r0; lis r3,0x8030; ori r3,r3,0x300; lis r4,0x8030; ori r4,r4,0x310;
     * lis r9,0x8030; lfs f1,0x150(r9); creqv 6,6,6; bl helper; mtlr r0; blr */
    static const uint32_t blob[] = {
        0x7C0802A6u, 0x3C608030u, 0x60630300u, 0x3C808030u, 0x60840310u, 0x3D208030u,
        0xC0290150u, 0x4CC63242u,
        0x48000000u | ((GW_PPC_TEST_FMT_GUEST - (GW_PPC_TEST_VCODE_S + 32)) & 0x03FFFFFCu) | 1u,
        0x7C0803A6u, 0x4E800020u,
    };
    static const char fmt[] = "%s in %.2fms";
    static const char name[] = "abc";
    unsigned i;

    for (i = 0; i < sizeof blob / sizeof blob[0]; ++i) {
        gw_w32((void *)(uintptr_t)(GW_PPC_TEST_VCODE_S + 4 * i), blob[i]);
    }
    memcpy((void *)(uintptr_t)0x80300300u, fmt, sizeof fmt);
    memcpy((void *)(uintptr_t)0x80300310u, name, sizeof name);
    gw_wf32((void *)(uintptr_t)0x80300150u, 2.5f);
    gw_ppc_test_fmt_out[0] = 0;

    gw_ppc_set_bridge(gw_ppc_test_vresolve, NULL, GW_PPC_TEST_VCODE_S,
                      GW_PPC_TEST_VCODE_S + (uint32_t)sizeof blob);
    gw_ppc_call(GW_PPC_TEST_VCODE_S, NULL, 0, 0, GW_PPC_TEST_STACK);
    if (strcmp(gw_ppc_test_fmt_out, "abc in 2.50ms") != 0) {
        gw_test_fail("format-ordered varargs printed \"%s\", expected \"abc in 2.50ms\"",
                     gw_ppc_test_fmt_out);
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

/* ---- native-pointer test -------------------------------------------------------------------
 * The other direction of the static bridge. `ppc_static_bridge` proves that a GUEST address
 * naming a game static reaches its native storage. This proves the half that was missing: an
 * address that is ALREADY native, because a bridged engine function RETURNED a pointer to a
 * global. grDatFiles_801C6330 ends `return &grDatFiles_8049EE10[i]`, and nothing translates a
 * returned pointer - the guest simply got a host address and dereferenced it, which used to
 * fault (`ea=0x107819A0`, the Gamecube stage blob).
 *
 * The offset is deliberately INTERIOR and not a symbol base: `&global[i]` is interior for every
 * i != 0, so a reverse lookup keyed on the base would pass this test only by accident at i == 0.
 *
 * The address is put in a register with lis/ori, exactly as guest code would hold a pointer it
 * was handed, so nothing about it says "static" at the point of use - only its VALUE does. */

#define GW_PPC_TEST_NCODE 0x80300280u /* guest code base */
#define GW_PPC_TEST_NOFF 0x2Cu        /* interior offset into ftData_803C52A0 (size 0x1C0) */

static int test_ppc_native_pointer(void) {
    int kind = 0;
    uint32_t native = gw_mex_bridge_lookup(GW_PPC_TEST_STATIC_GUEST, &kind);
    uint32_t at, expected, got;
    uint32_t blob[4];
    unsigned i;

    if (native == 0 || kind != 0) {
        gw_test_fail("bridge lookup of ftData_803C52A0 (0x803C52A0) failed: native=0x%08X kind=%d",
                     native, kind);
        return 1;
    }
    at = native + GW_PPC_TEST_NOFF;

    /* The range query must accept the whole interior access and reject one past the object. */
    if (!gw_mex_bridge_is_native_data(at, 4)) {
        gw_test_fail("native 0x%08X (ftData_803C52A0+0x%X) not recognised as a game global", at,
                     GW_PPC_TEST_NOFF);
        return 1;
    }
    /* One past ftData_803C52A0 is NOT a useful negative: the runs are merged, and the next
     * global starts exactly where this one ends, so the address is legitimately inside the same
     * run. (That is what the merge is for - .data is contiguous.) The bounds that can be asserted
     * without pinning a link-order detail are the two that matter: the exe's image base, which is
     * PE headers rather than game data, and a guest address, which must never be mistaken for a
     * native one - the two windows are 0x8xxxxxxx and 0x1xxxxxxx and must not overlap. */
    if (gw_mex_bridge_is_native_data(0x10000000u, 4)) {
        gw_test_fail("the exe's image base was accepted as game-global storage");
        return 1;
    }
    if (gw_mex_bridge_is_native_data(GW_PPC_TEST_STACK, 4)) {
        gw_test_fail("guest address 0x%08X was accepted as native game-global storage",
                     GW_PPC_TEST_STACK);
        return 1;
    }

    /* lis r9,hi ; ori r9,r9,lo ; lwz r3,0(r9) ; blr */
    blob[0] = 0x3D200000u | (at >> 16);
    blob[1] = 0x61290000u | (at & 0xFFFFu);
    blob[2] = 0x80690000u;
    blob[3] = 0x4E800020u;
    for (i = 0; i < 4; ++i) {
        gw_w32((void *)(uintptr_t)(GW_PPC_TEST_NCODE + 4 * i), blob[i]);
    }
    gw_ppc_set_bridge(NULL, NULL, GW_PPC_TEST_NCODE, GW_PPC_TEST_NCODE + (uint32_t)sizeof blob);

    expected = gw_r32((const void *)(uintptr_t)at);
    got = gw_ppc_call(GW_PPC_TEST_NCODE, NULL, 0, 0, GW_PPC_TEST_STACK);
    if (got != expected) {
        gw_test_fail("load through a native pointer returned 0x%08X, expected 0x%08X", got,
                     expected);
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

/* ---- the X-form floating load/store family ----------------------------------------------
 * ACE stage 357 needed lfsx and the interpreter had only stfiwx of that family. The blob below
 * walks all four widths/modes: lfsx and lfdx read, stfsx and stfdx write back, and the *ux
 * update forms are checked through the base register they are required to advance. A missing
 * member of this family panics, so reaching the end at all is most of the assertion; the value
 * and base-register checks catch a member that is present but decoded to the wrong width.
 *
 * r5 = scratch base; r6/r7/r8/r9/r10 = 0/8/16/24/32. The *ux forms advance r5 by a NON-ZERO
 * amount each time, so a member that ignores its base write-back lands its store in the wrong
 * place and the last two assertions fail.
 */
static int test_ppc_fp_indexed(void) {
    static const uint32_t blob[] = {
        0x7C25342Eu, /* lfsx   f1, r5, r6   ; f1 = single at +0             */
        0x7C453CAEu, /* lfdx   f2, r5, r7   ; f2 = double at +8             */
        0x7C45452Eu, /* stfsx  f2, r5, r8   ; single at +16 = f2            */
        0x7C254DAEu, /* stfdx  f1, r5, r9   ; double at +24 = f1            */
        0x7C65546Eu, /* lfsux  f3, r5, r10  ; r5 += 32, f3 = single at +32  */
        0x7C653D6Eu, /* stfsux f3, r5, r7   ; r5 += 8,  single at +40 = f3  */
        0x7C853CEEu, /* lfdux  f4, r5, r7   ; r5 += 8,  f4 = double at +48  */
        0x7C853DEEu, /* stfdux f4, r5, r7   ; r5 += 8,  double at +56 = f4  */
        0x4E800020u, /* blr                                                 */
    };
    uint32_t args[8];
    unsigned i;
    float got_s;
    double got_d;

    for (i = 0; i < sizeof blob / sizeof blob[0]; ++i) {
        gw_w32((void *) (uintptr_t) (GW_PPC_TEST_CODE + 4 * i), blob[i]);
    }
    gw_ppc_set_bridge(gw_ppc_test_resolve, NULL, GW_PPC_TEST_CODE,
                      GW_PPC_TEST_CODE + (uint32_t) sizeof blob);

    gw_wf32((void *) (uintptr_t) (GW_PPC_TEST_FDATA + 0), 2.5f);
    gw_wf64((void *) (uintptr_t) (GW_PPC_TEST_FDATA + 8), -7.25);
    gw_wf32((void *) (uintptr_t) (GW_PPC_TEST_FDATA + 16), 0.0f);
    gw_wf64((void *) (uintptr_t) (GW_PPC_TEST_FDATA + 24), 0.0);
    gw_wf32((void *) (uintptr_t) (GW_PPC_TEST_FDATA + 32), 1.75f);
    gw_wf32((void *) (uintptr_t) (GW_PPC_TEST_FDATA + 40), 0.0f);
    gw_wf64((void *) (uintptr_t) (GW_PPC_TEST_FDATA + 48), 3.5);
    gw_wf64((void *) (uintptr_t) (GW_PPC_TEST_FDATA + 56), 0.0);

    args[0] = 0u;                    /* r3  */
    args[1] = 0u;                    /* r4  */
    args[2] = GW_PPC_TEST_FDATA;     /* r5  */
    args[3] = 0u;                    /* r6  */
    args[4] = 8u;                    /* r7  */
    args[5] = 16u;                   /* r8  */
    args[6] = 24u;                   /* r9  */
    args[7] = 32u;                   /* r10 */
    gw_ppc_call(GW_PPC_TEST_CODE, args, 8, 0, GW_PPC_TEST_STACK);

    got_s = gw_rf32((const void *) (uintptr_t) (GW_PPC_TEST_FDATA + 16));
    if (got_s != -7.25f) {
        gw_test_fail("stfsx stored %.4f at +16, expected -7.25 (lfdx/stfsx)", (double) got_s);
        return 1;
    }
    got_d = gw_rf64((const void *) (uintptr_t) (GW_PPC_TEST_FDATA + 24));
    if (got_d != 2.5) {
        gw_test_fail("stfdx stored %.4f at +24, expected 2.5 (lfsx widened to double)", got_d);
        return 1;
    }
    got_s = gw_rf32((const void *) (uintptr_t) (GW_PPC_TEST_FDATA + 40));
    if (got_s != 1.75f) {
        gw_test_fail("stfsux stored %.4f at +40, expected 1.75 - lfsux/stfsux did not advance "
                     "their base register",
                     (double) got_s);
        return 1;
    }
    got_d = gw_rf64((const void *) (uintptr_t) (GW_PPC_TEST_FDATA + 56));
    if (got_d != 3.5) {
        gw_test_fail("stfdux stored %.4f at +56, expected 3.5 - lfdux/stfdux did not advance "
                     "their base register",
                     got_d);
        return 1;
    }
    return 0;
}

/* ---- a HOST-stack out-parameter reaching interpreted code --------------------------------
 * The whole of stage failure class 1: a native engine call site passes a pointer to one of its
 * own stack locals into a callback that turns out to be an m-ex blob, e.g.
 *   ft_80459A8C[i].active_cb(ground, gobj, (Vec3*) &desc)      ftcoll.c, ftColl_8007BAC0
 * Interpreted stores are bounds-checked against guest MEM1, so without the mirror in
 * gw_ppc_call that store dies as "guest access violation ... ea=0x001AFxxx".
 *
 * This test IS the byte-order claim, not just the plumbing: the local is checked with gw_r32,
 * i.e. read BIG-ENDIAN, because gwtool byte-swaps a game TU's accesses to its own stack locals
 * exactly as it does its accesses to the heap. A copy-back that swapped would fail here.
 */
static int test_ppc_host_stack_out_param(void) {
    static const uint32_t blob[] = {
        0x90640000u, /* stw r3, 0(r4)  -- r4 is the caller's stack local */
        0x4E800020u, /* blr */
    };
    volatile uint32_t out_param = 0u; /* a HOST stack local, exactly like ftcoll.c's `desc` */
    const uint32_t value = 0x12345678u;
    uint32_t args[2];
    unsigned i;
    uint32_t got;

    for (i = 0; i < sizeof blob / sizeof blob[0]; ++i) {
        gw_w32((void *) (uintptr_t) (GW_PPC_TEST_CODE + 4 * i), blob[i]);
    }
    gw_ppc_set_bridge(gw_ppc_test_resolve, NULL, GW_PPC_TEST_CODE,
                      GW_PPC_TEST_CODE + (uint32_t) sizeof blob);

    args[0] = value;                                   /* r3 */
    args[1] = (uint32_t) (uintptr_t) &out_param;       /* r4: a host stack address */
    /* NOT GW_PPC_TEST_STACK. That address, 0x80400000, sits inside a game global's guest extent,
     * which the other tests never notice because none of them touches the stack. The mirror does
     * - it is carved off the stack top - and there an interpreted store goes to the global's
     * native storage while the copy-back reads MEM1, so the test failed with a zero. */
    gw_ppc_call(GW_PPC_TEST_CODE, args, 2, 0, GW_PPC_TEST_STACK_MEM1);

    got = gw_r32((const void *) (uintptr_t) &out_param);
    if (got != value) {
        gw_test_fail("host-stack out-parameter holds 0x%08X (raw 0x%08X), expected 0x%08X "
                     "big-endian - the mirror or its copy-back is wrong",
                     got, (unsigned) out_param, value);
        return 1;
    }
    return 0;
}

/* fctiwz + stfiwx: how every PowerPC compiler spells (int)some_float. The value matters -
 * 900.0 is 0x408C200000000000, whose LOW 32 bits are 0 and whose HIGH 32 bits are 0x408C2000,
 * so a stfiwx that reads the wrong half of the FPR stores 0 here and 0 for every other
 * integer-valued float too. That is not a decode failure, so nothing panics; the guest just
 * sees a counter that never arms. */
static int test_ppc_fctiwz_stfiwx(void) {
    static const uint32_t blob[] = {
        0xC0250000u, /* lfs    f1, 0(r5)      ; f1 = 900.0f                 */
        0xFC20081Eu, /* fctiwz f1, f1         ; low word of f1 = 900        */
        0x7C2537AEu, /* stfiwx f1, r5, r6     ; word at +4 = 900            */
        0x4E800020u, /* blr                                                 */
    };
    uint32_t args[8];
    unsigned i;
    uint32_t got;

    for (i = 0; i < sizeof blob / sizeof blob[0]; ++i) {
        gw_w32((void *) (uintptr_t) (GW_PPC_TEST_CODE + 4 * i), blob[i]);
    }
    gw_ppc_set_bridge(gw_ppc_test_resolve, NULL, GW_PPC_TEST_CODE,
                      GW_PPC_TEST_CODE + (uint32_t) sizeof blob);

    gw_wf32((void *) (uintptr_t) (GW_PPC_TEST_FDATA + 0), 900.0f);
    gw_w32((void *) (uintptr_t) (GW_PPC_TEST_FDATA + 4), 0xDEADBEEFu);

    for (i = 0; i < 8; ++i) {
        args[i] = 0u;
    }
    args[2] = GW_PPC_TEST_FDATA; /* r5 */
    args[3] = 4u;                /* r6 */
    gw_ppc_call(GW_PPC_TEST_CODE, args, 8, 0, GW_PPC_TEST_STACK);

    got = gw_r32((const void *) (uintptr_t) (GW_PPC_TEST_FDATA + 4));
    if (got != 900u) {
        gw_test_fail("fctiwz+stfiwx stored 0x%08X, expected 900 - stfiwx read the wrong half "
                     "of the FPR",
                     got);
        return 1;
    }
    return 0;
}

/* A game static's guest address passed as a pointer argument must reach native code as the static's
 * NATIVE storage (ext:307's HSD_SetupTevStageAll(&tev$297)). */
#define GW_PPC_TEST_ARGHELPER_GUEST 0x80380370u
static uint32_t gw_ppc_test_arg_seen;
static uint32_t gw_ppc_test_arghelper(uint32_t a0, uint32_t a1, uint32_t a2, uint32_t a3,
                                      uint32_t a4, uint32_t a5, uint32_t a6, uint32_t a7) {
    (void)a1; (void)a2; (void)a3; (void)a4; (void)a5; (void)a6; (void)a7;
    gw_ppc_test_arg_seen = a0;
    return 0;
}
static gw_ppc_native_fn gw_ppc_test_argresolve(uint32_t guest_addr, void *ctx, gw_ppc_sig *sig) {
    (void)ctx;
    if (guest_addr == GW_PPC_TEST_ARGHELPER_GUEST) {
        sig->float_args = 0;
        sig->n_args = 1;
        sig->ret_float = 0;
        return gw_ppc_test_arghelper;
    }
    return NULL;
}
static int test_ppc_static_pointer_arg(void) {
    static const uint32_t blob[] = {
        0x7C0802A6u, /* mflr r0              */
        0x3C60803Cu, /* lis  r3, 0x803C      */
        0x606352A0u, /* ori  r3, r3, 0x52A0  ; &ftData_803C52A0, a game static */
        0x48000000u | ((GW_PPC_TEST_ARGHELPER_GUEST - (GW_PPC_TEST_CODE + 12)) & 0x03FFFFFCu) | 1u,
        0x7C0803A6u, /* mtlr r0              */
        0x4E800020u, /* blr                  */
    };
    int kind = -1;
    uint32_t native = gw_mex_bridge_lookup(GW_PPC_TEST_STATIC_GUEST, &kind);
    unsigned i;

    if (native == 0 || kind != 0) {
        gw_test_fail("bridge lookup of 0x%08X failed", GW_PPC_TEST_STATIC_GUEST);
        return 1;
    }
    for (i = 0; i < sizeof blob / sizeof blob[0]; ++i) {
        gw_w32((void *)(uintptr_t)(GW_PPC_TEST_CODE + 4 * i), blob[i]);
    }
    gw_ppc_set_bridge(gw_ppc_test_argresolve, NULL, GW_PPC_TEST_CODE,
                      GW_PPC_TEST_CODE + (uint32_t)sizeof blob);
    gw_ppc_test_arg_seen = 0;
    gw_ppc_call(GW_PPC_TEST_CODE, NULL, 0, 0, GW_PPC_TEST_STACK);
    if (gw_ppc_test_arg_seen != native) {
        gw_test_fail("static pointer argument arrived as 0x%08X, expected native 0x%08X",
                     gw_ppc_test_arg_seen, native);
        return 1;
    }
    return 0;
}

/* stwux must move its base register (ext:306's aligned-frame prologue is `stwux r10, r1, r9`),
 * and sthbrx must store two bytes, not the four it used to when XO 918 was mislabelled stwbrx. */
static int test_ppc_update_indexed_and_brx(void) {
    static const uint32_t blob[] = {
        0x7D45316Eu, /* stwux  r10, r5, r6    ; [r5-8] = r10, r5 -= 8       */
        0x90A50004u, /* stw    r5, 4(r5)      ; record the updated base     */
        0x7C803F2Cu, /* sthbrx r4, 0, r7      ; two bytes, swapped          */
        0x4E800020u, /* blr                                                 */
    };
    uint32_t args[8];
    unsigned i;
    uint32_t got;

    for (i = 0; i < sizeof blob / sizeof blob[0]; ++i) {
        gw_w32((void *) (uintptr_t) (GW_PPC_TEST_CODE + 4 * i), blob[i]);
    }
    gw_ppc_set_bridge(gw_ppc_test_resolve, NULL, GW_PPC_TEST_CODE,
                      GW_PPC_TEST_CODE + (uint32_t) sizeof blob);

    for (i = 0; i < 4; ++i) {
        gw_w32((void *) (uintptr_t) (GW_PPC_TEST_FDATA + 4 * i), 0xDEADBEEFu);
    }
    for (i = 0; i < 8; ++i) {
        args[i] = 0u;
    }
    args[1] = 0x1234u;                  /* r4  */
    args[2] = GW_PPC_TEST_FDATA + 16u;  /* r5  */
    args[3] = 0xFFFFFFF8u;              /* r6 = -8 */
    args[4] = GW_PPC_TEST_FDATA;        /* r7  */
    args[7] = 0xCAFEF00Du;              /* r10 */
    gw_ppc_call(GW_PPC_TEST_CODE, args, 8, 0, GW_PPC_TEST_STACK);

    got = gw_r32((const void *) (uintptr_t) (GW_PPC_TEST_FDATA + 8));
    if (got != 0xCAFEF00Du) {
        gw_test_fail("stwux stored 0x%08X at base-8, expected 0xCAFEF00D", got);
        return 1;
    }
    got = gw_r32((const void *) (uintptr_t) (GW_PPC_TEST_FDATA + 12));
    if (got != GW_PPC_TEST_FDATA + 8u) {
        gw_test_fail("stwux left the base at 0x%08X, expected 0x%08X", got,
                     GW_PPC_TEST_FDATA + 8u);
        return 1;
    }
    got = gw_r32((const void *) (uintptr_t) GW_PPC_TEST_FDATA);
    if (got != 0x3412BEEFu) {
        gw_test_fail("sthbrx left 0x%08X, expected 0x3412BEEF (two swapped bytes, rest intact)",
                     got);
        return 1;
    }
    return 0;
}

void gw_ppc_tests_register(void) {
    gw_test_register("ppc_call_bridged_helper", test_ppc_call_bridged_helper);
    gw_test_register("ppc_float_bridge", test_ppc_float_bridge);
    gw_test_register("ppc_ext_stack_args", test_ppc_ext_stack_args);
    gw_test_register("ppc_ext_aggregate_double", test_ppc_ext_aggregate_double);
    gw_test_register("ppc_varargs_bridge", test_ppc_varargs_bridge);
    gw_test_register("ppc_varargs_format_order", test_ppc_varargs_format_order);
    gw_test_register("ppc_static_bridge", test_ppc_static_bridge);
    gw_test_register("ppc_native_pointer", test_ppc_native_pointer);
    gw_test_register("ppc_reentry_cap", test_ppc_reentry_cap);
    gw_test_register("ppc_fp_aform_decode", test_ppc_fp_aform_decode);
    gw_test_register("ppc_fcmpu_orderings", test_ppc_fcmpu_orderings);
    gw_test_register("ppc_xoris_int_to_float", test_ppc_xoris_int_to_float);
    gw_test_register("ppc_fp_indexed", test_ppc_fp_indexed);
    gw_test_register("ppc_fctiwz_stfiwx", test_ppc_fctiwz_stfiwx);
    gw_test_register("ppc_host_stack_out_param", test_ppc_host_stack_out_param);
    gw_test_register("ppc_update_indexed_and_brx", test_ppc_update_indexed_and_brx);
    gw_test_register("ppc_static_pointer_arg", test_ppc_static_pointer_arg);
}
