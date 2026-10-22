/* State oracle for src/tu/grshrineroute.rs (see tu_grrcruise_ref.c / tu_grpushon_ref.c). The TU is
 * compiled from rust/target/cref/grshrineroute_tu.c: the game file without two dead
 * GET_GROUND(0) statements and with the Ground_801C5840()/Ground_801C5940() arguments sequenced
 * left to right as PPC does (tu_shard585 config). */
#include "grshrineroute_tu.c"
#include "oracle.h"
#include "tu_gr_common.h"

static int GNULL;
static int JNULL;
static int LDNULL;
static int FPOS;
static float SCALE;
static HSD_GObj* PGOBJ;

static HSD_JObj* mkjobj2(void)
{
    HSD_JObj* j = mkjobj();
    j->flags &= ~JOBJ_USE_QUATERNION; /* HSD_JObjSetRotationX/Y assert it clear */
    return j;
}
static HSD_LObj* mklobj(void)
{
    HSD_LObj* l = (HSD_LObj*) AZ(sizeof(HSD_LObj));
    l->flags = (u16) rnd();
    l->next = 0;
    return l;
}
static HSD_GObj* mkfgobj(void)
{
    return (HSD_GObj*) AZ(sizeof(HSD_GObj));
}
static struct grShrineRoute_YakumonoParam* mkyp(void)
{
    struct grShrineRoute_YakumonoParam* p =
        (struct grShrineRoute_YakumonoParam*) AZ(sizeof(struct grShrineRoute_YakumonoParam));
    p->x0 = AZ(16);
    p->x4 = AZ(16);
    p->x8 = AZ(16);
    p->xC = AZ(16);
    p->x10 = (int) (rnd() % 50);
    p->x14 = (float) (rnd() % 100) / 50.0f;
    p->x18 = (float) (rnd() % 100) / 50.0f;
    p->x1C = (float) (rnd() % 100) / 5.0f;
    p->x20 = (float) (rnd() % 100) / 5.0f;
    p->x24 = (rnd() % 3 == 0) ? 0 : (int) (1 + rnd() % 4);
    p->spawn_desc.kind = (u16) rnd();
    p->spawn_desc.x2 = (u8) rnd();
    p->spawn_desc.respawn = (u8) rnd();
    return p;
}
#include "tu_grshrineroute_stubs.inc"

/* variadic efSync_Spawn(gfx_id, gobj, jobj, &scale) */
void* efSync_Spawn(s32 gfx_id, HSD_GObj* g, ...)
{
    va_list ap;
    HSD_JObj* j;
    float* sc;
    va_start(ap, g);
    j = va_arg(ap, HSD_JObj*);
    sc = va_arg(ap, float*);
    va_end(ap);
    printf("efSync_Spawn");
    TRI(gfx_id); TRP(g); TRP(j); TRF(*sc);
    printf("\n");
    return 0;
}

static unsigned char* CB[64];
static int NCB;
/* the light gobj: a base object followed by n - 1 lights */
static HSD_GObj* mklights(int n)
{
    HSD_GObj* g = (HSD_GObj*) AZ(sizeof(HSD_GObj));
    HSD_LObj* prev = 0;
    int i;
    for (i = 0; i < n; i++) {
        HSD_LObj* l = mklobj();
        if (prev) prev->next = l;
        else g->hsd_obj = l;
        prev = l;
    }
    g->classifier = 12;
    return g;
}

/* the stage's light gobjs on GX plink 3: n decoys (wrong classifier) ahead of the real one */
static void setup_plink(void)
{
    HSD_GObj** pl = (HSD_GObj**) AZ(8 * sizeof(HSD_GObj*));
    int nd = (int) (rnd() % 3);
    int d;
    HSD_GObj* head = 0;
    HSD_GObj* tail = 0;
    HSD_GObjPLinkHead = pl;
    for (d = 0; d <= nd; d++) {
        HSD_GObj* l = mklights(16 + (int) (rnd() % 5));
        if (d < nd) l->classifier = 7; /* a decoy ahead of the real one */
        if (tail) tail->next = l;
        else head = l;
        tail = l;
    }
    pl[HSD_GOBJ_PLINK_LIGHT] = head;
}

static void greset(void)
{
    gr_reset();
    NCB = 0;
    GNULL = 0;
    JNULL = 0;
    LDNULL = 0;
    FPOS = 0;
    SCALE = 0.0f;
    PGOBJ = mkfgobj();
    PGOBJ->hsd_obj = mkjobj2();
    setup_plink();
}

