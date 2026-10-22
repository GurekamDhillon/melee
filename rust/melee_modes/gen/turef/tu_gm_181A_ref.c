/* State oracle for src/tu/gm_181A.rs (see tu_gm_180A_ref.c). The TU is included through the copy
 * tu_shard529.py writes (config "patch"): gm_80182554 addresses the record state as
 * lbl_80472ED8 + 0x6BC, which only holds on the console layout, so the copy points it at
 * lbl_80473594 (what the game's own TODO says it is). PlayerInitData is compared as a logical
 * fingerprint (its PPC bitfields are numbered from the other end natively). */
#include "gm_181A_tu.c"
#include "oracle.h"
#include <string.h>

void __assert(const char* f, u32 l, const char* c) { printf("ASSERT %s", c); exit(7); }

static ou64 bt(void)
{
    unsigned int r = rnd();
    return ((r & 1) ? 0x1000ull : 0ull) | ((r & 2) ? 0x100ull : 0ull);
}
static int gv(void)
{
    static const int v[] = { 0x190, 0x280, 0x370, 0x4B0, 0x4B1 };
    unsigned int r = rnd() % 8;
    return r < 5 ? v[r] : (int) (rnd() % 0x500);
}

#define PID_FIELDS(X) \
    X(rumble_enabled) X(xC_b1) X(vs_metal) X(xC_b3) X(vs_invisible) X(xC_b5) X(xC_b6) X(xC_b7) \
    X(xD_b0) X(xD_b1) X(xD_b2) X(xD_b3) X(xD_b4) X(xD_b5) X(xD_b6) X(xD_b7)
static void pid_randomize(PlayerInitData* p)
{
    p->ckind = (s8) rnd();
    p->slot_type = (u8) rnd();
    p->stocks = (s8) rnd();
    p->color = (u8) rnd();
    p->slot = (u8) rnd();
    p->spawn_pos = (s8) rnd();
    p->spawn_dir = (s8) rnd();
    p->sub_color = (u8) rnd();
    p->handicap = (s8) rnd();
    p->team = (u8) rnd();
    p->nametag = (u8) rnd();
    p->xB = (u8) rnd();
#define X(f) p->f = rnd();
    PID_FIELDS(X)
#undef X
    p->cpu_kind = (u8) rnd();
    p->cpu_level = (u8) rnd();
    p->damage = (u16) rnd();
    p->damage1 = (u16) rnd();
    p->hp = (u16) rnd();
    p->attack_ratio = rf();
    p->defense_ratio = rf();
    p->model_scale = rf();
}
static ou64 pid_fp(const PlayerInitData* p)
{
    ou64 h = 1469598103934665603ull;
#define M(v) h = h * 1000003ull + (ou64) (v);
    M((u8) p->ckind) M(p->slot_type) M((u8) p->stocks) M(p->color) M(p->slot) M((u8) p->spawn_pos) M((u8) p->spawn_dir)
    M(p->sub_color) M((u8) p->handicap) M(p->team) M(p->nametag) M(p->xB)
#define X(f) M(p->f)
    PID_FIELDS(X)
#undef X
    M(p->cpu_kind) M(p->cpu_level) M(p->damage) M(p->damage1) M(p->hp) M(fbits(p->attack_ratio)) M(fbits(p->defense_ratio)) M(fbits(p->model_scale))
#undef M
    return h;
}
static void pid_print(PlayerInitData* p) { printf(" pid %016llx", pid_fp(p)); }

#include "tu_gm_181A_stubs.inc"

#include <stdarg.h>
static RegClearSpawnEntry* mktable(void)
{
    RegClearSpawnEntry* t = (RegClearSpawnEntry*) AZ(sizeof(RegClearSpawnEntry) * 102);
    int i;
    int term = rnd() % 5 == 0 ? 200 : 3 + (int) (rnd() % 95);
    for (i = 0; i < 102; i++) {
        t[i].x0 = i == term ? 0x3E7 : (int) (rnd() % 100);
        t[i].x4 = (u8) (rnd() % 7);
        t[i].x5 = (u8) rnd();
        t[i].x6 = (u8) rnd();
        t[i].x7 = (u8) rnd();
        t[i].x8 = rf();
        t[i].xC = rf();
    }
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
        *(void**) dst = mktable();
        nx = va_arg(ap, unsigned long long);
        dst = (void*) (nx & 0xffffffffull ? (void*) nx : 0);
    }
    va_end(ap);
    printf("\n");
    return (HSD_Archive*) AZ(sizeof(HSD_Archive));
}

