/* Oracle for the Rust PPC interpreter (tools/mex-import/src/ppc): runs the execute core CUT FROM
 * pc/platform/gw_ppc.c (ppc_oracle.sh; nothing here re-implements an instruction) over random
 * programs and prints one line per case:
 *
 *   <seed> <n> <word>... = <exit> <pc> <state hash>
 *
 * The initial machine state is a pure function of <seed> (init_state below; tests/ppc_oracle.rs
 * has the same function), the program words are printed, and the final state is folded into one
 * FNV-1a 64 hash: r0-r31, f0-f31 (raw bits), lr, ctr, xer, cr, pc, the whole test memory and the
 * bridge call log. <exit> is 0 = returned (blr / tail call to the native caller), 1 = memory fault,
 * 2 = unimplemented opcode, 3 = gw_panic (bad mfspr/mtspr), 4 = fetch outside the code range,
 * 5 = instruction budget.
 *
 * Environment: PPC_ORACLE_DUMP=<seed> prints that case's final registers instead.
 */
#include <setjmp.h>
#include <stdarg.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <math.h>

#include "gw_ppc.h"

static inline uint16_t gw_bswap16(uint16_t v) { return (uint16_t)((v >> 8) | (v << 8)); }
static inline uint32_t gw_bswap32(uint32_t v) {
    return (v >> 24) | ((v >> 8) & 0xFF00u) | ((v << 8) & 0xFF0000u) | (v << 24);
}

#include "ppc_cut_types.c"

/* ---- glue the cut code calls ------------------------------------------------------------ */

#define MEM_BASE 0x80300000u
#define MEM_SIZE 0x10000u
#define CODE_BASE MEM_BASE
static uint8_t g_mem[MEM_SIZE];
static uint32_t g_code_hi;
static jmp_buf g_jmp;
static uint64_t g_log; /* bridge call log hash */

enum { EXIT_RET = 0, EXIT_MEM = 1, EXIT_BADOP = 2, EXIT_PANIC = 3, EXIT_FETCH = 4, EXIT_BUDGET = 5 };

static uint64_t fnv(uint64_t h, uint64_t v, int bytes) {
    int i;
    for (i = 0; i < bytes; ++i) {
        h ^= (uint8_t)(v >> (8 * i));
        h *= 0x100000001B3ull;
    }
    return h;
}

static void gw_panic(const char *fmt, ...) {
    (void)fmt;
    longjmp(g_jmp, EXIT_PANIC);
}
static void gw_ppc_bad_opcode(const gw_ppc_machine *m, uint32_t ip, uint32_t insn) {
    (void)m; (void)ip; (void)insn;
    longjmp(g_jmp, EXIT_BADOP);
}
static uint32_t ea_ok(uint32_t ea, uint32_t size) {
    if (ea < MEM_BASE || (uint64_t)ea + size > (uint64_t)MEM_BASE + MEM_SIZE) {
        longjmp(g_jmp, EXIT_MEM);
    }
    return ea - MEM_BASE;
}
static uint8_t gw_ppc_ld8(gw_ppc_machine *m, uint32_t ea) { (void)m; return g_mem[ea_ok(ea, 1)]; }
static uint16_t gw_ppc_ld16(gw_ppc_machine *m, uint32_t ea) {
    uint32_t o = ea_ok(ea, 2); (void)m;
    return (uint16_t)((g_mem[o] << 8) | g_mem[o + 1]);
}
static uint32_t gw_ppc_ld32(gw_ppc_machine *m, uint32_t ea) {
    uint32_t o = ea_ok(ea, 4); (void)m;
    return ((uint32_t)g_mem[o] << 24) | ((uint32_t)g_mem[o + 1] << 16) | ((uint32_t)g_mem[o + 2] << 8) | g_mem[o + 3];
}
static uint64_t gw_ppc_ld64(gw_ppc_machine *m, uint32_t ea) {
    uint32_t o = ea_ok(ea, 8);
    uint64_t v = 0;
    int i;
    (void)m;
    for (i = 0; i < 8; ++i) v = (v << 8) | g_mem[o + i];
    return v;
}
static void gw_ppc_st8(gw_ppc_machine *m, uint32_t ea, uint8_t v) { (void)m; g_mem[ea_ok(ea, 1)] = v; }
static void gw_ppc_st16(gw_ppc_machine *m, uint32_t ea, uint16_t v) {
    uint32_t o = ea_ok(ea, 2); (void)m;
    g_mem[o] = (uint8_t)(v >> 8); g_mem[o + 1] = (uint8_t)v;
}
static void gw_ppc_st32(gw_ppc_machine *m, uint32_t ea, uint32_t v) {
    uint32_t o = ea_ok(ea, 4); int i; (void)m;
    for (i = 0; i < 4; ++i) g_mem[o + i] = (uint8_t)(v >> (24 - 8 * i));
}
static void gw_ppc_st64(gw_ppc_machine *m, uint32_t ea, uint64_t v) {
    uint32_t o = ea_ok(ea, 8); int i; (void)m;
    for (i = 0; i < 8; ++i) g_mem[o + i] = (uint8_t)(v >> (56 - 8 * i));
}
static int gw_ppc_in_blob(const gw_ppc_machine *m, uint32_t addr) {
    (void)m;
    return addr >= CODE_BASE && addr < g_code_hi;
}
/* The recording bridge: what a native callee would do, reduced to a deterministic function of
 * the target and the argument registers (tests/ppc_oracle.rs's TestBridge is the same). */
