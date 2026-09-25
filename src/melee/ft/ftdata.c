#include "ftdata.h"

#include <Runtime/platform.h>

#include <sysdolphin/baselib/forward.h>

#include <string.h>

#include "fighter.h"
#include "forward.h"
#include "ft_0877.h"
#include "inlines.h"
#include "kinds/ftCaptain/ftcaptain.h"
#include "kinds/ftCaptain/ftcaptainspecialhi.h"
#include "kinds/ftCaptain/ftcaptainspeciallw.h"
#include "kinds/ftCaptain/ftcaptainspecialn.h"
#include "kinds/ftCaptain/ftcaptainspecials.h"
#include "kinds/ftCLink/ftclink.h"
#include "kinds/ftCrazyHand/ftcrazyhand.h"
#include "kinds/ftDonkey/ftdonkey.h"
#include "kinds/ftDonkey/ftdonkeyspecialhi.h"
#include "kinds/ftDonkey/ftdonkeyspeciallw.h"
#include "kinds/ftDonkey/ftdonkeyspecialn.h"
#include "kinds/ftDonkey/ftdonkeyspecials.h"
#include "kinds/ftDrMario/ftdrmario.h"
#include "kinds/ftEmblem/ftemblem.h"
#include "kinds/ftFalco/ftfalco.h"
#include "kinds/ftFox/ftfox.h"
#include "kinds/ftFox/ftfoxspecialhi.h"
#include "kinds/ftFox/ftfoxspeciallw.h"
#include "kinds/ftFox/ftfoxspecialn.h"
#include "kinds/ftFox/ftfoxspecials.h"
#include "kinds/ftGameWatch/ftgamewatch.h"
#include "kinds/ftGameWatch/ftgamewatchspecialhi.h"
#include "kinds/ftGameWatch/ftgamewatchspeciallw.h"
#include "kinds/ftGameWatch/ftgamewatchspecialn.h"
#include "kinds/ftGameWatch/ftgamewatchspecials.h"
#include "kinds/ftGanon/ftganon.h"
#include "kinds/ftGigaKoopa/ftgkoopa.h"
#include "kinds/ftKirby/ftkirby.h"
#include "kinds/ftKirby/ftkirbyspecialhi.h"
#include "kinds/ftKoopa/ftkoopa.h"
#include "kinds/ftKoopa/ftkoopaspecialhi.h"
#include "kinds/ftKoopa/ftkoopaspeciallw.h"
#include "kinds/ftKoopa/ftkoopaspecialn.h"
#include "kinds/ftKoopa/ftkoopaspecials.h"
#include "kinds/ftLink/ftlink.h"
#include "kinds/ftLink/ftlinkspecialhi.h"
#include "kinds/ftLink/ftlinkspeciallw.h"
#include "kinds/ftLink/ftlinkspecialn.h"
#include "kinds/ftLink/ftlinkspecials.h"
#include "kinds/ftLuigi/ftluigi.h"
#include "kinds/ftLuigi/ftluigispecialhi.h"
#include "kinds/ftLuigi/ftluigispeciallw.h"
#include "kinds/ftLuigi/ftluigispecialn.h"
#include "kinds/ftLuigi/ftluigispecials.h"
#include "kinds/ftMario/ftmario.h"
#include "kinds/ftMario/ftmariospecialhi.h"
#include "kinds/ftMario/ftmariospeciallw.h"
#include "kinds/ftMario/ftmariospecialn.h"
#include "kinds/ftMario/ftmariospecials.h"
#include "kinds/ftMario/ftmariostrings.h"
#include "kinds/ftMars/ftmars.h"
#include "kinds/ftMars/ftmarsspecialhi.h"
#include "kinds/ftMars/ftmarsspeciallw.h"
#include "kinds/ftMars/ftmarsspecialn.h"
#include "kinds/ftMars/ftmarsspecials.h"
#include "kinds/ftMasterHand/ftmasterhand.h"
#include "kinds/ftMewtwo/ftmewtwo.h"
#include "kinds/ftMewtwo/ftmewtwospecialhi.h"
#include "kinds/ftMewtwo/ftmewtwospeciallw.h"
#include "kinds/ftMewtwo/ftmewtwospecialn.h"
#include "kinds/ftMewtwo/ftmewtwospecials.h"
#include "kinds/ftNana/ftnana.h"
#include "kinds/ftNess/ftness.h"
#include "kinds/ftNess/ftnessspecialhi.h"
#include "kinds/ftNess/ftnessspeciallw.h"
#include "kinds/ftNess/ftnessspecialn.h"
#include "kinds/ftNess/ftnessspecials.h"
#include "kinds/ftPeach/ftpeach.h"
#include "kinds/ftPeach/ftpeachspecialhi.h"
#include "kinds/ftPeach/ftpeachspeciallw.h"
#include "kinds/ftPeach/ftpeachspecialn.h"
#include "kinds/ftPeach/ftpeachspecials.h"
#include "kinds/ftPichu/ftpichu.h"
#include "kinds/ftPikachu/ftpikachu.h"
#include "kinds/ftPikachu/ftpikachuspecialhi.h"
#include "kinds/ftPikachu/ftpikachuspeciallw.h"
#include "kinds/ftPikachu/ftpikachuspecialn.h"
#include "kinds/ftPikachu/ftpikachuspecials.h"
#include "kinds/ftPopo/ftpopo.h"
#include "kinds/ftPopo/ftpopospecialhi.h"
#include "kinds/ftPopo/ftpopospeciallw.h"
#include "kinds/ftPopo/ftpopospecialn.h"
#include "kinds/ftPopo/ftpopospecials.h"
#include "kinds/ftPurin/ftpurin.h"
#include "kinds/ftPurin/ftpurinspecialhi.h"
#include "kinds/ftPurin/ftpurinspeciallw.h"
#include "kinds/ftPurin/ftpurinspecialn.h"
#include "kinds/ftPurin/ftpurinspecials.h"
#include "kinds/ftSamus/ftsamus.h"
#include "kinds/ftSamus/ftsamusspecialhi.h"
#include "kinds/ftSamus/ftsamusspeciallw1.h"
#include "kinds/ftSamus/ftsamusspecialn.h"
#include "kinds/ftSamus/ftsamusspecials.h"
#include "kinds/ftSandbag/ftsandbag.h"
#include "kinds/ftSeak/ftseak.h"
#include "kinds/ftSeak/ftseakspecialhi.h"
#include "kinds/ftSeak/ftseakspeciallw.h"
#include "kinds/ftSeak/ftseakspecialn.h"
#include "kinds/ftSeak/ftseakspecials.h"
#include "kinds/ftYoshi/ftyoshi.h"
#include "kinds/ftYoshi/ftyoshiguard.h"
#include "kinds/ftYoshi/ftyoshispecialhi.h"
#include "kinds/ftYoshi/ftyoshispecialn.h"
#include "kinds/ftYoshi/ftyoshispecials.h"
#include "kinds/ftZakoBoy/ftboy.h"
#include "kinds/ftZakoGirl/ftgirl.h"
#include "kinds/ftZelda/ftzelda.h"
#include "kinds/ftZelda/ftzeldaspecialhi.h"
#include "kinds/ftZelda/ftzeldaspeciallw.h"
#include "kinds/ftZelda/ftzeldaspecialn.h"
#include "kinds/ftZelda/ftzeldaspecials.h"
#include "types.h"
#include <dolphin/dvd.h>
#include <melee/ef/efasync.h>
#include <melee/lb/lbarchive.h>
#include <melee/lb/lbarq.h>
#include <melee/lb/lbdvd.h>
#include <melee/lb/lbfile.h>
#include <melee/pl/player.h>
#include <sysdolphin/baselib/debug.h>
#include <sysdolphin/baselib/gobj.h>
#include <sysdolphin/baselib/jobj.h>
#include <sysdolphin/baselib/objalloc.h>

typedef struct ft_8045993C_t {
    /* +0 */ u32 pad_x0;
    /* +4 */ u8 pad_x4[0x2];
    /* +6:0 */ u16 x6_b0 : 1;
    /* +6:1-2 */ u16 x6_b1_b2 : 2;
} ft_8045993C_t;

/* 4598B8 */ ftData* gFtDataList[Ft_Kind_Max];
/* 45993C */ ft_8045993C_t ft_8045993C[6];
/* 45996C */ int ft_8045996C[Ft_Kind_Max];

/// @todo All one struct maybe?
#ifdef MUST_MATCH
static void order_bss(void)
{
    (void) gFtDataList;
    (void) ft_8045993C;
    (void) ft_8045996C;
}
#endif

void ft_8008521C(HSD_GObj* gobj)
{
    Fighter* fp = GET_FIGHTER(gobj);
    HSD_JObj* jobj = GET_JOBJ(gobj);
    Vec3 pos;

    HSD_JObjGetTranslation(jobj, &pos);
    fp->self_vel.x = pos.x - fp->cur_pos.x;
    fp->self_vel.y = pos.y - fp->cur_pos.y;
    fp->self_vel.z = pos.z - fp->cur_pos.z;
}

static inline void ft_800852B0_Reset_ft_8045993C(ftData** list, int i)
{
#if defined(TARGET_PC)
    (void) list;
    ft_8045993C[i].pad_x0 = 0;
    ft_8045993C[i].x6_b0 = 0;
    ft_8045993C[i].x6_b1_b2 = 0;
#else
    /// @todo Bitfields seem off
    ((ft_8045993C_t*) &list[Ft_Kind_Max])[i].pad_x0 = 0;
    ((ft_8045993C_t*) &list[Ft_Kind_Max])[i].x6_b0 = 0;
    ((ft_8045993C_t*) &list[Ft_Kind_Max])[i].x6_b1_b2 = 0;
#endif
}

void ft_800852B0(void)
{
    ftData** list;
#if defined(TARGET_PC)
    ftData_UnkCountStruct* unk0 = ftData_Table_Unk0;
    ftData_UnkCountStruct* pairs = ftData_UnkIntPairs;
#else
    ftData_UnkCountStruct* unk0 =
        (ftData_UnkCountStruct*) &CostumeListsForeachCharacter[Ft_Kind_Max];
    ftData_UnkCountStruct* pairs =
        (ftData_UnkCountStruct*) ((u8*) CostumeListsForeachCharacter + 5940);
#endif
    int i;
    int new_var = 0;

    for (i = 0; i < Ft_Kind_Max; ++i) {
        int costume_idx = new_var;
        list = gFtDataList;
        list[i] = NULL;
        for (costume_idx = new_var;
             costume_idx < (s32) CostumeListsForeachCharacter[i].numCostumes;
             ++costume_idx)
        {
            CostumeListsForeachCharacter[i].costume_list[costume_idx].joint =
                NULL;
            CostumeListsForeachCharacter[i].costume_list[costume_idx].pad_x8 =
                0;
        }
        unk0[i].data = NULL;
        pairs[i].data = NULL;
    }
    ft_800852B0_Reset_ft_8045993C(list, new_var);
    ft_800852B0_Reset_ft_8045993C(list, 1);
    ft_800852B0_Reset_ft_8045993C(list, 2);
    ft_800852B0_Reset_ft_8045993C(list, 3);
    ft_800852B0_Reset_ft_8045993C(list, 4);
    ft_800852B0_Reset_ft_8045993C(list, 5);
}

void ft_8008549C(void)
{
    int i;
    for (i = 0; i < Ft_Kind_Max; i++) {
        ft_8045996C[i] = 0;
    }
}

