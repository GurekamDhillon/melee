/* geno_game_v2.inc - Geno v2: Geno action states, native behaviours (glide, Mach Tornado, Drill
 * Rush, generic air / ground states) and specials bound to states. Included at the end of
 * geno_game.c (one translation unit: it shares that file's statics). Design and data format:
 * docs/geno.md section 16.
 *
 * HOW A GENO STATE WORKS. Profile state n is Melee motion GENO_MOTION_BASE + n. Its MotionState row
 * (below, Geno_Rows) is built at spawn from geno.json: a "like" motion's row gives the flags, move
 * id and camera callback; "subaction" picks the animation + script from the fighter's own files;
 * the four callbacks come from the state's behaviour (or are named one by one). fighter.c asks
 * Geno_MotionRow for the row when the motion id is in Geno's range, so Fighter_ChangeMotionState
 * enters the state exactly like any other: the script runs, hitboxes and hurtboxes work, damage,
 * grabs and death leave it through Melee's own code, and the collision callbacks use Melee's own
 * landing / ledge-grab helpers.
 *
 * ROLLBACK. The rows and parameter tables are game globals written only with values derived from
 * the registry (the same every time: idempotent, snapshotted, restored to the same bytes). All
 * per-fighter behaviour state is in GenoState (move_f / move_i / hold_*), plus the Fighter struct
 * itself (velocities, callbacks). No host state, no host time, no randomness.
 */

#include <melee/ft/ft_081B.h>
#include <melee/ft/ft_084E.h>
#include <melee/ft/ftchangeparam.h>
#include <melee/ft/kinds/ftCommon/ftCo_Attack100.h>
#include <melee/mp/forward.h>
#include <melee/mp/mpcoll.h>
#include <melee/ft/ft_0819.h>
#include <melee/ft/ftcliffcommon.h>

/* ---- game globals: rows and parameters per profile -------------------------------------------- */

MotionState Geno_Rows[GENO_MAX_PROFILES][GENO_MAX_STATES];
f32 Geno_Params[GENO_MAX_PROFILES][GENO_PARAMS];
s32 Geno_RowCount[GENO_MAX_PROFILES];

/* Internal target kinds (never in a script): the profile's first state with behaviour `id`, and
 * "Wait on the ground / Fall in the air". */
#define GENO_TGT_BHV 3u
#define GENO_AUTO GENO_TGT_AUTO
#define GENO_NONE_T 0xFFFFFFFFu

/* ---- parameters -------------------------------------------------------------------------------- */

typedef struct {
    const char* family;
    const char* name;
    int id;
    f32 def;
} GenoParamDef;

/* Named parameters (the "wNN" Brawl words are accepted too: see GenoGame_ParamFind). Defaults are
 * Meta Knight's Brawl values; names and meanings of the glide / tornado / drill words: docs/geno.md
 * section 16 and geno_v2_encodings.md. */
