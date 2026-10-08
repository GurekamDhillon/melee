/* Online Envoy, stage 2: STAGING and APPLYING a build (included by gw_script.c after gw_script_sim_state.inc).
 *
 * The Envoy script layer compiles a build (the records a player holds) into the passive ops it already commits offline: fighter values, hit
 * rules, capability caps, armour, crit configuration. Offline it writes them after each logic frame through sim_commit. Online it may not (every
 * Lua gameplay write is refused during a session, because nothing replays them), so the SAME ops are STAGED here, before the match, and the native
 * side applies them ONCE, before frame 0, from inside the rollback session's own open (gw_RB_Iterations) with the same setters, on both peers.
 *
 *   gd.netbuild_stage(slot, record, ops)  slot 1 or 2 (the host is 1), record = the build record text (gw_matchbuild.h), ops = the sim_commit op
 *                                        list. Validated like sim_commit, then like a build: only passive op kinds, only the slot's own port and
 *                                        entities. Allowed whenever no online match is running (menus, lobby, between games), by any script.
 *   gd.netbuild_clear([slot])            forget one slot or both.
 *   gd.netbuild()                        READ-ONLY, allowed online: what is staged, the agreement word, and what the match applied (per port:
 *                                        record digest, ops digest, counts, the live word the rollback hash sees).
 *
 * What is applied lives in the existing snapshotted BSS (fighter mods, hit rules, caps, armour, crit) under the owner GW_NB_OWNER, plus the game-side
 * table pc/gameworld/script_build.inc, whose word is mixed into the rollback hash. The staged copy here is host memory: it is only the INPUT of the
 * apply and is re-applied at the start of every match, so a restore never needs it.
 */
#include "gw_matchbuild.h"
#define GW_NB_OWNER 100
#define GW_NB_SLOTS 2
#define GW_NB_OPS 24
extern int gw_ScriptGame_BuildSet(int port, int rec_hi, int rec_lo, int ops_word, int nops, int nrec);
extern void gw_ScriptGame_BuildClear(void);
extern int gw_ScriptGame_BuildRead(int port, int field);
extern unsigned gw_ScriptGame_BuildHashWord(void);
extern int gw_ScriptGame_FighterModOwner(int slot);
extern void gw_ScriptGame_ArmorRelease(int owner);
extern int gw_RB_Enabled(void);
typedef struct {
    int have;
    char record[GW_MB_RECORD_MAX + 4];
    GwMbRecord rec;
    unsigned ops_word;
    int nops;
    GsSimOp ops[GW_NB_OPS];
} GsNbSlot;
static GsNbSlot gs_nb[GW_NB_SLOTS];
static int gs_nb_applied;

/* ---- stage 4: the TRIGGERED program staged beside the passive ops --------------------------------------------------------------------------------------
 * The script layer (mod_engine.lua E:native_program) compiles the equipped triggered records into flat words for the native evaluator
 * (pc/gameworld/script_mods_core.h) and the derived tables of every status mask into VARIANTS (fighter values, hit rules without their status bits,
 * crit configuration). Words are loaded into the game's snapshotted BSS at the arm; the variants stay here (constant for the match) and are applied by
 * gw_Script_NetModsVariant when the evaluator's status mask changes. The program digest is folded into the slot's ops word, so the agreement word, the
 * build table and the rollback hash all cover it. */
#define GW_MODS_WORDS 6568
#define GW_NB_VARIANTS 24
#define GW_NB_VOPS 12
extern int gw_ScriptMods_ProgWord(int port, int index, int value);
extern int gw_ScriptMods_ProgSeal(int port);
extern int gw_ScriptMods_Read(int port, int what, int idx);
extern int gw_ScriptGame_CritSet(int entity, int owner, int slot, int chance, int mult, int mult_max, int launch, int min_percent, int force, int flags);
extern int gw_ScriptGame_CritRead(int entity, int field);
typedef struct { int nops, curse_rule; GsSimOp ops[GW_NB_VOPS]; } GsNbVariant;
typedef struct { int have, nvariants; unsigned digest; int words[GW_MODS_WORDS]; GsNbVariant var[GW_NB_VARIANTS]; } GsNbProg;
static GsNbProg gs_nb_prog[GW_NB_SLOTS];
static GsNbProg gs_nb_prog_draft;