/* 3C0EC0 */ struct UnkCostumeList
    CostumeListsForeachCharacter[Ft_Kind_Max] = {
        { ftMr_CostumeList, ARRAY_SIZE(ftMr_CostumeList) },
        { ftFx_CostumeList, ARRAY_SIZE(ftFx_CostumeList) },
        { ftCa_CostumeList, ARRAY_SIZE(ftCa_CostumeList) },
        { ftDk_CostumeList, ARRAY_SIZE(ftDk_CostumeList) },
        { ftKb_CostumeList, ARRAY_SIZE(ftKb_CostumeList) },
        { ftKp_CostumeList, ARRAY_SIZE(ftKp_CostumeList) },
        { ftLk_CostumeList, ARRAY_SIZE(ftLk_CostumeList) },
        { ftSk_CostumeList, ARRAY_SIZE(ftSk_CostumeList) },
        { ftNs_CostumeList, ARRAY_SIZE(ftNs_CostumeList) },
        { ftPe_CostumeList, ARRAY_SIZE(ftPe_CostumeList) },
        { ftPp_CostumeList, ARRAY_SIZE(ftPp_CostumeList) },
        { ftNn_CostumeList, FTNANA_COSTUME_COUNT },
        { ftPk_CostumeList, ARRAY_SIZE(ftPk_CostumeList) },
        { ftSs_CostumeList, ARRAY_SIZE(ftSs_CostumeList) },
        { ftYs_CostumeList, ARRAY_SIZE(ftYs_CostumeList) },
        { ftPr_CostumeList, ARRAY_SIZE(ftPr_CostumeList) },
        { ftMt_CostumeList, ARRAY_SIZE(ftMt_CostumeList) },
        { ftLg_CostumeList, ARRAY_SIZE(ftLg_CostumeList) },
        { ftMs_CostumeList, ARRAY_SIZE(ftMs_CostumeList) },
        { ftZd_CostumeList, ARRAY_SIZE(ftZd_CostumeList) },
        { ftCl_CostumeList, ARRAY_SIZE(ftCl_CostumeList) },
        { ftDr_CostumeList, ARRAY_SIZE(ftDr_CostumeList) },
        { ftFc_CostumeList, ARRAY_SIZE(ftFc_CostumeList) },
        { ftPc_CostumeList, ARRAY_SIZE(ftPc_CostumeList) },
        { ftGw_CostumeList, ARRAY_SIZE(ftGw_CostumeList) },
        { ftGn_CostumeList, ARRAY_SIZE(ftGn_CostumeList) },
        { ftFe_CostumeList, ARRAY_SIZE(ftFe_CostumeList) },
        { ftMh_CostumeList, ARRAY_SIZE(ftMh_CostumeList) },
        { ftCh_CostumeList, ARRAY_SIZE(ftCh_CostumeList) },
        { ftBo_CostumeList, ARRAY_SIZE(ftBo_CostumeList) },
        { ftGl_CostumeList, ARRAY_SIZE(ftGl_CostumeList) },
        { ftGk_CostumeList, ARRAY_SIZE(ftGk_CostumeList) },
        { ftSb_CostumeList, ARRAY_SIZE(ftSb_CostumeList) },
};

ftData_UnkCountStruct ftData_Table_Unk0[Ft_Kind_Max] = {
    { 0, 303 }, { 0, 327 }, { 0, 318 }, { 0, 337 }, { 0, 479 }, { 0, 316 },
    { 0, 314 }, { 0, 317 }, { 0, 326 }, { 0, 318 }, { 0, 321 }, { 0, 321 },
    { 0, 320 }, { 0, 313 }, { 0, 314 }, { 0, 327 }, { 0, 314 }, { 0, 312 },
    { 0, 327 }, { 0, 311 }, { 0, 314 }, { 0, 303 }, { 0, 327 }, { 0, 320 },
    { 0, 323 }, { 0, 318 }, { 0, 327 }, { 0, 345 }, { 0, 344 }, { 0, 295 },
    { 0, 295 }, { 0, 316 }, { 0, 296 },
};

Event ftData_Table_Unk1[Ft_Kind_Max] = {
    NULL,
    NULL,
    NULL,
    NULL,
    ftKb_Init_800EE528,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    ftPr_Init_8013C2F8,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
};

HSD_GObjEvent ftData_OnLoad[Ft_Kind_Max] = {
    ftMr_Init_OnLoad, ftFx_Init_OnLoad, ftCa_Init_OnLoad, ftDk_Init_OnLoad,
    ftKb_Init_OnLoad, ftKp_Init_OnLoad, ftLk_Init_OnLoad, ftSk_Init_OnLoad,
    ftNs_Init_OnLoad, ftPe_Init_OnLoad, ftPp_Init_OnLoad, ftNn_Init_OnLoad,
    ftPk_Init_OnLoad, ftSs_Init_OnLoad, ftYs_Init_OnLoad, ftPr_Init_OnLoad,
    ftMt_Init_OnLoad, ftLg_Init_OnLoad, ftMs_Init_OnLoad, ftZd_Init_OnLoad,
    ftCl_Init_OnLoad, ftDr_Init_OnLoad, ftFc_Init_OnLoad, ftPc_Init_OnLoad,
    ftGw_Init_OnLoad, ftGn_Init_OnLoad, ftFe_Init_OnLoad, ftMh_Init_OnLoad,
    ftCh_Init_OnLoad, ftBo_Init_OnLoad, ftGl_Init_OnLoad, ftGk_Init_OnLoad,
    ftSb_Init_OnLoad,
};

HSD_GObjEvent ftData_OnDeath[Ft_Kind_Max] = {
    ftMr_Init_OnDeath, ftFx_Init_OnDeath, ftCa_Init_OnDeath, ftDk_Init_OnDeath,
    ftKb_Init_OnDeath, ftKp_Init_OnDeath, ftLk_Init_OnDeath, ftSk_Init_OnDeath,
    ftNs_Init_OnDeath, ftPe_Init_OnDeath, ftPp_Init_OnDeath, ftNn_Init_OnDeath,
    ftPk_Init_OnDeath, ftSs_Init_OnDeath, ftYs_Init_OnDeath, ftPr_Init_OnDeath,
    ftMt_Init_OnDeath, ftLg_Init_OnDeath, ftMs_Init_OnDeath, ftZd_Init_OnDeath,
    ftCl_Init_OnDeath, ftDr_Init_OnDeath, ftFc_Init_OnDeath, ftPc_Init_OnDeath,
    ftGw_Init_OnDeath, ftGn_Init_OnDeath, ftFe_Init_OnDeath, ftMh_Init_OnDeath,
    ftCh_Init_OnDeath, ftBo_Init_OnDeath, ftGl_Init_OnDeath, ftGk_Init_OnDeath,
    ftSb_Init_OnDeath,
};

HSD_GObjEvent ftData_OnUserDataRemove[Ft_Kind_Max] = {
    NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL,
    NULL, NULL, NULL, NULL, NULL, NULL, NULL, ftPr_Init_OnUserDataRemove,
    NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL,
    NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL,
    NULL,
};

MotionState* ftData_CharacterStateTables[Ft_Kind_Max] = {
    ftMr_Init_MotionStateTable,
    ftFx_Init_MotionStateTable,
    ftCa_Init_MotionStateTable,
    ftDk_Init_MotionStateTable,
    ftKb_Init_MotionStateTable,
    ftKp_Init_MotionStateTable,
    ftLk_Init_MotionStateTable,
    ftSk_Init_MotionStateTable,
    ftNs_Init_MotionStateTable,
    ftPe_Init_MotionStateTable,
    ftPp_Init_MotionStateTable,
    ftNn_Init_MotionStateTable,
    ftPk_Init_MotionStateTable,
    ftSs_Init_MotionStateTable,
    ftYs_Init_MotionStateTable,
    ftPr_Init_MotionStateTable,
    ftMt_Init_MotionStateTable,
    ftLg_Init_MotionStateTable,
    ftMs_Init_MotionStateTable,
    ftZd_Init_MotionStateTable,
    ftCl_Init_MotionStateTable,
    ftDr_Init_MotionStateTable,
    ftFc_Init_MotionStateTable,
    ftPc_Init_MotionStateTable,
    ftGw_Init_MotionStateTable,
    ftGn_Init_MotionStateTable,
    ftFe_Init_MotionStateTable,
    ftMh_Init_MotionStateTable,
    ftCh_Init_MotionStateTable,
    NULL,
    NULL,
    ftGk_Init_MotionStateTable,
    ftSb_Init_MotionStateTable,
};

MotionState* ftData_UnkMotionStates0[Ft_Kind_Max] = {
    ftMr_Init_UnkMotionStates0,
    NULL,
    NULL,
    NULL,
    ftKb_Init_UnkMotionStates0,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    ftLg_Init_UnkMotionStates0,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    ftGk_Init_UnkMotionStates0,
    NULL,
};

HSD_GObjEvent ftData_SpecialS[Ft_Kind_Max] = {
    ftMr_SpecialS_Enter,
    ftFx_SpecialSStart_Enter,
    ftCa_SpecialS_Enter,
    ftDk_SpecialS_Enter,
    ftKb_SpecialS_Enter,
    ftKp_SpecialS_Enter,
    ftLk_SpecialS_Enter,
    ftSk_SpecialS_Enter,
    ftNs_SpecialS_Enter,
    ftPe_SpecialS_Enter,
    ftPp_SpecialS_Enter,
    NULL,
    ftPk_SpecialS_Enter,
    ftSs_SpecialS_Enter,
    ftYs_SpecialS_Enter,
    ftPr_SpecialS_Enter,
    ftMt_SpecialS_Enter,
    ftLg_SpecialS_Enter,
    ftMs_SpecialS_Enter,
    ftZd_SpecialS_Enter,
    ftLk_SpecialS_Enter,
    ftMr_SpecialS_Enter,
    ftFx_SpecialSStart_Enter,
    ftPk_SpecialS_Enter,
    ftGw_SpecialS_Enter,
    ftCa_SpecialS_Enter,
    ftMs_SpecialS_Enter,
    NULL,
    NULL,
    NULL,
    NULL,
    ftKp_SpecialS_Enter,
    NULL,
};

HSD_GObjEvent ftData_SpecialAirHi[Ft_Kind_Max] = {
    ftMr_SpecialAirHi_Enter,
    ftFx_SpecialAirHiStart_Enter,
    ftCa_SpecialAirHi_Enter,
    ftDk_SpecialAirHi_Enter,
    ftKb_SpecialAirHi_Enter,
    ftKp_SpecialAirHi_Enter,
    ftLk_SpecialAirHi_Enter,
    ftSk_SpecialAirHi_Enter,
    ftNs_SpecialAirHiStart_Enter,
    ftPe_SpecialAirHi_Enter,
    ftPp_SpecialAirHi_Enter,
    NULL,
    ftPk_SpecialAirHi_Enter,
    ftSs_SpecialAirHi_Enter,
    ftYs_SpecialAirHi_Enter,
    ftPr_SpecialAirHi_Enter,
    ftMt_SpecialAirHiStart_Enter,
    ftLg_SpecialAirHi_Enter,
    ftMs_SpecialAirHi_Enter,
    ftZd_SpecialAirHi_Enter,
    ftLk_SpecialAirHi_Enter,
    ftMr_SpecialAirHi_Enter,
    ftFx_SpecialAirHiStart_Enter,
    ftPk_SpecialAirHi_Enter,
    ftGw_SpecialAirHi_Enter,
    ftCa_SpecialAirHi_Enter,
    ftMs_SpecialAirHi_Enter,
    NULL,
    NULL,
    NULL,
    NULL,
    ftKp_SpecialAirHi_Enter,
    NULL,
};

HSD_GObjEvent ftData_SpecialAirLw[Ft_Kind_Max] = {
    ftMr_SpecialAirLw_Enter,
    ftFx_SpecialAirLw_Enter,
    ftCa_SpecialAirLw_Enter,
    NULL,
    ftKb_SpecialAirLw_Enter,
    ftKp_SpecialAirLw_Enter,
    ftLk_SpecialAirLw_Enter,
    ftSk_SpecialAirLw_Enter,
    ftNs_SpecialAirLwStart_Enter,
    ftPe_SpecialAirLw_Enter,
    ftPp_SpecialAirLw_Enter,
    ftPp_SpecialAirLw_Enter,
    ftPk_SpecialAirLw_Enter,
    ftSs_SpecialAirLw_Enter,
    ftYs_SpecialAirLw_Enter,
    ftPr_SpecialAirLw_Enter,
    ftMt_SpecialAirLw_Enter,
    ftLg_SpecialAirLw_Enter,
    ftMs_SpecialAirLw_Enter,
    ftZd_SpecialAirLw_Enter,
    ftLk_SpecialAirLw_Enter,
    ftMr_SpecialAirLw_Enter,
    ftFx_SpecialAirLw_Enter,
    ftPk_SpecialAirLw_Enter,
    ftGw_SpecialAirLw_Enter,
    ftCa_SpecialAirLw_Enter,
    ftMs_SpecialAirLw_Enter,
    NULL,
    NULL,
    NULL,
    NULL,
    ftKp_SpecialAirLw_Enter,
    NULL,
};

