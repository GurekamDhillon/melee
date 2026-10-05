/* gd.cpu_pad / gd.cpu_script / gd.cpu_dest / gd.cpu_macro: a script writes a CPU's virtual
 * controller (fp->cpu.*), the same controller the retail AI writes. The game half is
 * pc/gameworld/script_cpu_ctl.inc. The command names and numbers are the retail CpuCmd_*
 * enum (src/melee/ft/ftcmdscript.h); the game-side test asserts the numbers match it. */
extern int gw_ScriptGame_CpuScriptMode(int slot);
extern int gw_ScriptGame_CpuCtl(int slot, int sub, int op, int a);
extern int gw_ScriptGame_CpuAttr(int slot, int id);

enum { GCC_PAD_COMMIT = 1, GCC_SCRIPT_BEGIN, GCC_SCRIPT_BYTE, GCC_SCRIPT_RUN, GCC_DEST_X, GCC_DEST_Y,
       GCC_TARGET, GCC_CANCEL, GCC_STAGE = 10,
       GCC_GOTO_X = 20, GCC_GOTO_Y, GCC_GOTO_RADIUS, GCC_GOTO_START, GCC_GOTO_STOP,
       GCC_G_ACTIVE = 100, GCC_G_RUNNING, GCC_G_DONE_COUNT, GCC_G_PAD_FRAMES, GCC_G_DEST_X,
       GCC_G_DEST_Y, GCC_G_BUTTONS, GCC_G_LX, GCC_G_LY, GCC_G_CX, GCC_G_CY, GCC_G_LTRIG,
       GCC_G_RTRIG, GCC_G_TARGET, GCC_G_CSP, GCC_G_DURATION, GCC_G_FREE, GCC_G_GOTO_STATE,
       GCC_G_GOTO_TICKS };

/* name, code, argument count, argument kind: s = signed stick/magnitude byte, u = 0..255 */
static const struct { const char *name; int code; int nargs; char kind; } gs_cpu_cmds[] = {
    {"press_a", 1, 0, 0}, {"release_a", 2, 0, 0}, {"press_b", 3, 0, 0}, {"release_b", 4, 0, 0},
    {"press_x", 5, 0, 0}, {"release_x", 6, 0, 0}, {"press_y", 7, 0, 0}, {"release_y", 8, 0, 0},
    {"press_r", 9, 0, 0}, {"release_r", 10, 0, 0}, {"press_l", 11, 0, 0}, {"release_l", 12, 0, 0},
    {"press_z", 13, 0, 0}, {"release_z", 14, 0, 0}, {"press_up", 15, 0, 0}, {"release_up", 16, 0, 0},
    {"press_down", 17, 0, 0}, {"release_down", 18, 0, 0}, {"press_right", 19, 0, 0},
    {"release_right", 20, 0, 0}, {"press_left", 21, 0, 0}, {"release_left", 22, 0, 0},
    {"press_start", 23, 0, 0}, {"release_start", 24, 0, 0}, {"release_all", 25, 0, 0},
    {"set_lstick_x", 0x80, 1, 's'}, {"set_lstick_y", 0x81, 1, 's'},
    {"set_cstick_x", 0x82, 1, 's'}, {"set_cstick_y", 0x83, 1, 's'},
    {"set_rtrigger", 0x84, 1, 'u'}, {"set_ltrigger", 0x85, 1, 'u'},
    {"press_a_for", 0x86, 1, 'u'}, {"release_a_for", 0x87, 1, 'u'},
    {"press_b_for", 0x88, 1, 'u'}, {"release_b_for", 0x89, 1, 'u'},
    {"press_x_for", 0x8A, 1, 'u'}, {"release_x_for", 0x8B, 1, 'u'},
    {"press_y_for", 0x8C, 1, 'u'}, {"release_y_for", 0x8D, 1, 'u'},
    {"wait", 0x8E, 1, 'u'}, {"wait_for", 0x8E, 1, 'u'},
    {"lstick_toward_destination", 0x8F, 1, 's'}, {"lstick_x_toward_destination", 0x90, 1, 's'},
    {"lstick_x_forward", 0x91, 1, 's'}, {"wait_if_motion", 0x92, 1, 'u'},
    {"unknown_0x93", 0x93, 1, 'u'}, /* retail stores it in fp->cpu.x18; its meaning is unknown */
    {"lstick_toward_fighter", 0x94, 1, 's'}, {"lstick_x_toward_fighter", 0x95, 1, 's'},
    {"lstick_toward_destination_clamped", 0xC0, 2, 's'},
    {"lstick_x_toward_destination_clamped", 0xC1, 2, 's'},
    {"lstick_forward_clamped", 0xC2, 2, 's'},
};
#define GS_CPU_NCMDS ((int)(sizeof gs_cpu_cmds / sizeof gs_cpu_cmds[0]))
#define GS_CPU_MAXBYTES 255 /* the retail buffer is 0x100 bytes and the Done terminator takes one */

