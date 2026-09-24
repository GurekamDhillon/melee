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

/* ============================================================================================
 * Geno Lab: read-only inspection for the Lua Lab (docs/geno.md "Geno Lab"). Field numbers are
 * in script_lab.h, shared with gw_script.c. Nothing below writes game state except the two
 * cosmetic debug-draw switches, which the native side refuses during a netplay session.
 * ============================================================================================ */
#include <melee/cm/camera.h>
#include <melee/ft/ftparts.h>
#include <melee/lb/types.h>
#include <sysdolphin/baselib/cobj.h>
#include <sysdolphin/baselib/jobj.h>

#include "script_lab.h"

float ScriptGame_LabF(int slot, int field)
{
    Fighter* fp = script_fighter(slot);
    CollData* cd;
    if (fp == NULL) {
        return 0.0f;
    }
    cd = &fp->coll_data;
    switch (field) {
    case LAB_F_ANIM_RATE:
        return fp->frame_speed_mul;
    case LAB_F_HITSTUN:
        return fp->x221C_b6 ? fp->mv.co.damage.x0 : 0.0f;
    case LAB_F_KB_VX:
        return fp->x8c_kb_vel.x;
    case LAB_F_KB_VY:
        return fp->x8c_kb_vel.y;
    case LAB_F_SHIELD:
        return fp->shield_health;
    case LAB_F_ECB_TOP_X:
        return cd->cur_pos.x + cd->ecb.top.x;
    case LAB_F_ECB_TOP_Y:
        return cd->cur_pos.y + cd->ecb.top.y;
    case LAB_F_ECB_BOTTOM_X:
        return cd->cur_pos.x + cd->ecb.bottom.x;
    case LAB_F_ECB_BOTTOM_Y:
        return cd->cur_pos.y + cd->ecb.bottom.y;
    case LAB_F_ECB_LEFT_X:
        return cd->cur_pos.x + cd->ecb.left.x;
    case LAB_F_ECB_LEFT_Y:
        return cd->cur_pos.y + cd->ecb.left.y;
    case LAB_F_ECB_RIGHT_X:
        return cd->cur_pos.x + cd->ecb.right.x;
    case LAB_F_ECB_RIGHT_Y:
        return cd->cur_pos.y + cd->ecb.right.y;
    case LAB_F_GR_VEL:
        return fp->gr_vel;
    case LAB_F_KB_APPLIED:
        return fp->dmg.kb_applied;
    case LAB_F_Z:
        return fp->cur_pos.z;
    case LAB_F_SCALE:
        return fp->x34_scale.y;
    case LAB_F_CMD_TIMER:
        return fp->cmd_timer;
    case LAB_F_KB_LAST:
        return fp->dmg.x18d8.kb_applied1;
    }
    return 0.0f;
}

static int lab_joint_count(Fighter* fp)
{
    int n;
    if (fp->parts == NULL || ftPartsTable == NULL || ftPartsTable[fp->kind] == NULL) {
        return 0;
    }
    n = (int) ftPartsTable[fp->kind]->parts_num;
    return n < 0 ? 0 : n > MAX_FT_PARTS ? MAX_FT_PARTS : n;
}

static int lab_joint_index(Fighter* fp, HSD_JObj* jobj)
{
    int i, n = lab_joint_count(fp);
    if (jobj == NULL) {
        return -1;
    }
    for (i = 0; i < n; i++) {
        if (fp->parts[i].joint == jobj) {
            return i;
        }
    }
    return -1;
}

int ScriptGame_LabI(int slot, int field)
{
    Fighter* fp = script_fighter(slot);
    if (fp == NULL) {
        return -1;
    }
    switch (field) {
    case LAB_I_ANIM_ID:
        return (int) fp->anim_id;
    case LAB_I_NAME_KIND:
        return (int) FTKB_CANON_KIND(fp->kind);
    case LAB_I_INTANG_TIMER:
        return (int) fp->x1990;
    case LAB_I_INVINC_TIMER:
        return (int) fp->x1994;
    case LAB_I_BODY_STATE:
        return (int) fp->x1988;
    case LAB_I_TIMED_STATE:
        return (int) fp->x198C;
    case LAB_I_JUMPS_USED:
        return (int) fp->x1968_jumpsUsed;
    case LAB_I_MAX_JUMPS:
        return (int) fp->co_attrs.max_jumps;
    case LAB_I_WALLJUMPS_USED:
        return (int) fp->x1969_walljumpUsed;
    case LAB_I_IN_HITLAG:
        return fp->x2219_b5 ? 1 : 0;
    case LAB_I_IN_HITSTUN:
        return fp->x221C_b6 ? 1 : 0;
    case LAB_I_IASA:
        return fp->allow_interrupt ? 1 : 0;
    case LAB_I_LEDGE_COOLDOWN:
        return (int) fp->x2064_ledgeCooldown;
    case LAB_I_ECB_LOCK:
        return (int) fp->ecb_lock;
    case LAB_I_DRAW_FLAGS:
        return (int) fp->x21FC_flag.byte;
    case LAB_I_SUB:
        return fp->is_sub_fighter ? 1 : 0;
    case LAB_I_HIDDEN:
        return (fp->x221E_b5 || fp->invisible) ? 1 : 0;
    case LAB_I_KIND:
        return (int) fp->kind;
    case LAB_I_LR_AGE:
        return (int) fp->x67F;
    case LAB_I_JUMP_AGE:
        return (int) fp->x67E;
    case LAB_I_JOINTS:
        return lab_joint_count(fp);
    case LAB_I_HURTBOXES:
        return (int) fp->hurt_capsules_len;
    }
    return -1;
}

