/* Packet O: scene-owned DAT slots. Included at the END of script_game.c;
 * all game pointers and fighter writes stay in MEM1. Legal DATs support scoped
 * animation; Yoshi/Fountain additionally run owned native retail controllers.
 * Stage Morph ideas only;
 * credits: workspace CREDITS.md. API errors are stable scalar codes. */
#include "script_stage_dat.h"
#include "script_stage_dat_models.h"
#ifndef STAGE_SLOT_FIXTURE
#include <melee/ft/kinds/ftCommon/ftCo_Fall.h>
#include <melee/ft/ftcommon.h>
#include <melee/ft/ftcamera.h>
#include <melee/ft/ftwaitanim.h>
#include <melee/cm/camera.h>
#endif
#define ST_SLOT_COUNT 16
#define ST_SLOT_GROUPS 64
#define ST_SLOT_HEADROOM (512*1024)
extern void* lbHeap_StageSlotTryAlloc(unsigned);
extern int lbAudioAx_StageSlotMusic(void);
#include "script_stage_music.inc"
typedef struct {
    int handle,owner,nlines,nverts,njoints,ngroups,music;
    unsigned bytes,model_estimate;
    void* data;
    HSD_Archive* archive;
    StDatLine* lines;
    HSD_GObj* models[ST_SLOT_GROUPS];
    HSD_GObj* controllers[2];
    HSD_JObj* markers[261];
    int mesh[ST_SLOT_GROUPS],nmesh,own_markers;
    int dynamic,dynamic_kind,callback_proof;
    HSD_JObj* collision_joints[256];
    StageCameraInfo camera;
    StageBlastZone blast;
    GroundParam* param;
    char file[260];
} ScriptStageSlot;
static ScriptStageSlot script_slots[ST_SLOT_COUNT];
static void script_slot_dynamic_bind(void);
#ifndef STAGE_SLOT_FIXTURE
static void script_slot_native_leave(void);
static void script_slot_native_enter(ScriptStageSlot*);
static unsigned script_slot_native_bytes(ScriptStageSlot*);
#else
#define script_slot_native_leave() ((void)0)
#define script_slot_native_enter(s) ((void)0)
#define script_slot_native_bytes(s) 0u
#endif
static struct {
    int active,owner,epoch,nlines,music;
    int lines[SCRIPT_STAGE_LINES];
    StageCameraInfo camera;
    StageBlastZone blast;
    GroundParam* param;
    float nearz,farz;
    HSD_JObj* markers[261];
} script_switch;
static ScriptStageSlot* script_slot(int handle,int owner)
{
    int i; for(i=0;i<ST_SLOT_COUNT;++i)
        if(script_slots[i].handle==handle && handle>0 && (!owner || script_slots[i].owner==owner)) return &script_slots[i];
    return NULL;
}
int ScriptGame_StageSlotFile(int kind,int at)
{
    const char* p;
    if(kind<0 || kind>=Gr_Kind_Count || at<0 || at>259) return 0;
    p=Ground_StageSlotDataFile(kind);
    if(!p)return 0;
    while(*p=='/')++p;
    { int n=(int)strlen(p),i,has_extension=0;
      if(at<n)return p[at];
      for(i=0;i<n;++i)if(p[i]=='.'){has_extension=1;break;}
      if(!has_extension && at<n+4)return ".dat"[at-n];
      return 0; }
}
static void script_slot_render(HSD_GObj* gobj,int pass)
{
    int i,k; extern int Script_StageSlotVisual(int);
    if(gx_suppress_draws) return;
    for(k=0;k<ST_SLOT_COUNT;++k) {
        ScriptStageSlot* s=&script_slots[k];float offset;
        if(!s->handle)continue;
        offset=script_mesh_float(Script_StageSlotVisual(s->handle));
        if(offset!=offset && s->handle!=script_switch.active)continue;
        for(i=0;i<s->ngroups;++i) if(s->models[i]==gobj) {
            if(s->dynamic==3 && s->handle==script_switch.active)return;
            HSD_JObj* root=gobj->hsd_obj;Vec3 p,saved;
            HSD_JObjGetTranslation(root,&saved);p=saved;
            if(offset==offset) {p.y+=offset;HSD_JObjSetTranslate(root,&p);}
            HSD_GObj_JObjCallback(gobj,pass);
            if(offset==offset)HSD_JObjSetTranslate(root,&saved);
            return;
        }
    }
}
static HSD_JObj* script_slot_joint(HSD_JObj* root,int* index)
{
    HSD_JObj* p;
    if(!root) return NULL;
    if((*index)--==0) return root;
    p=script_slot_joint(HSD_JObjGetChild(root),index);
    return p ? p : script_slot_joint(HSD_JObjGetNext(root),index);
}
static void script_slot_dispose(ScriptStageSlot* s)
{
    int i;
    for(i=0;i<2;++i)if(s->controllers[i])HSD_GObjFree(s->controllers[i]);
    for(i=0;i<s->ngroups;++i) if(s->models[i]) HSD_GObjFree(s->models[i]);
    for(i=0;i<s->nmesh;++i) ScriptGame_ModelDespawn(s->mesh[i]);
    if(s->own_markers) for(i=0;i<261;++i) if(s->markers[i]) HSD_JObjUnref(s->markers[i]);
    if(s->lines) lbHeap_80015CA8(0,s->lines);
    if(s->archive) lbHeap_80015CA8(0,s->archive);
    if(s->data) lbHeap_80015CA8(0,s->data);
    memset(s,0,sizeof *s);
}
/* Preflight bounds every pointer used by slot metadata. Model descriptor tree
 * validation is separate; counts never turn unchecked archive addresses into
 * memory. The loader rejects instance trees like StageAddModel. */
