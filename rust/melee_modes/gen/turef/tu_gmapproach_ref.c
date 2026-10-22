/* C reference for rust/melee_modes/src/tu/gmapproach.rs: the game's gmapproach.c verbatim, with the
 * engine calls it makes stubbed (logged; the Rust hooks log the same text). The scene descriptor, the
 * joint chain and the pad table are built here. Run: python rust/melee_modes/gen/turef/tu_shard457_gm.py */
#include "../../../../src/melee/gm/gmapproach.c"
#include "tu_gm_common.h"

HSD_PadStatus HSD_PadCopyStatus[4];

/* gobj kind tags (gobj.h externs; values only matter as ids to the stub log). */
u8 HSD_GObj_JObjKind = 1;
s8 HSD_GObj_LightKind = 2;
u8 HSD_GObj_CameraKind = 3;

static HSD_Archive ARCH;
static HSD_CObj CO;
static char LO[64]; /* HSD_LObj is only forward-declared in the TU headers */
static HSD_JObj LOADED_JOINT;
static HSD_GObj POOL[3];
static int g_pool;
static HSD_CObjDesc CAMDESC;
static LightList LIGHTS;
static LightList* LIGHTPTR[1];
static DynamicModelDesc MODEL;
static HSD_Joint MJOINT;
static DynamicModelDesc* MODELPTR[1];
static SceneCameraDesc CAMS[1];
static SceneDesc SD;
static HSD_JObj J[4];
static HSD_DObj D;
static HSD_MObj M;
static HSD_TObj T;
static HSD_AObj A;
static HSD_GObj GOBJ;

void HSD_GObj_803910D8(HSD_GObj* g, int x)
{
    (void) g;
    (void) x;
}
void HSD_GObj_LObjCallback(HSD_GObj* g, int x)
{
    (void) g;
    (void) x;
}
void HSD_GObj_JObjCallback(HSD_GObj* g, int x)
{
    (void) g;
    (void) x;
}

static const char* gobjid(void* p)
{
    int k;
    for (k = 0; k < 3; k++) {
        if (p == (void*) &POOL[k]) {
            return k == 0 ? "g0" : k == 1 ? "g1" : "g2";
        }
    }
    if (p == (void*) &GOBJ) return "gP";
    return "?";
}
static const char* jid(void* p)
{
    if (p == NULL) return "null";
    if (p == (void*) &J[0]) return "j0";
    if (p == (void*) &J[1]) return "j1";
    if (p == (void*) &J[2]) return "j2";
    if (p == (void*) &J[3]) return "j3";
    if (p == (void*) &LOADED_JOINT) return "loaded";
    return "?";
}

HSD_Archive* lbArchive_80016DBC(const char* filename, void* symbols, ...)
{
    lg("archive(%s)", filename);
    *(SceneDesc**) symbols = &SD;
    return &ARCH;
}
void lbArchive_80016EFC(HSD_Archive* a)
{
    lg("archclose(%s)", a == &ARCH ? "arch" : "?");
}
HSD_GObj* GObj_Create(u16 classifier, u8 p_link, u8 priority)
{
    HSD_GObj* g = &POOL[g_pool < 3 ? g_pool : 2];
    g_pool++;
    lg("gobj(%u,%u,%u)", (unsigned) classifier, (unsigned) p_link, (unsigned) priority);
    return g;
}
HSD_CObj* HSD_CObjLoadDesc(HSD_CObjDesc* desc)
{
    lg("cobj(%s)", (void*) desc == (void*) &CAMDESC ? "cam" : "?");
    return &CO;
}
void HSD_GObjObject_80390A70(HSD_GObj* gobj, u8 kind, void* obj)
{
    (void) obj;
    lg("setobj(%s,%u)", gobjid(gobj), (unsigned) kind);
}
void GObj_SetupGXLinkMax(HSD_GObj* gobj, GObj_RenderFunc render_cb, u32 priority)
{
    lg("gxmax(%s,%s,%u)", gobjid(gobj), (void*) render_cb == (void*) HSD_GObj_803910D8 ? "cb803910D8" : "?",
       (unsigned) priority);
}
HSD_LObj* lb_80011AC4(LightList** lights)
{
    lg("lobj(%s)", (void*) lights == (void*) LIGHTPTR ? "lights" : "?");
    return (HSD_LObj*) LO;
}
void GObj_SetupGXLink(HSD_GObj* gobj, GObj_RenderFunc render_cb, u8 gx_link, u32 priority)
{
    lg("gx(%s,%s,%u,%u)", gobjid(gobj),
       (void*) render_cb == (void*) HSD_GObj_LObjCallback ? "lobjcb"
       : (void*) render_cb == (void*) HSD_GObj_JObjCallback ? "jobjcb" : "?",
       (unsigned) gx_link, (unsigned) priority);
}
HSD_JObj* HSD_JObjLoadJoint(HSD_Joint* joint)
{
    lg("jload(%s)", (void*) joint == (void*) &MJOINT ? "mjoint" : "?");
    return &LOADED_JOINT;
}
void gm_8016895C(HSD_JObj* jobj, DynamicModelDesc* model, int idx)
{
    lg("m895C(%s,%s,%d)", jid(jobj), (void*) model == (void*) &MODEL ? "m0" : "?", idx);
}
void HSD_JObjReqAnimAll(HSD_JObj* jobj, f32 frame)
{
    lg("reqanim(%s,%08x)", jid(jobj), bits(frame));
}
void HSD_JObjAnimAll(HSD_JObj* jobj)
{
    lg("anim(%s)", jid(jobj));
}
HSD_GObjProc* HSD_GObj_SetupProc(HSD_GObj* gobj, HSD_GObjEvent proc, u8 prio)
{
    lg("proc(%s,%s,%u)", gobjid(gobj), (void*) proc == (void*) fn_801AD920 ? "p920" : "?", (unsigned) prio);
    return NULL;
}
void HSD_TObjReqAnimAll(HSD_TObj* tobj, f32 startframe)
{
    lg("treq(%s,%08x)", tobj == &T ? "t" : "?", bits(startframe));
}
void HSD_AObjSetRate(HSD_AObj* aobj, f32 rate)
{
    lg("arate(%s,%08x)", aobj == &A ? "a" : "?", bits(rate));
}
void gm_801A4B60(void)
{
    lg("gm4B60");
}
int lbAudioAx_80023F28(int id)
{
    lg("audio(%d)", id);
    return 0;
}
int lbAudioAx_800236DC(void)
{
    lg("audioclose");
    return 0;
}

