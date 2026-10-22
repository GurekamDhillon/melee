/* C reference for rust/melee_modes/src/tu/gm_1A7A.rs: the game's gm_1A7A.c verbatim (so the two static gobj
 * pointers and the setup inlines are the game's own); the gobj / joint / camera engine, the scene data of
 * gmregtyfall.c and the toy table are stubbed here (logged; the Rust hooks log the same text).
 * Run: python rust/melee_modes/gen/turef/tu_shard537_gm.py tu_gm_1A7A */
#include "../../../../src/melee/gm/gm_1A7A.c"
#include "tu_gm_common.h"

u8 HSD_GObj_JObjKind = 1;
s8 HSD_GObj_LightKind = 2;
u8 HSD_GObj_CameraKind = 3;

HSD_Joint* gm_804D6798;
SceneDesc* gm_804D67A8;
SceneDesc* gm_804D67AC;

typedef struct {
    HSD_JObj J[8];
} Tree;

static HSD_GObj GP[8];
static int g_ng;
static Tree TREES[4];
static int g_nt;
static int g_chain;
static HSD_Joint JT_MAIN, JT_C;
static DynamicModelDesc MD_A, MD_C;
static DynamicModelDesc* MDPTR_A[1];
static DynamicModelDesc* MDPTR_C[1];
static SceneCameraDesc CAMS[1];
static HSD_CObjDesc CAMD;
static HSD_CameraAnim* ANIMS[1];
static char CAMANIM[8];
static LightList* LIGHTPTR[1];
static char LO[16];
static SceneDesc SD_A, SD_C;
static HSD_CObj CO;
static HSD_AObj AO;
static float g_toy[8];
static int g_ck;

static int gid(HSD_GObj* g)
{
    return g == NULL ? -1 : (int) (g - GP);
}
static const char* jid(HSD_JObj* j)
{
    static char buf[4][8];
    static int k;
    int t;
    int n;
    char* b = buf[k++ & 3];
    if (j == NULL) {
        return "null";
    }
    for (t = 0; t < g_nt; t++) {
        for (n = 0; n < 8; n++) {
            if (j == &TREES[t].J[n]) {
                b[0] = 't';
                b[1] = (char) ('0' + t);
                b[2] = 'j';
                b[3] = (char) ('0' + n);
                b[4] = 0;
                return b;
            }
        }
    }
    return "?";
}

void __assert(const char* f, u32 l, const char* c)
{
    (void) f;
    (void) l;
    (void) c;
    *(volatile int*) 0 = 1;
}

