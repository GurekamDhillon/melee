/* State oracle for src/tu/gm_1832.rs (see tu_gm_180A_ref.c). The TU is included through the copy
 * tu_shard529.py writes (config "patch"): fn_801857C4 reads the splash image indices as
 * `(u8*) lbl_804735E8.x40)[0x90 + i]`, which only reaches xD0 with the console's 0x18-byte
 * HSD_ImageDesc; the copy names xD0 directly. The GX matrices and the SIS printf-style calls have
 * hand-written stubs. */
#include "gm_1832_tu.c"
#include "oracle.h"
#include <string.h>

void __assert(const char* f, u32 l, const char* c) { printf("ASSERT %s", c); exit(7); }

float MSL_TrigF_80400770[1] = { 1.5f };
float MSL_TrigF_80400774[1] = { 2.5f };

static HSD_GObjProc g_proc;
static HSD_GObj* g_ent[6];

static HSD_JObj* mkjobj(void)
{
    HSD_JObj* j = (HSD_JObj*) AZ(sizeof(HSD_JObj));
    j->translate.x = rf(); j->translate.y = rf(); j->translate.z = rf();
    j->scale.x = rf(); j->scale.y = rf(); j->scale.z = rf();
    return j;
}
/* a jobj with the dobj->mobj->tobj->aobj chain the intro animates */
static HSD_JObj* mkjobjt(void)
{
    HSD_JObj* j = mkjobj();
    j->u.dobj = (HSD_DObj*) AZ(sizeof(HSD_DObj));
    j->u.dobj->mobj = (HSD_MObj*) AZ(sizeof(HSD_MObj));
    j->u.dobj->mobj->tobj = (HSD_TObj*) AZ(sizeof(HSD_TObj));
    j->u.dobj->mobj->tobj->aobj = (HSD_AObj*) AZ(sizeof(HSD_AObj));
    return j;
}
static HSD_GObj* mkgobj(void)
{
    HSD_GObj* g = (HSD_GObj*) AZ(sizeof(HSD_GObj));
    g->hsd_obj = mkjobj();
    return g;
}
#include "tu_gm_1832_stubs.inc"

#include <stdarg.h>
static void* mktable(void)
{
    ou64 n = sizeof(*lbl_804D6604);
    float* t = (float*) AZ(n);
    ou64 i;
    for (i = 0; i < n / 4; i++) t[i] = rf();
    return t;
}
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
        *(void**) dst = strcmp(name, "gmIntroEasyTable") == 0 ? mktable() : (void*) mkscene();
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
    *out = mkjobjt();
    return 1;
}
int HSD_SisLib_803A6B98(HSD_Text* t, f32 x, f32 y, const char* fmt, ...)
{
    printf("HSD_SisLib_803A6B98");
    TRP(t); TRF(x); TRF(y); TRS(fmt);
    printf("\n");
    return 0;
}
s32 HSD_SisLib_803A70A0(HSD_Text* t, s32 idx, char* fmt, ...)
{
    printf("HSD_SisLib_803A70A0");
    TRP(t); TRI(idx); TRS(fmt);
    printf("\n");
    return 0;
}
void GXLoadPosMtxImm(f32 mtx[3][4], u32 id)
{
    int r, c;
    printf("GXLoadPosMtxImm");
    TRP(mtx); TRU(id);
    for (r = 0; r < 3; r++) for (c = 0; c < 4; c++) TRF(mtx[r][c]);
    printf("\n");
}
void GXSetProjection(f32 mtx[4][4], GXProjectionType type)
{
    int r, c;
    printf("GXSetProjection");
    TRI(type);
    for (r = 0; r < 4; r++) for (c = 0; c < 4; c++) TRF(mtx[r][c]);
    printf("\n");
}
void MTXOrtho(Mtx m, f32 t, f32 b, f32 l, f32 r, f32 n, f32 f)
{
    int i;
    printf("MTXOrtho");
    TRF(t); TRF(b); TRF(l); TRF(r); TRF(n); TRF(f);
    printf("\n");
    /* the PC build hands it a 4x4: fill all sixteen */
    for (i = 0; i < 16; i++) ((float*) m)[i] = rf();
}

