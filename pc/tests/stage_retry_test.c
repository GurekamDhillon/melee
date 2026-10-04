#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <limits.h>
#include "../platform/gw_stage_queue.h"
#define GS_MAX_SCRIPTS 1
#define SI_PRESENT 0
#define SI_STOCKS 1
static struct { GwStageQueue queue; GwStageQueueEntry request; int pending,old,started,committed,post; double t0; } gs_stage_director[2];
static struct {int n,cur,match_frame,match_active; struct {int used,disabled,stage_owner;} s[2];} gs;
static struct {int owner,used;} gs_hitstop[2];
static int gs_stage_directors,gs_stage_transition_owner,gs_stage_isolation_owner,gs_stage_handle_serial,gs_stage_session_epoch,gs_presentation_active;
static int refusal,commit_refusal,commits,after;
static double now;
#define gw_log(...) ((void)0)
static const char* gs_stage_error(int rc){return "refusal";}
static const char* gs_script_id(int s){return "fixture";}
static int gw_ScriptGame_FighterI(int s,int field){return 0;}
static int gw_ScriptGame_StageSwitchCheck(int slot,int owner,int place){return refusal;}
static int gw_ScriptGame_StageSlotAt(int owner,int i){return i==0?1:0;}
static int gw_ScriptGame_StageSlotRead(int h,int owner,int field,int i){return 1;}
static int gw_RB_Enabled(void){return 0;}
static int gw_Netplay_Enabled(void){return 0;}
static double gs_now_ms(void){return now;}
static void gw_Post_Remove(int owner,int post){}
static int gw_Post_Add(int a,int b,int c,int d,int e,int f,char* g,int h){return 1;}
static int gw_Post_StageCover(int a,int b,char* c,int d){return 1;}
static void gw_Post_Protect(int a,int b){}
static int gw_Post_Set(int a,int b,char* c,char* d,int e){return 1;}
static int gw_presentation_cancel(void* t,int owner){gs_hitstop[0].used=0;return 1;}
static void gw_presentation_start(void* t,int owner,double n,int d){gs_hitstop[0].owner=owner;gs_hitstop[0].used=1;}
static void gs_rw_branch(void){}
static void gs_rw_stop(void){}
static int gw_ScriptGame_StageSwitch(int slot,int owner,int serial,int place){if(commit_refusal)return commit_refusal; ++commits;return 0;}
static void gs_stage_hook(const char* phase,int from,int to){if(!strcmp(phase,"after"))++after;}
static void gs_stage_slots_release(int script){assert(0);}
#include "stage_retry_native.inc"
static void finish(void){gs_stage_presentation_tick();now+=100;gs_stage_presentation_tick();now+=100;gs_stage_presentation_tick();}
int main(void){
    GwStageQueueEntry entries[5]={{0}}; int i;
    gs.n=gs.match_active=gs.s[0].used=1;gs.s[0].stage_owner=7;
    for(i=0;i<5;++i){entries[i].slot=i+2;entries[i].after=entries[i].stocks=-1;entries[i].duration=6;entries[i].transition=2;}
    gs_stage_director[0].queue.entries=entries;gs_stage_director[0].queue.count=5;gs_stage_director[0].queue.loop=1;gs_stage_director[0].queue.manual=1;gs_stage_directors=1;
    /* Before acceptance: repeated transient checks retain manual trigger/index. */
    refusal=-9;for(i=0;i<10;++i){gs_stage_logic_tick();assert(gs_stage_directors&&!gs_stage_director[0].pending&&gs_stage_director[0].queue.next==0&&gs_stage_director[0].queue.manual);}
    refusal=0;entries[0].indicator=2;gs_stage_logic_tick();assert(gs_stage_director[0].pending);
    /* Fighter becomes busy during indicator; start cancels envelope, not queue. */
    refusal=-9;gs_stage_presentation_tick();assert(gs_stage_director[0].pending);gs_stage_logic_tick();gs_stage_presentation_tick();assert(!gs_stage_director[0].pending&&!gs_stage_transition_owner&&gs_stage_director[0].queue.next==0);
    refusal=0;entries[0].indicator=0;gs_stage_logic_tick();gs_stage_presentation_tick();assert(gs_hitstop[0].used);
    /* Commit refusal also cleans freeze and retries same entry without input. */
    commit_refusal=-9;now+=100;gs_stage_presentation_tick();assert(!gs_stage_director[0].pending&&!gs_hitstop[0].used&&gs_stage_director[0].queue.next==0&&!commits);
    commit_refusal=0;gs_stage_logic_tick();gs_stage_presentation_tick();refusal=-9;now+=100;gs_stage_presentation_tick();assert(!gs_stage_director[0].pending&&!gs_hitstop[0].used&&gs_stage_director[0].queue.next==0&&!commits);
    refusal=0;gs_stage_logic_tick();finish();assert(commits==1&&after==1&&gs_stage_director[0].queue.next==1&&!gs_stage_director[0].queue.manual);
    for(i=0;i<5;++i){gs_stage_director[0].queue.entries[gs_stage_director[0].queue.next].after=0;gs_stage_logic_tick();finish();}
    assert(commits==6&&after==6&&gs_stage_director[0].queue.next==1);
    puts("production retry: preaccept, indicator/start and commit -9 retained; advance only success; five-entry loop continued");return 0;
}