HSD_GObjEvent ftData_SpecialAirS[Ft_Kind_Max] = {
    ftMr_SpecialAirS_Enter,
    ftFx_SpecialAirSStart_Enter,
    ftCa_SpecialAirS_Enter,
    ftDk_SpecialAirS_Enter,
    ftKb_SpecialAirS_Enter,
    ftKp_SpecialAirS_Enter,
    ftLk_SpecialAirS_Enter,
    ftSk_SpecialAirS_Enter,
    ftNs_SpecialAirS_Enter,
    ftPe_SpecialAirS_Enter,
    ftPp_SpecialAirS_Enter,
    NULL,
    ftPk_SpecialAirS_Enter,
    ftSs_SpecialAirS_Enter,
    ftYs_SpecialAirS_Enter,
    ftPr_SpecialAirS_Enter,
    ftMt_SpecialAirS_Enter,
    ftLg_SpecialAirS_Enter,
    ftMs_SpecialAirS_Enter,
    ftZd_SpecialAirS_Enter,
    ftLk_SpecialAirS_Enter,
    ftMr_SpecialAirS_Enter,
    ftFx_SpecialAirSStart_Enter,
    ftPk_SpecialAirS_Enter,
    ftGw_SpecialAirS_Enter,
    ftCa_SpecialAirS_Enter,
    ftMs_SpecialAirS_Enter,
    NULL,
    NULL,
    NULL,
    NULL,
    ftKp_SpecialAirS_Enter,
    NULL,
};

HSD_GObjEvent ftData_SpecialAirN[Ft_Kind_Max] = {
    ftMr_SpecialAirN_Enter,
    ftFx_SpecialAirN_Enter,
    ftCa_SpecialAirN_Enter,
    ftDk_SpecialAirN_Enter,
    ftKb_SpecialAirN_Enter,
    ftKp_SpecialAirN_Enter,
    ftLk_SpecialAirN_Enter,
    ftSk_SpecialAirN_Enter,
    ftNs_SpecialAirNStart_Enter,
    ftPe_SpecialAirN_Enter,
    ftPp_SpecialAirN_Enter,
    ftPp_SpecialAirN_Enter,
    ftPk_SpecialAirN_Enter,
    ftSs_SpecialAirN_Enter,
    ftYs_SpecialAirN_Enter,
    ftPr_SpecialAirN_Enter,
    ftMt_SpecialAirN_Enter,
    ftLg_SpecialAirN_Enter,
    ftMs_SpecialAirN_Enter,
    ftZd_SpecialAirN_Enter,
    ftLk_SpecialAirN_Enter,
    ftMr_SpecialAirN_Enter,
    ftFx_SpecialAirN_Enter,
    ftPk_SpecialAirN_Enter,
    ftGw_SpecialAirN_Enter,
    ftCa_SpecialAirN_Enter,
    ftMs_SpecialAirN_Enter,
    NULL,
    NULL,
    NULL,
    NULL,
    ftKp_SpecialAirN_Enter,
    NULL,
};

HSD_GObjEvent ftData_SpecialN[Ft_Kind_Max] = {
    ftMr_SpecialN_Enter,
    ftFx_SpecialN_Enter,
    ftCa_SpecialN_Enter,
    ftDk_SpecialN_Enter,
    ftKb_SpecialN_Enter,
    ftKp_SpecialN_Enter,
    ftLk_SpecialN_Enter,
    ftSk_SpecialN_Enter,
    ftNs_SpecialNStart_Enter,
    ftPe_SpecialN_Enter,
    ftPp_SpecialN_Enter,
    ftPp_SpecialN_Enter,
    ftPk_SpecialN_Enter,
    ftSs_SpecialN_Enter,
    ftYs_SpecialN_Enter,
    ftPr_SpecialN_Enter,
    ftMt_SpecialN_Enter,
    ftLg_SpecialN_Enter,
    ftMs_SpecialN_Enter,
    ftZd_SpecialN_Enter,
    ftLk_SpecialN_Enter,
    ftMr_SpecialN_Enter,
    ftFx_SpecialN_Enter,
    ftPk_SpecialN_Enter,
    ftGw_SpecialN_Enter,
    ftCa_SpecialN_Enter,
    ftMs_SpecialN_Enter,
    NULL,
    NULL,
    NULL,
    NULL,
    ftKp_SpecialN_Enter,
    NULL,
};

HSD_GObjEvent ftData_SpecialLw[Ft_Kind_Max] = {
    ftMr_SpecialLw_Enter,
    ftFx_SpecialLw_Enter,
    ftCa_SpecialLw_Enter,
    ftDk_SpecialLw_Enter,
    ftKb_SpecialLw_Enter,
    ftKp_SpecialLw_Enter,
    ftLk_SpecialLw_Enter,
    ftSk_SpecialLw_Enter,
    ftNs_SpecialLwStart_Enter,
    ftPe_SpecialLw_Enter,
    ftPp_SpecialLw_Enter,
    ftPp_SpecialLw_Enter,
    ftPk_SpecialLw_Enter,
    ftSs_SpecialLw_Enter,
    ftYs_SpecialLw_Enter,
    ftPr_SpecialLw_Enter,
    ftMt_SpecialLw_Enter,
    ftLg_SpecialLw_Enter,
    ftMs_SpecialLw_Enter,
    ftZd_SpecialLw_Enter,
    ftLk_SpecialLw_Enter,
    ftMr_SpecialLw_Enter,
    ftFx_SpecialLw_Enter,
    ftPk_SpecialLw_Enter,
    ftGw_SpecialLw_Enter,
    ftCa_SpecialLw_Enter,
    ftMs_SpecialLw_Enter,
    NULL,
    NULL,
    NULL,
    NULL,
    ftKp_SpecialLw_Enter,
    NULL,
};

HSD_GObjEvent ftData_SpecialHi[Ft_Kind_Max] = {
    ftMr_SpecialHi_Enter,
    ftFx_SpecialHi_Enter,
    ftCa_SpecialHi_Enter,
    ftDk_SpecialHi_Enter,
    ftKb_SpecialHi_Enter,
    ftKp_SpecialHi_Enter,
    ftLk_SpecialHi_Enter,
    ftSk_SpecialHi_Enter,
    ftNs_SpecialHiStart_Enter,
    ftPe_SpecialHi_Enter,
    ftPp_SpecialHi_Enter,
    NULL,
    ftPk_SpecialHi_Enter,
    ftSs_SpecialHi_Enter,
    ftYs_SpecialHi_Enter,
    ftPr_SpecialHi_Enter,
    ftMt_SpecialHiStart_Enter,
    ftLg_SpecialHi_Enter,
    ftMs_SpecialHi_Enter,
    ftZd_SpecialHi_Enter,
    ftLk_SpecialHi_Enter,
    ftMr_SpecialHi_Enter,
    ftFx_SpecialHi_Enter,
    ftPk_SpecialHi_Enter,
    ftGw_SpecialHi_Enter,
    ftCa_SpecialHi_Enter,
    ftMs_SpecialHi_Enter,
    NULL,
    NULL,
    NULL,
    NULL,
    ftKp_SpecialHi_Enter,
    NULL,
};

HSD_GObjEvent ftData_OnAbsorb[Ft_Kind_Max] = {
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    ftNs_Init_OnAbsorb,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    ftGw_Init_OnAbsorb,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
};

Fighter_ItemEvent ftData_OnItemPickupExt[Ft_Kind_Max] = {
    ftMr_Init_OnItemPickup,
    ftFx_Init_OnItemPickup,
    ftCa_Init_OnItemPickup,
    ftDk_Init_OnItemPickup,
    ftKb_Init_OnItemPickup,
    ftKp_Init_OnItemPickup,
    ftLk_Init_OnItemPickupExt,
    ftSk_Init_OnItemPickup,
    ftNs_Init_OnItemPickup,
    ftPe_Init_OnItemPickup,
    ftPp_Init_OnItemPickup,
    ftPp_Init_OnItemPickup,
    ftPk_Init_OnItemPickup,
    ftSs_Init_OnItemPickup,
    ftYs_Init_OnItemPickup,
    ftPr_Init_OnItemPickup,
    ftMt_Init_OnItemPickup,
    ftLg_Init_OnItemPickup,
    ftMs_Init_OnItemPickup,
    ftZd_Init_OnItemPickup,
    ftCl_Init_OnItemPickupExt,
    ftDr_Init_OnItemPickup,
    ftFc_Init_OnItemPickup,
    ftPc_Init_OnItemPickup,
    ftGw_Init_OnItemPickup,
    ftGn_Init_OnItemPickup,
    ftFe_Init_OnItemPickup,
    NULL,
    NULL,
    ftBo_Init_OnItemPickup,
    ftGl_Init_OnItemPickup,
    ftGk_Init_OnItemPickup,
    NULL,
};

HSD_GObjEvent ftData_OnItemInvisible[Ft_Kind_Max] = {
    ftMr_Init_OnItemInvisible,
    ftFx_Init_OnItemInvisible,
    ftCa_Init_OnItemInvisible,
    ftDk_Init_OnItemInvisible,
    ftKb_Init_OnItemInvisible,
    ftKp_Init_OnItemInvisible,
    ftLk_Init_OnItemInvisible,
    ftSk_Init_OnItemInvisible,
    ftNs_Init_OnItemInvisible,
    ftPe_Init_OnItemInvisible,
    ftPp_Init_OnItemInvisible,
    ftPp_Init_OnItemInvisible,
    ftPk_Init_OnItemInvisible,
    ftSs_Init_OnItemInvisible,
    ftYs_Init_OnItemInvisible,
    ftPr_Init_OnItemInvisible,
    ftMt_Init_OnItemInvisible,
    ftLg_Init_OnItemInvisible,
    ftMs_Init_OnItemInvisible,
    ftZd_Init_OnItemInvisible,
    ftCl_Init_OnItemInvisible,
    ftDr_Init_OnItemInvisible,
    ftFc_Init_OnItemInvisible,
    ftPc_Init_OnItemInvisible,
    ftGw_Init_OnItemInvisible,
    ftGn_Init_OnItemInvisible,
    ftFe_Init_OnItemInvisible,
    NULL,
    NULL,
    ftBo_Init_OnItemInvisible,
    ftGl_Init_OnItemInvisible,
    ftGk_Init_OnItemInvisible,
    NULL,
};

HSD_GObjEvent ftData_OnItemVisible[Ft_Kind_Max] = {
    ftMr_Init_OnItemVisible,
    ftFx_Init_OnItemVisible,
    ftCa_Init_OnItemVisible,
    ftDk_Init_OnItemVisible,
    ftKb_Init_OnItemVisible,
    ftKp_Init_OnItemVisible,
    ftLk_Init_OnItemVisible,
    ftSk_Init_OnItemVisible,
    ftNs_Init_OnItemVisible,
    ftPe_Init_OnItemVisible,
    ftPp_Init_OnItemVisible,
    ftPp_Init_OnItemVisible,
    ftPk_Init_OnItemVisible,
    ftSs_Init_OnItemVisible,
    ftYs_Init_OnItemVisible,
    ftPr_Init_OnItemVisible,
    ftMt_Init_OnItemVisible,
    ftLg_Init_OnItemVisible,
    ftMs_Init_OnItemVisible,
    ftZd_Init_OnItemVisible,
    ftCl_Init_OnItemVisible,
    ftDr_Init_OnItemVisible,
    ftFc_Init_OnItemVisible,
    ftPc_Init_OnItemVisible,
    ftGw_Init_OnItemVisible,
    ftGn_Init_OnItemVisible,
    ftFe_Init_OnItemVisible,
    NULL,
    NULL,
    ftBo_Init_OnItemVisible,
    ftGl_Init_OnItemVisible,
    ftGk_Init_OnItemVisible,
    NULL,
};

Fighter_ItemEvent ftData_OnItemDropExt[Ft_Kind_Max] = {
    ftMr_Init_OnItemDrop,
    ftFx_Init_OnItemDrop,
    ftCa_Init_OnItemDrop,
    ftDk_Init_OnItemDrop,
    ftKb_Init_OnItemDrop,
    ftKp_Init_OnItemDrop,
    ftLk_Init_OnItemDropExt,
    ftSk_Init_OnItemDrop,
    ftNs_Init_OnItemDrop,
    ftPe_Init_OnItemDrop,
    ftPp_Init_OnItemDrop,
    ftPp_Init_OnItemDrop,
    ftPk_Init_OnItemDrop,
    ftSs_Init_OnItemDrop,
    ftYs_Init_OnItemDrop,
    ftPr_Init_OnItemDrop,
    ftMt_Init_OnItemDrop,
    ftLg_Init_OnItemDrop,
    ftMs_Init_OnItemDrop,
    ftZd_Init_OnItemDrop,
    ftCl_Init_OnItemDropExt,
    ftDr_Init_OnItemDrop,
    ftFc_Init_OnItemDrop,
    ftPc_Init_OnItemDrop,
    ftGw_Init_OnItemDrop,
    ftGn_Init_OnItemDrop,
    ftFe_Init_OnItemDrop,
    NULL,
    NULL,
    ftBo_Init_OnItemDrop,
    ftGl_Init_OnItemDrop,
    ftGk_Init_OnItemDrop,
    NULL,
};

