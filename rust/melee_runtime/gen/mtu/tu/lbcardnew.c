extern int atoi(const char*);
extern char* getenv(const char*);
/* TU: src/melee/lb/lbcardnew.c */
/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a
 * recording stub. Written by gen/mtu/mk_lb.py. Each line: op case ret state-hash trace-hash. */

#include TU_SRC
#include "mtu_common.h"
static DVDDiskID DID[1];
static LbCardEntry ENT[12];
static lbCardNew_SnapshotEntry SNP[127];
static Vec3 VA[4];
static u8 U8A[16];
static int IA[4];
static u32 UA[4];
static float FA[4];
static unsigned char BUF[256];
static char STRS[4][64] = { "a.dat", "sym", "LbAd.dat", "x" };

static DVDDiskID* pick_DVDDiskID(void) { u32 r = rnd(); return &DID[(r >> 8) % 1]; }
static DVDDiskID* picko_DVDDiskID(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &DID[(r >> 8) % 1]; }
static LbCardEntry* pick_LbCardEntry(void) { u32 r = rnd(); return &ENT[(r >> 8) % 12]; }
static LbCardEntry* picko_LbCardEntry(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &ENT[(r >> 8) % 12]; }
static lbCardNew_SnapshotEntry* pick_lbCardNew_SnapshotEntry(void) { u32 r = rnd(); return &SNP[(r >> 8) % 127]; }
static lbCardNew_SnapshotEntry* picko_lbCardNew_SnapshotEntry(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &SNP[(r >> 8) % 127]; }
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
static unsigned char WRK[16]; static unsigned char NODES[2100]; static const int CRC[15] = { 0, 0, 0, 0, -1, -2, -3, -4, -5, -6, -13, -14, -128, -7, -9 }; static const int HSR[16] = { 0, 0, 1, -258, -259, -260, -261, -262, -263, -257, -264, -265, -266, -267, -5, -3 };
#include "lbcardnew_gen.h"



static void preg_world(void)
{
    preg(DID, sizeof DID);
    preg(ENT, sizeof ENT);
    preg(SNP, sizeof SNP);
    preg(VA, sizeof VA); preg(U8A, sizeof U8A); preg(IA, sizeof IA); preg(UA, sizeof UA); preg(FA, sizeof FA); preg(BUF, sizeof BUF); preg(STRS, sizeof STRS);
}

static void setup_world(void)
{
    int i;
    (void) i;
    zero(DID, sizeof DID);
    zero(ENT, sizeof ENT);
    zero(SNP, sizeof SNP);
    for (i = 0; i < 16; i++) { U8A[i] = (u8) rnd(); }
    for (i = 0; i < 4; i++) { IA[i] = (int) fv(); UA[i] = fv(); FA[i] = rw(); VA[i].x = rw(); VA[i].y = rw(); VA[i].z = rw(); }
    for (i = 0; i < 256; i++) BUF[i] = (unsigned char) rnd();
    BUF[255] = 0;
    for (i = 0; i < 1; i++) fill_DVDDiskID(&DID[i]);
    for (i = 0; i < 12; i++) fill_LbCardEntry(&ENT[i]);
    for (i = 0; i < 127; i++) fill_lbCardNew_SnapshotEntry(&SNP[i]);
    fillg_all();
    fillo_all();
}

static void hash_world(void)
{
    int i;
    (void) i;
    for (i = 0; i < 1; i++) hash_DVDDiskID(&DID[i]);
    for (i = 0; i < 12; i++) hash_LbCardEntry(&ENT[i]);
    for (i = 0; i < 127; i++) hash_lbCardNew_SnapshotEntry(&SNP[i]);
    for (i = 0; i < 4; i++) { HM(B(VA[i].x)); HM(B(VA[i].y)); HM(B(VA[i].z)); HM(B(FA[i])); HM((u32) IA[i]); HM(UA[i]); }
    for (i = 0; i < 16; i++) HM(U8A[i]);
    for (i = 0; i < 256; i++) HM(BUF[i]);
    hashg_all();
    hasho_all();
}