/* The fighter's animation symbol for its current subaction (e.g. "PlyKirby5K_Share_ACTION_Wait1_
 * figatree"): read from the fighter's own file, so it names m-ex and custom moves too. */
const char* ScriptGame_LabAnimSymbol(int slot)
{
    Fighter* fp = script_fighter(slot);
    if (fp == NULL || fp->x24 == NULL || (int) fp->anim_id < 0) {
        return NULL;
    }
    return fp->x24[fp->anim_id].x0;
}

static HitCapsule* lab_hit(Fighter* fp, int i)
{
    if (i >= 0 && i < 4) {
        return &fp->x914[i];
    }
    if (i == 4) {
        return &fp->x1064_thrownHitbox;
    }
    return NULL;
}

int ScriptGame_HitI(int slot, int i, int field)
{
    Fighter* fp = script_fighter(slot);
    HitCapsule* h;
    if (fp == NULL || (h = lab_hit(fp, i)) == NULL) {
        return -1;
    }
    switch (field) {
    case LAB_HI_STATE:
        /* the thrown hitbox keeps a stale state between throws; it is live only while it has
           an owner (the thrower) */
        if (i == 4 && h->owner == NULL) {
            return 0;
        }
        return (int) h->state;
    case LAB_HI_GROUP:
        return (int) h->x4;
    case LAB_HI_BONE:
        return lab_joint_index(fp, h->jobj);
    case LAB_HI_ANGLE:
        return h->kb_angle;
    case LAB_HI_KBG:
        return (int) h->x24;
    case LAB_HI_WBK:
        return (int) h->x28;
    case LAB_HI_BKB:
        return (int) h->x2C;
    case LAB_HI_ELEMENT:
        return (int) h->element;
    case LAB_HI_SHIELD_DMG:
        return h->x34;
    case LAB_HI_SFX_SEVERITY:
        return h->sfx_severity;
    case LAB_HI_SFX_KIND:
        return (int) h->sfx_kind;
    case LAB_HI_HIT_AIR:
        return h->x40_b2 ? 1 : 0;
    case LAB_HI_HIT_GROUND:
        return h->x40_b3 ? 1 : 0;
    case LAB_HI_CLANK:
        return h->x40_b0 ? 1 : 0;
    case LAB_HI_REBOUND:
        return h->x40_b1 ? 1 : 0;
    }
    return -1;
}

float ScriptGame_HitF(int slot, int i, int field)
{
    Fighter* fp = script_fighter(slot);
    HitCapsule* h;
    if (fp == NULL || (h = lab_hit(fp, i)) == NULL) {
        return 0.0f;
    }
    switch (field) {
    case LAB_HF_DAMAGE:
        return h->damage;
    case LAB_HF_SIZE:
        return h->scale;
    case LAB_HF_X:
        return h->x4C.x;
    case LAB_HF_Y:
        return h->x4C.y;
    case LAB_HF_Z:
        return h->x4C.z;
    case LAB_HF_PX:
        return h->x58.x;
    case LAB_HF_PY:
        return h->x58.y;
    case LAB_HF_PZ:
        return h->x58.z;
    case LAB_HF_OX:
        return h->b_offset.x;
    case LAB_HF_OY:
        return h->b_offset.y;
    case LAB_HF_OZ:
        return h->b_offset.z;
    }
    return 0.0f;
}

