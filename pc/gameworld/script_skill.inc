/* Skill telemetry (included by script_game.c after script_fighter_caps_effects.inc).
 *
 * Read-only observers of the game's own decisions, never of raw inputs: the L-cancel decision is read where retail
 * makes it (ftCo_LandingAir_EnterWithLag), hits where the collision pass accepts them (ftcoll.c), the powershield
 * window where retail tests it (ftcoll.c); everything else is a transition of the fighter's own motion state, ground
 * flag, fast-fall flag, hitlag flag and position between two logic frames (ScriptGame_SkillFrame runs from
 * ScriptGame_StageFrame, after the fighters, the same place the armour and effect timers tick).
 *
 * Nothing here writes a fighter. Every record, the ring and the frame counter live in this translation unit's
 * snapshotted BSS, so a rewind or a rollback restores them exactly and the resimulated frames reproduce the same
 * rows. Script events are queued by the native side (Script_SkillEvent) and skipped while resimulating; the ring is
 * appended either way, so it always equals the live timeline. */
#include "script_skill.h"
extern void Script_SkillEvent(int kind, int entity, int i1, int i2, int i3, int f1_bits, int f2_bits);

enum {
    SK_LCANCEL = 1, SK_LCANCEL_MISS, SK_AUTO_CANCEL, SK_LCANCEL_HIT, SK_WAVEDASH, SK_WAVELAND, SK_LEDGE_DASH,
    SK_AIR_DODGE, SK_PERFECT_SHIELD, SK_TECH, SK_TECH_MISS, SK_DASH_DANCE, SK_SHORT_HOP, SK_FULL_HOP, SK_FAST_FALL,
    SK_SHIELD_DROP, SK_JC_GRAB, SK_JC_USMASH, SK_SDI, SK_COMBO, SK_COMBO_END, SK_SHOCK, SK_SHOCK_HIT, SK_SHOCK_END, SK_KINDS
};
/* Thresholds (reported by gd.skill_thresholds()). */
#define SK_WAVE_MIN_SPEED 0.25f       /* |ground speed| on the landing frame of an air dodge, world units per frame (measured Fox: wavedash 2.62, steep waveland 0.41/0.27, a decayed dodge 0) */
#define SK_WAVEDASH_MAX_AIRBORNE 4    /* the air dodge began within this many frames of the jump leaving the ground */
#define SK_LEDGE_DASH_FRAMES 50       /* the air dodge landing came within this many frames of leaving the ledge */
#define SK_DASH_DANCE_FRAMES 24       /* a dash re-entered the other way within this many frames of the last one */
#define SK_RING 256

typedef struct { int frame, kind, entity, i1, i2, i3; float f1, f2; } ScriptSkillRow;
typedef struct {
    Fighter* fighter;
    int motion, age, ground, air_frames;
    int jump_frame, dodge_frame, dodge_since_jump, cliff_frame, dash_frame, dash_dir, dash_run, shield_frame;
    int fall_fast, hitlag, sdi_count;
    int aerial_hit;              /* the current aerial has connected */
    int combo_count, combo_attacker, combo_last;
    float combo_damage, px, py;
} ScriptSkillRec;
static ScriptSkillRec script_skill_rec[12];
static ScriptSkillRow script_skill_ring[SK_RING];
static int script_skill_head, script_skill_n, script_skill_frame;

