extern int atoi(const char*);
extern char* getenv(const char*);
/* TU: src/sysdolphin/baselib/video.c */
/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a
 * recording stub. Written by gen/mtu/mk_lb.py. Each line: op case ret state-hash trace-hash. */

#include TU_SRC
#include "mtu_common.h"
static HSD_VIStatus VIS[2];
static GXRenderModeObj RMO[2];
static struct { float x, y, z; } VA[4];
static u8 U8A[16];
static int IA[4];
static u32 UA[4];
static float FA[4];
static unsigned char BUF[256];
static void* PPA[4];
static char STRS[4][64] = { "a.dat", "sym", "LbAd.dat", "x" };

static HSD_VIStatus* pick_HSD_VIStatus(void) { u32 r = rnd(); return &VIS[(r >> 8) % 2]; }
static HSD_VIStatus* picko_HSD_VIStatus(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &VIS[(r >> 8) % 2]; }
static GXRenderModeObj* pick_GXRenderModeObj(void) { u32 r = rnd(); return &RMO[(r >> 8) % 2]; }
static GXRenderModeObj* picko_GXRenderModeObj(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &RMO[(r >> 8) % 2]; }
struct HSD_GObj; typedef struct HSD_GObj HSD_GObj;
static void cbev1(HSD_GObj* g) { T(7001); T(PI(g)); }
static void cbev2(HSD_GObj* g) { T(7002); T(PI(g)); }
static int cbp1(HSD_GObj* g) { T(7003); T(PI(g)); return (int) (fv() & 1); }
static int cbp2(HSD_GObj* g) { T(7004); T(PI(g)); return (int) (fv() & 1); }
static void cbev3(HSD_GObj* g, HSD_GObj* h) { T(7005); T(PI(g)); T(PI(h)); }
static void cbi1(int a) { T(7006); T((u32) a); }
static void cbi2(int a) { T(7007); T((u32) a); }
static void autocb_0(unsigned int a0) { T(9000); T((u32) a0); }
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
static void freg_world(void) { freg((const void*) cbev1); freg((const void*) cbev2); freg((const void*) cbp1); freg((const void*) cbp2); freg((const void*) cbev3); freg((const void*) cbi1); freg((const void*) cbi2); freg((const void*) autocb_0); }
static unsigned char XB[1 << 20];
#include "video_gen.h"



static void preg_world(void)
{
    preg(VIS, sizeof VIS);
    preg(RMO, sizeof RMO);
    preg(VA, sizeof VA); preg(U8A, sizeof U8A); preg(IA, sizeof IA); preg(UA, sizeof UA); preg(FA, sizeof FA); preg(BUF, sizeof BUF); preg(STRS, sizeof STRS); preg(PPA, sizeof PPA);
}

static void setup_world(void)
{
    int i;
    (void) i;
    zero(VIS, sizeof VIS);
    zero(RMO, sizeof RMO);
    for (i = 0; i < 16; i++) { U8A[i] = (u8) rnd(); }
    for (i = 0; i < 4; i++) { IA[i] = (int) fv(); UA[i] = fv(); FA[i] = rw(); VA[i].x = rw(); VA[i].y = rw(); VA[i].z = rw(); }
    for (i = 0; i < 256; i++) BUF[i] = (unsigned char) rnd();
    BUF[255] = 0;
    for (i = 0; i < 4; i++) PPA[i] = 0;
    for (i = 0; i < 2; i++) fill_HSD_VIStatus(&VIS[i]);
    for (i = 0; i < 2; i++) fill_GXRenderModeObj(&RMO[i]);
    fillg_all();
    fillo_all();
}

static void hash_world(void)
{
    int i;
    (void) i;
    for (i = 0; i < 2; i++) hash_HSD_VIStatus(&VIS[i]);
    for (i = 0; i < 2; i++) hash_GXRenderModeObj(&RMO[i]);
    for (i = 0; i < 4; i++) { HM(B(VA[i].x)); HM(B(VA[i].y)); HM(B(VA[i].z)); HM(B(FA[i])); HM((u32) IA[i]); HM(UA[i]); }
    for (i = 0; i < 16; i++) HM(U8A[i]);
    for (i = 0; i < 256; i++) HM(BUF[i]);
    for (i = 0; i < 4; i++) HM(PI(PPA[i]));
    hashg_all();
    hasho_all();
}