static const GenoParamDef geno_params[] = {
    /* glide: Brawl's Misc Glide block, read by sora_melee ftStatusUniqProcessGlide (initStatus /
       execStatus, and the Glide_Start energy resets). Names from the disassembly (16.4). */
    { "glide", "angle_max", GENO_P_GLIDE_W0 + 0, 80.0f },
    { "glide", "angle_min", GENO_P_GLIDE_W0 + 1, -70.0f },
    { "glide", "start_vy", GENO_P_GLIDE_W0 + 2, 0.75f },
    { "glide", "start_gravity", GENO_P_GLIDE_W0 + 3, 1.0f },
    { "glide", "start_vx", GENO_P_GLIDE_W0 + 4, 1.0f },
    { "glide", "speed", GENO_P_GLIDE_W0 + 5, 1.7f },
    { "glide", "speed_accel", GENO_P_GLIDE_W0 + 6, 0.04f },
    { "glide", "max_speed", GENO_P_GLIDE_W0 + 7, 2.2f },
    { "glide", "stall_speed", GENO_P_GLIDE_W0 + 8, 0.7f },
    { "glide", "sink_accel", GENO_P_GLIDE_W0 + 9, 0.03f },
    { "glide", "max_sink", GENO_P_GLIDE_W0 + 10, 0.6f },
    { "glide", "recover_angle", GENO_P_GLIDE_W0 + 11, 15.0f },
    { "glide", "dive_angle", GENO_P_GLIDE_W0 + 12, -25.0f },
    { "glide", "dive_bonus", GENO_P_GLIDE_W0 + 13, 0.03f },
    { "glide", "w14", GENO_P_GLIDE_W0 + 14, 0.15f }, /* not read by Brawl's code */
    { "glide", "deadzone", GENO_P_GLIDE_W0 + 15, 0.25f },
    { "glide", "pitch_up", GENO_P_GLIDE_W0 + 16, 0.55f },
    { "glide", "pitch_down", GENO_P_GLIDE_W0 + 17, 0.75f },
    { "glide", "max_pitch_rate", GENO_P_GLIDE_W0 + 18, 7.0f },
    { "glide", "stall_pitch", GENO_P_GLIDE_W0 + 19, 1.0f },
    { "glide", "wing_node", GENO_P_GLIDE_W0 + 20, 44.0f }, /* Brawl's wing partial anim: unused */
    { "glide", "w21", GENO_P_GLIDE_W0 + 21, 0.0f },        /* not read by Brawl's code */
    { "glide", "hold_frames", GENO_P_GLIDE_HOLD, 16.0f },
    { "glide", "from_ground_jump", GENO_P_GLIDE_FROM_JUMP, 0.0f },
    { "glide", "end_helpless", GENO_P_GLIDE_END_HELPLESS, 0.0f },
    { "glide", "landing_lag", GENO_P_GLIDE_LANDING_LAG, 0.0f },
    { "glide", "entry", GENO_P_GLIDE_ENTRY, 1.0f },
    { "glide", "max_frames", GENO_P_GLIDE_MAX_FRAMES, 0.0f },
    { "glide", "pose_center", GENO_P_GLIDE_POSE, 0.0f },
    { "glide", "end_buttons", GENO_P_GLIDE_END_BUTTONS,
      (f32) (GENO_BTN_SHIELD | GENO_BTN_SPECIAL | GENO_BTN_JUMP) },
    /* tornado: MK's paramSpecialN (param ids 4000-4016 / 24000-24001; the int words are w11 and
       w16), read by ftMetaknightStatusUniqProcessSpecialNSpin and MK's kinetic types 0x64/0x65 */
    { "tornado", "entry_vy", GENO_P_TORNADO_W0 + 0, 1.0f },
    { "tornado", "entry_vx_mul", GENO_P_TORNADO_W0 + 1, 0.7f },
    { "tornado", "start_rate", GENO_P_TORNADO_W0 + 2, 80.0f },
    { "tornado", "ground_accel", GENO_P_TORNADO_W0 + 3, 0.12f },
    { "tornado", "ground_speed", GENO_P_TORNADO_W0 + 4, 2.0f },
    { "tornado", "air_accel", GENO_P_TORNADO_W0 + 5, 0.1f },
    { "tornado", "air_speed", GENO_P_TORNADO_W0 + 6, 1.7f },
    { "tornado", "brake", GENO_P_TORNADO_W0 + 7, 0.008f },
    { "tornado", "gravity", GENO_P_TORNADO_W0 + 8, -0.08f },
    { "tornado", "max_fall", GENO_P_TORNADO_W0 + 9, 0.5f },
    { "tornado", "tap_vy", GENO_P_TORNADO_W0 + 10, 1.0f },
    { "tornado", "tap_cooldown", GENO_P_TORNADO_W0 + 11, 10.0f },
    { "tornado", "max_rise", GENO_P_TORNADO_W0 + 12, 1.4f },
    { "tornado", "tap_rate", GENO_P_TORNADO_W0 + 13, 16.0f },
    { "tornado", "max_rate", GENO_P_TORNADO_W0 + 14, 80.0f },
    { "tornado", "rate_decay", GENO_P_TORNADO_W0 + 15, 1.5f },
    { "tornado", "spin_frames", GENO_P_TORNADO_W0 + 16, 70.0f },
    { "tornado", "late_decay", GENO_P_TORNADO_W0 + 17, 2.0f },
    { "tornado", "end_rate", GENO_P_TORNADO_W0 + 18, 10.0f },
    { "tornado", "w19", GENO_P_TORNADO_W0 + 19, 0.0f },
    { "tornado", "max_speed", GENO_P_TORNADO_MAX_SPEED, 2.5f },
    { "tornado", "end_helpless", GENO_P_TORNADO_END_HELPLESS, 1.0f },
    /* drill: MK's paramSpecialS (param ids 4017-4020 / 24002), read by SpecialSRush / SpecialSEnd
       (ft_metaknight.rel) and MK's kinetic types 0x66-0x68 */
    { "drill", "start_vx_mul", GENO_P_DRILL_W0 + 0, 0.5f },
    { "drill", "start_vy", GENO_P_DRILL_W0 + 1, 1.0f },
    { "drill", "start_gravity", GENO_P_DRILL_W0 + 2, -0.08f },
    { "drill", "steer", GENO_P_DRILL_W0 + 3, 3.0f },
    { "drill", "end_frames", GENO_P_DRILL_W0 + 4, 10.0f },
    { "drill", "w05", GENO_P_DRILL_W0 + 5, 0.0f },
    { "drill", "speed", GENO_P_DRILL_SPEED, 2.0f },
    { "drill", "angle_max", GENO_P_DRILL_ANGLE_MAX, 0.0f },
    { "drill", "bounce", GENO_P_DRILL_BOUNCE, 0.0f }, /* v3: Brawl never ends the rush early */
    { "drill", "pop_vx", GENO_P_DRILL_POP_VX, 1.0f },
    { "drill", "pop_vy", GENO_P_DRILL_POP_VY, 2.1f },
    { "drill", "end_helpless", GENO_P_DRILL_END_HELPLESS, 1.0f },
    { "glide", "script_entry_helpless", GENO_P_GLIDE_SCRIPT_HELPLESS, 1.0f },
    /* cape (v3): MK's paramSpecialLw (param ids 4021-4026, read only by MK kinetic types 0x69 /
       0x6A in sora_melee fn_27_359014) + the PSA's frames */
    { "cape", "keep_vx", GENO_P_CAPE_W0 + 0, 0.5f },
    { "cape", "keep_vy", GENO_P_CAPE_W0 + 1, 0.4f },
    { "cape", "steer_accel_x", GENO_P_CAPE_W0 + 2, 0.5f },
    { "cape", "steer_max_x", GENO_P_CAPE_W0 + 3, 2.5f },
    { "cape", "steer_accel_y", GENO_P_CAPE_W0 + 4, 0.5f },
    { "cape", "steer_max_y", GENO_P_CAPE_W0 + 5, 2.5f },
    { "cape", "steer_frame", GENO_P_CAPE_STEER_FRAME, 12.0f },
    { "cape", "decide_frame", GENO_P_CAPE_DECIDE_FRAME, 26.0f },
    { "cape", "neutral_x", GENO_P_CAPE_NEUTRAL_X, 0.3f },
    { "cape", "attack_buttons", GENO_P_CAPE_BUTTONS, (f32) (GENO_BTN_SPECIAL | GENO_BTN_ATTACK) },
};
#define GENO_NPARAMDEFS ((int) (sizeof(geno_params) / sizeof(geno_params[0])))

/* For the registry: "<family>" + "<name>" -> parameter id, -1 when unknown. */
int GenoGame_ParamFind(const char* family, const char* name)
{
    int i;
    for (i = 0; i < GENO_NPARAMDEFS; i++) {
        if (geno_streq(family, geno_params[i].family) && geno_streq(name, geno_params[i].name)) {
            return geno_params[i].id;
        }
    }
    /* every family also takes its Brawl block by word: "w00".."w31" */
    if (name[0] == 'w' && name[1] >= '0' && name[1] <= '9' && name[2] >= '0' && name[2] <= '9' &&
        name[3] == '\0')
    {
        int w = (name[1] - '0') * 10 + (name[2] - '0');
        if (geno_streq(family, "glide") && w <= 21) {
            return GENO_P_GLIDE_W0 + w;
        }
        if (geno_streq(family, "tornado") && w <= 19) {
            return GENO_P_TORNADO_W0 + w;
        }
        if (geno_streq(family, "drill") && w <= 5) {
            return GENO_P_DRILL_W0 + w;
        }
        if (geno_streq(family, "cape") && w <= 5) {
            return GENO_P_CAPE_W0 + w;
        }
    }
    return -1;
}

static f32 geno_param(GenoState* st, int id)
{
    int p = st->profile;
    if (p < 0 || p >= GENO_MAX_PROFILES || id < 0 || id >= GENO_PARAMS) {
        return 0.0f;
    }
    return Geno_Params[p][id];
}

