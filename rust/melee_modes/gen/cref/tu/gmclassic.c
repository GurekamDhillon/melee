/* TU: src/melee/gm/gmclassic.c */
/* C oracle for src/tu/gmclassic.rs: the whole TU included verbatim (two declarations patched so the
 * console's adjacent intro / state buffers are one object, see specs.py); every callee out of it is
 * a recording stub. Each line: op case ret state-hash trace-hash. */
#include TU_SRC
#include "mtu_common.h"
static GameModeState ST;
static UnkAllstarData AD;
static gmm_x0_528_t G528;
static PreloadedGameModeState PCS;
static CSSData CSS;
static StartMeleeData SMD;
static MatchExitInfo MEI;
static struct gmm_retval_ED98 R98;
static struct gmm_retval_EDB0 RB0;
static struct gmm_retval_EDBC RBC;
static u32 TIMEV;
static s32 BESTV;
static u8 BUF[0x200];
static int UNL;
static gmClassic_803DDEC8Data INIT;
#include "gmclassic_gen.h"

#define OI(p) ((gmClassicOrderIndex*) (p))

static void hash_all(void)
{
    int i;
    hash_UnkAllstarData(&AD);
    hash_gmm_x0_528_t(&G528);
    hash_PreloadedGameModeState(&PCS);
    hash_CSSData(&CSS);
    hash_StartMeleeData(&SMD);
    hash_MatchExitInfo(&MEI);
    hash_DebugGameOverData(&gmClassic_80470850);
    hash_gmClassicRuntimeData(&gmClassicRuntimeBuf);
    hash_gmClassic_803DDEC8Data(&gmClassic_803DDEC8);
    hash_gmm_retval_ED98(&R98);
    hash_gmm_retval_EDB0(&RB0);
    hash_gmm_retval_EDBC(&RBC);
    HM(TIMEV);
    HM((u32) BESTV);
    for (i = 0; i < 0x200; i++) HM(BUF[i]);
    HM(PI(gmClassic_804D68D0));
}

/* a valid shuffled order in the four order tables, as OnLoad leaves them */
static void make_orders(void)
{
    gmClassic_InitMatchupOrder(gmClassic_803DDEC8.x2B0, OI(&gm_804908A0[0x60]), OI(&gmClassicRuntimeBuf), 0x80);
    gmClassic_InitMatchupOrder(gmClassic_803DDEC8.x26C, OI(&gm_804908A0[0x54]), OI(&gmClassicRuntimeBuf), 0x74);
    gmClassic_InitMatchupOrder(gmClassic_803DDEC8.x1B8, OI(&gm_804908A0[0x34]), OI(&gmClassicRuntimeBuf), 0x54);
    gmClassic_InitMatchupOrder(gmClassic_803DDEC8.x0CC, OI(&gm_804908A0[0x0C]), OI(&gmClassicRuntimeBuf), 0x2C);
}

