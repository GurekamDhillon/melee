/* State oracle for src/tu/gm_1879.rs (see tu_gm_180A_ref.c). The C numbers bitfields from the
 * LSB, the shipping PPC build from the MSB: the state dump rebuilds the two bitfield bytes
 * (recomposed MSB-first) so both sides compare the PPC view. */
#include "melee/gm/gm_1879.c"
#include "oracle.h"
#include <string.h>

void __assert(const char* f, u32 l, const char* c) { printf("ASSERT %s", c); exit(7); }

static ou64 btn(void)
{
    unsigned int r = rnd();
    return ((r & 1) ? 0x100ull : 0ull) | ((r & 2) ? 0x1000ull : 0ull);
}
static unsigned char rev8(unsigned char b)
{
    unsigned char r = 0;
    int i;
    for (i = 0; i < 8; i++) if (b & (1 << i)) r |= (unsigned char) (1 << (7 - i));
    return r;
}
#include "tu_gm_1879_stubs.inc"

#include <stdarg.h>
static SceneDesc* mkscene(void)
{
    SceneDesc* s = (SceneDesc*) AZ(sizeof(SceneDesc));
    int i, k;
    s->models = (DynamicModelDesc**) AZ(16 * sizeof(void*));
    for (i = 0; i < 13; i++) {
        DynamicModelDesc* d = (DynamicModelDesc*) AZ(sizeof(DynamicModelDesc));
        s->models[i] = d;
        d->joint = (HSD_Joint*) AZ(64);
        if (i == 12 || rnd() % 4 != 0) {
            d->anims = (HSD_AnimJoint**) AZ(4 * sizeof(void*));
            for (k = 0; k < 4; k++) d->anims[k] = (rnd() % 2) ? (HSD_AnimJoint*) AZ(64) : 0;
        }
    }
    s->cameras = (SceneCameraDesc*) AZ(sizeof(SceneCameraDesc) * 2);
    s->cameras[0].desc = (HSD_CObjDesc*) AZ(64);
    s->lights = (LightList**) AZ(16);
    return s;
}
static SceneCameraDesc* mkcam(void)
{
    SceneCameraDesc* c = (SceneCameraDesc*) AZ(sizeof(SceneCameraDesc) * 2);
    c->desc = (HSD_CObjDesc*) AZ(64);
    c->anims = (HSD_CameraAnim**) AZ(2 * sizeof(void*));
    c->anims[0] = (HSD_CameraAnim*) AZ(64);
    return c;
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
        *(void**) dst = strcmp(name, "ScItrNormal_scene_data") == 0 ? (void*) mkscene() : (void*) mkcam();
        nx = va_arg(ap, unsigned long long);
        dst = (void*) (nx & 0xffffffffull ? (void*) nx : 0);
    }
    va_end(ap);
    printf("\n");
    return (HSD_Archive*) AZ(sizeof(HSD_Archive));
}

static HSD_CObj* mkcobj(int eye_ok)
{
    HSD_CObj* c = (HSD_CObj*) AZ(sizeof(HSD_CObj));
    c->eyepos = (HSD_WObj*) AZ(sizeof(HSD_WObj));
    if (eye_ok) {
        c->eyepos->aobj = (HSD_AObj*) AZ(sizeof(HSD_AObj));
        c->eyepos->aobj->curr_frame = rf();
    }
    c->interest = (HSD_WObj*) AZ(sizeof(HSD_WObj));
    if (rnd() % 2) {
        c->interest->aobj = (HSD_AObj*) AZ(sizeof(HSD_AObj));
        c->interest->aobj->curr_frame = rf();
    }
    return c;
}
static HSD_GObj* mkgobj(void* obj)
{
    HSD_GObj* g = (HSD_GObj*) AZ(sizeof(HSD_GObj));
    g->hsd_obj = obj;
    return g;
}
static HSD_JObj* mkjobj(void) { return (HSD_JObj*) AZ(sizeof(HSD_JObj)); }