static void script_crit_flush(void);
static void script_shock_frame(void);
static int script_skill_bits(float f) { union { float f; int i; } u; u.f = f; return u.i; }
static float script_skill_fabs(float f) { return f < 0 ? -f : f; }
static void script_skill_emit(int kind, int entity, int i1, int i2, int i3, float f1, float f2)
{
    ScriptSkillRow* r = &script_skill_ring[script_skill_head % SK_RING];
    r->frame = script_skill_frame; r->kind = kind; r->entity = entity;
    r->i1 = i1; r->i2 = i2; r->i3 = i3; r->f1 = f1; r->f2 = f2;
    script_skill_head++; if (script_skill_n < SK_RING) script_skill_n++;
    {   /* online Envoy stage 4: the native triggered rules take the row here, in the simulation (the host queue below is skipped while resimulating) */
        extern void ScriptMods_Skill(int, int, int, int, int, float, float, int, int);
        extern void ScriptMods_ShockEnd(int, int);
        if (kind == SK_SHOCK_END) ScriptMods_ShockEnd(entity, i1);
        else if (entity >= 1 && entity <= 12) ScriptMods_Skill(kind, entity, i1, i2, i3, f1, f2, script_skill_rec[entity-1].air_frames, script_skill_rec[entity-1].aerial_hit);
    }
    Script_SkillEvent(kind, entity, i1, i2, i3, script_skill_bits(f1), script_skill_bits(f2));
}
static ScriptSkillRec* script_skill_of(Fighter* fp, int* entity)
{
    int e = ScriptGame_CapsEntity(fp);
    if (entity) *entity = e;
    if (e < 1) return NULL;
    return script_skill_rec[e-1].fighter == fp ? &script_skill_rec[e-1] : NULL;
}
static int script_skill_aerial(int motion)
{
    return motion >= ftCo_MS_AttackAirN && motion <= ftCo_MS_AttackAirLw ? motion - ftCo_MS_AttackAirN : -1;
}

void ScriptGame_SkillLanding(Fighter* fp, int mode, float lag, float lag_final)
{
    int e, aerial = script_skill_aerial(fp->motion_id), hit;
    ScriptSkillRec* r = script_skill_of(fp, &e);
    if (!r || aerial < 0) return;
    hit = r->aerial_hit ? 1 : 0;
    if (mode < 0) { script_skill_emit(SK_AUTO_CANCEL, e, aerial, hit, 0, 0, 0); return; }
    /* i1 aerial 0 nair 1 fair 2 bair 3 uair 4 dair, i2 an hitbox of this aerial connected, i3 frames since the
     * trigger was pressed (retail's own timer, compared with the window), f1 normal lag, f2 lag after the cancel */
    script_skill_emit(mode ? SK_LCANCEL : SK_LCANCEL_MISS, e, aerial, hit, (int) fp->x67F, lag, lag_final);
    if (mode && hit) script_skill_emit(SK_LCANCEL_HIT, e, aerial, 1, (int) fp->x67F, lag, lag_final);
}

void ScriptGame_SkillPerfectShield(Fighter* shielder, Fighter* attacker, int projectile, float damage)
{
    int e, a = attacker ? ScriptGame_CapsEntity(attacker) : 0;
    (void) script_skill_of(shielder, &e);
    if (e < 1) return;
    /* i1 projectile, i2 attacking entity (0 none), i3 powershield frames already used, f1 damage */
    script_skill_emit(SK_PERFECT_SHIELD, e, projectile ? 1 : 0, a,
                      p_ftCommonData ? p_ftCommonData->x2B4 - shielder->mv.co.guard.x18 - 1 : 0, damage, 0);
}

void ScriptGame_SkillHit(Fighter* atk, Fighter* vic, float damage)
{
    int ve, ae = atk ? ScriptGame_CapsEntity(atk) : 0;
    ScriptSkillRec* v = script_skill_of(vic, &ve);
    ScriptSkillRec* a = atk ? script_skill_of(atk, NULL) : NULL;
    if (a && script_skill_aerial(atk->motion_id) >= 0) a->aerial_hit = 1;
    if (!v || damage <= 0) return;
    if (v->combo_count <= 0) { v->combo_count = 0; v->combo_damage = 0; }
    v->combo_count++; v->combo_damage += damage; v->combo_attacker = ae; v->combo_last = script_skill_frame;
    /* i1 hits so far (1 = the opener), i2 attacker entity, f1 combo damage, f2 this hit */
    script_skill_emit(SK_COMBO, ve, v->combo_count, ae, 0, v->combo_damage, damage);
}

static int script_skill_roll_dir(Fighter* fp, int e, int forward)
{
    int o = ScriptGame_CapsOpponent(e, script_skill_bits(100000.0f), 0, 0);
    Fighter* t = o ? ScriptGame_CapsFighter(o) : NULL;
    float dir = (forward ? 1.0f : -1.0f) * fp->facing_dir;
    if (!t) return 2;
    return ((t->cur_pos.x - fp->cur_pos.x) * dir > 0) ? 1 : 2; /* 1 toward the nearest opponent, 2 away */
}

