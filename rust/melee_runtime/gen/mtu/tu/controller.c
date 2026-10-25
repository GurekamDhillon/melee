extern int atoi(const char*);
extern char* getenv(const char*);
/* TU: src/sysdolphin/baselib/controller.c */
/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a
 * recording stub. Written by gen/mtu/mk_lb.py. Each line: op case ret state-hash trace-hash. */

#include TU_SRC
#include "mtu_common.h"
static HSD_PadData PQ[16];
static HSD_PadRumbleListData RL[2];
static struct { float x, y, z; } VA[4];
static u8 U8A[16];
static int IA[4];
static u32 UA[4];
static float FA[4];
static unsigned char BUF[256];
static char STRS[4][64] = { "a.dat", "sym", "LbAd.dat", "x" };

static HSD_PadData* pick_HSD_PadData(void) { u32 r = rnd(); return &PQ[(r >> 8) % 16]; }
static HSD_PadData* picko_HSD_PadData(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &PQ[(r >> 8) % 16]; }
static HSD_PadRumbleListData* pick_HSD_PadRumbleListData(void) { u32 r = rnd(); return &RL[(r >> 8) % 2]; }
static HSD_PadRumbleListData* picko_HSD_PadRumbleListData(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &RL[(r >> 8) % 2]; }
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

#include "controller_gen.h"



static void preg_world(void)
{
    preg(PQ, sizeof PQ);
    preg(RL, sizeof RL);
    preg(VA, sizeof VA); preg(U8A, sizeof U8A); preg(IA, sizeof IA); preg(UA, sizeof UA); preg(FA, sizeof FA); preg(BUF, sizeof BUF); preg(STRS, sizeof STRS);
}

static void setup_world(void)
{
    int i;
    (void) i;
    zero(PQ, sizeof PQ);
    zero(RL, sizeof RL);
    for (i = 0; i < 16; i++) { U8A[i] = (u8) rnd(); }
    for (i = 0; i < 4; i++) { IA[i] = (int) fv(); UA[i] = fv(); FA[i] = rw(); VA[i].x = rw(); VA[i].y = rw(); VA[i].z = rw(); }
    for (i = 0; i < 256; i++) BUF[i] = (unsigned char) rnd();
    BUF[255] = 0;
    for (i = 0; i < 16; i++) fill_HSD_PadData(&PQ[i]);
    for (i = 0; i < 2; i++) fill_HSD_PadRumbleListData(&RL[i]);
    fillg_all();
    fillo_all();
}

static void hash_world(void)
{
    int i;
    (void) i;
    for (i = 0; i < 16; i++) hash_HSD_PadData(&PQ[i]);
    for (i = 0; i < 2; i++) hash_HSD_PadRumbleListData(&RL[i]);
    for (i = 0; i < 4; i++) { HM(B(VA[i].x)); HM(B(VA[i].y)); HM(B(VA[i].z)); HM(B(FA[i])); HM((u32) IA[i]); HM(UA[i]); }
    for (i = 0; i < 16; i++) HM(U8A[i]);
    for (i = 0; i < 256; i++) HM(BUF[i]);
    hashg_all();
    hasho_all();
}

enum { OP_HSD_PadGetRawQueueCount = 0, OP_HSD_PadGetResetSwitch = 1, OP_HSD_PadRenewRawStatus = 2, OP_HSD_PadFlushQueue = 3, OP_HSD_PadRenewMasterStatus = 4, OP_HSD_PadRenewCopyStatus = 5, OP_HSD_PadRenewGameStatus = 6, OP_HSD_PadRenewStatus = 7, OP_HSD_PadReset = 8, OP_HSD_PadInit = 9 };

int main(int argc, char** argv)
{
    int op, k, only = -1;
    if (argc > 1) only = atoi(argv[1]);
    preg_world();
    preg_own();
    freg_all();
    freg_world();

    for (op = 0; op < 10; op++) {
        int n = 0;
        if (only >= 0 && op != only) continue;
        switch (op) {
        case 0: n = 400; break;  /* HSD_PadGetRawQueueCount */
        case 1: n = 400; break;  /* HSD_PadGetResetSwitch */
        case 2: n = 400; break;  /* HSD_PadRenewRawStatus */
        case 3: n = 400; break;  /* HSD_PadFlushQueue */
        case 4: n = 400; break;  /* HSD_PadRenewMasterStatus */
        case 5: n = 400; break;  /* HSD_PadRenewCopyStatus */
        case 6: n = 400; break;  /* HSD_PadRenewGameStatus */
        case 7: n = 400; break;  /* HSD_PadRenewStatus */
        case 8: n = 400; break;  /* HSD_PadReset */
        case 9: n = 400; break;  /* HSD_PadInit */
        }
        for (k = 0; k < n; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup_world();
{ HSD_PadLibData.qnum = (u8) (1 + fv() % 4); HSD_PadLibData.qread = (u8) (fv() % HSD_PadLibData.qnum); HSD_PadLibData.qwrite = (u8) (fv() % HSD_PadLibData.qnum); HSD_PadLibData.qcount = (u8) (fv() % (HSD_PadLibData.qnum + 1)); HSD_PadLibData.qtype = (u8) (fv() % 4); HSD_PadLibData.queue = PQ; HSD_PadLibData.clamp_stickType = (u8) ((fv() & 3) == 0 ? 1 : 0); HSD_PadLibData.adc_type = (s8) ((fv() & 3) == 0 ? 1 : 0); HSD_PadLibData.adc_th = (s8) (fv() % 50); HSD_PadLibData.adc_angle = rw(); HSD_PadLibData.cross_dir = (u8) (fv() % 5); HSD_PadLibData.clamp_stickMax = (s8) (30 + fv() % 98); HSD_PadLibData.clamp_stickMin = (s8) (fv() % 30); HSD_PadLibData.clamp_stickShift = (u8) (fv() % 3); HSD_PadLibData.repeat_start = (s32) (1 + fv() % 40); HSD_PadLibData.repeat_interval = (s32) (1 + fv() % 20); HSD_PadLibData.scale_stick = (s8) (fv() % 128); HSD_PadLibData.reset_switch_status = (u8) (fv() & 1); HSD_PadLibData.reset_switch = (u8) (fv() & 1); }
            if (getenv("MTU_PROG")) { printf("P %d %d%c", op, k, 10); fflush(0); }
            ARMED = 1;
            if (MTU_SETJMP()) { ARMED = 0; printf("%d %d ABORT\n", op, k); continue; }
            switch (op) {
            case 0: {   ret = (u32) HSD_PadGetRawQueueCount();  } break;
            case 1: {   ret = (u32) HSD_PadGetResetSwitch();  } break;
            case 2: {  bool a0 = (fv() & 1); HSD_PadRenewRawStatus(a0);  } break;
            case 3: {  HSD_FlushType a0 = fv(); HSD_PadFlushQueue(a0);  } break;
            case 4: {   HSD_PadRenewMasterStatus();  } break;
            case 5: {   HSD_PadRenewCopyStatus();  } break;
            case 6: {   HSD_PadRenewGameStatus();  } break;
            case 7: {   HSD_PadRenewStatus();  } break;
            case 8: {   HSD_PadReset();  } break;
            case 9: {  u8 a0 = fv(); HSD_PadData * a1 = pick_HSD_PadData(); u16 a2 = fv(); HSD_PadRumbleListData * a3 = pick_HSD_PadRumbleListData(); HSD_PadInit(a0, a1, a2, a3);  } break;
            }
            ARMED = 0;
            hash_world();

            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
