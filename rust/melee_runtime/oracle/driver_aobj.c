/* Oracle driver for baselib aobj.c (over fobj.c), id.c and list.c (see build_aobj.sh).
 * Generated inputs only; writes hex rows that tests/oracle_aobj.rs replays. */
#include <stdio.h>
#include <string.h>
#include <stdint.h>
#include <stdlib.h>
typedef float f32; typedef unsigned u32; typedef int s32; typedef short s16; typedef unsigned short u16;
typedef unsigned char u8;
typedef struct HSD_FObj {
    struct HSD_FObj* next; u8* ad; u8* ad_head; u32 length;
    u8 flags, op, op_intrp, obj_type, frac_value, frac_slope; u16 nb_pack; s16 startframe; u16 fterm;
    f32 time, p0, p1, d0, d1;
} HSD_FObj;
typedef struct HSD_FObjDesc { struct HSD_FObjDesc* next; u32 length; f32 startframe; u8 type, frac_value, frac_slope, dummy0; u8* ad; } HSD_FObjDesc;
typedef struct HSD_AObj { u32 flags; f32 curr_frame, rewind_frame, end_frame, framerate; HSD_FObj* fobj; void* hsd_obj; } HSD_AObj;
typedef struct HSD_AObjDesc { u32 flags; f32 end_frame; HSD_FObjDesc* fobjdesc; uintptr_t obj_id; } HSD_AObjDesc;
typedef union { f32 fv; s32 iv; f32 p[3]; } HSD_ObjData;
typedef void (*Upd)(void*, int, HSD_ObjData*);
typedef struct IDEntry { struct IDEntry* next; uintptr_t id; void* data; } IDEntry;
typedef struct { IDEntry* table[101]; } HSD_IDTable;
typedef struct SL { struct SL* next; void* data; } SL;

u32 HSD_AObjGetFlags(HSD_AObj*); void HSD_AObjSetFlags(HSD_AObj*, u32); void HSD_AObjClearFlags(HSD_AObj*, u32);
void HSD_AObjInitEndCallBack(void); void HSD_AObjInvokeCallBacks(void); void HSD_AObjReqAnim(HSD_AObj*, f32);
void HSD_AObjStopAnim(HSD_AObj*, void*, Upd); void HSD_AObjInterpretAnim(HSD_AObj*, void*, Upd);
HSD_AObj* HSD_AObjLoadDesc(HSD_AObjDesc*); void HSD_AObjRemove(HSD_AObj*); void HSD_AObjSetRate(HSD_AObj*, f32);
void HSD_AObjSetRewindFrame(HSD_AObj*, f32); void HSD_AObjSetEndFrame(HSD_AObj*, f32); void HSD_AObjSetCurrentFrame(HSD_AObj*, f32);
void HSD_AObjSetFObj(HSD_AObj*, HSD_FObj*); HSD_FObj* HSD_FObjLoadDesc(HSD_FObjDesc*);
void HSD_IDSetup(void); void HSD_IDInsertToTable(HSD_IDTable*, uintptr_t, void*); void HSD_IDRemoveByIDFromTable(HSD_IDTable*, uintptr_t);
void* HSD_IDGetDataFromTable(HSD_IDTable*, uintptr_t, s32*); void _HSD_IDForgetMemory(void*, void*);
SL* HSD_SListAllocAndAppend(SL*, void*); SL* HSD_SListAllocAndPrepend(SL*, void*); SL* HSD_SListRemove(SL*);

static uint64_t st = 0x2545F4914F6CDD1Dull;
static uint64_t nx(void) { st ^= st << 13; st ^= st >> 7; st ^= st << 17; return st; }
static u32 rn(u32 n) { return (u32) (nx() >> 33) % n; }
static u32 fb(f32 f) { u32 u; memcpy(&u, &f, 4); return u; }
static f32 rf(f32 r) { return (f32) ((double) (nx() >> 11) / 9007199254740992.0 * 2.0 - 1.0) * r; }
static FILE* o;
static int ncb; static u32 cbt[8192], cbv[8192];
static void cb(void* obj, int type, HSD_ObjData* d) { (void) obj; if (ncb < 8192) { cbt[ncb] = (u32) type; cbv[ncb] = (u32) d->iv; } ncb++; }