int ScriptGame_HurtI(int slot, int i, int field)
{
    Fighter* fp = script_fighter(slot);
    FighterHurtCapsule* u;
    if (fp == NULL || i < 0 || i >= (int) fp->hurt_capsules_len || i >= 15) {
        return -1;
    }
    u = &fp->hurt_capsules[i];
    switch (field) {
    case LAB_UI_STATE:
        return (int) u->capsule.state;
    case LAB_UI_BONE:
        return lab_joint_index(fp, u->capsule.bone);
    case LAB_UI_HEIGHT:
        return (int) u->height;
    case LAB_UI_GRABBABLE:
        return u->is_grabbable ? 1 : 0;
    }
    return -1;
}

float ScriptGame_HurtF(int slot, int i, int field)
{
    Fighter* fp = script_fighter(slot);
    FighterHurtCapsule* u;
    if (fp == NULL || i < 0 || i >= (int) fp->hurt_capsules_len || i >= 15) {
        return 0.0f;
    }
    u = &fp->hurt_capsules[i];
    switch (field) {
    case LAB_UF_AX:
        return u->capsule.a_pos.x;
    case LAB_UF_AY:
        return u->capsule.a_pos.y;
    case LAB_UF_AZ:
        return u->capsule.a_pos.z;
    case LAB_UF_BX:
        return u->capsule.b_pos.x;
    case LAB_UF_BY:
        return u->capsule.b_pos.y;
    case LAB_UF_BZ:
        return u->capsule.b_pos.z;
    case LAB_UF_SIZE:
        return u->capsule.scale;
    }
    return 0.0f;
}

/* A joint's world position: the translation column of the matrix the last HSD_JObjSetupMatrix
 * left in it (read as-is, never recomputed here, so reading cannot change the game). comp 0-2 =
 * x, y, z. */
float ScriptGame_JointF(int slot, int i, int comp)
{
    Fighter* fp = script_fighter(slot);
    HSD_JObj* j;
    if (fp == NULL || i < 0 || i >= lab_joint_count(fp) || comp < 0 || comp > 2) {
        return 0.0f;
    }
    j = fp->parts[i].joint;
    return j != NULL ? j->mtx[comp][3] : 0.0f;
}

/* The parent joint's index, -1 for the root or a joint outside the table, -2 for no joint. */
int ScriptGame_JointParent(int slot, int i)
{
    Fighter* fp = script_fighter(slot);
    HSD_JObj* j;
    if (fp == NULL || i < 0 || i >= lab_joint_count(fp)) {
        return -2;
    }
    j = fp->parts[i].joint;
    if (j == NULL) {
        return -2;
    }
    return lab_joint_index(fp, j->parent);
}

float ScriptGame_CameraF(int field)
{
    HSD_GObj* gobj = Camera_80030A50();
    HSD_CObj* c;
    if (gobj == NULL || (c = GET_COBJ(gobj)) == NULL) {
        return 0.0f;
    }
    if (field >= LAB_CAM_VIEW && field < LAB_CAM_VIEW + 12) {
        int k = field - LAB_CAM_VIEW;
        return c->view_mtx[k / 4][k % 4];
    }
    switch (field) {
    case LAB_CAM_OK:
        return 1.0f;
    case LAB_CAM_PROJ:
        return c->projection_type == PROJ_ORTHO     ? 2.0f
               : c->projection_type == PROJ_FRUSTUM ? 1.0f
                                                    : 0.0f;
    case LAB_CAM_P0:
        return c->projection_param.ortho.top; /* = perspective.fov */
    case LAB_CAM_P1:
        return c->projection_param.ortho.bottom; /* = perspective.aspect */
    case LAB_CAM_P2:
        return c->projection_param.ortho.left;
    case LAB_CAM_P3:
        return c->projection_param.ortho.right;
    case LAB_CAM_NEAR:
        return c->near;
    case LAB_CAM_FAR:
        return c->far;
    case LAB_CAM_VP_XMIN:
        return c->viewport.xmin;
    case LAB_CAM_VP_XMAX:
        return c->viewport.xmax;
    case LAB_CAM_VP_YMIN:
        return c->viewport.ymin;
    case LAB_CAM_VP_YMAX:
        return c->viewport.ymax;
    }
    return 0.0f;
}

/* Fighter.x21FC_flag, the develop-mode visualisation byte (ftdrawcommon.c ftDrawCommon_800805C8,
 * dbanim.c fn_CheckAnimationInfo). set != 0 writes `value` to every fighter object of the slot
 * (Nana too). Returns the byte before the write, -1 when the slot has no fighter. */
int ScriptGame_LabDebugDraw(int slot, int set, int value)
{
    Fighter* fp = script_fighter(slot);
    int old;
    if (fp == NULL) {
        return -1;
    }
    old = (int) fp->x21FC_flag.byte;
    if (set) {
        HSD_GObj* g;
        for (g = HSD_GObjPLinkHead[HSD_GOBJ_PLINK_FIGHTER]; g != NULL; g = g->next) {
            Fighter* f = GET_FIGHTER(g);
            if (f->player_id == fp->player_id) {
                f->x21FC_flag.byte = (u8) value;
            }
        }
    }
    return old;
}

