#ifndef GW_GC_ADAPTER_POLICY_H
#define GW_GC_ADAPTER_POLICY_H
#include <stdlib.h>
static inline int gw_gc_env_on(const char *key) {
    const char *v=getenv(key);return v && v[0]=='1';
}
static inline int gw_gc_forbidden(void) {
    return gw_gc_env_on("MELEE_PAD_IGNORE_ADAPTER") || gw_gc_env_on("MELEE_UNATTENDED");
}
static inline int gw_gc_policy_allows(int ignored, int unattended, int focused) {
    return !ignored && !unattended && focused;
}
#endif
