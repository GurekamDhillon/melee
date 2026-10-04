/* Retail state machine + retail collision matrix update, fake scene/render. */
#include <stdlib.h>
#include <stdint.h>
#include <math.h>
#define main stage_joint_fixture_main
#include "stage_joint_motion_test.c"
#undef main
typedef float f32;
typedef struct {union {struct {int xC4,xC6,xC8;HSD_JObj* xCC;float xD0,xD4,xD8,xDC;} izumi3;} u;}Ground;
typedef struct {void* hsd_obj;void* user_data;}HSD_GObj;
typedef HSD_GObj Ground_GObj;
#define GET_JOBJ(g) ((HSD_JObj*)(g)->hsd_obj)
#define GET_GROUND(g) ((Ground*)(g)->user_data)
#define HSD_RAND_TRACE() ((void)0)
static u32 seed,*HSD_RandSeedPtr=&seed;
static struct grIzumi_YakumonoParam* yakumono_param;
static int rand_range(int a,int b);
static HSD_JObj* HSD_JObjGetChild(HSD_JObj* j){return j->child;}
static float HSD_JObjGetTranslationY(HSD_JObj* j){return j->mtx[1][3];}
static void HSD_JObjSetTranslateY(HSD_JObj* j,float y){j->mtx[1][3]=y;}
static void HSD_JObjSetFlagsAll(HSD_JObj* j,unsigned f){j->flags|=f;}
static void HSD_JObjClearFlagsAll(HSD_JObj* j,unsigned f){j->flags&=~f;}
static void HSD_JObjRemoveAnimAll(HSD_JObj* j){(void)j;}
static void HSD_JObjSetScaleX(HSD_JObj* j,float x){j->mtx[0][0]=x;}
static void HSD_JObjSetScaleY(HSD_JObj* j,float y){j->mtx[1][1]=y;}
static void grAnime_801C7FF8(HSD_GObj* g,int a,int b,int c,float d,float e){(void)g;(void)a;(void)b;(void)c;(void)d;(void)e;}
/* Source-index controller calls are routed through the production adapter. */
static int ScriptGame_StageSlotJointUpdate(int source);
static void controller_joint_update(int source){assert(ScriptGame_StageSlotJointUpdate(source));}
#define mpLib_80055E9C controller_joint_update
#include "stage_fod_controller_retail.inc"
#undef mpLib_80055E9C
static int rand_range(int a,int b){return a>b?b+HSD_Randi(a-b):a<b?a+HSD_Randi(b-a):a;}
typedef struct {int joint;}TestLine;
typedef struct {TestLine lines[1];HSD_JObj* collision_joints[256];int dynamic_kind,njoints;HSD_GObj* controllers[2];}ScriptStageSlot;
static ScriptStageSlot* script_slot_native;
static int script_slot_joint_guard;
static int script_slot_native_initializing,script_slot_native_ground=1;
static struct {int nlines;int lines[1];}script_switch;
static struct {int base_j,base_l;}script_stage;
#define Gr_Kind_Izumi 12
static int line_enabled;
static int ScriptGame_StageSlotNativeContext(void){return 1;}
static int script_stage_seam_slot(int h){return h;}
static void mpLib_80057424(int j){(void)j;}
static void mpLib_8005667C(int j){(void)j;}
static void mpLib_80057528(int l){assert(l==5);line_enabled=1;}
static void mpLib_800575B0(int l){assert(l==5);line_enabled=0;}
static void mpUncheckBounding(void){}
static void lb_8000B1CC(HSD_JObj* j,void* unused,Vec3* p){(void)unused;p->x=j->mtx[0][3];p->y=j->mtx[1][3];p->z=0;}
#include "stage_native_joint_retail.inc"
static HSD_JObj platform,visual,child;
static Ground ground;
static HSD_GObj controller;
static uint32_t run(struct grIzumi_YakumonoParam* p,int source)
{
    unsigned visited=0,hash=2166136261u;int frame;MapJoint map={10,2,-10,10,0,0};ScriptStageSlot slot={0};
    memset(&platform,0,sizeof platform);memset(&visual,0,sizeof visual);memset(&child,0,sizeof child);memset(&ground,0,sizeof ground);
    platform.mtx[0][0]=platform.mtx[1][1]=platform.mtx[2][2]=1;
    visual.child=&child;controller=(HSD_GObj){&visual,&ground};
    slot.dynamic_kind=Gr_Kind_Izumi;slot.njoints=3;slot.controllers[source]=&controller;slot.collision_joints[source]=&platform;slot.lines[0].joint=source;
    script_slot_native=&slot;script_switch.nlines=1;script_switch.lines[0]=5;
    groundCollJoint[5]=(CollJoint){&map,&platform,CollJoint_Enabled};
    groundCollVtx[10].x0=-10;groundCollVtx[11].x0=10;
    ground.u.izumi3.xC8=source;ground.u.izumi3.xCC=&platform;ground.u.izumi3.xD0=ground.u.izumi3.xD4=source?p->x8:p->x0;
    ground.u.izumi3.xDC=p->xC;ground.u.izumi3.xD8=45;
    yakumono_param=p;seed=12345;
    for(frame=0;frame<120000;++frame){
        float before=groundCollVtx[10].pos.y;uint32_t bits;int absent;
        grIzumi_801CC358(&controller);visited|=1u<<ground.u.izumi3.xC4;
        assert(fabsf(groundCollVtx[10].pos.y-platform.mtx[1][3])<0.0001f);
        assert(fabsf(groundCollVtx[10].x14-before)<0.0001f); /* moving-floor carry history */
        absent=ground.u.izumi3.xC4==3 || ground.u.izumi3.xC4==4 || platform.mtx[1][3]<0;
        assert(line_enabled==!absent);
        memcpy(&bits,&ground.u.izumi3.xD0,4);hash=(hash^bits^seed)*16777619u;
    }
    assert(visited==31);assert(script_slot_joint_guard==0);
    script_slot_native_ground=0;assert(!ScriptGame_StageSlotJointUpdate(source)); /* item/particle context is not a source joint call */
    script_slot_native_ground=1;assert(!ScriptGame_StageSlotJointUpdate(5)); /* installed backend id passes through */
    return hash;
}
int main(int argc,char** argv)
{
    struct grIzumi_YakumonoParam p;float values[21];unsigned a,b,c,d;int i;
    assert(argc==22);for(i=0;i<21;++i)values[i]=(float)strtod(argv[i+1],NULL);memcpy(&p,values,sizeof p);
    a=run(&p,0);b=run(&p,0);c=run(&p,1);d=run(&p,1);assert(a==b && c==d && a!=c);
    printf("FoD retail RNG/controller/remapping: 480000 frames, both source joints/all 5 states, joint/carry/absent collision, repeat seed hashes=%u/%u passed\n",a,c);return 0;
}
