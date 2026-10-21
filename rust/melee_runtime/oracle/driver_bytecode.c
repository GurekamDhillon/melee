/* Oracle driver for baselib bytecode.c (HSD_ByteCodeEval) and lb_00CE.c expf / powf / powi
 * (see build_bytecode.sh). Generated inputs only; writes hex rows that tests/oracle_bytecode.rs
 * replays.
 *
 * Rows:
 *   E <x> <expf(x)>
 *   P <x> <y> <powf(x,y)>
 *   I <base> <exp> <powi>
 *   B <seed_in> <code hex> <nargs> <args...> <result> <seed_out>
 *
 * Programs are generated with a tracked stack depth so every HSD_ASSERT holds; branches (3) and
 * jumps (4) skip a block of net stack effect 0, so both paths agree on the depth. expf, powf and
 * fmodf only get operands pushed right before them from safe ranges (expf(NaN), expf(|x| >~ 13: the f32 terms reach inf/inf) and powf(x, 0*inf)
 * never terminate; fmodf's s64 cast of NaN is a TODO(core) difference). */
#include <stdio.h>
#include <string.h>
#include <stdint.h>
#include <stdlib.h>
#include <math.h>
typedef float f32; typedef unsigned u32; typedef int s32; typedef unsigned char u8;

float HSD_ByteCodeEval(u8* bytecode, const f32* args, s32 nb_args);
float expf(float); float powf(float, float); s32 powi(s32, s32);
extern u32* HSD_RandSeedPtr;

static uint64_t st = 0x9E3779B97F4A7C15ull;
static uint64_t nx(void) { st ^= st << 13; st ^= st >> 7; st ^= st << 17; return st; }
static u32 rn(u32 n) { return (u32) (nx() >> 33) % n; }
static u32 fb(f32 f) { u32 u; memcpy(&u, &f, 4); return u; }
static f32 rf(f32 lo, f32 hi) { return lo + (f32) ((double) (nx() >> 11) / 9007199254740992.0) * (hi - lo); }

#define CAP 1024
static u8 code[CAP];
static int n;
static void b1(u32 v) { code[n++] = (u8) v; }
static void b2(u32 v) { b1(v >> 8); b1(v); }
static void b4(u32 v) { b2(v >> 16); b2(v); }
static void pushw(u32 w) { b1(6); b4(w); }
static void pushf(f32 f) { pushw(fb(f)); }

static u32 rword(void)
{
    switch (rn(8)) {
    case 0: return (u32) rn(9) - 4;              /* small ints */
    case 1: return (u32) (nx() >> 32);           /* anything */
    case 2: return fb(rf(-2.f, 2.f));
    case 3: return fb(rf(-400.f, 400.f));
    case 4: return fb((f32) ((s32) rn(13) - 6)); /* integral floats */
    case 5: { static const u32 sp[] = { 0, 0x80000000u, 0x7F800000u, 0xFF800000u, 0x7FC00000u, 0x3F800000u,
                                         0xBF800000u, 0x80000001u, 0x7FFFFFFFu, 1u, 0x4F000000u, 0xCF000001u };
              return sp[rn(12)]; }
    case 6: return fb(rf(-1e9f, 1e9f));
    default: return (u32) rn(100);
    }
}

static const u8 un[] = { 0, 7, 8, 9, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10, 0x11, 0x12, 0x13, 0x15, 0x28, 0x16, 0x31 };
static const u8 bin[] = { 0x17, 0x18, 0x19, 0x1A, 0x1C, 0x1D, 0x1E, 0x1F, 0x20, 0x22, 0x23, 0x24, 0x25, 0x26, 0x33, 0x34,
                          0x35, 0x36, 0x37, 0x38, 0x29, 0x2A, 0x2B, 0x2C, 0x2D, 0x2E, 0x2F, 0x30, 0x32, 0x39, 0x3A,
                          0x3B, 0x27 };

