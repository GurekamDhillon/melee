#ifndef MELEE_FT_INLINES_H
#define MELEE_FT_INLINES_H

#include <Runtime/platform.h>

#include <melee/ft/forward.h>
#include <melee/mp/forward.h>

#include <dolphin/mtx.h>
#include <melee/ef/eflib.h>
#include <melee/ft/ftanim.h>
#include <melee/ft/types.h>
#include <melee/it/it_26B1.h>
#include <melee/lb/lbvector.h>
#include <sysdolphin/baselib/archive.h>
#include <sysdolphin/baselib/dobj.h>
#include <sysdolphin/baselib/gobj.h>
#include <sysdolphin/baselib/lobj.h>

#define PUSH_ATTRS(fp, attributeName)                                         \
    do {                                                                      \
        void* backup = (fp)->dat_attrs_backup;                                \
        attributeName* src = (attributeName*) (fp)->ft_data->ext_attr;        \
        void** da = &(fp)->dat_attrs;                                         \
        *(attributeName*) (fp)->dat_attrs_backup = *src;                      \
        *da = backup;                                                         \
    } while (0)

/// @todo Remove declarations. Doesn't really need to be a macro.
#define COPY_ATTRS(gobj, attributeName)                                       \
    Fighter* fp = GET_FIGHTER(gobj);                                          \
    attributeName* sA2 = (attributeName*) fp->dat_attrs;                      \
    attributeName* ext_attr = (attributeName*) fp->ft_data->ext_attr;         \
    *sA2 = *ext_attr;

#ifdef M2C
#define GET_FIGHTER(gobj) ((Fighter*) HSD_GObjGetUserData((HSD_GObj*) gobj))
#else
#define GET_FIGHTER(gobj) ((Fighter*) HSD_GObjGetUserData(gobj))
#endif

#if defined(TARGET_PC)
/* The row a fighter's costume uses in the fighter's OWN per-costume tables.
 *
 * Retail these are the same number, and every such table is sized to the fighter's retail
 * costume count. An m-ex disc raises the count without extending the tables - Akaneia gives
 * every one of the 26 retail fighters two more costumes, and its PlFx.dat is byte-identical to
 * vanilla's in this region - so the raw id runs off the end of each of them. m-ex's answer is
 * costume_file[k][costume].visibility_lookup_idx: the retail costumes map to themselves and
 * every added one maps to 0, verified for all 26 fighters on both Akaneia and ACE
 * (tools/mex_port/dump_mxdt.py). Mex_CostumeVisIdx returns the raw id when there is no mexData
 * or no row, so a vanilla disc is unchanged.
 *
 * USE THIS FOR THE FIGHTER'S OWN DISC DATA ONLY. The port's own runtime arrays
 * (CostumeListsForeachCharacter[k].costume_list, ftData_803C2360[k]) are rebuilt with 16 rows by
 * ftData_MexInitKinds and must keep being indexed by the real costume id, or two costumes would
 * share one archive slot. */
int Mex_CostumeVisIdx(int fk, int costume);
#define FT_COSTUME_VIS_IDX(fp) Mex_CostumeVisIdx((fp)->kind, (fp)->x619_costume_id)
#else
#define FT_COSTUME_VIS_IDX(fp) ((fp)->x619_costume_id)
#endif

static inline void Fighter_SetEffectHitlagCallbacks(Fighter* fp)
{
    fp->pre_hitlag_cb = efLib_PauseAll;
    fp->post_hitlag_cb = efLib_ResumeAll;
}

/// @deprecated Use #GET_FIGHTER instead.
static inline Fighter* getFighter(Fighter_GObj* gobj)
{
    return gobj->user_data;
}

/// @deprecated use #GET_FIGHTER instead.
static inline Fighter* getFighterPlus(Fighter_GObj* gobj)
{
    Fighter* fp = gobj->user_data;
    return fp;
}

static inline void* getFtSpecialAttrs(Fighter* fp)
{
    void* fighter_attr = fp->dat_attrs;
    return fighter_attr;
}

static inline void* getFtSpecialAttrsD(Fighter* fp) // Direct
{
    return fp->dat_attrs;
}

static inline s32 ftGetKind(Fighter* fp)
{
    return fp->kind;
}

static inline s32 ftGetAction(Fighter* fp)
{
    return fp->motion_id;
}

static inline void* getFtSpecialAttrs2CC(Fighter* fp)
{
    void* fighter_attr = fp->x2CC;
    return fighter_attr;
}

static inline ftCo_DatAttrs* getFtAttrs(Fighter* fp)
{
    return &fp->co_attrs;
}

static inline CollData* getFtColl(Fighter* fp)
{
    return &fp->coll_data;
}

static inline Fighter_GObj* getFtVictim(Fighter* fp)
{
    return fp->victim_gobj;
}

static inline Item_GObj* getFtTargetItem(Fighter* fp)
{
    return fp->target_item_gobj;
}

static inline bool ftGetGroundAir(Fighter* fp)
{
    return fp->ground_or_air;
}

static inline int getStickDirX(Fighter* fp)
{
    if (fp->input.lstick[0].x < 0.0f) {
        return -1;
    } else {
        return +1;
    }
}

static inline float stickGetDir(float x1, float x2)
{
    if (x1 < x2) {
        return -x1;
    } else {
        return x1;
    }
}

static inline void getAccelAndTarget(Fighter* fp, float* accel,
                                     float* target_vel)
{
    ftCo_DatAttrs* co_attrs = &fp->co_attrs;
    *accel = fp->input.lstick[0].x * fp->co_attrs.dash_accel_mul;
    *accel += fp->input.lstick[0].x > 0 ? +co_attrs->dash_accel_base
                                        : -co_attrs->dash_accel_base;
    *target_vel = fp->input.lstick[0].x * co_attrs->dash_max_velocity;
}

