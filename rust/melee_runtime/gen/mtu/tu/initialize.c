extern int atoi(const char*);
extern char* getenv(const char*);
/* TU: src/sysdolphin/baselib/initialize.c */
/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a
 * recording stub. Written by gen/mtu/mk_lb.py. Each line: op case ret state-hash trace-hash. */

#include TU_SRC
#include "mtu_common.h"
static GXRenderModeObj RMO[2];
static HSD_ObjAllocData OAD[2];
static Vec3 VA[4];
static u8 U8A[16];
static int IA[4];
static u32 UA[4];
static float FA[4];
static unsigned char BUF[256];
static void* PPA[4];
static char STRS[4][64] = { "a.dat", "sym", "LbAd.dat", "x" };

static GXRenderModeObj* pick_GXRenderModeObj(void) { u32 r = rnd(); return &RMO[(r >> 8) % 2]; }
static GXRenderModeObj* picko_GXRenderModeObj(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &RMO[(r >> 8) % 2]; }
static HSD_ObjAllocData* pick_HSD_ObjAllocData(void) { u32 r = rnd(); return &OAD[(r >> 8) % 2]; }
static HSD_ObjAllocData* picko_HSD_ObjAllocData(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &OAD[(r >> 8) % 2]; }
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
static void freg_world(void) { freg((const void*) cbev1); freg((const void*) cbev2); freg((const void*) cbp1); freg((const void*) cbp2); freg((const void*) cbev3); freg((const void*) cbi1); freg((const void*) cbi2);  }

static unsigned char ARN[4096] __attribute__((aligned(64)));
static int AZ, LOK, HIK;

#include "initialize_gen.h"



static void preg_world(void)
{
    preg(RMO, sizeof RMO);
    preg(OAD, sizeof OAD);
    preg(VA, sizeof VA); preg(U8A, sizeof U8A); preg(IA, sizeof IA); preg(UA, sizeof UA); preg(FA, sizeof FA); preg(BUF, sizeof BUF); preg(STRS, sizeof STRS); preg(PPA, sizeof PPA);
}

static void setup_world(void)
{
    int i;
    (void) i;
    zero(RMO, sizeof RMO);
    zero(OAD, sizeof OAD);
    for (i = 0; i < 16; i++) { U8A[i] = (u8) rnd(); }
    for (i = 0; i < 4; i++) { IA[i] = (int) fv(); UA[i] = fv(); FA[i] = rw(); VA[i].x = rw(); VA[i].y = rw(); VA[i].z = rw(); }
    for (i = 0; i < 256; i++) BUF[i] = (unsigned char) rnd();
    BUF[255] = 0;
    for (i = 0; i < 4; i++) PPA[i] = 0;
    for (i = 0; i < 2; i++) fill_GXRenderModeObj(&RMO[i]);
    for (i = 0; i < 2; i++) fill_HSD_ObjAllocData(&OAD[i]);
    fillg_all();
    fillo_all();
}

static void hash_world(void)
{
    int i;
    (void) i;
    for (i = 0; i < 2; i++) hash_GXRenderModeObj(&RMO[i]);
    for (i = 0; i < 2; i++) hash_HSD_ObjAllocData(&OAD[i]);
    for (i = 0; i < 4; i++) { HM(B(VA[i].x)); HM(B(VA[i].y)); HM(B(VA[i].z)); HM(B(FA[i])); HM((u32) IA[i]); HM(UA[i]); }
    for (i = 0; i < 16; i++) HM(U8A[i]);
    for (i = 0; i < 256; i++) HM(BUF[i]);
    for (i = 0; i < 4; i++) HM(PI(PPA[i]));
    hashg_all();
    hasho_all();
}

enum { OP_HSD_InitComponent = 0, OP_HSD_GXSetFifoObj = 1, OP_HSD_DVDInit = 2, OP_HSD_AllocateXFB = 3, OP_HSD_AllocateFifo = 4, OP_HSD_GXInit = 5, OP_HSD_OSInit = 6, OP_HSD_GetHeap = 7, OP_HSD_SetHeap = 8, OP_HSD_GetNextArena = 9, OP_HSD_CreateMainHeap = 10, OP_HSD_GetCurrentRenderPass = 11, OP_HSD_StartRender = 12, OP_HSD_Init_803755A8 = 13, OP_HSD_ObjInit = 14, OP_HSD_ObjDumpStat = 15, OP_HSD_SetInitParameterU32 = 16, OP_HSD_SetInitParameterPtr = 17 };

