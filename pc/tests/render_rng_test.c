/* Compiles the real random.c and Det_SetInRender with only the trace sink stubbed.
 * This is an RNG boundary fixture; render_determinism.lua exercises a real match. */
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include "render_rng_production.inc"
void Replay_RandTrace(u32 caller, u32 after, s32 global)
{ (void) caller; (void) after; (void) global; }
#define CHECK(v) do { if (!(v)) { printf("FAIL line %d: %s\n", __LINE__, #v); return 1; } } while (0)

static int isolated(u32 *sim)
{
    u32 *saved = HSD_RandSeedPtr;
    u32 after_draw;
    HSD_RandSeedPtr = sim;
    *sim = 1;
    CHECK(HSD_Rand() == 41 && *sim == 2745024u);
    gw_Det_SetInRender(1);
    HSD_Rand();
    CHECK(*sim == 2745024u && HSD_RandSeedPtr == sim);
    gw_Det_SetInRender(1);
    HSD_Randf();
    gw_Det_SetInRender(0);
    HSD_Randi(100);
    CHECK(*sim == 2745024u && HSD_RandSeedPtr == sim);
    gw_Det_SetInRender(0);
    /* Snapshot-visible scratch must not depend on a later empty/paused redraw. */
    after_draw = render_seed;
    gw_Det_SetInRender(1);
    gw_Det_SetInRender(0);
    CHECK(render_seed == after_draw && render_depth == 0);
    CHECK(HSD_Rand() == 51235 && *sim == 3357800067u);
    HSD_RandSeedPtr = saved;
    return 0;
}

static void scripted_rng(unsigned presents, u32 hashes[240])
{
    unsigned frame, pass;
    *HSD_RandSeedPtr = 0x12345678u;
    for (frame = 0; frame < 240; ++frame) {
        u32 choice = (u32) HSD_Randi(11);
        if (choice < 5) HSD_Randf(); /* a simulation-side effect spawn */
        hashes[frame] = *HSD_RandSeedPtr;
        for (pass = 0; pass < presents; ++pass) {
            gw_Det_SetInRender(1);
            HSD_Rand(); HSD_Randf(); HSD_Randi(5);
            gw_Det_SetInRender(0);
        }
    }
}
int main(void)
{
    u32 alternate = 0, baseline[240], once[240], twice[240];
    CHECK(isolated(HSD_RandSeedPtr) == 0);
    CHECK(isolated(&alternate) == 0);
    scripted_rng(0, baseline); scripted_rng(1, once); scripted_rng(2, twice);
    CHECK(memcmp(baseline, once, sizeof baseline) == 0);
    CHECK(memcmp(baseline, twice, sizeof baseline) == 0);
    puts("render RNG: retail LCG, nested scopes, redirected seed, 240 frames at 0/1/2 draws PASS");
    return 0;
}
