/* Exercise the real argument decoder without starting an audio voice. */
static int t_sound_options(lua_State* L) {
 int volume,pitch;gs_sound_options(L,&volume,&pitch);
 lua_pushinteger(L,volume);lua_pushinteger(L,pitch);return 2;
}
static int test_script_sound_options(void) {
 static const char code[]=
  "local v,p=decode(170); assert(v==127 and p==0)\n"
  "v,p=decode(170,{volume=72,pitch=350}); assert(v==72 and p==350)\n"
  "v,p=decode(170,{volume=0,pitch=-1200}); assert(v==0 and p==-1200)\n"
  "v,p=decode(170,{volume=127,pitch=1200}); assert(v==127 and p==1200)\n"
  "for _,o in ipairs({{volume=-1},{volume=128},{volume=.5},{pitch=1201},"
  "{pitch=-1201},{pitch=0/0},{pitch=math.huge},{pitch='100'},{vol=1}}) do\n"
  " assert(not pcall(decode,170,o)) end\n"
  "assert(not pcall(decode,170,false))\n";
 lua_State* L=luaL_newstate();int rc;
 if(!L)return 1;/* the port's Lua has no linit.c: open what the test needs, as the mission tests do */
 luaL_requiref(L, "_G", luaopen_base, 1); lua_pop(L, 1);
 luaL_requiref(L, "string", luaopen_string, 1); lua_pop(L, 1);
 luaL_requiref(L, "math", luaopen_math, 1); lua_pop(L, 1);
 lua_pushcfunction(L,t_sound_options);lua_setglobal(L,"decode");
 rc=luaL_dostring(L,code);
 if(rc)gw_test_fail("sound options: %s",lua_tostring(L,-1));
 lua_close(L);return rc?1:0;
}