static void gw_ppc_bridge_call(gw_ppc_machine *m, uint32_t guest_addr) {
    gw_ppc_ctx *c = &m->cpu;
    g_log = fnv(g_log, guest_addr, 4);
    g_log = fnv(g_log, c->gpr[3], 4);
    c->gpr[3] = (guest_addr * 2654435761u) ^ c->gpr[4];
    c->fpr[1].d = (double)(int32_t)c->gpr[5] * 0.25;
}

static uint32_t gw_ppc_cr_bit(const gw_ppc_ctx *c, uint32_t bi);
static int gw_ppc_execute_x(gw_ppc_machine *m, uint32_t insn);
static int gw_ppc_execute_fp(gw_ppc_machine *m, uint32_t insn, int single);
static int gw_ppc_execute_cr(gw_ppc_machine *m, uint32_t insn);
static double gw_ppc_sqrt(double x);

#include "ppc_cut_funcs.c"

/* ---- the random machine ----------------------------------------------------------------- */

static uint64_t g_rng;
static uint64_t rnd(void) { /* splitmix64 */
    uint64_t z = (g_rng += 0x9E3779B97F4A7C15ull);
    z = (z ^ (z >> 30)) * 0xBF58476D1CE4E5B9ull;
    z = (z ^ (z >> 27)) * 0x94D049BB133111EBull;
    return z ^ (z >> 31);
}
static uint32_t r32(void) { return (uint32_t)rnd(); }
static uint32_t below(uint32_t n) { return (uint32_t)(rnd() % n); }

static const uint64_t FP_SPECIAL[] = {
    0x0000000000000000ull, 0x8000000000000000ull, 0x3FF0000000000000ull, 0xBFF0000000000000ull,
    0x7FF0000000000000ull, 0xFFF0000000000000ull, 0x7FF8000000000000ull, 0x7FF8000012345678ull,
    0xFFF4000000000001ull, 0x0000000000000001ull, 0x41E0000000000000ull, 0xC1E0000000200000ull,
    0xC1E0000000000000ull, 0x41DFFFFFFFC00000ull, 0x3FE0000000000000ull, 0xBFE0000000000000ull,
    0x4004000000000000ull, 0x47EFFFFFE0000000ull, 0x36A0000000000000ull, 0x4330000080000000ull,
};

