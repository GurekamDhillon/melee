/* State oracle for src/tu/gm_19EF.rs (see tu_gm_180A_ref.c). The TU is included through the copy
 * tu_shard529.py writes (config "patch"): three expressions that address the coin jobj slots with
 * 32-bit pointer arithmetic are spelled as the named slot array so the native layout means what
 * the GC layout means. The state struct is compared field by field (native and GC offsets differ). */
#include "gm_19EF_tu.c"
#include "oracle.h"
#include <string.h>

void __assert(const char* f, u32 l, const char* c) { printf("ASSERT %s", c); exit(7); }

static ou64 btn(void)
{
    unsigned int r = rnd();
    return ((r & 1) ? 0x1000ull : 0ull) | ((r & 2) ? 0x100ull : 0ull);
}
static HSD_JObj* mkjobj(void)
{
    HSD_JObj* j = (HSD_JObj*) AZ(sizeof(HSD_JObj));
    return j;
}
/* a jobj with the dobj->mobj and aobj the screens reach through */
static HSD_JObj* mkjobjd(void)
{
    HSD_JObj* j = mkjobj();
    j->u.dobj = (HSD_DObj*) AZ(sizeof(HSD_DObj));
    j->u.dobj->mobj = (HSD_MObj*) AZ(64);
    j->aobj = (HSD_AObj*) AZ(sizeof(HSD_AObj));
    j->aobj->curr_frame = (float) (rnd() % 4);
    j->aobj->end_frame = (float) (rnd() % 4);
    return j;
}
/* what HSD_JObjLoadJoint returns: a tree with every child/next the screens dereference */
static HSD_JObj* mktree(void)
{
    HSD_JObj* root = mkjobj();
    HSD_JObj* c1 = mkjobj();
    HSD_JObj* c2 = mkjobj();
    HSD_JObj* c3 = mkjobj();
    HSD_JObj* n1 = mkjobj();
    HSD_JObj* k1 = mkjobj();
    HSD_JObj* k2 = mkjobj();
    HSD_JObj* k3 = mkjobj();
    HSD_JObj* prev = c2;
    int i;
    root->child = c1;
    c1->child = c2;
    c1->next = n1;
    c2->child = c3;
    for (i = 0; i < 12; i++) {
        HSD_JObj* s = mkjobj();
        prev->next = s;
        prev = s;
    }
    n1->child = k1;
    k1->next = k2;
    k2->next = k3;
    return root;
}
static HSD_GObj* mkgobj(HSD_JObj* j)
{
    HSD_GObj* g = (HSD_GObj*) AZ(sizeof(HSD_GObj));
    g->hsd_obj = j;
    return g;
}
#include "tu_gm_19EF_stubs.inc"

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
/* the symbols: the model joint (A_C) is a joint, the others scenes */
HSD_Archive* lbArchive_LoadSymbols(const char* file, void* d0, ...)
{
    va_list ap;
    void* dst = d0;
    printf("lbArchive_LoadSymbols");
    TRS(file);
    va_start(ap, d0);
    while (dst != 0) {
        const char* name = va_arg(ap, const char*);
        unsigned long long nx;
        TRP(dst);
        TRS(name);
        *(void**) dst = dst == (void*) &lbl_804D66AC ? (void*) AZ(64) : (void*) mkscene();
        nx = va_arg(ap, unsigned long long);
        dst = (void*) (nx & 0xffffffffull ? (void*) nx : 0);
    }
    va_end(ap);
    printf("\n");
    return (HSD_Archive*) AZ(sizeof(HSD_Archive));
}
int HSD_SisLib_803A6B98(HSD_Text* t, f32 x, f32 y, const char* fmt, ...)
{
    printf("HSD_SisLib_803A6B98");
    TRP(t); TRF(x); TRF(y); TRS(fmt);
    printf("\n");
    return 0;
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
    *out = mkjobjd();
    return 1;
}

