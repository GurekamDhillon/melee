/* Unified item surface. Definitions/assets are immutable host resources;
 * handles, motion, lifetime and terminal guards belong to snapshotted game memory. */
#include "../geno/geno_items.h"
#include "../geno/geno_item_errors.h"
#define GS_EV_ITEM_COLLECT 12
#define GS_EV_ITEM_EXPIRE 13
extern int gw_Geno_ItemDefineText(const char*,const char*,const char*,char*,int);
extern int gw_Geno_ItemFind(const char*);
extern int gw_Geno_ItemCount(void);
extern int gw_Geno_ItemParam(int,int);
extern int gw_Geno_ItemPayload(int,int);
extern const char* gw_Geno_ItemName(int);
extern const char* gw_Geno_ItemText(int,int);
extern unsigned gw_Geno_ItemTint(int);
extern int gw_Geno_ItemColourVariants(int);
extern int gw_Geno_ItemSpawn(int,int,int,int,int,int,int,int,int);
extern int gw_Geno_ItemSpawnKind(int,int,int,int,int,int,int,int,int);
extern int gw_Geno_ItemDespawn(int,int);
extern void gw_Geno_ItemReleaseOwner(int);
extern void gw_Geno_ItemsSceneReset(void);
extern int gw_Geno_ItemI(int,int);
extern int gw_Geno_ItemF(int,int);
extern int gw_Geno_ItemVisual(int,int);
extern int gw_Geno_ItemIndexForId(int);
extern int gw_Geno_ItemMeta(int,int);
extern int gw_Geno_ItemActive(int);
extern int gw_Mex_ItemReady(int);
extern const char* gw_Mex_ItemName(int);
static const char* const gs_item_colours[]={"red","green","blue","yellow","white"};
static int gs_want_item_collect,gs_want_item_expire;
static unsigned gs_item_collect_families,gs_item_expire_families;
static int gs_item_serial=0x40000000; /* disjoint from engine x1C serial IDs */
static int gs_item_models[GENO_ITEM_MAX_DEFS][5][2]; /* scene pins, asset index+1 */
static unsigned char gs_item_visual_state[GENO_ITEM_MAX_DEFS][5][2];
static void gs_item_models_reset(void) {
 memset(gs_item_models,0,sizeof gs_item_models);
 memset(gs_item_visual_state,0,sizeof gs_item_visual_state);
}
/* Native definitions always qualify. Explicit family opt-in still excludes
 * vanilla fighter/stage articles and Geno fighter-owned articles. */
