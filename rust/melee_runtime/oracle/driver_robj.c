/* Oracle driver for baselib robj.c (see build_robj.sh). Generated inputs only.
 * Each scenario builds a JObj forest and an RObj chain through HSD_RObjLoadDesc +
 * HSD_RObjResolveRefsAll (+ some HSD_RObjSetConstraintObj), then applies random operations.
 * The update callback hashes every call and writes the orientation / position vectors into the
 * object's matrix, so later reads of that matrix see them (as JObjUpdateFunc's would).
 *
 * File format (all tokens hex):
 *   S sc nj nr                                   scenario start
 *   J i parent flags f0..f9 hasScl s0 s1 s2     JObj (AddChild(parent, i) in index order)
 *   D k flags kind a b                           desc: kind 0 joint key a, 1 limit a=bits,
 *                                                2 exp a=func(0,1,2=NULL) b=nrv (NONE = NULL list),
 *                                                3 bytecode a=has code b=nrv, 4 ik a=bl b=rx
 *   V k i flags key                              rvalue list entry i of desc k
 *   L hash                                       state hash after LoadDesc + ResolveRefsAll
 *   C k j hash                                   SetConstraintObj(robj k, jobj j), state hash
 *   O seq op a b c f0..f9 result hash
 */
#include <stdio.h>
#include <string.h>
#include <stdint.h>
#include <stdlib.h>
typedef float f32;
typedef unsigned u32;
typedef unsigned char u8;
typedef struct J J;
typedef struct R R;
typedef struct D D;
typedef struct L L;
typedef struct V V;

J* orc_jobj_new(void);
u32 orc_flags(J*);
J* orc_parent(J*);
void orc_dump(J*, u32*);
void orc_set_srt(J*, const f32*);
void orc_set_scl(J*, const f32*);
f32* orc_mtx(J*);
void orc_set_dirty(J*);
D* orc_desc_new(u32, D*);
void orc_desc_joint(D*, u32);
void orc_desc_limit(D*, f32);
void orc_desc_ik(D*, f32, f32);
L* orc_rvlist(int);
void orc_rvlist_set(L*, int, u32, u32);
void orc_desc_exp(D*, f32 (*)(void*), L*);
void orc_desc_bc(D*, u8*, L*);
R* orc_robj_next(R*);
u32 orc_robj_flags(R*);
u32 orc_robj_limit(R*);
u32 orc_robj_ik(R*, int);
J* orc_robj_jobj(R*);
u32 orc_robj_nbargs(R*);
u32 orc_robj_isbc(R*);
V* orc_robj_rv(R*);
V* orc_rv_next(V*);
u32 orc_rv_flags(V*);
J* orc_rv_jobj(V*);

void HSD_JObjAddChild(J*, J*);
void HSD_JObjSetFlags(J*, u32);
void orc_setup(J*);
R* HSD_RObjLoadDesc(D*);
void HSD_RObjResolveRefsAll(R*, D*);
void HSD_RObjSetConstraintObj(R*, void*);
void HSD_RObjUpdateAll(R*, void*, void (*)(void*, int, void*));
int HSD_RObjGetGlobalPosition(R*, int, f32*);
R* HSD_RObjGetByType(R*, u32, u32);
void HSD_RObjSetFlags(R*, u32);

#define MAXJ 12
#define MAXR 10
#define NONE 0xFFFFFFFFu

static J* jo[MAXJ];
static int nj;
static R* ro[MAXR];
static int nr;
static R* chain;
static D* descs;

static uint64_t st = 0x2545F4914F6CDD1Dull;
static uint64_t nx(void) { st ^= st << 13; st ^= st >> 7; st ^= st << 17; return st; }
static u32 rn(u32 n) { return (u32) (nx() >> 33) % n; }
static f32 rf(f32 r) { return (f32) ((double) (nx() >> 11) / 9007199254740992.0 * 2.0 - 1.0) * r; }
/* NaNs are hashed as one canonical NaN: the sign / payload of a generated NaN is a host artifact
 * (x86 SSE gives 0xFFC00000, the Rust port's f64 fma path 0x7FC00000 in debug builds). */
static u32 cn(u32 u) { return (u & 0x7F800000u) == 0x7F800000u && (u & 0x007FFFFFu) ? 0x7FC00000u : u; }
static u32 fb(f32 f) { u32 u; memcpy(&u, &f, 4); return cn(u); }

static u32 h;
static void hw(u32 w) { int i; for (i = 0; i < 4; i++) { h ^= (w >> (8 * i)) & 0xFF; h *= 16777619u; } }
static u32 ji(J* j) { int i; if (!j) return NONE; for (i = 0; i < nj; i++) if (jo[i] == j) return i; fprintf(stderr, "bad jobj\n"); exit(5); }

