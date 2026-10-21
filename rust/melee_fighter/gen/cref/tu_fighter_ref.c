/* C reference for src/tu/fighter.rs: the whole of src/melee/ft/fighter.c (with technical_ai.inc,
 * cpu_assist.inc, technical_ai_policy.h) is #included verbatim; shipping contraction via tu_run.py.
 * Externals the tested functions reach are defined below as fixed-value stubs.
 *   SEG  q a b -> cpu_assist_seg_dist2
 *   POL  tech_ai_policy_step sequences (policy state carried per sequence)
 *   DEC  cpu_assist_decide / cpu_assist_roll
 *   RBH  RB_FighterHash (legacy and widened) over random fighters, with the CPU block
 * All numbers are hex bit patterns. */
#include "ft/fighter.c"

static int g_cpu, g_stocks, g_legacy_env;
static u32 g_intr, g_geno;
bool ftCo_IsCpuControlled(Fighter* fp) { return g_cpu; }
int Player_GetStocks(int slot) { return g_stocks; }
u32 ScriptGame_IntrWinHashWord(int e) { return g_intr; }
u32 GenoDefine_StateDigest(Fighter* fp) { return g_geno; }
int RbHashLegacyEnv(void) { return g_legacy_env; }

static u32 s = 0x2468ACEu;
static u32 rnd(void) { s ^= s << 13; s ^= s >> 17; s ^= s << 5; return s; }
static float rf(void)
{
    u32 k = rnd() % 16;
    if (k == 0) return 0.0f;
    if (k == 1) return -0.0f;
    if (k == 2) return (float) (rnd() % 7) * 1e-4f;
    return ((float) ((int) (rnd() % 200001) - 100000)) / 37.0f;
}
static u32 B(float f) { union { float f; u32 u; } c; c.f = f; return c.u; }

