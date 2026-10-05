/* The CPU's virtual controller, driven by a script: gd.cpu_pad / gd.cpu_script / gd.cpu_dest.
 *
 * Mode "script" (script_cpu_override == 3): the retail AI does not run and does not write
 * fp->cpu.*. At the same point in the frame (ftCo_800B3900, from Fighter_8006ABA0) this file
 * writes it instead, either from a held pad sample or by running a retail command script
 * (ftCo_800B3E04, the interpreter the AI itself uses) over the 0x100-byte buffer in
 * fp->cpu. Everything it keeps is in fp->cpu or in script_cpu_ctl below, both game memory, so
 * a savestate or rewind carries it. Nothing here reads the wall clock or a random number.
 *
 * Interface: ScriptGame_CpuCtl(slot, sub, op, a). op < 100 writes, op >= 100 reads. sub 1 is
 * Nana. A write on a slot that is not a CPU in script mode returns 0. */
typedef struct ScriptCpuCtl {
    int pad_frames;        /* completed logic frames the held pad still has */
    int pad_used;          /* a pad hold ended and the neutral frame is owed */
    int stage[7];          /* buttons, x, y, cx, cy, l, r: staged by ops 10..16 */
    int pad[7];            /* the committed hold */
    int running;           /* a command script is in progress */
    int neutral_end;       /* release everything when the script finishes */
    int done_count;        /* command scripts finished since the mode was entered */
    int target;            /* slot read by the toward-fighter commands, -1 = first other fighter */
    /* gd.cpu_goto, the rule-based mover. go_state: 0 none, 1 running, 2 arrived, 3 blocked, 4 fell */
    int go_state, go_ticks, go_timeout, go_stuck, go_press, go_jumps_at, go_phase;
    int go_stage[3];       /* staged destination x, y and radius, as float bits */
    float go_x, go_y, go_radius, go_last_x;
} ScriptCpuCtl;
static ScriptCpuCtl script_cpu_ctl[6][2];

enum {
    CPU_CTL_PAD_COMMIT = 1, CPU_CTL_SCRIPT_BEGIN, CPU_CTL_SCRIPT_BYTE, CPU_CTL_SCRIPT_RUN,
    CPU_CTL_DEST_X, CPU_CTL_DEST_Y, CPU_CTL_TARGET, CPU_CTL_CANCEL,
    CPU_CTL_STAGE = 10, /* 10..16 */
    CPU_CTL_GOTO_X = 20, CPU_CTL_GOTO_Y, CPU_CTL_GOTO_RADIUS, CPU_CTL_GOTO_START, CPU_CTL_GOTO_STOP,
    CPU_CTL_G_ACTIVE = 100, CPU_CTL_G_RUNNING, CPU_CTL_G_DONE_COUNT, CPU_CTL_G_PAD_FRAMES,
    CPU_CTL_G_DEST_X, CPU_CTL_G_DEST_Y, CPU_CTL_G_BUTTONS, CPU_CTL_G_LX, CPU_CTL_G_LY,
    CPU_CTL_G_CX, CPU_CTL_G_CY, CPU_CTL_G_LTRIG, CPU_CTL_G_RTRIG, CPU_CTL_G_TARGET,
    CPU_CTL_G_CSP, CPU_CTL_G_DURATION, CPU_CTL_G_FREE, CPU_CTL_G_GOTO_STATE, CPU_CTL_G_GOTO_TICKS
};

static void script_cpu_ctl_neutral(struct CpuFighter* d)
{
    d->buttons = 0;
    d->lstick.x = d->lstick.y = 0;
    d->cstick.x = d->cstick.y = 0;
    d->ltrigger = d->rtrigger = 0;
}

static int script_cpu_clamp(int v, int lo, int hi) { return v < lo ? lo : v > hi ? hi : v; }


