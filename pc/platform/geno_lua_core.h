/* geno_lua_core.h - Geno's fighter-Lua domain (slice 5, docs/geno.md section 23). NATIVE half, header-only so the
 * registry (geno_registry.c) and the isolated native test (pc/tests/geno_lua_test.c, `build.sh --native-test geno-lua`)
 * compile the same code. It knows nothing of the registry or of the fighter: a profile owns one gn_lua, a call gets
 * a GenoLuaIo (GENO_LUA_IO_WORDS big-endian game words, geno.h) and returns a fault code.
 *
 * WHAT IT GUARANTEES (each one is a test in geno_lua_test.c):
 *   - A module is a chunk that returns a table of FUNCTIONS and nothing else. It is loaded with an environment that is a
 *     read-only proxy over a fixed allowlist: type, select, ipairs, assert, error, freeze, and math.{abs,min,max,floor,
 *     ceil,sqrt,tointeger,huge,pi}. There is no pairs/next (iteration order depends on a per-state string seed), no
 *     pcall (a budget fault must not be catchable), no load/require/os/io/string/table/debug/coroutine, no
 *     setmetatable/getmetatable/rawset/rawget, no random, no clock. Libraries are NOT opened: the allowlist is built here.
 *   - A function may not keep state: its bytecode (and that of every function nested in it) is scanned and refused if it
 *     contains OP_SETUPVAL (assigns a captured variable) or OP_SETTABUP (assigns a global or a captured table's field), and
 *     every upvalue it captures must be nil/boolean/number/string, a function that passes the same test, or a table made
 *     by freeze() (a deep read-only proxy). So nothing survives between calls except what is in the typed state block.
 *   - Every call builds a fresh ctx table, runs under an instruction budget (GENO_LUA_INSN_BUDGET, counted in steps of
 *     GENO_LUA_INSN_STEP) and a heap budget (GENO_LUA_HEAP_BUDGET above the heap at the start of the call), with the
 *     collector stopped during the call and a full collection after it: the heap at the start of every call is the same.
 *   - The ONLY state is ctx.state, a proxy over the profile's declared slots, read and written straight in the io block.
 *     The caller keeps the written words only when the call returned 0.
 * Nothing here reads the clock, a random source, or anything outside the io block. Lua numbers are IEEE doubles; slots
 * narrow them to s32/f32.
 */
#ifndef GENO_LUA_CORE_H
#define GENO_LUA_CORE_H

#include <math.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "gw.h"
#include "../geno/geno.h"

#include "../third_party/lua-5.4.7/src/lua.h"
#include "../third_party/lua-5.4.7/src/lauxlib.h"
#include "../third_party/lua-5.4.7/src/lualib.h"
/* the vendored 5.4.7's internals, for the closure scan only (the version is pinned in pc/third_party/lua-5.4.7) */
#include "../third_party/lua-5.4.7/src/lobject.h"
#include "../third_party/lua-5.4.7/src/lopcodes.h"

#define GLUA_MAX_FNS 24
#define GLUA_NAME 32
#define GLUA_SLOT_NAME 24
#define GLUA_LOAD_HEAP (4u * 1024u * 1024u)

enum { GLUA_INT = 0, GLUA_FLOAT = 1, GLUA_BOOL = 2 };

typedef struct gn_lua {
    lua_State* L;
    size_t used, limit;        /* heap bytes in use; 0 limit = unlimited */
    int insns, insn_limit;
    int fault;                 /* GENO_LUA_FAULT_* of the call in progress */
    uint32_t* io;              /* the call in progress */
    int ncmd;
    int nslot;
    char slot_name[GENO_LUA_STATE_SLOTS][GLUA_SLOT_NAME];
    int slot_type[GENO_LUA_STATE_SLOTS];
    int ntarget;
    char target_name[GENO_MAX_STATES][GLUA_NAME];
    int nfn;
    char fn_name[GLUA_MAX_FNS][GLUA_NAME];
    int fn_ref[GLUA_MAX_FNS];
    char err[240];             /* the last refusal or fault, for the log */
    unsigned calls, faults;    /* diagnostics only */
} gn_lua;

static gn_lua* glua_of(lua_State* L) { return *(gn_lua**) lua_getextraspace(L); }