int main(void)
{
    static Fighter F;
    int i, j;
    for (i = 0; i < 3000; i++) {
        Vec3 q, a, b;
        q.x = rf(); q.y = rf(); q.z = rf();
        a.x = rf(); a.y = rf(); a.z = rf();
        if (rnd() % 8 == 0) { b = a; } else { b.x = rf(); b.y = rf(); b.z = rf(); }
        printf("SEG %08x %08x %08x %08x %08x %08x %08x %08x %08x %08x\n", B(q.x), B(q.y), B(q.z), B(a.x), B(a.y), B(a.z),
               B(b.x), B(b.y), B(b.z), B(cpu_assist_seg_dist2(&q, &a, &b)));
    }
    for (i = 0; i < 60; i++) {
        TechAiPolicy p;
        int skill = (int) (rnd() % 5), seed = (int) (rnd() | 1);
        tech_ai_policy_init(&p, skill, (unsigned) seed);
        for (j = 0; j < 80; j++) {
            TechAiObservation o;
            int ev;
            o.type = (int) (rnd() % 4);
            o.motion = (int) (rnd() % 3) + 60;
            o.floor = (int) (rnd() % 4 != 0);
            o.fresh_shoulder = (int) (rnd() % 3 != 0);
            o.tech_unlocked = (int) (rnd() % 3 != 0);
            o.eta = (float) ((int) (rnd() % 160) - 10) / 10.0f;
            /* hold the observation for runs so the policy gets through its wait */
            if (j % 8 != 0 && j > 0) { o.type = (int) (rnd() % 8 == 0 ? rnd() % 3 : 1 + (i & 1)); o.motion = 61; }
            ev = tech_ai_policy_step(&p, &o);
            printf("POL %d %d %d %u %d %d %d %d %d %08x %d %d %d %d %d %d %d %d %u\n", i, j, skill, (unsigned) seed, o.type, o.motion, o.floor,
                   o.fresh_shoulder, o.tech_unlocked, B(o.eta), ev, p.opportunities, p.lcancels, p.techs, p.misses, p.last_event, p.delay,
                   p.cooldown, p.seed);
        }
    }
    for (i = 0; i < 40; i++) {
        CpuAssistSlot sl;
        int t;
        memset(&sl, 0, sizeof sl);
        sl.seed = rnd() | 1;
        for (t = 0; t < CA_N; t++) sl.permille[t] = (int) (rnd() % 1001);
        for (j = 0; j < 50; j++) {
            int sub = -7, r;
            unsigned s0;
            t = (int) (rnd() % CA_N);
            s0 = sl.seed;
            sl.opportunities[t] = 0;
            r = cpu_assist_decide(&sl, t, &sub);
            printf("DEC %d %d %d %d %d %d %u %d %u\n", i, j, t, sl.permille[t], r, sub, sl.seed, sl.opportunities[t], s0);
        }
    }
    for (i = 0; i < 1500; i++) {
        Fighter* fp = &F;
        struct CpuFighter* c = &fp->cpu;
        u32 h0 = rnd(), res;
        int slot = (int) (rnd() % 12);
        int csp;
        memset(&F, 0, sizeof F);
        fp->motion_id = (int) (rnd() % 400);
        fp->cur_pos.x = rf(); fp->cur_pos.y = rf(); fp->cur_pos.z = rf();
        fp->self_vel.x = rf(); fp->self_vel.y = rf();
        fp->dmg.x1830_percent = rf(); fp->facing_dir = rnd() & 1 ? 1.0f : -1.0f;
        fp->ground_or_air = (int) (rnd() % 2);
        fp->x8c_kb_vel.x = rf(); fp->x8c_kb_vel.y = rf(); fp->gr_vel = rf();
        fp->dmg.x195c_hitlag_frames = rf();
        fp->x2219_b5 = (int) (rnd() % 2);
        fp->shield_health = rf();
        fp->x3E4_fighterCmdScript.frame_count = rf();
        fp->x1968_jumpsUsed = (int) (rnd() % 4);
        fp->x67F = (u8) rnd(); fp->x680 = (u8) rnd();
        g_cpu = (int) (rnd() % 2);
        g_stocks = (int) (rnd() % 5);
        g_intr = rnd() % 3 ? 0 : rnd();
        g_geno = rnd() % 3 ? 0 : rnd();
        g_legacy_env = (int) (rnd() % 2);
        RB_HashSetLegacy(-1);
        c->buttons = rnd();
        c->lstick.x = (s8) rnd(); c->lstick.y = (s8) rnd(); c->cstick.x = (s8) rnd(); c->cstick.y = (s8) rnd();
        c->ltrigger = (u8) rnd(); c->rtrigger = (u8) rnd();
        c->kind = (int) (rnd() % 8); c->level = (int) (rnd() % 10);
        c->x14 = (int) rnd(); c->x18 = (int) rnd(); c->x1C = (int) rnd(); c->x20 = (int) rnd(); c->x24 = (int) rnd();
        c->x28 = (int) rnd(); c->x2C = (int) rnd(); c->x30 = (int) rnd(); c->x34 = (int) rnd();
        c->x38 = rf(); c->x3C = rf(); c->x40 = rf();
        c->x7C = (int) rnd(); c->x80 = (int) rnd(); c->x84 = (int) rnd();
        c->xC8 = (u8) rnd(); c->xEC = (u8) rnd(); c->command_duration = rnd();
        csp = (int) (rnd() % 3 == 0 ? -1 : rnd() % (sizeof c->buffer + 1));
        c->csP = csp < 0 ? (void*) 0 : (void*) (c->buffer + csp);
        /* the four flag bytes the hash reads at +0xF8 of the struct as the C build lays it out */
        ((u8*) c)[0xF8] = (u8) rnd(); ((u8*) c)[0xF9] = (u8) rnd(); ((u8*) c)[0xFA] = (u8) rnd(); ((u8*) c)[0xFB] = (u8) rnd();
        res = RB_FighterHash(h0, fp, slot);
        printf("RBH %u %d %d %d %u %u %d %d %u\n", h0, slot, g_cpu, g_stocks, g_intr, g_geno, g_legacy_env, csp, res);
        printf("RBF %d %08x %08x %08x %08x %08x %08x %08x %08x %08x %08x %08x %d %d %d %08x %08x %d %d\n", fp->motion_id,
               B(fp->cur_pos.x), B(fp->cur_pos.y), B(fp->cur_pos.z), B(fp->self_vel.x), B(fp->self_vel.y), B(fp->dmg.x1830_percent),
               B(fp->facing_dir), B(fp->x8c_kb_vel.x), B(fp->x8c_kb_vel.y), B(fp->gr_vel), B(fp->dmg.x195c_hitlag_frames),
               fp->ground_or_air, fp->x2219_b5, fp->x1968_jumpsUsed, B(fp->shield_health), B(fp->x3E4_fighterCmdScript.frame_count),
               (int) fp->x67F, (int) fp->x680);
        printf("RBC %u %u %u %u %u %d %d %d %d %d %d %d %d %d %d %d %d %d %08x %08x %08x %d %d %d %d %u %u %u %u %u\n",
               (unsigned) c->buttons, (unsigned) (u8) c->lstick.x, (unsigned) (u8) c->lstick.y, (unsigned) (u8) c->cstick.x, (unsigned) (u8) c->cstick.y,
               (int) c->ltrigger, (int) c->rtrigger, (int) c->kind, c->level, c->x14, c->x18, c->x1C, c->x20, c->x24, c->x28, c->x2C, c->x30, c->x34,
               B(c->x38), B(c->x3C), B(c->x40), c->x7C, c->x80, c->x84, (int) c->xC8, (unsigned) c->xEC, (unsigned) c->command_duration,
               (unsigned) ((u8*) c)[0xF8], (unsigned) ((u8*) c)[0xF9], (unsigned) ((u8*) c)[0xFA]);
        printf("RBD %u\n", (unsigned) ((u8*) c)[0xFB]);
    }
    return 0;
}
