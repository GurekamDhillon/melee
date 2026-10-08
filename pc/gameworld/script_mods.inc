/* Online Envoy, stage 4: the native TRIGGERED-RULE evaluator in the game. Included by script_game.c after script_fighter_interrupt.inc (before script_build.inc,
 * whose hash word covers it).
 *
 * script_mods_core.h holds the evaluator (pure C, parity-tested against the Lua modifier engine). This file is its game half:
 *   - STATE: one SmWorld (the programs, 7 status slots per seat, recent-event frames, the event queue) plus a few words of bookkeeping, all plain game
 *     BSS, so a snapshot restore (rollback) puts it back exactly like every other piece of match state. Nothing here is read from the host mid-match.
 *   - EVENTS come from inside the simulation, never from the post-frame host queue (which is skipped while resimulating):
 *       hits        ScriptGame_ReportHitContext (script_hit_context.inc): move tag, element, grounded flags and both percents are in hand
 *       actions     gw_Script_GameEvent's top (ScriptMods_GameEvent): KO, stock lost, landing, jump / air jump / ledge grab / grab / throw / taunt
 *                   (derived from the motion change with ScriptGame_ActionCategory, like the host), shield hit, perfect shield (ftcoll.c's own sites)
 *       techniques  script_skill_emit: every skill row, the same rows the ring keeps
 *       crit, armour, clank, shock end   at their own sites
 *     Sub-fighters are ignored exactly as the Lua host ignores them (hits excepted: the Lua host counts an Ice Climber's partner as its port).
 *   - ONE TICK per logic frame from ScriptGame_StageFrame (which runs on resimulated frames too): sample the seats, run begin_frame + drain, then write the
 *     results through the setters a script uses offline, under the build owner: percent (the boss-guarded SetPercent choke point), timed armour,
 *     intangibility, forced crits, the interrupt window, the Shock mirror, and the derived tables of the current status mask (the host holds those, see
 *     gw_script_netbuild.inc, and applies them when the mask changes).
 *   - HASH: ScriptMods_HashWord is mixed into ScriptGame_BuildHashWord, hence RB_GameHash and the curated hash: a divergent status, recent frame, program
 *     or queue is a checksum desync at the next compared frame. 0 when no program is loaded.
 * Design: _research/envoy-netplay-scoping-2026-10-05.md architecture (c); docs/superpowers/plans/2026-10-08-envoy-online-stage4.md (workspace).
 */
#include "script_mods_core.h"
#define SCRIPT_MODS_OWNER 100 /* GW_NB_OWNER (gw_script_netbuild.inc): the build owns what it writes */

extern void Script_NetModsVariant(int port, int variant, int status_bits, int curse_bits);
void ScriptGame_SetPercent(int slot, int percent);

static SmWorld script_mods;
static struct {
    int mask[SM_PORTS];           /* the status mask whose derived tables are applied */
    int curse[SM_PORTS];          /* the Marked amount (float bits) added to the incoming launch rule */
    int variant[SM_PORTS];
    int shock_on[SM_PORTS], shock_charges[SM_PORTS], shock_expires[SM_PORTS];
    int ticks, events, variants_applied, fx_applied;
} script_mods_x;

static int script_mods_fbits(float f) { union { float f; int i; } u; u.f = f; return u.i; }
static const int script_mods_status_bits[SM_NSTATUS + 1] = { 0, 1, 0, 4, 8, 16, 32, 64 };
static int script_mods_bits(int mask) {
    int i, b = 0;
    for (i = 1; i <= SM_NSTATUS; ++i) if (mask & (1 << (i - 1))) b |= script_mods_status_bits[i];
    return b;
}
int ScriptMods_Active(void) { return sm_loaded(&script_mods); }

