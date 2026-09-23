/* geno_game_specials.inc - Meta Knight's native specials as Geno behaviours, part of
 * geno_game_v2.inc: Mach Tornado ("geno.tornado") and Drill Rush ("geno.drill.start",
 * "geno.drill", "geno.drill.end"). Hitboxes, timing and effects come from the state's subaction
 * script (Melee hitboxes + Geno REHIT / LINK); these callbacks move the fighter and pick the
 * transitions. The logic is Brawl's, recovered from sora_melee / ft_metaknight.rel (docs/geno.md
 * 16.4): the tornado's spin rate and tap lift (SpecialNSpin + MK kinetic types 0x64 / 0x65), the
 * drill's stick pitch (SpecialSRush) and the end's pitch decay (SpecialSEnd).
 *
 * Per-move state (GenoState, snapshotted):
 *   tornado: move_f[0] spin rate   move_i[0] spin countdown   move_i[1] frames since the last lift
 *            move_i[2] B tapped (Brawl RA-Bit16)              move_i[3] lifts taken
 *   drill:   move_f[0] pitch (deg, + = up)  move_f[1] speed   move_i[0] frames
 *            move_i[1] hit something        move_i[2] bounce request  move_i[3] bounce reason
 */

#define TP(i) geno_param(st, GENO_P_TORNADO_W0 + (i))
#define DP(i) geno_param(st, GENO_P_DRILL_W0 + (i))

/* Melee's analog dead zone for the specials' stick reads. */
static f32 geno_stick_dz(f32 v)
{
    return geno_absf(v) < 0.2875f ? 0.0f : geno_clampf(v, -1.0f, 1.0f);
}

/* Helpless fall with Melee's default landing lag, or Wait on the ground. */
static void geno_helpless(Fighter_GObj* gobj, Fighter* fp, GenoState* st)
{
    geno_do_change(gobj, fp, st,
                   GENO_TARGET(GENO_TGT_MOTION, fp->ground_or_air == GA_Ground
                                                    ? ftCo_MS_Wait
                                                    : ftCo_MS_FallSpecial));
}

/* Moving into a wall? (Brawl's touch bits 2 / 4 against the velocity's sign.) */
static int geno_into_wall(Fighter* fp, f32 vx)
{
    u32 f = fp->coll_data.env_flags;
    return (vx >= 0.0f && (f & Collide_RightWallMask)) || (vx < 0.0f && (f & Collide_LeftWallMask));
}

/* Turn around (the model too, as Melee's own turns do). */
static void geno_turn(Fighter* fp)
{
    fp->facing_dir = -fp->facing_dir;
    if (fp->parts != NULL) {
        ftPartSetRotY(fp, 0, 1.5707964f * fp->facing_dir);
    }
}

/* ---- Mach Tornado ------------------------------------------------------------------------------ */

static void geno_tornado_enter(Fighter_GObj* gobj, Fighter* fp, GenoState* st, int s)
{
    /* entering MK's tornado kinetics: vx *= P1; in the air vy += P0 */
    fp->self_vel.x *= TP(1);
    fp->gr_vel *= TP(1);
    if (fp->ground_or_air == GA_Air) {
        fp->self_vel.y += TP(0);
    }
    fp->fall_fast = 0;
    st->move_f[0] = TP(2); /* spin rate starts at 80 */
    st->move_i[0] = (s32) TP(16);
    st->move_i[1] = 0;
    st->move_i[2] = 0;
    st->move_i[3] = 0;
}

/* The spin ends when its rate has run down to P18 (the PSA's IC-Basic[24] <= 10 check). */
static void geno_tornado_anim(Fighter_GObj* gobj)
{
    Fighter* fp = GET_FIGHTER(gobj);
    GenoState* st = geno_state(fp);
    int s = geno_cur_state(fp, st);
    if (s >= 0 && st->move_f[0] <= TP(18)) {
        u32 t = (u32) Geno_StateNext(st->profile, s);
        if (t != GENO_NONE_T) {
            geno_go(gobj, fp, st, t);
        } else if (geno_param(st, GENO_P_TORNADO_END_HELPLESS) != 0.0f) {
            geno_helpless(gobj, fp, st);
        } else {
            geno_go(gobj, fp, st, GENO_AUTO);
        }
        return;
    }
    if (geno_anim_ended(gobj)) {
        geno_cb_anim_loop(gobj); /* the spin loops until the rate runs down */
    }
}

