/* Oracle driver for jobj.c's IK solvers (resolveIKJoint1/2 through HSD_JObjSetupMatrix), the
 * effector branch, HSD_RObjUpdateAll with the real JObjUpdateFunc, and JObjUpdateFunc itself
 * (see build_ik.sh). Generated inputs only.
 *
 * A scenario is a JObj forest: anchors (roots, no RObj; constraint targets), plain JObjs (some
 * with position / direction / orientation / limit constraints on anchors), and IK chains
 * JOINT1 [-> JOINT2] -> EFFECTOR with IK hints, an optional pole (JObj constraint subtype 3) on
 * JOINT1, optional limits 5 / 6 on JOINT2 and an effector target (subtype 1; sometimes inactive
 * or J1's own parent, which gives the zero-length path).
 *
 * File format (all tokens hex):
 *   S sc nj
 *   J i parent flags r0 r1 r2 r3 s0 s1 s2 t0 t1 t2 hasScl c0 c1 c2
 *   R i flags kind a        RObj desc of JObj i, chain order: kind 0 target JObj a,
 *                           1 limit a = desc bits, 4 ik hint a = bone length bits, b = rotate_x
 *   L hash                  state hash after building
 *   O seq op a b f0..f9 res hash
 *     op 0 SetupMatrix(a); 1 SRT := f0..f9 + SetMtxDirty(a); 2 JObjUpdateFunc(a, b, f0..f2),
 *     res = word 0 of the value after; 3 SetMtxDirty(a)
 */
#include <stdio.h>
#include <string.h>
#include <stdint.h>
#include <stdlib.h>
typedef float f32;
typedef unsigned u32;
typedef struct J J;
typedef struct R R;
typedef struct D D;

J* orc_jobj_new(void);
u32 orc_flags(J*);
void orc_dump(J*, u32*);
void orc_set_srt(J*, const f32*);
void orc_set_scl(J*, const f32*);
void orc_set_dirty(J*);
D* orc_desc_new(u32, D*);
void orc_desc_joint(D*, u32);
void orc_desc_limit(D*, f32);
void orc_desc_ik(D*, f32, f32);
R* orc_robj_next(R*);
u32 orc_robj_flags(R*);
u32 orc_robj_limit(R*);
u32 orc_robj_ik(R*, int);
J* orc_robj_jobj(R*);
void orc_set_robj(J*, R*);
R* orc_get_robj(J*);
void orc_update(J*, int, u32*);
void HSD_JObjAddChild(J*, J*);
void HSD_JObjSetFlags(J*, u32);
void orc_setup(J*);
R* HSD_RObjLoadDesc(D*);
void HSD_RObjResolveRefsAll(R*, D*);
void HSD_JObjSetDPtclCallback(void (*)(int, int, int, J*));

#define MAXJ 40
#define NONE 0xFFFFFFFFu
static J* jo[MAXJ];
static int nj;

static uint64_t st = 0x9E3779B97F4A7C15ull;
static uint64_t nx(void) { st ^= st << 13; st ^= st >> 7; st ^= st << 17; return st; }
static u32 rn(u32 n) { return (u32) (nx() >> 33) % n; }
static f32 rf(f32 r) { return (f32) ((double) (nx() >> 11) / 9007199254740992.0 * 2.0 - 1.0) * r; }
static u32 cn(u32 u) { return (u & 0x7F800000u) == 0x7F800000u && (u & 0x007FFFFFu) ? 0x7FC00000u : u; }
static u32 fb(f32 f) { u32 u; memcpy(&u, &f, 4); return u; }
static u32 h;
static void hw(u32 w) { int i; for (i = 0; i < 4; i++) { h ^= (w >> (8 * i)) & 0xFF; h *= 16777619u; } }
static u32 ji(J* j) { int i; if (!j) return NONE; for (i = 0; i < nj; i++) if (jo[i] == j) return i; fprintf(stderr, "bad jobj\n"); exit(5); }

f32 HSD_ByteCodeEval(void* c, f32* a, int n) { (void) c; (void) a; (void) n; fprintf(stderr, "unexpected bytecode\n"); exit(3); }
void* HSD_IDGetDataFromTable(void* t, u32 key, int* ok)
{
    (void) t;
    if (ok) *ok = 1;
    return key >= 1 && key <= (u32) nj ? jo[key - 1] : NULL;
}
static void dptcl(int z, int lo, int hi, J* j) { hw(0xD0000000u | (u32) z); hw((u32) lo); hw((u32) hi); hw(ji(j)); }

