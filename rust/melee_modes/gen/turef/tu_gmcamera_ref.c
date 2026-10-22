/* C reference for rust/melee_modes/src/tu/gmcamera.rs: the game's gmcamera.c verbatim (the VS camera
 * UI state machine), with its engine and UI calls stubbed and logged (the Rust hooks log the same text).
 * Run: python rust/melee_modes/gen/turef/tu_shard457_gm.py tu_gmcamera */
#include "../../../../src/melee/gm/gmcamera.c"
#include "tu_gm_common.h"

/* ---- driver-owned objects ---- */
HSD_PadStatus HSD_PadCopyStatus[4];
u8 HSD_GObj_JObjKind = 1;
static u8 SLARR[4096];
static SisLibUnkStruct2 SL2;
struct SIS* HSD_SisLib_804D1124[5]; /* the TU reads a SisLibUnkStruct over this (x0C_ptr at +0x0C) */
static HSD_JObj JOBJS[16];
static HSD_Text TEXTS[3];
static HSD_GObj GOBJS[4];
static HSD_Archive ARCH_SENT;
static char IMG_SENT[4];
static u8 DIGITS[16];
static VsSceneController SC;
static HSD_Joint JSLOT[2];
static HSD_Joint* JSLOTPTR;
static DynamicModelDesc MDESC;
static int g_loadcount;
static int g_pause[4];
static int SNAP40C[2], SNAP3B0[2], SNAP3CC[2], SNAP338[2], SNAPDC0C, SNAPDF20, SIS611C_RET, CARDTASK;
static void* CMSNAP_RET;
static int ARG_U, ARG_I, ARG_P0, ARG_P1;
static f32 ARG_F[4];
static s32 DUMMY_V0, DUMMY_V1;

static const char* JNAMES[16] = {"ka", "kb", "kc", "kd", "ke", "kf", "kg", "kh",
                                 "ki", "kj", "kk", "kl", "km", "kn", "ko", "kp"};
static const char* jname(void* p)
{
    int k;
    if (p == NULL) return "null";
    for (k = 0; k < 16; k++) {
        if (p == (void*) &JOBJS[k]) return JNAMES[k];
    }
    return "?";
}
static const char* tname(void* p)
{
    if (p == NULL) return "null";
    if (p == (void*) &TEXTS[0]) return "T0";
    if (p == (void*) &TEXTS[1]) return "T1";
    if (p == (void*) &TEXTS[2]) return "T2";
    return "?";
}
static const char* gname(void* p)
{
    if (p == (void*) &GOBJS[0]) return "G0";
    if (p == (void*) &GOBJS[1]) return "G1";
    if (p == (void*) &GOBJS[2]) return "G2";
    if (p == (void*) &GOBJS[3]) return "G3";
    return "?";
}

