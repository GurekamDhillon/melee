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

/* ---- Drill Rush -------------------------------------------------------------------------------- */

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

/* Drill start (Brawl 0x113, kinetic 0x66 / 0x67): vx *= P0; in the air vy = P1, gravity P2. */
static void geno_drill_start_enter(Fighter_GObj* gobj, Fighter* fp, GenoState* st, int s)
{
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

/* Bounce: straight to DrillEnd (its pop sends the fighter back). */
static void geno_drill_bounce(Fighter_GObj* gobj, Fighter* fp, GenoState* st, int reason)
{
    int e = geno_state_with(st, GENO_BHV_DRILL_END);
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
        geno_go(gobj, fp, st, geno_next_of(st, s));
    }
}

/* SpecialSRush: the stick's Y turns the travel by P3 degrees a frame (up climbs for either
 * facing); the speed is the clip's root motion (TransN), or drill.speed when it has none. */
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
    if (fp->ground_or_air == GA_Ground && a < 0.0f) {
        a = 0.0f; /* on the ground it can only turn up (and leaves the ground doing so) */
    }
    st->move_f[0] = a;
    if (fp->x594_b0 && geno_absf(fp->x6A4_transNOffset.z) > 0.001f) {
        spd = geno_absf(fp->x6A4_transNOffset.z);
    }
    fp->self_vel.x = lr * spd * cosf(a * GENO_DEG2RAD);
    fp->self_vel.y = spd * sinf(a * GENO_DEG2RAD);
    if (fp->ground_or_air == GA_Ground) {
        if (a > 0.0f) {
            ftCommon_8007D5D4(fp);
        } else {
            fp->gr_vel = fp->self_vel.x;
            fp->self_vel.y = 0.0f;
        }
    }
}

static void geno_drill_coll(Fighter_GObj* gobj)
{
    Fighter* fp = GET_FIGHTER(gobj);
    GenoState* st = geno_state(fp);
    geno_coll_both(gobj);
    if (((s32) geno_param(st, GENO_P_DRILL_BOUNCE) & 1) &&
        geno_into_wall(fp, fp->facing_dir) && st->move_i[2] == 0)
    {
        st->move_i[2] = 1;
        st->move_i[3] = 2;
    }
}

/* SpecialSEnd: the pitch eases back to level (x (1 - 2/P4) a frame); in the air the fighter pops
 * back and up (Brawl's air-end momentum -1 / +2.1). Ends helpless unless the drill hit. */
static void geno_drill_end_enter(Fighter_GObj* gobj, Fighter* fp, GenoState* st, int s)
{
    f32 lr = fp->facing_dir < 0.0f ? -1.0f : 1.0f;
    if (fp->ground_or_air == GA_Air) {
        fp->self_vel.x = -lr * geno_param(st, GENO_P_DRILL_POP_VX);
        fp->self_vel.y = geno_param(st, GENO_P_DRILL_POP_VY);
    } else {
        fp->gr_vel = 0.0f;
    }
}

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
    } else if (geno_param(st, GENO_P_DRILL_END_HELPLESS) != 0.0f && st->move_i[1] == 0) {
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

#undef TP
#undef DP