static unsigned gs_nb_mix(unsigned h, unsigned v) {
    h ^= v;
    h *= 0x01000193u;
    h ^= h >> 15;
    return h;
}
/* the digest of one op as the setters will see it: every scalar, nothing positional */
static unsigned gs_nb_op_word(const GsSimOp *o) {
    unsigned h = 0x811C9DC5u;
    int i, j;
    h = gs_nb_mix(h, (unsigned) o->kind);
    h = gs_nb_mix(h, (unsigned) o->slot);
    h = gs_nb_mix(h, (unsigned) o->enabled);
    if (o->kind == 1) { for (i = 0; i < 11; ++i) h = gs_nb_mix(h, (unsigned) o->bits[i]); }
    else if (o->kind == 3) {
        h = gs_nb_mix(h, (unsigned) o->hit_rules.count);
        h = gs_nb_mix(h, (unsigned) o->hit_rules.status);
        for (i = 0; i < o->hit_rules.count; ++i) for (j = 0; j < GS_HR_FIELDS; ++j) if (j != 12) h = gs_nb_mix(h, (unsigned) o->hit_rules.rules[i][j]);
    } else if (o->kind >= GS_SIM_CAPS) { for (i = 0; i < 9; ++i) h = gs_nb_mix(h, (unsigned) o->eff[i]); }
    return h;
}
static unsigned gs_nb_prog_digest(const GsNbProg *g) {
    unsigned h = 0x6D6F6473u;
    int i, j;
    for (i = 0; i < GW_MODS_WORDS; ++i) h = gs_nb_mix(h, (unsigned) g->words[i]);
    h = gs_nb_mix(h, (unsigned) g->nvariants);
    for (i = 0; i < g->nvariants; ++i) {
        h = gs_nb_mix(h, (unsigned) g->var[i].nops);
        h = gs_nb_mix(h, (unsigned) g->var[i].curse_rule);
        for (j = 0; j < g->var[i].nops; ++j) h = gs_nb_mix(h, gs_nb_op_word(&g->var[i].ops[j]));
    }
    return h;
}
static unsigned gs_nb_ops_word(const GsNbSlot *s) {
    unsigned h = 0x9E3779B1u;
    int i;
    for (i = 0; i < s->nops; ++i) h = gs_nb_mix(h, gs_nb_op_word(&s->ops[i]));
    return gs_nb_mix(h, (unsigned) s->nops);
}
/* The agreement word for the next game: both slots' record digests and op digests, in slot order. 0 = nothing staged. */
unsigned gw_Script_NetBuildWord(void) {
    unsigned h = 0x811C9DC5u;
    int i, any = 0;
    for (i = 0; i < GW_NB_SLOTS; ++i) {
        h = gs_nb_mix(h, (unsigned) (i + 1));
        if (!gs_nb[i].have) { h = gs_nb_mix(h, 0u); continue; }
        any = 1;
        h = gs_nb_mix(h, gs_nb[i].rec.dig_hi);
        h = gs_nb_mix(h, gs_nb[i].rec.dig_lo);
        h = gs_nb_mix(h, gs_nb[i].ops_word);
    }
    return any ? (h | 1u) : 0;
}
int gw_Script_NetBuildStaged(void) { int i, n = 0; for (i = 0; i < GW_NB_SLOTS; ++i) n += gs_nb[i].have; return n; }
const char *gw_Script_NetBuildRecord(int slot) { return slot >= 0 && slot < GW_NB_SLOTS && gs_nb[slot].have ? gs_nb[slot].record : ""; }
int gw_Script_NetBuildApplied(void) { return gs_nb_applied; }

