/* TARGET_PC-only: gd.cpu_assist, a per-entity technique assist applied ON TOP of the retail AI's own
 * virtual pad. Included by fighter.c after technical_ai.inc (which it shares helpers with).
 *
 * The AI keeps running in fight mode and is never reset: cpu_assist_overlay() is called from the same
 * point as technical_ai_overlay(), after fp->input has been built from fp->cpu.* and before the
 * pressed-button edges, the L/R normalisation and the L-cancel/tech/shield timers read it. It ORs
 * (or, for a stick direction the technique needs, overrides for the technique's own frames) the
 * technique's input into fp->input, so everything downstream is ordinary input and the skill
 * events fire by themselves. Nothing here writes physics, an action state, a timer or a hitbox.
 *
 * Each value is the probability (permille across the native boundary) that the engine performs the
 * technique when its opportunity arises. One draw per opportunity, from a per-entity xorshift32
 * kept in this slot: game BSS, so a savestate, rewind or rollback restores it and the counters.
 * Probability 0 never performs and probability 1000 always does; the draw happens either way, so a
 * run with every value 0 consumes only the generator and changes nothing in the match.
 *
 *   lcancel        a fresh light shoulder press (analog, never an air dodge) inside the L-cancel window
 *                  before a lagged aerial lands. Landing is predicted with the floor ray the technical
 *                  overlay uses (velocity and ECB); the press is made when the predicted landing is
 *                  within CA_LC_ETA frames.
 *   tech           a digital R press inside the tech window before a hitstun landing, with the stick held
 *                  toward tech_dir for the engine's tech window (the game reads the stick at the landing).
 *   perfect_shield a shield press when a foe's hitbox, extrapolated one frame along its own sweep, is
 *                  within reach of the fighter's body; held CA_PS_HOLD frames. It sees a hitbox only from
 *                  its second active frame (the first is created after inputs), so it catches multi-hit,
 *                  long-lived and approaching hitboxes, not a single-frame first hit.
 *   wavedash       on the ground, when the AI itself starts a jump from standing/walking/running (the
 *                  jumpsquat), an air dodge on the first frame of the jump, angled toward the AI's own stick direction (toward the nearest foe
 *                  when the stick is neutral), if floor lies ahead.
 *   fast_fall      a down flick when the fighter is descending in an ordinary air state. */
#define CA_N 5
enum { CA_LCANCEL, CA_PSHIELD, CA_TECH, CA_WAVEDASH, CA_FASTFALL };
enum { CA_DIR_IN, CA_DIR_AWAY, CA_DIR_TOWARD, CA_DIR_RANDOM };
#define CA_LC_ETA 5
#define CA_TECH_ETA 10
#define CA_PS_HOLD 12
#define CA_WD_ANGLE_X 0.9397f   /* cos 20 deg */
#define CA_WD_ANGLE_Y (-0.3420f)

typedef struct CpuAssistSlot {
    Fighter* fighter;
    int frames, logged, ps_log;
    int spawn, owner;
    unsigned seed;
    int permille[CA_N];
    int tech_dir;
    int opportunities[CA_N], performed[CA_N];
    /* internal state, per technique instance */
    int prev_motion, motion_age;
    int lc_state;                 /* 0 no landing seen, 1 decided no, 2 decided yes (not yet pressed), 3 pressed */
    int tech_state, tech_hold, tech_x;
    int ps_hold, ps_gap, ps_state;
    int wd_state, wd_hold, wd_dir;
    int ff_state, ff_tries;
} CpuAssistSlot;
static CpuAssistSlot cpu_assist_slots[6];
/* the first 300 injections of a slot are logged (what, which fighter, motion, the slot's own frame count) */
#define CA_LOG(s, fp, what) do { if ((s)->logged < 300) { ++(s)->logged; \
    OSReport("CPU-ASSIST: P%d %s frame=%d motion=%d\n", (fp)->player_id + 1, what, (s)->frames, (fp)->motion_id); } } while (0)

/* one draw: returns 0..999 */
static int cpu_assist_roll(CpuAssistSlot* s)
{
    unsigned x = s->seed;
    x ^= x << 13; x ^= x >> 17; x ^= x << 5;
    s->seed = x;
    return (int) (((unsigned long long) (x >> 8) * 1000ull) >> 24);
}
/* an opportunity of technique t: counts it, draws once; returns the draw's normalised position in
 * [0,1000) scaled by the probability (so a "random" direction can reuse the same draw) or -1 for no */
