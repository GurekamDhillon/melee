/* One live retail visit. Cache bytes, not native stage instances. No file read
 * or scene/heap reset appears in the commit path. Ground owns its real objects. */
#include <melee/ft/ftdevice.h>
#include <sysdolphin/baselib/particle.h>
#include <sysdolphin/baselib/generator.h>
#include <sysdolphin/baselib/psappsrt.h>
#include <sysdolphin/baselib/list.h>
extern HSD_Particle* hsd_804D0908[16];
extern struct hsd_804D0F60_t hsd_804D0F60;
extern u16 hsd_804D78E2;
extern u32* hsd_804D0948[65];
extern HSD_PSFormGroup** psFormGroupArray[65];
extern int Stage_StageSlotSelect(int);
extern void Ground_StageSlotRetailInstall(StageIdPair*,HSD_Archive*);
extern void Ground_StageSlotFighterLightingClear(void);
extern void mpIsland_StageSlotFree(void);
extern void grZakoGenerator_StageSlotRelease(void);
static struct {
    int stashed;unsigned bytes;int kind,music;
    void* working;unsigned working_bytes;HSD_Archive* archive;
    MapCollData* prepared;
    void* buffer;
    int switched;
} script_retail;

void ScriptGame_StageSlotRetailCaptureBegin(void)
{
    /* Normal scene init. Previous scene storage has already been released by
     * the scene owner: never dereference/free its stale ledger or raw cache. */
    Script_StageRawStash(4,0,0,0);
    memset(&script_retail,0,sizeof script_retail);
    memset(&script_retail_owned,0,sizeof script_retail_owned);
    script_retail_capture=!STAGE_SLOT_PARTIAL_FALLBACK;
    script_retail_live=script_retail_capture;
}
void ScriptGame_StageSlotRetailCaptureResume(void)
{script_retail_capture=!STAGE_SLOT_PARTIAL_FALLBACK && script_retail_live;}
void ScriptGame_StageSlotRetailCaptureEnd(void)
{
    if(STAGE_SLOT_PARTIAL_FALLBACK)return;
    /* Script draw objects are independent; they were created by StageReady
     * inside Ground's capture boundary, but must survive every visit. */
    st_owned_remove(&script_retail_owned,script_stage.draw);
    st_owned_remove(&script_retail_owned,script_stage.background_draw);
    script_retail_capture=0;
}
int ScriptGame_StageSlotRetailLoading(void){return script_retail_loading;}
void ScriptGame_StageSlotRetailBuffer(void* p){script_retail.buffer=p;}
static void script_retail_forget_devices(HSD_GObj* g)
{
    int i;
    if(st_owned_find(&script_retail_owned,g)<0)return;
    if(ft_80459A68[0].ground==g) {
        memset(&ft_80459A68[0],0,sizeof ft_80459A68[0]);
        if(ft_804D6578.x0>0)--ft_804D6578.x0;
    }
    if(ft_80459A8C[0].ground==g) {
        memset(&ft_80459A8C[0],0,sizeof ft_80459A8C[0]);
        if(ft_804D6570>0)--ft_804D6570;
    }
    for(i=0;i<2;++i)if(ftDevice_BuryThings[i].x0==g) {
        memset(&ftDevice_BuryThings[i],0,sizeof ftDevice_BuryThings[i]);
        if(ftDevice_BuryThingCount>0)--ftDevice_BuryThingCount;
    }
}
static void script_retail_clear_banks(void)
{
    int i;for(i=0;i<2;++i) {
        int bank=i?0x40:0x1E;
        hsd_804D0948[bank]=NULL;psFormGroupArray[bank]=NULL;
        psTexGroupArray[bank]=NULL;psNumCmdList[bank]=NULL;
        psCmdListArray[bank]=0;ptclref_804D0E5C[bank]=NULL;
    }
}
static int script_retail_bank(int bank){return bank==0x1E || bank==0x40;}
static void script_retail_particles(void)
{
    HSD_SList* node=(HSD_SList*)hsd_804D78F4;HSD_SList* prevnode=NULL;
    HSD_Generator* gen;HSD_Generator* prevgen=NULL;int i;
    /* grLib creates stage-bank generators on link zero too. Ownership follows
     * the reserved stage bank, not the render link, for both queued and live. */
    while(node) {
        HSD_Generator* g=node->data;
        if(script_retail_bank(g->bank)) {
            HSD_SList* next=HSD_SListRemove(node);
            if(prevnode)prevnode->next=next;else hsd_804D78F4=(u32)next;
            node=next;
        } else {prevnode=node;node=node->next;}
    }
    for(i=0;i<16;++i) {
        HSD_Particle* p=hsd_804D0908[i];HSD_Particle* prev=NULL;
        while(p) {
            HSD_Particle* next=p->next;
            if(script_retail_bank(p->bank) || (p->gen && script_retail_bank(p->gen->bank))) {
                if(prev)prev->next=next;else hsd_804D0908[i]=next;
                if(p->gen && p->gen->userfunc && p->gen->userfunc->hookDelete)
                    p->gen->userfunc->hookDelete(p);
                if(p->gen)p->gen->numChild--;
                if(p->appsrt)psRemoveParticleAppSRT(p);
                hsd_8039D048(p);HSD_ObjFree(&hsd_804D0F60.alloc_data,p);hsd_804D78E2--;
            } else prev=p;
            p=next;
        }
    }
    gen=hsd_804D78FC;
    while(gen) {
        HSD_Generator* next=gen->next;
        if(script_retail_bank(gen->bank)) {
            if(hsd_8039D3AC(gen,prevgen)==gen) {
                /* All children/SRT references must already have drained. */
                OSReport("stage switch: ERROR stage bank generator retained\n");
                prevgen=gen;
            }
        } else prevgen=gen;
        gen=next;
    }
    HSD_StageSlotParticlesClear();
}
static int script_slot_retail_cache_host(void)
{
    char path[262];const char* file;int entry;size_t actual;
    if(script_retail.stashed)return 1;
    if(!script_retail_live || !st_retail_supported(stage_info.grkind))return 0;
    file=Ground_StageSlotDataFile(stage_info.grkind);if(!file)return 0;
    path[0]='/';while(*file=='/')++file;
    if(strlen(file)>250)return 0;
    strcpy(path+1,file);
    if(!strchr(file,'.'))strcat(path,".dat");
    entry=DVDConvertPathToEntrynum(path);if(entry<0)return 0;
    script_retail.bytes=lbFile_8001634C(entry);
    if(script_retail.bytes<32 || script_retail.bytes>8*1024*1024)return 0;
    {
        void* tmp=lbHeap_StageSlotTryAlloc((script_retail.bytes+31)&~31u);
        if(!tmp)return 0;
        lbFile_8001668C(path,tmp,&actual);
        /* Native stash, not heap 0: see gw_Script_StageRawStash. */
        if(actual!=script_retail.bytes || !Script_StageRawStash(0,(unsigned)&script_retail,(unsigned)tmp,actual)) {
            lbHeap_80015CA8(0,tmp);return 0;
        }
        lbHeap_80015CA8(0,tmp);script_retail.stashed=1;
    }
    script_retail.kind=stage_info.grkind;script_retail.music=lbAudioAx_StageSlotMusic();
    return 1;
}
static void script_slot_retail_metadata(ScriptStageSlot* s)
{
    int i;unsigned before=lbHeap_Free(0);
    s->numeric_param=*s->param;
    for(i=0;i<261;++i)if(s->markers[i]) {
        lb_8000B1CC(s->markers[i],NULL,&s->marker_pos[i]);s->marker_present[i]=1;
    }
    for(i=0;i<s->ngroups;++i)if(s->models[i]) {
        HSD_GObjFree(s->models[i]);s->models[i]=NULL;
    }
    memset(s->markers,0,sizeof s->markers);
    lbHeap_80015CA8(0,s->archive);lbHeap_80015CA8(0,s->data);
    s->archive=NULL;s->data=NULL;s->param=&s->numeric_param;
    s->bytes=lbHeap_Free(0)>=before && lbHeap_Free(0)-before<s->bytes?
        s->bytes-(lbHeap_Free(0)-before):s->raw_bytes;
    s->model_estimate=0;
}
static int script_retail_kind(ScriptStageSlot* s)
{
    static const char* files[]={"GrNLa.dat","GrNBa.dat","GrSt.dat","GrOp.dat","GrIz.dat"};
    static const int kinds[]={Gr_Kind_Last,Gr_Kind_Battle,Gr_Kind_Story,Gr_Kind_OldPupupu,Gr_Kind_Izumi};
    int i;for(i=0;i<5;++i)if(script_stage_same_file(s->file,files[i]))return kinds[i];
    return -1;
}
static int script_slot_retail_check(ScriptStageSlot* s)
{
    int kind=script_retail_kind(s);
    unsigned required;
    if(!s->raw_bytes || kind<0 || !st_retail_supported(kind) ||
       !script_retail_live || !script_retail.stashed)return -18;
    if(!st_retail_supported(stage_info.grkind))return -18;
    if(!st_retail_capacity(s->nverts,s->nlines,s->njoints,script_stage.cap))return -7;
    /* Simultaneously retain the current visit and allocate the replacement.
     * Fixed collision caches plus native constructors are conservatively charged. */
    {int i;if(script_stage.target_model_ready)return -6;
     for(i=0;i<SCRIPT_STAGE_ENEMIES;++i)if(script_stage.enemy[i].active)return -6;
     for(i=0;i<3;++i)if(script_stage.enemy_archive[i])return -6;
     for(i=0;i<SCRIPT_STAGE_TARGETS;++i)if(script_stage.target[i].active)return -6;}
    required=s->raw_bytes+sizeof(HSD_Archive)+2048*sizeof(CollVtx)+1536*sizeof(CollLine)+
        SCRIPT_STAGE_JOINTS*sizeof(CollJoint)+2048*sizeof(Vec2)+1536*sizeof(MapLine)+
        SCRIPT_STAGE_JOINTS*sizeof(MapJoint)+2*1024*1024+ST_SLOT_HEADROOM;
    return lbHeap_Free(0)<required?-4:0;
}
static MapCollData* script_retail_prepare(MapCollData* src)
{
    MapCollData* dst;int cap=script_stage.cap;
    if(!src || !st_retail_capacity(src->vert_count,src->line_count,src->joint_count,cap))return NULL;
    dst=lbHeap_StageSlotTryAlloc(sizeof *dst);if(!dst)return NULL;
    *dst=*src;
    dst->verts=lbHeap_StageSlotTryAlloc((src->vert_count+cap*2)*sizeof *dst->verts);
    dst->lines=lbHeap_StageSlotTryAlloc((src->line_count+cap)*sizeof *dst->lines);
    dst->joints=lbHeap_StageSlotTryAlloc((src->joint_count+cap)*sizeof *dst->joints);
    if(!dst->verts || !dst->lines || !dst->joints) {
        if(dst->verts)lbHeap_80015CA8(0,dst->verts);
        if(dst->lines)lbHeap_80015CA8(0,dst->lines);
        if(dst->joints)lbHeap_80015CA8(0,dst->joints);
        lbHeap_80015CA8(0,dst);return NULL;
    }
    memset(dst->verts,0,(src->vert_count+cap*2)*sizeof *dst->verts);
    memset(dst->lines,0,(src->line_count+cap)*sizeof *dst->lines);
    memset(dst->joints,0,(src->joint_count+cap)*sizeof *dst->joints);
    memcpy(dst->verts,src->verts,src->vert_count*sizeof *dst->verts);
    memcpy(dst->lines,src->lines,src->line_count*sizeof *dst->lines);
    memcpy(dst->joints,src->joints,src->joint_count*sizeof *dst->joints);
    return dst;
}
static void script_retail_free_map(MapCollData* map)
{
    if(!map)return;
    HSD_Free(map->verts);HSD_Free(map->lines);HSD_Free(map->joints);HSD_Free(map);
}
void ScriptGame_StageSlotRetailCollision(void)
{
    MapCollData* old=script_stage.map;MapCollData* dst=script_retail.prepared;
    CollVtx* oldv=mpGetGroundCollVtx();CollLine* oldl=mpGetGroundCollLine();
    CollJoint* oldj=mpGetGroundCollJoint();
    int i,k,ov=script_stage.base_v,ol=script_stage.base_l,oj=script_stage.base_j;
    int nv=dst->vert_count,nl=dst->line_count,nj=dst->joint_count;
    script_stage.base_v=nv;script_stage.base_l=nl;script_stage.base_j=nj;script_stage.map=dst;
    script_arena.groups=nj;
    mpIsland_StageSlotFree();
    mpLibLoad(dst);
    dst->vert_count+=script_stage.cap*2;dst->line_count+=script_stage.cap;
    dst->joint_count+=script_stage.cap;
    for(i=0;i<script_stage.cap;++i) {
        MapLine* line=&dst->lines[nl+i];MapJoint* joint=&dst->joints[nj+i];
        CollJoint* runtime=&mpGetGroundCollJoint()[nj+i];
        *line=old->lines[ol+i];*joint=old->joints[oj+i];
        line->v0_idx+=nv-ov;line->v1_idx+=nv-ov;
        if(line->prev_id0>=0)line->prev_id0+=nl-ol;
        if(line->prev_id1>=0)line->prev_id1+=nl-ol;
        if(line->next_id0>=0)line->next_id0+=nl-ol;
        if(line->next_id1>=0)line->next_id1+=nl-ol;
        joint->vtx_start+=nv-ov;
        for(k=0;k<MapLineGroup_Count;++k)if(joint->ranges[k].count)joint->ranges[k].start+=nl-ol;
        for(k=0;k<2;++k) {
            dst->verts[nv+2*i+k]=old->verts[ov+2*i+k];
            mpGetGroundCollVtx()[nv+2*i+k]=oldv[ov+2*i+k];
        }
        mpGetGroundCollLine()[nl+i]=oldl[ol+i];mpGetGroundCollLine()[nl+i].x0=line;
        *runtime=oldj[oj+i];runtime->inner=joint;runtime->next=NULL;
        if(script_stage.line[i].active && ScriptGame_AreaVisible(script_stage.line[i].area))mpJointListAdd(nj+i);
    }
    script_stage_seam_refresh();
    HSD_Free(oldv);HSD_Free(oldl);HSD_Free(oldj);script_retail_free_map(old);
    script_retail.prepared=NULL;
}
static void script_retail_pool_check(const char* where)
{
    /* Diagnostic: a pool whose free counter and free list disagree makes the next
     * HSD_ObjAlloc dereference NULL. Log it at each phase of a visit. */
    HSD_ObjAllocData* pools[2]={&gobj_alloc_data,&gobjproc_alloc_data};int k;
    for(k=0;k<2;++k) {
        unsigned n=0;HSD_ObjAllocLink* l=pools[k]->freehead;
        while(l && n<=pools[k]->free+4){++n;l=l->next;}
        if(n!=(unsigned)pools[k]->free)OSReport("stage switch: ERROR pool %s %s free=%u list=%u used=%u\n",k?"proc":"gobj",where,
                 (unsigned)pools[k]->free,n,(unsigned)pools[k]->used);
    }
}
static void script_retail_destroy(void)
{
    int i;HSD_GObj* g;
    script_retail_capture=0;
    script_retail_pool_check("destroy-begin");
    Ground_StageSlotFighterLightingClear();
    script_retail_particles();
    /* Ground destructors own subordinate cameras and marker/collision links.
     * Run first while outgoing archive/map/bank data are all still installed. */
    for(;;) {
        g=NULL;
        for(i=0;i<script_retail_owned.count;++i) {
            HSD_GObj* candidate=script_retail_owned.entry[i].object;
            if(candidate->classifier==HSD_GOBJ_CLASS_STAGE && candidate->user_data &&
               candidate->user_data_kind==3) {g=candidate;break;}
        }
        if(!g)break;
        /* Retail Ground_801C4A08 frees gp->x18. Fountain's two platforms and its
         * main object share one x18 (grizumi.c: platground->x18 = xC8), freed once per
         * owner: the scene exit that always followed never noticed. Free it only while
         * it is still a live owned object, else the GObj pool free list becomes a cycle
         * and the next stage's first GObj_Create dereferences NULL. */
        {Ground* gp=g->user_data;
         if(gp->x18 && st_owned_find(&script_retail_owned,gp->x18)<0)gp->x18=NULL;}
        Ground_801C4A08(g);
    }
    while(script_retail_owned.count) {
        g=script_retail_owned.entry[script_retail_owned.count-1].object;
        if(g->p_link==HSD_GOBJ_PLINK_ITEM) {
            GET_ITEM(g)->destroy_type=3;Item_8026A8EC(g);
        } else if(g->classifier==HSD_GOBJ_CLASS_EFFECT) {
            efLib_DestroyAll(g);HSD_GObjFree(g);
        } else HSD_GObjFree(g);
    }
    script_retail_particles();script_retail_clear_banks();
    grZakoGenerator_StageSlotRelease();
    memset(it_804A0F60,0,sizeof script_slot_articles);
    if(script_retail.buffer)HSD_Free(script_retail.buffer);script_retail.buffer=NULL;
    if(script_retail.archive)lbHeap_80015CA8(0,script_retail.archive);
    if(script_retail.working)lbHeap_80015CA8(0,script_retail.working);
    script_retail.archive=NULL;script_retail.working=NULL;
    script_retail_pool_check("destroy-end");
}
static void script_retail_items(int remove_stage)
{
    HSD_GObj* g,*next;
    if(!HSD_GObjPLinkHead)return;
    for(g=HSD_GObjPLinkHead[HSD_GOBJ_PLINK_ITEM];g;g=next) {
        Item* ip=GET_ITEM(g);CollData* c;next=g->next;if(!ip)continue;
        if(remove_stage && (ip->kind>=It_Kind_Old_Kuri ||
           st_owned_find(&script_retail_owned,ip->owner)>=0 || st_owned_find(&script_retail_owned,ip->x51C)>=0 ||
           (script_retail.working && ((unsigned)ip->xC4_article_data-(unsigned)script_retail.working)<
              script_retail.working_bytes))) {ip->destroy_type=3;Item_8026A8EC(g);continue;}
        c=&ip->x378_itemColl;
        c->floor.index=c->ceiling.index=c->left_facing_wall.index=c->right_facing_wall.index=-1;
        c->floor_skip=c->ledge_id_left=c->ledge_id_right=c->joint_id_skip=c->joint_id_only=-1;
        c->floor.flags=c->ceiling.flags=c->left_facing_wall.flags=c->right_facing_wall.flags=0;
        c->env_flags=c->prev_env_flags=c->x13C=0;
        c->contact.x=c->contact.y=c->contact.z=0;c->x28_vec.x=c->x28_vec.y=c->x28_vec.z=0;
        c->prev_ecb=c->xE4_ecb=c->desired_ecb=c->ecb;mpColl_80043680(c,&ip->pos);
    }
}
#include "script_stage_slot_carry.inc"
static int script_retail_visit(unsigned rawkey,unsigned bytes,int kind,int music,int place)
{
    void* work;HSD_Archive* archive;MapCollData* map;StageIdPair pair;
    int i;unsigned free_in=lbHeap_Free(0),free_prepared,free_torn;
    (void)place; /* Full visits always use continuous carry, including legacy KO requests. */
    if(lbHeap_Free(0)<bytes+sizeof(HSD_Archive)+2048*sizeof(CollVtx)+1536*sizeof(CollLine)+
        SCRIPT_STAGE_JOINTS*sizeof(CollJoint)+2048*sizeof(Vec2)+1536*sizeof(MapLine)+
        SCRIPT_STAGE_JOINTS*sizeof(MapJoint)+2*1024*1024+ST_SLOT_HEADROOM)return -4;
    work=lbHeap_StageSlotTryAlloc((bytes+31)&~31u);
    archive=lbHeap_StageSlotTryAlloc(sizeof *archive);
    if(!work || !archive) {
        if(work)lbHeap_80015CA8(0,work);if(archive)lbHeap_80015CA8(0,archive);return -4;
    }
    if(!Script_StageRawStash(1,rawkey,(unsigned)work,bytes)) {
        lbHeap_80015CA8(0,archive);lbHeap_80015CA8(0,work);return -4;
    }
    if(HSD_ArchiveParse(archive,work,bytes)<0) {
        lbHeap_80015CA8(0,archive);lbHeap_80015CA8(0,work);return -2;
    }
    for(i=0;HSD_ArchiveGetExtern(archive,i);++i)
        HSD_ArchiveLocateExtern(archive,HSD_ArchiveGetExtern(archive,i),NULL);
    map=script_retail_prepare(HSD_ArchiveGetPublicAddress(archive,"coll_data"));
    if(!map){lbHeap_80015CA8(0,archive);lbHeap_80015CA8(0,work);return -4;}
    free_prepared=lbHeap_Free(0);
    /* Commit begins only after private archive and complete map allocations.
     * Contacts are detached before retail destroys their old joint owners. */
    ScriptGame_StageIsolationClear(0);
    script_switch_detach();script_retail_items(1);script_retail_destroy();free_torn=lbHeap_Free(0);
    script_retail.working=work;script_retail.working_bytes=bytes;script_retail.archive=archive;script_retail.prepared=map;
    script_retail_loading=1;script_retail_capture=1;
    pair.grkind=kind;pair.stkind=(StKind)Stage_StageSlotSelect(kind);
    /* Camera initialization defaults (Camera_80028B9C), before the actual
     * retail on_init can apply FD/background-specific clips. Keep its GObj. */
    Camera_800311DC(0.1f);Camera_800311CC(16384.0f);
    Ground_StageSlotRetailInstall(&pair,archive);
    script_retail_pool_check("installed");
    Camera_80030730(Ground_801C20D0());Ground_EnableMatchCamera();
    script_retail_loading=0;script_retail_capture=0;
    script_retail_carry();
    script_retail_items(0);
    ScriptGame_StageMusicRequest(music);
    OSReport("stage switch: complete retail visit kind=%d objects=%d heap0_free=%u in=%u prepared=%u torn=%u\n",
             kind,script_retail_owned.count,lbHeap_Free(0),free_in,free_prepared,free_torn);
    return 0;
}
static int script_slot_retail_switch(ScriptStageSlot* s,int owner,int place)
{
    int rc=script_retail_visit((unsigned)s,s->raw_bytes,script_retail_kind(s),s->music,place);
    if(rc)return rc;
    script_retail.switched=1;script_switch.owner=owner;script_switch.active=s->handle;
    s->dynamic=4;s->dynamic_kind=stage_info.grkind;++script_switch.epoch;
    return 0;
}
static int script_slot_retail_release(int owner)
{
    if(!script_retail.switched || (owner && owner!=script_switch.owner))return 1;
    if(script_retail_visit((unsigned)&script_retail,script_retail.bytes,script_retail.kind,script_retail.music,0)) {
        /* Never free a live visit's private archive on restoration failure. */
        OSReport("stage switch: original restoration refused (resource capacity)\n");return 0;
    }
    script_retail.switched=0;script_switch.owner=script_switch.active=0;++script_switch.epoch;return 1;
}
