/* State oracle for src/tu/gm_1884.rs (see tu_gm_180A_ref.c). The PPC bitfields of
 * StartMeleeRules are compared as a logical fingerprint (the C numbers them from the LSB, the
 * shipping PPC build from the MSB), never as bytes. */
/* __OSBusClock is a read of guest memory 0x800000F8 (a sign-extended wild pointer natively):
 * bind it to a variable holding the 162 MHz bus clock OSInit stores there */
#include <dolphin/os.h>
#undef __OSBusClock
extern unsigned int g_osbus;
#define __OSBusClock g_osbus
unsigned int g_osbus = 162000000u;
#include "melee/gm/gm_1884.c"
#include "oracle.h"
#include <string.h>

void __assert(const char* f, u32 l, const char* c) { printf("ASSERT %s", c); exit(7); }

static ou64 btn(void)
{
    unsigned int r = rnd();
    return ((r & 1) ? (1ull << 36) : 0ull) | ((r & 2) ? (1ull << 37) : 0ull) | ((r & 4) ? (1ull << 38) : 0ull) |
           ((r & 8) ? (1ull << 39) : 0ull) | ((r & 16) ? 0x100ull : 0ull);
}
static HSD_JObj* mkjobj(void)
{
    HSD_JObj* j = (HSD_JObj*) AZ(sizeof(HSD_JObj));
    j->translate.x = rf(); j->translate.y = rf(); j->translate.z = rf();
    return j;
}
static HSD_GObj* mkgobj(void)
{
    HSD_GObj* g = (HSD_GObj*) AZ(sizeof(HSD_GObj));
    g->hsd_obj = mkjobj();
    return g;
}
#include "tu_gm_1884_stubs.inc"