static unsigned gs_item_event_family(int def) {
 int kind;
 if(def>=0) return def<GENO_ITEM_MAX_DEFS?1:0;
 if(def==INT_MIN)return 0;kind=-def-1;
 if(kind<=34)return 2;
 if(kind>=237&&kind<5000&&!GENO_ART_IS_KIND(kind))return 4;
 return 0;
}
static int gs_item_event_accept(int script,int def) {
 unsigned family=gs_item_event_family(def);
 return family && (family==1 || (gs.s[script].item_event_families&family));
}
static int gs_item_path_valid(const char* path,size_t len) {
 size_t i;
 if(len<17||len>=MAX_PATH||strlen(path)!=len||strncmp(path,"items/",6)||strcmp(path+len-10,"/item.json")) return 0;
 for(i=6;i<len-10;++i) if(!((path[i]>='a'&&path[i]<='z')||(path[i]>='0'&&path[i]<='9')||path[i]=='_'||path[i]=='-')) return 0;
 return len>16;
}
static int gs_item_define_root(const char* root,const char* rel,char* error,int cap) {
#ifdef _WIN32
 char path[MAX_PATH],canonical[MAX_PATH]; unsigned char* bytes; DWORD full;
 size_t n; int def;
 if(!gs_item_path_valid(rel,strlen(rel))||snprintf(path,sizeof path,"%s\\%s",root,rel)>=sizeof path) { snprintf(error,cap,"invalid items/<name>/item.json path"); return -1; }
 full=GetFullPathNameA(root,sizeof canonical,canonical,NULL);
 if(!full||full>=sizeof canonical) {snprintf(error,cap,"item root path unavailable");return -1;}
 root=canonical;
 /* This reader verifies the final handle is inside the root, rejects directories,
  * reads from that same handle and rejects changed/truncated input. */
 bytes=gs_model_read_source(root,path,&n,65536);
 if(!bytes) { snprintf(error,cap,"item file missing, unreadable or outside the mod folder"); return -1; }
 if(memchr(bytes,0,n)) { free(bytes); snprintf(error,cap,"item JSON contains NUL"); return -1; }
 def=gw_Geno_ItemDefineText((char*)bytes,root,rel,error,cap); free(bytes); return def;
#else
 (void)root;(void)rel;snprintf(error,cap,"contained item files require Windows");return -1;
#endif
}
static int l_item_define(lua_State* L) {
 char root[MAX_PATH],error[256]; size_t len; const char* path=luaL_checklstring(L,1,&len); int def;
 gs_require_offline(L,"item_define");
 if(!gs_item_path_valid(path,len)||!gs_mission_script_root(root)) return gs_mission_error(L,"item definition requires items/<name>/item.json in the calling mod");
 def=gs_item_define_root(root,path,error,sizeof error);
 if(def<0) return gs_mission_error(L,error);
 lua_pushstring(L,gw_Geno_ItemName(def));return 1;
}
static int gs_item_dir_compare(const void* a,const void* b) {return strcmp((const char*)a,(const char*)b);}
static void gs_item_scan_mods(void) {
#ifdef _WIN32
 int i;
 for(i=0;i<gw_Mods_Count();++i) {
  char root[MAX_PATH],pattern[MAX_PATH],names[GENO_ITEM_MAX_DEFS][49];int count=0,k,overflow=0;
  WIN32_FIND_DATAA fd; HANDLE h;
  if(!gw_Mods_IsActive(i)) continue;
  if(snprintf(root,sizeof root,"%s\\%s",gw_Mods_Dir(),gw_Mods_Id(i))>=sizeof root||snprintf(pattern,sizeof pattern,"%s\\items\\*",root)>=sizeof pattern) continue;
  h=FindFirstFileA(pattern,&fd);if(h==INVALID_HANDLE_VALUE) continue;
  do {
   if(!(fd.dwFileAttributes&FILE_ATTRIBUTE_DIRECTORY)||fd.cFileName[0]=='.') continue;
   if(strlen(fd.cFileName)>48) {gw_log("geno item: directory name too long: %s",fd.cFileName);continue;}
   if(count==GENO_ITEM_MAX_DEFS) {overflow=1;continue;}
   snprintf(names[count++],49,"%s",fd.cFileName);
  } while(FindNextFileA(h,&fd)); FindClose(h);
  if(overflow) {gw_log("geno item: more than 64 directories in %s; mod item scan refused",root);continue;}
  qsort(names,count,sizeof names[0],gs_item_dir_compare);
  for(k=0;k<count;++k) {
   char rel[MAX_PATH],error[256];snprintf(rel,sizeof rel,"items/%s/item.json",names[k]);
   if(gs_item_define_root(root,rel,error,sizeof error)<0) gw_log("geno item: %s/%s refused: %s",gw_Mods_Id(i),rel,error);
  }
 }
#endif
}
static void gs_item_rawfield(lua_State* L,int table,const char* key) {table=lua_absindex(L,table);lua_pushstring(L,key);lua_rawget(L,table);}
static void gs_item_keys(lua_State* L,int table,const char* const* allowed) {
 int k;table=lua_absindex(L,table);lua_pushnil(L);
 while(lua_next(L,table)) {
  size_t keylen=0;const char* key=lua_type(L,-2)==LUA_TSTRING?lua_tolstring(L,-2,&keylen):NULL;
  if(key&&strlen(key)!=keylen)luaL_error(L,"item key contains NUL");
  for(k=0;allowed[k]&&(!key||strcmp(allowed[k],key));++k){}
  if(!allowed[k])luaL_error(L,"unknown item option/payload key");lua_pop(L,1);
 }
}
static int l_item_events(lua_State* L) {
 static const char* const keys[]={"vanilla","mex",NULL};int k;unsigned mask=0;
 if(gs.cur<0||gs.cur>=gs.n)return luaL_error(L,"item_events requires a script context");
 if(!lua_isnoneornil(L,1)) {
  luaL_checktype(L,1,LUA_TTABLE);gs_item_keys(L,1,keys);
  for(k=0;k<2;++k) {
   gs_item_rawfield(L,1,keys[k]);
   if(!lua_isnil(L,-1)) {luaL_checktype(L,-1,LUA_TBOOLEAN);if(lua_toboolean(L,-1))mask|=2u<<k;}
   lua_pop(L,1);
  }
  gs.s[gs.cur].item_event_families=mask;gs_update_want_events();
 }
 lua_createtable(L,0,2);gs_setbool(L,"vanilla",gs.s[gs.cur].item_event_families&2);
 gs_setbool(L,"mex",gs.s[gs.cur].item_event_families&4);return 1;
}
/* Load in a protected Lua call before spawning. Art failure is visual only and
 * never turns a successful native spawn into a duplicate script pickup. */
