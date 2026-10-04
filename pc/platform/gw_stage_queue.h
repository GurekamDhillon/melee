/* Pure queue scheduler: no OS, Lua or game writes. Caller owns arbitrary-length
 * validated entry storage. Timers count completed logic frames at 60 Hz. */
#ifndef GW_STAGE_QUEUE_H
#define GW_STAGE_QUEUE_H
#include <stddef.h>
#include <string.h>
typedef struct {
    int slot,after,stocks,transition,place,indicator,duration,shader;
    char event[64];
} GwStageQueueEntry;
typedef struct {
    GwStageQueueEntry* entries;
    size_t count,next;
    unsigned seed;
    int loop,shuffle,start,manual,signalled;
} GwStageQueue;
static unsigned gw_stage_random(GwStageQueue* q)
{
    unsigned x=q->seed ? q->seed : 0x6d2b79f5u;
    x^=x<<13; x^=x>>17; x^=x<<5; return q->seed=x;
}
static void gw_stage_shuffle(GwStageQueue* q)
{
    size_t i;
    if(!q->shuffle) return;
    for(i=q->count;i>1;--i) {
        size_t j=gw_stage_random(q)%i;
        GwStageQueueEntry e=q->entries[i-1]; q->entries[i-1]=q->entries[j]; q->entries[j]=e;
    }
}
static int gw_stage_due(const GwStageQueue* q,int frame,int stocks)
{
    const GwStageQueueEntry* e;
    if(!q->entries || q->next>=q->count) return 0;
    e=&q->entries[q->next];
    return q->manual || q->signalled || (e->after>=0 && frame-q->start>=e->after) ||
           (e->stocks>=0 && stocks<=e->stocks);
}
static void gw_stage_signal(GwStageQueue* q,const char* event)
{
    if(q->entries && q->next<q->count && q->entries[q->next].event[0] &&
       !strcmp(q->entries[q->next].event,event)) q->signalled=1;
}
static void gw_stage_advance(GwStageQueue* q,int frame)
{
    q->manual=q->signalled=0; q->start=frame;
    if(q->next<q->count) ++q->next;
    if(q->next==q->count && q->loop && q->count) {q->next=0; gw_stage_shuffle(q);}
}
#endif