int main(int argc, char** argv)
{
    int op, k, only = -1;
    if (argc > 1) only = atoi(argv[1]);
    preg_world();
    preg_own();
    freg_all();
    freg_world();
preg(ARN, sizeof ARN); preg(&GXNtsc480IntDf, sizeof GXNtsc480IntDf);
    for (op = 0; op < 18; op++) {
        int n = 0;
        if (only >= 0 && op != only) continue;
        switch (op) {
        case 0: n = 300; break;  /* HSD_InitComponent */
        case 1: n = 300; break;  /* HSD_GXSetFifoObj */
        case 2: n = 300; break;  /* HSD_DVDInit */
        case 3: n = 300; break;  /* HSD_AllocateXFB */
        case 4: n = 300; break;  /* HSD_AllocateFifo */
        case 5: n = 300; break;  /* HSD_GXInit */
        case 6: n = 300; break;  /* HSD_OSInit */
        case 7: n = 300; break;  /* HSD_GetHeap */
        case 8: n = 300; break;  /* HSD_SetHeap */
        case 9: n = 300; break;  /* HSD_GetNextArena */
        case 10: n = 300; break;  /* HSD_CreateMainHeap */
        case 11: n = 300; break;  /* HSD_GetCurrentRenderPass */
        case 12: n = 300; break;  /* HSD_StartRender */
        case 13: n = 300; break;  /* HSD_Init_803755A8 */
        case 14: n = 300; break;  /* HSD_ObjInit */
        case 15: n = 300; break;  /* HSD_ObjDumpStat */
        case 16: n = 300; break;  /* HSD_SetInitParameterU32 */
        case 17: n = 300; break;  /* HSD_SetInitParameterPtr */
        }
        for (k = 0; k < n; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup_world();
{ int q; AZ = (fv() & 3) == 0; LOK = fv() % 16; HIK = (fv() & 3) == 0 ? LOK : 32 + fv() % 16;
 memset(&memReport, 0, sizeof memReport); for (q = 0; q < 3; q++) FrameBuffer[q] = 0; for (q = 0; q < 2; q++) RMO[q].fbWidth = (u16) (fv() % 700), RMO[q].xfbHeight = (u16) (fv() % 500), RMO[q].viHeight = (u16) (fv() % 500), RMO[q].aa = (u8) (fv() & 1), RMO[q].field_rendering = (u8) (fv() & 1);
 hsd_heap_next_arena_lo = AZ ? 0 : ARN + 32 * (fv() % 16); hsd_heap_next_arena_hi = AZ ? 0 : ARN + 1024 + 32 * (fv() % 16);
 current_render_pass = fv() % 4; DefaultFifoObj = (fv() & 1) ? (GXFifoObj*) BUF : 0; current_pix_fmt = fv() % 8; init_done = fv() & 1; shown = fv() & 1; current_heap = (int) fv() - 1;
 rmode = (fv() & 1) ? &RMO[fv() % 2] : &GXNtsc480IntDf; current_z_fmt = fv() % 3; iparam_fifo_size = fv(); iparam_xfb_max_num = (int) fv(); iparam_heap_max_num = (int) fv() % 8; iparam_audio_heap_size = 32 * (fv() % 8);
}
            if (getenv("MTU_PROG")) { printf("P %d %d%c", op, k, 10); fflush(0); }
            ARMED = 1;
            if (MTU_SETJMP()) { ARMED = 0; printf("%d %d ABORT\n", op, k); continue; }
            switch (op) {
            case 0: {   HSD_InitComponent();  } break;
            case 1: {  GXFifoObj * a0 = (GXFifoObj *) BUF; HSD_GXSetFifoObj(a0);  } break;
            case 2: {   HSD_DVDInit();  } break;
            case 3: {  s32 a0 = (s32) (fv() % 4); GXRenderModeObj * a1 = pick_GXRenderModeObj(); ret = PI(HSD_AllocateXFB(a0, a1));  } break;
            case 4: {  u32 a0 = fv() % 3000; ret = PI(HSD_AllocateFifo(a0));  } break;
            case 5: {   HSD_GXInit();  } break;
            case 6: {   HSD_OSInit();  } break;
            case 7: {   ret = (u32) HSD_GetHeap();  } break;
            case 8: {  OSHeapHandle a0 = fv(); HSD_SetHeap(a0);  } break;
            case 9: {  void ** a0 = (void*) &PPA[fv() % 4]; void ** a1 = (void*) &PPA[fv() % 4]; HSD_GetNextArena(a0, a1);  } break;
            case 10: {  void * a0 = (!AZ && (fv() & 1)) ? (void*) (ARN + 32 * (fv() % 16)) : 0; void * a1 = (!AZ && (fv() & 1)) ? (void*) (ARN + 1024 + 32 * (fv() % 16)) : 0; ret = (u32) HSD_CreateMainHeap(a0, a1);  } break;
            case 11: {   ret = (u32) HSD_GetCurrentRenderPass();  } break;
            case 12: {  HSD_RenderPass a0 = (HSD_RenderPass) (fv() % 4); HSD_StartRender(a0);  } break;
            case 13: {   HSD_Init_803755A8();  } break;
            case 14: {   HSD_ObjInit();  } break;
            case 15: {   HSD_ObjDumpStat();  } break;
            case 16: {  HSD_InitParam a0 = (HSD_InitParam) (fv() % 6); u32 a1 = fv(); ret = (u32) HSD_SetInitParameterU32(a0, a1);  } break;
            case 17: {  HSD_InitParam a0 = fv(); void * a1 = (void*) BUF; ret = (u32) HSD_SetInitParameterPtr(a0, a1);  } break;
            }
            ARMED = 0;
            hash_world();

            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
