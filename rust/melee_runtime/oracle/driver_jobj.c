/* Oracle driver for baselib jobj.c (see build_jobj.sh). Generated inputs only.
 * Builds 24 JObjs (0..19 tree nodes, 20..23 "anchors" that only serve as AObj targets), 16 RObjs,
 * 16 DObjs and a random HSD_Joint forest, then applies random operations. Every operation is
 * written as one hex row with its arguments, its result and an FNV-1a hash of the full state
 * after it, so tests/oracle_jobj.rs replays the exact same operations on the Rust port.
 *
 * File format (all tokens hex):
 *   J k child next v0..v8        one row per joint (child/next = index or FFFFFFFF)
 *   O seq op a b c f0..f9 result hash
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
typedef struct T T;

J* orc_jobj_new(void);
J* orc_next(J*);
J* orc_parent(J*);
J* orc_child(J*);
u32 orc_flags(J*);
R* orc_robj(J*);
D* orc_dobj_raw(J*);
void orc_dump(J*, u32*);
void orc_set_srt(J*, const f32*);
void orc_set_aobj(J*, J*);
R* orc_robj_new(u32, f32);
R* orc_robj_next(R*);
D* orc_dobj_new(void);
D* orc_dobj_next(D*);
T* orc_joint_new(T*, T*, const f32*);
void orc_setup(J*);
void orc_set_dirty(J*);
int orc_is_dirty(J*);

void HSD_JObjCheckDepend(J*);
void JObjResetRST(J*, T*);
void HSD_JObjResetRST(J*, T*);
void HSD_JObjWalkTree(J*, void (*)(J*, f32**, int), f32**);
void HSD_JObjMakeMatrix(J*);
void HSD_JObjAddChild(J*, J*);
J* HSD_JObjReparent(J*, J*);
void HSD_JObjAddNext(J*, J*);
J* HSD_JObjGetPrev(J*);
J* HSD_JObjRemove(J*);
void HSD_JObjRemoveAll(J*);
D* HSD_JObjGetDObj(J*);
void HSD_JObjAddDObj(J*, D*);
void HSD_JObjPrependRObj(J*, R*);
void HSD_JObjDeleteRObj(J*, R*);
u32 HSD_JObjGetFlags(J*);
void HSD_JObjSetFlags(J*, u32);
void HSD_JObjSetFlagsAll(J*, u32);
void HSD_JObjClearFlags(J*, u32);
void HSD_JObjClearFlagsAll(J*, u32);
void RecalcParentTrspBits(J*);

#define NJ 24
#define NT 20 /* tree nodes */
#define NR 16
#define ND 16
#define NJT 48
#define NONE 0xFFFFFFFFu

static J* jo[NJ];
static R* ro[NR];
static D* dob[ND];
static T* jt[NJT];

static uint64_t st = 0x9E3779B97F4A7C15ull;
static uint64_t nx(void) { st ^= st << 13; st ^= st >> 7; st ^= st << 17; return st; }
static u32 rn(u32 n) { return (u32) (nx() >> 33) % n; }
static f32 rf(f32 r) { return (f32) ((double) (nx() >> 11) / 9007199254740992.0 * 2.0 - 1.0) * r; }
static u32 fb(f32 f) { u32 u; memcpy(&u, &f, 4); return u; }

static u32 ji(J* j) { int i; if (!j) return NONE; for (i = 0; i < NJ; i++) if (jo[i] == j) return i; fprintf(stderr, "bad jobj\n"); exit(5); }
static u32 ri(R* r) { int i; if (!r) return NONE; for (i = 0; i < NR; i++) if (ro[i] == r) return i; fprintf(stderr, "bad robj\n"); exit(5); }
static u32 di(D* d) { int i; if (!d) return NONE; for (i = 0; i < ND; i++) if (dob[i] == d) return i; fprintf(stderr, "bad dobj\n"); exit(5); }

static u32 h;
static void hw(u32 w) { int i; for (i = 0; i < 4; i++) { h ^= (w >> (8 * i)) & 0xFF; h *= 16777619u; } }

/* RObjUpdateAll stub log (helper_jobj.c) and WalkTree callback log go into the hash. */
int orc_update_log(J* j) { hw(0xABCD0000u); hw(ji(j)); return 0; }
static void walk_cb(J* j, f32** a, int type) { (void) a; hw(0xC0DE0000u); hw(ji(j)); hw((u32) type); }