static void geno_build_params(int p)
{
    int i, n;
    union {
        u32 u;
        f32 f;
    } b;
    for (i = 0; i < GENO_PARAMS; i++) {
        Geno_Params[p][i] = 0.0f;
    }
    for (i = 0; i < GENO_NPARAMDEFS; i++) {
        Geno_Params[p][geno_params[i].id] = geno_params[i].def;
    }
    n = Geno_ParamCount(p);
    for (i = 0; i < n; i++) {
        int id = Geno_ParamId(p, i);
        if (id >= 0 && id < GENO_PARAMS) {
            b.u = (u32) Geno_ParamBits(p, i);
            Geno_Params[p][id] = b.f;
        }
    }
}

/* ---- small helpers ------------------------------------------------------------------------------- */

static int geno_state_valid(GenoState* st, int s)
{
    return st->profile >= 0 && s >= 0 && s < GENO_MAX_STATES && s < Geno_StateCount(st->profile);
}

/* The Geno state the fighter is in, -1 when it is in none of its profile's states. */
static int geno_cur_state(Fighter* fp, GenoState* st)
{
    int s = (int) fp->motion_id - GENO_MOTION_BASE;
    return geno_state_valid(st, s) ? s : -1;
}

/* The profile's first state with behaviour `bhv`, -1 when it declares none. */
static int geno_state_with(GenoState* st, int bhv)
{
    int s, n = st->profile >= 0 ? Geno_StateCount(st->profile) : 0;
    for (s = 0; s < n && s < GENO_MAX_STATES; s++) {
        if (Geno_StateBehavior(st->profile, s) == bhv) {
            return s;
        }
    }
    return -1;
}

static f32 geno_clampf(f32 v, f32 lo, f32 hi)
{
    return v < lo ? lo : v > hi ? hi : v;
}

static f32 geno_absf(f32 v)
{
    return v < 0.0f ? -v : v;
}

#define GENO_DEG2RAD 0.017453292f

/* The animation has no frames left (tests: the capture flag). */
static int geno_anim_ended(Fighter_GObj* gobj)
{
    return geno_test_capture ? geno_test_anim_end : !ftAnim_IsFramesRemaining(gobj);
}

/* ---- motion rows ------------------------------------------------------------------------------ */

static MotionState* geno_vanilla_row(Fighter* fp, int msid)
{
    if (msid < 0 || msid >= GENO_MOTION_BASE) {
        return NULL;
    }
    if (msid >= (int) fp->x18) {
        return fp->x20_actionStateList != NULL && msid - (int) fp->x18 <= 0xFF
                   ? &fp->x20_actionStateList[msid - (int) fp->x18]
                   : NULL;
    }
    return fp->x1C_actionStateList != NULL ? &fp->x1C_actionStateList[msid] : NULL;
}

typedef void (*GenoCbFn)(Fighter_GObj* gobj);
typedef void (*GenoEnterFn)(Fighter_GObj* gobj, Fighter* fp, GenoState* st, int s);

typedef struct {
    int id;
    int slot;
    const char* name;
    GenoCbFn fn;
} GenoCbDef;

typedef struct {
    int id;
    const char* name;
    u8 cb[GENO_CB_SLOTS]; /* callback ids */
    GenoEnterFn enter;
    u32 next;       /* default anim-end target */
    u32 land;       /* default landing target */
    int like;       /* default like motion */
} GenoBhvDef;

static const GenoCbDef* geno_cb_def(int slot, int id);
static const GenoBhvDef* geno_bhv_def(int id);

/* Build profile p's rows (idempotent: the same bytes every time, from the registry and the
 * fighter's own vanilla / m-ex rows). */
static void geno_build_rows(Fighter* fp, int p)
{
    int n = Geno_StateCount(p);
    int s, slot;
    if (n > GENO_MAX_STATES) {
        n = GENO_MAX_STATES;
    }
    for (s = 0; s < n; s++) {
        MotionState* row = &Geno_Rows[p][s];
        const GenoBhvDef* b = geno_bhv_def(Geno_StateBehavior(p, s));
        u32 like_t = (u32) Geno_StateLike(p, s);
        int like = like_t != GENO_NONE_T ? geno_target_motion(fp, like_t) : b->like;
        MotionState* lr = geno_vanilla_row(fp, like);
        int anim = Geno_StateAnim(p, s);
        u32 from = (u32) Geno_StateAnimFrom(p, s);
        if (lr == NULL) {
            lr = geno_vanilla_row(fp, ftCo_MS_Fall);
        }
        if (lr != NULL) {
            *row = *lr;
        } else {
            geno_zero(row, sizeof(*row));
            row->anim_id = -1;
        }
        if (anim >= 0) {
            row->anim_id = anim;
        } else if (from != GENO_NONE_T) {
            MotionState* fr = geno_vanilla_row(fp, geno_target_motion(fp, from));
            row->anim_id = fr != NULL ? fr->anim_id : -1;
        }
        if (Geno_StateFlagsSet(p, s)) {
            row->x4_flags = (enum_t) Geno_StateFlags(p, s);
        }
        if (Geno_StateMoveId(p, s) >= 0) {
            row->move_id = (u8) Geno_StateMoveId(p, s);
        }
        for (slot = 0; slot < GENO_CB_SLOTS; slot++) {
            int id = Geno_StateCb(p, s, slot);
            const GenoCbDef* c;
            GenoCbFn fn;
            if (id == GENO_CB_LIKE) {
                continue; /* keep the like row's */
            }
            if (id < 0) {
                id = b->cb[slot];
            }
            if (id == GENO_CB_LIKE) {
                continue;
            }
            c = geno_cb_def(slot, id);
            fn = c != NULL ? c->fn : NULL;
            switch (slot) {
            case GENO_CB_ANIM:
                row->anim_cb = fn;
                break;
            case GENO_CB_IASA:
                row->input_cb = fn;
                break;
            case GENO_CB_PHYS:
                row->phys_cb = fn;
                break;
            default:
                row->coll_cb = fn;
                break;
            }
        }
        Geno_Event(20, fp->kind, fp->player_id, s, (int) row->anim_id);
    }
    Geno_RowCount[p] = n;
}

static void geno_v2_build(Fighter* fp, int p)
{
    if (p < 0 || p >= GENO_MAX_PROFILES) {
        return;
    }
    geno_build_params(p);
    geno_build_rows(fp, p);
}

/* fighter.c (Fighter_ChangeMotionState): the row of a motion in Geno's range. Never NULL: an
 * unknown id gets the fighter's Fall row (and a log line), so a bad id can not crash the engine. */