/* ---- externals (logged) ---- */
HSD_Text* HSD_SisLib_803A5ACC(s32 a, s32 b, f32 x, f32 y, f32 w, f32 h, f32 s)
{
    (void) w;
    (void) h;
    (void) s;
    lg("sis5ACC(%d,%d,%08x,%08x)", (int) a, (int) b, bits(x), bits(y));
    return &TEXTS[0];
}
u8* HSD_SisLib_803A6530(s32 a, s32 b, s32 c)
{
    lg("dig(%d,%d,%d)", (int) a, (int) b, (int) c);
    return DIGITS;
}
void HSD_SisLib_803A660C(s32 a, s32 b, s32 c)
{
    lg("label(%d,%d,%d)", (int) a, (int) b, (int) c);
}
void HSD_SisLib_803A6368(HSD_Text* t, s32 v)
{
    lg("text6368(%s,%d)", tname(t), (int) v);
}
void HSD_SisLib_803A62A0(s32 a, char* b, char* c)
{
    lg("sis62A0(%d,%s,%s)", (int) a, b, c);
}
int HSD_SisLib_803A611C(int a, HSD_GObj* g, u16 b, u8 c, u8 d, u8 e, u8 f, u32 h)
{
    lg("sis611C(%d,%s,%u,%u,%u,%u,%u,%u)", a, g == NULL ? "null" : "g", (unsigned) b, (unsigned) c, (unsigned) d,
       (unsigned) e, (unsigned) f, (unsigned) h);
    return SIS611C_RET;
}
void HSD_SisLib_803A5CC4(HSD_Text* t)
{
    lg("sis5CC4(%s)", tname(t));
}
int lbSnap_8001D40C(int i)
{
    lg("snap40C(%d)", i);
    return SNAP40C[i & 1];
}
int lbSnap_8001D3B0(int i)
{
    lg("snap3B0(%d)", i);
    return SNAP3B0[i & 1];
}
int lbSnap_8001D3CC(int i)
{
    lg("snap3CC(%d)", i);
    return SNAP3CC[i & 1];
}
int lbSnap_8001D338(int i)
{
    lg("snap338(%d)", i);
    return SNAP338[i & 1];
}
int lbSnap_8001DC0C(u8* img)
{
    lg("snapDC0C(%s)", img == (u8*) IMG_SENT ? "img" : "?");
    return SNAPDC0C;
}
int lbSnap_8001DF20(void)
{
    lg("snapDF20");
    return SNAPDF20;
}
int lbSnap_8001DF6C(int chan)
{
    lg("snapDF6C(%d)", chan);
    return 0;
}
VsSceneController* gmVs_GetSceneController(void)
{
    lg("vsctrl");
    return &SC;
}
bool gm_GetDbPauseFlag(int bit)
{
    lg("dbg(%d)", bit);
    return g_pause[bit & 3];
}
void gm_ClearDbPauseFlag(int bit)
{
    lg("dbclr(%d)", bit);
    g_pause[bit & 3] = 0;
}
void gm_SetDbPauseFlag(int bit)
{
    lg("dbset(%d)", bit);
    g_pause[bit & 3] = 1;
}
void gm_801A0FEC(s32 a, u8 b)
{
    lg("a0fec(%d,%u)", (int) a, (unsigned) b);
}
void gm_801A10FC(int a)
{
    lg("a10fc(%d)", a);
}
void lbAudioAx_80024E84(bool v)
{
    lg("audio84(%d)", (int) v);
}
void lbAudioAx_80024030(int id)
{
    lg("audio30(%d)", id);
}
void ifAll_802F33CC(void)
{
    lg("if33CC");
}
void ifAll_802F3394(void)
{
    lg("if3394");
}
void HSD_PadRumbleUnpauseAll(void)
{
    lg("rumbleUnp");
}
void HSD_PadRumblePauseAll(void)
{
    lg("rumblePau");
}
void cmSnap_800315C8(void)
{
    lg("cm15C8");
}
void* cmSnap_80031618(void)
{
    lg("cm1618");
    return CMSNAP_RET;
}
void cmSnap_800316B4(void)
{
    lg("cm16B4");
}
int lb_80011E24(HSD_JObj* src, HSD_JObj** out, ...)
{
    va_list ap;
    int idx;
    int last;
    va_start(ap, out);
    idx = va_arg(ap, int);
    last = va_arg(ap, int);
    va_end(ap);
    lg("lb11E24(%s,%d,%d)", jname(src), idx, last);
    *out = &JOBJS[(idx & 15)];
    return 0;
}
void HSD_JObjSetFlagsAll(HSD_JObj* j, u32 f)
{
    lg("setflags(%s,%08x)", jname(j), (unsigned) f);
}
void HSD_JObjClearFlagsAll(HSD_JObj* j, u32 f)
{
    lg("clrflags(%s,%08x)", jname(j), (unsigned) f);
}
void HSD_ForeachAnim(void* obj, HSD_Type type, HSD_TypeMask mask, void* func, AObj_Arg_Type arg_type, ...)
{
    (void) obj;
    (void) arg_type;
    lg("foreach(%u,%08x,%s)", (unsigned) type, (unsigned) mask, func == NULL ? "n" : "f");
}
void HSD_AObjStopAnim(HSD_AObj* aobj, void* obj, HSD_ObjUpdateFunc func)
{
    (void) aobj;
    (void) obj;
    (void) func;
    lg("stopanim");
}
enum_t lbCardNew_CompleteNextTask(void)
{
    lg("cardtask");
    return CARDTASK;
}
HSD_Archive* lbArchive_LoadArchive(const char* filename)
{
    lg("arch(%s)", filename);
    return &ARCH_SENT;
}
void* HSD_ArchiveGetPublicAddress(HSD_Archive* a, const char* name)
{
    (void) a;
    lg("pub(%s)", name);
    if (name[0] == 'I' && name[9] == 'n' && name[10] == 'f' && name[11] == 'o') {
        return &JSLOTPTR; /* IfCameraInfo_Top_model_set: a HSD_Joint** */
    }
    return &MDESC; /* IfCamera_Top_model_set: a DynamicModelDesc* */
}
HSD_GObj* GObj_Create(u16 classifier, u8 p_link, u8 priority)
{
    lg("gobj(%u,%u,%u)", (unsigned) classifier, (unsigned) p_link, (unsigned) priority);
    return &GOBJS[(g_loadcount + 1) & 3];
}
HSD_JObj* HSD_JObjLoadJoint(HSD_Joint* joint)
{
    HSD_JObj* r = &JOBJS[8 + (g_loadcount & 7)];
    lg("jload(%s,%d)", joint == NULL ? "null" : "j", g_loadcount);
    g_loadcount++;
    return r;
}
void HSD_GObjObject_80390A70(HSD_GObj* gobj, u8 kind, void* obj)
{
    lg("setobj(%s,%u,%s)", gname(gobj), (unsigned) kind, jname(obj));
}
void GObj_SetupGXLink(HSD_GObj* gobj, GObj_RenderFunc render_cb, u8 gx_link, u32 priority)
{
    lg("gx(%s,%s,%u,%u)", gname(gobj), (void*) render_cb == (void*) HSD_GObj_JObjCallback ? "jobjcb" : "?",
       (unsigned) gx_link, (unsigned) priority);
}
void gm_8016895C(HSD_JObj* jobj, DynamicModelDesc* mdl, int idx)
{
    (void) mdl;
    lg("m895C(%s,%d)", jname(jobj), idx);
}
void HSD_JObjReqAnimAll(HSD_JObj* jobj, f32 frame)
{
    lg("reqanim(%s,%08x)", jname(jobj), bits(frame));
}
void HSD_JObjAnimAll(HSD_JObj* jobj)
{
    lg("anim(%s)", jname(jobj));
}
HSD_GObjProc* HSD_GObj_SetupProc(HSD_GObj* gobj, HSD_GObjEvent proc, u8 prio)
{
    lg("proc(%s,%s,%u)", gname(gobj), (void*) proc == (void*) fn_801A31D8 ? "p31D8" : "?", (unsigned) prio);
    return NULL;
}
void HSD_GObj_JObjCallback(HSD_GObj* gobj, int arg1)
{
    (void) gobj;
    (void) arg1;
}

