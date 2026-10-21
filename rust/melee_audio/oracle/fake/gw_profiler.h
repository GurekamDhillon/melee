#pragma once
#define GW_PROF_AUDIO 0
#define GW_PROF_AUDIO_MIX 0
static inline int gw_prof_enabled(void) { return 0; }
static inline double gw_prof_clock_ms(void) { return 0; }
static inline void gw_prof_try_cpu_completed(int a, int b, double c) { (void)a; (void)b; (void)c; }
static inline void gw_prof_begin(int a, int b) { (void)a; (void)b; }
static inline void gw_prof_end(void) {}
