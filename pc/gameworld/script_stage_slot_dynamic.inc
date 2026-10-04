/* DAT clip/collision preparation. Yoshi/Fountain reserve Phase C controllers;
 * the native lifecycle installs their scoped retail procs only while active.
 * Other legal slots retain Phase B and the FD group-0 callback proof. */
#ifndef STAGE_SLOT_FIXTURE
#include <melee/gr/granime.h>
#include <sysdolphin/baselib/gobjuserdata.h>
#include <sysdolphin/baselib/gobjproc.h>
#include <sysdolphin/baselib/gobjgxlink.h>
static UnkArchiveStruct script_slot_archive_context;
static ScriptStageSlot* script_slot_callback_context;

UnkArchiveStruct* ScriptGame_StageSlotArchive(int group)
{
    ScriptStageSlot* s=script_slot_callback_context;
    if(!s || group<0 || group>=s->ngroups)return NULL;
    return &script_slot_archive_context;
}
static void script_slot_ground_free(void* p){lbHeap_80015CA8(0,p);}
static void script_slot_context_begin(ScriptStageSlot* s,StageInfo* saved)
{
    *saved=stage_info;script_slot_callback_context=s;
    memset(&script_slot_archive_context,0,sizeof script_slot_archive_context);
    script_slot_archive_context.unk0=s->archive;
    script_slot_archive_context.unk4=HSD_ArchiveGetPublicAddress(s->archive,"map_head");
    stage_info.grkind=s->dynamic_kind;stage_info.param=s->param;
    stage_info.yakumono_param=HSD_ArchiveGetPublicAddress(s->archive,"yakumono_param");
    stage_info.itemdata=HSD_ArchiveGetPublicAddress(s->archive,"itemdata");
    memcpy(stage_info.map_gobjs,s->models,sizeof stage_info.map_gobjs);
    memcpy(stage_info.x280,s->markers,sizeof stage_info.x280);
}
static void script_slot_context_end(const StageInfo* saved)
{stage_info=*saved;script_slot_callback_context=NULL;memset(&script_slot_archive_context,0,sizeof script_slot_archive_context);}