typedef struct { unsigned char b[GS_CPU_MAXBYTES + 1]; int n; int over; } GsCpuBuf;
static void gcb_put(GsCpuBuf *p, int v) {
    if (p->n >= GS_CPU_MAXBYTES) { p->over = 1; return; }
    p->b[p->n++] = (unsigned char)v;
}
static void gcb0(GsCpuBuf *p, int code) { gcb_put(p, code); }
static void gcb1(GsCpuBuf *p, int code, int a) { gcb_put(p, code); gcb_put(p, a); }
static void gcb2(GsCpuBuf *p, int code, int a, int b) { gcb_put(p, code); gcb_put(p, a); gcb_put(p, b); }
static void gcb_wait(GsCpuBuf *p, int frames) {
    while (frames > 255) { gcb1(p, 0x8E, 255); frames -= 255; }
    if (frames > 0) gcb1(p, 0x8E, frames);
}

/* the steps table at idx -> bytes. A step is a command name, or {name, a [, b]}. */
static void gs_cpu_compile(lua_State *L, int idx, GsCpuBuf *out) {
    int i, n, k;
    luaL_checktype(L, idx, LUA_TTABLE);
    n = (int)lua_rawlen(L, idx);
    if (n < 1) luaL_error(L, "cpu_script: no steps");
    for (i = 1; i <= n; ++i) {
        const char *name; int args[2] = {0, 0}, na = 0;
        lua_rawgeti(L, idx, i);
        if (lua_type(L, -1) == LUA_TSTRING) {
            name = lua_tostring(L, -1);
        } else if (lua_istable(L, -1)) {
            lua_rawgeti(L, -1, 1);
            if (lua_type(L, -1) != LUA_TSTRING) luaL_error(L, "cpu_script: step %d: a command name first", i);
            name = lua_tostring(L, -1);
            for (na = 0; na < 2; ++na) {
                lua_rawgeti(L, -2, na + 2);
                if (lua_isnil(L, -1)) { lua_pop(L, 1); break; }
                if (!lua_isnumber(L, -1)) luaL_error(L, "cpu_script: step %d: arguments are numbers", i);
                args[na] = (int)lua_tointeger(L, -1);
                if (!lua_isinteger(L, -1)) args[na] = (int)lua_tonumber(L, -1);
                lua_pop(L, 1);
            }
            lua_rawgeti(L, -2, 5);
            if (!lua_isnil(L, -1)) luaL_error(L, "cpu_script: step %d: at most two arguments", i);
            lua_pop(L, 1);
        } else {
            return (void)luaL_error(L, "cpu_script: step %d: a command name or {name, args}", i);
        }
        for (k = 0; k < GS_CPU_NCMDS && strcmp(gs_cpu_cmds[k].name, name) != 0; ++k) {}
        if (k == GS_CPU_NCMDS) luaL_error(L, "cpu_script: step %d: unknown command \"%s\" (gd.cpu_commands() lists them)", i, name);
        if (na != gs_cpu_cmds[k].nargs)
            luaL_error(L, "cpu_script: step %d: %s takes %d argument(s)", i, name, gs_cpu_cmds[k].nargs);
        {
            int a;
            for (a = 0; a < na; ++a) {
                int lo = gs_cpu_cmds[k].kind == 's' ? -127 : 0, hi = gs_cpu_cmds[k].kind == 's' ? 127 : 255;
                if (args[a] < lo || args[a] > hi)
                    luaL_error(L, "cpu_script: step %d: %s argument %d..%d, got %d", i, name, lo, hi, args[a]);
            }
        }
        gcb_put(out, gs_cpu_cmds[k].code);
        for (k = 0; k < na; ++k) gcb_put(out, args[k]);
        lua_pop(L, na ? 2 : 1);
        if (out->over) luaL_error(L, "cpu_script: more than %d bytes (the retail buffer); split it in two", GS_CPU_MAXBYTES);
    }
}

static int gs_cpu_fbits(float f) { union { float f; int i; } u; u.f = f; return u.i; }
static float gs_cpu_float(int i) { union { float f; int i; } u; u.i = i; return u.f; }

