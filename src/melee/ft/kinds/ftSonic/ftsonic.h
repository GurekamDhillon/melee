#ifndef MELEE_FT_CHARA_FTSONIC_H
#define MELEE_FT_CHARA_FTSONIC_H

#include <Runtime/platform.h>

#include <sysdolphin/baselib/forward.h>

#include <melee/ft/types.h>

/* Sonic's own DATA (model/anims/costumes), mounted on the Akaneia disc. Only the data lives here
 * -- Sonic's *behaviour* (moveset) stays Fox's for now, so there are no motion-state/special tables
 * in this file. See ftdata.c (the Ft_Kind_Sonic table slots) and the pc-port research notes. */

extern char ftSn_Init_DatFilename[];
extern char ftSn_Init_DataName[];
extern char ftSn_Init_AnimDatFilename[];
extern Fighter_CostumeStrings ftSn_Init_CostumeStrings[];
extern UnkCostumeStruct ftSn_CostumeList[7];

#endif
