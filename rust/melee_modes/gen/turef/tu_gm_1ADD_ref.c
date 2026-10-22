/* C reference for rust/melee_modes/src/tu/gm_1ADD.rs: the game's gm_1ADD.c verbatim; the allocator, the
 * gobj / joint / text engine and the pad arrays are stubbed here (logged; the Rust hooks log the same text).
 * Every case builds a random list of boxes and buttons with the TU's own constructors, clears the log, runs
 * one function and prints the log, the return value and the whole object graph.
 * Run: python rust/melee_modes/gen/turef/tu_shard537_gm.py tu_gm_1ADD */
#include "../../../../src/melee/gm/gm_1ADD.c"
#include "tu_gm_common.h"

HSD_PadStatus HSD_PadMasterStatus[4];
HSD_PadStatus HSD_PadCopyStatus[4];
u8 HSD_GObj_JObjKind = 1;
s8 HSD_GObj_LightKind = 2;
u8 HSD_GObj_CameraKind = 3;

typedef struct {
    HSD_JObj J[10];
    HSD_DObj D;
    HSD_MObj M;
    HSD_Material MAT;
} Tree;

#define ASLOT 64
static unsigned char ARENA[ASLOT * 64];
static int g_nalloc;
static HSD_GObj GP[64];
static int g_ng;
static Tree TREES[64];
static int g_nt;
static HSD_Text TX[64];
static int g_ntx;
static int g_us;
static int g_n611;
static char CO[8];
static HSD_Joint JOINTS[2];
static DynamicModelDesc MODELS[2];
static DynamicModelDesc* MODELPTRS[2];
static SceneCameraDesc CAMS[1];
static HSD_CObjDesc CAMDESC;
static SceneDesc SD;

void HSD_GObj_803910D8(HSD_GObj* g, int x) { (void) g; (void) x; }
void HSD_GObj_JObjCallback(HSD_GObj* g, int x) { (void) g; (void) x; }

static int gid(void* p) { return (int) ((HSD_GObj*) p - GP); }
static int txid(void* p) { return (int) ((HSD_Text*) p - TX); }
static const char* jid(HSD_JObj* j)
{
    static char buf[4][16];
    static int k;
    int t;
    int n;
    char* b = buf[k++ & 3];
    if (j == NULL) {
        return "null";
    }
    for (t = 0; t < g_nt; t++) {
        for (n = 0; n < 10; n++) {
            if (j == &TREES[t].J[n]) {
                b[0] = 't';
                b[1] = (char) ('0' + t / 10);
                b[2] = (char) ('0' + t % 10);
                b[3] = 'j';
                b[4] = (char) ('0' + n);
                b[5] = 0;
                return b;
            }
        }
    }
    return "?";
}
static int nid(void* p)
{
    if (p == NULL) {
        return -1;
    }
    if (p == (void*) &gm_803DD4D0) {
        return -2;
    }
    return (int) (((unsigned char*) p - ARENA) / ASLOT);
}

