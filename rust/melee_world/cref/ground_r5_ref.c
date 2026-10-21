/* Driver for the night-3 round-5 oracle (compiled normally). Rows (ints decimal, floats as u32
 * bits in hex):
 *   M null <q> <mex>                                      stage_info.param == NULL
 *   M <x68> <n> <nullp> <st:x18>x6 <q> <mex> <2AE8 bits|spin|->  ("-": rows NULL, not called)
 *   C reset | C a <v> <ok|assert> <t0..t3> | C q <v> <0|1>     the x694 table (values 0..6)
 *   X <i> <v> <2cf4> <m> <mv> <getmap> <2ad8 is &xA0>
 *   G <c3> <c34> <c9> <c10> <stk> <x1C.xyz> <x10> <in.xyz> | <rcruise xyz> <bigblue xyz> <icemt xyz>
 *     c = 0 NULL gobj, 1 NULL user_data, 2 a Ground
 *   F <tilt> <fogc: 0 no gobj, 1 NULL fog, 2 fog> <colnull> <fog rgba> <in rgba> | <20D0> <fog rgba after 205C>
 *     <2090 ret> <out rgba after 2090>
 *   L <classifiers of the list, -1 terminated> | <index of 498C's result or -1>
 *   K <cb set> | <hits> <arg is gobj>                                                                                    */
