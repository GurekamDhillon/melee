/* TU: src/melee/gr/grfourside.c */
/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a
 * recording stub. Written by gen/mtu/mk_stage_driver.py. Each line: op case ret state-hash trace-hash. */
#include TU_SRC
#include "mtu_common.h"
static HSD_GObj SGO;
static Ground SGP;
static char SJO[16];
static char YPB[64];
static Vec3 SV;
static HSD_JObj SJOB;
static struct grFourside_YakumonoParam YP;
static HSD_GObj MGO[8];
static Ground MGP[8];
static HSD_JObj MJO[8], FJ[10];
static CmSubject CMS;
static HSD_CObj COBJ;
static CollData CD;
static int PTRMODE;

#include "grfourside_gen.h"



static void setup(void)
{
    int i, n;
    (void) i; (void) n;
    zero(&SGO, sizeof SGO); zero(&SGP, sizeof SGP);
    fill_HSD_GObj(&SGO); fill_Ground(&SGP); fill_StageInfo(&stage_info);
    SGO.user_data = &SGP;
    SGO.hsd_obj = SJO;
    SV.x = rw(); SV.y = rw(); SV.z = rw();
    zero(&SJOB, sizeof SJOB); zero(&YP, sizeof YP); zero(MGO, sizeof MGO); zero(MGP, sizeof MGP); zero(MJO, sizeof MJO);
    zero(FJ, sizeof FJ); zero(&CMS, sizeof CMS); zero(&COBJ, sizeof COBJ); zero(&CD, sizeof CD);
    PTRMODE = 0;
    fill_HSD_JObj(&SJOB); SJOB.flags &= ~JOBJ_USE_QUATERNION;
    SGO.hsd_obj = &SJOB;
    fill_grFourside_YakumonoParam(&YP);
    YP.heli_wait = (int) (rnd() % 60); YP.heli_wait_add = (int) (rnd() % 40); YP.heli_stay_time = (int) (rnd() % 60);
    YP.crane_wait = (int) (rnd() % 50); YP.crane_wait_add = (int) (rnd() % 30); YP.crane_iron_wait = (int) (rnd() % 40);
    YP.crane_iron_wait_add = (int) (rnd() % 20);
    YP.crane_iron_up_min = (float) (rnd() % 20) * 0.5f; YP.crane_iron_up_max = (float) (rnd() % 60) * 0.5f;
    YP.crane_iron_down_min = (float) (rnd() % 20) * 0.5f; YP.crane_iron_down_max = (float) (rnd() % 60) * 0.5f;
    YP.crane_iron_spd = (float) (rnd() % 30) / 10.0f; YP.crane_iron_stop_acl = rw();
    YP.ufo_wait = (int) (rnd() % 60); YP.ufo_cs_offs = rw(); YP.ufo_stay_time = (int) (rnd() % 60);
    YP.ufo_stay_time_add = (int) (rnd() % 30); YP.ufo_challenge = (u16) (rnd() % 6); YP.x46 = (u16) (rnd() % 10);
    YP.x48 = (u16) (rnd() % 10);
    yakumono_param = &YP;
    fill_CmSubject(&CMS);
    fill_CollData(&CD);
    CD.x34_flags.b1234 = (rnd() & 1) ? 1 : fv();
    for (i = 0; i < 8; i++) {
        fill_HSD_GObj(&MGO[i]); fill_Ground(&MGP[i]); fill_HSD_JObj(&MJO[i]); MJO[i].flags &= ~JOBJ_USE_QUATERNION;
        MGO[i].user_data = &MGP[i];
        MGO[i].hsd_obj = &MJO[i];
    }
    for (i = 0; i < 10; i++) { fill_HSD_JObj(&FJ[i]); FJ[i].flags &= ~JOBJ_USE_QUATERNION; }

}

static void hash_all(void)
{
    int i;
    (void) i;
    if (PTRMODE & 1) { HM(PIB(SGP.u.foursideUfo.xC)); SGP.u.foursideUfo.xC = 0; }
    if (PTRMODE & 2) {
        HM(PIB(SGP.u.fourside.x0)); HM(PIB(SGP.u.fourside.x4)); HM(PIB(SGP.u.fourside.x8));
        SGP.u.fourside.x0 = 0; SGP.u.fourside.x4 = 0; SGP.u.fourside.x8 = 0;
    }
    if (PTRMODE & 4) {
        HM(SGP.u.foursideCrane.x1.b0); HM(SGP.u.foursideCrane.x1.b1); HM(SGP.u.foursideCrane.x1.b2); HM(SGP.u.foursideCrane.x1.b3);
        HM(SGP.u.foursideCrane.x1.b4); HM(SGP.u.foursideCrane.x1.b5); HM(SGP.u.foursideCrane.x1.b6); HM(SGP.u.foursideCrane.x1.b7);
        ((unsigned char*) &SGP.u)[1] = 0;
    }
    hash_HSD_GObj(&SGO); hash_Ground(&SGP); hash_StageInfo(&stage_info);
    HM(B(SV.x)); HM(B(SV.y)); HM(B(SV.z));
    hash_HSD_JObj(&SJOB); hash_grFourside_YakumonoParam(&YP); HM(PI(yakumono_param)); hash_CmSubject(&CMS); hash_CollData(&CD);
    for (i = 0; i < 8; i++) { hash_HSD_GObj(&MGO[i]); hash_Ground(&MGP[i]); hash_HSD_JObj(&MJO[i]); }
    for (i = 0; i < 10; i++) hash_HSD_JObj(&FJ[i]);

}

