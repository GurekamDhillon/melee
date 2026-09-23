/* geno_game_glide.inc - the glide behaviours (GlideStart / Glide / GlideAttack / GlideLanding /
 * GlideEnd), part of geno_game_v2.inc. Brawl's common glide (sora_melee
 * ftStatusUniqProcessGlide, shared by Meta Knight, Pit and Charizard; disassembled for Geno v2,
 * docs/geno.md 16.4) as a character-level Geno state. Melee's gravity, ledges, landing and damage
 * rules stay as they are; only the glide's own flight is Brawl's.
 *
 * Brawl's Misc Glide block P[0..21] is the "glide" parameter family (w00..w21, with names):
 *   P0 angle_max 80   P1 angle_min -70   P2 start_vy 0.75   P3 start_gravity 1.0
 *   P4 start_vx 1.0   P5 speed 1.7       P6 speed_accel 0.04 P7 max_speed 2.2
 *   P8 stall_speed 0.7 P9 sink_accel 0.03 P10 max_sink 0.6  P11 recover_angle 15
 *   P12 dive_angle -25 P13 dive_bonus 0.03 P14 (unused)     P15 deadzone 0.25
 *   P16 pitch_up 0.55 P17 pitch_down 0.75 P18 max_pitch_rate 7 P19 stall_pitch 1.0
 *   P20 wing node 44 (Brawl's partial wing animation: not used in Melee)   P21 (unused)
 *
 * Per-move state (GenoState, kept across the five states, snapshotted; Brawl's RA vars):
 *   move_f[0] angle (deg, + = nose up; F4)   move_f[1] speed (F5)   move_f[2] sink (F6)
 *   move_f[3] pitch rate (F7)                move_i[0] frames gliding
 *   move_i[1] end reason (event 21)          move_i[2] stall (Bit16)  move_i[3] wall (Bit19)
 */

#define GL_ANGLE move_f[0]
#define GL_SPEED move_f[1]
#define GL_SINK move_f[2]
#define GL_RATE move_f[3]
#define GL_TIME move_i[0]
#define GL_REASON move_i[1]
#define GL_STALL move_i[2]
#define GL_WALL move_i[3]

#define GLP(i) geno_param(st, GENO_P_GLIDE_W0 + (i))

/* ---- entry: jump held in an air jump ---------------------------------------------------------- */

static int geno_in_air_jump(Fighter* fp, GenoState* st)
{
    int m = (int) fp->motion_id;
    if (fp->ground_or_air != GA_Air) {
        return 0;
    }
    if (m == ftCo_MS_JumpAerialF || m == ftCo_MS_JumpAerialB) {
        return 1;
    }
    if (fp->x2D0 != NULL && ftCo_800D72A0(fp)) {
        return 1; /* Kirby / Jigglypuff multi-jump states */
    }
    if (geno_param(st, GENO_P_GLIDE_FROM_JUMP) != 0.0f &&
        (m == ftCo_MS_JumpF || m == ftCo_MS_JumpB))
    {
        return 1;
    }
    return 0;
}

/* Brawl: jump held for "Glide Frame Window" (MK 16) frames during a midair jump. */
static int geno_glide_entry(Fighter_GObj* gobj, Fighter* fp, GenoState* st)
{
    int start, need;
    if (geno_param(st, GENO_P_GLIDE_ENTRY) == 0.0f ||
        (start = geno_state_with(st, GENO_BHV_GLIDE_START)) < 0)
    {
        return 0;
    }
    if (!geno_in_air_jump(fp, st)) {
        st->hold_motion = -1;
        st->hold_frames = -1;
        return 0;
    }
    if (st->action_time <= 1 || st->hold_motion != (s32) fp->motion_id) {
        /* a new air jump (a multi-jump chain re-enters the same motion: action_time says so) */
        st->hold_motion = (s32) fp->motion_id;
        st->hold_frames = 0;
    }
    if (st->hold_frames < 0) {
        return 0; /* released during this jump */
    }
    if (!geno_jump_held(fp)) {
        st->hold_frames = -1;
        return 0;
    }
    st->hold_frames++;
    need = (int) geno_param(st, GENO_P_GLIDE_HOLD);
    if (st->hold_frames < (need > 0 ? need : 1)) {
        return 0;
    }
    Geno_Event(17, fp->kind, fp->player_id, st->hold_frames, (int) fp->motion_id);
    st->hold_frames = -1;
    return geno_do_change(gobj, fp, st, GENO_TARGET(GENO_TGT_GENO, start));
}