static void geno_tornado_phys(Fighter_GObj* gobj)
{
    Fighter* fp = GET_FIGHTER(gobj);
    GenoState* st = geno_state(fp);
    f32 sx = geno_stick_dz(fp->input.lstick[0].x);
    f32 r = st->move_f[0];
    int lift;
    /* frames since the last lift (Brawl's concurrent subroutine counts RA-Basic3 every frame) */
    st->move_i[1]++;
    /* execStatus: the spin rate, and B taps */
    if (st->move_i[0] >= 0) {
        st->move_i[0]--;
        r -= TP(15);
        if (fp->input.pressed_buttons & HSD_PAD_B) {
            st->move_i[2] = 1;
        }
        lift = st->move_i[1] >= (s32) TP(11) && st->move_i[2];
        if (lift) {
            if (fp->ground_or_air == GA_Ground) {
                ftCommon_8007D5D4(fp); /* a tap lifts off the ground */
            }
            r += TP(13);
        }
        r = geno_clampf(r, 0.0f, TP(14));
    } else {
        r -= TP(17);
        if (r < 0.0f) {
            r = 0.0f;
        }
    }
    st->move_f[0] = r;
    if (fp->ground_or_air == GA_Ground) {
        /* ground kinetics (0x64): accel P3 x stick to P4 x |stick|, brake P7 */
        f32 v = geno_accel_brake(fp->gr_vel, sx * TP(3), geno_absf(sx) * TP(4), TP(7));
        fp->gr_vel = geno_clampf(v, -TP(4), TP(4));
        if (geno_into_wall(fp, fp->gr_vel)) {
            fp->gr_vel = -fp->gr_vel;
            geno_turn(fp);
        }
        ftCommon_SetSelfMovementFromGroundedMovement(gobj);
        return;
    }
    /* air kinetics (0x65), gravity hook first: an accepted tap lifts by P10, rise capped at P12 */
    if (st->move_i[1] >= (s32) TP(11) && st->move_i[2]) {
        fp->self_vel.y += TP(10);
        st->move_i[2] = 0;
        st->move_i[1] = 0;
        st->move_i[3]++;
        Geno_Event(22, fp->kind, fp->player_id, st->move_i[3], (int) (fp->self_vel.y * 100.0f));
    }
    if (fp->self_vel.y > TP(12)) {
        fp->self_vel.y = TP(12);
    }
    /* gravity P8 (negative = down) to a fall speed of P9 */
    fp->self_vel.y += TP(8);
    if (fp->self_vel.y < -TP(9)) {
        fp->self_vel.y = -TP(9);
    }
    /* drift: accel P5 x stick (+ the air drift base), stable P6 x |stick|, brake P7 above it */
    {
        f32 acc = sx * TP(5);
        f32 cap = geno_param(st, GENO_P_TORNADO_MAX_SPEED);
        if (sx != 0.0f) {
            acc += sx > 0.0f ? fp->co_attrs.aerial_drift_base : -fp->co_attrs.aerial_drift_base;
        }
        fp->self_vel.x = geno_accel_brake(fp->self_vel.x, acc, geno_absf(sx) * TP(6), TP(7));
        fp->self_vel.x = geno_clampf(fp->self_vel.x, -cap, cap);
    }
    if (geno_into_wall(fp, fp->self_vel.x)) {
        fp->self_vel.x = -fp->self_vel.x; /* bounce off walls, turning around */
        geno_turn(fp);
    }
}

