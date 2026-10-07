/* gw_script_fighter_lua.inc - gd.fighter_lua(port): a READ of a defined fighter's Lua state (Geno slice 5, docs/geno.md section 23).
 * Included by gw_script.c. Read-only, so like the other readouts it needs no `gameplay` flag; the Lua that writes the state is not a
 * `gd` script at all (it runs inside the fighter's own states and has no `gd`). */
extern int gw_GenoGame_LuaInfo(int port, int what);
extern int gw_Geno_LuaSlots(int p);
extern int gw_Geno_LuaSlotType(int p, int i);
extern const char *gw_Geno_LuaSlotName(int p, int i);

/* gd.fighter_lua(port) -> nil when the player's fighter declares no Lua state, else
 * { profile = n, faults = n, last_fault = n, state = { <slot name> = int | number | bool, ... } } */
static int l_fighter_lua(lua_State *L) {
    int slot = gs_slot_arg(L, 1), p, n, i;
    p = gw_GenoGame_LuaInfo(slot, 0);
    if (p < 0) { lua_pushnil(L); return 1; }
    n = gw_Geno_LuaSlots(p);
    lua_createtable(L, 0, 4);
    gs_setnum(L, "profile", p);
    gs_setnum(L, "faults", gw_GenoGame_LuaInfo(slot, 1));
    gs_setnum(L, "last_fault", gw_GenoGame_LuaInfo(slot, 2));
    lua_createtable(L, 0, n);
    for (i = 0; i < n; ++i) {
        unsigned w = (unsigned) gw_GenoGame_LuaInfo(slot, 3 + i);
        int ty = gw_Geno_LuaSlotType(p, i);
        if (ty == 0) lua_pushinteger(L, (lua_Integer) (int) w);
        else if (ty == 1) { float f; memcpy(&f, &w, 4); lua_pushnumber(L, (lua_Number) f); }
        else lua_pushboolean(L, w != 0);
        lua_setfield(L, -2, gw_Geno_LuaSlotName(p, i));
    }
    lua_setfield(L, -2, "state");
    return 1;
}