/* one op, same rules as sim_commit, for the passive kinds only. Stack: the op table at -1 (left there). */
static void gs_nb_parse_op(lua_State *L, int slot, GsSimOp *o) {
    static const char *keys[11] = {"damage_dealt","damage_taken","run_speed","air_speed","shield_max","jump_height","air_jump_height","knockback_taken","fall_speed","weight","shield_regen"};
    const char *name;
    int j;
    memset(o, 0, sizeof *o);
    lua_getfield(L, -1, "op"); name = luaL_checkstring(L, -1);
    o->kind = !strcmp(name,"fighter_mod") ? 1 : !strcmp(name,"hit_rules") ? 3 : !strcmp(name,"fighter_caps") ? GS_SIM_CAPS :
        !strcmp(name,"fighter_armor") ? GS_SIM_ARMOR : !strcmp(name,"crit") ? GS_SIM_CRIT : 0;
    lua_pop(L, 1);
    if (!o->kind) luaL_error(L, "netbuild_stage: op '%s' is not a passive build op (fighter_mod, hit_rules, fighter_caps, fighter_armor, crit)", name);
    if (o->kind >= GS_SIM_CAPS) {
        gs_sim_parse_caps(L, o);
        if ((o->slot - 1) % 6 != slot) luaL_error(L, "netbuild_stage: op targets entity %d, not slot %d's own", o->slot, slot + 1); /* entities 1-6 primary, 7-12 secondary: port = (e-1) mod 6 */
        if (o->kind == GS_SIM_ARMOR && o->eff[8]) luaL_error(L, "netbuild_stage: armour clear is not a build op");
        return;
    }
    lua_getfield(L, -1, "port"); o->slot = gs_slot_arg(L, -1); lua_pop(L, 1);
    if (o->slot != slot) luaL_error(L, "netbuild_stage: op targets port %d, not slot %d's own", o->slot + 1, slot + 1);
    if (o->kind == 1) {
        lua_getfield(L, -1, "values"); o->enabled = !lua_isnil(L, -1);
        for (j = 0; j < 11; ++j) o->bits[j] = gs_fbits(1.0f);
        if (o->enabled) {
            int idx = lua_absindex(L, -1);
            luaL_checktype(L, idx, LUA_TTABLE); lua_pushnil(L);
            while (lua_next(L, idx)) {
                const char *key = lua_type(L, -2) == LUA_TSTRING ? lua_tostring(L, -2) : NULL;
                double value = luaL_checknumber(L, -1);
                for (j = 0; j < 11; ++j) if (key && !strcmp(key, keys[j])) break;
                if (j == 11 || !isfinite(value) || value < 0.1 || value > 4) luaL_error(L, "netbuild_stage: invalid fighter modifier");
                o->bits[j] = gs_fbits((float) value); lua_pop(L, 1);
            }
        }
        lua_pop(L, 1);
        if (!o->enabled) luaL_error(L, "netbuild_stage: a build op sets values; clearing is not one");
    } else { /* hit rules: both entities of the port */
        o->enabled = -1;
        gs_hr_parse(L, -1, &o->hit_rules);
    }
}


/* The optional 4th argument of gd.netbuild_stage: {words = {6568 ints}, variants = { {ops = {...}, curse_rule = n}, ... }}. Parsed into a draft. */
static void gs_nb_parse_program(lua_State *L, int arg, int slot, GsNbProg *out) {
    size_t n, i, j;
    int idx = lua_absindex(L, arg);
    memset(out, 0, sizeof *out);
    luaL_checktype(L, idx, LUA_TTABLE);
    lua_getfield(L, idx, "words"); luaL_checktype(L, -1, LUA_TTABLE);
    n = lua_rawlen(L, -1);
    if (n != GW_MODS_WORDS) luaL_error(L, "netbuild_stage: program needs %d words, got %d", GW_MODS_WORDS, (int) n);
    for (i = 0; i < n; ++i) { lua_rawgeti(L, -1, (lua_Integer) i + 1); out->words[i] = (int) luaL_checkinteger(L, -1); lua_pop(L, 1); }
    lua_pop(L, 1);
    if (out->words[0] != 0x534D5031 || out->words[1] < 0 || out->words[1] > 24) luaL_error(L, "netbuild_stage: bad program header");
    lua_getfield(L, idx, "variants"); luaL_checktype(L, -1, LUA_TTABLE);
    n = lua_rawlen(L, -1);
    if (n < 1 || n > GW_NB_VARIANTS) luaL_error(L, "netbuild_stage: 1..%d program variants", GW_NB_VARIANTS);
    out->nvariants = (int) n;
    for (i = 0; i < n; ++i) {
        size_t m;
        GsNbVariant *v = &out->var[i];
        lua_rawgeti(L, -1, (lua_Integer) i + 1); luaL_checktype(L, -1, LUA_TTABLE);
        lua_getfield(L, -1, "curse_rule"); v->curse_rule = (int) luaL_optinteger(L, -1, 0); lua_pop(L, 1);
        lua_getfield(L, -1, "ops"); luaL_checktype(L, -1, LUA_TTABLE);
        m = lua_rawlen(L, -1);
        if (m > GW_NB_VOPS) luaL_error(L, "netbuild_stage: at most %d ops in a variant", GW_NB_VOPS);
        for (j = 0; j < m; ++j) {
            lua_rawgeti(L, -1, (lua_Integer) j + 1); luaL_checktype(L, -1, LUA_TTABLE);
            gs_nb_parse_op(L, slot, &v->ops[j]);
            lua_pop(L, 1);
        }
        v->nops = (int) m;
        lua_pop(L, 2);
    }
    lua_pop(L, 1);
    for (i = 0; i < GW_MODS_WORDS; ++i) {} /* (the words beyond the header are bounds-checked by the native seal at the arm) */
    out->digest = gs_nb_prog_digest(out);
    out->have = 1;
}