/* The match camera's collision display (cm/camera.c: mpLib_8005A2DC and friends). `mask` picks
 * the LAB_STAGE_* bits to write from `value`; returns the bits now set (ZONES has no getter in the
 * decomp and reads back as 0). */
int ScriptGame_LabStageDraw(int mask, int value)
{
    int now = 0;
    if (Camera_80030A50() == NULL) {
        return -1;
    }
    if (mask & LAB_STAGE_COLL) {
        Camera_80030A60((value & LAB_STAGE_COLL) != 0);
    }
    if (mask & LAB_STAGE_TERRAIN) {
        Camera_80030B38((value & LAB_STAGE_TERRAIN) != 0);
    }
    if (mask & LAB_STAGE_LEDGES) {
        Camera_80030B64((value & LAB_STAGE_LEDGES) != 0);
    }
    if (mask & LAB_STAGE_POINTS) {
        Camera_80030B90((value & LAB_STAGE_POINTS) != 0);
    }
    if (mask & LAB_STAGE_ZONES) {
        Camera_80030A8C((value & LAB_STAGE_ZONES) != 0);
    }
    now |= Camera_80030A78() ? LAB_STAGE_COLL : 0;
    now |= Camera_80030B50() ? LAB_STAGE_TERRAIN : 0;
    now |= Camera_80030B7C() ? LAB_STAGE_LEDGES : 0;
    now |= Camera_80030BA8() ? LAB_STAGE_POINTS : 0;
    return now;
}

/* ---- Stage 2: subaction scripts, motions ---------------------------------------------------- */
#include <melee/ft/ftanim.h>
#include <melee/ft/ftcommon.h>

static MotionState* lab_motion_row(Fighter* fp, int msid)
{
    if (msid < 0) {
        return NULL;
    }
    if (msid >= 0x400) {
        /* a Geno v2 action state (pc/geno/geno_game_v2.inc) */
        extern MotionState* Geno_MotionRowIfAny(Fighter * fp, int msid);
        return Geno_MotionRowIfAny(fp, msid);
    }
    if (msid >= fp->x18) {
        return fp->x20_actionStateList != NULL ? &fp->x20_actionStateList[msid - fp->x18] : NULL;
    }
    return fp->x1C_actionStateList != NULL ? &fp->x1C_actionStateList[msid] : NULL;
}

/* The animation (subaction) index a motion plays, -2 when the motion has no row. The native side
 * bounds `msid` (the decomp's name tables) before asking. */
int ScriptGame_LabMotionAnim(int slot, int msid)
{
    Fighter* fp = script_fighter(slot);
    MotionState* ms;
    if (fp == NULL || (ms = lab_motion_row(fp, msid)) == NULL) {
        return -2;
    }
    return (int) ms->anim_id;
}

/* The number of common motion states (fp->x18): special states start here. */
int ScriptGame_LabCommonCount(int slot)
{
    Fighter* fp = script_fighter(slot);
    return fp != NULL ? (int) fp->x18 : -1;
}

/* The subaction script of animation `anim` (-1 = the one playing), as its guest address. Never
 * the live cursor: the start of the script, which the native side walks read-only. */
const void* ScriptGame_LabScript(int slot, int anim)
{
    Fighter* fp = script_fighter(slot);
    if (fp == NULL || fp->x24 == NULL) {
        return NULL;
    }
    if (anim < 0) {
        anim = (int) fp->anim_id;
    }
    if (anim < 0 || anim > 0x3FF) {
        return NULL;
    }
    return fp->x24[anim].xC;
}

const char* ScriptGame_LabAnimSymbolFor(int slot, int anim)
{
    Fighter* fp = script_fighter(slot);
    if (fp == NULL || fp->x24 == NULL || anim < 0 || anim > 0x3FF) {
        return NULL;
    }
    return fp->x24[anim].x0;
}

/* The playing animation's last frame. */
float ScriptGame_LabAnimEnd(int slot)
{
    Fighter* fp = script_fighter(slot);
    if (fp == NULL || (int) fp->anim_id < 0) {
        return 0.0f;
    }
    return ftAnim_8006F484(fp->gobj);
}

/* Offline only (gw_script.c refuses it in a session and calls it at a frame boundary): put the
 * fighter into motion `msid` from its first frame, the plain Fighter_ChangeMotionState a state's
 * entry function would make. `lift` > 0: airborne and that much higher first (aerials). Floats
 * arrive as bits. Returns 0, or -1 without a fighter / row. */