MotionState* Geno_MotionRow(Fighter* fp, int msid)
{
    GenoState* st = geno_state(fp);
    int s = msid - GENO_MOTION_BASE;
    if (geno_state_valid(st, s) && s < Geno_RowCount[st->profile]) {
        return &Geno_Rows[st->profile][s];
    }
    Geno_Event(8, fp->kind, fp->player_id, s, 0);
    return geno_vanilla_row(fp, ftCo_MS_Fall);
}

/* ---- entering a state ----------------------------------------------------------------------------- */

/* The animation frame a state starts at. The glide is posed by its angle (pose_center - angle, and
 * the glide always starts level): starting its clip at frame 0 showed the clip's first pose (nose
 * straight up) for one frame before the phys callback scrubbed it (v3 fix). */
static f32 geno_start_frame(Fighter* fp, GenoState* st, int s)
{
    if (Geno_StateBehavior(st->profile, s) == GENO_BHV_GLIDE) {
        f32 pose = geno_param(st, GENO_P_GLIDE_POSE);
        if (pose > 0.0f) {
            return pose; /* - angle, which geno_glide_enter sets to 0 */
        }
    }
    return 0.0f;
}

static int geno_enter_state(Fighter_GObj* gobj, Fighter* fp, GenoState* st, int s, u32 target)
{
    const GenoBhvDef* b = geno_bhv_def(Geno_StateBehavior(st->profile, s));
    int from = (int) fp->motion_id;
    /* a KEEP_FRAME swap (the same move's ground / air version) carries the root-motion progress:
       no second origin step, the same travel facing, the gravity accumulated so far */
    s32 k_started = st->motion_started, k_ledge = st->ledge;
    f32 k_vy = st->motion_vy, k_facing = st->motion_facing, k_grav = st->motion_gravity;
    if (st->profile >= GENO_MAX_PROFILES || s >= Geno_RowCount[st->profile]) {
        Geno_Event(8, fp->kind, fp->player_id, s, 0);
        return 0;
    }
    st->changes++;
    st->state_entries++;
    st->enter_from = from;
    st->enter_keep = (target & GENO_TGT_KEEP_FRAME) != 0;
    Geno_Event(16, fp->kind, fp->player_id, s, from);
    if (geno_test_capture) {
        geno_test_target = target;
        geno_test_changes++;
        fp->motion_id = GENO_MOTION_BASE + s;
        Geno_OnActionChange(gobj);
    } else if (target & GENO_TGT_KEEP_FRAME) {
        Fighter_ChangeMotionState(gobj, GENO_MOTION_BASE + s, GENO_KEEP_FRAME_FLAGS,
                                  fp->cur_anim_frame, 1.0f, 0.0f, NULL);
    } else {
        Fighter_ChangeMotionState(gobj, GENO_MOTION_BASE + s, Ft_MF_None,
                                  geno_start_frame(fp, st, s), 1.0f, 0.0f, NULL);
    }
    if (st->hidden) {
        fp->x221E_b5 = 1; /* v3 HIDDEN survives Geno-to-Geno changes (ChangeMotionState clears it) */
    }
    if (st->enter_keep) {
        st->motion_started = k_started;
        st->motion_vy = k_vy;
        st->motion_facing = k_facing;
        st->motion_gravity = k_grav;
        st->ledge = k_ledge;
    }
    if (b->enter != NULL) {
        b->enter(gobj, fp, st, s);
    }
    return 1;
}

/* A default / declared target: resolves the internal kinds, then performs it. */
static int geno_go(Fighter_GObj* gobj, Fighter* fp, GenoState* st, u32 t)
{
    if (t == GENO_NONE_T) {
        return 0;
    }
    if (t == GENO_TGT_HELPLESS) {
        return geno_do_change(gobj, fp, st,
                              GENO_TARGET(GENO_TGT_MOTION, fp->ground_or_air == GA_Ground
                                                               ? ftCo_MS_Wait
                                                               : ftCo_MS_FallSpecial));
    }
    if (t == GENO_AUTO) {
        /* Melee's own Wait / Fall entries (geno_do_change's common targets) */
        return geno_do_change(gobj, fp, st,
                              GENO_TARGET(GENO_TGT_MOTION, fp->ground_or_air == GA_Ground
                                                               ? ftCo_MS_Wait
                                                               : ftCo_MS_Fall));
    }
    if ((t >> 28) == GENO_TGT_BHV) {
        int s = geno_state_with(st, (int) (t & 0xFFFF));
        if (s < 0) {
            /* the profile has no such state: the generic end */
            return geno_go(gobj, fp, st, GENO_AUTO);
        }
        t = GENO_TARGET(GENO_TGT_GENO, s);
    }
    return geno_do_change(gobj, fp, st, t);
}

/* The state's anim-end target (geno.json "next", else the behaviour's default). */
static u32 geno_next_of(GenoState* st, int s)
{
    u32 t = (u32) Geno_StateNext(st->profile, s);
    return t != GENO_NONE_T ? t : geno_bhv_def(Geno_StateBehavior(st->profile, s))->next;
}

static u32 geno_land_of(GenoState* st, int s)
{
    u32 t = (u32) Geno_StateLand(st->profile, s);
    return t != GENO_NONE_T ? t : geno_bhv_def(Geno_StateBehavior(st->profile, s))->land;
}

static f32 geno_lag_of(GenoState* st, int s)
{
    union {
        u32 u;
        f32 f;
    } b;
    b.u = (u32) Geno_StateLagBits(st->profile, s);
    return b.f;
}

/* ---- generic callbacks ---------------------------------------------------------------------------- */

/* anim "next": at the end of the animation, the state's next target */
static void geno_cb_anim_next(Fighter_GObj* gobj)
{
    Fighter* fp = GET_FIGHTER(gobj);
    GenoState* st = geno_state(fp);
    int s = geno_cur_state(fp, st);
    if (geno_anim_ended(gobj)) {
        geno_go(gobj, fp, st, s >= 0 ? geno_next_of(st, s) : GENO_AUTO);
    }
}

/* anim "loop": the animation restarts (the move's vars survive: they are not per action) */
static void geno_cb_anim_loop(Fighter_GObj* gobj)
{
    Fighter* fp = GET_FIGHTER(gobj);
    if (geno_anim_ended(gobj) && !geno_test_capture) {
        Fighter_ChangeMotionState(gobj, fp->motion_id,
                                  Ft_MF_KeepFastFall | Ft_MF_KeepGfx | Ft_MF_KeepSfx |
                                      Ft_MF_SkipModelPartVis | Ft_MF_SkipItemVis |
                                      Ft_MF_SkipNametagVis | Ft_MF_KeepSwordTrail,
                                  0.0f, 1.0f, 0.0f, NULL);
    }
}