static void state_hash(void)
{
    int i, k;
    u32 w[26];
    R* r;
    for (i = 0; i < nj; i++) {
        hw(orc_flags(jo[i]));
        orc_dump(jo[i], w);
        for (k = 0; k < 26; k++) hw(k == 22 ? w[k] : cn(w[k]));
        for (r = orc_get_robj(jo[i]); r; r = orc_robj_next(r)) {
            u32 f = orc_robj_flags(r);
            hw(f);
            switch (f & 0x70000000u) {
            case 0x10000000u: hw(ji(orc_robj_jobj(r))); break;
            case 0x20000000u: hw(cn(orc_robj_limit(r))); break;
            case 0x40000000u: hw(cn(orc_robj_ik(r, 0))); hw(cn(orc_robj_ik(r, 1))); break;
            }
        }
    }
}

static FILE* o;
static D* dfirst;
static D* dprev;
static void desc(int i, u32 flags, u32 kind, u32 a, u32 b)
{
    D* d = orc_desc_new(flags, dprev);
    if (!dfirst) dfirst = d;
    dprev = d;
    if (kind == 0) orc_desc_joint(d, a + 1); /* ID keys are 1-based */
    else if (kind == 1) { f32 l; memcpy(&l, &a, 4); orc_desc_limit(d, l); }
    else { f32 bl, rx; memcpy(&bl, &a, 4); memcpy(&rx, &b, 4); orc_desc_ik(d, bl, rx); }
    fprintf(o, "R %x %08x %x %08x %08x\n", i, flags, kind, a, b);
}
static void chain_done(int i)
{
    if (dfirst) {
        R* r = HSD_RObjLoadDesc(dfirst);
        HSD_RObjResolveRefsAll(r, dfirst);
        orc_set_robj(jo[i], r);
    }
    dfirst = dprev = NULL;
}

static int na;
static int new_jobj(u32 par, u32 fl, int big)
{
    f32 v[10], s[3];
    int k, i = nj, has = rn(4) == 0;
    for (k = 0; k < 10; k++)
        v[k] = k >= 4 && k < 7 ? rf(0.5f) + (rn(6) ? 1.0f : -1.0f) : rf(k < 4 ? 3.f : (big ? 20.f : 6.f));
    v[3] = 0;
    for (k = 0; k < 3; k++) s[k] = rf(0.5f) + 1.0f;
    if (rn(3) == 0) fl |= 8u;
    jo[i] = orc_jobj_new();
    nj++;
    orc_set_srt(jo[i], v);
    if (has) orc_set_scl(jo[i], s);
    if (fl) HSD_JObjSetFlags(jo[i], fl);
    if (par != NONE) HSD_JObjAddChild(jo[par], jo[i]);
    fprintf(o, "J %x %x %x", i, par, fl);
    for (k = 0; k < 10; k++) fprintf(o, " %08x", fb(v[k]));
    fprintf(o, " %x %08x %08x %08x\n", has, has ? fb(s[0]) : 0, has ? fb(s[1]) : 0, has ? fb(s[2]) : 0);
    return i;
}
static u32 anchor(void) { return rn(na); }
static u32 act(void) { return rn(8) ? 0x80000000u : 0; }
static u32 ik_flags(void) { return 0x80000000u | 0x40000000u | (rn(2) ? 4u : 0); /* inactive hints trip the C asserts */ }
static u32 rbits(f32 r) { return fb(rf(r)); }