int ScriptGame_LabSetMotion(int slot, int msid, int rate_bits, int lift_bits)
{
    Fighter* fp = script_fighter(slot);
    MotionState* ms;
    union {
        int i;
        float f;
    } rate, lift;
    if (fp == NULL || (ms = lab_motion_row(fp, msid)) == NULL) {
        return -1;
    }
    rate.i = rate_bits;
    lift.i = lift_bits;
    if (lift.f > 0.0f) {
        /* an aerial: into the air `lift` units up first (ftCommon_8007D5D4, "become airborne"),
           or the next collision check lands it at once */
        if (fp->ground_or_air == GA_Ground) {
            ftCommon_8007D5D4(fp);
        }
        fp->cur_pos.y += lift.f;
        fp->self_vel.x = fp->self_vel.y = 0.0f;
    }
    if (msid >= 0x400) {
        /* a Geno state: its behaviour's own entry (pc/geno/geno_game_v2.inc), not a bare change */
        extern int Geno_LabEnterState(Fighter * fp, int s);
        return Geno_LabEnterState(fp, msid - 0x400);
    }
    Fighter_ChangeMotionState(fp->gobj, msid, 0, 0.0f, rate.f, 0.0f, NULL);
    return 0;
}

/* ---- attributes (ftCo_DatAttrs by decomp name, as the fighter has them now) ---------------- */
#define LAB_ATTR(field, is_int) { #field, (int) __builtin_offsetof(ftCo_DatAttrs, field), is_int }

static const struct {
    const char* name;
    int offset;
    int is_int;
} lab_attrs[] = {
    LAB_ATTR(walk_accel_mul, 0),
    LAB_ATTR(walk_accel_base, 0),
    LAB_ATTR(walk_max_vel, 0),
    LAB_ATTR(slow_walk_max, 0),
    LAB_ATTR(mid_walk_point, 0),
    LAB_ATTR(fast_walk_min, 0),
    LAB_ATTR(ground_friction, 0),
    LAB_ATTR(dash_initial_velocity, 0),
    LAB_ATTR(dash_accel_mul, 0),
    LAB_ATTR(dash_accel_base, 0),
    LAB_ATTR(dash_max_velocity, 0),
    LAB_ATTR(run_animation_scaling, 0),
    LAB_ATTR(max_run_brake_frames, 0),
    LAB_ATTR(ground_max_horizontal_velocity, 0),
    LAB_ATTR(jump_startup_time, 0),
    LAB_ATTR(jump_h_initial_velocity, 0),
    LAB_ATTR(jump_v_initial_velocity, 0),
    LAB_ATTR(ground_to_air_jump_momentum_multiplier, 0),
    LAB_ATTR(jump_h_max_velocity, 0),
    LAB_ATTR(hop_v_initial_velocity, 0),
    LAB_ATTR(air_jump_v_multiplier, 0),
    LAB_ATTR(air_jump_h_multiplier, 0),
    LAB_ATTR(max_jumps, 1),
    LAB_ATTR(gravity, 0),
    LAB_ATTR(terminal_velocity, 0),
    LAB_ATTR(air_drift_stick_mul, 0),
    LAB_ATTR(aerial_drift_base, 0),
    LAB_ATTR(air_drift_max, 0),
    LAB_ATTR(aerial_friction, 0),
    LAB_ATTR(fast_fall_velocity, 0),
    LAB_ATTR(air_max_horizontal_velocity, 0),
    LAB_ATTR(jab_2_input_window, 0),
    LAB_ATTR(jab_3_input_window, 0),
    LAB_ATTR(standing_turn_frames, 0),
    LAB_ATTR(weight, 0),
    LAB_ATTR(model_scaling, 0),
    LAB_ATTR(initial_shield_size, 0),
    LAB_ATTR(shield_break_initial_velocity, 0),
    LAB_ATTR(rapid_jab_window, 1),
    LAB_ATTR(clank_animation_length, 0),
    /* stage E: the landing lags the frame-data export reports */
    LAB_ATTR(normal_landing_lag, 0),
    LAB_ATTR(landingairn_lag, 0),
    LAB_ATTR(landingairf_lag, 0),
    LAB_ATTR(landingairb_lag, 0),
    LAB_ATTR(landingairhi_lag, 0),
    LAB_ATTR(landingairlw_lag, 0),
};
#define LAB_NATTRS ((int) (sizeof(lab_attrs) / sizeof(lab_attrs[0])))

int ScriptGame_LabAttrCount(void)
{
    return LAB_NATTRS;
}