static void state_hash(void)
{
    int i, k;
    u32 w[26];
    for (i = 0; i < NJ; i++) {
        J* j = jo[i];
        R* r;
        D* d;
        int guard;
        hw(ji(orc_next(j)));
        hw(ji(orc_parent(j)));
        hw(ji(orc_child(j)));
        hw(orc_flags(j));
        orc_dump(j, w);
        for (k = 0; k < 26; k++) hw(w[k]);
        for (r = orc_robj(j), guard = 0; r && guard < 64; r = orc_robj_next(r), guard++) hw(ri(r));
        hw(NONE);
        for (d = orc_dobj_raw(j), guard = 0; d && guard < 64; d = orc_dobj_next(d), guard++) hw(di(d));
        hw(NONE);
    }
}

static int in_subtree(J* x, J* root) { for (; x; x = orc_parent(x)) if (x == root) return 1; return 0; }
static J* root_of(J* x) { while (orc_parent(x)) x = orc_parent(x); return x; }
static int robj_free(R* r)
{
    int i;
    for (i = 0; i < NJ; i++) { R* c; for (c = orc_robj(jo[i]); c; c = orc_robj_next(c)) if (c == r) return 0; }
    return orc_robj_next(r) == NULL;
}
static int dobj_free(D* d)
{
    int i;
    for (i = 0; i < NJ; i++) { D* c; for (c = orc_dobj_raw(jo[i]); c; c = orc_dobj_next(c)) if (c == d) return 0; }
    return 1;
}
static J* pick(int null_ok, int n) { if (null_ok && rn(16) == 0) return NULL; return jo[rn(n)]; }
static u32 rmask(void)
{
    static const int bits[] = { 3, 4, 5, 6, 7, 12, 14, 17, 18, 19, 20, 23, 24, 25, 26, 28, 29, 30 };
    u32 m = 0;
    int n = 1 + rn(3), i;
    for (i = 0; i < n; i++) m |= 1u << bits[rn(18)];
    if (rn(6) == 0) m |= 3u << 21; /* the joint field only ever becomes 0 or EFFECTOR */
    if (rn(12) == 0) m |= 1u << 12;
    return m;
}