static void geno_cb_nothing(Fighter_GObj* gobj) {}

static void geno_cb_phys_air(Fighter_GObj* gobj)
{
    ft_80084DB0(gobj); /* gravity (fast fall) + drift, as aerial attacks */
}

static void geno_cb_phys_air_nodrift(Fighter_GObj* gobj)
{
    Fighter* fp = GET_FIGHTER(gobj);
    ftCommon_Fall(fp, fp->co_attrs.gravity, fp->co_attrs.terminal_velocity);
    ftCommon_CalcSelfAccel_DeaccelAir(fp);
}

static void geno_cb_phys_ground(Fighter_GObj* gobj)
{
    ft_80084F3C(gobj); /* ground friction */
}

/* phys "auto": friction on the ground, gravity + drift in the air (states that live in both) */
static void geno_cb_phys_auto(Fighter_GObj* gobj)
{
    if (GET_FIGHTER(gobj)->ground_or_air == GA_Ground) {
        ft_80084F3C(gobj);
    } else {
        ft_80084DB0(gobj);
    }
}

/* Landing from an air state: become grounded, then the state's landing target (a Geno state, a
 * motion) or Melee's landing (with the state's landing lag when it has one). */
static void geno_land(Fighter_GObj* gobj)
{
    Fighter* fp = GET_FIGHTER(gobj);
    GenoState* st = geno_state(fp);
    int s = geno_cur_state(fp, st);
    u32 t = s >= 0 ? geno_land_of(st, s) : GENO_AUTO;
    f32 lag = s >= 0 ? geno_lag_of(st, s) : 0.0f;
    if (t == GENO_AUTO || t == GENO_NONE_T) {
        if (lag > 0.0f) {
            ftCo_LandingFallSpecial_Enter(gobj, true, lag);
        } else {
            ftCo_Landing_Enter_Basic(gobj);
        }
        return;
    }
    ftCommon_8007D7FC(fp);
    if (!geno_go(gobj, fp, st, t)) {
        ftCo_Landing_Enter_Basic(gobj);
    }
}

/* coll "air": Melee's air collision with ledge grab (platforms pass-through while holding down,
 * like FallSpecial), landing -> geno_land */
static void geno_cb_coll_air(Fighter_GObj* gobj)
{
    ft_80083090(gobj, ftCo_80096CC8, geno_land);
}

/* coll "air_noledge": as aerial attacks (no ledge grab) */
static void geno_cb_coll_air_noledge(Fighter_GObj* gobj)
{
    ft_80082C74(gobj, geno_land);
}

/* coll "ground": walking off an edge -> Fall */
static void geno_cb_coll_ground(Fighter_GObj* gobj)
{
    ft_80084104(gobj);
}

/* coll "ground_stop": stops at edges */
static void geno_cb_coll_ground_stop(Fighter_GObj* gobj)
{
    ft_80084280(gobj);
}

/* A state that works on the ground and in the air without changing action (the tornado, the
 * drill): the situation flips, the state stays. */
static void geno_become_air(Fighter_GObj* gobj)
{
    ftCommon_8007D5D4(GET_FIGHTER(gobj));
}

static void geno_become_ground(Fighter_GObj* gobj)
{
    ftCommon_8007D7FC(GET_FIGHTER(gobj));
}

static void geno_coll_both(Fighter_GObj* gobj)
{
    Fighter* fp = GET_FIGHTER(gobj);
    if (fp->ground_or_air == GA_Ground) {
        ft_8008403C(gobj, geno_become_air);
    } else {
        ft_80082C74(gobj, geno_become_ground);
    }
}

/* ---- v3: geno.anim_motion (root motion on the ground and in the air) -------------------------------
 *
 * Brawl moves many specials by the clip's root motion (TransN) in the air as well as on the ground
 * (Shuttle Loop, Dimensional Cape's F/B reappear). Melee extracts TransN only on subaction rows whose
 * flags have 0x80000000 (fp->x594_b0: TransN is zeroed on the model and its per-frame delta lands in
 * fp->x6A4_transNOffset, z forward / y up) and only a few of its phys helpers consume it. This
 * behaviour consumes it everywhere:
 *   ground  gr_vel = forward x facing (Melee's own ft_800850E0); with "liftoff", an upward delta
 *           takes off (then the air rule applies the same frame); walking off an edge -> airborne,
 *           same state
 *   air     self_vel = (forward x facing, up) + gravity x "gravity" accumulated from the state's start
 *           (0 = pure root motion, Brawl's motion kinetics); "origin": the first frame also moves by
 *           the clip's frame-0 offset (Brawl clips that start away from the origin)
 *   ledge   per action: the state's "ledge" default, overridden by scripts (PUT LEDGE, PSA
 *           Allow/Disallow Ledgegrab): 0 none, 1 front (Melee's own), 2 front and back
 *   landing the state's "land" target (a Geno state, a motion) or Melee's landing; "landing_lag"
 *           -> LandingFallSpecial with that lag
 * A row without 0x80000000 has no root motion: the state then holds still (ground: friction). */

static u32 geno_motion_opts(GenoState* st, int s)
{
    return s >= 0 ? (u32) Geno_StateMotion(st->profile, s) : 0u;
}

static f32 geno_motion_gravity(GenoState* st, int s)
{
    union {
        u32 u;
        f32 f;
    } b;
    b.u = s >= 0 ? (u32) Geno_StateGravityBits(st->profile, s) : 0u;
    return b.f;
}