static void script_skill_one(int e, Fighter* fp, ScriptSkillRec* r)
{
    int m = fp->motion_id, prev = r->motion, prev_age = r->age, changed = m != prev, frame = script_skill_frame;
    int hitlag = fp->x2219_b5 ? 1 : 0;
    if (changed) {
        r->motion = m; r->age = 0;
        if (m == ftCo_MS_KneeBend) { /* jumpsquat: nothing to report until it resolves */ }
        if ((m == ftCo_MS_JumpF || m == ftCo_MS_JumpB) && prev == ftCo_MS_KneeBend) {
            float vy = fp->self_vel.y, hop = fp->co_attrs.hop_v_initial_velocity, full = fp->co_attrs.jump_v_initial_velocity;
            int is_short = vy < (hop + full) * 0.5f;
            r->jump_frame = frame;
            /* i1 jumpsquat frames, f1 vertical speed on the first airborne frame, f2 the fighter's own hop/full speeds
             * (hop for a short hop, full otherwise) so a script can see the margin */
            script_skill_emit(is_short ? SK_SHORT_HOP : SK_FULL_HOP, e, prev_age + 1, 0, 0, vy, is_short ? hop : full);
        }
        if (m == ftCo_MS_EscapeAir ||
            (m == ftCo_MS_LandingFallSpecial && (prev == ftCo_MS_KneeBend || prev == ftCo_MS_JumpF || prev == ftCo_MS_JumpB))) {
            /* The air dodge may begin and land inside one logic frame (a perfect wavedash): the observer then sees
             * jumpsquat/jump -> landing and never the dodge, so that transition reports the dodge too. */
            int since = prev == ftCo_MS_KneeBend ? 0 :
                        (prev == ftCo_MS_JumpF || prev == ftCo_MS_JumpB) ? prev_age + 1 :
                        (frame - r->jump_frame <= 60) ? frame - r->jump_frame : -1;
            r->dodge_frame = frame; r->dodge_since_jump = since;
            /* i1 frames since a ground jump left the floor (-1: not within 60), f1/f2 stick */
            script_skill_emit(SK_AIR_DODGE, e, since, 0, 0, fp->input.lstick[0].x, fp->input.lstick[0].y);
        }
        if (m == ftCo_MS_LandingFallSpecial &&
            (prev == ftCo_MS_EscapeAir || prev == ftCo_MS_KneeBend || prev == ftCo_MS_JumpF || prev == ftCo_MS_JumpB)) {
            float speed = fp->gr_vel, mag = script_skill_fabs(speed);
            if (mag >= SK_WAVE_MIN_SPEED) {
                int since = r->dodge_since_jump;
                int dash = since >= 0 && since <= SK_WAVEDASH_MAX_AIRBORNE;
                int dodge_frames = prev == ftCo_MS_EscapeAir ? prev_age + 1 : 1;
                /* i1 frames from the jump leaving the floor to the air dodge, i2 air dodge frames before the landing,
                 * f1 signed ground speed, f2 its magnitude */
                script_skill_emit(dash ? SK_WAVEDASH : SK_WAVELAND, e, since, dodge_frames, 0, speed, mag);
                if (frame - r->cliff_frame <= SK_LEDGE_DASH_FRAMES)
                    script_skill_emit(SK_LEDGE_DASH, e, frame - r->cliff_frame, dodge_frames, 0, speed, mag);
            } else OSReport("skill: air dodge landing below the wave threshold, speed x1000 = %d\n", (int) (speed * 1000.0f));
        }
        if (m == ftCo_MS_Dash) {
            float dir = fp->facing_dir;
            if (frame - r->dash_frame <= SK_DASH_DANCE_FRAMES && dir != (float) r->dash_dir) {
                r->dash_run++;
                script_skill_emit(SK_DASH_DANCE, e, r->dash_run, frame - r->dash_frame, 0, 0, 0);
            } else r->dash_run = 0;
            r->dash_frame = frame; r->dash_dir = dir < 0 ? -1 : 1;
        }
        if (m == ftCo_MS_Passive || m == ftCo_MS_PassiveStandF || m == ftCo_MS_PassiveStandB ||
            m == ftCo_MS_PassiveWall || m == ftCo_MS_PassiveWallJump || m == ftCo_MS_PassiveCeil) {
            /* i1 0 in place, 1 toward the nearest opponent, 2 away, 3 wall, 4 ceiling; i2 1 for a roll, i3 1 if wall jump */
            int type = m == ftCo_MS_Passive ? 0 :
                       m == ftCo_MS_PassiveWall || m == ftCo_MS_PassiveWallJump ? 3 :
                       m == ftCo_MS_PassiveCeil ? 4 : script_skill_roll_dir(fp, e, m == ftCo_MS_PassiveStandF);
            script_skill_emit(SK_TECH, e, type, (m == ftCo_MS_PassiveStandF || m == ftCo_MS_PassiveStandB), m == ftCo_MS_PassiveWallJump, 0, 0);
        }
        if ((m == ftCo_MS_DownBoundU || m == ftCo_MS_DownBoundD) &&
            prev != ftCo_MS_DownBoundU && prev != ftCo_MS_DownBoundD)
            script_skill_emit(SK_TECH_MISS, e, m == ftCo_MS_DownBoundD, 0, 0, 0, 0); /* i1 1 face down */
        if (m == ftCo_MS_GuardOn) r->shield_frame = frame;
        if (m == ftCo_MS_Pass && (prev == ftCo_MS_Guard || prev == ftCo_MS_GuardOn || prev == ftCo_MS_GuardSetOff))
            script_skill_emit(SK_SHIELD_DROP, e, frame - r->shield_frame, 0, 0, 0, 0);
        if (prev == ftCo_MS_KneeBend && (m == ftCo_MS_Catch || m == ftCo_MS_CatchDash))
            script_skill_emit(SK_JC_GRAB, e, prev_age + 1, 0, 0, 0, 0);
        if (prev == ftCo_MS_KneeBend && m == ftCo_MS_AttackHi4)
            script_skill_emit(SK_JC_USMASH, e, prev_age + 1, 0, 0, 0, 0);
        if (m == ftCo_MS_CliffWait || m == ftCo_MS_CliffCatch) r->cliff_frame = frame;
        if ((prev == ftCo_MS_CliffWait || prev == ftCo_MS_CliffCatch) &&
            m != ftCo_MS_CliffWait && m != ftCo_MS_CliffCatch) r->cliff_frame = frame; /* the frame it left the ledge */
        if (script_skill_aerial(m) < 0) r->aerial_hit = 0;
    } else r->age++;
    /* Air time and fast fall (retail's own flag). */
    if (fp->ground_or_air == GA_Ground) r->air_frames = 0; else r->air_frames++;
    if (fp->fall_fast && !r->fall_fast && fp->ground_or_air != GA_Ground)
        script_skill_emit(SK_FAST_FALL, e, r->air_frames, 0, 0, fp->self_vel.y, 0);
    r->fall_fast = fp->fall_fast ? 1 : 0;
    /* Smash DI: a fighter in hitlag does not otherwise move. */
    if (hitlag && r->hitlag) {
        float dx = fp->cur_pos.x - r->px, dy = fp->cur_pos.y - r->py;
        if (dx != 0 || dy != 0) {
            r->sdi_count++;
            script_skill_emit(SK_SDI, e, r->sdi_count, (int) fp->dmg.x195c_hitlag_frames, 0, dx, dy);
        }
    }
    if (!hitlag) r->sdi_count = 0;
    r->hitlag = hitlag; r->px = fp->cur_pos.x; r->py = fp->cur_pos.y;
    /* A combo lasts while the victim stays in hitstun or hitlag; a KO ends it. */
    if (r->combo_count > 0) {
        int ko = m < ftCo_MS_Wait;
        if (!(fp->x221C_b6 || hitlag) && frame > r->combo_last + 1) ko = ko | 2;
        if (ko) {
            script_skill_emit(SK_COMBO_END, e, r->combo_count, r->combo_attacker, (ko & 1) ? 1 : 0, r->combo_damage, 0);
            r->combo_count = 0; r->combo_damage = 0; r->combo_attacker = 0;
        }
    }
}