static int gs_item_asset_load(lua_State* L) {
 int def=(int)lua_tointeger(L,1),colour=(int)lua_tointeger(L,2),part=(int)lua_tointeger(L,3),mod;
 const char* model=gw_Geno_ItemText(def,part?2:1); const char* requested=gw_Geno_ItemText(def,3);
 char root[MAX_PATH],rel[181],dir[MAX_PATH],*last;
 if(!*model) { lua_pushinteger(L,0);return 1; }
 snprintf(root,sizeof root,"%s",gw_Geno_ItemText(def,0));
 if(*requested) {
  mod=gw_Mods_Find(requested);
  if(mod<0||!gw_Mods_IsActive(mod)) { lua_pushinteger(L,0);return 1; }
  if(snprintf(root,sizeof root,"%s\\%s",gw_Mods_Dir(),gw_Mods_Id(mod))>=sizeof root) return luaL_error(L,"item art root too long");
 }
 if(snprintf(rel,sizeof rel,"%s%s%s",model,!part&&gw_Geno_ItemColourVariants(def)?"_":"",!part&&gw_Geno_ItemColourVariants(def)?gs_item_colours[colour]:"")>=sizeof rel) return luaL_error(L,"item model path too long");
 last=strrchr(rel,'/');
 if(last) { *last++=0; if(snprintf(dir,sizeof dir,"%s\\%s",root,rel)>=sizeof dir) return luaL_error(L,"item model path too long"); }
 else { last=rel;snprintf(dir,sizeof dir,"%s",root); }
 lua_pushinteger(L,gs_model_open_at_root(L,dir,last,root));return 1;
}
/* One colour/part per engine tick, outside script calls and their budget.
 * Failed optional art is a completed fallback, never retried by item_spawn. */
static void gs_item_visual_load_one(lua_State* L,int def,int colour,int part,lua_CFunction loader) {
 int top=lua_gettop(L),cur=gs.cur;lua_Hook hook=lua_gethook(L);int mask=lua_gethookmask(L),count=lua_gethookcount(L);
 gs.cur=-1;lua_sethook(L,NULL,0,0);gs_item_models[def][colour][part]=0;
 lua_pushcfunction(L,loader);lua_pushinteger(L,def);lua_pushinteger(L,colour);lua_pushinteger(L,part);
 if(lua_pcall(L,3,1,0)!=LUA_OK)gw_log("geno item: visual fallback def=%d: %s",def,lua_tostring(L,-1));
 else gs_item_models[def][colour][part]=(int)lua_tointeger(L,-1);
 gs_item_visual_state[def][colour][part]=1;
 lua_settop(L,top);lua_sethook(L,hook,mask,count);gs.cur=cur;
}
static void gs_item_visual_tick(void) {
 int def,colour,part;lua_State* L=gs.L;if(!L)return;
 for(def=0;def<gw_Geno_ItemCount();++def)for(colour=0;colour<5;++colour)for(part=0;part<2;++part) {
  if(gs_item_visual_state[def][colour][part])continue;
  gs_item_visual_load_one(L,def,colour,part,gs_item_asset_load);return;
 }
}
static int gs_item_visual_ready(int def,int colour) {
 return gs_item_visual_state[def][colour][0]&&gs_item_visual_state[def][colour][1];
}
static const char* gs_item_spawn_error(int result) {
 switch(result) {
 case GENO_ITEM_ERR_UNSUPPORTED:return "unsupported item kind";
 case GENO_ITEM_ERR_UNSAFE:return "unsafe item kind (fighter, projectile or stage item)";
 case GENO_ITEM_ERR_UNINITIALIZED:return "item descriptor/logic uninitialized";
 case GENO_ITEM_ERR_POOL:return "item pool full";
 case GENO_ITEM_ERR_CAP:return "item cap reached";
 case GENO_ITEM_ERR_CONFLICT:return "native item range overlaps m-ex or reserved article kinds";
 default:return "item spawn failed";
 }
}
/* Callback error cleanup also covers a budget hook firing just after the C
 * function returned, before Lua could retain the handle. */
