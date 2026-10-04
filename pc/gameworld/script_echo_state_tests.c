/* Executes production MEM1 history/replay helpers with engine services stubbed.
 * Does not claim geometry, shield, clank, EXE or LAB rollback integration. */
#include <assert.h>
#include <stdio.h>
#include <string.h>
#include <stdarg.h>
#include <float.h>
#include <math.h>
#define SCRIPT_ECHO_H
#define GALE01_005BB0
typedef unsigned char u8;
typedef struct {float x,y,z;} Vec3;
typedef struct HSD_GObj {void* user_data;struct HSD_GObj* next;} HSD_GObj;
typedef struct {void* victim;unsigned x4;} HitVictim;
typedef struct HitCapsule {
 int state;unsigned x4,unk_count;float damage;Vec3 b_offset;float scale;
 int kb_angle;unsigned x24,x28,x2C,element;int x34;
 unsigned x43_b1;u8 x44,x45;void* jobj;Vec3 x4C,x58,hurt_coll_pos;
 float coll_distance;HitVictim victims_1[12],victims_2[12];HSD_GObj* owner;
} HitCapsule;
typedef struct Fighter {
 unsigned player_id;int is_sub_fighter,motion_id,ground_or_air,x2068_attackID,x206C_attack_instance,team,x2225_b4;HSD_GObj* gobj;
 float cur_anim_frame,facing_dir;Vec3 cur_pos,x34_scale;HitCapsule x914[4];
 HitCapsule x1064_thrownHitbox;
 struct {float x1830_percent;} dmg;
} Fighter;
typedef struct {HSD_GObj* owner;unsigned short xD8C_attack_instance;int xD88_attackID;} Item;
#define GA_Air 1
#define GA_Ground 0
#define HitCapsule_Disabled 0
#define ftCo_MS_AttackAirN 65
#define HSD_GOBJ_PLINK_FIGHTER 0
#define GET_FIGHTER(g) ((Fighter*)(g)->user_data)
static HSD_GObj* HSD_GObjPLinkHead[1];
static int input[1536],resource_alive=1,context_motion,context_move,context_ground;
static int credited_attack,credited_instance;
static int teams,team_attack;
int gm_8016B168(void) {return teams;}
int gm_8016B0D4(void) {return team_attack;}
static int script_hit_family_move(Fighter* f) {return f->motion_id>=65 && f->motion_id<=69 ? 5 : 1;}
int Script_HitRulesInput(int n) {return input[n];}
int Script_EchoInput(int n) {return input[n];}
int Script_StageResourceOwnerAlive(int n) {(void)n;return resource_alive;}
int Script_EchoOwnerAlive(int n) {(void)n;return resource_alive;}
int ftLib_80086960(HSD_GObj* g) {return g!=0;}
/* Use the actual retail allocator and deduplication, with only player storage stubbed. */
#define _plstale_h_
#define _player_h_
#define MELEE_PL_TYPES_H
#define GALE01_0860C4
#define MELEE_FT_INLINES_H
#define MELEE_FT_TYPES_H
#define MELEE_IT_INLINES_H
#define MELEE_IT_TYPES_H
typedef unsigned short u16;
typedef int s32;
typedef struct {int current_index;struct {u16 move_id,attack_instance;} StaleMoves[10];} StaleMoveTable;
static StaleMoveTable stale_tables[6];
StaleMoveTable* Player_GetStaleMoveTableIndexPtr(int slot) {return &stale_tables[slot];}
#define GET_ITEM(g) ((Item*)(g)->user_data)
#include "../../src/melee/pl/plstale.c"
#define retail_instance staleAttackInstance
void Geno_GiveStunBonus(Fighter* f,int n) {(void)f;(void)n;}
static void OSReport(const char* f,...) {(void)f;}
void Script_HitRuleContext(int a,int b,int c,int d,int e) {(void)a;(void)b;(void)c;(void)d;(void)e;}
void Script_GameEvent(int a,int b,int c,int d,int e) {(void)a;(void)b;(void)c;(void)d;(void)e;}
void ScriptGame_ReportHitContext(Fighter* a,Fighter* b,int c,int d) {(void)a;(void)b;(void)c;(void)d;}
void Script_HitContext(int a,int b,int c,int motion,int move,int ground,int vg,int ab,int vb) {
 (void)a;(void)b;(void)c;(void)vg;(void)ab;(void)vb;context_motion=motion;context_move=move;context_ground=ground;
}
int lbColl_80008688(HitCapsule* h,int mode,void* victim) {
 int i;(void)mode;for(i=0;i<h->x44;++i)if(h->victims_1[i].victim==victim)return 0;
 if(h->x44<12)h->victims_1[h->x44++].victim=victim;return 1;
}
void ftColl_8007891C(HSD_GObj* a,HSD_GObj* b,float damage) {
 Fighter* f=GET_FIGHTER(a);(void)b;(void)damage;credited_attack=f->x2068_attackID;credited_instance=f->x206C_attack_instance;
 plStale_UpdateStaleMovesFromFighter(a,b);
}
#include "script_hit_rules.inc"
void ScriptGame_EchoPrepare(void);
#include "script_echo.inc"
static unsigned char saved[sizeof script_echo],result[sizeof script_echo];
static Fighter owner,victim,sub;static HSD_GObj objects[3];
static void frame(float x,int active) {
 owner.cur_pos.x=x;owner.cur_anim_frame+=1;owner.x914[0].state=active;
 owner.x914[0].x4C.x=x;owner.x914[0].x58.x=x-1;
 ScriptGame_EchoPrepare();ScriptGame_EchoFrame();
}
static void test_review_limits_group_stale(void) {
 int handle;HitCapsule *a,*b;
 ScriptGame_EchoReset();resource_alive=1;owner.motion_id=65;owner.cur_anim_frame=0;
 owner.x914[0].damage=200;owner.x914[0].unk_count=200;owner.x914[0].x24=800;
 owner.x914[0].x2C=800;owner.x914[0].x28=800;owner.x914[0].x4=7;
 owner.x914[1]=owner.x914[0];owner.x914[1].state=1;retail_instance=777;
 plStale_ResetStaleMoveTableForPlayer(0);owner.x206C_attack_instance=plStale_IncrementAttackInstance();
 ftColl_8007891C(owner.gobj,victim.gobj,200);assert(stale_tables[0].current_index==1);
 handle=ScriptGame_EchoAdd(0,1,1,-1,-1,-1,script_echo_bits(4),script_echo_bits(4),-1,0);
 assert(handle);frame(10,1);frame(20,1);
 a=ScriptGame_EchoCapsule(&owner,4);b=ScriptGame_EchoCapsule(&owner,5);
 assert(a->damage==500 && a->unk_count==500);
 assert(a->x24==1000 && a->x28==1000 && a->x2C==1000);
 assert(ScriptGame_EchoCredit(&owner,&victim,a,a->damage));assert(credited_instance==778);
 assert(stale_tables[0].current_index==2 && stale_tables[0].StaleMoves[1].attack_instance==778);
 assert(a->x4==b->x4);ScriptGame_EchoConnected(a,&victim);
 assert(ScriptGame_EchoBlocked(b,&victim));
 puts("echo review regressions: damage500 KB1000 retail instance shared hit-group PASS");
}
static void test_review_thrown_history_owner(void) {
 int handle,victim_handle;HitCapsule* body;
 ScriptGame_EchoReset();owner.x914[0].state=owner.x914[1].state=0;
 owner.motion_id=220;owner.cur_anim_frame=0;victim.motion_id=38;
 victim.x1064_thrownHitbox=owner.x914[0];body=&victim.x1064_thrownHitbox;
 body->state=1;body->owner=owner.gobj;body->scale=3;body->x43_b1=0;
 body->x4C.x=123;body->x58.x=122;victim.x34_scale.y=2;
 handle=ScriptGame_EchoAdd(0,1,1,-1,-1,-1,script_echo_bits(1),script_echo_bits(1),-1,0);
 victim_handle=ScriptGame_EchoAdd(2,1,1,-1,-1,-1,script_echo_bits(1),script_echo_bits(1),-1,0);
 assert(handle && victim_handle);frame(200,1);frame(201,1);
 assert(ScriptGame_FighterHistoryRead(2,0,4,0));
 assert(ScriptGame_FighterHistoryRead(2,0,4,18)==0);
 assert(ScriptGame_FighterHistoryRead(2,0,4,19)==4);
 assert(ScriptGame_FighterHistoryRead(0,0,4,0));
 body=ScriptGame_EchoCapsule(&owner,8);assert(body->state && body->x4C.x==123 && body->scale==6);
 assert(!ScriptGame_EchoSameGroup(body,ScriptGame_EchoCapsule(&owner,4)));
 assert(!ScriptGame_EchoCapsule(&victim,8)->state);
 assert(ScriptGame_EchoCredit(&owner,&sub,body,body->damage));assert(credited_attack==owner.x2068_attackID);
 owner.x1064_thrownHitbox.owner=sub.gobj;
 assert(!ScriptGame_EchoTargetAllowed(&owner,&sub));
 teams=1;team_attack=0;owner.team=victim.team=2;assert(!ScriptGame_EchoTargetAllowed(&owner,&victim));
 team_attack=1;assert(ScriptGame_EchoTargetAllowed(&owner,&victim));
 victim.team=3;team_attack=0;assert(ScriptGame_EchoTargetAllowed(&owner,&victim));
 teams=0;owner.x1064_thrownHitbox.owner=NULL;
 assert(ScriptGame_EchoCapsuleCount(&owner)==9);
 victim.x1064_thrownHitbox.owner=NULL;frame(202,0);
 assert(!ScriptGame_FighterHistoryRead(2,0,4,0));
 puts("echo review thrown body: fifth history resolved geometry real thrower ownership no victim replay stale-owner filter PASS");
}
static void test_review_namespace_fallback(void) {
 int one,two;unsigned count;HitCapsule *a,*b;
 ScriptGame_EchoReset();resource_alive=1;owner.motion_id=65;owner.cur_anim_frame=0;
 owner.x1064_thrownHitbox.owner=NULL;victim.x1064_thrownHitbox.owner=NULL;
 owner.x914[0].state=1;owner.x914[1]=owner.x914[0];owner.x914[1].state=1;
 one=ScriptGame_EchoAdd(0,1,1,-1,-1,-1,script_echo_bits(1),script_echo_bits(1),-1,0);
 two=ScriptGame_EchoAdd(0,1,1,-1,-1,-1,script_echo_bits(1),script_echo_bits(1),-1,0);
 assert(one && two);frame(10,1);frame(20,1);
 a=ScriptGame_EchoCapsule(&owner,4);b=ScriptGame_EchoCapsule(&owner,9);
 assert(a->x4==b->x4 && !ScriptGame_EchoSameGroup(a,b));
 assert(!ScriptGame_EchoSameGroup(a,&owner.x914[0]));
 ScriptGame_EchoConnected(a,&victim);assert(!ScriptGame_EchoBlocked(b,&victim));
 count=script_echo.history.ring[0].count;owner.cur_pos.x=999;
 /* No collision traversal this logic frame: end hook still records its age. */
 ScriptGame_EchoFrame();assert(script_echo.history.ring[0].count==count+1);
 assert(ScriptGame_FighterHistoryRead(0,0,-1,6)==script_echo_bits(999));
 puts("echo review namespace fallback: copies/live independently grouped all-disabled collision age recording PASS");
}
static void test_review_move_instance_reentry(void) {
 HitCapsule* echo;int handle;
 ScriptGame_EchoReset();owner.motion_id=65;owner.cur_anim_frame=0;owner.x206C_attack_instance=50;
 owner.x914[0].state=1;owner.x914[1].state=0;
 handle=ScriptGame_EchoAdd(0,1,1,-1,-1,-1,script_echo_bits(1),script_echo_bits(1),-1,1);assert(handle);
 ScriptGame_EchoPrepare();ScriptGame_EchoFrame();
 ScriptGame_EchoPrepare();ScriptGame_EchoFrame();echo=ScriptGame_EchoCapsule(&owner,4);
 ScriptGame_EchoConnected(echo,&victim);assert(ScriptGame_EchoBlocked(echo,&victim));
 owner.x206C_attack_instance=51;/* same nair, same animation frame, distinct live move */
 ScriptGame_EchoPrepare();ScriptGame_EchoFrame();assert(ScriptGame_EchoBlocked(echo,&victim));
 ScriptGame_EchoPrepare();ScriptGame_EchoFrame();assert(!ScriptGame_EchoBlocked(echo,&victim));
 ScriptGame_EchoConnected(echo,&victim);ScriptGame_EchoPrepare();ScriptGame_EchoFrame();
 assert(ScriptGame_EchoBlocked(echo,&victim));/* same instance held in hitlag */
 puts("echo review identity: same-frame same-motion retail-instance reentry resets target, frozen instance retains PASS");
}
int main(void) {
 int handle,i;HitCapsule* h;float before;
 plStale_InitAttackInstance();
 memset(&owner,0,sizeof owner);memset(&victim,0,sizeof victim);memset(&sub,0,sizeof sub);
 owner.motion_id=65;owner.ground_or_air=1;owner.x34_scale.y=2;owner.x2068_attackID=17;
 owner.x914[0].damage=10;owner.x914[0].unk_count=10;owner.x914[0].scale=3;owner.x914[0].element=1;
 owner.x914[0].x24=100;owner.x914[0].x2C=20;owner.x914[0].x4=7;
 victim.player_id=1;victim.motion_id=14;victim.x34_scale.y=1;
 sub=owner;sub.is_sub_fighter=1;sub.x914[0].damage=12;
 objects[0].user_data=&owner;objects[0].next=&objects[1];objects[1].user_data=&victim;
 objects[1].next=&objects[2];objects[2].user_data=&sub;HSD_GObjPLinkHead[0]=objects;
 owner.gobj=&objects[0];victim.gobj=&objects[1];sub.gobj=&objects[2];
 handle=ScriptGame_EchoAdd(0,1,2,10,-1,1,script_echo_bits(.4f),script_echo_bits(.5f),-1,1);
 assert(handle>0);frame(10,1);frame(20,1);frame(30,1);
 h=ScriptGame_EchoCapsule(&owner,4);assert(h->state && h->x4C.x==10 && h->x58.x==9);
 assert(h->damage==4 && h->x24==50 && h->scale==6 && h->x43_b1);
 owner.x2068_attackID=20;assert(ScriptGame_EchoCredit(&owner,&victim,h,4));
 assert(credited_attack==17 && credited_instance && owner.x2068_attackID==20);owner.x2068_attackID=17;
 assert(ScriptGame_EchoSourceX(h,script_echo_bits(999))==script_echo_bits(10));
 assert(ScriptGame_FighterHistoryRead(0,0,-1,6)==script_echo_bits(30));
 assert(ScriptGame_FighterHistoryRead(1,0,-1,0)==1);
 assert(ScriptGame_FighterHistoryRead(2,0,-1,10)==0);
 assert(ScriptGame_EchoReportContext(&owner,&victim,h));assert(context_motion==65 && context_move==5 && context_ground==0);
 assert(!ScriptGame_EchoBlocked(h,&victim));ScriptGame_EchoConnected(h,&victim);assert(ScriptGame_EchoBlocked(h,&victim));
 assert(ScriptGame_EchoState(0,handle,2)==4);
 frame(40,0);frame(50,1);frame(60,1);frame(70,1);
 h=ScriptGame_EchoCapsule(&owner,4);assert(h->state && ScriptGame_EchoBlocked(h,&victim));
 memcpy(saved,&script_echo,sizeof script_echo);frame(80,1);memcpy(result,&script_echo,sizeof script_echo);
 memcpy(&script_echo,saved,sizeof script_echo);owner.cur_anim_frame-=1;frame(80,1);
 assert(!memcmp(result,&script_echo,sizeof script_echo));
 for(i=0;i<75;++i)frame((float)i,1);
 assert(ScriptGame_FighterHistoryRead(0,60,-1,0));assert(!ScriptGame_FighterHistoryRead(0,61,-1,0));
 owner.motion_id=66;frame(100,1);frame(101,1);frame(102,1);assert(!ScriptGame_EchoState(0,handle,0));
 owner.motion_id=65;frame(110,1);frame(111,1);frame(112,1);h=ScriptGame_EchoCapsule(&owner,4);
 assert(!ScriptGame_EchoBlocked(h,&victim));
 /* Metadata creation must not double the already-resolved damage. */
 before=h->damage;memset(input,0,sizeof input);input[0]=81;input[1]=input[2]=input[3]=-1;
 input[5]=-1;input[6]=script_hr_bits(2);input[7]=input[8]=input[11]=input[14]=input[15]=script_hr_bits(1);
 assert(ScriptGame_HitRulesReplace(0,1,1,0));ScriptGame_HitRuleForget(h);
 ScriptGame_HitRuleEchoCreate(&owner,h,5,0,1);assert(h->damage==before);
 ScriptGame_EchoClear(1);assert(!ScriptGame_EchoState(0,handle,0));
 assert(!ScriptGame_EchoAdd(0,1,1,-1,-1,-1,script_echo_bits(1),script_echo_bits(1),31,0));
 for(i=0;i<8;++i)assert(ScriptGame_EchoAdd(0,1,i+1,-1,-1,-1,script_echo_bits(1),script_echo_bits(1),-1,0));
 assert(!ScriptGame_EchoAdd(0,1,9,-1,-1,-1,script_echo_bits(1),script_echo_bits(1),-1,0));
 owner.motion_id=0;frame(0,0);assert(!ScriptGame_FighterHistoryRead(0,0,-1,0));
 resource_alive=0;ScriptGame_EchoFrame();assert(!ScriptGame_EchoOwner(0));
 ScriptGame_EchoReset();assert(!ScriptGame_FighterHistoryRead(1,0,-1,0));
 test_review_limits_group_stale();
 test_review_thrown_history_owner();
 test_review_namespace_fallback();
 test_review_move_instance_reentry();
 printf("echo production replay: recorded timing radius subfighter once gap move context no-double-apply capacity8 cleanup deterministic snapshot 0 differing bytes PASS; fixture state=%u bytes\n",(unsigned)sizeof script_echo);
 return 0;
}
