#include <gameworld/profiler_game.h>
/* Reserve recipe informed by Joyastick's MeleeVS findings (research note and
 * CREDITS.md); independently implemented against our doldecomp functions.
 * Entire ownership/flag record is script_game.c BSS, included in snapshots. */
#include <melee/cm/types.h>
#include <melee/ft/ftcamera.h>
#include <melee/ft/ftcommon.h>
#include <melee/ft/ft_0892.h>
#include <melee/ft/ftcoll.h>
#include <melee/ft/ftwaitanim.h>
#include <melee/ft/kinds/ftCommon/ftCo_Fall.h>
#include <melee/ft/kinds/ftZelda/forward.h>
#include <melee/ft/kinds/ftSeak/forward.h>
typedef struct {
    Fighter* fighter;
    int dormant, frozen, intangible, invisible, hide1, hide2, ignore;
    struct CpuFighter cpu;
    int camera_state, camera_force, camera_timer, camera_framed;
} ScriptBenchEntity;
typedef struct { int owner; ScriptBenchEntity entity[2]; } ScriptBench;
static ScriptBench script_bench[6];

/* Codes are the native/Lua contract: no changes before the whole pair passes. */
static int script_bench_refused(Fighter* fp, int owned, int dormant)
{
    int ms;
    if (!fp || !fp->x890_cameraBox) return 1;
    if (dormant) return 0; /* retail's inactive Zelda/Sheik half, never awakened */
    ms=fp->motion_id;
    if (fp->kind==Ft_Kind_MasterH || fp->kind==Ft_Kind_CrezyH) return 2;
    if (ms>=ftCo_MS_DeadDown && ms<=ftCo_MS_RebirthWait) return 3;
    if (fp->victim_gobj || fp->x1A5C ||
        (ms>=ftCo_MS_CapturePulledHi && ms<=ftCo_MS_CaptureFoot) ||
        (ms>=ftCo_MS_ThrownF && ms<=ftCo_MS_ThrownlwWomen) ||
        (ms>=ftCo_MS_ShoulderedWait && ms<=ftCo_MS_ThrownCrazyHand)) return 4;
    if (fp->dmg.x195c_hitlag_frames>0 || (fp->x221F_b3 && !owned)) return 5;
    if ((fp->kind==Ft_Kind_Zelda && ms>=ftZd_MS_SpecialLw && ms<=ftZd_MS_SpecialAirLw2) ||
        (fp->kind==Ft_Kind_Seak && ms>=ftSk_MS_SpecialLw && ms<=ftSk_MS_SpecialAirLw2)) return 6;
    /* Interrupting a bespoke state can leave character-owned articles/callbacks.
     * Accept ordinary locomotion, wait, fall and teeter only. */
    if (fp->item_gobj || fp->x1984_heldItemSpec || !((ms>=ftCo_MS_Wait && ms<=ftCo_MS_FallAerialB) ||
          ms==ftCo_MS_Ottotto || ms==ftCo_MS_OttottoWait)) return 7;
    return 0;
}
static void script_bench_save(ScriptBenchEntity* e, Fighter* fp, int dormant)
{
    e->fighter=fp; e->dormant=dormant; e->cpu=fp->cpu;
    e->frozen=fp->x221F_b3; e->intangible=fp->x2219_b1;
    e->invisible=fp->invisible; e->hide1=fp->x221E_b1;
    e->hide2=fp->x221E_b2; e->ignore=fp->x221F_b1;
    e->camera_state=fp->x890_cameraBox->state;
    e->camera_force=fp->x890_cameraBox->force_inactive;
    e->camera_timer=fp->x890_cameraBox->state_timer;
    e->camera_framed=fp->x890_cameraBox->was_framed;
}
static void script_bench_assert(Fighter* fp)
{
    fp->x221F_b3=1; fp->cpu.kind=CpuKind_5; fp->invisible=1;
    fp->x2219_b1=1; fp->x221E_b1=1; fp->x221E_b2=1; fp->x221F_b1=1;
    fp->x890_cameraBox->state=CmSubjectState_Inactive;
    fp->x890_cameraBox->force_inactive=1;
}
static int script_bench_target_live(void* target, int plink)
{
    HSD_GObj* live;
    if (!target) return 1;
    if (!HSD_GObjPLinkHead) return 0;
    for (live=HSD_GObjPLinkHead[plink]; live; live=live->next)
        if (live->user_data==target) return 1;
    return 0;
}
static void script_bench_restore(ScriptBenchEntity* e, Fighter* fp)
{
    int lost=0;
    fp->cpu=e->cpu;
    /* External targets can despawn while the saved controller is paused. Never
     * resurrect a freed fighter/item through its cached plan. Internal buffer
     * pointers still refer to this same fighter's CPU memory, not the backup. */
    if (!script_bench_target_live(fp->cpu.x44,HSD_GOBJ_PLINK_FIGHTER)) { fp->cpu.x44=NULL; lost=1; }
    if (!script_bench_target_live(fp->cpu.xF0,HSD_GOBJ_PLINK_FIGHTER)) { fp->cpu.xF0=NULL; lost=1; }
    if (!script_bench_target_live(fp->cpu.x4C,HSD_GOBJ_PLINK_ITEM)) { fp->cpu.x4C=NULL; lost=1; }
    if (!script_bench_target_live(fp->cpu.xF4,HSD_GOBJ_PLINK_ITEM)) { fp->cpu.xF4=NULL; lost=1; }
    if (lost) {
        fp->cpu.xC8=fp->cpu.xEC=0;
        fp->cpu.csP=NULL; fp->cpu.command_duration=0;
        fp->cpu.write_pos=fp->cpu.buffer;
        fp->cpu.x18=fp->cpu.x1C; fp->cpu.x60=0;
        fp->cpu.xF8_b5=fp->cpu.xF8_b6=fp->cpu.xF8_b7=0;
        fp->cpu.xF9_b1=fp->cpu.xF9_b7=0;
    }
    fp->x221F_b3=e->frozen; fp->x2219_b1=e->intangible;
    fp->invisible=e->invisible; fp->x221E_b1=e->hide1;
    fp->x221E_b2=e->hide2; fp->x221F_b1=e->ignore;
    fp->x890_cameraBox->state=e->camera_state;
    fp->x890_cameraBox->force_inactive=e->camera_force;
    fp->x890_cameraBox->state_timer=e->camera_timer;
    fp->x890_cameraBox->was_framed=e->camera_framed;
}
/* cpu_mode initializes the whole controller, not just its kind. Preserve that
 * latest selection in the snapshot-owned record, then keep the reserve inert. */
