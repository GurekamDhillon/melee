#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "../gameworld/script_stage_owned.h"
#define STAGE_SLOT_PARTIAL_FALLBACK 0
#define HSD_GOBJ_CLASS_STAGE 3
#define HSD_GOBJ_CLASS_EFFECT 8
#define HSD_GOBJ_PLINK_ITEM 6
#define OSReport(...) ((void)0)
typedef struct Obj {int classifier,p_link,user_data_kind,alive;void* user_data;struct Obj* camera;} HSD_GObj;
typedef struct {int destroy_type;} Item;
#define GET_ITEM(g) ((Item*)(g)->user_data)
typedef struct {int value;} Article;
typedef struct {void* callbacks;void* on_touch_line;void* on_check_shadow_render;} StageData;
typedef struct {int grkind,stkind;} StageIdPair;
typedef struct {HSD_GObj* x18;} Ground; /* retail Ground::x18, shared by Fountain platforms */
typedef struct {int value;} HSD_Archive;
typedef struct {float x,y;} Vec2;
#define MapLineGroup_Count 5
typedef struct {int v0_idx,v1_idx,prev_id0,prev_id1,next_id0,next_id1;} MapLine;
typedef struct {struct {int start,count;}ranges[5];int vtx_start;} MapJoint;
typedef struct {int vert_count,line_count,joint_count;Vec2* verts;MapLine* lines;MapJoint* joints;} MapCollData;
typedef struct {int value;}CollVtx;
typedef struct {MapLine* x0;int flags;}CollLine;
typedef struct CollJoint {struct CollJoint* next;MapJoint* inner;int value;}CollJoint;
typedef struct {int kind,grkind;void* on_touch_line;void* on_check_shadow_render;} StageInfo;
static StageInfo stage_info;
static StageData data={0},*stage_datas[3]={&data,&data,&data};
static StOwned script_retail_owned,script_slot_owned;
static struct {int stashed;unsigned bytes;int kind,music;void* working;unsigned working_bytes;HSD_Archive* archive;void* prepared;void* buffer;int switched;} script_retail;
static struct {HSD_GObj* draw;HSD_GObj* background_draw;MapCollData* map;int base_v,base_l,base_j,cap;struct {int active,area;}line[2];}script_stage;
static struct {int groups;}script_arena;
static int script_retail_capture,script_retail_live,script_retail_loading;
static struct {HSD_GObj* ground;int type;void* active_cb;}ft_80459A68[1],ft_80459A8C[1];
static struct {HSD_GObj* x0;int x4;void* cb;}ftDevice_BuryThings[2];
static struct {int x0;}ft_804D6578;
static int ft_804D6570,ftDevice_BuryThingCount;
static void *hsd_804D0948[65],*psFormGroupArray[65],*psTexGroupArray[65],*psNumCmdList[65],*ptclref_804D0E5C[65];
static int psCmdListArray[65];
static Article *it_804A0F60[30],*script_slot_articles[30];
static Ground ground[2];
static int zako_releases,island_frees,raw_stash_calls;
static int destroyed,rawfree,particle_drains,phase,loads,starts,initcalls;
static HSD_GObj objects[10];
static CollVtx* runtimev;static CollLine* runtimel;static CollJoint* runtimej;
static int relinked,refreshed;
static CollVtx* mpGetGroundCollVtx(void){return runtimev;}
static CollLine* mpGetGroundCollLine(void){return runtimel;}
static CollJoint* mpGetGroundCollJoint(void){return runtimej;}
static int ScriptGame_AreaVisible(int area){return area==0;}
static void mpJointListAdd(int joint){assert(joint>=script_stage.base_j);++relinked;}
static void script_stage_seam_refresh(void){++refreshed;}
static void mpLibLoad(MapCollData* map){int i;runtimev=calloc(12,sizeof *runtimev);runtimel=calloc(12,sizeof *runtimel);runtimej=calloc(12,sizeof *runtimej);for(i=0;i<map->line_count;++i)runtimel[i].x0=&map->lines[i];for(i=0;i<map->joint_count;++i)runtimej[i].inner=&map->joints[i];}
static int ScriptGame_StageSlotNativeContext(void){return 0;}
static void script_retail_forget_devices(HSD_GObj*);
static void ScriptGame_StageSlotDestroyed(HSD_GObj*);
static void ScriptGame_StageSlotCreated(HSD_GObj*);
static void ScriptGame_StageSlotRetailCaptureResume(void);
static void ScriptGame_StageSlotRetailCaptureEnd(void);
static void HSD_GObjFree(HSD_GObj* g){assert(g->alive);g->alive=0;ScriptGame_StageSlotDestroyed(g);++destroyed;}
static void Ground_801C4A08(HSD_GObj* g){Ground* gp=g->user_data;assert(script_retail.archive);if(g->camera){HSD_GObjFree(g->camera);g->camera=NULL;}
    /* retail frees gp->x18 once per owner: a second free of the shared object is the double free the production loop must avoid */
    if(gp && gp->x18)HSD_GObjFree(gp->x18);HSD_GObjFree(g);}