static int script_slot_ptr(HSD_Archive* a,const void* p,unsigned bytes)
{
    unsigned at=(unsigned)((const u8*)p-a->data);
    return p && at<=a->header.data_size && bytes<=a->header.data_size-at;
}
int ScriptGame_StageSlotLoad(int owner,int handle,int music)
{
    /* Serialized scene-thread load: keep the 32 KiB census off the game stack. */
    static StDatModels models;
    ScriptStageSlot* s=NULL; StDatView v; unsigned n=0,bytes,need,before; size_t read_bytes;
    char path[262],file[260]; int i,g,entry,rc,stadium;unsigned long long group_mask=~0ull;
    UnkStageDat* head; StageCameraInfo saved_camera; StageBlastZone saved_blast;
    extern int Script_StageArchiveInput(int which,int at);
    if(owner<=0 || handle<=0 || !script_stage.map || Netplay_Enabled() || RB_Enabled() || Replay_Active()) return -1;
    if(!script_stage_text(0,file,sizeof file)) return -2;
    stadium=script_stage_same_file(file,"GrPs.dat");
    /* grStadium_OnInit creates groups 1/2, and group 2's init creates group 5
     * (normal terrain). Other groups are transformation archives. Its initial
     * collision retains joints 4/6: grpstadium.c:182-188, 317, 360-370. */
    if(stadium)group_mask=(1ull<<1)|(1ull<<2)|(1ull<<5);
    /* Retail FD starts groups 0..8. Group 9 belongs to later backgrounds. */
    if(script_stage_same_file(file,"GrNLa.dat"))group_mask=(1ull<<9)-1;
    for(i=0;i<ST_SLOT_COUNT;++i) if(!script_slots[i].handle) { s=&script_slots[i]; break; }
    if(!s) return -3;
    bytes=Script_StageArchiveInput(0,0);
    path[0]='/'; strcpy(path+1,file);
    if(!bytes) { entry=DVDConvertPathToEntrynum(path); if(entry<0) return -2; bytes=lbFile_8001634C(entry); }
    if(bytes<32 || bytes>8*1024*1024) return -2;
    need=((bytes+31)&~31u)+sizeof(HSD_Archive)+script_stage.cap*sizeof(StDatLine)+ST_SLOT_HEADROOM;
    if(lbHeap_Free(0)<need) return -4;
    before=lbHeap_Free(0);
    s->data=lbHeap_StageSlotTryAlloc((bytes+31)&~31u);
    s->archive=lbHeap_StageSlotTryAlloc(sizeof(HSD_Archive));
    if(!s->data || !s->archive) { script_slot_dispose(s); return -4; }
    if(Script_StageArchiveInput(0,0)) {
        u8* dst=s->data; unsigned j;
        for(j=0;j+4<=bytes;j+=4) *(u32*)(dst+j)=(u32)Script_StageArchiveInput(3,j);
        for(;j<bytes;++j) dst[j]=(u8)Script_StageArchiveInput(4,j);
        read_bytes=bytes;
    } else lbFile_8001668C(path,s->data,&read_bytes);
    rc=read_bytes==bytes ? st_dat_open(s->data,bytes,&v) : ST_DAT_RANGE;
    if(!rc) rc=st_dat_collision(&v,script_stage.cap>1?script_stage.cap-1:0,&n);
    if(!rc)rc=st_dat_models_mask(&v,&models,group_mask);
    if(rc) { script_slot_dispose(s); return -10-rc; }
    {
        void* probe;
        if(lbHeap_Free(0)<models.runtime_allowance+ST_SLOT_HEADROOM){script_slot_dispose(s);return -4;}
        probe=lbHeap_StageSlotTryAlloc(models.runtime_allowance);
        if(!probe){script_slot_dispose(s);return -4;}
        lbHeap_80015CA8(0,probe);
    }
    s->lines=lbHeap_StageSlotTryAlloc(n*sizeof(StDatLine));
    if(!s->lines) { script_slot_dispose(s); return -4; }
    for(i=0;i<(int)n;++i) st_dat_line(&v,i,&s->lines[i]);
    if(stadium) {
        int remap[768],out=0,k;for(i=0;i<(int)n;++i)remap[i]=-1;
        for(i=0;i<(int)n;++i)if(s->lines[i].joint==4 || s->lines[i].joint==6)remap[i]=out++;
        out=0;for(i=0;i<(int)n;++i)if(remap[i]>=0) {
            StDatLine line=s->lines[i];int* ids[4]={&line.prev0,&line.next0,&line.prev1,&line.next1};
            for(k=0;k<4;++k)if(*ids[k]>=0)*ids[k]=remap[*ids[k]];
            s->lines[out++]=line;
        }
        n=out;if(!n){script_slot_dispose(s);return -2;}
    }
    s->nlines=n; s->nverts=v.nv; s->njoints=v.nj;
    if(HSD_ArchiveParse(s->archive,s->data,bytes)<0) { script_slot_dispose(s); return -2; }
    for(i=0;HSD_ArchiveGetExtern(s->archive,i);++i)
        HSD_ArchiveLocateExtern(s->archive,HSD_ArchiveGetExtern(s->archive,i),NULL);
    head=HSD_ArchiveGetPublicAddress(s->archive,"map_head");
    s->param=HSD_ArchiveGetPublicAddress(s->archive,"grGroundParam");
    if(!script_slot_ptr(s->archive,head,sizeof *head) || !script_slot_ptr(s->archive,s->param,sizeof *s->param) ||
       head->unkC<=0 || head->unkC>ST_SLOT_GROUPS || !script_slot_ptr(s->archive,head->unk8,head->unkC*sizeof *head->unk8)) {
        script_slot_dispose(s); return -2;
    }
    if(!(s->param->y>0.001f && s->param->y<=100)){script_slot_dispose(s);return -2;}
    for(g=0;g<head->unkC;++g) {
        HSD_Joint* source=head->unk8[g].unk0; HSD_Joint branch; HSD_JObj* root;
        if(!(group_mask&(1ull<<g)) || !source) { s->ngroups=g+1; continue; }
        if(!script_slot_ptr(s->archive,source,sizeof *source) || source->flags&JOBJ_INSTANCE || script_stage_has_instance(source->child)) {
            script_slot_dispose(s); return -2;
        }
        /* Reserve a conservative HSD runtime allowance, not a fabricated exact
         * texture measurement. DAT texture bytes are already included in bytes. */
        if(lbHeap_Free(0)<ST_SLOT_HEADROOM+64*1024) { script_slot_dispose(s); return -4; }
        branch=*source; branch.next=NULL;
        root=HSD_JObjLoadJoint(&branch);
        if(!root) { script_slot_dispose(s); return -4; }
        s->models[g]=GObj_Create(HSD_GOBJ_CLASS_STAGE,13,0);
        if(!s->models[g]) { HSD_JObjUnref(root); script_slot_dispose(s); return -4; }
        s->ngroups=g+1;
#ifndef STAGE_SLOT_FIXTURE
        {HSD_JObj* wrapper=HSD_JObjAlloc();Vec3 scale={s->param->y,s->param->y,s->param->y};
         if(!wrapper){HSD_JObjUnref(root);script_slot_dispose(s);return -4;}
         HSD_JObjSetScale(wrapper,&scale);HSD_JObjAddChild(wrapper,root);
         HSD_GObjObject_80390A70(s->models[g],HSD_GObj_JObjKind,wrapper);}
#else
        HSD_GObjObject_80390A70(s->models[g],HSD_GObj_JObjKind,root);
#endif
        GObj_SetupGXLink(s->models[g],script_slot_render,3,0);
        /* Reproduce Ground_801C34AC's joint-index -> general-point bindings. */
        {
            struct SlotMarkers { HSD_Joint* joint; s16* pairs; int count; }* rows=head->unk0;
            int r;
            if(head->unk4<0 || head->unk4>261 || (head->unk4 && !script_slot_ptr(s->archive,rows,head->unk4*sizeof *rows))) {
                script_slot_dispose(s); return -2;
            }
            for(r=0;r<head->unk4;++r) if(rows[r].joint==source) {
                int k;
                if(rows[r].count<0 || rows[r].count>261 || !script_slot_ptr(s->archive,rows[r].pairs,rows[r].count*4)) {
                    script_slot_dispose(s); return -2;
                }
                for(k=0;k<rows[r].count;++k) {
                    int index=rows[r].pairs[k*2], id=rows[r].pairs[k*2+1];
                    if(index<0 || index>4095 || id<0 || id>260) { script_slot_dispose(s); return -2; }
                    s->markers[id]=script_slot_joint(root,&index);
                }
            }
        }
    }
    /* Derive marker bounds with retail functions while preserving all live
     * host parameter state. No logic or draw can interleave this scalar call. */
    {
        HSD_JObj* saved[261];
        saved_camera=stage_info.cam_info; saved_blast=stage_info.blast_zone;
        memcpy(saved,stage_info.x280,sizeof saved); memcpy(stage_info.x280,s->markers,sizeof saved);
        Ground_801C38D0(s->param->x8,s->param->x14,s->param->x1C,s->param->x18);
        Ground_801C38EC(s->param->x10,s->param->xC);
        Ground_801C3970(s->param->x28);
        Ground_801C3900(s->param->x2E,s->param->x30,s->param->x34,s->param->x38,
                       s->param->x3C,s->param->x40,s->param->x44,s->param->x48);
        Ground_801C392C(s->param->x50,s->param->x54,s->param->x58,s->param->x5C,s->param->x60,s->param->x64);
        Ground_801C3960(s->param->x20);Ground_801C3950(s->param->x24);
        Ground_801C39C0(); Ground_801C3BB4();
        s->camera=stage_info.cam_info; s->blast=stage_info.blast_zone;
        stage_info.cam_info=saved_camera; stage_info.blast_zone=saved_blast;
        memcpy(stage_info.x280,saved,sizeof saved);
    }
    s->handle=handle; s->owner=owner; s->music=music;
    s->bytes=before-lbHeap_Free(0);s->model_estimate=s->bytes>bytes+n*sizeof(StDatLine)+sizeof(HSD_Archive)?
        s->bytes-bytes-n*sizeof(StDatLine)-sizeof(HSD_Archive):0;
    if(s->music<0 && s->param->stage_param_count>0 && s->param->stage_param_count<=256 &&
       script_slot_ptr(s->archive,s->param->stage_params,s->param->stage_param_count*sizeof(StageParam)))
        s->music=s->param->stage_params[0].x4;
    strcpy(s->file,file);
    OSReport("stage slot: handle=%d file=%s bytes=%u lines=%d groups=%d heap0=%u/%u headroom=%u\n",
             handle,file,s->bytes,s->nlines,s->ngroups,lbHeap_Free(0),lbHeap_Capacity(0),ST_SLOT_HEADROOM);
    return handle;
}
/* Mission hook consumes exported GXMS sidecars, not hand-entered collision.
 * The native API has already created visual-only hidden instances; this record
 * adopts them only after all capacity and coordinate checks pass. */