Fighter_ItemEvent ftData_OnItemPickup[Ft_Kind_Max] = {
    ftMr_Init_OnItemPickup,
    ftFx_Init_OnItemPickup,
    ftCa_Init_OnItemPickup,
    ftDk_Init_OnItemPickup,
    ftKb_Init_OnItemPickup,
    ftKp_Init_OnItemPickup,
    ftLk_Init_OnItemPickup,
    ftSk_Init_OnItemPickup,
    ftNs_Init_OnItemPickup,
    ftPe_Init_OnItemPickup,
    ftPp_Init_OnItemPickup,
    ftPp_Init_OnItemPickup,
    ftPk_Init_OnItemPickup,
    ftSs_Init_OnItemPickup,
    ftYs_Init_OnItemPickup,
    ftPr_Init_OnItemPickup,
    ftMt_Init_OnItemPickup,
    ftLg_Init_OnItemPickup,
    ftMs_Init_OnItemPickup,
    ftZd_Init_OnItemPickup,
    ftCl_Init_OnItemPickup,
    ftDr_Init_OnItemPickup,
    ftFc_Init_OnItemPickup,
    ftPc_Init_OnItemPickup,
    ftGw_Init_OnItemPickup,
    ftGn_Init_OnItemPickup,
    ftFe_Init_OnItemPickup,
    NULL,
    NULL,
    ftBo_Init_OnItemPickup,
    ftGl_Init_OnItemPickup,
    ftGk_Init_OnItemPickup,
    NULL,
};

Fighter_ItemEvent ftData_OnItemDrop[Ft_Kind_Max] = {
    ftMr_Init_OnItemDrop,
    ftFx_Init_OnItemDrop,
    ftCa_Init_OnItemDrop,
    ftDk_Init_OnItemDrop,
    ftKb_Init_OnItemDrop,
    ftKp_Init_OnItemDrop,
    ftLk_Init_OnItemDrop,
    ftSk_Init_OnItemDrop,
    ftNs_Init_OnItemDrop,
    ftPe_Init_OnItemDrop,
    ftPp_Init_OnItemDrop,
    ftPp_Init_OnItemDrop,
    ftPk_Init_OnItemDrop,
    ftSs_Init_OnItemDrop,
    ftYs_Init_OnItemDrop,
    ftPr_Init_OnItemDrop,
    ftMt_Init_OnItemDrop,
    ftLg_Init_OnItemDrop,
    ftMs_Init_OnItemDrop,
    ftZd_Init_OnItemDrop,
    ftCl_Init_OnItemDrop,
    ftDr_Init_OnItemDrop,
    ftFc_Init_OnItemDrop,
    ftPc_Init_OnItemDrop,
    ftGw_Init_OnItemDrop,
    ftGn_Init_OnItemDrop,
    ftFe_Init_OnItemDrop,
    NULL,
    NULL,
    ftBo_Init_OnItemDrop,
    ftGl_Init_OnItemDrop,
    ftGk_Init_OnItemDrop,
    NULL,
};

HSD_GObjEvent ftData_UnkMotionStates1[Ft_Kind_Max] = {
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    ftPk_Init_UnkMotionStates1,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
};

HSD_GObjEvent ftData_UnkMotionStates2[Ft_Kind_Max] = {
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    ftPk_Init_UnkMotionStates2,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
};

HSD_GObjEvent ftData_OnKnockbackEnter[Ft_Kind_Max] = {
    ftMr_Init_OnKnockbackEnter,
    ftFx_Init_OnKnockbackEnter,
    NULL,
    ftDk_Init_OnKnockbackEnter,
    ftKb_Init_OnKnockbackEnter,
    ftKp_Init_OnKnockbackEnter,
    ftLk_Init_OnKnockbackEnter,
    ftSk_Init_OnKnockbackEnter,
    ftNs_Init_OnKnockbackEnter,
    ftPe_Init_OnKnockbackEnter,
    ftPp_Init_OnKnockbackEnter,
    ftPp_Init_OnKnockbackEnter,
    ftPk_Init_OnKnockbackEnter,
    NULL,
    ftYs_Init_OnKnockbackEnter,
    ftPr_Init_OnKnockbackEnter,
    ftMt_Init_OnKnockbackEnter,
    ftLg_Init_OnKnockbackEnter,
    ftMs_Init_OnKnockbackEnter,
    ftZd_Init_OnKnockbackEnter,
    ftCl_Init_OnKnockbackEnter,
    ftDr_Init_OnKnockbackEnter,
    ftFc_Init_OnKnockbackEnter,
    ftPc_Init_OnKnockbackEnter,
    NULL,
    ftGn_Init_OnKnockbackEnter,
    ftFe_Init_OnKnockbackEnter,
    NULL,
    NULL,
    NULL,
    NULL,
    ftGk_Init_OnKnockbackEnter,
    ftSb_Init_OnKnockbackEnter,
};

HSD_GObjEvent ftData_OnKnockbackExit[Ft_Kind_Max] = {
    ftMr_Init_OnKnockbackExit,
    ftFx_Init_OnKnockbackExit,
    NULL,
    ftDk_Init_OnKnockbackExit,
    ftKb_Init_OnKnockbackExit,
    ftKp_Init_OnKnockbackExit,
    ftLk_Init_OnKnockbackExit,
    ftSk_Init_OnKnockbackExit,
    ftNs_Init_OnKnockbackExit,
    ftPe_Init_OnKnockbackExit,
    ftPp_Init_OnKnockbackExit,
    ftPp_Init_OnKnockbackExit,
    ftPk_Init_OnKnockbackExit,
    NULL,
    ftYs_Init_OnKnockbackExit,
    ftPr_Init_OnKnockbackExit,
    ftMt_Init_OnKnockbackExit,
    ftLg_Init_OnKnockbackExit,
    ftMs_Init_OnKnockbackExit,
    ftZd_Init_OnKnockbackExit,
    ftCl_Init_OnKnockbackExit,
    ftDr_Init_OnKnockbackExit,
    ftFc_Init_OnKnockbackExit,
    ftPc_Init_OnKnockbackExit,
    NULL,
    ftGn_Init_OnKnockbackExit,
    ftFe_Init_OnKnockbackExit,
    NULL,
    NULL,
    NULL,
    NULL,
    ftGk_Init_OnKnockbackExit,
    ftSb_Init_OnKnockbackExit,
};

HSD_GObjEvent ftData_UnkMotionStates3[Ft_Kind_Max] = {
    NULL,
    NULL,
    NULL,
    NULL,
    ftKb_Init_UnkMotionStates3,
    ftKp_Init_UnkMotionStates3,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    ftGk_Init_UnkMotionStates3,
    NULL,
};

HSD_GObjEvent ftData_UnkMotionStates4[Ft_Kind_Max] = {
    NULL,
    NULL,
    NULL,
    ftDk_Init_UnkMotionStates4,
    ftKb_Init_UnkMotionStates4,
    NULL,
    NULL,
    ftSk_Init_UnkMotionStates4,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    ftSs_Init_UnkMotionStates4,
    NULL,
    NULL,
    ftMt_Init_UnkMotionStates4,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    ftGw_Init_UnkMotionStates4,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
};

HSD_GObjEvent ftKindCalcIndiviParamTable[Ft_Kind_Max] = {
    ftMr_Init_LoadSpecialAttrs, ftFx_Init_LoadSpecialAttrs,
    ftCa_Init_LoadSpecialAttrs, ftDk_Init_LoadSpecialAttrs,
    ftKb_Init_LoadSpecialAttrs, ftKp_Init_LoadSpecialAttrs,
    ftLk_Init_LoadSpecialAttrs, ftSk_Init_LoadSpecialAttrs,
    ftNs_Init_LoadSpecialAttrs, ftPe_Init_LoadSpecialAttrs,
    ftPp_Init_LoadSpecialAttrs, ftNn_Init_LoadSpecialAttrs,
    ftPk_Init_LoadSpecialAttrs, ftSs_Init_LoadSpecialAttrs,
    ftYs_Init_LoadSpecialAttrs, ftPr_Init_LoadSpecialAttrs,
    ftMt_Init_LoadSpecialAttrs, ftLg_Init_LoadSpecialAttrs,
    ftMs_Init_LoadSpecialAttrs, ftZd_Init_LoadSpecialAttrs,
    ftCl_Init_LoadSpecialAttrs, ftDr_Init_LoadSpecialAttrs,
    ftFc_Init_LoadSpecialAttrs, ftPc_Init_LoadSpecialAttrs,
    ftGw_Init_LoadSpecialAttrs, ftGn_Init_LoadSpecialAttrs,
    ftFe_Init_LoadSpecialAttrs, ftMh_Init_LoadSpecialAttrs,
    ftCh_Init_LoadSpecialAttrs, ftBo_Init_LoadSpecialAttrs,
    ftGl_Init_LoadSpecialAttrs, ftGk_Init_LoadSpecialAttrs,
    ftSb_Init_LoadSpecialAttrs,
};

/// Standard Character .dat File Names
struct StringPair {
    char* a;
    char* b;
};

struct StringPair ftData_803C1F40[Ft_Kind_Max] = {
    { ftMr_Init_DatFilename, ftMr_Init_DataName },
    { ftFx_Init_DatFilename, ftFx_Init_DataName },
    { ftCa_Init_DatFilename, ftCa_Init_DataName },
    { ftDk_Init_DatFilename, ftDk_Init_DataName },
    { ftKb_Init_DatFilename, ftKb_Init_DataName },
    { ftKp_Init_DatFilename, ftKp_Init_DataName },
    { ftLk_Init_DatFilename, ftLk_Init_DataName },
    { ftSk_Init_DatFilename, ftSk_Init_DataName },
    { ftNs_Init_DatFilename, ftNs_Init_DataName },
    { ftPe_Init_DatFilename, ftPe_Init_DataName },
    { ftPp_Init_DatFilename, ftPp_Init_DataName },
    { ftNn_Init_DatFilename, ftNn_Init_DataName },
    { ftPk_Init_DatFilename, ftPk_Init_DataName },
    { ftSs_Init_DatFilename, ftSs_Init_DataName },
    { ftYs_Init_DatFilename, ftYs_Init_DataName },
    { ftPr_Init_DatFilename, ftPr_Init_DataName },
    { ftMt_Init_DatFilename, ftMt_Init_DataName },
    { ftLg_Init_DatFilename, ftLg_Init_DataName },
    { ftMs_Init_DatFilename, ftMs_Init_DataName },
    { ftZd_Init_DatFilename, ftZd_Init_DataName },
    { ftCl_Init_DatFilename, ftCl_Init_DataName },
    { ftDr_Init_DatFilename, ftDr_Init_DataName },
    { ftFc_Init_DatFilename, ftFc_Init_DataName },
    { ftPc_Init_DatFilename, ftPc_Init_DataName },
    { ftGw_Init_DatFilename, ftGw_Init_DataName },
    { ftGn_Init_DatFilename, ftGn_Init_DataName },
    { ftFe_Init_DatFilename, ftFe_Init_DataName },
    { ftMh_Init_DatFilename, ftMh_Init_DataName },
    { ftCh_Init_DatFilename, ftCh_Init_DataName },
    { ftBo_Init_DatFilename, ftBo_Init_DataName },
    { ftGl_Init_DatFilename, ftGl_Init_DataName },
    { ftGk_Init_DatFilename, ftGk_Init_DataName },
    { ftSb_Init_DatFilename, ftSb_Init_DataName },
};

Event ftData_UnkMotionStates5[Ft_Kind_Max] = {
    NULL, NULL, NULL, NULL, ftKb_Init_UnkMotionStates5,
    NULL, NULL, NULL, NULL, NULL,
    NULL, NULL, NULL, NULL, NULL,
    NULL, NULL, NULL, NULL, NULL,
    NULL, NULL, NULL, NULL, NULL,
    NULL, NULL, NULL, NULL, NULL,
    NULL, NULL, NULL,
};

Fighter_UnkMtxEvent ftData_UnkMtxFunc0[Ft_Kind_Max] = {
    NULL,
    NULL,
    NULL,
    NULL,
    ftKb_UnkMtxFunc0,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    ftPr_Init_UnkMtxFunc0,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
};