static int cpu_assist_decide(CpuAssistSlot* s, int t, int* sub)
{
    int r = cpu_assist_roll(s), p = s->permille[t];
    ++s->opportunities[t];
    if (r >= p) return 0;
    if (sub) *sub = p > 0 ? r * 1000 / p : 0; /* uniform over 0..999 */
    return 1;
}

static int cpu_assist_supported(Fighter* fp)
{
    return fp != NULL && !fp->is_sub_fighter && fp->player_id < 6 &&
        Player_GetPlayerSlotType(fp->player_id) == Gm_PKind_Cpu && ftCo_IsCpuControlled(fp);
}

static int cpu_assist_config_ok(int lc, int ps, int tech, int tdir, int wd, int ff)
{
    return lc >= 0 && lc <= 1000 && ps >= 0 && ps <= 1000 && tech >= 0 && tech <= 1000 &&
           wd >= 0 && wd <= 1000 && ff >= 0 && ff <= 1000 && tdir >= 0 && tdir <= CA_DIR_RANDOM;
}

int ScriptGame_CpuAssist(int slot, int owner, int lc, int ps, int tech, int tdir, int wd, int ff, int seed)
{
    CpuAssistSlot* s;
    HSD_GObj* gobj;
    Fighter* fp;
    if (slot < 0 || slot >= 6 || owner <= 0 || seed <= 0 || !cpu_assist_config_ok(lc, ps, tech, tdir, wd, ff) ||
        Netplay_Enabled() || RB_Enabled() || Replay_Active()) return 0;
    s = &cpu_assist_slots[slot];
    if (s->owner && s->owner != owner) return 0;
    gobj = Player_GetEntity(slot);
    fp = technical_ai_live(gobj) ? GET_FIGHTER(gobj) : NULL;
    if (!cpu_assist_supported(fp)) return 0;
    memset(s, 0, sizeof *s);
    s->fighter = fp; s->spawn = fp->x8_spawnNum; s->owner = owner; s->seed = (unsigned) seed;
    s->permille[CA_LCANCEL] = lc; s->permille[CA_PSHIELD] = ps; s->permille[CA_TECH] = tech;
    s->permille[CA_WAVEDASH] = wd; s->permille[CA_FASTFALL] = ff; s->tech_dir = tdir;
    s->prev_motion = -1;
    return 1;
}

int ScriptGame_CpuAssistClearSlot(int slot, int owner)
{
    if (slot < 0 || slot >= 6 || owner <= 0) return 0;
    if (cpu_assist_slots[slot].owner != owner) return 0;
    memset(&cpu_assist_slots[slot], 0, sizeof cpu_assist_slots[slot]);
    return 1;
}

void ScriptGame_CpuAssistClear(int owner)
{
    int i;
    for (i = 0; i < 6; ++i) if (owner == 0 || cpu_assist_slots[i].owner == owner)
        memset(&cpu_assist_slots[i], 0, sizeof cpu_assist_slots[i]);
}

/* field: 0 enabled, 1..5 permille (lcancel, perfect_shield, tech, wavedash, fast_fall), 6 tech_dir, 7 seed (current
 * generator state), 8..12 opportunities, 13..17 performed */
int ScriptGame_CpuAssistState(int slot, int owner, int field)
{
    CpuAssistSlot* s;
    if (slot < 0 || slot >= 6 || owner <= 0) return 0;
    s = &cpu_assist_slots[slot];
    if (s->owner != owner) return 0;
    if (field == 0) return 1;
    if (field >= 1 && field <= 5) return s->permille[field - 1];
    if (field == 6) return s->tech_dir;
    if (field == 7) return (int) s->seed;
    if (field >= 8 && field <= 12) return s->opportunities[field - 8];
    if (field >= 13 && field <= 17) return s->performed[field - 13];
    return 0;
}