/* ---- GlideStart (Brawl 0x84): the jump's momentum, a vertical boost, gravity -------------------- */

static void geno_glide_start_enter(Fighter_GObj* gobj, Fighter* fp, GenoState* st, int s)
{
    if (fp->ground_or_air == GA_Ground) {
        ftCommon_8007D5D4(fp);
    }
    /* Brawl resets the gravity / stop energies here (ftKineticEnergyGravity case 7,
       ftKineticEnergyStop case 19): vy += P2, vx *= P4 */
    fp->self_vel.y += GLP(2);
    fp->self_vel.x *= GLP(4);
    fp->fall_fast = 0;
    st->GL_ANGLE = 0.0f;
    st->GL_SPEED = 0.0f;
    st->GL_SINK = 0.0f;
    st->GL_RATE = 0.0f;
    st->GL_TIME = 0;
    st->GL_REASON = 0;
    st->GL_STALL = 0;
    st->GL_WALL = 0;
}

static void geno_glide_start_anim(Fighter_GObj* gobj)
{
    Fighter* fp = GET_FIGHTER(gobj);
    GenoState* st = geno_state(fp);
    int s = geno_cur_state(fp, st);
    if (geno_anim_ended(gobj) && s >= 0) {
        geno_go(gobj, fp, st, geno_next_of(st, s));
    }
}

/* Brawl's "accelBrake" (fn_27_896EC) for a velocity component: add `a`; above `stable` in the
 * direction of travel, brake by `brake` instead, never undershooting `stable`. */
static f32 geno_accel_brake(f32 v, f32 a, f32 stable, f32 brake)
{
    f32 n = v + a;
    if (stable >= 0.0f && brake > 0.0f) {
        if (n > stable && v > 0.0f) {
            f32 d = a < -brake ? a : -brake;
            n = v + d;
            if (n < stable && v >= stable) {
                n = stable;
            }
            return n;
        }
        if (n < -stable && v < 0.0f) {
            f32 d = a > brake ? a : brake;
            n = v + d;
            if (n > -stable && v <= -stable) {
                n = -stable;
            }
            return n;
        }
    }
    return n;
}

static void geno_glide_start_phys(Fighter_GObj* gobj)
{
    Fighter* fp = GET_FIGHTER(gobj);
    GenoState* st = geno_state(fp);
    ftCo_DatAttrs* ca = &fp->co_attrs;
    /* gravity x P3 to the terminal velocity; vx braked toward 0 by the air friction, capped */
    ftCommon_Fall(fp, ca->gravity * GLP(3), ca->terminal_velocity);
    fp->self_vel.x = geno_accel_brake(fp->self_vel.x, 0.0f, 0.0f, ca->aerial_friction);
    fp->self_vel.x = geno_clampf(fp->self_vel.x, -2.5f, 2.5f);
}

/* ---- Glide (Brawl 0x85) --------------------------------------------------------------------------- */

static void geno_glide_enter(Fighter_GObj* gobj, Fighter* fp, GenoState* st, int s)
{
    /* initStatus: the glide starts at its own speed (the momentum is not kept), level, and the
       vertical speed it enters with becomes the starting sink */
    st->GL_SPEED = GLP(5);
    st->GL_SINK = -fp->self_vel.y;
    st->GL_ANGLE = 0.0f;
    st->GL_RATE = 0.0f;
    st->GL_STALL = 0;
    st->GL_WALL = 0;
    fp->self_vel.x = GLP(5) * fp->facing_dir;
    fp->self_vel.y = 0.0f;
    fp->fall_fast = 0;
}

