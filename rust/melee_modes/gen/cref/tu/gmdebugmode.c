/* TU: src/melee/gm/gmdebugmode.c */
/* C oracle for src/tu/gmdebugmode.rs: the whole TU included verbatim; every callee out of it is a
 * recording stub. Each line: op case ret state-hash trace-hash. */
#include TU_SRC
#include "mtu_common.h"
static GameModeState ST;
#include "gmdebugmode_gen.h"

static void hash_all(void)
{
    hash_enterMenu0_Data(&menu_enter_data);
    hash_ResultsMatchInfo(&results0_data);
    hash_StartMeleeData(&vs_enter_data);
    hash_MatchExitInfo(&vs_exit_data);
    hash_DebugGameOverData(&game_over_data);
    hash_DebugResultsData(&results1_enter_data);
    hash_exitIntroData(&intro_easy_exit_data);
    HM(RH(&approach_data, sizeof approach_data));
    hash_un_80301C64_t(&intro_allstar_enter_data);
    hash_exitIntroData(&intro_allstar_exit_data);
    hash_DebugMemcardData(&memcard_enter_data);
    hash_DebugMemcardData(&memcard_exit_data);
    HM(RH(&intro_easy_enter_data, sizeof intro_easy_enter_data));
    HM(RH(&prize_enter_data, sizeof prize_enter_data));
}

int main(void)
{
    int op, k, i;
    preg(&ST, sizeof ST);
    preg(gm_Mode_Debug_States, sizeof gm_Mode_Debug_States);
    preg(&intro_easy_enter_data, sizeof intro_easy_enter_data);
    preg(&vs_enter_data, sizeof vs_enter_data);
    preg(&vs_exit_data, sizeof vs_exit_data);
    preg(&results0_data, sizeof results0_data);
    preg(&prize_enter_data, sizeof prize_enter_data);
    preg(&game_over_data, sizeof game_over_data);
    preg(&results1_enter_data, sizeof results1_enter_data);
    preg(&intro_easy_exit_data, sizeof intro_easy_exit_data);
    preg(&intro_allstar_enter_data, sizeof intro_allstar_enter_data);
    preg(&intro_allstar_exit_data, sizeof intro_allstar_exit_data);
    preg(&menu_enter_data, sizeof menu_enter_data);
    preg(&approach_data, sizeof approach_data);
    preg(&memcard_enter_data, sizeof memcard_enter_data);
    preg(&memcard_exit_data, sizeof memcard_exit_data);
    preg(un_803FA258, sizeof un_803FA258);
    preg(un_803FA4E0, sizeof un_803FA4E0);
    preg(un_803FA790, sizeof un_803FA790);
    preg(un_803FC4CC, sizeof un_803FC4CC);
    freg_all();
    for (op = 0; op < 20; op++) {
        for (k = 0; k < 400; k++) {
            u32 ret = 0;
            tu_case(op, k);
            zero(&ST, sizeof ST);
            zero(&menu_enter_data, sizeof menu_enter_data);
            zero(&approach_data, sizeof approach_data);
            zero(&intro_easy_exit_data, sizeof intro_easy_exit_data);
            zero(&intro_allstar_enter_data, sizeof intro_allstar_enter_data);
            zero(&intro_allstar_exit_data, sizeof intro_allstar_exit_data);
            zero(&memcard_enter_data, sizeof memcard_enter_data);
            zero(&memcard_exit_data, sizeof memcard_exit_data);
            zero(&intro_easy_enter_data, sizeof intro_easy_enter_data);
            zero(&prize_enter_data, sizeof prize_enter_data);
            fill_ResultsMatchInfo(&results0_data);
            fill_StartMeleeData(&vs_enter_data);
            fill_MatchExitInfo(&vs_exit_data);
            fill_DebugGameOverData(&game_over_data);
            fill_DebugResultsData(&results1_enter_data);
            fill_HSD_PadStatus(&HSD_PadCopyStatus[0]);
            HSD_PadCopyStatus[0].button = rnd();
            for (i = 0; i < 128; i++) un_803FA258[i] = (int) fv();
            switch (op) {
            case 0: onExitIntro(&ST); break;
            case 1: ST.info.enter_data = &menu_enter_data; onEnterMenu0(&ST); break;
            case 2: ret = (u32) fn_801B09F8((int) (fv() % 3)); break;
            case 3: ST.info.enter_data = &menu_enter_data; onEnterMenu1(&ST); break;
            case 4: ret = (u32) fn_801B0A8C((int) (fv() % 3)); break;
            case 5: ST.info.enter_data = &menu_enter_data; onEnterMenu2(&ST); break;
            case 6: ST.info.enter_data = &prize_enter_data; onEnterPrize(&ST); break;
            case 7: onExitPrize(&ST); break;
            case 8: ST.info.enter_data = &vs_enter_data; onEnterVs(&ST); break;
            case 9: ST.info.enter_data = &results0_data; onEnterResults0(&ST); break;
            case 10: ST.info.exit_data = &results0_data; onExitResults0(&ST); break;
            case 11: ST.info.enter_data = &intro_easy_enter_data; onEnterIntroEasy(&ST); break;
            case 12: ST.info.enter_data = &intro_allstar_enter_data; onEnterIntroAllstar(&ST); break;
            case 13: ST.info.enter_data = &game_over_data; onEnterGameOver(&ST); break;
            case 14: ST.info.enter_data = &approach_data; onEnterApproach(&ST); break;
            case 15: ST.info.enter_data = &results1_enter_data; onEnterResults1(&ST); break;
            case 16: onExitResults1(&ST); break;
            case 17: ST.info.enter_data = &memcard_enter_data; onEnterMemCard(&ST); break;
            case 18: ST.info.exit_data = &memcard_exit_data; onExitMemCard(&ST); break;
            case 19: for (i = 0; i < 16; i++) hash_GameModeState(&gm_Mode_Debug_States[i]); break;
            }
            hash_all();
            hash_HSD_PadStatus(&HSD_PadCopyStatus[0]);
            for (i = 0; i < 128; i++) HM((u32) un_803FA258[i]);
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