#define F(f) dump(#f, &(lbl_80479A98.f), sizeof(lbl_80479A98.f))
static void state(void)
{
    F(x0); F(x4); F(x8); F(xC); F(x10); F(x14); F(x15); F(x16); F(x18); F(x1C); F(x1E); F(x20); F(x22); F(x24);
    F(x28.typed.gobj); F(x28.typed.jobjs); F(x54); F(x58); F(x5C); F(x60); F(x64); F(x68); F(x6C); F(x70);
    DUMP("b0", lbl_804D66B0); DUMP("b8", lbl_804D66B8); DUMP("c0", lbl_804D66C0); DUMP("c8", lbl_804D66C8);
    DUMP("d0", lbl_804D66D0); DUMP("d8", lbl_804D66D8); DUMP("e0", lbl_804D66E0); DUMP("e8", lbl_804D66E8);
    DUMP("a98", lbl_804D6698); DUMP("a9c", lbl_804D669C); DUMP("aa0", lbl_804D66A0); DUMP("aa4", lbl_804D66A4);
    DUMP("aa8", lbl_804D66A8); DUMP("aac", lbl_804D66AC); DUMP("af0", lbl_804D66F0); DUMP("af4", lbl_804D66F4);
    DUMP("af8", lbl_804D66F8);
    DUMPARENA();
    state_end();
}
static int pick(void)
{
    static const int v[] = { 0xA, 0x19, 0x4F, 0x50, 0x5A, 0x64, 0x6E, 0x78 };
    unsigned int r = rnd() % 10;
    return r < 8 ? v[r] : (int) (rnd() % 0x80);
}
static void seed_state(void)
{
    int i;
    memset(&lbl_80479A98, 0, sizeof(lbl_80479A98));
    {
        static const unsigned char x0s[] = { 0, 1, 8, 9, 0xA, 3 };
        lbl_80479A98.x0 = x0s[rnd() % 6];
    }
    lbl_80479A98.x4 = (int) (rnd() % 4);
    lbl_80479A98.x8 = (int) (rnd() % 2000);
    lbl_80479A98.xC = rnd() % 2 ? lbl_80479A98.x8 : lbl_80479A98.x8 / 2;
    lbl_80479A98.x10 = 1 + rnd() % 30;
    lbl_80479A98.x14 = (u8) (rnd() % 2);
    lbl_80479A98.x15 = (u8) (rnd() % 4);
    lbl_80479A98.x16 = (u8) (rnd() % 4);
    lbl_80479A98.x1C = (u16) (rnd() % 20);
    lbl_80479A98.x1E = (u16) (rnd() % 30);
    lbl_80479A98.x20 = (u8) (rnd() % 2);
    lbl_80479A98.x22 = (u16) (rnd() % 500);
    lbl_80479A98.x24 = (HSD_Text*) AZ(sizeof(HSD_Text));
    lbl_80479A98.x28.typed.gobj = mkgobj(mktree());
    for (i = 0; i < 10; i++) lbl_80479A98.x28.typed.jobjs[i] = mkjobjd();
    lbl_80479A98.x54 = mkjobjd();
    lbl_80479A98.x58 = mkjobjd();
    lbl_80479A98.x5C = mkjobjd();
    lbl_80479A98.x60 = (u8) (rnd() % 2);
    lbl_80479A98.x64 = (int) (rnd() % 60);
    lbl_80479A98.x68 = (int) (rnd() % 30);
    lbl_80479A98.x6C = (int) (rnd() % 30);
    lbl_80479A98.x70 = (int) (rnd() % 12);
    lbl_804D66B0.x0 = mkjobjd(); lbl_804D66B0.x4 = (int) (rnd() % 0x340);
    lbl_804D66B8.x0 = mkjobjd();
    lbl_804D66C0.x0 = mkjobjd(); lbl_804D66C0.x4 = pick();
    lbl_804D66C8.x0 = mkjobjd(); lbl_804D66C8.x4 = pick();
    lbl_804D66D0.x0 = mkjobjd(); lbl_804D66D0.x4 = rnd() % 4 == 0 ? 0x2B1 + (int) (rnd() % 2) : (int) (rnd() % 0x2B4);
    lbl_804D66D8.x0 = mkjobjd();
    lbl_804D66E0.x0 = mktree(); lbl_804D66E0.x4 = rnd() % 4 == 0 ? 0x31 + (int) (rnd() % 2) : (int) (rnd() % 0x54);
    lbl_804D66E8.x0 = mkjobjd();
    lbl_804D6698 = (HSD_Archive*) AZ(sizeof(HSD_Archive));
    lbl_804D66F0 = (HSD_Archive*) AZ(sizeof(HSD_Archive));
    lbl_804D66F4 = mkjobjd();
    lbl_804D66F8 = rnd() % 4 == 0 ? 0xBC + (rnd() % 3) : rnd() % 0xC0;
}