/* ---- gd.cpu_goto: a rule-based mover over the stage's floors and platforms ------------------
 * No search. Every logic frame it reads the fighter's state, asks the collision system where the
 * floor is (mpCheckFloor rays, the same query gd.floor_below uses: they see the floor lines and
 * platforms, which is the topology mpIsland is built from) and writes the virtual controller:
 *   same level   walk or dash along the floor; a gap along the way is jumped
 *   higher       walk under the target, jump (a second jump at the apex if the first falls short)
 *   lower        walk to the target x and drop through a platform, or walk off the edge
 *   in the air   drift toward the target, second jump when below it, climb a ledge once caught
 * Reports arrived (grounded within radius of x and 4 of y), blocked (timeout, no progress, or no
 * jump reaches) or fell (a KO state). */
#define GOTO_STEP 4.0f
static int script_goto_floor(float x, float y, float depth, float* fy)
{
    Vec3 pos; u32 flags = 0;
    if (!mpCheckFloor(x, y, x, y - depth, 0.0f, &pos, NULL, &flags, NULL, -1, -1, -1, NULL, NULL)) return 0;
    *fy = pos.y;
    return 1;
}
/* can the fighter walk from x0 to x1 along a floor that never steps more than 3 units? */
static int script_goto_walkable(float x0, float y0, float x1)
{
    float dir = x1 >= x0 ? 1.0f : -1.0f, x, prev = y0, fy;
    int guard = 0;
    for (x = x0; (x1 - x) * dir > 0.0f && guard < 400; x += dir * GOTO_STEP, ++guard) {
        if (!script_goto_floor(x + dir * GOTO_STEP, prev + 3.0f, 8.0f, &fy)) return 0;
        if (fy - prev > 3.0f || prev - fy > 3.0f) return 0;
        prev = fy;
    }
    return 1;
}
/* one jump reaches v^2 / (2 g); a second from the apex adds the air jump's own */
static float script_goto_reach(Fighter* fp, int jumps_left)
{
    float g = fp->co_attrs.gravity, v1 = fp->co_attrs.jump_v_initial_velocity;
    float v2 = v1 * fp->co_attrs.air_jump_v_multiplier;
    float h = g > 0.0f ? v1 * v1 / (2.0f * g) : 0.0f;
    if (jumps_left > 0 && g > 0.0f) h += v2 * v2 / (2.0f * g);
    return h;
}
static void script_goto_stick(Fighter* fp, int x, int y)
{
    fp->cpu.lstick.x = (s8) script_cpu_clamp(x, -127, 127);
    fp->cpu.lstick.y = (s8) script_cpu_clamp(y, -127, 127);
}
/* stick for the remaining distance: full when far, scaled when near, released when friction
 * (or air drag) will stop the fighter inside the radius */
