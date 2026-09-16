#include "ftsonic.h"

#include <Runtime/platform.h>

#include <melee/ft/forward.h>

/* Sonic's own DATA (model/anims/costumes), authored by the Akaneia build for fighter 031 and
 * mounted on the content-expanded disc. Only DATA lives in this file: the fighter .dat, the anim
 * .dat, and the seven costume .dats with their joint symbols. Sonic's *behaviour* (moveset,
 * specials, motion-state tables) intentionally stays Fox's for now -- that is a later phase that
 * runs Sonic's real PPC moveset. See ftdata.c for the Ft_Kind_Sonic table slots that reference
 * these symbols. */

/* 459B28 */ UnkCostumeStruct ftSn_CostumeList[7];

char ftSn_Init_DatFilename[] = "PlSn.dat";
char ftSn_Init_DataName[] = "ftDataSonic";
char ftSn_Init_AnimDatFilename[] = "PlSnAJ.dat";

char ftSn_Init_PlSnNrDat[] = "PlSnNr.dat";
char ftSn_Init_PlySonic5K_Share_joint[] = "PlySonic5K_Share_joint";
char ftSn_Init_PlySonic5K_Share_matanim_joint[] =
    "PlySonic5K_Share_matanim_joint";
char ftSn_Init_PlSnReDat[] = "PlSnRe.dat";
char ftSn_Init_PlySonic5KRe_Share_joint[] = "PlySonic5KRe_Share_joint";
char ftSn_Init_PlySonic5KRe_Share_matanim_joint[] =
    "PlySonic5KRe_Share_matanim_joint";
char ftSn_Init_PlSnGrDat[] = "PlSnGr.dat";
char ftSn_Init_PlySonic5KGr_Share_joint[] = "PlySonic5KGr_Share_joint";
char ftSn_Init_PlySonic5KGr_Share_matanim_joint[] =
    "PlySonic5KGr_Share_matanim_joint";
char ftSn_Init_PlSnYeDat[] = "PlSnYe.dat";
char ftSn_Init_PlySonic5KYe_Share_joint[] = "PlySonic5KYe_Share_joint";
char ftSn_Init_PlySonic5KYe_Share_matanim_joint[] =
    "PlySonic5KYe_Share_matanim_joint";
char ftSn_Init_PlSnBkDat[] = "PlSnBk.dat";
char ftSn_Init_PlySonic5KBk_Share_joint[] = "PlySonic5KBk_Share_joint";
char ftSn_Init_PlySonic5KBk_Share_matanim_joint[] =
    "PlySonic5KBk_Share_matanim_joint";
char ftSn_Init_PlSnOrDat[] = "PlSnOr.dat";
char ftSn_Init_PlySonic5KOr_Share_joint[] = "PlySonic5KOr_Share_joint";
char ftSn_Init_PlySonic5KOr_Share_matanim_joint[] =
    "PlySonic5KOr_Share_matanim_joint";
char ftSn_Init_PlSnWhDat[] = "PlSnWh.dat";
char ftSn_Init_PlySonic5KWh_Share_joint[] = "PlySonic5KWh_Share_joint";
char ftSn_Init_PlySonic5KWh_Share_matanim_joint[] =
    "PlySonic5KWh_Share_matanim_joint";

Fighter_CostumeStrings ftSn_Init_CostumeStrings[] = {
    { ftSn_Init_PlSnNrDat, ftSn_Init_PlySonic5K_Share_joint,
      ftSn_Init_PlySonic5K_Share_matanim_joint },
    { ftSn_Init_PlSnReDat, ftSn_Init_PlySonic5KRe_Share_joint,
      ftSn_Init_PlySonic5KRe_Share_matanim_joint },
    { ftSn_Init_PlSnGrDat, ftSn_Init_PlySonic5KGr_Share_joint,
      ftSn_Init_PlySonic5KGr_Share_matanim_joint },
    { ftSn_Init_PlSnYeDat, ftSn_Init_PlySonic5KYe_Share_joint,
      ftSn_Init_PlySonic5KYe_Share_matanim_joint },
    { ftSn_Init_PlSnBkDat, ftSn_Init_PlySonic5KBk_Share_joint,
      ftSn_Init_PlySonic5KBk_Share_matanim_joint },
    { ftSn_Init_PlSnOrDat, ftSn_Init_PlySonic5KOr_Share_joint,
      ftSn_Init_PlySonic5KOr_Share_matanim_joint },
    { ftSn_Init_PlSnWhDat, ftSn_Init_PlySonic5KWh_Share_joint,
      ftSn_Init_PlySonic5KWh_Share_matanim_joint },
};

//// End of File
