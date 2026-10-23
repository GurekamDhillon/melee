/* State oracle for src/tu/grshrine.rs (see tu_grtcaptain_ref.c). */
#include "melee/gr/grshrine.c"
#include "oracle.h"
#include "tu_gr_common.h"
#include "tu_grshrine_stubs.inc"

static void state(void)
{
    printf("rs %u\n", ORS);
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
    SIZE(StageCallbacks);
    SIZE(StageData);
    reg("cb", grSh_StageCallbacks, sizeof(grSh_StageCallbacks));
    reg("sd", &grSh_StageData, sizeof(grSh_StageData));
    reg("dat", grSh_StageData.data1, 16);
    FN(grShrine_80201C60); FN(grShrine_80201C64); FN(grShrine_UnkStage0_OnLoad); FN(grShrine_UnkStage0_OnStart);
    FN(grShrine_80201D18); FN(grShrine_80201D20); FN(stageGObj0_OnInit); FN(grShrine_80201E34);
    FN(grShrine_80201E3C); FN(grShrine_80201E40); FN(grShrine_80201E44); FN(grShrine_80201E88);
    FN(grShrine_80201E90); FN(grShrine_80201E94); FN(grShrine_80201E98); FN(grShrine_80201E9C);
    FN(grShrine_80201F14); FN(grShrine_80201F1C); FN(grShrine_80201F40); FN(grShrine_80201F44);
    FN(grShrine_80201F4C);
    fnreg("grDisplay_801C5DB0", (void*) grDisplay_801C5DB0);
    DUMP("cb", grSh_StageCallbacks);
    DUMP("sd", grSh_StageData);
    state_end();

    for (c = 0; c < 60; c++) { /* grShrine_80201D20 and the callbacks it installs */
        CASE(c);
        rseed(100 + c);
        gr_reset();
        printf("r"); TRP(grShrine_80201D20(c % 3)); printf("\n");
        state();
    }
    for (c = 0; c < 20; c++) { /* OnInit / OnStart / OnLoad / OnDemoInit */
        CASE(1000 + c);
        rseed(2000 + c);
        gr_reset();
        grShrine_80201C64();
        state();
        grShrine_UnkStage0_OnStart();
        grShrine_UnkStage0_OnLoad();
        grShrine_80201C60((int) rnd());
        state();
    }
    for (c = 0; c < 24; c++) { /* every stage callback */
        HSD_GObj* g;
        CASE(2000 + c);
        rseed(3000 + c);
        gr_reset();
        g = mkgobj();
        /* random bitfield values, the rest of the Ground zero */
        {
            Ground* gp = (Ground*) g->user_data;
            gp->x10_flags.b0 = rnd() & 1; gp->x10_flags.b3 = rnd() & 1; gp->x10_flags.b5 = rnd() & 1;
            gp->x11_flags.b012 = rnd() & 7; gp->x11_flags.b4 = rnd() & 1; gp->x11_flags.b7 = rnd() & 1;
        }
        stageGObj0_OnInit(g); state();
        printf("r %d\n", (int) grShrine_80201E34(g));
        grShrine_80201E3C(g); grShrine_80201E40(g);
        grShrine_80201E44(g); state();
        printf("r %d\n", (int) grShrine_80201E88(g));
        grShrine_80201E90(g); grShrine_80201E94(g); grShrine_80201E98(g);
        grShrine_80201E9C(g); state();
        printf("r %d\n", (int) grShrine_80201F14(g));
        grShrine_80201F1C(g); grShrine_80201F40(g); state();
        printf("r %d\n", (int) grShrine_80201D18());
        printf("r"); TRP(grShrine_80201F44((int) rnd())); printf("\n");
        {
            Vec3 v;
            int a1;
            HSD_JObj* j;
            v.x = rf(); v.y = rf(); v.z = rf();
            a1 = (int) rnd();
            j = mkjobj();
            printf("r %d\n", (int) grShrine_80201F4C(&v, a1, j));
        }
        state();
    }
    return 0;
}
