/* Offline runner for the kit's existing tests; no guest memory, disc or GPU. */
#include "../platform/gw_kit.c"
#include <assert.h>

void gw_log(const char *fmt, ...) {
    va_list ap;
    va_start(ap, fmt); vprintf(fmt, ap); va_end(ap); puts("");
}
void gw_test_fail(const char *fmt, ...) {
    va_list ap;
    va_start(ap, fmt); vfprintf(stderr, fmt, ap); va_end(ap); fputs("\n", stderr);
}
int gw_UiFile_Read(const char *name, void *dst, int cap) {
    (void)name; (void)dst; (void)cap;
    return -1;
}
void gw_test_register(const char *name, gw_test_fn fn) {
    if (!strcmp(name, "kit_text_layout") || !strcmp(name, "kit_panel_and_row") ||
        !strcmp(name, "kit_poly4_and_tracking") || !strcmp(name, "kit_atlas_roles")) {
        printf("SKIP %s (generated UI assets unavailable in offline fixture)\n", name);
        return;
    }
    assert(fn() == 0);
    printf("PASS %s\n", name);
}
void kit_native_tests(void) { gw_kit_tests_register(); }