void __assert(const char* f, u32 l, const char* c)
{
    (void) f;
    (void) l;
    *(volatile int*) 0 = 1;
}
void memzero(void* mem, ssize_t size) { memset(mem, 0, (size_t) size); }
void* HSD_MemAlloc(ssize_t size)
{
    int n = g_nalloc++;
    lg("alloc(%d)=m%d;", (int) size, n);
    memset(&ARENA[n * ASLOT], 0xA5, ASLOT);
    return &ARENA[n * ASLOT];
}
void HSD_Free(void* p) { lg("free(m%d);", nid(p)); }
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
    lg("gobj(g%d,%d,%s);", gid(g), (int) kind, kind == 3 ? "cobj" : jid((HSD_JObj*) obj));
}
void GObj_SetupGXLink(HSD_GObj* g, GObj_RenderFunc cb, u8 gx, u32 prio)
{
    lg("gxlink(g%d,%s,%d,%d);", gid(g), cb == HSD_GObj_JObjCallback ? "jobjcb" : "?", (int) gx, (int) prio);
}
void GObj_SetupGXLinkMax(HSD_GObj* g, GObj_RenderFunc cb, u32 prio)
{
    lg("gxlinkmax(g%d,%s,%d);", gid(g), cb == HSD_GObj_803910D8 ? "910D8" : "?", (int) prio);
}
HSD_GObjProc* HSD_GObj_SetupProc(HSD_GObj* g, HSD_GObjEvent proc, u8 prio)
{
    lg("proc(g%d,%s,%d);", gid(g), proc == fn_801AE948 ? "948" : "?", (int) prio);
    return NULL;
}
void HSD_GObjFree(HSD_GObj* g) { lg("gfree(g%d);", gid(g)); }
HSD_JObj* HSD_JObjLoadJoint(HSD_Joint* j)
{
    int n = g_nt++;
    int k;
    Tree* t = &TREES[n];
    memset(t, 0, sizeof *t);
    t->J[0].child = &t->J[1];
    t->J[1].child = &t->J[2];
    t->J[2].child = &t->J[3];
    t->J[2].next = &t->J[4];
    t->J[4].child = &t->J[5];
    t->J[5].next = &t->J[6];
    t->J[4].next = &t->J[7];
    t->J[3].next = &t->J[8];
    t->J[8].child = &t->J[9];
    t->J[2].u.dobj = &t->D;
    t->D.mobj = &t->M;
    t->M.mat = &t->MAT;
    for (k = 0; k < 10; k++) {
        int m = (n + k) % 3;
        t->J[k].translate.x = (float) (n * 3 + k) * 0.25f;
        t->J[k].translate.y = (float) (n + k * 5) * 0.5f;
        t->J[k].flags = m == 0 ? JOBJ_MTX_DIRTY : (m == 1 ? JOBJ_MTX_INDEP_SRT : 0);
    }
    lg("loadjoint(%s)=t%d;", j == &JOINTS[0] ? "jt0" : (j == &JOINTS[1] ? "jt1" : "?"), n);
    return &t->J[0];
}
void HSD_JObjSetMtxDirtySub(HSD_JObj* j)
{
    lg("dirty(%s);", jid(j));
    j->flags |= JOBJ_MTX_DIRTY;
}
void HSD_JObjSetFlagsAll(HSD_JObj* j, u32 f)
{
    lg("setflagsall(%s,%x);", jid(j), (unsigned) f);
    j->flags |= f;
}
void gm_8016895C(HSD_JObj* j, DynamicModelDesc* m, int idx)
{
    lg("8016895C(%s,%s,%d);", jid(j), m == &MODELS[0] ? "md0" : (m == &MODELS[1] ? "md1" : "?"), idx);
}
void HSD_JObjReqAnimAll(HSD_JObj* j, f32 f) { lg("reqanim(%s,%08x);", jid(j), (unsigned) bits(f)); }
void HSD_JObjAnimAll(HSD_JObj* j) { lg("anim(%s);", jid(j)); }
HSD_CObj* HSD_CObjLoadDesc(HSD_CObjDesc* d)
{
    lg("cobjload(%s);", d == &CAMDESC ? "camdesc" : "?");
    return (HSD_CObj*) CO;
}
HSD_Text* HSD_SisLib_803A5ACC(int a, s32 b, f32 x, f32 y, f32 z, f32 w, f32 h)
{
    int n = g_ntx++;
    memset(&TX[n], 0, sizeof TX[n]);
    lg("text(%d,%d,%08x,%08x,%08x,%08x,%08x)=x%d;", a, (int) b, (unsigned) bits(x), (unsigned) bits(y),
       (unsigned) bits(z), (unsigned) bits(w), (unsigned) bits(h), n);
    return &TX[n];
}
void HSD_SisLib_803A6368(HSD_Text* t, s32 v) { lg("setText(x%d,%d);", txid(t), (int) v); }
void HSD_SisLib_803A62A0(s32 a, char* f, char* s) { lg("sisload(%d,%s,%s);", (int) a, f, s); }
int HSD_SisLib_803A611C(int a, HSD_GObj* g, u16 b, u8 c, u8 d, u8 e, u8 f, u32 h)
{
    int r = 77 + g_n611++;
    lg("sis611C(%d,g%d,%d,%d,%d,%d,%d,%u)=%d;", a, gid(g), (int) b, (int) c, (int) d, (int) e, (int) f, (unsigned) h, r);
    return r;
}
void HSD_SisLib_803A5CC4(HSD_Text* t) { lg("textfree(x%d);", txid(t)); }
HSD_Archive* lbArchive_80016DBC(const char* filename, void* symbols, ...)
{
    va_list ap;
    const char* sym;
    va_start(ap, symbols);
    sym = va_arg(ap, const char*);
    lg("archive(%s,%s,%d);", filename, sym, va_arg(ap, int));
    va_end(ap);
    *(SceneDesc**) symbols = &SD;
    return NULL;
}
bool lbLang_IsSavedLanguageUS(void) { lg("isUS;"); return g_us; }

