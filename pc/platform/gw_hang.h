/* Diagnostic-only host state. Never saved, read by simulation or used for pacing. */
#ifndef GW_HANG_H
#define GW_HANG_H
#include <stdint.h>
#ifdef __cplusplus
extern "C" {
#endif
#define GW_HANG_EXIT 86
void gw_hang_start(void);
void gw_hang_tick(void);
void gw_hang_logic(void);
void gw_hang_pause(int flags); /* 1 manual/LAB/step, 2 timed hitstop/freeze */
void gw_hang_scene(int scene);
void gw_hang_transition(int phase); /* 1 launch requested; 2 new scene; 0 cancel */
void gw_hang_shim(const char *name); /* string literal only */
void gw_hang_callback(const char *owner, const char *name, int active);
void gw_hang_instructions(void);
void gw_hang_script_state(int pending, int expired, const char *last);
void gw_hang_log_time(void);
void gw_hang_final(const char *reason, unsigned code);
void gw_hang_test_stall(unsigned seconds); /* explicit opt-in test command */
void gw_hang_tests_register(void);
/* Existing crash-handler module/RVA resolution without logging/stdio locks. */
const char *gw_hang_describe_addr(uintptr_t pc, char *out, unsigned cap);
#ifdef __cplusplus
}
#endif
#endif
