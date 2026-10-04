#include <assert.h>
#include <stdio.h>
#include <string.h>
#include "../gameworld/script_stage_owned.h"
typedef struct {int classifier,p_link,gx_link;void* hsd_obj;void* user_data;}HSD_GObj;
typedef int bool;typedef struct {float x,y,z;}Vec3;typedef struct {int value;}HSD_JObj;
typedef struct {int destroy_type;}Item;
typedef struct {int value;}Article;
typedef struct {int dynamic,dynamic_kind,handle,njoints;unsigned bytes;void* archive;void* param;HSD_GObj* models[64];HSD_GObj* controllers[2];void* markers[261];HSD_JObj* collision_joints[256];}ScriptStageSlot;
typedef struct {int kind;Article* article;}GroundItemData;
typedef struct {int grkind,flags,unk8C;void* param;void* yakumono_param;GroundItemData** itemdata;HSD_GObj* map_gobjs[64];void* x280[261];int cam_info,blast_zone;bool(*on_check_shadow_render)(Vec3*,int,HSD_JObj*);}StageInfo;
typedef struct {void* unk0;void* unk4;int other;}UnkArchiveStruct;
#define HSD_GOBJ_GXLINK_NONE 255
#define HSD_GOBJ_PLINK_ITEM 6
#define HSD_GOBJ_CLASS_EFFECT 8
#define HSD_GOBJ_CLASS_STAGE 3
#define Gr_Kind_Story 3
#define Gr_Kind_Izumi 4
#define GET_ITEM(g) ((Item*)(g)->user_data)
#define OSReport(...) ((void)0)
static StOwned script_slot_owned;
static ScriptStageSlot* script_slot_native,*script_slot_callback_context;
static StageInfo stage_info,script_slot_native_info,script_slot_proc_saved,script_slot_native_base;
static UnkArchiveStruct script_slot_archive_context;
static Article* it_804A0F60[30],*script_slot_articles[30];
static unsigned script_slot_native_heap,script_slot_native_min,heap_free=3000000;
static unsigned script_slot_native_cost,script_slot_proc_heap;
static void* script_slot_host_yakumono;
static bool (*script_slot_host_shadow)(Vec3*,int,HSD_JObj*);
static bool grStory_801E36D8(Vec3* p,int i,HSD_JObj* j){(void)p;(void)i;(void)j;return 1;}
static bool grIzumi_801CD280(Vec3* p,int i,HSD_JObj* j){return grStory_801E36D8(p,i,j);}
static void* ys_yaku,*fod_yaku;
static void* grStory_StageSlotParamSwap(void* p){void* old=ys_yaku;ys_yaku=p;return old;}
static void* grIzumi_StageSlotParamSwap(void* p){void* old=fod_yaku;fod_yaku=p;return old;}
static int script_slot_native_initializing,script_slot_controller_next;
static int script_slot_native_ground;
static int particles,items,effects,native_timer;
static unsigned lbHeap_Free(int h){(void)h;return heap_free;}
static void* HSD_ArchiveGetPublicAddress(void* p,const char* s){return strcmp(s,"itemdata")==0?NULL:p;}
static int ScriptGame_StageSlotOwned(HSD_GObj*);
static int ScriptGame_StageSlotNativeContext(void);
static void ScriptGame_StageSlotCreated(HSD_GObj*);
static void ScriptGame_StageSlotDestroyed(HSD_GObj*);
static unsigned script_slot_native_bytes(ScriptStageSlot*);
static int ScriptGame_StageSlotJointUpdate(int source){(void)source;return 1;}
static void HSD_JObjAnimAll(void* j){(void)j;}
static void HSD_GObjProc_RemoveAllProcs(HSD_GObj* g){(void)g;}
static void HSD_GObjGXLink_8039084C(HSD_GObj* g){g->gx_link=255;}
static void script_slot_render(HSD_GObj* g,int p){(void)g;(void)p;}
static void GObj_SetupGXLink(HSD_GObj* g,void(*cb)(HSD_GObj*,int),int a,int b){(void)cb;(void)b;g->gx_link=a;}
static void it_8026B40C(Article* a,int kind){it_804A0F60[kind]=a;}
static void HSD_StageSlotParticlesClear(void){particles=0;}
static void HSD_GObjFree(HSD_GObj* g){ScriptGame_StageSlotDestroyed(g);}
static void Item_8026A8EC(HSD_GObj* g){assert(GET_ITEM(g)->destroy_type==3);--items;HSD_GObjFree(g);}
static void efLib_DestroyAll(HSD_GObj* g){(void)g;--effects;}
static void grStory_801E3030(void)
{
    int i;native_timer=120;
    for(i=0;i<4;++i)assert(st_owned_add(&script_slot_owned,script_slot_native->models[i],1));
}
static void grIzumi_801CBB88(void)
{
    native_timer=120;
    assert(st_owned_add(&script_slot_owned,script_slot_native->models[0],1));
    assert(st_owned_add(&script_slot_owned,script_slot_native->models[1],1));
    assert(st_owned_add(&script_slot_owned,script_slot_native->models[3],1));
    assert(st_owned_add(&script_slot_owned,script_slot_native->controllers[0],1));
    assert(st_owned_add(&script_slot_owned,script_slot_native->controllers[1],1));
}
#include "stage_native_lifecycle_retail.inc"
int main(void)
{
    Article hostarticle={9};HSD_GObj models[6]={0},npc={0},food={0},effect={0};Item ip={0},fp={0};
    ScriptStageSlot slot={0};StageInfo original,active;int visit,i;
    slot.dynamic=3;slot.handle=4;slot.archive=&slot;
    for(i=0;i<4;++i){slot.models[i]=&models[i];models[i].classifier=3;}slot.controllers[0]=&models[4];slot.controllers[1]=&models[5];
    memset(&stage_info,0x59,sizeof stage_info);original=stage_info;
    ys_yaku=fod_yaku=&hostarticle;
    for(i=0;i<30;++i)it_804A0F60[i]=&hostarticle;
    npc=(HSD_GObj){6,6,255,NULL,&ip};food=(HSD_GObj){6,6,255,NULL,&fp};effect=(HSD_GObj){8,8,255};
    for(visit=0;visit<100;++visit){
        slot.dynamic_kind=visit&1?Gr_Kind_Izumi:Gr_Kind_Story;
        script_slot_native_enter(&slot);
        assert(script_slot_owned.count==(slot.dynamic_kind==Gr_Kind_Story?4:5));
        active=original;active.grkind=slot.dynamic_kind;active.on_check_shadow_render=slot.dynamic_kind==Gr_Kind_Story?grStory_801E36D8:grIzumi_801CD280;
        assert(native_timer==120);assert(memcmp(&stage_info,&active,sizeof active)==0);
        assert(ScriptGame_StageSlotOwned(&models[0]));
        assert(ScriptGame_StageSlotProcBegin(&models[0]));
        ScriptGame_StageSlotCreated(&npc);ScriptGame_StageSlotCreated(&food);ScriptGame_StageSlotCreated(&effect);
        items=2;effects=1;particles=7;native_timer=17;heap_free-=300;
        ScriptGame_StageSlotProcEnd();assert(memcmp(&stage_info,&active,sizeof active)==0);
        assert(!ScriptGame_StageSlotParticleBegin(0));
        assert(ScriptGame_StageSlotParticleBegin(3));heap_free-=300;
        assert(!ScriptGame_StageSlotParticleBegin(3)); /* nested scope is borrowed */
        ScriptGame_StageSlotProcEnd();assert(script_slot_native_bytes(&slot)==600);
        /* Natural destruction unregisters a child before switch teardown. */
        fp.destroy_type=3;Item_8026A8EC(&food);
        script_slot_native_leave();
        assert(!script_slot_native && script_slot_owned.count==0 && !items && !effects && !particles);
        assert(memcmp(&stage_info,&original,sizeof original)==0);
        assert(ys_yaku==&hostarticle && fod_yaku==&hostarticle);
        for(i=0;i<30;++i)assert(it_804A0F60[i]==&hostarticle);
    }
    assert(slot.bytes==60000 && heap_free==2940000); /* retained pool cost */
    puts("native enter/proc/teardown: 100 YS/FoD visits, clean timer, owned GObj/item/effect/particle cleanup, StageInfo/articles restored passed");return 0;
}