static int script_goto_stick_for(float dx, float vx, float friction, float radius)
{
    float adx = dx < 0.0f ? -dx : dx, dir = dx < 0.0f ? -1.0f : 1.0f;
    float stop = friction > 0.0f ? vx * vx / (2.0f * friction) : 0.0f;
    if (adx <= radius * 0.5f) return 0;
    if (vx * dir > 0.0f && stop + radius * 0.5f >= adx) return 0;
    if (adx > 28.0f) return (int) (dir * 127.0f);
    if (adx > 10.0f) return (int) (dir * 64.0f);
    return (int) (dir * 48.0f);
}
static void script_goto_tick(Fighter* fp, ScriptCpuCtl* c)
{
    struct CpuFighter* d = &fp->cpu;
    float x = fp->cur_pos.x, y = fp->cur_pos.y, dx = c->go_x - x, dy = c->go_y - y;
    float r = c->go_radius, adx = dx < 0.0f ? -dx : dx, fy;
    int air = fp->ground_or_air == GA_Air;
    int jumps_left = (int) fp->co_attrs.max_jumps - (int) fp->x1968_jumpsUsed;
    int mid = fp->motion_id;
    if (c->go_state != 1) return;
    ++c->go_ticks;
    script_cpu_ctl_neutral(d);
    if (mid < ftCo_MS_Wait) { /* a KO, the sleep after it or the rebirth */ c->go_state = 4; return; }
    /* the engine flags a fighter that has just stepped off an edge as grounded for one more frame */
    if (!air && !(mid >= ftCo_MS_JumpF && mid <= ftCo_MS_Fall) && adx <= r && dy < 4.0f && dy > -4.0f) {
        c->go_state = 2; return;
    }
    if (c->go_ticks > c->go_timeout) { c->go_state = 3; return; }
    if (c->go_press) { c->go_press = 0; goto stall; } /* a jump button needs a released frame */
    if (mid == ftCo_MS_CliffCatch) goto stall;
    if (mid == ftCo_MS_CliffWait) { script_goto_stick(fp, 0, 127); goto stall; }
    if (!air) {
        float vx = fp->gr_vel, fr = fp->co_attrs.ground_friction;
        int plat = (fp->coll_data.floor.flags & LINE_FLAG_PLATFORM) != 0;
        c->go_phase = 0;
        if (dy > 4.0f) {
            if (dy > script_goto_reach(fp, jumps_left) - 2.0f) { c->go_state = 3; return; }
            if (adx > 3.0f && adx > r) { script_goto_stick(fp, script_goto_stick_for(dx, vx, fr, 1.0f), 0); goto stall; }
            d->buttons |= HSD_PAD_X; c->go_press = 1;
            script_goto_stick(fp, dx > 0 ? 40 : dx < 0 ? -40 : 0, 0);
            goto stall;
        }
        if (dy < -4.0f) {
            if (adx > 2.0f && script_goto_walkable(x, y, c->go_x)) {
                script_goto_stick(fp, script_goto_stick_for(dx, vx, fr, 1.0f), 0); goto stall;
            }
            if (adx > 2.0f && !script_goto_floor(x + (dx > 0 ? 6.0f : -6.0f), y + 3.0f, 8.0f, &fy)) {
                script_goto_stick(fp, dx > 0 ? 127 : -127, 0); goto stall; /* off the edge */
            }
            if (plat && adx <= 2.0f) { script_goto_stick(fp, 0, -127); goto stall; } /* drops through */
            script_goto_stick(fp, dx > 0 ? 100 : -100, 0); goto stall;
        }
        if (script_goto_walkable(x, y, c->go_x)) {
            script_goto_stick(fp, script_goto_stick_for(dx, vx, fr, r), 0);
            if (d->lstick.x == 0 && adx > r) script_goto_stick(fp, dx > 0 ? 48 : -48, 0);
            goto stall;
        }
        /* a gap: jump it where the floor ends */
        if (!script_goto_floor(x + (dx > 0 ? 6.0f : -6.0f), y + 3.0f, 8.0f, &fy)) {
            d->buttons |= HSD_PAD_X; c->go_press = 1;
        }
        script_goto_stick(fp, dx > 0 ? 127 : -127, 0);
    } else {
        float vy = fp->self_vel.y;
        c->go_phase = 1;
        script_goto_stick(fp, script_goto_stick_for(dx, fp->self_vel.x, 0.04f, r > 3.0f ? r : 3.0f), 0);
        if (d->lstick.x == 0 && adx > r) script_goto_stick(fp, dx > 0 ? 40 : -40, 0);
        /* below the target and no longer rising: the second jump, once, toward it */
        if (dy > 2.0f && vy <= 0.5f && jumps_left > 0 && c->go_jumps_at != (int) fp->x1968_jumpsUsed + 1) {
            c->go_jumps_at = (int) fp->x1968_jumpsUsed + 1;
            d->buttons |= HSD_PAD_X; c->go_press = 1;
            script_goto_stick(fp, dx > 0 ? 100 : -100, 0);
        }
        if (dy > 200.0f && jumps_left <= 0 && vy < 0.0f) { c->go_state = 4; return; }
    }
stall:
    {
        float mv = x - c->go_last_x;
        if (!air && d->lstick.x != 0 && mv < 0.02f && mv > -0.02f) ++c->go_stuck; else c->go_stuck = 0;
        c->go_last_x = x;
        if (c->go_stuck > 90) c->go_state = 3;
    }
}