/* The profile's n-th state with behaviour `bhv` (declaration order), -1 when it has fewer. */
static int geno_state_nth(GenoState* st, int bhv, int n)
{
    int s, k = 0, c = st->profile >= 0 ? Geno_StateCount(st->profile) : 0;
    for (s = 0; s < c && s < GENO_MAX_STATES; s++) {
        if (Geno_StateBehavior(st->profile, s) == bhv) {
            if (k == n) {
                return s;
            }
            k++;
        }
    }
    return -1;
}

/* Which of the behaviour's states the fighter is in (its order), -1 when none. */
static int geno_state_order(GenoState* st, int bhv, int s)
{
    int i, k = 0;
    for (i = 0; i < s && i < GENO_MAX_STATES; i++) {
        if (Geno_StateBehavior(st->profile, i) == bhv) {
            k++;
        }
    }
    return s >= 0 && Geno_StateBehavior(st->profile, s) == bhv ? k : -1;
}

/* Ground / air partner of a paired behaviour (order 2k = ground, 2k + 1 = air): swap with the frame
 * kept when the situation no longer matches the state. Returns 1 when it swapped. */
static int geno_pair_swap(Fighter_GObj* gobj, Fighter* fp, GenoState* st, int bhv)
{
    int s = geno_cur_state(fp, st);
    int k = geno_state_order(st, bhv, s);
    int want_air = fp->ground_or_air == GA_Air;
    int t;
    if (k < 0 || (k & 1) == want_air) {
        return 0;
    }
    t = geno_state_nth(st, bhv, (k & ~1) | want_air);
    if (t < 0 || t == s) {
        return 0;
    }
    return geno_do_change(gobj, fp, st, GENO_TARGET(GENO_TGT_GENO, t) | GENO_TGT_KEEP_FRAME);
}

/* ---- Drill Rush -------------------------------------------------------------------------------- *
 *
 * v3: rebuilt from Brawl's code (ft_metaknight SpecialSRush C710 / C820, SpecialSEnd C9B8 / CB48,
 * sora kinetic types 0x66-0x68 and the drill angle hook fn_27_359CE4; docs/geno.md 17):
 *   start  (22f) ground: vx x w00, ground brake; air: vx x w00, vy = w01, gravity w02. Ground / air
 *          versions swap with the frame kept (no second x w00 / vy on a swap). No ledge grab.
 *   rush   (45f, fixed: the clip's end) pitch += stick Y x w03 a frame (up climbs for either facing),
 *          NO limit (the clip's 45 frames bound it to 135 deg); velocity = the clip's TransN delta
 *          rotated by the pitch; no gravity. Hits, shields and walls do NOT end it (Brawl's status
 *          has no hit / wall checks: it drills through, rehit 6). Ledge grab from the script's PUT
 *          LEDGE (Brawl: Allow Ledgegrab 1 at frame 20). Along the floor it slides (pitch <= 0).
 *   end    (30f) at the floor -> the ground end (Wait at the end, off an edge -> Fall); in the air
 *          -> the air end: velocity (-1 x facing, +2.1) (Brawl's Set Momentum), FallSpecial at the
 *          end, LandingFallSpecial on landing. The pitch eases x (1 - 2 / w04) a frame.
 * `bounce` (1 wall | 2 hit | 4 shield -> DrillEnd at once) stays available but is 0 by default: it
 * was a Geno v2 addition Brawl does not have. */

static void geno_drill_hit(Fighter_GObj* gobj)
{
    Fighter* fp = GET_FIGHTER(gobj);
    GenoState* st = geno_state(fp);
    st->move_i[1] = 1;
    if (((s32) geno_param(st, GENO_P_DRILL_BOUNCE) & 2) && st->move_i[2] == 0) {
        st->move_i[2] = 1;
        st->move_i[3] = 1;
    }
}

static void geno_drill_shield(Fighter_GObj* gobj)
{
    Fighter* fp = GET_FIGHTER(gobj);
    GenoState* st = geno_state(fp);
    if ((s32) geno_param(st, GENO_P_DRILL_BOUNCE) & 4) {
        st->move_i[2] = 1;
        st->move_i[3] = 3;
    }
}