void ScriptGame_SkillFrame(void)
{
    int e;
    ++script_skill_frame;
    for (e = 1; e <= 12; ++e) {
        Fighter* fp = ScriptGame_CapsFighter(e);
        ScriptSkillRec* r = &script_skill_rec[e-1];
        if (!fp) { if (r->fighter) memset(r, 0, sizeof *r); continue; }
        if (r->fighter != fp) {
            memset(r, 0, sizeof *r);
            r->fighter = fp; r->motion = fp->motion_id; r->ground = fp->ground_or_air;
            r->jump_frame = r->dodge_frame = r->cliff_frame = r->dash_frame = -10000;
            r->px = fp->cur_pos.x; r->py = fp->cur_pos.y; r->fall_fast = fp->fall_fast ? 1 : 0;
            continue;
        }
        script_skill_one(e, fp, r);
    }
    script_shock_frame();
    script_crit_flush();
}
void ScriptGame_SkillReset(void)
{
    memset(script_skill_rec, 0, sizeof script_skill_rec);
    memset(script_skill_ring, 0, sizeof script_skill_ring);
    script_skill_head = script_skill_n = 0;
}
/* Ring access for the native side: idx 0 is the newest row. Floats come back as bit patterns. */
int ScriptGame_SkillRowCount(void) { return script_skill_n; }
int ScriptGame_SkillRow(int idx, int field)
{
    const ScriptSkillRow* r;
    if (idx < 0 || idx >= script_skill_n) return 0;
    r = &script_skill_ring[(script_skill_head - 1 - idx + SK_RING * 2) % SK_RING];
    switch (field) {
    case 0: return r->frame; case 1: return r->kind; case 2: return r->entity; case 3: return r->i1;
    case 4: return r->i2; case 5: return r->i3; case 6: return script_skill_bits(r->f1);
    case 7: return script_skill_bits(r->f2);
    }
    return 0;
}
/* Per-entity state: 0 combo count, 1 combo damage (bits), 2 combo attacker, 3 air frames, 4 frames in motion, 5 aerial hit. */
int ScriptGame_SkillState(int entity, int field)
{
    ScriptSkillRec* r;
    if (entity < 1 || entity > 12) return 0;
    r = &script_skill_rec[entity-1];
    if (!r->fighter || r->fighter != ScriptGame_CapsFighter(entity)) return 0;
    switch (field) {
    case 0: return r->combo_count; case 1: return script_skill_bits(r->combo_damage); case 2: return r->combo_attacker;
    case 3: return r->air_frames; case 4: return r->age; case 5: return r->aerial_hit;
    }
    return 0;
}
/* Thresholds: 0 wave speed (bits), 1 wavedash airborne frames, 2 ledge dash frames, 3 dash dance frames,
 * 4 L-cancel window (retail common data), 5 powershield window (retail common data). */