static void state(void)
{
    int i;
    printf("rs %u\n", ORS);
    printf("si %d %d %u %d\n", (int) stage_info.unk8C.b4, (int) stage_info.unk8C.b5, (unsigned) stage_info.flags,
           (int) stage_info.x6DC);
    DUMP("yp", yakumono_param);
    gnd_canon_begin();
    for (i = 0; i < NCB; i++) canon_add(CB[i], 1);
    DUMPARENA();
    canon_restore();
    gnd_canon_end();
    state_end();
}

static CollData* mkcoll(int v)
{
    CollData* c = (CollData*) AZ(sizeof(CollData));
    *(unsigned char*) &c->x34_flags = (unsigned char) (v << 1);
    printf("cf %d\n", (int) c->x34_flags.b1234);
    CB[NCB++] = (unsigned char*) &c->x34_flags;
    return c;
}

/* a map-ground gobj (a symbol / effect / hub) */
static HSD_GObj* mksym(void)
{
    HSD_GObj* g = mkgobj();
    ((Ground*) g->user_data)->map_id = rnd() % 5;
    ((HSD_JObj*) g->hsd_obj)->flags &= ~JOBJ_USE_QUATERNION;
    return g;
}
/* the route hub: symbols, the effect gobj word, platforms */
static HSD_GObj* mkhub(int randomize)
{
    HSD_GObj* g = mkgobj();
    Ground* gp = (Ground*) g->user_data;
    int i;
    ((HSD_JObj*) g->hsd_obj)->flags &= ~JOBJ_USE_QUATERNION;
    for (i = 0; i < 6; i++) gp->u.map.symbol[i] = (!randomize && rnd() % 5 == 0) ? 0 : mksym();
    for (i = 0; i < 3; i++) {
        gp->u.shrineroute.platforms[i].jobj = (rnd() % 4 == 0) ? 0 : mkjobj2();
        gp->u.shrineroute.platforms[i].offset.x = rf();
        gp->u.shrineroute.platforms[i].offset.y = rf();
        gp->u.shrineroute.platforms[i].offset.z = rf();
    }
    if (!randomize) return g;
    gp->u.shrineroute.xC4 = (u16) (rnd() % 8);
    gp->u.shrineroute.xC6 = (u16) (rnd() % 64);
    gp->u.shrineroute.xC8 = (u16) (0xBD + rnd() % 6);
    gp->u.shrineroute.xCA = (u16) (rnd() % 2);
    gp->u.shrineroute.xCC = (u16) (rnd() % 2);
    gp->u.shrineroute.xCE = (u16) (rnd() % 3 == 0 ? 0 : rnd() % 70);
    gp->u.shrineroute.xD0 = (u16) rnd();
    gp->u.shrineroute.xD4 = (rnd() % 3 == 0) ? 0 : (u32) (size_t) mksym();
    return g;
}
#define FN(x) fnreg(#x, (void*) x)
int main(void)
{
    int c, k;
    arena_init();
    SIZE(Ground);
    SIZE(HSD_GObj);
    SIZE(HSD_JObj);
    SIZE(HSD_LObj);
    SIZE(CollData);
    SIZE(StageCallbacks);
    SIZE(StageData);
    SIZE(HSD_LightDesc);
    SIZE(struct grShrineRoute_YakumonoParam);
    SIZE(struct grSh_Route_LightConfig);
    reg("cb", stage_callbacks, sizeof(stage_callbacks));
    reg("sd", &grSh_Route_StageData, sizeof(grSh_Route_StageData));
    reg("dat", grSh_Route_StageData.data1, 16);
    reg("lc", light_config, sizeof(light_config));
    reg("l0", &lobj0, sizeof(lobj0));
    reg("l1", &lobj1, sizeof(lobj1));
    reg("s0", &lobj0_shininess, 4);
    reg("s1", &lobj1_shininess, 4);
    FN(grShrineRoute_OnDemoInit); FN(grShrineRoute_OnInit); FN(grShrineRoute_OnLoad); FN(grShrineRoute_OnStart);
    FN(grShrineRoute_802088B8); FN(grShrineRoute_802088C0); FN(stageGObj0_OnInit); FN(grShrineRoute_802089D8);
    FN(grShrineRoute_802089E0); FN(grShrineRoute_802089E4); FN(grShrineRoute_802089E8); FN(grShrineRoute_80208A28);
    FN(grShrineRoute_80208A30); FN(grShrineRoute_80208A34); FN(fn_80208A38); FN(grShrineRoute_80208D14);
    FN(grShrineRoute_80208F0C); FN(grShrineRoute_80208F14); FN(grShrineRoute_80208F70); FN(grShrineRoute_80209AEC);
    FN(grShrineRoute_80209AF0); FN(grShrineRoute_80209BE4); FN(grShrineRoute_80209BEC); FN(grShrineRoute_8020A100);
    FN(grShrineRoute_8020A104); FN(grShrineRoute_8020A214); FN(grShrineRoute_8020A21C); FN(grShrineRoute_8020A864);
    FN(stageGObj1_3_OnInit); FN(grShrineRoute_8020A894); FN(grShrineRoute_8020A89C); FN(grShrineRoute_8020A8A0);
    FN(grShrineRoute_8020A8A4); FN(grShrineRoute_8020AA40); FN(grShrineRoute_8020AB58); FN(grShrineRoute_8020AC44);
    FN(grShrineRoute_8020AD24); FN(onJointCollision); FN(grShrineRoute_8020AE08); FN(grShrineRoute_8020AF38);
    FN(grShrineRoute_8020B020); FN(grShrineRoute_8020B0AC); FN(grShrineRoute_OnTouchLine);
    FN(grShrineRoute_OnCheckShadowRender); FN(grShrineRoute_ScriptQuery);
    fnreg("grDisplay_801C5DB0", (void*) grDisplay_801C5DB0);
    DUMP("cb", stage_callbacks);
    DUMP("sd", grSh_Route_StageData);
    DUMP("lc", light_config);
    DUMP("tb", grSh_Route_803E58E0);
    DUMP("l0", lobj0);
    DUMP("l1", lobj1);
    DUMP("s0", lobj0_shininess);
    DUMP("s1", lobj1_shininess);
    state_end();

    for (c = 0; c < 40; c++) { /* OnInit / OnLoad / OnStart / OnDemoInit */
        greset();
        CASE(c);
        rseed(100 + c);
        yakumono_param = 0;
        GNULL = c % 2;
        grShrineRoute_OnInit();
        state();
        grShrineRoute_OnLoad();
        state();
        yakumono_param = mkyp();
        grShrineRoute_OnStart();
        grShrineRoute_OnDemoInit((int) rnd());
        printf("r %d\n", (int) grShrineRoute_802088B8());
        state();
    }
    for (c = 0; c < 28; c++) { /* grShrineRoute_802088C0: the stage gobj setup, each id */
        greset();
        CASE(1000 + c);
        rseed(2000 + c);
        yakumono_param = mkyp();
        GNULL = c % 2;
        printf("r"); TRP(grShrineRoute_802088C0(c % 7)); printf("\n");
        state();
    }
    for (c = 0; c < 20; c++) { /* the plain callbacks */
        HSD_GObj* g;
        greset();
        CASE(2000 + c);
        rseed(3000 + c);
        g = mkgobj();
        stageGObj0_OnInit(g);
        stageGObj1_3_OnInit(g);
        grShrineRoute_802089E8(g);
        state();
        grShrineRoute_802089E0(g); grShrineRoute_802089E4(g); grShrineRoute_80208A30(g); grShrineRoute_80208A34(g);
        grShrineRoute_80209AEC(g); grShrineRoute_8020A100(g); grShrineRoute_8020A864(g); grShrineRoute_8020A89C(g);
        grShrineRoute_8020A8A0(g);
        printf("r %d %d %d %d %d %d\n", (int) grShrineRoute_802089D8(g), (int) grShrineRoute_80208A28(g),
               (int) grShrineRoute_80208F0C(g), (int) grShrineRoute_80209BE4(g), (int) grShrineRoute_8020A214(g),
               (int) grShrineRoute_8020A894(g));
        printf("r %d\n", (int) (grShrineRoute_OnTouchLine(0) == 0));
        state();
    }
    for (c = 0; c < 60; c++) { /* fn_80208A38 (the symbols) and grShrineRoute_80208D14 (the route hub setup) */
        HSD_GObj* g;
        greset();
        CASE(3000 + c);
        rseed(4000 + c);
        yakumono_param = mkyp();
        g = mkgobj();
        fn_80208A38(g);
        state();
        g = mkgobj();
        JNULL = c % 2;
        grShrineRoute_80208D14(g);
        state();
    }
    for (c = 0; c < 80; c++) { /* grShrineRoute_80208F14 */
        HSD_GObj* g;
        Ground* gp;
        greset();
        CASE(4000 + c);
        rseed(5000 + c);
        g = mkgobj();
        gp = (Ground*) g->user_data;
        gp->u.shrineroute.xCA = (u16) (rnd() % 2);
        gp->u.shrineroute.xCC = (u16) (rnd() % 2);
        printf("r %d\n", (int) grShrineRoute_80208F14());
        state();
    }
    for (c = 0; c < 700; c++) { /* grShrineRoute_80208F70: the route state machine */
        HSD_GObj* g;
        greset();
        CASE(5000 + c);
        rseed(6000 + c);
        yakumono_param = mkyp();
        g = mkhub(1);
        for (k = 0; k < 4; k++) {
            Ground* hp = (Ground*) g->user_data;
            /* case 4 reads the effect gobj even when it is NULL (a console null read): keep it set */
            if (hp->u.shrineroute.xC4 == 4 && hp->u.shrineroute.xD4 == 0) hp->u.shrineroute.xD4 = (u32) (size_t) mksym();
            /* case 0 asserts the chosen symbol exists (case 2 frees them): keep the table full */
            if (hp->u.shrineroute.xC4 == 0) {
                int si;
                for (si = 0; si < 6; si++) if (hp->u.map.symbol[si] == 0) hp->u.map.symbol[si] = mksym();
            }
            grShrineRoute_80208F70(g);
            state();
            if (k == 1) {
                Ground* gp = (Ground*) g->user_data;
                gp->u.shrineroute.xC4 = (u16) (rnd() % 7);
                gp->u.shrineroute.xCA = (u16) (rnd() % 2);
                gp->u.shrineroute.xCC = (u16) (rnd() % 2);
            }
        }
    }
    for (c = 0; c < 60; c++) { /* grShrineRoute_80209AF0 / 80209BEC: the chandelier */
        HSD_GObj* g;
        Ground* gp;
        greset();
        CASE(6000 + c);
        rseed(7000 + c);
        g = mkgobj();
        ((HSD_JObj*) g->hsd_obj)->flags &= ~JOBJ_USE_QUATERNION;
        ((HSD_JObj*) g->hsd_obj)->rotate.x = (rnd() % 2) ? 0.25f : (float) (rnd() % 600) / 1000.0f - 0.3f;
        ((HSD_JObj*) g->hsd_obj)->rotate.y = (rnd() % 2) ? 0.17f : (float) (rnd() % 400) / 1000.0f - 0.2f;
        grShrineRoute_80209AF0(g);
        gp = (Ground*) g->user_data;
        state();
        for (k = 0; k < 25; k++) {
            if (k % 7 == 3) {
                gp->u.shrineroute3.xC8 = (float) (rnd() % 200) / 100000.0f - 0.001f;
                gp->u.shrineroute3.xCC = (float) (rnd() % 200) / 100000.0f - 0.001f;
                ((HSD_JObj*) g->hsd_obj)->rotate.x = (rnd() % 2) ? 0.2615f : -0.2615f;
                ((HSD_JObj*) g->hsd_obj)->rotate.y = (rnd() % 2) ? 0.1744f : -0.1744f;
            }
            if (k % 5 == 2 && gp->u.shrineroute3.xC4) {
                gp->u.shrineroute3.xC4->rotate.x = (rnd() % 2) ? 6.28f : -6.28f;
                gp->u.shrineroute3.xC4->rotate.y = (float) (rnd() % 700) / 100.0f - 3.5f;
                gp->u.shrineroute3.xD8 = (float) (rnd() % 100) / 100.0f;
                gp->u.shrineroute3.xDC = -(float) (rnd() % 100) / 100.0f;
            }
            grShrineRoute_80209BEC(g);
            state();
        }
    }
    for (c = 0; c < 40; c++) { /* grShrineRoute_8020A8A4 (the chandelier respawn) */
        HSD_GObj* g;
        greset();
        CASE(7000 + c);
        rseed(8000 + c);
        g = mkgobj();
        ((Ground*) g->user_data)->u.shrineroute3.xC4 = (c % 4 == 0) ? 0 : mkjobj2();
        grShrineRoute_8020A8A4(g);
        state();
    }
    for (c = 0; c < 80; c++) { /* grShrineRoute_8020A104: the stage lights setup (+ 8020AA40/AB58/AC44/A21C inside) */
        HSD_GObj* g;
        greset();
        CASE(8000 + c);
        rseed(9000 + c);
        yakumono_param = mkyp();
        g = mkgobj();
        grShrineRoute_8020A104(g);
        state();
        for (k = 0; k < 3; k++) {
            grShrineRoute_8020A21C(g);
            state();
        }
    }
    for (c = 0; c < 200; c++) { /* grShrineRoute_8020A21C on randomised light lists */
        HSD_GObj* g;
        Ground* gp;
        int n, i;
        greset();
        CASE(9000 + c);
        rseed(10000 + c);
        g = mkgobj();
        gp = (Ground*) g->user_data;
        n = 3 + (int) (rnd() % 18);
        gp->u.shrineroute2.xC4 = (c % 11 == 0) ? 0 : mkfgobj();
        gp->u.shrineroute2.x168 = (u32) n;
        for (i = 0; i < n; i++) {
            gp->u.shrineroute2.xC8[i] = mklobj();
            gp->u.shrineroute2.x118[i] = rnd();
        }
        gp->u.shrineroute2.x16C = mklobj();
        gp->u.shrineroute2.x170 = mklobj();
        grShrineRoute_8020A21C(g);
        state();
    }
    for (c = 0; c < 40; c++) { /* grShrineRoute_8020AA40 / 8020AB58 / 8020AC44 */
        HSD_GObj* g;
        greset();
        CASE(10000 + c);
        rseed(11000 + c);
        g = mklights(16 + (int) (rnd() % 5));
        grShrineRoute_8020AA40(g);
        grShrineRoute_8020AA40(0);
        printf("r"); TRP(grShrineRoute_8020AB58(g)); printf("\n");
        printf("r"); TRP(grShrineRoute_8020AC44(g)); printf("\n");
        LDNULL = 1;
        g = (c % 5 == 0) ? 0 : ((c % 5 == 1) ? mkfgobj() : mklights(1 + (int) (rnd() % 4)));
        printf("r"); TRP(grShrineRoute_8020AB58(g)); printf("\n");
        printf("r"); TRP(grShrineRoute_8020AC44(g)); printf("\n");
        state();
    }
    for (c = 0; c < 80; c++) { /* grShrineRoute_8020AD24, onJointCollision, grShrineRoute_8020AE08 */
        HSD_GObj* g;
        Ground* gp;
        CollData* cd;
        int a1, a2;
        s32* out;
        greset();
        CASE(11000 + c);
        rseed(12000 + c);
        yakumono_param = mkyp();
        FPOS = c % 2;
        SCALE = (c % 2) ? 1.0f : 0.0f;
        a1 = (int) grShrineRoute_8020AD24(1);
        a2 = (int) grShrineRoute_8020AD24((int) (rnd() % 3));
        printf("r %d %d\n", a1, a2);
        g = mkgobj();
        gp = (Ground*) g->user_data;
        for (k = 0; k < 6; k++) {
            int v = (int) (rnd() % 3);
            int gk = (int) (rnd() % 4);
            cd = mkcoll(v);
            cd->x0_gobj = (rnd() % 3 == 0) ? mkfgobj() : PGOBJ;
            onJointCollision(gp, 8, cd, 0, (mpLib_GroundEnum) gk, 0.0f);
            printf("cc %d %d\n", (int) gp->u.shrineroute.xCA, (int) gp->u.shrineroute.xCC);
        }
        gp->u.shrineroute.xC4 = (u16) (rnd() % 5);
        out = (s32*) AZ(8);
        printf("r %d\n", (int) grShrineRoute_8020AE08(g, PGOBJ, out));
        printf("out %d\n", (int) *out);
        state();
    }
    for (c = 0; c < 60; c++) { /* grShrineRoute_8020AF38 / 8020B020 / 8020B0AC */
        HSD_GObj* g;
        int a1, a2;
        greset();
        CASE(12000 + c);
        rseed(13000 + c);
        yakumono_param = mkyp();
        g = mkhub(0);
        grShrineRoute_8020AF38(g, 0xBD + (int) (rnd() % 6));
        state();
        a1 = (c % 4 == 0) ? -1 : 0xBD + (int) (rnd() % 6);
        a2 = (int) (rnd() % 2);
        grShrineRoute_8020B020(g, a1, a2);
        state();
        grShrineRoute_8020B0AC();
    }
    for (c = 0; c < 40; c++) { /* grShrineRoute_OnCheckShadowRender, grShrineRoute_ScriptQuery */
        Vec3* v;
        HSD_JObj* j;
        int i, a1;
        greset();
        CASE(13000 + c);
        rseed(14000 + c);
        v = (Vec3*) AZ(sizeof(Vec3));
        v->x = rf(); v->y = rf(); v->z = rf();
        a1 = (int) rnd();
        j = mkjobj2();
        printf("r %d\n", (int) grShrineRoute_OnCheckShadowRender(v, a1, j));
        for (i = 0; i < 6; i++) printf("q %d\n", (int) grShrineRoute_ScriptQuery(i % 5, (int) (rnd() % 8) - 1));
        state();
    }
    return 0;
}