/* Drill start (Brawl 0x113, kinetic 0x66 ground / 0x67 air). */
static void geno_drill_start_enter(Fighter_GObj* gobj, Fighter* fp, GenoState* st, int s)
{
    if (st->enter_keep) {
        return; /* the other situation's version of the same start: speeds untouched (Brawl) */
    }
    fp->self_vel.x *= DP(0);
    fp->gr_vel *= DP(0);
    if (fp->ground_or_air == GA_Air) {
        fp->self_vel.y = DP(1);
    }
    fp->fall_fast = 0;
    st->move_f[0] = 0.0f;
    st->move_i[1] = 0;
}

static void geno_drill_start_phys(Fighter_GObj* gobj)
{
    Fighter* fp = GET_FIGHTER(gobj);
    GenoState* st = geno_state(fp);
    if (fp->ground_or_air == GA_Ground) {
        ft_80084F3C(gobj);
        return;
    }
    fp->self_vel.y += DP(2);
    if (fp->self_vel.y < -fp->co_attrs.terminal_velocity) {
        fp->self_vel.y = -fp->co_attrs.terminal_velocity;
    }
    ftCommon_CalcSelfAccel_DeaccelAir(fp);
}

/* ground and air in one start; the ground / air versions swap with the frame kept */
static void geno_drill_start_coll(Fighter_GObj* gobj)
{
    Fighter* fp = GET_FIGHTER(gobj);
    geno_coll_both(gobj);
    geno_pair_swap(gobj, fp, geno_state(fp), GENO_BHV_DRILL_START);
}

static void geno_drill_enter(Fighter_GObj* gobj, Fighter* fp, GenoState* st, int s)
{
    st->move_f[0] = 0.0f;
    st->move_f[1] = geno_param(st, GENO_P_DRILL_SPEED);
    st->move_i[0] = 0;
    st->move_i[1] = 0;
    st->move_i[2] = 0;
    st->move_i[3] = 0;
    fp->fall_fast = 0;
    fp->deal_dmg_cb = geno_drill_hit;
    fp->shield_hit_cb = geno_drill_shield;
}

/* The end that matches the situation: the profile's geno.drill.end states in order ground, air
 * (Brawl's rush exit attaches MK to the floor when he is on it; SpecialSEnd then picks by If On
 * Ground). A profile with one end state uses it for both. */
static int geno_drill_end_state(Fighter* fp, GenoState* st)
{
    int e = geno_state_nth(st, GENO_BHV_DRILL_END, fp->ground_or_air == GA_Air ? 1 : 0);
    return e >= 0 ? e : geno_state_with(st, GENO_BHV_DRILL_END);
}

/* Bounce (only with the optional `bounce` mask): straight to the end (its pop sends MK back). */
static void geno_drill_bounce(Fighter_GObj* gobj, Fighter* fp, GenoState* st, int reason)
{
    int e = geno_drill_end_state(fp, st);
    Geno_Event(19, fp->kind, fp->player_id, reason, st->move_i[0]);
    st->move_i[3] = reason;
    if (e >= 0) {
        geno_do_change(gobj, fp, st, GENO_TARGET(GENO_TGT_GENO, e));
    } else {
        geno_helpless(gobj, fp, st);
    }
}

static void geno_drill_anim(Fighter_GObj* gobj)
{
    Fighter* fp = GET_FIGHTER(gobj);
    GenoState* st = geno_state(fp);
    int s = geno_cur_state(fp, st);
    st->move_i[0]++;
    if (st->move_i[2] != 0) {
        geno_drill_bounce(gobj, fp, st, st->move_i[3]);
        return;
    }
    if (geno_anim_ended(gobj) && s >= 0) {
        u32 t = (u32) Geno_StateNext(st->profile, s);
        int e = geno_drill_end_state(fp, st);
        if (t != GENO_NONE_T) {
            geno_go(gobj, fp, st, t);
        } else if (e >= 0) {
            geno_do_change(gobj, fp, st, GENO_TARGET(GENO_TGT_GENO, e));
        } else {
            geno_helpless(gobj, fp, st);
        }
    }
}

