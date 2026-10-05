/* Lazy profile descriptors in a fixed byte-budget arena, not the HSD heap.
 * Arena bytes, cursor, pointer tables and capacities are section-3 game globals:
 * gw_snap captures/restores them together, including in the headless suite.
 * The 2 MiB budget is explicit; profile ids still grow to GENO_MAX_PROFILES.
 * Allocation occurs only on first use/growth, never per action/frame thereafter.
 */
#include "geno_profile_storage.h"
#include <sysdolphin/baselib/debug.h>
extern int Geno_ProfileCount(void);
MotionState** Geno_Rows;
f32** Geno_Params;
s32* Geno_RowCount;
int Geno_ProfileCapacity;
static union {
    u64 align;
    unsigned char bytes[GENO_PROFILE_STORAGE_BYTES];
} Geno_ProfileStorage;
static unsigned int Geno_ProfileStorageUsed;

unsigned Geno_ProfileStorageBytes(void) { return Geno_ProfileStorageUsed; }
static void* geno_profile_alloc(int bytes)
{
    void* p = geno_storage_take(Geno_ProfileStorage.bytes, GENO_PROFILE_STORAGE_BYTES,
                               &Geno_ProfileStorageUsed, bytes);
    if (p == NULL) {
        OSReport("geno: profile descriptor storage exhausted: requested %d bytes, used %u of %u; refused\n",
                 bytes, Geno_ProfileStorageUsed, GENO_PROFILE_STORAGE_BYTES);
        __assert(__FILE__, __LINE__, "Geno profile descriptor byte budget exhausted");
        return NULL;
    }
    geno_zero(p, bytes); return p;
}
static void geno_profiles_prepare(void)
{
    int n = Geno_ProfileCount();
    if (n <= Geno_ProfileCapacity) return;
    /* Keep old rows valid until scene reset. Copy pointers so existing rows
       retain their addresses. The arena cursor is restored by snapshots too. */
    MotionState** rows = geno_profile_alloc(n * sizeof(*rows));
    f32** params = geno_profile_alloc(n * sizeof(*params));
    s32* counts = geno_profile_alloc(n * sizeof(*counts));
    int i;
    for (i = 0; i < Geno_ProfileCapacity; ++i) {
        rows[i] = Geno_Rows[i]; params[i] = Geno_Params[i]; counts[i] = Geno_RowCount[i];
    }
    Geno_Rows = rows; Geno_Params = params; Geno_RowCount = counts; Geno_ProfileCapacity = n;
}
static void geno_profile_prepare(int p)
{
    geno_profiles_prepare();
    if (p < 0 || p >= Geno_ProfileCapacity) return;
    if (!Geno_Rows[p]) Geno_Rows[p] = geno_profile_alloc(GENO_MAX_STATES * sizeof(MotionState));
    if (!Geno_Params[p]) Geno_Params[p] = geno_profile_alloc(GENO_PARAMS * sizeof(f32));
}
