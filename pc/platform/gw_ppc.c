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

void gw_ppc_set_bridge(gw_ppc_resolver_fn resolve, void *ctx, uint32_t code_lo, uint32_t code_hi) {
    gw_ppc_m.resolve = resolve;
    gw_ppc_m.bridge_ctx = ctx;
    gw_ppc_m.code_lo = code_lo;
    gw_ppc_m.code_hi = code_hi;
}

/* ---- guest memory access (bounds-checked, big-endian) ---------------------------------- */

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

static void gw_ppc_access_violation(uint32_t ip, uint32_t ea) {
    gw_panic("ppc: guest access violation at ip=0x%08X ea=0x%08X", ip, ea);
}

static uint8_t gw_ppc_ld8(gw_ppc_machine *m, uint32_t ea) {
    if (!gw_ppc_ea_ok(ea, 1)) {
        gw_ppc_access_violation(m->cpu.pc - 4, ea);
    }
    return gw_r8((const void *)(uintptr_t)ea);
}
static uint16_t gw_ppc_ld16(gw_ppc_machine *m, uint32_t ea) {
    if (!gw_ppc_ea_ok(ea, 2)) {
        gw_ppc_access_violation(m->cpu.pc - 4, ea);
    }
    return gw_r16((const void *)(uintptr_t)ea);
}
static uint32_t gw_ppc_ld32(gw_ppc_machine *m, uint32_t ea) {
    if (!gw_ppc_ea_ok(ea, 4)) {
        gw_ppc_access_violation(m->cpu.pc - 4, ea);
    }
    return gw_r32((const void *)(uintptr_t)ea);
}
static uint64_t gw_ppc_ld64(gw_ppc_machine *m, uint32_t ea) {
    if (!gw_ppc_ea_ok(ea, 8)) {
        gw_ppc_access_violation(m->cpu.pc - 4, ea);
    }
    return gw_r64((const void *)(uintptr_t)ea);
}
static void gw_ppc_st8(gw_ppc_machine *m, uint32_t ea, uint8_t v) {
    if (!gw_ppc_ea_ok(ea, 1)) {
        gw_ppc_access_violation(m->cpu.pc - 4, ea);
    }
    gw_w8((void *)(uintptr_t)ea, v);
}
static void gw_ppc_st16(gw_ppc_machine *m, uint32_t ea, uint16_t v) {
    if (!gw_ppc_ea_ok(ea, 2)) {
        gw_ppc_access_violation(m->cpu.pc - 4, ea);
    }
    gw_w16((void *)(uintptr_t)ea, v);
}
static void gw_ppc_st32(gw_ppc_machine *m, uint32_t ea, uint32_t v) {
    if (!gw_ppc_ea_ok(ea, 4)) {
        gw_ppc_access_violation(m->cpu.pc - 4, ea);
    }
    gw_w32((void *)(uintptr_t)ea, v);
}
static void gw_ppc_st64(gw_ppc_machine *m, uint32_t ea, uint64_t v) {
    if (!gw_ppc_ea_ok(ea, 8)) {
        gw_ppc_access_violation(m->cpu.pc - 4, ea);
    }
    gw_w64((void *)(uintptr_t)ea, v);
}

/* ---- panics --------------------------------------------------------------------------- */

static void gw_ppc_bad_opcode(uint32_t ip, uint32_t insn) {
    gw_panic("ppc: unimplemented opcode at ip=0x%08X word=0x%08X", ip, insn);
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
    if (m->resolve == NULL) {
        gw_panic("ppc: branch to 0x%08X outside blob with no resolver (ip=0x%08X)", guest_addr,
                 c->pc - 4);
    }
    fn = m->resolve(guest_addr, m->bridge_ctx);
    if (fn == NULL) {
        gw_panic("ppc: resolver returned NULL for guest address 0x%08X", guest_addr);
    }
    /* Marshal r3..r10 into the native call and take the word back into r3. */
    c->gpr[3] = fn(c->gpr[3], c->gpr[4], c->gpr[5], c->gpr[6], c->gpr[7], c->gpr[8], c->gpr[9],
                   c->gpr[10]);
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

static int gw_ppc_in_blob(const gw_ppc_machine *m, uint32_t addr) {
    return addr >= m->code_lo && addr < m->code_hi;
}

/* ---- instruction fetch ----------------------------------------------------------------- */

static uint32_t gw_ppc_fetch(gw_ppc_machine *m, uint32_t ip) {
    if (!gw_ppc_in_blob(m, ip)) {
        gw_panic("ppc: pc=0x%08X outside blob code range [0x%08X,0x%08X)", ip, m->code_lo,
                 m->code_hi);
    }
    return gw_ppc_ld32(m, ip);
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
            gw_ppc_bad_opcode(ip, insn);
        }
        ra = (insn >> 16) & 0x1F;
        imm = (int32_t)(int16_t)(insn & 0xFFFF);
        gw_ppc_cmp(c, (insn >> 23) & 7, c->gpr[ra], (uint32_t)imm, 0);
        break;
    case 10: /* cmplwi */
        if ((insn >> 21) & 1) {
            gw_ppc_bad_opcode(ip, insn);
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
        gw_ppc_bad_opcode(ip, insn);
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
            gw_ppc_bad_opcode(ip, insn);
        }
        gw_ppc_cmp(c, (insn >> 23) & 7, c->gpr[ra], c->gpr[rb], 0);
        break;
    case 32: /* cmpl / cmplw */
        if ((insn >> 21) & 1) {
            gw_ppc_bad_opcode(ip, insn);
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
            gw_ppc_access_violation(ip, ea);
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
        gw_ppc_bad_opcode(ip, insn);
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
        gw_ppc_bad_opcode(ip, insn);
    }
    return 0;
}

