/* Game fakes for production script_stage_slots.inc, deliberately named fields
 * mirror the real structures so changes to clearing/placement are exercised. */
#include "../gameworld/script_model.h"
static int script_stage_same_file(const char* a,const char* b){return strcmp(a,b)==0;}
typedef unsigned char u8;
typedef unsigned u32;
typedef short s16;
typedef int bool;
typedef float Mtx[3][4];
typedef struct {float x,y,z;} Vec3;
typedef struct {float x,y;} Vec2;
typedef struct {float left,right,top,bottom;} StageBlastZone;
typedef struct {StageBlastZone cam_bounds;float cam_x_offset,cam_y_offset;} StageCameraInfo;
typedef struct {int x4;} StageParam;
typedef struct {float y,x8,x14,x1C,x18,x10,xC,x28,x2E,x30,x34,x38,x3C,x40,x44,x48,x50,x54,x58,x5C,x60,x64,x20,x24;int stage_param_count;StageParam* stage_params;} GroundParam;
typedef struct HSD_Joint {unsigned flags;struct HSD_Joint* child,*next;Vec3 scale,position;} HSD_Joint;
typedef struct HSD_JObj {Vec3 p;} HSD_JObj;
typedef struct HSD_GObj {void* hsd_obj;void* user_data;struct HSD_GObj* next;} HSD_GObj;
typedef struct {struct {unsigned data_size;} header;u8* data;} HSD_Archive;
typedef struct {HSD_Joint* unk0;} Group;
typedef struct {int unkC,unk4;Group* unk8;void* unk0;} UnkStageDat;
typedef struct {int index;unsigned flags;Vec3 normal;} Surface;
typedef struct {
    int floor_skip,ledge_id_left,ledge_id_right,joint_id_skip,joint_id_only;
    Surface floor,ceiling,left_facing_wall,right_facing_wall;
    int env_flags,prev_env_flags,x13C,x130_flags;
    Vec3 contact,x28_vec,cur_pos,prev_pos,last_pos;
    int prev_ecb,xE4_ecb,desired_ecb,ecb;
} CollData;
typedef struct {
    struct CpuFighter {int kind,level,output;} cpu;
    HSD_GObj* gobj;void* victim_gobj;int x1A5C,motion_id,ground_or_air;
    CollData coll_data;Vec3 cur_pos,prev_pos;
    struct {struct {struct {int ledge_id;} cliff;} co;} mv;
} Fighter;
typedef struct {int kind;CollData x378_itemColl;Vec3 pos;} Item;
typedef struct {const char* data1;} StageData;
typedef struct {short prev_id0,next_id0,prev_id1,next_id1;unsigned lo_flags;} MapLine;
typedef struct {MapLine* lines;} MapCollData;
enum {ftCo_MS_RebirthWait=10,ftCo_MS_CapturePulledHi=100,ftCo_MS_CaptureFoot=110,
      ftCo_MS_ThrownF=111,ftCo_MS_ThrownlwWomen=115,ftCo_MS_CliffCatch=120,
      ftCo_MS_CliffWait=121,ftCo_MS_CliffJumpQuick2=132,GA_Air=1,GA_Ground=0,
      Gr_Kind_Count=221,HSD_GOBJ_PLINK_FIGHTER=0,HSD_GOBJ_PLINK_ITEM=1,
      HSD_GOBJ_CLASS_STAGE=3,It_Kind_Old_Kuri=100,JOBJ_INSTANCE=4096,JOBJ_JOINT1=1};
