/* Included by script_game.c: all simulation state is snapshotted game BSS.
 * Each of six player slots has independent primary/sub-fighter state. Capsule
 * metadata never changes HitCapsule's retail/DAT ABI. */
#include "script_hit_rules_core.h"
#define HR_META_MAX 1024
typedef struct { int owner,status,count; HRRule rules[HR_MAX]; } HRPort;
typedef struct {
 HitCapsule* hit; int owner,port,move,grounded,original,stun,count,ids[HR_MAX];
 float damage_scale,original_damage; unsigned original_growth,original_base,original_count; int original_shield;
} HRMeta;
static HRPort script_hit_ports[12];
/* Pending percent is separate from retail percentTemp: all same-frame launch
 * calculations see the original hit contributions. These bytes live in game BSS. */
static float script_hit_percent_delta[12];
static HRMeta script_hit_meta[HR_META_MAX];
static int script_hit_next_handle;
static int script_hit_capacity_warned;
static int script_hit_meta_count;
extern int Script_HitRulesInput(int at);
static int script_hit_entity(Fighter* f) {return f->player_id*2+(f->is_sub_fighter ? 1 : 0); }
static HRMeta* script_hit_metadata(HitCapsule* h,int create) {
 unsigned start=((unsigned)h>>2)%HR_META_MAX; int i;
 if(!create && !script_hit_meta_count)return NULL;
 for(i=0;i<HR_META_MAX;++i) {
  HRMeta* m=&script_hit_meta[(start+i)%HR_META_MAX];
  if(m->hit==h) return m;
 }
 if(create) for(i=0;i<HR_META_MAX;++i) {
  HRMeta* m=&script_hit_meta[(start+i)%HR_META_MAX];
  if(!m->hit) { memset(m,0,sizeof *m); m->hit=h;++script_hit_meta_count; return m; }
 }
 return NULL;
}
int ScriptGame_HitRulesOwner(int slot) { return slot>=0 && slot<12 ? script_hit_ports[slot].owner : -1; }
int ScriptGame_HitRulesCount(int slot) { return slot>=0 && slot<12 ? script_hit_ports[slot].count : 0; }
int ScriptGame_FighterStatus(int slot) { return slot>=0 && slot<12 ? script_hit_ports[slot].status : 0; }
static int script_hr_bits(float f) { union {float f;int i;} u;u.f=f;return u.i; }
static float script_hr_float(int b) { union {float f;int i;} u;u.i=b;return u.f; }
int ScriptGame_HitRuleRead(int slot,int at,int field) {
 HRRule* r;
 if(slot<0 || slot>=12 || at<0 || at>=script_hit_ports[slot].count) return 0;
 r=&script_hit_ports[slot].rules[at];
 switch(field) {
 case 0:return r->id;case 1:return r->move;case 2:return r->grounded;case 3:return r->original_element;
 case 4:return r->status_bits;case 5:return r->element;case 6:return script_hr_bits(r->damage);
 case 7:return script_hr_bits(r->growth);case 8:return script_hr_bits(r->base);case 9:return r->shield;
 case 10:return r->stun;case 11:return script_hr_bits(r->knockback_taken);case 12:return r->handle;case 13:return r->incoming;case 14:return script_hr_bits(r->percent_damage);case 15:return script_hr_bits(r->launch);
 } return 0;
}
int ScriptGame_HitRulesReplace(int slot,int owner,int count,int status) {
 HRPort next; int i,j,next_handle=script_hit_next_handle;
 if(slot<0 || slot>=12 || owner<=0 || count<0 || count>HR_MAX || status<0 ||
   (script_hit_ports[slot].owner && script_hit_ports[slot].owner!=owner)) return 0;
 memset(&next,0,sizeof next); next.owner=(count || status) ? owner : 0; next.count=count;next.status=status;
 for(i=0;i<count;++i) {
  HRRule* r=&next.rules[i]; int b[HR_FIELDS];
  for(j=0;j<HR_FIELDS;++j) b[j]=Script_HitRulesInput(i*HR_FIELDS+j);
  r->id=b[0];r->move=b[1];r->grounded=b[2];r->original_element=b[3];r->status_bits=b[4];r->element=b[5];
  r->damage=script_hr_float(b[6]);r->growth=script_hr_float(b[7]);r->base=script_hr_float(b[8]);
  r->shield=b[9];r->stun=b[10];r->knockback_taken=script_hr_float(b[11]);r->handle=b[12];r->incoming=b[13];r->percent_damage=script_hr_float(b[14]);r->launch=script_hr_float(b[15]);
  if(!hr_ratio_valid(r->damage,.1f,4) || !hr_ratio_valid(r->growth,.1f,4) ||
     !hr_ratio_valid(r->base,.1f,4) || !hr_ratio_valid(r->knockback_taken,.1f,4) ||
     !hr_ratio_valid(r->percent_damage,-1e9f,1e9f) || !hr_ratio_valid(r->launch,-1e9f,1e9f))return 0;
  if(!r->handle) { if(next_handle==0x7fffffff) return 0; r->handle=++next_handle; }
 }
 script_hit_ports[slot]=next;script_hit_next_handle=next_handle; return 1;
}
void ScriptGame_HitRulesRelease(int owner) {
 int i;
 for(i=0;i<12;++i) if(!owner || script_hit_ports[i].owner==owner) memset(&script_hit_ports[i],0,sizeof script_hit_ports[i]);
 for(i=0;i<HR_META_MAX;++i) {
  HRMeta* m=&script_hit_meta[i];
  if(!owner) memset(m,0,sizeof *m);
  else if(m->owner==owner) {m->owner=m->count=m->stun=0;m->damage_scale=1;memset(m->ids,0,sizeof m->ids);}
 }
 if(!owner){memset(script_hit_percent_delta,0,sizeof script_hit_percent_delta);script_hit_capacity_warned=0;script_hit_meta_count=0;}
}
void ScriptGame_HitRuleForget(HitCapsule* hit) {
 HRMeta* m=script_hit_metadata(hit,0); if(m){memset(m,0,sizeof *m);--script_hit_meta_count;}
}
void ScriptGame_HitRuleSpecial(HitCapsule* hit) {
 HRMeta* m=script_hit_metadata(hit,0);
 if(m) { hit->damage=m->original_damage;hit->unk_count=m->original_count;hit->x24=m->original_growth;hit->x2C=m->original_base;hit->x34=m->original_shield;ScriptGame_HitRuleForget(hit); }
}
void ScriptGame_HitRuleCreate(Fighter* owner,HitCapsule* hit,int item) {
 HRMeta* m; HRResult v; HRPort* p; int i;
 ScriptGame_HitRuleForget(hit);
 if(!owner || owner->player_id>=6) return;
 for(i=0;i<12;++i)if(script_hit_ports[i].count)break;
 if(i==12)return;
 if(!item && owner->x1064_thrownHitbox.owner) owner=GET_FIGHTER(owner->x1064_thrownHitbox.owner);
 if(!owner || owner->player_id>=6)return;
 p=&script_hit_ports[script_hit_entity(owner)];
 if(!hr_ordinary(hit->element)) return;
 m=script_hit_metadata(hit,1); if(!m) { if(!script_hit_capacity_warned){script_hit_capacity_warned=1;OSReport("hit rules: capsule metadata capacity (1024) exhausted; creation unchanged\n");}return; }
 m->owner=p->owner;m->port=script_hit_entity(owner);m->move=item ? 9 : script_hit_family_move(owner);m->grounded=owner->ground_or_air==GA_Ground;m->original=hit->element;
 v=hr_evaluate(p->rules,p->count,m->move,m->grounded,m->original,0,0);
 m->original_count=hit->unk_count;m->original_damage=hit->damage;m->original_growth=hit->x24;m->original_base=hit->x2C;m->original_shield=hit->x34;
 m->damage_scale=v.damage;m->stun=v.stun;m->count=v.count;for(i=0;i<v.count;++i)m->ids[i]=v.ids[i];
 hit->element=v.element;if(v.damage!=1) {hit->damage*=v.damage;if(hit->damage>500)hit->damage=500;}
 if(v.damage!=1)hit->unk_count=(unsigned)(hit->unk_count*v.damage);
 if(v.growth!=1) {hit->x24=(unsigned)(hit->x24*v.growth);if(hit->x24>1000)hit->x24=1000;}
 if(v.base!=1) {hit->x2C=(unsigned)(hit->x2C*v.base);if(hit->x2C>1000)hit->x2C=1000;}
 if(v.shield) {hit->x34+=v.shield;if(hit->x34>100)hit->x34=100;if(hit->x34< -100)hit->x34=-100;}
}
void ScriptGame_HitRuleItemCreate(Item* item,HitCapsule* hit) {
 extern int ftLib_80086960(HSD_GObj*);
 ScriptGame_HitRuleCreate(item->owner && ftLib_80086960(item->owner) ? GET_FIGHTER(item->owner) : NULL,hit,1);
}
static HRResult script_hit_family(HitCapsule*,Fighter*,int);
/* This reads per-victim status, and returns a scalar. Never mutate shared hit. */
float ScriptGame_HitRuleContact(HitCapsule* hit,Fighter* victim,float value,int knockback) {
 HRMeta* m=script_hit_metadata(hit,0);HRResult v,w;HRPort* defender;
 if(!victim || victim->player_id>=6 || !hr_ordinary(hit->element))return value;
 v=hr_evaluate(m ? script_hit_ports[m->port].rules : NULL,m ? script_hit_ports[m->port].count : 0,
     m ? m->move : 0,m ? m->grounded : -1,m ? m->original : hit->element,script_hit_ports[script_hit_entity(victim)].status,1);
 defender=&script_hit_ports[script_hit_entity(victim)];
 w=hr_evaluate(defender->rules,defender->count,m ? m->move : 0,m ? m->grounded : -1,m ? m->original : hit->element,defender->status,2);
 value=hr_safe_float((double)value*(knockback ? hr_cap(v.knockback_taken*w.knockback_taken) : hr_cap(v.damage*w.damage)));
 if(knockback) {
  HRResult outgoing=script_hit_family(hit,victim,0),incoming=script_hit_family(hit,victim,1);
  value=hr_safe_float((double)value*hr_family_cap(outgoing.launch,4)*hr_family_cap(incoming.launch,4));
 }
 return !knockback && hr_cap(v.damage*w.damage)!=1 && value>500 ? 500 : value;
}
/* New fields are evaluated at the accepted contact using current rule tables.
 * Metadata supplies source identity and original move/element, not a mutable hit. */
