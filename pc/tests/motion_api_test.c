/* Actual Lua bindings with a scalar registry fixture: parser/ownership evidence,
 * not renderer, bridge or simulation parity evidence. */
#include <assert.h>
#include <stdio.h>
#include <string.h>
#include <limits.h>
#include <math.h>
#include <stdlib.h>
#include "../third_party/lua-5.4.7/src/lua.h"
#include "../third_party/lua-5.4.7/src/lauxlib.h"
#include "../third_party/lua-5.4.7/src/lualib.h"
#include "../platform/gw_motion.h"
static struct {int cur;} gs;
static int calls,next_handle,live_handle;static unsigned live_owner;static GwMotionOptions saved;
void gw_motion_defaults(GwMotionOptions*o,int kind){memset(o,0,sizeof *o);o->kind=kind;o->port=1;o->trigger=kind==GW_MOTION_AFTERIMAGE?2:0;o->copies=3;o->spacing=3;o->lifetime=16;o->length=12;o->smoothing=4;o->index=-2;o->scale=1;o->curve=1.5f;o->width=.8f;o->taper=1;o->intensity=.45f;o->depth=1;o->params[0]=1;}
int gw_motion_add(unsigned owner,const GwMotionOptions*o,const char**error){(void)error;calls++;saved=*o;live_owner=owner;return live_handle=++next_handle;}
int gw_motion_get(unsigned owner,int h,GwMotionOptions*o){if(h!=live_handle||!h||owner!=live_owner)return 0;*o=saved;return 1;}
int gw_motion_set(unsigned owner,int h,const GwMotionOptions*o,const char**error){(void)error;if(h!=live_handle||owner!=live_owner)return 0;calls++;saved=*o;return 1;}
int gw_motion_remove(unsigned owner,int h){if(h!=live_handle||owner!=live_owner)return 0;live_handle=0;return 1;}
void gw_motion_intensity(float value){(void)value;}
void gw_motion_stats(uint64_t out[GW_MOTION_STATS_COUNT]){memset(out,0,GW_MOTION_STATS_COUNT*sizeof *out);}
const char*gw_motion_stat_name(unsigned i){(void)i;return "fixture";}
static void gs_setint(lua_State*L,const char*k,lua_Integer n){lua_pushinteger(L,n);lua_setfield(L,-2,k);}
static void gs_item_keys(lua_State*L,int t,const char*const*keys){lua_pushnil(L);while(lua_next(L,t)){const char*k=luaL_checkstring(L,-2);int i;for(i=0;keys[i]&&strcmp(k,keys[i]);i++){}if(!keys[i])luaL_error(L,"unknown option");lua_pop(L,1);}}
#include "../platform/gw_script_motion.inc"
static void run(lua_State*L,const char*s){if(luaL_loadstring(L,s)||lua_pcall(L,0,0,0)){fprintf(stderr,"%s\n",lua_tostring(L,-1));exit(1);}}
int main(void){lua_State*L=luaL_newstate();luaL_requiref(L,"_G",luaopen_base,1);lua_pop(L,1);lua_newtable(L);
#define API(n,f) lua_pushcfunction(L,f);lua_setfield(L,-2,n)
API("afterimage_add",l_afterimage_add);API("afterimage_set",l_afterimage_set);API("afterimage_remove",l_afterimage_remove);API("tracer_add",l_tracer_add);API("tracer_hitboxes",l_tracer_hitboxes);API("tracer_set",l_tracer_set);API("tracer_remove",l_tracer_remove);lua_setglobal(L,"gd");
run(L,"h=assert(gd.afterimage_add(6,{sub=true,copies=12,lifetime=60}));assert(gd.afterimage_set(h,{spacing=8}));assert(not pcall(gd.afterimage_add,1,{copies=13}));assert(not pcall(gd.afterimage_add,1,{trigger='always'}));assert(not pcall(gd.afterimage_set,h,{width=0/0}));");
assert(saved.port==6&&saved.sub==1&&saved.trigger==2&&saved.copies==12&&saved.spacing==8);gs.cur=1;
run(L,"assert(not pcall(gd.afterimage_set,h,{}));assert(not pcall(gd.afterimage_remove,h))");gs.cur=0;
run(L,"assert(gd.afterimage_remove(h));assert(not pcall(gd.afterimage_set,h,{}));t=assert(gd.tracer_add{port=2,anchor={item=700},width=24,length=60});assert(gd.tracer_set(t,{shader='electric'}));");assert(saved.item==700&&saved.shader==3);
int before=calls;
run(L,"assert(not pcall(gd.tracer_hitboxes,1,{port=2}),'conflicting port accepted');assert(not pcall(gd.tracer_add,{shader='fire\\0ignored'}),'NUL shader accepted');assert(not pcall(gd.tracer_add,{anchor='right_hand\\0ignored'}),'NUL anchor accepted');assert(not pcall(gd.afterimage_add,1,{trigger='always\\0ignored',debug=true}),'NUL trigger accepted')");
run(L,"assert(not pcall(gd.tracer_add,{anchor={hitbox=4294967296}}),'oversized hitbox accepted');assert(not pcall(gd.tracer_add,{anchor={hitbox=-4294967296}}),'negative oversized hitbox accepted');assert(not pcall(gd.tracer_add,{anchor={joint='right_hand\\0ignored'}}),'nested NUL joint accepted')");
assert(calls==before);run(L,"assert(gd.tracer_add{anchor={joint='right_hand'}})");assert(saved.index==-2);
/* Colour variety options: ranges, shapes, finiteness, clearing, partial set keeps the rest. */
run(L,"p=assert(gd.afterimage_add(2,{palette={{1,0,0,1},{0,1,0,0.5}},hue_shift=-45,scale_falloff=0.5}))");
assert(saved.port==2&&saved.palette_count==2&&saved.palette[1][1]==1&&saved.palette[1][3]==0.5f&&saved.hue_shift==-45&&saved.scale_falloff==0.5f);
run(L,"assert(gd.afterimage_set(p,{copies=4}))");assert(saved.palette_count==2&&saved.copies==4); /* a partial set keeps the palette */
run(L,"assert(gd.afterimage_set(p,{palette={}}))");assert(saved.palette_count==0&&saved.palette[0][0]==0&&saved.palette[1][1]==0);
run(L,"assert(gd.afterimage_set(p,{palette={{1,1,1,1},{1,1,1,1},{1,1,1,1},{1,1,1,1},{1,1,1,1},{1,1,1,1}}}))");assert(saved.palette_count==6);
before=calls;
run(L,"assert(not pcall(gd.afterimage_set,p,{palette={{1,0,0}}}),'short colour accepted');"
      "assert(not pcall(gd.afterimage_set,p,{palette={{1,0,0,2}}}),'colour above 1 accepted');"
      "assert(not pcall(gd.afterimage_set,p,{palette={{1,0,0,0/0}}}),'nan colour accepted');"
      "assert(not pcall(gd.afterimage_set,p,{palette={{1,1,1,1},{1,1,1,1},{1,1,1,1},{1,1,1,1},{1,1,1,1},{1,1,1,1},{1,1,1,1}}}),'7 palette colours accepted');"
      "assert(not pcall(gd.afterimage_set,p,{palette=5}),'palette scalar accepted');"
      "assert(not pcall(gd.afterimage_set,p,{palette={5}}),'palette number entry accepted');"
      "assert(not pcall(gd.afterimage_set,p,{hue_shift=361}),'hue_shift 361 accepted');"
      "assert(not pcall(gd.afterimage_set,p,{hue_shift=1/0}),'inf hue_shift accepted');"
      "assert(not pcall(gd.afterimage_set,p,{scale_falloff=1.5}),'falloff 1.5 accepted')");
assert(calls==before);
run(L,"g=assert(gd.tracer_add{gradient={{1,0,0,1},{0,1,0,1},{0,0,1,0}},pulse={4,0.6},hue_drift=90,hue_span=-180,swell=1.5,params={1,2,0.5,2}})");
assert(saved.gradient_count==3&&saved.gradient[2][2]==1&&saved.pulse[0]==4&&saved.pulse[1]==0.6f&&saved.hue_drift==90&&saved.hue_span==-180&&saved.swell==1.5f&&saved.params[1]==2);
run(L,"assert(gd.tracer_set(g,{shader='glow'}))");assert(saved.gradient_count==3&&saved.shader==1);
run(L,"assert(gd.tracer_set(g,{gradient={}}))");assert(saved.gradient_count==0&&saved.gradient[2][2]==0);
run(L,"assert(gd.tracer_set(g,{gradient={{0,0,0,1},{1,1,1,1},{1,1,1,1},{1,1,1,1}}}))");assert(saved.gradient_count==4);
before=calls;
run(L,"assert(not pcall(gd.tracer_set,g,{gradient={{1,0,0,1}}}),'one-stop gradient accepted');"
      "assert(not pcall(gd.tracer_set,g,{gradient={{1,0,0,1},{1,1,1,1},{1,1,1,1},{1,1,1,1},{1,1,1,1}}}),'5-stop gradient accepted');"
      "assert(not pcall(gd.tracer_set,g,{pulse={4}}),'short pulse accepted');"
      "assert(not pcall(gd.tracer_set,g,{pulse={21,0.5}}),'pulse rate 21 accepted');"
      "assert(not pcall(gd.tracer_set,g,{pulse={4,1.5}}),'pulse depth 1.5 accepted');"
      "assert(not pcall(gd.tracer_set,g,{pulse={4,0/0}}),'nan pulse accepted');"
      "assert(not pcall(gd.tracer_set,g,{hue_drift=721}),'hue_drift 721 accepted');"
      "assert(not pcall(gd.tracer_set,g,{hue_span=-721}),'hue_span -721 accepted');"
      "assert(not pcall(gd.tracer_set,g,{swell=4.5}),'swell 4.5 accepted');"
      "assert(not pcall(gd.tracer_set,g,{swell=-1.5}),'swell -1.5 accepted');"
      "assert(not pcall(gd.tracer_set,g,{swell=0/0}),'nan swell accepted');"
      "assert(not pcall(gd.tracer_set,g,{gradient={{1,0,0,1},{1,1,1,'x'}}}),'string colour accepted')");
assert(calls==before);
lua_close(L);puts("motion production Lua parser: limits, fixture owner/stale checks, explicit port, NUL/integer refusal and colour-variety options PASS");return 0;}
