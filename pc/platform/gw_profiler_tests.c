/* In-engine suite tests use only the public native diagnostics API. */
#include "gw_profiler.h"
#include "gw_test.h"
static int profiler_percentiles(void) {
    GwProfStats s;
    int enabled = gw_prof_enabled(), i, failed;
    gw_prof_set_enabled(1); gw_prof_reset();
    for (i = 1; i <= 100; ++i) gw_prof_sample(GW_PROF_GPU, i, 1);
    gw_prof_stats(GW_PROF_GPU, &s);
    failed = s.count != 100 || s.mean != 50.5 || s.p50 != 50 || s.p95 != 95 || s.p99 != 99 || s.max != 100;
    gw_prof_reset(); gw_prof_set_enabled(enabled);
    if (failed) gw_test_fail("nearest-rank GPU samples differ from 1..100 expected distribution");
    return failed;
}
static int profiler_nested_overflow(void) {
    GwProfStats s;
    int enabled = gw_prof_enabled(), i, failed;
    gw_prof_set_enabled(1); gw_prof_reset();
    for (i = 0; i < 73; ++i) gw_prof_begin(GW_PROF_FIGHTER, 1);
    for (i = 0; i < 73; ++i) gw_prof_end();
    gw_prof_stats(GW_PROF_FIGHTER, &s);
    failed = s.count != 64 || s.max < s.p99 || s.p99 < s.p95 || s.p95 < s.p50;
    gw_prof_begin(GW_PROF_INPUT, 0); gw_prof_end();
    gw_prof_stats(GW_PROF_INPUT, &s);
    failed |= s.count != 1;
    gw_prof_reset(); gw_prof_set_enabled(enabled);
    if (failed) gw_test_fail("bounded nested zones failed to unwind after depth overflow");
    return failed;
}
static int profiler_disabled_storage(void) {
    GwProfStats s;
    int enabled = gw_prof_enabled(), i, failed;
    gw_prof_set_enabled(0); gw_prof_reset();
    for (i = 0; i < 10000; ++i) {
        gw_prof_begin(GW_PROF_LOGIC, 0); gw_prof_end();
        gw_prof_sample(GW_PROF_GPU, 1, 0); gw_prof_counter(GW_PROF_VERTICES, 1);
        gw_prof_frame_begin(); gw_prof_frame_end();
    }
    gw_prof_stats(GW_PROF_LOGIC, &s); failed = s.count != 0;
    gw_prof_stats(GW_PROF_GPU, &s); failed |= s.count != 0;
    gw_prof_stats(GW_PROF_VERTICES, &s); failed |= s.count != 0;
    failed |= gw_prof_frames() != 0;
    gw_prof_set_enabled(enabled);
    if (failed) gw_test_fail("disabled diagnostics wrote measurements");
    return failed;
}
void gw_profiler_tests_register(void) {
    gw_test_register("profiler_nearest_rank", profiler_percentiles);
    gw_test_register("profiler_nesting_overflow", profiler_nested_overflow);
    gw_test_register("profiler_disabled_storage", profiler_disabled_storage);
}
