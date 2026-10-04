/* One active retail controller set. Preloaded models remain scene-owned;
 * descendants created by initialization/item callbacks belong to this visit. */
#include "script_stage_owned.h"
#include <melee/gr/grstory.h>
#include <melee/gr/grizumi.h>
#include <melee/it/it_3F14.h>
#include <melee/it/it_26B1.h>
#include <melee/ef/eflib.h>
static StOwned script_slot_owned;
static ScriptStageSlot* script_slot_native;
static StageInfo script_slot_native_info,script_slot_proc_saved,script_slot_native_base;
static Article* script_slot_articles[30];
static unsigned script_slot_native_heap;
static unsigned script_slot_native_min;
static unsigned script_slot_native_cost,script_slot_proc_heap;
static void* script_slot_host_yakumono;
static bool (*script_slot_host_shadow)(Vec3*,int,HSD_JObj*);
static int script_slot_native_initializing,script_slot_controller_next,script_slot_joint_guard;
static int script_slot_native_ground;
extern void Ground_StageSlotPrepare(HSD_GObj*,int);
extern void HSD_StageSlotParticlesClear(void);
extern void* grStory_StageSlotParamSwap(void*);
extern void* grIzumi_StageSlotParamSwap(void*);

int ScriptGame_StageSlotOwned(HSD_GObj* g)
{return script_slot_native && st_owned_find(&script_slot_owned,g)>=0;}
int ScriptGame_StageSlotNativeContext(void)
{return script_slot_native && script_slot_callback_context==script_slot_native;}
void ScriptGame_StageSlotCreated(HSD_GObj* g)
{
    if(ScriptGame_StageSlotNativeContext() && !st_owned_add(&script_slot_owned,g,0))
        OSReport("stage switch: ERROR ownership capacity exhausted\n");
}
int ScriptGame_StageSlotCanCreate(void)
{ return !ScriptGame_StageSlotNativeContext() || script_slot_owned.count<ST_OWNED_MAX; }
void ScriptGame_StageSlotDestroyed(HSD_GObj* g)
{st_owned_remove(&script_slot_owned,g);}