/* ---- opcode 59 (single-precision) and 63 (double-precision) floating point ------------
 * Single ops compute in double then round to single (frsp semantics); Gekko's 25-bit-mantissa
 * rounding is not modeled in Phase 1. NaN/exception behavior is approximated. */

static int gw_ppc_execute_fp(gw_ppc_machine *m, uint32_t insn, int single) {
    gw_ppc_ctx *c = &m->cpu;
    uint32_t xo = (insn >> 1) & 0x3FF;
    uint32_t ip = c->pc - 4;
    uint32_t fd = (insn >> 21) & 0x1F;
    uint32_t fa = (insn >> 16) & 0x1F;
    uint32_t fb = (insn >> 11) & 0x1F;
    uint32_t fc = (insn >> 6) & 0x1F;
    double a = c->fpr[fa].d;
    double b = c->fpr[fb].d;
    double r;

    /* fmr/fneg/fabs/frsp/fctiwz are rD,rB form (bits 16..20 are zero) and appear in opcode 63. */
    switch (xo) {
    case 21: /* fadd / fadds */
        r = a + b;
        c->fpr[fd].d = single ? (double)(float)r : r;
        break;
    case 20: /* fsub / fsubs */
        r = a - b;
        c->fpr[fd].d = single ? (double)(float)r : r;
        break;
    case 25: /* fmul / fmuls */
        r = a * b;
        c->fpr[fd].d = single ? (double)(float)r : r;
        break;
    case 18: /* fdiv / fdivs */
        r = a / b;
        c->fpr[fd].d = single ? (double)(float)r : r;
        break;

    case 72: /* fmr: fD = fB */
        c->fpr[fd].d = b;
        break;
    case 40: /* fneg */
        c->fpr[fd].d = -b;
        break;
    case 264: /* fabs */
        c->fpr[fd].d = b < 0 ? -b : b;
        break;
    case 12: /* frsp: round to single */
        c->fpr[fd].d = (double)(float)b;
        break;
    case 15: /* fctiwz: convert to 32-bit integer (truncate), store sign-extended in the FPR */
    {
        int32_t iv = (int32_t)b;
        c->fpr[fd].u64 = (uint64_t)(int64_t)iv;
        break;
    }
    case 14: /* fctiw (round to nearest) */
    {
        int32_t iv = (int32_t)(b >= 0 ? b + 0.5 : b - 0.5);
        c->fpr[fd].u64 = (uint64_t)(int64_t)iv;
        break;
    }

    case 0: /* fcmpu */
    case 32: /* fcmpo (FP exception not modeled) */
    {
        uint32_t crfd = (insn >> 23) & 7;
        uint32_t so = (c->xer >> 31) & 1u;
        uint32_t field;
        if (a != a || b != b) { /* NaN: unordered */
            field = so;         /* LT=GT=EQ=0 */
        } else if (a > b) {
            field = (2u << 2) | so; /* GT */
        } else if (a < b) {
            field = (1u << 3) | so; /* LT */
        } else {
            field = (1u << 1) | so; /* EQ */
        }
        gw_ppc_cr_set_field(c, crfd, field);
        break;
    }

    case 23: /* fsel: fD = (fA >= 0 or NaN) ? fC : fB */
        c->fpr[fd].d = (a >= 0.0 || a != a) ? c->fpr[fc].d : b;
        break;

    default:
        gw_ppc_bad_opcode(ip, insn);
    }
    return 0;
}

/* ---- run loop ------------------------------------------------------------------------ */

static uint32_t gw_ppc_run(gw_ppc_machine *m) {
    for (;;) {
        uint32_t ip = m->cpu.pc;
        uint32_t insn = gw_ppc_fetch(m, ip);
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

static gw_ppc_native_fn gw_ppc_test_resolve(uint32_t guest_addr, void *ctx) {
    (void)ctx;
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

void gw_ppc_tests_register(void) {
    gw_test_register("ppc_call_bridged_helper", test_ppc_call_bridged_helper);
}
