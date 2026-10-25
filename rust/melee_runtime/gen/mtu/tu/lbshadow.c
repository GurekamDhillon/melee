extern int atoi(const char*);
extern char* getenv(const char*);
/* TU: src/melee/lb/lbshadow.c */
/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a
 * recording stub. Written by gen/mtu/mk_lb.py. Each line: op case ret state-hash trace-hash. */
#include <sysdolphin/baselib/jobj.h>

#include TU_SRC
#include "mtu_common.h"
static HSD_GObj GO[4];
static LbShadow LS[2];
static HSD_JObj JO[3];
static HSD_Shadow SHD[3];
static HSD_CObj CO[2];
static HSD_LObj LO[3];
static Fighter FTR[3];
static CmSubject CM[2];
static HSD_TObj TO[2];
static HSD_ImageDesc ID[2];
static HSD_Spline SP[1];
static Vec3 VA[4];
static u8 U8A[16];
static int IA[4];
static u32 UA[4];
static float FA[4];
static unsigned char BUF[256];
static char STRS[4][64] = { "a.dat", "sym", "LbAd.dat", "x" };

static HSD_GObj* pick_HSD_GObj(void) { u32 r = rnd(); return &GO[(r >> 8) % 4]; }
static HSD_GObj* picko_HSD_GObj(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &GO[(r >> 8) % 4]; }
static LbShadow* pick_LbShadow(void) { u32 r = rnd(); return &LS[(r >> 8) % 2]; }
static LbShadow* picko_LbShadow(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &LS[(r >> 8) % 2]; }
static HSD_JObj* pick_HSD_JObj(void) { u32 r = rnd(); return &JO[(r >> 8) % 3]; }
static HSD_JObj* picko_HSD_JObj(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &JO[(r >> 8) % 3]; }
static HSD_Shadow* pick_HSD_Shadow(void) { u32 r = rnd(); return &SHD[(r >> 8) % 3]; }
static HSD_Shadow* picko_HSD_Shadow(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &SHD[(r >> 8) % 3]; }
static HSD_CObj* pick_HSD_CObj(void) { u32 r = rnd(); return &CO[(r >> 8) % 2]; }
static HSD_CObj* picko_HSD_CObj(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &CO[(r >> 8) % 2]; }
static HSD_LObj* pick_HSD_LObj(void) { u32 r = rnd(); return &LO[(r >> 8) % 3]; }
static HSD_LObj* picko_HSD_LObj(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &LO[(r >> 8) % 3]; }
static Fighter* pick_Fighter(void) { u32 r = rnd(); return &FTR[(r >> 8) % 3]; }
static Fighter* picko_Fighter(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &FTR[(r >> 8) % 3]; }
static CmSubject* pick_CmSubject(void) { u32 r = rnd(); return &CM[(r >> 8) % 2]; }
static CmSubject* picko_CmSubject(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &CM[(r >> 8) % 2]; }
static HSD_TObj* pick_HSD_TObj(void) { u32 r = rnd(); return &TO[(r >> 8) % 2]; }
static HSD_TObj* picko_HSD_TObj(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &TO[(r >> 8) % 2]; }
static HSD_ImageDesc* pick_HSD_ImageDesc(void) { u32 r = rnd(); return &ID[(r >> 8) % 2]; }
static HSD_ImageDesc* picko_HSD_ImageDesc(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &ID[(r >> 8) % 2]; }
static HSD_Spline* pick_HSD_Spline(void) { u32 r = rnd(); return &SP[(r >> 8) % 1]; }
static HSD_Spline* picko_HSD_Spline(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &SP[(r >> 8) % 1]; }
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
static HSD_GObj* PLH[16]; static HSD_GObj* GXH[8]; Mtx HSD_identityMtx;
#include "lbshadow_gen.h"



static void preg_world(void)
{
    preg(GO, sizeof GO);
    preg(LS, sizeof LS);
    preg(JO, sizeof JO);
    preg(SHD, sizeof SHD);
    preg(CO, sizeof CO);
    preg(LO, sizeof LO);
    preg(FTR, sizeof FTR);
    preg(CM, sizeof CM);
    preg(TO, sizeof TO);
    preg(ID, sizeof ID);
    preg(SP, sizeof SP);
    preg(VA, sizeof VA); preg(U8A, sizeof U8A); preg(IA, sizeof IA); preg(UA, sizeof UA); preg(FA, sizeof FA); preg(BUF, sizeof BUF); preg(STRS, sizeof STRS);
}