/// used for all fighters except Kirby and Purin
static inline void Fighter_OnItemPickup(Fighter_GObj* gobj, bool catchItemFlag,
                                        bool bool2, bool bool3)
{
    Fighter* fp = GET_FIGHTER(gobj);
    if (!itIsHeavy(fp->item_gobj)) {
        switch (itGetHoldKind(fp->item_gobj)) {
        case 1:
            ftAnim_80070FB4(gobj, bool2, 1);
            break;
        case 2:
            ftAnim_80070FB4(gobj, bool2, 0);
            break;
        case 3:
            ftAnim_80070FB4(gobj, bool2, 2);
            break;
        case 4:
            ftAnim_80070FB4(gobj, bool2, 3);
            break;
        default:
            break;
        }
        if (catchItemFlag) {
            ftAnim_80070C48(gobj, bool3);
        }
    }
}

static inline void Fighter_OnItemInvisible(Fighter_GObj* gobj, bool flag)
{
    Fighter* fp = GET_FIGHTER(gobj);
    if (!itIsHeavy(fp->item_gobj)) {
        ftAnim_80070CC4(gobj, flag);
    }
}

static inline void Fighter_OnItemVisible(Fighter_GObj* gobj, bool flag)
{
    Fighter* fp = GET_FIGHTER(gobj);
    if (!itIsHeavy(fp->item_gobj)) {
        ftAnim_80070C48(gobj, flag);
    }
}

static inline void Fighter_OnItemDrop(Fighter_GObj* gobj, bool dropItemFlag,
                                      bool bool2, bool bool3)
{
    ftAnim_80070FB4(gobj, bool2, -1);
    if (dropItemFlag) {
        ftAnim_80070CC4(gobj, bool3);
    }
}

static inline void Fighter_OnKnockbackEnter(Fighter_GObj* gobj, s32 arg1)
{
    ftAnim_800704F0(gobj, arg1, 3.0f);
    ftAnim_800704F0(gobj, 0, 3.0f);
}

static inline void Fighter_OnKnockbackExit(Fighter_GObj* gobj, s32 arg1)
{
    ftAnim_800704F0(gobj, arg1, 0.0f);
    ftAnim_800704F0(gobj, 0, 0.0f);
}

static inline void Fighter_UnsetCmdVar0(Fighter_GObj* gobj)
{
    Fighter* fp = GET_FIGHTER(gobj);
    fp->cmd_vars[0] = 0;
}

static inline void Fighter_SetDamageCallback(Fighter_GObj* gobj,
                                             HSD_GObjEvent cb)
{
    Fighter* fp = GET_FIGHTER(gobj);
    fp->take_dmg_cb = cb;
    fp->death2_cb = cb;
}

static inline void Fighter_SetDamageCallbacks(Fighter* fp,
                                              HSD_GObjEvent take_dmg_cb,
                                              HSD_GObjEvent death2_cb)
{
    fp->take_dmg_cb = take_dmg_cb;
    fp->death2_cb = death2_cb;
}

static inline void Fighter_ClearCmdVars(Fighter* fp)
{
    fp->cmd_vars[3] = 0;
    fp->cmd_vars[2] = 0;
    fp->cmd_vars[1] = 0;
    fp->cmd_vars[0] = 0;
}

static inline CollData* Fighter_GetCollData(Fighter* fp)
{
    return &fp->coll_data;
}

static inline void ftCommon_HandleTeleportCollisions(Fighter_GObj* gobj,
                                                     Fighter* fp,
                                                     CollData* coll,
                                                     const int* angle_clamp,
                                                     HSD_GObjEvent on_collide)
{
    if ((coll->env_flags & Collide_CeilingMask) &&
        lbVector_AngleXY(&coll->ceiling.normal, &fp->self_vel) >
            MTXDegToRad(90.0f + *angle_clamp))
    {
        on_collide(gobj);
    }
    if ((coll->env_flags & Collide_LeftWallMask) &&
        lbVector_AngleXY(&coll->left_facing_wall.normal, &fp->self_vel) >
            MTXDegToRad(90.0f + *angle_clamp))
    {
        on_collide(gobj);
    }
    if ((coll->env_flags & Collide_RightWallMask) &&
        lbVector_AngleXY(&coll->right_facing_wall.normal, &fp->self_vel) >
            MTXDegToRad(90.0f + *angle_clamp))
    {
        on_collide(gobj);
    }
}

/// @todo This and #ftCheckThrowB3, etc. are probably one macro or something.
static inline bool ftCheckThrowB0(Fighter* fp)
{
    if (fp->throw_flags_b0) {
        fp->throw_flags_b0 = false;
        return true;
    } else {
        return false;
    }
}

static inline bool ftCheckThrowB3(Fighter* fp)
{
    if (fp->throw_flags_b3) {
        fp->throw_flags_b3 = false;
        return true;
    } else {
        return false;
    }
}

static inline bool ftCheckThrowB4(Fighter* fp)
{
    if (fp->throw_flags_b4) {
        fp->throw_flags_b4 = false;
        return true;
    } else {
        return false;
    }
}

static inline float ftGetFacingDir(Fighter_GObj* gobj)
{
    return GET_FIGHTER(gobj)->facing_dir;
}

static inline int ftGetFacingDirInt(Fighter* fp)
{
    if (fp->facing_dir < 0.0f) {
        return -1;
    } else {
        return +1;
    }
}

#endif
