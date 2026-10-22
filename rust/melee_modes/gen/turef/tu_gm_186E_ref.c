/* State oracle for src/tu/gm_186E.rs (see tu_gm_180A_ref.c). */
#include "melee/gm/gm_186E.c"
#include "oracle.h"

static HSD_JObj* mkjobj(void)
{
    HSD_JObj* j = (HSD_JObj*) AZ(sizeof(HSD_JObj));
    j->flags = rnd();
    j->translate.x = rf(); j->translate.y = rf(); j->translate.z = rf();
    j->scale.x = rf(); j->scale.y = rf(); j->scale.z = rf();
    return j;
}
static HSD_GObj* mkgobj(void)
{
    HSD_GObj* g = (HSD_GObj*) AZ(sizeof(HSD_GObj));
    g->hsd_obj = mkjobj();
    return g;
}

void __assert(const char* f, u32 l, const char* c) { printf("ASSERT %s", c); exit(7); }
#include "tu_gm_186E_stubs.inc"

#include <stdarg.h>
static SceneDesc* mkscene(void)
{
    SceneDesc* s = (SceneDesc*) AZ(sizeof(SceneDesc));
    s->models = (DynamicModelDesc**) AZ(16);
    s->models[0] = (DynamicModelDesc*) AZ(sizeof(DynamicModelDesc));
    s->models[0]->joint = (HSD_Joint*) AZ(64);
    s->cameras = (SceneCameraDesc*) AZ(sizeof(SceneCameraDesc) * 2);
    s->cameras[0].desc = (HSD_CObjDesc*) AZ(64);
    s->lights = (LightList**) AZ(16);
    s->fogs = (SceneFogDesc*) AZ(sizeof(SceneFogDesc) * 2);
    s->fogs[0].desc = (HSD_FogDesc*) AZ(64);
    return s;
}
HSD_Archive* lbArchive_80016DBC(const char* file, void* d0, ...)
{
    va_list ap;
    void* dst = d0;
    printf("lbArchive_80016DBC");
    TRS(file);
    va_start(ap, d0);
    while (dst != 0) {
        const char* name = va_arg(ap, const char*);
        unsigned long long nx;
        TRP(dst);
        TRS(name);
        *(void**) dst = mkscene();
        nx = va_arg(ap, unsigned long long);
        dst = (void*) (nx & 0xffffffffull ? (void*) nx : 0);
    }
    va_end(ap);
    printf("\n");
    return (HSD_Archive*) AZ(sizeof(HSD_Archive));
}
int lb_80011E24(HSD_JObj* j, HSD_JObj** out, ...)
{
    va_list ap;
    int id, end;
    va_start(ap, out);
    id = va_arg(ap, int);
    end = va_arg(ap, int);
    va_end(ap);
    printf("lb_80011E24");
    TRP(j); TRP(out); TRI(id); TRI(end);
    printf("\n");
    *out = mkjobj();
    return 1;
}

static void state(void)
{
    DUMP("a10", lbl_804D6610);
    DUMP("a14", lbl_804D6614);
    DUMP("a18", lbl_804D6618);
    DUMP("b0", lbl_804736B0);
    DUMPARENA();
    state_end();
}

int main(void)
{
    int c;
    arena_init();
    SIZE(SceneDesc);
    SIZE(HSD_GObj);
    SIZE(HSD_JObj);
    SIZE(gm_804736B0_t);
    reg("a10", &lbl_804D6610, sizeof(lbl_804D6610));
    reg("a14", &lbl_804D6614, sizeof(lbl_804D6614));
    reg("a18", &lbl_804D6618, sizeof(lbl_804D6618));
    reg("b0", &lbl_804736B0, sizeof(lbl_804736B0));
    reg("tbl", lbl_803D9498, sizeof(lbl_803D9498));
    fnreg("fn_80186EFC", (void*) fn_80186EFC);
    fnreg("fn_80186F6C", (void*) fn_80186F6C);
    fnreg("fn_80187494", (void*) fn_80187494);
    fnreg("HSD_GObj_JObjCallback", (void*) HSD_GObj_JObjCallback);
    fnreg("HSD_GObj_LObjCallback", (void*) HSD_GObj_LObjCallback);
    fnreg("HSD_GObj_FogCallback", (void*) HSD_GObj_FogCallback);
    DUMP("tbl", lbl_803D9498);
    state_end();

    for (c = 0; c < 400; c++) { /* fn_80186EFC: the frame counter */
        HSD_GObj* g;
        CASE(c);
        rseed(100 + c);
        arena_reset();
        g = mkgobj();
        lbl_804736B0.x0 = (int) (rnd() % 2);
        lbl_804736B0.x4 = (c % 5 == 0) ? 0x8B + (int) (rnd() % 3) : (int) (rnd() % 0xA0);
        lbl_804736B0.x8 = mkjobj();
        lbl_804736B0.xC = mkjobj();
        fn_80186EFC(g);
        state();
    }
    for (c = 0; c < 300; c++) { /* fn_80186F6C: place the model */
        HSD_GObj* g;
        CASE(1000 + c);
        rseed(2000 + c);
        arena_reset();
        g = mkgobj();
        lbl_804D6618.x0 = c % 28;
        lbl_804D6618.x4 = (int) (rnd() % 8);
        lbl_804736B0.x8 = mkjobj();
        fn_80186F6C(g);
        state();
    }
    for (c = 0; c < 28; c++) { /* fn_801873F0 */
        CASE(2000 + c);
        rseed(3000 + c);
        arena_reset();
        lbl_804D6618.x0 = (int) (rnd() % 30);
        lbl_804D6618.x4 = (int) (rnd() % 8);
        lbl_804D6610 = (HSD_Archive*) AZ(sizeof(HSD_Archive));
        printf("r"); TRP(fn_801873F0()); printf("\n");
        state();
    }
    for (c = 0; c < 40; c++) { /* fn_80187494 */
        HSD_GObj* g;
        CASE(3000 + c);
        rseed(4000 + c);
        arena_reset();
        g = mkgobj();
        g->gxlink_prios = rnd();
        fn_80187494(g, (int) rnd());
        state();
    }
    for (c = 0; c < 8; c++) { /* fn_801874FC */
        CASE(4000 + c);
        rseed(5000 + c);
        arena_reset();
        lbl_804D6614 = mkscene();
        lbl_804736B0.x8 = 0;
        lbl_804736B0.xC = 0;
        fn_801874FC();
        state();
    }
    for (c = 0; c < 2; c++) { /* fn_80187714 */
        CASE(5000 + c);
        rseed(6000 + c);
        arena_reset();
        fn_80187714();
        state();
    }
    for (c = 0; c < 12; c++) { /* OnFrame */
        CASE(6000 + c);
        rseed(7000 + c);
        arena_reset();
        lbl_804D6610 = (HSD_Archive*) AZ(sizeof(HSD_Archive));
        lbl_804736B0.x0 = (int) (rnd() % 2);
        gm_Scene_IntroAllstar_OnFrame();
        state();
    }
    for (c = 0; c < 12; c++) { /* OnEnter */
        struct enterdata* d;
        CASE(7000 + c);
        rseed(8000 + c);
        arena_reset();
        d = (struct enterdata*) AZ(sizeof(struct enterdata));
        d->x0 = (int) (rnd() % 28);
        d->x4 = (int) (rnd() % 8);
        lbl_804736B0.x0 = 5;
        lbl_804736B0.x4 = 9;
        gm_Scene_IntroAllstar_OnEnter(d);
        state();
    }
    return 0;
}
