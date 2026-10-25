extern int atoi(const char*);
extern char* getenv(const char*);
/* TU: src/sysdolphin/baselib/rumble.c */
/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a
 * recording stub. Written by gen/mtu/mk_lb.py. Each line: op case ret state-hash trace-hash. */

#include TU_SRC
#include "mtu_common.h"
static HSD_PadRumbleListData RL[8];
static HSD_Rumble CM[40];
static struct { float x, y, z; } VA[4];
static u8 U8A[16];
static int IA[4];
static u32 UA[4];
static float FA[4];
static unsigned char BUF[256];
static char STRS[4][64] = { "a.dat", "sym", "LbAd.dat", "x" };

static HSD_PadRumbleListData* pick_HSD_PadRumbleListData(void) { u32 r = rnd(); return &RL[(r >> 8) % 8]; }
static HSD_PadRumbleListData* picko_HSD_PadRumbleListData(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &RL[(r >> 8) % 8]; }
static HSD_Rumble* pick_HSD_Rumble(void) { u32 r = rnd(); return &CM[(r >> 8) % 40]; }
static HSD_Rumble* picko_HSD_Rumble(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &CM[(r >> 8) % 40]; }
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

#include "rumble_gen.h"



static void preg_world(void)
{
    preg(RL, sizeof RL);
    preg(CM, sizeof CM);
    preg(VA, sizeof VA); preg(U8A, sizeof U8A); preg(IA, sizeof IA); preg(UA, sizeof UA); preg(FA, sizeof FA); preg(BUF, sizeof BUF); preg(STRS, sizeof STRS);
}

static void setup_world(void)
{
    int i;
    (void) i;
    zero(RL, sizeof RL);
    zero(CM, sizeof CM);
    for (i = 0; i < 16; i++) { U8A[i] = (u8) rnd(); }
    for (i = 0; i < 4; i++) { IA[i] = (int) fv(); UA[i] = fv(); FA[i] = rw(); VA[i].x = rw(); VA[i].y = rw(); VA[i].z = rw(); }
    for (i = 0; i < 256; i++) BUF[i] = (unsigned char) rnd();
    BUF[255] = 0;
    for (i = 0; i < 8; i++) fill_HSD_PadRumbleListData(&RL[i]);
    for (i = 0; i < 40; i++) fill_HSD_Rumble(&CM[i]);
    fillg_all();
    fillo_all();
}

static void hash_world(void)
{
    int i;
    (void) i;
    for (i = 0; i < 8; i++) hash_HSD_PadRumbleListData(&RL[i]);
    for (i = 0; i < 40; i++) hash_HSD_Rumble(&CM[i]);
    for (i = 0; i < 4; i++) { HM(B(VA[i].x)); HM(B(VA[i].y)); HM(B(VA[i].z)); HM(B(FA[i])); HM((u32) IA[i]); HM(UA[i]); }
    for (i = 0; i < 16; i++) HM(U8A[i]);
    for (i = 0; i < 256; i++) HM(BUF[i]);
    hashg_all();
    hasho_all();
}

enum { OP_HSD_PadRumbleOn = 0, OP_HSD_PadRumbleOffN = 1, OP_HSD_PadRumbleRemove = 2, OP_HSD_PadRumbleRemoveAll = 3, OP_HSD_PadRumbleRemoveId = 4, OP_HSD_PadRumblePause = 5, OP_HSD_PadRumblePauseAll = 6, OP_HSD_PadRumbleUnpauseAll = 7, OP_HSD_PadRumbleAdd = 8, OP_HSD_Rumble_80378524 = 9, OP_HSD_PadRumbleInterpret1 = 10, OP_HSD_PadRumbleInterpret = 11, OP_HSD_PadRumbleInit = 12 };

