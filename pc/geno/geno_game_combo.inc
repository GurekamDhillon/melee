/* geno_game_combo.inc - geno 11: the combo graph's game half (docs/geno.md section 25). Included at the end of geno_game.c (one
 * translation unit: it shares that file's statics: geno_state, geno_go, geno_do_change, geno_dg, geno_lua_block).
 *
 * WHAT IT IS. A define may declare routes "from move M, in action frames [lo, hi], when M has hit / whiffed / been blocked, on the
 * ground or in the air, an input (light / heavy / special / jump / grab / shield, a direction relative to the facing, a hold) ->
 * move T". The registry parses them (geno_combo_registry.inc) into words sorted by priority; this file runs them.
 *
 * WHERE IT RUNS (fighter.c, the per-frame input process):
 *   Geno_ComboInput   every frame, hitlag included, before the input callback: records which input classes were pressed this frame
 *                     (age 0) with the stick direction, ages older presses by one logic frame OUTSIDE hitlag only (a press made
 *                     during hitlag is still fresh when hitlag ends: Melee players press during hitlag), counts holds.
 *   Geno_ComboTick    outside hitlag (retail's own `!x2219_b5` block, the one that runs the input callback), BEFORE the action's
 *                     input callback: the first route (priority order) whose source is the current motion, whose window holds the
 *                     action frame (GenoState.action_time: 1-based, hitlag frames excluded, as scripts and counter windows count),
 *                     whose contact condition, ground/air, string depth, meter and Lua gate hold, and whose input class was
 *                     pressed within its buffer with the right direction (and held long enough) is taken: the press is consumed,
 *                     the meter pays, the string grows by one link, the target is entered the way Geno enters it. Then the
 *                     retail callback is skipped for that frame (as when it changes the action itself). No route: retail runs.
 *   So retail priorities stand: shield, grab, jump and the move's own IASA all still run when no route fires, and a route only
 *   exists where the define wrote one. Hitstun, DI and the victim's side are untouched: a route is the attacker's cancel, never
 *   a hold on the victim, so a string drops exactly where Melee's frame advantage says it drops.
 *
 * THE STRING. `links` counts routes taken since the string began; it ends (links 0, "once" marks cleared) the moment the fighter's
 * motion is not the one the last link entered (the move ended, the fighter was hit, landed, shielded ...). A route is taken only
 * while links < its depth (default max_links), so no route set can loop forever.
 *
 * ROLLBACK. Geno_ComboBlock[6][2] is a game global (common symbol in a pc_geno object, so gw_snap snapshots it); no host time, no
 * pointers. Every word is in GenoDefine_StateDigest for a define that declares a combo block (geno_combo_digest); 0 is folded for
 * every other fighter, so no existing hash changes. Inputs come from the fighter's own input record (the rolled-back pad). */

extern int Geno_ComboParam(int p, int what);
extern int Geno_ComboWord(int p, int route, int word);
extern void Geno_ComboNote(int p, int route, int from, int to, int frame, int ki_links);
extern void ftCo_AttackAir_EnterFromMsid(Fighter_GObj* gobj, FtMotionId msid);
extern void ftCo_JumpAerial_Enter_Basic(Fighter_GObj* gobj);
extern void ftCo_Jump_Enter(Fighter_GObj* gobj);

GenoComboBlock Geno_ComboBlock[GENO_PLAYERS][2];

#define GENO_COMBO_STALE 255
/* 1 only inside Geno_ComboTick's own action change (the route's target entry): that change continues the string. Always 0 between
 * calls, so it is not state. */
static s32 geno_combo_entering;
/* Retail's thresholds (PlCo); the fallbacks are only for the headless tests, which run without PlCo loaded. */
#define GENO_COMBO_PLCO(field, fallback) (p_ftCommonData != NULL ? p_ftCommonData->field : (fallback))

static GenoComboBlock* geno_combo_block(Fighter* fp)
{
    int p = fp->player_id < GENO_PLAYERS ? fp->player_id : GENO_PLAYERS - 1;
    return &Geno_ComboBlock[p][fp->is_sub_fighter ? 1 : 0];
}

void* GenoGame_ComboBlockOf(Fighter* fp)
{
    return geno_combo_block(fp);
}

static int geno_combo_on(GenoState* st)
{
    return st->profile >= 0 && Geno_ComboParam(st->profile, GENO_CP_ROUTES) > 0;
}

