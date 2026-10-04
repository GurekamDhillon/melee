/* Scalar-only API and journal encoding; authoritative state is game BSS. */
#define GS_HR_MAX 8
#define GS_HR_FIELDS 14
extern int gw_ScriptGame_HitRulesOwner(int);
extern int gw_ScriptGame_HitRulesCount(int);
extern int gw_ScriptGame_FighterStatus(int);
extern int gw_ScriptGame_HitRuleRead(int,int,int);
extern int gw_ScriptGame_HitRulesReplace(int,int,int,int);
extern void gw_ScriptGame_HitRulesRelease(int);
typedef struct { int count,status,rules[GS_HR_MAX][GS_HR_FIELDS]; } GsHitRules;
static const int* gs_hit_rules_input;
int gw_Script_HitRulesInput(int at) { return gs_hit_rules_input && at>=0 && at<GS_HR_MAX*GS_HR_FIELDS ? gs_hit_rules_input[at] : 0; }
static const char* const gs_hr_moves[]={"unknown","jab","dash_attack","tilt","smash","aerial","grab","throw","special","projectile"};
static const char* gs_hr_element_name(int e) { return e==0?"normal":e==1?"fire":e==2?"electric":e==5?"ice":e==13?"darkness":"any"; }
static int gs_hr_element(lua_State* L,int at) {
 const char* n=luaL_checkstring(L,at);int e;
 if(!strcmp(n,"any"))return -1;
 for(e=0;e<14;++e)if((e==0 || e==1 || e==2 || e==5 || e==13) && !strcmp(n,gs_hr_element_name(e)))return e;
 return luaL_error(L,"hit rules allow only normal/fire/electric/ice/darkness elements");
}
static void gs_hr_keys(lua_State* L,int at,const char* const* keys,int n) {
 int j;at=lua_absindex(L,at);lua_pushnil(L);
 while(lua_next(L,at)) {const char* k=lua_type(L,-2)==LUA_TSTRING?lua_tostring(L,-2):NULL;
  for(j=0;j<n;++j)if(k && !strcmp(k,keys[j]))break;
  if(j==n)luaL_error(L,"hit rule unknown field '%s' (fraction/split elements unsupported)",k?k:"non-string");lua_pop(L,1);
 }
}
static int gs_hr_int(lua_State* L,int at,const char* key,int fallback,int lo,int hi) {
 lua_Integer n;lua_getfield(L,at,key);n=lua_isnil(L,-1)?fallback:luaL_checkinteger(L,-1);lua_pop(L,1);
 if(n<lo || n>hi)luaL_error(L,"hit rule %s out of range",key);return (int)n;
}
static void gs_hr_rule(lua_State* L,int at,int* b) {
 static const char* const top[]={"id","match","change"};
 static const char* const match[]={"move","grounded","element","status_bits","incoming"};
 static const char* const change[]={"element","damage","knockback_growth","knockback_base","shield_damage","hitstun","knockback_taken"};
 static const char* const mult[]={"damage","knockback_growth","knockback_base","knockback_taken"};
 int j;at=lua_absindex(L,at);luaL_checktype(L,at,LUA_TTABLE);gs_hr_keys(L,at,top,3);memset(b,0,GS_HR_FIELDS*sizeof *b);
 b[0]=gs_hr_int(L,at,"id",0,0,0x7fffffff);b[1]=b[2]=b[3]=b[5]=-1;b[6]=b[7]=b[8]=b[11]=gs_fbits(1);
 lua_getfield(L,at,"match");if(!lua_isnil(L,-1)) {
  int m=lua_absindex(L,-1);luaL_checktype(L,m,LUA_TTABLE);gs_hr_keys(L,m,match,5);
  lua_getfield(L,m,"move");if(!lua_isnil(L,-1)) {const char* s=luaL_checkstring(L,-1);if(strcmp(s,"any")){for(j=0;j<10;++j)if(!strcmp(s,gs_hr_moves[j]))break;if(j==10)luaL_error(L,"unknown hit rule move tag");b[1]=j;}}lua_pop(L,1);
  lua_getfield(L,m,"grounded");if(!lua_isnil(L,-1)){luaL_checktype(L,-1,LUA_TBOOLEAN);b[2]=lua_toboolean(L,-1);}lua_pop(L,1);
  lua_getfield(L,m,"element");if(!lua_isnil(L,-1))b[3]=gs_hr_element(L,-1);lua_pop(L,1);
  b[4]=gs_hr_int(L,m,"status_bits",0,0,0x7fffffff);
  lua_getfield(L,m,"incoming");if(!lua_isnil(L,-1)){luaL_checktype(L,-1,LUA_TBOOLEAN);b[13]=lua_toboolean(L,-1);}lua_pop(L,1);
 }lua_pop(L,1);
 lua_getfield(L,at,"change");luaL_checktype(L,-1,LUA_TTABLE);{
  int c=lua_absindex(L,-1);gs_hr_keys(L,c,change,7);
  lua_getfield(L,c,"element");if(!lua_isnil(L,-1))b[5]=gs_hr_element(L,-1);lua_pop(L,1);
  for(j=0;j<4;++j){double v;lua_getfield(L,c,mult[j]);v=lua_isnil(L,-1)?1:luaL_checknumber(L,-1);lua_pop(L,1);if(!isfinite(v)||v<0.1||v>4)luaL_error(L,"hit rule multiplier %s must be 0.1..4",mult[j]);b[j==3?11:6+j]=gs_fbits((float)v);}
  b[9]=gs_hr_int(L,c,"shield_damage",0,-100,100);b[10]=gs_hr_int(L,c,"hitstun",0,0,120);
 }lua_pop(L,1);
 if((b[4]||b[13]) && (b[5]>=0 || b[7]!=gs_fbits(1) || b[8]!=gs_fbits(1) || b[9] || b[10]))luaL_error(L,"contact rule permits only damage and knockback_taken changes");
 if(!b[4]&&!b[13]&&b[11]!=gs_fbits(1))luaL_error(L,"knockback_taken requires status_bits or incoming");
}
static void gs_hr_read(int port,GsHitRules* r) {int i,j;memset(r,0,sizeof *r);r->count=gw_ScriptGame_HitRulesCount(port);r->status=gw_ScriptGame_FighterStatus(port);for(i=0;i<r->count;++i)for(j=0;j<GS_HR_FIELDS;++j)r->rules[i][j]=gw_ScriptGame_HitRuleRead(port,i,j);}
static void gs_hr_owner(lua_State* L,int port) {int owner=gw_ScriptGame_HitRulesOwner(port);if(owner && owner!=gs.cur+1)luaL_error(L,"hit rules/status owned by another script");}
static int gs_hr_apply(int port,int owner,const GsHitRules* r) {
 int ok,i,j;GsHitRules next=*r;
 for(i=0;i<next.count;++i)if(!next.rules[i][12] && i<gw_ScriptGame_HitRulesCount(port)) {
  for(j=0;j<GS_HR_FIELDS;++j)if(j!=12 && next.rules[i][j]!=gw_ScriptGame_HitRuleRead(port,i,j))break;
  if(j==GS_HR_FIELDS)next.rules[i][12]=gw_ScriptGame_HitRuleRead(port,i,12);
 }
 gs_hit_rules_input=&next.rules[0][0];ok=gw_ScriptGame_HitRulesReplace(port,owner,next.count,next.status);gs_hit_rules_input=NULL;return ok;
}
static void gs_hr_parse(lua_State* L,int at,GsHitRules* r) {
 int i;size_t n;at=lua_absindex(L,at);memset(r,0,sizeof *r);r->status=gs_hr_int(L,at,"status_bits",0,0,0x7fffffff);
 lua_getfield(L,at,"rules");luaL_checktype(L,-1,LUA_TTABLE);n=lua_rawlen(L,-1);if(n>GS_HR_MAX)luaL_error(L,"hit rules capacity is 8 per fighter entity");r->count=(int)n;
 for(i=0;i<r->count;++i){lua_rawgeti(L,-1,i+1);gs_hr_rule(L,-1,r->rules[i]);lua_pop(L,1);}lua_pop(L,1);
}