/// Character model group (e.g. high poly, low poly, metal) visibility change
/// callbacks
ftData_UnkModelStruct ftData_UnkIntBoolFunc0 = {
    {
        NULL,
        NULL,
        NULL,
        NULL,
        ftKb_UnkIntBoolFunc0,
        NULL,
        NULL,
        NULL,
        NULL,
        NULL,
        NULL,
        NULL,
        NULL,
        NULL,
        NULL,
        ftPr_Init_UnkIntBoolFunc0,
        NULL,
        NULL,
        NULL,
        NULL,
        NULL,
        NULL,
        NULL,
        NULL,
        NULL,
        NULL,
        NULL,
        NULL,
        NULL,
        NULL,
        NULL,
        NULL,
        NULL,
        NULL,
    },
    {
        NULL,
        NULL,
        NULL,
        NULL,
        ftKb_Init_UnkMotionStates6,
        NULL,
        NULL,
        NULL,
        NULL,
        NULL,
        NULL,
        NULL,
        NULL,
        NULL,
        NULL,
        ftPr_Init_UnkMotionStates6,
        NULL,
        NULL,
        NULL,
        NULL,
        NULL,
        NULL,
        NULL,
        NULL,
        NULL,
        NULL,
        NULL,
        NULL,
        NULL,
        NULL,
        NULL,
        NULL,
        NULL,
        NULL,
    },
};

struct {
    HSD_GObjEvent x0;
    void (*x4)(Fighter_GObj*, int, float frame);
} ftData_UnkCallbackPairs0[Ft_Kind_Max] = {
    { NULL, NULL },
    { NULL, NULL },
    { NULL, NULL },
    { NULL, NULL },
    { ftKb_Init_UnkCallbackPairs0_0, ftKb_Init_UnkCallbackPairs0_1 },
};

/// Costume and Joint Strings
Fighter_CostumeStrings* ftData_803C2360[Ft_Kind_Max] = {
    ftMr_Init_CostumeStrings, ftFx_Init_CostumeStrings,
    ftCa_Init_CostumeStrings, ftDk_Init_CostumeStrings,
    ftKb_Init_CostumeStrings, ftKp_Init_CostumeStrings,
    ftLk_Init_CostumeStrings, ftSk_Init_CostumeStrings,
    ftNs_Init_CostumeStrings, ftPe_Init_CostumeStrings,
    ftPp_Init_CostumeStrings, ftNn_Init_CostumeStrings,
    ftPk_Init_CostumeStrings, ftSs_Init_CostumeStrings,
    ftYs_Init_CostumeStrings, ftPr_Init_CostumeStrings,
    ftMt_Init_CostumeStrings, ftLg_Init_CostumeStrings,
    ftMs_Init_CostumeStrings, ftZd_Init_CostumeStrings,
    ftCl_Init_CostumeStrings, ftDr_Init_CostumeStrings,
    ftFc_Init_CostumeStrings, ftPc_Init_CostumeStrings,
    ftGw_Init_CostumeStrings, ftGn_Init_CostumeStrings,
    ftFe_Init_CostumeStrings, ftMh_Init_CostumeStrings,
    ftCh_Init_CostumeStrings, ftBo_Init_CostumeStrings,
    ftGl_Init_CostumeStrings, ftGk_Init_CostumeStrings,
    ftSb_Init_CostumeStrings,
};

char* ftData_803C23E4[Ft_Kind_Max] = {
    ftMr_Init_AnimDatFilename, ftFx_Init_AnimDatFilename,
    ftCa_Init_AnimDatFilename, ftDk_Init_AnimDatFilename,
    ftKb_Init_AnimDatFilename, ftKp_Init_AnimDatFilename,
    ftLk_Init_AnimDatFilename, ftSk_Init_AnimDatFilename,
    ftNs_Init_AnimDatFilename, ftPe_Init_AnimDatFilename,
    ftPp_Init_AnimDatFilename, ftNn_Init_AnimDatFilename,
    ftPk_Init_AnimDatFilename, ftSs_Init_AnimDatFilename,
    ftYs_Init_AnimDatFilename, ftPr_Init_AnimDatFilename,
    ftMt_Init_AnimDatFilename, ftLg_Init_AnimDatFilename,
    ftMs_Init_AnimDatFilename, ftZd_Init_AnimDatFilename,
    ftCl_Init_AnimDatFilename, ftDr_Init_AnimDatFilename,
    ftFc_Init_AnimDatFilename, ftPc_Init_AnimDatFilename,
    ftGw_Init_AnimDatFilename, ftGn_Init_AnimDatFilename,
    ftFe_Init_AnimDatFilename, ftMh_Init_AnimDatFilename,
    ftCh_Init_AnimDatFilename, ftBo_Init_AnimDatFilename,
    ftGl_Init_AnimDatFilename, ftGk_Init_AnimDatFilename,
    ftSb_Init_AnimDatFilename,
};

/// Demo Lookup Strings
Fighter_DemoStrings* ftData_803C2468[Ft_Kind_Max] = {
    &ftMr_Init_DemoMotionFilenames,
    &ftFx_Init_DemoMotionFilenames,
    &ftCa_Init_DemoMotionFilenames,
    &ftDk_Init_DemoMotionFilenames,
    &ftKb_Init_DemoMotionFilenames,
    &ftKp_Init_DemoMotionFilenames,
    &ftLk_Init_DemoMotionFilenames,
    &ftSk_Init_DemoMotionFilenames,
    &ftNs_Init_DemoMotionFilenames,
    &ftPe_Init_DemoMotionFilenames,
    &ftPp_Init_DemoMotionFilenames,
    &ftNn_Init_DemoMotionFilenames,
    &ftPk_Init_DemoMotionFilenames,
    &ftSs_Init_DemoMotionFilenames,
    &ftYs_Init_DemoMotionFilenames,
    &ftPr_Init_DemoMotionFilenames,
    &ftMt_Init_DemoMotionFilenames,
    &ftLg_Init_DemoMotionFilenames,
    &ftMs_Init_DemoMotionFilenames,
    &ftZd_Init_DemoMotionFilenames,
    &ftCl_Init_DemoMotionFilenames,
    &ftDr_Init_DemoMotionFilenames,
    &ftFc_Init_DemoMotionFilenames,
    &ftPc_Init_DemoMotionFilenames,
    &ftGw_Init_DemoMotionFilenames,
    &ftGn_Init_DemoMotionFilenames,
    &ftFe_Init_DemoMotionFilenames,
    NULL,
    NULL,
    NULL,
    NULL,
    &ftGk_Init_DemoMotionFilenames,
    NULL,
};

Fighter_MotionFileStringGetter ftData_803C24EC[Ft_Kind_Max] = {
    ftMr_Init_GetMotionFileString,
    NULL,
    NULL,
    NULL,
    ftKb_Init_GetMotionFileString,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    ftLg_Init_GetMotionFileString,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    ftGk_Init_GetMotionFileString,
    NULL,
};

Fighter_UnkPtrEvent ftData_UnkDemoCallbacks0[Ft_Kind_Max] = {
    ftMr_Init_UnkDemoCallbacks0,
    NULL,
    NULL,
    NULL,
    ftKb_Init_UnkDemoCallbacks0,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    ftLg_Init_UnkDemoCallbacks0,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    ftGk_Init_UnkDemoCallbacks0,
    NULL,
};

ftData_UnkCountStruct ftData_UnkIntPairs[Ft_Kind_Max] = {
    { 0, 16 }, { 0, 14 }, { 0, 14 }, { 0, 14 }, { 0, 18 }, { 0, 14 },
    { 0, 14 }, { 0, 14 }, { 0, 14 }, { 0, 14 }, { 0, 14 }, { 0, 14 },
    { 0, 14 }, { 0, 14 }, { 0, 14 }, { 0, 14 }, { 0, 14 }, { 0, 16 },
    { 0, 14 }, { 0, 14 }, { 0, 14 }, { 0, 14 }, { 0, 14 }, { 0, 14 },
    { 0, 14 }, { 0, 14 }, { 0, 14 }, { 0, 14 }, { 0, 14 }, { 0, 14 },
    { 0, 14 }, { 0, 15 }, { 0, 14 },
};

u8 ftData_UnkBytePerCharacter[Ft_Kind_Max] = {
    1,  3,  4,  8, 5, 12, 6, 17, 10, 15, 14, 14, 7,  2,  9,  11, 13,
    18, 16, 17, 6, 1, 3,  7, -1, 19, 49, -1, -1, -1, -1, 12, -1,
};

#if defined(TARGET_PC)
/* ---- m-ex fighters -----------------------------------------------------------------------------
 * Ported from m-ex (https://github.com/akaneia/m-ex): its patches make every per-fighter-kind
 * table of the engine read MxDt.dat instead of the retail arrays. The port sizes those arrays for
 * the m-ex kind slots (Ft_Kind_Mex0.., forward.h) and fills each slot's row ONCE, at fighter init,
 * from the disc's MxDt.dat:
 *   - callbacks: fighter_function[slot][internal] - each fighter's DEFAULT vanilla function per
 *     table (m-ex gives an added fighter its clone base's), mapped to native through the bridge.
 *     The table <-> slot pairing is verified against the retail rows (tools/mex_port, 26 of 46
 *     slots have a retail table). The fighter's own ftFunction overrides stay with the runtime's
 *     hooks (gw_mex_ftfunction_runtime.c), installed when its file loads (ftData_8008572C).
 *   - data: Pl file + ftData symbol, anim file and count, costumes, demo/results symbols.
 *   - tables m-ex does not describe: copied from the retail fighter it was cloned from (the one
 *     whose default onLoad it shares) - Kirby's copy tables, demo-motion count, effect file.
 * A disc without MxDt.dat leaves every slot empty. */
static Fighter_CostumeStrings ftData_MexCostumeStrings[Ft_Kind_Max - Ft_Kind_Mex0][16];

/* Whether a motion-table row (Fighter_WaitAnimData, main or demo table) of an m-ex fighter's file
 * is a real row: its name is NULL (a hole) or a string inside the archive, and its subaction
 * script is inside the archive (every row has one, holes too). The tables' lengths are not in the
 * file, and what follows a table is other data - Akaneia's Lucas puts his hand tables' part-index
 * bytes (0x0B0C0D0E...) right after the demo table - which fails this at once. */
static bool ftData_MexMotionRowOk(const Fighter_WaitAnimData* row, const u8* lo, const u8* hi)
{
    const u8* name = (const u8*) row->x0;
    const u8* cmd = (const u8*) row->xC;
    return (name == NULL || (name >= lo && name < hi)) && cmd != NULL && cmd >= lo && cmd < hi;
}

/* The animation flags an m-ex fighter's motion gets (ftData_8008572C explains why the low 6 bits,
 * the authoring kind, are rewritten at all). A kind below 64 is written as is - unchanged from
 * before the slot cap was raised. A kind of 64 or more does not fit 6 bits: its own motions get
 * FT_ANIM_KIND_SELF, which ftAnim_AuthorKind reads back as the fighter playing it, and a motion
 * authored for 0x21 - the generic thrown skeleton retail uses for what a VICTIM plays (Mario's
 * TMarioThrowF, and every m-ex fighter's T*Throw* rows) - keeps 0x21, as retail has it, because
 * "self" would be the victim, not this fighter. */
static u32 ftData_MexAnimFlags(u32 flags, int kind)
{
    if (kind < 64) {
        return (flags & ~0x3Fu) | (u32) kind;
    }
    if ((flags & 0x3Fu) == 0x21u) {
        return flags;
    }
    return (flags & ~0x3Fu) | (u32) FT_ANIM_KIND_SELF;
}
static UnkCostumeStruct ftData_MexCostumeLists[Ft_Kind_Max - Ft_Kind_Mex0][16];

