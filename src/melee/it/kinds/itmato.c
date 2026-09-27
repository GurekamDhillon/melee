#include "itmato.h"

#include <Runtime/platform.h>

#include <dolphin/mtx.h>
#include <melee/gr/ground.h>
#include <melee/it/inlines.h>
#include <melee/it/itCommonItems.h>
#include <melee/it/item.h>
#include <melee/it/itzako.h>
#include <melee/it/types.h>
#include <melee/lb/lb_00B0.h>

ItemStateTable it_803F83E0[] = {
    {
        -1,
        itMato_UnkMotion0_Anim,
        itMato_UnkMotion0_Phys,
        itMato_UnkMotion0_Coll,
    },
};

void it_802D84F8(Item_GObj* item_gobj)
{
    Item* it = GET_ITEM(item_gobj);
    it_8027B730(item_gobj);
    it->facing_dir = 0;
    it->xD5C = 0;
    it->xDCC_flag.b3 = 0;
    it_802D8554(item_gobj);
}

void it_802D8554(Item_GObj* item_gobj)
{
    Item* it = GET_ITEM(item_gobj);
    it->x40_vel.x = it->x40_vel.y = 0;
    Item_80268E5C(item_gobj, 0, ITEM_ANIM_UPDATE);
}

bool itMato_UnkMotion0_Anim(Item_GObj* item_gobj)
{
    return false;
}

void itMato_UnkMotion0_Phys(Item_GObj* item_gobj)
{
    Item* it = GET_ITEM(item_gobj);
    if (it->xDD4_itemVar.mato.x0 != NULL) {
        Vec3 pos;
        lb_8000B1CC(it->xDD4_itemVar.mato.x0, NULL, &pos);
        it->pos = pos;
    }
#if defined(TARGET_PC)
    else {
        /* A scripted target (pc/gameworld/script_game.c) has no stage joint to follow: hold it at
         * its own spot, or the model-less article's item physics drops it onto the floor below. */
        extern int ScriptGame_TargetPin(Item_GObj*);
        ScriptGame_TargetPin(item_gobj);
    }
#endif
}

bool itMato_UnkMotion0_Coll(Item_GObj* item_gobj)
{
    return false;
}

bool it_802D85F4(Item_GObj* item_gobj)
{
#if defined(TARGET_PC)
    /* Scripted targets also use Mato's normal hit/destroy path, but are owned by the
     * Lua stage layer, not Ground_801C4210's Target Test counter. */
    extern int ScriptGame_TargetDestroyed(Item_GObj*);
    if (ScriptGame_TargetDestroyed(item_gobj)) return true;
#endif
    Ground_801C4338();
    return true;
}