/* Omitted write selector targets both entities; omitted reads show primary.
 * Explicit false selects primary, true selects the independent partner. */
static int gs_hr_selector(lua_State* L,int at) {
 if(lua_isnoneornil(L,at))return -1;
 luaL_checktype(L,at,LUA_TBOOLEAN);return lua_toboolean(L,at);
}
static void gs_hr_targets_owner(lua_State* L,int port,int sub) {
 if(sub<=0)gs_hr_owner(L,port*2);
 if(sub!=0)gs_hr_owner(L,port*2+1);
}
static int gs_hr_apply_targets(int port,int sub,int owner,const GsHitRules* r) {
 int ok=1,i;GsHitRules partner=*r;
 if(sub<=0)ok=gs_hr_apply(port*2,owner,r);
 if(sub<0 && ok)for(i=0;i<r->count;++i)partner.rules[i][12]=gw_ScriptGame_HitRuleRead(port*2,i,12);
 if(sub!=0 && ok)ok=gs_hr_apply(port*2+1,owner,&partner);return ok;
}
static int l_hit_rule_add(lua_State* L) {
 int port=gs_slot_arg(L,1),sub=gs_hr_selector(L,3),entity,first=port*2+(sub>0),last=port*2+(sub!=0),handle=0;
 int parsed[GS_HR_FIELDS];GsHitRules draft[2];
 gs_require_gameplay(L,"hit_rule_add");gs_hr_targets_owner(L,port,sub);gs_hr_rule(L,2,parsed);
 for(entity=first;entity<=last;++entity) {
  GsHitRules* r=&draft[entity-first];gs_hr_read(entity,r);
  if(r->count==GS_HR_MAX)return luaL_error(L,"hit rules capacity is 8 per fighter entity");
  memcpy(r->rules[r->count++],parsed,sizeof parsed);
 }
 gs_rw_branch();
 for(entity=first;entity<=last;++entity) {
  GsHitRules* r=&draft[entity-first];if(handle)r->rules[r->count-1][12]=handle;
  if(!gs_hr_apply(entity,gs.cur+1,r))return luaL_error(L,"hit rule add refused");
  if(!handle)handle=gw_ScriptGame_HitRuleRead(entity,r->count-1,12);
 }
 lua_pushinteger(L,handle);return 1;
}
static int l_hit_rule_remove(lua_State* L) {
 int handle,p,i,found=0;lua_Integer raw=luaL_checkinteger(L,1);GsHitRules r;gs_require_gameplay(L,"hit_rule_remove");
 if(raw<1 || raw>0x7fffffff)return luaL_error(L,"invalid hit rule handle");handle=(int)raw;
 /* Ownership is preflighted across both copies before any edit. */
 for(p=0;p<12;++p){gs_hr_read(p,&r);for(i=0;i<r.count;++i)if(r.rules[i][12]==handle){gs_hr_owner(L,p);found=1;}}
 if(found)gs_rw_branch();
 for(p=0;p<12;++p){gs_hr_read(p,&r);for(i=0;i<r.count;++i)if(r.rules[i][12]==handle){memmove(r.rules[i],r.rules[i+1],(r.count-i-1)*sizeof r.rules[0]);memset(r.rules[--r.count],0,sizeof r.rules[0]);gs_hr_apply(p,gs.cur+1,&r);break;}}
 lua_pushboolean(L,found);return 1;
}
static int l_hit_rules_clear(lua_State* L) {
 int port=gs_slot_arg(L,1),sub=gs_hr_selector(L,2),e,first=port*2+(sub>0),last=port*2+(sub!=0);GsHitRules r;
 gs_require_gameplay(L,"hit_rules_clear");gs_hr_targets_owner(L,port,sub);gs_rw_branch();
 for(e=first;e<=last;++e){gs_hr_read(e,&r);r.count=0;memset(r.rules,0,sizeof r.rules);gs_hr_apply(e,gs.cur+1,&r);}
 lua_pushboolean(L,1);return 1;
}
static int l_fighter_status(lua_State* L) {
 int port=gs_slot_arg(L,1),sub=gs_hr_selector(L,3),e,first=port*2+(sub>0),last=port*2+(sub!=0);GsHitRules r;
 if(!lua_isnoneornil(L,2)){
  lua_Integer n;gs_require_gameplay(L,"fighter_status");gs_hr_targets_owner(L,port,sub);n=luaL_checkinteger(L,2);
  if(n<0||n>0x7fffffff)return luaL_error(L,"fighter_status bits must be 0..2147483647");gs_rw_branch();
  for(e=first;e<=last;++e){gs_hr_read(e,&r);r.status=(int)n;if(!gs_hr_apply(e,gs.cur+1,&r))return luaL_error(L,"fighter_status refused");}
 }
 lua_pushinteger(L,gw_ScriptGame_FighterStatus(port*2+(sub>0)));return 1;
}
static int l_hit_rules(lua_State* L) {
 int port=gs_slot_arg(L,1),sub=gs_hr_selector(L,2),p=port*2+(sub>0),i,j;GsHitRules r;gs_hr_read(p,&r);
 lua_createtable(L,r.count,3);gs_setint(L,"status_bits",r.status);gs_setint(L,"owner",gw_ScriptGame_HitRulesOwner(p));gs_setbool(L,"sub",sub>0);
 for(i=0;i<r.count;++i){int* b=r.rules[i];lua_createtable(L,0,4);gs_setint(L,"id",b[0]);gs_setint(L,"handle",b[12]);lua_createtable(L,0,5);gs_setstr(L,"move",b[1]<0?"any":gs_hr_moves[b[1]]);if(b[2]>=0)gs_setbool(L,"grounded",b[2]);gs_setstr(L,"element",gs_hr_element_name(b[3]));gs_setint(L,"status_bits",b[4]);gs_setbool(L,"incoming",b[13]);lua_setfield(L,-2,"match");lua_createtable(L,0,7);if(b[5]>=0)gs_setstr(L,"element",gs_hr_element_name(b[5]));
  for(j=0;j<4;++j){static const char* const keys[]={"damage","knockback_growth","knockback_base","knockback_taken"};union{int i;float f;}v;v.i=b[j==3?11:6+j];gs_setnum(L,keys[j],v.f);}gs_setint(L,"shield_damage",b[9]);gs_setint(L,"hitstun",b[10]);lua_setfield(L,-2,"change");lua_rawseti(L,-2,i+1);}
 return 1;
}
