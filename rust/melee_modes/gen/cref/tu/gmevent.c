/* TU: src/melee/gm/gmevent.c */
/* C oracle for src/tu/gmevent.rs: the whole TU included verbatim; every callee out of it is a
 * recording stub. Written by mk_gmevent_driver.py. Each line: op case ret state-hash trace-hash. */
#include TU_SRC
#include "mtu_common.h"
#define NLV 51
static GameModeState ST;
static struct gmm_x0 GM;
static PreloadedGameModeState PCS;
static VsSceneController VSC;
static VsSceneState VSS;
static MatchEnd MEND;
static HSD_GObj GNEW, GOA[4], G1;
static PlayerInitData PID;
static struct gm_801BAB40_src SRC;
static struct gm_804D6900_t* LVP[NLV];
static struct gm_804D6900_t LV[NLV];
static struct gm_evinit EVI[NLV];
static struct gm_evbonus EVB[NLV];
static struct gm_evstage_table EVS[NLV];
static struct gm_801BAB40_src PIN[NLV][6];
static struct gm_804D6900_x4_t X4[NLV];
static struct gm_801BAB40_src X4SRC[NLV];
static u8 CHL[64];
static const int SPECIAL[16] = { 4, 9, 12, 15, 19, 21, 23, 27, 29, 35, 39, 43, 44, 48, 36, 49 };
#include "gmevent_gen.h"

static void build_levels(int charlist, int nulls, int ptrmode)
{
    int l, j;
    for (l = 0; l < NLV; l++) {
        fill_gm_804D6900_t(&LV[l]);
        LV[l].kind = (u8) (fv() % 3);
        LV[l].flags = (u8) ((fv() & 0x1F) | ((fv() % 6) << 5));
        fill_gm_evinit(&EVI[l]);
        fill_gm_evbonus(&EVB[l]);
        fill_gm_evstage_table(&EVS[l]);
        EVS[l].count = (u8) (1 + fv() % 6);
        for (j = 0; j < 6; j++) {
            fill_gm_801BAB40_src(&PIN[l][j]);
            EVS[l].entries[j] = &PIN[l][j];
        }
        /* slot 4 with no character would make onEnterVs write the character into EventData byte 0xB,
         * which is a bitfield byte (native C and melee_types order bits differently): never in data */
        if (PIN[l][4].c_kind == ChKind_None) PIN[l][4].c_kind = 5;
        LV[l].evinit = &EVI[l];
        LV[l].evbonus = &EVB[l];
        LV[l].evstage_table = &EVS[l];
        for (j = 0; j < 5; j++) LV[l].player_init[j] = (j == 0 || j == 4 || !nulls || (fv() & 3)) ? &PIN[l][j] : 0;
        X4[l].x0 = (int) (fv() % 400);
        fill_gm_801BAB40_src(&X4SRC[l]);
        /* x4->x4 is data (coins, a frame count...) except where the code takes it as a player record */
        X4[l].x4 = ptrmode ? (intptr_t) &X4SRC[l] : (intptr_t) (int) fv();
        LV[l].x4 = charlist ? (struct gm_804D6900_x4_t*) CHL : &X4[l];
        LVP[l] = &LV[l];
    }
    gm_804D6900[0] = LVP;
}