static void geno_cb_phys_anim_motion(Fighter_GObj* gobj)
{
    Fighter* fp = GET_FIGHTER(gobj);
    GenoState* st = geno_state(fp);
    int s = geno_cur_state(fp, st);
    u32 o = geno_motion_opts(st, s);
    f32 g = st->motion_gravity >= 0.0f ? st->motion_gravity : geno_motion_gravity(st, s);
    f32 fwd = 0.0f, up = 0.0f;
    f32 lr = fp->facing_dir < 0.0f ? -1.0f : 1.0f;
    if (o & GENO_MOTION_ENTRY_FACING) {
        if (!st->motion_started) {
            st->motion_facing = lr;
        }
        lr = st->motion_facing < 0.0f ? -1.0f : 1.0f;
    }
    if (fp->x594_b0) {
        fwd = fp->x6A4_transNOffset.z;
        up = fp->x6A4_transNOffset.y;
        if (!st->motion_started && (o & GENO_MOTION_ORIGIN)) {
            fwd += fp->x68C_transNPos.z; /* the clip's frame-0 offset (already model-scaled) */
            up += fp->x68C_transNPos.y;
        }
    }
    st->motion_started = 1;
    if (fp->ground_or_air == GA_Ground) {
        if (!fp->x594_b0) {
            ft_80084F3C(gobj); /* no root motion: friction */
            return;
        }
        if ((o & GENO_MOTION_LIFTOFF) && up > 0.001f) {
            Geno_Event(23, fp->kind, fp->player_id, s, st->action_time);
            ftCommon_8007D5D4(fp); /* take off: the air rule below moves this frame */
        } else {
            fp->gr_vel = fwd * lr;
            ftCommon_SetSelfMovementFromGroundedMovement(gobj);
            return;
        }
    }
    if (g != 0.0f) {
        st->motion_vy -= fp->co_attrs.gravity * g;
        if (st->motion_vy < -fp->co_attrs.terminal_velocity) {
            st->motion_vy = -fp->co_attrs.terminal_velocity;
        }
    }
    if (!fp->x594_b0 && g == 0.0f) {
        fp->self_vel.x = 0.0f; /* no root motion and no gravity: hold still (Brawl's stall) */
        fp->self_vel.y = 0.0f;
        return;
    }
    fp->self_vel.x = fwd * lr;
    fp->self_vel.y = up + st->motion_vy;
}

/* Landing out of a root-motion state (and any state that uses geno_land): as geno_land, logged. */
static void geno_motion_land(Fighter_GObj* gobj)
{
    Fighter* fp = GET_FIGHTER(gobj);
    GenoState* st = geno_state(fp);
    int s = geno_cur_state(fp, st);
    Geno_Event(24, fp->kind, fp->player_id, s, s >= 0 ? (int) geno_land_of(st, s) : -1);
    if (s >= 0 && geno_land_of(st, s) == GENO_TGT_STAY) {
        ftCommon_8007D7FC(fp); /* grounded; the state (and a script's GROUND check) goes on */
        return;
    }
    geno_land(gobj);
}

/* Melee's air collision with a ledge grab in the facing direction (dir +-1) or either direction (0:
 * mpColl then tests both ledges; ftCliffCommon turns the fighter to face the ledge it caught). */
static void geno_coll_air_ledge(Fighter_GObj* gobj, int both, HSD_GObjEvent land)
{
    Fighter* fp = GET_FIGHTER(gobj);
    CollData* coll = &fp->coll_data;
    bool touched;
    coll->last_pos = coll->cur_pos;
    coll->cur_pos = fp->cur_pos;
    if (fp->x2064_ledgeCooldown || fp->x2224_b2) {
        touched = mpColl_80047AC8(coll, ftCo_80096CC8, gobj);
    } else {
        mpCollSetFacingDir(coll, both ? 0 : (fp->facing_dir < 0.0f ? -1 : 1));
        touched = mpColl_80047E14(coll, ftCo_80096CC8, gobj);
    }
    fp->cur_pos = coll->cur_pos;
    if (ft_80081A00(gobj)) {
        return;
    }
    if (touched) {
        land(gobj);
        return;
    }
    if (ftCliffCommon_80081298(gobj)) {
        Geno_Event(25, fp->kind, fp->player_id, (int) fp->motion_id - GENO_MOTION_BASE, both ? 2 : 1);
    }
}

/* This action's ledge grab: a script's PUT LEDGE, else the state's "ledge" default. */
static int geno_ledge_mode(Fighter* fp, GenoState* st)
{
    int s = geno_cur_state(fp, st);
    return st->ledge >= 0 ? st->ledge : (int) (geno_motion_opts(st, s) & GENO_MOTION_LEDGE_MASK);
}

int GenoGame_TestLedgeMode(Fighter* fp)
{
    return geno_ledge_mode(fp, geno_state(fp));
}

f32 GenoGame_TestStartFrame(Fighter* fp, int s)
{
    return geno_start_frame(fp, geno_state(fp), s);
}

static void geno_cb_coll_anim_motion(Fighter_GObj* gobj)
{
    Fighter* fp = GET_FIGHTER(gobj);
    GenoState* st = geno_state(fp);
    int ledge;
    if (fp->ground_or_air == GA_Ground) {
        ft_8008403C(gobj, geno_become_air); /* off an edge: airborne, same state */
        return;
    }
    ledge = geno_ledge_mode(fp, st);
    if (ledge == 0) {
        ft_80082C74(gobj, geno_motion_land);
    } else {
        geno_coll_air_ledge(gobj, ledge == 2, geno_motion_land);
    }
}

/* The Brawl-style direction the stick asks for: jump held counts (X/Y), and the stick up past
 * Melee's tap-jump threshold. */
static int geno_jump_held(Fighter* fp)
{
    return (fp->input.held_buttons[0] & HSD_PAD_XY) != 0 ||
           (p_ftCommonData != NULL &&
            fp->input.lstick[0].y >= p_ftCommonData->tap_jump_threshold);
}

/* ---- glide ---------------------------------------------------------------------------------------- */
#include "geno_game_glide.inc"

/* ---- Mach Tornado and Drill Rush ------------------------------------------------------------------ */
#include "geno_game_specials.inc"

/* ---- tables ---------------------------------------------------------------------------------------- */

enum {
    GENO_CBID_NEXT = 1,
    GENO_CBID_LOOP = 2,
    GENO_CBID_HOLD = 3,
    GENO_CBID_NONE = 4,
    GENO_CBID_AIR = 5,
    GENO_CBID_AIR_NODRIFT = 6,
    GENO_CBID_GROUND = 7,
    GENO_CBID_AIR_NOLEDGE = 8,
    GENO_CBID_GROUND_STOP = 9,
    GENO_CBID_BOTH = 10,
    GENO_CBID_AUTO = 11,
    GENO_CBID_ANIM_MOTION = 12, /* v3 */
    GENO_CBID_INTERRUPT = 13,   /* v3 */
    GENO_CBID_CAPE = 50,        /* v3 */
    GENO_CBID_CAPE_AFTER = 51,  /* v3 */
    GENO_CBID_GLIDE_AFTER = 25, /* v3 */
    GENO_CBID_GLIDE_START = 20,
    GENO_CBID_GLIDE = 21,
    GENO_CBID_GLIDE_ATTACK = 22,
    GENO_CBID_GLIDE_END = 24,
    GENO_CBID_TORNADO = 30,
    GENO_CBID_DRILL = 40,
    GENO_CBID_DRILL_END = 41,
    GENO_CBID_DRILL_START = 42,
};

