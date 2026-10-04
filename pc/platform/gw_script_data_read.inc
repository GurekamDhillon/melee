#include "gw_script_data_io.h"
static int l_data_exists(lua_State* L) {
    char path[MAX_PATH]; const char* error; int result;
    gs_data_path(L,luaL_checkstring(L,1),path,sizeof path);
    result=gs_data_exists_path(path,&error);
    if (result<0) { lua_pushnil(L); lua_pushstring(L,error); return 2; }
    lua_pushboolean(L,result); return 1;
}
static int l_data_read(lua_State* L) {
    char path[MAX_PATH]; char* buf; size_t n; const char* error;
    gs_data_path(L,luaL_checkstring(L,1),path,sizeof path);
    if (!gs_data_read_path(path,&buf,&n,&error)) {
        lua_pushnil(L); lua_pushstring(L,error); return 2;
    }
    lua_pushlstring(L,buf,n); free(buf); return 1;
}