/* the CPU slot and whether the script asked for Nana (opts.sub) */
static int gs_cpu_target_check(lua_State *L, const char *fn, int slot, int sub) {
    int act = gw_ScriptGame_CpuCtl(slot, sub, GCC_G_ACTIVE, 0);
    if (act < 0) return luaL_error(L, "%s: P%d has no %s CPU fighter", fn, slot + 1, sub ? "sub-fighter" : "live");
    if (act == 0) return luaL_error(L, "%s: P%d is not in script mode (gd.cpu_mode(%d, \"script\"))", fn, slot + 1, slot + 1);
    return 1;
}
static int gs_cpu_sub_opt(lua_State *L, int idx) {
    int sub = 0;
    if (lua_istable(L, idx)) { lua_getfield(L, idx, "sub"); sub = lua_toboolean(L, -1); lua_pop(L, 1); }
    return sub;
}

/* gd.cpu_pad(port, {x=,y=,cx=,cy=,l=,r=,buttons=,sub=} [, frames]): the virtual pad, held for
 * `frames` completed logic frames (default 1), then neutral. Cancels a running cpu_script. */
static int l_cpu_pad(lua_State *L) {
    int slot = gs_slot_arg(L, 1), sub, frames, k;
    int v[7];
    luaL_checktype(L, 2, LUA_TTABLE);
    frames = (int)luaL_optinteger(L, 3, 1);
    if (frames < 1) return luaL_error(L, "cpu_pad: frames must be at least 1");
    gs_require_offline(L, "cpu_pad");
    sub = gs_cpu_sub_opt(L, 2);
    lua_getfield(L, 2, "buttons");
    v[0] = (int)gs_parse_buttons(L, lua_gettop(L));
    lua_pop(L, 1);
    v[1] = gs_clampi(gs_field_int(L, 2, "x", 0), -127, 127);
    v[2] = gs_clampi(gs_field_int(L, 2, "y", 0), -127, 127);
    v[3] = gs_clampi(gs_field_int(L, 2, "cx", 0), -127, 127);
    v[4] = gs_clampi(gs_field_int(L, 2, "cy", 0), -127, 127);
    v[5] = gs_clampi(gs_field_int(L, 2, "l", 0), 0, 255);
    v[6] = gs_clampi(gs_field_int(L, 2, "r", 0), 0, 255);
    gs_cpu_target_check(L, "cpu_pad", slot, sub);
    gs_rw_branch();
    for (k = 0; k < 7; ++k) gw_ScriptGame_CpuCtl(slot, sub, GCC_STAGE + k, v[k]);
    lua_pushboolean(L, gw_ScriptGame_CpuCtl(slot, sub, GCC_PAD_COMMIT, frames));
    return 1;
}

static int gs_cpu_load(lua_State *L, const char *fn, int slot, int sub, const GsCpuBuf *buf, int hold) {
    int i, ok = 1;
    gs_require_offline(L, fn);
    gs_cpu_target_check(L, fn, slot, sub);
    gs_rw_branch();
    gw_ScriptGame_CpuCtl(slot, sub, GCC_SCRIPT_BEGIN, 0);
    for (i = 0; i < buf->n && ok; ++i) ok = gw_ScriptGame_CpuCtl(slot, sub, GCC_SCRIPT_BYTE, buf->b[i]);
    if (ok) ok = gw_ScriptGame_CpuCtl(slot, sub, GCC_SCRIPT_RUN, hold ? 0 : 1);
    return ok;
}

/* gd.cpu_script(port, steps [, {hold=bool, sub=bool}]): run retail CPU commands on the port's
 * virtual controller. The interpreter is the game's; its first command runs on the next logic
 * frame. The controller starts neutral and, unless hold, ends neutral. */
static int l_cpu_script(lua_State *L) {
    int slot = gs_slot_arg(L, 1), hold = 0;
    GsCpuBuf buf = {{0}, 0, 0};
    gs_cpu_compile(L, 2, &buf);
    if (lua_istable(L, 3)) { lua_getfield(L, 3, "hold"); hold = lua_toboolean(L, -1); lua_pop(L, 1); }
    lua_pushboolean(L, gs_cpu_load(L, "cpu_script", slot, gs_cpu_sub_opt(L, 3), &buf, hold));
    return 1;
}

/* gd.cpu_dest(port, x, y [, {sub=}]) sets fp->cpu.x54, the point the *_toward_destination commands
 * steer at. gd.cpu_dest(port) reads it back. */