static void script_bench_cpu_changed(Fighter* fp)
{
    int slot=fp->player_id, i;
    if (slot<0 || slot>=6 || !script_bench[slot].owner) return;
    for (i=0; i<2; ++i) if (script_bench[slot].entity[i].fighter==fp) {
        script_bench[slot].entity[i].cpu=fp->cpu;
        script_bench_assert(fp);
        return;
    }
}
static int script_bench_pair(int slot, Fighter** pair, int* dormant)
{
    HSD_GObj* gobj;
    pair[0]=script_fighter(slot); pair[1]=NULL; dormant[0]=dormant[1]=0;
    if (!pair[0] || pair[0]->is_sub_fighter) return 1;
    gobj=Player_GetEntityAtIndex(slot,1);
    if (gobj && gobj!=pair[0]->gobj) {
        if (!script_gobj_live(gobj)) return 1;
        pair[1]=GET_FIGHTER(gobj);
        if (!pair[1]) return 1;
        dormant[1]=(pair[0]->kind==Ft_Kind_Zelda || pair[0]->kind==Ft_Kind_Seak) &&
                   (pair[1]->kind==Ft_Kind_Zelda || pair[1]->kind==Ft_Kind_Seak) && pair[1]->x221F_b3;
        if (!pair[1]->is_sub_fighter && !dormant[1]) return 6;
    }
    return 0;
}
int ScriptGame_FighterBenchCheck(int slot, int owner, int calling)
{
    Fighter* pair[2]; int dormant[2], i, rc;
    ScriptBench* b;
    if (slot<0 || slot>=6 || owner<=0) return 1;
    b=&script_bench[slot];
    if (b->owner && b->owner!=owner) return 8;
    if (calling && !b->owner) return 9;
    rc=script_bench_pair(slot,pair,dormant); if (rc) return rc;
    for (i=0; i<2; ++i) {
        if (b->owner && b->entity[i].fighter!=pair[i]) return 10;
        if (pair[i]) {
            rc=script_bench_refused(pair[i],b->owner!=0,dormant[i]);
            if (rc) return rc;
        }
    }
    return 0;
}
int ScriptGame_FighterBenched(int slot)
{
    return slot>=0 && slot<6 && script_bench[slot].owner!=0;
}
/* Scalar read contract for Lua's second return: 0 present, 1 kind, 2 action,
 * 3 airborne, 4/5 x/y float bits, 6 CPU kind, 7 frozen, 8 invisible,
 * 9 intangible, 10 camera excluded, 11 dormant, 12 benched, 13 refusal code. */