int main(void)
{
    int op, k, i;
    INIT = gmClassic_803DDEC8;
    preg(&ST, sizeof ST);
    preg(&AD, sizeof AD);
    preg(&G528, sizeof G528);
    preg(&PCS, sizeof PCS);
    preg(&CSS, sizeof CSS);
    preg(&SMD, sizeof SMD);
    preg(&MEI, sizeof MEI);
    preg(&R98, sizeof R98);
    preg(&RB0, sizeof RB0);
    preg(&RBC, sizeof RBC);
    preg(&TIMEV, sizeof TIMEV);
    preg(&BESTV, sizeof BESTV);
    preg(BUF, sizeof BUF);
    preg(&gmClassicRuntimeBuf, sizeof gmClassicRuntimeBuf);
    preg(&gmClassic_803DDEC8, sizeof gmClassic_803DDEC8);
    preg(&gm_804D4318, sizeof gm_804D4318);
    preg(&gm_804D4320, sizeof gm_804D4320);
    preg(&gm_804D4328, sizeof gm_804D4328);
    preg(&gmClassic_804D68D0, sizeof gmClassic_804D68D0);
    preg(gm_Mode_Classic_States, sizeof gm_Mode_Classic_States);
    preg(gmClassic_80470708, sizeof gmClassic_80470708);
    preg(&gmClassic_80470850, sizeof gmClassic_80470850);
    preg(&gmClassic_8047086C, sizeof gmClassic_8047086C);
    preg(&gmClassic_80472AF8, sizeof gmClassic_80472AF8);
    freg_all();
    for (op = 0; op < 17; op++) {
        for (k = 0; k < 500; k++) {
            u32 ret = 0;
            tu_case(op, k);
            zero(&ST, sizeof ST);
            gmClassic_803DDEC8 = INIT;
            fill_UnkAllstarData(&AD);
            fill_gmm_x0_528_t(&G528);
            fill_PreloadedGameModeState(&PCS);
            fill_CSSData(&CSS);
            fill_StartMeleeData(&SMD);
            fill_MatchExitInfo(&MEI);
            fill_DebugGameOverData(&gmClassic_80470850);
            fill_gmClassicRuntimeData(&gmClassicRuntimeBuf);
            fill_gmm_retval_ED98(&R98);
            fill_gmm_retval_EDB0(&RB0);
            fill_gmm_retval_EDBC(&RBC);
            TIMEV = fv();
            BESTV = (s32) fv();
            for (i = 0; i < 0x200; i++) BUF[i] = (u8) fv();
            gmClassic_804D68D0 = 0;
            UNL = (op == 6 || op == 8 || op == 10 || op == 13);
            ST.id = (u8) fv();
            /* the scene rows' matchups, as OnLoad's picker leaves them */
            for (i = 0; i < 11; i++) gmClassic_803DDEC8.x00[i].xC = &gmClassic_803DDEC8.x0CC[fv() % 39];
            switch (op) {
            case 0: ST.info.enter_data = &gmClassicRuntimeBuf.intro; gmClassic_801B3500(&ST); break;
            case 1: ST.info.enter_data = &SMD; gmClassic_801B3A34(&ST); break;
            case 2: ST.info.exit_data = &MEI; gmClassic_801B3B40(&ST); break;
            case 3: ST.info.enter_data = &gmClassic_80470850; gmClassic_801B3D44(&ST); break;
            case 4: ST.info.exit_data = &gmClassic_80470850; gmClassic_801B3D84(&ST); break;
            case 5: ST.info.enter_data = &CSS; gmClassic_801B3DD8(&ST); break;
            case 6:
                gmClassic_803DDEC8 = INIT;
                make_orders();
                ST.info.exit_data = &CSS;
                gmClassic_801B3E44(&ST);
                break;
            case 7: gmClassic_801B3F18(&ST); break;
            case 8: gm_Mode_Classic_OnLoad(); break;
            case 9: gm_Mode_Classic_OnInit(); break;
            case 10:
                gmClassic_803DDEC8 = INIT;
                make_orders();
                ret = PI(gmClassic_801B2D54(gmClassic_803DDEC8.x00));
                break;
            case 11: {
                int pool = (int) (fv() % 5);
                u8 order[64];
                gmClassicMatchup* tbl = pool == 0 ? gmClassic_803DDEC8.x0CC : pool == 1 ? gmClassic_803DDEC8.x1B8 :
                                        pool == 2 ? gmClassic_803DDEC8.x26C : pool == 3 ? gmClassic_803DDEC8.x2B0 :
                                        gmClassic_803DDEC8.x0C0;
                for (i = 0; i < 64; i++) order[i] = (u8) (fv() % 2 + (u32) i % 9);
                for (i = 0; i < 11; i++) gmClassic_803DDEC8.x00[i].xC = (fv() & 1) ? 0 : &gmClassic_803DDEC8.x0CC[fv() % 39];
                ret = PI(gmClassic_801B2BA4(tbl, order, gmClassic_803DDEC8.x00));
                break;
            }
            case 12: {
                int pool = (int) (fv() % 4);
                gmClassicMatchup* tbl = pool == 0 ? gmClassic_803DDEC8.x0CC : pool == 1 ? gmClassic_803DDEC8.x1B8 :
                                        pool == 2 ? gmClassic_803DDEC8.x26C : gmClassic_803DDEC8.x2B0;
                gmClassic_InitMatchupOrder(tbl, OI(&BUF[0x10]), OI(&BUF[0x40]), (int) (fv() % 0x80));
                break;
            }
            case 13: ret = (u32) gm_OnePSceneDirect(&G528, &AD.x0, (int) fv()); break;
            case 14: {
                int pool = (int) (fv() % 5);
                gmClassicMatchup* tbl = pool == 0 ? gmClassic_803DDEC8.x0CC : pool == 1 ? gmClassic_803DDEC8.x1B8 :
                                        pool == 2 ? gmClassic_803DDEC8.x26C : pool == 3 ? gmClassic_803DDEC8.x2B0 :
                                        gmClassic_803DDEC8.x0C0;
                ret = (u32) gmClassic_GetMatchupCount(tbl);
                break;
            }
            case 15: ret = (u32) gmClassic_GetStKind(&gmClassic_803DDEC8.x00[fv() % 11], &AD); break;
            case 16: for (i = 0; i < 26; i++) hash_GameModeState(&gm_Mode_Classic_States[i]); break;
            }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