#define BUF 256
static int put_varint(u8* buf, int n, u32 v) { do { u8 b = v & 0x7F; v >>= 7; if (v) b |= 0x80; buf[n++] = b; } while (v); return n; }
static u8 rfrac(void)
{
    static const u8 kinds[] = { 0x00, 0x20, 0x40, 0x60, 0x80 };
    u8 k = kinds[rn(5)];
    if (k == 0) return 0;
    return k | (u8) rn(9);
}
static int put_val(u8* buf, int n, u8 frac)
{
    u8 k = frac & 0xE0;
    int sz = (k == 0x20 || k == 0x40) ? 2 : (k == 0x60 || k == 0x80) ? 1 : 0;
    if (frac == 0) {
        u32 b = fb(rf(10.f)); int i;
        for (i = 0; i < 4; i++) buf[n++] = (u8) (b >> (8 * i));
        return n;
    }
    while (sz--) buf[n++] = (u8) nx();
    return n;
}
static int gen_stream(u8* buf, u8 fv, u8 fs)
{
    int n = 0;
    memset(buf, 0, BUF);
    while (n < BUF - 64) {
        u32 op = 1 + rn(6);
        u32 nb = 1 + rn(4), k;
        buf[n++] = (u8) ((op & 0xF) | (((nb - 1) & 7) << 4));
        for (k = 0; k < nb && n < BUF - 32; k++) {
            if (op == 5) { n = put_val(buf, n, fs); continue; }
            n = put_val(buf, n, fv);
            if (op == 4) n = put_val(buf, n, fs);
            n = put_varint(buf, n, rn(12));
        }
        if (rn(6) == 0) break;
    }
    return n;
}
static void hex(const u8* p, int n) { int k; fprintf(o, " "); for (k = 0; k < n; k++) fprintf(o, "%02x", p[k]); }
static void dump_fobj(HSD_FObj* f)
{
    fprintf(o, " %x %x %x %x %x %08x %08x %08x %08x %08x %x", f->flags, f->op, f->op_intrp, f->nb_pack, f->fterm,
            fb(f->time), fb(f->p0), fb(f->p1), fb(f->d0), fb(f->d1), f->ad ? (u32) (f->ad - f->ad_head) : 0xFFFFFFFFu);
}
static void dump(HSD_AObj* a)
{
    HSD_FObj* f; int n = 0;
    fprintf(o, " | %08x %08x %08x %08x %08x %x", a->flags, fb(a->curr_frame), fb(a->rewind_frame), fb(a->end_frame),
            fb(a->framerate), HSD_AObjGetFlags(a));
    for (f = a->fobj; f; f = f->next) n++;
    fprintf(o, " %d", n);
    for (f = a->fobj; f; f = f->next) dump_fobj(f);
}
static f32 rframe(void)
{
    u32 m = rn(8);
    return m == 0 ? 0.f : m < 4 ? (f32) (int) rf(40.f) : m < 7 ? rf(60.f) : rf(3000.f);
}
static u32 rflags(void)
{
    u32 m = rn(4);
    if (m == 0) return (u32) nx();
    return (rn(2) ? 1u << 26 : 0) | (rn(2) ? 1u << 27 : 0) | (rn(2) ? 1u << 28 : 0) | (rn(2) ? 1u << 29 : 0) |
           (rn(2) ? 1u << 30 : 0) | (rn(4) == 0 ? 0x80000000u | rn(256) : 0);
}
int main(int argc, char** argv)
{
    int c, i, k;
    static u8 bufs[3][BUF];
    (void) argc;
    o = fopen(argv[1], "w");
    for (c = 0; c < 120; c++) {
        HSD_FObjDesc d[3]; HSD_AObjDesc ad; HSD_AObj* a; int nf = 1 + rn(3);
        for (k = 0; k < nf; k++) {
            d[k].next = k + 1 < nf ? &d[k + 1] : 0; d[k].frac_value = rfrac(); d[k].frac_slope = rfrac();
            d[k].type = (u8) (1 + rn(12)); d[k].dummy0 = 0;
            d[k].length = (u32) gen_stream(bufs[k], d[k].frac_value, d[k].frac_slope); d[k].ad = bufs[k];
            d[k].startframe = (f32) (int) rf(20.f);
        }
        ad.flags = rflags(); ad.end_frame = rframe(); ad.fobjdesc = rn(10) == 0 ? 0 : d; ad.obj_id = 0;
        if (!ad.fobjdesc) nf = 0;
        fprintf(o, "A %08x %08x %d", ad.flags, fb(ad.end_frame), nf);
        for (k = 0; k < nf; k++) {
            fprintf(o, " %x %08x %x %x %x", d[k].length, fb(d[k].startframe), d[k].type, d[k].frac_value, d[k].frac_slope);
            hex(bufs[k], BUF);
        }
        a = HSD_AObjLoadDesc(&ad);
        dump(a); fprintf(o, "\n");
        for (i = 0; i < 80; i++) {
            u32 op = rn(24), u = 0; f32 f = 0; int nocb = rn(20) == 0;
            ncb = 0;
            switch (op) {
            case 0: f = rframe(); HSD_AObjReqAnim(a, f); break;
            case 1: u = rflags(); HSD_AObjSetFlags(a, u); break;
            case 2: u = rflags(); HSD_AObjClearFlags(a, u); break;
            case 3: { u32 m = rn(6); f = m < 3 ? 1.f : m == 3 ? 0.5f : m == 4 ? rf(4.f) : 2.f; HSD_AObjSetRate(a, f); break; }
            case 4: f = rframe(); HSD_AObjSetRewindFrame(a, f); break;
            case 5: f = rframe(); HSD_AObjSetEndFrame(a, f); break;
            case 6: f = rframe(); HSD_AObjSetCurrentFrame(a, f); break;
            case 7: HSD_AObjStopAnim(a, 0, nocb ? 0 : cb); break;
            case 8: HSD_AObjInitEndCallBack(); HSD_AObjInvokeCallBacks(); break;
            case 9:
                /* replace the fobj chain with a fresh copy of the first desc only, or with none */
                u = (ad.fobjdesc && rn(4) != 0) ? 1 : 0;
                if (u) { HSD_FObjDesc one = d[0]; one.next = 0; HSD_AObjSetFObj(a, HSD_FObjLoadDesc(&one)); }
                else HSD_AObjSetFObj(a, 0);
                break;
            default: HSD_AObjInterpretAnim(a, 0, nocb ? 0 : cb); break;
            }
            if (ncb > 8192) { fprintf(stderr, "too many callbacks\n"); return 2; }
            fprintf(o, "O %d %08x %08x %d %d", op, u, fb(f), nocb, ncb);
            for (k = 0; k < ncb; k++) fprintf(o, " %x %08x", cbt[k], cbv[k]);
            dump(a); fprintf(o, "\n");
        }
        HSD_AObjRemove(a);
    }
    /* id.c: the default table and a local one, keys from a small pool so collisions,
     * replacement and chains in one bucket all happen. */
    {
        static HSD_IDTable local; int t;
        HSD_IDSetup(); memset(&local, 0, sizeof local);
        for (i = 0; i < 20000; i++) {
            u32 op = rn(10), m = rn(4); HSD_IDTable* tb = rn(2) ? 0 : &local; uintptr_t key;
            key = m == 0 ? rn(8) * 101u + rn(3) : m == 1 ? rn(400) : m == 2 ? (uintptr_t) nx() : (uintptr_t) rn(8) * 101u;
            t = tb ? 1 : 0;
            if (op < 4) {
                uintptr_t v = (uintptr_t) (nx() >> 40);
                HSD_IDInsertToTable(tb, key, (void*) v);
                fprintf(o, "I %d %llx %llx\n", t, (unsigned long long) key, (unsigned long long) v);
            } else if (op < 6) {
                HSD_IDRemoveByIDFromTable(tb, key);
                fprintf(o, "D %d %llx\n", t, (unsigned long long) key);
            } else if (op == 9 && rn(50) == 0) {
                _HSD_IDForgetMemory(0, 0);
                fprintf(o, "Z\n");
            } else {
                s32 ok = 7; int want = rn(5) != 0;
                void* r = HSD_IDGetDataFromTable(tb, key, want ? &ok : 0);
                fprintf(o, "G %d %llx %d %llx %d\n", t, (unsigned long long) key, want, (unsigned long long) (uintptr_t) r, ok);
            }
        }
    }
    /* list.c: one list of small ints, full contents after every op. */
    {
        SL* l = 0; SL* p;
        for (i = 0; i < 5000; i++) {
            u32 op = rn(4), v = rn(1000); if (op == 3) op = 2;
            if (op == 0) l = HSD_SListAllocAndAppend(l, (void*) (uintptr_t) v);
            else if (op == 1) l = HSD_SListAllocAndPrepend(l, (void*) (uintptr_t) v);
            else l = HSD_SListRemove(l);
            fprintf(o, "L %d %x |", op, v);
            for (p = l; p; p = p->next) fprintf(o, " %x", (u32) (uintptr_t) p->data);
            fprintf(o, "\n");
        }
    }
    fclose(o);
    return 0;
}