static void* glua_alloc(void* ud, void* ptr, size_t osize, size_t nsize)
{
    gn_lua* g = (gn_lua*) ud;
    size_t old = ptr ? osize : 0;
    void* p;
    if (nsize == 0) {
        if (ptr) {
            g->used -= osize;
            free(ptr);
        }
        return NULL;
    }
    if (nsize > old && g->limit && g->used + (nsize - old) > g->limit) {
        if (!g->fault) g->fault = GENO_LUA_FAULT_HEAP;
        return NULL;
    }
    p = realloc(ptr, nsize);
    if (p == NULL) return NULL;
    g->used = g->used - old + nsize;
    return p;
}

static void glua_hook(lua_State* L, lua_Debug* ar)
{
    gn_lua* g = glua_of(L);
    (void) ar;
    g->insns += GENO_LUA_INSN_STEP;
    if (g->insns > g->insn_limit) {
        if (!g->fault) g->fault = GENO_LUA_FAULT_INSNS;
        luaL_error(L, "instruction budget exceeded");
    }
}

static int glua_fault(lua_State* L, int code, const char* msg)
{
    gn_lua* g = glua_of(L);
    if (!g->fault) g->fault = code;
    return luaL_error(L, "%s", msg);
}

/* ---- frozen tables --------------------------------------------------------------------------------- */

static int glua_is_frozen(lua_State* L, int idx)
{
    int r = 0;
    if (lua_getmetatable(L, idx)) {
        lua_getfield(L, -1, "__gnfrozen");
        r = lua_toboolean(L, -1);
        lua_pop(L, 2);
    }
    return r;
}

static int glua_frozen_len(lua_State* L)
{
    lua_pushinteger(L, (lua_Integer) lua_rawlen(L, lua_upvalueindex(1)));
    return 1;
}

static int glua_frozen_newindex(lua_State* L)
{
    return glua_fault(L, GENO_LUA_FAULT_ERROR, "attempt to modify a frozen table");
}

/* Pushes the frozen form of the value at idx (a table becomes a read-only proxy over a deep copy; anything else is
 * pushed as it is). Returns 0 (with a message pushed) for a table key, or nesting deeper than 8. */
static int glua_freeze_val(lua_State* L, int idx, int depth)
{
    int data, proxy;
    idx = lua_absindex(L, idx);
    if (lua_type(L, idx) != LUA_TTABLE || glua_is_frozen(L, idx)) {
        lua_pushvalue(L, idx);
        return 1;
    }
    if (depth > 8) {
        lua_pushliteral(L, "freeze: tables nested deeper than 8");
        return 0;
    }
    luaL_checkstack(L, 10, "freeze");
    lua_newtable(L);
    data = lua_gettop(L);
    lua_pushnil(L);
    while (lua_next(L, idx)) {
        if (lua_type(L, -2) == LUA_TTABLE || lua_type(L, -2) == LUA_TFUNCTION) {
            lua_pushliteral(L, "freeze: a table or function used as a key");
            return 0;
        }
        if (!glua_freeze_val(L, -1, depth + 1)) return 0;
        lua_pushvalue(L, -3); /* key */
        lua_insert(L, -2);
        lua_rawset(L, data); /* data[key] = frozen value */
        lua_pop(L, 1);       /* the original value; the key stays for lua_next */
    }
    lua_newtable(L);
    proxy = lua_gettop(L);
    lua_newtable(L); /* the metatable */
    lua_pushvalue(L, data);
    lua_setfield(L, -2, "__index");
    lua_pushvalue(L, data);
    lua_pushcclosure(L, glua_frozen_len, 1);
    lua_setfield(L, -2, "__len");
    lua_pushcfunction(L, glua_frozen_newindex);
    lua_setfield(L, -2, "__newindex");
    lua_pushboolean(L, 1);
    lua_setfield(L, -2, "__gnfrozen");
    lua_pushboolean(L, 0);
    lua_setfield(L, -2, "__metatable");
    lua_setmetatable(L, proxy);
    lua_remove(L, data);
    return 1;
}

static int glua_freeze_fn(lua_State* L)
{
    luaL_checktype(L, 1, LUA_TTABLE);
    if (!glua_freeze_val(L, 1, 0)) return lua_error(L);
    return 1;
}

/* ---- the ctx: typed state, commands ------------------------------------------------------------------ */

static int glua_slot_find(gn_lua* g, const char* k)
{
    int i;
    for (i = 0; i < g->nslot; i++) {
        if (strcmp(g->slot_name[i], k) == 0) return i;
    }
    return -1;
}