static void setup_world(void)
{
    int i;
    (void) i;
    zero(GO, sizeof GO);
    zero(LS, sizeof LS);
    zero(JO, sizeof JO);
    zero(SHD, sizeof SHD);
    zero(CO, sizeof CO);
    zero(LO, sizeof LO);
    zero(FTR, sizeof FTR);
    zero(CM, sizeof CM);
    zero(TO, sizeof TO);
    zero(ID, sizeof ID);
    zero(SP, sizeof SP);
    for (i = 0; i < 16; i++) { U8A[i] = (u8) rnd(); }
    for (i = 0; i < 4; i++) { IA[i] = (int) fv(); UA[i] = fv(); FA[i] = rw(); VA[i].x = rw(); VA[i].y = rw(); VA[i].z = rw(); }
    for (i = 0; i < 256; i++) BUF[i] = (unsigned char) rnd();
    BUF[255] = 0;
    for (i = 0; i < 4; i++) fill_HSD_GObj(&GO[i]);
    for (i = 0; i < 2; i++) fill_LbShadow(&LS[i]);
    for (i = 0; i < 3; i++) fill_HSD_JObj(&JO[i]);
    for (i = 0; i < 3; i++) fill_HSD_Shadow(&SHD[i]);
    for (i = 0; i < 2; i++) fill_HSD_CObj(&CO[i]);
    for (i = 0; i < 3; i++) fill_HSD_LObj(&LO[i]);
    for (i = 0; i < 3; i++) fill_Fighter(&FTR[i]);
    for (i = 0; i < 2; i++) fill_CmSubject(&CM[i]);
    for (i = 0; i < 2; i++) fill_HSD_TObj(&TO[i]);
    for (i = 0; i < 2; i++) fill_HSD_ImageDesc(&ID[i]);
    for (i = 0; i < 1; i++) fill_HSD_Spline(&SP[i]);
    fillg_all();
    fillo_all();
}

static void hash_world(void)
{
    int i;
    (void) i;
    for (i = 0; i < 4; i++) hash_HSD_GObj(&GO[i]);
    for (i = 0; i < 2; i++) hash_LbShadow(&LS[i]);
    for (i = 0; i < 3; i++) hash_HSD_JObj(&JO[i]);
    for (i = 0; i < 3; i++) hash_HSD_Shadow(&SHD[i]);
    for (i = 0; i < 2; i++) hash_HSD_CObj(&CO[i]);
    for (i = 0; i < 3; i++) hash_HSD_LObj(&LO[i]);
    for (i = 0; i < 3; i++) hash_Fighter(&FTR[i]);
    for (i = 0; i < 2; i++) hash_CmSubject(&CM[i]);
    for (i = 0; i < 2; i++) hash_HSD_TObj(&TO[i]);
    for (i = 0; i < 2; i++) hash_HSD_ImageDesc(&ID[i]);
    for (i = 0; i < 1; i++) hash_HSD_Spline(&SP[i]);
    for (i = 0; i < 4; i++) { HM(B(VA[i].x)); HM(B(VA[i].y)); HM(B(VA[i].z)); HM(B(FA[i])); HM((u32) IA[i]); HM(UA[i]); }
    for (i = 0; i < 16; i++) HM(U8A[i]);
    for (i = 0; i < 256; i++) HM(BUF[i]);
    hashg_all();
    hasho_all();
}

enum { OP_splGetCardinalTangent = 0, OP_splGetBSplineTangent = 1, OP_splGetBezierTangent = 2, OP_lbShadow_8000E9F0 = 3, OP_lbShadow_8000ED54 = 4, OP_lbShadow_8000EE8C = 5, OP_lbShadow_8000EEE0 = 6, OP_lbShadow_8000EFEC = 7, OP_lbShadow_8000F214 = 8, OP_lbShadow_8000F38C = 9 };

