/* geno_game_lua.inc - Geno slice 5: the game half of fighter Lua (docs/geno.md section 23). Included by geno_game_v2.inc (one
 * translation unit with geno_game.c; it shares that file's statics).
 *
 * WHO OWNS WHAT. The native half runs the Lua (geno_lua_core.h) and owns no simulation state. THIS half owns all of it:
 *   Geno_LuaBlock[player][sub]   the typed state (16 words) + the fault count: a game global (common symbol), so a savestate, a
 *                                rewind and SyncTest cover it with no extra code; folded into GenoDefine_StateDigest (the rollback
 *                                hash) for a define that declares a layout, and for no other fighter
 *   geno_lua_io                  the per-call exchange block: filled before the call, read after it, zeroed again, never carrying
 *                                anything from one call to the next
 * A call is TRANSACTIONAL: the typed state is copied into the io block, Lua works on the copy, and only a call that returned 0 is
 * copied back and has its commands applied (in call order, at this boundary). A fault drops both and sends the fighter to its
 * auto state (Wait on the ground, Fall in the air) so it cannot be stuck in a state whose Lua no longer runs; the fault count is
 * simulation state, so both peers fault on the same tick.
 *
 * WHEN IT RUNS (this increment): "enter" once from geno_enter_state, after the behaviour's own entry; "frame" as the state's anim
 * callback "lua", once per logic frame. Physics, collision, landing and ledge rules stay the state's native callbacks.
 * Resimulation re-runs both exactly as it re-runs the native callbacks: the call reads only the fighter, the pad and the block, and
 * writes only the same. */

extern int Geno_LuaFn(int p, int s, int phase);
extern int Geno_LuaSlots(int p);
extern int Geno_LuaCall(int p, int s, int phase, void* io);

typedef struct GenoLuaIo {
    u32 w[GENO_LUA_IO_WORDS];
} GenoLuaIo;

static GenoLuaIo geno_lua_io;
static s32 geno_lua_depth; /* nested enter -> go -> enter chains; always back to 0 when a call returns */

static u32 geno_lua_fbits(f32 f)
{
    union {
        f32 f;
        u32 u;
    } c;
    c.f = f;
    return c.u;
}

static f32 geno_lua_bitsf(u32 u)
{
    union {
        f32 f;
        u32 u;
    } c;
    c.u = u;
    return c.f;
}

static GenoLuaBlock* geno_lua_block(Fighter* fp)
{
    int p = fp->player_id < GENO_PLAYERS ? fp->player_id : GENO_PLAYERS - 1;
    return &Geno_LuaBlock[p][fp->is_sub_fighter ? 1 : 0];
}

/* For tests and the LAB: the fighter's block. */
void* GenoGame_LuaBlockOf(Fighter* fp)
{
    return geno_lua_block(fp);
}

/* gd.fighter_lua(port): what: 0 profile (-1 none), 1 fault count, 2 last fault, 3+i slot i's word. The player's main fighter. */
int GenoGame_LuaInfo(int port, int what)
{
    Fighter_GObj* g;
    HSD_GObj* cur;
    if (HSD_GObjPLinkHead == NULL) {
        return -1;
    }
    for (cur = HSD_GObjPLinkHead[HSD_GOBJ_PLINK_FIGHTER]; cur != NULL; cur = cur->next) {
        Fighter* fp = GET_FIGHTER(cur);
        if (fp->player_id == port && !fp->is_sub_fighter) {
            GenoState* st = geno_state(fp);
            GenoLuaBlock* lb = geno_lua_block(fp);
            g = cur;
            (void) g;
            if (what == 0) {
                return st->profile >= 0 && Geno_LuaSlots(st->profile) > 0 ? st->profile : -1;
            }
            if (what == 1) {
                return (int) lb->faults;
            }
            if (what == 2) {
                return lb->last_fault;
            }
            if (what >= 3 && what < 3 + GENO_LUA_STATE_SLOTS) {
                return lb->slot[what - 3];
            }
            return -1;
        }
    }
    return -1;
}

/* The Lua of profile p, state s: 1 when `phase` has a function (the guard that keeps every fighter without Lua at one cheap test). */
static int geno_lua_has(GenoState* st, int s, int phase)
{
    return st->profile >= 0 && s >= 0 && Geno_LuaFn(st->profile, s, phase);
}

/* GENO_LUA_IO_HIT_FROM: where the last hit's attacker (the source port the hit recorded) is, relative to the fighter's facing,
 * from the two fighters' positions now. 0 when the hit had no fighter attacker or it is not on the field. */