/* Store one slot's build: the record (already parsed) and the ops (already validated). The digest of the ops is computed here, once. */
static void gs_nb_install(int slot, const char *record, const GwMbRecord *rec, const GsSimOp *ops, int n) {
    GsNbSlot *s = &gs_nb[slot];
    memset(s, 0, sizeof *s);
    s->have = 1;
    s->nops = n;
    s->rec = *rec;
    snprintf(s->record, sizeof s->record, "%s", record);
    memcpy(s->ops, ops, (size_t) n * sizeof ops[0]);
    s->ops_word = gs_nb_ops_word(s);
}

static int l_netbuild_stage(lua_State *L) {
    lua_Integer slot1 = luaL_checkinteger(L, 1);
    size_t rl, n;
    const char *record = luaL_checklstring(L, 2, &rl);
    GsNbSlot *s;
    GwMbRecord rec;
    char why[160];
    int i, slot;
    if (!gs_cur_script()) return luaL_error(L, "netbuild_stage: no current script");
    if (slot1 < 1 || slot1 > GW_NB_SLOTS) return luaL_error(L, "netbuild_stage: slot is 1 (host) or 2 (guest)");
    slot = (int) slot1 - 1;
    if ((gw_RB_Enabled() || gw_Netplay_Enabled()) && gs.match_active) return luaL_error(L, "netbuild_stage: a build is staged between games, not during one");
    if (rl == 0 || rl > GW_MB_RECORD_MAX) return luaL_error(L, "netbuild_stage: record must be 1..%d bytes", GW_MB_RECORD_MAX);
    if (!gw_mb_record_parse(record, &rec, why, sizeof why)) return luaL_error(L, "netbuild_stage: bad record: %s", why);
    if (rec.port != slot + 1) return luaL_error(L, "netbuild_stage: the record is for port %ld, not slot %d", rec.port, slot + 1);
    luaL_checktype(L, 3, LUA_TTABLE); n = lua_rawlen(L, 3);
    if (n > GW_NB_OPS) return luaL_error(L, "netbuild_stage: at most %d ops", GW_NB_OPS);
    {   /* parse into a draft first: a refused call leaves the staged build as it was */
        static GsSimOp draft[GW_NB_OPS];
        memset(draft, 0, sizeof draft);
        for (i = 0; i < (int) n; ++i) {
            lua_rawgeti(L, 3, i + 1); luaL_checktype(L, -1, LUA_TTABLE);
            gs_nb_parse_op(L, slot, &draft[i]);
            lua_pop(L, 1);
        }
        if (lua_gettop(L) >= 4 && !lua_isnil(L, 4)) gs_nb_parse_program(L, 4, slot, &gs_nb_prog_draft); else memset(&gs_nb_prog_draft, 0, sizeof gs_nb_prog_draft);
        gs_nb_install(slot, record, &rec, draft, (int) n);
        s = &gs_nb[slot];
        memcpy(&gs_nb_prog[slot], &gs_nb_prog_draft, sizeof gs_nb_prog[slot]);
        if (gs_nb_prog[slot].have) s->ops_word = gs_nb_mix(s->ops_word ^ 0x9E3779B1u, gs_nb_prog[slot].digest);
    }
    gw_log("netbuild: staged slot=%d modifiers=%d keystones=%d ops=%d record=%s ops=%08x word=%08x program=%s rules=%d variants=%d", slot + 1, rec.nmods, rec.nkeystones, s->nops, rec.digest, s->ops_word, gw_Script_NetBuildWord(),
           gs_nb_prog[slot].have ? "yes" : "no", gs_nb_prog[slot].have ? gs_nb_prog[slot].words[1] : 0, gs_nb_prog[slot].have ? gs_nb_prog[slot].nvariants : 0);
    lua_pushstring(L, rec.digest);
    return 1;
}
static int l_netbuild_clear(lua_State *L) {
    int i;
    if (!lua_isnoneornil(L, 1)) {
        lua_Integer slot = luaL_checkinteger(L, 1);
        if (slot < 1 || slot > GW_NB_SLOTS) return luaL_error(L, "netbuild_clear: slot is 1 or 2");
        memset(&gs_nb[slot - 1], 0, sizeof gs_nb[0]); memset(&gs_nb_prog[slot - 1], 0, sizeof gs_nb_prog[0]);
    } else for (i = 0; i < GW_NB_SLOTS; ++i) { memset(&gs_nb[i], 0, sizeof gs_nb[0]); memset(&gs_nb_prog[i], 0, sizeof gs_nb_prog[0]); }
    lua_pushboolean(L, 1);
    return 1;
}