/* A match ended or a new one is being armed: nothing carries over (programs included). */
void ScriptMods_Clear(void)
{
    sm_reset(&script_mods);
    memset(&script_mods_x, 0, sizeof script_mods_x);
}
int ScriptMods_ProgWord(int port, int index, int value) { return sm_prog_word(&script_mods, port, index, value); }
/* Seal a loaded program. (The state is zero already: ScriptMods_Clear ran at the scene start, and sealing one seat must not wipe the other's.) */
int ScriptMods_ProgSeal(int port)
{
    int ok = sm_prog_seal(&script_mods, port);
    if (ok) {
        OSReport("envoy mods: port=%d program sealed rules=%d variants=%d digest=%08x\n", port + 1, script_mods.prog[port].w[1],
                 script_mods.prog[port].w[2], script_mods.prog[port].digest);
    }
    return ok;
}

/* ---- events ------------------------------------------------------------------------------------------------------------------------ */
static int script_mods_listens(int port, int kind)
{
    const SmProgram* p;
    int r, t;
    if (port < 0 || port >= SM_PORTS || !script_mods.prog[port].loaded) return 0;
    p = &script_mods.prog[port];
    for (r = 0; r < p->w[1]; ++r) {
        const int* rule = &p->w[SM_HDR + r * SM_RW];
        for (t = 0; t < rule[1]; ++t) if (rule[4 + t * SM_TW] == kind) return 1;
    }
    return 0;
}
static int script_mods_stocks(int port) { return port >= 0 && port < 6 ? (int) Player_GetStocks(port) : 0; }
static void script_mods_put(const SmEvent* e)
{
    if (sm_emit(&script_mods, e)) script_mods_x.events++;
}
static void script_mods_simple(int kind, int port, int target)
{
    SmEvent e;
    if (port < 0 || port >= SM_PORTS) return;
    if (target >= SM_PORTS) target = -1;
    sm_event_init(&e, kind, port, target);
    script_mods_put(&e);
}
static void script_mods_ctx(SmCtx* c, int has_percent, float percent, int ground, int port)
{
    memset(c, 0, sizeof *c);
    if (has_percent) { c->has_percent = 1; c->percent = percent; }
    if (ground >= 0) { c->has_grounded = 1; c->grounded = ground; }
    if (port >= 0 && port < 6 && Player_GetPlayerSlotType(port) != 3) { c->has_stocks = 1; c->stocks = script_mods_stocks(port); }
}
/* A connecting hit (ScriptGame_ReportHitContext): hit_dealt for the attacker's seat, hit_taken for the victim's, as mod_lab.lua L:hit builds them.
 * move: 1 jab 2 dash attack 3 tilt 4 smash 5 aerial 6 grab 7 throw 8 special 9 projectile; atk_ground -1 when unknown. */