int ScriptGame_StageSlotMission(int owner,int handle)
{
    ScriptStageSlot* s=NULL;int i,k,nlines,nmesh;unsigned before;
    extern int Script_StageSlotMissionInput(int,int,int);
    if(owner<=0 || Netplay_Enabled() || RB_Enabled() || Replay_Active())return -1;
    for(i=0;i<ST_SLOT_COUNT;++i)if(!script_slots[i].handle){s=&script_slots[i];break;}
    if(!s)return -3;
    nlines=Script_StageSlotMissionInput(0,0,0);nmesh=Script_StageSlotMissionInput(0,0,1);
    if(nlines<1 || nlines>=script_stage.cap || nmesh<1 || nmesh>64)return -7;
    if(lbHeap_Free(0)<ST_SLOT_HEADROOM+nlines*sizeof(StDatLine)+261*sizeof(HSD_JObj))return -4;
    before=lbHeap_Free(0);
    s->lines=lbHeap_StageSlotTryAlloc(nlines*sizeof(StDatLine));if(!s->lines)return -4;
    memset(s->lines,0,nlines*sizeof(StDatLine));
    for(i=0;i<nlines;++i) {
        StDatLine* l=&s->lines[i];int flags;
        l->x0=script_mesh_float(Script_StageSlotMissionInput(1,i,0));l->y0=script_mesh_float(Script_StageSlotMissionInput(1,i,1));
        l->x1=script_mesh_float(Script_StageSlotMissionInput(1,i,2));l->y1=script_mesh_float(Script_StageSlotMissionInput(1,i,3));
        l->kind=Script_StageSlotMissionInput(1,i,4);flags=Script_StageSlotMissionInput(1,i,5);
        if(l->kind<1 || l->kind>4 || !(l->x0>=-49000 && l->x0<=49000 && l->y0>=-49000 && l->y0<=49000 &&
            l->x1>=-49000 && l->x1<=49000 && l->y1>=-49000 && l->y1<=49000)) {script_slot_dispose(s);return -16;}
        l->hi=1<<(l->kind-1);l->lo=(flags&1?LINE_FLAG_PLATFORM:0)|(flags&2?LINE_FLAG_LEDGE:0);
        l->prev0=l->next0=l->prev1=l->next1=-1;l->joint=i;
    }
    s->nlines=nlines;s->nverts=nlines*2;s->njoints=nlines;s->camera=stage_info.cam_info;s->param=stage_info.param;
    s->camera.cam_x_offset=s->camera.cam_y_offset=0;
    s->camera.cam_bounds.left=script_mesh_float(Script_StageSlotMissionInput(3,0,0));
    s->camera.cam_bounds.right=script_mesh_float(Script_StageSlotMissionInput(3,1,0));
    s->camera.cam_bounds.top=script_mesh_float(Script_StageSlotMissionInput(3,2,0));
    s->camera.cam_bounds.bottom=script_mesh_float(Script_StageSlotMissionInput(3,3,0));
    s->blast.left=script_mesh_float(Script_StageSlotMissionInput(3,4,0));
    s->blast.right=script_mesh_float(Script_StageSlotMissionInput(3,5,0));
    s->blast.top=script_mesh_float(Script_StageSlotMissionInput(3,6,0));
    s->blast.bottom=script_mesh_float(Script_StageSlotMissionInput(3,7,0));
    s->own_markers=1;
    for(i=0;i<261;++i)if(Script_StageSlotMissionInput(4,i,0)) {
        HSD_Joint desc;memset(&desc,0,sizeof desc);desc.scale.x=desc.scale.y=desc.scale.z=1;
        desc.flags=JOBJ_JOINT1;
        desc.position.x=script_mesh_float(Script_StageSlotMissionInput(4,i,1));
        desc.position.y=script_mesh_float(Script_StageSlotMissionInput(4,i,2));
        desc.position.z=script_mesh_float(Script_StageSlotMissionInput(4,i,3));
        s->markers[i]=HSD_JObjLoadJoint(&desc);
    }
    /* No failures after adopting visual instances. Their load references remain
     * caller-owned; their scene-pinned bytes are accounted by the model budget. */
    for(k=0;k<nmesh;++k)s->mesh[k]=Script_StageSlotMissionInput(2,k,0);
    s->nmesh=nmesh;s->handle=handle;s->owner=owner;s->music=Script_StageSlotMissionInput(0,0,2);
    s->bytes=before-lbHeap_Free(0);s->model_estimate=Script_StageSlotMissionInput(0,0,3);
    OSReport("stage slot: mission slot=%d lines=%d meshes=%d game_bytes=%u pinned_native=%u\n",handle,nlines,nmesh,s->bytes,s->model_estimate);
    return handle;
}
int ScriptGame_StageSlotRead(int handle,int owner,int field,int index)
{
    ScriptStageSlot* s=script_slot(handle,owner); StageBlastZone* b; Vec3 p;
    if(field==30) return lbHeap_Free(0);
    if(field==31) return lbHeap_Capacity(0);
    if(field==32) return ST_SLOT_HEADROOM;
    if(field==33) return script_stage.cap;
    if(!s) return 0;
    switch(field) {
    case 0:return s->handle; case 1:return s->bytes+script_slot_native_bytes(s); case 2:return s->nverts;
    case 3:return s->nlines; case 4:return s->njoints; case 5:return s->ngroups;
    case 6:return script_switch.active==handle; case 7:return s->music;
    case 8:return index>=0 && index<261 && s->markers[index]!=NULL;
    case 9:case 10:case 11:
        if(index<0 || index>=261 || !s->markers[index]) return 0;
        lb_8000B1CC(s->markers[index],NULL,&p);
        return script_mesh_bits(field==9?p.x:field==10?p.y:p.z);
    case 29:return script_switch.epoch;
    case 34:return s->model_estimate;
    case 35:return s->dynamic;
    case 36:return s->callback_proof;
    }
    if(field>=12 && field<=19) {
        b=field<16?&s->camera.cam_bounds:&s->blast;
        return script_mesh_bits((field&3)==0?b->left:(field&3)==1?b->right:(field&3)==2?b->top:b->bottom);
    }
    return 0;
}
int ScriptGame_StageSlotAt(int owner,int index)
{ return index>=0 && index<ST_SLOT_COUNT && script_slots[index].owner==owner ? script_slots[index].handle : 0; }
static void script_switch_contacts(Fighter* fp)
{
    CollData* c=&fp->coll_data;
    c->floor_skip=c->ledge_id_right=c->ledge_id_left=c->joint_id_skip=c->joint_id_only=-1;
    c->floor.index=c->left_facing_wall.index=c->right_facing_wall.index=c->ceiling.index=-1;
    c->floor.flags=c->left_facing_wall.flags=c->right_facing_wall.flags=c->ceiling.flags=0;
    c->env_flags=c->prev_env_flags=c->x13C=0;
    c->contact.x=c->contact.y=c->contact.z=0;
    c->x28_vec.x=c->x28_vec.y=c->x28_vec.z=0;
    ftCommon_UnlockECB(fp);
    c->prev_ecb=c->xE4_ecb=c->desired_ecb=c->ecb;
    mpColl_80043680(c,&fp->cur_pos);
    fp->prev_pos=fp->cur_pos;
}
/* Release action-owned geometry while its lines are still valid. Fall entry
 * can re-arm the CPU, so preserve each entity's controller independently. */
