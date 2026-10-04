/* Isolated executable around the actual production game-BSS helpers.
 * This is not a LAB/game execution or endianness/rewind integration test. */
#include <assert.h>
#include <stdio.h>
#include <string.h>
#include <stdarg.h>
#include <math.h>
#include <float.h>
typedef struct HSD_GObj { void* user_data; } HSD_GObj;
typedef struct HitCapsule { float damage; unsigned unk_count,x24,x2C,element; int x34; } HitCapsule;
typedef struct Fighter { unsigned player_id; int ground_or_air,motion_id; struct {HSD_GObj* owner;} x1064_thrownHitbox; int is_sub_fighter; } Fighter;
typedef struct Item { HSD_GObj* owner; } Item;
#define GET_FIGHTER(g) ((Fighter*)(g)->user_data)
#define GA_Ground 0
static int script_hit_family_move(Fighter* f) {return f->motion_id;}
static void OSReport(const char* f,...) {(void)f;}
static int input[512],stun,trace[96],trace_owner[96],trace_count;
int Script_HitRulesInput(int n) {return input[n];}
int ftLib_80086960(HSD_GObj* g) {return g!=0;}
void Geno_GiveStunBonus(Fighter* v,int n) {(void)v;stun=n;}
void Script_HitRuleContext(int victim,int id,int move,int ground,int owner) {(void)victim;(void)move;(void)ground;if(trace_count<96){trace_owner[trace_count]=owner;trace[trace_count++]=id;}}
void Script_GameEvent(int a,int b,int c,int d,int e){(void)a;(void)b;(void)c;(void)d;(void)e;}
void ScriptGame_ReportHitContext(Fighter* a,Fighter* b,int c,int d){(void)a;(void)b;(void)c;(void)d;}
#include "../gameworld/script_hit_rules.inc"
static void rule(int id,int element,float damage,int status,int incoming) {
 memset(input,0,sizeof input);input[0]=id;input[1]=input[2]=input[3]=-1;input[4]=status;input[5]=element;
 input[6]=script_hr_bits(damage);input[7]=input[8]=input[11]=input[14]=input[15]=script_hr_bits(1);input[13]=incoming;
}
static HitCapsule capsule(int element) {HitCapsule h;memset(&h,0,sizeof h);h.damage=10;h.unk_count=10;h.x24=100;h.x2C=20;h.element=element;return h;}
typedef struct {HRPort ports[12];HRMeta meta[HR_META_MAX];int serial,count,warn;float percent_delta[12];} HitRuleSnapshot;
static HitRuleSnapshot saved_state,result_state,replayed_state;
static HitCapsule capacity_hits[HR_META_MAX+1];
static void save_hit_state(HitRuleSnapshot* out) {
 memset(out,0,sizeof *out);memcpy(out->ports,script_hit_ports,sizeof out->ports);memcpy(out->meta,script_hit_meta,sizeof out->meta);
 memcpy(out->percent_delta,script_hit_percent_delta,sizeof out->percent_delta);
 out->serial=script_hit_next_handle;out->count=script_hit_meta_count;out->warn=script_hit_capacity_warned;
}
static void restore_hit_state(const HitRuleSnapshot* in) {
 memcpy(script_hit_ports,in->ports,sizeof in->ports);memcpy(script_hit_meta,in->meta,sizeof in->meta);
 memcpy(script_hit_percent_delta,in->percent_delta,sizeof in->percent_delta);
 script_hit_next_handle=in->serial;script_hit_meta_count=in->count;script_hit_capacity_warned=in->warn;
}
static void test_progression(void) {
 Fighter a={0,0,4,{0}},b={1,0,0,{0}};HitCapsule h,copy;int i,j;
 /* Catch the old 8-rule truncation, including helper reads/IDs at the last slot. */
 rule(300,-1,1,0,0);input[14]=script_hr_bits(1.25f);input[15]=script_hr_bits(1.125f);
 for(i=1;i<32;++i){memcpy(input+i*16,input,16*sizeof(int));input[i*16]=300+i;}
 assert(ScriptGame_HitRulesReplace(0,12,32,0));
 assert(ScriptGame_HitRulesCount(0)==32 && ScriptGame_HitRuleRead(0,31,0)==331);
 assert(!ScriptGame_HitRulesReplace(0,12,33,0));
 h=capsule(0);copy=h;ScriptGame_HitRuleCreate(&a,&h,0);
 assert(!memcmp(&h,&copy,sizeof h));assert(ScriptGame_HitRuleContact(&h,&b,10,1)==40);
 ScriptGame_HitRulePercentQueue(&h,&b,10);save_hit_state(&saved_state);
 assert(ScriptGame_HitRulePercentCommit(&b,10)==90);save_hit_state(&result_state);
 restore_hit_state(&saved_state);assert(ScriptGame_HitRulePercentCommit(&b,10)==90);save_hit_state(&replayed_state);
 assert(!memcmp(&result_state,&replayed_state,sizeof result_state));
 for(i=0;i<32;++i){input[i*16+14]=script_hr_bits(64);input[i*16+15]=script_hr_bits(4);}
 assert(ScriptGame_HitRulesReplace(0,12,32,0));
 for(i=0;i<32;++i)input[i*16+13]=1;
 assert(ScriptGame_HitRulesReplace(2,12,32,0));
 assert(ScriptGame_HitRuleContact(&h,&b,10,1)==160);
 assert(isfinite(ScriptGame_HitRuleContact(&h,&b,FLT_MAX,1)));
 assert(ScriptGame_HitRuleContact(&h,&b,NAN,1)==0); /* outgoing4 * incoming4 */
 trace_count=0;ScriptGame_HitRuleContext(&h,&b);assert(trace_count==64);trace_count=0;
 ScriptGame_HitRulePercentQueue(&h,&b,10);assert(ScriptGame_HitRulePercentCommit(&b,10)==999); /* 64*64 */
 for(i=0;i<32;++i){input[i*16+14]=script_hr_bits(.05f);input[i*16+15]=script_hr_bits(.05f);}
 assert(ScriptGame_HitRulesReplace(2,12,32,0));
 assert(ScriptGame_HitRuleContact(&h,&b,10,1)>1.9999f && ScriptGame_HitRuleContact(&h,&b,10,1)<2.0001f);
 ScriptGame_HitRulePercentQueue(&h,&b,10);assert(fabsf(ScriptGame_HitRulePercentCommit(&b,10)-96)<.0001f);
 for(i=0;i<32;++i)input[i*16+13]=0;
 assert(ScriptGame_HitRulesReplace(0,12,32,0));
 ScriptGame_HitRulePercentQueue(&h,&b,10);assert(fabsf(ScriptGame_HitRulePercentCommit(&b,10)-.075f)<.00001f);
 /* The scalar helper must refuse malformed packets atomically, not poison BSS. */
 for(j=0;j<3;++j){float bad=j==0?NAN:j==1?INFINITY:-INFINITY;
  for(i=14;i<=15;++i){int old=input[i];input[i]=script_hr_bits(bad);save_hit_state(&saved_state);
   assert(!ScriptGame_HitRulesReplace(0,12,32,0));save_hit_state(&result_state);
   assert(!memcmp(&saved_state,&result_state,sizeof saved_state));input[i]=old;}}
 {int at=31*16+14,old=input[at];input[at]=script_hr_bits(NAN);save_hit_state(&saved_state);
  assert(!ScriptGame_HitRulesReplace(0,12,32,0));save_hit_state(&result_state);
  assert(!memcmp(&saved_state,&result_state,sizeof saved_state));input[at]=old;}
 input[6]=script_hr_bits(.05f);assert(!ScriptGame_HitRulesReplace(0,12,32,0));input[6]=script_hr_bits(1);
 input[14]=script_hr_bits(1000000128.f);assert(!ScriptGame_HitRulesReplace(0,12,32,0));input[14]=script_hr_bits(.05f);
 input[15]=script_hr_bits(-1000000128.f);assert(!ScriptGame_HitRulesReplace(0,12,32,0));
 rule(400,-1,1,0,0);input[14]=script_hr_bits(64);assert(ScriptGame_HitRulesReplace(0,12,1,0));
 rule(401,-1,1,0,1);input[14]=script_hr_bits(64);assert(ScriptGame_HitRulesReplace(2,12,1,0));
 h=capsule(0);ScriptGame_HitRuleCreate(&a,&h,0);
 ScriptGame_HitRulePercentQueue(&h,&b,FLT_MAX);ScriptGame_HitRulePercentQueue(&h,&b,FLT_MAX);
 assert(isfinite(script_hit_percent_delta[2]) && ScriptGame_HitRulePercentCommit(&b,FLT_MAX)==999);
 ScriptGame_HitRulePercentQueue(&h,&b,NAN);ScriptGame_HitRulePercentQueue(&h,&b,INFINITY);
 assert(script_hit_percent_delta[2]==0 && ScriptGame_HitRulePercentCommit(&b,NAN)==0);
 assert(ScriptGame_HitRulePercentCommit(&b,INFINITY)==999 && ScriptGame_HitRulePercentCommit(&b,-INFINITY)==0);
 ScriptGame_HitRulesRelease(0);
}
static void test_signed_contributions(void) {
 Fighter a={0,0,4,{0}},b={1,0,0,{0}};HitCapsule h;int i,j;
 static const int order[6][3]={{0,1,2},{0,2,1},{1,0,2},{1,2,0},{2,0,1},{2,1,0}};
 static const float values[]={1000000000.f,-1000000000.f,2.5f};
 /* Encoded1+rawdelta: steady-1.9 and conditionalcost+.6 must reach the
  * final incoming .15 floor, not an early floor plus the conditional cost. */
 rule(501,-1,1,0,1);input[14]=input[15]=script_hr_bits(-.9f);
 memcpy(input+16,input,16*sizeof(int));input[16]=502;input[20]=8;input[30]=input[31]=script_hr_bits(1.6f);
 assert(ScriptGame_HitRulesReplace(2,12,2,8));
 h=capsule(0);ScriptGame_HitRuleCreate(&a,&h,0);
 ScriptGame_HitRulePercentQueue(&h,&b,10);assert(fabsf(ScriptGame_HitRulePercentCommit(&b,10)-1.5f)<.00001f);
 assert(fabsf(ScriptGame_HitRuleContact(&h,&b,10,1)-.5f)<.00001f);
 ScriptGame_HitRulesRelease(0);
 for(i=0;i<6;++i){
  rule(600,-1,1,0,0);
  for(j=0;j<3;++j){if(j)memcpy(input+j*16,input,16*sizeof(int));input[j*16]=600+j;
   input[j*16+14]=input[j*16+15]=script_hr_bits(values[order[i][j]]);}
  assert(ScriptGame_HitRulesReplace(0,12,3,0));
  h=capsule(0);ScriptGame_HitRuleCreate(&a,&h,0);trace_count=0;ScriptGame_HitRuleContext(&h,&b);
  assert(trace_count==3);ScriptGame_HitRulePercentQueue(&h,&b,10);
  assert(fabsf(ScriptGame_HitRulePercentCommit(&b,10)-5)<.00001f);
  assert(fabsf(ScriptGame_HitRuleContact(&h,&b,10,1)-5)<.00001f);
 }
 ScriptGame_HitRulesRelease(0);trace_count=0;
}
static void test_context_replacement(void) {
 Fighter a={0,0,4,{0}},b={1,0,0,{0}};HitCapsule h;int i;
 /* An old capsule retains32 creation IDs after its current attacker table is
  * replaced; current attacker and defender can contribute32 new IDs each. */
 rule(300,-1,1.01f,0,0);
 for(i=1;i<32;++i){memcpy(input+i*16,input,16*sizeof(int));input[i*16]=300+i;}
 assert(ScriptGame_HitRulesReplace(0,12,32,0));
 h=capsule(0);ScriptGame_HitRuleCreate(&a,&h,0);
 assert(script_hit_metadata(&h,0)->count==32);
 rule(700,-1,1,0,0);input[14]=script_hr_bits(1.25f);
 for(i=1;i<32;++i){memcpy(input+i*16,input,16*sizeof(int));input[i*16]=700+i;}
 assert(ScriptGame_HitRulesReplace(0,12,32,0));
 for(i=0;i<32;++i){input[i*16]=900+i;input[i*16+13]=1;}
 assert(ScriptGame_HitRulesReplace(2,13,32,0));
 trace_count=0;ScriptGame_HitRuleContext(&h,&b);
 assert(trace_count==96 && trace[0]==300 && trace[31]==331 && trace[32]==700 && trace[63]==731 && trace[64]==900 && trace[95]==931);
 assert(trace_owner[0]==12 && trace_owner[63]==12 && trace_owner[64]==13 && trace_owner[95]==13);
 ScriptGame_HitRulesRelease(0);
}
int main(void) {
 test_signed_contributions();test_progression();test_context_replacement();trace_count=0;
 Fighter a={0,0,4,{0}},b={1,0,0,{0}},c={2,1,0,{0}},partner={1,0,0,{0},1};HSD_GObj owner={&a};Item item={&owner};
 HitCapsule h,copy;int e;
 rule(200,-1,1,0,0);input[14]=script_hr_bits(1.6f);input[15]=script_hr_bits(1.3f);
 assert(ScriptGame_HitRulesReplace(0,12,1,0));
 assert(ScriptGame_HitRuleRead(0,0,14)==script_hr_bits(1.6f));
 ScriptGame_HitRulesRelease(0);
 /* Percent-only contact never touches retail hitbox, stale/clank/shield or launch damage. */
 rule(201,-1,1,0,0);input[14]=script_hr_bits(1.6f);
 assert(ScriptGame_HitRulesReplace(0,12,1,0));
 h=capsule(0);copy=h;ScriptGame_HitRuleCreate(&a,&h,0);assert(!memcmp(&copy,&h,sizeof h));
 assert(ScriptGame_HitRuleContact(&h,&b,10,0)==10);
 assert(ScriptGame_HitRuleContact(&h,&b,100,1)==100);
 ScriptGame_HitRulePercentQueue(&h,&b,10);ScriptGame_HitRulePercentQueue(&h,&c,5);
 assert(ScriptGame_HitRuleContact(&h,&b,100,1)==100); /* another same-frame launch */
 assert(ScriptGame_HitRulePercentCommit(&b,10)==16);
 assert(ScriptGame_HitRulePercentCommit(&c,5)==8);
 assert(ScriptGame_HitRulePercentCommit(&b,10)==10); /* exactly once */
 /* Current conditional + unconditional family adds, one final cap; victims remain independent. */
 rule(202,-1,1,0,0);input[14]=script_hr_bits(1.4f);input[15]=script_hr_bits(1.2f);
 memcpy(input+16,input,16*sizeof(int));input[16]=203;input[20]=8;
 input[30]=script_hr_bits(1.3f);input[31]=script_hr_bits(1.2f);
 assert(ScriptGame_HitRulesReplace(0,12,2,0));
 rule(204,-1,1,0,1);input[14]=script_hr_bits(1.2f);input[15]=script_hr_bits(1.1f);
 assert(ScriptGame_HitRulesReplace(2,12,1,8));
 h=capsule(0);ScriptGame_HitRuleCreate(&a,&h,0);copy=h;
 assert(ScriptGame_HitRuleContact(&h,&c,100,1)>119.99f && ScriptGame_HitRuleContact(&h,&c,100,1)<120.01f);
 assert(ScriptGame_HitRuleContact(&h,&b,100,1)>153.99f && ScriptGame_HitRuleContact(&h,&b,100,1)<154.01f);
 ScriptGame_HitRulePercentQueue(&h,&b,10);ScriptGame_HitRulePercentQueue(&h,&c,10);
 assert(ScriptGame_HitRulePercentCommit(&b,10)>20.39f && ScriptGame_HitRulePercentCommit(&b,10)<20.41f);
 assert(ScriptGame_HitRulePercentCommit(&c,10)>13.99f && ScriptGame_HitRulePercentCommit(&c,10)<14.01f);
 assert(!memcmp(&h,&copy,sizeof h));
 ScriptGame_HitRulePercentQueue(&h,&b,10);save_hit_state(&saved_state);
 assert(ScriptGame_HitRulePercentCommit(&b,10)>20.39f);save_hit_state(&result_state);
 restore_hit_state(&saved_state);assert(ScriptGame_HitRulePercentCommit(&b,10)>20.39f);save_hit_state(&replayed_state);
 assert(!memcmp(&result_state,&replayed_state,sizeof result_state));
 ScriptGame_HitRulePercentQueue(&h,&b,10);ScriptGame_HitRulePercentClear(&b);
 assert(ScriptGame_HitRulePercentCommit(&b,10)==10);
 /* Incoming reductions retain their value down to the .05 safety floor; immune hit elements remain entirely unaffected. */
 rule(205,-1,1,0,1);input[14]=script_hr_bits(0.1f);input[15]=script_hr_bits(0.1f);
 assert(ScriptGame_HitRulesReplace(2,12,1,0));
 ScriptGame_HitRulesReplace(0,12,0,0);
 h=capsule(0);ScriptGame_HitRuleCreate(&a,&h,0);ScriptGame_HitRulePercentQueue(&h,&b,10);
 assert(fabsf(ScriptGame_HitRulePercentCommit(&b,10)-1.5f)<.00001f);
 assert(ScriptGame_HitRuleContact(&h,&b,100,1)>9.99f && ScriptGame_HitRuleContact(&h,&b,100,1)<10.01f);
 h=capsule(7);ScriptGame_HitRulePercentQueue(&h,&b,10);assert(ScriptGame_HitRulePercentCommit(&b,10)==10);
 assert(ScriptGame_HitRuleContact(&h,&b,100,1)==100);
 ScriptGame_HitRulesRelease(0);
 h=capsule(0);ScriptGame_HitRuleCreate(&a,&h,0);assert(script_hit_meta_count==0);
 rule(100,1,2,0,0);assert(ScriptGame_HitRulesReplace(6,12,1,0));a.player_id=3;h=capsule(0);ScriptGame_HitRuleCreate(&a,&h,0);assert(h.element==1&&h.damage==20);ScriptGame_HitRulesRelease(0);a.player_id=0;
 rule(101,1,2,0,0);input[10]=7;assert(ScriptGame_HitRulesReplace(0,12,1,0));
 h=capsule(0);ScriptGame_HitRuleCreate(&a,&h,0);assert(h.element==1&&h.damage==20&&h.unk_count==20);
 ScriptGame_HitRuleWon(&h,&b);assert(stun==7);ScriptGame_HitRuleContext(&h,&b);assert(trace_count==1&&trace[0]==101);
 ScriptGame_HitRuleSpecial(&h);assert(h.damage==10&&h.unk_count==10&&h.x24==100&&h.x2C==20);assert(!script_hit_metadata(&h,0));
 for(e=0;e<18;++e) {h=capsule(e);copy=h;ScriptGame_HitRuleCreate(&a,&h,0);if(hr_ordinary(e))assert(h.damage==20&&h.element==1);else assert(!memcmp(&h,&copy,sizeof h));}
 /* Contact rules are current; creation move/ground/original element retained. */
 rule(102,-1,2,8,0);input[11]=script_hr_bits(1.5f);assert(ScriptGame_HitRulesReplace(0,12,1,0));
 rule(0,-1,1,0,0);assert(ScriptGame_HitRulesReplace(2,12,0,8));
 h=capsule(0);ScriptGame_HitRuleItemCreate(&item,&h);copy=h;
 assert(ScriptGame_HitRuleContact(&h,&b,10,0)==20);assert(ScriptGame_HitRuleContact(&h,&c,10,0)==10);
 assert(ScriptGame_HitRuleContact(&h,&partner,10,0)==10);
 assert(ScriptGame_HitRulesReplace(3,12,0,8));assert(ScriptGame_HitRuleContact(&h,&partner,10,0)==20);
 assert(ScriptGame_HitRulesReplace(2,12,0,0));assert(ScriptGame_HitRuleContact(&h,&b,10,0)==10);
 assert(ScriptGame_HitRulesReplace(2,12,0,8));
 assert(ScriptGame_HitRuleContact(&h,&b,100,1)==150);assert(!memcmp(&h,&copy,sizeof h));
 assert(ScriptGame_HitRulesReplace(0,12,0,0));assert(ScriptGame_HitRuleContact(&h,&b,10,0)==10);
 /* Incoming elemental vulnerability applies even to attacker with no rules. */
 rule(103,-1,2,0,1);input[3]=5;assert(ScriptGame_HitRulesReplace(2,12,1,0));
 h=capsule(5);ScriptGame_HitRuleForget(&h);assert(ScriptGame_HitRuleContact(&h,&b,10,0)==20);ScriptGame_HitRuleCreate(&a,&h,0);assert(ScriptGame_HitRuleContact(&h,&b,10,0)==20);
 h=capsule(0);ScriptGame_HitRuleCreate(&a,&h,0);assert(ScriptGame_HitRuleContact(&h,&b,10,0)==10);
 h=capsule(0);h.damage=700;h.unk_count=700;h.x24=1200;h.x2C=1200;h.x34=150;copy=h;
 ScriptGame_HitRuleCreate(&a,&h,0);assert(!memcmp(&h,&copy,sizeof h));assert(ScriptGame_HitRuleContact(&h,&b,700,0)==700);
 /* Replaying identical state from the same saved bytes is exact for these helpers. */
 save_hit_state(&saved_state);
 rule(104,2,4,0,0);assert(ScriptGame_HitRulesReplace(0,12,1,0));h=capsule(0);ScriptGame_HitRuleCreate(&a,&h,0);
 save_hit_state(&result_state);restore_hit_state(&saved_state);
 assert(ScriptGame_HitRulesReplace(0,12,1,0));h=capsule(0);ScriptGame_HitRuleCreate(&a,&h,0);save_hit_state(&replayed_state);
 assert(!memcmp(&result_state,&replayed_state,sizeof result_state));
 /* Updating charged damage and baseline restore must not compound. */
 h.damage=12;h.unk_count=12;ScriptGame_HitRuleDamage(&h);assert(h.damage==48&&h.unk_count==48);
 ScriptGame_HitRuleSpecial(&h);assert(h.damage==12&&h.unk_count==12);
 ScriptGame_HitRuleRetire(&h,sizeof h);assert(!script_hit_metadata(&h,0));
 rule(105,2,1,0,0);assert(ScriptGame_HitRulesReplace(11,12,1,16));
 a.player_id=5;a.is_sub_fighter=1;h=capsule(0);ScriptGame_HitRuleCreate(&a,&h,0);assert(h.element==2);
 a.is_sub_fighter=0;h=capsule(0);ScriptGame_HitRuleCreate(&a,&h,0);assert(h.element==0);
 ScriptGame_HitRulesRelease(12);assert(!ScriptGame_HitRulesCount(0)&&!ScriptGame_FighterStatus(2));
 ScriptGame_HitRulesRelease(0);for(e=0;e<HR_META_MAX;++e)assert(!script_hit_meta[e].hit);
 rule(106,1,2,0,0);assert(ScriptGame_HitRulesReplace(10,12,1,0));
 for(e=0;e<HR_META_MAX+1;++e){capacity_hits[e]=capsule(0);ScriptGame_HitRuleCreate(&a,&capacity_hits[e],0);}
 assert(script_hit_meta_count==HR_META_MAX && script_hit_capacity_warned==1);
 assert(capacity_hits[HR_META_MAX].damage==10 && capacity_hits[HR_META_MAX].element==0);
 ScriptGame_HitRuleRetire(capacity_hits,sizeof capacity_hits);assert(script_hit_meta_count==0);
 ScriptGame_HitRulesRelease(0);for(e=0;e<12;++e)assert(!ScriptGame_HitRulesCount(e)&&!ScriptGame_FighterStatus(e));
 puts("native hit rules: creation, special immunity, current contact, victims, incoming, hitstun, cleanup independent partner states and helper replay PASS");return 0;
}