void HSD_JObjSetMtxDirtySub(HSD_JObj* j)
{
    lg("dirty(%s)", jname(j));
}
void __assert(const char* f, u32 l, const char* c)
{
    (void) f;
    (void) l;
    (void) c;
}

/* ---- per-case state ---- */
static const char* FUNC_NAMES[] = {
    "gmCamera_801A2224", "gmCamera_801A2334", "gmCamera_801A253C", "gmCamera_801A25C8", "gmCamera_801A2640",
    "gmCamera_801A2650", "gmCamera_801A26C0", "gmCamera_801A2798", "gmCamera_801A2800", "gmCamera_801A28AC",
    "gmCamera_801A292C", "gmCamera_801A2AAC", "gmCamera_801A2BB0", "gmCamera_801A2BF0_get_translate_x",
    "gmCamera_801A2BF0", "gmCamera_801A2D44", "gmCamera_801A2FBC", "gmCamera_801A2FFC", "gmCamera_801A3048",
    "gmCamera_801A3098", "gmCamera_801A30E4", "fn_801A31D8", "gmCamera_801A31FC",
};
#define NFUNCS ((int) (sizeof FUNC_NAMES / sizeof FUNC_NAMES[0]))
#define NCASES 200

static void setup_case(void)
{
    int i;
    int k;
    gmCamera_VsCamUiState.ifvscam = (rnd() & 1) ? (void*) &ARCH_SENT : NULL;
    gmCamera_VsCamUiState.x4 = (rnd() % 17) == 16 ? NULL : &JOBJS[rnd() % 16];
    gmCamera_VsCamUiState.x8 = (rnd() % 17) == 16 ? NULL : &JOBJS[rnd() % 16];
    gmCamera_VsCamUiState.xC = (s32) (rnd() % 9);
    gmCamera_VsCamUiState.x10 = (s32) (rnd() % 3);
    gmCamera_VsCamUiState.x14 = (s32) (rnd() & 1);
    gmCamera_VsCamUiState.x18 = (s32) (rnd() & 1);
    gmCamera_VsCamUiState.snap_image = (rnd() & 1) ? (void*) IMG_SENT : NULL;
    gmCamera_VsCamUiState.x20 = (int) (rnd() % 4);
    for (i = 0; i < 2; i++) {
        gmCamera_VsCamUiState.x24[i].x0 = rnd() & 1;
        gmCamera_VsCamUiState.x24[i].x4 = (int) (rnd() % 7);
        gmCamera_VsCamUiState.x24[i].x8 = (int) (rnd() % 4);
        gmCamera_VsCamUiState.x24[i].xC = (int) (rnd() % 6);
    }
    gmCamera_VsCamUiState.x44 = (s32) (rnd() % 6);
    for (i = 0; i < 3; i++) {
        gmCamera_VsCamUiState.x48[i] = (rnd() & 1) ? &TEXTS[i] : NULL;
    }
    gmCamera_VsCamUiState.x54 = (int) (rnd() % 50);
    for (k = 0; k < 4; k++) {
        g_pause[k] = rnd() & 1;
    }
    for (k = 0; k < 2; k++) {
        SNAP40C[k] = (int) (rnd() % 2);
    }
    for (k = 0; k < 2; k++) {
        SNAP3B0[k] = (int) (rnd() % 2);
    }
    for (k = 0; k < 2; k++) {
        SNAP3CC[k] = (int) (rnd() % 2);
    }
    for (k = 0; k < 2; k++) {
        SNAP338[k] = (int) (rnd() % 2);
    }
    SNAPDC0C = (int) (rnd() % 2);
    SNAPDF20 = (int) (rnd() % 5);
    CMSNAP_RET = (rnd() & 1) ? (void*) IMG_SENT : NULL;
    k = (int) (rnd() % 3);
    CARDTASK = k == 0 ? 0 : k == 1 ? 11 : 3;
    SIS611C_RET = (int) (rnd() % 40);
    HSD_PadCopyStatus[3].trigger = rnd();
    SC.state.pauser = (s32) (rnd() % 3);
    SC.state.hud_enabled = (u8) (rnd() & 1);
    SC.state.unk_3 = (u8) (rnd() & 1);
    ARG_U = (int) (rnd() % 12000);
    ARG_I = (int) (rnd() % 9);
    ARG_P0 = (int) (rnd() % 3);
    ARG_P1 = (int) (rnd() % 3);
    for (k = 0; k < 4; k++) {
        ARG_F[k] = rf();
    }
    g_loadcount = 0;
    memset(GOBJS, 0, sizeof GOBJS);
    for (k = 0; k < 4096; k++) {
        SLARR[k] = (u8) rnd();
    }
    {
        SisLibUnkStruct2* p2 = &SL2;
        memcpy((char*) HSD_SisLib_804D1124 + (size_t) &((SisLibUnkStruct*) 0)->x0C_ptr, &p2, sizeof p2);
    }
    SL2.x08_arr = SLARR;
    JSLOTPTR = &JSLOT[0];
    MDESC.joint = &JSLOT[1];
}