/* The Glide's per-frame flight: Brawl's execStatus, decoded. */
static void geno_glide_step(Fighter* fp, GenoState* st)
{
    f32 lr = fp->facing_dir < 0.0f ? -1.0f : 1.0f;
    f32 sx = fp->input.lstick[0].x, sy = fp->input.lstick[0].y;
    f32 ang = st->GL_ANGLE, rate = st->GL_RATE;
    f32 mag = sqrtf(sx * sx + sy * sy);
    f32 dz = GLP(15);
    f32 s, sink, vx, vy, len, rad;
    if (mag > dz) {
        /* the stick's angle in the facing frame: up / back pitch the nose up, forward / down
           pitch it down (Brawl: [45, 180] or (-180, -135) up, [-135, 45) down) */
        f32 a = atan2f(sy, sx) * 57.29578f;
        f32 acc;
        if (lr <= 0.0f) {
            a = (a > 0.0f ? 180.0f : -180.0f) - a;
        }
        acc = (a >= 45.0f || a < -135.0f) ? GLP(16) : -GLP(17);
        acc *= (mag - dz) / (1.0f - dz);
        if (rate * acc < 0.0f) {
            rate = 0.0f; /* instant reversal */
        }
        rate = geno_clampf(rate + acc, -GLP(18), GLP(18));
        st->GL_RATE = rate;
        ang += rate;
    } else if (st->GL_STALL) {
        rate = (rate > 0.0f ? rate : 0.0f) + GLP(19);
        rate = geno_clampf(rate, -GLP(18), GLP(18));
        st->GL_RATE = rate;
        ang += rate;
    }
    ang = geno_clampf(ang, GLP(1), GLP(0));
    st->GL_ANGLE = ang;
    /* speed: climbing loses, diving gains (P6 per frame at +-90 degrees) */
    s = st->GL_SPEED - GLP(6) * (ang / 90.0f);
    if (st->GL_WALL) {
        s -= 0.01f;
    }
    if (s < 0.0f) {
        s = 0.0f;
    }
    if (ang < GLP(12) && GLP(12) != GLP(1)) {
        /* a steep dive accelerates more, up to P13 at the lower limit */
        s += GLP(13) * (GLP(12) - ang) / (GLP(12) - GLP(1));
    }
    rad = ang * GENO_DEG2RAD;
    vx = lr * s * cosf(rad);
    vy = s * sinf(rad);
    /* the sink grows by P9 a frame to P10 */
    sink = geno_clampf(st->GL_SINK + GLP(9), -GLP(10), GLP(10));
    st->GL_SINK = sink;
    vy -= sink;
    len = sqrtf(vx * vx + vy * vy);
    if (len > GLP(7) && len > 0.0f) {
        vx *= GLP(7) / len;
        vy *= GLP(7) / len;
    }
    if (len < GLP(8) || s <= 0.0f) {
        st->GL_STALL = 1; /* stalled: Brawl's Glide action ends on this bit */
        st->GL_RATE = 0.0f;
    }
    fp->self_vel.x = vx;
    fp->self_vel.y = vy;
    st->GL_SPEED = s;
}

/* Leave the glide: 1 end button, 2 stall, 3 attack, 4 timeout. */
static void geno_glide_end(Fighter_GObj* gobj, Fighter* fp, GenoState* st, int reason)
{
    int s = geno_cur_state(fp, st);
    st->GL_REASON = reason;
    Geno_Event(21, fp->kind, fp->player_id, reason, st->GL_TIME);
    if (reason == 3) {
        int k = geno_state_with(st, GENO_BHV_GLIDE_ATTACK);
        if (k >= 0) {
            geno_do_change(gobj, fp, st, GENO_TARGET(GENO_TGT_GENO, k));
            return;
        }
    }
    geno_go(gobj, fp, st, s >= 0 ? geno_next_of(st, s) : GENO_AUTO);
}

