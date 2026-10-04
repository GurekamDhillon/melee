#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include "../platform/gw_stage_queue.h"
int main(void)
{
    GwStageQueueEntry e[3]={{0}},f[3]={{0}};
    GwStageQueue a={0},b={0}; size_t i;
    for(i=0;i<3;++i) { e[i].slot=i+1; e[i].after=e[i].stocks=-1; }
    e[0].after=60; e[1].stocks=3; strcpy(e[2].event,"on_hit");
    a.entries=e; a.count=3; a.start=10;
    assert(!gw_stage_due(&a,69,0)); assert(gw_stage_due(&a,70,5));
    gw_stage_advance(&a,70); assert(!gw_stage_due(&a,300,4)); assert(gw_stage_due(&a,71,3));
    gw_stage_advance(&a,71); gw_stage_signal(&a,"on_land"); assert(!gw_stage_due(&a,1000,0));
    gw_stage_signal(&a,"on_hit"); assert(gw_stage_due(&a,72,6));
    gw_stage_advance(&a,72); assert(!gw_stage_due(&a,1000,0));
    a.next=0; a.loop=1; a.manual=1; assert(gw_stage_due(&a,72,8));
    gw_stage_advance(&a,72); assert(!a.manual && !a.signalled);
    a.next=2; gw_stage_advance(&a,100); assert(a.next==0 && a.start==100);
    memcpy(f,e,sizeof e); a.seed=b.seed=123; a.shuffle=b.shuffle=1;
    b.entries=f; b.count=3;
    for(i=0;i<50;++i) {gw_stage_shuffle(&a);gw_stage_shuffle(&b);assert(!memcmp(e,f,sizeof e));}
    /* Arbitrary length is storage-limited, not a fixed scene or slot count. */
    a.entries=calloc(10001,sizeof *a.entries); a.count=10001; a.next=10000; a.loop=0;
    a.entries[10000].after=0; assert(gw_stage_due(&a,100,0));
    gw_stage_advance(&a,100); assert(!gw_stage_due(&a,100,0)); free(a.entries);
    puts("stage queue: timer/stocks/event/manual, deterministic shuffle, loop and 10001 entries passed");
    return 0;
}