/* The initial state of case `seed`; tests/ppc_oracle.rs::init_state is the same function. */
static void init_state(uint64_t seed, gw_ppc_ctx *c) {
    int i;
    g_rng = seed * 0x2545F4914F6CDD1Dull + 1;
    for (i = 0; i < 32; ++i) {
        uint32_t k = below(4);
        c->gpr[i] = k == 0 ? below(17) - 8 : k == 1 ? (r32() | 0x80000000u) : r32();
    }
    c->gpr[1] = MEM_BASE + 0xF000;
    c->gpr[27] = below(0x40) * 4;
    for (i = 28; i < 32; ++i) c->gpr[i] = MEM_BASE + 0x8000 + (uint32_t)(i - 28) * 0x800 + below(0x40) * 4;
    for (i = 0; i < 32; ++i) {
        uint32_t k = below(5);
        if (k == 0) {
            c->fpr[i].u64 = FP_SPECIAL[below(sizeof FP_SPECIAL / sizeof FP_SPECIAL[0])];
        } else if (k == 1) { /* a single-precision value */
            float f;
            uint32_t b = r32();
            memcpy(&f, &b, 4);
            c->fpr[i].d = (double)f;
        } else if (k == 2) { /* a modest number */
            c->fpr[i].d = ((double)(int32_t)r32()) / (double)(1u << below(24));
        } else {
            c->fpr[i].u64 = rnd();
        }
    }
    c->cr = r32();
    c->xer = r32() & 0xE000007Fu;
    c->ctr = below(4);
    c->lr = 0;
    c->pc = CODE_BASE;
    for (i = 0; i < (int)MEM_SIZE; ++i) g_mem[i] = (uint8_t)(((uint32_t)i * 131u + 7u) ^ ((uint32_t)i >> 8));
}

/* ---- the random program -------------------------------------------------------------- */

static uint32_t D(uint32_t op, uint32_t t, uint32_t a, uint32_t d) { return op << 26 | t << 21 | a << 16 | (d & 0xFFFF); }
static uint32_t X(uint32_t t, uint32_t a, uint32_t b, uint32_t xo, uint32_t rc) { return 31u << 26 | t << 21 | a << 16 | b << 11 | xo << 1 | rc; }
static uint32_t dest(void) { uint32_t r; do { r = below(27); } while (r == 1); return r; }
static uint32_t src(void) { return below(32); }
static uint32_t base(void) { return below(50) == 0 ? 0 : 28 + below(4); }
static uint32_t disp(uint32_t align) { return ((below(0x400) - 0x200) & ~(align - 1)); }

static const uint32_t X_ARITH[] = {266, 40, 104, 235, 75, 11, 491, 459, 8, 136, 200, 232, 202, 10, 138, 234};
static const uint32_t X_LOGIC[] = {60, 412, 476, 284, 28, 444, 316, 124, 24, 536, 792, 824, 26, 954, 922};
static const uint32_t X_MEM[] = {23, 87, 215, 279, 407, 343, 151, 790, 534, 662, 918, 55, 119, 311, 375, 183,
                                 247, 439, 535, 567, 599, 631, 663, 695, 727, 759, 983, 1014, 1015,
                                 54, 86, 150, 246, 278, 470, 598, 854, 982};
static const uint32_t A_FP[] = {18, 20, 21, 22, 24, 25, 26, 28, 29, 30, 31, 23};
static const uint32_t X_FP[] = {72, 40, 264, 136, 12, 15, 14, 0, 32};
static const uint32_t CR_OPS[] = {0, 33, 129, 193, 225, 257, 289, 417, 449};
static const uint32_t BO[] = {12, 4, 20, 16, 18, 0, 2, 8, 10, 13, 5};

#define N_OF(a) (sizeof(a) / sizeof((a)[0]))

