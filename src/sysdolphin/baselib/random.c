#include "random.h"

static u32 seed = 1;
u32* HSD_RandSeedPtr = &seed;

#if defined(TARGET_PC)
/* HSD_JObjDisp creates SPtcl generators while drawing. Their initial count must
 * not consume the simulation stream. Keep this stream in snapshotted game
 * memory; start each outer traversal from a read-only copy of the logic seed so
 * extra paused views cannot perturb a later generator. Nested cameras share it.
 * HSD_RandSeedPtr (including a stage's redirected seed) is never changed. */
static u32 render_seed;
static u32 render_depth;

void HSD_RandRenderBegin(void)
{
    if (render_depth++ == 0) {
        render_seed = *HSD_RandSeedPtr;
    }
}

void HSD_RandRenderEnd(void)
{
    if (render_depth != 0 && --render_depth == 0) {
        /* Snapshots hash game BSS too. No presentation history may remain at
         * a logic boundary; the next outer Begin derives this value again. */
        render_seed = 0;
    }
}

static u32* HSD_RandCurrentSeed(void)
{
    return render_depth != 0 ? &render_seed : HSD_RandSeedPtr;
}

/* Trace stream: 1 = default simulation, 0 = redirected simulation, 2 = render.
 * Keep the call here so __builtin_return_address names the actual consumer. */
extern void Replay_RandTrace(u32 caller, u32 seed_after, s32 stream);
#define HSD_RAND_TRACE(p)                                                       \
    Replay_RandTrace((u32) __builtin_return_address(0), *(p),                    \
                     render_depth != 0 ? 2 : (p) == &seed)
#else
#define HSD_RAND_TRACE(p)
#endif

s32 HSD_Rand(void)
{
#if defined(TARGET_PC)
    u32* p = HSD_RandCurrentSeed();
    *p = *p * 214013 + 2531011;
    HSD_RAND_TRACE(p);
    return *p >> 0x10;
#else
    *HSD_RandSeedPtr = *HSD_RandSeedPtr * 214013 + 2531011;
    return *HSD_RandSeedPtr >> 0x10;
#endif
}

f32 HSD_Randf(void)
{
#if defined(TARGET_PC)
    u32* p = HSD_RandCurrentSeed();
    *p = *p * 214013 + 2531011;
    HSD_RAND_TRACE(p);
    return (f32) (*p >> 0x10) / (1 << 16);
#else
    *HSD_RandSeedPtr = *HSD_RandSeedPtr * 214013 + 2531011;
    return (f32) (*HSD_RandSeedPtr >> 0x10) / (1 << 16);
#endif
}

s32 HSD_Randi(s32 max_val)
{
#if defined(TARGET_PC)
    /* The same draw as HSD_Rand, inlined so the trace names Randi's caller. */
    u32* p = HSD_RandCurrentSeed();
    *p = *p * 214013 + 2531011;
    HSD_RAND_TRACE(p);
    return max_val * (s32) (*p >> 0x10) / (1 << 16);
#else
    return max_val * HSD_Rand() / (1 << 16);
#endif
}

void _HSD_RandForgetMemory(void* low, void* high)
{
    if (low <= (void*) HSD_RandSeedPtr && (void*) HSD_RandSeedPtr < high) {
        HSD_RandSeedPtr = &seed;
    }
    return;
}