int main(void)
{
    int c;
    arena_init();
    SIZE(DebugGameOverData);
    SIZE(HSD_JObj);
    SIZE(HSD_Text);
    reg("b0", &lbl_804D66B0, sizeof(lbl_804D66B0));
    reg("b8", &lbl_804D66B8, sizeof(lbl_804D66B8));
    reg("c0", &lbl_804D66C0, sizeof(lbl_804D66C0));
    reg("c8", &lbl_804D66C8, sizeof(lbl_804D66C8));
    reg("d0", &lbl_804D66D0, sizeof(lbl_804D66D0));
    reg("d8", &lbl_804D66D8, sizeof(lbl_804D66D8));
    reg("e0", &lbl_804D66E0, sizeof(lbl_804D66E0));
    reg("e8", &lbl_804D66E8, sizeof(lbl_804D66E8));
    reg("a9c", &lbl_804D669C, sizeof(lbl_804D669C));
    reg("aa0", &lbl_804D66A0, sizeof(lbl_804D66A0));
    reg("aa4", &lbl_804D66A4, sizeof(lbl_804D66A4));
    reg("aa8", &lbl_804D66A8, sizeof(lbl_804D66A8));
    reg("aac", &lbl_804D66AC, sizeof(lbl_804D66AC));
    reg("af4", &lbl_804D66F4, sizeof(lbl_804D66F4));
    reg("st", &lbl_80479A98, sizeof(lbl_80479A98));
    fnreg("fn_8019F810", (void*) fn_8019F810);
    fnreg("fn_8019F6EC", (void*) fn_8019F6EC);
    fnreg("fn_801A0B60", (void*) fn_801A0B60);
    fnreg("HSD_GObj_JObjCallback", (void*) HSD_GObj_JObjCallback);
    fnreg("HSD_GObj_LObjCallback", (void*) HSD_GObj_LObjCallback);
    fnreg("HSD_GObj_803910D8", (void*) HSD_GObj_803910D8);

    for (c = 0; c < 600; c++) { /* fn_8019EFC4 */
        CASE(c);
        rseed(100 + c);
        arena_reset();
        seed_state();
        fn_8019EFC4(&HSD_PadCopyStatus[0]);
        state();
    }
    for (c = 0; c < 120; c++) { /* fn_8019F1D0 */
        CASE(1000 + c);
        rseed(2000 + c);
        arena_reset();
        seed_state();
        lbl_80479A98.x8 = c % 5 == 0 ? 0 : c % 5 == 1 ? (int) (rnd() % 10) : c % 5 == 2 ? (int) 0x7FFFFFFF : (int) (rnd() % 100000);
        fn_8019F1D0();
        state();
    }
    for (c = 0; c < 1500; c++) { /* fn_8019F2D4 */
        static const unsigned int args[] = { 0, 0x1100, 0x1000, 0x100, 0x40000, 0x80000, 0xC0000, 0x41100 };
        CASE(2000 + c);
        rseed(3000 + c);
        arena_reset();
        seed_state();
        fn_8019F2D4(args[rnd() % 8]);
        state();
    }
    for (c = 0; c < 600; c++) { /* fn_8019F6EC */
        CASE(4000 + c);
        rseed(5000 + c);
        arena_reset();
        seed_state();
        if (c % 3 == 0) lbl_80479A98.x0 = 9;
        fn_8019F6EC(mkgobj(mkjobjd()));
        state();
    }
    for (c = 0; c < 1500; c++) { /* fn_8019F810 */
        CASE(6000 + c);
        rseed(7000 + c);
        arena_reset();
        seed_state();
        HSD_PadCopyStatus[lbl_80479A98.x15].trigger = rnd();
        fn_8019F810();
        state();
    }
    for (c = 0; c < 48; c++) { /* fn_8019F9C4 */
        CASE(9000 + c);
        rseed(10000 + c);
        arena_reset();
        seed_state();
        fn_8019F9C4(rnd() % 40);
        state();
    }
    for (c = 0; c < 80; c++) { /* gm_Scene_GOver_OnEnter / OnExit */
        DebugGameOverData* d;
        CASE(10000 + c);
        rseed(11000 + c);
        arena_reset();
        seed_state();
        d = (DebugGameOverData*) AZ(sizeof(DebugGameOverData));
        d->x0 = c % 7 == 0 ? 0 : rnd() % 100000;
        d->x4 = (int) rnd();
        d->x8 = rnd();
        d->xC = (int) rnd();
        d->ckind = rnd() % 30;
        d->slot = (u8) (rnd() % 4);
        d->x15 = (u8) rnd();
        d->x16 = (u16) (rnd() % 100);
        d->x18 = (u16) (rnd() % 6000);
        gm_Scene_GOver_OnEnter(d);
        state();
        gm_Scene_GOver_OnExit(d);
        DUMP("d", *d);
        state();
    }
    for (c = 0; c < 400; c++) { /* fn_801A0B60 */
        CASE(11000 + c);
        rseed(12000 + c);
        arena_reset();
        seed_state();
        fn_801A0B60(mkgobj(mkjobjd()));
        state();
    }
    for (c = 0; c < 4; c++) { /* ComingSoon */
        CASE(12000 + c);
        rseed(13000 + c);
        arena_reset();
        seed_state();
        gm_Scene_ComingSoon_OnEnter(0);
        state();
        gm_Scene_ComingSoon_OnExit(0);
        state();
    }
    return 0;
}
