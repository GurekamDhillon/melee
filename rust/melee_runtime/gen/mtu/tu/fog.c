extern int atoi(const char*);
extern char* getenv(const char*);
/* TU: src/sysdolphin/baselib/fog.c */
/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a
 * recording stub. Written by gen/mtu/mk_lb.py. Each line: op case ret state-hash trace-hash. */

#include TU_SRC
#include "mtu_common.h"
static HSD_Fog FG[3];
static HSD_FogAdj FGA[3];
static HSD_FogDesc FD[2];
static HSD_FogAdjDesc FAD[2];
static HSD_AObj AO[2];
static HSD_AObjDesc AD[2];
static HSD_ObjData OBD[2];
static HSD_ClassInfo CI[3];
static HSD_FogInfo FIN[2];
static Vec3 VA[4];
static u8 U8A[16];
static int IA[4];
static u32 UA[4];
static float FA[4];
static unsigned char BUF[256];
static void* PPA[4];
static char STRS[4][64] = { "a.dat", "sym", "LbAd.dat", "x" };

static HSD_Fog* pick_HSD_Fog(void) { u32 r = rnd(); return &FG[(r >> 8) % 3]; }
static HSD_Fog* picko_HSD_Fog(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &FG[(r >> 8) % 3]; }
static HSD_FogAdj* pick_HSD_FogAdj(void) { u32 r = rnd(); return &FGA[(r >> 8) % 3]; }
static HSD_FogAdj* picko_HSD_FogAdj(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &FGA[(r >> 8) % 3]; }
static HSD_FogDesc* pick_HSD_FogDesc(void) { u32 r = rnd(); return &FD[(r >> 8) % 2]; }
static HSD_FogDesc* picko_HSD_FogDesc(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &FD[(r >> 8) % 2]; }
static HSD_FogAdjDesc* pick_HSD_FogAdjDesc(void) { u32 r = rnd(); return &FAD[(r >> 8) % 2]; }
static HSD_FogAdjDesc* picko_HSD_FogAdjDesc(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &FAD[(r >> 8) % 2]; }
static HSD_AObj* pick_HSD_AObj(void) { u32 r = rnd(); return &AO[(r >> 8) % 2]; }
static HSD_AObj* picko_HSD_AObj(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &AO[(r >> 8) % 2]; }
static HSD_AObjDesc* pick_HSD_AObjDesc(void) { u32 r = rnd(); return &AD[(r >> 8) % 2]; }
static HSD_AObjDesc* picko_HSD_AObjDesc(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &AD[(r >> 8) % 2]; }
static HSD_ObjData* pick_HSD_ObjData(void) { u32 r = rnd(); return &OBD[(r >> 8) % 2]; }
static HSD_ObjData* picko_HSD_ObjData(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &OBD[(r >> 8) % 2]; }
static HSD_ClassInfo* pick_HSD_ClassInfo(void) { u32 r = rnd(); return &CI[(r >> 8) % 3]; }
static HSD_ClassInfo* picko_HSD_ClassInfo(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &CI[(r >> 8) % 3]; }
static HSD_FogInfo* pick_HSD_FogInfo(void) { u32 r = rnd(); return &FIN[(r >> 8) % 2]; }
static HSD_FogInfo* picko_HSD_FogInfo(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &FIN[(r >> 8) % 2]; }
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

static void cb_rel(HSD_Class* c) { T(8203); T(PI(c)); }
static void cb_dest(HSD_Class* c) { T(8204); T(PI(c)); }

#include "fog_gen.h"



static void preg_world(void)
{
    preg(FG, sizeof FG);
    preg(FGA, sizeof FGA);
    preg(FD, sizeof FD);
    preg(FAD, sizeof FAD);
    preg(AO, sizeof AO);
    preg(AD, sizeof AD);
    preg(OBD, sizeof OBD);
    preg(CI, sizeof CI);
    preg(FIN, sizeof FIN);
    preg(VA, sizeof VA); preg(U8A, sizeof U8A); preg(IA, sizeof IA); preg(UA, sizeof UA); preg(FA, sizeof FA); preg(BUF, sizeof BUF); preg(STRS, sizeof STRS); preg(PPA, sizeof PPA);
}

