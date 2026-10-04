/* Production preload and callback cleanup, with deterministic loader/destructor. */
static int gs_item_fixture_loads,gs_item_fixture_despawns;
static int gs_item_fixture_load(lua_State* L) {
 ++gs_item_fixture_loads;
 if(lua_gethook(L)!=NULL || gs.cur!=-1)return luaL_error(L,"loader stayed on script budget");
 lua_pushinteger(L,17);return 1;
}
static int gs_item_fixture_bad_load(lua_State* L) {++gs_item_fixture_loads;return luaL_error(L,"fixture optional asset refusal");}
static int gs_item_fixture_despawn(int handle,int owner) {
 if(handle==101&&owner==7)++gs_item_fixture_despawns;return 1;
}
static int test_script_item_preload(void) {
 int ok=1,top,cur,oldcount;lua_Hook hook;int mask,count;
 unsigned char saved_state[5][2];int saved_models[5][2];
 gs_init();if(!gs.L)return 1;
 top=lua_gettop(gs.L);cur=gs.cur;hook=lua_gethook(gs.L);mask=lua_gethookmask(gs.L);count=lua_gethookcount(gs.L);
 memcpy(saved_state,gs_item_visual_state[0],sizeof saved_state);memcpy(saved_models,gs_item_models[0],sizeof saved_models);
 memset(gs_item_visual_state[0],0,sizeof saved_state);gs_item_fixture_loads=0;
 if(gs_item_visual_ready(0,0))ok=0;
 gs_item_visual_load_one(gs.L,0,0,0,gs_item_fixture_load);
 if(gs_item_visual_ready(0,0)||gs_item_models[0][0][0]!=17)ok=0;
 gs_item_visual_load_one(gs.L,0,0,1,gs_item_fixture_bad_load);
 if(!gs_item_visual_ready(0,0)||gs_item_models[0][0][1]!=0||gs_item_fixture_loads!=2)ok=0;
 if(lua_gettop(gs.L)!=top||gs.cur!=cur||lua_gethook(gs.L)!=hook||lua_gethookmask(gs.L)!=mask||lua_gethookcount(gs.L)!=count)ok=0;
 oldcount=gs_item_call_count;if(oldcount>=GS_ITEM_CALL_SPAWNS) {gw_test_fail("item fixture called during full spawn transaction");memcpy(gs_item_visual_state[0],saved_state,sizeof saved_state);memcpy(gs_item_models[0],saved_models,sizeof saved_models);return 1;}gs_item_call_spawns[oldcount].handle=101;gs_item_call_spawns[oldcount].owner=7;
 gs_item_call_count=oldcount+1;gs_item_fixture_despawns=0;
 gs_item_call_finish(oldcount,0,gs_item_fixture_despawn);
 if(gs_item_fixture_despawns!=1||gs_item_call_count!=oldcount)ok=0;
 gs_item_call_spawns[oldcount].handle=101;gs_item_call_spawns[oldcount].owner=7;gs_item_call_count=oldcount+1;
 gs_item_call_finish(oldcount,1,gs_item_fixture_despawn);
 if(gs_item_fixture_despawns!=1||gs_item_call_count!=oldcount)ok=0;
 memcpy(gs_item_visual_state[0],saved_state,sizeof saved_state);memcpy(gs_item_models[0],saved_models,sizeof saved_models);
 if(!ok)gw_test_fail("item preload budget/readiness/fallback or atomic error cleanup failed");return ok?0:1;
}
