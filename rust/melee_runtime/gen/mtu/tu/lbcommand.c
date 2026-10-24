extern int atoi(const char*);
extern char* getenv(const char*);
/* TU: src/melee/lb/lbcommand.c */
/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a
 * recording stub. Written by gen/mtu/mk_lb.py. Each line: op case ret state-hash trace-hash. */

#include TU_SRC
#include "mtu_common.h"
static Vec3 VA[4];
static u8 U8A[16];
static int IA[4];
static u32 UA[4];
static float FA[4];
static unsigned char BUF[256];
static char STRS[4][64] = { "a.dat", "sym", "LbAd.dat", "x" };

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

#include "lbcommand_gen.h"

static CommandInfo CI[3]; static CmdUnion CU[20]; static u32 CMD;
static u32 HP(const void* p) { unsigned long long v = (unsigned long long) p; return (v >> 32) == 0 ? (u32) v : PI(p); }
static void setcu(u32 cmd)
{
    int q;
    for (q = 0; q < 16; q++) {
        zero(&CU[q], sizeof CU[q]);
        switch (cmd) {
        case 1: CU[q].Command_00.value = fv(); break;
        case 2: CU[q].Command_02.value = fv(); break;
        case 3: CU[q].Command_03.value = fv() % 5; break;
        case 5: case 7: CU[q].Command_05.ptr = &CU[fv() % 16]; break;
        case 9: CU[q].Command_09.id = fv(); CU[q].Command_09.param_1 = fv(); CU[q].Command_09.param_2 = fv(); break;
        default: CU[q].Command_04.x = fv(); break;
        }
    }
}
static void setci(u32 cmd)
{
    int q;
    for (q = 0; q < 3; q++) {
        CommandInfo* c = &CI[q];
        zero(c, sizeof *c);
        c->timer = rw(); c->frame_count = rw(); c->u = &CU[fv() % 16];
        c->loop_count = cmd == 3 ? fv() % 2 : cmd == 4 ? 2 : cmd == 5 ? fv() % 3 : cmd == 6 ? 1 + fv() % 3 : fv() % 4;
        c->event_return[0] = &CU[fv() % 16]; c->event_return[1] = (CmdUnion*) (unsigned long long) (fv() % 4); c->event_return[2] = &CU[fv() % 16];
        c->loop_count_dup = fv(); c->unk_x18 = fv();
    }
}



static void preg_world(void)
{
    preg(VA, sizeof VA); preg(U8A, sizeof U8A); preg(IA, sizeof IA); preg(UA, sizeof UA); preg(FA, sizeof FA); preg(BUF, sizeof BUF); preg(STRS, sizeof STRS);
}

static void setup_world(void)
{
    int i;
    (void) i;
    for (i = 0; i < 16; i++) { U8A[i] = (u8) rnd(); }
    for (i = 0; i < 4; i++) { IA[i] = (int) fv(); UA[i] = fv(); FA[i] = rw(); VA[i].x = rw(); VA[i].y = rw(); VA[i].z = rw(); }
    for (i = 0; i < 256; i++) BUF[i] = (unsigned char) rnd();
    BUF[255] = 0;
    fillg_all();
    fillo_all();
}

static void hash_world(void)
{
    int i;
    (void) i;
    for (i = 0; i < 4; i++) { HM(B(VA[i].x)); HM(B(VA[i].y)); HM(B(VA[i].z)); HM(B(FA[i])); HM((u32) IA[i]); HM(UA[i]); }
    for (i = 0; i < 16; i++) HM(U8A[i]);
    for (i = 0; i < 256; i++) HM(BUF[i]);
    hashg_all();
    hasho_all();
}

enum { OP_Command_00 = 0, OP_Command_01 = 1, OP_Command_02 = 2, OP_Command_03 = 3, OP_Command_04 = 4, OP_Command_05 = 5, OP_Command_06 = 6, OP_Command_07 = 7, OP_Command_08 = 8, OP_Command_09 = 9, OP_Command_Execute = 10 };

int main(int argc, char** argv)
{
    int op, k, only = -1;
    if (argc > 1) only = atoi(argv[1]);
    preg_world();
    preg_own();
    freg_all();
    freg_world();
preg(CI, sizeof CI); preg(CU, sizeof CU);
    for (op = 0; op < 11; op++) {
        int n = 0;
        if (only >= 0 && op != only) continue;
        switch (op) {
        case 0: n = 300; break;  /* Command_00 */
        case 1: n = 300; break;  /* Command_01 */
        case 2: n = 300; break;  /* Command_02 */
        case 3: n = 300; break;  /* Command_03 */
        case 4: n = 300; break;  /* Command_04 */
        case 5: n = 300; break;  /* Command_05 */
        case 6: n = 300; break;  /* Command_06 */
        case 7: n = 300; break;  /* Command_07 */
        case 8: n = 300; break;  /* Command_08 */
        case 9: n = 300; break;  /* Command_09 */
        case 10: n = 300; break;  /* Command_Execute */
        }
        for (k = 0; k < n; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup_world();
{ CMD = op < 10 ? (u32) op : fv() % 12; setcu(CMD); setci(CMD); }
            if (getenv("MTU_PROG")) { printf("P %d %d%c", op, k, 10); fflush(0); }
            switch (op) {
            case 0: {  CommandInfo * a0 = &CI[fv() % 3]; Command_00(a0);  } break;
            case 1: {  CommandInfo * a0 = &CI[fv() % 3]; Command_01(a0);  } break;
            case 2: {  CommandInfo * a0 = &CI[fv() % 3]; Command_02(a0);  } break;
            case 3: {  CommandInfo * a0 = &CI[fv() % 3]; Command_03(a0);  } break;
            case 4: {  CommandInfo * a0 = &CI[fv() % 3]; Command_04(a0);  } break;
            case 5: {  CommandInfo * a0 = &CI[fv() % 3]; Command_05(a0);  } break;
            case 6: {  CommandInfo * a0 = &CI[fv() % 3]; Command_06(a0);  } break;
            case 7: {  CommandInfo * a0 = &CI[fv() % 3]; Command_07(a0);  } break;
            case 8: {  CommandInfo * a0 = &CI[fv() % 3]; Command_08(a0);  } break;
            case 9: {  CommandInfo * a0 = &CI[fv() % 3]; Command_09(a0);  } break;
            case 10: {  CommandInfo * a0 = &CI[fv() % 3]; u32 a1 = CMD; ret = (u32) Command_Execute(a0, a1);  } break;
            }
            hash_world();
{ int q, k; for (q = 0; q < 3; q++) { CommandInfo* c = &CI[q]; HM(B(c->timer)); HM(B(c->frame_count)); HM(PI(c->u)); HM(c->loop_count); for (k = 0; k < 3; k++) HM(HP(c->event_return[k])); HM(c->loop_count_dup); HM(c->unk_x18); } }
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