#define F(f) dump(#f, &(f), sizeof(f))
static void state(void)
{
    F(lbl_804735A8.x0); F(lbl_804735A8.x4); F(lbl_804735A8.x38); F(lbl_804735A8.x3A); F(lbl_804735A8.x3C); F(lbl_804735A8.x3E);
    F(lbl_804735E8.x40); F(lbl_804735E8.x88); F(lbl_804735E8.xD0); F(lbl_804735E8.xDC);
    F(lbl_804735E8.xE0); F(lbl_804735E8.xE1); F(lbl_804735E8.xE2); F(lbl_804735E8.xE3);
    F(lbl_8047368C);
    F(lbl_804D65F0); F(lbl_804D65F4); F(lbl_804D65F8); F(lbl_804D65FC); F(lbl_804D6600); F(lbl_804D6604); F(lbl_804D6608);
    printf("proc %d\n", (int) g_proc.flags_3);
    DUMPARENA();
    state_end();
}
static void seed_state(void)
{
    int i;
    static const int x38s[] = { 4, 9, 0x2C, 0x45, 0x63, 0x8B, 0x8C, 5 };
    memset(&lbl_804735A8, 0, sizeof(lbl_804735A8));
    memset(&lbl_804735E8, 0, sizeof(lbl_804735E8));
    memset(&lbl_8047368C, 0, sizeof(lbl_8047368C));
    memset(&g_proc, 0, sizeof(g_proc));
    lbl_804735A8.x0 = (int) (rnd() % 2);
    for (i = 0; i < 13; i++) lbl_804735A8.x4[i] = mkjobjt();
    lbl_804735A8.x38 = (u16) (rnd() % 3 ? x38s[rnd() % 8] : (int) (rnd() % 0x90));
    lbl_804735A8.x3A = (u16) (rnd() % 0x34);
    lbl_804735A8.x3C = (u16) (rnd() % 0x34);
    lbl_8047368C.model_scale_kind = (int) (rnd() % 5);
    lbl_8047368C.game_type = (int) (rnd() % 4);
    lbl_8047368C.xEC = (u8) (rnd() % 4);
    lbl_8047368C.xED = rnd() % 4 == 0 ? 0x78 : (u8) (rnd() % 26);
    lbl_8047368C.xEE = (u8) (rnd() % 30);
    lbl_8047368C.xEF = (u8) (1 + rnd() % 2);
    lbl_8047368C.xF0 = (u8) (1 + rnd() % 2);
    for (i = 0; i < 3; i++) {
        lbl_8047368C.xF1[i] = rnd() % 6 == 0 ? 33 : (u8) (rnd() % 30);
        lbl_8047368C.xF4[i] = rnd() % 6 == 0 ? 0xE : rnd() % 8 == 0 ? 29 : rnd() % 8 == 0 ? 33 : (u8) (rnd() % 30);
        lbl_8047368C.xF7[i] = (u8) (rnd() % 4);
        lbl_8047368C.xFA[i] = (u8) (rnd() % 4);
        lbl_8047368C.xFD[i] = (u8) (rnd() % 2);
        lbl_8047368C.x100[i] = (u8) (rnd() % 2);
    }
    lbl_804735E8.xDC = mkgobj();
    lbl_804735E8.xE0 = (u8) (1 + rnd() % 3);
    lbl_804735E8.xE1 = (u8) (rnd() % 2);
    for (i = 0; i < 10; i++) lbl_804735E8.xD0[i] = (u8) (rnd() % lbl_804735E8.xE0);
    lbl_804D65F0 = mkgobj();
    lbl_804D65F4 = (HSD_Archive*) AZ(sizeof(HSD_Archive));
    lbl_804D65F8 = (HSD_Archive*) AZ(sizeof(HSD_Archive));
    lbl_804D65FC = mkscene();
    lbl_804D6600 = mkscene();
    lbl_804D6604 = (__typeof__(lbl_804D6604)) mktable();
    lbl_804D6608 = (int) (rnd() % 3) - 1;
    for (i = 0; i < 6; i++) g_ent[i] = mkgobj();
}

