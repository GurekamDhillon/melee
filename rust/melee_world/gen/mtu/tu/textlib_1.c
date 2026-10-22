/* TU: src/melee/if/textlib_1.c */
/* C oracle for src/tu/textlib_1.rs: the whole TU included verbatim; every callee out of it is a
 * recording stub. Each line: op case ret state-hash trace-hash. */
#include TU_SRC
#include "mtu_common.h"
#define NM 3
static DevText DT[4];
static HSD_GObj GOBJ0, GOBJ1;
static HSD_GObjProc PROC0;
static char MEM[256];
static struct un_80304138_objalloc_t OBJS[4];
static struct un_80304138_objalloc_t MENU[NM];
static struct un_80304138_objalloc_t_x8 ENT[NM][8];
static char* STRS[3];
static char SBUF[4][16];
static int INTV[NM][8];
static unsigned long long VALS[NM][8];
static int NULLOK;
static int NROWS[NM];
#include "textlib_1_gen.h"

static int cbA(enum soundtest_callback_arg0 a) { T(7001); T((u32) a); T(PI(un_804D6E48)); return (int) (fv() % 3); }
static int cbB(enum soundtest_callback_arg0 a) { T(7002); T((u32) a); T(PI(un_804D6E48)); return (int) (fv() & 1); }
static int cbC(enum soundtest_callback_arg0 a) { T(7003); T((u32) a); un_804D6E4C = (u8) fv(); return (int) (fv() % 3) - 1; }
static soundtest_callback pickcb(void)
{
    switch (rnd() % 4) {
    case 0: return cbA;
    case 1: return cbB;
    case 2: return cbC;
    default: return 0;
    }
}

static void setup(void)
{
    int m, i, perm[NM];
    zero(DT, sizeof DT); zero(&GOBJ0, sizeof GOBJ0); zero(&GOBJ1, sizeof GOBJ1); zero(&PROC0, sizeof PROC0);
    zero(OBJS, sizeof OBJS); zero(MENU, sizeof MENU); zero(ENT, sizeof ENT);
    zero(INTV, sizeof INTV); zero(VALS, sizeof VALS); zero(MEM, sizeof MEM);
    fill_HSD_GObj(&GOBJ0); fill_HSD_GObj(&GOBJ1);
    for (i = 0; i < 4; i++) { fill_DevText(&DT[i]); DT[i].prev = 0; DT[i].next = 0; DT[i].buf = MEM; }
    for (i = 0; i < 4; i++) { fill_un_80304138_objalloc_t(&OBJS[i]); OBJS[i].x4 = 0; OBJS[i].x8 = 0; OBJS[i].xC = 0; OBJS[i].x10 = 0; OBJS[i].x14 = 0; OBJS[i].prev = 0; OBJS[i].next = 0; }
    for (i = 0; i < 4; i++) {
        int j;
        for (j = 0; j < 15; j++) SBUF[i][j] = (char) (33 + rnd() % 90);
        SBUF[i][15] = 0;
    }
    for (i = 0; i < 3; i++) STRS[i] = (rnd() % 5 == 0) ? 0 : SBUF[i];
    for (m = 0; m < NM; m++) {
        int n = 1 + (int) (rnd() % 5);
        NROWS[m] = n;
        for (i = 0; i < 8; i++) {
            struct un_80304138_objalloc_t_x8* e = &ENT[m][i];
            fill_un_80304138_objalloc_t_x8(e);
            e->x0 = (i == n) ? 9 : (int) (rnd() % 9);
            if (i > n) e->x0 = (int) (rnd() % 10);
            e->x4 = pickcb();
            e->x8 = (rnd() % 6 == 0) ? 0 : SBUF[rnd() % 4];
            e->xC = STRS;
            e->x14 = rg();
            e->x18 = rg() * 10.0f;
            e->x1C = rg() * 10.0f;
            INTV[m][i] = (int) fv();
            VALS[m][i] = fv64();
            e->x10 = &VALS[m][i];
            if (e->x0 == 2) {
                e->x14 = 0.0f;
                e->x18 = (float) (1 + rnd() % 3);
                INTV[m][i] = (int) (rnd() % (u32) e->x18);
                e->x10 = &INTV[m][i];
            } else if (e->x0 == 3) {
                e->x10 = &INTV[m][i];
            }
        }
        fill_un_80304138_objalloc_t(&MENU[m]);
        MENU[m].x0 = (u8) (rnd() % (u32) n);
        MENU[m].x8 = ENT[m];
        MENU[m].x4 = &DT[m];
        MENU[m].xC = pickcb();
        MENU[m].x10 = &GOBJ0;
        MENU[m].x14 = (rnd() & 1) ? &PROC0 : 0;
        MENU[m].prev = 0;
        MENU[m].next = 0;
        DT[m].h = (u8) n;
        DT[m].w = 1 + rnd() % 10;
    }
    for (i = 0; i < NM; i++) perm[i] = i;
    for (i = NM - 1; i > 0; i--) { int j = (int) (rnd() % (u32) (i + 1)); int t = perm[i]; perm[i] = perm[j]; perm[j] = t; }
    {
        int len = 1 + (int) (rnd() % NM);
        for (i = 0; i + 1 < len; i++) { MENU[perm[i]].prev = &MENU[perm[i + 1]]; MENU[perm[i + 1]].next = &MENU[perm[i]]; }
        un_804D6E40 = &MENU[perm[0]];
        un_804D6E44 = (rnd() & 1) ? &MENU[perm[rnd() % (u32) len]] : 0;
    }
    un_804D6E48 = (rnd() & 1) ? &ENT[0][rnd() % 4] : 0;
    un_804D6E4C = (u8) ((rnd() & 1) ? 0 : (rnd() % 10));
    for (i = 0; i < 4; i++) {
        static const u32 bits[7] = { 0x1000, 0x800, 0x400, 0x20, 0x40, 0x100, 0x200 };
        int j;
        u32 trig = 0;
        zero(&HSD_PadCopyStatus[i], sizeof HSD_PadCopyStatus[i]);
        for (j = 0; j < 7; j++) if (rnd() % 7 == 0) trig |= bits[j];
        HSD_PadCopyStatus[i].trigger = trig;
        HSD_PadCopyStatus[i].button = (rnd() & 1) ? (trig | 0x20) : trig;
        HSD_PadCopyStatus[i].stickX = (s8) ((int) (rnd() % 161) - 80);
        HSD_PadCopyStatus[i].stickY = (s8) ((int) (rnd() % 161) - 80);
        if (rnd() % 3 == 0) HSD_PadCopyStatus[i].stickX = 0;
        if (rnd() % 3 == 0) HSD_PadCopyStatus[i].stickY = 0;
    }
    NULLOK = 0;
}