static void hash_all(void)
{
    if (TRACEON) { int q; hash_StartMeleeRules(&vs_enter_data.rules); printf("H rules %08X%c", H, 10); for (q = 0; q < 6; q++) { hash_PlayerInitData(&vs_enter_data.players[q]); printf("H player%d %08X%c", q, H, 10); } }
    if (TRACEON) { struct EventData* e = &GM.vs.unk_530; printf("H ev x10 %d x18 %d x20 %d x24 %d x28 %d x2C %d x30 %d x34 %d x38 %d x3C %d x40 %d bits %d%d%d%d%d%d%d%d%c", e->x10, e->x18, e->x20, e->x24, e->x28, e->x2C, e->x30, e->x34, e->x38, e->x3C, e->x40, e->xB_0, e->xB_1, e->xB_2, e->xB_3, e->xB_4, e->xB_5, e->xB_6, e->xB_7, 10); }
    hash_EventData(&GM.vs.unk_530); if (TRACEON) printf("H EventData %08X%c", H, 10);
    hash_CSSData(&css_data); if (TRACEON) printf("H CSSData %08X%c", H, 10);
    hash_StartMeleeData(&vs_enter_data); if (TRACEON) printf("H StartMeleeData %08X%c", H, 10);
    hash_MatchExitInfo(&vs_exit_data[0]); if (TRACEON) printf("H MatchExitInfo %08X%c", H, 10);
    hash_MatchExitInfo(&vs_exit_data[1]); if (TRACEON) printf("H MatchExitInfo %08X%c", H, 10);
    hash_PreloadedGameModeState(&PCS); if (TRACEON) printf("H PreloadedGameModeState %08X%c", H, 10);
    hash_VsSceneController(&VSC); if (TRACEON) printf("H VsSceneController %08X%c", H, 10);
    HM(VSS.timer_seconds);
    HM(VSS.unk_2C); if (TRACEON) printf("H vss %08X%c", H, 10);
    hash_MatchEnd(&MEND); if (TRACEON) printf("H MatchEnd %08X%c", H, 10);
    hash_PlayerInitData(&PID); if (TRACEON) printf("H PlayerInitData %08X%c", H, 10);
    hash_gm_801BAB40_src(&SRC); if (TRACEON) printf("H gm_801BAB40_src %08X%c", H, 10);
}

