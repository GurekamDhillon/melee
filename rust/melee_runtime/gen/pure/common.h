/* shared by the pure oracle drivers: xorshift64 inputs and FNV-1a64 hashing of outputs */
#include <dolphin/types.h>
typedef unsigned long long u64_;
static u64_ g_s = 0x9E3779B97F4A7C15ULL;
static u64_ rnd(void) { g_s ^= g_s << 13; g_s ^= g_s >> 7; g_s ^= g_s << 17; return g_s; }
static float rndf(void) {
    u64_ r = rnd();
    switch ((int)(r % 16)) {
    case 0: return 0.0f;
    case 1: return -0.0f;
    case 2: return 1.0f;
    case 3: return -1.0f;
    case 4: return (float)((int)(rnd() % 2001) - 1000) * 0.001f;
    case 5: return (float)((int)(rnd() % 20001) - 10000);
    case 6: return (float)((int)(rnd() % 2001) - 1000) * 1e-20f;
    case 7: return (float)((int)(rnd() % 2001) - 1000) * 1e12f;
    default: return (float)((int)(rnd() % 2000001) - 1000000) * 0.0001f;
    }
}
static u64_ fnv(const void* p, int n, u64_ h) {
    const unsigned char* b = (const unsigned char*)p;
    int i;
    for (i = 0; i < n; i++) { h ^= b[i]; h *= 0x100000001B3ULL; }
    return h;
}
#define FNV0 0xCBF29CE484222325ULL