int ScriptGame_SkillConst(int idx)
{
    switch (idx) {
    case 0: return script_skill_bits(SK_WAVE_MIN_SPEED); case 1: return SK_WAVEDASH_MAX_AIRBORNE;
    case 2: return SK_LEDGE_DASH_FRAMES; case 3: return SK_DASH_DANCE_FRAMES;
    case 4: return p_ftCommonData ? p_ftCommonData->xE4 : 0;
    case 5: return p_ftCommonData ? p_ftCommonData->x2B4 : 0;
    }
    return 0;
}


/* Route stages, read-only (the Adventure stages gd.stage_objectives() lists route markers for).
 * Underground Maze (Gr_Kind_ShrineRoute, src/melee/gr/grshrineroute.c; NOT grkinokoroute.c, which is Mushroom Kingdom):
 * map points 0xBD..0xC2 are six SPOTS; the stage places a symbol on each (fn_80208A38: one random spot gets the exit
 * symbol, map_id 3, the other five an event-room symbol, map_id 1). Each frame Ground_801C3DB4 tests the first player's x
 * against a (x1C * scale) wide box around each point (the y test is the stage's own tall box); the first point inside that
 * passes the stage predicate and is not already done (xC6 bit) becomes stage_info.x720 and, in phase 0, is taken: an exit
 * spot sets phase 6 and stage_info.flags |= 0x10 (the route is complete, the stage ends); an event-room spot starts that room
 * (phase 1-3, xC8 = the point id, the done bit set when the event ends). 0xC3..0xC6 are scanned by Ground_801C0C2C but
 * the stage has no such points (they read as absent).
 * F-Zero Grand Prix (Gr_Kind_BigBlueRoute, grbigblueroute.c): map points 5,6,7 are the checkpoints (the first player's x
 * passing one advances the stage's counter, gp+0xC8, mirrored in stage_info.x6DC); the finish is the usual route box over
 * points 0x99..0xB2 (Ground_801C3D44(0, 30, 4000): candidate stage_info.x714, latched stage_info.x6D0 when flags & 0x10).
 * group 0 (header): field 0 kind (0 none, 1 maze, 2 fzero), 1 candidate point id (-1 none), 2 latched/taken id (-1 none),
 * 3 maze phase / fzero checkpoints passed, 4 maze done mask, 5 maze: exit chosen (stage_info.flags & 0x10).
 * group 1 (main points: maze 0xBD+idx idx 0..9, fzero 0x99+idx idx 0..25), group 2 (fzero checkpoints 5,6,7):
 * field 0 present, 1/2/3 x/y/z bits, 4 point id, 5 maze: symbol kind (3 exit, 1 room, -1 none), 6 maze: done. */