HSD_GObj* GObj_Create(u16 c, u8 pl, u8 pr)
{
    int n = g_ng++;
    memset(&GP[n], 0, sizeof GP[n]);
    lg("gcreate(%d,%d,%d)=g%d;", (int) c, (int) pl, (int) pr, n);
    return &GP[n];
}
void HSD_GObjObject_80390A70(HSD_GObj* g, u8 kind, void* obj)
{
    g->hsd_obj = obj;
    lg("gobj(g%d,%d,%s);", gid(g), (int) kind,
       obj == (void*) LO ? "lobj" : (obj == (void*) &CO ? "cobj" : jid((HSD_JObj*) obj)));
}
void HSD_GObj_LObjCallback(HSD_GObj* g, int x) { (void) g; (void) x; }
void HSD_GObj_803910D8(HSD_GObj* g, int x) { (void) g; (void) x; }
void HSD_GObj_JObjCallback(HSD_GObj* g, int x) { (void) g; (void) x; }
void GObj_SetupGXLink(HSD_GObj* g, GObj_RenderFunc cb, u8 gx, u32 prio)
{
    lg("gxlink(g%d,%s,%d,%u);", gid(g),
       cb == HSD_GObj_LObjCallback ? "lobjcb" : (cb == HSD_GObj_JObjCallback ? "jobjcb" : "?"), (int) gx, (unsigned) prio);
}
void GObj_SetupGXLinkMax(HSD_GObj* g, GObj_RenderFunc cb, u32 prio)
{
    lg("gxlinkmax(g%d,%s,%u);", gid(g), cb == HSD_GObj_803910D8 ? "910D8" : "?", (unsigned) prio);
}
HSD_GObjProc* HSD_GObj_SetupProc(HSD_GObj* g, HSD_GObjEvent proc, u8 prio)
{
    lg("proc(g%d,%s,%d);", gid(g),
       proc == fn_801A7A8C ? "7A8C" : (proc == fn_801A7A44 ? "7A44" : (proc == fn_801A7A68 ? "7A68" : "?")), (int) prio);
    return NULL;
}
void HSD_GObjFree(HSD_GObj* g) { lg("gfree(g%d);", gid(g)); }
HSD_JObj* HSD_JObjLoadJoint(HSD_Joint* j)
{
    int n = g_nt++;
    int k;
    Tree* t = &TREES[n];
    memset(t, 0, sizeof *t);
    for (k = 0; k < g_chain; k++) {
        t->J[k].child = &t->J[k + 1];
    }
    t->J[5].next = &t->J[6];
    for (k = 0; k < 8; k++) {
        t->J[k].flags = ((n + k + g_chain) % 3 == 0) ? JOBJ_MTX_INDEP_SRT : 0;
    }
    lg("loadjoint(%s)=t%d;", j == &JT_MAIN ? "main" : (j == &JT_C ? "chara" : "?"), n);
    return &t->J[0];
}
void(HSD_JObjSetMtxDirty)(HSD_JObj* j) { lg("dirty(%s);", jid(j)); }
void gm_8016895C(HSD_JObj* j, DynamicModelDesc* m, int idx)
{
    lg("8016895C(%s,%s,%d);", jid(j), m == &MD_A ? "mdA" : "?", idx);
}
void HSD_JObjReqAnimAll(HSD_JObj* j, f32 f) { lg("reqanim(%s,%08x);", jid(j), (unsigned) bits(f)); }
void HSD_JObjAnimAll(HSD_JObj* j) { lg("anim(%s);", jid(j)); }
void HSD_CObjAddAnim(HSD_CObj* c, HSD_CameraAnim* a)
{
    lg("cobjaddanim(%s,%s);", c == &CO ? "cobj" : "?", (char*) a == CAMANIM ? "anim" : "?");
}
void HSD_CObjReqAnim(HSD_CObj* c, f32 f) { lg("cobjreq(%s,%08x);", c == &CO ? "cobj" : "?", (unsigned) bits(f)); }
void HSD_CObjAnim(HSD_CObj* c) { lg("cobjanim(%s);", c == &CO ? "cobj" : "?"); }
HSD_LObj* lb_80011AC4(LightList** l)
{
    lg("11AC4(%s);", l == LIGHTPTR ? "lights" : "?");
    return (HSD_LObj*) LO;
}
HSD_CObj* lb_80013B14(HSD_CameraDescPerspective* d)
{
    lg("13B14(%s);", (void*) d == (void*) &CAMD ? "camd" : "?");
    return &CO;
}
void lb_8000C1C0(HSD_JObj* j, HSD_JObj* c) { lg("C1C0(%s,%s);", jid(j), jid(c)); }
void lb_8000C290(HSD_JObj* j, HSD_JObj* c) { lg("C290(%s,%s);", jid(j), jid(c)); }
CharacterKind gm_801BEFB0(void) { lg("EFB0;"); return (CharacterKind) g_ck; }
bool gm_801A659C(int i) { int r = (i + 3) % 7; lg("659C(%d)=%d;", i, r); return r; }
void* gm_GetCurrentSceneEnterData(void) { lg("enterdata;"); return NULL; }
void gm_801A9630(void) { lg("9630;"); }
void mn_8022F0F0(int v) { lg("F0F0(%d);", v); }
float Toy_803060BC(int a, int b)
{
    float r = g_toy[b];
    lg("toy(%d,%d)=%08x;", a, b, (unsigned) bits(r));
    return r;
}

