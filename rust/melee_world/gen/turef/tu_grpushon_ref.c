/* State oracle for src/tu/grpushon.rs (see tu_grtcaptain_ref.c). The TU is compiled from
 * rust/target/cref/grpushon_tu.c: the game file with two literal edits (tu_shard585 config):
 * the GX link head spelled HSD_GObjGXLinkHead[4] (next_gx is at +0x10 in the 32-bit layout) and the
 * call arguments Ground_801C5840()/Ground_801C5940() sequenced left to right as PPC does. */
#include "grpushon_tu.c"
#include "oracle.h"
#include "tu_gr_common.h"

static float SCALE;
static int FMODE;
static int LDNULL;
static unsigned char GKIND[16];

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
static HSD_GObj* mkmapgobj(void)
{
    HSD_GObj* g = mkgobj();
    GKIND[NGP - 1] = 1;
    if (rnd() % 4 == 0) {
        g->user_data = 0;
    } else {
        ((Ground*) g->user_data)->u.map.xC4_b0 = rnd() & 1;
    }
    return g;
}
static HSD_GObj* mkstagegobj(int id)
{
    HSD_GObj* g = mkgobj();
    GKIND[NGP - 1] = (id == 2) ? 2 : 1;
    return g;
}
static struct grPushon_YakumonoParam* mkypp(void)
{
    struct grPushon_YakumonoParam* p =
        (struct grPushon_YakumonoParam*) AZ(sizeof(struct grPushon_YakumonoParam));
    int i;
    p->x0 = (int) (rnd() % 100);
    p->x4 = (DynamicsDesc*) AZ(16);
    p->x8 = (DynamicsDesc*) AZ(16);
    p->xC = (DynamicsDesc*) AZ(16);
    p->x10 = (DynamicsDesc*) AZ(16);
    p->x14 = (DynamicsDesc*) AZ(16);
    p->x18 = rnd() & 1;
    for (i = 0; i < 0x1E; i++) {
        p->x1c[i].x0 = (s32) rnd();
        p->x1c[i].x4 = (s16) rnd();
        p->x1c[i].x6 = (s16) rnd();
    }
    for (i = 0; i < 0x21; i++) {
        if (i < 8) {
            p->x10c[i].key = i;
        } else if (i == 8) {
            p->x10c[i].key = -1;
        } else {
            p->x10c[i].key = (s32) (rnd() % 20);
        }
        p->x10c[i].value = (s32) (rnd() % 1000);
    }
    return p;
}
#include "tu_grpushon_stubs.inc"

static HSD_GObj* mklightgobj(int n)
{
    HSD_GObj* g = (HSD_GObj*) AZ(sizeof(HSD_GObj));
    HSD_LObj* prev = 0;
    int i;
    for (i = 0; i < n; i++) {
        HSD_LObj* l = mklobj();
        if (i > 0) l->flags = (u16) ((l->flags & ~3) | 2);
        if (prev) {
            prev->next = l;
        } else {
            g->hsd_obj = l;
        }
        prev = l;
    }
    g->classifier = 12;
    g->obj_kind = (u8) HSD_GObj_LightKind;
    return g;
}

static void mkgx(int c)
{
    HSD_GObj** gx;
    int nd, d;
    HSD_GObj* head = 0;
    HSD_GObj* tail = 0;
    HSD_GObj_LightKind = (s8) (rnd() % 8);
    gx = (HSD_GObj**) AZ(5 * sizeof(HSD_GObj*));
    HSD_GObjGXLinkHead = gx;
    nd = (int) (rnd() % 3);
    for (d = 0; d <= nd; d++) {
        HSD_GObj* l = mklightgobj(3 + (int) (rnd() % 6));
        if (d < nd || c % 5 == 0) {
            /* a decoy: wrong classifier, wrong kind, or no model */
            int w = (int) (rnd() % 3);
            if (w == 0) l->classifier = 7;
            if (w == 1) l->obj_kind = (u8) (HSD_GObj_LightKind + 1);
            if (w == 2) { l->classifier = 12; l->obj_kind = (u8) HSD_GObj_LightKind; }
        }
        if (tail) {
            tail->next_gx = l;
        } else {
            head = l;
        }
        tail = l;
    }
    gx[4] = head;
}

static void greset(void)
{
    gr_reset();
    memset(GKIND, 0, sizeof(GKIND));
    SCALE = 0.0f;
    FMODE = 0;
    LDNULL = 0;
}

static void state(void)
{
    int i;
    printf("rs %u\n", ORS);
    printf("si %d %d\n", (int) stage_info.unk8C.b4, (int) stage_info.unk8C.b5);
    DUMP("yp", yakumono_param);
    DUMP("gx", HSD_GObjGXLinkHead);
    gnd_canon_begin();
    for (i = 0; i < NGP; i++) {
        if (GKIND[i] != 1) continue;
        printf("m%d %d\n", i, (int) GPS[i]->u.map.xC4_b0);
        canon_add(&GPS[i]->u, 4);
    }
    DUMPARENA();
    canon_restore();
    gnd_canon_end();
    state_end();
}