static int l_cpu_dest(lua_State *L) {
    int slot = gs_slot_arg(L, 1), sub = gs_cpu_sub_opt(L, 4);
    if (lua_isnoneornil(L, 2)) {
        int act = gw_ScriptGame_CpuCtl(slot, sub, GCC_G_ACTIVE, 0);
        if (act < 0) return luaL_error(L, "cpu_dest: P%d has no such CPU fighter", slot + 1);
        lua_pushnumber(L, gs_cpu_float(gw_ScriptGame_CpuCtl(slot, sub, GCC_G_DEST_X, 0)));
        lua_pushnumber(L, gs_cpu_float(gw_ScriptGame_CpuCtl(slot, sub, GCC_G_DEST_Y, 0)));
        return 2;
    } else {
        double x = luaL_checknumber(L, 2), y = luaL_checknumber(L, 3);
        if (!(fabs(x) <= 49000.0 && fabs(y) <= 49000.0)) return luaL_error(L, "cpu_dest: finite x, y within +-49000");
        gs_require_offline(L, "cpu_dest");
        gs_cpu_target_check(L, "cpu_dest", slot, sub);
        gs_rw_branch();
        gw_ScriptGame_CpuCtl(slot, sub, GCC_DEST_X, gs_cpu_fbits((float)x));
        gw_ScriptGame_CpuCtl(slot, sub, GCC_DEST_Y, gs_cpu_fbits((float)y));
        lua_pushboolean(L, 1);
        return 1;
    }
}

/* gd.cpu_target(port, other_port | nil): the fighter the toward_fighter commands steer at
 * (default: the first other fighter on stage). */
static int l_cpu_target(lua_State *L) {
    int slot = gs_slot_arg(L, 1), other = lua_isnoneornil(L, 2) ? -1 : gs_slot_arg(L, 2);
    gs_require_offline(L, "cpu_target");
    gs_cpu_target_check(L, "cpu_target", slot, 0);
    gs_rw_branch();
    gw_ScriptGame_CpuCtl(slot, 1, GCC_TARGET, other); /* Nana, when there is one */
    lua_pushboolean(L, gw_ScriptGame_CpuCtl(slot, 0, GCC_TARGET, other));
    return 1;
}

/* gd.cpu_script_done(port [, sub]): true when no command script, pad hold or cpu_goto is in progress */
static int l_cpu_script_done(lua_State *L) {
    int slot = gs_slot_arg(L, 1), sub = lua_toboolean(L, 2);
    int act = gw_ScriptGame_CpuCtl(slot, sub, GCC_G_ACTIVE, 0);
    if (act < 0) return luaL_error(L, "cpu_script_done: P%d has no such CPU fighter", slot + 1);
    lua_pushboolean(L, act == 1 && !gw_ScriptGame_CpuCtl(slot, sub, GCC_G_RUNNING, 0) &&
                       gw_ScriptGame_CpuCtl(slot, sub, GCC_G_PAD_FRAMES, 0) == 0 &&
                       gw_ScriptGame_CpuCtl(slot, sub, GCC_G_GOTO_STATE, 0) != 1);
    return 1;
}

/* gd.cpu_script_status(port [, sub]): the controller as the engine will read it */
static int l_cpu_script_status(lua_State *L) {
    int slot = gs_slot_arg(L, 1), sub = lua_toboolean(L, 2);
    int act = gw_ScriptGame_CpuCtl(slot, sub, GCC_G_ACTIVE, 0);
    if (act < 0) return luaL_error(L, "cpu_script_status: P%d has no such CPU fighter", slot + 1);
    lua_createtable(L, 0, 16);
    gs_setbool(L, "script_mode", act == 1);
    gs_setbool(L, "running", gw_ScriptGame_CpuCtl(slot, sub, GCC_G_RUNNING, 0) != 0);
    gs_setint(L, "completed", gw_ScriptGame_CpuCtl(slot, sub, GCC_G_DONE_COUNT, 0));
    gs_setint(L, "pad_frames", gw_ScriptGame_CpuCtl(slot, sub, GCC_G_PAD_FRAMES, 0));
    gs_setint(L, "buttons", gw_ScriptGame_CpuCtl(slot, sub, GCC_G_BUTTONS, 0));
    gs_setint(L, "x", gw_ScriptGame_CpuCtl(slot, sub, GCC_G_LX, 0));
    gs_setint(L, "y", gw_ScriptGame_CpuCtl(slot, sub, GCC_G_LY, 0));
    gs_setint(L, "cx", gw_ScriptGame_CpuCtl(slot, sub, GCC_G_CX, 0));
    gs_setint(L, "cy", gw_ScriptGame_CpuCtl(slot, sub, GCC_G_CY, 0));
    gs_setint(L, "l", gw_ScriptGame_CpuCtl(slot, sub, GCC_G_LTRIG, 0));
    gs_setint(L, "r", gw_ScriptGame_CpuCtl(slot, sub, GCC_G_RTRIG, 0));
    gs_setint(L, "target", gw_ScriptGame_CpuCtl(slot, sub, GCC_G_TARGET, 0) + 1); /* 0 = first other */
    gs_setint(L, "script_pos", gw_ScriptGame_CpuCtl(slot, sub, GCC_G_CSP, 0));     /* -1 = none */
    gs_setint(L, "wait_left", gw_ScriptGame_CpuCtl(slot, sub, GCC_G_DURATION, 0));
    return 1;
}