static void Item_8026A8EC(HSD_GObj* g){assert(GET_ITEM(g)->destroy_type==3);HSD_GObjFree(g);}
static void efLib_DestroyAll(HSD_GObj* g){assert(g->alive);}
static void script_retail_particles(void){++particle_drains;}
static void script_retail_pool_check(const char* where){(void)where;}
static void Ground_StageSlotFighterLightingClear(void){}
static void Ground_StageSlotFighterLightingRebuild(void){}
static void mpIsland_StageSlotFree(void){++island_frees;}
static void grZakoGenerator_StageSlotRelease(void){++zako_releases;}
static int Script_StageRawStash(int op,unsigned key,unsigned addr,unsigned bytes){(void)op;(void)key;(void)addr;(void)bytes;++raw_stash_calls;return 1;}
static void HSD_Free(void* p){assert(script_retail_owned.count==0);free(p);}
static void lbHeap_80015CA8(int heap,void* p){(void)heap;assert(script_retail_owned.count==0);++rawfree;free(p);}
static void Ground_801BFFB0(void){assert(phase==0);phase=1;memset(&stage_info,0,sizeof stage_info);}
static void grDatFiles_StageSlotInstall(HSD_Archive* a){assert(a && phase==1);phase=2;}
static void Ground_801C28CC(void* p,int kind){(void)p;(void)kind;assert(phase==2);phase=3;}
static void Ground_801C5878(void){assert(phase==3);phase=4;}
static void Ground_801C0800(StageIdPair* p){int i;(void)p;assert(phase==4);phase=5;++initcalls;ScriptGame_StageSlotRetailCaptureResume();for(i=0;i<4;++i){objects[i]=(HSD_GObj){i<2?3:13,i, i<2?3:0,1,i<2?(void*)&ground[i]:NULL,NULL};ScriptGame_StageSlotCreated(&objects[i]);}ground[0].x18=ground[1].x18=&objects[3];objects[0].camera=&objects[2];ScriptGame_StageSlotRetailCaptureEnd();}
static void Ground_OnLoad(StageIdPair* p){(void)p;assert(phase==5);phase=6;++loads;}
static void Ground_801C0FB8(StageIdPair* p){(void)p;assert(phase==6);phase=7;++starts;ScriptGame_StageSlotRetailCaptureResume();objects[4]=(HSD_GObj){3,5,0,1,NULL,NULL};ScriptGame_StageSlotCreated(&objects[4]);ScriptGame_StageSlotRetailCaptureEnd();}
/* The production Ground helper accesses xA0; fixture remaps that field. */
#define xA0 kind
#include "stage_retail_fixture.inc"
static MapCollData* newmap(int nv,int nl,int nj)
{
    MapCollData* map=malloc(sizeof *map);map->vert_count=nv;map->line_count=nl;map->joint_count=nj;
    map->verts=calloc(12,sizeof *map->verts);map->lines=calloc(12,sizeof *map->lines);map->joints=calloc(12,sizeof *map->joints);return map;
}
static void collision_fixture(void)
{
    MapCollData *old=newmap(6,5,4),*next=newmap(4,4,3);int i;
    script_stage.map=old;script_stage.base_v=2;script_stage.base_l=3;script_stage.base_j=2;script_stage.cap=2;
    runtimev=calloc(12,sizeof *runtimev);runtimel=calloc(12,sizeof *runtimel);runtimej=calloc(12,sizeof *runtimej);
    for(i=0;i<2;++i) {
        MapLine* line=&old->lines[3+i];MapJoint* joint=&old->joints[2+i];
        line->v0_idx=2+2*i;line->v1_idx=3+2*i;line->prev_id0=line->prev_id1=i?3:-1;line->next_id0=line->next_id1=i?-1:4;
        joint->vtx_start=line->v0_idx;joint->ranges[4].start=3+i;joint->ranges[4].count=1;
        runtimev[2+2*i].value=100+i;runtimel[3+i].x0=line;runtimel[3+i].flags=91+i;
        runtimej[2+i].inner=joint;runtimej[2+i].value=200+i;script_stage.line[i].active=1;
    }
    next->lines[1]=(MapLine){1,2,0,0,2,2};script_retail.prepared=next;
    ScriptGame_StageSlotRetailCollision();
    assert(script_stage.map==next && next->vert_count==8 && next->line_count==6 && next->joint_count==5);
    assert(script_arena.groups==3 && script_stage.base_v==4 && script_stage.base_l==4 && script_stage.base_j==3);
    assert(next->lines[1].v0_idx==1 && next->lines[1].next_id0==2); /* native literal indices untouched */
    assert(next->lines[4].v0_idx==4 && next->lines[4].next_id0==5 && next->lines[5].prev_id1==4);
    assert(next->joints[3].vtx_start==4 && next->joints[4].ranges[4].start==5);
    assert(runtimev[4].value==100 && runtimev[6].value==101 && runtimel[4].x0==&next->lines[4] && runtimel[5].flags==92);
    assert(runtimej[3].inner==&next->joints[3] && runtimej[4].value==201 && relinked==2 && refreshed==1);
    assert(script_stage.draw && script_stage.background_draw); /* independent render ownership unchanged */
    HSD_Free(runtimev);HSD_Free(runtimel);HSD_Free(runtimej);script_retail_free_map(next);
    script_stage.map=NULL;
}
int main(void)
{
    int visit;HSD_GObj fighter={4,4,0,1,NULL,NULL},hud={9,9,0,1,NULL,NULL};
    StageIdPair pair={1,1};
    ScriptGame_StageSlotRetailCaptureBegin();
    ScriptGame_StageSlotRetailCaptureEnd();
    ScriptGame_StageSlotCreated(&fighter);ScriptGame_StageSlotCreated(&hud);
    assert(script_retail_owned.count==0);
    script_stage.draw=&fighter;script_stage.background_draw=&hud;
    collision_fixture();
    for(visit=0;visit<100;++visit) {
        phase=0;script_retail.archive=malloc(sizeof(HSD_Archive));script_retail.working=malloc(64);
        script_retail.buffer=malloc(32);
        Ground_StageSlotRetailInstall(&pair,script_retail.archive);
        assert(phase==7 && script_retail_owned.count==5);
        assert(!script_retail_capture && fighter.alive && hud.alive);
        ft_80459A68[0].ground=&objects[0];ft_804D6578.x0=1;
        ft_80459A8C[0].ground=&objects[0];ft_804D6570=1;
        ftDevice_BuryThings[0].x0=&objects[0];ftDevice_BuryThingCount=1;
        hsd_804D0948[30]=psFormGroupArray[30]=psTexGroupArray[30]=psNumCmdList[30]=ptclref_804D0E5C[30]=&data;
        psCmdListArray[30]=1;
        script_retail_destroy();
        assert(script_retail_owned.count==0 && !ft_804D6578.x0 && !ft_804D6570 && !ftDevice_BuryThingCount);
        assert(!hsd_804D0948[30] && !psFormGroupArray[30] && !psTexGroupArray[30] && !psNumCmdList[30] && !ptclref_804D0E5C[30] && !psCmdListArray[30]);
        assert(fighter.alive && hud.alive && !script_retail.archive && !script_retail.working && !script_retail.buffer);
    }
    assert(zako_releases==100 && island_frees==1 && raw_stash_calls==1);
    fprintf(stderr,"d=%d r=%d i=%d l=%d s=%d p=%d\n",destroyed,rawfree,initcalls,loads,starts,particle_drains);
    assert(destroyed==500 && rawfree==200 && initcalls==100 && loads==100 && starts==100 && particle_drains==200);
    puts("full retail source: native map indices and independent seam/runtime rebase; 100 init/load/start + Ground destruction visits, camera dependencies, scoped original capture, device/bank/archive teardown passed");
    return 0;
}