/* The landing of a falling fighter along its own velocity; 1 and *eta (frames) when a floor lies within 12 frames. */
static int cpu_assist_eta(Fighter* fp, float* eta, float* floor_y_out)
{
    Vec3 floor, normal;
    float vx = fp->self_vel.x + fp->x8c_kb_vel.x, vy = fp->self_vel.y + fp->x8c_kb_vel.y, x, y;
    int line;
    unsigned flags;
    if (!(vy < -0.05f && vy > -100.0f && vx > -100.0f && vx < 100.0f)) return 0;
    x = fp->cur_pos.x + fp->coll_data.ecb.bottom.x;
    y = fp->cur_pos.y + fp->coll_data.ecb.bottom.y;
    if (!mpCheckFloor(x, y + 0.01f, x + vx * 12.0f, y + vy * 12.0f, 0.0f, &floor, &line, &flags, &normal,
                      fp->coll_data.floor_skip, fp->coll_data.joint_id_skip, fp->coll_data.joint_id_only, NULL,
                      fp->gobj)) return 0;
    *eta = (y - floor.y) / -vy;
    if (floor_y_out) *floor_y_out = floor.y;
    return *eta > 0.0f && *eta <= 12.0f;
}

static Fighter* cpu_assist_nearest_foe(Fighter* fp)
{
    Fighter* best = NULL;
    float bd = 1e30f;
    int i;
    for (i = 0; i < 6; ++i) {
        HSD_GObj* g;
        Fighter* o;
        float d;
        if (i == fp->player_id) continue;
        g = Player_GetEntity(i);
        if (!technical_ai_live(g)) continue;
        o = GET_FIGHTER(g);
        if (!o || o == fp || o->motion_id < ftCo_MS_Wait) continue;
        d = o->cur_pos.x - fp->cur_pos.x;
        if (d < 0) d = -d;
        if (d < bd) { bd = d; best = o; }
    }
    return best;
}

/* squared distance from point q to the segment a-b */
static float cpu_assist_seg_dist2(const Vec3* q, const Vec3* a, const Vec3* b)
{
    float abx = b->x - a->x, aby = b->y - a->y, abz = b->z - a->z;
    float apx = q->x - a->x, apy = q->y - a->y, apz = q->z - a->z;
    float len2 = abx * abx + aby * aby + abz * abz, t = 0.0f, dx, dy, dz;
    if (len2 > 1e-6f) {
        t = (apx * abx + apy * aby + apz * abz) / len2;
        if (t < 0.0f) t = 0.0f; else if (t > 1.0f) t = 1.0f;
    }
    dx = apx - abx * t; dy = apy - aby * t; dz = apz - abz * t;
    return dx * dx + dy * dy + dz * dz;
}

static struct { int foe, hb, hc, state, motion, frame; float d2, r, damage, px, py, ax, ay; } cpu_assist_last_threat;

/* A foe's hitbox, moved one more frame along its own sweep (x58 -> x4C, the last two collision positions), overlaps one
 * of this fighter's enabled hurtbox capsules (centre to capsule axis within the two radii). That is the collision
 * pass's own overlap test applied one frame ahead, so it names a hit that is about to land (or is already landing on
 * a multi-frame hitbox), not every hitbox that is merely near. */
static int cpu_assist_threat(Fighter* fp)
{
    int i, h, c;
    for (i = 0; i < 6; ++i) {
        HSD_GObj* g;
        Fighter* o;
        if (i == fp->player_id) continue;
        g = Player_GetEntity(i);
        if (!technical_ai_live(g)) continue;
        o = GET_FIGHTER(g);
        if (!o || o == fp) continue;
        for (h = 0; h < 4; ++h) {
            HitCapsule* hb = &o->x914[h];
            Vec3 p;
            if (hb->state == HitCapsule_Disabled || hb->damage <= 0.0f) continue;
            {   /* a hitbox that already hit this fighter in its current activation cannot hit again (the victim list) */
                int v, seen = 0;
                for (v = 0; v < 12; ++v)
                    if (hb->victims_1[v].victim == (void*) fp || hb->victims_1[v].victim == (void*) fp->gobj) seen = 1;
                if (seen) continue;
            }
            p.x = hb->x4C.x + (hb->x4C.x - hb->x58.x);
            p.y = hb->x4C.y + (hb->x4C.y - hb->x58.y);
            p.z = hb->x4C.z + (hb->x4C.z - hb->x58.z);
            for (c = 0; c < fp->hurt_capsules_len && c < 15; ++c) {
                FighterHurtCapsule* hc = &fp->hurt_capsules[c];
                float r = hb->scale + hc->capsule.scale;
                if (hc->capsule.state != HurtCapsule_Enabled) continue;
                if (cpu_assist_seg_dist2(&p, &hc->capsule.a_pos, &hc->capsule.b_pos) <= r * r) {
                    cpu_assist_last_threat.foe = i + 1; cpu_assist_last_threat.hb = h; cpu_assist_last_threat.hc = c;
                    cpu_assist_last_threat.state = (int) hb->state; cpu_assist_last_threat.motion = o->motion_id;
                    cpu_assist_last_threat.d2 = cpu_assist_seg_dist2(&p, &hc->capsule.a_pos, &hc->capsule.b_pos);
                    cpu_assist_last_threat.r = r; cpu_assist_last_threat.damage = hb->damage;
                    cpu_assist_last_threat.px = p.x; cpu_assist_last_threat.py = p.y;
                    cpu_assist_last_threat.ax = hc->capsule.a_pos.x; cpu_assist_last_threat.ay = hc->capsule.a_pos.y;
                    return 1;
                }
            }
        }
    }
    return 0;
}

