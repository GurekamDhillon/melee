/* TU: src/melee/gm/gmmain_lib.c */
/* C oracle for src/tu/gmmain_lib.rs (the functions listed in the ops below): the whole TU included
 * verbatim; every callee out of it is a recording stub. Each line: op case ret state-hash trace-hash. */
#include TU_SRC
#include "mtu_common.h"
static char NAMES[4][8];
static void zero(void* p, unsigned n);
void memzero(void* p, ssize_t n) { zero(p, (unsigned) n); }
#include "gmmain_lib_gen.h"

#define NOPS 6
static void fill_all(void)
{
    int i, j;
    fill_gmm_x0(&gmMainLib_8045A6C0);
    fill_GameRules(&gmMainLib_DefaultGameRules);
    fill_GamePrefs(&gmMainLib_DefaultGamePrefs);
    fill_gmMainLib_8046B0F0_t(&gmMainLib_8046B0F0);
    for (i = 0; i < 4; i++) {
        for (j = 0; j < 7; j++) NAMES[i][j] = (char) (1 + fv() % 100);
        NAMES[i][fv() % 7] = 0;
        NAMES[i][7] = 0;
    }
}
static void hash_all(void)
{
    hash_gmm_x0(&gmMainLib_8045A6C0);
    hash_GameRules(&gmMainLib_DefaultGameRules);
    hash_GamePrefs(&gmMainLib_DefaultGamePrefs);
    hash_gmMainLib_8046B0F0_t(&gmMainLib_8046B0F0);
}

int main(void)
{
    int op, k;
    preg(&gmMainLib_8045A6C0, sizeof gmMainLib_8045A6C0);
    preg(&gmMainLib_DefaultGameRules, sizeof gmMainLib_DefaultGameRules);
    preg(&gmMainLib_DefaultGamePrefs, sizeof gmMainLib_DefaultGamePrefs);
    preg(&gmMainLib_8046B0F0, sizeof gmMainLib_8046B0F0);
    preg(NAMES, sizeof NAMES);
    /* no function pointers are compared in this driver */
    for (op = 0; op < NOPS; op++) {
        for (k = 0; k < 800; k++) {
            u32 ret = 0;
            tu_case(op, k);
            fill_all();
            switch (op) {
            case 0: ret = PI(gmMainLib_GetTrophyFlags()); break;
            case 1: ret = PI(gmMainLib_GetTrophyCategoryFlags()); break;
            case 2: ret = PI(gmMainLib_GetTrophyCount()); break;
            case 3: gmMainLib_MexApplyTournamentDefaults(); break;
            case 4: gmMainLib_8015FCC0(); break;
            default: gmMainLib_8015F600(1 + (int) (k % 8), (int) ((k / 8) & 1)); break;
            }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