const char* ScriptGame_LabAttrName(int i)
{
    return i >= 0 && i < LAB_NATTRS ? lab_attrs[i].name : NULL;
}

/* The value as a float (ints converted); 0 without a fighter. */
float ScriptGame_LabAttrF(int slot, int i)
{
    Fighter* fp = script_fighter(slot);
    u8* base;
    if (fp == NULL || i < 0 || i >= LAB_NATTRS) {
        return 0.0f;
    }
    base = (u8*) &fp->co_attrs + lab_attrs[i].offset;
    if (lab_attrs[i].is_int) {
        return (float) *(s32*) base;
    }
    return *(f32*) base;
}

/* ---- Lab: the fighter's draw list (DObjs, their materials' TObjs, the model-part states) ---------
 * Read-only, like the joints: what the renderer will draw this frame. `d` indexes fp->dobj_list (the
 * part-visibility tables' DObj numbering), `t` the DObj's MObj TObj chain. docs/geno.md "Geno Lab". */
#include <sysdolphin/baselib/aobj.h>
#include <sysdolphin/baselib/dobj.h>
#include <sysdolphin/baselib/mobj.h>
#include <sysdolphin/baselib/tobj.h>

static HSD_DObj* lab_dobj(Fighter* fp, int d)
{
    if (fp == NULL || d < 0 || d >= (int) fp->dobj_list.count || fp->dobj_list.data == NULL) {
        return NULL;
    }
    return fp->dobj_list.data[d];
}

static HSD_TObj* lab_tobj(HSD_DObj* dobj, int t)
{
    HSD_TObj* tp;
    if (dobj == NULL || dobj->mobj == NULL || t < 0) {
        return NULL;
    }
    for (tp = dobj->mobj->tobj; tp != NULL && t > 0; tp = tp->next) {
        t--;
    }
    return tp;
}

/* field: LAB_DI_* */
int ScriptGame_LabDObjI(int slot, int d, int field)
{
    Fighter* fp = script_fighter(slot);
    HSD_DObj* dobj;
    HSD_TObj* tp;
    int n = 0;
    if (fp == NULL) {
        return -1;
    }
    switch (field) {
    case LAB_DI_COUNT:
        return (int) fp->dobj_list.count;
    case LAB_DI_MODELS:
        return (int) fp->x5AC.model_num;
    case LAB_DI_MODEL_STATE:
        return d >= 0 && d < 12 ? (int) fp->x5F4_arr[d].idx : -1;
    case LAB_DI_COSTUME_TOBJS:
        return (int) fp->tobj_list.n_costume_tobjs;
    }
    dobj = lab_dobj(fp, d);
    if (dobj == NULL) {
        return -1;
    }
    switch (field) {
    case LAB_DI_FLAGS:
        return (int) dobj->flags;
    case LAB_DI_RENDER:
        return dobj->mobj != NULL ? (int) dobj->mobj->rendermode : 0;
    case LAB_DI_TOBJS:
        for (tp = dobj->mobj != NULL ? dobj->mobj->tobj : NULL; tp != NULL; tp = tp->next) {
            n++;
        }
        return n;
    }
    return -1;
}

/* field: LAB_TF_*; d = -1 reads costume TObj t (fp->tobj_list, the eye texture anims) */
float ScriptGame_LabTObjF(int slot, int d, int t, int field)
{
    Fighter* fp = script_fighter(slot);
    HSD_TObj* tp;
    if (fp == NULL) {
        return 0.0f;
    }
    if (d < 0) {
        tp = t >= 0 && t < (int) fp->tobj_list.n_costume_tobjs && t < 5 ? fp->tobj_list.costume_tobjs[t] : NULL;
    } else {
        tp = lab_tobj(lab_dobj(fp, d), t);
    }
    if (tp == NULL) {
        return -1.0f;
    }
    switch (field) {
    case LAB_TF_ID:
        return (float) tp->id;
    case LAB_TF_SRC:
        return (float) tp->src;
    case LAB_TF_FLAGS:
        return (float) (tp->flags & 0x7FFFFFFF);
    case LAB_TF_TU:
        return tp->translate.x;
    case LAB_TF_TV:
        return tp->translate.y;
    case LAB_TF_SU:
        return tp->scale.x;
    case LAB_TF_SV:
        return tp->scale.y;
    case LAB_TF_FRAME:
        return tp->aobj != NULL ? tp->aobj->curr_frame : -1.0f;
    case LAB_TF_FMT:
        return tp->imagedesc != NULL ? (float) tp->imagedesc->format : -1.0f;
    case LAB_TF_W:
        return tp->imagedesc != NULL ? (float) tp->imagedesc->width : -1.0f;
    case LAB_TF_H:
        return tp->imagedesc != NULL ? (float) tp->imagedesc->height : -1.0f;
    }
    return 0.0f;
}