static const char* FUNC_NAMES[] = {"fn_801AD920", "gm_801ADB04", "gm_Scene_Approach_OnFrame", "gm_Scene_Approach_OnEnter",
                                   "gm_Scene_Approach_OnExit"};
#define NFUNCS ((int) (sizeof FUNC_NAMES / sizeof FUNC_NAMES[0]))
#define NCASES 200

static void setup_case(void)
{
    int k;
    gm_80480D98.x0 = rnd() & 1 ? &ARCH : NULL;
    gm_80480D98.x4 = (int) (rnd() % 30);
    gm_80480D98.x8 = (u16) rnd();
    gm_80480D98.xA = (u16) rnd();
    gm_80480D98.xC = (u16) rnd();
    gm_80480D98.xE = (u8) (rnd() % 4);
    for (k = 0; k < 4; k++) {
        HSD_PadCopyStatus[k].trigger = rnd();
    }
    g_pool = 0;
    memset(POOL, 0, sizeof POOL);
    /* the joint chain: gobj->hsd_obj = J0; child J1; J1.next = J2; J2.next = J3 with the dobj chain */
    memset(J, 0, sizeof J);
    J[0].child = &J[1];
    J[1].next = &J[2];
    J[2].next = &J[3];
    J[3].u.dobj = &D;
    memset(&D, 0, sizeof D);
    D.mobj = &M;
    memset(&M, 0, sizeof M);
    M.tobj = &T;
    memset(&T, 0, sizeof T);
    T.aobj = &A;
    memset(&A, 0, sizeof A);
    memset(&GOBJ, 0, sizeof GOBJ);
    GOBJ.hsd_obj = &J[0];
    CAMS[0].desc = &CAMDESC;
    LIGHTPTR[0] = &LIGHTS;
    MODELPTR[0] = &MODEL;
    MODEL.joint = &MJOINT;
    SD.cameras = CAMS;
    SD.lights = LIGHTPTR;
    SD.models = MODELPTR;
}

static void fingerprint(int f, int i, u8 arg0, u8 arg1)
{
    printf("%d.%d %s|log=%s|x0=%d|x4=%d|x8=%u|xA=%u|xC=%u|xE=%u|gx=%u,%u,%u|arg=%d,%d\n", f, i, FUNC_NAMES[f], g_log,
           gm_80480D98.x0 != NULL, gm_80480D98.x4, (unsigned) gm_80480D98.x8, (unsigned) gm_80480D98.xA,
           (unsigned) gm_80480D98.xC, (unsigned) gm_80480D98.xE, (unsigned) (unsigned) POOL[0].gxlink_prios,
           (unsigned) (unsigned) POOL[1].gxlink_prios, (unsigned) (unsigned) POOL[2].gxlink_prios, (int) (signed char) arg0,
           (int) (signed char) arg1);
}

int main(void)
{
    int f;
    int i;
    for (f = 0; f < NFUNCS; f++) {
        for (i = 0; i < NCASES; i++) {
            u8 arg[2];
            setup_case();
            for (int k = 0; k < 4; k++) {
                (void) k;
            }
            arg[0] = (u8) (rnd() % 40);
            arg[1] = (u8) (rnd() % 4);
            g_loglen = 0;
            g_log[0] = 0;
            if (f == 0) {
                fn_801AD920(&GOBJ);
            } else if (f == 1) {
                gm_801ADB04();
            } else if (f == 2) {
                gm_Scene_Approach_OnFrame();
            } else if (f == 3) {
                gm_Scene_Approach_OnEnter(arg);
            } else {
                gm_Scene_Approach_OnExit(NULL);
            }
            fingerprint(f, i, arg[0], arg[1]);
        }
    }
    return 0;
}
