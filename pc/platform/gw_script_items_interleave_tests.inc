/* Real game allocation interleave plus a loaded, procedural visual binding.
 * Invoke in an offline LAB scene with the drive definition/common items loaded.
 * Does not launch a game, read optional assets, or render GPU commands. */
extern int gw_Geno_ItemInterleaveFixture(int def);
extern int gw_Item_StageArticleTableTest(void);
static int test_script_item_stage_articles(void) {
 int rc=gw_Item_StageArticleTableTest();
 if(rc)gw_test_fail("stage item retirement/rebinding failed (%d)",rc);return rc?1:0;
}
static int test_script_item_interleave(void) {
 static const char* files[]={"drive.gxmesh","drive.coll.json","kit.gxtex"};
 static const char side[]="{\"version\":1,\"atlas\":\"kit\",\"lines\":[]}";
 unsigned char mesh[138]={0},tex[128]={0};
 char dir[MAX_PATH],path[MAX_PATH];lua_State* L=NULL;
 int def,i,rc=1,oldcount,model=0,saved[2];
 gs_init();if(!gs.L)return 1;
 def=gw_Geno_ItemFind("drive");
 if(def<0||!gs.match_active||gw_RB_Enabled()||gw_Netplay_Enabled()) {
  gw_log("script_item_interleave: SKIP (requires offline LAB with drive/common items loaded)");return 0;
 }
 oldcount=gs_stage_nmodels;saved[0]=gs_item_models[def][0][0];saved[1]=gs_item_models[def][0][1];
 snprintf(dir,sizeof dir,"%s\\gw_item_interleave_test",gs.exe_dir);CreateDirectoryA(dir,NULL);
 memcpy(mesh,"GXMS",4);gw_w32(mesh+4,2);gw_w32(mesh+8,3);gw_w32(mesh+12,3);
 for(i=16;i<28;i+=4)gw_w32(mesh+i,(unsigned)gs_fbits(2));
 gw_w32(mesh+28,36);gw_w32(mesh+32,132);
 gw_w32(mesh+68,(unsigned)gs_fbits(1));gw_w32(mesh+104,(unsigned)gs_fbits(1));
 for(i=0;i<3;++i){gw_w32(mesh+36+32*i+28,(unsigned)gs_fbits(1));gw_w16(mesh+132+2*i,(uint16_t)i);}
 memcpy(tex,"GXTX",4);gw_w32(tex+4,1);gw_w32(tex+8,GX_TF_RGBA8);
 gw_w32(tex+12,4);gw_w32(tex+16,4);gw_w32(tex+20,0xFFFFFFFF);
 gw_w32(tex+28,64);gw_w32(tex+36,64);memset(tex+64,255,64);
 if(!t_model_file(dir,files[0],mesh,sizeof mesh)||!t_model_file(dir,files[1],side,sizeof side-1)||
    !t_model_file(dir,files[2],tex,sizeof tex))goto done;
 L=luaL_newstate();if(!L)goto done;
 lua_pushstring(L,dir);lua_pushcclosure(L,t_model_open,1);lua_pushstring(L,"drive");
 if(lua_pcall(L,1,1,0)!=LUA_OK){gw_test_fail("interleave model: %s",lua_tostring(L,-1));goto done;}
 model=(int)lua_tointeger(L,-1);
 if(model!=oldcount+1||gs_stage_nmodels!=oldcount+1)goto done;
 /* Same binding cache used by gw_ItemDraw: asset index, never JObj/DObj/Article. */
 gs_item_models[def][0][0]=model;gs_item_models[def][0][1]=0;
 rc=gw_Geno_ItemInterleaveFixture(def);
 if(rc)gw_test_fail("item allocation/descriptor interleave failed (%d)",rc);
 if(gs_item_models[def][0][0]!=model||gs_stage_nmodels!=oldcount+1)rc=1;
 done:
 gs_item_models[def][0][0]=saved[0];gs_item_models[def][0][1]=saved[1];
 if(L)lua_close(L);
 /* Only this fixture's appended pin is retired; existing scene pins survive. */
 if(gs_stage_nmodels==oldcount+1) {
  GsStageModel* art=&gs_stage_models[oldcount];free(art->mesh);
  if(art->atlas_owner){free(art->image);free(art->glow_image);}
  memset(art,0,sizeof *art);gs_stage_nmodels=oldcount;
 }
 for(i=0;i<3;++i){snprintf(path,sizeof path,"%s\\%s",dir,files[i]);DeleteFileA(path);}RemoveDirectoryA(dir);
 if(rc)gw_test_fail("loaded-model item interleave requires native acceptance");return rc?1:0;
}