/* PS diagnostics (the follow-up log after a shield press): each enabled foe hitbox against this fighter's hurtboxes, now and one frame ahead */
static void cpu_assist_dump_hits(Fighter* fp)
{
    int i, h, c;
    for (i = 0; i < 6; ++i) {
        HSD_GObj* g;
        Fighter* o;
        if (i == fp->player_id) continue;
        g = Player_GetEntity(i);
        if (!technical_ai_live(g)) continue;
        o = GET_FIGHTER(g);
        if (!o || o == fp) continue;
        for (h = 0; h < 4; ++h) {
            HitCapsule* hb = &o->x914[h];
            float gap_now = 1e9f, gap_next = 1e9f;
            Vec3 p;
            int vict = 0, v;
            if (hb->state == HitCapsule_Disabled) continue;
            p.x = hb->x4C.x + (hb->x4C.x - hb->x58.x); p.y = hb->x4C.y + (hb->x4C.y - hb->x58.y); p.z = hb->x4C.z + (hb->x4C.z - hb->x58.z);
            for (c = 0; c < fp->hurt_capsules_len && c < 15; ++c) {
                FighterHurtCapsule* hc = &fp->hurt_capsules[c];
                float r = hb->scale + hc->capsule.scale, d;
                if (hc->capsule.state != HurtCapsule_Enabled) continue;
                d = cpu_assist_seg_dist2(&hb->x4C, &hc->capsule.a_pos, &hc->capsule.b_pos) - r * r;
                if (d < gap_now) gap_now = d;
                d = cpu_assist_seg_dist2(&p, &hc->capsule.a_pos, &hc->capsule.b_pos) - r * r;
                if (d < gap_next) gap_next = d;
            }
            for (v = 0; v < 12; ++v) if (hb->victims_1[v].victim != NULL) ++vict;
            OSReport("CPU-ASSIST:     foe P%d hb%d state=%d dmg=%f scale=%f gap_now=%f gap_next=%f victims=%d foe_motion=%d hitlag=%f\n", i + 1, h, (int) hb->state, (double) hb->damage, (double) hb->scale, (double) gap_now, (double) gap_next, vict, o->motion_id, (double) o->dmg.x195c_hitlag_frames);
        }
    }
}

static int cpu_assist_air_actionable(int m)
{
    return (m >= ftCo_MS_JumpF && m <= ftCo_MS_FallAerial) ||
           (m >= ftCo_MS_AttackAirN && m <= ftCo_MS_AttackAirLw);
}