static uint32_t gen(uint32_t i, uint32_t n) {
    uint32_t k = below(100);
    if (below(400) == 0) { /* rare: something the interpreter refuses */
        switch (below(5)) {
        case 0: return D(11, (below(8) << 2) | 1, src(), r32()); /* cmpi L=1 */
        case 1: return X(dest(), src(), src(), 266 | 512, 0);      /* addo (OE) */
        case 2: return 4u << 26 | r32() >> 6;                     /* paired-single */
        case 3: return X(dest(), below(32), below(32), 339, 0);   /* mfspr <random> */
        default: return 19u << 26 | X(0, 0, 0, 150, 0) << 6 >> 6; /* op19 xo 150 (isync): unimplemented there */
        }
    }
    if (k < 14) { /* integer immediate */
        static const uint32_t ops[] = {14, 15, 7, 8, 12, 13, 24, 25, 26, 27, 28, 29};
        uint32_t op = ops[below(N_OF(ops))];
        if (op >= 24) return D(op, src(), dest(), r32());
        return D(op, dest(), src(), r32());
    }
    if (k < 22) { /* rotate */
        uint32_t op = below(3) == 0 ? 20 : below(2) ? 21 : 23;
        return op << 26 | src() << 21 | dest() << 16 | (op == 23 ? src() : below(32)) << 11 | below(32) << 6 | below(32) << 1 | below(2);
    }
    if (k < 28) { /* compare */
        uint32_t crf = below(8) << 2;
        switch (below(4)) {
        case 0: return D(11, crf, src(), r32());
        case 1: return D(10, crf, src(), r32());
        case 2: return X(crf, src(), src(), 0, 0);
        default: return X(crf, src(), src(), 32, 0);
        }
    }
    if (k < 38) return X(dest(), src(), src(), X_ARITH[below(N_OF(X_ARITH))], below(2));
    if (k < 46) return X(src(), dest(), below(32), X_LOGIC[below(N_OF(X_LOGIC))], below(2));
    if (k < 60) { /* d-form loads/stores */
        uint32_t op = 32 + below(24);
        uint32_t al = (op >= 48) ? ((op >= 50 && op != 52 && op != 53) ? 8 : 4) : 1;
        if (op == 46 || op == 47) { /* lmw/stmw: high registers only, so the bases survive mostly */
            if (op == 46 && i + 1 < n) op = 47;
            return D(op, 24 + below(8), 28 + below(4), disp(4));
        }
        if (op >= 48) return D(op, below(32), base(), disp(al));
        return D(op, (op & 4) ? src() : dest(), base(), disp(al));
    }
    if (k < 68) { /* x-form memory and cache */
        uint32_t xo = X_MEM[below(N_OF(X_MEM))];
        uint32_t t = (xo >= 535 && xo <= 983) || xo == 215 || xo == 407 || xo == 151 || xo == 662 || xo == 918 ||
                             xo == 183 || xo == 247 || xo == 439 ? below(32) : dest();
        return X(t, base(), 27, xo, 0);
    }
    if (k < 72) { /* cr / spr moves */
        static const uint32_t sprs[] = {1, 8, 9};
        uint32_t spr = sprs[below(3)];
        uint32_t sprf = (spr & 0x1F) << 5 | (spr >> 5);
        switch (below(4)) {
        case 0: return X(dest(), 0, 0, 19, 0);
        case 1: return 31u << 26 | src() << 21 | below(256) << 12 | 144u << 1;
        case 2: return X(dest(), sprf >> 5, sprf & 0x1F, 339, 0);
        default: return spr == 8 && below(2) ? X(src(), 0, 0, 444, 0) : X(src(), sprf >> 5, sprf & 0x1F, 467, 0);
        }
    }
    if (k < 76) { /* cr logical */
        uint32_t xo = CR_OPS[below(N_OF(CR_OPS))];
        if (xo == 0) return 19u << 26 | below(8) << 23 | below(8) << 18 | 0;
        return 19u << 26 | below(32) << 21 | below(32) << 16 | below(32) << 11 | xo << 1;
    }
    if (k < 90) { /* floating point */
        uint32_t op = below(2) ? 59 : 63;
        if (below(2)) {
            return op << 26 | below(32) << 21 | below(32) << 16 | below(32) << 11 | below(32) << 6 | A_FP[below(N_OF(A_FP))] << 1 | below(2);
        }
        {
            uint32_t xo = X_FP[below(N_OF(X_FP))];
            uint32_t t = (xo == 0 || xo == 32) ? below(8) << 2 : below(32);
            return op << 26 | t << 21 | below(32) << 16 | below(32) << 11 | xo << 1;
        }
    }
    /* branches */
    switch (below(8)) {
    case 0: case 1: case 2: { /* bc forward inside the program */
        uint32_t ahead = 1 + below(4);
        uint32_t lk = below(12) == 0;
        if (i + ahead >= n) ahead = n - i;
        return 16u << 26 | BO[below(N_OF(BO))] << 21 | below(32) << 16 | ((ahead * 4) & 0xFFFC) | lk;
    }
    case 3: /* bl to a native function */
        return 18u << 26 | ((0x1000u + below(0x100) * 4) & 0x3FFFFFC) | 2 | 1;
    case 4: /* b (tail) to a native function */
        if (below(4) == 0) return 18u << 26 | ((0x1000u + below(0x100) * 4) & 0x3FFFFFC) | 2;
        return 18u << 26 | 4u; /* b +4 */
    case 5: /* blrl / bctrl / conditional blr / bdnz-style backward */
        switch (below(4)) {
        case 0: return 0x4E800021u;                             /* blrl */
        case 1: return 0x4E800421u;                             /* bctrl */
        case 2: return 19u << 26 | BO[below(N_OF(BO))] << 21 | below(32) << 16 | 16u << 1; /* bclr cond */
        default: return 16u << 26 | 16u << 21 | ((0u - 4u * (1 + below(3))) & 0xFFFC); /* bdnz back */
        }
    default:
        return D(14, dest(), src(), r32());
    }
}