/* ---- Stage E: the knockback preview (the game's own knockback functions) -------------------- */
#include <melee/ft/ftcoll.h>
#include <melee/ft/kinds/ftCommon/ftCo_Damage.h>
#include <melee/gm/gmvs.h>
#include <melee/gr/stage.h>

/* The knockback a hit would give the fighter in `slot` now: ftColl_80079AB0 (the fighter-hit
 * path of ftColl, with the stage factor and both players' attack / defense ratios), then
 * ftCo_Damage_CalcKnockback (crouch, ice, smash charge, Y scale, armour, the minimum). `pct_bits`
 * < 0 (as a float) uses the fighter's percent. The percent and the pending damage the formula reads
 * (x1830 / x1838) and kb_applied are set for the two calls and restored bit for bit before
 * returning, so the game sees no change. Offline reads only (gw_script.c). Floats arrive as bits.
 * `post` = 0 skips the second step (raw formula). Returns -1 without a fighter. */
float ScriptGame_LabKnockback(int slot, int attacker_slot, int dmg_bits, int kbg, int wbk, int bkb,
                              int pct_bits, int post)
{
    Fighter* fp = script_fighter(slot);
    Fighter* at = attacker_slot >= 0 ? script_fighter(attacker_slot) : NULL;
    HitCapsule hit = { 0 };
    union {
        int i;
        float f;
    } dmg, pct;
    float save_pct, save_tmp, save_kb, kb, atk;
    if (fp == NULL) {
        return -1.0f;
    }
    dmg.i = dmg_bits;
    pct.i = pct_bits;
    hit.damage = dmg.f;
    hit.unk_count = (u32) (dmg.f + 0.5f);
    hit.x24 = (u32) kbg;
    hit.x28 = (u32) wbk;
    hit.x2C = (u32) bkb;
    save_pct = fp->dmg.x1830_percent;
    save_tmp = fp->dmg.x1838_percentTemp;
    save_kb = fp->dmg.kb_applied;
    if (pct.f >= 0.0f) {
        fp->dmg.x1830_percent = pct.f;
    }
    fp->dmg.x1838_percentTemp = (float) hit.unk_count;
    atk = at != NULL ? Player_GetAttackRatio(at->player_id) : 1.0f;
    kb = ftColl_80079AB0(fp, &hit, hit.unk_count, gm_8016B248(), atk,
                         Player_GetDefenseRatio(fp->player_id), fp->co_attrs.weight);
    if (post) {
        fp->dmg.kb_applied = kb;
        ftCo_Damage_CalcKnockback(fp);
        kb = fp->dmg.kb_applied;
    }
    fp->dmg.x1830_percent = save_pct;
    fp->dmg.x1838_percentTemp = save_tmp;
    fp->dmg.kb_applied = save_kb;
    return kb;
}

/* ftCo_8008D8E8: the knockback level (0-3, 3 = tumble) of a knockback value, from its hitstun. */
int ScriptGame_LabKbLevel(int kb_bits)
{
    union {
        int i;
        float f;
    } kb;
    kb.i = kb_bits;
    return (int) ftCo_8008D8E8(kb.f * p_ftCommonData->x154);
}

/* The ftCommonData constants the launch uses (PlCo.dat as loaded), and the stage's blast zones. */
float ScriptGame_LabCommonF(int which)
{
    ftCommonData* d = p_ftCommonData;
    switch (which) {
    case LAB_C_KB_SPEED:
        return d->x100;
    case LAB_C_KB_MAX:
        return d->x108;
    case LAB_C_ANGLE_AIR_361:
        return d->x144_radians;
    case LAB_C_ANGLE_GROUND_MAX:
        return d->x148;
    case LAB_C_ANGLE_GROUND_KB0:
        return d->x14C;
    case LAB_C_ANGLE_GROUND_KB1:
        return d->x150;
    case LAB_C_HITSTUN_MUL:
        return d->x154;
    case LAB_C_DI_DEGREES:
        return d->x1A8;
    case LAB_C_KB_DECAY:
        return d->x204_knockbackFrameDecay;
    case LAB_C_SQUAT_MUL:
        return d->kb_squat_mul;
    case LAB_C_BLAST_LEFT:
        return Stage_GetBlastZoneLeftOffset();
    case LAB_C_BLAST_RIGHT:
        return Stage_GetBlastZoneRightOffset();
    case LAB_C_BLAST_TOP:
        return Stage_GetBlastZoneTopOffset();
    case LAB_C_BLAST_BOTTOM:
        return Stage_GetBlastZoneBottomOffset();
    case LAB_C_LCANCEL_WINDOW:
        return (float) d->xE4;
    case LAB_C_LCANCEL_DIV:
        return d->xE8;
    }
    return 0.0f;
}