static float glua_bits_f(uint32_t u)
{
    float f;
    memcpy(&f, &u, 4);
    return f;
}

static uint32_t glua_f_bits(float f)
{
    uint32_t u;
    memcpy(&u, &f, 4);
    return u;
}

static int glua_state_index(lua_State* L)
{
    gn_lua* g = glua_of(L);
    int slot;
    uint32_t w;
    if (lua_type(L, 2) != LUA_TSTRING) return glua_fault(L, GENO_LUA_FAULT_STATE, "ctx.state: the key is not a name");
    slot = glua_slot_find(g, lua_tostring(L, 2));
    if (slot < 0) return glua_fault(L, GENO_LUA_FAULT_STATE, "ctx.state: no such slot is declared");
    w = gw_r32(&g->io[GENO_LUA_IO_STATE + slot]);
    switch (g->slot_type[slot]) {
    case GLUA_INT: lua_pushinteger(L, (lua_Integer) (int32_t) w); break;
    case GLUA_FLOAT: lua_pushnumber(L, (lua_Number) glua_bits_f(w)); break;
    default: lua_pushboolean(L, w != 0); break;
    }
    return 1;
}

static int glua_state_newindex(lua_State* L)
{
    gn_lua* g = glua_of(L);
    int slot;
    uint32_t w = 0;
    if (lua_type(L, 2) != LUA_TSTRING) return glua_fault(L, GENO_LUA_FAULT_STATE, "ctx.state: the key is not a name");
    slot = glua_slot_find(g, lua_tostring(L, 2));
    if (slot < 0) return glua_fault(L, GENO_LUA_FAULT_STATE, "ctx.state: no such slot is declared");
    switch (g->slot_type[slot]) {
    case GLUA_INT: {
        int ok = 0;
        lua_Integer v = 0;
        if (lua_type(L, 3) == LUA_TNUMBER) v = lua_tointegerx(L, 3, &ok);
        if (!ok || v < INT32_MIN || v > INT32_MAX) {
            return glua_fault(L, GENO_LUA_FAULT_STATE, "ctx.state: an int slot takes an integer that fits 32 bits");
        }
        w = (uint32_t) (int32_t) v;
        break;
    }
    case GLUA_FLOAT: {
        lua_Number v;
        if (lua_type(L, 3) != LUA_TNUMBER) return glua_fault(L, GENO_LUA_FAULT_STATE, "ctx.state: a float slot takes a number");
        v = lua_tonumber(L, 3);
        if (!(v == v) || fabs(v) > 3.0e38) return glua_fault(L, GENO_LUA_FAULT_STATE, "ctx.state: a float slot takes a finite value");
        w = glua_f_bits((float) v);
        break;
    }
    default:
        if (lua_type(L, 3) != LUA_TBOOLEAN) return glua_fault(L, GENO_LUA_FAULT_STATE, "ctx.state: a bool slot takes true or false");
        w = lua_toboolean(L, 3) ? 1u : 0u;
        break;
    }
    gw_w32(&g->io[GENO_LUA_IO_STATE + slot], w);
    return 0;
}

static int glua_cmd(lua_State* L, uint32_t op, uint32_t a, uint32_t b)
{
    gn_lua* g = glua_of(L);
    uint32_t* c;
    if (g->ncmd >= GENO_LUA_MAX_CMDS) return glua_fault(L, GENO_LUA_FAULT_CMDS, "more commands than one call may queue");
    c = &g->io[GENO_LUA_IO_CMDS + 3 * g->ncmd];
    gw_w32(&c[0], op);
    gw_w32(&c[1], a);
    gw_w32(&c[2], b);
    g->ncmd++;
    gw_w32(&g->io[GENO_LUA_IO_NCMDS], (uint32_t) g->ncmd);
    return 0;
}

static float glua_checkfinite(lua_State* L, int arg, double lim, const char* what)
{
    lua_Number v;
    if (lua_type(L, arg) != LUA_TNUMBER) glua_fault(L, GENO_LUA_FAULT_COMMAND, what);
    v = lua_tonumber(L, arg);
    if (!(v == v) || fabs(v) > lim) glua_fault(L, GENO_LUA_FAULT_COMMAND, what);
    return (float) v;
}