/* Apply one staged slot: the same native setters a committed op uses, under the build owner. Returns the number of ops that were accepted. */
static int gs_nb_apply_slot(int slot) {
    GsNbSlot *s = &gs_nb[slot];
    GsSimCommit dummy;
    int i, ok = 0;
    memset(&dummy, 0, sizeof dummy);
    dummy.script = GW_NB_OWNER - 1; /* gs_sim_apply_caps reads the owner as script+1 */
    for (i = 0; i < s->nops; ++i) {
        GsSimOp *o = &s->ops[i];
        int accepted = 1;
        if (o->kind == 1) {
            accepted = gw_ScriptGame_FighterMod(o->slot, GW_NB_OWNER, 1, o->bits[0], o->bits[1], o->bits[2], o->bits[3], o->bits[4], o->bits[5], o->bits[6], o->bits[7]);
            if (accepted) gw_ScriptGame_FighterModExtra(o->slot, GW_NB_OWNER, o->bits[8], o->bits[9], o->bits[10]);
        } else if (o->kind == 3) accepted = gs_hr_apply_targets(o->slot, -1, GW_NB_OWNER, &o->hit_rules);
        else gs_sim_apply_caps(&dummy, o);
        if (!accepted) gw_log("netbuild: slot %d op %d (kind %d) refused by its setter", slot + 1, i, o->kind);
        else ++ok;
    }
    return ok;
}

/* Stage 4: the evaluator's status mask changed (or the Marked amount did): apply the derived tables of `variant`, called from inside the simulation by
 * ScriptMods_StageFrame. The same setters the arm used, under the build owner. status_bits is a function of the mask (the evaluator computes it);
 * curse_bits is the Marked amount (float bits), added to the incoming launch rule the program marked. Deterministic: constant staged data, snapshotted targets. */