/* SpecialSRush: the stick's Y turns the travel by w03 degrees a frame (up climbs for either
 * facing), no limit unless drill.angle_max; the speed is the clip's root motion (TransN), or
 * drill.speed when the row has none. */
static void geno_drill_phys(Fighter_GObj* gobj)
{
    Fighter* fp = GET_FIGHTER(gobj);
    GenoState* st = geno_state(fp);
    f32 sy = geno_stick_dz(fp->input.lstick[0].y);
    f32 lim = geno_param(st, GENO_P_DRILL_ANGLE_MAX);
    f32 a = st->move_f[0] + sy * DP(3);
    f32 spd = st->move_f[1];
    f32 lr = fp->facing_dir < 0.0f ? -1.0f : 1.0f;
    if (lim > 0.0f) {
        a = geno_clampf(a, -lim, lim);
    }
    st->move_f[0] = a;
    if (fp->x594_b0) {
        spd = fp->x6A4_transNOffset.z;
    }
    fp->self_vel.x = lr * spd * cosf(a * GENO_DEG2RAD);
    fp->self_vel.y = spd * sinf(a * GENO_DEG2RAD);
    if (fp->ground_or_air == GA_Ground) {
        if (fp->self_vel.y > 0.0f) {
            ftCommon_8007D5D4(fp); /* pitched up: off the floor */
        } else {
            fp->gr_vel = fp->self_vel.x; /* into / along the floor: it slides */
            fp->self_vel.y = 0.0f;
        }
    }
}

/* ground and air; in the air the script's PUT LEDGE opens the ledge grab (Brawl: frame 20) */
static void geno_drill_coll(Fighter_GObj* gobj)
{
    Fighter* fp = GET_FIGHTER(gobj);
    GenoState* st = geno_state(fp);
    if (fp->ground_or_air == GA_Air && st->ledge > 0) {
        geno_coll_air_ledge(gobj, st->ledge == 2, geno_become_ground);
    } else {
        geno_coll_both(gobj);
    }
    if (((s32) geno_param(st, GENO_P_DRILL_BOUNCE) & 1) &&
        geno_into_wall(fp, fp->facing_dir) && st->move_i[2] == 0)
    {
        st->move_i[2] = 1;
        st->move_i[3] = 2;
    }
}

/* SpecialSEnd: in the air MK pops back and up (Brawl's Set Momentum -1 / +2.1); on the ground he
 * stops. The pitch eases back to level (x (1 - 2/w04) a frame). */
static void geno_drill_end_enter(Fighter_GObj* gobj, Fighter* fp, GenoState* st, int s)
{
    f32 lr = fp->facing_dir < 0.0f ? -1.0f : 1.0f;
    if (st->enter_keep) {
        return;
    }
    if (fp->ground_or_air == GA_Air) {
        fp->self_vel.x = -lr * geno_param(st, GENO_P_DRILL_POP_VX);
        fp->self_vel.y = geno_param(st, GENO_P_DRILL_POP_VY);
    } else {
        fp->gr_vel = 0.0f;
    }
}

/* At the end: the state's "next", else FallSpecial in the air (Brawl: status group 10002) with
 * end_helpless, else Wait / Fall. */
static void geno_drill_end_anim(Fighter_GObj* gobj)
{
    Fighter* fp = GET_FIGHTER(gobj);
    GenoState* st = geno_state(fp);
    int s = geno_cur_state(fp, st);
    if (!geno_anim_ended(gobj) || s < 0) {
        return;
    }
    if ((u32) Geno_StateNext(st->profile, s) != GENO_NONE_T) {
        geno_go(gobj, fp, st, geno_next_of(st, s));
    } else if (geno_param(st, GENO_P_DRILL_END_HELPLESS) != 0.0f) {
        geno_helpless(gobj, fp, st);
    } else {
        geno_go(gobj, fp, st, GENO_AUTO);
    }
}