static void geno_combo_end_string(GenoComboBlock* cb)
{
    int i;
    cb->links = 0;
    cb->motion = -1;
    for (i = 0; i < GENO_COMBO_MAX_ROUTES / 32; i++) {
        cb->once[i] = 0;
    }
}

/* Geno_FighterReset (spawn, respawn: the meter starts again each stock). */
static void geno_combo_reset(Fighter* fp, GenoState* st)
{
    GenoComboBlock* cb = geno_combo_block(fp);
    int i;
    geno_zero(cb, sizeof(*cb));
    cb->last_route = -1;
    cb->motion = -1;
    for (i = 0; i < GENO_CI_COUNT; i++) {
        cb->age[i] = GENO_COMBO_STALE;
    }
    if (geno_combo_on(st)) {
        cb->ki = Geno_ComboParam(st->profile, GENO_CP_KI_START);
    }
}

static void geno_combo_ki_add(Fighter* fp, GenoState* st, s32 d)
{
    GenoComboBlock* cb = geno_combo_block(fp);
    s32 mx = Geno_ComboParam(st->profile, GENO_CP_KI_MAX);
    cb->ki += d;
    if (cb->ki > mx) {
        cb->ki = mx;
    }
    if (cb->ki < 0) {
        cb->ki = 0;
    }
}

/* Geno_OnActionChange: presses made before the action began never feed its routes (the press that started a move does not also
 * cancel it), the contact record belongs to one action, and a motion that is not the last link's ends the string. */
static void geno_combo_action(Fighter* fp, GenoState* st)
{
    GenoComboBlock* cb;
    int i;
    if (!geno_combo_on(st)) {
        return;
    }
    cb = geno_combo_block(fp);
    for (i = 0; i < GENO_CI_COUNT; i++) {
        cb->age[i] = GENO_COMBO_STALE;
    }
    cb->hit = 0;
    cb->blocked = 0;
    if (!geno_combo_entering && cb->links > 0 && (s32) fp->motion_id != cb->motion) {
        geno_combo_end_string(cb);
    }
}

/* ftcoll.c: the fighter's hitbox (kind 0), its hit on a shield (1), its article (2) connected. */
void Geno_ComboContact(Fighter* fp, int kind)
{
    GenoState* st;
    GenoComboBlock* cb;
    if (fp == NULL) {
        return;
    }
    st = geno_state(fp);
    if (!geno_combo_on(st)) {
        return;
    }
    cb = geno_combo_block(fp);
    if (kind == 1) {
        if (!cb->blocked) {
            geno_combo_ki_add(fp, st, Geno_ComboParam(st->profile, GENO_CP_KI_ON_BLOCK));
        }
        cb->blocked = 1;
        return;
    }
    cb->hit = 1;
    geno_combo_ki_add(fp, st, Geno_ComboParam(st->profile, GENO_CP_KI_ON_HIT));
}

/* Geno_OnHit: a hit taken. */
static void geno_combo_taken_hit(Fighter* fp, GenoState* st)
{
    if (geno_combo_on(st)) {
        geno_combo_block(fp)->hits_seen = st->hit_count;
        geno_combo_ki_add(fp, st, Geno_ComboParam(st->profile, GENO_CP_KI_ON_TAKEN));
    }
}

/* The stick's direction relative to the facing; neutral inside the deadzone (Melee's tilt threshold). */
static int geno_combo_dir(Fighter* fp, f32 x, f32 y)
{
    f32 ax = x < 0.0f ? -x : x, ay = y < 0.0f ? -y : y;
    f32 dz = GENO_COMBO_PLCO(x98, 0.3f);
    if (ax < dz && ay < dz) {
        return GENO_CD_NEUTRAL;
    }
    if (ay > ax) {
        return y > 0.0f ? GENO_CD_UP : GENO_CD_DOWN;
    }
    return x * fp->facing_dir >= 0.0f ? GENO_CD_FORWARD : GENO_CD_BACK;
}

/* A smash input this frame: A with a fresh stick flick (retail's own thresholds and windows, as ftCo_AttackS4/Hi4/Lw4 test them),
 * or a C-stick that crossed the smash threshold this frame. Writes the direction. */
