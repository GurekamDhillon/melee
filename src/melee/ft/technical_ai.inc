/* TARGET_PC-only original assist, included by fighter.c. Script owner lifecycle
 * lives in gw_script.c. Only native CPU Fox/Falco, primary entity, CpuKind_4.
 * Baseline directions/attacks/recovery remain untouched; no gameplay-state edits. */
#include "technical_ai_policy.h"
#include <string.h>

typedef struct TechAiSlot {
    Fighter* fighter;
    int spawn, owner;
    TechAiPolicy policy;
} TechAiSlot;
static TechAiSlot technical_ai_slots[6];
extern int Netplay_Enabled(void);
extern int RB_Enabled(void);
extern int Replay_Active(void);

static int technical_ai_live(HSD_GObj* gobj)
{
    HSD_GObj* current;
    if (!gobj || !HSD_GObjPLinkHead) return 0;
    for (current = HSD_GObjPLinkHead[HSD_GOBJ_PLINK_FIGHTER]; current; current = current->next)
        if (current == gobj) return 1;
    return 0;
}

static int technical_ai_supported(Fighter* fp)
{
    return fp != NULL && !fp->is_sub_fighter &&
        (fp->kind == Ft_Kind_Fox || fp->kind == Ft_Kind_Falco) &&
        Player_GetPlayerSlotType(fp->player_id) == Gm_PKind_Cpu &&
        ftCo_IsCpuControlled(fp) && fp->cpu.kind == CpuKind_4;
}

int ScriptGame_CpuTechnical(int slot, int owner, int skill, int seed)
{
    TechAiSlot* state;
    HSD_GObj* gobj;
    Fighter* fp;
    if (slot < 0 || slot >= 6 || owner <= 0 || skill < 0 || skill > 3 ||
        (skill != 0 && seed <= 0) || Netplay_Enabled() || RB_Enabled() ||
        Replay_Active()) return 0;
    state = &technical_ai_slots[slot];
    if (state->owner && state->owner != owner) return 0;
    if (skill == 0) { memset(state, 0, sizeof *state); return 1; }
    gobj = Player_GetEntity(slot);
    fp = technical_ai_live(gobj) ? GET_FIGHTER(gobj) : NULL;
    if (!technical_ai_supported(fp)) return 0;
    state->fighter = fp;
    state->spawn = fp->x8_spawnNum;
    state->owner = owner;
    tech_ai_policy_init(&state->policy, skill, (unsigned) seed);
    return 1;
}

void ScriptGame_CpuTechnicalClear(int owner)
{
    int i;
    for (i = 0; i < 6; ++i) if (owner == 0 || technical_ai_slots[i].owner == owner)
        memset(&technical_ai_slots[i], 0, sizeof technical_ai_slots[i]);
}

int ScriptGame_CpuTechnicalState(int slot, int owner, int field)
{
    TechAiSlot* s;
    if (slot < 0 || slot >= 6 || owner <= 0) return 0;
    s = &technical_ai_slots[slot];
    if (s->owner != owner) return 0;
    switch (field) {
    case 0: return s->policy.skill;
    case 1: return s->policy.opportunities;
    case 2: return s->policy.lcancels;
    case 3: return s->policy.techs;
    case 4: return s->policy.misses;
    case 5: return s->policy.last_event;
    case 6: return s->policy.delay;
    }
    return 0;
}

/* Called once during CPU input synthesis, before LR normalization/edge timers.
 * Native map ray uses current velocity and ECB, never a future trajectory oracle.
 * Pausing freezes the assist. Unsupported/standing/replaced actors fail closed. */