/* One logic frame of the controller. Pure over (ctl, fp, target): tested without a match. */
static void script_cpu_ctl_tick(Fighter* fp, ScriptCpuCtl* c, Fighter* target)
{
    struct CpuFighter* d = &fp->cpu;
    d->x44 = target;
    if (c->go_state == 1 && c->pad_frames == 0 && !c->running) {
        script_goto_tick(fp, c);
        if (c->go_state != 1) script_cpu_ctl_neutral(d);
        return;
    }
    if (c->pad_frames > 0) {
        d->buttons = (HSD_Pad) c->pad[0];
        d->lstick.x = (s8) c->pad[1]; d->lstick.y = (s8) c->pad[2];
        d->cstick.x = (s8) c->pad[3]; d->cstick.y = (s8) c->pad[4];
        d->ltrigger = (u8) c->pad[5]; d->rtrigger = (u8) c->pad[6];
        --c->pad_frames;
        c->pad_used = 1;
        return;
    }
    if (c->pad_used) {
        script_cpu_ctl_neutral(d);
        c->pad_used = 0;
        return;
    }
    if (d->csP != NULL) ftCo_800B3E04(fp);
    if (c->running && d->csP == NULL) {
        c->running = 0;
        c->done_count++;
        if (c->neutral_end) script_cpu_ctl_neutral(d);
    }
}

static void script_cpu_script_tick(Fighter* fp)
{
    int slot = fp->player_id, i;
    ScriptCpuCtl* c = &script_cpu_ctl[slot][fp->is_sub_fighter ? 1 : 0];
    Fighter* target = NULL;
    if (c->target >= 0) {
        target = script_fighter(c->target);
    } else {
        for (i = 0; i < 6 && target == NULL; ++i) if (i != slot) target = script_fighter(i);
    }
    if (target == fp) target = NULL;
    script_cpu_ctl_tick(fp, c, target);
}

static void script_cpu_ctl_reset(void) { memset(script_cpu_ctl, 0, sizeof script_cpu_ctl); }

static Fighter* script_cpu_ctl_fighter(int slot, int sub)
{
    Fighter* fp = script_fighter(slot);
    HSD_GObj* other;
    if (!fp) return NULL;
    if (!sub) return fp->is_sub_fighter ? NULL : fp;
    other = Player_GetEntityAtIndex(slot, 1);
    if (!other || other == fp->gobj || !script_gobj_live(other)) return NULL;
    fp = GET_FIGHTER(other);
    return fp && fp->is_sub_fighter ? fp : NULL;
}

/* Enter script mode: the stand initialisation (kind 0, quiet, persisted in the slot), then the
 * override. Nana follows the primary. */
int ScriptGame_CpuScriptMode(int slot)
{
    if (!ScriptGame_CpuMode(slot, 0)) return 0;
    script_cpu_override[slot] = 3;
    memset(script_cpu_ctl[slot], 0, sizeof script_cpu_ctl[slot]);
    script_cpu_ctl[slot][0].target = script_cpu_ctl[slot][1].target = -1;
    return 1;
}

