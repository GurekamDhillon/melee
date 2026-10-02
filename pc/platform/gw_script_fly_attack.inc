/* Debug cursor controls. Included after the existing fly helpers. */
extern int gw_GenoFly_Target(int slot, int x_bits, int y_bits);
extern int gw_GenoFly_AttackSet(int slot, int on, int damage, int radius_bits);
extern int gw_GenoFly_State(int slot, int field);
extern void gw_GenoFly_Clear(int slot);
extern void gw_GenoFly_Reset(void);
extern int gw_GenoFly_Watch(void);
static int gs_fly_cursor_owner[6];

static int gs_fly_cursor_caller(void) {
    return gs.cur == gs.console ? GS_MAX_SCRIPTS + 1 : gs.cur + 1;
}
static int gs_fly_cursor_available(int slot, int owner) {
    if (!gw_GenoFly_State(slot, 0) && !gw_GenoFly_State(slot, 1)) gs_fly_cursor_owner[slot] = 0;
    return !gs_fly_cursor_owner[slot] || gs_fly_cursor_owner[slot] == owner;
}
static void gs_fly_clear_owner(int owner) {
    int slot;
    for (slot = 0; slot < 6; ++slot) if (gs_fly_cursor_owner[slot] == owner) {
        gw_GenoFly_Clear(slot);
        gs_fly_cursor_owner[slot] = 0;
    }
}
static void gs_fly_cursor_reset(void) {
    gw_GenoFly_Reset();
    memset(gs_fly_cursor_owner, 0, sizeof gs_fly_cursor_owner);
}
static void gs_fly_cursor_watch(void) {
    if (gw_GenoFly_Watch()) memset(gs_fly_cursor_owner, 0, sizeof gs_fly_cursor_owner);
}
static float gs_fly_number(int bits) { union { int i; float f; } u; u.i = bits; return u.f; }
static int gs_fly_target_set(int slot, int owner, float x, float y) {
    int rc;
    if (!gs_fly_cursor_available(slot, owner)) return -4;
    rc = gs_fly_set(slot, 1);
    if (rc < 0) return rc;
    rc = gw_GenoFly_Target(slot, gs_fly_bits(x), gs_fly_bits(y));
    if (rc >= 0) gs_fly_cursor_owner[slot] = owner;
    return rc;
}
static int gs_fly_attack_set(int slot, int owner, int on, int damage, float radius) {
    int rc;
    if (!gs_fly_cursor_available(slot, owner)) return -4;
    if (on) { rc = gs_fly_set(slot, 1); if (rc < 0) return rc; }
    gs_rw_branch();
    rc = gw_GenoFly_AttackSet(slot, on, damage, gs_fly_bits(radius));
    if (rc >= 0) gs_fly_cursor_owner[slot] = owner;
    return rc;
}
static const char *gs_fly_cursor_error(int rc) {
    return rc == -4 ? "cursor belongs to another script" : rc == -3 ? "invalid cursor parameters" : gs_fly_err(rc);
}
static int l_fly_target(lua_State *L) {
    int slot = gs_slot_arg(L, 1), rc;
    float x = (float)luaL_checknumber(L, 2), y = (float)luaL_checknumber(L, 3);
    if (!isfinite(x) || !isfinite(y) || fabsf(x) > 10000 || fabsf(y) > 10000)
        return luaL_error(L, "fly_target requires finite coordinates within +/-10000");
    gs_require_offline(L, "fly_target");
    rc = gs_fly_target_set(slot, gs_fly_cursor_caller(), x, y);
    if (rc < 0) return luaL_error(L, "fly_target: %s", gs_fly_cursor_error(rc));
    lua_pushboolean(L, 1); return 1;
}
static int l_fly_attack(lua_State *L) {
    int slot = gs_slot_arg(L, 1), on, rc;
    lua_Integer damage;
    float radius;
    luaL_checktype(L, 2, LUA_TBOOLEAN);
    on = lua_toboolean(L, 2);
    damage = luaL_optinteger(L, 3, 3);
    radius = (float)luaL_optnumber(L, 4, 6);
    if (damage < 1 || damage > 30 || !isfinite(radius) || radius < 1 || radius > 30)
        return luaL_error(L, "fly_attack requires damage 1..30 and radius 1..30");
    gs_require_offline(L, "fly_attack");
    rc = gs_fly_attack_set(slot, gs_fly_cursor_caller(), on, (int)damage, radius);
    if (rc < 0) return luaL_error(L, "fly_attack: %s", gs_fly_cursor_error(rc));
    lua_pushboolean(L, gw_GenoFly_State(slot, 0)); return 1;
}
static int l_fly_clear(lua_State *L) {
    int slot = gs_slot_arg(L, 1);
    gs_require_offline(L, "fly_clear");
    if (!gs_fly_cursor_available(slot, gs_fly_cursor_caller()))
        return luaL_error(L, "fly_clear: cursor belongs to another script");
    gs_rw_branch(); gw_GenoFly_Clear(slot); gs_fly_cursor_owner[slot] = 0;
    return 0;
}
static int l_fly_state(lua_State *L) {
    int slot = gs_slot_arg(L, 1);
    lua_createtable(L, 0, 8);
    gs_setbool(L, "flying", gs_fly_on(slot));
    gs_setbool(L, "attacking", gw_GenoFly_State(slot, 0));
    gs_setbool(L, "targeting", gw_GenoFly_State(slot, 1));
    gs_setint(L, "damage", gw_GenoFly_State(slot, 2));
    gs_setnum(L, "radius", gs_fly_number(gw_GenoFly_State(slot, 3)));
    gs_setnum(L, "x", gs_fly_number(gw_GenoFly_State(slot, 4)));
    gs_setnum(L, "y", gs_fly_number(gw_GenoFly_State(slot, 5)));
    gs_setint(L, "pulses", gw_GenoFly_State(slot, 6));
    return 1;
}
/* Caller has already checked the offline boundary. Return -2 for other commands. */
static int gs_fly_cursor_console(char a[][64], int n, int slot) {
    int rc, on, damage = 3, k = 1;
    float x, y, radius = 6;
    char *end;
    if (n < 1 || (_stricmp(a[0], "target") && _stricmp(a[0], "attack") && _stricmp(a[0], "clear"))) return -2;
    if (n > 1 && a[1][0] >= '1' && a[1][0] <= '6' && !a[1][1] &&
        (_stricmp(a[0], "target") || n == 4)) { slot = a[1][0] - '1'; k = 2; }
    if (_stricmp(a[0], "target") == 0) {
        if (n != k + 2) goto usage;
        x = strtof(a[k], &end); if (*end || !*a[k] || !isfinite(x) || fabsf(x) > 10000) goto usage;
        y = strtof(a[k + 1], &end); if (*end || !*a[k + 1] || !isfinite(y) || fabsf(y) > 10000) goto usage;
        rc = gs_fly_target_set(slot, GS_MAX_SCRIPTS + 1, x, y);
    } else if (_stricmp(a[0], "attack") == 0) {
        if (n < k + 1 || n > k + 3 || (_stricmp(a[k], "on") && _stricmp(a[k], "off"))) goto usage;
        on = !_stricmp(a[k], "on");
        if (n > k + 1) { long d = strtol(a[k + 1], &end, 10); if (*end || d < 1 || d > 30) goto usage; damage = (int)d; }
        if (n > k + 2) { radius = strtof(a[k + 2], &end); if (*end || !isfinite(radius) || radius < 1 || radius > 30) goto usage; }
        rc = gs_fly_attack_set(slot, GS_MAX_SCRIPTS + 1, on, damage, radius);
    } else {
        if (n != k) goto usage;
        if (!gs_fly_cursor_available(slot, GS_MAX_SCRIPTS + 1)) rc = -4;
        else { gs_rw_branch(); gw_GenoFly_Clear(slot); gs_fly_cursor_owner[slot] = 0; rc = 0; }
    }
    if (rc < 0) { gw_Console_Print(GS_RED, "fly: %s", gs_fly_cursor_error(rc)); return -1; }
    gw_Console_Print(GS_GREEN, "P%d cursor target=%s attack=%s pulses=%d", slot + 1,
        gw_GenoFly_State(slot, 1) ? "on" : "off", gw_GenoFly_State(slot, 0) ? "on" : "off", gw_GenoFly_State(slot, 6));
    return 0;
usage:
    gw_Console_Print(GS_RED, "usage: fly target [port] x y | fly attack [port] on|off [damage radius] | fly clear [port]");
    return -1;
}