static void script_switch_detach(void)
{
    HSD_GObj* g;
    if(!HSD_GObjPLinkHead)return;
    for(g=HSD_GObjPLinkHead[HSD_GOBJ_PLINK_FIGHTER];g;g=g->next) {
        Fighter* fp=GET_FIGHTER(g);struct CpuFighter cpu;
        if(!fp)continue;
        cpu=fp->cpu;
        if((fp->coll_data.env_flags&Collide_LedgeGrabMask) ||
           (fp->motion_id>=ftCo_MS_CliffCatch && fp->motion_id<=ftCo_MS_CliffJumpQuick2)) {
            ftCo_Fall_Enter(g);
            /* cliff.ledge_id aliases fall.smid: do not overwrite the new
             * action's union after Fall has initialized it. */
        }
        script_switch_contacts(fp);fp->cpu=cpu;
    }
}
static int script_switch_room(ScriptStageSlot* s)
{
    int i,k,free=0;
    for(i=0;i<script_stage.cap;++i) {
        if(!script_stage.line[i].active)++free;
        else for(k=0;k<script_switch.nlines;++k)if(script_stage.line[i].handle==script_switch.lines[k]){++free;break;}
    }
    return free>=s->nlines+1;
}
/* Return a reason before ANY writes. State entry from ledge/capture cannot
 * preserve an arbitrary move's stage references; captures/death/transforms are
 * refused. Ordinary attacks/hitstun keep their action when their point fits. */