int ScriptGame_CpuCtl(int slot, int sub, int op, int a)
{
    Fighter* fp;
    ScriptCpuCtl* c;
    struct CpuFighter* d;
    if (slot < 0 || slot >= 6 || (sub != 0 && sub != 1)) return op >= 100 ? -1 : 0;
    fp = script_cpu_ctl_fighter(slot, sub);
    if (!fp || Player_GetPlayerSlotType(slot) != Gm_PKind_Cpu) return op >= 100 ? -1 : 0;
    c = &script_cpu_ctl[slot][sub];
    d = &fp->cpu;
    if (op < 100 && script_cpu_override[slot] != 3) return 0;
    switch (op) {
    case CPU_CTL_GOTO_X: case CPU_CTL_GOTO_Y: case CPU_CTL_GOTO_RADIUS:
        c->go_stage[op - CPU_CTL_GOTO_X] = a;
        return 1;
    case CPU_CTL_GOTO_START:
        ftCo_800B4A78(fp);
        c->pad_frames = 0; c->pad_used = 0; c->running = 0;
        c->go_x = script_mesh_float(c->go_stage[0]); c->go_y = script_mesh_float(c->go_stage[1]);
        c->go_radius = script_mesh_float(c->go_stage[2]);
        c->go_state = 1; c->go_ticks = 0; c->go_stuck = 0; c->go_press = 0; c->go_jumps_at = -1;
        c->go_timeout = script_cpu_clamp(a, 1, 36000); c->go_last_x = fp->cur_pos.x; c->go_phase = 0;
        {   /* a destination with no floor under it cannot be arrived at */
            float fy;
            if (!script_goto_floor(c->go_x, c->go_y + 4.0f, 14.0f, &fy) || fy - c->go_y > 4.0f || c->go_y - fy > 4.0f)
                c->go_state = 3;
        }
        return 1;
    case CPU_CTL_GOTO_STOP:
        c->go_state = 0;
        script_cpu_ctl_neutral(d);
        return 1;
    case CPU_CTL_PAD_COMMIT:
        memcpy(c->pad, c->stage, sizeof c->pad);
        c->pad[0] &= 0xFFFF;
        c->pad[1] = script_cpu_clamp(c->pad[1], -127, 127); c->pad[2] = script_cpu_clamp(c->pad[2], -127, 127);
        c->pad[3] = script_cpu_clamp(c->pad[3], -127, 127); c->pad[4] = script_cpu_clamp(c->pad[4], -127, 127);
        c->pad[5] = script_cpu_clamp(c->pad[5], 0, 255); c->pad[6] = script_cpu_clamp(c->pad[6], 0, 255);
        c->pad_frames = script_cpu_clamp(a, 1, 0x7FFFFFF);
        c->running = 0; c->pad_used = 1; c->go_state = 0;
        d->csP = NULL; d->command_duration = 0; d->write_pos = d->buffer;
        return 1;
    case CPU_CTL_SCRIPT_BEGIN:
        ftCo_800B4A78(fp); /* neutral controller, no script, write position reset */
        c->pad_frames = 0; c->pad_used = 0; c->running = 0; c->go_state = 0;
        return 1;
    case CPU_CTL_SCRIPT_BYTE:
        if (d->write_pos < d->buffer || d->write_pos >= d->buffer + sizeof d->buffer - 1) return 0;
        *d->write_pos++ = (s8) a;
        return 1;
    case CPU_CTL_SCRIPT_RUN:
        if (d->write_pos == d->buffer) return 0;
        c->neutral_end = a != 0; c->running = 1;
        ftCo_800B49F4(fp); /* appends Done, arms the interpreter for the next frame */
        return 1;
    case CPU_CTL_DEST_X: d->x54.x = script_mesh_float(a); return 1;
    case CPU_CTL_DEST_Y: d->x54.y = script_mesh_float(a); return 1;
    case CPU_CTL_TARGET:
        if (a < -1 || a >= 6) return 0;
        c->target = a;
        return 1;
    case CPU_CTL_CANCEL:
        ftCo_800B4A78(fp);
        c->pad_frames = 0; c->pad_used = 0; c->running = 0; c->go_state = 0;
        return 1;
    case CPU_CTL_G_ACTIVE: return script_cpu_override[slot] == 3;
    case CPU_CTL_G_RUNNING: return c->running;
    case CPU_CTL_G_DONE_COUNT: return c->done_count;
    case CPU_CTL_G_PAD_FRAMES: return c->pad_frames;
    case CPU_CTL_G_DEST_X: { union { float f; int i; } u; u.f = d->x54.x; return u.i; }
    case CPU_CTL_G_DEST_Y: { union { float f; int i; } u; u.f = d->x54.y; return u.i; }
    case CPU_CTL_G_BUTTONS: return (int) d->buttons;
    case CPU_CTL_G_LX: return d->lstick.x;
    case CPU_CTL_G_LY: return d->lstick.y;
    case CPU_CTL_G_CX: return d->cstick.x;
    case CPU_CTL_G_CY: return d->cstick.y;
    case CPU_CTL_G_LTRIG: return d->ltrigger;
    case CPU_CTL_G_RTRIG: return d->rtrigger;
    case CPU_CTL_G_TARGET: return c->target;
    case CPU_CTL_G_CSP: return d->csP ? (int) (d->csP - d->buffer) : -1;
    case CPU_CTL_G_DURATION: return (int) d->command_duration;
    case CPU_CTL_G_GOTO_STATE: return c->go_state;
    case CPU_CTL_G_GOTO_TICKS: return c->go_ticks;
    case CPU_CTL_G_FREE: return (int) (sizeof d->buffer - 1 - (d->write_pos - d->buffer));
    default:
        if (op >= CPU_CTL_STAGE && op < CPU_CTL_STAGE + 7) { c->stage[op - CPU_CTL_STAGE] = a; return 1; }
    }
    return op >= 100 ? -1 : 0;
}