/* Stable ids per slot; names are what geno.json's "anim" / "iasa" / "phys" / "coll" keys take. */
static const GenoCbDef geno_cbs[] = {
    { GENO_CBID_NEXT, GENO_CB_ANIM, "next", geno_cb_anim_next },
    { GENO_CBID_LOOP, GENO_CB_ANIM, "loop", geno_cb_anim_loop },
    { GENO_CBID_HOLD, GENO_CB_ANIM, "hold", geno_cb_nothing },
    { GENO_CBID_GLIDE_START, GENO_CB_ANIM, "glide.start", geno_glide_start_anim },
    { GENO_CBID_GLIDE, GENO_CB_ANIM, "glide", geno_glide_anim },
    { GENO_CBID_TORNADO, GENO_CB_ANIM, "tornado", geno_tornado_anim },
    { GENO_CBID_DRILL, GENO_CB_ANIM, "drill", geno_drill_anim },
    { GENO_CBID_DRILL_END, GENO_CB_ANIM, "drill.end", geno_drill_end_anim },
    { GENO_CBID_GLIDE_AFTER, GENO_CB_ANIM, "glide.after", geno_glide_after_anim },
    { GENO_CBID_CAPE, GENO_CB_ANIM, "cape", geno_cape_anim },
    { GENO_CBID_INTERRUPT, GENO_CB_IASA, "interrupt", geno_cb_iasa_interrupt },
    { GENO_CBID_CAPE, GENO_CB_PHYS, "cape", geno_cape_phys },
    { GENO_CBID_CAPE, GENO_CB_COLL, "cape", geno_cape_coll },
    { GENO_CBID_CAPE_AFTER, GENO_CB_COLL, "cape.after", geno_cape_after_coll },
    { GENO_CBID_NONE, GENO_CB_IASA, "none", geno_cb_nothing },
    { GENO_CBID_GLIDE, GENO_CB_IASA, "glide", geno_glide_iasa },
    { GENO_CBID_NONE, GENO_CB_PHYS, "none", geno_cb_nothing },
    { GENO_CBID_AIR, GENO_CB_PHYS, "air", geno_cb_phys_air },
    { GENO_CBID_AIR_NODRIFT, GENO_CB_PHYS, "air_nodrift", geno_cb_phys_air_nodrift },
    { GENO_CBID_GROUND, GENO_CB_PHYS, "ground", geno_cb_phys_ground },
    { GENO_CBID_AUTO, GENO_CB_PHYS, "auto", geno_cb_phys_auto },
    { GENO_CBID_ANIM_MOTION, GENO_CB_PHYS, "anim_motion", geno_cb_phys_anim_motion },
    { GENO_CBID_GLIDE_START, GENO_CB_PHYS, "glide.start", geno_glide_start_phys },
    { GENO_CBID_GLIDE, GENO_CB_PHYS, "glide", geno_glide_phys },
    { GENO_CBID_GLIDE_ATTACK, GENO_CB_PHYS, "glide.attack", geno_glide_attack_phys },
    { GENO_CBID_GLIDE_END, GENO_CB_PHYS, "glide.end", geno_glide_end_phys },
    { GENO_CBID_TORNADO, GENO_CB_PHYS, "tornado", geno_tornado_phys },
    { GENO_CBID_DRILL, GENO_CB_PHYS, "drill", geno_drill_phys },
    { GENO_CBID_DRILL_END, GENO_CB_PHYS, "drill.end", geno_drill_end_phys },
    { GENO_CBID_DRILL_START, GENO_CB_PHYS, "drill.start", geno_drill_start_phys },
    { GENO_CBID_NONE, GENO_CB_COLL, "none", geno_cb_nothing },
    { GENO_CBID_AIR, GENO_CB_COLL, "air", geno_cb_coll_air },
    { GENO_CBID_AIR_NOLEDGE, GENO_CB_COLL, "air_noledge", geno_cb_coll_air_noledge },
    { GENO_CBID_GROUND, GENO_CB_COLL, "ground", geno_cb_coll_ground },
    { GENO_CBID_GROUND_STOP, GENO_CB_COLL, "ground_stop", geno_cb_coll_ground_stop },
    { GENO_CBID_BOTH, GENO_CB_COLL, "both", geno_coll_both },
    { GENO_CBID_ANIM_MOTION, GENO_CB_COLL, "anim_motion", geno_cb_coll_anim_motion },
    { GENO_CBID_GLIDE, GENO_CB_COLL, "glide", geno_glide_coll },
    { GENO_CBID_DRILL, GENO_CB_COLL, "drill", geno_drill_coll },
    { GENO_CBID_DRILL_START, GENO_CB_COLL, "drill.start", geno_drill_start_coll },
};
#define GENO_NCBS ((int) (sizeof(geno_cbs) / sizeof(geno_cbs[0])))

#define BHV(id) (GENO_TARGET(GENO_TGT_BHV, (id)))
#define MOT(id) (GENO_TARGET(GENO_TGT_MOTION, (id)))

