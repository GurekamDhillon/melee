#include "ft_0BF0.h"

#include "fighter.h"
#include "forward.h"
#include "ftparts.h"
#include "kinds/ftCommon/forward.h"
#include "kinds/ftFox/types.h"
#include "kinds/ftMario/ftmariospecialn.h"
#include "types.h"
#include <melee/it/kinds/itdrmariopill.h>
#include <melee/it/kinds/itfoxblaster.h>
#include <melee/it/kinds/itnessbat.h>

void ftCo_800BF034(Fighter_GObj* gobj)
{
    u8 _[8];
    Fighter* fp = GET_FIGHTER(gobj);
    Fighter_ChangeMotionState(gobj, ftCo_MS_DeadUpFallHitCameraIce, Ft_MF_None,
                              0, 1, 0, NULL);
    fp->x2219_b2 = true;
    fp->x2219_b1 = true;
    switch (fp->kind) {
    case Ft_Kind_Fox: {
        ftFox_DatAttrs* da = fp->dat_attrs;
        fp->item_gobj =
            it_802AE994(gobj, ftParts_GetBoneIndex(fp, FtPart_RThumbNb),
                        da->x20_FOX_BLASTER_GUN_ITKIND);
        return;
    }
    case Ft_Kind_Ness: {
        fp->item_gobj =
            it_802AD590(gobj, ftParts_GetBoneIndex(fp, FtPart_RThumbNb));
        return;
    }
    default:
        break;
    }
#if defined(TARGET_PC)
    /* m-ex onIntroL (slot 41), injected at 0x800BF0EC - the function's epilogue, so it runs
     * AFTER the per-kind work above. The Fox and Ness cases return early here while on hardware
     * they fall into the same epilogue; that difference cannot reach an m-ex fighter, because
     * the dispatch is per kind and neither Fox nor Ness can have an m-ex hook. */
    {
        extern void Mex_OnIntroLDispatch(int kind, void* gobj, void* vanilla);
        Mex_OnIntroLDispatch(fp->kind, gobj, NULL);
    }
#endif
}

void ftCo_800BF108(Fighter_GObj* gobj)
{
    u8 _[8] = { 0 };
    Fighter* fp = GET_FIGHTER(gobj);
    Fighter_ChangeMotionState(gobj, ftCo_MS_Sleep, Ft_MF_None, 0, 1, 0, NULL);
    fp->x2219_b2 = true;
    fp->x2219_b1 = true;
    switch (fp->kind) {
    case Ft_Kind_Fox: {
        ftFox_DatAttrs* da = fp->dat_attrs;
        fp->item_gobj =
            it_802AE994(gobj, ftParts_GetBoneIndex(fp, FtPart_RThumbNb),
                        da->x20_FOX_BLASTER_GUN_ITKIND);
        return;
    }
    case Ft_Kind_Ness: {
        fp->item_gobj =
            it_802AD590(gobj, ftParts_GetBoneIndex(fp, FtPart_RThumbNb));
        return;
    }
    case Ft_Kind_DrMario: {
        fp->item_gobj = itDrMarioPill_802C09C4(
            gobj, &fp->cur_pos, ftMr_SpecialN_VitaminRandom(gobj),
            It_Kind_DrMario_Vitamin, 2,
            ftParts_GetBoneIndex(fp, FtPart_RThumbNb), fp->facing_dir);
        return;
    }
    default:
        break;
    }
}

bool ftCo_800BF228(Fighter_GObj* gobj)
{
    if (gobj != NULL) {
        Fighter* fp = GET_FIGHTER(gobj);
        if (fp != NULL && (fp->motion_id == ftCo_MS_DeadUpFallHitCameraIce ||
                           fp->motion_id == ftCo_MS_Sleep))
        {
            return true;
        }
    }
    return false;
}
