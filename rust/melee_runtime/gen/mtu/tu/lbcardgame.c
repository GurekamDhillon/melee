extern int atoi(const char*);
extern char* getenv(const char*);
/* TU: src/melee/lb/lbcardgame.c */
/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a
 * recording stub. Written by gen/mtu/mk_lb.py. Each line: op case ret state-hash trace-hash. */

#include TU_SRC
#include "mtu_common.h"
static HSD_GObj GO[4];
static SceneDesc SD[2];
static DynamicModelDesc DM[2];
static SceneCameraDesc SC[2];
static HSD_JObj JO[2];
static Vec3 VA[4];
static u8 U8A[16];
static int IA[4];
static u32 UA[4];
static float FA[4];
static unsigned char BUF[256];
static char STRS[4][24] = { "a.dat", "sym", "LbAd.dat", "x" };

static HSD_GObj* pick_HSD_GObj(void) { u32 r = rnd(); return &GO[(r >> 8) % 4]; }
static HSD_GObj* picko_HSD_GObj(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &GO[(r >> 8) % 4]; }
static SceneDesc* pick_SceneDesc(void) { u32 r = rnd(); return &SD[(r >> 8) % 2]; }
static SceneDesc* picko_SceneDesc(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &SD[(r >> 8) % 2]; }
static DynamicModelDesc* pick_DynamicModelDesc(void) { u32 r = rnd(); return &DM[(r >> 8) % 2]; }
static DynamicModelDesc* picko_DynamicModelDesc(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &DM[(r >> 8) % 2]; }
static SceneCameraDesc* pick_SceneCameraDesc(void) { u32 r = rnd(); return &SC[(r >> 8) % 2]; }
static SceneCameraDesc* picko_SceneCameraDesc(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &SC[(r >> 8) % 2]; }
static HSD_JObj* pick_HSD_JObj(void) { u32 r = rnd(); return &JO[(r >> 8) % 2]; }
static HSD_JObj* picko_HSD_JObj(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &JO[(r >> 8) % 2]; }
static void cbev1(HSD_GObj* g) { T(7001); T(PI(g)); }
static void cbev2(HSD_GObj* g) { T(7002); T(PI(g)); }
static int cbp1(HSD_GObj* g) { T(7003); T(PI(g)); return (int) (fv() & 1); }
static int cbp2(HSD_GObj* g) { T(7004); T(PI(g)); return (int) (fv() & 1); }
static void cbev3(HSD_GObj* g, HSD_GObj* h) { T(7005); T(PI(g)); T(PI(h)); }
static void* pick_fn(int kind)
{
    u32 r = rnd();
    if ((r & 3) == 0) return 0;
    switch (kind) {
    case 1: return (r & 4) ? (void*) cbev1 : (void*) cbev2;
    case 2: return (r & 4) ? (void*) cbp1 : (void*) cbp2;
    case 3: return (void*) cbev3;
    }
    return 0;
}
static void* pick_cb(int kind) { void* f = pick_fn(kind); return f ? f : (kind == 2 ? (void*) cbp1 : kind == 3 ? (void*) cbev3 : (void*) cbev1); }
static void freg_world(void) { freg((const void*) cbev1); freg((const void*) cbev2); freg((const void*) cbp1); freg((const void*) cbp2); freg((const void*) cbev3); }
static void* ICN[4]; static DynamicModelDesc* DMP[2]; static unsigned char NTB[7 * 0x1F2C + 16];
#include "lbcardgame_gen.h"



static void preg_world(void)
{
    preg(GO, sizeof GO);
    preg(SD, sizeof SD);
    preg(DM, sizeof DM);
    preg(SC, sizeof SC);
    preg(JO, sizeof JO);
    preg(VA, sizeof VA); preg(U8A, sizeof U8A); preg(IA, sizeof IA); preg(UA, sizeof UA); preg(FA, sizeof FA); preg(BUF, sizeof BUF); preg(STRS, sizeof STRS);
}

static void setup_world(void)
{
    int i;
    (void) i;
    zero(GO, sizeof GO);
    zero(SD, sizeof SD);
    zero(DM, sizeof DM);
    zero(SC, sizeof SC);
    zero(JO, sizeof JO);
    for (i = 0; i < 16; i++) { U8A[i] = (u8) rnd(); }
    for (i = 0; i < 4; i++) { IA[i] = (int) fv(); UA[i] = fv(); FA[i] = rw(); VA[i].x = rw(); VA[i].y = rw(); VA[i].z = rw(); }
    for (i = 0; i < 256; i++) BUF[i] = (unsigned char) rnd();
    for (i = 0; i < 4; i++) fill_HSD_GObj(&GO[i]);
    for (i = 0; i < 2; i++) fill_SceneDesc(&SD[i]);
    for (i = 0; i < 2; i++) fill_DynamicModelDesc(&DM[i]);
    for (i = 0; i < 2; i++) fill_SceneCameraDesc(&SC[i]);
    for (i = 0; i < 2; i++) fill_HSD_JObj(&JO[i]);
    fillg_all();
    fillo_all();
}

static void hash_world(void)
{
    int i;
    (void) i;
    for (i = 0; i < 4; i++) hash_HSD_GObj(&GO[i]);
    for (i = 0; i < 2; i++) hash_SceneDesc(&SD[i]);
    for (i = 0; i < 2; i++) hash_DynamicModelDesc(&DM[i]);
    for (i = 0; i < 2; i++) hash_SceneCameraDesc(&SC[i]);
    for (i = 0; i < 2; i++) hash_HSD_JObj(&JO[i]);
    for (i = 0; i < 4; i++) { HM(B(VA[i].x)); HM(B(VA[i].y)); HM(B(VA[i].z)); HM(B(FA[i])); HM((u32) IA[i]); HM(UA[i]); }
    for (i = 0; i < 16; i++) HM(U8A[i]);
    for (i = 0; i < 256; i++) HM(BUF[i]);
    hashg_all();
    hasho_all();
}