int main(void)
{
    int op, k;
    preg(&SGO, sizeof SGO);
    preg(&SGP, sizeof SGP);
    preg(SJO, sizeof SJO);
    preg(YPB, sizeof YPB);
    preg(&SV, sizeof SV);
    preg(&SJOB, sizeof SJOB);
    preg(&YP, sizeof YP);
    preg(MGO, sizeof MGO);
    preg(MGP, sizeof MGP);
    preg(MJO, sizeof MJO);
    preg(FJ, sizeof FJ);
    preg(&CMS, sizeof CMS);
    preg(&COBJ, 16);
    preg(&CD, sizeof CD);
    preg(grFs_StageCallbacks, sizeof grFs_StageCallbacks);
    preg(&grFs_StageData, sizeof grFs_StageData);
    preg(grFs_803E3CE8, sizeof grFs_803E3CE8);
    freg_all();
    for (op = 0; op < 44; op++) {
        for (k = 0; k < 500; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup();
            switch (op) {
            case 0: { grFourside_801F2D0C((int) fv()); } break;
            case 1: { PTRMODE |= 7; grFourside_801F2D10(); } break;
            case 2: { grFourside_801F2DA0(); } break;
            case 3: { grFourside_801F2DA4(); } break;
            case 4: { ret = (u32) grFourside_801F2DC8(); } break;
            case 5: { PTRMODE |= 7; ret = PI(grFourside_801F2DD0((int) (rnd() % 7))); } break;
            case 6: { stageGObj0_OnInit(&SGO); } break;
            case 7: { ret = (u32) grFourside_801F2EE8(&SGO); } break;
            case 8: { grFourside_801F2EF0(&SGO); } break;
            case 9: { grFourside_801F2EF4(&SGO); } break;
            case 10: { stageGObj2_OnInit(&SGO); } break;
            case 11: { ret = (u32) grFourside_801F2F24(&SGO); } break;
            case 12: { grFourside_801F2F2C(&SGO); } break;
            case 13: { grFourside_801F2F30(&SGO); } break;
            case 14: { PTRMODE |= 2; grFourside_801F2F34(&SGO); } break;
            case 15: { ret = (u32) grFourside_801F3070(&SGO); } break;
            case 16: { grFourside_801F3078(&SGO); } break;
            case 17: { grFourside_801F309C(&SGO); } break;
            case 18: { { int j = (int) fv(); int x = (int) fv(); float d = rw(); grFourside_801F30A0(&SGP, j, &CD, x, (mpLib_GroundEnum) fv(), d); } } break;
            case 19: { grFourside_801F30F0(&SGO); } break;
            case 20: { ret = (u32) grFourside_801F3144(&SGO); } break;
            case 21: { grFourside_801F314C(&SGO); } break;
            case 22: { grFourside_801F3150(&SGO); } break;
            case 23: { PTRMODE |= 4; SGP.u.foursideCrane.x1.b0 = rnd() & 1; SGP.u.foursideCrane.x1.b1 = rnd() & 1; SGP.u.foursideCrane.x1.b2 = rnd() & 1; SGP.u.foursideCrane.x1.b3 = rnd() & 1; SGP.u.foursideCrane.x1.b4 = rnd() & 1; SGP.u.foursideCrane.x1.b5 = rnd() & 1; SGP.u.foursideCrane.x1.b6 = rnd() & 1; SGP.u.foursideCrane.x1.b7 = rnd() & 1; grFourside_801F3154(&SGO); } break;
            case 24: { ret = (u32) grFourside_801F326C(&SGO); } break;
            case 25: { PTRMODE |= 4; SGP.u.foursideCrane.x0 = (u8) (rnd() % 8); SGP.u.foursideCrane.x1.b0 = rnd() & 1; SGP.u.foursideCrane.x1.b1 = rnd() & 1; SGP.u.foursideCrane.x1.b2 = rnd() & 1; SGP.u.foursideCrane.x1.b3 = rnd() & 1; SGP.u.foursideCrane.x1.b4 = rnd() & 1; SGP.u.foursideCrane.x1.b5 = rnd() & 1; SGP.u.foursideCrane.x1.b6 = rnd() & 1; SGP.u.foursideCrane.x1.b7 = rnd() & 1; SGP.u.foursideCrane.x4 = (int) (rnd() % 320) - 5; SGP.u.foursideCrane.x8 = (float) (rnd() % 200) / 10.0f - 10.0f; SGP.u.foursideCrane.xC = (float) (rnd() % 200) / 10.0f - 10.0f; SGP.u.foursideCrane.x10 = (float) (rnd() % 200) / 10.0f - 10.0f; SGP.u.foursideCrane.x14 = (float) (rnd() % 30) / 10.0f; SGP.u.foursideCrane.x18 = rw(); SGP.u.foursideCrane.x1C = rw(); grFourside_801F3274(&SGO); } break;
            case 26: { grFourside_801F37F8(&SGO); } break;
            case 27: { PTRMODE |= 1; grFourside_801F37FC(&SGO); } break;
            case 28: { ret = (u32) grFourside_801F388C(&SGO); } break;
            case 29: { PTRMODE |= 1; SGP.u.foursideUfo.x0 = (u8) (rnd() % 5); SGP.u.foursideUfo.x1 = (rnd() & 3) ? (u8) (rnd() % 3) : (u8) 0xFF; SGP.u.foursideUfo.x2 = (u8) (rnd() % 8); SGP.u.foursideUfo.x3 = (u8) (rnd() % 3); SGP.u.foursideUfo.x4 = (int) (rnd() % 80) - 5; SGP.u.foursideUfo.x8 = (int) (rnd() & 1); SGP.u.foursideUfo.xC = (rnd() & 1) ? &CMS : 0; grFourside_801F3894(&SGO); } break;
            case 30: { grFourside_801F3B6C(&SGO); } break;
            case 31: { PTRMODE |= 1; SGP.u.foursideUfo.x0 = (u8) (rnd() % 5); SGP.u.foursideUfo.x1 = (rnd() & 3) ? (u8) (rnd() % 3) : (u8) 0xFF; SGP.u.foursideUfo.x2 = (u8) (rnd() % 8); SGP.u.foursideUfo.x3 = (u8) (rnd() % 3); SGP.u.foursideUfo.x4 = (int) (rnd() % 80) - 5; SGP.u.foursideUfo.x8 = (int) (rnd() & 1); SGP.u.foursideUfo.xC = (rnd() & 1) ? &CMS : 0; grFourside_801F3B70(&SGO); } break;
            case 32: { grFourside_801F3C40(&SGO); } break;
            case 33: { ret = (u32) grFourside_801F3CC0(&SGO); } break;
            case 34: { SGP.u.fourside2.x0 = (u8) (rnd() % 5); SGP.u.fourside2.x1 = (u8) (rnd() % 4); SGP.u.fourside2.x4 = (int) (rnd() % 900); SGP.u.fourside2.x8 = (int) (rnd() % 30); grFourside_801F3CC8(&SGO); } break;
            case 35: { grFourside_801F3F0C(&SGO); } break;
            case 36: { MGP[3].u.fourside2.x0 = (u8) (rnd() % 5); MGP[3].u.fourside2.x1 = (u8) (rnd() % 3); MGP[3].u.fourside2.x4 = 695 + (int) (rnd() % 12); ret = (u32) grFourside_801F3F10(); } break;
            case 37: { PTRMODE |= 2; SGP.u.fourside.x0 = (rnd() & 1) ? &FJ[0] : 0; SGP.u.fourside.x4 = (rnd() & 1) ? &FJ[1] : 0; SGP.u.fourside.x8 = (rnd() & 1) ? &FJ[2] : 0; fn_801F3F74(&SGO, (int) (rnd() % 3)); } break;
            case 38: { ret = PI(grFourside_801F41E0((enum_t) fv())); } break;
            case 39: { ret = (u32) grFourside_801F41E8(&SV, (int) fv(), (HSD_JObj*) SJO); } break;
            case 40: { { int j; PTRMODE |= 4; SGP.u.foursideCrane.x1.b0 = rnd() & 1; SGP.u.foursideCrane.x1.b1 = rnd() & 1; SGP.u.foursideCrane.x1.b2 = rnd() & 1; SGP.u.foursideCrane.x1.b3 = rnd() & 1; SGP.u.foursideCrane.x1.b4 = rnd() & 1; SGP.u.foursideCrane.x1.b5 = rnd() & 1; SGP.u.foursideCrane.x1.b6 = rnd() & 1; SGP.u.foursideCrane.x1.b7 = rnd() & 1; grFourside_801F3154(&SGO); for (j = 0; j < 900; j++) grFourside_801F3274(&SGO); } } break;
            case 41: { { int j; PTRMODE |= 1; grFourside_801F37FC(&SGO); for (j = 0; j < 500; j++) { grFourside_801F3894(&SGO); if ((rnd() & 7) == 0) { MGP[5].u.fourside2.x8 += 1; } } } } break;
            case 42: { { int j; grFourside_801F3C40(&SGO); for (j = 0; j < 700; j++) { grFourside_801F3CC8(&SGO); if ((rnd() & 63) == 0) { MGP[5].u.fourside2.x0 = (u8) (rnd() % 3); MGP[5].u.fourside2.x1 = (u8) (rnd() % 3); } } } } break;
            case 43: { { int i; for (i = 0; i < 7; i++) hash_StageCallbacks(&grFs_StageCallbacks[i]); hash_StageData(&grFs_StageData); HM(SH(grFs_StageData.data1)); HM((u32) grFs_StageData.joint_count); } } break;
            }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