static int glua_go(lua_State* L)
{
    gn_lua* g = glua_of(L);
    const char* name;
    int i;
    if (lua_type(L, 1) != LUA_TSTRING) return glua_fault(L, GENO_LUA_FAULT_COMMAND, "ctx.go takes a state name");
    name = lua_tostring(L, 1);
    if (strcmp(name, "auto") == 0) return glua_cmd(L, GENO_LUA_CMD_GO, GENO_TGT_AUTO, 0);
    if (strcmp(name, "helpless") == 0) return glua_cmd(L, GENO_LUA_CMD_GO, GENO_TGT_HELPLESS, 0);
    for (i = 0; i < g->ntarget; i++) {
        if (strcmp(g->target_name[i], name) == 0) return glua_cmd(L, GENO_LUA_CMD_GO, GENO_TARGET(GENO_TGT_GENO, i), 0);
    }
    return glua_fault(L, GENO_LUA_FAULT_COMMAND, "ctx.go: no such state");
}

static int glua_velocity(lua_State* L)
{
    float f = glua_checkfinite(L, 1, 1000.0, "ctx.velocity takes finite numbers (forward, up)");
    float u = glua_checkfinite(L, 2, 1000.0, "ctx.velocity takes finite numbers (forward, up)");
    return glua_cmd(L, GENO_LUA_CMD_VELOCITY, glua_f_bits(f), glua_f_bits(u));
}

static int glua_hitbox_damage(lua_State* L)
{
    float d;
    lua_Integer m;
    int ok = 0;
    if (lua_type(L, 1) == LUA_TNUMBER) m = lua_tointegerx(L, 1, &ok);
    else m = 0;
    if (!ok || m < 1 || m > 15) return glua_fault(L, GENO_LUA_FAULT_COMMAND, "ctx.hitbox_damage: the mask is 1..15 (bit n = hitbox slot n)");
    d = glua_checkfinite(L, 2, 1000.0, "ctx.hitbox_damage takes a finite damage");
    if (d < 0.0f) return glua_fault(L, GENO_LUA_FAULT_COMMAND, "ctx.hitbox_damage: damage is not negative");
    return glua_cmd(L, GENO_LUA_CMD_HITBOX_DAMAGE, (uint32_t) m, glua_f_bits(d));
}

static int glua_loop(lua_State* L)
{
    return glua_cmd(L, GENO_LUA_CMD_LOOP, 0, 0);
}

/* ---- closure scan -------------------------------------------------------------------------------------- */

static int glua_scan_proto(gn_lua* g, const Proto* p, const char* fn)
{
    int i;
    for (i = 0; i < p->sizecode; i++) {
        OpCode op = GET_OPCODE(p->code[i]);
        if (op == OP_SETUPVAL) {
            snprintf(g->err, sizeof g->err, "function %s assigns a captured variable (keep state in ctx.state)", fn);
            return 0;
        }
        if (op == OP_SETTABUP) {
            snprintf(g->err, sizeof g->err, "function %s assigns a global or a field of a captured table (keep state in ctx.state)", fn);
            return 0;
        }
    }
    for (i = 0; i < p->sizep; i++) {
        if (!glua_scan_proto(g, p->p[i], fn)) return 0;
    }
    return 1;
}

typedef struct {
    const void* seen[16];
    int n;
} glua_seen;

static int glua_check_fn(gn_lua* g, int idx, const char* fn, int depth, glua_seen* sn)
{
    lua_State* L = g->L;
    const LClosure* cl;
    int i;
    idx = lua_absindex(L, idx);
    if (lua_iscfunction(L, idx)) return 1; /* the allowlist's own functions */
    cl = (const LClosure*) lua_topointer(L, idx);
    for (i = 0; i < sn->n; i++) {
        if (sn->seen[i] == cl) return 1;
    }
    if (depth > 6 || sn->n >= 16) {
        snprintf(g->err, sizeof g->err, "function %s captures functions nested too deep", fn);
        return 0;
    }
    sn->seen[sn->n++] = cl;
    if (!glua_scan_proto(g, cl->p, fn)) return 0;
    for (i = 1;; i++) {
        const char* nm = lua_getupvalue(L, idx, i);
        int t, ok = 1;
        if (nm == NULL) break;
        t = lua_type(L, -1);
        if (strcmp(nm, "_ENV") == 0) {
            /* our proxy; nothing to check */
        } else if (t == LUA_TNIL || t == LUA_TBOOLEAN || t == LUA_TNUMBER || t == LUA_TSTRING) {
        } else if (t == LUA_TFUNCTION) {
            ok = glua_check_fn(g, -1, fn, depth + 1, sn);
        } else if (t == LUA_TTABLE) {
            if (!glua_is_frozen(L, -1)) {
                snprintf(g->err, sizeof g->err, "function %s captures the mutable table '%s' (wrap constants in freeze(), or pass data through ctx)", fn, nm);
                ok = 0;
            }
        } else {
            snprintf(g->err, sizeof g->err, "function %s captures '%s', which is not a plain value", fn, nm);
            ok = 0;
        }
        lua_pop(L, 1);
        if (!ok) return 0;
    }
    return 1;
}