static float gs_nb_f(int bits) { union { int i; float f; } u; u.i = bits; return u.f; }
void gw_Script_NetModsVariant(int port, int variant, int status_bits, int curse_bits) {
    GsNbProg *pr;
    GsNbVariant *v;
    int j, k, have_mod = 0, have_rules = 0, have_crit = 0, entity = port + 1, owner = GW_NB_OWNER, present = 0;
    if (port < 0 || port >= GW_NB_SLOTS || !gs_nb_prog[port].have) return;
    pr = &gs_nb_prog[port];
    if (variant < 0 || variant >= pr->nvariants) { gw_log("netbuild: variant %d of port %d out of range", variant, port + 1); return; }
    v = &pr->var[variant];
    for (j = 0; j < v->nops; ++j) {
        GsSimOp *o = &v->ops[j];
        if (o->kind == 1) {
            have_mod = 1;
            if (gw_ScriptGame_FighterMod(o->slot, owner, 1, o->bits[0], o->bits[1], o->bits[2], o->bits[3], o->bits[4], o->bits[5], o->bits[6], o->bits[7]))
                gw_ScriptGame_FighterModExtra(o->slot, owner, o->bits[8], o->bits[9], o->bits[10]);
        } else if (o->kind == 3) {
            GsHitRules r = o->hit_rules;
            have_rules = 1;
            r.status = status_bits;
            if (v->curse_rule >= 1 && v->curse_rule <= r.count && r.rules[v->curse_rule - 1][0] == 1002 && curse_bits)
                r.rules[v->curse_rule - 1][15] = (int) gs_fbits(gs_nb_f(r.rules[v->curse_rule - 1][15]) + gs_nb_f(curse_bits));
            gs_hr_apply_targets(port, -1, owner, &r);
        }
    }
    if (!have_mod && gw_ScriptGame_FighterModOwner(port) == owner) gw_ScriptGame_FighterMod(port, owner, 0, gs_fbits(1.0f), gs_fbits(1.0f), gs_fbits(1.0f), gs_fbits(1.0f), gs_fbits(1.0f), gs_fbits(1.0f), gs_fbits(1.0f), gs_fbits(1.0f));
    if (!have_rules) { GsHitRules r; memset(&r, 0, sizeof r); r.status = status_bits; gs_hr_apply_targets(port, -1, owner, &r); }
    /* crit: keep a configuration the build already owns (a `begin` would also clear the forced-crit count); rewrite slots, zero the ones the variant lacks */
    {
        int has_now = gw_ScriptGame_CritRead(entity, 0) == owner ? gw_ScriptGame_CritRead(entity, 1) : 0, want = 0;
        int owned = gw_ScriptGame_CritRead(entity, 0) == owner;
        for (j = 0; j < v->nops; ++j) {
            GsSimOp *o = &v->ops[j];
            if (o->kind != GS_SIM_CRIT) continue;
            have_crit = 1;
            if (o->eff[7] & 1) {  /* begin: only when nothing is owned yet or the floor differs */
                if (!owned || gw_ScriptGame_CritRead(entity, 2) != o->eff[5]) { gw_ScriptGame_CritSet(entity, owner, o->eff[0], o->eff[1], o->eff[2], o->eff[3], o->eff[4], o->eff[5], 0, o->eff[7]); owned = 1; }
                continue;
            }
            gw_ScriptGame_CritSet(entity, owner, o->eff[0], o->eff[1], o->eff[2], o->eff[3], o->eff[4], o->eff[5], o->eff[6], o->eff[7]);
            want |= 1 << o->eff[0];
        }
        if (!have_crit) { if (owned) gw_ScriptGame_CritSet(entity, owner, 0, 0, 0, 0, 0, 0, 0, 2); }
        else for (k = 0; k < 10; ++k) if ((has_now & (1 << k)) && !(want & (1 << k)))
            gw_ScriptGame_CritSet(entity, owner, k, gs_fbits(0.0f), gs_fbits(1.0f), gs_fbits(1.0f), gs_fbits(1.0f), 0, 0, 4);
    }
    (void) present;
}

/* Everything the build owns goes: called at every scene start, so a match without a build is exactly as it always was. */
void gw_Script_NetBuildSceneBegin(void) {
    if (gs_nb_applied) {
        gw_ScriptGame_FighterModsRelease(GW_NB_OWNER);
        gw_ScriptGame_HitRulesRelease(GW_NB_OWNER);
        gs_caps_release(GW_NB_OWNER);
        gw_ScriptGame_ArmorRelease(GW_NB_OWNER);
        gw_log("netbuild: released the previous match's build");
    }
    gw_ScriptGame_BuildClear();
    gs_nb_applied = 0;
}
/* gw_RB_Iterations, when the session opens (frame -124, before the first snapshot): both peers, the same frame, the same ops. A match in which
 * a scene had no agreed build (nothing staged) is untouched. Offline matches never reach here. */