static HRResult script_hit_family(HitCapsule* hit,Fighter* victim,int incoming) {
 HRMeta* m=script_hit_metadata(hit,0);int entity=script_hit_entity(victim);
 HRPort* p=incoming ? &script_hit_ports[entity] : m ? &script_hit_ports[m->port] : NULL;
 return hr_evaluate(p ? p->rules : NULL,p ? p->count : 0,m ? m->move : 0,
   m ? m->grounded : -1,m ? m->original : hit->element,script_hit_ports[entity].status,incoming ? 4 : 3);
}
void ScriptGame_HitRulePercentQueue(HitCapsule* hit,Fighter* victim,float damage) {
 HRResult outgoing,incoming;float scale;double delta;int entity;
 if(!victim || victim->player_id>=6 || !hr_ordinary(hit->element) || !hr_finite(damage) || damage<=0)return;
 outgoing=script_hit_family(hit,victim,0);incoming=script_hit_family(hit,victim,1);
 scale=hr_percent_cap(outgoing.percent_damage,0)*hr_percent_cap(incoming.percent_damage,1);
 entity=script_hit_entity(victim);
 /* A legal 64*64 product with large finite damage can overflow float. Keep
  * the snapshotted delta finite; apply display safety only at the final commit. */
 delta=(hr_finite(script_hit_percent_delta[entity]) ? (double)script_hit_percent_delta[entity] : 0)+(double)damage*((double)scale-1);
 script_hit_percent_delta[entity]=hr_safe_float(delta);
}
float ScriptGame_HitRulePercentCommit(Fighter* victim,float damage) {
 int entity;float delta;double result;
 if(!victim || victim->player_id>=6)return damage;
 entity=script_hit_entity(victim);delta=script_hit_percent_delta[entity];script_hit_percent_delta[entity]=0;
 if(!hr_finite(damage))damage=damage>0 ? 999 : 0;
 if(!hr_finite(delta))delta=0;
 result=(double)damage+delta;return result<0 ? 0 : result>999 ? 999 : (float)result;
}
void ScriptGame_HitRulePercentClear(Fighter* victim) {
 if(victim && victim->player_id<6)script_hit_percent_delta[script_hit_entity(victim)]=0;
}
void ScriptGame_HitRuleWon(HitCapsule* hit,Fighter* victim) {
 HRMeta* m=script_hit_metadata(hit,0);
 extern void Geno_GiveStunBonus(Fighter*,int);
 if(m && hr_ordinary(hit->element)) Geno_GiveStunBonus(victim,m->stun);
}
void ScriptGame_HitRuleContext(HitCapsule* hit,Fighter* victim) {
 HRMeta* m=script_hit_metadata(hit,0); HRResult v,w,x,y;HRPort* defender;int i;
 extern void Script_HitRuleContext(int,int,int,int,int);
 if(!victim || victim->player_id>=6 || !hr_ordinary(hit->element))return;
 v=hr_evaluate(m ? script_hit_ports[m->port].rules : NULL,m ? script_hit_ports[m->port].count : 0,
     m ? m->move : 0,m ? m->grounded : -1,m ? m->original : hit->element,script_hit_ports[script_hit_entity(victim)].status,1);
 defender=&script_hit_ports[script_hit_entity(victim)];
 w=hr_evaluate(defender->rules,defender->count,m ? m->move : 0,m ? m->grounded : -1,m ? m->original : hit->element,defender->status,2);
 x=script_hit_family(hit,victim,0);y=script_hit_family(hit,victim,1);
 for(i=0;m && i<m->count;++i) Script_HitRuleContext(victim->player_id,m->ids[i],m->move,m->grounded,m->owner);
 for(i=0;i<v.count;++i) Script_HitRuleContext(victim->player_id,v.ids[i],m->move,m->grounded,script_hit_ports[m->port].owner);
 for(i=0;i<w.count;++i) Script_HitRuleContext(victim->player_id,w.ids[i],m ? m->move : 0,m ? m->grounded : -1,defender->owner);
 for(i=0;i<x.count;++i) Script_HitRuleContext(victim->player_id,x.ids[i],m ? m->move : 0,m ? m->grounded : -1,m ? script_hit_ports[m->port].owner : 0);
 for(i=0;i<y.count;++i) Script_HitRuleContext(victim->player_id,y.ids[i],m ? m->move : 0,m ? m->grounded : -1,defender->owner);
}