int main(int argc, char** argv)
{
    FILE* o = fopen(argv[1], "w");
    int i, seq = 0, round;
    u32 refd[NJT] = { 0 };
    if (!o) return 1;
    /* joint forest: each joint is referenced at most once, always from a lower index */
    for (i = NJT - 1; i >= 0; i--) {
        f32 v[9];
        int k, c = -1, n = -1;
        for (k = 0; k < 9; k++) v[k] = k >= 3 && k < 6 ? rf(2.f) + (rf(1.f) > 0 ? 2.5f : -2.5f) : rf(k < 3 ? 4.f : 40.f);
        if (i + 1 < NJT && rn(3)) { int t = i + 1 + rn(NJT - i - 1); if (!refd[t]) { c = t; refd[t] = 1; } }
        if (i + 1 < NJT && rn(2)) { int t = i + 1 + rn(NJT - i - 1); if (!refd[t]) { n = t; refd[t] = 1; } }
        jt[i] = orc_joint_new(c >= 0 ? jt[c] : NULL, n >= 0 ? jt[n] : NULL, v);
        fprintf(o, "J %x %x %x", i, c >= 0 ? (u32) c : NONE, n >= 0 ? (u32) n : NONE);
        for (k = 0; k < 9; k++) fprintf(o, " %08x", fb(v[k]));
        fprintf(o, "\n");
    }
    for (i = 0; i < NJ; i++) jo[i] = orc_jobj_new();
    for (i = 0; i < NR; i++) {
        u32 fl = (i & 1) ? 0x40000000u : 0x10000000u;
        f32 bl = rf(20.f);
        ro[i] = orc_robj_new(fl, bl);
        fprintf(o, "R %x %08x %08x\n", i, fl, fb(bl));
    }
    for (i = 0; i < ND; i++) dob[i] = orc_dobj_new();

    for (round = 0; round < 60; round++) {
        int step;
        if (round) {
            /* new forest: detach everything */
            for (i = 0; i < NT; i++) {
                fprintf(o, "O %x 5 %x %x %x", seq++, i, NONE, NONE);
                h = 2166136261u;
                HSD_JObjRemoveAll(jo[i]);
                state_hash();
                fprintf(o, " 0 %08x\n", h);
            }
        }
        for (step = 0; step < 400; step++) {
            u32 op = rn(26), a = NONE, b = NONE, c = NONE, res = 0;
            f32 v[10] = { 0 };
            J *ja, *jb;
            h = 2166136261u;
            switch (op) {
            case 0: /* AddChild */
                ja = pick(1, NT); jb = pick(1, NT);
                if (ja && jb && (orc_parent(jb) || orc_next(jb) || ja == jb || in_subtree(ja, jb) ||
                                 (orc_child(ja) && (orc_flags(ja) & 0x1000))))
                    continue;
                a = ji(ja); b = ji(jb);
                HSD_JObjAddChild(ja, jb);
                break;
            case 1: /* Reparent */
                ja = pick(1, NT); jb = pick(1, NT);
                if (ja && jb && (ja == jb || in_subtree(jb, ja) || (orc_child(jb) && (orc_flags(jb) & 0x1000))))
                    continue;
                a = ji(ja); b = ji(jb);
                res = ji(HSD_JObjReparent(ja, jb));
                break;
            case 2: /* AddNext */
                ja = pick(1, NT); jb = pick(1, NT);
                if (ja && jb) {
                    J* p = orc_parent(ja);
                    if (root_of(ja) == root_of(jb)) continue;
                    if (orc_child(jb) && (orc_flags(jb) & 0x1000)) continue;
                    (void) p;
                }
                a = ji(ja); b = ji(jb);
                HSD_JObjAddNext(ja, jb);
                break;
            case 3:
                ja = pick(1, NT); a = ji(ja);
                res = ji(HSD_JObjGetPrev(ja));
                break;
            case 4: /* Remove */
                ja = pick(1, NT);
                if (ja && orc_child(ja) && orc_next(orc_child(ja))) continue;
                a = ji(ja);
                res = ji(HSD_JObjRemove(ja));
                break;
            case 5:
                if (rn(4)) continue;
                ja = pick(1, NT); a = ji(ja);
                HSD_JObjRemoveAll(ja);
                break;
            case 6: case 7: case 8: case 9:
                ja = pick(1, NJ); a = ji(ja); b = rmask();
                if (op == 6) HSD_JObjSetFlags(ja, b);
                else if (op == 7) HSD_JObjClearFlags(ja, b);
                else if (op == 8) HSD_JObjSetFlagsAll(ja, b);
                else HSD_JObjClearFlagsAll(ja, b);
                break;
            case 10:
                ja = pick(1, NJ); a = ji(ja);
                orc_set_dirty(ja);
                break;
            case 11:
                ja = pick(1, NJ); a = ji(ja);
                HSD_JObjCheckDepend(ja);
                break;
            case 12: case 13:
                ja = pick(1, NJ); a = ji(ja);
                orc_setup(ja);
                break;
            case 14:
                ja = pick(0, NJ); a = ji(ja);
                HSD_JObjMakeMatrix(ja);
                break;
            case 15: case 16: case 17:
                ja = pick(0, NJ); a = ji(ja);
                for (i = 0; i < 10; i++) {
                    f32 r = i < 3 ? 4.f : i == 3 ? 1.f : i < 7 ? 2.f : 50.f;
                    v[i] = rf(r);
                    if (i >= 4 && i < 7) v[i] += v[i] < 0 ? -0.25f : 0.25f;
                }
                orc_set_srt(ja, v);
                break;
            case 18: case 19:
                ja = pick(1, NT); b = rn(NJT); a = ji(ja);
                if (op == 18) JObjResetRST(ja, jt[b]);
                else HSD_JObjResetRST(ja, jt[b]);
                break;
            case 20:
                ja = pick(1, NJ); a = ji(ja);
                HSD_JObjWalkTree(ja, walk_cb, NULL);
                break;
            case 21: {
                R* r = rn(16) ? ro[rn(NR)] : NULL;
                ja = pick(1, NJ);
                if (r && !robj_free(r)) continue;
                a = ji(ja); b = ri(r);
                HSD_JObjPrependRObj(ja, r);
                break;
            }
            case 22: {
                R* r = rn(16) ? ro[rn(NR)] : NULL;
                ja = pick(1, NJ); a = ji(ja); b = ri(r);
                HSD_JObjDeleteRObj(ja, r);
                break;
            }
            case 23: {
                D* d = rn(16) ? dob[rn(ND)] : NULL;
                ja = pick(1, NJ);
                if (d && !dobj_free(d)) continue;
                a = ji(ja); b = di(d);
                HSD_JObjAddDObj(ja, d);
                if (rn(2) == 0) { c = 1; res = di(HSD_JObjGetDObj(ja)); }
                break;
            }
            case 24: /* AObj whose hsd_obj is an anchor (or NULL) */
                ja = pick(0, NT); a = ji(ja);
                jb = rn(5) ? jo[NT + rn(NJ - NT)] : NULL; b = ji(jb);
                orc_set_aobj(ja, jb);
                break;
            default:
                ja = pick(1, NJ); a = ji(ja);
                res = HSD_JObjGetFlags(ja) ^ (ja ? (u32) orc_is_dirty(ja) << 31 : 0);
                if (rn(2)) { c = 1; RecalcParentTrspBits(ja); }
                break;
            }
            state_hash();
            fprintf(o, "O %x %x %x %x %x", seq++, op, a, b, c);
            for (i = 0; i < 10; i++) fprintf(o, " %08x", fb(v[i]));
            fprintf(o, " %x %08x\n", res, h);
        }
    }
    fclose(o);
    return 0;
}