static void seed(int full)
{
    gm_1832_804736C0_t* d = &lbl_804736C0;
    memset(d, 0, sizeof(*d));
    d->x0 = mkscene();
    d->x4 = mkcam();
    d->x8 = full ? mkgobj(mkcobj(rnd() % 2)) : 0;
    d->x34 = (u16) rnd();
    d->x36.stage_index = rnd() % 12;
    d->x36.done = rnd() % 2;
    d->x36.flash = rnd() % 2;
    d->x36.active = rnd() % 2;
    d->x37.frame_counter = rnd() % 16;
    d->x37.anim_state = rnd() % 4;
    d->x37.state2 = rnd() % 4;
    d->x38 = (u8) (rnd() % 4);
    lbl_804D6620 = (HSD_Archive*) AZ(sizeof(HSD_Archive));
}
static void state(void)
{
    gm_1832_804736C0_t c; memcpy(&c, &lbl_804736C0, sizeof(c));
    ((unsigned char*) &c)[offsetof(gm_1832_804736C0_t, x36)] = (unsigned char) ((lbl_804736C0.x36.stage_index << 3) | (lbl_804736C0.x36.done << 2) | (lbl_804736C0.x36.flash << 1) | lbl_804736C0.x36.active);
    ((unsigned char*) &c)[offsetof(gm_1832_804736C0_t, x37)] = (unsigned char) ((lbl_804736C0.x37.frame_counter << 4) | (lbl_804736C0.x37.anim_state << 2) | lbl_804736C0.x37.state2);
    DUMP("s", c);
    DUMP("a20", lbl_804D6620);
    DUMPARENA();
    state_end();
}

int main(void)
{
    int c;
    arena_init();
    SIZE(gm_1832_804736C0_t);
    SIZE(SceneDesc);
    SIZE(HSD_CObj);
    SIZE(HSD_AObj);
    SIZE(gm_80187F48_EnterData);
    reg("s", &lbl_804736C0, sizeof(lbl_804736C0));
    reg("a20", &lbl_804D6620, sizeof(lbl_804D6620));
    fnreg("fn_80187910", (void*) fn_80187910);
    fnreg("fn_80187AB4", (void*) fn_80187AB4);
    fnreg("fn_80187C9C", (void*) fn_80187C9C);
    fnreg("fn_80187CF4", (void*) fn_80187CF4);
    fnreg("Camera_800304E0", (void*) Camera_800304E0);
    fnreg("HSD_GObj_803910D8", (void*) HSD_GObj_803910D8);
    fnreg("HSD_GObj_JObjCallback", (void*) HSD_GObj_JObjCallback);
    fnreg("HSD_GObj_LObjCallback", (void*) HSD_GObj_LObjCallback);
    DUMP("bgm", gm_1832_NormalBgmIds);
    DUMP("masks", gm_1832_NormalAudioMasks);
    state_end();

    for (c = 0; c < 500; c++) { /* fn_80187910 */
        CASE(c);
        rseed(100 + c);
        arena_reset();
        seed(0);
        {
            HSD_GObj* g = mkgobj(mkcobj(1));
            fn_80187910(g);
        }
        state();
    }
    for (c = 0; c < 600; c++) { /* fn_80187AB4 */
        HSD_GObj* g;
        CASE(1000 + c);
        rseed(2000 + c);
        arena_reset();
        seed(0);
        g = mkgobj(mkjobj());
        fn_80187AB4(g);
        state();
    }
    for (c = 0; c < 3; c++) { /* fn_80187C9C */
        CASE(2000 + c);
        rseed(3000 + c);
        arena_reset();
        seed(0);
        fn_80187C9C(mkgobj(mkjobj()), (int) rnd());
        state();
    }
    for (c = 0; c < 600; c++) { /* fn_80187CF4 */
        HSD_GObj* g;
        CASE(3000 + c);
        rseed(4000 + c);
        arena_reset();
        seed(0);
        g = mkgobj(mkjobj());
        fn_80187CF4(g);
        state();
    }
    for (c = 0; c < 48; c++) { /* OnEnter */
        gm_80187F48_EnterData* d;
        CASE(4000 + c);
        rseed(5000 + c);
        arena_reset();
        seed(0);
        d = (gm_80187F48_EnterData*) AZ(sizeof(gm_80187F48_EnterData));
        d->x0 = (u8) (rnd() % 4);
        d->stage_index = (u8) (c % 12);
        d->stkind = (u16) (rnd() % 40);
        gm_Scene_IntroNormal_OnEnter(d);
        state();
    }
    for (c = 0; c < 4; c++) { /* OnLeave / OnFrame / 8018841C */
        CASE(5000 + c);
        rseed(6000 + c);
        arena_reset();
        seed(0);
        gm_Scene_IntroNormal_OnLeave(0);
        gm_Scene_IntroNormal_OnFrame();
        printf("r %d\n", (int) gm_8018841C());
        printf("r %d\n", (int) gm_8018841C());
        state();
    }
    for (c = 0; c < 40; c++) { /* gm_801883C0 */
        CASE(6000 + c);
        rseed(7000 + c);
        arena_reset();
        seed(c % 4 != 0);
        printf("r %08x\n", fbits(gm_801883C0()));
        state();
    }
    return 0;
}