#define LINE_FLAG_PLATFORM 256
#define LINE_FLAG_LEDGE 512
#define Collide_LedgeGrabMask 0x3000000
#define SCRIPT_STAGE_LINES 768
#define OSReport(...) ((void)0)
#define GET_FIGHTER(g) ((Fighter*)(g)->user_data)
#define GET_ITEM(g) ((Item*)(g)->user_data)
static struct {StageCameraInfo cam_info;StageBlastZone blast_zone;GroundParam* param;HSD_JObj* x280[261];} stage_info;
static MapLine fixture_maplines[1024];
static MapCollData fixture_map={fixture_maplines};
static struct {int cap,base_l;MapCollData* map;struct {int active,handle;float x0,y0,x1,y1;} line[768];ScriptMeshInstance instance[256];} script_stage;
static struct {int owner;} script_camera_params;
static HSD_GObj* fixture_heads[2];static HSD_GObj** HSD_GObjPLinkHead=fixture_heads;
static Fighter fixture_fighter;static HSD_GObj fixture_gobj;
static HSD_GObj fixture_itemobjs[2];static Item fixture_items[2];
static GroundParam fixture_param;
static StageData* stage_datas[221];
#include "../../src/melee/gr/ground_stage_slots.inc"
static int fixture_online,fixture_isolated,fixture_allocs,fixture_frees;
static int fixture_safety_active,fixture_stage_item_deleted,fixture_portable_deleted;
static int fixture_model_frees;
static int HSD_GObj_JObjKind;
static int script_mesh_bits(float x){union{float f;int i;}v;v.f=x;return v.i;}
static float script_mesh_float(int x){union{float f;int i;}v;v.i=x;return v.f;}
static unsigned lbHeap_Free(int x){(void)x;return 10000000;}
static unsigned lbHeap_Capacity(int x){(void)x;return 11000000;}
static void* lbHeap_StageSlotTryAlloc(unsigned n){++fixture_allocs;return calloc(1,n);}
static void lbHeap_80015CA8(int heap,void* p){(void)heap;++fixture_frees;free(p);}
static int Netplay_Enabled(void){return fixture_online;}
static int RB_Enabled(void){return 0;}
static int Replay_Active(void){return 0;}
static int script_stage_host_allowed(void){return 1;}
static MapCollData* mpLib_8004D164(void){return script_stage.map;}
static int Script_StageArchiveInput(int a,int b){(void)a;(void)b;return 0;}
static int Script_StageSlotMissionInput(int a,int b,int c){(void)a;(void)b;(void)c;return 0;}
static int script_stage_text(int a,char* p,int cap){(void)a;snprintf(p,cap,"fixture.dat");return 1;}
static int DVDConvertPathToEntrynum(const char* p){(void)p;return -1;}
static unsigned lbFile_8001634C(int i){(void)i;return 0;}
static void lbFile_8001668C(const char* p,void* data,size_t* bytes){(void)p;(void)data;*bytes=0;}
static int HSD_ArchiveParse(HSD_Archive* a,void* p,unsigned n){(void)a;(void)p;(void)n;return -1;}
static const char* HSD_ArchiveGetExtern(HSD_Archive* a,int i){(void)a;(void)i;return NULL;}
static void HSD_ArchiveLocateExtern(HSD_Archive* a,const char* p,void* v){(void)a;(void)p;(void)v;}
static void* HSD_ArchiveGetPublicAddress(HSD_Archive* a,const char* p){(void)a;(void)p;return NULL;}
static int script_stage_has_instance(HSD_Joint* j){(void)j;return 0;}
static HSD_JObj* HSD_JObjLoadJoint(HSD_Joint* j){HSD_JObj* p=calloc(1,sizeof *p);p->p=j->position;return p;}
static void HSD_JObjUnref(HSD_JObj* p){free(p);}
static HSD_JObj* HSD_JObjGetChild(HSD_JObj* p){(void)p;return NULL;}
static HSD_JObj* HSD_JObjGetNext(HSD_JObj* p){(void)p;return NULL;}
static void HSD_JObjGetTranslation(HSD_JObj* p,Vec3* v){*v=p->p;}
static void HSD_JObjSetTranslate(HSD_JObj* p,const Vec3* v){p->p=*v;}
static HSD_GObj* GObj_Create(int a,int b,int c){(void)a;(void)b;(void)c;return calloc(1,sizeof(HSD_GObj));}
static void HSD_GObjFree(HSD_GObj* g){++fixture_model_frees;free(g->hsd_obj);free(g);}
static void HSD_GObjObject_80390A70(HSD_GObj* g,int kind,HSD_JObj* p){(void)kind;g->hsd_obj=p;}
static void GObj_SetupGXLink(HSD_GObj* g,void(*f)(HSD_GObj*,int),int a,int b){(void)g;(void)f;(void)a;(void)b;}
static void HSD_GObj_JObjCallback(HSD_GObj* g,int pass){(void)g;(void)pass;}
static int gx_suppress_draws;
static int Script_StageSlotVisual(int h){(void)h;return 0x7fc00000;}
static void lb_8000B1CC(HSD_JObj* j,void* a,Vec3* p){(void)a;*p=j->p;}
static void Ground_801C39C0(void){}
static void Ground_801C3BB4(void){}
static void Ground_801C38D0(float a,float b,float c,float d){(void)a;(void)b;(void)c;(void)d;}
static void Ground_801C38EC(float a,float b){(void)a;(void)b;}
static void Ground_801C3970(float a){(void)a;}
static void Ground_801C3900(float a,float b,float c,float d,float e,float f,float g,float h){(void)a;(void)b;(void)c;(void)d;(void)e;(void)f;(void)g;(void)h;}
static void Ground_801C392C(float a,float b,float c,float d,float e,float f){(void)a;(void)b;(void)c;(void)d;(void)e;(void)f;}
static void Ground_801C3960(float a){(void)a;}
static void Ground_801C3950(float a){(void)a;}
static int lbAudioAx_StageSlotMusic(void){return -1;}
static int lbAudioAx_80023F28(int x){(void)x;return 0;}
static int ScriptGame_ModelSlot(int h){int i;for(i=0;i<256;++i)if(script_stage.instance[i].handle==h)return i;return -1;}
static void ScriptGame_ModelDespawn(int h){int i=ScriptGame_ModelSlot(h);if(i>=0)memset(&script_stage.instance[i],0,sizeof script_stage.instance[i]);}
static void ftCommon_UnlockECB(Fighter* f){(void)f;}
static void mpColl_80043680(CollData* c,Vec3* p){c->cur_pos=c->prev_pos=c->last_pos=*p;c->x130_flags|=32;}
static int fixture_ledge_line=-1,fixture_ledge_releases;
static void ftCo_Fall_Enter(HSD_GObj* g){Fighter* f=GET_FIGHTER(g);
    if(f->motion_id>=ftCo_MS_CliffCatch && f->motion_id<=ftCo_MS_CliffJumpQuick2 && fixture_ledge_line>=0){
        assert(script_stage.line[fixture_ledge_line].active);++fixture_ledge_releases;
    }
    f->ground_or_air=GA_Air;f->motion_id=20;f->mv.co.cliff.ledge_id=77;/* models Fall union initialization */f->cpu.kind=4;f->cpu.output=123;}