static void script_slot_reset_joints(HSD_JObj* j,HSD_Joint* d)
{
    if(!j || !d)return;
    HSD_JObjSetTranslate(j,&d->position);HSD_JObjSetScale(j,&d->scale);
    HSD_JObjSetRotationX(j,d->rotation.x);HSD_JObjSetRotationY(j,d->rotation.y);HSD_JObjSetRotationZ(j,d->rotation.z);
    if(d->flags&JOBJ_HIDDEN)HSD_JObjSetFlags(j,JOBJ_HIDDEN);else HSD_JObjClearFlags(j,JOBJ_HIDDEN);
    script_slot_reset_joints(HSD_JObjGetChild(j),d->child);
    script_slot_reset_joints(HSD_JObjGetNext(j),d->next);
}
HSD_GObj* ScriptGame_StageSlotGround(int id)
{
    ScriptStageSlot* s=script_slot_native;HSD_GObj* g;UnkStageDat* head;
    Vec3 zero={0,0,0},scale;
    if(!script_slot_native_initializing || !s || id<0 || id>=s->ngroups)return NULL;
    if(s->dynamic_kind==Gr_Kind_Izumi && id==4) {
        if(script_slot_controller_next>=2)return NULL;
        g=s->controllers[script_slot_controller_next++];
    } else g=s->models[id];
    if(!g)return NULL;
    st_owned_add(&script_slot_owned,g,1);
    HSD_JObjRemoveAnimAll(g->hsd_obj);
    head=HSD_ArchiveGetPublicAddress(s->archive,"map_head");
    script_slot_reset_joints(HSD_JObjGetChild(g->hsd_obj),head->unk8[id].unk0);
    scale.x=scale.y=scale.z=s->param->y;
    HSD_JObjSetTranslate(g->hsd_obj,&zero);HSD_JObjSetScale(g->hsd_obj,&scale);
    HSD_JObjClearFlags(g->hsd_obj,JOBJ_HIDDEN);
    Ground_StageSlotPrepare(g,id);
    /* Native constructors expect an unlinked render callback. */
    if(g->gx_link!=HSD_GOBJ_GXLINK_NONE)HSD_GObjGXLink_8039084C(g);
    return g;
}
int ScriptGame_StageSlotJointUpdate(int);
int ScriptGame_StageSlotCollision(int group)
{
    ScriptStageSlot* s=script_slot_native;UnkStageDat* head;const GrJoint* binding;unsigned count;int i;unsigned char seen[256]={0};
    if(!ScriptGame_StageSlotNativeContext())return 0;
    if(group<0 || group>=s->ngroups)return 1;
    head=HSD_ArchiveGetPublicAddress(s->archive,"map_head");
    for(i=0;i<head->unk8[group].unk24;++i) {
        int source=head->unk8[group].unk20[i].x;
        if(!seen[source]){seen[source]=1;ScriptGame_StageSlotJointUpdate(source);}
    }
    binding=Ground_StageSlotJoints(s->dynamic_kind,&count);
    for(i=0;i<(int)count;++i)if(binding[i].y==group && !seen[binding[i].x]) {
        seen[binding[i].x]=1;ScriptGame_StageSlotJointUpdate(binding[i].x);
    }
    return 1;
}
int ScriptGame_StageSlotJointUpdate(int source)
{
    int i;ScriptStageSlot* s=script_slot_native;
    if(!ScriptGame_StageSlotNativeContext() || (!script_slot_native_initializing && !script_slot_native_ground) ||
       script_slot_joint_guard || source<0 || source>=s->njoints)return 0;
    script_slot_joint_guard=1;
    for(i=0;i<script_switch.nlines;++i)if(s->lines[i].joint==source) {
        int at=script_stage_seam_slot(script_switch.lines[i]);
        int joint=script_stage.base_j+at,line=script_stage.base_l+at;
        HSD_JObj* j=s->collision_joints[source];
        if(!j)continue;
        mpLib_80055E9C(joint);mpLib_80057424(joint);mpLib_8005667C(joint);
        if(s->dynamic_kind==Gr_Kind_Izumi && (source==0 || source==1)) {
            Vec3 pos;int k,absent=0;lb_8000B1CC(j,NULL,&pos);
            for(k=0;k<2;++k)if(s->controllers[k]) {
                Ground* gp=s->controllers[k]->user_data;
                if(gp->u.izumi3.xC8==source && (gp->u.izumi3.xC4==3 || gp->u.izumi3.xC4==4))absent=1;
            }
            if(absent || pos.y<0 || (HSD_JObjGetFlags(j)&JOBJ_HIDDEN))mpLib_800575B0(line);
            else mpLib_80057528(line);
        }
    }
    script_slot_joint_guard=0;mpUncheckBounding();return 1;
}
static void script_slot_native_scope_begin(void)
{
    script_slot_proc_heap=lbHeap_Free(0);
    script_slot_context_begin(script_slot_native,&script_slot_proc_saved);
    /* Preserve retail controller writes across callbacks, while camera/blast
     * globals outside this scope remain destination-owned. */
    stage_info=script_slot_native_info;
    stage_info.cam_info=script_slot_proc_saved.cam_info;
    stage_info.blast_zone=script_slot_proc_saved.blast_zone;
}
int ScriptGame_StageSlotProcBegin(HSD_GObj* g)
{
    if(!ScriptGame_StageSlotOwned(g))return 0;
    script_slot_native_scope_begin();script_slot_native_ground=g->classifier==HSD_GOBJ_CLASS_STAGE;return 1;
}
int ScriptGame_StageSlotParticleBegin(int link)
{
    if(link!=3 || !script_slot_native || script_slot_callback_context)return 0;
    script_slot_native_ground=0;
    script_slot_native_scope_begin();return 1;
}
void ScriptGame_StageSlotProcEnd(void)
{
    unsigned freebytes=lbHeap_Free(0);
    if(freebytes<script_slot_proc_heap)script_slot_native_cost+=script_slot_proc_heap-freebytes;
    else {
        unsigned released=freebytes-script_slot_proc_heap;
        script_slot_native_cost=released>script_slot_native_cost?0:script_slot_native_cost-released;
    }
    if(freebytes<script_slot_native_min) {
        script_slot_native_min=freebytes;
        OSReport("stage switch: native peak slot=%d heap0_free=%u runtime_bytes=%u owned=%d\n",
                 script_slot_native->handle,freebytes,script_slot_native_bytes(script_slot_native),script_slot_owned.count);
    }
    script_slot_native_info=stage_info;
    script_slot_context_end(&script_slot_proc_saved);
    script_slot_native_ground=0;
}
static unsigned script_slot_native_bytes(ScriptStageSlot* s)
{
    return s==script_slot_native?script_slot_native_cost:0;
}
static void script_slot_native_enter(ScriptStageSlot* s)
{
    StageInfo saved;int i;
    if(s->dynamic!=3)return;
    script_slot_native_base=stage_info;
    script_slot_native=s;script_slot_native_heap=lbHeap_Free(0);
    script_slot_host_shadow=stage_info.on_check_shadow_render;
    script_slot_native_min=script_slot_native_heap;
    script_slot_native_cost=0;
    memcpy(script_slot_articles,it_804A0F60,sizeof script_slot_articles);
    memset(it_804A0F60,0,sizeof script_slot_articles);
    script_slot_context_begin(s,&saved);
    script_slot_host_yakumono=s->dynamic_kind==Gr_Kind_Story?
        grStory_StageSlotParamSwap(stage_info.yakumono_param):grIzumi_StageSlotParamSwap(stage_info.yakumono_param);
    if(stage_info.itemdata)for(i=0;stage_info.itemdata[i];++i)
        it_8026B40C(stage_info.itemdata[i]->unk4,stage_info.itemdata[i]->unk0);
    script_slot_native_initializing=1;script_slot_controller_next=0;
    if(s->dynamic_kind==Gr_Kind_Story)grStory_801E3030();
    else grIzumi_801CBB88();
    stage_info.on_check_shadow_render=s->dynamic_kind==Gr_Kind_Story?grStory_801E36D8:grIzumi_801CD280;
    /* On-init functions already establish their retail start frames. Do not
     * consume an extra animation tick during the transition. Refresh only
     * collision after those initial poses/controller writes. */
    for(i=0;i<s->njoints;++i)if(s->collision_joints[i])ScriptGame_StageSlotJointUpdate(i);
    script_slot_native_initializing=0;
    script_slot_native_info=stage_info;
    script_slot_context_end(&saved);
    stage_info.on_check_shadow_render=script_slot_native_info.on_check_shadow_render;
    stage_info.grkind=script_slot_native_info.grkind;
    stage_info.flags=script_slot_native_info.flags;stage_info.unk8C=script_slot_native_info.unk8C;
    if(lbHeap_Free(0)<script_slot_native_heap)script_slot_native_cost=script_slot_native_heap-lbHeap_Free(0);
    script_slot_native_min=lbHeap_Free(0);
    OSReport("stage switch: native init slot=%d heap0_free=%u runtime_bytes=%u owned=%d\n",
             s->handle,lbHeap_Free(0),script_slot_native_bytes(s),script_slot_owned.count);
}
static void script_slot_native_leave(void)
{
    ScriptStageSlot* s=script_slot_native;
    StageInfo saved;
    unsigned before,after;
    if(!s)return;
    before=lbHeap_Free(0);
    script_slot_context_begin(s,&saved);
    HSD_StageSlotParticlesClear();
    while(script_slot_owned.count) {
        StOwnedEntry e=script_slot_owned.entry[script_slot_owned.count-1];HSD_GObj* g=e.object;
        st_owned_remove(&script_slot_owned,g);
        if(e.borrowed) {
            HSD_GObjProc_RemoveAllProcs(g);
            if(g->gx_link!=HSD_GOBJ_GXLINK_NONE)HSD_GObjGXLink_8039084C(g);
            if(g!=s->controllers[0] && g!=s->controllers[1])GObj_SetupGXLink(g,script_slot_render,3,0);
        } else if(g->p_link==HSD_GOBJ_PLINK_ITEM) {
            /* Switching is teardown, not an item death/explosion event. */
            GET_ITEM(g)->destroy_type=3;Item_8026A8EC(g);
        }
        else if(g->classifier==HSD_GOBJ_CLASS_EFFECT) {
            /* efLib_Destroy deliberately refuses asynchronous effects. Their
             * parent/parameter links still need clearing before GObj teardown. */
            efLib_DestroyAll(g);HSD_GObjFree(g);
        }
        else HSD_GObjFree(g);
    }
    memcpy(it_804A0F60,script_slot_articles,sizeof script_slot_articles);
    HSD_StageSlotParticlesClear();
    script_slot_context_end(&saved);
    if(s->dynamic_kind==Gr_Kind_Story)grStory_StageSlotParamSwap(script_slot_host_yakumono);
    else grIzumi_StageSlotParamSwap(script_slot_host_yakumono);
    stage_info.on_check_shadow_render=script_slot_host_shadow;
    stage_info.grkind=script_slot_native_base.grkind;
    stage_info.flags=script_slot_native_base.flags;stage_info.unk8C=script_slot_native_base.unk8C;
    after=lbHeap_Free(0);
    /* HSD object pools retain their backing chunks after their live objects
     * die. Charge those cached chunks to this slot, rather than losing them
     * from the next visit's heap accounting. Other scene allocations outside
     * owned callbacks do not enter this counter. */
    if(after>=before) {
        unsigned released=after-before;
        if(released<script_slot_native_cost)s->bytes+=script_slot_native_cost-released;
    } else s->bytes+=script_slot_native_cost+before-after;
    script_slot_native=NULL;
    OSReport("stage switch: native teardown slot=%d heap0_free=%u minimum=%u owned=0\n",s->handle,lbHeap_Free(0),script_slot_native_min);
}