static int nargs;
/* Emit a block; returns its net stack effect (always >= -depth). budget bounds the size. */
static void block(int* depth, int budget, int net0);
static void op(int* depth, int budget)
{
    u32 r = rn(100);
    if (n > CAP - 64) return;
    if (r < 22 || *depth == 0) {
        if (rn(3) == 0 && nargs > 0) { b1(2); b2(rn(nargs)); } else pushw(rword());
        (*depth)++;
    } else if (r < 45) {
        b1(un[rn(sizeof un)]);
    } else if (r < 68) {
        u8 b = bin[rn(sizeof bin)];
        if (b == 0x1F || b == 0x20) {
            /* The oracle has no gwtool division lowering: native idiv traps on x/0 and
             * INT_MIN/-1, so the divisor is a pushed constant that is neither. */
            s32 d = (s32) rn(2001) - 1000;
            if (d == 0 || d == -1) d = 7;
            if (rn(8) == 0) d = (s32) (nx() >> 32) | 1;
            if (d == -1) d = 3;
            pushw((u32) d); b1(b);
        } else if (*depth >= 2) { b1(b); (*depth)--; }
    } else if (r < 73) {
        b1(0x3C); b1(rn(*depth)); (*depth)++;
    } else if (r < 78) {
        u32 k = rn(*depth + 1); if (k == *depth && k > 0) k--; b1(5); b1(k); *depth -= k;
    } else if (r < 83) {
        int sk = rn(3); /* safe expf / powf / fmodf */
        if (sk == 0) { pushf(rf(-10.f, 10.f)); b1(0x14); }
        else if (sk == 1) { pushf(rn(4) ? rf(0.1f, 10.f) : (rn(2) ? 1.f : rf(-3.f, 0.f))); pushf(rn(2) ? rf(0.05f, 3.f) : rf(-3.f, -0.05f)); b1(0x21); }
        else { pushf(rf(-500.f, 500.f)); pushf(rn(5) ? rf(0.1f, 50.f) : rf(-50.f, -0.1f)); b1(0x1B); }
        (*depth)++;
    } else if (r < 93 && budget > 2) {
        /* branch: cond; 3 <skip>; block(net 0) */
        int pos, d2;
        if (rn(2)) { pushw(rn(2) ? 0 : (u32) (nx() >> 40) + 1); (*depth)++; }
        b1(3); pos = n; b2(0); (*depth)--;
        d2 = *depth; block(&d2, budget / 2, 1);
        code[pos] = (u8) ((n - pos - 2) >> 8); code[pos + 1] = (u8) (n - pos - 2);
    } else if (budget > 2) {
        int pos, d2;
        b1(4); pos = n; b2(0);
        d2 = *depth; block(&d2, budget / 2, 1);
        code[pos] = (u8) ((n - pos - 2) >> 8); code[pos + 1] = (u8) (n - pos - 2);
    }
    if (*depth < 0) { fprintf(stderr, "depth\n"); exit(5); }
}
static void block(int* depth, int budget, int net0)
{
    int d0 = *depth, k = (int) rn(budget + 1);
    while (k-- > 0) op(depth, budget - 1);
    if (net0) {
        while (*depth < d0) { pushw(rword()); (*depth)++; }
        if (*depth > d0) { b1(5); b1(*depth - d0); *depth = d0; }
    }
}

int main(int argc, char** argv)
{
    FILE* o = fopen(argv[1], "w");
    int i, j;
    if (!o) return 1;
    for (i = 0; i < 3000; i++) {
        f32 x = i < 21 ? (f32) (i - 10) : (i < 41 ? (f32) (i - 30) * 0.001f : rf(-10.f, 10.f));
        fprintf(o, "E %08x %08x\n", fb(x), fb(expf(x)));
    }
    fflush(o); fprintf(stderr, "E done\n");
    for (i = 0; i < 3000; i++) {
        f32 x = rn(8) ? rf(0.1f, 10.f) : rf(-5.f, 0.f), y = rf(-4.f, 4.f);
        if (rn(20) == 0) x = 0.f;
        if (rn(20) == 0) y = 0.f;
        if (y == 0.f && x != 0.f && x != 1.f) y = 1.f; /* expf(0 * inf) never ends */
        fprintf(o, "P %08x %08x %08x\n", fb(x), fb(y), fb(powf(x, y)));
    }
    for (i = 0; i < 2000; i++) {
        s32 b = (s32) rn(21) - 10, e = (s32) rn(40) - 5;
        if (rn(10) == 0) b = (s32) (nx() >> 32);
        fprintf(o, "I %08x %08x %08x\n", (u32) b, (u32) e, (u32) powi(b, e));
    }
    for (i = 0; i < 20000; i++) {
        f32 args[8];
        int depth = 0;
        u32 seed = (u32) (nx() >> 32), res;
        n = 0;
        nargs = (int) rn(9);
        for (j = 0; j < nargs; j++) { u32 w = rword(); memcpy(&args[j], &w, 4); }
        block(&depth, 4 + (int) rn(12), 0);
        if (depth == 0) { pushw(rword()); depth++; }
        b1(1);
        *HSD_RandSeedPtr = seed; if (i % 1000 == 0) { fflush(o); fprintf(stderr, "B %d\n", i); }
        if (getenv("BC_DEBUG")) { fprintf(stderr, "prog %d: ", i); for (j = 0; j < n; j++) fprintf(stderr, "%02x", code[j]); fprintf(stderr, "\n"); }
        res = fb(HSD_ByteCodeEval(code, args, nargs));
        fprintf(o, "B %08x ", seed);
        for (j = 0; j < n; j++) fprintf(o, "%02x", code[j]);
        fprintf(o, " %d", nargs);
        for (j = 0; j < nargs; j++) fprintf(o, " %08x", fb(args[j]));
        fprintf(o, " %08x %08x\n", res, *HSD_RandSeedPtr);
    }
    fclose(o);
    return 0;
}