#include <stdarg.h>
static DynamicModelDesc** mkmodels(void)
{
    DynamicModelDesc** t = (DynamicModelDesc**) AZ(16);
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
        *(void**) dst = mkmodels();
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

#define RULE_FIELDS(X) \
    X(match_kind) X(x0_3) X(timer_enabled) X(timer_counts_up) X(x1_0) X(x1_1) X(x1_2) X(x1_3) X(x1_4) X(x1_5) \
    X(timer_shows_hours) X(friendly_fire) X(is_stock) X(x2_1) X(x2_2) X(single_button) X(disable_pausing) \
    X(x2_5) X(x2_6) X(x2_7) X(x3_0) X(x3_1) X(x3_2) X(x3_3) X(x3_4) X(x3_5) X(x3_6) X(x3_7) X(x4_0) X(is_vs) \
    X(x4_2) X(x4_3) X(x4_4) X(x4_5) X(x4_6) X(x4_7) X(x5_0) X(x5_1) X(x5_2) X(x5_3) X(x5_4) X(x5_5) X(x5_6) X(x5_7)

static StartMeleeData g_smd;

static void seed_rules(void)
{
#define X(f) g_smd.rules.f = rnd();
    RULE_FIELDS(X)
#undef X
    g_smd.rules.is_teams = (u8) rnd();
    g_smd.rules.item_freq = (s8) rnd();
    g_smd.rules.x20 = ((ou64) rnd() << 32) | rnd();
}
static void print_smd(void)
{
    ou64 h = 1469598103934665603ull;
#define X(f) h = h * 1000003ull + (ou64) g_smd.rules.f;
    RULE_FIELDS(X)
#undef X
    printf("rules %016llx %d %d %016llx\n", h, (int) g_smd.rules.is_teams, (int) g_smd.rules.item_freq, (ou64) g_smd.rules.x20);
    dump("players", g_smd.players, sizeof(g_smd.players));
}

static void state(void)
{
    DUMP("lbl", lbl_80473700);
    DUMP("gm", gm_80473814);
    DUMP("a28", lbl_804D6628);
    DUMP("a2c", lbl_804D662C);
    DUMPARENA();
    state_end();
}
static void seed_state(void)
{
    int i;
    rfill(&lbl_80473700, sizeof(lbl_80473700));
    rfill(&gm_80473814, sizeof(gm_80473814));
    lbl_80473700.count = (int) (rnd() % 4);
    lbl_80473700.mode = (int) (rnd() % 4);
    for (i = 0; i < 25; i++) lbl_80473700.char_data[i] = (int) (rnd() % 1200);
    lbl_80473700.result_cache[0] = (int) (rnd() % 1200);
    lbl_80473700.result_cache[1] = (int) (rnd() % 2);
    {
        int* p = (int*) &lbl_80473700;
        p[67] = (int) (rnd() % 1200);
        p[68] = (int) (rnd() % 2);
    }
    for (i = 0; i < 39; i++) {
        gm_80473814.jobjs[i] = mkjobj();
        gm_80473814.anim_frames[i] = (int) (rnd() % 40);
    }
    gm_80473814.gobj = 0;
    gm_80473814.x00 = (u8) (rnd() % 9);
    gm_80473814.x01 = (u8) (rnd() % 2);
    gm_80473814.x03 = (u8) (rnd() % 6);
    gm_80473814.menu_values[0] = (int) (rnd() % 6);
    gm_80473814.menu_values[1] = (int) (rnd() % 30);
    gm_80473814.menu_values[2] = (int) (rnd() % 3);
    gm_80473814.menu_values[3] = (int) (rnd() % 6);
    {
        unsigned int r = rnd() % 8;
        gm_80473814.menu_values[4] = r == 0 ? 0 : r == 1 ? 1 : r == 2 ? 998 : r == 3 ? 999 : r == 4 ? 0x3E7 : (int) (rnd() % 1000);
    }
    gm_80473814.menu_values[5] = (int) (rnd() % 2);
    gm_80473814.menu_values[6] = (int) (rnd() % 3);
    gm_80473814.text = (HSD_Text*) AZ(sizeof(HSD_Text));
    lbl_804D662C = mkmodels();
    lbl_804D6628 = (HSD_Archive*) AZ(sizeof(HSD_Archive));
}
static HSD_JObj* chain3(void)
{
    HSD_JObj* a = mkjobj();
    HSD_JObj* b = mkjobj();
    HSD_JObj* c = mkjobj();
    a->next = b;
    b->next = c;
    return a;
}

int main(void)
{
    int c;
    arena_init();
    SIZE(TrainingModeState);
    SIZE(CssSubStruct);
    SIZE(PlayerInitData);
    SIZE(HSD_Text);
    SIZE(TrainingItemEntry);
    reg("lbl", &lbl_80473700, sizeof(lbl_80473700));
    reg("gm", &gm_80473814, sizeof(gm_80473814));
    reg("a28", &lbl_804D6628, sizeof(lbl_804D6628));
    reg("a2c", &lbl_804D662C, sizeof(lbl_804D662C));
    reg("smd", &g_smd, sizeof(g_smd));
    fnreg("fn_801891F4", (void*) fn_801891F4);
    fnreg("fn_80188EE8", (void*) fn_80188EE8);
    fnreg("HSD_GObj_JObjCallback", (void*) HSD_GObj_JObjCallback);
    DUMP("items", gmTraining_ItemTable);
    DUMP("speeds", lbl_803B7C68);
    state_end();

    for (c = 0; c < 60; c++) { /* gm_80188454 */
        CASE(c);
        rseed(100 + c);
        arena_reset();
        seed_state();
        printf("r %d\n", gm_80188454(c % 27));
        state();
    }
    for (c = 0; c < 200; c++) { /* fn_8018846C, fn_801884F8 */
        CASE(1000 + c);
        rseed(2000 + c);
        arena_reset();
        seed_state();
        printf("r %d\n", fn_8018846C());
        state();
        printf("r %d\n", fn_801884F8());
        state();
    }
    for (c = 0; c < 200; c++) { /* fn_80188550 */
        CASE(2000 + c);
        rseed(3000 + c);
        arena_reset();
        seed_state();
        lbl_80473700.count = c % 4;
        fn_80188550((c / 4) % 4);
        state();
    }
    for (c = 0; c < 40; c++) { /* fn_80188644 */
        CASE(3000 + c);
        rseed(4000 + c);
        arena_reset();
        seed_state();
        lbl_80473700.count = c % 4;
        fn_80188644();
        state();
    }
    for (c = 0; c < 300; c++) { /* HUD digit counters */
        CASE(4000 + c);
        rseed(5000 + c);
        arena_reset();
        seed_state();
        fn_80188738(c % 7 == 0 ? 0 : chain3());
        state();
        fn_80188910(c % 7 == 1 ? 0 : chain3());
        state();
        fn_80188B3C(c % 7 == 2 ? 0 : chain3());
        state();
        {
            HSD_JObj* r = mkjobj();
            r->child = c % 7 == 3 ? 0 : chain3();
            fn_80188D3C(c % 7 == 4 ? 0 : r);
        }
        state();
    }
    for (c = 0; c < 500; c++) { /* fn_80188EE8 */
        CASE(5000 + c);
        rseed(6000 + c);
        arena_reset();
        seed_state();
        gm_80473814.jobjs[9] = chain3();
        gm_80473814.jobjs[5] = chain3();
        gm_80473814.jobjs[19] = chain3();
        gm_80473814.jobjs[27]->child = chain3();
        if (c % 3 == 0) gm_80473814.anim_frames[22] = 9 + (c % 4) * 10 + (int) (rnd() % 3);
        fn_80188EE8(mkgobj());
        state();
    }
    for (c = 0; c < 2500; c++) { /* fn_801891F4 */
        CASE(6000 + c);
        rseed(7000 + c);
        arena_reset();
        seed_state();
        if (c % 5 == 0) gm_80473814.x00 = (u8) (c / 5 % 9);
        fn_801891F4();
        state();
    }
    for (c = 0; c < 4; c++) { /* fn_80189B88 */
        CASE(9000 + c);
        rseed(10000 + c);
        arena_reset();
        seed_state();
        fn_80189B88();
        state();
    }
    for (c = 0; c < 40; c++) { /* gm_80189CDC */
        CASE(10000 + c);
        rseed(11000 + c);
        arena_reset();
        seed_state();
        rfill(g_smd.players, sizeof(g_smd.players));
        seed_rules();
        g_smd.players[0].slot = (u8) (rnd() % 5);
        gm_80189CDC(&g_smd);
        print_smd();
        state();
    }
    for (c = 0; c < 8; c++) { /* fn_8018A000 */
        CASE(11000 + c);
        rseed(12000 + c);
        arena_reset();
        seed_state();
        printf("r"); TRP(fn_8018A000()); printf("\n");
        state();
    }
    return 0;
}
