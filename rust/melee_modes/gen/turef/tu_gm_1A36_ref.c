/* C reference for rust/melee_modes/src/tu/gm_1A36.rs: the game's gm_1A36.c verbatim (so the static
 * `controller_map`, gm_801A3820 and fn_801A396C are the game's own); HSD_PadCopyStatus and the mode table
 * are defined here. The whole controller_map is printed after every call.
 * Run: python rust/melee_modes/gen/turef/tu_shard537_gm.py tu_gm_1A36 */
#include "../../../../src/melee/gm/gm_1A36.c"
#include "tu_gm_common.h"

HSD_PadStatus HSD_PadCopyStatus[4];
HSD_PadStatus HSD_PadMasterStatus[4];

static GameMode MODES[4];
static void init_a(void) { lg("ia;"); }
static void init_b(void) { lg("ib;"); }
GameMode* gm_GetAllGameModes(void) { return MODES; }

static const char* FUNC_NAMES[] = {"gm_GetButtonsPressed", "gm_GetButtonsTriggered", "gm_801A36C0", "gm_801A36E0",
                                   "gm_801A3714",          "gm_801A3820",            "fn_801A396C", "gm_EvaluateAllControllerInputs",
                                   "gm_801A3E88",          "gm_801A3EF4"};
#define NFUNCS ((int) (sizeof FUNC_NAMES / sizeof FUNC_NAMES[0]))
#define NCASES 300

static const int BITS[] = {0, 1, 2, 3, 5, 6, 8, 9, 12, 16, 17, 18, 19};
#define NBITS ((int) (sizeof BITS / sizeof BITS[0]))

/* A button word biased toward the bits the functions look at, with the odd stray bit. */
static u64 rbtn(void)
{
    u64 v = 0;
    int k;
    u32 r;
    for (k = 0; k < NBITS; k++) {
        r = rnd();
        if (r % 3 == 0) {
            v |= (u64) 1 << BITS[k];
        }
    }
    r = rnd();
    if (r % 4 == 0) {
        u32 s = rnd();
        v |= (u64) 1 << (s % 40);
    }
    return v;
}

static void setup_case(void)
{
    int i;
    u32 r;
    for (i = 0; i < 5; i++) {
        controller_map.x0[i].button = rbtn();
        controller_map.x0[i].trigger = (rnd() % 2) ? rbtn() : 0;
        controller_map.x0[i].repeat = rbtn();
        controller_map.x0[i].release = (rnd() % 2) ? rbtn() : 0;
        controller_map.x0[i].repeat2 = rbtn();
        r = rnd();
        controller_map.x0[i].repeat_timer = (r % 4 == 0) ? 0 : (int) (r >> 8) % 7 - 2;
        r = rnd();
        controller_map.x0[i].x2C = (int) (r % 0x72);
    }
    controller_map.xF0 = fn_801A396C;
    controller_map.xF4 = (u16) (rnd() % 0x30);
    controller_map.xF6 = (u8) (rnd() % 10);
    controller_map.xF8 = (u16) (rnd() % 0x50);
    controller_map.xFA = (u8) (rnd() % 10);
    controller_map.xFC = (u16) (rnd() % 0x70);
    controller_map.xFE = (u8) (rnd() % 10);
    for (i = 0; i < 4; i++) {
        HSD_PadCopyStatus[i].button = (u32) rbtn();
        HSD_PadCopyStatus[i].trigger = (u32) rbtn();
        HSD_PadCopyStatus[i].repeat = (u32) rbtn();
        HSD_PadCopyStatus[i].release = (u32) rbtn();
    }
    memset(MODES, 0, sizeof MODES);
    MODES[0].kind = 1;
    MODES[0].on_init = init_a;
    MODES[1].kind = 2;
    MODES[2].kind = 3;
    MODES[2].on_init = init_b;
    MODES[3].kind = GM_COUNT;
    MODES[3].on_init = init_a;
}

static void dump(int f, int i, u64 ret)
{
    int k;
    printf("%d.%d %s|log=%s|ret=%llx", f, i, FUNC_NAMES[f], g_log, (unsigned long long) ret);
    for (k = 0; k < 5; k++) {
        struct gm_controller_map* c = &controller_map.x0[k];
        printf("|%llx,%llx,%llx,%llx,%llx,%d,%d", (unsigned long long) c->button, (unsigned long long) c->trigger,
               (unsigned long long) c->repeat, (unsigned long long) c->release, (unsigned long long) c->repeat2,
               (int) c->repeat_timer, (int) c->x2C);
    }
    printf("|xF0=%d,%u,%u,%u,%u,%u,%u\n", controller_map.xF0 == fn_801A396C ? 1 : (controller_map.xF0 == NULL ? 0 : 2),
           (unsigned) controller_map.xF4, (unsigned) controller_map.xF6, (unsigned) controller_map.xF8,
           (unsigned) controller_map.xFA, (unsigned) controller_map.xFC, (unsigned) controller_map.xFE);
}

int main(void)
{
    int f;
    int i;
    for (f = 0; f < NFUNCS; f++) {
        for (i = 0; i < NCASES; i++) {
            u64 ret = 0;
            int idx;
            u64 a1;
            u64 a2;
            setup_case();
            g_loglen = 0;
            g_log[0] = 0;
            idx = (int) (rnd() % 5);
            a1 = rbtn();
            a2 = rbtn();
            a2 |= (u64) 1 << (32 + rnd() % 8);
            switch (f) {
            case 0: ret = gm_GetButtonsPressed((u8) idx); break;
            case 1: ret = gm_GetButtonsTriggered((u8) idx); break;
            case 2: ret = gm_801A36C0((u8) idx); break;
            case 3: gm_801A36E0(idx, (int) (rnd() % 100) - 20); break;
            case 4: gm_801A3714(idx, a1, a2); break;
            case 5: gm_801A3820(idx, a1, a2); break;
            case 6: fn_801A396C(idx); break;
            case 7: gm_EvaluateAllControllerInputs(); break;
            case 8: gm_801A3E88(); break;
            default: gm_801A3EF4(); break;
            }
            dump(f, i, ret);
        }
    }
    return 0;
}