int main(void)
{
    int op, k, i;
    preg(&ST, sizeof ST);
    preg(&GM, sizeof GM);
    preg(&PCS, sizeof PCS);
    preg(&VSC, sizeof VSC);
    preg(&VSS, sizeof VSS);
    preg(&MEND, sizeof MEND);
    preg(&GNEW, sizeof GNEW);
    preg(GOA, sizeof GOA);
    preg(&G1, sizeof G1);
    preg(&PID, sizeof PID);
    preg(&SRC, sizeof SRC);
    preg(&css_data, sizeof css_data);
    preg(&vs_enter_data, sizeof vs_enter_data);
    preg(vs_exit_data, sizeof vs_exit_data);
    preg(gm_804D6900, sizeof gm_804D6900);
    preg(gm_Mode_Event_States, sizeof gm_Mode_Event_States);
    preg(CHL, sizeof CHL);
    freg_all();
    for (op = 0; op < 62; op++) {
        for (k = 0; k < 300; k++) {
            u32 ret = 0;
            tu_case(op, k);
            zero(&ST, sizeof ST);
            zero(&GM, sizeof GM);
            gmMainLib_804D3EE0 = &GM;
            fill_EventData(&GM.vs.unk_530);
            GM.vs.unk_530.unk_535 = (u8) (((fv() & 3) == 0) ? SPECIAL[fv() % 16] : fv() % 51);
            GM.vs.unk_530.x20 = (int) (fv() % 6);
            fill_CSSData(&css_data);
            fill_StartMeleeData(&vs_enter_data);
            fill_MatchExitInfo(&vs_exit_data[0]);
            fill_MatchExitInfo(&vs_exit_data[1]);
            fill_PreloadedGameModeState(&PCS);
            fill_VsSceneController(&VSC);
            VSS.timer_seconds = fv();
            VSS.unk_2C = (u16) fv();
            fill_MatchEnd(&MEND);
            fill_PlayerInitData(&PID);
            fill_gm_801BAB40_src(&SRC);
            for (i = 0; i < 20; i++) CHL[i] = (u8) (fv() % 25);
            CHL[20] = 33;
            ST.id = (u8) fv();
            build_levels(op == 5 || op == 6, op == 6 || op == 7 || op == 9 || op == 10, op == 7 || op == 9 || op == 10 || op == 12 || op == 13 || op == 37);
            switch (op) {

            case 0: gm_801BA8FC(); break;
            case 1: { int lo = (int) (fv() % 4); int hi = lo + (int) (fv() % (u32) (5 - lo)); int a3 = (int) (fv() & 1); gm_801BA938(&GM.vs.unk_530, lo, hi, a3); } break;
            case 2: ST.info.enter_data = &css_data; onEnterCss(&ST); break;
            case 3: ST.info.exit_data = &css_data; onExitCss(&ST); break;
            case 4: gm_801BAB40(&PID, &SRC); break;
            case 5: { int n = (int) (fv() % 6); ST.info.enter_data = &vs_enter_data; ret = (u32) gm_801BAC9C(&ST, n); } break;
            case 6: ST.info.enter_data = &vs_enter_data; onEnterVs(&ST); break;
            case 7: ST.info.exit_data = &vs_exit_data[1]; onExitVs(&ST); break;
            case 8: gm_Mode_Event_OnInit(); break;
            case 9: gm_801BBB64(); break;
            case 10: gm_Mode_Event_OnLoad(); break;
            case 11: gm_Mode_Event_OnUnload(); break;
            case 12: fn_801BBFE8(); break;
            case 13: gm_801BC00C(); break;
            case 14: ret = (u32) gm_801BC488(); break;
            case 15: gm_801BC4F4(&G1); break;
            case 16: gm_801BC670(&G1); break;
            case 17: gm_801BC754(&G1); break;
            case 18: gm_801BC9E8(&G1); break;
            case 19: gm_801BCAF0(&G1); break;
            case 20: gm_801BCC9C(&G1); break;
            case 21: gm_801BCF20(&G1); break;
            case 22: gm_801BCF40(&G1); break;
            case 23: gm_801BD028(&G1); break;
            case 24: gm_801BD164(&G1); break;
            case 25: gm_801BD30C(&G1); break;
            case 26: gm_801BD44C(&G1); break;
            case 27: gm_801BD46C(&G1); break;
            case 28: gm_801BD658(&G1); break;
            case 29: gm_801BD7FC(&G1); break;
            case 30: gm_801BD93C(&G1); break;
            case 31: gm_801BDAD4(&G1); break;
            case 32: gm_801BDAF4(&G1); break;
            case 33: gm_801BDC08(&G1); break;
            case 34: gm_801BDD44(&G1); break;
            case 35: gm_801BDE94(&G1); break;
            case 36: gm_801BE37C(&G1); break;
            case 37: gm_801BE39C(&G1); break;
            case 38: gm_801BE618(&G1); break;
            case 39: gm_801BE638(&G1); break;
            case 40: failEvent(&G1); break;
            case 41: ret = PI(gm_GetEventData()); break;
            case 42: ret = (u32) gm_GetNextColor((u8) fv()); break;
            case 43: gm_801BBB64_inline(&GM.vs.unk_530); break;
            case 44: gm_801BC00C_inline(&SRC); break;
            case 45: ret = (u32) (int) gm_801BC00C_GetCharacter(&SRC); break;
            case 46: ret = (u32) gm_801BC00C_GetCharacterKind(&SRC); break;
            case 47: gm_801BEA10((int) fv()); break;
            case 48: gm_801BEA4C((int) fv()); break;
            case 49: { int a = (int) fv(); int b = (int) (fv() % 3); gm_801BEA88(a, b); } break;
            case 50: gm_801BEAF0((int) fv()); break;
            case 51: gm_801BEB2C((int) fv()); break;
            case 52: gm_801BEB68((int) fv()); break;
            case 53: gm_801BEB74((u8) fv()); break;
            case 54: ret = (u32) gm_801BEB80(); break;
            case 55: ret = (u32) gm_801BEB8C((u8) (fv() % 51)); break;
            case 56: ret = (u32) gm_801BEBA8((u8) (fv() % 51)); break;
            case 57: ret = (u32) gm_801BEBC0((u8) (fv() % 56)); break;
            case 58: ret = (u32) gm_801BEBF8(gm_803DF918[fv() % 51]); break;
            case 59: ret = PI(gm_801BEC54()); break;
            case 60: { int i; for (i = 0; i < 3; i++) hash_GameModeState(&gm_Mode_Event_States[i]); } break;
            case 61: { int i; for (i = 0; i < 51; i++) { HM(FI((const void*) gm_803DF94C[i]->x0)); HM(FI((const void*) gm_803DF94C[i]->x4)); } } break;
            }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR); fflush(0);
        }
    }
    return 0;
}
