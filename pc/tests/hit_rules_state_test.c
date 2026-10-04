/* Isolated executable around the actual production game-BSS helpers.
 * This is not a LAB/game execution or endianness/rewind integration test. */
#include <assert.h>
#include <stdio.h>
#include <string.h>
#include <stdarg.h>
typedef struct HSD_GObj { void* user_data; } HSD_GObj;
typedef struct HitCapsule { float damage; unsigned unk_count,x24,x2C,element; int x34; } HitCapsule;
typedef struct Fighter { unsigned player_id; int ground_or_air,motion_id; struct {HSD_GObj* owner;} x1064_thrownHitbox; int is_sub_fighter; } Fighter;
typedef struct Item { HSD_GObj* owner; } Item;
#define GET_FIGHTER(g) ((Fighter*)(g)->user_data)
#define GA_Ground 0
static int script_hit_family_move(Fighter* f) {return f->motion_id;}
static void OSReport(const char* f,...) {(void)f;}
static int input[112],stun,trace[24],trace_count;
int Script_HitRulesInput(int n) {return input[n];}
int ftLib_80086960(HSD_GObj* g) {return g!=0;}
void Geno_GiveStunBonus(Fighter* v,int n) {(void)v;stun=n;}
void Script_HitRuleContext(int victim,int id,int move,int ground,int owner) {(void)owner;(void)victim;(void)move;(void)ground;if(trace_count<24)trace[trace_count++]=id;}
void Script_GameEvent(int a,int b,int c,int d,int e){(void)a;(void)b;(void)c;(void)d;(void)e;}
void ScriptGame_ReportHitContext(Fighter* a,Fighter* b,int c,int d){(void)a;(void)b;(void)c;(void)d;}
#include "../gameworld/script_hit_rules.inc"
static void rule(int id,int element,float damage,int status,int incoming) {
 memset(input,0,sizeof input);input[0]=id;input[1]=input[2]=input[3]=-1;input[4]=status;input[5]=element;
 input[6]=script_hr_bits(damage);input[7]=input[8]=input[11]=script_hr_bits(1);input[13]=incoming;
}
static HitCapsule capsule(int element) {HitCapsule h;memset(&h,0,sizeof h);h.damage=10;h.unk_count=10;h.x24=100;h.x2C=20;h.element=element;return h;}
typedef struct {HRPort ports[12];HRMeta meta[HR_META_MAX];int serial,count,warn;} HitRuleSnapshot;
static HitRuleSnapshot saved_state,result_state,replayed_state;
static HitCapsule capacity_hits[HR_META_MAX+1];
static void save_hit_state(HitRuleSnapshot* out) {
 memset(out,0,sizeof *out);memcpy(out->ports,script_hit_ports,sizeof out->ports);memcpy(out->meta,script_hit_meta,sizeof out->meta);
 out->serial=script_hit_next_handle;out->count=script_hit_meta_count;out->warn=script_hit_capacity_warned;
}
static void restore_hit_state(const HitRuleSnapshot* in) {
 memcpy(script_hit_ports,in->ports,sizeof in->ports);memcpy(script_hit_meta,in->meta,sizeof in->meta);
 script_hit_next_handle=in->serial;script_hit_meta_count=in->count;script_hit_capacity_warned=in->warn;
}
int main(void) {
 Fighter a={0,0,4,{0}},b={1,0,0,{0}},c={2,1,0,{0}},partner={1,0,0,{0},1};HSD_GObj owner={&a};Item item={&owner};
 HitCapsule h,copy;int e;
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
