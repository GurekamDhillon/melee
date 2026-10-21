/* Oracle driver for baselib fobj.c, util.c, crypt.c (see build_anim.sh). Generated inputs only;
 * writes hex rows that tests/oracle_anim.rs replays. */
#include <stdio.h>
#include <string.h>
#include <stdint.h>
#include <stdlib.h>
typedef float f32; typedef unsigned u32; typedef int s32; typedef short s16; typedef unsigned short u16;
typedef unsigned char u8; typedef signed char s8;
typedef struct HSD_FObj {
    struct HSD_FObj* next; u8* ad; u8* ad_head; u32 length;
    u8 flags, op, op_intrp, obj_type, frac_value, frac_slope; u16 nb_pack; s16 startframe; u16 fterm;
    f32 time, p0, p1, d0, d1;
} HSD_FObj;
typedef struct HSD_FObjDesc { struct HSD_FObjDesc* next; u32 length; f32 startframe; u8 type, frac_value, frac_slope, dummy0; u8* ad; } HSD_FObjDesc;
typedef union { f32 fv; s32 iv; f32 p[3]; } HSD_ObjData;
typedef void (*Upd)(void*, int, HSD_ObjData*);
typedef struct { u8 r, g, b, a; } GXColor;
void HSD_FObjReqAnimAll(HSD_FObj*, f32); void HSD_FObjStopAnim(HSD_FObj*, void*, Upd, f32);
void HSD_FObjInterpretAnim(HSD_FObj*, void*, Upd, f32); HSD_FObj* HSD_FObjLoadDesc(HSD_FObjDesc*);
void HSD_MulColor(GXColor*, GXColor*, GXColor*); u32 HSD_GetNbBits(u32); s32 HSD_Index2PosNrmMtx(u32);
int HSD_Encrypt(u8*, int); int HSD_Decrypt(u8*, int);

static uint64_t st = 0x9E3779B97F4A7C15ull;
static uint64_t nx(void) { st ^= st << 13; st ^= st >> 7; st ^= st << 17; return st; }
static u32 rn(u32 n) { return (u32) (nx() >> 33) % n; }
static u32 fb(f32 f) { u32 u; memcpy(&u, &f, 4); return u; }
static f32 rf(f32 r) { return (f32) ((double) (nx() >> 11) / 9007199254740992.0 * 2.0 - 1.0) * r; }
static FILE* o;
static int ncb; static u32 cbt[4096], cbv[4096];
static void cb(void* obj, int type, HSD_ObjData* d) { (void) obj; if (ncb < 4096) { cbt[ncb] = (u32) type; cbv[ncb] = (u32) d->iv; } ncb++; }