static const char* FUNC_NAMES[] = {"fn_801A7A44", "fn_801A7A68", "fn_801A7A8C", "gm_801A7B00"};
#define NFUNCS 4
static const int NCASES[] = {20, 20, 200, 600};

static void world(void)
{
    int k;
    u32 r;
    g_ng = g_nt = 0;
    memset(GP, 0, sizeof GP);
    memset(TREES, 0, sizeof TREES);
    memset(&MD_A, 0, sizeof MD_A);
    memset(&MD_C, 0, sizeof MD_C);
    MD_C.joint = &JT_C;
    MDPTR_A[0] = &MD_A;
    MDPTR_C[0] = &MD_C;
    memset(CAMS, 0, sizeof CAMS);
    CAMS[0].desc = &CAMD;
    ANIMS[0] = (HSD_CameraAnim*) CAMANIM;
    CAMS[0].anims = ANIMS;
    LIGHTPTR[0] = NULL;
    memset(&SD_A, 0, sizeof SD_A);
    memset(&SD_C, 0, sizeof SD_C);
    SD_A.models = MDPTR_A;
    SD_A.cameras = CAMS;
    SD_A.lights = LIGHTPTR;
    SD_C.models = MDPTR_C;
    gm_804D67A8 = &SD_A;
    gm_804D67AC = &SD_C;
    gm_804D6798 = &JT_MAIN;
    memset(&CO, 0, sizeof CO);
    memset(&AO, 0, sizeof AO);
    CO.aobj = &AO;
    r = rnd();
    AO.curr_frame = (float) (r % 5);
    r = rnd();
    AO.end_frame = (float) (r % 5);
    for (k = 0; k < 8; k++) {
        g_toy[k] = rf();
    }
    r = rnd();
    g_ck = (int) (r % 30);
    r = rnd();
    g_chain = 1 + (int) (r % 7);
}

static void dump_tree(int t)
{
    HSD_JObj* c = &TREES[t].J[1];
    printf("|T%d:%08x,%08x,%08x,%08x,%08x,%08x,%08x,%x", t, (unsigned) bits(c->translate.x), (unsigned) bits(c->translate.y),
           (unsigned) bits(c->translate.z), (unsigned) bits(c->rotate.y), (unsigned) bits(c->scale.x),
           (unsigned) bits(c->scale.y), (unsigned) bits(c->scale.z), (unsigned) c->flags);
}

int main(void)
{
    int f;
    int i;
    for (f = 0; f < NFUNCS; f++) {
        for (i = 0; i < NCASES[f]; i++) {
            int t;
            world();
            g_loglen = 0;
            g_log[0] = 0;
            gm_804D67B0 = NULL;
            gm_804D67B4 = NULL;
            if (f == 0 || f == 1) {
                HSD_JObj* j = HSD_JObjLoadJoint(&JT_MAIN);
                GP[0].hsd_obj = j;
                g_ng = 1;
                g_loglen = 0;
                g_log[0] = 0;
                if (f == 0) fn_801A7A44(&GP[0]); else fn_801A7A68(&GP[0]);
            } else if (f == 2) {
                g_ng = 4;
                GP[0].hsd_obj = &CO;
                gm_804D67B0 = &GP[1];
                gm_804D67B4 = &GP[2];
                fn_801A7A8C(&GP[0]);
            } else {
                gm_801A7B00();
            }
            printf("%d.%d %s|log=%s|b0=%d,b4=%d", f, i, FUNC_NAMES[f], g_log, gid(gm_804D67B0), gid(gm_804D67B4));
            for (t = 0; t < g_nt; t++) {
                dump_tree(t);
            }
            printf("|gx=%llx,%llx\n", (unsigned long long) GP[0].gxlink_prios, (unsigned long long) GP[1].gxlink_prios);
        }
    }
    return 0;
}