int main(int argc, char **argv) {
    uint32_t cases = argc > 1 ? (uint32_t)atoi(argv[1]) : 6000;
    const char *dump = getenv("PPC_ORACLE_DUMP");
    uint64_t seed;
    for (seed = 1; seed <= cases; ++seed) {
        gw_ppc_machine m;
        uint32_t words[64];
        uint32_t n, i;
        int exit_kind;
        volatile uint32_t budget = 4000;
        uint64_t h = 0xCBF29CE484222325ull;
        memset(&m, 0, sizeof m);
        init_state(seed, &m.cpu);
        n = 8 + below(48);
        for (i = 0; i < n; ++i) words[i] = gen(i, n);
        words[n] = 0x4E800020u; /* blr */
        for (i = 0; i <= n; ++i) {
            uint32_t w = words[i];
            g_mem[4 * i] = (uint8_t)(w >> 24); g_mem[4 * i + 1] = (uint8_t)(w >> 16);
            g_mem[4 * i + 2] = (uint8_t)(w >> 8); g_mem[4 * i + 3] = (uint8_t)w;
        }
        g_code_hi = CODE_BASE + 4 * (n + 1);
        g_log = 0xCBF29CE484222325ull;
        exit_kind = setjmp(g_jmp);
        if (exit_kind == 0) {
            for (;;) {
                uint32_t ip = m.cpu.pc, insn;
                if (budget-- == 0) longjmp(g_jmp, EXIT_BUDGET);
                if (!gw_ppc_in_blob(&m, ip)) longjmp(g_jmp, EXIT_FETCH);
                insn = gw_ppc_ld32(&m, ip);
                m.cpu.pc = ip + 4;
                if (gw_ppc_execute(&m, insn)) break;
            }
            exit_kind = EXIT_RET;
        }
        for (i = 0; i < 32; ++i) h = fnv(h, m.cpu.gpr[i], 4);
        for (i = 0; i < 32; ++i) h = fnv(h, m.cpu.fpr[i].u64, 8);
        h = fnv(h, m.cpu.lr, 4); h = fnv(h, m.cpu.ctr, 4); h = fnv(h, m.cpu.xer, 4);
        h = fnv(h, m.cpu.cr, 4); h = fnv(h, m.cpu.pc, 4);
        for (i = 0; i < MEM_SIZE; ++i) h = fnv(h, g_mem[i], 1);
        h = fnv(h, g_log, 8);
        if (dump && (uint64_t)atoi(dump) == seed) {
            for (i = 0; i < 32; ++i) printf("r%u=%08X f%u=%016llX\n", i, m.cpu.gpr[i], i, (unsigned long long)m.cpu.fpr[i].u64);
            printf("lr=%08X ctr=%08X xer=%08X cr=%08X pc=%08X exit=%d\n", m.cpu.lr, m.cpu.ctr, m.cpu.xer, m.cpu.cr, m.cpu.pc, exit_kind);
            return 0;
        }
        printf("%llu %u", (unsigned long long)seed, n + 1);
        for (i = 0; i <= n; ++i) printf(" %08X", words[i]);
        printf(" = %d %08X %016llX\n", exit_kind, m.cpu.pc, (unsigned long long)h);
    }
    return 0;
}