void ScriptMods_Hit(int attacker, int victim, int element, int move, int atk_ground, int vic_ground, float atk_damage, float vic_damage)
{
    static const unsigned move_tag[10] = { 0, SM_T_JAB, SM_T_DASH_ATTACK, SM_T_TILT, SM_T_SMASH, SM_T_AERIAL, SM_T_GRAB, SM_T_THROW, SM_T_SPECIAL, SM_T_PROJECTILE };
    unsigned tags = 0, taken;
    SmEvent e;
    SmCtx actx, vctx;
    if (!sm_loaded(&script_mods) || victim < 0 || victim >= SM_PORTS) return;
    if (move >= 0 && move < 10) tags |= move_tag[move];
    tags |= element == 0 ? SM_T_NORMAL : element == 1 ? SM_T_FIRE : element == 2 ? SM_T_ELECTRIC : element == 5 ? SM_T_ICE : element == 13 ? SM_T_DARKNESS : 0;
    taken = tags;
    if (atk_ground >= 0) tags |= atk_ground ? SM_T_GROUNDED : SM_T_AIRBORNE;
    taken |= vic_ground ? SM_T_GROUNDED : SM_T_AIRBORNE;
    script_mods_ctx(&actx, 1, atk_damage, atk_ground, attacker);
    script_mods_ctx(&vctx, 1, vic_damage, vic_ground, victim);
    if (attacker >= 0 && attacker < SM_PORTS) {
        sm_event_init(&e, SM_E_HIT_DEALT, attacker, victim);
        e.tags = tags; e.has_self = 1; e.self = actx; e.has_target = 1; e.target_ctx = vctx;
        script_mods_put(&e);
    }
    sm_event_init(&e, SM_E_HIT_TAKEN, victim, attacker >= 0 && attacker < SM_PORTS ? attacker : -1);
    e.tags = taken; e.has_self = 1; e.self = vctx; e.has_target = 1; e.target_ctx = actx;
    script_mods_put(&e);
}
/* The engine's Script_GameEvent sites, tapped before the host's own gating (so a resimulated frame sees them too). */
void ScriptMods_GameEvent(int what, int a, int b, int c, int d)
{
    if (!sm_loaded(&script_mods)) return;
    switch (what) {
    case 1: /* LAB_EV_ACTION: a port, b old motion, c new motion, d sub */
        if (b != c && !d) {
            int cat = ScriptGame_ActionCategory(c);
            static const int kind[7] = { 0, SM_E_JUMP, SM_E_AIR_JUMP, SM_E_LEDGE_GRAB, SM_E_GRAB, SM_E_THROW, SM_E_TAUNT };
            if (cat >= 1 && cat <= 6) script_mods_simple(kind[cat], a, -1);
        }
        break;
    case 4: /* LAB_EV_LAND: a port, b motion, c sub */
        if (!c) script_mods_simple(SM_E_LANDING, a, -1);
        break;
    case 14: /* KO: a attacker (-1 none), b victim */
        if (a >= 0) script_mods_simple(SM_E_KO_DEALT, a, b);
        break;
    case 15: /* a port lost a stock */
        script_mods_simple(SM_E_STOCK_LOST, a, -1);
        break;
    case 16: /* ftcoll.c's own action sites: a port, b motion, c category (7 shield hit, 8 perfect shield; 1-6 are derived above), d sub */
        if (!d && c == 7) script_mods_simple(SM_E_SHIELD_HIT, a, -1);
        else if (!d && c == 8) script_mods_simple(SM_E_PERFECT_SHIELD, a, -1);
        break;
    }
}
/* One skill row (script_skill_emit), mod_lab.lua L:skill: entity 1..12, kinds as SK_*. */
void ScriptMods_Skill(int kind, int entity, int i1, int i2, int i3, float f1, float f2, int air_frames, int aerial_hit)
{
    static const int map[24] = { 0, SM_E_LCANCEL, SM_E_LCANCEL_MISS, SM_E_AUTO_CANCEL, SM_E_LCANCEL_HIT, SM_E_WAVEDASH, SM_E_WAVELAND, SM_E_LEDGE_DASH,
        SM_E_AIR_DODGE, 0 /* perfect shield: the legacy site */, SM_E_TECH, SM_E_TECH_MISS, SM_E_DASH_DANCE, SM_E_SHORT_HOP, SM_E_FULL_HOP, SM_E_FAST_FALL,
        SM_E_SHIELD_DROP, SM_E_JC_GRAB, SM_E_JC_USMASH, SM_E_SDI, SM_E_COMBO, SM_E_COMBO_END, 0, 0 };
    SmEvent e;
    int port, target = -1, ev;
    if (!sm_loaded(&script_mods) || kind < 1 || kind > 21 || entity < 1 || entity > 6) return;
    ev = map[kind];
    if (!ev) return;
    port = entity - 1;
    if (ev == SM_E_COMBO || ev == SM_E_COMBO_END) {
        /* a combo belongs to its attacker (entity i2); no attacker, no event */
        if (i2 < 1 || i2 > 6) return;
        target = port; port = i2 - 1;
    }
    if (port >= SM_PORTS || !script_mods_listens(port, ev)) return;
    if (target >= SM_PORTS) target = -1;
    sm_event_init(&e, ev, port, target);
    e.has_self = 1;
    script_mods_ctx(&e.self, 0, 0, -1, port);
    e.self.has_air = 1; e.self.air_frames = air_frames; e.self.has_ahit = 1; e.self.aerial_hit = aerial_hit != 0;
    if (ev == SM_E_LCANCEL || ev == SM_E_LCANCEL_HIT || ev == SM_E_LCANCEL_MISS || ev == SM_E_AUTO_CANCEL) { e.aerial = i1 >= 0 && i1 < 5 ? i1 : -1; e.hit = i2 != 0; }
    if (ev == SM_E_TECH) e.direction = i1 >= 0 && i1 < 5 ? i1 : -1;
    if (ev == SM_E_COMBO || ev == SM_E_COMBO_END) { e.count = i1; e.has_damage = 1; e.damage = f1; }
    script_mods_put(&e);
    (void) i3; (void) f2;
}
/* A Shock charge was spent: the native Shock ended with "spent" (SK_SHOCK_END why 2), so the status ends with it (mod_lab.lua L:shock_end). */
void ScriptMods_ShockEnd(int entity, int why)
{
    int port = entity - 1;
    if (!sm_loaded(&script_mods) || entity < 1 || entity > 6 || port >= SM_PORTS || why != 2) return;
    {
        SmStatus* s = &script_mods.st[port][SM_S_SHOCK];
        if (s->on) { s->on = s->stacks = s->max = s->expires = s->next_tick = s->cause = 0; s->amount = 0; }
        script_mods_x.shock_on[port] = 0;
    }
}
/* The crit decision (script_crit_flush): entities 1..12. */
void ScriptMods_Crit(int attacker_e, int victim_e, float strength)
{
    SmEvent e;
    int port = attacker_e - 1, target = victim_e >= 1 && victim_e <= 6 ? victim_e - 1 : -1;
    if (!sm_loaded(&script_mods) || attacker_e < 1 || attacker_e > 6 || port >= SM_PORTS || !script_mods_listens(port, SM_E_CRIT)) return;
    if (target >= SM_PORTS) target = -1;
    sm_event_init(&e, SM_E_CRIT, port, target);
    e.tags = SM_T_CRITICAL; e.has_strength = 1; e.strength = strength;
    script_mods_put(&e);
}
/* An armour reaction (ScriptGame_ArmorReact): mod_lab.lua L:armor. */
void ScriptMods_Armor(int entity, int absorbed, int broke)
{
    SmEvent e;
    int port = entity - 1;
    if (!sm_loaded(&script_mods) || entity < 1 || entity > 6 || port >= SM_PORTS || !script_mods_listens(port, SM_E_ARMOR)) return;
    sm_event_init(&e, SM_E_ARMOR, port, -1);
    e.absorbed = absorbed != 0; e.broke = broke != 0;
    script_mods_put(&e);
}
/* Fighter against fighter clank (ftcoll.c script_clank_observe): both seats get an event, as L:clank does. */
void ScriptMods_Clank(int a, int b, float damage_a, float damage_b)
{
    SmEvent e;
    if (!sm_loaded(&script_mods) || a < 0 || b < 0 || a == b || a >= SM_PORTS || b >= SM_PORTS) return;
    sm_event_init(&e, SM_E_CLANK, a, b); e.has_clank = 1; e.damage_a = damage_a; e.damage_b = damage_b; script_mods_put(&e);
    sm_event_init(&e, SM_E_CLANK, b, a); e.has_clank = 1; e.damage_a = damage_a; e.damage_b = damage_b; script_mods_put(&e);
}