static int geno_combo_smash(Fighter* fp, int a, int* dir)
{
    f32 thr = GENO_COMBO_PLCO(dash_smash_stick_threshold, 0.8f);
    f32 cx = fp->input.cstick[0].x, cy = fp->input.cstick[0].y, px = fp->input.cstick[1].x, py = fp->input.cstick[1].y;
    if ((ABS(cx) >= thr && ABS(px) < thr) || (ABS(cy) >= thr && ABS(py) < thr)) {
        *dir = geno_combo_dir(fp, cx, cy);
        return 1;
    }
    if (!a || fp->ground_or_air != GA_Ground) {
        return 0;
    }
    if (ABS(fp->input.lstick[0].x) >= thr && fp->active_timer.lstick.x < GENO_COMBO_PLCO(dash_smash_window, 2)) {
        *dir = fp->input.lstick[0].x * fp->facing_dir >= 0.0f ? GENO_CD_FORWARD : GENO_CD_BACK;
        return 1;
    }
    if (fp->input.lstick[0].y >= GENO_COMBO_PLCO(xCC, 0.6625f) && fp->active_timer.lstick.y < GENO_COMBO_PLCO(xD0, 4)) {
        *dir = GENO_CD_UP;
        return 1;
    }
    if (fp->input.lstick[0].y <= GENO_COMBO_PLCO(xD4, -0.6625f) && fp->active_timer.lstick.y < GENO_COMBO_PLCO(xD8, 4)) {
        *dir = GENO_CD_DOWN;
        return 1;
    }
    return 0;
}

static void geno_combo_press(GenoComboBlock* cb, int cls, int dir)
{
    cb->age[cls] = 0;
    cb->dir[cls] = dir;
}

void Geno_ComboInput(Fighter_GObj* gobj)
{
    Fighter* fp = GET_FIGHTER(gobj);
    GenoState* st = geno_state(fp);
    GenoComboBlock* cb;
    HSD_Pad pr, hd;
    int i, a, d, lag;
    if (!geno_combo_on(st)) {
        return;
    }
    cb = geno_combo_block(fp);
    lag = fp->x2219_b5 ? 1 : 0;
    if (!lag) {
        for (i = 0; i < GENO_CI_COUNT; i++) {
            if (cb->age[i] < GENO_COMBO_STALE) {
                cb->age[i]++;
            }
        }
    }
    pr = fp->input.pressed_buttons;
    hd = fp->input.held_buttons[0];
    a = (pr & HSD_PAD_A) != 0;
    d = GENO_CD_NEUTRAL;
    if (geno_combo_smash(fp, a, &d)) {
        geno_combo_press(cb, GENO_CI_HEAVY, d);
    } else if (a) {
        geno_combo_press(cb, GENO_CI_LIGHT, geno_combo_dir(fp, fp->input.lstick[0].x, fp->input.lstick[0].y));
    }
    if (pr & HSD_PAD_B) {
        geno_combo_press(cb, GENO_CI_SPECIAL, geno_combo_dir(fp, fp->input.lstick[0].x, fp->input.lstick[0].y));
    }
    if (pr & HSD_PAD_XY) {
        geno_combo_press(cb, GENO_CI_JUMP, geno_combo_dir(fp, fp->input.lstick[0].x, fp->input.lstick[0].y));
    }
    if (pr & HSD_PAD_Z) {
        geno_combo_press(cb, GENO_CI_GRAB, geno_combo_dir(fp, fp->input.lstick[0].x, fp->input.lstick[0].y));
    }
    if (pr & (HSD_PAD_L | HSD_PAD_R)) {
        geno_combo_press(cb, GENO_CI_SHIELD, geno_combo_dir(fp, fp->input.lstick[0].x, fp->input.lstick[0].y));
    }
    /* holds: A for light and heavy (a C-stick held past the threshold also holds heavy), B, X/Y, Z, L/R */
    cb->hold[GENO_CI_LIGHT] = (hd & HSD_PAD_A) ? cb->hold[GENO_CI_LIGHT] + 1 : 0;
    cb->hold[GENO_CI_HEAVY] = ((hd & HSD_PAD_A) || ABS(fp->input.cstick[0].x) >= GENO_COMBO_PLCO(dash_smash_stick_threshold, 0.8f) ||
                               ABS(fp->input.cstick[0].y) >= GENO_COMBO_PLCO(dash_smash_stick_threshold, 0.8f))
                                  ? cb->hold[GENO_CI_HEAVY] + 1
                                  : 0;
    cb->hold[GENO_CI_SPECIAL] = (hd & HSD_PAD_B) ? cb->hold[GENO_CI_SPECIAL] + 1 : 0;
    cb->hold[GENO_CI_JUMP] = (hd & HSD_PAD_XY) ? cb->hold[GENO_CI_JUMP] + 1 : 0;
    cb->hold[GENO_CI_GRAB] = (hd & HSD_PAD_Z) ? cb->hold[GENO_CI_GRAB] + 1 : 0;
    cb->hold[GENO_CI_SHIELD] = (hd & (HSD_PAD_L | HSD_PAD_R)) ? cb->hold[GENO_CI_SHIELD] + 1 : 0;
    for (i = 0; i < GENO_CI_COUNT; i++) {
        if (cb->hold[i] > 9999) {
            cb->hold[i] = 9999;
        }
    }
}