static void fingerprint(int f, int i, int ret)
{
    int k;
    printf("%d.%d %s|log=%s|ifv=%s|x4=%s|x8=%s|xC=%d|x10=%d|x14=%d|x18=%d|snap=%s|x20=%d|"
           "p0=%d,%d,%d,%d|p1=%d,%d,%d,%d|x44=%d|x48=%s,%s,%s|x54=%d|pause=%d,%d,%d,%d|"
           "sc=%d,%d,%d|tx=",
           f, i, FUNC_NAMES[f], g_log, gmCamera_VsCamUiState.ifvscam ? "arch" : "null",
           jname(gmCamera_VsCamUiState.x4), jname(gmCamera_VsCamUiState.x8), (int) gmCamera_VsCamUiState.xC,
           (int) gmCamera_VsCamUiState.x10, (int) gmCamera_VsCamUiState.x14, (int) gmCamera_VsCamUiState.x18,
           gmCamera_VsCamUiState.snap_image ? "img" : "null", gmCamera_VsCamUiState.x20,
           (int) gmCamera_VsCamUiState.x24[0].x0, gmCamera_VsCamUiState.x24[0].x4, gmCamera_VsCamUiState.x24[0].x8,
           gmCamera_VsCamUiState.x24[0].xC, (int) gmCamera_VsCamUiState.x24[1].x0, gmCamera_VsCamUiState.x24[1].x4,
           gmCamera_VsCamUiState.x24[1].x8, gmCamera_VsCamUiState.x24[1].xC, (int) gmCamera_VsCamUiState.x44,
           tname(gmCamera_VsCamUiState.x48[0]), tname(gmCamera_VsCamUiState.x48[1]),
           tname(gmCamera_VsCamUiState.x48[2]), gmCamera_VsCamUiState.x54, g_pause[0], g_pause[1], g_pause[2],
           g_pause[3], (int) SC.state.pauser, (int) SC.state.hud_enabled, (int) SC.state.unk_3);
    for (k = 0; k < 16; k++) {
        printf("%08x,", bits(JOBJS[k].translate.x));
    }
    printf("|txt=");
    for (k = 0; k < 3; k++) {
        printf("%08x,%08x,%d;", bits(TEXTS[k].font_size.x), bits(TEXTS[k].font_size.y), (int) TEXTS[k].default_kerning);
    }
    printf("|ret=%d|buf=%u,%u,%u,%u,%u,%u,%u,%u\n", ret, (unsigned) DIGITS[0], (unsigned) DIGITS[1],
           (unsigned) DIGITS[2], (unsigned) DIGITS[3], (unsigned) DIGITS[4], (unsigned) DIGITS[5],
           (unsigned) DIGITS[6], (unsigned) DIGITS[7]);
}