static void technical_ai_overlay(Fighter* fp)
{
    TechAiSlot* slot;
    TechAiObservation o = { 0 };
    Vec3 floor, normal;
    float vx, vy, x, y, horizon = 12.0f;
    int event, line;
    unsigned flags;
    if (fp->player_id >= 6 || fp->is_sub_fighter) return;
    slot = &technical_ai_slots[fp->player_id];
    if (!slot->owner) return;
    if (slot->fighter != fp || slot->spawn != fp->x8_spawnNum) {
        memset(slot, 0, sizeof *slot);
        return;
    }
    if (Netplay_Enabled() || RB_Enabled() || Replay_Active()) {
        memset(slot, 0, sizeof *slot);
        return;
    }
    if (gm_GetDbPauseFlag(0) || gm_GetDbPauseFlag(2)) return;
    if (!technical_ai_supported(fp) || fp->ground_or_air != GA_Air ||
        fp->dmg.x195c_hitlag_frames > 0.0f) {
        tech_ai_policy_step(&slot->policy, &o);
        return;
    }
    o.motion = fp->motion_id;
    /* Analogue L-cancel cannot trigger a digital air dodge. Require active lag
     * flag; normal autorecovery/cancellable aerials remain under baseline AI. */
    if (fp->motion_id >= ftCo_MS_AttackAirN &&
        fp->motion_id <= ftCo_MS_AttackAirLw && fp->cmd_vars[0] &&
        !fp->allow_interrupt) o.type = 1;
    /* Only live hitstun states: a digital tech pulse never invents an air dodge
     * from actionable tumble. Ground collision still chooses native tech/no-tech. */
    else if (fp->motion_id >= ftCo_MS_DamageFlyHi &&
             fp->motion_id <= ftCo_MS_DamageFlyRoll && fp->x221C_b6)
        o.type = 2;
    vx = fp->self_vel.x + fp->x8c_kb_vel.x;
    vy = fp->self_vel.y + fp->x8c_kb_vel.y;
    if (!o.type || !(vy < -0.05f && vy > -100.0f && vx > -100.0f && vx < 100.0f)) {
        tech_ai_policy_step(&slot->policy, &o);
        return;
    }
    x = fp->cur_pos.x + fp->coll_data.ecb.bottom.x;
    y = fp->cur_pos.y + fp->coll_data.ecb.bottom.y;
    if (mpCheckFloor(x, y + 0.01f, x + vx * horizon, y + vy * horizon,
                    0.0f, &floor, &line, &flags, &normal,
                    fp->coll_data.floor_skip, fp->coll_data.joint_id_skip,
                    fp->coll_data.joint_id_only, NULL, fp->gobj)) {
        o.floor = 1;
        o.eta = (y - floor.y) / -vy;
    }
    o.fresh_shoulder = !(fp->input.held_buttons[1] & HSD_PAD_LR) &&
        !(fp->input.held_buttons[0] & (HSD_PAD_L | HSD_PAD_R | HSD_PAD_Z)) &&
        fp->input.triggers[0] == 0.0f;
    o.tech_unlocked = fp->x680 >= p_ftCommonData->x1C;
    event = tech_ai_policy_step(&slot->policy, &o);
    if (event == 1) fp->input.triggers[0] = 0.35f;
    else if (event == 2) fp->input.held_buttons[0] |= HSD_PAD_R;
}

/* Disc-free deterministic native policy checks; nonzero reports a failed case. */
int ScriptGame_CpuTechnicalTest(void)
{
    TechAiPolicy a, b;
    TechAiObservation o = { 1, 65, 1, 1, 1, 4.0f };
    TechAiSlot prior = technical_ai_slots[5];
    int i, pulse = 0, errors = 0;
    tech_ai_policy_init(&a, 3, 1);
    tech_ai_policy_init(&b, 3, 1);
    for (i = 0; i < 10; ++i) {
        int x = tech_ai_policy_step(&a, &o);
        int y = tech_ai_policy_step(&b, &o);
        if (x != y || (i < 2 && x)) ++errors;
        pulse += x == 1;
    }
    if (pulse != 1 || a.lcancels != 1 || a.delay != 2) ++errors;
    tech_ai_policy_init(&a, 3, 1); o.floor = 0;
    for (i = 0; i < 20; ++i) if (tech_ai_policy_step(&a, &o)) ++errors;
    if (a.opportunities) ++errors;
    o.floor = 1; o.type = 2; o.tech_unlocked = 0;
    for (i = 0; i < 10; ++i) if (tech_ai_policy_step(&a, &o)) ++errors;
    o.tech_unlocked = 1;
    if (tech_ai_policy_step(&a, &o) != 2) ++errors;
    for (i = 0; i < 60; ++i) if (tech_ai_policy_step(&a, &o)) ++errors;
    tech_ai_policy_init(&a, 3, 1); o.fresh_shoulder = 0;
    for (i = 0; i < 10; ++i) if (tech_ai_policy_step(&a, &o)) ++errors;
    o.fresh_shoulder = 1;
    if (tech_ai_policy_step(&a, &o) != 2) ++errors;
    tech_ai_policy_init(&a, 1, 1); o.type = 1;
    for (i = 0; i < 12; ++i) if (tech_ai_policy_step(&a, &o)) ++errors;
    if (a.misses != 1 || a.delay != 6) ++errors;
    /* Ownership clear is exact; no unrelated script loses its assist. */
    memset(&technical_ai_slots[5], 0, sizeof technical_ai_slots[5]);
    technical_ai_slots[5].owner = 723;
    technical_ai_slots[5].policy.skill = 3;
    ScriptGame_CpuTechnicalClear(724);
    if (ScriptGame_CpuTechnicalState(5, 723, 0) != 3 ||
        ScriptGame_CpuTechnicalState(5, 724, 0) != 0) ++errors;
    ScriptGame_CpuTechnicalClear(723);
    if (technical_ai_slots[5].owner) ++errors;
    technical_ai_slots[5] = prior;
    return errors;
}