static void geno_drill_end_phys(Fighter_GObj* gobj)
{
    Fighter* fp = GET_FIGHTER(gobj);
    GenoState* st = geno_state(fp);
    f32 n = DP(4);
    if (n > 0.0f) {
        st->move_f[0] -= 2.0f * st->move_f[0] / n;
    } else {
        st->move_f[0] = 0.0f;
    }
    if (fp->ground_or_air == GA_Ground) {
        ft_80084F3C(gobj);
        return;
    }
    ftCommon_Fall(fp, fp->co_attrs.gravity, fp->co_attrs.terminal_velocity);
    ftCommon_CalcSelfAccel_DeaccelAir(fp);
}

/* ---- Dimensional Cape (v3) ---------------------------------------------------------------------
 *
 * Brawl (IR actions 0x115 / 0x11B / 0x11C, subactions 472-481, MK kinetic types 0x69 / 0x6A in
 * sora_melee fn_27_359014; docs/geno.md 17): the start keeps half the momentum (w00 x, w01 y) with no
 * gravity; at frame 12 MK vanishes and the stick steers him on both axes (accel w02 toward
 * stick x w03 on x, w04 / w05 on y, no gravity); at frame 26 Special or Attack HELD picks the
 * reappear with the slash, else the plain reappear. The slash's variant is the stick x relative to
 * the facing at that moment: neutral -> N, back -> Brawl's "F" clips, forward -> "B" clips (Brawl's
 * names). The reappear states are root-motion states (geno.cape.attack / geno.cape.end = the
 * geno.anim_motion callbacks) found by order in the profile (N, N air, F, F air, B, B air; end,
 * end air). A ground state that leaves the ground (and an air one that lands) swaps to its partner
 * with the frame kept, as Brawl's ground / air loops do.
 *
 * Per-move state: move_i[0] frames since the start (not reset by a keep-frame swap),
 *                 move_i[1] the reappear picked (event 27)
 */

#define CP(i) geno_param(st, GENO_P_CAPE_W0 + (i))

static void geno_cape_enter(Fighter_GObj* gobj, Fighter* fp, GenoState* st, int s)
{
    if (st->enter_keep) {
        return; /* a ground / air swap: the move goes on */
    }
    /* kinetic 0x69: the momentum is kept at w00 (x) / w01 (y), gravity off */
    fp->self_vel.x *= CP(0);
    fp->self_vel.y *= CP(1);
    fp->gr_vel *= CP(0);
    fp->fall_fast = 0;
    st->move_i[0] = 0;
    st->move_i[1] = -1;
}

/* Kinetic 0x6A, one axis: accelerate by `acc` toward stick x max, never past +-max. */
static f32 geno_cape_steer(f32 v, f32 stick, f32 acc, f32 max)
{
    f32 want = stick * max;
    f32 d = geno_clampf(want - v, -acc, acc);
    return geno_clampf(v + d, -max, max);
}

static void geno_cape_phys(Fighter_GObj* gobj)
{
    Fighter* fp = GET_FIGHTER(gobj);
    GenoState* st = geno_state(fp);
    s32 f = st->move_i[0]++;
    f32 sx = geno_stick_dz(fp->input.lstick[0].x), sy = geno_stick_dz(fp->input.lstick[0].y);
    if (f == (s32) geno_param(st, GENO_P_CAPE_STEER_FRAME)) {
        Geno_Event(26, fp->kind, fp->player_id, f, (int) (sx * 100.0f));
    }
    if (f < (s32) geno_param(st, GENO_P_CAPE_STEER_FRAME)) {
        /* the kept momentum, no gravity, no friction */
        if (fp->ground_or_air == GA_Ground) {
            ftCommon_SetSelfMovementFromGroundedMovement(gobj);
        }
        return;
    }
    if (fp->ground_or_air == GA_Ground) {
        if (sy > 0.0f) {
            ftCommon_8007D5D4(fp); /* steered up: off the ground (the air partner takes over) */
            fp->self_vel.x = fp->gr_vel;
            fp->self_vel.y = 0.0f;
        } else {
            fp->gr_vel = geno_cape_steer(fp->gr_vel, sx, CP(2), CP(3));
            ftCommon_SetSelfMovementFromGroundedMovement(gobj);
            return;
        }
    }
    fp->self_vel.x = geno_cape_steer(fp->self_vel.x, sx, CP(2), CP(3));
    fp->self_vel.y = geno_cape_steer(fp->self_vel.y, sy, CP(4), CP(5));
}