extern int gw_Netplay_Envoy(void);
void gw_Script_NetBuildApply(void) {
    int i;
    if (gw_Script_NetBuildStaged() == 0) return;
    /* A staged build is only ever for an Envoy match: an online match whose room did not agree on the Envoy mode (Random Opponent, a host with the switch off)
       must not pick up what an earlier set left staged. (Offline rollback sessions - the bench SyncTest - apply what a test staged by hand.) */
    if (gw_Netplay_Enabled() && !gw_Netplay_Envoy()) { gw_log("netbuild: a staged build exists but this online match is not an Envoy match - not applied"); return; }
    gw_Script_NetBuildSceneBegin();
    for (i = 0; i < GW_NB_SLOTS; ++i) {
        int ok;
        if (!gs_nb[i].have) continue;
        ok = gs_nb_apply_slot(i);
        if (gs_nb_prog[i].have) {   /* stage 4: the triggered program goes into the game's snapshotted state */
            int k, sealed;
            for (k = 0; k < GW_MODS_WORDS; ++k) gw_ScriptMods_ProgWord(i, k, gs_nb_prog[i].words[k]);
            sealed = gw_ScriptMods_ProgSeal(i);
            gw_log("netbuild: slot %d triggered program %s (rules %d, variants %d, digest %08x)", i + 1, sealed ? "sealed" : "REFUSED", gs_nb_prog[i].words[1], gs_nb_prog[i].nvariants, gs_nb_prog[i].digest);
        }
        gw_ScriptGame_BuildSet(i, (int) gs_nb[i].rec.dig_hi, (int) gs_nb[i].rec.dig_lo, (int) gs_nb[i].ops_word, ok, gs_nb[i].rec.nmods);
        gw_log("netbuild: applied slot=%d ops=%d/%d record=%s ops=%08x", i + 1, ok, gs_nb[i].nops, gs_nb[i].rec.digest, gs_nb[i].ops_word);
        if (i == 0) { /* test hook: one peer's table is poisoned (MELEE_ENVOY_POISON=1), so its hash word must disagree */
            const char *p = getenv("MELEE_ENVOY_POISON");
            if (p != NULL && p[0] == '1') {
                gw_ScriptGame_BuildSet(0, (int) gs_nb[0].rec.dig_hi, (int) (gs_nb[0].rec.dig_lo ^ 1u), (int) gs_nb[i].ops_word, ok, gs_nb[0].rec.nmods);
                gw_log("netbuild: POISONED slot 1's record word on this peer (test)");
            }
        }
    }
    gs_nb_applied = 1;
    gw_log("netbuild: word=%08x hash-word=%08x", gw_Script_NetBuildWord(), gw_ScriptGame_BuildHashWord());
}

/* The engine's Script_GameEvent sites reach the evaluator before the host's own gating (called from gw_Script_GameEvent, resimulated frames included). */
extern void gw_ScriptMods_GameEvent(int what, int a, int b, int c, int d);

/* gd.netmods(): READ-ONLY, allowed online. {active, ticks, events, variants_applied, fx_applied, ports = { {loaded, rules, digest, frame, dropped, queue, mask, variant,
 * statuses = {burn = {stacks, max, left, amount, cause}, ...}} , ... }}. `left` is frames until expiry. For presentation and tests; gameplay never reads it. */
static int l_netmods(lua_State *L) {
    static const char *names[8] = {"", "burn", "shock", "chill", "curse", "haste", "guarded", "momentum"};
    int p, i, active = 0;
    char hex[20];
    for (p = 0; p < GW_NB_SLOTS; ++p) active += gw_ScriptMods_Read(p, 0, 0);
    lua_createtable(L, 0, 8);
    gs_setbool(L, "active", active != 0);
    gs_setint(L, "ticks", gw_ScriptMods_Read(0, 7, 0));
    gs_setint(L, "events", gw_ScriptMods_Read(0, 8, 0));
    gs_setint(L, "variants_applied", gw_ScriptMods_Read(0, 9, 0));
    gs_setint(L, "fx_applied", gw_ScriptMods_Read(0, 10, 0));
    lua_createtable(L, GW_NB_SLOTS, 0);
    for (p = 0; p < GW_NB_SLOTS; ++p) {
        int frame = gw_ScriptMods_Read(p, 2, 0);
        lua_createtable(L, 0, 10);
        gs_setbool(L, "loaded", gw_ScriptMods_Read(p, 0, 0));
        gs_setint(L, "rules", gw_ScriptMods_Read(p, 6, 0));
        snprintf(hex, sizeof hex, "%08x", (unsigned) gw_ScriptMods_Read(p, 1, 0)); lua_pushstring(L, hex); lua_setfield(L, -2, "digest");
        gs_setint(L, "frame", frame);
        gs_setint(L, "dropped", gw_ScriptMods_Read(p, 3, 0));
        gs_setint(L, "queue", gw_ScriptMods_Read(p, 11, 0));
        gs_setint(L, "mask", gw_ScriptMods_Read(p, 5, 0));
        gs_setint(L, "variant", gw_ScriptMods_Read(p, 4, 0));
        lua_createtable(L, 0, 7);
        for (i = 1; i <= 7; ++i) {
            if (!gw_ScriptMods_Read(p, 20, i)) continue;
            lua_createtable(L, 0, 5);
            gs_setint(L, "stacks", gw_ScriptMods_Read(p, 21, i));
            gs_setint(L, "max", gw_ScriptMods_Read(p, 22, i));
            gs_setint(L, "left", gw_ScriptMods_Read(p, 23, i) - frame);
            { union { int i; float f; } u; u.i = gw_ScriptMods_Read(p, 25, i); gs_setnum(L, "amount", u.f); }
            gs_setint(L, "cause", gw_ScriptMods_Read(p, 26, i));
            lua_setfield(L, -2, names[i]);
        }
        lua_setfield(L, -2, "statuses");
        lua_rawseti(L, -2, p + 1);
    }
    lua_setfield(L, -2, "ports");
    return 1;
}