enum { OP_lb_8001C600 = 0, OP_lb_8001C658 = 1, OP_getCurrentIcon = 2, OP_lb_8001C87C = 3, OP_lb_8001C8BC = 4, OP_updateCardStatus = 5, OP_lbCardGame_SetCardStatus = 6, OP_lb_8001CBBC = 7, OP_fn_8001CC30 = 8, OP_lb_8001CC4C = 9, OP_dont_inline_helper = 10, OP_lb_8001CC84 = 11, OP_lb_8001CDB4 = 12, OP_lbCardGame_SaveChanges = 13, OP_lbCardGame_DecideGameMode = 14, OP_gobj1_Proc = 15, OP_gobj0_RenderFunc = 16, OP_lbCardGame_InitScene = 17, OP_lbCardGame_LoadArchive = 18, OP_lbCardGame_Reset = 19, OP_lbCardGame_Init = 20 };

int main(int argc, char** argv)
{
    int op, k, only = -1;
    if (argc > 1) only = atoi(argv[1]);
    preg_world();
    preg_own();
    freg_all();
    freg_world();
preg(ICN, sizeof ICN); preg(DMP, sizeof DMP); preg(NTB, sizeof NTB);
    for (op = 0; op < 21; op++) {
        int n = 0;
        if (only >= 0 && op != only) continue;
        switch (op) {
        case 0: n = 300; break;  /* lb_8001C600 */
        case 1: n = 300; break;  /* lb_8001C658 */
        case 2: n = 300; break;  /* getCurrentIcon */
        case 3: n = 300; break;  /* lb_8001C87C */
        case 4: n = 300; break;  /* lb_8001C8BC */
        case 5: n = 300; break;  /* updateCardStatus */
        case 6: n = 300; break;  /* lbCardGame_SetCardStatus */
        case 7: n = 300; break;  /* lb_8001CBBC */
        case 8: n = 300; break;  /* fn_8001CC30 */
        case 9: n = 300; break;  /* lb_8001CC4C */
        case 10: n = 300; break;  /* dont_inline_helper */
        case 11: n = 300; break;  /* lb_8001CC84 */
        case 12: n = 300; break;  /* lb_8001CDB4 */
        case 13: n = 300; break;  /* lbCardGame_SaveChanges */
        case 14: n = 300; break;  /* lbCardGame_DecideGameMode */
        case 15: n = 300; break;  /* gobj1_Proc */
        case 16: n = 300; break;  /* gobj0_RenderFunc */
        case 17: n = 300; break;  /* lbCardGame_InitScene */
        case 18: n = 300; break;  /* lbCardGame_LoadArchive */
        case 19: n = 300; break;  /* lbCardGame_Reset */
        case 20: n = 300; break;  /* lbCardGame_Init */
        }
        for (k = 0; k < n; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup_world();
{ int q; for (q = 0; q < 4; q++) ICN[q] = (fv() & 7) ? (void*) (BUF + 8 * q) : 0; state.icon_data = (op == OP_lbCardGame_LoadArchive && (fv() & 1)) ? 0 : ICN; DMP[0] = &DM[0]; DMP[1] = &DM[1]; SD[0].models = DMP; SD[0].cameras = SC; SD[1].models = DMP + 1; SD[1].cameras = SC + 1; state.scene_data = (fv() & 7) ? &SD[fv() & 1] : 0; state.jobj_translate_idx = 0; state.enable = (fv() & 7) != 0 || op == OP_lb_8001C8BC || op == OP_lbCardGame_SaveChanges; state.x10 = fv() & 1; OS_BusClock = (fv() & 1) ? 162000000u : 4u + fv() % 100000u; }
            if (getenv("MTU_PROG")) { printf("P %d %d%c", op, k, 10); fflush(0); }
            switch (op) {
            case 0: {   lb_8001C600();  } break;
            case 1: {   ret = PI(lb_8001C658());  } break;
            case 2: {   ret = PI(getCurrentIcon());  } break;
            case 3: {   ret = (u32) lb_8001C87C();  } break;
            case 4: {   ret = (u32) lb_8001C8BC();  } break;
            case 5: {   ret = (u32) updateCardStatus();  } break;
            case 6: {  LbCardStatus a0 = fv(); lbCardGame_SetCardStatus(a0);  } break;
            case 7: {   ret = (u32) lb_8001CBBC();  } break;
            case 8: {  bool a0 = fv(); fn_8001CC30(a0);  } break;
            case 9: {   ret = (u32) lb_8001CC4C();  } break;
            case 10: {   ret = (u32) dont_inline_helper();  } break;
            case 11: {   lb_8001CC84();  } break;
            case 12: {   lb_8001CDB4();  } break;
            case 13: {   lbCardGame_SaveChanges();  } break;
            case 14: {   ret = (u32) lbCardGame_DecideGameMode();  } break;
            case 15: {  HSD_GObj * a0 = pick_HSD_GObj(); gobj1_Proc(a0);  } break;
            case 16: {  HSD_GObj * a0 = pick_HSD_GObj(); int a1 = fv(); gobj0_RenderFunc(a0, a1);  } break;
            case 17: {   lbCardGame_InitScene();  } break;
            case 18: {  int a0 = fv(); lbCardGame_LoadArchive(a0);  } break;
            case 19: {   lbCardGame_Reset();  } break;
            case 20: {   lbCardGame_Init();  } break;
            }
            hash_world();

            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