enum { OP_HSD_VISearchXFBByStatus = 0, OP_HSD_VISetUserPreRetraceCallback = 1, OP_HSD_VISetUserPostRetraceCallback = 2, OP_HSD_VISetUserGXDrawDoneCallback = 3, OP_HSD_VIPreRetraceCB = 4, OP_HSD_VIPostRetraceCB = 5, OP_HSD_VIGXDrawDoneCB = 6, OP_HSD_VIGetDrawDoneWaitingFlag = 7, OP_HSD_VIGetXFBDrawEnable = 8, OP_HSD_VIWaitXFBDrawEnable = 9, OP_HSD_VICopyEFB2XFBHiResoAA = 10, OP_HSD_VICopyEFB2XFBPtr = 11, OP_HSD_VIGXSetDrawDone = 12, OP_HSD_VISetXFBWaitDone = 13, OP_HSD_VICopyXFBAsync = 14, OP_HSD_VIDrawDoneXFB = 15, OP_HSD_VIWaitXFBFlush_sub = 16, OP_HSD_VIWaitXFBFlush = 17, OP_HSD_VIWaitXFBFlushNoYield = 18, OP_HSD_VIGetXFBLastDrawDone = 19, OP_HSD_VISetConfigure = 20, OP_HSD_VISetBlack = 21, OP_HSD_VIInit = 22 };

