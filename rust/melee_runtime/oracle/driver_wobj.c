/* Oracle driver for baselib wobj.c and the dobj.c flag functions (see build_wobj.sh).
 * Generated inputs only. Per scenario: a 3-JObj chain (root, child, grandchild) with random SRT,
 * a random path spline on each JObj, one WObj (no AObj / AObj with no target / AObj targeting
 * JObj 1 or 2) and a DObj, then random operations. All tokens hex:
 *   S s mode tgt
 *   J k v0..v9                      SRT of JObj k
 *   P k type numcv tension total ncv cv.. nseg seg.. npoly poly..
 *   D flags                         seed flags of the DObj
 *   R k v0..v9                      new SRT of JObj k (+ SetMtxDirty), before its O row
 *   O s i op a b c d r0 r1 r2 wflags px py pz dflags jhash
 * jhash: FNV-1a over the 3 JObjs' flags and mtx words (little-endian bytes).
 */
#include <stdio.h>
#include <string.h>
#include <stdint.h>
#include <stdlib.h>
typedef float f32;
typedef unsigned u32;
typedef struct J J;
typedef struct W W;
typedef struct D D;

J* orc_jobj_new(void);
u32 orc_flags(J*);
void orc_set_srt(J*, const f32*);
void orc_set_dirty(J*);
D* orc_dobj_new(void);
void HSD_JObjAddChild(J*, J*);
W* orc_wobj_new(int, J*);
void orc_wobj_update(W*, int, u32);
void orc_wobj_set_position(W*, const f32*);
void orc_wobj_get_position(W*, f32*);
void orc_wobj_clear_flags(W*, u32);
void orc_wobj_dump(W*, u32*);
void orc_jobj_set_spline(J*, int, int, f32, void*, f32, f32*, void*);
const f32* orc_jobj_mtx(J*);
u32* orc_dobj_flags(D*);
void HSD_WObjSetPositionX(W*, f32);
void HSD_WObjSetPositionY(W*, f32);
void HSD_WObjSetPositionZ(W*, f32);
u32 HSD_DObjGetFlags(D*);
void HSD_DObjSetFlags(D*, u32);
void HSD_DObjClearFlags(D*, u32);
void HSD_DObjModifyFlags(D*, u32, u32);
int orc_update_log(J* j) { (void) j; return 0; }

static uint64_t st = 0xD1B54A32D192ED03ull;
static uint64_t nx(void) { st ^= st << 13; st ^= st >> 7; st ^= st << 17; return st; }
static u32 rn(u32 n) { return (u32) (nx() >> 33) % n; }
static f32 rf(f32 r) { return (f32) ((double) (nx() >> 11) / 9007199254740992.0 * 2.0 - 1.0) * r; }
static f32 ru(void) { return (f32) ((double) (nx() >> 11) / 9007199254740992.0); }
static u32 fb(f32 f) { u32 u; memcpy(&u, &f, 4); return u; }
static f32 bf(u32 u) { f32 f; memcpy(&f, &u, 4); return f; }

static u32 fnv(u32 h, u32 w)
{
    int i;
    for (i = 0; i < 4; i++) {
        h ^= (w >> (8 * i)) & 0xFF;
        h *= 16777619u;
    }
    return h;
}

static u32 rmask(void)
{
    switch (rn(4)) {
    case 0: return 1;
    case 1: return 2;
    case 2: return rn(4);
    default: return (u32) nx();
    }
}

static void rsrt(f32* v)
{
    int t;
    for (t = 0; t < 4; t++) v[t] = rf(1.0F);
    for (t = 4; t < 7; t++) v[t] = 0.25F + ru() * 2.0F;
    for (t = 7; t < 10; t++) v[t] = rf(50.0F);
}