static void hash_all(void)
{
    int m, i;
    for (i = 0; i < 4; i++) { hash_DevText(&DT[i]); hash_un_80304138_objalloc_t(&OBJS[i]); }
    hash_HSD_GObj(&GOBJ0); hash_HSD_GObj(&GOBJ1);
    for (m = 0; m < NM; m++) {
        hash_un_80304138_objalloc_t(&MENU[m]);
        for (i = 0; i < 8; i++) {
            hash_un_80304138_objalloc_t_x8(&ENT[m][i]);
            HM((u32) INTV[m][i]); HM((u32) VALS[m][i]); HM((u32) (VALS[m][i] >> 32));
        }
    }
    HM(PI(un_804D6E40)); HM(PI(un_804D6E44)); HM(PI(un_804D6E48)); HM(un_804D6E4C);
}

int main(void)
{
    int op, k, i, m;
    for (i = 0; i < 4; i++) preg(&DT[i], sizeof(DevText));
    preg(&GOBJ0, sizeof GOBJ0); preg(&GOBJ1, sizeof GOBJ1); preg(&PROC0, sizeof PROC0); preg(MEM, sizeof MEM);
    for (i = 0; i < 4; i++) preg(&OBJS[i], sizeof OBJS[i]);
    for (m = 0; m < NM; m++) preg(&MENU[m], sizeof MENU[m]);
    for (m = 0; m < NM; m++) preg(ENT[m], sizeof ENT[m]);
    preg(INTV, sizeof INTV); preg(VALS, sizeof VALS); preg(SBUF, sizeof SBUF); preg(&un_804A2688, sizeof un_804A2688);
    freg_all();
    freg((const void*) cbA); freg((const void*) cbB); freg((const void*) cbC);
    for (op = 0; op < 14; op++) {
        for (k = 0; k < 1500; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup();
            switch (op) {
            case 0: { int r = (int) (rnd() % 8); int code = (int) (rnd() % 8); ret = (u32) un_80302E00(&ENT[0][r], code); break; }
            case 1: ret = (u32) un_80302EA4(ENT[(rnd() % NM)]); break;
            case 2: un_80302FFC(&MENU[rnd() % NM]); break;
            case 3: ret = (u32) un_80303444(&MENU[rnd() % NM]); break;
            case 4: ret = (u32) un_80303720(&MENU[rnd() % NM]); break;
            case 5: ret = (u32) un_803039A4((u8) (rnd() % 4)); break;
            case 6: un_80303AC4(&MENU[rnd() % NM]); break;
            case 7: fn_80303EF4(&GOBJ0); break;
            case 8: {
                int a = (int) fv(), b = (int) fv(), c = (int) fv();
                int oi = (int) (rnd() % 4);
                int ei = (int) (rnd() % NM);
                un_80303FD4(&GOBJ1, &OBJS[oi], ENT[ei], a, b, c);
                break;
            }
            case 9: un_80304138(); break;
            case 10: {
                int a = (int) fv(), b = (int) fv(), c = (int) fv();
                ret = PI(un_80304168(ENT[rnd() % NM], a, b, c));
                break;
            }
            case 11: {
                int a = (int) fv(), b = (int) fv(), c = (int) fv();
                int mi = (int) (rnd() % NM);
                int ei = (int) (rnd() % NM);
                NULLOK = 1;
                ret = PI(un_80304210(&MENU[mi], ENT[ei], a, b, c));
                break;
            }
            case 12: un_80304334(&MENU[rnd() % NM]); break;
            case 13: un_80304344(&MENU[rnd() % NM]); break;
            }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
