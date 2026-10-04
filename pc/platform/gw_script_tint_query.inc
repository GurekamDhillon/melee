extern int gw_ScriptGame_TintCount(int slot);
static int l_dobj_tints(lua_State* L) {
    int n=gw_ScriptGame_TintCount(gs_slot_arg(L,1));
    lua_createtable(L,0,2); gs_setint(L,"count",n); gs_setbool(L,"any",n>0); return 1;
}