/* ---- the domain ---------------------------------------------------------------------------------------- */

static void glua_free(gn_lua* g)
{
    if (g == NULL) return;
    if (g->L) lua_close(g->L);
    free(g);
}

static void glua_budget(gn_lua* g, int heap)
{
    g->insns = 0;
    g->insn_limit = GENO_LUA_INSN_BUDGET;
    g->limit = g->used + (size_t) heap;
    g->fault = 0;
}

static int glua_state_meta(lua_State* L)
{
    lua_newtable(L);
    lua_pushcfunction(L, glua_state_index);
    lua_setfield(L, -2, "__index");
    lua_pushcfunction(L, glua_state_newindex);
    lua_setfield(L, -2, "__newindex");
    lua_pushboolean(L, 0);
    lua_setfield(L, -2, "__metatable");
    return 1;
}

/* A new, empty domain (no module). NULL when Lua cannot allocate. */
static gn_lua* glua_new(void)
{
    gn_lua* g = (gn_lua*) calloc(1, sizeof *g);
    if (g == NULL) return NULL;
    g->L = lua_newstate(glua_alloc, g);
    if (g->L == NULL) {
        free(g);
        return NULL;
    }
    *(gn_lua**) lua_getextraspace(g->L) = g;
    lua_gc(g->L, LUA_GCSTOP);
    lua_sethook(g->L, glua_hook, LUA_MASKCOUNT, GENO_LUA_INSN_STEP);
    return g;
}

static void glua_set_layout(gn_lua* g, int n, const char names[][GLUA_SLOT_NAME], const int* types)
{
    int i;
    g->nslot = n > GENO_LUA_STATE_SLOTS ? GENO_LUA_STATE_SLOTS : n;
    for (i = 0; i < g->nslot; i++) {
        snprintf(g->slot_name[i], sizeof g->slot_name[i], "%s", names[i]);
        g->slot_type[i] = types[i];
    }
}

static void glua_set_targets(gn_lua* g, int n, const char names[][GLUA_NAME])
{
    int i;
    g->ntarget = n > GENO_MAX_STATES ? GENO_MAX_STATES : n;
    for (i = 0; i < g->ntarget; i++) snprintf(g->target_name[i], sizeof g->target_name[i], "%s", names[i]);
}

/* Builds the allowlist environment (a frozen proxy) and leaves it on the stack. */
static void glua_push_env(lua_State* L)
{
    static const char* const base[] = {"type", "select", "ipairs", "assert", "error", NULL};
    static const char* const mathf[] = {"abs", "min", "max", "floor", "ceil", "sqrt", "tointeger", "huge", "pi", NULL};
    int i, env, m;
    luaL_requiref(L, "_G", luaopen_base, 1); /* the real globals: we copy five functions out and never expose it */
    env = lua_gettop(L) + 1;
    lua_newtable(L);
    for (i = 0; base[i]; i++) {
        lua_getglobal(L, base[i]);
        lua_setfield(L, env, base[i]);
    }
    luaL_requiref(L, "math", luaopen_math, 0);
    m = lua_gettop(L);
    lua_newtable(L);
    for (i = 0; mathf[i]; i++) {
        lua_getfield(L, m, mathf[i]);
        lua_setfield(L, -2, mathf[i]);
    }
    lua_setfield(L, env, "math");
    lua_pushcfunction(L, glua_freeze_fn);
    lua_setfield(L, env, "freeze");
    lua_pop(L, 1); /* math */
    glua_freeze_val(L, env, 0);
    lua_remove(L, env);
    lua_remove(L, env - 1); /* _G */
    lua_pushnil(L);
    lua_setglobal(L, "dofile");
}