static int script_bench_entity_read(Fighter* fp, int benched, int dormant, int field)
{
    if (!fp) return 0;
    switch (field) {
    case 0: return 1;
    case 1: return fp->kind;
    case 2: return fp->motion_id;
    case 3: return fp->ground_or_air==GA_Air;
    case 4: return script_mesh_bits(fp->cur_pos.x);
    case 5: return script_mesh_bits(fp->cur_pos.y);
    case 6: return fp->cpu.kind;
    case 7: return fp->x221F_b3;
    case 8: return fp->invisible;
    case 9: return fp->x2219_b1;
    case 10: return !fp->x890_cameraBox || fp->x890_cameraBox->force_inactive ||
                    fp->x890_cameraBox->state==CmSubjectState_Inactive;
    case 11: return dormant;
    case 12: return benched;
    case 13: return script_bench_refused(fp,benched,dormant);
    }
    return 0;
}
int ScriptGame_FighterReserveRead(int slot, int entity, int field)
{
    Fighter* fp; Fighter* primary; HSD_GObj* gobj;
    ScriptBench* b; int owned=0, dormant=0;
    if (slot<0 || slot>=6 || entity<0 || entity>1) return 0;
    primary=script_fighter(slot);
    if (!primary) return 0;
    if (!entity) fp=primary;
    else {
        gobj=Player_GetEntityAtIndex(slot,1);
        if (!gobj || gobj==primary->gobj || !script_gobj_live(gobj)) return 0;
        fp=GET_FIGHTER(gobj);
        if (!fp) return 0;
    }
    b=&script_bench[slot];
    if (b->owner && b->entity[entity].fighter==fp) {
        owned=1; dormant=b->entity[entity].dormant;
    } else if (entity) {
        dormant=(primary->kind==Ft_Kind_Zelda || primary->kind==Ft_Kind_Seak) &&
                (fp->kind==Ft_Kind_Zelda || fp->kind==Ft_Kind_Seak) && fp->x221F_b3;
    }
    return script_bench_entity_read(fp,owned,dormant,field);
}
int ScriptGame_FighterBench_profile_body(int slot, int owner);
int ScriptGame_FighterBench(int slot, int owner) {
    int result;
    PC_PROF_BEGIN(GW_PROF_BENCH_CALL, slot << 16);
    result = ScriptGame_FighterBench_profile_body(slot, owner);
    PC_PROF_END();
    return result;
}

