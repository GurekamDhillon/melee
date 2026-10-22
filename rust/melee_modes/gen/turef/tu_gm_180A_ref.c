/* State oracle for src/tu/gm_180A.rs: the game TU #included verbatim (shipping contraction via
 * run.py), every callee outside it a tracing stub (tu_gm_180A_stubs.inc, generated), cases on a
 * synthetic state from the shared seeded stream. tests/tu_gm_180A.rs replays them on the port. */
#include "melee/gm/gm_180A.c"
#include "oracle.h"

static HSD_JObj* mkchain(void)
{
    HSD_JObj* root = (HSD_JObj*) AZ(sizeof(HSD_JObj));
    HSD_JObj* c1 = (HSD_JObj*) AZ(sizeof(HSD_JObj));
    HSD_JObj* c2 = (HSD_JObj*) AZ(sizeof(HSD_JObj));
    HSD_JObj* prev = c2;
    int k;
    root->child = c1;
    c1->child = c2;
    for (k = 0; k < 7; k++) {
        HSD_JObj* n = (HSD_JObj*) AZ(sizeof(HSD_JObj));
        prev->next = n;
        prev = n;
    }
    return root;
}

#include "tu_gm_180A_stubs.inc"

#include <stdarg.h>
/* symbols the archive loader would resolve */
static void* sym(const char* name)
{
    DynamicModelDesc** t = (DynamicModelDesc**) AZ(16);
    (void) name;
    t[0] = (DynamicModelDesc*) AZ(sizeof(DynamicModelDesc));
    t[0]->joint = (HSD_Joint*) AZ(64);
    return t;
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
        *(void**) dst = sym(name);
        nx = va_arg(ap, unsigned long long);
        dst = (void*) (nx & 0xffffffffull ? (void*) nx : 0);
    }
    va_end(ap);
    printf("\n");
    return (HSD_Archive*) AZ(sizeof(HSD_Archive));
}

static void state(void)
{
    printf("e48 %d %d %d %d\n", lbl_80472E48.b76, lbl_80472E48.b54, lbl_80472E48.b32, lbl_80472E48.b10);
    dump("e48t", (char*) &lbl_80472E48 + 1, sizeof(lbl_80472E48) - 1);
    DUMP("ec8", lbl_80472EC8);
    printf("g %d %d\n", lbl_804D65D4, lbl_804D65D8);
    dump("cc", &lbl_804D65CC, 8);
    dump("d0", &lbl_804D65D0, 8);
    dump("c8", &lbl_804D65C8, 8);
    DUMPARENA();
    state_end();
}
static void seed_e48(int smallx)
{
    int i;
    rfill(&lbl_80472E48, sizeof(lbl_80472E48));
    lbl_80472E48.b76 = rnd() & 3;
    lbl_80472E48.b54 = rnd() & 3;
    lbl_80472E48.b32 = rnd() & 3;
    lbl_80472E48.b10 = rnd() & 3;
    lbl_80472E48.unk_4 = (int) (rnd() % 300) - 20;
    lbl_80472E48.x10 = (u8) (rnd() % 4);
    lbl_80472E48.xC = rnd() % (smallx ? 0x120 : 0x40000000);
    for (i = 0; i < 0x1B; i++) lbl_80472E48.x14[i] = (int) (rnd() % 2500) - 100;
    for (i = 0; i < 4; i++) lbl_80472EC8[i] = (int) (rnd() % 2500) - 100;
}