int ScriptGame_StageSlotDynamic(int handle,int owner,int want)
{
    ScriptStageSlot* s=script_slot(handle,owner);UnkStageDat* head;int g,i,kind=-1;StageInfo saved;unsigned before,count;const GrJoint* bindings;
    static const char* files[]={"GrNLa.dat","GrNBa.dat","GrSt.dat","GrOp.dat","GrIz.dat","GrPs.dat"};
    static const int kinds[]={Gr_Kind_Last,Gr_Kind_Battle,Gr_Kind_Story,Gr_Kind_OldPupupu,Gr_Kind_Izumi,Gr_Kind_PStadium};
    if(!s)return -5;
    if(!STAGE_SLOT_PARTIAL_FALLBACK) {
        for(i=0;i<6;++i)if(script_stage_same_file(s->file,files[i]))kind=kinds[i];
        if(kind<0 || !st_retail_supported(kind))return -18;
        s->dynamic_kind=kind;s->dynamic=4;return 4;
    }
    for(i=0;i<6;++i)if(script_stage_same_file(s->file,files[i]))kind=kinds[i];
    if(want<0)want=kind>=0;
    /* A static FD still needs its authored starting backdrop pose. */
    if(!want && kind!=Gr_Kind_Last)return 0;
    if(kind<0 || s->nmesh)return -18;
    if(s->dynamic)return s->dynamic;
    if(lbHeap_Free(0)<ST_SLOT_HEADROOM+1024*1024)return -4;
    head=HSD_ArchiveGetPublicAddress(s->archive,"map_head");
    if(want && (kind==Gr_Kind_Story || kind==Gr_Kind_Izumi)) {
        void* yaku=HSD_ArchiveGetPublicAddress(s->archive,"yakumono_param");
        struct GroundItemData** rows=HSD_ArchiveGetPublicAddress(s->archive,"itemdata");
        int found=0;
        if(!script_slot_ptr(s->archive,yaku,kind==Gr_Kind_Story?36:84))return -2;
        if(rows)for(i=0;i<31;++i) {
            if(!script_slot_ptr(s->archive,&rows[i],sizeof rows[i]))return -2;
            if(!rows[i])break;
            if(i==30 || !script_slot_ptr(s->archive,rows[i],sizeof *rows[i]) ||
               !script_slot_ptr(s->archive,rows[i]->unk4,sizeof *rows[i]->unk4))return -2;
            if(rows[i]->unk0==It_Kind_Heiho)found=1;
        }
        if(kind==Gr_Kind_Story && !found)return -2;
    }
    /* Validate the clip arrays and binding indices before allocating animation
     * objects. The legal archive's HSD descriptors are trusted engine input. */
    for(g=0;g<s->ngroups;++g)if(s->models[g]) {
        struct UnkStageDat_x8_t* d=&head->unk8[g];
        if((d->unk4 && !script_slot_ptr(s->archive,d->unk4,sizeof *d->unk4)) ||
           (d->unk8 && !script_slot_ptr(s->archive,d->unk8,sizeof *d->unk8)) ||
           (d->unkC && !script_slot_ptr(s->archive,d->unkC,sizeof *d->unkC)) ||
           d->unk24<0 || d->unk24>256 ||
           (d->unk24 && !script_slot_ptr(s->archive,d->unk20,d->unk24*sizeof *d->unk20)))return -2;
        for(i=0;i<d->unk24;++i)if(d->unk20[i].x<0 || d->unk20[i].x>=s->njoints || d->unk20[i].z<0)return -2;
    }
    s->dynamic_kind=kind;
    before=lbHeap_Free(0);bindings=Ground_StageSlotJoints(kind,&count);
    if(count>256)return -2;
    for(g=0;g<s->ngroups;++g)if(s->models[g]) {
        HSD_GObj* obj=s->models[g];HSD_JObj* source=HSD_JObjGetChild(obj->hsd_obj);
        Ground* gp;
        struct UnkStageDat_x8_t* d=&head->unk8[g];
        gp=lbHeap_StageSlotTryAlloc(sizeof *gp);if(!gp)return -4;
        memset(gp,0,sizeof *gp);gp->map_id=g;gp->gobj=obj;
        for(i=0;i<8;++i)gp->x20[i]=-1;
        GObj_InitUserData(obj,3,script_slot_ground_free,gp);
        script_slot_context_begin(s,&saved);
        if(kind==Gr_Kind_Last && g==0) {
            StageCallbacks* cb=Ground_StageSlotProofCallbacks();
            if(cb && cb->on_init){cb->on_init(obj);s->callback_proof=1;}
        } else if(want && kind==Gr_Kind_Story && g==3) {
            /* Warm both retail clips before committing a switch. */
            grAnime_801C7FF8(obj,0,7,0,0,1);
            grAnime_801C7FF8(obj,5,7,1,0,1);
        } else grAnime_801C8138(obj,g,0);
        script_slot_context_end(&saved);
        for(i=0;i<d->unk24;++i) {
            int index=d->unk20[i].z;
            HSD_JObj* joint=script_slot_joint(source,&index);
            if(!joint)return -2;
            s->collision_joints[d->unk20[i].x]=joint;
        }
        for(i=0;i<(int)count;++i)if(bindings[i].y==g) {
            int index=bindings[i].z;HSD_JObj* joint;
            if(bindings[i].x<0 || bindings[i].x>=s->njoints || index<0)return -2;
            joint=script_slot_joint(source,&index);if(!joint)return -2;
            s->collision_joints[bindings[i].x]=joint;
        }
    }
    if(kind==Gr_Kind_Last) {
        extern void Ground_StageSlotFDBackdrop(void);
        script_slot_context_begin(s,&saved);
        Ground_StageSlotFDBackdrop();
        for(g=0;g<s->ngroups;++g)if(s->models[g])HSD_JObjAnimAll(s->models[g]->hsd_obj);
        script_slot_context_end(&saved);
    }
    s->bytes+=before-lbHeap_Free(0);s->model_estimate+=before-lbHeap_Free(0);
    if(want && (kind==Gr_Kind_Story || kind==Gr_Kind_Izumi)) {
        unsigned reserve=lbHeap_Free(0);
        if(kind==Gr_Kind_Izumi) {
            for(i=0;i<2;++i) {
                script_slot_context_begin(s,&saved);
                s->controllers[i]=Ground_GetStageGObj(4);
                if(s->controllers[i])grAnime_801C8138(s->controllers[i],4,0);
                script_slot_context_end(&saved);
                if(!s->controllers[i])return -4;
                HSD_GObjProc_RemoveAllProcs(s->controllers[i]);
                if(s->controllers[i]->gx_link!=HSD_GOBJ_GXLINK_NONE)HSD_GObjGXLink_8039084C(s->controllers[i]);
            }
        }
        s->bytes+=reserve-lbHeap_Free(0);s->model_estimate+=reserve-lbHeap_Free(0);
        s->dynamic=3;return 3;
    }
    s->dynamic=want?2:0;return s->dynamic;
}
static void script_slot_dynamic_bind(void)
{
    ScriptStageSlot* s;int i;
    if(!script_switch.active)return;
    s=script_slot(script_switch.active,0);
    if(!s || !s->dynamic || s->dynamic==4)return;
    for(i=0;i<script_switch.nlines;++i) {
        int at=script_stage_seam_slot(script_switch.lines[i]),group=s->lines[i].joint,j;
        CollVtx* vertices;
        if(at<0 || group<0 || group>=256 || !s->collision_joints[group])continue;
        j=script_stage.base_j+at;
        mpGetGroundCollJoint()[j].x20=s->collision_joints[group];
        /* The JObj matrix already contains the destination stage scale. */
        vertices=&mpGetGroundCollVtx()[script_stage.base_v+2*at];
        vertices[0].x0=s->lines[i].x0;vertices[0].x4=s->lines[i].y0;
        vertices[1].x0=s->lines[i].x1;vertices[1].x4=s->lines[i].y1;
        mpLib_80055E9C(j);mpLib_80057424(j);mpLib_8005667C(j);
        vertices=&mpGetGroundCollVtx()[script_stage.base_v+2*at];
        script_stage.line[at].x0=vertices[0].pos.x;script_stage.line[at].y0=vertices[0].pos.y;
        script_stage.line[at].x1=vertices[1].pos.x;script_stage.line[at].y1=vertices[1].pos.y;
    }
    mpUncheckBounding();
}
void ScriptGame_StageSlotsFrame(void)
{
    ScriptStageSlot* s;int g;StageInfo saved;
    if(!script_switch.active)return;
    s=script_slot(script_switch.active,0);
    if(!s || !s->dynamic || s->dynamic==4)return;
    if(s->dynamic==3)return; /* Retail scheduler animates/controllers once. */
    for(g=0;g<s->ngroups;++g)if(s->models[g])HSD_JObjAnimAll(s->models[g]->hsd_obj);
    if(s->callback_proof && s->models[0]) {
        StageCallbacks* cb=Ground_StageSlotProofCallbacks();
        script_slot_context_begin(s,&saved);
        if(cb && cb->gobj_proc)cb->gobj_proc(s->models[0]);
        script_slot_context_end(&saved);
    }
    script_slot_dynamic_bind();
}
#else
static void script_slot_dynamic_bind(void){}
#endif