int ScriptGame_FighterBench_profile_body(int slot, int owner)
{
    Fighter* pair[2]; int dormant[2], i, rc;
    ScriptBench* b;
    rc=ScriptGame_FighterBenchCheck(slot,owner,0); if (rc) return rc;
    b=&script_bench[slot];
    if (b->owner) return 0;
    script_bench_pair(slot,pair,dormant);
    for (i=0; i<2; ++i) if (pair[i]) {
        Fighter* fp=pair[i];
        script_bench_save(&b->entity[i],fp,dormant[i]);
        if (!dormant[i]) {
            ftCommon_8007E2FC(fp->gobj);
            if (fp->ground_or_air==GA_Ground) ft_8008A2BC(fp->gobj);
            else ftCo_Fall_Enter(fp->gobj);
        }
        script_bench_assert(fp);
    }
    b->owner=owner;
    OSReport("script reserve: benched port=%d owner=%d entities=%d\n",slot+1,owner,pair[1]?2:1);
    return 0;
}
/* Called before AI and physics as well as StageFrame: one-shot render writes
 * lose races to state entry. A stale/replaced entity is never dereferenced. */
int ScriptGame_FighterBenchTick(Fighter* fp)
{
    int slot=fp->player_id, i;
    if (slot<0 || slot>=6 || !script_bench[slot].owner) return 0;
    for (i=0; i<2; ++i) if (script_bench[slot].entity[i].fighter==fp) {
        script_bench_assert(fp); return 1;
    }
    return 0;
}
typedef struct {
    int (*floor)(float,float,Vec3*,int*,u32*,Vec3*);
    void (*fall)(Fighter*);
    void (*ground)(Fighter*);
    void (*wait)(HSD_GObj*);
    void (*camera)(Fighter*);
} ScriptBenchOps;
static int script_bench_floor(float x,float y,Vec3* p,int* line,u32* flags,Vec3* n)
{
    return mpCheckFloor(x,y+0.5f,x,y-0.5f,0,p,line,flags,n,-1,-1,-1,NULL,NULL);
}
static void script_bench_fall(Fighter* fp) { ftCo_Fall_Enter(fp->gobj); }
static const ScriptBenchOps script_bench_ops={script_bench_floor,script_bench_fall,
    ftCommon_8007D6A4,ft_8008A2BC,ftCamera_80076064};
static void script_bench_clean(Fighter* fp)
{
    ftCommon_8007E2FC(fp->gobj);
    fp->xF8_playerNudgeVel.x=fp->xF8_playerNudgeVel.y=0;
    memset(&fp->input,0,sizeof fp->input);
    Fighter_ResetInputData_80068854(fp->gobj);
    fp->cpu.buttons=0;
    fp->cpu.lstick.x=fp->cpu.lstick.y=0;
    fp->cpu.cstick.x=fp->cpu.cstick.y=0;
    fp->cpu.ltrigger=fp->cpu.rtrigger=0;
    fp->wall_jump_input_timer=254;
    fp->fall_fast=0;
}
static void script_bench_place(Fighter* fp, float x, float y, float facing, int intangible,
                               const ScriptBenchOps* ops)
{
    Vec3 pos=fp->cur_pos, normal;
    int floor=-1; u32 flags=0;
    int grounded=ops->floor(x,y,&pos,&floor,&flags,&normal);
    pos.x=x; if (!grounded) pos.y=y;
    fp->cur_pos=fp->prev_pos=pos;
    fp->facing_dir=facing;
    script_bench_clean(fp);
    /* Enter Fall resets callbacks and establishes air/ECB state. Only lock a
     * floor actually found at the call point, never the reserve's old floor. */
    ops->fall(fp);
    ftCommon_UnlockECB(fp);
    mpColl_80043680(&fp->coll_data,&fp->cur_pos);
    if (grounded) {
        fp->coll_data.floor.index=floor; fp->coll_data.floor.flags=flags;
        fp->coll_data.floor.normal=normal;
        ops->ground(fp);
        /* The animation helper 8008A7A8 is NOT a state entry: it leaves Fall's
         * gravity/collision callbacks installed even with ground_or_air=Ground. */
        ops->wait(fp->gobj);
    }
    /* Movement enter can set flags; restore reserve flags after it, at the caller. */
    if (intangible) ftColl_8007B760(fp->gobj,intangible);
    script_bench_clean(fp); /* callbacks may initialize velocity/input */
    ops->camera(fp);
}
int ScriptGame_FighterCall_profile_body(int slot,int owner,int xb,int yb,int fb,int intangible);
int ScriptGame_FighterCall(int slot,int owner,int xb,int yb,int fb,int intangible) {
    int result;
    PC_PROF_BEGIN(GW_PROF_BENCH_CALL, slot << 16);
    result = ScriptGame_FighterCall_profile_body(slot, owner, xb, yb, fb, intangible);
    PC_PROF_END();
    return result;
}

