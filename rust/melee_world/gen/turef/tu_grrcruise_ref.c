/* State oracle for src/tu/grrcruise.rs (see tu_grtcaptain_ref.c / tu_grpushon_ref.c). The TU is
 * compiled from rust/target/cref/grrcruise_tu.c: the game file with the literal edits of the
 * tu_shard585 config (native-width allocations, the console overlays written under both names). */
#include "grrcruise_tu.c"
#include "oracle.h"
#include "tu_gr_common.h"
/* the game links lbtrigf.c's atan2f (not the CRT's) */
float MSL_TrigF_80400770[1] = { __builtin_nanf("") };
float MSL_TrigF_80400774[1] = { __builtin_inff() };
#include <melee/lb/lbtrigf.c>

static int GNULL;
static int PRSUB; /* print/canonicalise the sub-entry bitfield bytes (only meaningful for that overlay) */
static int JNULL;
static HSD_GObj* G3;

static float rnz(void)
{
    float m = (float) (1 + rnd() % 2000) / 16.0f;
    return (rnd() & 1) ? m : -m;
}
static HSD_JObj* mkjtree(int d)
{
    HSD_JObj* j = mkjobj();
    j->flags &= ~JOBJ_USE_QUATERNION; /* HSD_JObjSetRotationZ asserts it clear */
    if (d < 2) {
        if (rnd() % 3 == 0) j->child = mkjtree(d + 1);
        if (rnd() % 3 == 0) j->next = mkjtree(d + 1);
    }
    return j;
}
static HSD_DObj* mkdobjs(void)
{
    int n = (int) (rnd() % 4);
    HSD_DObj* h = 0;
    int i;
    for (i = 0; i < n; i++) {
        HSD_DObj* d = (HSD_DObj*) AZ(sizeof(HSD_DObj));
        d->next = h;
        h = d;
    }
    return h;
}
static float rsmall(int n, int d) /* (rnd() % n) / d */
{
    return (float) (rnd() % (u32) n) / (float) d;
}
static struct grRCruise_YakumonoParam* mkyp(void)
{
    struct grRCruise_YakumonoParam* p =
        (struct grRCruise_YakumonoParam*) AZ(sizeof(struct grRCruise_YakumonoParam));
    p->x0 = rsmall(100, 100) + 0.01f;
    p->x4 = rsmall(300, 100) + 0.5f;
    p->x8 = rsmall(100, 40);
    p->xC = (s32) (rnd() % 12);
    p->x10 = (s32) (rnd() % 14);
    p->x14 = (s32) (1 + rnd() % 5);
    p->x18 = (s32) (rnd() % 14);
    p->x1C = (s32) (1 + rnd() % 5);
    p->x20 = (s32) (rnd() % 8);
    p->x24 = (s32) (rnd() % 10);
    p->x28 = (s32) (1 + rnd() % 4);
    p->x2C = rsmall(100, 10);
    p->x30 = p->x2C + (rnd() % 4 == 0 ? 0.0f : rsmall(50, 10));
    p->x34 = rsmall(30, 10);
    p->x38 = rsmall(60, 10);
    p->x3C = (s32) (rnd() % 8);
    p->x40 = (s32) (rnd() % 10);
    p->x44 = (s32) (1 + rnd() % 4);
    return p;
}
#include "tu_grrcruise_stubs.inc"

static struct grRCruise_Entry* ENT[4];
static unsigned char* CB[64]; /* the x34_flags bytes of the CollData made since greset(): hashed canonically */
static int NCB;
static int NENT;
static void greset(void)
{
    gr_reset();
    NENT = 0;
    NCB = 0;
    PRSUB = 0;
    GNULL = 0;
    JNULL = 0;
    G3 = 0;
}