static u32 geno_lua_hit_from(Fighter* fp, GenoState* st)
{
    HSD_GObj* cur;
    if (st->hit_port < 0 || HSD_GObjPLinkHead == NULL) {
        return 0;
    }
    for (cur = HSD_GObjPLinkHead[HSD_GOBJ_PLINK_FIGHTER]; cur != NULL; cur = cur->next) {
        Fighter* ap = GET_FIGHTER(cur);
        if (ap != fp && ap->player_id == st->hit_port && !ap->is_sub_fighter) {
            f32 dx = (ap->cur_pos.x - fp->cur_pos.x) * (fp->facing_dir < 0.0f ? -1.0f : 1.0f);
            return dx >= 0.0f ? 1u : 2u;
        }
    }
    return 0;
}

static void geno_lua_fill(Fighter* fp, GenoState* st, GenoLuaBlock* lb, int anim_ended)
{
    GenoLuaIo* io = &geno_lua_io;
    int i;
    for (i = 0; i < GENO_LUA_IO_WORDS; i++) {
        io->w[i] = 0;
    }
    io->w[GENO_LUA_IO_FLAGS] = (fp->ground_or_air != GA_Ground ? 1u : 0u) | (fp->facing_dir < 0.0f ? 2u : 0u) | (anim_ended ? 4u : 0u);
    io->w[GENO_LUA_IO_ACTION_FRAME] = (u32) st->action_time;
    io->w[GENO_LUA_IO_MOTION] = (u32) fp->motion_id;
    io->w[GENO_LUA_IO_HELD] = geno_buttons(fp, 0);
    io->w[GENO_LUA_IO_PRESSED] = geno_buttons(fp, 1);
    io->w[GENO_LUA_IO_STICK_X] = geno_lua_fbits(fp->input.lstick[0].x);
    io->w[GENO_LUA_IO_STICK_Y] = geno_lua_fbits(fp->input.lstick[0].y);
    io->w[GENO_LUA_IO_POS_X] = geno_lua_fbits(fp->cur_pos.x);
    io->w[GENO_LUA_IO_POS_Y] = geno_lua_fbits(fp->cur_pos.y);
    io->w[GENO_LUA_IO_VEL_X] = geno_lua_fbits(fp->self_vel.x);
    io->w[GENO_LUA_IO_VEL_Y] = geno_lua_fbits(fp->self_vel.y);
    io->w[GENO_LUA_IO_PERCENT] = geno_lua_fbits(fp->dmg.x1830_percent);
    io->w[GENO_LUA_IO_HIT_DAMAGE] = geno_lua_fbits(st->hit_damage);
    io->w[GENO_LUA_IO_HIT_FROM] = geno_lua_hit_from(fp, st);
    io->w[GENO_LUA_IO_COUNTERED] = (u32) st->counters;
    for (i = 0; i < GENO_LUA_STATE_SLOTS; i++) {
        io->w[GENO_LUA_IO_STATE + i] = (u32) lb->slot[i];
    }
}

/* Runs one phase of state s's Lua. Returns 1 when a command changed the fighter's action (the caller must not also run the
 * default end-of-animation transition), 0 otherwise (nothing to run, no command changed it, or the call faulted and the fighter was
 * sent to auto -- which is also an action change, so that returns 1). */