/* Loads `src` as the module, checks it, and registers every function it returns. 1 on success; 0 with g->err set. */
static int glua_load(gn_lua* g, const char* src, size_t n, const char* chunk)
{
    lua_State* L = g->L;
    int top = lua_gettop(L), st;
    g->err[0] = 0;
    glua_budget(g, 0);
    g->limit = g->used + GLUA_LOAD_HEAP;
    glua_push_env(L); /* env */
    if (luaL_loadbufferx(L, src, n, chunk, "t") != LUA_OK) {
        snprintf(g->err, sizeof g->err, "%s", lua_tostring(L, -1));
        lua_settop(L, top);
        return 0;
    }
    lua_pushvalue(L, -2);
    if (lua_setupvalue(L, -2, 1) == NULL) {
        snprintf(g->err, sizeof g->err, "the module has no environment");
        lua_settop(L, top);
        return 0;
    }
    st = lua_pcall(L, 0, 1, 0);
    if (st != LUA_OK) {
        snprintf(g->err, sizeof g->err, "%s%s", lua_tostring(L, -1) ? lua_tostring(L, -1) : "error",
                 g->fault == GENO_LUA_FAULT_INSNS ? " (loading the module)" : "");
        lua_settop(L, top);
        return 0;
    }
    if (lua_type(L, -1) != LUA_TTABLE) {
        snprintf(g->err, sizeof g->err, "the module must return a table of functions");
        lua_settop(L, top);
        return 0;
    }
    {
        glua_seen sn;
        int tbl = lua_gettop(L);
        lua_pushnil(L);
        while (lua_next(L, tbl)) {
            const char* k;
            if (lua_type(L, -2) != LUA_TSTRING || lua_type(L, -1) != LUA_TFUNCTION || lua_iscfunction(L, -1)) {
                snprintf(g->err, sizeof g->err, "the module table may hold only functions with name keys (constants belong in locals)");
                lua_settop(L, top);
                return 0;
            }
            k = lua_tostring(L, -2);
            if (strlen(k) >= GLUA_NAME || g->nfn >= GLUA_MAX_FNS) {
                snprintf(g->err, sizeof g->err, "function name '%s' is too long or the module has more than %d functions", k, GLUA_MAX_FNS);
                lua_settop(L, top);
                return 0;
            }
            sn.n = 0;
            if (!glua_check_fn(g, -1, k, 0, &sn)) {
                lua_settop(L, top);
                return 0;
            }
            snprintf(g->fn_name[g->nfn], GLUA_NAME, "%s", k);
            lua_pushvalue(L, -1);
            g->fn_ref[g->nfn++] = luaL_ref(L, LUA_REGISTRYINDEX);
            lua_pop(L, 1); /* the value; the key stays */
        }
    }
    lua_settop(L, top);
    /* the state-proxy metatable, kept in the registry */
    glua_state_meta(L);
    lua_setfield(L, LUA_REGISTRYINDEX, "geno.lua.state");
    lua_gc(L, LUA_GCCOLLECT);
    g->limit = 0;
    return 1;
}

static int glua_find(const gn_lua* g, const char* name)
{
    int i;
    for (i = 0; i < g->nfn; i++) {
        if (strcmp(g->fn_name[i], name) == 0) return i;
    }
    return -1;
}

static void glua_setnum(lua_State* L, const char* k, lua_Number v)
{
    lua_pushnumber(L, v);
    lua_setfield(L, -2, k);
}

static void glua_setbool(lua_State* L, const char* k, int v)
{
    lua_pushboolean(L, v);
    lua_setfield(L, -2, k);
}

static void glua_setint(lua_State* L, const char* k, lua_Integer v)
{
    lua_pushinteger(L, v);
    lua_setfield(L, -2, k);
}