extern int grShrineRoute_ScriptQuery(int field, int idx);
int ScriptGame_ObjectiveRoute(int group, int idx, int field)
{
    int kind = stage_info.grkind == Gr_Kind_ShrineRoute ? 1 : stage_info.grkind == Gr_Kind_BigBlueRoute ? 2 : 0;
    if (group == 0) {
        int ph;
        switch (field) {
        case 0: return kind;
        case 1: return kind == 1 ? stage_info.x720 : kind == 2 ? stage_info.x714 : -1;
        case 2:
            if (kind == 2) return (stage_info.flags & 0x10) ? stage_info.x6D0 : -1;
            if (kind == 1) {
                ph = grShrineRoute_ScriptQuery(0, 0);
                return (ph >= 1 && ph <= 3) || ph == 6 ? grShrineRoute_ScriptQuery(2, 0) : -1;
            }
            return -1;
        case 3:
            if (kind == 1) { ph = grShrineRoute_ScriptQuery(0, 0); return ph < 0 ? 0 : ph; }
            return kind == 2 ? stage_info.x6DC : 0;
        case 4: { int m = kind == 1 ? grShrineRoute_ScriptQuery(1, 0) : 0; return m < 0 ? 0 : m; }
        case 5: return kind == 1 && (stage_info.flags & 0x10) ? 1 : 0;
        }
        return 0;
    } else {
        Vec3 p; int id, cand = -1;
        if (!kind || idx < 0 || field < 0 || field > 6) return 0;
        if (group == 1) {
            if (kind == 1) { if (idx > 9) return 0; id = 0xBD + idx; cand = stage_info.x720; }
            else { if (idx > 25) return 0; id = 0x99 + idx; cand = stage_info.x714; }
        } else if (group == 2 && kind == 2) {
            if (idx > 2) return 0;
            id = 5 + idx;
        } else return 0;
        if (!Ground_801C2D24(id, &p)) return 0;
        switch (field) {
        case 0: return 1;
        case 1: return script_skill_bits(p.x); case 2: return script_skill_bits(p.y); case 3: return script_skill_bits(p.z);
        case 4: return id;
        case 5: return kind == 1 && group == 1 ? grShrineRoute_ScriptQuery(3, idx) : -1;
        case 6: { int m = kind == 1 && group == 1 ? grShrineRoute_ScriptQuery(1, 0) : 0; return m > 0 && (m & (1 << idx)) ? 1 : 0; }
        }
        (void) cand;
        return 0;
    }
}