#include <stdio.h>
#include <stdint.h>
#include <string.h>
typedef struct HSD_GObj { unsigned short classifier; struct HSD_GObj* next; void* hsd_obj; void* user_data; } HSD_GObj;
typedef void (*HSD_GObjEvent)(HSD_GObj*);
typedef struct { unsigned char r, g, b, a; } GXColor;
typedef struct HSD_Fog { int type; GXColor color; } HSD_Fog;
HSD_GObj* HSD_GObjPLinkHead[16];
typedef struct { float x, y, z; } V3;
typedef struct StageParam { int stkind; int x4, x8; unsigned xC, x10; short x14, x16, x18; } StageParam;
typedef struct GroundParam { short x68; StageParam* stage_params; int stage_param_count; } GroundParam;
typedef struct Ground {
    unsigned char pad[0xC]; HSD_GObjEvent xC_callback; int map_id;
    union { struct { unsigned char b; V3 x4, x10, x1C; } scroll; struct { short a[8]; float x10; } icemt; } u;
} Ground;
struct StageInfoR5 { struct { float cam_vertical_tilt; } cam_info; HSD_GObj* x12C; int xA0; GroundParam* param; HSD_GObj* map_gobjs[64]; void* x280[261]; void* x694[4]; };
extern struct StageInfoR5 stage_info;
int r5_mex(int); void r5_1d38(HSD_GObj*);
float Ground_801C20D0(void); void Ground_801C205C(GXColor*); int Ground_801C2090(GXColor*); HSD_GObj* Ground_801C498C(void);
static int cb_hits; static HSD_GObj* cb_arg;
static void cb(HSD_GObj* g) { cb_hits++; cb_arg = g; } void r5_2bbc(HSD_GObj*, int); void r5_2bd4(void*);
int* Ground_801C2AD8(void); float Ground_801C2AE8(int); HSD_GObj* Ground_GetMapGObj(int);
int Ground_801C2C8C(void*); void* Ground_801C2CF4(int); void Ground_801C2D0C(int, void*);
void grRCruise_80201918(V3*); void grBigBlue_801EF7D8(V3*); void grIceMt_801FA728(V3*);
static int asserted;
static int cur_stk;
int Stage_80225194(void) { return cur_stk; }
void OSReport(const char* f, ...) { (void) f; }
void r5_assert(int line) { (void) line; asserted = 1; }
static uint32_t rs = 0x12345678u;
static uint32_t rnd(void) { rs ^= rs << 13; rs ^= rs >> 17; rs ^= rs << 5; return rs; }
static uint32_t fb(float f) { uint32_t u; memcpy(&u, &f, 4); return u; }
static float rf(void)
{
    uint32_t u = rnd();
    float f;
    if (rnd() % 4 == 0) { memcpy(&f, &u, 4); return f; }
    return (float) (int) (rnd() % 20001 - 10000) / 7.0f;
}
static __attribute__((noinline)) int try_2ae8(int q, float* r)
{
    /* The C spins after OSReport when no row matches the Mex stkind: predicted here, not run. */
    int m = r5_mex(q);
    for (int i = 0; i < stage_info.param->stage_param_count; i++)
        if (stage_info.param->stage_params[i].stkind == m) { *r = Ground_801C2AE8(q); return 1; }
    return 0;
}
static __attribute__((noinline)) int try_2bd4(uintptr_t v)
{
    asserted = 0;
    r5_2bd4((void*) v);
    return !asserted;
}
int main(void)
{
    static StageParam rows[8];
    static GroundParam gp;
    for (int k = 0; k < 6000; k++) {
        int q = (int) (rnd() % 8) - 1;
        if (k % 50 == 0) {
            stage_info.param = NULL;
            printf("M null %d %d\n", q, r5_mex(q));
            continue;
        }
        int n = (int) (rnd() % 7) - 1, nullp = rnd() % 30 == 0;
        gp.x68 = (short) rnd(); gp.stage_param_count = n; gp.stage_params = nullp ? NULL : rows;
        stage_info.param = &gp;
        for (int i = 0; i < 8; i++) { rows[i].stkind = (int) (rnd() % 8) - 1; rows[i].x18 = (short) rnd(); }
        printf("M %d %d %d", gp.x68, n, nullp);
        for (int i = 0; i < 6; i++) printf(" %d:%d", rows[i].stkind, rows[i].x18);
        printf(" %d %d", q, r5_mex(q));
        if (nullp && n > 0) { printf(" -\n"); continue; }
        float r;
        if (try_2ae8(q, &r)) printf(" %08x\n", fb(r));
        else printf(" spin\n");
    }
    for (int k = 0; k < 4000; k++) {
        if (k % 20 == 0) { memset(stage_info.x694, 0, sizeof stage_info.x694); printf("C reset\n"); }
        uintptr_t v = rnd() % 7;
        if (rnd() % 2) {
            if (try_2bd4(v)) printf("C a %d ok", (int) v);
            else printf("C a %d assert", (int) v);
            for (int i = 0; i < 4; i++) printf(" %d", (int) (uintptr_t) stage_info.x694[i]);
            printf("\n");
        } else {
            printf("C q %d %d\n", (int) v, Ground_801C2C8C((void*) v));
        }
    }
    for (int k = 0; k < 2000; k++) {
        int i = (int) (rnd() % 261), m = (int) (rnd() % 64);
        uintptr_t v = rnd() & 0xFFFF, mv = rnd() & 0xFFFF;
        Ground_801C2D0C(i, (void*) v);
        r5_2bbc((HSD_GObj*) mv, m);
        printf("X %d %d %d %d %d %d %d\n", i, (int) v, (int) (uintptr_t) Ground_801C2CF4(i), m, (int) mv,
               (int) (uintptr_t) Ground_GetMapGObj(m), Ground_801C2AD8() == &stage_info.xA0);
    }
    static HSD_GObj go[4];
    static Ground gr[4];
    static const int ids[4] = { 3, 34, 9, 10 };
    for (int k = 0; k < 4000; k++) {
        int c[4];
        memset(stage_info.map_gobjs, 0, sizeof stage_info.map_gobjs);
        V3 x1C = { rf(), rf(), rf() };
        float x10 = rf();
        cur_stk = rnd() % 2 ? 0x4C : (int) (rnd() % 90);
        for (int j = 0; j < 4; j++) {
            c[j] = (int) (rnd() % 3);
            memset(&gr[j], 0xA5, sizeof gr[j]);
            if (j < 2) gr[j].u.scroll.x1C = x1C; else gr[j].u.icemt.x10 = x10;
            go[j].user_data = c[j] == 1 ? NULL : &gr[j];
            stage_info.map_gobjs[ids[j]] = c[j] == 0 ? NULL : &go[j];
        }
        V3 a = { rf(), rf(), rf() }, b = a, d = a;
        printf("G %d %d %d %d %d %08x %08x %08x %08x %08x %08x %08x |", c[0], c[1], c[2], c[3], cur_stk,
               fb(x1C.x), fb(x1C.y), fb(x1C.z), fb(x10), fb(a.x), fb(a.y), fb(a.z));
        grRCruise_80201918(&a); grBigBlue_801EF7D8(&b); grIceMt_801FA728(&d);
        printf(" %08x %08x %08x %08x %08x %08x %08x %08x %08x\n",
               fb(a.x), fb(a.y), fb(a.z), fb(b.x), fb(b.y), fb(b.z), fb(d.x), fb(d.y), fb(d.z));
    }
    static HSD_GObj fg;
    static HSD_Fog fog;
    for (int k = 0; k < 3000; k++) {
        stage_info.cam_info.cam_vertical_tilt = rf();
        int fc = (int) (rnd() % 3), cn = rnd() % 4 == 0;
        stage_info.x12C = fc == 0 ? NULL : &fg;
        fg.hsd_obj = fc == 1 ? NULL : &fog;
        uint32_t fv = rnd(), iv = rnd();
        memcpy(&fog.color, &fv, 4);
        GXColor in, out;
        memcpy(&in, &iv, 4);
        memcpy(&out, &fv, 4); out.r ^= 0x5A;
        uint32_t ov0; memcpy(&ov0, &out, 4);
        printf("F %08x %d %d %08x %08x |", fb(stage_info.cam_info.cam_vertical_tilt), fc, cn, fv, iv);
        printf(" %08x", fb(Ground_801C20D0()));
        Ground_801C205C(cn ? NULL : &in);
        uint32_t f2; memcpy(&f2, &fog.color, 4);
        int r = Ground_801C2090(cn ? NULL : &out);
        uint32_t ov; memcpy(&ov, &out, 4);
        printf(" %08x %d %08x %08x\n", f2, r, ov, ov0);
    }
    static HSD_GObj lg[8];
    for (int k = 0; k < 2000; k++) {
        int n = (int) (rnd() % 9);
        printf("L");
        for (int j = 0; j < n; j++) {
            lg[j].classifier = (unsigned short) (rnd() % 4 == 0 ? 13 : rnd() % 3 == 0 ? rnd() : rnd() % 20);
            lg[j].next = j + 1 < n ? &lg[j + 1] : NULL;
            printf(" %d", lg[j].classifier);
        }
        HSD_GObjPLinkHead[3] = n ? &lg[0] : NULL;
        HSD_GObjPLinkHead[2] = &lg[7]; HSD_GObjPLinkHead[4] = &lg[7];
        HSD_GObj* g = Ground_801C498C();
        printf(" -1 | %d\n", g ? (int) (g - lg) : -1);
    }
    for (int k = 0; k < 200; k++) {
        static HSD_GObj kg; static Ground kgr;
        int set = (int) (rnd() % 2);
        kg.user_data = &kgr; kgr.xC_callback = set ? cb : NULL;
        cb_hits = 0; cb_arg = NULL;
        r5_1d38(&kg);
        printf("K %d | %d %d\n", set, cb_hits, cb_arg == &kg);
    }
    return 0;
}