static void state(void)
{
    int i, k;
    printf("rs %u\n", ORS);
    printf("si %d %d\n", (int) stage_info.unk8C.b4, (int) stage_info.unk8C.b5);
    DUMP("yp", yakumono_param);
    gnd_canon_begin();
    for (i = 0; i < NGP; i++) {
        Ground* g = GPS[i];
        printf("u%d %d %d\n", i, (int) g->u.rcruise.xC4.b0, (int) g->u.scroll.x0);
        canon_add(&g->u, 1);
        for (k = 0; PRSUB && k < 3; k++) {
            printf("s%d.%d %d\n", i, k, (int) g->u.rcruise.x3C[k].x1_b0);
            canon_add((unsigned char*) &g->u.rcruise.x3C[k] + 1, 1);
        }
    }
    for (i = 0; i < NENT; i++) {
        for (k = 0; k < 17; k++) {
            printf("e%d.%d %d\n", i, k, (int) (ENT[i][k].pad_01 & 1));
            canon_add(&ENT[i][k].pad_01, 1);
        }
    }
    for (i = 0; i < NCB; i++) canon_add(CB[i], 1);
    DUMPARENA();
    canon_restore();
    gnd_canon_end();
    state_end();
}

/* a rcruise main-stage ground: entries (17), vanish table (20), the hub camera structure */
static HSD_GObj* mkmain(int randomize)
{
    HSD_GObj* g = mkgobj();
    Ground* gp = (Ground*) g->user_data;
    int i;
    gp->u.rcruise.entries = (struct grRCruise_Entry*) AZ(17 * sizeof(struct grRCruise_Entry));
    gp->u.map.vanish = (struct Map_VanishEntry*) AZ(20 * sizeof(struct Map_VanishEntry));
    gp->u.rcruise.x4 = (struct lb_80011A50_t*) AZ(sizeof(struct lb_80011A50_t));
    ENT[NENT++] = gp->u.rcruise.entries;
    gp->u.rcruise.x4->x4.x = rf();
    gp->u.rcruise.x4->x4.z = rf();
    gp->u.rcruise.x4->unk_angle_int = (int) (rnd() % 600);
    gp->u.rcruise.x4->unk_scale = rf();
    gp->u.rcruise.x8 = rsmall(100, 1000) - 0.05f;
    gp->u.rcruise.xC = (s32) (rnd() % 2);
    if (!randomize) return g;
    gp->u.rcruise.x10 = (int) (rnd() % 2);
    gp->u.rcruise.x14 = rf();
    gp->u.rcruise.x18 = rsmall(40000, 20) - 1000.0f;
    gp->u.rcruise.x1C = (rnd() % 2) ? 1.0f : -1.0f;
    gp->u.rcruise.x20 = rsmall(200, 100) - 1.0f;
    gp->u.rcruise.x24 = rsmall(100, 7);
    gp->u.rcruise.x28 = rsmall(100, 7);
    gp->u.rcruise.x2C = (s32) (rnd() % 3);
    gp->u.rcruise.x30 = (s32) (rnd() % 5);
    gp->u.rcruise.x34 = (s32) (rnd() % 3);
    gp->u.rcruise.x38 = (s32) (rnd() % 4);
    for (i = 0; i < 17; i++) {
        struct grRCruise_Entry* e = &gp->u.rcruise.entries[i];
        e->x0 = (u8) (rnd() % 7);
        e->pad_01 = (u8) (rnd() & 1);
        e->x2 = (s16) (rnd() % 50);
        e->x4 = (s32) (rnd() % 12);
        e->x8 = (s32) (rnd() % 3);
        e->xC = rf();
        e->x10 = rsmall(60, 10) - 3.0f;
        e->x14 = rnd() % 8 == 0 ? 0 : mkjobj();
        if (e->x14 == 0 && 0) e->x14 = mkjobj();
    }
    for (i = 0; i < 20; i++) {
        struct Map_VanishEntry* v = &gp->u.map.vanish[i];
        v->x0 = (s16) (rnd() % 5);
        v->x2 = (s16) (rnd() % 30);
        v->jobj = mkjobj();
    }
    for (i = 0; i < 3; i++) {
        struct grRCruise_SubEntry* s = &gp->u.rcruise.x3C[i];
        s->x0 = (u8) (rnd() % 7);
        s->x1_b0 = rnd() & 1;
        s->x2 = (s16) (rnd() % 50);
        s->x4 = (s32) (rnd() % 12);
        s->x8 = (s32) (rnd() % 3);
        s->xC = mkjtree(1);
    }
    return g;
}
/* entries that must dereference x14: make them all valid for the state machines */
static void fixent(Ground* gp)
{
    int i;
    for (i = 0; i < 17; i++) {
        if (gp->u.rcruise.entries[i].x14 == 0) gp->u.rcruise.entries[i].x14 = mkjobj();
    }
}
static CollData* mkcoll(int v)
{
    CollData* c = (CollData*) AZ(sizeof(CollData));
    *(unsigned char*) &c->x34_flags = (unsigned char) (v << 1);
    printf("cf %d\n", (int) c->x34_flags.b1234);
    CB[NCB++] = (unsigned char*) &c->x34_flags;
    c->cur_pos.x = rf();
    c->cur_pos.y = rf();
    c->cur_pos.z = rf();
    return c;
}

