/* TU: src/melee/if/textdraw.c */
/* C oracle for src/tu/textdraw.rs: the whole TU included verbatim; every callee out of it is a
 * recording stub. Each line: op case ret state-hash trace-hash. */
#include TU_SRC
#include "mtu_common.h"
static HSD_GObj GO[4], GOBJ;
static HSD_CObj COBJA, COBJB, COBJC;
static char BUFS[32][192];
static char STR[128];
static DevText* HEAD;
#define P (devtext_pool.entries)
#include "textdraw_gen.h"

static void setup(void)
{
    int i, n, nd, nf, perm[32];
    DevText *prev;
    zero(&devtext_pool, sizeof devtext_pool);
    zero(GO, sizeof GO); zero(&GOBJ, sizeof GOBJ);
    for (i = 0; i < 4; i++) fill_HSD_GObj(&GO[i]);
    fill_HSD_GObj(&GOBJ);
    for (i = 0; i < 32; i++) {
        int j;
        fill_DevText(&P[i]);
        P[i].w = 1 + rnd() % 10;
        P[i].h = 1 + rnd() % 6;
        P[i].id = (s8) ((int) (rnd() % 7) - 2);
        P[i].unk = rnd() % 24;
        P[i].buf = BUFS[i];
        for (j = 0; j < 192; j++) BUFS[i][j] = (rnd() % 4 == 0) ? 0 : (char) rnd();
        P[i].prev = 0;
        P[i].next = 0;
    }
    for (i = 0; i < 128; i++) STR[i] = (rnd() % 9 == 0) ? 0 : (char) (rnd() % 94 + 33);
    STR[127] = 0;
    for (i = 0; i < 32; i++) perm[i] = i;
    for (i = 31; i > 0; i--) { int j = (int) (rnd() % (u32) (i + 1)); int t = perm[i]; perm[i] = perm[j]; perm[j] = t; }
    nd = (int) (rnd() % 7);
    nf = (int) (rnd() % 10);
    devtext_drawlist = 0;
    devtext_poolhead = 0;
    prev = 0;
    for (i = 0; i < nd; i++) {
        DevText* t = &P[perm[i]];
        t->prev = prev;
        if (prev) prev->next = t; else devtext_drawlist = t;
        prev = t;
    }
    prev = 0;
    for (i = 0; i < nf; i++) {
        DevText* t = &P[perm[nd + i]];
        t->prev = prev;
        if (prev) prev->next = t; else devtext_poolhead = t;
        prev = t;
    }
    HEAD = (nd && (rnd() & 1)) ? &P[perm[rnd() % (u32) nd]] : devtext_drawlist;
    n = (int) (rnd() % 4);
    devtext_cobj = (n == 0) ? 0 : (n == 1 ? &COBJC : 0);
    devtext_gobj = (rnd() & 1) ? &GO[rnd() % 4] : 0;
    devtext_setup_classifier = (int) fv();
    devtext_setup_p_link = (int) fv();
    devtext_setup_priority = (int) fv();
    devtext_setup_gx_link = (int) fv();
    devtext_setup_render_priority = (int) fv();
    HSD_GObj_CameraKind = (u8) (rnd() % 5);
}

static void hash_all(void)
{
    int i;
    for (i = 0; i < 32; i++) hash_DevText(&P[i]);
    for (i = 0; i < 4; i++) hash_HSD_GObj(&GO[i]);
    hash_HSD_GObj(&GOBJ);
    HM(PI(HEAD)); HM(PI(devtext_drawlist)); HM(PI(devtext_poolhead)); HM(PI(devtext_cobj)); HM(PI(devtext_gobj));
    HM((u32) devtext_setup_classifier); HM((u32) devtext_setup_p_link); HM((u32) devtext_setup_priority);
    HM((u32) devtext_setup_gx_link); HM((u32) devtext_setup_render_priority);
    HM(RH(STR, sizeof STR));
}

int main(void)
{
    int op, k, i;
    for (i = 0; i < 32; i++) preg(&P[i], sizeof(DevText));
    for (i = 0; i < 4; i++) preg(&GO[i], sizeof(HSD_GObj));
    preg(&GOBJ, sizeof GOBJ); preg(&COBJA, sizeof COBJA); preg(&COBJB, sizeof COBJB); preg(&COBJC, sizeof COBJC);
    freg_all();
    for (op = 0; op < 14; op++) {
        for (k = 0; k < 1500; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup();
            switch (op) {
            case 0: ret = (u32) DevText_StrLen((rnd() % 8 == 0) ? 0 : STR + rnd() % 8); break;
            case 1: {
                int num = (rnd() % 16 == 0) ? (int) 0x80000000u : (int) fv();
                if (rnd() & 1) num = -num;
                DevText_NumToStr(num, STR + rnd() % 4);
                break;
            }
            case 2: DevText_InitPool(); break;
            case 3: if (HEAD) DevText_Remove(&HEAD); break;
            case 4: DevText_Unlink((rnd() % 8 == 0) ? 0 : &P[rnd() % 32]); break;
            case 5: ret = (u32) DevText_IsListed((rnd() % 8 == 0) ? 0 : &P[rnd() % 32]); break;
            case 6: DevText_SetupCObj(); break;
            case 7: DevText_Draw(&P[rnd() % 32]); break;
            case 8: {
                int pass = (rnd() & 1) ? 2 : (int) fv();
                DevText_DrawAll(&GOBJ, pass);
                break;
            }
            case 9: {
                int a = (int) fv(), b = (int) fv(), c = (int) fv(), d = (int) (fv() & 63);
                DevText_CreateCObj(a, b, c, d, (u8) fv());
                break;
            }
            case 10: {
                int a = (int) fv(), b = (int) fv(), c = (int) fv(), d = (int) (fv() & 63), e = (int) fv();
                ret = PI(DevText_Setup(a, b, c, d, e, (u8) fv()));
                break;
            }
            case 11: DevText_AddToList(&devtext_drawlist, &P[rnd() % 32]); break;
            case 12: DevText_Show(&GOBJ, &P[rnd() % 32]); break;
            case 13: ret = PI(DevText_GetGObj()); break;
            }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