/* expression functions, mirrored in tests/oracle_robj.rs */
static f32 fn0(void* p) { const f32* a = p; f32 s = 0; int i; for (i = 0; i < 8; i++) s = s * 0.5f + a[i]; return s; }
static f32 fn1(void* p) { const f32* a = p; return a[0] - a[1] * 0.25f; }
static u8 code[4] = { 1, 2, 3, 4 };
f32 HSD_ByteCodeEval(u8* c, f32* a, int n) { return (c ? a[0] : -a[1]) + (f32) n; }
void* HSD_IDGetDataFromTable(void* t, u32 key, int* ok)
{
    (void) t;
    if (ok) *ok = 1;
    return key >= 1 && key <= (u32) nj ? jo[key - 1] : NULL;
}

static int dbg = -1, cur_seq;
static void cb(void* obj, int type, void* val)
{
    f32* m = orc_mtx(obj);
    int isv = val && type >= 0x32 && type <= 0x35;
    if (cur_seq == dbg) /* debug: driver_robj.exe out.txt <seq> prints that op's callbacks */
        fprintf(stderr, "cb %x %x %08x %08x %08x\n", type, ji(obj), val ? fb(((f32*) val)[0]) : 0,
                isv ? fb(((f32*) val)[1]) : 0, isv ? fb(((f32*) val)[2]) : 0);
    hw(0xCB000000u | (u32) type);
    hw(ji(obj));
    if (!val) { hw(NONE); return; }
    if (type >= 0x32 && type <= 0x35) {
        const f32* v = val;
        int col = type == 0x35 ? 3 : type - 0x32;
        hw(fb(v[0])); hw(fb(v[1])); hw(fb(v[2]));
        m[0 * 4 + col] = v[0];
        m[1 * 4 + col] = v[1];
        m[2 * 4 + col] = v[2];
    } else {
        hw(fb(*(const f32*) val));
    }
}

static void state_hash(void)
{
    int i, k;
    u32 w[26];
    R* r;
    for (i = 0; i < nj; i++) {
        hw(orc_flags(jo[i]));
        orc_dump(jo[i], w);
        for (k = 0; k < 26; k++) hw(k == 22 ? w[k] : cn(w[k]));
    }
    for (r = chain; r; r = orc_robj_next(r)) {
        u32 f = orc_robj_flags(r);
        hw(f);
        switch (f & 0x70000000u) {
        case 0x10000000u: hw(ji(orc_robj_jobj(r))); break;
        case 0x20000000u: hw(cn(orc_robj_limit(r))); break;
        case 0x40000000u: hw(orc_robj_ik(r, 0)); hw(orc_robj_ik(r, 1)); break;
        case 0: {
            V* v;
            hw(orc_robj_nbargs(r));
            hw(orc_robj_isbc(r));
            for (v = orc_robj_rv(r); v; v = orc_rv_next(v)) { hw(orc_rv_flags(v)); hw(ji(orc_rv_jobj(v))); }
            hw(NONE);
        } break;
        }
    }
}

static u32 rv_flags(void)
{
    static const u32 bits[] = { 0x1, 0x2, 0x4, 0x8, 0x10, 0x20, 0x40, 0x80, 0x100, 0x200, 0x400, 0x800, 0x1000,
                                0x10000, 0x20000, 0x40000, 0x80000, 0x100000, 0x200000, 0x400000, 0x800000,
                                0x1000000, 0x2000000 };
    u32 m = 0;
    int n = 1 + rn(5), i;
    for (i = 0; i < n; i++) m |= bits[rn(23)];
    return m;
}