/* Called after a retail damage-only setter (charge, powershield and script edits).
 * Creation callers forget their previous incarnation before that setter. */
void ScriptGame_HitRuleDamage(HitCapsule* hit) {
 HRMeta* m=script_hit_metadata(hit,0);
 if(!m)return;
 m->original_damage=hit->damage;m->original_count=hit->unk_count;
 if(!hr_ordinary(hit->element)) {ScriptGame_HitRuleForget(hit);return;}
 if(m->damage_scale!=1){hit->damage*=m->damage_scale;if(hit->damage>500)hit->damage=500;}
 hit->unk_count=(unsigned)(hit->unk_count*m->damage_scale);
}
void ScriptGame_HitRuleRetire(void* data,unsigned size) {
 int i;unsigned start=(unsigned)data;
 for(i=0;i<HR_META_MAX;++i) {
  unsigned p=(unsigned)script_hit_meta[i].hit;
  if(p>=start && p-start<size){memset(&script_hit_meta[i],0,sizeof script_hit_meta[i]);--script_hit_meta_count;}
 }
}

float ScriptGame_HitRuleBaseDamage(HitCapsule* hit) {
 HRMeta* m=script_hit_metadata(hit,0);return m ? m->original_damage : hit->damage;
}
void ScriptGame_HitRuleDamageOnly(HitCapsule* hit) {
 HRMeta* m=script_hit_metadata(hit,0);if(!m)return;
 m->original_damage=hit->damage;if(m->damage_scale!=1){hit->damage*=m->damage_scale;if(hit->damage>500)hit->damage=500;}
}

/* Shared queued trace for throw/escape paths which bypass the collision log. */
void ScriptGame_HitRuleReport(Fighter* attacker,Fighter* victim,HitCapsule* hit,float damage) {
 extern void Script_GameEvent(int,int,int,int,int);union {float f;int i;} bits;
 bits.f=damage;
 Script_GameEvent(2,attacker->player_id,victim->player_id,255|(attacker->is_sub_fighter?256:0)|(victim->is_sub_fighter?512:0),bits.i);
 ScriptGame_ReportHitContext(attacker,victim,hit->element,0);
 ScriptGame_HitRuleContext(hit,victim);
}