int main(int argc, char** argv)
{
    FILE* o = fopen(argv[1], "w");
    int s, i, k, n, t;
    (void) argc;
    for (s = 0; s < 150; s++) {
        J* jo[3];
        W* w;
        D* d;
        int mode, tgt;
        for (k = 0; k < 3; k++) {
            jo[k] = orc_jobj_new();
        }
        HSD_JObjAddChild(jo[0], jo[1]);
        HSD_JObjAddChild(jo[1], jo[2]);
        n = rn(6);
        mode = n == 0 ? 0 : n == 1 ? 1 : 2;
        tgt = 1 + rn(2);
        fprintf(o, "S %x %x %x\n", s, mode, tgt);
        for (k = 0; k < 3; k++) {
            f32 v[10];
            rsrt(v);
            orc_set_srt(jo[k], v);
            fprintf(o, "J %x", k);
            for (t = 0; t < 10; t++) fprintf(o, " %x", fb(v[t]));
            fprintf(o, "\n");
        }
        for (k = 0; k < 3; k++) {
            int type = rn(4), numcv = 2 + rn(5), ncv = numcv * 3 + 4;
            f32 tension = rf(1.0F), total = 1.0F + ru() * 20.0F;
            f32* cv = calloc(ncv * 3, 4);
            f32* seg = calloc(numcv, 4);
            f32* poly = calloc(numcv * 5, 4);
            for (t = 0; t < ncv * 3; t++) cv[t] = rf(30.0F);
            seg[0] = 0.0F;
            for (t = 1; t < numcv - 1; t++) seg[t] = seg[t - 1] + (1.0F - seg[t - 1]) * (0.1F + ru() * 0.6F);
            seg[numcv - 1] = 1.0F;
            for (t = 0; t < (numcv - 1) * 5; t++) poly[t] = ru() * 4.0F + (t % 5 == 4 ? 0.1F : 0.0F);
            orc_jobj_set_spline(jo[k], type, numcv, tension, cv, total, seg, poly);
            fprintf(o, "P %x %x %x %x %x %x", k, type, numcv, fb(tension), fb(total), ncv * 3);
            for (t = 0; t < ncv * 3; t++) fprintf(o, " %x", fb(cv[t]));
            fprintf(o, " %x", numcv);
            for (t = 0; t < numcv; t++) fprintf(o, " %x", fb(seg[t]));
            fprintf(o, " %x", (numcv - 1) * 5);
            for (t = 0; t < (numcv - 1) * 5; t++) fprintf(o, " %x", fb(poly[t]));
            fprintf(o, "\n");
        }
        w = orc_wobj_new(mode, jo[tgt]);
        d = orc_dobj_new();
        *orc_dobj_flags(d) = (u32) nx();
        fprintf(o, "D %x\n", *orc_dobj_flags(d));
        for (i = 0; i < 100; i++) {
            u32 op = rn(10), a = 0, b = 0, c = 0, dd = 0, r[3] = { 0, 0, 0 }, sd[4];
            f32 v[10];
            switch (op) {
            case 0:
                a = fb(rf(40.0F)); b = fb(rf(40.0F)); c = fb(rf(40.0F));
                v[0] = bf(a); v[1] = bf(b); v[2] = bf(c);
                orc_wobj_set_position(w, v);
                break;
            case 1: a = fb(rf(40.0F)); HSD_WObjSetPositionX(w, bf(a)); break;
            case 2: a = fb(rf(40.0F)); HSD_WObjSetPositionY(w, bf(a)); break;
            case 3: a = fb(rf(40.0F)); HSD_WObjSetPositionZ(w, bf(a)); break;
            case 4:
                orc_wobj_get_position(w, v);
                r[0] = fb(v[0]); r[1] = fb(v[1]); r[2] = fb(v[2]);
                break;
            case 5: a = rmask(); orc_wobj_clear_flags(w, a); break;
            case 6:
            case 7:
                a = rn(9);
                if (a == 4 && mode != 2) a = 5;
                if (op == 7 && mode == 2) a = 4; /* path updates are common */
                n = rn(8);
                b = n == 0 ? fb(rf(1.0F) - 0.5F)
                  : n == 1 ? fb(1.0F + ru())
                  : n == 2 ? (rn(2) ? 0u : 0x3F800000u)
                  : n == 3 ? fb(rf(40.0F))
                  : fb(ru());
                op = 6;
                orc_wobj_update(w, a, b);
                break;
            case 8:
                a = rn(3);
                rsrt(v);
                orc_set_srt(jo[a], v);
                orc_set_dirty(jo[a]);
                fprintf(o, "R %x", a);
                for (t = 0; t < 10; t++) fprintf(o, " %x", fb(v[t]));
                fprintf(o, "\n");
                break;
            default:
                a = rn(4); b = (u32) nx(); c = (u32) nx();
                switch (a) {
                case 0: r[0] = HSD_DObjGetFlags(d); r[1] = HSD_DObjGetFlags(NULL); break;
                case 1: HSD_DObjSetFlags(d, b); HSD_DObjSetFlags(NULL, b); break;
                case 2: HSD_DObjClearFlags(d, b); HSD_DObjClearFlags(NULL, b); break;
                default: HSD_DObjModifyFlags(d, b, c); HSD_DObjModifyFlags(NULL, b, c); break;
                }
                break;
            }
            orc_wobj_dump(w, sd);
            fprintf(o, "O %x %x %x %x %x %x %x %x %x %x %x %x %x %x %x", s, i, op, a, b, c, dd, r[0], r[1], r[2],
                    sd[0], sd[1], sd[2], sd[3], *orc_dobj_flags(d));
            {
                u32 h = 2166136261u;
                for (k = 0; k < 3; k++) {
                    const f32* m = orc_jobj_mtx(jo[k]);
                    h = fnv(h, orc_flags(jo[k]));
                    for (t = 0; t < 12; t++) h = fnv(h, fb(m[t]));
                }
                fprintf(o, " %x\n", h);
            }
        }
    }
    fclose(o);
    return 0;
}