static void state(void)
{
    lbl_80472ED8_t* d = &lbl_80472ED8;
    dump("d0", &d->x0, sizeof(d->x0));
    dump("d4", &d->x4, sizeof(d->x4));
    dump("d8", &d->x8, sizeof(d->x8));
    printf("pidC %016llx\n", pid_fp(&d->xC));
    dump("d30", d->pad_30, sizeof(d->pad_30));
    dump("d54", d->x54, sizeof(d->x54));
    dump("d6a4", &d->x6A4, sizeof(d->x6A4));
    dump("d6a8", &d->x6A8, sizeof(d->x6A8));
    dump("d6ac", &d->x6AC, sizeof(d->x6AC));
    dump("d6b0", &d->x6B0, sizeof(d->x6B0));
    dump("d6b4", &d->x6B4, sizeof(d->x6B4));
    dump("d6b8", &d->x6B8, sizeof(d->x6B8));
    DUMP("rec", lbl_80473594);
    DUMP("blocks", lbl_803D8D08);
    printf("mm %d %u %d\n", (int) lbl_804D65E0.x0_0, (unsigned) lbl_804D65E0.x2, lbl_804D65E0.x4);
    DUMP("e8", lbl_804D65E8);
    DUMPARENA();
    state_end();
}
static void seed_state(void)
{
    int i, k;
    memset(&lbl_80472ED8, 0, sizeof(lbl_80472ED8));
    lbl_80472ED8.x0 = (int) (rnd() % 8);
    lbl_80472ED8.x4 = (int) (rnd() % 8);
    lbl_80472ED8.x8 = rnd() % 4 == 0 ? 0x5A + (int) (rnd() % 3) : (int) (rnd() % 0x70);
    pid_randomize(&lbl_80472ED8.xC);
    for (i = 0; i < 101; i++) {
        unsigned int r = rnd() % 8;
        lbl_80472ED8.x54[i].x0 = r == 0 ? -2 : r == 1 ? -1 : r == 2 ? 0x3E7 : (int) (rnd() % 101);
        lbl_80472ED8.x54[i].x4 = (u8) (rnd() % 7);
        lbl_80472ED8.x54[i].x5 = (u8) rnd();
        lbl_80472ED8.x54[i].x6 = (u8) rnd();
        lbl_80472ED8.x54[i].x7 = (u8) rnd();
        lbl_80472ED8.x54[i].x8 = rf();
        lbl_80472ED8.x54[i].xC = rf();
    }
    lbl_80473594.x0 = (u8) (rnd() % 3);
    lbl_80473594.x2 = (u16) (rnd() % 400);
    lbl_80473594.x4 = (int) (rnd() % 2000);
    {
        static const int modes[] = { 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0 };
        lbl_80473594.x8 = modes[rnd() % 7];
    }
    lbl_80473594.xC = (int) (rnd() % 27);
    lbl_80473594.x10 = (s8) (rnd() % 4);
    lbl_80473594.x11 = (u8) (rnd() % 100);
    for (k = 0; k < 6; k++) {
        for (i = 0; i < 27; i++) lbl_803D8D08[k].scores[i] = rnd() % 2000;
        for (i = 0; i < 28; i++) lbl_803D8D08[k].icons[i] = (u8) (rnd() % 4);
        for (i = 0; i < 28; i++) lbl_803D8D08[k].times[i] = (u16) (rnd() % 400);
    }
    lbl_804D65E8 = (u8) (rnd() % 4);
}