static const char* FUNC_NAMES[] = {"gm_801ADDD8", "gm_801ADE1C", "gm_801AE050", "gm_801AE44C", "gm_801AE544",
                                   "gm_801AE640", "gm_801AE74C", "gm_801AE848", "fn_801AE948", "gm_801AEBB0",
                                   "gm_801AECC4", "gm_801AEDC8"};
#define NFUNCS ((int) (sizeof FUNC_NAMES / sizeof FUNC_NAMES[0]))
#define NCASES 300

static void reset_world(void)
{
    int i;
    memset(ARENA, 0, sizeof ARENA);
    g_nalloc = g_ng = g_nt = g_ntx = g_n611 = 0;
    memset(GP, 0, sizeof GP);
    memset(TX, 0, sizeof TX);
    memset(TREES, 0, sizeof TREES);
    memset(&gm_803DD4D0, 0, sizeof gm_803DD4D0);
    gm_804D686C = 0;
    gm_804D6868 = NULL;
    memset(MODELS, 0, sizeof MODELS);
    MODELS[0].joint = &JOINTS[0];
    MODELS[1].joint = &JOINTS[1];
    MODELPTRS[0] = &MODELS[0];
    MODELPTRS[1] = &MODELS[1];
    memset(CAMS, 0, sizeof CAMS);
    CAMS[0].desc = &CAMDESC;
    SD.models = MODELPTRS;
    SD.cameras = CAMS;
    SD.lights = NULL;
    SD.fogs = NULL;
    for (i = 0; i < 4; i++) {
        HSD_PadMasterStatus[i].err = (s8) ((rnd() % 3) == 0 ? -1 : 0);
        HSD_PadCopyStatus[i].trigger = rnd();
    }
}