static void setup_world(void)
{
    int i;
    (void) i;
    zero(FG, sizeof FG);
    zero(FGA, sizeof FGA);
    zero(FD, sizeof FD);
    zero(FAD, sizeof FAD);
    zero(AO, sizeof AO);
    zero(AD, sizeof AD);
    zero(OBD, sizeof OBD);
    zero(CI, sizeof CI);
    zero(FIN, sizeof FIN);
    for (i = 0; i < 16; i++) { U8A[i] = (u8) rnd(); }
    for (i = 0; i < 4; i++) { IA[i] = (int) fv(); UA[i] = fv(); FA[i] = rw(); VA[i].x = rw(); VA[i].y = rw(); VA[i].z = rw(); }
    for (i = 0; i < 256; i++) BUF[i] = (unsigned char) rnd();
    BUF[255] = 0;
    for (i = 0; i < 4; i++) PPA[i] = 0;
    for (i = 0; i < 3; i++) fill_HSD_Fog(&FG[i]);
    for (i = 0; i < 3; i++) fill_HSD_FogAdj(&FGA[i]);
    for (i = 0; i < 2; i++) fill_HSD_FogDesc(&FD[i]);
    for (i = 0; i < 2; i++) fill_HSD_FogAdjDesc(&FAD[i]);
    for (i = 0; i < 2; i++) fill_HSD_AObj(&AO[i]);
    for (i = 0; i < 2; i++) fill_HSD_AObjDesc(&AD[i]);
    for (i = 0; i < 2; i++) fill_HSD_ObjData(&OBD[i]);
    for (i = 0; i < 3; i++) fill_HSD_ClassInfo(&CI[i]);
    for (i = 0; i < 2; i++) fill_HSD_FogInfo(&FIN[i]);
    fillg_all();
    fillo_all();
}

static void hash_world(void)
{
    int i;
    (void) i;
    for (i = 0; i < 3; i++) hash_HSD_Fog(&FG[i]);
    for (i = 0; i < 3; i++) hash_HSD_FogAdj(&FGA[i]);
    for (i = 0; i < 2; i++) hash_HSD_FogDesc(&FD[i]);
    for (i = 0; i < 2; i++) hash_HSD_FogAdjDesc(&FAD[i]);
    for (i = 0; i < 2; i++) hash_HSD_AObj(&AO[i]);
    for (i = 0; i < 2; i++) hash_HSD_AObjDesc(&AD[i]);
    for (i = 0; i < 2; i++) hash_HSD_ObjData(&OBD[i]);
    for (i = 0; i < 3; i++) hash_HSD_ClassInfo(&CI[i]);
    for (i = 0; i < 2; i++) hash_HSD_FogInfo(&FIN[i]);
    for (i = 0; i < 4; i++) { HM(B(VA[i].x)); HM(B(VA[i].y)); HM(B(VA[i].z)); HM(B(FA[i])); HM((u32) IA[i]); HM(UA[i]); }
    for (i = 0; i < 16; i++) HM(U8A[i]);
    for (i = 0; i < 256; i++) HM(BUF[i]);
    for (i = 0; i < 4; i++) HM(PI(PPA[i]));
    hashg_all();
    hasho_all();
}

enum { OP_HSD_FogSet = 0, OP_HSD_FogAlloc = 1, OP_HSD_FogLoadDesc = 2, OP_HSD_FogInit = 3, OP_HSD_FogAdjAlloc = 4, OP_HSD_FogAdjLoadDesc = 5, OP_HSD_FogAdjInit = 6, OP_HSD_Fog_8037DE7C = 7, OP_HSD_FogReqAnim = 8, OP_HSD_FogReqAnimByFlags = 9, OP_HSD_FogInterpretAnim = 10, OP_FogUpdateFunc = 11, OP_FogRelease = 12, OP_FogInfoInit = 13, OP_FogAdjInfoInit = 14 };

