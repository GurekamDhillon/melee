/*
 * script_game.c - the game-side half of the Lua scripting API (pc/platform/gw_script.c).
 *
 * Game-world code: compiled for ppc32 and run through gwtool like the rest of melee, so the
 * fighter and player structs are read with the right byte order and nothing here swaps by hand.
 * The native side calls these as gw_ScriptGame_* (gwtool prefixes every game symbol) and only
 * ever passes and receives scalars, so no struct crosses the boundary.
 *
 * Fields are numbered (SCRIPT_F_* / SCRIPT_I_*); gw_script.c owns the names scripts see.
 */

#include <Runtime/platform.h>

#include <melee/ft/fighter.h>
#include <melee/ft/inlines.h>
#include <melee/ft/types.h>
#include <melee/gm/gm_1A3F.h>
#include <melee/gm/gmmain_lib.h>
#include <melee/gm/gmscene.h>
#include <melee/gr/ground.h>
#include <melee/gr/types.h>
#include <melee/pl/player.h>
#include <sysdolphin/baselib/gobj.h>

enum {
    SCRIPT_F_X = 0,
    SCRIPT_F_Y,
    SCRIPT_F_VX,
    SCRIPT_F_VY,
    SCRIPT_F_PERCENT,
    SCRIPT_F_FACING,
    SCRIPT_F_ANIM_FRAME,
    SCRIPT_F_HITLAG,
};

enum {
    SCRIPT_I_PRESENT = 0, /* 1 when the slot has a live fighter */
    SCRIPT_I_KIND,        /* internal FighterKind */
    SCRIPT_I_CHAR,        /* external CharacterKind (what the CSS picked) */
    SCRIPT_I_ACTION,      /* motion/action state id */
    SCRIPT_I_AIRBORNE,
    SCRIPT_I_STOCKS,
    SCRIPT_I_COSTUME,
    SCRIPT_I_SLOT_TYPE, /* 0 human, 1 cpu, 2 demo, 3 none */
};

/* A slot's fighter counts only while its gobj is in the live fighter list. The scene start clears
 * the player table (Player_ForgetEntities, gmscene.c) and a fighter freed mid-scene clears its
 * own slot (Fighter_Unload_8006DABC), so the table should never hold a freed fighter; this is the
 * second line, because every script read and write goes through here and a stale pointer means
 * reading freed memory (the 0x8B8B8B8B fill). At most six fighters, so the walk is cheap. */
static int script_gobj_live(HSD_GObj* gobj)
{
    HSD_GObj* cur;
    if (gobj == NULL || HSD_GObjPLinkHead == NULL) {
        return 0;
    }
    for (cur = HSD_GObjPLinkHead[HSD_GOBJ_PLINK_FIGHTER]; cur != NULL; cur = cur->next) {
        if (cur == gobj) {
            return 1;
        }
    }
    return 0;
}

static Fighter* script_fighter(int slot)
{
    HSD_GObj* gobj;
    if (slot < 0 || slot >= 6) {
        return NULL;
    }
    gobj = Player_GetEntity(slot);
    if (!script_gobj_live(gobj)) {
        return NULL;
    }
    return GET_FIGHTER(gobj);
}

float ScriptGame_FighterF(int slot, int field)
{
    Fighter* fp = script_fighter(slot);
    if (fp == NULL) {
        return 0.0f;
    }
    switch (field) {
    case SCRIPT_F_X:
        return fp->cur_pos.x;
    case SCRIPT_F_Y:
        return fp->cur_pos.y;
    case SCRIPT_F_VX:
        return fp->self_vel.x;
    case SCRIPT_F_VY:
        return fp->self_vel.y;
    case SCRIPT_F_PERCENT:
        return fp->dmg.x1830_percent;
    case SCRIPT_F_FACING:
        return fp->facing_dir;
    case SCRIPT_F_ANIM_FRAME:
        return fp->cur_anim_frame;
    case SCRIPT_F_HITLAG:
        return fp->dmg.x195c_hitlag_frames;
    }
    return 0.0f;
}

int ScriptGame_FighterI(int slot, int field)
{
    Fighter* fp;
    if (field == SCRIPT_I_SLOT_TYPE) {
        return (slot >= 0 && slot < 6) ? (int) Player_GetPlayerSlotType(slot) : 3;
    }
    fp = script_fighter(slot);
    if (fp == NULL) {
        return field == SCRIPT_I_PRESENT ? 0 : -1;
    }
    switch (field) {
    case SCRIPT_I_PRESENT:
        return 1;
    case SCRIPT_I_KIND:
        return (int) fp->kind;
    case SCRIPT_I_CHAR:
        return (int) Player_GetPlayerCharacter(slot);
    case SCRIPT_I_ACTION:
        return (int) fp->motion_id;
    case SCRIPT_I_AIRBORNE:
        return fp->ground_or_air == GA_Air ? 1 : 0;
    case SCRIPT_I_STOCKS:
        return (int) Player_GetStocks(slot);
    case SCRIPT_I_COSTUME:
        return (int) Player_GetCostumeId(slot);
    }
    return -1;
}

/* Gameplay writes: only for scripts whose manifest says "gameplay": true (gw_script.c checks). */
void ScriptGame_SetPercent(int slot, int percent)
{
    if (script_fighter(slot) != NULL) {
        Player_SetHUDDamage(slot, percent);
    }
}

void ScriptGame_SetStocks(int slot, int stocks)
{
    if (slot >= 0 && slot < 6) {
        Player_SetStocks(slot, stocks);
    }
}

int ScriptGame_StageKind(void)
{
    return (int) stage_info.grkind;
}

int ScriptGame_GameMode(void)
{
    return (int) gm_GetCurrentGameMode();
}

/* Scene launch at runtime: the native side has already set the scene text
 * (gw_SceneLaunch_SetText). Leaving the current scene for the target mode directly is NOT safe
 * from inside a match (tried: re-entering Training from Training skipped the mode's unload and
 * exhausted the heap in lbMemory_80014FC8), so this takes the console's own soft-reset path -
 * the one the reset switch triggers (gm_801A4014): the mode unwinds cleanly, the game re-enters
 * GM_BOOT, and the boot mode hands over to the configured scene exactly as it does for
 * MELEE_SCENE at start-up (gmboot.c). `game_mode` is only reported. */
void ScriptGame_LaunchScene(int game_mode)
{
    (void) game_mode;
    gmMainLib_8046B0F0.resetting = true;
    gm_801A4B60();
}