/* gd.cpu_goto(port, x, y [, {radius=2, timeout=900, sub=}]): the rule-based mover (the game half
 * documents the rules). It runs inside the simulation until it arrives, is blocked, or the fighter
 * falls to a KO; gd.cpu_goto_status reads which. Cancelled by cpu_pad, cpu_script and cpu_cancel. */
static int l_cpu_goto(lua_State *L) {
    int slot = gs_slot_arg(L, 1), sub = gs_cpu_sub_opt(L, 4);
    double x = luaL_checknumber(L, 2), y = luaL_checknumber(L, 3), radius = 2.0;
    int timeout = 900;
    if (!(fabs(x) <= 49000.0 && fabs(y) <= 49000.0)) return luaL_error(L, "cpu_goto: finite x, y within +-49000");
    if (lua_istable(L, 4)) {
        lua_getfield(L, 4, "radius"); if (lua_isnumber(L, -1)) radius = lua_tonumber(L, -1); lua_pop(L, 1);
        timeout = gs_field_int(L, 4, "timeout", 900);
    }
    if (!(radius >= 0.5 && radius <= 200.0)) return luaL_error(L, "cpu_goto: radius 0.5..200");
    if (timeout < 1 || timeout > 36000) return luaL_error(L, "cpu_goto: timeout 1..36000 frames");
    gs_require_offline(L, "cpu_goto");
    gs_cpu_target_check(L, "cpu_goto", slot, sub);
    gs_rw_branch();
    gw_ScriptGame_CpuCtl(slot, sub, GCC_GOTO_X, gs_cpu_fbits((float)x));
    gw_ScriptGame_CpuCtl(slot, sub, GCC_GOTO_Y, gs_cpu_fbits((float)y));
    gw_ScriptGame_CpuCtl(slot, sub, GCC_GOTO_RADIUS, gs_cpu_fbits((float)radius));
    lua_pushboolean(L, gw_ScriptGame_CpuCtl(slot, sub, GCC_GOTO_START, timeout));
    return 1;
}

/* gd.cpu_goto_status(port [, sub]): "idle", "running", "arrived", "blocked" or "fell", and the
 * frames the walk has taken */
static int l_cpu_goto_status(lua_State *L) {
    static const char *const names[] = {"idle", "running", "arrived", "blocked", "fell"};
    int slot = gs_slot_arg(L, 1), sub = lua_toboolean(L, 2);
    int st = gw_ScriptGame_CpuCtl(slot, sub, GCC_G_GOTO_STATE, 0);
    if (st < 0) return luaL_error(L, "cpu_goto_status: P%d has no such CPU fighter", slot + 1);
    lua_pushstring(L, names[st > 4 ? 0 : st]);
    lua_pushinteger(L, gw_ScriptGame_CpuCtl(slot, sub, GCC_G_GOTO_TICKS, 0));
    return 2;
}

/* gd.cpu_cancel(port [, sub]): stop whatever the controller is doing and go neutral */
static int l_cpu_cancel(lua_State *L) {
    int slot = gs_slot_arg(L, 1), sub = lua_toboolean(L, 2);
    gs_require_offline(L, "cpu_cancel");
    gs_cpu_target_check(L, "cpu_cancel", slot, sub);
    gs_rw_branch();
    lua_pushboolean(L, gw_ScriptGame_CpuCtl(slot, sub, GCC_CANCEL, 0));
    return 1;
}

/* gd.cpu_commands(): {name = {code=, args=}} for every command gd.cpu_script accepts */
static int l_cpu_commands(lua_State *L) {
    int k;
    lua_createtable(L, 0, GS_CPU_NCMDS);
    for (k = 0; k < GS_CPU_NCMDS; ++k) {
        lua_createtable(L, 0, 2);
        gs_setint(L, "code", gs_cpu_cmds[k].code);
        gs_setint(L, "args", gs_cpu_cmds[k].nargs);
        lua_setfield(L, -2, gs_cpu_cmds[k].name);
    }
    return 1;
}

