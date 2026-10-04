#include <assert.h>
#include <stdio.h>
#include <string.h>
#include <limits.h>
#define GS_MAX_SCRIPTS 2
typedef struct {int slot,place,transition,shader,duration,indicator;} GwStageQueueEntry;
typedef struct {void* entries;} GwStageQueue;
static struct {GwStageQueue queue;GwStageQueueEntry request;int pending,old,started,committed,post;double t0;} gs_stage_director[3];
static struct {int match_active,cur,match_frame;struct {int stage_owner;} s[3];} gs;
static struct {int owner,used;} gs_hitstop[3];
static int gs_stage_transition_owner,gs_stage_isolation_owner,gs_stage_handle_serial;
static int gs_stage_session_epoch,gs_presentation_active;
static int cover_live,protected_cover,in_callback,commits,ticks,after,refusals,advanced;
static double now;
static double gs_now_ms(void){return now;}
static int gw_RB_Enabled(void){return 0;}
static int gw_Netplay_Enabled(void){return 0;}
#define gw_log(...) ((void)0)
#define gw_ScriptGame_StageSwitchCheck(...) 0
#define gs_stage_error(rc) "error"
static void gs_stage_slots_release(int s){assert(0);}
static int gw_Post_Add(int o,int s,int order,int stage,int half,int owns,char* e,int cap){return cover_live=1;}
static int gw_Post_StageCover(int o,int flash,char* e,int cap){return cover_live=1;}
static void gw_Post_Protect(int o,int h){protected_cover=1;}
static int gw_Post_Set(int o,int h,const char* p,char* e,int cap){if(!cover_live)++refusals;return cover_live;}
static void gw_presentation_start(void* timer,int o,double t,int duration){gs_hitstop[0].owner=o;gs_hitstop[0].used=1;}
static void gw_stage_advance(GwStageQueue* q,int frame){++advanced;}
static void gs_stage_cancel(int s){cover_live=0;gs_hitstop[0].used=0;gs_stage_director[s].pending=0;gs_stage_transition_owner=0;}
static void gs_rw_branch(void){}
static void gs_rw_stop(void){}
static int music_calls;
static int lbAudioAx_80023F28(int id){assert(!in_callback&&!gs_hitstop[0].used);++music_calls;return 0;}
static int lbAudioAx_StageSlotMusic(void){return 5;}
#define OSReport(...) ((void)0)
#include "stage_music_queue_retail.inc"
static int gw_ScriptGame_StageSwitch(int slot,int owner,int serial,int place) {
    assert(gs_hitstop[0].used&&cover_live);++commits;
    ScriptGame_StageMusicRequest(5);return 0;
}
static void gs_stage_hook(const char* phase,int from,int to) {
    in_callback=1;
    /* Caller clears its own pass; protected director cover remains. */
    if(!protected_cover)cover_live=0;
    if(!strcmp(phase,"after"))++after;
    in_callback=0;
}
#include "stage_transition_presentation_retail.inc"
int main(void) {
    gs.match_active=1;gs.s[0].stage_owner=7;gs_stage_transition_owner=1;
    gs_stage_director[0].pending=1;gs_stage_director[0].queue.entries=(void*)1;
    gs_stage_director[0].request.duration=60;
    for(ticks=0;ticks<=60;++ticks) {
        now=ticks*1000.0/60;
        gs_stage_presentation_tick();
        ScriptGame_StageMusicTick(!gs_hitstop[0].used);
        if(ticks<60)assert(cover_live && !music_calls);
    }
    assert(commits==1&&after==1&&advanced==1&&!refusals&&music_calls==1&&ticks==61);
    puts("production transition director: cover survives hook clear, midpoint commits during freeze, 61 main ticks, thaw music passed");
    return 0;
}