static void cpu_assist_overlay(Fighter* fp)
{
    CpuAssistSlot* s;
    int m, fresh, air, sub = 0, from;
    float eta = 0.0f;
    if (fp->player_id >= 6 || fp->is_sub_fighter) return;
    s = &cpu_assist_slots[fp->player_id];
    if (!s->owner) return;
    if (Netplay_Enabled() || RB_Enabled() || Replay_Active() || !cpu_assist_supported(fp)) {
        memset(s, 0, sizeof *s);
        return;
    }
    if (s->fighter != fp || s->spawn != fp->x8_spawnNum) {
        /* A new stock or a replaced entity: the configuration, the generator and the counters stay (they belong to the
         * slot), the per-instance state starts over. */
        s->fighter = fp; s->spawn = fp->x8_spawnNum; s->prev_motion = -1;
        s->lc_state = s->tech_state = s->tech_hold = s->ps_hold = s->ps_state = s->ps_gap = 0;
        s->wd_state = s->wd_hold = s->ff_state = s->ff_tries = 0;
    }
    if (gm_GetDbPauseFlag(0) || gm_GetDbPauseFlag(2)) return;
    if (fp->dmg.x195c_hitlag_frames > 0.0f) return;
    m = fp->motion_id;
    air = fp->ground_or_air == GA_Air;
    ++s->frames;
    from = s->prev_motion;
    if (m != s->prev_motion) { s->prev_motion = m; s->motion_age = 1; } else if (s->motion_age < 1000) ++s->motion_age;
    fresh = !(fp->input.held_buttons[1] & HSD_PAD_LR) && !(fp->input.held_buttons[0] & (HSD_PAD_L | HSD_PAD_R | HSD_PAD_Z)) &&
            fp->input.triggers[0] == 0.0f;

    /* ---- L-cancel ---- */
    if (air && m >= ftCo_MS_AttackAirN && m <= ftCo_MS_AttackAirLw && fp->cmd_vars[0] && !fp->allow_interrupt &&
        cpu_assist_eta(fp, &eta, NULL)) {
        if (s->lc_state == 0 && eta <= (float) CA_LC_ETA + 1.0f) s->lc_state = cpu_assist_decide(s, CA_LCANCEL, NULL) ? 2 : 1;
        if (s->lc_state == 2 && eta <= (float) CA_LC_ETA && fresh) {
            fp->input.triggers[0] = 0.35f;
            s->lc_state = 3;
            ++s->performed[CA_LCANCEL];
            CA_LOG(s, fp, "lcancel");
        }
    } else if (!air || m < ftCo_MS_AttackAirN || m > ftCo_MS_AttackAirLw) {
        s->lc_state = 0;
    }

    /* ---- tech ---- */
    if (air && m >= ftCo_MS_DamageFlyHi && m <= ftCo_MS_DamageFlyRoll && fp->x221C_b6 && cpu_assist_eta(fp, &eta, NULL)) {
        if (s->tech_state == 0 && eta <= (float) CA_TECH_ETA + 1.0f) {
            if (cpu_assist_decide(s, CA_TECH, &sub)) {
                int d = s->tech_dir;
                if (d == CA_DIR_RANDOM) d = sub < 333 ? CA_DIR_IN : sub < 667 ? CA_DIR_AWAY : CA_DIR_TOWARD;
                if (d == CA_DIR_IN) s->tech_x = 0;
                else {
                    Fighter* foe = cpu_assist_nearest_foe(fp);
                    float toward = foe ? (foe->cur_pos.x >= fp->cur_pos.x ? 1.0f : -1.0f) : fp->facing_dir;
                    s->tech_x = (d == CA_DIR_TOWARD ? toward : -toward) > 0.0f ? 1 : -1;
                }
                s->tech_state = 2;
            } else {
                s->tech_state = 1;
            }
        }
        if (s->tech_state == 2 && eta <= (float) CA_TECH_ETA && fresh && fp->x680 >= p_ftCommonData->x1C) {
            fp->input.held_buttons[0] |= HSD_PAD_R;
            s->tech_hold = (int) (p_ftCommonData->x250 + 0.5f);
            s->tech_state = 3;
            ++s->performed[CA_TECH];
            CA_LOG(s, fp, "tech");
        }
    } else if (!air || m < ftCo_MS_DamageFlyHi || m > ftCo_MS_DamageFlyRoll) {
        if (s->tech_state != 0 && !(m >= ftCo_MS_DamageFlyHi && m <= ftCo_MS_DamageFlyRoll)) s->tech_state = 0;
    }
    if (s->tech_hold > 0) {
        /* the game reads the stick when it lands the fighter, up to its tech window after the press */
        fp->input.lstick[0].x = (float) s->tech_x;
        fp->input.lstick[0].y = 0.0f;
        --s->tech_hold;
    }

    /* ---- perfect shield ---- */
    if (s->ps_log > 0) {
        --s->ps_log;
        OSReport("CPU-ASSIST: P%d ps+%d motion=%d kb=%f ps_window=%d guard=%f\n", fp->player_id + 1, 3 - s->ps_log, m, (double) fp->dmg.kb_applied, (int) fp->x221C_b2, (double) fp->mv.co.guard.x18);
        cpu_assist_dump_hits(fp);
    }
    if (s->ps_hold > 0) {
        fp->input.held_buttons[0] |= HSD_PAD_R;
        --s->ps_hold;
    } else if (!air && m >= ftCo_MS_Wait && m <= ftCo_MS_WalkFast) {
        if (cpu_assist_threat(fp)) {
            if (s->ps_state == 0) s->ps_state = cpu_assist_decide(s, CA_PSHIELD, NULL) ? 2 : 1;
            if (s->ps_state == 2 && fresh) {
                fp->input.held_buttons[0] |= HSD_PAD_R;
                s->ps_hold = CA_PS_HOLD - 1;
                s->ps_state = 3;
                ++s->performed[CA_PSHIELD];
                CA_LOG(s, fp, "perfect_shield");
                OSReport("CPU-ASSIST:   threat foe=P%d hitbox=%d hurt=%d state=%d foe_motion=%d d2=%f r=%f dmg=%f hit=(%f,%f) hurt_a=(%f,%f)\n", cpu_assist_last_threat.foe, cpu_assist_last_threat.hb, cpu_assist_last_threat.hc, cpu_assist_last_threat.state, cpu_assist_last_threat.motion, (double) cpu_assist_last_threat.d2, (double) cpu_assist_last_threat.r, (double) cpu_assist_last_threat.damage, (double) cpu_assist_last_threat.px, (double) cpu_assist_last_threat.py, (double) cpu_assist_last_threat.ax, (double) cpu_assist_last_threat.ay);
                s->ps_log = 4;
            }
            s->ps_gap = 0;
        } else if (++s->ps_gap > 8) {
            s->ps_state = 0;
        }
    } else if (++s->ps_gap > 8) {
        s->ps_state = 0;
    }

    /* ---- wavedash ----
     * The jump begins on the frame the AI presses it (the action turns to KneeBend), the jumpsquat runs its frames, and the
     * air dodge is read on the FIRST input frame of the jump action (JumpF/JumpB) as a fresh L/R press: pressing inside the
     * jumpsquat is lost, and a press held across the change is not an edge. The decision is made once per jumpsquat. */
    if (s->wd_hold > 0) {
        fp->input.lstick[0].x = (float) s->wd_dir * CA_WD_ANGLE_X;
        fp->input.lstick[0].y = CA_WD_ANGLE_Y;
        if (s->wd_hold >= 2) fp->input.held_buttons[0] |= HSD_PAD_R;
        --s->wd_hold;
    } else if (m == ftCo_MS_KneeBend && !air) {
        if (s->motion_age == 1) {
            float lx = fp->input.lstick[0].x, fx;
            Fighter* foe = cpu_assist_nearest_foe(fp);
            Vec3 floor; int line; unsigned flags; Vec3 normal;
            s->wd_state = cpu_assist_decide(s, CA_WAVEDASH, NULL) ? 2 : 1;
            if (s->wd_state == 2) {
                fx = lx > 0.3f ? 1.0f : lx < -0.3f ? -1.0f : (foe ? (foe->cur_pos.x >= fp->cur_pos.x ? 1.0f : -1.0f) : fp->facing_dir);
                s->wd_dir = fx > 0.0f ? 1 : -1;
                /* room: floor under the point 12 units ahead (a wavedash slides) */
                if (!mpCheckFloor(fp->cur_pos.x + fx * 12.0f, fp->cur_pos.y + 4.0f, fp->cur_pos.x + fx * 12.0f,
                                  fp->cur_pos.y - 6.0f, 0.0f, &floor, &line, &flags, &normal, -1, -1, -1, NULL, NULL))
                    s->wd_state = 1;
            }
        }
    } else if (s->wd_state == 2 && from == ftCo_MS_KneeBend && (m == ftCo_MS_JumpF || m == ftCo_MS_JumpB) && s->motion_age == 1) {
        s->wd_state = 3;
        s->wd_hold = 3;
        fp->input.lstick[0].x = (float) s->wd_dir * CA_WD_ANGLE_X;
        fp->input.lstick[0].y = CA_WD_ANGLE_Y;
        fp->input.held_buttons[0] |= HSD_PAD_R;
        --s->wd_hold;
        ++s->performed[CA_WAVEDASH];
        CA_LOG(s, fp, "wavedash");
    } else if (m != ftCo_MS_JumpF && m != ftCo_MS_JumpB) {
        s->wd_state = 0;
    }

    /* ---- fast fall ---- */
    if (!air) {
        s->ff_state = 0; s->ff_tries = 0;
    } else if (cpu_assist_air_actionable(m) && fp->self_vel.y < 0.0f && !fp->fall_fast) {
        if (s->ff_state == 0) s->ff_state = cpu_assist_decide(s, CA_FASTFALL, NULL) ? 2 : 1;
        if (s->ff_state == 2 && s->ff_tries < 4 && fp->input.lstick[0].y > -p_ftCommonData->x88) {
            fp->input.lstick[0].y = -1.0f;
            if (s->ff_tries++ == 0) { ++s->performed[CA_FASTFALL]; CA_LOG(s, fp, "fast_fall"); }
        }
    }
}