static void geno_glide_anim(Fighter_GObj* gobj)
{
    Fighter* fp = GET_FIGHTER(gobj);
    GenoState* st = geno_state(fp);
    f32 limit = geno_param(st, GENO_P_GLIDE_MAX_FRAMES);
    st->GL_TIME++;
    if (st->GL_STALL) {
        geno_glide_end(gobj, fp, st, 2);
        return;
    }
    if (limit > 0.0f && (f32) st->GL_TIME >= limit) {
        geno_glide_end(gobj, fp, st, 4);
        return;
    }
    if (geno_param(st, GENO_P_GLIDE_POSE) == 0.0f && geno_anim_ended(gobj)) {
        geno_cb_anim_loop(gobj);
    }
}

static void geno_glide_iasa(Fighter_GObj* gobj)
{
    Fighter* fp = GET_FIGHTER(gobj);
    GenoState* st = geno_state(fp);
    u32 pressed = geno_buttons(fp, 1);
    u32 ends = (u32) geno_param(st, GENO_P_GLIDE_END_BUTTONS);
    if (pressed & GENO_BTN_ATTACK) {
        geno_glide_end(gobj, fp, st, 3);
    } else if (pressed & ends) {
        geno_glide_end(gobj, fp, st, 1);
    }
}

static void geno_glide_phys(Fighter_GObj* gobj)
{
    Fighter* fp = GET_FIGHTER(gobj);
    GenoState* st = geno_state(fp);
    f32 pose = geno_param(st, GENO_P_GLIDE_POSE);
    geno_glide_step(fp, st);
    if (pose != 0.0f && !geno_test_capture) {
        /* Brawl poses the glide by angle (Glide_Direction, rate 0, frame = 90 - angle): scrub the
           animation to that frame (it advances by the rate before the next anim callback) */
        f32 want = pose - st->GL_ANGLE;
        ftAnim_SetAnimRate(gobj, want - fp->cur_anim_frame);
    }
}

/* coll: Melee's air collision with ledge grab; a wall touch slows the glide (Brawl's Bit19). */
static void geno_glide_coll(Fighter_GObj* gobj)
{
    Fighter* fp = GET_FIGHTER(gobj);
    GenoState* st = geno_state(fp);
    geno_cb_coll_air(gobj);
    st->GL_WALL = (fp->coll_data.env_flags & Collide_WallMask) != 0;
}

/* ---- GlideAttack / GlideLanding / GlideEnd -------------------------------------------------------- */

static void geno_glide_attack_enter(Fighter_GObj* gobj, Fighter* fp, GenoState* st, int s)
{
}

/* GlideAttack and GlideEnd keep the glide's momentum under Melee's gravity and air friction. */
static void geno_glide_attack_phys(Fighter_GObj* gobj)
{
    Fighter* fp = GET_FIGHTER(gobj);
    ftCommon_Fall(fp, fp->co_attrs.gravity, fp->co_attrs.terminal_velocity);
    ftCommon_CalcSelfAccel_DeaccelAir(fp);
}

static void geno_glide_landing_enter(Fighter_GObj* gobj, Fighter* fp, GenoState* st, int s)
{
    fp->gr_vel = fp->self_vel.x;
}

static void geno_glide_end_enter(Fighter_GObj* gobj, Fighter* fp, GenoState* st, int s)
{
    if (geno_param(st, GENO_P_GLIDE_END_HELPLESS) != 0.0f) {
        ftCommon_UseAllJumps(fp);
    }
}

static void geno_glide_end_phys(Fighter_GObj* gobj)
{
    geno_glide_attack_phys(gobj);
}

#undef GLP
