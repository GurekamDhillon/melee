extern int atoi(const char*);
extern char* getenv(const char*);
/* TU: src/sysdolphin/baselib/objalloc.c */
/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a
 * recording stub. Written by gen/mtu/mk_lb.py. Each line: op case ret state-hash trace-hash. */

#include TU_SRC
#include "mtu_common.h"
static HSD_ObjAllocData OD[4];
static struct { float x, y, z; } VA[4];
static u8 U8A[16];
static int IA[4];
static u32 UA[4];
static float FA[4];
static unsigned char BUF[256];
static char STRS[4][64] = { "a.dat", "sym", "LbAd.dat", "x" };

static HSD_ObjAllocData* pick_HSD_ObjAllocData(void) { u32 r = rnd(); return &OD[(r >> 8) % 4]; }
static HSD_ObjAllocData* picko_HSD_ObjAllocData(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &OD[(r >> 8) % 4]; }
struct HSD_GObj; typedef struct HSD_GObj HSD_GObj;
static void cbev1(HSD_GObj* g) { T(7001); T(PI(g)); }
static void cbev2(HSD_GObj* g) { T(7002); T(PI(g)); }
static int cbp1(HSD_GObj* g) { T(7003); T(PI(g)); return (int) (fv() & 1); }
static int cbp2(HSD_GObj* g) { T(7004); T(PI(g)); return (int) (fv() & 1); }
static void cbev3(HSD_GObj* g, HSD_GObj* h) { T(7005); T(PI(g)); T(PI(h)); }
static void cbi1(int a) { T(7006); T((u32) a); }
static void cbi2(int a) { T(7007); T((u32) a); }
static void* pick_fn(int kind)
{
    u32 r = rnd();
    if ((r & 3) == 0) return 0;
    switch (kind) {
    case 1: return (r & 4) ? (void*) cbev1 : (void*) cbev2;
    case 2: return (r & 4) ? (void*) cbp1 : (void*) cbp2;
    case 3: return (void*) cbev3;
    case 4: return (r & 4) ? (void*) cbi1 : (void*) cbi2;
    }
    return 0;
}
static void* pick_cb(int kind) { void* f = pick_fn(kind); return f ? f : (kind == 2 ? (void*) cbp1 : kind == 3 ? (void*) cbev3 : kind == 4 ? (void*) cbi1 : (void*) cbev1); }
static void freg_world(void) { freg((const void*) cbev1); freg((const void*) cbev2); freg((const void*) cbp1); freg((const void*) cbp2); freg((const void*) cbev3); freg((const void*) cbi1); freg((const void*) cbi2); }
static unsigned char POOLMEM[16384]; static unsigned char LST[4][64]; static void hw(const unsigned char* b, int n) { int i; for (i = 0; i < n; i += 8) { unsigned long long v = *(const unsigned long long*) (b + i); u32 p = PI((const void*) v); if (p != 0xFFFFFFFFu) HM(p); else { HM((u32) v); HM((u32) (v >> 32)); } } }
#include "objalloc_gen.h"



static void preg_world(void)
{
    preg(OD, sizeof OD);
    preg(VA, sizeof VA); preg(U8A, sizeof U8A); preg(IA, sizeof IA); preg(UA, sizeof UA); preg(FA, sizeof FA); preg(BUF, sizeof BUF); preg(STRS, sizeof STRS);
}

static void setup_world(void)
{
    int i;
    (void) i;
    zero(OD, sizeof OD);
    for (i = 0; i < 16; i++) { U8A[i] = (u8) rnd(); }
    for (i = 0; i < 4; i++) { IA[i] = (int) fv(); UA[i] = fv(); FA[i] = rw(); VA[i].x = rw(); VA[i].y = rw(); VA[i].z = rw(); }
    for (i = 0; i < 256; i++) BUF[i] = (unsigned char) rnd();
    BUF[255] = 0;
    for (i = 0; i < 4; i++) fill_HSD_ObjAllocData(&OD[i]);
    fillg_all();
    fillo_all();
}

static void hash_world(void)
{
    int i;
    (void) i;
    for (i = 0; i < 4; i++) hash_HSD_ObjAllocData(&OD[i]);
    for (i = 0; i < 4; i++) { HM(B(VA[i].x)); HM(B(VA[i].y)); HM(B(VA[i].z)); HM(B(FA[i])); HM((u32) IA[i]); HM(UA[i]); }
    for (i = 0; i < 16; i++) HM(U8A[i]);
    for (i = 0; i < 256; i++) HM(BUF[i]);
    hashg_all();
    hasho_all();
}

