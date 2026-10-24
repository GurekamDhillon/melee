/* Source-owned statics and the private selector are observable in this driver.
 * Every row compares logical fields and pointer identities, never raw addresses. */
#include "../../../src/sysdolphin/baselib/random.c"
extern int printf(const char*, ...);

static u32 input_seed = 0x51A2E789;
static u32 redirect_seed;
static u32 trace_count, trace_seed, trace_stream;
static u32 next(void) {
    input_seed ^= input_seed << 13;
    input_seed ^= input_seed >> 17;
    input_seed ^= input_seed << 5;
    return input_seed;
}
void Replay_RandTrace(u32 caller, u32 value, s32 stream) {
    (void) caller;
    trace_count++;
    trace_seed = value;
    trace_stream = (u32) stream;
}
static u32 seed_id(u32* ptr) {
    return ptr == &seed ? 1 : ptr == &render_seed ? 2 : 3;
}
static void forget_range(u32 mode) {
    u64 p = (u64) HSD_RandSeedPtr;
    u64 lo = p, hi = p + 4;
    if (mode == 1) hi = p;
    if (mode == 2) { lo = p + 4; hi = p + 8; }
    if (mode == 3) { lo = p - 4; hi = p; }
    if (mode == 4) { lo = p - 4; hi = p + 4; }
    _HSD_RandForgetMemory((void*) lo, (void*) hi);
}
int main(void) {
    u32 i;
    for (i = 0; i < 8000; i++) {
        u32 op = i % 8, rc = 0, arg = next(), mode = (i / 8) % 5;
        static const u32 depths[] = {0, 1, 2, 0xFFFFFFFF, 0xFFFFFFFE};
        seed = next(); render_seed = next(); redirect_seed = next();
        render_depth = depths[(i / 40) % 5];
        HSD_RandSeedPtr = (i / 200) % 2 ? &redirect_seed : &seed;
        if (i % 13 == 0) arg = 0;
        if (i % 13 == 1) arg = 0x80000000;
        if (i % 13 == 2) arg = 0x7FFFFFFF;
        trace_count = trace_seed = trace_stream = 0;
        printf("%08x %08x %08x %08x %08x %08x %08x %08x", op,
               seed, render_seed, render_depth, redirect_seed,
               seed_id(HSD_RandSeedPtr), arg, mode);
        switch (op) {
        case 0: HSD_RandRenderBegin(); break;
        case 1: HSD_RandRenderEnd(); break;
        case 2: rc = seed_id(HSD_RandCurrentSeed()); break;
        case 3: rc = (u32) HSD_Rand(); break;
        case 4: { union { f32 f; u32 u; } value; value.f = HSD_Randf(); rc = value.u; break; }
        case 5: rc = (u32) HSD_Randi((s32) arg); break;
        case 6: forget_range(mode); break;
        case 7: rc = (u32) randi((s32) arg); break;
        }
        printf(" %08x %08x %08x %08x %08x %08x %08x %08x %08x %08x\n",
               rc, seed, render_seed, render_depth, redirect_seed,
               seed_id(HSD_RandSeedPtr), seed_id(HSD_RandCurrentSeed()),
               trace_count, trace_seed, trace_stream);
    }
    return 0;
}