#define BUF 512
static u8 buf[BUF + 64];
static int put_varint(int n, u32 v) { do { u8 b = v & 0x7F; v >>= 7; if (v) b |= 0x80; buf[n++] = b; } while (v); return n; }
static u8 rfrac(void)
{
    static const u8 kinds[] = { 0x00, 0x20, 0x40, 0x60, 0x80, 0xA0 };
    u8 k = kinds[rn(6)];
    if (k == 0 && rn(2)) return 0;
    return k | (u8) (rn(5) == 0 ? rn(32) : rn(9));
}
static int put_val(int n, u8 frac)
{
    u8 k = frac & 0xE0;
    int sz = (k == 0x20 || k == 0x40) ? 2 : (k == 0x60 || k == 0x80) ? 1 : 0;
    if (frac == 0) {
        u32 b = fb(rf(rn(4) == 0 ? 1000.f : 10.f)); int i;
        for (i = 0; i < 4; i++) buf[n++] = (u8) (b >> (8 * i));
        return n;
    }
    while (sz--) buf[n++] = (u8) nx();
    return n;
}
static int gen_stream(u8 fv, u8 fs)
{
    int n = 0;
    memset(buf, 0, sizeof buf);
    if (rn(10) == 0) { int i; for (i = 0; i < 200; i++) buf[i] = (u8) nx(); return 200; }
    while (n < BUF - 64) {
        u32 op = rn(20) == 0 ? rn(16) : 1 + rn(6);
        u32 nb = rn(4) == 0 ? 1 + rn(40) : 1 + rn(4), k;
        u8 h = (u8) ((op & 0xF) | (((nb - 1) & 7) << 4));
        if (nb - 1 >= 8) { buf[n++] = h | 0x80; n = put_varint(n, (nb - 1) >> 3); } else buf[n++] = h;
        for (k = 0; k < nb && n < BUF - 32; k++) {
            if (op == 5) { n = put_val(n, fs); continue; }
            n = put_val(n, fv);
            if (op == 4) n = put_val(n, fs);
            n = put_varint(n, rn(6) == 0 ? rn(300) : rn(12));
        }
        if (rn(8) == 0) break;
    }
    return n;
}
static void dump(HSD_FObj* f)
{
    fprintf(o, " %x %x %x %x %x %08x %08x %08x %08x %08x %x", f->flags, f->op, f->op_intrp, f->nb_pack, f->fterm,
            fb(f->time), fb(f->p0), fb(f->p1), fb(f->d0), fb(f->d1), (u32) (f->ad - f->ad_head));
}
static void hex(const u8* p, int n) { int k; fprintf(o, " "); for (k = 0; k < n; k++) fprintf(o, "%02x", p[k]); }
int main(int argc, char** argv)
{
    int c, i;
    (void) argc;
    o = fopen(argv[1], "w");
    for (c = 0; c < 1000; c++) {
        HSD_FObjDesc d; HSD_FObj* f; int len; f32 req;
        d.next = 0; d.frac_value = rfrac(); d.frac_slope = rfrac(); d.type = (u8) rn(40); d.dummy0 = 0;
        len = gen_stream(d.frac_value, d.frac_slope);
        if (rn(6) == 0) len = rn(len + 1);
        d.length = (u32) len; d.ad = buf;
        d.startframe = rn(8) == 0 ? rf(70000.f) : (f32) (int) rf(20.f);
        f = HSD_FObjLoadDesc(&d);
        req = rn(3) == 0 ? rf(10.f) : 0.f;
        fprintf(o, "C %x %08x %x %x %x", d.length, fb(d.startframe), d.type, d.frac_value, d.frac_slope);
        hex(buf, BUF);
        fprintf(o, " %08x %x\n", fb(req), (u16) f->startframe);
        HSD_FObjReqAnimAll(f, req);
        for (i = 0; i < 60; i++) {
            u32 m = rn(10); f32 rate = m < 6 ? 1.f : m == 6 ? 0.5f : m == 7 ? rf(4.f) : m == 8 ? 3.f : rf(12.f);
            int stop = rn(60) == 0, nocb = rn(30) == 0, k;
            ncb = 0;
            if (stop) HSD_FObjStopAnim(f, 0, nocb ? 0 : cb, rate); else HSD_FObjInterpretAnim(f, 0, nocb ? 0 : cb, rate);
            if (ncb > 4096) { fprintf(stderr, "too many callbacks\n"); return 2; }
            fprintf(o, "F %08x %d %d %d", fb(rate), stop, nocb, ncb);
            for (k = 0; k < ncb; k++) fprintf(o, " %x %08x", cbt[k], cbv[k]);
            dump(f);
            if (stop && rn(2)) { req = rf(5.f); HSD_FObjReqAnimAll(f, req); fprintf(o, " R %08x", fb(req)); dump(f); }
            fprintf(o, "\n");
        }
        free(f);
    }
    for (c = 0; c < 2000; c++) {
        GXColor a, b, r; u32 x = (u32) nx(); u8 data[96]; int len = 16 + rn(80), e, k;
        memcpy(&a, &x, 4); x = (u32) nx(); memcpy(&b, &x, 4);
        if (c % 7 == 0) x = 0;
        if (c % 11 == 0) x = 0xFFFFFFFFu;
        HSD_MulColor(&a, &b, &r);
        fprintf(o, "U %02x%02x%02x%02x %02x%02x%02x%02x %02x%02x%02x%02x %08x %x %x %x\n", a.r, a.g, a.b, a.a, b.r, b.g, b.b,
                b.a, r.r, r.g, r.b, r.a, x, HSD_GetNbBits(x), c % 10, HSD_Index2PosNrmMtx(c % 10));
        for (k = 0; k < len; k++) data[k] = (u8) (c % 5 == 0 ? rn(3) : nx());
        fprintf(o, "E %x", len); hex(data, len);
        e = HSD_Encrypt(data, len);
        fprintf(o, " %d", e); hex(data, len);
        if (c % 3 == 0 && len > 16) data[16 + rn(len - 16)] ^= (u8) (1 << rn(8));
        if (c % 9 == 0) for (k = 0; k < len; k++) data[k] = (u8) nx();
        hex(data, len);
        e = HSD_Decrypt(data, len);
        fprintf(o, " %d", e); hex(data, len);
        fprintf(o, "\n");
    }
    fclose(o);
    return 0;
}