#define FN(x) fnreg(#x, (void*) x)
int main(void)
{
    int c, k;
    arena_init();
    SIZE(Ground);
    SIZE(HSD_GObj);
    SIZE(HSD_LObj);
    SIZE(CollData);
    SIZE(StageCallbacks);
    SIZE(StageData);
    SIZE(struct grPushon_YakumonoParam);
    SIZE(struct grPushOn_LightConfig);
    reg("cb", grPushOn_StageCallbacks, sizeof(grPushOn_StageCallbacks));
    reg("sd", &grPushOn_StageData, sizeof(grPushOn_StageData));
    reg("dat", grPushOn_StageData.data1, 16);
    reg("ld1", &grPushOn_803E7B74, sizeof(grPushOn_803E7B74));
    reg("ld2", &grPushOn_803E7B90, sizeof(grPushOn_803E7B90));
    reg("f34", &grPushOn_804D4934, 4);
    reg("f48", &grPushOn_804D4948, 4);
    FN(grPushOn_802182C4); FN(grPushOn_802182C8); FN(grPushOn_80218330); FN(grPushOn_80218378);
    FN(grPushOn_802183DC); FN(grPushOn_802183E4); FN(grPushOn_802184CC); FN(grPushOn_80218590);
    FN(grPushOn_80218598); FN(grPushOn_8021859C); FN(grPushOn_802185A0); FN(grPushOn_80218670);
    FN(fn_80218678); FN(grPushOn_802186C8); FN(grPushOn_802187A4); FN(grPushOn_802187A8);
    FN(grPushOn_80218880); FN(grPushOn_80218888); FN(grPushOn_80218ED0); FN(grPushOn_80218ED4);
    FN(grPushOn_80218FC0); FN(fn_802190A0); FN(grPushOn_802190D0); FN(grPushOn_80219204);
    FN(grPushOn_80219230); FN(fn_802192A4); FN(grPushOn_80219458); FN(grPushOn_80219528);
    fnreg("grDisplay_801C5DB0", (void*) grDisplay_801C5DB0);
    fnreg("HSD_AObjSetFlags", (void*) HSD_AObjSetFlags);
    DUMP("cb", grPushOn_StageCallbacks);
    DUMP("sd", grPushOn_StageData);
    DUMP("ld1", grPushOn_803E7B74);
    DUMP("ld2", grPushOn_803E7B90);
    DUMP("cfg", light_configs);
    DUMP("tbl", grPushOn_803E7CCC);
    DUMP("v40", grPushOn_803B8440);
    DUMP("v4c", grPushOn_803B844C);
    DUMP("v58", grPushOn_803B8458);
    DUMP("v64", grPushOn_803B8464);
    DUMP("f34", grPushOn_804D4934);
    DUMP("f48", grPushOn_804D4948);
    state_end();

    for (c = 0; c < 60; c++) { /* grPushOn_802183E4 and the callbacks it installs */
        greset();
        CASE(c);
        rseed(100 + c);
        yakumono_param = mkypp();
        stage_info.unk8C.b4 = rnd() & 1;
        stage_info.unk8C.b5 = rnd() & 1;
        mkgx(c);
        printf("r"); TRP(grPushOn_802183E4(c % 3)); printf("\n");
        state();
    }
    for (c = 0; c < 40; c++) { /* OnInit / OnLoad / OnStart / OnDemoInit */
        greset();
        CASE(1000 + c);
        rseed(2000 + c);
        mkgx(c);
        yakumono_param = (c % 2) ? mkypp() : 0;
        grPushOn_802182C8();
        state();
        grPushOn_80218330();
        state();
        yakumono_param = mkypp();
        grPushOn_80218378();
        state();
        grPushOn_802182C4((int) rnd());
        printf("r %d\n", (int) grPushOn_802183DC());
        state();
    }
    for (c = 0; c < 40; c++) { /* the plain callbacks of the three gobjs */
        HSD_GObj* g;
        greset();
        CASE(2000 + c);
        rseed(3000 + c);
        g = mkgobj(); GKIND[NGP - 1] = 1;
        grPushOn_802184CC(g); state();
        printf("r %d\n", (int) grPushOn_80218590(g));
        grPushOn_80218598(g); grPushOn_8021859C(g);
        grPushOn_802185A0(g); state();
        printf("r %d\n", (int) grPushOn_80218670(g));
        grPushOn_802187A4(g);
        printf("r %d\n", (int) grPushOn_80218880(g));
        grPushOn_80218ED0(g);
        {
            Vec3 v;
            int a1;
            HSD_JObj* j;
            v.x = rf(); v.y = rf(); v.z = rf();
            a1 = (int) rnd();
            j = mkjobj();
            printf("r %d\n", (int) grPushOn_80219528(&v, a1, j));
        }
        state();
    }
    for (c = 0; c < 60; c++) { /* fn_80218678 (map xC4_b0 query) and grPushOn_802186C8 */
        HSD_GObj* g;
        greset();
        CASE(3000 + c);
        rseed(4000 + c);
        g = mkgobj(); GKIND[NGP - 1] = 1;
        ((Ground*) g->user_data)->u.map.xC4_b0 = rnd() & 1;
        printf("r %d\n", (int) fn_80218678());
        grPushOn_802186C8(g);
        state();
        printf("r %d\n", (int) fn_80218678());
    }
    for (c = 0; c < 40; c++) { /* fn_802190A0: the contact callback */
        HSD_GObj* g;
        CollData* coll;
        int jt, a1, a2;
        float df;
        greset();
        CASE(4000 + c);
        rseed(5000 + c);
        g = mkgobj(); GKIND[NGP - 1] = 1;
        coll = (CollData*) AZ(sizeof(CollData));
        *(unsigned char*) &coll->x34_flags = (unsigned char) rnd();
        if (c % 2) *(unsigned char*) &coll->x34_flags = (unsigned char) ((*(unsigned char*) &coll->x34_flags & ~0x78) | 0x08);
        jt = (int) (rnd() % 5);
        a1 = (int) rnd();
        a2 = (int) rnd();
        df = rf();
        fn_802190A0(g->user_data, a1, coll, a2, (mpLib_GroundEnum) jt, df);
        printf("raw %d\n", (int) (*(unsigned char*) &coll->x34_flags));
        state();
    }
    for (c = 0; c < 50; c++) { /* grPushOn_802190D0: the stage lights */
        HSD_GObj* g;
        greset();
        CASE(5000 + c);
        rseed(6000 + c);
        g = mklightgobj(1 + (int) (rnd() % 12));
        SCALE = (c % 3 == 0) ? 1.0f : 0.0f;
        grPushOn_802190D0(g);
        state();
    }
    for (c = 0; c < 80; c++) { /* grPushOn_80218ED4 / grPushOn_80218FC0: append a light */
        HSD_GObj* g;
        greset();
        CASE(6000 + c);
        rseed(7000 + c);
        LDNULL = (c % 4 == 1);
        if (c % 10 == 0) {
            g = 0;
        } else if (c % 10 == 1) {
            g = mklightgobj(0);
        } else {
            g = mklightgobj(1 + (int) (rnd() % 4));
        }
        printf("r"); TRP(grPushOn_80218ED4(g)); printf("\n");
        printf("r"); TRP(grPushOn_80218FC0(g)); printf("\n");
        state();
    }
    for (c = 0; c < 120; c++) { /* grPushOn_802187A8: the light setup, grPushOn_80218888 inside */
        HSD_GObj* g;
        greset();
        CASE(7000 + c);
        rseed(8000 + c);
        g = mkgobj(); GKIND[NGP - 1] = 1;
        GKIND[NGP - 1] = 2;
        mkgx(c);
        grPushOn_802187A8(g);
        state();
    }
    for (c = 0; c < 200; c++) { /* grPushOn_80218888: choose the lights nearest the fighter */
        HSD_GObj* g;
        Ground* gp;
        int n, i;
        greset();
        CASE(8000 + c);
        rseed(9000 + c);
        g = mkgobj(); GKIND[NGP - 1] = 1;
        gp = (Ground*) g->user_data;
        GKIND[NGP - 1] = 2;
        n = 1 + (int) (rnd() % 20);
        gp->u.pushon.gobj = (c % 9 == 0) ? 0 : (void*) mkfgobj();
        gp->u.pushon.count = n;
        for (i = 0; i < n; i++) {
            gp->u.pushon.lobjs[i] = mklobj();
            gp->u.pushon.lobj_flags[i] = rnd();
        }
        gp->u.pushon.point_light = mklobj();
        gp->u.pushon.spot_light = mklobj();
        grPushOn_80218888(g);
        state();
    }
    for (c = 0; c < 20; c++) { /* grPushOn_80219204 */
        int o1, o2;
        greset();
        CASE(9000 + c);
        rseed(10000 + c);
        yakumono_param = mkypp();
        grPushOn_80219204(0x99 + (int) (rnd() % 30), &o1, &o2);
        printf("r %d %d\n", o1, o2);
        state();
    }
    for (c = 0; c < 60; c++) { /* grPushOn_80219230 (character kind lookup) */
        greset();
        CASE(10000 + c);
        rseed(11000 + c);
        yakumono_param = mkypp();
        printf("r %d\n", grPushOn_80219230((int) (rnd() % 8)));
        state();
    }
    for (c = 0; c < 200; c++) { /* fn_802192A4 (is the fighter inside a rectangle) */
        HSD_GObj* g;
        s32* res;
        greset();
        CASE(11000 + c);
        rseed(12000 + c);
        yakumono_param = mkypp();
        FMODE = (c % 2);
        SCALE = (c % 4 < 2) ? 1.0f : 0.0f;
        g = mkfgobj();
        res = (s32*) AZ(8);
        printf("r %d\n", (int) fn_802192A4(0, g, res));
        printf("res %d\n", (int) *res);
        state();
    }
    for (c = 0; c < 100; c++) { /* grPushOn_80219458 */
        greset();
        CASE(12000 + c);
        rseed(13000 + c);
        yakumono_param = mkypp();
        printf("r"); TRP(grPushOn_80219458(c % 7 == 0 ? -1 : (int) rnd())); printf("\n");
        state();
    }
    return 0;
}