static int geno_combo_dir_ok(int want, int got)
{
    if (want == GENO_CD_ANY) {
        return 1;
    }
    if (want == GENO_CD_SIDE) {
        return got == GENO_CD_FORWARD || got == GENO_CD_BACK;
    }
    return want == got;
}

/* The motion a route's source names (-1 any). */
static s32 geno_combo_from_motion(u32 from)
{
    if ((from & 0xFFFF) == GENO_COMBO_FROM_ANY) {
        return -1;
    }
    if ((from >> 28) == GENO_TGT_GENO) {
        return GENO_MOTION_BASE + (s32) (from & 0xFFFF);
    }
    return (s32) (from & 0xFFFF);
}

/* 1 when route r's conditions other than the input hold now (the LAB shows these as "open"). */
static int geno_combo_open(Fighter* fp, GenoState* st, GenoComboBlock* cb, int r)
{
    int p = st->profile;
    s32 from = geno_combo_from_motion((u32) Geno_ComboWord(p, r, GENO_CR_FROM));
    int on = Geno_ComboWord(p, r, GENO_CR_ON), wh = Geno_ComboWord(p, r, GENO_CR_WHERE);
    int lo = Geno_ComboWord(p, r, GENO_CR_WIN_LO), hi = Geno_ComboWord(p, r, GENO_CR_WIN_HI);
    int slot = Geno_ComboWord(p, r, GENO_CR_SLOT);
    s32 t = st->action_time;
    if (from >= 0 && (s32) fp->motion_id != from) {
        return 0;
    }
    if (from < 0 && cb->links == 0) {
        return 0; /* "*" only continues a string */
    }
    if ((wh == GENO_CW_GROUND && fp->ground_or_air != GA_Ground) || (wh == GENO_CW_AIR && fp->ground_or_air == GA_Ground)) {
        return 0;
    }
    if (t < lo || (hi > 0 && t > hi)) {
        return 0;
    }
    if ((on == GENO_CO_HIT && !cb->hit) || (on == GENO_CO_WHIFF && (cb->hit || cb->blocked)) ||
        (on == GENO_CO_BLOCK && !cb->blocked) || (on == GENO_CO_CONTACT && !cb->hit && !cb->blocked))
    {
        return 0;
    }
    if (!Geno_ComboWord(p, r, GENO_CR_STARTS) && cb->links >= Geno_ComboWord(p, r, GENO_CR_DEPTH)) {
        return 0;
    }
    if (cb->links >= Geno_ComboParam(p, GENO_CP_MAX_LINKS) && !Geno_ComboWord(p, r, GENO_CR_STARTS)) {
        return 0;
    }
    if (Geno_ComboWord(p, r, GENO_CR_ONCE) && (cb->once[r >> 5] & (1u << (r & 31)))) {
        return 0;
    }
    if (cb->ki < Geno_ComboWord(p, r, GENO_CR_KI_MIN) || cb->ki < Geno_ComboWord(p, r, GENO_CR_KI_COST)) {
        return 0;
    }
    if (slot >= 0 && slot < GENO_LUA_STATE_SLOTS && geno_lua_block(fp)->slot[slot] < Geno_ComboWord(p, r, GENO_CR_SLOT_MIN)) {
        return 0;
    }
    return 1;
}

