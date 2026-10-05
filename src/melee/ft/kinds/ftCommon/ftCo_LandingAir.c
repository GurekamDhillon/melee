#include "ftCo_LandingAir.h"

#include <Runtime/platform.h>

#include <melee/ft/forward.h>

#include "forward.h"
#include "ftCo_Landing.h"
#include <melee/ft/fighter.h>
#include <melee/ft/ftanim.h>
#include <melee/ft/ftcommon.h>
#include <melee/ft/types.h>

#if defined(TARGET_PC)
#include <gameworld/script_skill.h>
#endif

void ftCo_LandingAir_EnterWithLag(Fighter_GObj* gobj)
{
    u8 _[20] = { 0 };
    float lag;
    FtMotionId msid = ftCo_MS_None;
#if defined(TARGET_PC)
    float skill_lag0 = 0.0f;
    int skill_mode = -1; /* 1 L-cancelled, 0 missed, -1 auto-cancelled */
#endif
    Fighter* fp = GET_FIGHTER(gobj);
    if (fp->cmd_vars[0]) {
        switch (fp->motion_id) {
        case ftCo_MS_AttackAirN:
            lag = fp->co_attrs.landingairn_lag;
            msid = ftCo_MS_LandingAirN;
            break;
        case ftCo_MS_AttackAirF:
            lag = fp->co_attrs.landingairf_lag;
            msid = ftCo_MS_LandingAirF;
            break;
        case ftCo_MS_AttackAirB:
            lag = fp->co_attrs.landingairb_lag;
            msid = ftCo_MS_LandingAirB;
            break;
        case ftCo_MS_AttackAirHi:
            lag = fp->co_attrs.landingairhi_lag;
            msid = ftCo_MS_LandingAirHi;
            break;
        case ftCo_MS_AttackAirLw:
            msid = ftCo_MS_LandingAirLw;
            lag = fp->co_attrs.landingairlw_lag;
            break;
        }
#if defined(TARGET_PC)
        skill_lag0 = lag;
        skill_mode = 0;
#endif
        if (msid != ftCo_MS_None && fp->x67F < p_ftCommonData->xE4) {
#if defined(TARGET_PC)
            skill_mode = 1;
#endif
            float div_lag = lag / p_ftCommonData->xE8;
            int int_lag = div_lag;
            if ((int) div_lag == 0) {
                int_lag = 1;
            }
            lag = int_lag;
        }
    }
#if defined(TARGET_PC)
    /* Skill telemetry: the retail L-cancel decision above, read-only. */
    ScriptGame_SkillLanding(fp, msid != ftCo_MS_None ? skill_mode : -1, skill_lag0,
                           msid != ftCo_MS_None ? lag : 0.0f);
#endif
    if (msid != ftCo_MS_None) {
        ftCo_LandingAir_EnterWithMsidLag(gobj, msid, lag);
    } else {
        ftCo_Landing_Enter_Basic(gobj);
    }
}

void ftCo_LandingAir_EnterWithMsidLag(Fighter_GObj* gobj, FtMotionId msid,
                                      float lag)
{
    u8 _[8];
    ftCommon_8007D7FC(GET_FIGHTER(gobj));
    Fighter_ChangeMotionState(gobj, msid, Ft_MF_None, 0, 1, 0, NULL);
    ftAnim_SetAnimRate(gobj, (ftAnim_8006F484(gobj) + 0.1f) / lag);
}

void ftCo_LandingAir_Anim(Fighter_GObj* gobj)
{
    ftCo_Landing_Anim(gobj);
}

void ftCo_LandingAir_IASA(Fighter_GObj* gobj) {}

void ftCo_LandingAir_Phys(Fighter_GObj* gobj)
{
    ftCo_Landing_Phys(gobj);
}

void ftCo_LandingAir_Coll(Fighter_GObj* gobj)
{
    ftCo_Landing_Coll(gobj);
}