enum { OP_convertSdkResult = 0, OP_lb_80019C38_noinline = 1, OP_getNewTask = 2, OP_resetTaskArray = 3, OP_executeNextTask = 4, OP_resetState = 5, OP_onCardComplete = 6, OP_convertHsdResult = 7, OP_fn_8001A0B0 = 8, OP_taskMount = 9, OP_taskCheck = 10, OP_setTaskFilename = 11, OP_setupCardEntries = 12, OP_taskOpen = 13, OP_taskUnk3 = 14, OP_taskFormat = 15, OP_taskDelete = 16, OP_taskRename = 17, OP_taskCreate = 18, OP_readCardFileSize = 19, OP_taskRead = 20, OP_taskWrite = 21, OP_taskSetStatus = 22, OP_taskReadHeader = 23, OP_taskListSnapshots = 24, OP_taskFindFile = 25, OP_lb_8001B6E0 = 26, OP_lbCardNew_CompleteNextTask = 27, OP_lbCardNew_CompleteAllTasks = 28, OP_setupTask = 29, OP_lb_8001A4CC_dontinline = 30, OP_lb_8001B7E0 = 31, OP_lb_8001B8C8 = 32, OP_lbCardNew_DeleteSnap = 33, OP_lb_8001BA44 = 34, OP_lb_8001BB48 = 35, OP_lb_8001BC18 = 36, OP_lb_8001BD34 = 37, OP_lb_8001BE30 = 38, OP_lb_8001BF04 = 39, OP_lb_8001BFD8 = 40, OP_lb_8001C0F4 = 41, OP_lb_8001C2D8 = 42, OP_lbCardNew_ProbeEx = 43, OP_lb_8001C4A8 = 44, OP_lbCardNew_AllocWorkArea = 45, OP_lbCardNew_ForgetMemory = 46, OP_lbCardNew_Init = 47 };