/* ---- technique macros, as command sequences timed from the fighter's own attributes --------- */
enum { CA_JUMP_SQUAT, CA_HOP_V, CA_JUMP_V, CA_GRAVITY, CA_TERMINAL, CA_LAND_N, CA_LAND_F, CA_LAND_B,
       CA_LAND_HI, CA_LAND_LW, CA_LAND_NORMAL, CA_DASH_INIT, CA_DASH_MAX, CA_WALK_MAX, CA_MAX_JUMPS,
       CA_FAST_FALL, CA_AIR_MAX, CA_FRICTION, CA_GROUND_TO_AIR, CA_JUMP_H, CA_AIRDODGE_LAG,
       CA_AIR_DRIFT, CA_SHIELD, CA_TECH_WINDOW, CA_COUNT };
static const char *const gs_cpu_attr_names[CA_COUNT] = {
    "jump_squat", "hop_v", "jump_v", "gravity", "terminal_velocity", "landing_nair", "landing_fair",
    "landing_bair", "landing_uair", "landing_dair", "landing_normal", "dash_initial", "dash_max",
    "walk_max", "max_jumps", "fast_fall", "air_max_horizontal", "ground_friction",
    "ground_to_air_momentum", "jump_h_initial", "airdodge_landing", "air_drift_max", "shield_size",
    "tech_window"};
static float gs_cpu_attr(int slot, int id) { return gs_cpu_float(gw_ScriptGame_CpuAttr(slot, id)); }

/* gd.cpu_attrs(port): the attributes the macros are timed from */
static int l_cpu_attrs(lua_State *L) {
    int slot = gs_slot_arg(L, 1), i;
    lua_createtable(L, 0, CA_COUNT + 2);
    if (gw_ScriptGame_FighterI(slot, SI_PRESENT) != 1) return 1;
    for (i = 0; i < CA_COUNT; ++i) gs_setnum(L, gs_cpu_attr_names[i], gs_cpu_attr(slot, i));
    return 1;
}

/* Frames a short hop stays in the air: the fighter's own hop velocity, gravity and terminal
 * velocity, stepped as the engine steps them (position, then velocity). */
static int gs_cpu_hop_air_frames(float v0, float g, float term) {
    float y = 0.0f, v = v0;
    int n = 0;
    do { y += v; v -= g; if (v < -term) v = -term; ++n; } while (y > 0.0f && n < 240);
    return n;
}

#ifndef GS_CPU_WD_OFFSET
#define GS_CPU_WD_OFFSET 0   /* airdodge input frame, relative to the end of the jump squat */
#endif
#ifndef GS_CPU_LAND_DELAY
#define GS_CPU_LAND_DELAY 2  /* ticks from the height reaching zero to the landing action */
#endif
#ifndef GS_CPU_LC_LEAD
#define GS_CPU_LC_LEAD 3     /* L-cancel press this many frames before the predicted landing */
#endif
#ifndef GS_CPU_AERIAL_AT
#define GS_CPU_AERIAL_AT 2   /* aerial input this many frames after take-off */
#endif

typedef struct { int sign; int rel; } GsCpuDir;
/* "forward"/"back" follow the facing when the command runs; "left"/"right"/-1/1 are absolute */
static GsCpuDir gs_cpu_dir(lua_State *L, int idx, const char *fn) {
    GsCpuDir d = {1, 1};
    if (lua_isnoneornil(L, idx)) return d;
    if (lua_type(L, idx) == LUA_TSTRING) {
        const char *s = lua_tostring(L, idx);
        if (!strcmp(s, "in")) return d; /* tech: handled by the macro */
        if (!strcmp(s, "forward")) return d;
        if (!strcmp(s, "back")) { d.sign = -1; return d; }
        if (!strcmp(s, "left")) { d.sign = -1; d.rel = 0; return d; }
        if (!strcmp(s, "right")) { d.rel = 0; return d; }
    } else if (lua_isinteger(L, idx) && (lua_tointeger(L, idx) == 1 || lua_tointeger(L, idx) == -1)) {
        d.sign = (int)lua_tointeger(L, idx); d.rel = 0; return d;
    }
    luaL_error(L, "%s: dir is \"forward\", \"back\", \"left\", \"right\", -1 or 1", fn);
    return d;
}
static void gcb_stick_x(GsCpuBuf *p, GsCpuDir d, int mag) {
    if (d.rel) gcb1(p, 0x91, d.sign * mag); else gcb1(p, 0x80, d.sign * mag);
}
static int gs_cpu_opt(lua_State *L, int idx, const char *k, int def) {
    return idx ? gs_field_int(L, idx, k, def) : def;
}