/* ---- the tick ------------------------------------------------------------------------------------------------------------------------ */
static void script_mods_apply_fx(const SmFx* x)
{
    int entity = x->port + 1;
    switch (x->op) {
    case SM_FX_ARMOR:
        ScriptGame_ArmorSet(entity, SCRIPT_MODS_OWNER, x->atype, script_mods_fbits(x->value), x->frames, -1, script_mods_fbits(-1.0f), script_mods_fbits(-1.0f), x->direction);
        break;
    case SM_FX_INTANGIBLE:
        ScriptGame_CapsEffect(entity, SCRIPT_MODS_OWNER, 0, script_mods_fbits(1.0f), x->frames);
        break;
    case SM_FX_INTERRUPT:
        ScriptGame_IntrWinSet(entity, SCRIPT_MODS_OWNER, x->frames, x->exits, x->flags);
        break;
    case SM_FX_CRIT_NEXT:
        /* a forced crit needs the build's crit configuration to exist (mod_lab.lua technique_ops): its floor is kept */
        if (ScriptGame_CritRead(entity, 0) == SCRIPT_MODS_OWNER)
            ScriptGame_CritSet(entity, SCRIPT_MODS_OWNER, 0, 0, 0, 0, 0, ScriptGame_CritRead(entity, 2), x->count, 8);
        break;
    }
    script_mods_x.fx_applied++;
}
/* Once per logic frame (ScriptGame_StageFrame, after the skill observer so this frame's rows are queued). */
void ScriptMods_StageFrame(void)
{
    SmPlayer pl[SM_PORTS];
    int p;
    if (!sm_loaded(&script_mods)) return;
    for (p = 0; p < SM_PORTS; ++p) {
        Fighter* fp = script_fighter(p);
        memset(&pl[p], 0, sizeof pl[p]);
        if (fp != NULL) {
            pl[p].present = 1;
            pl[p].percent = fp->dmg.x1830_percent;
            pl[p].grounded = fp->ground_or_air != GA_Air;
            pl[p].stocks = (int) Player_GetStocks(p);
            pl[p].x = fp->cur_pos.x; pl[p].y = fp->cur_pos.y; pl[p].has_pos = 1;
        }
    }
    sm_tick(&script_mods, pl);
    script_mods_x.ticks++;
    for (p = 0; p < SM_PORTS; ++p) {
        float d = script_mods.damage[p];
        if (pl[p].present && d != 0.0f) {
            float v = pl[p].percent + d;
            if (v < 0.0f) v = 0.0f;
            if (v > 999.0f) v = 999.0f;
            ScriptGame_SetPercent(p, (int) v);
        }
    }
    { int i; for (i = 0; i < script_mods.nfx; ++i) if (pl[script_mods.fx[i].port].present) script_mods_apply_fx(&script_mods.fx[i]); }
    for (p = 0; p < SM_PORTS; ++p) {
        int mask, curse;
        if (!pl[p].present || !script_mods.prog[p].loaded) continue;
        mask = sm_mask(&script_mods, p);
        curse = script_mods.st[p][SM_S_CURSE].on ? script_mods_fbits(script_mods.st[p][SM_S_CURSE].amount) : 0;
        if (mask != script_mods_x.mask[p] || curse != script_mods_x.curse[p]) {
            int variant = sm_variant(&script_mods, p);
            Script_NetModsVariant(p, variant, script_mods_bits(mask), curse);
            script_mods_x.mask[p] = mask; script_mods_x.curse[p] = curse; script_mods_x.variant[p] = variant;
            script_mods_x.variants_applied++;
        }
        {   /* the Shock status mirrored into the native Shock (mod_lab.lua technique_ops): written when it starts or changes, cleared when it ends */
            const SmStatus* s = &script_mods.st[p][SM_S_SHOCK];
            if (s->on) {
                int left = s->expires - script_mods.frame, charges = s->stacks;
                if (left < 1) left = 1;
                if (left > 3600) left = 3600;
                if (charges < 1) charges = 1;
                if (charges > 8) charges = 8;
                if (!script_mods_x.shock_on[p] || script_mods_x.shock_charges[p] != charges || s->expires > script_mods_x.shock_expires[p]) {
                    ScriptGame_ShockSet(p + 1, SCRIPT_MODS_OWNER, left, charges, script_mods_fbits(1.5f), 0, 1);
                    script_mods_x.shock_on[p] = 1; script_mods_x.shock_charges[p] = charges; script_mods_x.shock_expires[p] = s->expires;
                }
            } else if (script_mods_x.shock_on[p]) {
                ScriptGame_ShockSet(p + 1, SCRIPT_MODS_OWNER, 1, 1, script_mods_fbits(1.5f), 0, 4);
                script_mods_x.shock_on[p] = 0;
            }
        }
    }
}