int main(int argc, char** argv)
{
    int op, k, only = -1;
    if (argc > 1) only = atoi(argv[1]);
    preg_world();
    preg_own();
    freg_all();
    freg_world();
freg((const void*) cb_rel); freg((const void*) cb_dest);
    for (op = 0; op < 15; op++) {
        int n = 0;
        if (only >= 0 && op != only) continue;
        switch (op) {
        case 0: n = 300; break;  /* HSD_FogSet */
        case 1: n = 300; break;  /* HSD_FogAlloc */
        case 2: n = 300; break;  /* HSD_FogLoadDesc */
        case 3: n = 300; break;  /* HSD_FogInit */
        case 4: n = 300; break;  /* HSD_FogAdjAlloc */
        case 5: n = 300; break;  /* HSD_FogAdjLoadDesc */
        case 6: n = 300; break;  /* HSD_FogAdjInit */
        case 7: n = 300; break;  /* HSD_Fog_8037DE7C */
        case 8: n = 300; break;  /* HSD_FogReqAnim */
        case 9: n = 300; break;  /* HSD_FogReqAnimByFlags */
        case 10: n = 300; break;  /* HSD_FogInterpretAnim */
        case 11: n = 300; break;  /* FogUpdateFunc */
        case 12: n = 300; break;  /* FogRelease */
        case 13: n = 300; break;  /* FogInfoInit */
        case 14: n = 300; break;  /* FogAdjInfoInit */
        }
        for (k = 0; k < n; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup_world();
{ int q; for (q = 0; q < 3; q++) { FG[q].fog_adj = (fv() & 1) ? &FGA[fv() % 3] : 0; FG[q].aobj = (fv() & 1) ? &AO[fv() & 1] : 0; FG[q].parent.parent.class_info = &CI[0]; FGA[q].parent.parent.class_info = &CI[0]; } for (q = 0; q < 2; q++) FD[q].fogadjdesc = (fv() & 1) ? &FAD[fv() & 1] : 0; hsdFog.parent.parent.head.parent = &CI[1]; for (q = 0; q < 3; q++) { CI[q].release = (void*) cb_rel; CI[q].destroy = (void*) cb_dest; } }
            if (getenv("MTU_PROG")) { printf("P %d %d%c", op, k, 10); fflush(0); }
            ARMED = 1;
            if (MTU_SETJMP()) { ARMED = 0; printf("%d %d ABORT\n", op, k); continue; }
            switch (op) {
            case 0: {  HSD_Fog * a0 = pick_HSD_Fog(); HSD_FogSet(a0);  } break;
            case 1: {   ret = PI(HSD_FogAlloc());  } break;
            case 2: {  HSD_FogDesc * a0 = pick_HSD_FogDesc(); ret = PI(HSD_FogLoadDesc(a0));  } break;
            case 3: {  HSD_Fog * a0 = pick_HSD_Fog(); HSD_FogDesc * a1 = pick_HSD_FogDesc(); HSD_FogInit(a0, a1);  } break;
            case 4: {   ret = PI(HSD_FogAdjAlloc());  } break;
            case 5: {  HSD_FogAdjDesc * a0 = pick_HSD_FogAdjDesc(); ret = PI(HSD_FogAdjLoadDesc(a0));  } break;
            case 6: {  HSD_FogAdj * a0 = pick_HSD_FogAdj(); HSD_FogAdjDesc * a1 = pick_HSD_FogAdjDesc(); HSD_FogAdjInit(a0, a1);  } break;
            case 7: {  HSD_Fog * a0 = pick_HSD_Fog(); HSD_AObjDesc * a1 = pick_HSD_AObjDesc(); HSD_Fog_8037DE7C(a0, a1);  } break;
            case 8: {  HSD_Fog * a0 = pick_HSD_Fog(); f32 a1 = rw(); HSD_FogReqAnim(a0, a1);  } break;
            case 9: {  HSD_Fog * a0 = pick_HSD_Fog(); u32 a1 = fv(); f32 a2 = rw(); HSD_FogReqAnimByFlags(a0, a1, a2);  } break;
            case 10: {  HSD_Fog * a0 = pick_HSD_Fog(); HSD_FogInterpretAnim(a0);  } break;
            case 11: {  void * a0 = (void*) picko_HSD_Fog(); enum_t a1 = fv(); HSD_ObjData * a2 = pick_HSD_ObjData(); FogUpdateFunc(a0, a1, a2);  } break;
            case 12: {  HSD_Fog * a0 = pick_HSD_Fog(); FogRelease(a0);  } break;
            case 13: {   FogInfoInit();  } break;
            case 14: {   FogAdjInfoInit();  } break;
            }
            ARMED = 0;
            hash_world();

            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