static void ftCamera_80076064(Fighter* f){(void)f;}
static int ScriptGame_ArenaOwner(void){return 0;}
static int ScriptGame_StageIsolate(int owner,int on){(void)owner;fixture_isolated=on;return 1;}
static void ScriptGame_StageIsolationClear(int owner){(void)owner;fixture_isolated=0;}
static int ScriptGame_StageAddLine(int x,int y,int X,int Y,int kind,int flags,int h)
{
    int i;(void)x;(void)X;(void)Y;(void)kind;(void)flags;
    for(i=0;i<script_stage.cap;++i)if(!script_stage.line[i].active) {
        script_stage.line[i].active=1;script_stage.line[i].handle=h;
        {MapLine* l=&script_stage.map->lines[script_stage.base_l+i];l->prev_id0=l->next_id0=l->prev_id1=l->next_id1=-1;}
        script_stage.line[i].x0=script_mesh_float(x);script_stage.line[i].y0=script_mesh_float(y);
        script_stage.line[i].x1=script_mesh_float(X);script_stage.line[i].y1=script_mesh_float(Y);
        if(script_mesh_float(y)==-49000)fixture_safety_active=1;
        return h;
    }
    return -1;
}
static int script_stage_seam_slot(int h){int i;for(i=0;i<script_stage.cap;++i)if(script_stage.line[i].active && script_stage.line[i].handle==h)return i;return -1;}
static int ScriptGame_StageRemove(int h){int at=script_stage_seam_slot(h);if(at<0)return 0;
    if(fixture_ledge_line>=0){int i;assert(fixture_fighter.motion_id<ftCo_MS_CliffCatch || fixture_fighter.motion_id>ftCo_MS_CliffJumpQuick2);assert(fixture_fighter.coll_data.ledge_id_left==-1);
        for(i=0;i<script_stage.cap;++i)if(script_stage.line[i].active){MapLine* l=&script_stage.map->lines[script_stage.base_l+i];assert(l->prev_id0==-1 && l->next_id0==-1 && l->prev_id1==-1 && l->next_id1==-1);}}
    script_stage.line[at].active=0;if(at==0)fixture_safety_active=0;return 1;}
static void script_stage_seam_refresh(void){}
static int mpCheckFloor(float x,float y,float X,float Y,int a,Vec3* p,int* line,u32* flags,Vec3* n,int b,int c,int d,void* e,void* f)
{
    (void)X;(void)a;(void)b;(void)c;(void)d;(void)e;(void)f;
    if(x< -10 || x>10 || y<0 || Y>0)return 0;
    *p=(Vec3){x,0,0};*line=10;*flags=1;*n=(Vec3){0,1,0};return 1;
}
static void Item_8026A8EC(HSD_GObj* g){if(GET_ITEM(g)->kind>=100)fixture_stage_item_deleted=1;else fixture_portable_deleted=1;fixture_heads[1]=g->next;}
static void fixture_dirty_contacts(Fighter* f)
{
    CollData* c=&f->coll_data;memset(c,0,sizeof *c);
    c->floor_skip=c->ledge_id_left=c->ledge_id_right=99;
    c->ceiling.index=c->left_facing_wall.index=c->right_facing_wall.index=98;
    c->env_flags=c->prev_env_flags=123;c->prev_pos=(Vec3){999,999,999};
}
static void fixture_init(void)
{
    memset(&script_stage,0,sizeof script_stage);memset(&fixture_fighter,0,sizeof fixture_fighter);
    memset(&stage_info,0,sizeof stage_info);memset(fixture_maplines,0,sizeof fixture_maplines);
    script_stage.cap=8;script_stage.map=&fixture_map;script_stage.base_l=10;
    stage_info.param=&fixture_param;stage_info.blast_zone=(StageBlastZone){-100,100,100,-100};
    fixture_param.y=1;
    fixture_gobj.user_data=&fixture_fighter;fixture_gobj.next=NULL;fixture_fighter.gobj=&fixture_gobj;
    fixture_fighter.motion_id=20;fixture_fighter.ground_or_air=GA_Ground;fixture_heads[0]=&fixture_gobj;
    fixture_items[0].kind=100;fixture_items[1].kind=1;
    fixture_itemobjs[0].user_data=&fixture_items[0];fixture_itemobjs[0].next=&fixture_itemobjs[1];
    fixture_itemobjs[1].user_data=&fixture_items[1];fixture_itemobjs[1].next=NULL;fixture_heads[1]=&fixture_itemobjs[0];
    fixture_online=fixture_isolated=fixture_safety_active=fixture_stage_item_deleted=fixture_portable_deleted=0;
}