static int geno_lua_run(Fighter_GObj* gobj, Fighter* fp, GenoState* st, int s, int phase)
{
    GenoLuaBlock* lb;
    GenoLuaIo* io = &geno_lua_io;
    int rc, n, i, changed = 0;
    f32 lr;
    if (!geno_lua_has(st, s, phase)) {
        return 0;
    }
    if (geno_lua_depth >= 4) {
        /* enter -> go -> enter ... without end: a fault of the chain, handled as any other */
        lb = geno_lua_block(fp);
        lb->faults++;
        lb->last_fault = GENO_LUA_FAULT_CMDS;
        return geno_go(gobj, fp, st, GENO_AUTO);
    }
    lb = geno_lua_block(fp);
    geno_lua_fill(fp, st, lb, phase == GENO_LUA_PHASE_FRAME && geno_anim_ended(gobj));
    geno_lua_depth++;
    rc = Geno_LuaCall(st->profile, s, phase, io);
    if (rc != 0) {
        lb->faults++;
        lb->last_fault = rc;
        for (i = 0; i < GENO_LUA_IO_WORDS; i++) {
            io->w[i] = 0;
        }
        geno_go(gobj, fp, st, GENO_AUTO);
        geno_lua_depth--;
        return 1; /* sent to auto: the action changed */
    }
    for (i = 0; i < GENO_LUA_STATE_SLOTS; i++) {
        lb->slot[i] = (s32) io->w[GENO_LUA_IO_STATE + i];
    }
    n = (int) io->w[GENO_LUA_IO_NCMDS];
    if (n > GENO_LUA_MAX_CMDS) {
        n = GENO_LUA_MAX_CMDS;
    }
    lr = fp->facing_dir < 0.0f ? -1.0f : 1.0f;
    for (i = 0; i < n; i++) {
        u32 op = io->w[GENO_LUA_IO_CMDS + 3 * i], a = io->w[GENO_LUA_IO_CMDS + 3 * i + 1], b = io->w[GENO_LUA_IO_CMDS + 3 * i + 2];
        switch (op) {
        case GENO_LUA_CMD_GO:
            if (geno_go(gobj, fp, st, a)) {
                changed = 1;
            }
            break;
        case GENO_LUA_CMD_VELOCITY: {
            f32 fwd = geno_lua_bitsf(a), up = geno_lua_bitsf(b);
            if (fp->ground_or_air == GA_Ground) {
                fp->gr_vel = fwd * lr;
                if (geno_test_capture) {
                    fp->self_vel.x = fwd * lr;
                } else {
                    ftCommon_SetSelfMovementFromGroundedMovement(gobj);
                }
            } else {
                fp->self_vel.x = fwd * lr;
                fp->self_vel.y = up;
            }
            break;
        }
        case GENO_LUA_CMD_HITBOX_DAMAGE: {
            int h;
            f32 d = geno_lua_bitsf(b);
            for (h = 0; h < 4; h++) {
                if (a & (1u << h)) {
                    fp->x914[h].damage = d < 0.0f ? 0.0f : d;
                    ScriptGame_HitRuleDamageOnly(&fp->x914[h]);
                }
            }
            break;
        }
        case GENO_LUA_CMD_LOOP:
            if (!geno_test_capture) {
                Fighter_ChangeMotionState(gobj, fp->motion_id,
                                          Ft_MF_KeepFastFall | Ft_MF_KeepGfx | Ft_MF_KeepSfx | Ft_MF_SkipModelPartVis |
                                              Ft_MF_SkipItemVis | Ft_MF_SkipNametagVis | Ft_MF_KeepSwordTrail,
                                          0.0f, 1.0f, 0.0f, NULL);
            } else {
                geno_test_changes++;
            }
            break;
        case GENO_LUA_CMD_TURN:
            fp->facing_dir = -fp->facing_dir;
            if (fp->parts != NULL) {
                ftPartSetRotY(fp, 0, 1.5707964f * fp->facing_dir); /* as GENO_VAL_FACING does */
            }
            lr = fp->facing_dir < 0.0f ? -1.0f : 1.0f; /* a later velocity command in this call is relative to the new facing */
            break;
        default:
            break;
        }
    }
    for (i = 0; i < GENO_LUA_IO_WORDS; i++) {
        io->w[i] = 0;
    }
    geno_lua_depth--;
    return changed;
}

/* geno_enter_state, after the behaviour's own entry. */
static void geno_lua_enter(Fighter_GObj* gobj, Fighter* fp, GenoState* st, int s)
{
    if (geno_lua_has(st, s, GENO_LUA_PHASE_ENTER)) {
        geno_lua_run(gobj, fp, st, s, GENO_LUA_PHASE_ENTER);
    }
}

/* anim callback "lua": the state's "frame" function once a frame; with no command that changed the action, the animation's end
 * takes the state's "next" as for every Geno state. */
static void geno_cb_anim_lua(Fighter_GObj* gobj)
{
    Fighter* fp = GET_FIGHTER(gobj);
    GenoState* st = geno_state(fp);
    int s = geno_cur_state(fp, st);
    if (s < 0) {
        geno_cb_anim_next(gobj);
        return;
    }
    if (geno_lua_run(gobj, fp, st, s, GENO_LUA_PHASE_FRAME)) {
        return;
    }
    if (geno_anim_ended(gobj)) {
        geno_go(gobj, fp, st, geno_next_of(st, s));
    }
}

/* GenoDefine_StateDigest's Lua term: 0 when the profile declares no layout (the define keeps the digest it had). */
static u32 geno_lua_digest(Fighter* fp, GenoState* st)
{
    GenoLuaBlock* lb;
    u32 h = 0x4C554131u; /* "LUA1" */
    int i;
    if (st->profile < 0 || Geno_LuaSlots(st->profile) <= 0) {
        return 0;
    }
    lb = geno_lua_block(fp);
    for (i = 0; i < GENO_LUA_STATE_SLOTS; i++) {
        h = geno_dg(h, (u32) lb->slot[i]);
    }
    h = geno_dg(h, lb->faults);
    h = geno_dg(h, (u32) lb->last_fault);
    return h != 0 ? h : 1u;
}