#define FN(x) fnreg(#x, (void*) x)
int main(void)
{
    int c, k;
    arena_init();
    SIZE(Ground);
    SIZE(HSD_GObj);
    SIZE(HSD_JObj);
    SIZE(HSD_DObj);
    SIZE(CollData);
    SIZE(StageCallbacks);
    SIZE(StageData);
    SIZE(GrJoint);
    SIZE(struct grRCruise_YakumonoParam);
    SIZE(struct grRCruise_Entry);
    SIZE(struct grRCruise_SubEntry);
    SIZE(struct Map_VanishEntry);
    SIZE(struct Map_VanishDesc);
    SIZE(struct lb_80011A50_t);
    reg("cb", grRc_StageCallbacks, sizeof(grRc_StageCallbacks));
    reg("sd", &grRc_StageData, sizeof(grRc_StageData));
    reg("dat", grRc_StageData.data1, 16);
    reg("jt", grRc_803E4DA8, sizeof(grRc_803E4DA8));
    FN(grRCruise_801FF164); FN(grRCruise_801FF168); FN(grRCruise_801FF298); FN(grRCruise_801FF29C);
    FN(grRCruise_801FF2C0); FN(grRCruise_801FF2C8); FN(grRCruise_801FF3B4); FN(grRCruise_801FF3E0);
    FN(grRCruise_801FF3E8); FN(grRCruise_801FF3EC); FN(grRCruise_801FF3F0); FN(grRCruise_801FF434);
    FN(grRCruise_801FF43C); FN(grRCruise_801FF440); FN(grRCruise_801FF444); FN(grRCruise_801FF5B4);
    FN(grRCruise_801FF6CC); FN(grRCruise_801FF6D4); FN(grRCruise_801FF738); FN(grRCruise_801FF73C);
    FN(grRCruise_801FF794); FN(grRCruise_801FF79C); FN(grRCruise_801FF7A0); FN(grRCruise_801FF7A4);
    FN(grRCruise_801FF8DC); FN(grRCruise_801FF8E4); FN(grRCruise_801FF920); FN(grRCruise_801FF924);
    FN(grRCruise_801FFAD4); FN(grRCruise_801FFADC); FN(grRCruise_80200070); FN(grRCruise_80200074);
    FN(grRCruise_8020014C); FN(grRCruise_80200154); FN(grRCruise_8020045C); FN(fn_80200460);
    FN(grRCruise_80200540); FN(grRCruise_80200578); FN(grRCruise_8020071C); FN(grRCruise_80200B48);
    FN(grRCruise_80200C04); FN(grRCruise_802010A4); FN(grRCruise_80201110); FN(grRCruise_80201288);
    FN(grRCruise_80201410); FN(grRCruise_80201588); FN(grRCruise_80201918); FN(grRCruise_80201988);
    FN(grRCruise_80201B60); FN(fn_80201BE0); FN(grRCruise_80201C50); FN(grRCruise_80201C58);
    fnreg("grDisplay_801C5DB0", (void*) grDisplay_801C5DB0);
    fnreg("HSD_AObjSetFlags", (void*) HSD_AObjSetFlags);
    fnreg("HSD_DObjSetFlags", (void*) HSD_DObjSetFlags);
    fnreg("HSD_DObjClearFlags", (void*) HSD_DObjClearFlags);
    DUMP("cb", grRc_StageCallbacks);
    DUMP("sd", grRc_StageData);
    DUMP("jt", grRc_803E4DA8);
    DUMP("sl", grRc_804D4790);
    DUMP("bl", grRc_803E4FF0);
    DUMP("vd", lbl_803E5014);
    state_end();

    for (c = 0; c < 28; c++) { /* grRCruise_801FF2C8: the stage gobj setup */
        greset();
        CASE(c);
        rseed(100 + c);
        GNULL = c % 2;
        printf("r"); TRP(grRCruise_801FF2C8(c % 7)); printf("\n");
        state();
    }
    for (c = 0; c < 20; c++) { /* OnInit, and the trivial callbacks */
        greset();
        CASE(1000 + c);
        rseed(2000 + c);
        yakumono_param = 0;
        grRCruise_801FF168();
        state();
        grRCruise_801FF164((int) rnd());
        grRCruise_801FF298();
        grRCruise_801FF29C();
        printf("r %d\n", (int) grRCruise_801FF2C0());
        state();
    }
    for (c = 0; c < 30; c++) { /* the plain gobj callbacks */
        HSD_GObj* g;
        greset();
        CASE(2000 + c);
        rseed(3000 + c);
        JNULL = c % 2;
        g = mkgobj();
        grRCruise_801FF3B4(g);
        grRCruise_801FF3F0(g);
        state();
        grRCruise_801FF73C(g);
        grRCruise_801FF8E4(g);
        grRCruise_801FF444(g);
        state();
        printf("r %d %d %d %d\n", (int) grRCruise_801FF3E0(g), (int) grRCruise_801FF434(g),
               (int) grRCruise_801FF6CC(g), (int) grRCruise_801FF794(g));
        grRCruise_801FF3E8(g); grRCruise_801FF3EC(g); grRCruise_801FF43C(g); grRCruise_801FF440(g);
        grRCruise_801FF738(g); grRCruise_801FF79C(g); grRCruise_801FF7A0(g); grRCruise_801FF920(g);
        grRCruise_80200070(g); grRCruise_8020045C(g);
        printf("r %d %d %d\n", (int) grRCruise_801FF8DC(g), (int) grRCruise_801FFAD4(g), (int) grRCruise_8020014C(g));
        printf("r %d %d\n", (int) grRCruise_80201C58(0, 0, 0), (int) (grRCruise_80201C50(0) == 0));
        state();
    }
    for (c = 0; c < 40; c++) { /* grRCruise_801FF5B4: the main platform's setup (+ its sub-setups) */
        HSD_GObj* g;
        greset();
        CASE(3000 + c);
        rseed(4000 + c);
        g = mkgobj();
        JNULL = 0;
        yakumono_param = mkyp();
        grRCruise_801FF5B4(g);
        ENT[NENT++] = ((Ground*) g->user_data)->u.rcruise.entries;
        state();
        /* the following frames */
        for (k = 0; k < 6; k++) {
            Ground* gp = (Ground*) g->user_data;
            fixent(gp);
            gp->u.rcruise.x10 = (int) (c % 3 == 0);
            grRCruise_801FF6D4(g);
            state();
        }
    }
    for (c = 0; c < 120; c++) { /* the state machines on randomised states */
        HSD_GObj* g;
        Ground* gp;
        greset();
        CASE(4000 + c);
        PRSUB = 1;
        rseed(5000 + c);
        yakumono_param = mkyp();
        g = mkmain(1);
        gp = (Ground*) g->user_data;
        fixent(gp);
        for (k = 0; k < 8; k++) {
            int e;
            if (c % 4 == 0) grRCruise_80201588(g);
            if (c % 4 == 1) grRCruise_8020071C(g);
            if (c % 4 == 2) grRCruise_80200C04(g);
            if (c % 4 == 3) { grRCruise_80201588(g); grRCruise_8020071C(g); grRCruise_80200C04(g); }
            for (e = 0; e < 3; e++) {
                int a1, a3, jid;
                float df;
                CollData* cd = mkcoll((int) (rnd() % 4));
                a1 = (int) (rnd() % 50);
                a3 = (int) (rnd() % 3000);
                df = rf();
                grRCruise_80200578(g, a1, cd, a3, (mpLib_GroundEnum) 0, df);
                cd = mkcoll((int) (rnd() % 3));
                jid = (int) gp->u.rcruise.entries[rnd() % 17].x2;
                df = rf();
                grRCruise_802010A4(g, jid, cd, 0, (mpLib_GroundEnum) 0, df);
            }
            state();
        }
    }
    for (c = 0; c < 40; c++) { /* grRCruise_80200540 / 80200B48 / 80201410 and their asserts' happy path */
        HSD_GObj* g;
        Ground* gp;
        greset();
        CASE(5000 + c);
        PRSUB = 1;
        rseed(6000 + c);
        g = mkmain(1);
        gp = (Ground*) g->user_data;
        grRCruise_80200540(g);
        state();
        grRCruise_80200B48(g);
        state();
        grRCruise_80201410(g);
        state();
        grRCruise_801FF6D4(g);
        state();
    }
    for (c = 0; c < 80; c++) { /* grRCruise_80200074 / 80200154 / fn_80200460: the three pop-up blocks */
        HSD_GObj* g;
        Ground* gp;
        greset();
        CASE(6000 + c);
        PRSUB = 1;
        rseed(7000 + c);
        yakumono_param = mkyp();
        g = mkgobj();
        gp = (Ground*) g->user_data;
        grRCruise_80200074(g);
        state();
        for (k = 0; k < 10; k++) {
            int i, e;
            if (k % 3 == 1) {
                for (i = 0; i < 3; i++) {
                    gp->u.rcruise.x3C[i].x0 = (u8) (rnd() % 7);
                    gp->u.rcruise.x3C[i].x1_b0 = rnd() & 1;
                    gp->u.rcruise.x3C[i].x4 = (s32) (rnd() % 14);
                    gp->u.rcruise.x3C[i].x8 = (s32) (rnd() % 2);
                }
            }
            for (e = 0; e < 2; e++) {
                int jid;
                float df;
                CollData* cd = mkcoll((int) (rnd() % 3));
                jid = (int) gp->u.rcruise.x3C[rnd() % 4 % 3].x2;
                df = rf();
                fn_80200460(g, jid, cd, 0, (mpLib_GroundEnum) 0, df);
            }
            grRCruise_80200154(g);
            state();
        }
    }
    for (c = 0; c < 20; c++) { /* grRCruise_801FF7A4 (the ship) */
        HSD_GObj* g;
        greset();
        CASE(7000 + c);
        rseed(8000 + c);
        JNULL = c % 2;
        g = mkgobj();
        grRCruise_801FF7A4(g);
        state();
    }
    for (c = 0; c < 20; c++) { /* grRCruise_801FF924 (the scroller) and its render callback */
        HSD_GObj* g;
        greset();
        CASE(8000 + c);
        rseed(9000 + c);
        g = mkgobj();
        grRCruise_801FF924(g);
        state();
        fn_80201BE0(g, (int) (rnd() % 4));
        fn_80201BE0(g, (int) (rnd() % 4));
        state();
    }
    for (c = 0; c < 80; c++) { /* grRCruise_801FFADC (the scroll follow) and 80201918 */
        HSD_GObj* g;
        HSD_GObj* ag;
        Ground* gp;
        Vec3* v;
        greset();
        CASE(9000 + c);
        rseed(10000 + c);
        g = mkgobj();
        gp = (Ground*) g->user_data;
        ag = mkgobj();
        gp->u.scroll.anim_gobj = ag;
        ((Ground*) ag->user_data)->u.scroll.int_jobj = mkjobj();
        ((Ground*) ag->user_data)->u.scroll.cam_jobj = mkjobj();
        gp->u.scroll.x0 = c % 4 == 0;
        gp->u.scroll.x10.x = rf(); gp->u.scroll.x10.y = rf(); gp->u.scroll.x10.z = rf();
        gp->u.scroll.x1C.x = rf(); gp->u.scroll.x1C.y = rf(); gp->u.scroll.x1C.z = rf();
        if (c % 9 == 1) g->hsd_obj = 0;
        grRCruise_801FFADC(g);
        state();
        v = (Vec3*) AZ(sizeof(Vec3));
        v->x = rf();
        grRCruise_80201918(v);
        printf("v"); TRF(v->x); TRF(v->y); TRF(v->z); printf("\n");
        state();
    }
    for (c = 0; c < 60; c++) { /* grRCruise_80201110 (the cruise light) */
        HSD_GObj* g;
        greset();
        CASE(10000 + c);
        rseed(11000 + c);
        g = mkmain(0);
        {
            int ai = (int) (30 * (rnd() % 20));
            if (c % 3 != 0) ai += (int) (rnd() % 2);
            ((Ground*) g->user_data)->u.rcruise.x4->unk_angle_int = ai;
        }
        ((Ground*) g->user_data)->u.rcruise.x4->x4.x = (c % 5 == 0) ? 0.1f : rf();
        ((Ground*) g->user_data)->u.rcruise.x4->x4.z = (c % 7 == 0) ? 5.0f : rf();
        ((Ground*) g->user_data)->u.rcruise.x8 = (c % 4 == 0) ? 5.0f : (rsmall(100, 100) - 0.5f);
        for (k = 0; k < 4; k++) {
            grRCruise_80201110(g);
            printf("l"); TRF(((Ground*) g->user_data)->u.rcruise.x4->x4.x); TRF(((Ground*) g->user_data)->u.rcruise.x4->x4.z);
            TRF(((Ground*) g->user_data)->u.rcruise.x4->unk_scale); TRF(((Ground*) g->user_data)->u.rcruise.x8);
            printf("\n");
        }
        state();
    }
    for (c = 0; c < 60; c++) { /* grRCruise_80201988 (line -> joint) */
        greset();
        CASE(11000 + c);
        rseed(12000 + c);
        stage_info.grkind = (c % 3 == 0) ? 7 : Gr_Kind_RCruise;
        printf("r %d\n", (int) grRCruise_80201988(c % 11 == 0 ? -1 : (int) rnd()));
        state();
    }
    for (c = 0; c < 40; c++) { /* grRCruise_80201B60 / 80201288 */
        HSD_JObj* j;
        greset();
        CASE(12000 + c);
        rseed(13000 + c);
        j = mkjtree(0);
        grRCruise_80201B60(j, (int) (rnd() % 2));
        grRCruise_80201B60(0, (int) (rnd() % 2));
        grRCruise_80201288(j, HSD_DObjSetFlags, (u32) rnd());
        grRCruise_80201288(j, HSD_DObjClearFlags, (u32) rnd());
        grRCruise_80201288(0, HSD_DObjClearFlags, (u32) rnd());
        state();
    }
    return 0;
}