int main(int argc, char** argv)
{
    int op, k, only = -1;
    if (argc > 1) only = atoi(argv[1]);
    preg_world();
    preg_own();
    freg_all();
    freg_world();

    for (op = 0; op < 13; op++) {
        int n = 0;
        if (only >= 0 && op != only) continue;
        switch (op) {
        case 0: n = 300; break;  /* HSD_PadRumbleOn */
        case 1: n = 300; break;  /* HSD_PadRumbleOffN */
        case 2: n = 300; break;  /* HSD_PadRumbleRemove */
        case 3: n = 300; break;  /* HSD_PadRumbleRemoveAll */
        case 4: n = 300; break;  /* HSD_PadRumbleRemoveId */
        case 5: n = 300; break;  /* HSD_PadRumblePause */
        case 6: n = 300; break;  /* HSD_PadRumblePauseAll */
        case 7: n = 300; break;  /* HSD_PadRumbleUnpauseAll */
        case 8: n = 300; break;  /* HSD_PadRumbleAdd */
        case 9: n = 300; break;  /* HSD_Rumble_80378524 */
        case 10: n = 300; break;  /* HSD_PadRumbleInterpret1 */
        case 11: n = 300; break;  /* HSD_PadRumbleInterpret */
        case 12: n = 300; break;  /* HSD_PadRumbleInit */
        }
        for (k = 0; k < n; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup_world();
{ int q, ch, cnt[4]; HSD_PadRumbleListData* fr = 0; HSD_PadRumbleListData* tail[4]; for (q = 0; q < 40; q++) { CM[q].command.op = (q >= 39) ? 0 : (q % 4 == 1) ? 4 : (q % 4 == 3) ? 5 : 1 + fv() % 3; CM[q].command.frame = 1 + fv() % 6; } for (ch = 0; ch < 4; ch++) { cnt[ch] = 0; tail[ch] = 0; HSD_Rumble_804C22E0[ch].listdatap = 0; } for (q = 0; q < 8; q++) { u32 f; RL[q].next = 0; RL[q].listp = &CM[fv() % 30]; RL[q].headp = RL[q].listp; RL[q].stack = &CM[fv() % 30]; RL[q].pause = (u8) ((fv() & 3) == 0); RL[q].id = fv() % 6; RL[q].pri = (u8) (fv() % 5); RL[q].loop_count = (u16) (fv() % 5); RL[q].wait = (u16) (fv() % 3); f = fv() % 5; RL[q].frame = f == 0 ? -1 : f == 1 ? -2 : (s32) (1 + fv() % 6); ch = (int) (fv() % 5); if (ch == 4) { RL[q].next = fr; fr = &RL[q]; } else { if (tail[ch]) tail[ch]->next = &RL[q]; else HSD_Rumble_804C22E0[ch].listdatap = &RL[q]; tail[ch] = &RL[q]; cnt[ch]++; } } for (ch = 0; ch < 4; ch++) HSD_Rumble_804C22E0[ch].nb_list = (u16) cnt[ch]; HSD_PadLibData.rumble_info.listdatap = fr; HSD_PadLibData.rumble_info.max_list = (u16) (1 + fv() % 8); HSD_PadLibData.rumble_info.unk2 = (u8) ((fv() & 3) == 0); }
            if (getenv("MTU_PROG")) { printf("P %d %d%c", op, k, 10); fflush(0); }
            ARMED = 1;
            if (MTU_SETJMP()) { ARMED = 0; printf("%d %d ABORT\n", op, k); continue; }
            switch (op) {
            case 0: {  u8 a0 = (u8) (fv() % 4); HSD_PadRumbleOn(a0);  } break;
            case 1: {  u8 a0 = (u8) (fv() % 4); HSD_PadRumbleOffN(a0);  } break;
            case 2: {  u8 a0 = (u8) (fv() % 4); HSD_PadRumbleRemove(a0);  } break;
            case 3: {   HSD_PadRumbleRemoveAll();  } break;
            case 4: {  u8 a0 = (u8) (fv() % 4); int a1 = (int) (fv() % 6); HSD_PadRumbleRemoveId(a0, a1);  } break;
            case 5: {  u8 a0 = (u8) (fv() % 4); int a1 = (int) (fv() % 3); HSD_PadRumblePause(a0, a1);  } break;
            case 6: {   HSD_PadRumblePauseAll();  } break;
            case 7: {   HSD_PadRumbleUnpauseAll();  } break;
            case 8: {  u8 a0 = (u8) (fv() % 4); int a1 = (int) (fv() % 6); int a2 = fv(); int a3 = (int) (fv() % 5); void * a4 = (void*) &CM[fv() % 30]; ret = (u32) HSD_PadRumbleAdd(a0, a1, a2, a3, a4);  } break;
            case 9: {  int a0 = fv(); HSD_Rumble_80378524(a0);  } break;
            case 10: {  HSD_PadRumbleListData * a0 = pick_HSD_PadRumbleListData(); u8 * a1 = U8A; ret = (u32) HSD_PadRumbleInterpret1(a0, a1);  } break;
            case 11: {   HSD_PadRumbleInterpret();  } break;
            case 12: {  u16 a0 = (u16) (fv() % 9); void * a1 = (void*) RL; HSD_PadRumbleInit(a0, a1);  } break;
            }
            ARMED = 0;
            hash_world();

            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