int main(int argc, char** argv)
{
    int op, k, only = -1;
    if (argc > 1) only = atoi(argv[1]);
    preg_world();
    preg_own();
    freg_all();
    freg_world();
preg(XB, sizeof XB);
    for (op = 0; op < 23; op++) {
        int n = 0;
        if (only >= 0 && op != only) continue;
        switch (op) {
        case 0: n = 300; break;  /* HSD_VISearchXFBByStatus */
        case 1: n = 300; break;  /* HSD_VISetUserPreRetraceCallback */
        case 2: n = 300; break;  /* HSD_VISetUserPostRetraceCallback */
        case 3: n = 300; break;  /* HSD_VISetUserGXDrawDoneCallback */
        case 4: n = 300; break;  /* HSD_VIPreRetraceCB */
        case 5: n = 300; break;  /* HSD_VIPostRetraceCB */
        case 6: n = 300; break;  /* HSD_VIGXDrawDoneCB */
        case 7: n = 300; break;  /* HSD_VIGetDrawDoneWaitingFlag */
        case 8: n = 300; break;  /* HSD_VIGetXFBDrawEnable */
        case 9: n = 300; break;  /* HSD_VIWaitXFBDrawEnable */
        case 10: n = 300; break;  /* HSD_VICopyEFB2XFBHiResoAA */
        case 11: n = 300; break;  /* HSD_VICopyEFB2XFBPtr */
        case 12: n = 300; break;  /* HSD_VIGXSetDrawDone */
        case 13: n = 300; break;  /* HSD_VISetXFBWaitDone */
        case 14: n = 300; break;  /* HSD_VICopyXFBAsync */
        case 15: n = 300; break;  /* HSD_VIDrawDoneXFB */
        case 16: n = 300; break;  /* HSD_VIWaitXFBFlush_sub */
        case 17: n = 300; break;  /* HSD_VIWaitXFBFlush */
        case 18: n = 300; break;  /* HSD_VIWaitXFBFlushNoYield */
        case 19: n = 300; break;  /* HSD_VIGetXFBLastDrawDone */
        case 20: n = 300; break;  /* HSD_VISetConfigure */
        case 21: n = 300; break;  /* HSD_VISetBlack */
        case 22: n = 300; break;  /* HSD_VIInit */
        }
        for (k = 0; k < n; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup_world();
{ int q; for (q = 0; q < 2; q++) if (fv() & 3) { VIS[q].rmode.fbWidth %= 200; VIS[q].rmode.efbHeight %= 100; RMO[q].fbWidth %= 200; RMO[q].efbHeight %= 100; } for (q = 0; q < 3; q++) { HSD_VIData.xfb[q].status = (HSD_VIXFBDrawDispStatus) (fv() % 9); HSD_VIData.xfb[q].buffer = (fv() & 3) ? (void*) (BUF + 8 * q) : 0; }
 HSD_VIData.efb.status = (fv() % 3); HSD_VIData.nb_xfb = (fv() & 3) ? 1 + fv() % 3 : 0; HSD_VIData.perf.frame_period = fv() % 5; HSD_VIData.drawdone.waiting = fv() & 1; }
            if (getenv("MTU_PROG")) { printf("P %d %d%c", op, k, 10); fflush(0); }
            ARMED = 1;
            if (MTU_SETJMP()) { ARMED = 0; printf("%d %d ABORT\n", op, k); continue; }
            switch (op) {
            case 0: {  HSD_VIXFBDrawDispStatus a0 = (HSD_VIXFBDrawDispStatus) (fv() % 9); ret = (u32) HSD_VISearchXFBByStatus(a0);  } break;
            case 1: {  HSD_VIRetraceCallback a0 = (HSD_VIRetraceCallback) autocb_0; ret = FI((const void*) HSD_VISetUserPreRetraceCallback(a0));  } break;
            case 2: {  HSD_VIRetraceCallback a0 = (HSD_VIRetraceCallback) autocb_0; ret = FI((const void*) HSD_VISetUserPostRetraceCallback(a0));  } break;
            case 3: {  HSD_VIGXDrawDoneCallback a0 = (HSD_VIGXDrawDoneCallback) pick_cb(4); ret = FI((const void*) HSD_VISetUserGXDrawDoneCallback(a0));  } break;
            case 4: {  u32 a0 = fv(); HSD_VIPreRetraceCB(a0);  } break;
            case 5: {  u32 a0 = fv(); HSD_VIPostRetraceCB(a0);  } break;
            case 6: {   HSD_VIGXDrawDoneCB();  } break;
            case 7: {   ret = (u32) HSD_VIGetDrawDoneWaitingFlag();  } break;
            case 8: {   ret = (u32) HSD_VIGetXFBDrawEnable();  } break;
            case 9: {   ret = (u32) HSD_VIWaitXFBDrawEnable();  } break;
            case 10: {  GXRenderModeObj * a0 = pick_GXRenderModeObj(); HSD_VICopyEFB2XFBHiResoAA(a0);  } break;
            case 11: {  HSD_VIStatus * a0 = pick_HSD_VIStatus(); void * a1 = (void*) (XB + 4096); HSD_RenderPass a2 = (HSD_RenderPass) (fv() % 5); HSD_VICopyEFB2XFBPtr(a0, a1, a2);  } break;
            case 12: {  int a0 = fv(); HSD_VIGXSetDrawDone(a0);  } break;
            case 13: {  int a0 = (s32) (fv() % 3); HSD_VISetXFBWaitDone(a0);  } break;
            case 14: {  HSD_RenderPass a0 = (HSD_RenderPass) (fv() % 5); HSD_VICopyXFBAsync(a0);  } break;
            case 15: {  int a0 = (s32) (fv() % 3); HSD_VIDrawDoneXFB(a0);  } break;
            case 16: {   ret = (u32) HSD_VIWaitXFBFlush_sub();  } break;
            case 17: {   HSD_VIWaitXFBFlush();  } break;
            case 18: {   HSD_VIWaitXFBFlushNoYield();  } break;
            case 19: {   ret = (u32) HSD_VIGetXFBLastDrawDone();  } break;
            case 20: {  GXRenderModeObj * a0 = pick_GXRenderModeObj(); HSD_VISetConfigure(a0);  } break;
            case 21: {  bool a0 = (fv() & 1); HSD_VISetBlack(a0);  } break;
            case 22: {  HSD_VIStatus * a0 = pick_HSD_VIStatus(); void * a1 = (void*) BUF; void * a2 = (fv() & 1) ? (void*) (BUF + 8) : 0; void * a3 = (fv() & 1) ? (void*) (BUF + 16) : 0; HSD_VIInit(a0, a1, a2, a3);  } break;
            }
            ARMED = 0;
            hash_world();

            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