void ftData_MexInitKinds(void)
{
    extern int Mex_SlotInternal(int slot);
    extern const char* Mex_FtPlFile(int k);
    extern const char* Mex_FtPlSymbol(int k);
    extern const char* Mex_FtAnimFile(int k);
    extern int Mex_FtAnimCount(int k);
    extern int Mex_FtCostumeCount(int k);
    extern const char* Mex_FtCostumeString(int k, int c, int which);
    extern void* Mex_FtDemoStrings(int k);
    extern void* Mex_FtFunc(int slot, int k);
    extern int Mex_FtBaseKind(int k);
    extern void Player_MexSetMapping(int ckind, int fkind);
    extern void ftKb_MexCopyKindData(int dst, int src, int internal);
    extern void ftKb_MexCopyKindHat(int dst, int src, int internal);
    static bool done;
    int slot, n = 0;

    if (done) {
        return;
    }
    done = true;
    for (slot = 0; slot < Ft_Kind_Max - Ft_Kind_Mex0; slot++) {
        FighterKind fk = Ft_Kind_Mex0 + slot;
        int k = Mex_SlotInternal(slot);
        int base, c, ncost;
        if (k < 0) {
            continue;
        }
        base = Mex_FtBaseKind(k);

        /* Each callback: m-ex's default for this fighter, or - when m-ex leaves the slot empty
         * because the fighter's own ftFunction overrides it - its clone base's. The engine
         * asserts several of these are non-NULL (ftKindCalcIndiviParamTable). */
        ftData_OnLoad[fk] = (HSD_GObjEvent) Mex_FtFunc(0, k);
        if (ftData_OnLoad[fk] == NULL) {
            ftData_OnLoad[fk] = ftData_OnLoad[base];
        }
        ftData_OnDeath[fk] = (HSD_GObjEvent) Mex_FtFunc(1, k);
        if (ftData_OnDeath[fk] == NULL) {
            ftData_OnDeath[fk] = ftData_OnDeath[base];
        }
        ftData_OnUserDataRemove[fk] = (HSD_GObjEvent) Mex_FtFunc(2, k);
        if (ftData_OnUserDataRemove[fk] == NULL) {
            ftData_OnUserDataRemove[fk] = ftData_OnUserDataRemove[base];
        }
        ftData_CharacterStateTables[fk] = (MotionState*) Mex_FtFunc(3, k);
        if (ftData_CharacterStateTables[fk] == NULL) {
            ftData_CharacterStateTables[fk] = ftData_CharacterStateTables[base];
        }
        ftData_SpecialN[fk] = (HSD_GObjEvent) Mex_FtFunc(4, k);
        if (ftData_SpecialN[fk] == NULL) {
            ftData_SpecialN[fk] = ftData_SpecialN[base];
        }
        ftData_SpecialAirN[fk] = (HSD_GObjEvent) Mex_FtFunc(5, k);
        if (ftData_SpecialAirN[fk] == NULL) {
            ftData_SpecialAirN[fk] = ftData_SpecialAirN[base];
        }
        ftData_SpecialS[fk] = (HSD_GObjEvent) Mex_FtFunc(6, k);
        if (ftData_SpecialS[fk] == NULL) {
            ftData_SpecialS[fk] = ftData_SpecialS[base];
        }
        ftData_SpecialAirS[fk] = (HSD_GObjEvent) Mex_FtFunc(7, k);
        if (ftData_SpecialAirS[fk] == NULL) {
            ftData_SpecialAirS[fk] = ftData_SpecialAirS[base];
        }
        ftData_SpecialHi[fk] = (HSD_GObjEvent) Mex_FtFunc(8, k);
        if (ftData_SpecialHi[fk] == NULL) {
            ftData_SpecialHi[fk] = ftData_SpecialHi[base];
        }
        ftData_SpecialAirHi[fk] = (HSD_GObjEvent) Mex_FtFunc(9, k);
        if (ftData_SpecialAirHi[fk] == NULL) {
            ftData_SpecialAirHi[fk] = ftData_SpecialAirHi[base];
        }
        ftData_SpecialLw[fk] = (HSD_GObjEvent) Mex_FtFunc(10, k);
        if (ftData_SpecialLw[fk] == NULL) {
            ftData_SpecialLw[fk] = ftData_SpecialLw[base];
        }
        ftData_SpecialAirLw[fk] = (HSD_GObjEvent) Mex_FtFunc(11, k);
        if (ftData_SpecialAirLw[fk] == NULL) {
            ftData_SpecialAirLw[fk] = ftData_SpecialAirLw[base];
        }
        ftData_OnAbsorb[fk] = (HSD_GObjEvent) Mex_FtFunc(12, k);
        if (ftData_OnAbsorb[fk] == NULL) {
            ftData_OnAbsorb[fk] = ftData_OnAbsorb[base];
        }
        ftData_OnItemPickupExt[fk] = (Fighter_ItemEvent) Mex_FtFunc(13, k);
        if (ftData_OnItemPickupExt[fk] == NULL) {
            ftData_OnItemPickupExt[fk] = ftData_OnItemPickupExt[base];
        }
        ftData_OnItemInvisible[fk] = (HSD_GObjEvent) Mex_FtFunc(14, k);
        if (ftData_OnItemInvisible[fk] == NULL) {
            ftData_OnItemInvisible[fk] = ftData_OnItemInvisible[base];
        }
        ftData_OnItemVisible[fk] = (HSD_GObjEvent) Mex_FtFunc(15, k);
        if (ftData_OnItemVisible[fk] == NULL) {
            ftData_OnItemVisible[fk] = ftData_OnItemVisible[base];
        }
        ftData_OnItemDropExt[fk] = (Fighter_ItemEvent) Mex_FtFunc(16, k);
        if (ftData_OnItemDropExt[fk] == NULL) {
            ftData_OnItemDropExt[fk] = ftData_OnItemDropExt[base];
        }
        ftData_OnItemPickup[fk] = (Fighter_ItemEvent) Mex_FtFunc(17, k);
        if (ftData_OnItemPickup[fk] == NULL) {
            ftData_OnItemPickup[fk] = ftData_OnItemPickup[base];
        }
        ftData_OnItemDrop[fk] = (Fighter_ItemEvent) Mex_FtFunc(18, k);
        if (ftData_OnItemDrop[fk] == NULL) {
            ftData_OnItemDrop[fk] = ftData_OnItemDrop[base];
        }
        ftData_UnkMotionStates1[fk] = (HSD_GObjEvent) Mex_FtFunc(19, k);
        if (ftData_UnkMotionStates1[fk] == NULL) {
            ftData_UnkMotionStates1[fk] = ftData_UnkMotionStates1[base];
        }
        ftData_UnkMotionStates2[fk] = (HSD_GObjEvent) Mex_FtFunc(20, k);
        if (ftData_UnkMotionStates2[fk] == NULL) {
            ftData_UnkMotionStates2[fk] = ftData_UnkMotionStates2[base];
        }
        ftData_OnKnockbackEnter[fk] = (HSD_GObjEvent) Mex_FtFunc(21, k);
        if (ftData_OnKnockbackEnter[fk] == NULL) {
            ftData_OnKnockbackEnter[fk] = ftData_OnKnockbackEnter[base];
        }
        ftData_OnKnockbackExit[fk] = (HSD_GObjEvent) Mex_FtFunc(22, k);
        if (ftData_OnKnockbackExit[fk] == NULL) {
            ftData_OnKnockbackExit[fk] = ftData_OnKnockbackExit[base];
        }
        ftData_UnkMotionStates3[fk] = (HSD_GObjEvent) Mex_FtFunc(23, k);
        if (ftData_UnkMotionStates3[fk] == NULL) {
            ftData_UnkMotionStates3[fk] = ftData_UnkMotionStates3[base];
        }
        ftData_UnkMotionStates4[fk] = (HSD_GObjEvent) Mex_FtFunc(24, k);
        if (ftData_UnkMotionStates4[fk] == NULL) {
            ftData_UnkMotionStates4[fk] = ftData_UnkMotionStates4[base];
        }
        ftKindCalcIndiviParamTable[fk] = (HSD_GObjEvent) Mex_FtFunc(25, k);
        if (ftKindCalcIndiviParamTable[fk] == NULL) {
            ftKindCalcIndiviParamTable[fk] = ftKindCalcIndiviParamTable[base];
        }
        ftData_UnkMtxFunc0[fk] = (Fighter_UnkMtxEvent) Mex_FtFunc(26, k);
        if (ftData_UnkMtxFunc0[fk] == NULL) {
            ftData_UnkMtxFunc0[fk] = ftData_UnkMtxFunc0[base];
        }
        ftData_803C24EC[fk] = (Fighter_MotionFileStringGetter) Mex_FtFunc(38, k);
        if (ftData_803C24EC[fk] == NULL) {
            ftData_803C24EC[fk] = ftData_803C24EC[base];
        }
        ftData_UnkDemoCallbacks0[fk] = (Fighter_UnkPtrEvent) Mex_FtFunc(39, k);
        if (ftData_UnkDemoCallbacks0[fk] == NULL) {
            ftData_UnkDemoCallbacks0[fk] = ftData_UnkDemoCallbacks0[base];
        }
        ftData_UnkMotionStates0[fk] = (MotionState*) Mex_FtFunc(40, k);
        if (ftData_UnkMotionStates0[fk] == NULL) {
            ftData_UnkMotionStates0[fk] = ftData_UnkMotionStates0[base];
        }

        ftData_803C1F40[fk].a = (char*) Mex_FtPlFile(k);
        ftData_803C1F40[fk].b = (char*) Mex_FtPlSymbol(k);
        ftData_803C23E4[fk] = (char*) Mex_FtAnimFile(k);
        ftData_Table_Unk0[fk].count = Mex_FtAnimCount(k);
        ftData_803C2468[fk] = (Fighter_DemoStrings*) Mex_FtDemoStrings(k);
        ncost = Mex_FtCostumeCount(k);
        if (ncost > 16) {
            ncost = 16;
        }
        for (c = 0; c < ncost; c++) {
            ftData_MexCostumeStrings[slot][c].dat_filename = (char*) Mex_FtCostumeString(k, c, 0);
            ftData_MexCostumeStrings[slot][c].joint_name = (char*) Mex_FtCostumeString(k, c, 1);
            ftData_MexCostumeStrings[slot][c].matanim_joint_name =
                (char*) Mex_FtCostumeString(k, c, 2);
        }
        ftData_803C2360[fk] = ftData_MexCostumeStrings[slot];
        CostumeListsForeachCharacter[fk].costume_list = ftData_MexCostumeLists[slot];
        CostumeListsForeachCharacter[fk].numCostumes = ncost;

        ftData_Table_Unk1[fk] = ftData_Table_Unk1[base];
        ftData_UnkMotionStates5[fk] = ftData_UnkMotionStates5[base];
        ftData_UnkIntPairs[fk].count = ftData_UnkIntPairs[base].count;
        /* Ported from m-ex (https://github.com/akaneia/m-ex).
         * Source patch: asm/m-ex/MnSlChrData - Effect ID Table/ (EffectID.asm, Fighter_LoadSync.asm).
         * Behaviour: a fighter's effect bank is MxDt's effect_index[internal], not the clone
         * base's - it holds the fighter's own models and generators (efasync.c, ids >= 5000).
         * 255 (none) or a bank the effect table has no file for keeps the clone base's. */
        {
            extern int Mex_FtEffectIndex(int k);
            extern const char* Mex_EffectString(int i, int which);
            int eb = Mex_FtEffectIndex(k);
            if (eb >= 0 && eb < EF_BANK_MAX && Mex_EffectString(eb, 0) != NULL) {
                ftData_UnkBytePerCharacter[fk] = (u8) eb;
            } else {
                if (eb >= EF_BANK_MAX && eb != 255) {
                    OSReport("gw: ERROR m-ex fighter %d's effect bank %d is past the %d the "
                             "particle system holds - it uses its clone base's effects\n",
                             k, eb, EF_BANK_MAX);
                }
                ftData_UnkBytePerCharacter[fk] = ftData_UnkBytePerCharacter[base];
            }
        }
        ftData_UnkCallbackPairs0[fk] = ftData_UnkCallbackPairs0[base];
        /* Kirby's copy ability for this fighter: its own hat archive, hat costumes, effect
         * bank and copied-special callbacks come from MxDt.dat, indexed by the m-ex INTERNAL
         * kind. Only the runtime costume-archive row still follows the clone base. */
        ftKb_MexCopyKindHat(fk, base, k);
        ftKb_MexCopyKindData(fk, base, k);
        Player_MexSetMapping(ChKind_Mex0 + slot, fk);
        n++;
    }
    /* Retail fighters the disc gives MORE costumes than retail (ACE's Mario has 7): append the
     * extra costumes' files after the retail ones, which stay exactly as they were. */
    for (slot = 0; slot < Ft_Kind_MasterH; slot++) {
        static Fighter_CostumeStrings strs[Ft_Kind_MasterH][16];
        static UnkCostumeStruct lists[Ft_Kind_MasterH][16];
        int ncost = Mex_FtCostumeCount(slot);
        int have = CostumeListsForeachCharacter[slot].numCostumes, c;
        if (ncost <= have || ftData_803C2360[slot] == NULL) {
            continue;
        }
        if (ncost > 16) {
            ncost = 16;
        }
        for (c = 0; c < ncost; c++) {
            if (c < have) {
                strs[slot][c] = ftData_803C2360[slot][c];
            } else {
                strs[slot][c].dat_filename = (char*) Mex_FtCostumeString(slot, c, 0);
                strs[slot][c].joint_name = (char*) Mex_FtCostumeString(slot, c, 1);
                strs[slot][c].matanim_joint_name = (char*) Mex_FtCostumeString(slot, c, 2);
            }
        }
        ftData_803C2360[slot] = strs[slot];
        CostumeListsForeachCharacter[slot].costume_list = lists[slot];
        CostumeListsForeachCharacter[slot].numCostumes = ncost;
    }
    if (n != 0) {
        OSReport("gw: %d m-ex fighter kinds from MxDt.dat (kinds %d..%d)\n", n,
                 Ft_Kind_Mex0, Ft_Kind_Mex0 + n - 1);
    }
}

