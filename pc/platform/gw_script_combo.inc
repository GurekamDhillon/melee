/* gw_script_combo.inc - gd.combo(port) and gd.combo_ki(port, n): the Geno combo graph (geno 11, docs/geno.md section 25).
 * Included by gw_script.c. gd.combo is a READ (no gameplay flag); gd.combo_ki writes the meter (offline, gameplay scripts, a
 * rewind branch, like gd.set_percent). */
extern int gw_GenoGame_ComboInfo(int port, int what);
extern int gw_GenoGame_ComboSetKi(int port, int ki);
extern int gw_Geno_ComboWord(int p, int route, int word);
extern const char *gw_Geno_ComboName(int p, int route);
extern const char *gw_Geno_ComboStateName(int p, int s);

static void gs_combo_target(lua_State *L, int p, unsigned t, const char *key)
{
    static const char *const acts[] = { "special_n", "special_s", "special_hi", "special_lw", "?", "?", "?", "?",
                                        "jump", "nair", "fair", "bair", "uair", "dair", "jump_short" };
    char buf[64];
    unsigned kind = t >> 28, id = t & 0xFFFF;
    if (t == GENO_TGT_AUTO) snprintf(buf, sizeof buf, "auto");
    else if (t == GENO_TGT_HELPLESS) snprintf(buf, sizeof buf, "helpless");
    else if (kind == GENO_TGT_GENO) snprintf(buf, sizeof buf, "geno:%s", gw_Geno_ComboStateName(p, (int) id));
    else if (kind == GENO_COMBO_TGT_ACT) snprintf(buf, sizeof buf, "%s", id < 15 ? acts[id] : "?");
    else if (id == GENO_COMBO_FROM_ANY) snprintf(buf, sizeof buf, "*");
    else snprintf(buf, sizeof buf, "motion:%u", id);
    lua_pushstring(L, buf);
    lua_setfield(L, -2, key);
}

/* gd.combo(port) -> nil when the player's fighter has no combo graph, else
 * { links, max_links, ki, ki_max (meter units), last_route, taken, hit, blocked, action_frame,
 *   routes = { {index, name, from, to, input, dir, on, where, lo, hi, buffer, priority, ki_cost, state = 0 closed | 1 open | 2 ready}, ... } }
 * `open` is every route whose source, window and conditions hold now (2: its input is buffered too: it fires this frame). */
static int l_combo(lua_State *L) {
    static const char *const in[] = { "light", "heavy", "special", "jump", "grab", "shield" };
    static const char *const dir[] = { "any", "neutral", "forward", "back", "up", "down", "side" };
    static const char *const on[] = { "any", "hit", "whiff", "block", "contact" };
    static const char *const wh[] = { "any", "ground", "air" };
    int slot = gs_slot_arg(L, 1), p, n, r, all = lua_toboolean(L, 2);
    n = gw_GenoGame_ComboInfo(slot, 3);
    if (n < 0) { lua_pushnil(L); return 1; }
    p = gw_GenoGame_ComboInfo(slot, 11);
    lua_createtable(L, 0, 12);
    gs_setnum(L, "links", gw_GenoGame_ComboInfo(slot, 0));
    gs_setnum(L, "max_links", gw_GenoGame_ComboInfo(slot, 7));
    gs_setnum(L, "ki", gw_GenoGame_ComboInfo(slot, 1) / 100.0);
    gs_setnum(L, "ki_max", gw_GenoGame_ComboInfo(slot, 8) / 100.0);
    gs_setnum(L, "last_route", gw_GenoGame_ComboInfo(slot, 2));
    gs_setnum(L, "taken", gw_GenoGame_ComboInfo(slot, 6));
    gs_setnum(L, "hit", gw_GenoGame_ComboInfo(slot, 4));
    gs_setnum(L, "blocked", gw_GenoGame_ComboInfo(slot, 5));
    gs_setnum(L, "action_frame", gw_GenoGame_ComboInfo(slot, 9));
    gs_setnum(L, "route_count", n);
    {
        int last = gw_GenoGame_ComboInfo(slot, 2);
        lua_pushstring(L, last >= 0 ? gw_Geno_ComboName(p, last) : "");
        lua_setfield(L, -2, "last_name");
    }
    lua_newtable(L);
    for (r = 0; r < n; ++r) {
        int stt = gw_GenoGame_ComboInfo(slot, 16 + r), i;
        if (!all && stt <= 0) continue;
        lua_createtable(L, 0, 14);
        gs_setnum(L, "index", r);
        lua_pushstring(L, gw_Geno_ComboName(p, r)); lua_setfield(L, -2, "name");
        gs_combo_target(L, p, (unsigned) gw_Geno_ComboWord(p, r, GENO_CR_FROM), "from");
        gs_combo_target(L, p, (unsigned) gw_Geno_ComboWord(p, r, GENO_CR_TO), "to");
        i = gw_Geno_ComboWord(p, r, GENO_CR_INPUT);
        lua_pushstring(L, i >= 0 && i < 6 ? in[i] : "none"); lua_setfield(L, -2, "input");
        i = gw_Geno_ComboWord(p, r, GENO_CR_DIR);
        lua_pushstring(L, i >= 0 && i < 7 ? dir[i] : "?"); lua_setfield(L, -2, "dir");
        i = gw_Geno_ComboWord(p, r, GENO_CR_ON);
        lua_pushstring(L, i >= 0 && i < 5 ? on[i] : "?"); lua_setfield(L, -2, "on");
        i = gw_Geno_ComboWord(p, r, GENO_CR_WHERE);
        lua_pushstring(L, i >= 0 && i < 3 ? wh[i] : "?"); lua_setfield(L, -2, "where");
        gs_setnum(L, "lo", gw_Geno_ComboWord(p, r, GENO_CR_WIN_LO));
        gs_setnum(L, "hi", gw_Geno_ComboWord(p, r, GENO_CR_WIN_HI));
        gs_setnum(L, "buffer", gw_Geno_ComboWord(p, r, GENO_CR_BUFFER));
        gs_setnum(L, "priority", gw_Geno_ComboWord(p, r, GENO_CR_PRIORITY));
        gs_setnum(L, "ki_cost", gw_Geno_ComboWord(p, r, GENO_CR_KI_COST) / 100.0);
        gs_setnum(L, "state", stt);
        lua_rawseti(L, -2, (lua_Integer) lua_rawlen(L, -2) + 1);
    }
    lua_setfield(L, -2, "routes");
    return 1;
}

/* gd.combo_ki(port, n): set the meter (meter units). Returns the new value or nil (no combo graph). */
static int l_combo_ki(lua_State *L) {
    int slot = gs_slot_arg(L, 1), v;
    double n = luaL_checknumber(L, 2);
    gs_require_offline(L, "combo_ki");
    gs_rw_branch();
    v = gw_GenoGame_ComboSetKi(slot, (int) (n * 100.0 + 0.5));
    if (v < 0) { lua_pushnil(L); return 1; }
    lua_pushnumber(L, v / 100.0);
    return 1;
}