enum { OP_snap_note_render = 0, OP_HSD_ObjAllocTopUp = 1, OP_HSD_ObjAllocReport = 2, OP_HSD_ObjAllocAddFree = 3, OP_HSD_ObjAlloc = 4, OP_HSD_ObjFree = 5, OP_removeAll = 6, OP_HSD_ObjAllocInit = 7, OP__HSD_ObjAllocForgetMemory = 8 };

int main(int argc, char** argv)
{
    int op, k, only = -1;
    if (argc > 1) only = atoi(argv[1]);
    preg_world();
    preg_own();
    freg_all();
    freg_world();
preg(POOLMEM, sizeof POOLMEM); preg(LST, sizeof LST);
    for (op = 0; op < 9; op++) {
        int n = 0;
        if (only >= 0 && op != only) continue;
        switch (op) {
        case 0: n = 400; break;  /* snap_note_render */
        case 1: n = 400; break;  /* HSD_ObjAllocTopUp */
        case 2: n = 400; break;  /* HSD_ObjAllocReport */
        case 3: n = 400; break;  /* HSD_ObjAllocAddFree */
        case 4: n = 400; break;  /* HSD_ObjAlloc */
        case 5: n = 400; break;  /* HSD_ObjFree */
        case 6: n = 400; break;  /* removeAll */
        case 7: n = 400; break;  /* HSD_ObjAllocInit */
        case 8: n = 400; break;  /* _HSD_ObjAllocForgetMemory */
        }
        for (k = 0; k < n; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup_world();
{ int q, k, i; alloc_datas = (fv() & 1) ? &OD[fv() % 4] : 0; obj_heap.top = 0; obj_heap.curr = 0; obj_heap.size = (u32) -1; obj_heap.remain = fv(); for (i = 0; i < 16384; i++) POOLMEM[i] = (unsigned char) (i * 31 + 7); for (i = 0; i < 256; i++) LST[i >> 6][i & 63] = (unsigned char) (i * 13 + 1); for (q = 0; q < 4; q++) { OD[q].size = 8 + 8 * (fv() % 4); OD[q].align = 7; OD[q].next = (q < 3 && (fv() % 4)) ? &OD[q + 1] : 0; k = fv() % 3; for (i = 0; i < k; i++) *(void**) (LST[q] + 32 * i) = (i + 1 < k) ? (void*) (LST[q] + 32 * (i + 1)) : 0; OD[q].freehead = k ? (HSD_ObjAllocLink*) LST[q] : 0; OD[q].free = k; OD[q].used = fv() % 5; OD[q].peak = OD[q].used + fv() % 3; OD[q].num_limit = fv() % 8; OD[q].heap_limit_size = fv() % 1000; OD[q].heap_limit_num = (fv() & 1) ? (u32) -1 : fv() % 8; } }
            if (getenv("MTU_PROG")) { printf("P %d %d%c", op, k, 10); fflush(0); }
            ARMED = 1;
            if (MTU_SETJMP()) { ARMED = 0; printf("%d %d ABORT\n", op, k); continue; }
            switch (op) {
            case 0: {  HSD_ObjAllocData * a0 = pick_HSD_ObjAllocData(); int a1 = fv(); snap_note_render(a0, a1);  } break;
            case 1: {   HSD_ObjAllocTopUp();  } break;
            case 2: {   HSD_ObjAllocReport();  } break;
            case 3: {  HSD_ObjAllocData * a0 = pick_HSD_ObjAllocData(); u32 a1 = (u32) (1 + fv() % 8); ret = (u32) HSD_ObjAllocAddFree(a0, a1);  } break;
            case 4: {  HSD_ObjAllocData * a0 = pick_HSD_ObjAllocData(); ret = PI(HSD_ObjAlloc(a0));  } break;
            case 5: {  HSD_ObjAllocData * a0 = pick_HSD_ObjAllocData(); void * a1 = (void*) (LST[0] + 8 * (fv() % 32)); HSD_ObjFree(a0, a1);  } break;
            case 6: {  HSD_ObjAllocData * a0 = pick_HSD_ObjAllocData(); removeAll(a0);  } break;
            case 7: {  HSD_ObjAllocData * a0 = pick_HSD_ObjAllocData(); gc_size_t a1 = (size_t) (1 + fv() % 40); u32 a2 = (u32) 8; HSD_ObjAllocInit(a0, a1, a2);  } break;
            case 8: {  void * a0 = (void*) BUF; void * a1 = (void*) BUF; _HSD_ObjAllocForgetMemory(a0, a1);  } break;
            }
            ARMED = 0;
            hash_world();
{ hw(POOLMEM, 16384); hw(LST[0], 256); }
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