/* Is `kind` an m-ex fighter with a row? */
bool ftData_IsMexKind(FighterKind kind)
{
    return kind >= Ft_Kind_Mex0 && kind < Ft_Kind_Max &&
           ftData_803C1F40[kind].a != NULL;
}
#endif

void ftData_80085560(int idx, int increment)
{
    ft_8045996C[idx] += increment;
    if (ft_8045996C[idx] < 0) {
        OSReport("fighter reference counter error!\n");
        HSD_ASSERT(1944, 0);
    }
}

char ftData_assert_msg_0[] = "cant get corps model array!\n";
char ftData_assert_msg_1[] = "HSD_ArchiveParse error!\n";

void ftData_800855C8(FighterKind kind, u8 color)
{
    int i;
    int lo;
    int hi;

#if defined(TARGET_PC)
    ftData_MexInitKinds();
#endif

    if (color != 0xFF &&
        color >= CostumeListsForeachCharacter[kind].numCostumes)
    {
        color = 0;
    }
    if (ftData_803C1F40[kind].a != NULL) {
        lbDvd_800178E8(2, ftData_803C1F40[kind].a, 4, 4, 0, 1, 4, 2, 0);
    }
    if (color == 0xFF) {
        lo = 0;
        hi = CostumeListsForeachCharacter[kind].numCostumes;
    } else {
        lo = color;
        hi = color + 1;
    }
    for (i = lo; i < hi; i++) {
        if (ftData_803C2360[kind][i].dat_filename != NULL) {
            lbDvd_800178E8(2, ftData_803C2360[kind][i].dat_filename, 4, 4, 0,
                           1, 3, 1, 0);
        }
    }
    if (ftData_UnkBytePerCharacter[kind] != (char) -1) {
        efAsync_LoadAsync(ftData_UnkBytePerCharacter[kind]);
    }
    if (ftData_803C23E4[kind] != NULL) {
        lbDvd_800178E8(1, ftData_803C23E4[kind], 5, 5, 0, 0, 1, 8, 0);
    }
}

void ftData_8008572C(FighterKind kind)
{
#if defined(TARGET_PC)
    ftData_MexInitKinds();
#endif
    if (gFtDataList[kind] == NULL) {
#if defined(TARGET_PC)
        /* captured for m-ex fighters, whose code is relocated inside this file */
        HSD_Archive* ftData_LoadedArchive = NULL;
        lbArchive_80017040(&ftData_LoadedArchive, ftData_803C1F40[kind].a,
                           &gFtDataList[kind], ftData_803C1F40[kind].b, 0);
#else
        lbArchive_80017040(NULL, ftData_803C1F40[kind].a, &gFtDataList[kind],
                           ftData_803C1F40[kind].b, 0);
#endif
#if defined(TARGET_PC)
        if (ftData_IsMexKind(kind)) {
            OSReport("gw: ftData_8008572C kind=%d file=%s sym=%s\n", kind,
                     ftData_803C1F40[kind].a, ftData_803C1F40[kind].b);
        }
        /* The wait-anim table's x10_animCurrFlags packs the FighterKind of the figatree it was
         * authored for into the low 6 bits (Fighter::x597_bits; the union's u32 bitfields are
         * allocated MSB-first on the big-endian PPC target, so the trailing 6-bit field lands in
         * bits 0-5). The figatree's authoring kind does not match the port's kind: on a vanilla
         * m-ex fighter's data is authored for m-ex's INTERNAL kind (Sonic: 31), never the port's
         * kind for it (Ft_Kind_Mex0 + slot). ftAnim_8006FE08/ftAnim_8006F954 then see fp->kind
         * != x597_bits and route the fighter down the cross-kind
         * remap path (ftAnim_8006FCE4 -> lbAnim_8001E7E8), which skips the "constant" track types
         * (5/6/7) and writes through a NULL FObj. Rewrite the low 6 bits to the port's own kind so
         * Sonic always uses the native animation path (ftAnim_8006F4C8 -> lbAnim_8001E6D8). Only
         * those 6 bits change; the parts-mask/flag bits stay as authored, which is correct because
         * ftParts_8007506C/ftPartsRemap now key off this kind too (ftCommonData_ExtendKindTable
         * hands each m-ex kind its own parts table). */
        if (ftData_IsMexKind(kind)) {
            ftData* fd = gFtDataList[kind];
            int i;
            /* The main motion table has the same problem as the demo table below: its count here
             * is the CLONE BASE's (ftData_Table_Unk0), and an m-ex fighter's own table can be
             * shorter (Akaneia's Charizard and Lucas run 1-2 rows past theirs). So the same two
             * bounds: the nearest data landmark (see the demo table's note below), and a row
             * that is not a motion row (ftData_MexMotionRowOk). Rows past that keep their flags
             * (the cross-kind path, which works). */
            {
                u8* arch_lo = (u8*) ftData_LoadedArchive->data;
                u8* arch_hi = arch_lo + ftData_LoadedArchive->header.data_size;
                u8* tab_lo = (u8*) fd->xC;
                u8* tab_hi = arch_hi;
                int w, n_main = 0, n_holes = 0;
                if ((u8*) fd > tab_lo && (u8*) fd < tab_hi) {
                    tab_hi = (u8*) fd;
                }
                for (w = 0; w < (int) (sizeof(ftData) / 4); w++) {
                    u8* p = (u8*) ((void**) fd)[w];
                    if (p > tab_lo && p < tab_hi && p >= arch_lo && p < arch_hi) {
                        tab_hi = p;
                    }
                }
                for (i = 0; i < ftData_Table_Unk0[kind].count; i++) {
                    u8* entry = (u8*) &fd->xC[i];
                    if (entry < arch_lo || entry + sizeof(fd->xC[0]) > tab_hi) {
                        break;
                    }
                    if (!ftData_MexMotionRowOk(&fd->xC[i], arch_lo, arch_hi)) {
                        /* An empty row (no name, clip or script) is a hole inside the table, not
                         * its end: retail Kirby's row 4, Fox's 4 and a dozen more are empty, so
                         * stopping there left every m-ex clone of them (Brawl Meta Knight) with 4
                         * rewritten rows and the rest played as the wrong kind's. Holes are
                         * skipped; any other non-motion row still ends the table. A walk over
                         * every fighter file on the ACE disc ends on each table's last row. */
                        const Fighter_WaitAnimData* row = &fd->xC[i];
                        if (row->x0 == NULL && row->xC == NULL && row->x4 == 0 && row->x8 == 0) {
                            n_holes++;
                            continue;
                        }
                        break;
                    }
                    fd->xC[i].x10_animCurrFlags =
                        (s32) ftData_MexAnimFlags((u32) fd->xC[i].x10_animCurrFlags, kind);
                    n_main++;
                }
                if (n_main + n_holes != ftData_Table_Unk0[kind].count) {
                    OSReport("gw: kind %d: motion table ends after %d entries (the base has %d); "
                             "the rest keep their authored flags\n",
                             kind, n_main + n_holes, ftData_Table_Unk0[kind].count);
                }
                OSReport("gw: kind %d: %d motion rows rewritten, %d empty rows skipped\n", kind,
                         n_main, n_holes);
            }
            /* Same for the demo motions (results-screen / intro poses, fd->x14): they are
             * authored for m-ex's kind too, and the cross-kind path crashed on the results
             * screen when Sonic won (user-found, ftDemo_CreateFighter -> lbAnim_8001E7E8).
             *
             * Nothing gives the length of THIS fighter's demo table: ftData_UnkIntPairs[kind]
             * .count is the clone base's, MxDt.dat has no equivalent, and an m-ex fighter's
             * table can be shorter. Walking the base's count overran Sonic's by four entries
             * and wrote the kind into whatever followed - for PlSn.dat that was the spring
             * article's ItemStateArray, whose zeroed anim_joint became 0x25 and crashed
             * HSD_JObjAddAnim the first time the spring spawned. So trust the DATA instead of
             * the count: every real motion has a subaction script (xC), including the holes
             * that have no figatree, and PlSn.dat's table is followed by structs whose xC word
             * is zero - so the first entry without one ends the table.
             *
             * That heuristic is NECESSARY BUT NOT SUFFICIENT, and Tails proved it (user-found,
             * crash in ftData_80085A14: "fighter figatree over! c000000"). PlTs.dat puts the
             * ftData struct itself directly after the demo table, and ftData+0x8 - which lands
             * on entry 14's xC word - is the parts table, a perfectly non-NULL pointer. So the
             * walk sailed past the end and entry 14's x10 field, which is ftData+0xC, IS
             * `fd->xC`: the rewrite replaced the pointer to the main animation table with
             * (xC & ~0x3F) | kind. ftData_80085A14 then read x8 through that mangled, unaligned
             * pointer on its very first iteration and got garbage. Dedede has the same layout
             * and was being corrupted the same way, two entries deep, without having crashed yet.
             *
             * So bound the walk by the DATA's own landmarks as well: every pointer field in the
             * ftData struct (all 24 words of it are pointers) that lands above the table marks
             * something that is not the table, and so does the struct itself. The nearest of
             * them is the furthest the table can possibly reach. Erring low is safe - a motion
             * whose flags are left alone still animates, through the cross-kind path - while
             * erring high corrupts whatever follows.
             *
             * And that was not sufficient either: Akaneia's Lucas puts his hand-animation tables
             * (ftData->x1C[n]: part-index bytes, then AnimJoint pointers) after the demo table,
             * and nothing points there from ftData directly. Row 15's flags word was hand set
             * 1's first AnimJoint pointer; (ptr & ~0x3F) | kind moved it 0x18 bytes into a node,
             * and picking up an item walked that as a tree into a garbage AObjDesc
             * (HSD_JObjLoadJoint AV via ftMr_Init_OnItemPickup -> ftAnim_80070904). A row must
             * now also look like a motion row (ftData_MexMotionRowOk): row 14 there is the
             * part-index bytes 0x0B0C0D0E, not a name pointer, and ends the table. */
            if (fd->x14 != NULL) {
                u8* arch_lo = (u8*) ftData_LoadedArchive->data;
                u8* arch_hi = arch_lo + ftData_LoadedArchive->header.data_size;
                u8* demo_lo = (u8*) fd->x14;
                u8* demo_hi = arch_hi;
                int n_demo = 0;
                int w;

                if ((u8*) fd > demo_lo && (u8*) fd < demo_hi) {
                    demo_hi = (u8*) fd;
                }
                for (w = 0; w < (int) (sizeof(ftData) / 4); w++) {
                    u8* p = (u8*) ((void**) fd)[w];
                    if (p > demo_lo && p < demo_hi && p >= arch_lo && p < arch_hi) {
                        demo_hi = p;
                    }
                }

                for (i = 0; i < ftData_UnkIntPairs[kind].count; i++) {
                    u8* entry = (u8*) &fd->x14[i];
                    u32 flags;
                    if (entry < arch_lo || entry + sizeof(fd->x14[0]) > arch_hi ||
                        entry + sizeof(fd->x14[0]) > demo_hi ||
                        !ftData_MexMotionRowOk(&fd->x14[i], arch_lo, arch_hi))
                    {
                        break;
                    }
                    flags = (u32) fd->x14[i].x10_animCurrFlags;
                    fd->x14[i].x10_animCurrFlags = (s32) ftData_MexAnimFlags(flags, kind);
                    n_demo++;
                }
                if (n_demo != ftData_UnkIntPairs[kind].count) {
                    OSReport("gw: kind %d has %d demo motions, not the base's %d\n", kind,
                             n_demo, ftData_UnkIntPairs[kind].count);
                    ftData_UnkIntPairs[kind].count = n_demo;
                }
            }
            /* Ported from m-ex (https://github.com/akaneia/m-ex): load Sonic's PlSn.dat
             * ftFunction PPC blob and install the onLoad override, so the engine's existing
             * onLoad dispatch (fighter.c -> Mex_OnLoadDispatch) runs Sonic's PPC onLoad through
             * the interpreter instead of Fox's vanilla entry. See gw_mex_ftfunction_runtime.c. */
            extern void Mex_FtFunctionInstall(int kind, void* arch_data, u32 arch_data_size);
            Mex_FtFunctionInstall((int) kind, ftData_LoadedArchive->data,
                                  ftData_LoadedArchive->header.data_size);
        }
#endif
    }
}

void ftData_8008578C(int arg0, u8 color)
{
    if (color != 0xFF &&
        color >= CostumeListsForeachCharacter[Ft_Kind_Kirby].numCostumes)
    {
        color = 0;
    }
    ftKb_SpecialN_800EEC34(
        arg0, color, CostumeListsForeachCharacter[Ft_Kind_Kirby].numCostumes);
}