int ScriptGame_FighterCall_profile_body(int slot,int owner,int xb,int yb,int fb,int intangible)
{
    ScriptBench* b; int i, rc;
    float x=script_mesh_float(xb), y=script_mesh_float(yb), facing=script_mesh_float(fb);
    rc=ScriptGame_FighterBenchCheck(slot,owner,1); if (rc) return rc;
    if (!(x>=-49000 && x<=49000 && y>=-49000 && y<=49000) ||
        (facing!=1 && facing!=-1) || intangible<0 || intangible>600) return 11;
    b=&script_bench[slot];
    for (i=0; i<2; ++i) {
        ScriptBenchEntity* e=&b->entity[i]; Fighter* fp=e->fighter;
        if (!fp) continue;
        script_bench_restore(e,fp);
        if (!e->dormant) {
            script_bench_place(fp,x,y,facing,intangible,&script_bench_ops);
            /* Re-seeding deliberately activates the live subject. Preserve
             * special hide/control flags but do not restore its old camera anchor. */
            script_bench_restore(e,fp);
            script_bench_clean(fp); /* restoring the CPU also restores its outputs */
            ftCamera_80076064(fp);
            fp->x890_cameraBox->force_inactive=e->camera_force;
        } else {
            fp->cur_pos.x=x; fp->cur_pos.y=y; fp->prev_pos=fp->cur_pos;
            mpColl_80043680(&fp->coll_data,&fp->cur_pos);
            script_bench_clean(fp); fp->facing_dir=facing;
        }
    }
    memset(b,0,sizeof *b);
    OSReport("script reserve: called port=%d x=%g y=%g facing=%g intangible=%d\n",slot+1,x,y,facing,intangible);
    return 0;
}
void ScriptGame_FighterBenchRelease(int owner)
{
    int slot,i;
    for (slot=0; slot<6; ++slot) {
        ScriptBench* b=&script_bench[slot];
        if (!b->owner || (owner && owner!=b->owner)) continue;
        for (i=0; i<2; ++i) {
            ScriptBenchEntity* e=&b->entity[i];
            HSD_GObj* live;
            if (!e->fighter || !HSD_GObjPLinkHead) continue;
            for (live=HSD_GObjPLinkHead[HSD_GOBJ_PLINK_FIGHTER]; live; live=live->next) {
                if (GET_FIGHTER(live)!=e->fighter) continue;
                script_bench_restore(e,e->fighter); break;
            }
        }
        OSReport("script reserve: released port=%d owner=%d\n",slot+1,b->owner);
        memset(b,0,sizeof *b);
    }
}
static void script_bench_frame(void)
{
    HSD_GObj* gobj;
    if (!HSD_GObjPLinkHead) return;
    for (gobj=HSD_GObjPLinkHead[HSD_GOBJ_PLINK_FIGHTER]; gobj; gobj=gobj->next)
        if (gobj->user_data) ScriptGame_FighterBenchTick(GET_FIGHTER(gobj));
}
#include "script_fighter_bench_tests.inc"