int main(int argc, char** argv)
{
    int op, k, only = -1;
    if (argc > 1) only = atoi(argv[1]);
    preg_world();
    preg_own();
    freg_all();
    freg_world();
preg(PLH, sizeof PLH); preg(GXH, sizeof GXH);
    for (op = 0; op < 10; op++) {
        int n = 0;
        if (only >= 0 && op != only) continue;
        switch (op) {
        case 0: n = 400; break;  /* splGetCardinalTangent */
        case 1: n = 400; break;  /* splGetBSplineTangent */
        case 2: n = 400; break;  /* splGetBezierTangent */
        case 3: n = 400; break;  /* lbShadow_8000E9F0 */
        case 4: n = 400; break;  /* lbShadow_8000ED54 */
        case 5: n = 400; break;  /* lbShadow_8000EE8C */
        case 6: n = 400; break;  /* lbShadow_8000EEE0 */
        case 7: n = 400; break;  /* lbShadow_8000EFEC */
        case 8: n = 400; break;  /* lbShadow_8000F214 */
        case 9: n = 400; break;  /* lbShadow_8000F38C */
        }
        for (k = 0; k < n; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup_world();
{ int q; for (q = 0; q < 16; q++) PLH[q] = 0; for (q = 0; q < 8; q++) GXH[q] = 0; PLH[8] = (fv() & 7) ? &GO[0] : 0; GXH[4] = (fv() & 7) ? &GO[fv() & 3] : 0; HSD_GObjPLinkHead = PLH; HSD_GObjGXLinkHead = GXH; for (q = 0; q < 4; q++) { GO[q].next = q < 3 ? &GO[q + 1] : 0; GO[q].next_gx = q < 3 ? &GO[q + 1] : 0; GO[q].user_data = &FTR[q % 3]; GO[q].hsd_obj = &LO[q % 3]; } for (q = 0; q < 3; q++) LO[q].next = q < 2 ? &LO[q + 1] : 0; for (q = 0; q < 3; q++) { FTR[q].x890_cameraBox = (fv() & 3) ? &CM[fv() & 1] : 0; FTR[q].x20A4.shadow = (fv() & 3) ? &SHD[fv() % 3] : 0; SHD[q].camera = &CO[q & 1]; SHD[q].texture = &TO[q & 1]; } for (q = 0; q < 2; q++) TO[q].imagedesc = &ID[q]; SP[0].type = (u8) (fv() % 5); SP[0].numcv = (s16) (1 + fv() % 6); SP[0].tension = rw(); SP[0].cv = (Vec3*) (BUF + 12); for (q = 0; q < 63; q++) ((float*) BUF)[q] = rw(); }
            if (getenv("MTU_PROG")) { printf("P %d %d%c", op, k, 10); fflush(0); }
            ARMED = 1;
            if (MTU_SETJMP()) { ARMED = 0; printf("%d %d ABORT\n", op, k); continue; }
            switch (op) {
            case 0: {  Vec3 * a0 = (Vec3*) FA; Vec3 * a1 = VA; f32 a2 = rw(); f32 a3 = ((fv() & 7) == 0 ? 1.0f : (float) (fv() % 1001) / 1000.0f); splGetCardinalTangent(a0, a1, a2, a3);  } break;
            case 1: {  Vec3 * a0 = (Vec3*) FA; Vec3 * a1 = VA; f32 a2 = ((fv() & 7) == 0 ? 1.0f : (float) (fv() % 1001) / 1000.0f); splGetBSplineTangent(a0, a1, a2);  } break;
            case 2: {  Vec3 * a0 = (Vec3*) FA; Vec3 * a1 = VA; f32 a2 = ((fv() & 7) == 0 ? 1.0f : (float) (fv() % 1001) / 1000.0f); splGetBezierTangent(a0, a1, a2);  } break;
            case 3: {  Vec3 * a0 = (Vec3*) FA; HSD_Spline * a1 = pick_HSD_Spline(); f32 a2 = ((fv() & 7) == 0 ? 1.0f : (float) (fv() % 1001) / 1000.0f); lbShadow_8000E9F0(a0, a1, a2);  } break;
            case 4: {  LbShadow * a0 = pick_LbShadow(); HSD_JObj * a1 = pick_HSD_JObj(); lbShadow_8000ED54(a0, a1);  } break;
            case 5: {  LbShadow * a0 = pick_LbShadow(); lbShadow_8000EE8C(a0);  } break;
            case 6: {  HSD_GObj * a0 = pick_HSD_GObj(); lbShadow_8000EEE0(a0);  } break;
            case 7: {   lbShadow_8000EFEC();  } break;
            case 8: {  HSD_Shadow * a0 = pick_HSD_Shadow(); lbShadow_8000F214(a0);  } break;
            case 9: {  s32 a0 = fv(); lbShadow_8000F38C(a0);  } break;
            }
            ARMED = 0;
            hash_world();

            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