static void call_fn(int f, int* ret)
{
    u8 buf[16];
    *ret = 0;
    switch (f) {
    case 0: {
        u8* r;
        memset(buf, 0, sizeof buf);
        r = gmCamera_801A2224(buf, (u32) ARG_U);
        *ret = (int) (r - buf);
        memcpy(DIGITS, buf, 8);
        break;
    }
    case 1: {
        HSD_Text* t = gmCamera_801A2334(ARG_P0 & 1, ARG_F[0], ARG_F[1], ARG_F[2], ARG_F[3]);
        *ret = t != NULL;
        break;
    }
    case 2: {
        s32 a0 = DUMMY_V0;
        s32 a1 = DUMMY_V1;
        gmCamera_801A253C(ARG_P0 == 0 ? NULL : &a0, ARG_P1 == 0 ? NULL : &a1);
        *ret = (int) (a0 + a1);
        break;
    }
    case 3: gmCamera_801A25C8(); break;
    case 4: *ret = (int) gmCamera_801A2640(); break;
    case 5: gmCamera_801A2650(); break;
    case 6: gmCamera_801A26C0(); break;
    case 7: gmCamera_801A2798(); break;
    case 8: gmCamera_801A2800(); break;
    case 9: gmCamera_801A28AC(); break;
    case 10: gmCamera_801A292C(); break;
    case 11: gmCamera_801A2AAC(); break;
    case 12: gmCamera_801A2BB0(); break;
    case 13: {
        s32 v = ARG_P0 & 1;
        f32 r = gmCamera_801A2BF0_get_translate_x(&v);
        *ret = (int) bits(r);
        break;
    }
    case 14: gmCamera_801A2BF0(); break;
    case 15: gmCamera_801A2D44(); break;
    case 16: gmCamera_801A2FBC(); break;
    case 17: gmCamera_801A2FFC(); break;
    case 18: gmCamera_801A3048(ARG_I); break;
    case 19: gmCamera_801A3098(); break;
    case 20: gmCamera_801A30E4(); break;
    case 21: {
        HSD_GObj g;
        memset(&g, 0, sizeof g);
        g.hsd_obj = &JOBJS[ARG_I & 15];
        fn_801A31D8(&g);
        break;
    }
    case 22: gmCamera_801A31FC(); break;
    }
}

#ifndef FROM_F
#define FROM_F 0
#endif
#ifndef TO_F
#define TO_F 22
#endif
int main(void)
{
    int f;
    int i;
    for (f = FROM_F; f <= TO_F && f < NFUNCS; f++) {
        for (i = 0; i < NCASES; i++) {
            int ret = 0;
            setup_case();
            g_loglen = 0;
            g_log[0] = 0;
            memset(DIGITS, 0, sizeof DIGITS);
            call_fn(f, &ret);
            fingerprint(f, i, ret);
        }
    }
    return 0;
}