static int l_netbuild(lua_State *L) {
    int i;
    char hex[20];
    lua_createtable(L, 0, 6);
    gs_setint(L, "staged", gw_Script_NetBuildStaged());
    snprintf(hex, sizeof hex, "%08x", gw_Script_NetBuildWord()); lua_pushstring(L, hex); lua_setfield(L, -2, "word");
    gs_setbool(L, "applied", gs_nb_applied);
    lua_createtable(L, GW_NB_SLOTS, 0);
    for (i = 0; i < GW_NB_SLOTS; ++i) {
        lua_createtable(L, 0, 10);
        gs_setbool(L, "staged", gs_nb[i].have);
        if (gs_nb[i].have) {
            lua_pushstring(L, gs_nb[i].record); lua_setfield(L, -2, "record");
            lua_pushstring(L, gs_nb[i].rec.digest); lua_setfield(L, -2, "digest");
            snprintf(hex, sizeof hex, "%08x", gs_nb[i].ops_word); lua_pushstring(L, hex); lua_setfield(L, -2, "ops");
            gs_setint(L, "nops", gs_nb[i].nops);
        }
        gs_setbool(L, "active", gw_ScriptGame_BuildRead(i, 0));
        if (gw_ScriptGame_BuildRead(i, 0)) {
            snprintf(hex, sizeof hex, "%08x%08x", (unsigned) gw_ScriptGame_BuildRead(i, 1), (unsigned) gw_ScriptGame_BuildRead(i, 2)); lua_pushstring(L, hex); lua_setfield(L, -2, "applied_record");
            snprintf(hex, sizeof hex, "%08x", (unsigned) gw_ScriptGame_BuildRead(i, 3)); lua_pushstring(L, hex); lua_setfield(L, -2, "applied_ops");
            gs_setint(L, "applied_nops", gw_ScriptGame_BuildRead(i, 4));
            {   /* what the match actually runs on, read back from the game's own tables: fighter values by name, and the hit rules (count, and a word over their fields) */
                static const char *keys[11] = {"damage_dealt","damage_taken","run_speed","air_speed","shield_max","jump_height","air_jump_height","knockback_taken","fall_speed","weight","shield_regen"};
                unsigned rw = 0x811C9DC5u;
                int k, e, f, rc = 0;
                if (gw_ScriptGame_FighterModOwner(i) != 0) { /* the overlay exists only when the build has a value op */
                    lua_createtable(L, 0, 11);
                    for (k = 0; k < 11; ++k) { union { int i; float f; } u; u.i = gw_ScriptGame_FighterModRead(i, k); gs_setnum(L, keys[k], u.f); }
                    lua_setfield(L, -2, "values");
                }
                for (e = i * 2; e <= i * 2 + 1; ++e) {
                    int n = gw_ScriptGame_HitRulesCount(e);
                    rc += n;
                    for (k = 0; k < n; ++k) for (f = 0; f < GS_HR_FIELDS; ++f) if (f != 12) rw = gs_nb_mix(rw, (unsigned) gw_ScriptGame_HitRuleRead(e, k, f));
                }
                gs_setint(L, "rules", rc);
                snprintf(hex, sizeof hex, "%08x", rw); lua_pushstring(L, hex); lua_setfield(L, -2, "rules_word");
            }
            snprintf(hex, sizeof hex, "%08x", (unsigned) gw_ScriptGame_BuildRead(i, 6)); lua_pushstring(L, hex); lua_setfield(L, -2, "live");
        }
        lua_rawseti(L, -2, i + 1);
    }
    lua_setfield(L, -2, "slots");
    return 1;
}
