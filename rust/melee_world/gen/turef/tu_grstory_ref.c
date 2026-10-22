/* State oracle for src/tu/grstory.rs (see tu_grtcaptain_ref.c / gen_target_stage.py). */
#include "melee/gr/grstory.c"
#include "oracle.h"
#include "tu_gr_common.h"

static struct grStory_YakumonoParam* mkyp(void)
{
    struct grStory_YakumonoParam* p = (struct grStory_YakumonoParam*) AZ(sizeof(struct grStory_YakumonoParam));
    int i;
    p->timer_min = rf();
    p->timer_rand = (float) (rnd() % 20);
    p->spawnmany_rarity = (float) (rnd() % 5);
    for (i = 0; i < 6; i++) p->vpos[i] = rf();
    return p;
}
#include "tu_grstory_stubs.inc"

static void state(void)
{
    printf("rs %u\n", ORS);
    printf("si %d %d\n", (int) stage_info.unk8C.b4, (int) stage_info.unk8C.b5);
    DUMP("yp", yakumono_param);
    gnd_canon_begin();
    DUMPARENA();
    gnd_canon_end();
    state_end();
}

#define FN(x) fnreg(#x, (void*) x)
int main(void)
{
    int c;
    arena_init();
    SIZE(Ground);
    SIZE(HSD_GObj);
    SIZE(StageCallbacks);
    SIZE(StageData);
    SIZE(struct grStory_YakumonoParam);
    reg("cb", grSt_StageCallbacks, sizeof(grSt_StageCallbacks));
    reg("sd", &grSt_StageData, sizeof(grSt_StageData));
    reg("dat", grSt_StageData.data1, 16);
    FN(grStory_801E302C); FN(grStory_801E3030); FN(grStory_UnkStage0_OnLoad); FN(grStory_UnkStage0_OnStart);
    FN(grStory_801E30D0); FN(grStory_801E30D8); FN(grStory_801E31C0); FN(grStory_801E3224);
    FN(grStory_801E322C); FN(grStory_801E3230); FN(grStory_801E3234); FN(grStory_801E332C);
    FN(grStory_801E3334); FN(grStory_801E336C); FN(grStory_801E3370); FN(grStory_801E33D8);
    FN(grStory_801E33E0); FN(grStory_801E3414); FN(grStory_801E3418); FN(grStory_801E366C);
    FN(grStory_801E36D0); FN(grStory_801E36D8); FN(grStory_StageSlotParamSwap);
    fnreg("grDisplay_801C5DB0", (void*) grDisplay_801C5DB0);
    DUMP("cb", grSt_StageCallbacks);
    DUMP("sd", grSt_StageData);
    state_end();

    for (c = 0; c < 80; c++) { /* grStory_801E30D8 and the callbacks it installs */
        CASE(c);
        rseed(100 + c);
        gr_reset();
        yakumono_param = mkyp();
        stage_info.unk8C.b4 = rnd() & 1;
        stage_info.unk8C.b5 = rnd() & 1;
        printf("r"); TRP(grStory_801E30D8(c % 4)); printf("\n");
        state();
    }
    for (c = 0; c < 20; c++) { /* OnInit / OnStart / OnLoad / OnDemoInit */
        CASE(1000 + c);
        rseed(2000 + c);
        gr_reset();
        yakumono_param = 0;
        grStory_801E3030();
        state();
        grStory_UnkStage0_OnStart();
        grStory_UnkStage0_OnLoad();
        grStory_801E302C((int) rnd());
        state();
    }
    for (c = 0; c < 30; c++) { /* the plain stage callbacks */
        HSD_GObj* g;
        CASE(2000 + c);
        rseed(3000 + c);
        gr_reset();
        yakumono_param = mkyp();
        g = mkgobj();
        {
            Ground* gp = (Ground*) g->user_data;
            gp->x10_flags.b0 = rnd() & 1; gp->x10_flags.b5 = rnd() & 1; gp->x11_flags.b012 = rnd() & 7;
            gp->x11_flags.b7 = rnd() & 1;
        }
        grStory_801E31C0(g); state();
        printf("r %d\n", (int) grStory_801E3224(g));
        grStory_801E322C(g); grStory_801E3230(g);
        grStory_801E3234(g); state();
        printf("r %d\n", (int) grStory_801E332C(g));
        grStory_801E336C(g);
        grStory_801E3370(g); state();
        printf("r %d\n", (int) grStory_801E33D8(g));
        grStory_801E3414(g);
        printf("r %d\n", (int) grStory_801E30D0());
        printf("r"); TRP(grStory_801E36D0((int) rnd())); printf("\n");
        {
            Vec3 v;
            int a1;
            HSD_JObj* j;
            v.x = rf(); v.y = rf(); v.z = rf();
            a1 = (int) rnd();
            j = mkjobj();
            printf("r %d\n", (int) grStory_801E36D8(&v, a1, j));
        }
        state();
    }
    for (c = 0; c < 400; c++) { /* grStory_801E3418 (shy guy spawner) and grStory_801E3334 */
        HSD_GObj* g;
        Ground* gp;
        CASE(3000 + c);
        rseed(4000 + c);
        gr_reset();
        yakumono_param = mkyp();
        g = mkgobj();
        gp = (Ground*) g->user_data;
        gp->u.shyguys.timer = (c % 3 == 0) ? (int) (rnd() % 4) : 0;
        gp->u.shyguys.count = (s8) (rnd() % 8);
        gp->u.shyguys.pattern = (s8) ((int) (rnd() % 8) - 1);
        if (c % 4 == 3) {
            grStory_801E3334(g);
        } else {
            grStory_801E3418(g);
        }
        state();
    }
    for (c = 0; c < 300; c++) { /* grStory_801E366C (Randall puff) and grStory_801E33E0 */
        HSD_GObj* g;
        Ground* gp;
        CASE(5000 + c);
        rseed(6000 + c);
        gr_reset();
        yakumono_param = mkyp();
        g = mkgobj();
        gp = (Ground*) g->user_data;
        gp->u.randall.timer = (s16) ((int) (rnd() % 8) - 4);
        gp->u.randall.jobj = (c % 5 == 0) ? 0 : mkjobj();
        if (c % 3 == 0) {
            grStory_801E33E0(g);
        } else {
            grStory_801E366C(g);
        }
        state();
    }
    for (c = 0; c < 6; c++) { /* grStory_StageSlotParamSwap */
        void* prev;
        void* a;
        CASE(7000 + c);
        rseed(8000 + c);
        gr_reset();
        yakumono_param = (c % 2) ? 0 : mkyp();
        a = AZ(32);
        prev = grStory_StageSlotParamSwap(a);
        printf("r"); TRP(prev); printf("\n");
        state();
    }
    return 0;
}