int main(int argc, char** argv)
{
    FILE* o = fopen(argv[1], "w");
    int sc, seq = 0;
    if (argc > 2) dbg = (int) strtol(argv[2], 0, 16);
    if (!o) return 1;
    for (sc = 0; sc < 400; sc++) {
        int i, k, step;
        D* prev = NULL;
        R* r;
        nj = 3 + rn(MAXJ - 2);
        nr = 1 + rn(MAXR);
        fprintf(o, "S %x %x %x\n", sc, nj, nr);
        for (i = 0; i < nj; i++) {
            f32 v[10], s[3];
            u32 par = i && rn(4) ? rn(i) : NONE, fl = rn(3) == 0 ? 8u : 0u;
            int has = rn(4) == 0;
            for (k = 0; k < 10; k++) v[k] = k >= 4 && k < 7 ? rf(1.5f) + (rf(1.f) > 0 ? 1.5f : -1.5f) : rf(k < 4 ? 3.f : 30.f);
            v[3] = 0; /* quaternion w unused (no USE_QUATERNION) */
            for (k = 0; k < 3; k++) s[k] = rf(2.f) + 1.0f;
            jo[i] = orc_jobj_new();
            orc_set_srt(jo[i], v);
            if (has) orc_set_scl(jo[i], s);
            if (fl) HSD_JObjSetFlags(jo[i], fl);
            if (par != NONE) HSD_JObjAddChild(jo[par], jo[i]);
            fprintf(o, "J %x %x %x", i, par, fl);
            for (k = 0; k < 10; k++) fprintf(o, " %08x", fb(v[k]));
            fprintf(o, " %x %08x %08x %08x\n", has, has ? fb(s[0]) : 0, has ? fb(s[1]) : 0, has ? fb(s[2]) : 0);
        }
        descs = NULL;
        for (k = 0; k < nr; k++) {
            u32 kind = rn(10), fl, a = 0, b = 0, act = rn(5) ? 0x80000000u : 0;
            D* d;
            if (kind < 4) kind = 0; else if (kind < 6) kind = 1; else if (kind < 8) kind = 2; else kind = kind == 8 ? 3 : 4;
            switch (kind) {
            case 0: fl = 0x10000000u | act | (1 + rn(5)); break;
            case 1: fl = 0x20000000u | rn(14); break;
            case 2: fl = act | rn(13); break;
            case 3: fl = 0x30000000u | act | rn(13); break;
            default: fl = 0x40000000u | act; break;
            }
            d = orc_desc_new(fl, prev);
            if (!descs) descs = d;
            prev = d;
            if (kind == 0) { a = 1 + rn(nj); orc_desc_joint(d, a); }
            else if (kind == 1) { f32 l = rf(rn(2) ? 3.f : 40.f); a = fb(l); orc_desc_limit(d, l); }
            else if (kind == 4) { f32 bl = rf(10.f), rx = rf(3.f); a = fb(bl); b = fb(rx); orc_desc_ik(d, bl, rx); }
            else {
                int n = rn(4);
                L* l = rn(8) ? orc_rvlist(n) : NULL;
                a = rn(3);
                b = l ? (u32) n : NONE;
                if (kind == 2) orc_desc_exp(d, a == 0 ? fn0 : a == 1 ? fn1 : NULL, l);
                else { a = rn(4) != 0; orc_desc_bc(d, a ? code : NULL, l); }
                fprintf(o, "D %x %08x %x %08x %08x\n", k, fl, kind, a, b);
                for (i = 0; l && i < n; i++) {
                    u32 f = rv_flags(), key = 1 + rn(nj);
                    orc_rvlist_set(l, i, f, key);
                    fprintf(o, "V %x %x %08x %x\n", k, i, f, key);
                }
                continue;
            }
            fprintf(o, "D %x %08x %x %08x %08x\n", k, fl, kind, a, b);
        }
        chain = HSD_RObjLoadDesc(descs);
        for (r = chain, k = 0; r; r = orc_robj_next(r)) ro[k++] = r;
        HSD_RObjResolveRefsAll(chain, descs);
        h = 2166136261u;
        state_hash();
        fprintf(o, "L %08x\n", h);
        for (k = 0; k < nr; k++) {
            if ((orc_robj_flags(ro[k]) & 0x70000000u) == 0x10000000u && rn(4) == 0) {
                u32 j = rn(nj);
                HSD_RObjSetConstraintObj(ro[k], jo[j]);
                h = 2166136261u;
                state_hash();
                fprintf(o, "C %x %x %08x\n", k, j, h);
            }
        }
        for (step = 0; step < 40; step++) {
            u32 op = rn(10), a = 0, b = 0, c = 0, res = 0;
            f32 v[10] = { 0 }, p[3];
            h = 2166136261u;
            cur_seq = seq;
            if (op < 4) { /* UpdateAll */
                op = 0;
                a = rn(nj);
                HSD_RObjUpdateAll(chain, jo[a], cb);
            } else if (op < 6) {
                op = 1;
                a = rn(6);
                p[0] = p[1] = p[2] = 0;
                res = (u32) HSD_RObjGetGlobalPosition(chain, (int) a, p);
                hw(fb(p[0])); hw(fb(p[1])); hw(fb(p[2]));
            } else if (op == 6) {
                R* g;
                static const u32 ty[] = { 0, 0x10000000u, 0x20000000u, 0x30000000u, 0x40000000u };
                op = 2;
                a = ty[rn(5)];
                b = rn(3) ? 0 : rn(6);
                g = HSD_RObjGetByType(chain, a, b);
                for (res = NONE, k = 0; k < nr; k++) if (ro[k] == g) res = k;
            } else if (op == 7) {
                op = 3;
                a = rn(nj);
                for (k = 0; k < 10; k++) v[k] = k >= 4 && k < 7 ? rf(1.5f) + 1.5f : rf(k < 4 ? 3.f : 30.f);
                v[3] = 0;
                orc_set_srt(jo[a], v);
                orc_set_dirty(jo[a]);
            } else if (op == 8) {
                op = 4;
                a = rn(nr);
                b = rn(2) ? 0x80000000u : rn(8);
                HSD_RObjSetFlags(ro[a], b);
            } else {
                op = 5;
                a = rn(nj);
                orc_setup(jo[a]);
            }
            state_hash();
            fprintf(o, "O %x %x %x %x %x", seq++, op, a, b, c);
            for (k = 0; k < 10; k++) fprintf(o, " %08x", fb(v[k]));
            fprintf(o, " %x %08x\n", res, h);
        }
    }
    fclose(o);
    return 0;
}