static int geno_combo_input_ok(GenoState* st, GenoComboBlock* cb, int r)
{
    int p = st->profile, in = Geno_ComboWord(p, r, GENO_CR_INPUT);
    if (in == GENO_CI_NONE) {
        return 1;
    }
    if (in < 0 || in >= GENO_CI_COUNT) {
        return 0;
    }
    if (Geno_ComboWord(p, r, GENO_CR_HOLD) > 0) {
        /* a hold route: the button is held now, for long enough (a charge), whatever the press's age */
        return cb->hold[in] >= Geno_ComboWord(p, r, GENO_CR_HOLD) && cb->age[in] != GENO_COMBO_STALE &&
               geno_combo_dir_ok(Geno_ComboWord(p, r, GENO_CR_DIR), cb->dir[in]);
    }
    return cb->age[in] <= Geno_ComboWord(p, r, GENO_CR_BUFFER) && geno_combo_dir_ok(Geno_ComboWord(p, r, GENO_CR_DIR), cb->dir[in]);
}

/* Enters a route's target. 1 when the action changed. */
static int geno_combo_enter(Fighter_GObj* gobj, Fighter* fp, GenoState* st, u32 to)
{
    if ((to >> 28) == GENO_COMBO_TGT_ACT && to != GENO_TGT_AUTO && to != GENO_TGT_HELPLESS && to != GENO_NONE_T) {
        int id = (int) (to & 0xFFFF);
        int air = fp->ground_or_air != GA_Ground;
        if (id <= GENO_CA_SPECIAL_LW) {
            if (geno_test_capture) {
                geno_test_target = to;
                geno_test_changes++;
                return 1;
            }
            /* the define's bound special (its geno.json "specials"), ground or air version */
            return Geno_SpecialEnter(gobj, id + (air ? GENO_SP_AIR_N : GENO_SP_N));
        }
        if (id == GENO_CA_JUMP || id == GENO_CA_JUMP_SHORT) {
            if (geno_test_capture) {
                geno_test_target = to;
                geno_test_changes++;
                return 1;
            }
            if (!air) {
                /* the jump squat is skipped, so the short-hop flag the squat would have set is set here: always written (the
                 * union holds the last action's data), 1 for jump_short; ftCo_800CB110 reads it as mv.co.jump.x0 */
                fp->mv.co.kneebend.is_short_hop = (id == GENO_CA_JUMP_SHORT);
                ftCo_Jump_Enter(gobj);
                return 1;
            }
            if ((int) fp->x1968_jumpsUsed < (int) fp->co_attrs.max_jumps) {
                ftCo_JumpAerial_Enter_Basic(gobj);
                return 1;
            }
            return 0;
        }
        if (id >= GENO_CA_AIR_N && id <= GENO_CA_AIR_LW) {
            if (!air) {
                return 0;
            }
            if (geno_test_capture) {
                geno_test_target = to;
                geno_test_changes++;
                return 1;
            }
            ftCo_AttackAir_EnterFromMsid(gobj, (FtMotionId) (ftCo_MS_AttackAirN + (id - GENO_CA_AIR_N)));
            return 1;
        }
        return 0;
    }
    return geno_go(gobj, fp, st, to);
}

int Geno_ComboTick(Fighter_GObj* gobj)
{
    Fighter* fp = GET_FIGHTER(gobj);
    GenoState* st = geno_state(fp);
    GenoComboBlock* cb;
    int p, n, r;
    if (!geno_combo_on(st)) {
        return 0;
    }
    cb = geno_combo_block(fp);
    if (cb->links > 0 && (s32) fp->motion_id != cb->motion) {
        geno_combo_end_string(cb);
    }
    p = st->profile;
    n = Geno_ComboParam(p, GENO_CP_ROUTES);
    for (r = 0; r < n; r++) {
        u32 to;
        int in, from_motion, from_frame, entered;
        if (!geno_combo_open(fp, st, cb, r) || !geno_combo_input_ok(st, cb, r)) {
            continue;
        }
        to = (u32) Geno_ComboWord(p, r, GENO_CR_TO);
        in = Geno_ComboWord(p, r, GENO_CR_INPUT);
        from_motion = (int) fp->motion_id;
        from_frame = (int) st->action_time;
        if (Geno_ComboWord(p, r, GENO_CR_STARTS)) {
            geno_combo_end_string(cb);
        }
        geno_combo_entering = 1;
        entered = geno_combo_enter(gobj, fp, st, to);
        geno_combo_entering = 0;
        if (!entered) {
            continue; /* the target refused (no air jump left, an unbound special): the next route may still fire */
        }
        /* geno_combo_action ran inside the change (ages cleared, contact cleared); now the string's bookkeeping */
        cb = geno_combo_block(fp);
        if (in >= 0 && in < GENO_CI_COUNT) {
            cb->age[in] = GENO_COMBO_STALE;
        }
        cb->ki -= Geno_ComboWord(p, r, GENO_CR_KI_COST);
        if (cb->ki < 0) {
            cb->ki = 0;
        }
        if (Geno_ComboWord(p, r, GENO_CR_ONCE)) {
            cb->once[r >> 5] |= 1u << (r & 31);
        }
        cb->links++;
        cb->taken++;
        cb->last_route = r;
        cb->motion = (s32) fp->motion_id;
        /* the log line names the route (the names live in the native registry) */
        Geno_ComboNote(p, r, from_motion, (int) fp->motion_id, from_frame, (int) cb->ki | (cb->links << 24));
        return 1;
    }
    return 0;
}