#define GS_ITEM_CALL_SPAWNS 128
static struct {int handle,owner;} gs_item_call_spawns[GS_ITEM_CALL_SPAWNS];
static int gs_item_call_count;
static int gs_item_call_begin(void) {return gs_item_call_count;}
static void gs_item_call_finish(int mark,int success,int (*despawn)(int,int)) {
 int i;if(!success)for(i=gs_item_call_count-1;i>=mark;--i)
  despawn(gs_item_call_spawns[i].handle,gs_item_call_spawns[i].owner);
 gs_item_call_count=mark;
}
static void gs_item_call_end(int mark,int success) {gs_item_call_finish(mark,success,gw_Geno_ItemDespawn);}
static int l_item_spawn(lua_State* L) {
 static const char* const option_keys[]={"vx","vy","payload",NULL};
 static const char* const payload_keys[]={"colour","amount",NULL};
 int def=-1,kind=-1,handle,result,colour=0,amount=1; double x,y,vx=0,vy=2.4; const char* name=NULL;
 gs_require_offline(L,"item_spawn");
 if(!gs.match_active) return gs_mission_error(L,"item_spawn requires an active match");
 if(lua_type(L,1)==LUA_TNUMBER) {
  lua_Integer value=luaL_checkinteger(L,1);if(value<0||value>=5000) return gs_mission_error(L,"unsupported item kind");kind=(int)value;
 } else {
  size_t len;name=luaL_checklstring(L,1,&len);if(strlen(name)!=len) return luaL_error(L,"item name contains NUL");
  def=gw_Geno_ItemFind(name);
  if(def<0 && (!strncmp(name,"mex:",4)||!strncmp(name,"vanilla:",8))) {
   const char* digits=name+(!strncmp(name,"mex:",4)?4:8);char* end;long value=strtol(digits,&end,10);
   if(end!=digits&&!*end&&value>=0&&value<5000&&((digits==name+4&&value>=237)||(digits==name+8&&value<237)))kind=(int)value;
  }
 }
 if(kind>=GENO_ITEM_KIND_BASE&&kind<GENO_ITEM_KIND_END&&gw_Geno_ItemActive(kind)) def=kind-GENO_ITEM_KIND_BASE;
 if(def>=0&&!gw_Geno_ItemParam(def,GENO_IP_DEFINED)) return gs_mission_error(L,"undefined native item kind");
 if(def<0&&kind<0) return gs_mission_error(L,"unknown item name");
 
 x=luaL_checknumber(L,2);y=luaL_checknumber(L,3);
 if(def>=0) { colour=gw_Geno_ItemPayload(def,0);amount=gw_Geno_ItemPayload(def,1); }
 if(!lua_isnoneornil(L,4)) {
  luaL_checktype(L,4,LUA_TTABLE);
  gs_item_keys(L,4,option_keys);
  gs_item_rawfield(L,4,"vx");if(!lua_isnil(L,-1)) vx=luaL_checknumber(L,-1);lua_pop(L,1);
  gs_item_rawfield(L,4,"vy");if(!lua_isnil(L,-1)) vy=luaL_checknumber(L,-1);lua_pop(L,1);
  gs_item_rawfield(L,4,"payload");
  if(!lua_isnil(L,-1)) {
   if(def<0)return gs_mission_error(L,"payload is supported only for standalone Geno items");
   luaL_checktype(L,-1,LUA_TTABLE);gs_item_keys(L,-1,payload_keys);gs_item_rawfield(L,-1,"colour");
   if(!lua_isnil(L,-1)) { int i;size_t clen;const char* c=luaL_checklstring(L,-1,&clen);if(strlen(c)!=clen)return luaL_error(L,"item colour contains NUL");for(i=0;i<5&&strcmp(c,gs_item_colours[i]);++i) {}if(i==5)return luaL_error(L,"unknown item payload colour");colour=i; }lua_pop(L,1);
   gs_item_rawfield(L,-1,"amount");if(!lua_isnil(L,-1)) { lua_Integer n=luaL_checkinteger(L,-1);if(n<0||n>1000000)return luaL_error(L,"item amount must be 0..1000000");amount=(int)n; }lua_pop(L,1);
  }lua_pop(L,1);
 }
 if(!isfinite(x)||!isfinite(y)||fabs(x)>100000||fabs(y)>100000||!isfinite(vx)||!isfinite(vy)||fabs(vx)>100||fabs(vy)>100) return luaL_error(L,"item position/velocity must be finite and within bounds");
 if(gs_item_serial==INT_MAX) return gs_mission_error(L,"item handle space exhausted");
 if(def>=0&&!gw_Geno_ItemActive(GENO_ITEM_KIND_BASE+def))return gs_mission_error(L,gs_item_spawn_error(GENO_ITEM_ERR_CONFLICT));
 if(def>=0&&!gs_item_visual_ready(def,colour))return gs_mission_error(L,"item visuals still preparing; retry next frame");
 if(gs_item_call_count>=GS_ITEM_CALL_SPAWNS)return gs_mission_error(L,"item spawn limit per callback reached");
 if(!lua_checkstack(L,2))return luaL_error(L,"item result stack unavailable");
 if(gs.budget<0||(!gw_turbo_enabled()&&gs_now_ms()>gs.deadline))return luaL_error(L,"item_spawn: script budget exhausted before spawn");
 handle=++gs_item_serial;gs_rw_branch();
 result=def>=0?gw_Geno_ItemSpawn(def,gs_fbits((float)x),gs_fbits((float)y),gs_fbits((float)vx),gs_fbits((float)vy),handle,gs.cur+1,colour,amount):
  gw_Geno_ItemSpawnKind(kind,gs_fbits((float)x),gs_fbits((float)y),gs_fbits((float)vx),gs_fbits((float)vy),handle,gs.cur+1,colour,amount);
 if(result<=0)return gs_mission_error(L,gs_item_spawn_error(result));
 gs_item_call_spawns[gs_item_call_count].handle=result;gs_item_call_spawns[gs_item_call_count++].owner=gs.cur+1;
 if(!gw_turbo_enabled()&&gs_now_ms()>gs.deadline) {
  gw_Geno_ItemDespawn(result,gs.cur+1);--gs_item_call_count;
  return luaL_error(L,"item_spawn: script budget exhausted; spawn cancelled");
 }
 lua_pushinteger(L,result);return 1;
}
static int l_item_despawn(lua_State* L) {
 lua_Integer h=luaL_checkinteger(L,1);gs_require_offline(L,"item_despawn");
 if(h<1||h>INT_MAX) return luaL_error(L,"invalid item handle");
 if(!gs.match_active) {lua_pushboolean(L,0);return 1;}
 gs_rw_branch();lua_pushboolean(L,gw_Geno_ItemDespawn((int)h,gs.cur+1));return 1;
}
static void gs_item_payload(lua_State* L,int colour,int amount) {
 lua_createtable(L,0,2);gs_setstr(L,"colour",colour>=0&&colour<5?gs_item_colours[colour]:"unknown");gs_setint(L,"amount",amount);
}
static const char* gs_item_layer(int kind) {
 return gw_Geno_ItemActive(kind)?"geno":GENO_ART_IS_KIND(kind)?"geno_article":kind>=237?"mex":"vanilla";
}
static void gs_item_row(lua_State* L,int serial,int kind) {
 int def=kind-GENO_ITEM_KIND_BASE,handle=gw_Geno_ItemMeta(serial,GENO_II_HANDLE);
 gs_setstr(L,"layer",gs_item_layer(kind));
 if(gw_Geno_ItemActive(kind)) gs_setstr(L,"name",gw_Geno_ItemName(def));
 else if(kind>=237&&strcmp(gs_item_layer(kind),"geno_article")) gs_setstr(L,"name",gw_Mex_ItemName(kind));
 else if(kind<237) { char name[32];snprintf(name,sizeof name,"vanilla:%d",kind);gs_setstr(L,"name",name); }
 if(handle>0) {
  gs_setint(L,"handle",handle);gs_setint(L,"script_owner",gw_Geno_ItemMeta(serial,GENO_II_OWNER));
  if(gw_Geno_ItemActive(kind)) {gs_item_payload(L,gw_Geno_ItemMeta(serial,GENO_II_COLOUR),gw_Geno_ItemMeta(serial,GENO_II_AMOUNT));lua_setfield(L,-2,"payload");}
 }
 if(gw_Geno_ItemActive(kind)) {
  int index=gw_Geno_ItemIndexForId(serial);
  if(index>=0) {
   gs_setnum(L,"visual_y",gs_camera_float(gw_Geno_ItemVisual(index,GENO_IV_Y)));
   gs_setnum(L,"rotation",gs_camera_float(gw_Geno_ItemVisual(index,GENO_IV_ROT_Y)));
   gs_setnum(L,"tilt",gs_camera_float(gw_Geno_ItemVisual(index,GENO_IV_TILT)));
   gs_setnum(L,"scale",gs_camera_float(gw_Geno_ItemVisual(index,GENO_IV_SCALE)));
   gs_setbool(L,"visible",gw_Geno_ItemI(index,GENO_II_VISIBLE));
   gs_setint(L,"age",gw_Geno_ItemI(index,GENO_II_AGE));
  }
 }
}
void gw_ItemEvent(int what,int handle,int def,int port,int xb,int yb,int colour,int amount) {
 GsEvent* e;unsigned family=gs_item_event_family(def);
 if(!family||!(family&(what==0?gs_item_collect_families:gs_item_expire_families)))return;
 if(!gs.L||gw_Snap_Resimulating()||(what==0?!gs_want_item_collect:!gs_want_item_expire)) return;
 if(gs.nev>=GS_MAX_EVENTS) {++gs.ev_dropped;return;}
 e=&gs.ev[gs.nev++];memset(e,0,sizeof *e);e->what=what==0?GS_EV_ITEM_COLLECT:GS_EV_ITEM_EXPIRE;
 e->a=handle;e->b=def;e->c=port;e->d=xb;e->item_y=yb;e->item_colour=colour;e->item_amount=amount;e->item_reason=what;
#ifdef GS_SPINE_EVENTS
 gs_spine_event_capture(e);
#endif
}
static void gs_item_event_push(lua_State* L,const GsEvent* e) {
 int kind=e->b>=0?GENO_ITEM_KIND_BASE+e->b:-e->b-1;
 lua_createtable(L,0,9);gs_setint(L,"item",e->a);gs_setint(L,"kind",kind);gs_setstr(L,"layer",gs_item_layer(kind));
 gs_setstr(L,"reason",e->item_reason==0?"collected":e->item_reason==1?"expired":"destroyed");
 if(e->b>=0) gs_setstr(L,"name",gw_Geno_ItemName(e->b));
 else if(kind>=237) gs_setstr(L,"name",gw_Mex_ItemName(kind));
 else { char name[32];snprintf(name,sizeof name,"vanilla:%d",kind);gs_setstr(L,"name",name); }
 if(e->c>=0) gs_setint(L,"port",e->c+1);
 gs_setnum(L,"x",gs_camera_float(e->d));gs_setnum(L,"y",gs_camera_float(e->item_y));
 if(e->b>=0) {gs_item_payload(L,e->item_colour,e->item_amount);lua_setfield(L,-2,"payload");}
}