int main(void)
{
    int c;
    arena_init();
    SIZE(lbl_80472ED8_t);
    SIZE(RegClearSpawnEntry);
    SIZE(RegClearRecordState);
    SIZE(RecordBlock);
    SIZE(PlayerInitData);
    SIZE(UnkMultimanData);
    reg("rec", &lbl_80473594, sizeof(lbl_80473594));
    reg("blocks", lbl_803D8D08, sizeof(lbl_803D8D08));
    reg("d", &lbl_80472ED8, sizeof(lbl_80472ED8));
    fnreg("fn_80181E18", (void*) fn_80181E18);
    fnreg("fn_80182F40", (void*) fn_80182F40);
    DUMP("blocks0", lbl_803D8D08);
    state_end();

    for (c = 0; c < 8; c++) { /* getters */
        CASE(c);
        rseed(100 + c);
        arena_reset();
        seed_state();
        printf("r %d %u %d\n", gm_80181A14(), (unsigned) gm_80181A24(), (int) gm_80181A34());
        state();
    }
    for (c = 0; c < 300; c++) { /* record setters */
        CASE(1000 + c);
        rseed(2000 + c);
        arena_reset();
        seed_state();
        { int a = (int) (rnd() % 40); int b = 0x20 + (int) (rnd() % 8); bool v = (bool) (rnd() % 2); gm_80181A44(a, b, v); }
        { int a = (int) (rnd() % 40); int b = 0x20 + (int) (rnd() % 8); u16 v = (u16) rnd(); gm_80181AC8(a, b, v); }
        { int a = (int) (rnd() % 40); int b = 0x20 + (int) (rnd() % 8); s32 v = (s32) rnd(); gm_80181B64(a, b, v); }
        state();
    }
    for (c = 0; c < 200; c++) { /* fn_80181BFC */
        int out_i = -1;
        int r;
        CASE(2000 + c);
        rseed(3000 + c);
        arena_reset();
        seed_state();
        r = fn_80181BFC(c % 2 ? &out_i : 0);
        printf("r %d %d\n", r, out_i);
        state();
    }
    for (c = 0; c < 400; c++) { /* fn_80181C80 */
        CASE(3000 + c);
        rseed(4000 + c);
        arena_reset();
        seed_state();
        fn_80181C80((int) (rnd() % 101));
        state();
    }
    for (c = 0; c < 1500; c++) { /* fn_80181E18 */
        CASE(4000 + c);
        rseed(5000 + c);
        arena_reset();
        seed_state();
        fn_80181E18();
        state();
    }
    for (c = 0; c < 60; c++) { /* gm_80182174 */
        CASE(7000 + c);
        rseed(8000 + c);
        arena_reset();
        seed_state();
        gm_80182174();
        state();
    }
    for (c = 0; c < 14; c++) { /* gm_IsMultimanSmashMode */
        CASE(8000 + c);
        rseed(9000 + c);
        arena_reset();
        seed_state();
        printf("r %d\n", (int) gm_IsMultimanSmashMode());
        state();
    }
    for (c = 0; c < 10; c++) { /* gm_80182554 */
        CASE(9000 + c);
        rseed(10000 + c);
        arena_reset();
        seed_state();
        { int a = (int) (rnd() % 27); int b = 0x21 + (int) (rnd() % 6); gm_80182554(a, b); }
        state();
    }
    for (c = 0; c < 2000; c++) { /* gm_80182578 */
        CASE(10000 + c);
        rseed(11000 + c);
        arena_reset();
        seed_state();
        gm_80182578();
        state();
    }
    for (c = 0; c < 800; c++) { /* fn_80182B5C */
        CASE(13000 + c);
        rseed(14000 + c);
        arena_reset();
        seed_state();
        fn_80182B5C();
        state();
    }
    for (c = 0; c < 200; c++) { /* gm_80182DF0 */
        UnkMultimanData* r;
        CASE(15000 + c);
        rseed(16000 + c);
        arena_reset();
        seed_state();
        lbl_804D65E0.x0_0 = (u16) (rnd() % 2);
        lbl_804D65E0.x2 = (u16) rnd();
        lbl_804D65E0.x4 = (int) rnd();
        { int a = (int) (rnd() % 40); int b = 0x20 + (int) (rnd() % 8); r = gm_80182DF0(a, b); }
        printf("r %d\n", r == &lbl_804D65E0);
        state();
    }
    for (c = 0; c < 2000; c++) { /* fn_80182F40 */
        CASE(16000 + c);
        rseed(17000 + c);
        arena_reset();
        seed_state();
        fn_80182F40(0);
        state();
    }
    for (c = 0; c < 2; c++) { /* gm_80183218 */
        CASE(19000 + c);
        rseed(20000 + c);
        arena_reset();
        seed_state();
        gm_80183218();
        state();
    }
    return 0;
}