int main(void)
{
    int c, i;
    arena_init();
    SIZE(struct lbl_80472E48_t);
    SIZE(HSD_GObj);
    SIZE(HSD_JObj);
    SIZE(DynamicModelDesc);
    SIZE(HSD_Archive);
    SIZE(HSD_PadStatus);
    reg("e48", &lbl_80472E48, sizeof(lbl_80472E48));
    reg("ec8", lbl_80472EC8, sizeof(lbl_80472EC8));
    reg("pad", HSD_PadCopyStatus, sizeof(HSD_PadCopyStatus));
    fnreg("fn_80180C14", (void*) fn_80180C14);
    fnreg("fn_80180C60", (void*) fn_80180C60);
    fnreg("fn_80181598", (void*) fn_80181598);
    fnreg("HSD_GObj_JObjCallback", (void*) HSD_GObj_JObjCallback);

    /* getters / setters */
    for (c = 0; c < 8; c++) {
        CASE(c);
        rseed(100 + c);
        arena_reset();
        seed_e48(0);
        printf("r %d\n", gm_80180AE4());
        printf("r"); TRP(gm_80180AF4()); printf("\n");
        {
            int a0 = (int) rnd();
            int a1 = (int) rnd();
            gm_80181A00(a0, a1);
        }
        state();
    }
    /* gm_80180B18 / gm_80180BA0 */
    for (c = 0; c < 8; c++) {
        CASE(100 + c);
        rseed(200 + c);
        arena_reset();
        seed_e48(0);
        gm_80180B18();
        state();
        gm_80180BA0();
        state();
    }
    /* fn_80180C14 */
    for (c = 0; c < 12; c++) {
        HSD_GObj* g;
        CASE(200 + c);
        rseed(300 + c);
        arena_reset();
        seed_e48(0);
        g = (HSD_GObj*) AZ(sizeof(HSD_GObj));
        g->hsd_obj = mkchain();
        fn_80180C14(g);
        state();
    }
    /* fn_80180C60: the distance counter */
    for (c = 0; c < 600; c++) {
        HSD_GObj* g;
        unsigned int s;
        float gf;
        CASE(300 + c);
        rseed(1000 + c);
        arena_reset();
        seed_e48(0);
        /* peek the distance the stub will return and sometimes make it equal to the last one */
        s = ORS;
        gf = rf();
        ORS = s;
        {
            int d = (int) (0.1f * gf);
            if (d < 0) d = 0;
            if (rnd() % 3 == 0) lbl_80472EC8[1] = d;
            if (rnd() % 3 == 0) lbl_80472EC8[3] = (int) (rnd() % 0x80);
            if (rnd() % 4 == 0) lbl_80472E48.b76 = 1;
        }
        lbl_804D65D4 = (int) (rnd() % 2);
        g = (HSD_GObj*) AZ(sizeof(HSD_GObj));
        g->hsd_obj = mkchain();
        fn_80180C60(g);
        state();
    }
    /* fn_80181598: the pause/continue logic */
    for (c = 0; c < 600; c++) {
        CASE(4000 + c);
        rseed(9000 + c);
        arena_reset();
        seed_e48(1);
        for (i = 0; i < 4; i++) {
            rfill(&HSD_PadCopyStatus[i], sizeof(HSD_PadStatus));
            HSD_PadCopyStatus[i].trigger = (rnd() % 2) ? 0x100u : 0u;
        }
        lbl_804D65D8 = (int) (rnd() % 0x120);
        if (rnd() % 4 == 0) lbl_80472E48.b10 = 1;
        if (rnd() % 4 == 0) lbl_80472E48.b10 = 0;
        if (rnd() % 3 == 0) lbl_80472E48.b32 = 1;
        fn_80181598();
        state();
    }
    /* scene setup */
    for (c = 0; c < 6; c++) {
        CASE(8000 + c);
        rseed(500 + c);
        arena_reset();
        seed_e48(0);
        lbl_804D65CC = (DynamicModelDesc**) sym("a");
        lbl_804D65D0 = (DynamicModelDesc**) sym("b");
        fn_80181708();
        state();
    }
    for (c = 0; c < 4; c++) {
        CASE(8100 + c);
        rseed(600 + c);
        arena_reset();
        seed_e48(0);
        gm_80181998();
        state();
    }
    return 0;
}