/* ---- reading and hashing ----------------------------------------------------------------------------------------------------------------- */
/* what: 0 loaded, 1 program digest, 2 frame, 3 dropped, 4 applied variant, 5 applied mask, 6 rules, 7 ticks, 8 events, 9 variants applied, 10 fx applied,
 * 11 queue length, 12 recent[idx], 20+: status idx 1..7 fields: 20 on, 21 stacks, 22 max, 23 expires, 24 next tick, 25 amount bits, 26 cause */
int ScriptMods_Read(int port, int what, int idx)
{
    if (port < 0 || port >= SM_PORTS) return 0;
    switch (what) {
    case 0: return script_mods.prog[port].loaded;
    case 1: return (int) script_mods.prog[port].digest;
    case 2: return script_mods.frame;
    case 3: return script_mods.dropped;
    case 4: return script_mods_x.variant[port];
    case 5: return script_mods_x.mask[port];
    case 6: return script_mods.prog[port].loaded ? script_mods.prog[port].w[1] : 0;
    case 7: return script_mods_x.ticks;
    case 8: return script_mods_x.events;
    case 9: return script_mods_x.variants_applied;
    case 10: return script_mods_x.fx_applied;
    case 11: return script_mods.nq;
    case 12: return idx >= 1 && idx < SM_NEV ? script_mods.recent[port][idx] : 0;
    }
    if (what >= 20 && what <= 26 && idx >= 1 && idx <= SM_NSTATUS) {
        const SmStatus* s = &script_mods.st[port][idx];
        switch (what) {
        case 20: return s->on; case 21: return s->stacks; case 22: return s->max; case 23: return s->expires; case 24: return s->next_tick;
        case 25: return script_mods_fbits(s->amount); case 26: return s->cause;
        }
    }
    return 0;
}
unsigned ScriptMods_HashWord(void)
{
    unsigned h = sm_hash(&script_mods);
    int p;
    if (h == 0) return 0;
    for (p = 0; p < SM_PORTS; ++p) {
        h = sm_mix(h, (unsigned) script_mods_x.mask[p]); h = sm_mix(h, (unsigned) script_mods_x.curse[p]); h = sm_mix(h, (unsigned) script_mods_x.variant[p]);
        h = sm_mix(h, (unsigned) (script_mods_x.shock_on[p] * 64 + script_mods_x.shock_charges[p]));
    }
    return h | 1u;
}
/* Test hook: poison the state so the hash must change. status 1..7: that status slot's next tick (only visible while it is on); status 0: the frame counter, which
 * is always in the word (MELEE_MODS_POISON=1 on one peer: the rollback hash must report a desync at the first compared frame). */