static void dump_graph(void)
{
    int t;
    int n;
    int x;
    int g;
    struct unkd4d0* b;
    printf("|g686C=%d,head=%d,%d", gm_804D686C, nid(gm_803DD4D0.next), (int) gm_803DD4D0.unkC);
    for (b = gm_803DD4D0.next; b != NULL; b = b->next) {
        struct unkd4d0_8* bt;
        printf("|B%d:%d,%d,%08x,%08x,g%d,x%d,p%d", nid(b), (int) b->unkC, (int) b->unk10, (unsigned) bits(b->unk14),
               (unsigned) bits(b->unk18), b->unk1C ? gid(b->unk1C) : -1, b->unk20 ? txid(b->unk20) : -1, nid(b->unk4));
        for (bt = b->unk8; bt != NULL; bt = bt->next) {
            printf(";b%d:%d,%d,%08x,%08x,g%d,x%d,p%d", nid(bt), (int) bt->unk8, (int) bt->unkC,
                   (unsigned) bits(bt->unk10), (unsigned) bits(bt->unk14), bt->unk20 ? gid(bt->unk20) : -1,
                   bt->unk24 ? txid(bt->unk24) : -1, nid(bt->unk4));
        }
    }
    for (g = 0; g < g_ng; g++) {
        printf("|G%d:%llx,%d", g, (unsigned long long) GP[g].gxlink_prios, GP[g].hsd_obj == NULL ? -1 : 1);
    }
    for (t = 0; t < g_nt; t++) {
        printf("|T%d:", t);
        for (n = 0; n < 10; n++) {
            printf("%08x,%08x,%x;", (unsigned) bits(TREES[t].J[n].translate.x), (unsigned) bits(TREES[t].J[n].translate.y),
                   (unsigned) TREES[t].J[n].flags);
        }
        printf("%u,%u,%u", (unsigned) TREES[t].MAT.diffuse.r, (unsigned) TREES[t].MAT.diffuse.g,
               (unsigned) TREES[t].MAT.diffuse.b);
    }
    for (x = 0; x < g_ntx; x++) {
        printf("|X%d:%u,%u,%u,%08x,%08x,%u,%u", x, (unsigned) TX[x].text_color.r, (unsigned) TX[x].text_color.g,
               (unsigned) TX[x].text_color.b, (unsigned) bits(TX[x].x34.x), (unsigned) bits(TX[x].x34.y),
               (unsigned) TX[x].default_alignment, (unsigned) TX[x].default_fitting);
    }
}

int main(void)
{
    int f;
    int i;
    for (f = 0; f < NFUNCS; f++) {
        for (i = 0; i < NCASES; i++) {
            unsigned long long ret = 0;
            int nbox;
            int nbtn;
            int k;
            u32 a;
            u32 b;
            u32 c;
            float x;
            float y;
            reset_world();
            g_us = (int) (rnd() & 1);
            g_loglen = 0;
            g_log[0] = 0;
            gm_801AEBB0();
            nbox = (int) (rnd() % 4);
            for (k = 0; k < nbox; k++) {
                a = rnd();
                b = rnd();
                x = rf();
                y = rf();
                gm_801ADE1C((int) (a % 3), (int) (b % 30), x, y);
            }
            nbtn = (int) (rnd() % 5);
            for (k = 0; k < nbtn; k++) {
                a = rnd();
                b = rnd();
                c = rnd();
                x = rf();
                y = rf();
                gm_801AE050((int) (a % 3), (int) (b % 2), (int) (c % 10), x, y);
            }
            g_loglen = 0;
            g_log[0] = 0;
            a = rnd();
            b = rnd();
            x = rf();
            y = rf();
            c = rnd();
            switch (f) {
            case 0: gm_801ADDD8(); break;
            case 1: ret = gm_801ADE1C((int) (a % 3), (int) (b % 30), x, y); break;
            case 2: ret = gm_801AE050((int) (a % 3), (int) (b % 2), (int) (c % 10), x, y); break;
            case 3: gm_801AE44C((int) (a % 3), x); break;
            case 4: gm_801AE544((int) (a % 3), x); break;
            case 5: gm_801AE640((int) (a % 3), (bool) (b % 2)); break;
            case 6: gm_801AE74C((int) (a % 3), (bool) (b % 2)); break;
            case 7: gm_801AE848((int) (a % 3)); break;
            case 8: fn_801AE948(NULL); break;
            case 9: gm_801AEBB0(); break;
            case 10: ret = (unsigned long long) gid(gm_801AECC4((int) (a % 2))); break;
            default: ret = gm_801AEDC8(); break;
            }
            printf("%d.%d %s|log=%s|ret=%llx", f, i, FUNC_NAMES[f], g_log, ret);
            dump_graph();
            printf("\n");
        }
    }
    return 0;
}