#include "script_crit.inc"
#include "script_shock.inc"

/* Bonus-stage objectives, read-only. Break the Targets targets are It_Kind_Mato items (retail targets and gd.spawn_target alike);
 * a broken target is removed by Mato's own destroy path, so every live Mato item is an intact target. */
int ScriptGame_ObjectiveKind(void) { return It_Kind_Mato; }

/* Race to the Finish (Gr_Kind_Pushon, src/melee/gr/grpushon.c), read-only. The stage has no door objects and no items: its
 * "doors" are map points 0x99..0xB2 (26 slots; Ground_801C2D24 gives a point's world position, false where the map has none).
 * grPushOn_802186C8 asks the ground (Ground_801C3D44) to test the first player's position every frame against a 25 x 20 box
 * around each of those points, gated by fn_80218678 (a collision callback on map joints 3-5, fn_802190A0, sets its flag); the
 * first point that passes is stored in stage_info.x714 and, once the stage latches it, stage_info.x6D0 (Ground_801C1DD4).
 * That index selects the entry of the stage's Yakumono parameter table (grPushOn_80219204: x1c[id - 0x99] = {score value,
 * coin count}) that the results screen shows as the special score (gmregclear.c, OUTCOME_UNK_1P_BONUS_STAGE_END). Four more
 * rectangles (grPushOn_803E7CCC, scaled by Ground_801C0498) are the fighter-device area test grPushOn_802192A4 runs.
 * idx -1: field 0 is Race to the Finish (1/0), 1 the latched door id (-1 none), 2 the touch flag, 3 the box candidate
 * (stage_info.x714), 4 stage scale bits. idx 0..25: field 0 present, 1/2/3 x/y/z bits, 4 score value, 5 coin count,
 * 6 latched (this is the door the race ended at), 7 candidate (the first player is inside this door's box now). */
extern int grPushOn_80219204(int arg0, int* out1, int* out2);
extern int fn_80218678(void);
extern float grPushOn_803E7CCC[13];
int ScriptGame_ObjectiveDoor(int idx, int field)
{
    Vec3 p; int id, score = 0, coins = 0;
    if (idx == -1) {
        if (stage_info.grkind != Gr_Kind_Pushon) return field == 1 ? -1 : field == 3 ? -1 : 0;
        switch (field) {
        case 0: return 1; case 1: return Ground_801C1DD4(); case 2: return fn_80218678() ? 1 : 0;
        case 3: return stage_info.x714; case 4: return script_skill_bits(Ground_801C0498());
        }
        return 0;
    }
    if (stage_info.grkind != Gr_Kind_Pushon || idx < 0 || idx > 25) return 0;
    id = 0x99 + idx;
    if (!Ground_801C2D24(id, &p)) return 0;
    switch (field) {
    case 0: return 1;
    case 1: return script_skill_bits(p.x); case 2: return script_skill_bits(p.y); case 3: return script_skill_bits(p.z);
    case 4: grPushOn_80219204(id, &score, &coins); return score;
    case 5: grPushOn_80219204(id, &score, &coins); return coins;
    case 6: return Ground_801C1DD4() == id;
    case 7: return stage_info.x714 == id;
    }
    return 0;
}
/* The four device rectangles: field 0 x min, 1 x max, 2 y top, 3 y bottom (top - 50), bits, already scaled. */
int ScriptGame_ObjectiveZone(int idx, int field)
{
    float scale;
    if (stage_info.grkind != Gr_Kind_Pushon || idx < 0 || idx > 3 || field < 0 || field > 3) return 0;
    scale = Ground_801C0498();
    switch (field) {
    case 0: return script_skill_bits(scale * grPushOn_803E7CCC[idx * 3]);
    case 1: return script_skill_bits(scale * grPushOn_803E7CCC[idx * 3 + 1]);
    case 2: return script_skill_bits(scale * grPushOn_803E7CCC[idx * 3 + 2]);
    case 3: return script_skill_bits(scale * (-50.0f + grPushOn_803E7CCC[idx * 3 + 2]));
    }
    return 0;
}