/* Disc-free native checks of the pure parts; nonzero reports a failed case. */
int ScriptGame_CpuAssistTest(void)
{
    CpuAssistSlot a, b, prior = cpu_assist_slots[5];
    int errors = 0, i, hits, sub = 0;
    memset(&a, 0, sizeof a); memset(&b, 0, sizeof b);
    a.seed = b.seed = 12345u;
    /* probability 0 never performs, 1000 always does, and each counts every opportunity */
    for (i = 0; i < 500; ++i) {
        a.permille[CA_TECH] = 0; if (cpu_assist_decide(&a, CA_TECH, NULL)) ++errors;
        a.permille[CA_LCANCEL] = 1000; if (!cpu_assist_decide(&a, CA_LCANCEL, NULL)) ++errors;
    }
    if (a.opportunities[CA_TECH] != 500 || a.opportunities[CA_LCANCEL] != 500) ++errors;
    /* the same seed gives the same sequence; a different seed does not */
    a.seed = 777u; b.seed = 777u;
    a.permille[CA_PSHIELD] = b.permille[CA_PSHIELD] = 500;
    for (i = 0; i < 200; ++i) if (cpu_assist_roll(&a) != cpu_assist_roll(&b)) { ++errors; break; }
    a.seed = 777u; b.seed = 778u;
    for (i = 0, hits = 0; i < 50; ++i) hits += cpu_assist_roll(&a) == cpu_assist_roll(&b);
    if (hits > 10) ++errors;
    /* a half probability performs about half of the time (xorshift32, 20000 draws) */
    a.seed = 99u; a.permille[CA_WAVEDASH] = 500; a.opportunities[CA_WAVEDASH] = 0;
    for (i = 0, hits = 0; i < 20000; ++i) hits += cpu_assist_decide(&a, CA_WAVEDASH, &sub);
    if (hits < 9500 || hits > 10500 || a.opportunities[CA_WAVEDASH] != 20000) ++errors;
    /* the reused draw is uniform over the accepted share */
    a.seed = 5u; a.permille[CA_TECH] = 250;
    for (i = 0; i < 4000; ++i) if (cpu_assist_decide(&a, CA_TECH, &sub) && (sub < 0 || sub > 999)) ++errors;
    /* validation */
    if (cpu_assist_config_ok(-1, 0, 0, 0, 0, 0) || cpu_assist_config_ok(0, 1001, 0, 0, 0, 0) ||
        cpu_assist_config_ok(0, 0, 0, 4, 0, 0) || !cpu_assist_config_ok(1000, 1000, 1000, CA_DIR_RANDOM, 1000, 1000)) ++errors;
    /* ownership: a clear by another owner changes nothing */
    memset(&cpu_assist_slots[5], 0, sizeof cpu_assist_slots[5]);
    cpu_assist_slots[5].owner = 723; cpu_assist_slots[5].permille[CA_TECH] = 400;
    ScriptGame_CpuAssistClear(724);
    if (ScriptGame_CpuAssistState(5, 723, 3) != 400 || ScriptGame_CpuAssistState(5, 724, 3) != 0) ++errors;
    if (ScriptGame_CpuAssistClearSlot(5, 724) || !ScriptGame_CpuAssistClearSlot(5, 723) || cpu_assist_slots[5].owner) ++errors;
    cpu_assist_slots[5] = prior;
    return errors;
}