void ftData_800857E0(FighterKind kind)
{
    if (ftData_UnkMotionStates5[kind] != NULL) {
        ftData_UnkMotionStates5[kind]();
    }
}

void ftData_80085820(FighterKind kind, int costume_id)
{
#if defined(TARGET_PC)
    UnkCostumeStruct* temp_r5;
    /* The port's per-costume runtime arrays have sixteen rows and the rows past this fighter's
     * count are zeroed, so a costume it does not have loads a NULL filename and dies inside the
     * DVD layer with nothing in the log that names the costume. Say what happened and use
     * costume 0, which every fighter has. MELEE_SCENE refuses an id above 15 at parse time; this
     * is the per-fighter half of the same check, and it is the only place that can make it. */
    if (costume_id < 0 ||
        costume_id >= (int) CostumeListsForeachCharacter[kind].numCostumes) {
        OSReport("gw: kind %d has no costume %d (%d costumes) - using 0\n", kind, costume_id,
                 (int) CostumeListsForeachCharacter[kind].numCostumes);
        costume_id = 0;
    }
    temp_r5 = &CostumeListsForeachCharacter[kind].costume_list[costume_id];
#else
    UnkCostumeStruct* temp_r5 =
        &CostumeListsForeachCharacter[kind].costume_list[costume_id];
#endif
#if defined(TARGET_PC)
    if (ftData_IsMexKind(kind)) {
        OSReport("gw: ftData_80085820 kind=%d costume=%d file=%s joint=%s\n",
                 kind, costume_id,
                 ftData_803C2360[kind][costume_id].dat_filename,
                 ftData_803C2360[kind][costume_id].joint_name);
    }
#endif
    if (temp_r5->joint == NULL) {
        if (ftData_803C2360[kind][costume_id].matanim_joint_name != NULL) {
            lbArchive_80017040(
                &temp_r5->x14_archive,
                ftData_803C2360[kind][costume_id].dat_filename, temp_r5,
                ftData_803C2360[kind][costume_id].joint_name, &temp_r5->x4,
                ftData_803C2360[kind][costume_id].matanim_joint_name, 0);
        } else {
            lbArchive_80017040(
                &temp_r5->x14_archive,
                ftData_803C2360[kind][costume_id].dat_filename, temp_r5,
                ftData_803C2360[kind][costume_id].joint_name, 0,
                ftData_803C2360[kind][costume_id].matanim_joint_name);
            CostumeListsForeachCharacter[kind].costume_list[costume_id].x4 =
                NULL;
        }
    }
}

void ftData_800858E4(FighterKind kind, int costume_id)
{
    UnkCostumeStruct* temp_r5 =
        &CostumeListsForeachCharacter[kind].costume_list[costume_id];
    if (temp_r5->joint == NULL) {
        if (ftData_803C2360[kind][costume_id].matanim_joint_name != NULL) {
            lbArchive_80017040(
                &temp_r5->x14_archive,
                ftData_803C2360[kind][costume_id].dat_filename, temp_r5,
                ftData_803C2360[kind][costume_id].joint_name, &temp_r5->x4,
                ftData_803C2360[kind][costume_id].matanim_joint_name, 0);
        } else {
            lbArchive_80017040(
                &temp_r5->x14_archive,
                ftData_803C2360[kind][costume_id].dat_filename, temp_r5,
                ftData_803C2360[kind][costume_id].joint_name, 0,
                ftData_803C2360[kind][costume_id].matanim_joint_name);
            CostumeListsForeachCharacter[kind].costume_list[costume_id].x4 =
                NULL;
        }
    }
}

void ftData_800859A8(Fighter* fp)
{
    HSD_GObj* gobj;
    s8 temp_r6 = fp->x61C;
    if (temp_r6 == -1) {
        return;
    }
    for (gobj = HSD_GObjPLinkHead[HSD_GOBJ_PLINK_FIGHTER]; gobj != NULL;
         gobj = gobj->next)
    {
        Fighter* cur_fp = GET_FIGHTER(gobj);
        if (fp != cur_fp && temp_r6 == cur_fp->x61C) {
            return;
        }
    }
    ft_8045993C[temp_r6].x6_b0 = false;
}

void ftData_80085A14(FighterKind kind)
{
    void* sp18;
    void* a_head;
    ftData* temp_r27 = gFtDataList[kind];
    u32 temp_r0;
    int i;
    u8 _[4];
    size_t sp10;

    PAD_STACK(4);

    if (ftData_Table_Unk0[kind].data == NULL) {
        lbFile_800168A0(1, ftData_803C23E4[kind], &sp18, &sp10);
        a_head = sp18;
        HSD_ASSERT(0x974, a_head);
        for (i = 0; i < (u32) ftData_Table_Unk0[kind].count; i++) {
            temp_r0 = temp_r27->xC[i].x8;
            if (temp_r0 != 0) {
#if defined(TARGET_PC)
                if (temp_r0 > 0x10000) { /* the buffer size, fighter.c (m-ex anims exceed 0x8000) */
#else
                if (temp_r0 > 0x8000) {
#endif
                    HSD_ASSERTREPORT(0x9AF, 0, "fighter figatree over! %x\n",
                                     temp_r0);
                }
                temp_r27->xC[i].x14 =
                    (uintptr_t) ((u8*) a_head + temp_r27->xC[i].x4);
            }
        }
        ftData_Table_Unk0[kind].data = a_head;
    }
}

void ftData_80085B10(Fighter* fp)
{
    FighterKind kind = fp->kind;
    fp->x59C = HSD_ObjAlloc(&fighter_x59C_alloc_data);
    fp->x5A0 = HSD_ObjAlloc(&fighter_x59C_alloc_data);
    fp->x5A4 = NULL;
    fp->x5A8 = NULL;
    fp->x58C = ftData_Table_Unk0[kind].count;
    ftData_80085A14(kind);
}

void ftData_80085B98(Fighter* fp, int arg1, int arg2)
{
    u32 temp_r30;
    int i;
    u32 temp_r0;
    struct Fighter_WaitAnimData* temp_r3;

    temp_r30 = (u32) ftData_UnkIntPairs[fp->kind].data;
    fp->x59C = HSD_ObjAlloc(&fighter_x59C_alloc_data);
    fp->x5A0 = HSD_ObjAlloc(&fighter_x59C_alloc_data);
    fp->x5A4 = 0;
    fp->x5A8 = 0;
    fp->x58C = ftData_UnkIntPairs[fp->kind].count;
    if (arg2 >= fp->x58C) {
        HSD_ASSERTREPORT(0x9D2, 0, "Demo Status error! %d\n", arg2);
    }
    if (temp_r30 != 0U) {
        for (i = arg1; i <= arg2; i++) {
            temp_r3 = &fp->ft_data->x14[i];
            temp_r0 = temp_r3->x8;
            if (temp_r3->x8 != 0U) {
                if (temp_r0 > 0xB000) {
                    HSD_ASSERTREPORT(0x9DC, 0, "fighter figatree over! %x\n",
                                     temp_r0);
                }
                temp_r3 = &fp->ft_data->x14[i];
                temp_r3->x14 = temp_r30 + temp_r3->x4;
            }
        }
        ftData_UnkIntPairs[fp->kind].data = 0;
    }
}

void ftData_80085CD8(Fighter* fp, Fighter* arg1, int msid)
{
    HSD_Archive sp14;
    Fighter* temp_r3_3;
    s32 temp_ret;
    s32 temp_ret_2;
    struct Fighter_x59C_t* temp_r4;
    struct Fighter_WaitAnimData* temp_r3;
    u32 temp_r3_2;
    u32 temp_r4_2;

    if (msid < arg1->x58C) {
        temp_r3 = (struct Fighter_WaitAnimData*) ftData_80085FD4(arg1, msid);
        temp_r3_2 = temp_r3->x14;
        if (temp_r3_2 != (u32) fp->x5A4) {
            if (temp_r3_2 != 0) {
                temp_r3_3 = ftData_80086060(fp);
                if ((temp_r3_3 != NULL) &&
                    (temp_r3->x14 == (u32) temp_r3_3->x5A4))
                {
                    memcpy(fp->x59C, temp_r3_3->x59C, temp_r3->x8);
                    temp_r4 = fp->x59C;
                    temp_ret = lbArchiveRelocate(
                        &sp14, temp_r4->x0, temp_r3->x8,
                        (intptr_t) temp_r4 - (intptr_t) temp_r3_3->x59C);
                    if (temp_ret == -1) {
                        HSD_ASSERTREPORT(
                            0x9FA, 0, "lbArchiveRelocate error! %x\n", msid);
                    }
                } else {
                    temp_r4_2 = temp_r3->x14;
                    if (temp_r4_2 < 0x80000000) {
                        lbArq_80014BD0(temp_r4_2, fp->x59C,
                                       OSRoundUp32B(temp_r3->x8), 0, 0);
                    } else {
                        memcpy(fp->x59C, (void*) temp_r4_2, temp_r3->x8);
                    }
                    temp_ret_2 =
                        HSD_ArchiveParse(&sp14, fp->x59C->x0, temp_r3->x8);
                    if (temp_ret_2 == -1) {
                        HSD_ASSERTREPORT(0xA0F, 0,
                                         "HSD_ArchiveParse error! %x\n", msid);
                    }
                }
                fp->x590 = HSD_ArchiveGetPublicAddress(&sp14, temp_r3->x0);
            } else {
                fp->x590 = NULL;
            }
            fp->x5A4 = (void*) temp_r3->x14;
        }
    }
}

FigaTree* ftData_80085E50(Fighter* arg0, int msid)
{
    HSD_Archive sp10;
    Fighter* temp_r3_3;
    int temp_ret;
    int temp_ret_2;
    struct Fighter_x59C_t* temp_r4;
    struct ftData_80085FD4_ret* temp_r3;
    u32 temp_r3_2;
    u32 temp_r4_2;

    if (msid < arg0->x58C) {
        temp_r3 = ftData_80085FD4(arg0, msid);
        temp_r3_2 = temp_r3->x14;
        if (temp_r3_2 != (u32) arg0->x5A8) {
            if (temp_r3_2 != 0) {
                temp_r3_3 = ftData_80086060(arg0);
                if ((temp_r3_3 != NULL) &&
                    (temp_r3->x14 == (u32) temp_r3_3->x5A4))
                {
                    memcpy(arg0->x59C, temp_r3_3->x59C, temp_r3->x8);
                    temp_r4 = arg0->x59C;
                    temp_ret = lbArchiveRelocate(
                        &sp10, temp_r4->x0, temp_r3->x8,
                        (intptr_t) temp_r4 - (intptr_t) temp_r3_3->x59C);
                    if (temp_ret == -1) {
                        HSD_ASSERTREPORT(
                            0xA30, 0, "lbArchiveRelocate error! %x\n", msid);
                    }
                } else {
                    temp_r4_2 = temp_r3->x14;
                    if (temp_r4_2 < 0x80000000) {
                        lbArq_80014BD0(temp_r4_2, arg0->x5A0,
                                       OSRoundUp32B(temp_r3->x8), 0, 0);
                    } else {
                        memcpy(arg0->x5A0, (void*) temp_r4_2, temp_r3->x8);
                    }
                    temp_ret_2 =
                        HSD_ArchiveParse(&sp10, arg0->x5A0->x0, temp_r3->x8);
                    if (temp_ret_2 == -1) {
                        HSD_ASSERTREPORT(0xA45, 0,
                                         "HSD_ArchiveParse error! %x\n", msid);
                    }
                }
                arg0->x598 = HSD_ArchiveGetPublicAddress(&sp10, temp_r3->x0);
            } else {
                arg0->x598 = 0;
            }
            arg0->x5A8 = (void*) temp_r3->x14;
        }
        return arg0->x598;
    }
    return NULL;
}

struct ftData_80085FD4_ret* ftData_80085FD4(Fighter* fp, int msid)
{
    if (fp->kind == Ft_Kind_Nana &&
        Player_GetPlayerSlotType(fp->player_id) != Gm_PKind_Demo &&
        fp->x24[msid].x14 == 0)
    {
        return (struct ftData_80085FD4_ret*) &gFtDataList[Ft_Kind_Popo]
            ->xC[msid];
    }
    return (struct ftData_80085FD4_ret*) &fp->x24[msid];
}

Fighter* ftData_80086060(Fighter* fp)
{
    if (fp->kind == Ft_Kind_Nana &&
        Player_GetPlayerSlotType(fp->player_id) != Gm_PKind_Demo)
    {
        Fighter_GObj* gobj = Player_GetEntityAtIndex(fp->player_id, 0);
        if (gobj != NULL) {
            return GET_FIGHTER(gobj);
        }
    }
    return NULL;
}