/* airdodge angle: `deg` below horizontal, full stick, toward dir */
static void gcb_airdodge(GsCpuBuf *p, GsCpuDir d, int deg) {
    double rad = deg * 3.14159265358979 / 180.0;
    int x = (int)(127.0 * cos(rad) + 0.5), y = (int)(-127.0 * sin(rad) - 0.5);
    gcb_stick_x(p, d, x);
    gcb1(p, 0x81, y);
    gcb0(p, 11); /* press L */
    gcb1(p, 0x8E, 2);
    gcb0(p, 12);
}

/* macros: short_hop, full_hop, dash, dash_dance, wavedash, waveland, lcancel_aerial, shield,
 * perfect_shield, tech, jump_cancel_grab. Returns false for a name it does not know. */
static int gs_cpu_macro_build(lua_State *L, int slot, const char *name, int opts, GsCpuBuf *p) {
    int J = (int)(gs_cpu_attr(slot, CA_JUMP_SQUAT) + 0.5f);
    GsCpuDir dir = {1, 1};
    if (opts) {
        lua_getfield(L, opts, "dir");
        if (!lua_isnil(L, -1)) dir = gs_cpu_dir(L, lua_gettop(L), "cpu_macro");
        lua_pop(L, 1);
    }
    if (gw_ScriptGame_FighterI(slot, SI_PRESENT) != 1) luaL_error(L, "cpu_macro: P%d has no fighter", slot + 1);
    if (J < 1) J = 1;
    if (!strcmp(name, "short_hop")) {
        gcb0(p, 5); gcb1(p, 0x8E, 1); gcb0(p, 6); gcb_wait(p, J);
    } else if (!strcmp(name, "full_hop")) {
        gcb0(p, 5); gcb_wait(p, J + 2); gcb0(p, 6);
    } else if (!strcmp(name, "dash")) {
        gcb_stick_x(p, dir, 127); gcb_wait(p, gs_cpu_opt(L, opts, "frames", 12));
        gcb1(p, 0x80, 0);
    } else if (!strcmp(name, "dash_dance")) {
        int cycles = gs_clampi(gs_cpu_opt(L, opts, "cycles", 3), 1, 12), hold = gs_clampi(gs_cpu_opt(L, opts, "hold", 6), 1, 40), c;
        for (c = 0; c < cycles; ++c) {
            gcb_stick_x(p, dir, 127); gcb_wait(p, hold);
            gcb_stick_x(p, dir, -127); gcb_wait(p, hold);
        }
        gcb1(p, 0x80, 0);
    } else if (!strcmp(name, "wavedash")) {
        gcb0(p, 5); gcb1(p, 0x8E, 1); gcb0(p, 6);
        gcb_wait(p, J - 1 + gs_cpu_opt(L, opts, "offset", GS_CPU_WD_OFFSET));
        gcb_airdodge(p, dir, gs_clampi(gs_cpu_opt(L, opts, "angle", 20), 5, 80));
        gcb1(p, 0x81, 0); gcb1(p, 0x80, 0);
    } else if (!strcmp(name, "waveland")) {
        gcb_airdodge(p, dir, gs_clampi(gs_cpu_opt(L, opts, "angle", 20), 5, 80));
        gcb1(p, 0x81, 0); gcb1(p, 0x80, 0);
    } else if (!strcmp(name, "lcancel_aerial")) {
        /* Ticks are the macro's own: the first command runs at tick 1 (the jump starts on it),
         * the fighter is first airborne on tick 1+J, and the engine lands it GS_CPU_LAND_DELAY
         * ticks after the integrated height first reaches zero (measured, Fox, Final Destination). */
        const char *aerial = "nair";
        int sim = gs_cpu_hop_air_frames(gs_cpu_attr(slot, CA_HOP_V), gs_cpu_attr(slot, CA_GRAVITY), gs_cpu_attr(slot, CA_TERMINAL));
        int landing = J + sim + GS_CPU_LAND_DELAY;
        int lcancel = gs_cpu_opt(L, opts, "lcancel", 1), at = gs_clampi(gs_cpu_opt(L, opts, "at", GS_CPU_AERIAL_AT), 0, 60);
        int lead = gs_clampi(gs_cpu_opt(L, opts, "lead", GS_CPU_LC_LEAD), 1, 20);
        int face = gw_ScriptGame_FighterF(slot, SF_FACING) < 0.0f ? -1 : 1, t = 1;
        if (opts) { lua_getfield(L, opts, "aerial"); if (lua_type(L, -1) == LUA_TSTRING) aerial = lua_tostring(L, -1); lua_pop(L, 1); }
        gcb0(p, 5); gcb1(p, 0x8E, 1); gcb0(p, 6); t = 2;
        gcb_wait(p, 1 + J + at - t); t = 1 + J + at;
        if (!strcmp(aerial, "nair")) { gcb0(p, 1); gcb1(p, 0x8E, 1); gcb0(p, 2); }
        else {
            int cx = 0, cy = 0;
            if (!strcmp(aerial, "fair")) cx = face * dir.sign * 127;
            else if (!strcmp(aerial, "bair")) cx = -face * dir.sign * 127;
            else if (!strcmp(aerial, "uair")) cy = 127;
            else if (!strcmp(aerial, "dair")) cy = -127;
            else luaL_error(L, "cpu_macro lcancel_aerial: aerial is nair, fair, bair, uair or dair");
            gcb1(p, 0x82, cx); gcb1(p, 0x83, cy); gcb1(p, 0x8E, 1); gcb1(p, 0x82, 0); gcb1(p, 0x83, 0);
        }
        t += 1;
        if (landing - lead > t) gcb_wait(p, landing - lead - t);
        if (lcancel) { gcb0(p, 11); gcb1(p, 0x8E, 2); gcb0(p, 12); }
    } else if (!strcmp(name, "shield")) {
        gcb0(p, 11); gcb_wait(p, gs_clampi(gs_cpu_opt(L, opts, "frames", 30), 1, 600)); gcb0(p, 12);
    } else if (!strcmp(name, "perfect_shield")) {
        /* press shield `at` frames from now (the caller times it to the incoming hit), hold briefly */
        gcb_wait(p, gs_clampi(gs_cpu_opt(L, opts, "at", 0), 0, 600));
        gcb0(p, 11); gcb_wait(p, gs_clampi(gs_cpu_opt(L, opts, "hold", 12), 1, 600)); gcb0(p, 12);
    } else if (!strcmp(name, "tech")) {
        /* shield press toward dir; `at` frames from now. dir "in" = no stick (tech in place) */
        int tech_in = 0;
        if (opts) { lua_getfield(L, opts, "dir"); tech_in = lua_type(L, -1) == LUA_TSTRING && !strcmp(lua_tostring(L, -1), "in"); lua_pop(L, 1); }
        /* The game reads the stick when it LANDS the fighter, up to its tech window (ftCommonData
         * x250: ftCo_800986B0) after the shield press, not when the press happens: a stick let go
         * three frames later made every tech that landed later an in-place tech, in either
         * direction. So the stick is held for the whole window; `window` overrides it. */
        int win = gs_clampi(gs_cpu_opt(L, opts, "window", (int)(gs_cpu_attr(slot, CA_TECH_WINDOW) + 0.5f)), 3, 120);
        gcb_wait(p, gs_clampi(gs_cpu_opt(L, opts, "at", 0), 0, 600));
        if (!tech_in) gcb_stick_x(p, dir, 127);
        gcb0(p, 11); gcb1(p, 0x8E, 3); gcb0(p, 12);
        if (!tech_in) { gcb_wait(p, win - 3); gcb1(p, 0x80, 0); }
    } else if (!strcmp(name, "jump_cancel_grab")) {
        gcb0(p, 5); gcb_wait(p, gs_clampi(gs_cpu_opt(L, opts, "z_at", 1), 1, J > 1 ? J - 1 : 1));
        gcb0(p, 13); gcb1(p, 0x8E, 2); gcb0(p, 14); gcb0(p, 6);
    } else {
        return 0;
    }
    if (p->over) luaL_error(L, "cpu_macro: macro too long");
    return 1;
}

/* gd.cpu_macro(port, name [, opts]): build a technique macro for this fighter and run it.
 * opts: dir ("forward" default, "back", "left", "right"), hold, sub, plus per-macro fields. */
static int l_cpu_macro(lua_State *L) {
    int slot = gs_slot_arg(L, 1);
    const char *name = luaL_checkstring(L, 2);
    GsCpuBuf buf = {{0}, 0, 0};
    int hold = 0;
    if (!lua_isnoneornil(L, 3)) luaL_checktype(L, 3, LUA_TTABLE);
    if (!gs_cpu_macro_build(L, slot, name, lua_istable(L, 3) ? 3 : 0, &buf))
        return luaL_error(L, "cpu_macro: unknown macro \"%s\" (short_hop full_hop dash dash_dance wavedash waveland lcancel_aerial shield perfect_shield tech jump_cancel_grab)", name);
    if (lua_istable(L, 3)) { lua_getfield(L, 3, "hold"); hold = lua_toboolean(L, -1); lua_pop(L, 1); }
    lua_pushboolean(L, gs_cpu_load(L, "cpu_macro", slot, gs_cpu_sub_opt(L, 3), &buf, hold));
    lua_pushinteger(L, buf.n);
    return 2;
}
