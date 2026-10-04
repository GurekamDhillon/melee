#include <assert.h>
#include <stdio.h>
#include <stdint.h>
#include <string.h>
typedef uint32_t u32;typedef int32_t s32;typedef float f32;
typedef struct {float x,y,z;}Vec3;
typedef struct {int value;}HSD_JObj;
typedef struct {union {struct {int timer,pattern,count;}shyguys;struct {int timer;HSD_JObj* jobj;}randall;}u;}Ground;
typedef struct {Ground* user_data;}Ground_GObj;
#define GET_GROUND(g) ((g)->user_data)
#define HSD_RAND_TRACE() ((void)0)
#define PAD_STACK(n)
#define ARRAY_SIZE(a) (sizeof(a)/sizeof((a)[0]))
#define It_Kind_Heiho 7
static u32 seed,*HSD_RandSeedPtr=&seed;
static int alive,spawned,puffs;
static Vec3 pos_record[100];static int variant_record[100],delay_record[100];
static int it_8026B3C0(int kind){assert(kind==It_Kind_Heiho);return alive;}
static void it_802D8618(int i,Vec3* p,int variant,int delay){assert(i>=0 && i<5 && spawned<100);pos_record[spawned]=*p;variant_record[spawned]=variant;delay_record[spawned++]=delay;++alive;}
static void grLib_801C97DC(int effect,int bank,HSD_JObj* j){assert(effect==44 && bank==0 && j);++puffs;}
#include "stage_ys_controller_retail.inc"
int main(void)
{
    struct grStory_YakumonoParam p={600,1800,8,{30,45,60,75,90,0}};
    Ground shy={0},cloud={0};Ground_GObj s={&shy},r={&cloud};HSD_JObj joint={0};u32 seeds[2];Vec3 saved[100];int visit,frame,previous=-1;
    yakumono_param=&p;
    for(visit=0;visit<2;++visit){
        memset(&shy,0,sizeof shy);memset(&cloud,0,sizeof cloud);seed=12345;alive=spawned=puffs=0;
        reset_shyguy_timer(&shy);assert(shy.u.shyguys.timer==120 && seed!=12345);
        cloud.u.randall.jobj=&joint;
        grStory_801E366C(&r);assert(puffs==0 && cloud.u.randall.timer==-1);
        grStory_801E366C(&r);assert(puffs==1 && cloud.u.randall.timer>=10 && cloud.u.randall.timer<=29);
        for(frame=0;frame<3000;++frame){
            int old=spawned;grStory_801E3418(&s);grStory_801E366C(&r);
            if(spawned!=old){int i;assert(shy.u.shyguys.pattern!=previous);previous=shy.u.shyguys.pattern;
              assert(spawned-old==1 || (spawned-old>=3 && spawned-old<=5));
              for(i=old;i<spawned;++i){assert(variant_record[i]>=0 && variant_record[i]<3);assert(delay_record[i]==25*(i-old));assert(pos_record[i].x==-292 || pos_record[i].x==304);assert(pos_record[i].z==2);}
              /* Retail timer pauses while any Shy Guy remains. */
              for(i=0;i<20;++i)grStory_801E3418(&s);assert(shy.u.shyguys.timer==120);alive=0;
            }
        }
        seeds[visit]=seed;if(!visit)memcpy(saved,pos_record,sizeof saved);else assert(!memcmp(saved,pos_record,sizeof saved));previous=-1;
    }
    assert(seeds[0]==seeds[1] && puffs>0 && spawned>0);
    puts("YS retail RNG: overwritten timer/count draws retained, Shy Guy positions/variants/stagger/alive pause, Randall puff timing, repeat seed passed");return 0;
}
