#include "ftCo_SpecialAir.h"

#include <Runtime/platform.h>

#include <dolphin/mtx.h>
#include <melee/ft/fighter.h>
#include <melee/ft/ftcommon.h>
#include <melee/ft/ftdata.h>
#include <melee/ft/types.h>

bool ftCo_SpecialAir_CheckInput(Fighter_GObj* gobj)
{
    Fighter* fp = gobj->user_data;
    if (fp->input.pressed_buttons & HSD_PAD_B) {
        if (fp->input.lstick[0].y >= p_ftCommonData->x21C) {
            if (ftData_SpecialAirHi[fp->kind] == NULL) {
                return false;
            }
#if defined(TARGET_PC)
            {
                extern void Mex_SpecialHiAirDispatch(int kind, void* gobj, void* vanilla);
                Mex_SpecialHiAirDispatch(fp->kind, gobj, (void*) ftData_SpecialAirHi[fp->kind]);
            }
#else
            ftData_SpecialAirHi[fp->kind](gobj);
#endif
            fp->x2227_b5 = true;
            return true;
        }
        if (fp->input.lstick[0].y <= -p_ftCommonData->x21C) {
            if (ftData_SpecialAirLw[fp->kind] == NULL) {
                return false;
            }
#if defined(TARGET_PC)
            {
                extern void Mex_SpecialLwAirDispatch(int kind, void* gobj, void* vanilla);
                Mex_SpecialLwAirDispatch(fp->kind, gobj, (void*) ftData_SpecialAirLw[fp->kind]);
            }
#else
            ftData_SpecialAirLw[fp->kind](gobj);
#endif
            fp->x2227_b5 = true;
            return true;
        }
        if (ABS(fp->input.lstick[0].x) >= p_ftCommonData->x218) {
            if (ftData_SpecialAirS[fp->kind] == NULL) {
                return false;
            }
            if (fp->input.lstick[0].x * fp->facing_dir < -p_ftCommonData->x220)
            {
                ftCommon_UpdateFacing(fp);
            }
#if defined(TARGET_PC)
            {
                extern void Mex_SpecialSAirDispatch(int kind, void* gobj, void* vanilla);
                Mex_SpecialSAirDispatch(fp->kind, gobj, (void*) ftData_SpecialAirS[fp->kind]);
            }
#else
            ftData_SpecialAirS[fp->kind](gobj);
#endif
            fp->x2227_b5 = true;
            return true;
        }
        if (ftData_SpecialAirN[fp->kind] == NULL) {
            return false;
        }
        if (fp->x676_x < p_ftCommonData->x224 &&
            ((fp->facing_dir == -1 && fp->x2228_b7 == 1) ||
             (fp->facing_dir == +1 && fp->x2228_b7 == 0)))
        {
            fp->facing_dir = -fp->facing_dir;
        }
#if defined(TARGET_PC)
        {
            extern void Mex_SpecialNAirDispatch(int kind, void* gobj, void* vanilla);
            Mex_SpecialNAirDispatch(fp->kind, gobj, (void*) ftData_SpecialAirN[fp->kind]);
        }
#else
        ftData_SpecialAirN[fp->kind](gobj);
#endif
        fp->x2227_b5 = true;
        return true;
    }
    return false;
}
