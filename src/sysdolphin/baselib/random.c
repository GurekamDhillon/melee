#include "random.h"

static u32 seed = 1;
u32* HSD_RandSeedPtr = &seed;

#if defined(TARGET_PC)
/* MELEE_STATE_TRACE (pc/platform/gw_replay.c): every draw during .slp playback, with its caller,
 * to <trace>.rand.csv - two runs of one replay diffed by this show which consumer made them part
 * ways. The caller is the native return address, resolved against melee-pc.map. */
extern void Replay_RandTrace(u32 caller, u32 seed_after, s32 global);
#define HSD_RAND_TRACE()                                                       \
    Replay_RandTrace((u32) __builtin_return_address(0), *HSD_RandSeedPtr,      \
                     HSD_RandSeedPtr == &seed)
#else
#define HSD_RAND_TRACE()
#endif

s32 HSD_Rand(void)
{
    *HSD_RandSeedPtr = *HSD_RandSeedPtr * 214013 + 2531011;
    HSD_RAND_TRACE();
    return *HSD_RandSeedPtr >> 0x10;
}

f32 HSD_Randf(void)
{
    *HSD_RandSeedPtr = *HSD_RandSeedPtr * 214013 + 2531011;
    HSD_RAND_TRACE();
    return (f32) (*HSD_RandSeedPtr >> 0x10) / (1 << 16);
}

s32 HSD_Randi(s32 max_val)
{
#if defined(TARGET_PC)
    /* the same draw as HSD_Rand, inlined so the trace names Randi's caller */
    *HSD_RandSeedPtr = *HSD_RandSeedPtr * 214013 + 2531011;
    HSD_RAND_TRACE();
    return max_val * (s32) (*HSD_RandSeedPtr >> 0x10) / (1 << 16);
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