/* coll: ground and air in one move (off an edge / landing keeps the state), no ledge grab; the
 * partner state takes over with the frame kept. */
static void geno_cape_coll(Fighter_GObj* gobj)
{
    Fighter* fp = GET_FIGHTER(gobj);
    geno_coll_both(gobj);
    geno_pair_swap(gobj, fp, geno_state(fp), GENO_BHV_CAPE);
}

/* anim: at decide_frame, Brawl's three Change Actions (Special held / Attack held -> the slash, else
 * the plain reappear). The start clip (20 frames) holds its last frame until then. */
static void geno_cape_anim(Fighter_GObj* gobj)
{
    Fighter* fp = GET_FIGHTER(gobj);
    GenoState* st = geno_state(fp);
    int air = fp->ground_or_air == GA_Air;
    u32 held = geno_buttons(fp, 0);
    int v, t;
    if (st->move_i[0] < (s32) geno_param(st, GENO_P_CAPE_DECIDE_FRAME)) {
        return;
    }
    if (held & (u32) geno_param(st, GENO_P_CAPE_BUTTONS)) {
        f32 fwd = geno_stick_dz(fp->input.lstick[0].x) * (fp->facing_dir < 0.0f ? -1.0f : 1.0f);
        v = geno_absf(fwd) < geno_param(st, GENO_P_CAPE_NEUTRAL_X) ? 1 : fwd < 0.0f ? 2 : 3;
        t = geno_state_nth(st, GENO_BHV_CAPE_ATTACK, 2 * (v - 1) + air);
    } else {
        v = 0;
        t = geno_state_nth(st, GENO_BHV_CAPE_END, air);
    }
    st->move_i[1] = v + (air ? 10 : 0);
    Geno_Event(27, fp->kind, fp->player_id, st->move_i[1], (int) held);
    if (t < 0) {
        geno_helpless(gobj, fp, st);
        return;
    }
    geno_do_change(gobj, fp, st, GENO_TARGET(GENO_TGT_GENO, t));
}

/* The reappears: root motion (geno.anim_motion), ground / air partners swap with the frame kept. */
static void geno_cape_after_coll(Fighter_GObj* gobj)
{
    Fighter* fp = GET_FIGHTER(gobj);
    GenoState* st = geno_state(fp);
    int s = geno_cur_state(fp, st);
    int bhv = s >= 0 ? Geno_StateBehavior(st->profile, s) : GENO_BHV_NONE;
    geno_cb_coll_anim_motion(gobj);
    if (geno_cur_state(fp, st) == s) {
        geno_pair_swap(gobj, fp, st, bhv);
    }
}

int GenoGame_TestPairSwap(Fighter* fp, int bhv)
{
    return geno_pair_swap(fp->gobj, fp, geno_state(fp), bhv);
}

/* iasa "interrupt": the script's Allow Interrupt (Melee's IASA command) opens Wait's / Fall's
 * interrupts, as Melee's own attacks do. */
static void geno_cb_iasa_interrupt(Fighter_GObj* gobj)
{
    Fighter* fp = GET_FIGHTER(gobj);
    if (!fp->allow_interrupt) {
        return;
    }
    if (fp->ground_or_air == GA_Ground) {
        ftCo_Wait_IASA(gobj);
    } else {
        ftCo_Fall_IASA(gobj);
    }
}

#undef CP

#undef TP
#undef DP