int ScriptGame_StageSwitchCheck(int handle,int owner,int place)
{
    HSD_GObj* g; ScriptStageSlot* s=script_slot(handle,owner);
    if(!s) return -5;
#ifndef STAGE_SLOT_FIXTURE
    if(s->dynamic==3 && lbHeap_Free(0)<ST_SLOT_HEADROOM+256*1024)return -4;
#endif
    { int i,has_floor=0;for(i=0;i<s->nlines;++i)if(s->lines[i].kind==1)has_floor=1;
      if(!has_floor)return -2; }
    if(Netplay_Enabled() || RB_Enabled() || Replay_Active()) return -1;
    if(script_switch.owner && script_switch.owner!=owner) return -6;
    if(!script_stage.map || mpLib_8004D164()!=script_stage.map || !script_switch_room(s)) return -7;
    if(place<0 || place>1) return -8;
    if(!script_stage_host_allowed()) return -6;
    if(ScriptGame_ArenaOwner() || script_camera_params.owner) return -6;
    if(HSD_GObjPLinkHead) for(g=HSD_GObjPLinkHead[HSD_GOBJ_PLINK_FIGHTER];g;g=g->next) {
        Fighter* fp=GET_FIGHTER(g);
        if(!fp) continue;
        if(fp->victim_gobj || fp->x1A5C || fp->motion_id<=ftCo_MS_RebirthWait ||
           (fp->motion_id>=ftCo_MS_CapturePulledHi && fp->motion_id<=ftCo_MS_CaptureFoot) ||
           (fp->motion_id>=ftCo_MS_ThrownF && fp->motion_id<=ftCo_MS_ThrownlwWomen)) return -9;
    }
    return 0;
}
static void script_switch_clear_lines(void)
{
    int i;
    /* mpIsland_8005B004 follows id0 until -1 or back to its START line;
     * it cannot safely walk a partially retired DAT cycle. Cut the whole
     * retiring graph before StageRemove refreshes any one joint's island. */
    for(i=0;i<script_switch.nlines;++i) {
        int at=script_stage_seam_slot(script_switch.lines[i]);
        if(at>=0) {
            MapLine* l=&script_stage.map->lines[script_stage.base_l+at];
            l->prev_id0=l->next_id0=l->prev_id1=l->next_id1=-1;
        }
    }
    for(i=0;i<script_switch.nlines;++i) {
#ifndef STAGE_SLOT_FIXTURE
        int at=script_stage_seam_slot(script_switch.lines[i]);
        if(at>=0)mpGetGroundCollJoint()[script_stage.base_j+at].x20=NULL;
#endif
        ScriptGame_StageRemove(script_switch.lines[i]);
    }
    script_switch.nlines=0;
}
int ScriptGame_StageSwitch(int handle,int owner,int serial,int place)
{
    ScriptStageSlot* s=script_slot(handle,owner); int i,rc,safety;
    HSD_GObj* g;
    rc=ScriptGame_StageSwitchCheck(handle,owner,place); if(rc) return rc;
    if(!ScriptGame_StageIsolate(owner,1)) return -6;
    if(!script_switch.owner) {
        script_switch.owner=owner; script_switch.camera=stage_info.cam_info;
        script_switch.blast=stage_info.blast_zone; script_switch.param=stage_info.param;
        script_switch.music=lbAudioAx_StageSlotMusic();
#ifndef STAGE_SLOT_FIXTURE
        Camera_StageSlotClipGet(&script_switch.nearz,&script_switch.farz);
#endif
        memcpy(script_switch.markers,stage_info.x280,sizeof script_switch.markers);
    }
    script_switch_detach();
    script_slot_native_leave();
    stage_info.param=s->param;
    script_switch_clear_lines();
    safety=ScriptGame_StageAddLine(script_mesh_bits(-49000),script_mesh_bits(-49000),
        script_mesh_bits(49000),script_mesh_bits(-49000),1,4,serial++);
    /* The no-allocation commit path is preflighted. A script cannot interleave. */
    for(i=0;i<s->nlines;++i) {
        StDatLine* l=&s->lines[i]; int h,j,index;
        float scale=s->nmesh?1.0f:s->param->y;
        h=ScriptGame_StageAddLine(script_mesh_bits(l->x0*scale),script_mesh_bits(l->y0*scale),
              script_mesh_bits(l->x1*scale),script_mesh_bits(l->y1*scale),l->kind,
              4|((l->hi|l->lo)&LINE_FLAG_PLATFORM?1:0)|(l->lo&LINE_FLAG_LEDGE?2:0),serial++);
        script_switch.lines[script_switch.nlines++]=h;
        index=script_stage_seam_slot(h); j=script_stage.base_l+index;
        script_stage.map->lines[j].lo_flags=l->lo;
    }
    /* Preserve all DAT adjacency, including wall/ceiling and ledge endpoints,
     * after handles have their actual reserved line ids. */
    for(i=0;i<s->nlines;++i) {
        StDatLine* l=&s->lines[i];
        MapLine* ml=&script_stage.map->lines[script_stage.base_l+script_stage_seam_slot(script_switch.lines[i])];
        ml->prev_id0=l->prev0<0?-1:script_stage.base_l+script_stage_seam_slot(script_switch.lines[l->prev0]);
        ml->next_id0=l->next0<0?-1:script_stage.base_l+script_stage_seam_slot(script_switch.lines[l->next0]);
        ml->prev_id1=l->prev1<0?-1:script_stage.base_l+script_stage_seam_slot(script_switch.lines[l->prev1]);
        ml->next_id1=l->next1<0?-1:script_stage.base_l+script_stage_seam_slot(script_switch.lines[l->next1]);
    }
    script_stage_seam_refresh();
    script_switch.active=handle; stage_info.param=s->param;
    script_slot_dynamic_bind();
    for(i=0;i<ST_SLOT_COUNT;++i) {
        int k;for(k=0;k<script_slots[i].nmesh;++k){int at=ScriptGame_ModelSlot(script_slots[i].mesh[k]);if(at>=0)script_stage.instance[at].field[SM_VISIBLE]=script_slots[i].handle==handle;}
    }
    stage_info.cam_info=s->camera; stage_info.blast_zone=s->blast;
#ifndef STAGE_SLOT_FIXTURE
    Camera_800311DC(script_stage_same_file(s->file,"GrNLa.dat")?1:script_switch.nearz);
    Camera_800311CC(script_stage_same_file(s->file,"GrNLa.dat")?30000:script_switch.farz);
#endif
    memcpy(stage_info.x280,s->markers,sizeof s->markers);
    script_slot_native_enter(s);
    if(s->dynamic!=3)script_slot_dynamic_bind();
    if(HSD_GObjPLinkHead) for(g=HSD_GObjPLinkHead[HSD_GOBJ_PLINK_FIGHTER];g;g=g->next) {
        Fighter* fp=GET_FIGHTER(g); float x,y,best=1e30f,px=0,py=0; int floor=-1;
        struct CpuFighter saved_cpu;
        if(!fp) continue;
        saved_cpu=fp->cpu;
        x=fp->cur_pos.x; y=fp->cur_pos.y;
        for(i=0;i<s->nlines;++i) {
            StDatLine current=s->lines[i];StDatLine* l=&current;float t,fx,fy,d;
#ifndef STAGE_SLOT_FIXTURE
            {int at=script_stage_seam_slot(script_switch.lines[i]);CollVtx* v=&mpGetGroundCollVtx()[script_stage.base_v+2*at];
                if(!(mpGetGroundCollLine()[script_stage.base_l+at].flags&LINE_FLAG_ENABLED))continue;
                l->x0=v[0].pos.x;l->y0=v[0].pos.y;l->x1=v[1].pos.x;l->y1=v[1].pos.y;}
#endif
            if(l->kind!=1 || l->x0==l->x1) continue;
            t=(x-l->x0)/(l->x1-l->x0); if(t<0)t=0; if(t>1)t=1;
            fx=l->x0+t*(l->x1-l->x0); fy=l->y0+t*(l->y1-l->y0);
            d=(fx-x)*(fx-x)+(fy-y)*(fy-y);
            if(d<best) {best=d; px=fx; py=fy;}
            if(x>=(l->x0<l->x1?l->x0:l->x1) && x<=(l->x0>l->x1?l->x0:l->x1) && y>=fy-0.1f) floor=i;
        }
        if(floor<0 || x<s->blast.left+s->camera.cam_x_offset || x>s->blast.right+s->camera.cam_x_offset ||
           y<s->blast.bottom+s->camera.cam_y_offset || y>s->blast.top+s->camera.cam_y_offset) {
            int outside=x<s->blast.left+s->camera.cam_x_offset || x>s->blast.right+s->camera.cam_x_offset ||
                y<s->blast.bottom+s->camera.cam_y_offset || y>s->blast.top+s->camera.cam_y_offset;
            fp->cur_pos.x=px; fp->cur_pos.y=place && outside?s->blast.bottom+s->camera.cam_y_offset-100:py+0.1f;
            /* Deliberate state entry only for relocated/ledge fighters. */
            ftCo_Fall_Enter(fp->gobj);
        } else if((fp->coll_data.env_flags&Collide_LedgeGrabMask) ||
                  (fp->motion_id>=ftCo_MS_CliffCatch && fp->motion_id<=ftCo_MS_CliffJumpQuick2)) {
            ftCo_Fall_Enter(fp->gobj);
        }
        script_switch_contacts(fp);
        if(fp->ground_or_air==GA_Ground) {
            Vec3 p,normal; int line=-1; u32 flags=0;
            if(mpCheckFloor(fp->cur_pos.x,fp->cur_pos.y+0.5f,fp->cur_pos.x,fp->cur_pos.y-0.5f,
                  0,&p,&line,&flags,&normal,-1,-1,-1,NULL,NULL)) {
                fp->coll_data.floor.index=line; fp->coll_data.floor.flags=flags; fp->coll_data.floor.normal=normal;
            } else ftCo_Fall_Enter(fp->gobj);
        }
        ftCamera_80076064(fp);
        /* State entry can re-arm AI. Restore each entity, including Nana and
         * inactive transformation halves; persistent Player cpu_type is unchanged. */
        fp->cpu=saved_cpu;
    }
    /* Static stage NPCs are stage-bound; portable items/fighter projectiles
     * survive but lose all collision contacts into the removed map. */
    if(HSD_GObjPLinkHead) {
        HSD_GObj* next;
        for(g=HSD_GObjPLinkHead[HSD_GOBJ_PLINK_ITEM];g;g=next) {
            Item* ip=GET_ITEM(g); next=g->next; if(!ip) continue;
            if(ip->kind>=It_Kind_Old_Kuri) { Item_8026A8EC(g); continue; }
            ip->x378_itemColl.floor.index=ip->x378_itemColl.ceiling.index=-1;
            ip->x378_itemColl.left_facing_wall.index=ip->x378_itemColl.right_facing_wall.index=-1;
            ip->x378_itemColl.floor_skip=ip->x378_itemColl.ledge_id_left=ip->x378_itemColl.ledge_id_right=-1;
            ip->x378_itemColl.joint_id_skip=ip->x378_itemColl.joint_id_only=-1;
            ip->x378_itemColl.env_flags=ip->x378_itemColl.prev_env_flags=0;
            mpColl_80043680(&ip->x378_itemColl,&ip->pos);
        }
    }
    ScriptGame_StageRemove(safety);
    ScriptGame_StageMusicRequest(s->music);
    ++script_switch.epoch;
    OSReport("stage switch: slot=%d epoch=%d lines=%d phase=%d heap0_free=%u (offline)\n",handle,script_switch.epoch,s->nlines,s->dynamic,lbHeap_Free(0));
    return 0;
}
void ScriptGame_StageSlotsRelease(int owner)
{
    int i;
    if(script_switch.owner && (!owner || script_switch.owner==owner)) {
        script_switch_detach();
        script_slot_native_leave();
        script_switch_clear_lines();
        stage_info.cam_info=script_switch.camera; stage_info.blast_zone=script_switch.blast;
        stage_info.param=script_switch.param;
#ifndef STAGE_SLOT_FIXTURE
        Camera_800311DC(script_switch.nearz);Camera_800311CC(script_switch.farz);
#endif
        memcpy(stage_info.x280,script_switch.markers,sizeof script_switch.markers);
        ScriptGame_StageIsolationClear(script_switch.owner);
        ScriptGame_StageMusicRequest(script_switch.music);
        script_switch.active=script_switch.owner=0;
        for(i=0;i<ST_SLOT_COUNT;++i){int k;for(k=0;k<script_slots[i].nmesh;++k){int at=ScriptGame_ModelSlot(script_slots[i].mesh[k]);if(at>=0)script_stage.instance[at].field[SM_VISIBLE]=0;}}
        /* Never let a fighter retain the slot map on unload. */
        if(HSD_GObjPLinkHead) {
            HSD_GObj* g; for(g=HSD_GObjPLinkHead[HSD_GOBJ_PLINK_FIGHTER];g;g=g->next)
                if(g->user_data) { Fighter* fp=GET_FIGHTER(g);struct CpuFighter cpu=fp->cpu;
                    ftCo_Fall_Enter(g);script_switch_contacts(fp);fp->cpu=cpu; }
        }
    }
    for(i=0;i<ST_SLOT_COUNT;++i) if(script_slots[i].handle && (!owner || script_slots[i].owner==owner)) script_slot_dispose(&script_slots[i]);
}
int ScriptGame_StageSlotFree(int handle,int owner)
{
    ScriptStageSlot* s=script_slot(handle,owner);
    if(!s || script_switch.active==handle) return 0;
    script_slot_dispose(s); return 1;
}
int ScriptGame_StageSlotsActive(void)
{
    int i; for(i=0;i<ST_SLOT_COUNT;++i) if(script_slots[i].handle) return 1;
    return 0;
}

#include "script_stage_slot_groups.inc"
#include "script_stage_slot_dynamic.inc"
#ifndef STAGE_SLOT_FIXTURE
#include "script_stage_slot_native.inc"
#endif
