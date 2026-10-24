extern int atoi(const char*);
extern char* getenv(const char*);
/* TU: src/melee/ty/toy.c */
/* C oracle for the Rust port of Toy_80304B94 / Toy_80304CC8 / _Toy_80304D30 / Toy_80311960 (+ the callees that run
 * for real in both builds: un_80304470, Toy_803048C0, Toy_80304B0C, Toy_80305918, Toy_803060BC). The whole TU is
 * included verbatim; every callee out of it is a recording stub (gen_tu.py: toy_gen.h). Hand-written driver
 * (the generated drivers are for item worlds); mirrored by rust/melee_items/tests/tu_toy.rs.
 * Each line: op case ret state-hash trace-hash. */
#include <Runtime/platform.h>

#include TU_SRC
#include "mtu_common.h"

#define NT 293
static u16 SAVEF[NT];
static u16 CATF;
static s16 TCNT;
static s16 SKIP[16];
static struct TrophyData TBL_JP[NT + 2], TBL_US[NT + 2];
/* variadic callees: linked only (no op reaches them; the Rust ext hooks for them stay unset and would panic) */
HSD_Archive* lbArchive_LoadSymbols(const char* f, void* s, ...) { (void) f; (void) s; T(0xDEAD0001u); return 0; }
HSD_Archive* lbArchive_80016DBC(const char* f, void* s, ...) { (void) f; (void) s; T(0xDEAD0002u); return 0; }
int sprintf(char* o, const char* f, ...) { (void) o; (void) f; T(0xDEAD0003u); return 0; }
void DevText_Printf(DevText* d, char* f, ...) { (void) d; (void) f; T(0xDEAD0004u); }
#include "toy_gen.h"

/* real per-category totals (Toy_80304B94 for the US language) */
static const int CATN[9] = { 0x1B, 0x7E, 0x48, 0xC, 0x17, 0x10, 0xA, 0x4, 0x2 };

static void fill_td(struct TrophyData* t, int id, int cat)
{
    t->id = id;
    t->x04 = rnd() % 9;
    t->x08 = rw();
    t->x0C = rw();
    t->x10 = rw();
    t->x14 = rw();
    t->x18 = rw();
    t->x1C = rw();
    t->x20 = (s8) cat;
    t->x21 = (s8) (rnd() & 7);
    t->x22 = 0;
    t->x23 = 0;
}

static void setup_stem(void)
{
    int i, c, n, full, lo, cat;
    full = rnd() % 3 == 0;
    for (i = 0; i < 302; i++) Toy_804A284C[i] = (rnd() & 3) == 0 ? 0 : (u16) rnd();
    for (i = 0; i < NT; i++) {
        SAVEF[i] = (rnd() & 3) == 0 ? 0 : (u16) rnd();
        if (full) {
            if ((SAVEF[i] & 0xFF) == 0) SAVEF[i] |= 1;
            if ((Toy_804A284C[5 + i] & 0xFF) == 0) Toy_804A284C[5 + i] |= 1;
        }
    }
    CATF = (u16) rnd();
    TCNT = (s16) rnd();
    n = rnd() % 7;
    for (i = 0; i < n; i++) SKIP[i] = (s16) (rnd() % NT);
    SKIP[n] = -1;
    _Toy_sbss_804D6EB4 = SKIP;
    zero(TBL_US, sizeof TBL_US);
    zero(TBL_JP, sizeof TBL_JP);
    c = 0;
    lo = 0;
    for (i = 0; i < NT; i++) {
        while (c < 8 && lo >= CATN[c]) { lo = 0; c++; }
        cat = c;
        lo++;
        if ((rnd() & 7) == 0) cat = rnd() % 9;
        fill_td(&TBL_US[i], i, cat);
    }
    TBL_US[NT].id = -1;
    n = rnd() % 60;
    for (i = 0; i < n; i++) fill_td(&TBL_JP[i], rnd() % NT, rnd() % 9);
    TBL_JP[n].id = -1;
    _Toy_sbss_804D6EC0 = TBL_JP;
    _Toy_sbss_804D6EC4 = TBL_US;
}

static void hash_stem(void)
{
    int i;
    hashg_all();
    for (i = 0; i < 302; i++) HM(Toy_804A284C[i]);
    for (i = 0; i < NT; i++) HM(SAVEF[i]);
    HM(CATF);
    HM((u16) TCNT);
    HM(PI(_Toy_sbss_804D6EB4));
    HM(PI(_Toy_sbss_804D6EC0));
    HM(PI(_Toy_sbss_804D6EC4));
}

int main(int argc, char** argv)
{
    int op, k, only = -1;
    if (argc > 1) only = atoi(argv[1]);
    preg(Toy_804A284C, sizeof Toy_804A284C);
    preg(SAVEF, sizeof SAVEF);
    preg(&CATF, sizeof CATF);
    preg(&TCNT, sizeof TCNT);
    preg(SKIP, sizeof SKIP);
    preg(TBL_JP, sizeof TBL_JP);
    preg(TBL_US, sizeof TBL_US);
    freg_all();
    for (op = 0; op < 6; op++) {
        int n = 600;
        if (only >= 0 && op != only) continue;
        for (k = 0; k < n; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup_stem();
            if (getenv("MTU_PROG")) { printf("P %d %d%c", op, k, 10); fflush(0); }
            switch (op) {
            case 0: { int a0 = (int) (rnd() % 13) - 2; ret = (u32) Toy_80304B94(a0); } break;
            case 1: ret = (u32) _Toy_80304D30(); break;
            case 2: Toy_80311960(); break;
            case 3: { int a0 = (int) (rnd() % NT); ret = (u32) Toy_80304CC8(a0); } break;
            case 4: ret = (u32) un_80304470(); break;
            case 5: { int a0 = (int) (rnd() % (NT + 4)) - 2; int a1 = (int) (rnd() % 9); ret = B(Toy_803060BC(a0, a1)); } break;
            }
            hash_stem();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