/* GenoDefine_StateDigest's combo term: 0 when the profile declares no combo block. */
static u32 geno_combo_digest(Fighter* fp, GenoState* st)
{
    GenoComboBlock* cb;
    const u32* w;
    u32 h = 0x434F4D42u; /* "COMB" */
    int i;
    if (!geno_combo_on(st)) {
        return 0;
    }
    cb = geno_combo_block(fp);
    w = (const u32*) cb;
    for (i = 0; i < (int) (sizeof(*cb) / 4); i++) {
        h = geno_dg(h, w[i]);
    }
    return h != 0 ? h : 1u;
}

/* gd.combo(port) (the LAB): what 0 links, 1 ki (hundredths), 2 last route, 3 routes, 4 hit, 5 blocked, 6 links taken since the
 * reset, 7 max links, 8 ki max, 9 action frame, 10 + r: route r's state now (0 closed, 1 open: waiting for its input, 2 open and
 * its input is buffered). -1 for a fighter without a combo block. The player's main fighter. */
int GenoGame_ComboInfo(int port, int what)
{
    HSD_GObj* cur;
    if (HSD_GObjPLinkHead == NULL) {
        return -1;
    }
    for (cur = HSD_GObjPLinkHead[HSD_GOBJ_PLINK_FIGHTER]; cur != NULL; cur = cur->next) {
        Fighter* fp = GET_FIGHTER(cur);
        if (fp->player_id == port && !fp->is_sub_fighter) {
            GenoState* st = geno_state(fp);
            GenoComboBlock* cb = geno_combo_block(fp);
            if (!geno_combo_on(st)) {
                return -1;
            }
            switch (what) {
            case 0: return cb->links;
            case 1: return cb->ki;
            case 2: return cb->last_route;
            case 3: return Geno_ComboParam(st->profile, GENO_CP_ROUTES);
            case 4: return cb->hit;
            case 5: return cb->blocked;
            case 6: return cb->taken;
            case 7: return Geno_ComboParam(st->profile, GENO_CP_MAX_LINKS);
            case 8: return Geno_ComboParam(st->profile, GENO_CP_KI_MAX);
            case 9: return st->action_time;
            case 11: return st->profile;
            }
            if (what >= 16 && what < 16 + Geno_ComboParam(st->profile, GENO_CP_ROUTES)) {
                int r = what - 16;
                if (!geno_combo_open(fp, st, cb, r)) {
                    return 0;
                }
                return geno_combo_input_ok(st, cb, r) ? 2 : 1;
            }
            return -1;
        }
    }
    return -1;
}

/* offline LAB / tests: set the meter (hundredths, clamped). Returns the new value, -1 without a combo block. */
int GenoGame_ComboSetKi(int port, int ki)
{
    HSD_GObj* cur;
    if (HSD_GObjPLinkHead == NULL) {
        return -1;
    }
    for (cur = HSD_GObjPLinkHead[HSD_GOBJ_PLINK_FIGHTER]; cur != NULL; cur = cur->next) {
        Fighter* fp = GET_FIGHTER(cur);
        if (fp->player_id == port && !fp->is_sub_fighter) {
            GenoState* st = geno_state(fp);
            if (!geno_combo_on(st)) {
                return -1;
            }
            geno_combo_block(fp)->ki = 0;
            geno_combo_ki_add(fp, st, ki);
            return geno_combo_block(fp)->ki;
        }
    }
    return -1;
}
