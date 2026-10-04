/* Synthetic descriptors only. Execute extracted retail constructor, scaler,
 * draw binder, FoD on_load and actual switch clear/rebuild helpers. */
#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
typedef float f32;
typedef struct { float x,y,z; } Vec3;
typedef struct HSD_LObj {
    struct HSD_LObj* next;
    Vec3 position,interest;
    unsigned flags,color,attenuation,animation;
    unsigned priority,hw_color,id,spec_id;
    float shininess,attn[6];
    Vec3 lvec;
    unsigned lightobj[16],spec_lightobj[16];
    float frame,rate;
    int loop;
} HSD_LObj;
typedef struct HSD_GObj {
    int classifier,obj_kind,p_link,gx_link,priority;
    struct HSD_GObj* next;
    void* hsd_obj;
    void (*render)(struct HSD_GObj*,int);
    void (*process)(struct HSD_GObj*);
} HSD_GObj;
typedef HSD_GObj Fighter_GObj;
#define HSD_GOBJ_PLINK_LIGHT 3
#define LOBJ_TYPE 1
#define ALL_TYPE_MASK 0
#define AOBJ_ARG_AU 0
#define AOBJ_LOOP 1
#define GET_LOBJ(g) ((HSD_LObj*)(g)->hsd_obj)
#define OSReport(...) ((void)0)
static HSD_GObj* heads[12];
static HSD_GObj** HSD_GObjPLinkHead=heads;
static int HSD_GObj_LightKind=7;
static HSD_LObj* lobj0;
static HSD_LObj* lobj1;
static HSD_LObj* current;
static HSD_LObj desc[2];
static float scale;
static int fog,invalidated,capture;
typedef struct { unsigned char r,g,b,a; } GXColor;
typedef struct { unsigned type; float start,end; GXColor color; unsigned adj[10]; } HSD_Fog;
typedef HSD_Fog HSD_FogDesc;
typedef struct { float y; } GroundParam;
typedef struct { GroundParam* param; HSD_GObj* x12C; } StageInfo;
static GroundParam params;
static StageInfo stage_info;
static HSD_FogDesc fog_desc;
static HSD_FogDesc* selected_fog;
static GXColor background;
static int HSD_GObj_FogKind=9;
static HSD_FogDesc* foo(void){return selected_fog;}
static HSD_Fog* HSD_FogLoadDesc(HSD_FogDesc* d){HSD_Fog* f=malloc(sizeof *f);*f=*d;return f;}
static void Camera_SetBackgroundColor(unsigned char r,unsigned char g,unsigned char b){background=(GXColor){r,g,b,255};}
static void Ground_801C1E2C(HSD_GObj* g,int code){(void)g;(void)code;}
static float Ground_801C0498(void){return scale;}
static HSD_LObj* Ground_801C49B4(void){return desc;}
static HSD_GObj* GObj_Create(int c,int p,int priority) {
    HSD_GObj* g=calloc(1,sizeof *g);g->classifier=c;g->p_link=p;
    g->priority=priority;g->next=heads[p];heads[p]=g;return g;
}
static HSD_LObj* lb_80011AC4(HSD_LObj* d) {
    HSD_LObj* a=malloc(sizeof *a);HSD_LObj* b=malloc(sizeof *b);
    *a=d[0];*b=d[1];a->next=b;b->next=NULL;return a;
}
static HSD_LObj* lb_8000CDC0(HSD_LObj* a){return a->next;}
static int HSD_LObjGetPosition(HSD_LObj* a,Vec3* v){*v=a->position;return 1;}
static int HSD_LObjGetInterest(HSD_LObj* a,Vec3* v){*v=a->interest;return 1;}
static void HSD_LObjSetPosition(HSD_LObj* a,Vec3* v){a->position=*v;}
static void HSD_LObjSetInterest(HSD_LObj* a,Vec3* v){a->interest=*v;}
static void HSD_GObjObject_80390A70(HSD_GObj* g,int k,void* a){g->obj_kind=k;g->hsd_obj=a;}
static void GObj_SetupGXLink(HSD_GObj* g,void (*cb)(HSD_GObj*,int),unsigned l,unsigned p){g->render=cb;g->gx_link=l;g->priority=p;}
static void HSD_LObjReqAnimAll(HSD_LObj* a,float f){while(a){a->frame=f;a=a->next;}}
static void HSD_GObj_SetupProc(HSD_GObj* g,void (*cb)(HSD_GObj*),int p){assert(p==1);g->process=cb;}
static void HSD_LObjAnimAll(void* a){(void)a;}
static void HSD_LObj_803668EC(void* a){current=a;}
static void* HSD_CObjGetCurrent(void){return &scale;}
static void HSD_LObjSetupInit(void* c){assert(c==&scale);}
static int HSD_GObjGetClassifier(HSD_GObj* g){return g->classifier;}
static HSD_GObj* HSD_GObjGetNext(HSD_GObj* g){return g->next;}
static HSD_LObj* HSD_LObjGetNext(HSD_LObj* a){return a->next;}
static void HSD_AObjSetFlags(void){ }
static void HSD_ForeachAnim(HSD_LObj* a,int t,int m,void (*f)(void),int arg,int flags){assert(flags==AOBJ_LOOP);a->loop=1;}
static void HSD_LObjDeleteCurrentAll(void* a){assert(a==NULL);current=NULL;}
static void HSD_FogSet(void* f){assert(f==NULL);fog=0;}
static void HSD_StateInvalidate(int mask){assert(mask==-1);++invalidated;}
static void HSD_GObjFree(HSD_GObj* g) {
    HSD_GObj** at=&heads[g->p_link];while(*at!=g)at=&(*at)->next;
    *at=g->next;
    if(g->classifier==12){assert(!current);free(GET_LOBJ(g)->next);free(g->hsd_obj);}
    if(g->classifier==10)free(g->hsd_obj);
    free(g);
}
void ScriptGame_StageSlotRetailCaptureResume(void){capture=1;}
void ScriptGame_StageSlotRetailCaptureEnd(void){capture=0;}
static void ftCo_8009F54C(HSD_GObj*,int);
#include "stage_lighting_extracted.inc"
static void destination(int stage) {
    int i,k;memset(desc,0,sizeof desc);scale=1.0f+stage*.1f;
    params.y=scale;stage_info.param=&params;stage_info.x12C=NULL;
    memset(&fog_desc,0,sizeof fog_desc);fog_desc.type=stage+1;
    fog_desc.start=10.f+stage;fog_desc.end=100.f+stage;
    fog_desc.color=(GXColor){(unsigned char)(stage+20),30,40,255};
    for(k=0;k<10;++k)fog_desc.adj[k]=stage*10+k;
    selected_fog=stage==0?NULL:&fog_desc;
    for(i=0;i<2;++i) {
        desc[i].position=(Vec3){1.f+stage,2.f+i,3.f};
        desc[i].interest=(Vec3){4.f,5.f+stage,6.f+i};
        desc[i].flags=4+i+stage*8;desc[i].color=0x10203040+stage*17+i;
        desc[i].attenuation=100+stage*3+i;desc[i].animation=200+stage*2+i;
        desc[i].frame=13;desc[i].rate=1.f+stage*.05f;
        desc[i].priority=stage+i;desc[i].shininess=20.f+stage+i;
        desc[i].hw_color=stage*30+i;desc[i].id=1u<<i;desc[i].spec_id=4u<<i;
        desc[i].lvec=(Vec3){1.f,stage+2.f,3.f};
        for(k=0;k<6;++k)desc[i].attn[k]=stage+k*.1f;
        for(k=0;k<16;++k){desc[i].lightobj[k]=stage*50+k;desc[i].spec_lightobj[k]=stage*60+k;}
    }
}
static void same_light(HSD_LObj* a,HSD_LObj* b) {
    assert(a->flags==b->flags && a->color==b->color);
    assert(a->attenuation==b->attenuation && a->animation==b->animation);
    assert(a->frame==b->frame && a->rate==b->rate && a->loop==b->loop);
    assert(a->priority==b->priority && a->shininess==b->shininess);
    assert(a->hw_color==b->hw_color && a->id==b->id && a->spec_id==b->spec_id);
    assert(!memcmp(a->attn,b->attn,sizeof a->attn) && !memcmp(&a->lvec,&b->lvec,sizeof a->lvec));
    assert(!memcmp(a->lightobj,b->lightobj,sizeof a->lightobj));
    assert(!memcmp(a->spec_lightobj,b->spec_lightobj,sizeof a->spec_lightobj));
    assert(a->position.x==b->position.x && a->position.y==b->position.y && a->position.z==b->position.z);
    assert(a->interest.x==b->interest.x && a->interest.y==b->interest.y && a->interest.z==b->interest.z);
}
int main(void) {
    int src,dst;HSD_LObj expected[2];HSD_Fog expected_fog;GXColor expected_background;
    /* FD/BF/YS/DL/FoD use distinct synthetic values; FoD uses its actual
     * production on_load to enforce its extra looping behavior. */
    for(src=0;src<5;++src)for(dst=0;dst<5;++dst) {
        HSD_GObj* fighter;HSD_GObj* stage_light;HSD_GObj* unrelated;
        destination(dst);Ground_801C1E94();ftCo_8009F4A4();if(dst==4)grIzumi_OnLoad();
        expected[0]=*lobj0;expected[1]=*lobj1;
        expected_background=background;
        if(stage_info.x12C){expected_fog=*(HSD_Fog*)stage_info.x12C->hsd_obj;HSD_GObjFree(stage_info.x12C);}
        Ground_StageSlotFighterLightingClear();
        destination(src);ftCo_8009F4A4();if(src==4)grIzumi_OnLoad();
        lobj0->color=0xBAD; lobj1->color=0xBAD; /* outgoing animation/tint */
        stage_light=GObj_Create(13,3,0);unrelated=GObj_Create(11,3,0);
        current=lobj0;fog=1;invalidated=0;
        Ground_StageSlotFighterLightingClear();
        assert(!current && !fog && invalidated==1 && heads[3]==unrelated);
        assert(unrelated->next==stage_light && !stage_light->next);
        destination(dst);Ground_801C1E94();Ground_StageSlotFighterLightingRebuild();
        if(dst==4)grIzumi_OnLoad();
        fighter=heads[3];assert(fighter->classifier==12 && fighter->gx_link==4);
        assert(fighter->obj_kind==HSD_GObj_LightKind && fighter->priority==0);
        assert(fighter->render==ftCo_8009F54C && fighter->process==ftCo_8009F480);
        assert(fighter->hsd_obj==lobj0 && lobj0->next==lobj1 && !lobj1->next);
        same_light(lobj0,&expected[0]);same_light(lobj1,&expected[1]);
        assert(!memcmp(&background,&expected_background,sizeof background));
        if(dst==0)assert(!stage_info.x12C);
        else {
            HSD_GObj* f=stage_info.x12C;
            assert(f && f->classifier==10 && f->obj_kind==HSD_GObj_FogKind && f->gx_link==0);
            assert(!memcmp(f->hsd_obj,&expected_fog,sizeof expected_fog));HSD_GObjFree(f);
        }
        assert(!capture && invalidated==2);
        fighter->render(fighter,0);assert(current==lobj0);
        Ground_StageSlotFighterLightingClear();HSD_GObjFree(unrelated);HSD_GObjFree(stage_light);
    }
    puts("fighter lighting: all fields equal for 25 stage pairs; FoD loop and cache/binder pass");
    return 0;
}