int main(int argc, char** argv)
{
    int sc, seq = 0;
    o = fopen(argv[1], "w");
    if (!o) return 1;
    (void) argc;
    HSD_JObjSetDPtclCallback(dptcl);
    for (sc = 0; sc < 400; sc++) {
        int i, k, step, np, nc, plain0;
        nj = 0;
        na = 1 + rn(4);
        np = rn(5);
        nc = 1 + rn(3);
        fprintf(o, "S %x\n", sc);
        for (i = 0; i < na; i++) {
            new_jobj(i && rn(3) == 0 ? rn(i) : NONE, 0, 1);
            chain_done(i);
        }
        plain0 = nj;
        for (i = 0; i < np; i++) {
            int j = new_jobj(rn(3) ? rn(nj) : NONE, 0, 0);
            int nr = rn(3);
            for (k = 0; k < nr; k++) {
                if (rn(3)) desc(j, 0x10000000u | act() | (1 + rn(5)), 0, anchor(), 0);
                else desc(j, 0x20000000u | act() | rn(14), 1, rbits(rn(2) ? 3.f : 60.f), 0);
            }
            chain_done(j);
        }
        for (i = 0; i < nc; i++) {
            u32 par = rn(4) ? rn(nj) : NONE;
            int j1, j2 = -1, e, two = rn(10) < 7;
            j1 = new_jobj(par, 1u << 21, 0);
            desc(j1, ik_flags(), 4, fb(rf(4.f) + 5.f), rn(3) ? rbits(1.5f) : 0);
            if (rn(2)) desc(j1, 0x10000000u | act() | 3, 0, anchor(), 0);
            if (rn(3) == 0) desc(j1, 0x20000000u | act() | (1 + rn(13)), 1, rbits(60.f), 0); /* RObjUpdateAll + remake */
            chain_done(j1);
            if (rn(5) == 0) { new_jobj(j1, 0, 0); chain_done(nj - 1); } /* a plain sibling first */
            if (two) {
                j2 = new_jobj(j1, 2u << 21, 0);
                desc(j2, ik_flags(), 4, fb(rf(4.f) + 5.f), 0);
                if (rn(2)) desc(j2, 0x20000000u | (rn(6) ? 0x80000000u : 0) | 5, 1, rbits(120.f), 0);
                if (rn(2)) desc(j2, 0x20000000u | (rn(6) ? 0x80000000u : 0) | 6, 1, rbits(120.f), 0);
                chain_done(j2);
            }
            e = new_jobj(two ? (u32) j2 : (u32) j1, 3u << 21, 0);
            {
                u32 tgt = anchor();
                if (par != NONE && rn(6) == 0) tgt = par; /* zero-length path */
                desc(e, 0x10000000u | (rn(10) ? 0x80000000u : 0) | 1, 0, tgt, 0);
            }
            chain_done(e);
            if (rn(3) == 0) { new_jobj(e, 0, 0); chain_done(nj - 1); }
        }
        (void) plain0;
        h = 2166136261u;
        state_hash();
        fprintf(o, "L %08x\n", h);
        for (step = 0; step < 50; step++) {
            u32 op = rn(10), a = rn(nj), b = 0, res = 0, w[3] = { 0, 0, 0 };
            f32 v[10] = { 0 };
            h = 2166136261u;
            if (op < 4) {
                op = 0;
                orc_setup(jo[a]);
            } else if (op < 6) {
                op = 1;
                for (k = 0; k < 10; k++) v[k] = k >= 4 && k < 7 ? rf(0.5f) + 1.0f : rf(k < 4 ? 3.f : 8.f);
                v[3] = 0;
                orc_set_srt(jo[a], v);
                orc_set_dirty(jo[a]);
            } else if (op < 9) {
                static const u32 ty[] = { 0, 1, 1, 2, 3, 5, 6, 7, 8, 9, 10, 11, 12, 13, 20, 25, 30, 39, 0x28, 0x29,
                                          0x2A, 0x32, 0x33, 0x34, 0x35, 0x36, 0x36, 0x37, 0x38, 0x39, 0x3A };
                op = 2;
                b = ty[rn(sizeof ty / sizeof ty[0])];
                for (k = 0; k < 3; k++) {
                    u32 m = rn(8);
                    v[k] = m == 0 ? rf(2e-3f) : m == 1 ? (f32) rn(2) : rf(m < 4 ? 1.f : 20.f);
                }
                if (b == 0x28 || b == 0x29 || b == 0x2A) { u32 iv = (u32) nx(); memcpy(&v[0], &iv, 4); }
                for (k = 0; k < 3; k++) w[k] = fb(v[k]);
                orc_update(jo[a], (int) b, w);
                res = w[0];
            } else {
                op = 3;
                orc_set_dirty(jo[a]);
            }
            state_hash();
            fprintf(o, "O %x %x %x %x", seq++, op, a, b);
            for (k = 0; k < 10; k++) fprintf(o, " %08x", fb(v[k]));
            fprintf(o, " %x %08x\n", res, h);
        }
    }
    fclose(o);
    return 0;
}