void ScriptMods_Poison(int port, int status)
{
    if (status == 0) script_mods.frame ^= 1;
    else if (port >= 0 && port < SM_PORTS && status <= SM_NSTATUS) script_mods.st[port][status].next_tick ^= 1;
}

/* Fixture for the native suite (the evaluator proper is parity-tested against the Lua engine by pc/tests/script_mods_core_test.c; this proves the game half:
 * program loading, the hooks' event shapes, the tick, the hash word, snapshot exactness). Returns a bit mask of failures, 0 = all passed. */
int ScriptMods_Test(void)
{
    static SmWorld saved_w;
    static unsigned char saved_x[sizeof script_mods_x];
    SmPlayer pl[SM_PORTS];
    unsigned h0, h1, h2;
    int rc = 0, i, port, base;
    memcpy(&saved_w, &script_mods, sizeof saved_w);
    memcpy(saved_x, &script_mods_x, sizeof saved_x);
    ScriptMods_Clear();
    if (ScriptMods_HashWord() != 0 || ScriptMods_Active()) rc |= 1;                      /* nothing loaded: no word */
    for (port = 0; port < SM_PORTS; ++port) {
        ScriptMods_ProgWord(port, 0, SM_MAGIC); ScriptMods_ProgWord(port, 1, port == 0 ? 1 : 0); ScriptMods_ProgWord(port, 2, 1);
        if (port == 0) {                     /* hit_dealt: put Burn (180 frames, amount 1) on the target */
            base = SM_HDR;
            ScriptMods_ProgWord(port, base + 0, 0); ScriptMods_ProgWord(port, base + 1, 1); ScriptMods_ProgWord(port, base + 2, 1);
            ScriptMods_ProgWord(port, base + 4, SM_E_HIT_DEALT); ScriptMods_ProgWord(port, base + 5, 0);
            base += 4 + SM_NTRIG * SM_TW;
            ScriptMods_ProgWord(port, base + SM_EF_OP, SM_X_STATUS); ScriptMods_ProgWord(port, base + SM_EF_SUBJECT, 1);
            ScriptMods_ProgWord(port, base + SM_EF_STATUS, SM_S_BURN); ScriptMods_ProgWord(port, base + SM_EF_DURATION, 180);
            ScriptMods_ProgWord(port, base + SM_EF_MAX, 1); ScriptMods_ProgWord(port, base + SM_EF_REFRESH, 0);
            ScriptMods_ProgWord(port, base + SM_EF_AMOUNT, script_mods_fbits(1.0f));
        }
        if (!ScriptMods_ProgSeal(port)) rc |= 2;
    }
    if (ScriptMods_ProgWord(0, SM_PROG_WORDS, 1) || ScriptMods_ProgWord(SM_PORTS, 0, 1) || ScriptMods_ProgWord(0, -1, 1)) rc |= 4; /* bounds */
    if (!ScriptMods_Active() || ScriptMods_Read(0, 6, 0) != 1 || ScriptMods_Read(1, 6, 0) != 0) rc |= 8;
    memset(pl, 0, sizeof pl);
    for (i = 0; i < SM_PORTS; ++i) { pl[i].present = 1; pl[i].percent = 20.0f; pl[i].grounded = 1; pl[i].stocks = 3; }
    h0 = ScriptMods_HashWord();
    if (h0 == 0) rc |= 16;
    ScriptMods_Hit(0, 1, 0, 5, 1, 1, 20.0f, 20.0f);                                       /* port 1 hits port 2 */
    if (ScriptMods_Read(0, 11, 0) != 2) rc |= 32;                                        /* hit_dealt and hit_taken are queued */
    sm_tick(&script_mods, pl);
    if (ScriptMods_Read(1, 20, SM_S_BURN) != 1 || ScriptMods_Read(1, 23, SM_S_BURN) != 1 + 180 || ScriptMods_Read(0, 20, SM_S_BURN) != 0) rc |= 64;
    if (ScriptMods_Read(0, 12, SM_E_HIT_DEALT) != 1 || ScriptMods_Read(1, 12, SM_E_HIT_TAKEN) != 1) rc |= 128;   /* recents */
    h1 = ScriptMods_HashWord();
    if (h1 == h0 || ScriptMods_HashWord() != h1) rc |= 256;                              /* the word follows the state and is stable */
    {   /* snapshot exactness: a memcpy of the state is all a restore does */
        static SmWorld snap; unsigned char xs[sizeof script_mods_x];
        memcpy(&snap, &script_mods, sizeof snap); memcpy(xs, &script_mods_x, sizeof xs);
        for (i = 0; i < 70; ++i) sm_tick(&script_mods, pl);                              /* Burn ticks at frame 61: one point of damage */
        h2 = ScriptMods_HashWord();
        if (h2 == h1) rc |= 512;
        memcpy(&script_mods, &snap, sizeof snap); memcpy(&script_mods_x, xs, sizeof xs);
        if (ScriptMods_HashWord() != h1) rc |= 1024;
        for (i = 0; i < 70; ++i) sm_tick(&script_mods, pl);                              /* the resimulation reproduces the same word */
        if (ScriptMods_HashWord() != h2) rc |= 2048;
    }
    ScriptMods_Poison(1, SM_S_BURN);
    if (ScriptMods_HashWord() == h2) rc |= 4096;                                         /* a diverged status is a different word */
    ScriptMods_Poison(1, SM_S_BURN);
    if (ScriptMods_HashWord() != h2) rc |= 8192;
    ScriptMods_GameEvent(15, 1, 2, 0, 0);                                                /* port 2 lost a stock: its statuses end */
    sm_tick(&script_mods, pl);
    if (ScriptMods_Read(1, 20, SM_S_BURN) != 0) rc |= 16384;
    ScriptMods_Clear();
    if (ScriptMods_HashWord() != 0 || ScriptMods_Active()) rc |= 32768;
    memcpy(&script_mods, &saved_w, sizeof saved_w);
    memcpy(&script_mods_x, saved_x, sizeof saved_x);
    return rc;
}
