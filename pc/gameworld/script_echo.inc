/* Included in script_game.c: this entire bounded state shares LAB MEM1 BSS snapshots.
 * No heap, native pointers, presentation state or Lua runs in collision. */
#include "script_echo.h"
#include <melee/lb/lbcollision.h>
#define ECHO_CAPSULE HitCapsule
#include "script_echo_core.h"
typedef struct {
 int handle,owner,delay,move,match_element,airborne,damage,knockback,element,once;
 unsigned generation,identity[ECHO_HITS],contact_epoch,targets[ECHO_HITS];
 int source_attack,stale_instance;
 HitCapsule capsule[ECHO_HITS];
} ScriptEchoRule;
static struct {
 EchoState history;
 ScriptEchoRule rule[12][8];
 Fighter* fighter[12];
 unsigned generation[12],attack[12],identity[12][ECHO_HITS],epoch,captured[12];
 int next_handle,prepared;
} script_echo;
extern int Script_EchoInput(int field);
extern unsigned short plStale_IncrementAttackInstance(void);
extern int gm_8016B168(void);
extern int gm_8016B0D4(void);
static int script_echo_entity(Fighter* fp) {
 return fp && fp->player_id<6 ? fp->player_id*2+(fp->is_sub_fighter ? 1 : 0) : -1;
}
static int script_echo_bits(float f) {union {float f;int i;} u;u.f=f;return u.i;}
static float script_echo_float(int i) {union {float f;int i;} u;u.i=i;return u.f;}
static int script_echo_scale(int bits) {
 float f=script_echo_float(bits);return f==f && f>=0 && f<=4;
}
int ScriptGame_FighterHistoryDepth(void) {return ECHO_DEPTH;}
int ScriptGame_FighterHistoryRead(int entity,int age,int hit,int field) {
 const EchoRecord* r=echo_history(&script_echo.history,entity,age);HitCapsule* h;
 if(!r)return 0;
 if(hit<0) switch(field) {
 case 0:return 1;case 1:return r->generation;case 2:return r->motion;
 case 3:return r->move;case 4:return r->airborne;case 5:return script_echo_bits(r->frame);
 case 6:case 7:case 8:return script_echo_bits(r->position[field-6]);
 case 9:return script_echo_bits(r->facing);case 10:return r->hit_count;case 11:return r->attack;
 case 12:return r->attack_instance;
 default:return 0;
 }
 if(hit>=ECHO_HITS)return 0;h=(HitCapsule*)&r->hit[hit];
 switch(field) {
 case 0:return h->state!=HitCapsule_Disabled;case 1:return r->identity[hit];
 case 2:return script_echo_bits(h->damage);case 3:return h->kb_angle;
 case 4:return h->x24;case 5:return h->x2C;case 6:return h->x28;case 7:return h->element;
 case 8:return script_echo_bits(h->x4C.x);case 9:return script_echo_bits(h->x4C.y);
 case 10:return script_echo_bits(h->x4C.z);case 11:return script_echo_bits(h->x58.x);
 case 12:return script_echo_bits(h->x58.y);case 13:return script_echo_bits(h->x58.z);
 case 14:return script_echo_bits(h->scale);case 15:return r->hit_element[hit];
 case 16:return r->hit_move[hit];case 17:return r->hit_grounded[hit];case 18:return r->hit_source[hit];
 case 19:return r->hit_source_index[hit];default:return 0;
 }
}
int ScriptGame_EchoOwner(int entity) {
 int i;if(entity<0 || entity>=12)return 0;
 for(i=0;i<8;++i)if(script_echo.rule[entity][i].handle)return script_echo.rule[entity][i].owner;
 return 0;
}
int ScriptGame_EchoNextHandle(void) {return script_echo.next_handle==0x7fffffff ? 0 : script_echo.next_handle+1;}
int ScriptGame_EchoRead(int entity,int index,int field) {
 ScriptEchoRule* r;int i,n=0;if(entity<0 || entity>=12 || index<0)return 0;
 for(i=0;i<8;++i)if(script_echo.rule[entity][i].handle) {
 if(n++!=index)continue;r=&script_echo.rule[entity][i];
 switch(field) {case 0:return r->handle;case 1:return r->delay;case 2:return r->move;
 case 3:return r->match_element;case 4:return r->airborne;case 5:return r->damage;
 case 6:return r->knockback;case 7:return r->element;case 8:return r->once;case 9:return r->owner;}
 }return 0;
}
static int script_echo_valid(ScriptEchoRule* r) {
 return r->owner>0 && r->delay>=1 && r->delay<=60 && r->move>=-1 && r->move<=14 && r->move!=9 &&
 r->match_element>=-1 && r->match_element<=16 && r->airborne>=-1 && r->airborne<=1 &&
 (r->element==-1 || hr_ordinary(r->element)) && (r->once==0 || r->once==1) &&
 script_echo_scale(r->damage) && script_echo_scale(r->knockback);
}
int ScriptGame_EchoReplace(int entity,int owner,int count) {
 ScriptEchoRule next[8];int i,j,k,maxhandle=script_echo.next_handle;
 if(entity<0 || entity>=12 || owner<=0 || count<0 || count>8 ||
 (ScriptGame_EchoOwner(entity) && ScriptGame_EchoOwner(entity)!=owner))return 0;
 memset(next,0,sizeof next);
 for(i=0;i<count;++i) {
 ScriptEchoRule* r=&next[i];int b[9];for(j=0;j<9;++j)b[j]=Script_EchoInput(i*9+j);
 r->handle=b[0];r->owner=owner;r->delay=b[1];r->move=b[2];r->match_element=b[3];
 r->airborne=b[4];r->damage=b[5];r->knockback=b[6];r->element=b[7];r->once=b[8];
 if(r->handle<=0 || !script_echo_valid(r))return 0;
 for(j=0;j<i;++j)if(next[j].handle==r->handle)return 0;
 for(j=0;j<12;++j)for(k=0;k<8;++k)if(j!=entity && script_echo.rule[j][k].handle==r->handle)return 0;
 if(r->handle>maxhandle)maxhandle=r->handle;
 for(j=0;j<8;++j) {ScriptEchoRule* old=&script_echo.rule[entity][j];
 if(old->handle==r->handle && !memcmp(old,r,10*sizeof(int))) {*r=*old;break;}}
 }
 for(i=0;i<8;++i)for(j=0;j<ECHO_HITS;++j)ScriptGame_HitRuleForget(&script_echo.rule[entity][i].capsule[j]);
 memcpy(script_echo.rule[entity],next,sizeof next);script_echo.next_handle=maxhandle;
 return 1;
}
int ScriptGame_EchoAddWithHandle(int entity,int owner,int delay,int move,int match_element,int airborne,
 int damage,int knockback,int element,int once,int handle) {
 ScriptEchoRule r;int i,j,k;
 if(entity<0 || entity>=12 || (ScriptGame_EchoOwner(entity) && ScriptGame_EchoOwner(entity)!=owner))return 0;
 memset(&r,0,sizeof r);r.owner=owner;r.delay=delay;r.move=move;r.match_element=match_element;
 r.airborne=airborne;r.damage=damage;r.knockback=knockback;r.element=element;r.once=once;
 if(!script_echo_valid(&r))return 0;
 if(handle<0)return 0;
 if(handle)for(j=0;j<12;++j)for(k=0;k<8;++k)if(script_echo.rule[j][k].handle==handle)return 0;
 for(i=0;i<8;++i)if(!script_echo.rule[entity][i].handle) {
 if(!handle) {if(script_echo.next_handle==0x7fffffff)return 0;handle=++script_echo.next_handle;}
 if(handle>script_echo.next_handle)script_echo.next_handle=handle;
 r.handle=handle;script_echo.rule[entity][i]=r;return handle;
 }return 0;
}
int ScriptGame_EchoAdd(int entity,int owner,int delay,int move,int match_element,int airborne,
 int damage,int knockback,int element,int once) {
 return ScriptGame_EchoAddWithHandle(entity,owner,delay,move,match_element,airborne,damage,knockback,element,once,0);
}
int ScriptGame_EchoRemove(int handle,int owner) {
 int i,j,k;for(i=0;i<12;++i)for(j=0;j<8;++j) {ScriptEchoRule* r=&script_echo.rule[i][j];
 if(r->handle==handle && r->owner==owner) {
 for(k=0;k<ECHO_HITS;++k)ScriptGame_HitRuleForget(&r->capsule[k]);memset(r,0,sizeof *r);return 1;}}
 return 0;
}
void ScriptGame_EchoClear(int owner) {
 int i,j;for(i=0;i<12;++i)for(j=0;j<8;++j)if(script_echo.rule[i][j].handle &&
 (!owner || owner==script_echo.rule[i][j].owner))ScriptGame_EchoRemove(script_echo.rule[i][j].handle,script_echo.rule[i][j].owner);
}
void ScriptGame_EchoReset(void) {ScriptGame_EchoClear(0);memset(&script_echo,0,sizeof script_echo);}
void ScriptGame_EchoFrame(void) {
 int i,j;
 /* StageFrame runs at logic end. Frozen/all-disabled collision scenes still
  * contribute one history age, before advancing the epoch for the next logic. */
 if(!script_echo.prepared)ScriptGame_EchoPrepare();
 script_echo.prepared=0;++script_echo.epoch;
 for(i=0;i<12;++i)for(j=0;j<8;++j) {ScriptEchoRule* r=&script_echo.rule[i][j];
 if(r->owner && !Script_StageResourceOwnerAlive(r->owner))ScriptGame_EchoRemove(r->handle,r->owner);}
}
static Fighter* script_echo_live_owner(HSD_GObj* owner) {
 HSD_GObj* g;if(!owner)return NULL;
 for(g=HSD_GObjPLinkHead[HSD_GOBJ_PLINK_FIGHTER];g;g=g->next)if(g==owner) {
 Fighter* fp=GET_FIGHTER(g);return script_echo_entity(fp)>=0 && fp->motion_id>10 ? fp : NULL;
 }return NULL;
}
static Fighter* script_echo_body_actor(Fighter* fp) {
 HSD_GObj* g;
 if(fp->x1064_thrownHitbox.state!=HitCapsule_Disabled && script_echo_live_owner(fp->x1064_thrownHitbox.owner))return fp;
 /* Expose the thrower's owned body hit in the thrower's own history as well.
  * Collision then uses the true thrower/team/credit; no thrown-victim impersonation. */
 for(g=HSD_GObjPLinkHead[HSD_GOBJ_PLINK_FIGHTER];g;g=g->next) {
 Fighter* body=GET_FIGHTER(g);
 if(body->x1064_thrownHitbox.state!=HitCapsule_Disabled && body->x1064_thrownHitbox.owner==fp->gobj)return body;
 }return NULL;
}
void ScriptGame_EchoCapture(Fighter* fp) {
 int entity=script_echo_entity(fp),i,j;EchoRecord record;const EchoRecord* previous;
 if(entity<0 || (script_echo.fighter[entity]==fp && script_echo.captured[entity]==script_echo.epoch))return;
 script_echo.captured[entity]=script_echo.epoch;
 previous=echo_history(&script_echo.history,entity,0);
 if(script_echo.fighter[entity]!=fp || fp->motion_id<=10) {
 echo_entity_clear(&script_echo.history,entity);previous=0;
 for(i=0;i<8;++i) {ScriptEchoRule* rule=&script_echo.rule[entity][i];rule->generation=0;
 for(j=0;j<ECHO_HITS;++j) {ScriptGame_HitRuleForget(&rule->capsule[j]);memset(&rule->capsule[j],0,sizeof(HitCapsule));}}
 script_echo.fighter[entity]=fp;
 }
 if(fp->motion_id<=10) {
 int target_rule,target_owner,target_hit;
 for(target_owner=0;target_owner<12;++target_owner)for(target_rule=0;target_rule<8;++target_rule)
 for(target_hit=0;target_hit<ECHO_HITS;++target_hit)script_echo.rule[target_owner][target_rule].targets[target_hit]&=~(1u<<entity);
 return;
 }
 if(!previous || previous->motion!=fp->motion_id || fp->cur_anim_frame<previous->frame ||
 previous->attack_instance!=(int)fp->x206C_attack_instance ||
 script_echo.attack[entity]!=(unsigned)fp->x2068_attackID)++script_echo.generation[entity];
 script_echo.attack[entity]=fp->x2068_attackID;
 memset(&record,0,sizeof record);record.generation=script_echo.generation[entity];
 record.motion=fp->motion_id;record.move=script_hit_family_move(fp);record.airborne=fp->ground_or_air==GA_Air;
 record.attack=fp->x2068_attackID;record.attack_instance=fp->x206C_attack_instance;
 record.frame=fp->cur_anim_frame;record.position[0]=fp->cur_pos.x;record.position[1]=fp->cur_pos.y;
 record.position[2]=fp->cur_pos.z;record.facing=fp->facing_dir;
 for(i=0;i<ECHO_HITS;++i) {
 Fighter* actor=i==4 ? script_echo_body_actor(fp) : fp;
 Fighter* source=actor && i==4 ? script_echo_live_owner(actor->x1064_thrownHitbox.owner) : fp;
 HitCapsule* h=actor ? (i==4 ? &actor->x1064_thrownHitbox : &actor->x914[i]) : NULL;
 HRMeta* metadata;
 if(!h || !source || h->state==HitCapsule_Disabled)continue;
 record.hit_source[i]=script_echo_entity(source);record.hit_source_index[i]=i;
 metadata=script_hit_metadata(h,0);
 record.hit_move[i]=metadata ? metadata->move : script_hit_family_move(source);
 record.hit_grounded[i]=metadata ? metadata->grounded : source->ground_or_air==GA_Ground;
 record.hit_element[i]=metadata ? metadata->original : (int)h->element;
 if(!previous || previous->hit[i].state==HitCapsule_Disabled || previous->generation!=record.generation ||
 previous->hit[i].x4!=h->x4)++script_echo.identity[entity][i];
 record.identity[i]=script_echo.identity[entity][i];record.hit[i]=*h;++record.hit_count;
 if(!h->x43_b1)record.hit[i].scale*=actor->x34_scale.y;record.hit[i].x43_b1=1;
 /* History describes resolved geometry/properties, never live victim pointers. */
 memset(record.hit[i].victims_1,0,sizeof h->victims_1);memset(record.hit[i].victims_2,0,sizeof h->victims_2);
 record.hit[i].x44=record.hit[i].x45=0;record.hit[i].jobj=NULL;record.hit[i].owner=NULL;
 }
 echo_record(&script_echo.history,entity,&record);
 for(i=0;i<8;++i) {
 ScriptEchoRule* rule=&script_echo.rule[entity][i];const EchoRecord* old;
 if(!rule->handle)continue;old=echo_history(&script_echo.history,entity,rule->delay);
 if(old && old->generation!=rule->generation) {
 memset(rule->targets,0,sizeof rule->targets);rule->source_attack=old->attack;
 rule->stale_instance=plStale_IncrementAttackInstance();
 }
 for(j=0;j<ECHO_HITS;++j) {
 HitCapsule* h=&rule->capsule[j];HitVictim v1[12],v2[12];u8 c1=h->x44,c2=h->x45;
 int preserve=old && rule->generation==old->generation && (rule->once || rule->identity[j]==old->identity[j]);
 if(old && !rule->once && old->identity[j]!=rule->identity[j])rule->targets[j]=0;
 if(preserve){memcpy(v1,h->victims_1,sizeof v1);memcpy(v2,h->victims_2,sizeof v2);}
 ScriptGame_HitRuleForget(h);memset(h,0,sizeof *h);
 if(!old || (rule->move>=0 && rule->move<10 && rule->move!=old->move) ||
 (rule->move>=10 && old->motion!=ftCo_MS_AttackAirN+rule->move-10) ||
 (rule->airborne>=0 && rule->airborne!=old->airborne) || old->hit[j].state==HitCapsule_Disabled ||
 old->hit_source[j]!=entity ||
 (rule->match_element>=0 && rule->match_element!=(int)old->hit[j].element))continue;
 *h=old->hit[j];h->jobj=NULL;h->owner=NULL;
 if(preserve){memcpy(h->victims_1,v1,sizeof v1);memcpy(h->victims_2,v2,sizeof v2);h->x44=c1;h->x45=c2;}
 h->damage*=script_echo_float(rule->damage);h->unk_count=(unsigned)(h->unk_count*script_echo_float(rule->damage));
 if(h->damage>500)h->damage=500;if(h->unk_count>500)h->unk_count=500;
 h->x24=(unsigned)(h->x24*script_echo_float(rule->knockback));h->x28=(unsigned)(h->x28*script_echo_float(rule->knockback));
 h->x2C=(unsigned)(h->x2C*script_echo_float(rule->knockback));if(rule->element>=0)h->element=rule->element;
 if(h->x24>1000)h->x24=1000;if(h->x28>1000)h->x28=1000;if(h->x2C>1000)h->x2C=1000;
 /* Record metadata from the historical state, with no double application of outgoing rules. */
 ScriptGame_HitRuleEchoCreate(fp,h,old->hit_move[j],old->hit_grounded[j],old->hit_element[j]);
 rule->identity[j]=old->identity[j];
 }
 /* A new physical capsule in an existing hit group inherits accepted targets. */
 for(j=0;j<ECHO_HITS;++j)if(rule->capsule[j].state!=HitCapsule_Disabled) {
 int k;for(k=0;k<ECHO_HITS;++k)if((j==4)==(k==4) && rule->capsule[k].state!=HitCapsule_Disabled &&
 rule->capsule[j].x4==rule->capsule[k].x4)rule->targets[j]|=rule->targets[k];
 }
 rule->generation=old ? old->generation : 0;
 }
}
void ScriptGame_EchoPrepare(void) {
 HSD_GObj* g;if(script_echo.prepared)return;script_echo.prepared=1;
 for(g=HSD_GObjPLinkHead[HSD_GOBJ_PLINK_FIGHTER];g;g=g->next)ScriptGame_EchoCapture(GET_FIGHTER(g));
}
int ScriptGame_EchoCapsuleCount(Fighter* fp) {
 int entity=script_echo_entity(fp),i;if(entity<0)return 4;
 for(i=7;i>=0;--i)if(script_echo.rule[entity][i].handle)return 4+(i+1)*ECHO_HITS;
 return 4;
}
HitCapsule* ScriptGame_EchoCapsule(Fighter* fp,int index) {
 int entity=script_echo_entity(fp);if(!fp || index<0)return NULL;if(index<4)return &fp->x914[index];
 return entity>=0 && index<44 ? &script_echo.rule[entity][(index-4)/ECHO_HITS].capsule[(index-4)%ECHO_HITS] : NULL;
}
int ScriptGame_EchoIsCapsule(HitCapsule* hit) {
 int i,j,k;
 /* Game pointers and the fixture ABI are 32-bit. Ordinary retail capsules
  * avoid scanning the echo pool on every group comparison or contact. */
 if((unsigned)hit-(unsigned)&script_echo.rule[0][0]>=sizeof script_echo.rule)return 0;
 for(i=0;i<12;++i)for(j=0;j<8;++j)for(k=0;k<ECHO_HITS;++k)
 if(hit==&script_echo.rule[i][j].capsule[k])return script_echo.rule[i][j].handle;
 return 0;
}
static int script_echo_body_kind(HitCapsule* hit) {
 int i,j;if(!ScriptGame_EchoIsCapsule(hit))return 0;
 for(i=0;i<12;++i)for(j=0;j<8;++j)if(hit==&script_echo.rule[i][j].capsule[4])return 1;
 return 0;
}
int ScriptGame_EchoHitIndex(Fighter* fp,HitCapsule* hit) {
 int i;for(i=0;i<4;++i)if(hit==&fp->x914[i])return i;
 return -1;
}
void ScriptGame_EchoConnected(HitCapsule* hit,Fighter* victim) {
 int i,j,k;if(!victim || !ScriptGame_EchoIsCapsule(hit))return;
 for(i=0;i<12;++i)for(j=0;j<8;++j) {ScriptEchoRule* r=&script_echo.rule[i][j];
 for(k=0;k<ECHO_HITS;++k)if(hit==&r->capsule[k]) {
 int target=script_echo_entity(victim);if(target>=0)r->targets[k]|=1u<<target;
 r->contact_epoch=script_echo.epoch;
 {int n;for(n=0;n<ECHO_HITS;++n)if(r->once || ((n==4)==(k==4) && r->capsule[n].x4==hit->x4)) {
 lbColl_80008688(&r->capsule[n],0,victim);if(target>=0)r->targets[n]|=1u<<target;}}return;}}
}
int ScriptGame_EchoSameGroup(HitCapsule* a,HitCapsule* b) {
 return a->x4==b->x4 && ScriptGame_EchoIsCapsule(a)==ScriptGame_EchoIsCapsule(b) &&
 script_echo_body_kind(a)==script_echo_body_kind(b);
}
int ScriptGame_EchoTargetAllowed(Fighter* source,Fighter* target) {
 if(!source || !target || source==target || source->player_id==target->player_id)return 0;
 return !gm_8016B168() || gm_8016B0D4() || target->x2225_b4 || source->team!=target->team;
}
int ScriptGame_EchoBlocked(HitCapsule* hit,Fighter* victim) {
 int i,j,k,target=script_echo_entity(victim);if(target<0 || !ScriptGame_EchoIsCapsule(hit))return 0;
 for(i=0;i<12;++i)for(j=0;j<8;++j)for(k=0;k<ECHO_HITS;++k)if(hit==&script_echo.rule[i][j].capsule[k])
 return (script_echo.rule[i][j].targets[k]>>target)&1;
 return 0;
}
int ScriptGame_EchoState(int entity,int handle,int field) {
 int i,j;if(entity<0 || entity>=12)return 0;
 for(i=0;i<8;++i) {ScriptEchoRule* r=&script_echo.rule[entity][i];if(r->handle!=handle)continue;
 if(field==2)return r->contact_epoch && script_echo.epoch-r->contact_epoch<4 ? 4-(script_echo.epoch-r->contact_epoch) : 0;
 for(j=0;j<ECHO_HITS;++j)if(r->capsule[j].state!=HitCapsule_Disabled)return field==1 ? (int)r->capsule[j].element : 1;
 return 0;}return 0;
}
int ScriptGame_EchoReportContext(Fighter* attacker,Fighter* victim,HitCapsule* hit) {
 int entity=script_echo_entity(attacker),i,j;union {float f;int i;} a,v;
 extern void Script_HitContext(int,int,int,int,int,int,int,int,int);
 if(entity<0 || !victim)return 0;
 for(i=0;i<8;++i)for(j=0;j<ECHO_HITS;++j)if(hit==&script_echo.rule[entity][i].capsule[j]) {
 ScriptEchoRule* rule=&script_echo.rule[entity][i];const EchoRecord* r=echo_history(&script_echo.history,entity,rule->delay);
 if(!r)return 0;a.f=attacker->dmg.x1830_percent;v.f=victim->dmg.x1830_percent;
 Script_HitContext(attacker->player_id,victim->player_id,hit->element,r->motion,r->hit_move[j],r->hit_grounded[j],
 victim->ground_or_air==GA_Ground,a.i,v.i);return 1;}
 return 0;
}
int ScriptGame_EchoCredit(Fighter* attacker,Fighter* victim,HitCapsule* hit,float damage) {
 int entity=script_echo_entity(attacker),i,j;
 extern void ftColl_8007891C(HSD_GObj*,HSD_GObj*,float);
 if(entity<0 || !victim || !ScriptGame_EchoIsCapsule(hit))return 0;
 for(i=0;i<8;++i)for(j=0;j<ECHO_HITS;++j)if(hit==&script_echo.rule[entity][i].capsule[j]) {
 ScriptEchoRule* rule=&script_echo.rule[entity][i];int attack=attacker->x2068_attackID;
 unsigned instance=attacker->x206C_attack_instance;
 attacker->x2068_attackID=rule->source_attack;attacker->x206C_attack_instance=rule->stale_instance;
 ftColl_8007891C(attacker->gobj,victim->gobj,damage);
 attacker->x2068_attackID=attack;attacker->x206C_attack_instance=instance;return 1;
 }return 0;
}
int ScriptGame_EchoSourceX(HitCapsule* hit,int fallback) {
 int i,j,k;if(!ScriptGame_EchoIsCapsule(hit))return fallback;
 for(i=0;i<12;++i)for(j=0;j<8;++j)for(k=0;k<ECHO_HITS;++k)
 if(hit==&script_echo.rule[i][j].capsule[k]) {
 const EchoRecord* r=echo_history(&script_echo.history,i,script_echo.rule[i][j].delay);
 return r ? script_echo_bits(r->position[0]) : fallback;
 }return fallback;
}