int main(void)
{
    int c, i;
    arena_init();
    SIZE(HSD_ImageDesc);
    SIZE(HSD_SObj);
    SIZE(HSD_SObjDesc2);
    SIZE(HSD_GObjProc);
    SIZE(ClassicModeEnterData);
    SIZE(struct gm_1832_8047368C_t);
    SIZE(*lbl_804D6604);
    reg("a0", &lbl_804735A8, sizeof(lbl_804735A8));
    reg("img", &lbl_804735E8, sizeof(lbl_804735E8));
    reg("k", &lbl_8047368C, sizeof(lbl_8047368C));
    reg("a5f0", &lbl_804D65F0, sizeof(lbl_804D65F0));
    reg("a5fc", &lbl_804D65FC, sizeof(lbl_804D65FC));
    reg("a600", &lbl_804D6600, sizeof(lbl_804D6600));
    reg("a604", &lbl_804D6604, sizeof(lbl_804D6604));
    reg("proc", &g_proc, sizeof(g_proc));
    fnreg("fn_801849E0", (void*) fn_801849E0);
    fnreg("fn_80184A04", (void*) fn_80184A04);
    fnreg("fn_80184A28", (void*) fn_80184A28);
    fnreg("fn_80184A4C", (void*) fn_80184A4C);
    fnreg("fn_80184A70", (void*) fn_80184A70);
    fnreg("fn_80184A94", (void*) fn_80184A94);
    fnreg("fn_80184AB8", (void*) fn_80184AB8);
    fnreg("fn_8018504C", (void*) fn_8018504C);
    fnreg("fn_801852FC", (void*) fn_801852FC);
    fnreg("fn_8018564C", (void*) fn_8018564C);
    fnreg("fn_8018569C", (void*) fn_8018569C);
    fnreg("fn_8018575C", (void*) fn_8018575C);
    fnreg("fn_801857C4", (void*) fn_801857C4);
    fnreg("fn_801859C8", (void*) fn_801859C8);
    fnreg("HSD_GObj_JObjCallback", (void*) HSD_GObj_JObjCallback);
    fnreg("HSD_GObj_LObjCallback", (void*) HSD_GObj_LObjCallback);
    fnreg("HSD_GObj_FogCallback", (void*) HSD_GObj_FogCallback);
    fnreg("HSD_GObj_803910D8", (void*) HSD_GObj_803910D8);
    fnreg("HSD_SObjLib_803A49E0", (void*) HSD_SObjLib_803A49E0);
    DUMP("scale", gm_1832_CharScale);
    DUMP("offset", gm_1832_CharOffset);
    DUMP("samus", gm_1832_SamusOffset);
    DUMP("sjis", lbl_803D9414);
    DUMP("sjis2", lbl_804D40A0);
    state_end();

    for (c = 0; c < 1200; c++) { /* the six layout callbacks */
        void (*fns[6])(HSD_GObj*) = { fn_801849E0, fn_80184A04, fn_80184A28, fn_80184A4C, fn_80184A70, fn_80184A94 };
        HSD_GObj* g;
        CASE(c);
        rseed(100 + c);
        arena_reset();
        seed_state();
        g = c % 2 ? g_ent[rnd() % 6] : mkgobj();
        fns[c % 6](g);
        state();
    }
    for (c = 0; c < 1200; c++) { /* fn_80184AB8 */
        CASE(2000 + c);
        rseed(3000 + c);
        arena_reset();
        seed_state();
        fn_80184AB8(mkgobj());
        state();
    }
    for (c = 0; c < 300; c++) { /* fn_8018504C */
        CASE(5000 + c);
        rseed(6000 + c);
        arena_reset();
        seed_state();
        if (c % 3 == 0) lbl_804735A8.x38 = 0x2D;
        fn_8018504C();
        state();
    }
    for (c = 0; c < 60; c++) { /* fn_801851C0 */
        CASE(7000 + c);
        rseed(8000 + c);
        arena_reset();
        seed_state();
        printf("r %d\n", fn_801851C0());
        state();
    }
    for (c = 0; c < 80; c++) { /* fn_801852FC */
        CASE(8000 + c);
        rseed(9000 + c);
        arena_reset();
        seed_state();
        fn_801852FC(mkgobj());
        state();
    }
    for (c = 0; c < 3; c++) { /* fn_80185408 */
        CASE(9000 + c);
        rseed(10000 + c);
        arena_reset();
        seed_state();
        fn_80185408((int) rnd(), 0.0f, 480.0f, 200.0f, 640.0f);
        state();
    }
    {
        static const double vals[] = { 10.0, 0.0, 1.0, 2.0, 100.0, 0.25, 1e10, -1.0, 4.0, 9.0, 3.0, 1e-300, 7.5, 123456.789 };
        double zz = 0.0;
        for (i = 0; i < 14; i++) {
            printf("sq %016llx\n", dbits(fn_801855BC(vals[i])));
        }
        printf("sq nan %016llx\n", dbits(fn_801855BC(zz / zz)));
        printf("sq -0 %016llx\n", dbits(fn_801855BC(-zz)));
        printf("sq inf %016llx\n", dbits(fn_801855BC(1.0 / zz)));
    }
    for (c = 0; c < 20; c++) { /* fn_8018564C / fn_8018569C / fn_8018575C */
        CASE(10000 + c);
        rseed(11000 + c);
        arena_reset();
        seed_state();
        fn_8018564C(mkgobj());
        fn_8018569C(mkgobj());
        fn_8018575C(mkgobj());
        state();
    }
    for (c = 0; c < 80; c++) { /* fn_801857C4 */
        CASE(11000 + c);
        rseed(12000 + c);
        arena_reset();
        seed_state();
        fn_801857C4(mkgobj());
        state();
    }
    for (c = 0; c < 50; c++) { /* fn_801859C8 */
        HSD_GObj* g;
        HSD_SObj* head = 0;
        CASE(12000 + c);
        rseed(13000 + c);
        arena_reset();
        seed_state();
        for (i = 0; i < 4; i++) {
            HSD_SObj* s = (HSD_SObj*) AZ(sizeof(HSD_SObj));
            s->x40 = rnd() % 16;
            s->x48 = rnd() % 4;
            s->next = head;
            head = s;
        }
        g = mkgobj();
        g->hsd_obj = head;
        fn_801859C8(g);
        state();
    }
    for (c = 0; c < 60; c++) { /* fn_80185A0C */
        CASE(13000 + c);
        rseed(14000 + c);
        arena_reset();
        seed_state();
        printf("r %d\n", fn_80185A0C());
        state();
    }
    for (c = 0; c < 80; c++) { /* fn_80185D64 */
        CASE(14000 + c);
        rseed(15000 + c);
        arena_reset();
        seed_state();
        fn_80185D64();
        state();
    }
    for (c = 0; c < 150; c++) { /* fn_80185E34 / fn_80185F5C */
        CASE(15000 + c);
        rseed(16000 + c);
        arena_reset();
        seed_state();
        printf("r %d\n", fn_80185E34());
        fn_80185F5C((s32) (rnd() % 3));
        state();
    }
    for (c = 0; c < 60; c++) { /* fn_80186080 / fn_801861B8 */
        CASE(16000 + c);
        rseed(17000 + c);
        arena_reset();
        seed_state();
        fn_80186080();
        state();
        fn_801861B8();
        state();
    }
    for (c = 0; c < 40; c++) { /* fn_80186400 */
        CASE(17000 + c);
        rseed(18000 + c);
        arena_reset();
        seed_state();
        fn_80186400();
        state();
    }
    for (c = 0; c < 150; c++) { /* fn_80186634 */
        CASE(18000 + c);
        rseed(19000 + c);
        arena_reset();
        seed_state();
        fn_80186634(0);
        state();
    }
    for (c = 0; c < 4; c++) { /* OnFrame */
        CASE(19000 + c);
        rseed(20000 + c);
        arena_reset();
        seed_state();
        gm_Scene_IntroEasy_OnFrame();
        state();
    }
    for (c = 0; c < 150; c++) { /* OnEnter */
        ClassicModeEnterData* d;
        CASE(20000 + c);
        rseed(21000 + c);
        arena_reset();
        seed_state();
        d = (ClassicModeEnterData*) AZ(sizeof(ClassicModeEnterData));
        {
            unsigned char* b = (unsigned char*) d;
            int k;
            for (k = 0; k < 0x20; k++) b[k] = 0;
            d->x0 = (int) (rnd() % 5);
            d->x4 = (int) (rnd() % 4);
            b[8] = (u8) (rnd() % 4);
            b[9] = rnd() % 4 == 0 ? 0x78 : (u8) (rnd() % 26);
            b[10] = (u8) (rnd() % 30);
            b[11] = (u8) (1 + rnd() % 2);
            b[12] = (u8) (1 + rnd() % 2);
            for (k = 13; k < 32; k++) b[k] = (u8) (rnd() % 6 == 0 ? 33 : rnd() % 30);
            if (c % 10 == 0) b[0x10] = 0x1A;
            if (c % 10 == 1) b[0x11] = 0x1A;
        }
        gm_Scene_IntroEasy_OnEnter(d);
        state();
    }
    return 0;
}
