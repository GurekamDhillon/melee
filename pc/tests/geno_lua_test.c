/* geno_lua_test.c - isolated native test of the fighter-Lua domain (pc/platform/geno_lua_core.h).
 * `tools/port/build.sh --native-test geno-lua` (no game, no bridge). What it proves is listed at the top of the header. */
#include "geno_lua_core.h"

static int fails;
#define CHECK(c, ...) do { if (!(c)) { fails++; printf("FAIL line %d: ", __LINE__); printf(__VA_ARGS__); printf("\n"); } } while (0)

static uint32_t io[GENO_LUA_IO_WORDS];

static void io_reset(void)
{
    memset(io, 0, sizeof io);
}
static uint32_t rd(int i) { return gw_r32(&io[i]); }

static const char SLOTS[][GLUA_SLOT_NAME] = {"charge", "ratio", "armed"};
static const int TYPES[] = {GLUA_INT, GLUA_FLOAT, GLUA_BOOL};
static const char TARGETS[][GLUA_NAME] = {"Charge", "Release"};

static gn_lua* mk(const char* src)
{
    gn_lua* g = glua_new();
    if (!g) return NULL;
    glua_set_layout(g, 3, SLOTS, TYPES);
    glua_set_targets(g, 2, TARGETS);
    if (!glua_load(g, src, strlen(src), "=test")) {
        printf("  (load refused: %s)\n", g->err);
        glua_free(g);
        return NULL;
    }
    return g;
}

/* a module that must be refused at load, with the words its reason has to contain */
static void refused(const char* what, const char* src, const char* reason)
{
    gn_lua* g = glua_new();
    int ok;
    glua_set_layout(g, 3, SLOTS, TYPES);
    glua_set_targets(g, 2, TARGETS);
    ok = glua_load(g, src, strlen(src), "=test");
    CHECK(!ok, "%s: the module loaded but must be refused", what);
    CHECK(ok || strstr(g->err, reason) != NULL, "%s: the reason \"%s\" lacks \"%s\"", what, g->err, reason);
    glua_free(g);
}

/* a call that must fault with `code` */
static void faults(const char* what, const char* src, int code)
{
    gn_lua* g = mk(src);
    int rc;
    CHECK(g != NULL, "%s: module did not load", what);
    if (!g) return;
    io_reset();
    rc = glua_call(g, glua_find(g, "f"), io);
    CHECK(rc == code, "%s: fault %d, wanted %d (%s)", what, rc, code, g->err);
    CHECK(rd(GENO_LUA_IO_FAULT) == (uint32_t) code, "%s: io fault word", what);
    CHECK(rd(GENO_LUA_IO_NCMDS) == 0, "%s: a fault must leave no commands", what);
    glua_free(g);
}

