/* Offline bank authoring, included after geno_define_registry.inc. The plan,
 * definition identities and netplay hash stay fixed. A single pinned candidate
 * prevents a second disk edit between validation and the next spawn being used.
 * ftData_AnimBankReload owns cache invalidation and requires an empty fighter list. */
#include "geno_anim_bank.h"
#include "geno_anim_bank_core.h"
extern int gw_RB_Enabled(void);
extern int gw_Netplay_Enabled(void);
extern int gw_Snap_Resimulating(void);
extern int gw_ftData_AnimBankLimit(void);
extern void gw_Script_AnimBankReloaded(void);
extern int gw_ftData_AnimBankReload(int kind, unsigned size);
extern void* gw_DVDReadModFileFresh(const char* path, uint32_t* size, char* why, int cap);
static struct {
    unsigned char* bytes;
    uint32_t size;
    int kind, phase, applying;
    char text[256];
} gn_anim_reload;
static uint64_t gn_anim_digest;
static int gn_anim_offline(void)
{
    return gn_anim_reload_policy(gw_Netplay_Enabled(), gw_RB_Enabled(), gw_Snap_Resimulating());
}
int gw_Geno_AnimBankPhase(void) { return gn_anim_reload.phase; }
const char* gw_Geno_AnimBankStatus(void) { return gn_anim_reload.text; }
uint64_t gw_Geno_AnimBankDigest(void) { return gn_anim_digest; }
void gw_Geno_AnimBankCancel(void)
{
    free(gn_anim_reload.bytes); gn_anim_reload.bytes = NULL;
    gn_anim_reload.phase = 3;
    snprintf(gn_anim_reload.text, sizeof gn_anim_reload.text, "cancelled; current bank retained");
    gw_log("geno: anim bank reload: %s", gn_anim_reload.text);
}
/* The destination is a game address (MEM1 or ARAM), not a native buffer handed
 * to the interpreter. Raw archive bytes are already big-endian, like DVD DMA. */
int gw_Geno_AnimBankCopy(int kind, unsigned address, unsigned size)
{
    unsigned char* dst;
    if (!gn_anim_reload.applying || !gn_anim_offline() || !gn_anim_reload.bytes ||
        gn_anim_reload.kind != kind || gn_anim_reload.size != size) return 0;
    if (address < 0x80000000u) {
        if (!gw_aram || address > gw_aram_size || size > gw_aram_size - address) return 0;
        dst = gw_aram + address;
    } else {
        unsigned off = address - 0x80000000u;
        if (!gw_mem1 || off > gw_mem1_size || size > gw_mem1_size - off) return 0;
        dst = gw_mem1 + off;
    }
    memcpy(dst, gn_anim_reload.bytes, size);
    return 1;
}
void gw_Geno_AnimBankBeforeSpawn(void)
{
    int rc;
    uint32_t i;
    if (gn_anim_reload.phase != 1 || gn_anim_reload.applying) return;
    if (!gn_anim_offline()) {
        gw_Geno_AnimBankCancel();
        snprintf(gn_anim_reload.text, sizeof gn_anim_reload.text, "offline-only; pending reload cancelled");
        gw_log("geno: anim bank reload: %s", gn_anim_reload.text);
        return;
    }
    gn_anim_reload.applying = 1;
    rc = gw_ftData_AnimBankReload(gn_anim_reload.kind, gn_anim_reload.size);
    gn_anim_reload.applying = 0;
    if (!rc) return; /* some fighter still owns animation data */
    if (rc > 0) {
        uint64_t h = gn_anim_digest ? gn_anim_digest : UINT64_C(14695981039346656037);
        h = (h ^ (unsigned)gn_anim_reload.kind) * UINT64_C(1099511628211);
        for (i = 0; i < gn_anim_reload.size; ++i)
            h = (h ^ gn_anim_reload.bytes[i]) * UINT64_C(1099511628211);
        gn_anim_digest = h;
        gw_Script_AnimBankReloaded();
        gn_anim_reload.phase = 2;
        snprintf(gn_anim_reload.text, sizeof gn_anim_reload.text,
                 "kind %d ready: validated bank installed; animation caches cleared", gn_anim_reload.kind);
    } else {
        const char* reason;
        switch (rc) {
        case -1: reason = "define descriptor is unavailable"; break;
        case -2: reason = "animation storage or file is shared by another fighter"; break;
        case -3: reason = "cached/preloaded bank length differs from the validated file"; break;
        case -4: reason = "no resident/preloaded bank; load the target fighter first"; break;
        case -5: reason = "guest copy refused (address bounds or offline policy)"; break;
        default: reason = "game cache invalidation refused"; break;
        }
        gn_anim_reload.phase = 3;
        snprintf(gn_anim_reload.text, sizeof gn_anim_reload.text,
                 "kind %d reload refused: %s; restart required", gn_anim_reload.kind, reason);
    }
    free(gn_anim_reload.bytes); gn_anim_reload.bytes = NULL;
    gw_log("geno: anim bank reload: %s", gn_anim_reload.text);
}
int gw_Geno_ReloadAnimBank(const char* define, char* why, int cap)
{
    int ck, kind, i;
    uint32_t size = 0;
    unsigned char* bytes;
    struct gn_fm* fm;
    const GenoPlan* plan;
    if (!gn_anim_offline()) {
        snprintf(why, cap, "animation bank reload is offline-only (netplay/rollback/resimulation refused)");
        goto refused;
    }
    if (gn_anim_reload.phase == 1) {
        snprintf(why, cap, "an animation bank reload is already pending"); goto refused;
    }
    ck = define ? gw_Geno_DefineResolve(define) : -1; kind = ck - 1;
    fm = ck > 0 ? gn_fm_for_kind(kind) : NULL;
    if (!fm) { snprintf(why, cap, "define must name an installed Geno fighter with its own animation bank"); goto refused; }
    plan = gn_define_plan(kind);
    if (!plan) { snprintf(why, cap, "define has no usable loaded plan; restart required"); goto refused; }
    /* Updating the DVD preload storage must not silently change another define. */
    for (i = 0; i < gw_Geno_DefineCount(); ++i) {
        int other = gw_Geno_DefineCKAt(i) - 1;
        struct gn_fm* shared = gn_fm_for_kind(other);
        if (other != kind && shared && !_stricmp(fm->anim_file, shared->anim_file)) {
            snprintf(why, cap, "animation bank is shared by another define; restart required"); goto refused;
        }
    }
    bytes = (unsigned char*)gw_DVDReadModFileFresh(fm->anim_file, &size, why, cap);
    if (!bytes) goto refused;
    if (!gn_anim_bank_validate(plan, bytes, size, (unsigned)gw_ftData_AnimBankLimit(), why, (size_t)cap)) {
        free(bytes); goto refused;
    }
    gn_anim_reload.bytes = bytes; gn_anim_reload.size = size;
    gn_anim_reload.kind = kind; gn_anim_reload.phase = 1;
    snprintf(gn_anim_reload.text, sizeof gn_anim_reload.text,
             "kind %d pending: restart/leave the match to release animation users", kind);
    snprintf(why, cap, "%s", gn_anim_reload.text);
    gw_log("geno: anim bank reload: %s; %s (%u bytes) validated against %s",
           why, fm->anim_file, size, fm->plan_file);
    return 1;
refused:
    gw_log("geno: anim bank reload refused: %s", why);
    return 0;
}