static const GenoBhvDef geno_bhvs[] = {
    { GENO_BHV_NONE, "",
      { GENO_CBID_NEXT, GENO_CBID_NONE, GENO_CBID_AIR, GENO_CBID_AIR },
      NULL, GENO_AUTO, GENO_AUTO, ftCo_MS_Fall },
    { GENO_BHV_AIR, "geno.air",
      { GENO_CBID_NEXT, GENO_CBID_NONE, GENO_CBID_AIR, GENO_CBID_AIR },
      NULL, GENO_AUTO, GENO_AUTO, ftCo_MS_Fall },
    { GENO_BHV_GROUND, "geno.ground",
      { GENO_CBID_NEXT, GENO_CBID_NONE, GENO_CBID_GROUND, GENO_CBID_GROUND },
      NULL, GENO_AUTO, GENO_AUTO, ftCo_MS_Landing },
    { GENO_BHV_ANIM_MOTION, "geno.anim_motion",
      { GENO_CBID_NEXT, GENO_CBID_NONE, GENO_CBID_ANIM_MOTION, GENO_CBID_ANIM_MOTION },
      NULL, GENO_AUTO, GENO_AUTO, ftCo_MS_Fall },
    { GENO_BHV_GLIDE_START, "geno.glide.start",
      { GENO_CBID_GLIDE_START, GENO_CBID_NONE, GENO_CBID_GLIDE_START, GENO_CBID_AIR },
      geno_glide_start_enter, BHV(GENO_BHV_GLIDE), BHV(GENO_BHV_GLIDE_LANDING), ftCo_MS_Fall },
    { GENO_BHV_GLIDE, "geno.glide",
      { GENO_CBID_GLIDE, GENO_CBID_GLIDE, GENO_CBID_GLIDE, GENO_CBID_GLIDE },
      geno_glide_enter, BHV(GENO_BHV_GLIDE_END), BHV(GENO_BHV_GLIDE_LANDING), ftCo_MS_Fall },
    { GENO_BHV_GLIDE_ATTACK, "geno.glide.attack",
      { GENO_CBID_GLIDE_AFTER, GENO_CBID_NONE, GENO_CBID_GLIDE_ATTACK, GENO_CBID_AIR_NOLEDGE },
      geno_glide_attack_enter, MOT(ftCo_MS_Fall), GENO_AUTO, ftCo_MS_AttackAirN },
    { GENO_BHV_GLIDE_LANDING, "geno.glide.landing",
      { GENO_CBID_NEXT, GENO_CBID_NONE, GENO_CBID_GROUND, GENO_CBID_GROUND },
      geno_glide_landing_enter, GENO_AUTO, GENO_AUTO, ftCo_MS_LandingFallSpecial },
    { GENO_BHV_GLIDE_END, "geno.glide.end",
      { GENO_CBID_GLIDE_AFTER, GENO_CBID_NONE, GENO_CBID_GLIDE_END, GENO_CBID_AIR },
      geno_glide_end_enter, GENO_AUTO, GENO_AUTO, ftCo_MS_Fall },
    { GENO_BHV_TORNADO, "geno.tornado",
      { GENO_CBID_TORNADO, GENO_CBID_NONE, GENO_CBID_TORNADO, GENO_CBID_BOTH },
      geno_tornado_enter, GENO_NONE_T, GENO_AUTO, ftCo_MS_Fall },
    { GENO_BHV_DRILL, "geno.drill",
      { GENO_CBID_DRILL, GENO_CBID_NONE, GENO_CBID_DRILL, GENO_CBID_DRILL },
      geno_drill_enter, BHV(GENO_BHV_DRILL_END), GENO_AUTO, ftCo_MS_Fall },
    { GENO_BHV_DRILL_END, "geno.drill.end",
      { GENO_CBID_DRILL_END, GENO_CBID_NONE, GENO_CBID_DRILL_END, GENO_CBID_BOTH },
      geno_drill_end_enter, GENO_NONE_T, GENO_AUTO, ftCo_MS_Fall },
    { GENO_BHV_CAPE, "geno.cape",
      { GENO_CBID_CAPE, GENO_CBID_NONE, GENO_CBID_CAPE, GENO_CBID_CAPE },
      geno_cape_enter, GENO_NONE_T, GENO_AUTO, ftCo_MS_Fall },
    { GENO_BHV_CAPE_ATTACK, "geno.cape.attack",
      { GENO_CBID_NEXT, GENO_CBID_NONE, GENO_CBID_ANIM_MOTION, GENO_CBID_CAPE_AFTER },
      NULL, GENO_TGT_HELPLESS, GENO_TGT_STAY, ftCo_MS_Fall },
    { GENO_BHV_CAPE_END, "geno.cape.end",
      { GENO_CBID_NEXT, GENO_CBID_INTERRUPT, GENO_CBID_ANIM_MOTION, GENO_CBID_CAPE_AFTER },
      NULL, GENO_TGT_HELPLESS, GENO_TGT_STAY, ftCo_MS_Fall },
    { GENO_BHV_DRILL_START, "geno.drill.start",
      { GENO_CBID_NEXT, GENO_CBID_NONE, GENO_CBID_DRILL_START, GENO_CBID_DRILL_START },
      geno_drill_start_enter, BHV(GENO_BHV_DRILL), GENO_AUTO, ftCo_MS_Fall },
};
#define GENO_NBHVS ((int) (sizeof(geno_bhvs) / sizeof(geno_bhvs[0])))

#undef BHV
#undef MOT

static const GenoCbDef* geno_cb_def(int slot, int id)
{
    int i;
    for (i = 0; i < GENO_NCBS; i++) {
        if (geno_cbs[i].slot == slot && geno_cbs[i].id == id) {
            return &geno_cbs[i];
        }
    }
    return NULL;
}

static const GenoBhvDef* geno_bhv_def(int id)
{
    int i;
    for (i = 0; i < GENO_NBHVS; i++) {
        if (geno_bhvs[i].id == id) {
            return &geno_bhvs[i];
        }
    }
    return &geno_bhvs[0];
}

/* For the registry. */
int GenoGame_BehaviorFind(const char* name)
{
    int i;
    for (i = 1; i < GENO_NBHVS; i++) {
        if (geno_streq(name, geno_bhvs[i].name)) {
            return geno_bhvs[i].id;
        }
    }
    return -1;
}

int GenoGame_CallbackFind(int slot, const char* name)
{
    int i;
    if (geno_streq(name, "like")) {
        return GENO_CB_LIKE;
    }
    for (i = 0; i < GENO_NCBS; i++) {
        if (geno_cbs[i].slot == slot && geno_streq(name, geno_cbs[i].name)) {
            return geno_cbs[i].id;
        }
    }
    return -1;
}

/* ---- per-frame entry points ------------------------------------------------------------------------ */

/* Geno_PreAnim, after the registered checks: the glide's jump-hold entry. */
static int geno_v2_preanim(Fighter_GObj* gobj, Fighter* fp, GenoState* st)
{
    return geno_glide_entry(gobj, fp, st);
}

/* The special sites (ftCo_Attack100.c, ftCo_SpecialAir.c, ftCo_SpecialS.c), before Melee's / m-ex's
 * own special: a special the profile binds to a target enters that target instead. Returns 1 when
 * it did (the caller then skips the fighter's own special). 0 at once for a fighter with no
 * profile. */
int Geno_SpecialEnter(Fighter_GObj* gobj, int which)
{
    Fighter* fp = GET_FIGHTER(gobj);
    GenoState* st = geno_state(fp);
    u32 t;
    if (st->profile < 0) {
        return 0;
    }
    t = (u32) Geno_Special(st->profile, which);
    if (t == GENO_NONE_T || !geno_target_ok(fp, t)) {
        return 0;
    }
    Geno_Event(18, fp->kind, fp->player_id, which, (int) t);
    return geno_do_change(gobj, fp, st, t);
}

/* Lab (pc/gameworld/script_game.c): a Geno row by motion id, NULL when none. */
MotionState* Geno_MotionRowIfAny(Fighter* fp, int msid)
{
    GenoState* st = geno_state(fp);
    int s = msid - GENO_MOTION_BASE;
    return geno_state_valid(st, s) && s < Geno_RowCount[st->profile] ? &Geno_Rows[st->profile][s]
                                                                      : NULL;
}