/* The victim's own numbers the flight uses: gravity, terminal velocity, aerial friction,
 * facing, grounded. */
float ScriptGame_LabFlightF(int slot, int which)
{
    Fighter* fp = script_fighter(slot);
    if (fp == NULL) {
        return 0.0f;
    }
    switch (which) {
    case 0:
        return fp->co_attrs.gravity;
    case 1:
        return fp->co_attrs.terminal_velocity;
    case 2:
        return fp->co_attrs.aerial_friction;
    case 3:
        return fp->facing_dir;
    case 4:
        return fp->ground_or_air == GA_Ground ? 1.0f : 0.0f;
    case 5:
        return fp->co_attrs.weight;
    case 6:
        return fp->dmg.x1830_percent;
    }
    return 0.0f;
}

/* ---- Stage E: name a SyncTest mismatch inside a Fighter (the rollback visualiser) ---------- */
#define LAB_FF(name, field) { name, (int) __builtin_offsetof(Fighter, field), (int) sizeof(((Fighter*) 0)->field) }
static const struct {
    const char* name;
    int off, size;
} lab_ffields[] = {
    LAB_FF("kind", kind),
    LAB_FF("motion_id", motion_id),
    LAB_FF("anim_id", anim_id),
    LAB_FF("facing_dir", facing_dir),
    LAB_FF("x34_scale", x34_scale),
    LAB_FF("x44_mtx", x44_mtx),
    LAB_FF("x74_self_accel", x74_self_accel),
    LAB_FF("self_vel", self_vel),
    LAB_FF("x8c_kb_vel", x8c_kb_vel),
    LAB_FF("x98_atk_shield_kb", x98_atk_shield_kb),
    LAB_FF("cur_pos", cur_pos),
    LAB_FF("prev_pos", prev_pos),
    LAB_FF("pos_delta", pos_delta),
    LAB_FF("ground_or_air", ground_or_air),
    LAB_FF("gr_vel", gr_vel),
    LAB_FF("xF0_ground_kb_vel", xF0_ground_kb_vel),
    LAB_FF("input", input),
    LAB_FF("co_attrs", co_attrs),
    LAB_FF("coll_data", coll_data),
    LAB_FF("ecb_lock", ecb_lock),
    LAB_FF("cur_anim_frame", cur_anim_frame),
    LAB_FF("frame_speed_mul", frame_speed_mul),
    LAB_FF("x8B0", x8B0),
    LAB_FF("x914 (hitboxes)", x914),
    LAB_FF("xDF4", xDF4),
    LAB_FF("x1064_thrownHitbox", x1064_thrownHitbox),
    LAB_FF("hurt_capsules", hurt_capsules),
    LAB_FF("x1614", x1614),
    LAB_FF("dmg", dmg),
    LAB_FF("x1968_jumpsUsed", x1968_jumpsUsed),
    LAB_FF("x2064_ledgeCooldown", x2064_ledgeCooldown),
    LAB_FF("mv (state vars)", mv),
};
#define LAB_NFF ((int) (sizeof lab_ffields / sizeof lab_ffields[0]))

/* (player << 16) | offset when `va` lies inside a live Fighter struct, else -1 */
int ScriptGame_LabFighterAt(u32 va)
{
    HSD_GObj* g;
    for (g = HSD_GObjPLinkHead[HSD_GOBJ_PLINK_FIGHTER]; g != NULL; g = g->next) {
        Fighter* f = GET_FIGHTER(g);
        u32 base = (u32) (uintptr_t) f;
        if (f != NULL && va >= base && va < base + (u32) sizeof(Fighter)) {
            return ((int) f->player_id << 16) | (int) (va - base);
        }
    }
    return -1;
}

static int lab_ffield(int off)
{
    int i;
    for (i = 0; i < LAB_NFF; ++i) {
        if (off >= lab_ffields[i].off && off < lab_ffields[i].off + lab_ffields[i].size) {
            return i;
        }
    }
    return -1;
}
const char* ScriptGame_LabFighterFieldName(int off)
{
    int i = lab_ffield(off);
    return i >= 0 ? lab_ffields[i].name : NULL;
}
int ScriptGame_LabFighterFieldBase(int off)
{
    int i = lab_ffield(off);
    return i >= 0 ? lab_ffields[i].off : off;
}