int main(void)
{
    gn_lua* g;
    int rc, i, a, b;
    static const char GOOD[] =
        "local MAX = 60\n"
        "local TUNE = freeze{ base = 1.5, per = 0.25, table = {1,2,3} }\n"
        "local function ramp(n) return TUNE.base + TUNE.per * n end\n"
        "local M = {}\n"
        "function M.f(ctx)\n"
        "  local s = ctx.state\n"
        "  if ctx.input.special_held and s.charge < MAX then s.charge = s.charge + 1 else ctx.go('Release') end\n"
        "  s.ratio = ramp(s.charge)\n"
        "  s.armed = s.charge >= 3\n"
        "  ctx.velocity(ramp(s.charge) * ctx.self.facing, 0)\n"
        "  ctx.hitbox_damage(3, math.min(20, 6 + 0.15 * s.charge))\n"
        "  ctx.loop()\n"
        "end\n"
        "function M.fresh(ctx) ctx.n = (ctx.n or 0) + 1; ctx.state.charge = ctx.n end\n"
        "return M\n";
    /* ---- a good module: reads, typed writes, commands in order ---- */
    g = mk(GOOD);
    CHECK(g != NULL, "the good module must load");
    if (!g) return 1;
    CHECK(g->nfn == 2, "two functions registered, got %d", g->nfn);
    io_reset();
    gw_w32(&io[GENO_LUA_IO_HELD], GENO_BTN_SPECIAL);
    gw_w32(&io[GENO_LUA_IO_STATE + 0], 4);
    rc = glua_call(g, glua_find(g, "f"), io);
    CHECK(rc == 0, "good call faulted: %d %s", rc, g->err);
    CHECK((int32_t) rd(GENO_LUA_IO_STATE + 0) == 5, "charge 4 -> 5, got %d", (int) rd(GENO_LUA_IO_STATE + 0));
    CHECK(rd(GENO_LUA_IO_STATE + 2) == 1, "armed set");
    CHECK(glua_bits_f(rd(GENO_LUA_IO_STATE + 1)) == (float) (1.5 + 0.25 * 5), "ratio is the f32 of 2.75");
    CHECK(rd(GENO_LUA_IO_NCMDS) == 3, "three commands queued, got %u", (unsigned) rd(GENO_LUA_IO_NCMDS));
    CHECK(rd(GENO_LUA_IO_CMDS + 0) == GENO_LUA_CMD_VELOCITY && glua_bits_f(rd(GENO_LUA_IO_CMDS + 1)) == 2.75f,
          "command 0 is velocity 2.75");
    CHECK(rd(GENO_LUA_IO_CMDS + 3) == GENO_LUA_CMD_HITBOX_DAMAGE && rd(GENO_LUA_IO_CMDS + 4) == 3 &&
              glua_bits_f(rd(GENO_LUA_IO_CMDS + 5)) == (float) (6 + 0.15 * 5), "command 1 is hitbox_damage(3, 6.75)");
    CHECK(rd(GENO_LUA_IO_CMDS + 6) == GENO_LUA_CMD_LOOP, "command 2 is loop");
    /* released: go to Release (state 1) */
    io_reset();
    gw_w32(&io[GENO_LUA_IO_STATE + 0], 7);
    rc = glua_call(g, glua_find(g, "f"), io);
    CHECK(rc == 0 && rd(GENO_LUA_IO_CMDS + 0) == GENO_LUA_CMD_GO && rd(GENO_LUA_IO_CMDS + 1) == GENO_TARGET(GENO_TGT_GENO, 1),
          "release commands go -> Release");
    /* the cap: charge 60 stops counting */
    io_reset();
    gw_w32(&io[GENO_LUA_IO_HELD], GENO_BTN_SPECIAL);
    gw_w32(&io[GENO_LUA_IO_STATE + 0], 60);
    glua_call(g, glua_find(g, "f"), io);
    CHECK((int32_t) rd(GENO_LUA_IO_STATE + 0) == 60 && rd(GENO_LUA_IO_CMDS + 0) == GENO_LUA_CMD_GO, "at the cap it releases");
    /* ---- stateless: ctx is fresh every call; nothing survives except the state words ---- */
    for (i = 0; i < 3; i++) {
        io_reset();
        glua_call(g, glua_find(g, "fresh"), io);
        CHECK((int32_t) rd(GENO_LUA_IO_STATE + 0) == 1, "fresh ctx each call (run %d gave %d)", i, (int) rd(GENO_LUA_IO_STATE + 0));
    }
    /* the heap at the start of every call is the same (no leak, collector settled) */
    a = (int) g->used;
    for (i = 0; i < 50; i++) { io_reset(); glua_call(g, glua_find(g, "f"), io); }
    b = (int) g->used;
    CHECK(a == b, "heap after 50 calls %d != %d before: state leaks between calls", b, a);
    glua_free(g);

    /* ---- refused at load ---- */
    refused("global assignment in a function", "local M = {}\nfunction M.f(ctx) total = 1 end\nreturn M\n", "assigns a global");
    refused("captured counter", "local n = 0\nlocal M = {}\nfunction M.f(ctx) n = n + 1 end\nreturn M\n", "assigns a captured variable");
    refused("captured counter in a nested function",
            "local n = 0\nlocal M = {}\nfunction M.f(ctx) return function() n = n + 1 end end\nreturn M\n", "assigns a captured variable");
    refused("a mutable table upvalue", "local cache = {}\nlocal M = {}\nfunction M.f(ctx) return cache[1] end\nreturn M\n", "mutable table");
    refused("the module table as an upvalue", "local M = {}\nfunction M.g() return 1 end\nfunction M.f(ctx) return M.g() end\nreturn M\n", "mutable table");
    refused("a constant in the module table", "return { f = function(ctx) end, MAX = 60 }\n", "only functions");
    refused("no table returned", "return 5\n", "return a table");
    refused("a global write at load time", "x = 1\nreturn {}\n", "frozen");
    refused("a syntax error", "return {\n", "test");
    refused("an endless loop at load time", "while true do end\nreturn {}\n", "budget");
    refused("freeze with a table key", "local T = freeze{ [{}] = 1 }\nreturn {}\n", "key");
    /* ---- faults at run time ---- */
    faults("an endless loop", "return { f = function(ctx) while true do end end }\n", GENO_LUA_FAULT_INSNS);
    faults("an allocation bomb",
           "return { f = function(ctx) local t = {} for i = 1, 400 do t[i] = {i,i,i,i,i,i,i,i,i,i,i,i,i,i,i,i,i,i,i,i,i,i,i,i} end end }\n",
           GENO_LUA_FAULT_HEAP);
    faults("pairs is not there", "return { f = function(ctx) for k in pairs(ctx) do end end }\n", GENO_LUA_FAULT_ERROR);
    faults("pcall is not there", "return { f = function(ctx) pcall(error) end }\n", GENO_LUA_FAULT_ERROR);
    faults("string is not there", "return { f = function(ctx) return string.rep('x', 3) end }\n", GENO_LUA_FAULT_ERROR);
    faults("setmetatable is not there", "return { f = function(ctx) setmetatable(ctx, {}) end }\n", GENO_LUA_FAULT_ERROR);
    faults("math.random is not there", "return { f = function(ctx) return math.random(5) end }\n", GENO_LUA_FAULT_ERROR);
    faults("an undeclared slot (write)", "return { f = function(ctx) ctx.state.nope = 1 end }\n", GENO_LUA_FAULT_STATE);
    faults("an undeclared slot (read)", "return { f = function(ctx) return ctx.state.nope end }\n", GENO_LUA_FAULT_STATE);
    faults("a float into an int slot", "return { f = function(ctx) ctx.state.charge = 1.5 end }\n", GENO_LUA_FAULT_STATE);
    faults("an int slot beyond 32 bits", "return { f = function(ctx) ctx.state.charge = 3000000000 end }\n", GENO_LUA_FAULT_STATE);
    faults("a number into a bool slot", "return { f = function(ctx) ctx.state.armed = 1 end }\n", GENO_LUA_FAULT_STATE);
    faults("a nan into a float slot", "return { f = function(ctx) ctx.state.ratio = 0/0 end }\n", GENO_LUA_FAULT_STATE);
    faults("an unknown state name", "return { f = function(ctx) ctx.go('Nowhere') end }\n", GENO_LUA_FAULT_COMMAND);
    faults("an infinite velocity", "return { f = function(ctx) ctx.velocity(math.huge, 0) end }\n", GENO_LUA_FAULT_COMMAND);
    faults("a bad hitbox mask", "return { f = function(ctx) ctx.hitbox_damage(0, 5) end }\n", GENO_LUA_FAULT_COMMAND);
    faults("nine commands", "return { f = function(ctx) for i = 1, 9 do ctx.loop() end end }\n", GENO_LUA_FAULT_CMDS);
    faults("a frozen table write", "return { f = function(ctx) local T = freeze{1,2} T[1] = 5 end }\n", GENO_LUA_FAULT_ERROR);
    faults("an error()", "return { f = function(ctx) error('boom') end }\n", GENO_LUA_FAULT_ERROR);
    /* ---- the instruction fault is tick-exact: the same count every run ---- */
    {
        int n0 = -1;
        for (i = 0; i < 3; i++) {
            g = mk("return { f = function(ctx) local n = 0 while true do n = n + 1 end end }\n");
            io_reset();
            glua_call(g, glua_find(g, "f"), io);
            if (n0 < 0) n0 = g->insns;
            CHECK(g->insns == n0 && g->insns > GENO_LUA_INSN_BUDGET, "budget fault count repeats (%d vs %d)", g->insns, n0);
            glua_free(g);
        }
    }
    /* ---- a faulting call leaves the domain usable and the heap settled ---- */
    g = mk("return { bad = function(ctx) while true do end end, ok = function(ctx) ctx.state.charge = 9 end }\n");
    io_reset();
    glua_call(g, glua_find(g, "ok"), io); /* warm-up: the first call interns the ctx's key strings */
    a = (int) g->used;
    io_reset();
    glua_call(g, glua_find(g, "bad"), io);
    io_reset();
    rc = glua_call(g, glua_find(g, "ok"), io);
    CHECK(rc == 0 && (int32_t) rd(GENO_LUA_IO_STATE + 0) == 9, "the domain works after a fault");
    CHECK((int) g->used <= a + 64, "heap after a fault %d vs %d", (int) g->used, a);
    glua_free(g);
    /* ---- the allowlist's functions work ---- */
    g = mk("return { f = function(ctx) local t = freeze{4,5,6} local s = 0 for _, v in ipairs(t) do s = s + v end "
           "ctx.state.charge = s + #t + math.floor(2.7) + math.abs(-1) + select('#', 1, 2) + math.max(1, 9) end }\n");
    io_reset();
    rc = glua_call(g, glua_find(g, "f"), io);
    CHECK(rc == 0 && (int32_t) rd(GENO_LUA_IO_STATE + 0) == 15 + 3 + 2 + 1 + 2 + 9, "allowlist arithmetic, got %d (%s)", (int) rd(GENO_LUA_IO_STATE + 0), g->err);
    glua_free(g);
    printf(fails ? "geno-lua: %d FAILED\n" : "geno-lua: all passed\n", fails);
    return fails ? 1 : 0;
}
