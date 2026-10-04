#include <assert.h>
#include <stdio.h>
#include <stdint.h>
#include <string.h>
/* Host fixture pointers need uintptr_t; game-world u32 pointers are 32-bit. */
typedef uintptr_t u32;
typedef struct {int refs;}HSD_JObj;
typedef struct HSD_Generator HSD_Generator;
typedef struct HSD_Particle HSD_Particle;
typedef struct {void(*hookDelete)(HSD_Particle*);}UserFunc;
typedef struct HSD_psAppSRT {HSD_Generator* gp;int usedCount;}HSD_psAppSRT;
struct HSD_Generator {HSD_Generator* next;unsigned type;int numChild,linkNo;float random;int genLife;HSD_JObj* jobj;HSD_psAppSRT* appsrt;UserFunc* userfunc;};
struct HSD_Particle {HSD_Particle* next;HSD_Generator* gen;void* appsrt;};
typedef struct HSD_SList {struct HSD_SList* next;void* data;}HSD_SList;
static u32 hsd_804D78F4;
static HSD_Generator* hsd_804D78FC;
static HSD_Particle* hsd_804D0908[16];
static int hsd_804D78E2,hsd_804D78E0,freed_particles,freed_generators,freed_queue;
static struct {int alloc_data;}hsd_804D0F60,hsd_804D0F90;
static void HSD_ObjFree(void* pool,void* obj){assert(obj);if(pool==&hsd_804D0F60.alloc_data)++freed_particles;else{assert(pool==&hsd_804D0F90.alloc_data);++freed_generators;}}
static HSD_SList* HSD_SListRemove(HSD_SList* p){++freed_queue;return p->next;}
static void HSD_JObjUnref(HSD_JObj* p){assert(p->refs>1);--p->refs;}
static void psRemoveGeneratorSRT(HSD_Generator* p){p->appsrt=NULL;}
static void psRemoveParticleAppSRT(HSD_Particle* p){if(p->gen && p->gen->appsrt)--p->gen->appsrt->usedCount;p->appsrt=NULL;}
static void hsd_8039D048(void* p){(void)p;}
static void hsd_8039D0A0(HSD_Generator* p){assert(p->numChild==0);}
#include "stage_particle_cleanup_retail.inc"
int main(void)
{
    HSD_JObj joint={2};HSD_Generator host={0},owned={0},child={0};HSD_Particle hostp={0},p={0},q={0};HSD_SList hostqueue={0},queue={0};HSD_psAppSRT srt={&owned,2};int visit;
    for(visit=0;visit<100;++visit){
        host.next=&owned;owned.next=&child;child.next=NULL;owned.linkNo=child.linkNo=3;host.linkNo=0;
        owned.type=0x1900;owned.jobj=&joint;owned.appsrt=&srt;owned.numChild=1;child.numChild=1;joint.refs=2;srt.usedCount=2;
        p.gen=&owned;p.appsrt=&srt;p.next=&q;q.gen=&child;q.appsrt=NULL;q.next=NULL;hostp.next=NULL;
        hsd_804D0908[0]=&hostp;hsd_804D0908[3]=&p;hsd_804D78FC=&host;hsd_804D78E2=3;hsd_804D78E0=3;
        hostqueue.data=&host;hostqueue.next=&queue;queue.data=&owned;queue.next=NULL;hsd_804D78F4=(u32)&hostqueue;
        HSD_StageSlotParticlesClear();
        assert(hsd_804D78FC==&host && host.next==NULL && hsd_804D0908[3]==NULL && hsd_804D0908[0]==&hostp);
        assert(hsd_804D78E2==1 && hsd_804D78E0==1 && joint.refs==1 && owned.appsrt==NULL);
        assert((HSD_SList*)hsd_804D78F4==&hostqueue && hostqueue.next==NULL);
        HSD_StageSlotParticlesClear();
    }
    assert(freed_particles==200 && freed_generators==200 && freed_queue==100);
    puts("retail particle/generator cleanup: 100 visits, queued joint refs, child particles/SRT, host link untouched, idempotent passed");return 0;
}