int main(int argc, char** argv)
{
    int op, k, only = -1;
    if (argc > 1) only = atoi(argv[1]);
    preg_world();
    preg_own();
    freg_all();
    freg_world();
preg(WRK, sizeof WRK); preg(NODES, sizeof NODES);
    for (op = 0; op < 48; op++) {
        int n = 0;
        if (only >= 0 && op != only) continue;
        switch (op) {
        case 0: n = 300; break;  /* convertSdkResult */
        case 1: n = 300; break;  /* lb_80019C38_noinline */
        case 2: n = 300; break;  /* getNewTask */
        case 3: n = 300; break;  /* resetTaskArray */
        case 4: n = 300; break;  /* executeNextTask */
        case 5: n = 300; break;  /* resetState */
        case 6: n = 300; break;  /* onCardComplete */
        case 7: n = 300; break;  /* convertHsdResult */
        case 8: n = 300; break;  /* fn_8001A0B0 */
        case 9: n = 300; break;  /* taskMount */
        case 10: n = 300; break;  /* taskCheck */
        case 11: n = 300; break;  /* setTaskFilename */
        case 12: n = 300; break;  /* setupCardEntries */
        case 13: n = 300; break;  /* taskOpen */
        case 14: n = 300; break;  /* taskUnk3 */
        case 15: n = 300; break;  /* taskFormat */
        case 16: n = 300; break;  /* taskDelete */
        case 17: n = 300; break;  /* taskRename */
        case 18: n = 300; break;  /* taskCreate */
        case 19: n = 300; break;  /* readCardFileSize */
        case 20: n = 300; break;  /* taskRead */
        case 21: n = 300; break;  /* taskWrite */
        case 22: n = 300; break;  /* taskSetStatus */
        case 23: n = 300; break;  /* taskReadHeader */
        case 24: n = 300; break;  /* taskListSnapshots */
        case 25: n = 300; break;  /* taskFindFile */
        case 26: n = 300; break;  /* lb_8001B6E0 */
        case 27: n = 300; break;  /* lbCardNew_CompleteNextTask */
        case 28: n = 300; break;  /* lbCardNew_CompleteAllTasks */
        case 29: n = 300; break;  /* setupTask */
        case 30: n = 300; break;  /* lb_8001A4CC_dontinline */
        case 31: n = 300; break;  /* lb_8001B7E0 */
        case 32: n = 300; break;  /* lb_8001B8C8 */
        case 33: n = 300; break;  /* lbCardNew_DeleteSnap */
        case 34: n = 300; break;  /* lb_8001BA44 */
        case 35: n = 300; break;  /* lb_8001BB48 */
        case 36: n = 300; break;  /* lb_8001BC18 */
        case 37: n = 300; break;  /* lb_8001BD34 */
        case 38: n = 300; break;  /* lb_8001BE30 */
        case 39: n = 300; break;  /* lb_8001BF04 */
        case 40: n = 300; break;  /* lb_8001BFD8 */
        case 41: n = 300; break;  /* lb_8001C0F4 */
        case 42: n = 300; break;  /* lb_8001C2D8 */
        case 43: n = 300; break;  /* lbCardNew_ProbeEx */
        case 44: n = 300; break;  /* lb_8001C4A8 */
        case 45: n = 300; break;  /* lbCardNew_AllocWorkArea */
        case 46: n = 300; break;  /* lbCardNew_ForgetMemory */
        case 47: n = 300; break;  /* lbCardNew_Init */
        }
        for (k = 0; k < n; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup_world();
{ int q; state.work_area = (op == OP_lbCardNew_AllocWorkArea && (fv() & 1)) ? 0 : (void*) WRK; state.lib_area = (op == OP_lbCardNew_AllocWorkArea && (fv() & 1)) ? 0 : (void*) NODES; state.save_data = BUF; state.status = (fv() & 1) ? &IA[0] : 0; state.comment = (char*) BUF + 16; state.banner = BUF + 64; state.icons = BUF + 128; state.snapshot_entries = SNP; state.free_blocks = (fv() & 1) ? &IA[1] : 0; state.free_files = (fv() & 1) ? &IA[2] : 0; state.sectorsize = (fv() & 1) ? 0x2000 : (int) fv(); state.unused_bytes = (int) (fv() % 0x400000); state.unused_files = (int) (fv() % 4); state.mounted = (int) (fv() & 1); state.tasks_remaining = (int) (fv() % 4) - 1; state.saved_error = (int) (fv() % 18); state.chan = (int) (fv() % 3); for (q = 0; q < 11; q++) state.task_array[q].type = fv() % 16; state.task_array[fv() % 11].type = 14; for (q = 0; q < 9; q++) { ENT[q].file_size = (int) (fv() % 1000); ENT[q].data = BUF + 16 * q; } ENT[9].file_size = -1; for (q = 0; q < 10; q++) state.task_array[q].file_entries = ENT; }
            if (getenv("MTU_PROG")) { printf("P %d %d%c", op, k, 10); fflush(0); }
            switch (op) {
            case 0: {  int a0 = ((fv() & 1) ? CRC[fv() % 15] : (int) fv()); ret = (u32) convertSdkResult(a0);  } break;
            case 1: {   ret = PI(lb_80019C38_noinline());  } break;
            case 2: {   ret = PI(getNewTask());  } break;
            case 3: {   resetTaskArray();  } break;
            case 4: {  lbCardResult a0 = (int) (fv() % 18); ret = (u32) executeNextTask(a0);  } break;
            case 5: {  int a0 = fv(); void * a1 = (void*) BUF; int * a2 = IA; LbCardOnFinishedCallback a3 = (LbCardOnFinishedCallback) pick_cb(4); resetState(a0, a1, a2, a3);  } break;
            case 6: {  s32 a0 = fv(); s32 a1 = ((fv() & 1) ? CRC[fv() % 15] : (int) fv()); onCardComplete(a0, a1);  } break;
            case 7: {  int a0 = ((fv() & 1) ? HSR[fv() % 16] : (int) fv()); ret = (u32) convertHsdResult(a0);  } break;
            case 8: {  int a0 = (int) (fv() % 9); int a1 = ((fv() & 1) ? HSR[fv() % 16] : (int) fv()); fn_8001A0B0(a0, a1);  } break;
            case 9: {   ret = (u32) taskMount();  } break;
            case 10: {   ret = (u32) taskCheck();  } break;
            case 11: {  const char * a0 = STRS[rnd() % 4]; void * a1 = (void*) ENT; setTaskFilename(a0, a1);  } break;
            case 12: {  CardState * a0 = &state.card_state; void * a1 = (void*) BUF; LbCardEntry * a2 = ENT; setupCardEntries(a0, a1, a2);  } break;
            case 13: {  const char * a0 = STRS[rnd() % 4]; LbCardEntry * a1 = ENT; ret = (u32) taskOpen(a0, a1);  } break;
            case 14: {   ret = (u32) taskUnk3();  } break;
            case 15: {   ret = (u32) taskFormat();  } break;
            case 16: {  char * a0 = STRS[rnd() % 4]; ret = (u32) taskDelete(a0);  } break;
            case 17: {  const char * a0 = STRS[rnd() % 4]; const char * a1 = STRS[rnd() % 4]; ret = (u32) taskRename(a0, a1);  } break;
            case 18: {  const char * a0 = STRS[rnd() % 4]; ret = (u32) taskCreate(a0);  } break;
            case 19: {  const volatile int * a0 = IA; ret = (u32) readCardFileSize(a0);  } break;
            case 20: {  LbCardEntry * a0 = ENT; ret = (u32) taskRead(a0);  } break;
            case 21: {  LbCardEntry * a0 = ENT; ret = (u32) taskWrite(a0);  } break;
            case 22: {   ret = (u32) taskSetStatus();  } break;
            case 23: {   ret = (u32) taskReadHeader();  } break;
            case 24: {   ret = (u32) taskListSnapshots();  } break;
            case 25: {  const char * a0 = STRS[rnd() % 4]; ret = (u32) taskFindFile(a0);  } break;
            case 26: {  s32 a0 = (int) (fv() % 9); ret = (u32) lb_8001B6E0(a0);  } break;
            case 27: {   ret = (u32) lbCardNew_CompleteNextTask();  } break;
            case 28: {  int a0 = (int) (fv() % 14); ret = (u32) lbCardNew_CompleteAllTasks(a0);  } break;
            case 29: {  CardTaskType a0 = fv(); int a1 = fv(); ret = PI(setupTask(a0, a1));  } break;
            case 30: {  const char * a0 = STRS[rnd() % 4]; void * a1 = (void*) ENT; lb_8001A4CC_dontinline(a0, a1);  } break;
            case 31: {  int a0 = fv(); char * a1 = STRS[rnd() % 4]; void * a2 = (void*) ENT; void * a3 = (void*) BUF; int * a4 = IA; ret = (u32) lb_8001B7E0(a0, a1, a2, a3, a4);  } break;
            case 32: {  int a0 = fv(); ret = (u32) lb_8001B8C8(a0);  } break;
            case 33: {  int a0 = fv(); const char * a1 = STRS[rnd() % 4]; void * a2 = (void*) BUF; ret = (u32) lbCardNew_DeleteSnap(a0, a1, a2);  } break;
            case 34: {  int a0 = fv(); const char * a1 = STRS[rnd() % 4]; void * a2 = (void*) BUF; ret = (u32) lb_8001BA44(a0, a1, a2);  } break;
            case 35: {  int a0 = fv(); char * a1 = STRS[rnd() % 4]; void * a2 = (void*) ENT; void * a3 = (void*) BUF; char * a4 = STRS[rnd() % 4]; void * a5 = (void*) BUF; void * a6 = (void*) BUF; void * a7 = (void*) BUF; ret = (u32) lb_8001BB48(a0, a1, a2, a3, a4, a5, a6, a7);  } break;
            case 36: {  int a0 = fv(); char * a1 = STRS[rnd() % 4]; void ** a2 = (void**) ENT; void * a3 = (void*) BUF; char * a4 = STRS[rnd() % 4]; void * a5 = (void*) BUF; void * a6 = (void*) BUF; bool * a7 = IA; ret = (u32) lb_8001BC18(a0, a1, a2, a3, a4, a5, a6, a7);  } break;
            case 37: {  int a0 = fv(); const char * a1 = STRS[rnd() % 4]; void * a2 = (void*) ENT; void * a3 = (void*) BUF; ret = (u32) lb_8001BD34(a0, a1, a2, a3);  } break;
            case 38: {  int a0 = fv(); const char * a1 = STRS[rnd() % 4]; void * a2 = (void*) ENT; char * a3 = STRS[rnd() % 4]; void * a4 = (void*) BUF; void * a5 = (void*) BUF; void * a6 = (void*) BUF; void * a7 = (void*) pick_cb(4); ret = (u32) lb_8001BE30(a0, a1, a2, a3, a4, a5, a6, a7);  } break;
            case 39: {  int a0 = fv(); char * a1 = STRS[rnd() % 4]; void * a2 = (void*) ENT; char * a3 = STRS[rnd() % 4]; void * a4 = (void*) BUF; void * a5 = (void*) BUF; void * a6 = (void*) BUF; ret = (u32) lb_8001BF04(a0, a1, a2, a3, a4, a5, a6);  } break;
            case 40: {  int a0 = fv(); lbCardNew_SnapshotEntry * a1 = SNP; int * a2 = IA; int * a3 = IA; ret = (u32) lb_8001BFD8(a0, a1, a2, a3);  } break;
            case 41: {  int a0 = fv(); const char * a1 = STRS[rnd() % 4]; const char * a2 = STRS[rnd() % 4]; const char * a3 = STRS[rnd() % 4]; void * a4 = (void*) BUF; ret = (u32) lb_8001C0F4(a0, a1, a2, a3, a4);  } break;
            case 42: {  int a0 = fv(); const char * a1 = STRS[rnd() % 4]; const char * a2 = STRS[rnd() % 4]; const char * a3 = STRS[rnd() % 4]; ret = (u32) lb_8001C2D8(a0, a1, a2, a3);  } break;
            case 43: {  int a0 = fv(); ret = (u32) lbCardNew_ProbeEx(a0);  } break;
            case 44: {  void * a0 = (void*) ENT; void * a1 = (void*) BUF; ret = (u32) lb_8001C4A8(a0, a1);  } break;
            case 45: {   lbCardNew_AllocWorkArea();  } break;
            case 46: {   lbCardNew_ForgetMemory();  } break;
            case 47: {   lbCardNew_Init();  } break;
            }
            hash_world();

            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