/* The fighter's own attributes the technique macros are tuned from, as float bits (the bridge
 * passes ints). -1 bits are never a valid value: an unknown id or a missing fighter returns 0. */
int ScriptGame_CpuAttr(int slot, int id)
{
    Fighter* fp = script_fighter(slot);
    union { float f; int i; } u;
    if (!fp) return 0;
    switch (id) {
    case 0: u.f = fp->co_attrs.jump_startup_time; break;
    case 1: u.f = fp->co_attrs.hop_v_initial_velocity; break;
    case 2: u.f = fp->co_attrs.jump_v_initial_velocity; break;
    case 3: u.f = fp->co_attrs.gravity; break;
    case 4: u.f = fp->co_attrs.terminal_velocity; break;
    case 5: u.f = fp->co_attrs.landingairn_lag; break;
    case 6: u.f = fp->co_attrs.landingairf_lag; break;
    case 7: u.f = fp->co_attrs.landingairb_lag; break;
    case 8: u.f = fp->co_attrs.landingairhi_lag; break;
    case 9: u.f = fp->co_attrs.landingairlw_lag; break;
    case 10: u.f = fp->co_attrs.normal_landing_lag; break;
    case 11: u.f = fp->co_attrs.dash_initial_velocity; break;
    case 12: u.f = fp->co_attrs.dash_max_velocity; break;
    case 13: u.f = fp->co_attrs.walk_max_vel; break;
    case 14: u.f = (float) fp->co_attrs.max_jumps; break;
    case 15: u.f = fp->co_attrs.fast_fall_velocity; break;
    case 16: u.f = fp->co_attrs.air_max_horizontal_velocity; break;
    case 17: u.f = fp->co_attrs.ground_friction; break;
    case 18: u.f = fp->co_attrs.ground_to_air_jump_momentum_multiplier; break;
    case 19: u.f = fp->co_attrs.jump_h_initial_velocity; break;
    case 20: u.f = p_ftCommonData->x344; break; /* airdodge landing lag (special-fall landing) */
    case 21: u.f = fp->co_attrs.air_drift_max; break;
    case 22: u.f = fp->co_attrs.initial_shield_size; break;
    case 23: u.f = p_ftCommonData->x250; break; /* the tech window, frames (ftCo_800986B0) */
    default: return 0;
    }
    return u.i;
}