static void glua_push_ctx(gn_lua* g)
{
    lua_State* L = g->L;
    uint32_t* io = g->io;
    uint32_t fl = gw_r32(&io[GENO_LUA_IO_FLAGS]);
    uint32_t held = gw_r32(&io[GENO_LUA_IO_HELD]), pressed = gw_r32(&io[GENO_LUA_IO_PRESSED]);
    float sx = glua_bits_f(gw_r32(&io[GENO_LUA_IO_STICK_X]));
    float sy = glua_bits_f(gw_r32(&io[GENO_LUA_IO_STICK_Y]));
    static const struct { uint32_t bit; const char *held, *pressed; } btn[] = {
        {GENO_BTN_ATTACK, "attack_held", "attack_pressed"}, {GENO_BTN_SPECIAL, "special_held", "special_pressed"},
        {GENO_BTN_JUMP, "jump_held", "jump_pressed"},       {GENO_BTN_SHIELD, "shield_held", "shield_pressed"},
        {GENO_BTN_GRAB, "grab_held", "grab_pressed"}};
    size_t i;
    lua_createtable(L, 0, 8);
    lua_createtable(L, 0, 14);
    for (i = 0; i < sizeof btn / sizeof btn[0]; i++) {
        glua_setbool(L, btn[i].held, (held & btn[i].bit) != 0);
        glua_setbool(L, btn[i].pressed, (pressed & btn[i].bit) != 0);
    }
    glua_setnum(L, "stick_x", sx);
    glua_setnum(L, "stick_y", sy);
    glua_setnum(L, "stick_fwd", (fl & 2) ? -sx : sx);
    lua_setfield(L, -2, "input");
    lua_createtable(L, 0, 12);
    glua_setbool(L, "air", (fl & 1) != 0);
    glua_setbool(L, "anim_ended", (fl & 4) != 0);
    glua_setnum(L, "facing", (fl & 2) ? -1.0 : 1.0);
    glua_setnum(L, "percent", glua_bits_f(gw_r32(&io[GENO_LUA_IO_PERCENT])));
    glua_setnum(L, "x", glua_bits_f(gw_r32(&io[GENO_LUA_IO_POS_X])));
    glua_setnum(L, "y", glua_bits_f(gw_r32(&io[GENO_LUA_IO_POS_Y])));
    glua_setnum(L, "vel_x", glua_bits_f(gw_r32(&io[GENO_LUA_IO_VEL_X])));
    glua_setnum(L, "vel_y", glua_bits_f(gw_r32(&io[GENO_LUA_IO_VEL_Y])));
    glua_setint(L, "action_frame", (int32_t) gw_r32(&io[GENO_LUA_IO_ACTION_FRAME]));
    glua_setint(L, "motion", (int32_t) gw_r32(&io[GENO_LUA_IO_MOTION]));
    lua_setfield(L, -2, "self");
    lua_newtable(L); /* ctx.state: an empty proxy; every access goes to the metatable */
    lua_getfield(L, LUA_REGISTRYINDEX, "geno.lua.state");
    lua_setmetatable(L, -2);
    lua_setfield(L, -2, "state");
    lua_pushcfunction(L, glua_go);
    lua_setfield(L, -2, "go");
    lua_pushcfunction(L, glua_velocity);
    lua_setfield(L, -2, "velocity");
    lua_pushcfunction(L, glua_hitbox_damage);
    lua_setfield(L, -2, "hitbox_damage");
    lua_pushcfunction(L, glua_loop);
    lua_setfield(L, -2, "loop");
}

/* Runs function `fn` (an index from glua_find) over `io`. 0 = ok; GENO_LUA_FAULT_* otherwise (the io's state words and
 * commands are then to be discarded by the caller). */
static int glua_call(gn_lua* g, int fn, uint32_t* io)
{
    lua_State* L = g->L;
    int st, top = lua_gettop(L);
    g->calls++;
    g->err[0] = 0;
    g->io = io;
    g->ncmd = 0;
    gw_w32(&io[GENO_LUA_IO_NCMDS], 0);
    gw_w32(&io[GENO_LUA_IO_FAULT], 0);
    glua_budget(g, GENO_LUA_HEAP_BUDGET);
    lua_rawgeti(L, LUA_REGISTRYINDEX, g->fn_ref[fn]);
    glua_push_ctx(g);
    st = lua_pcall(L, 1, 0, 0);
    if (st != LUA_OK) {
        if (!g->fault) g->fault = st == LUA_ERRMEM ? GENO_LUA_FAULT_HEAP : GENO_LUA_FAULT_ERROR;
        snprintf(g->err, sizeof g->err, "%s", lua_type(L, -1) == LUA_TSTRING ? lua_tostring(L, -1) : "error");
        g->faults++;
        gw_w32(&io[GENO_LUA_IO_NCMDS], 0);
    }
    lua_settop(L, top);
    lua_gc(L, LUA_GCCOLLECT); /* the heap at the start of the next call is the same as at the start of this one */
    g->limit = 0;
    st = g->fault;
    g->fault = 0;
    g->io = NULL;
    gw_w32(&io[GENO_LUA_IO_FAULT], (uint32_t) st);
    return st;
}

#endif
