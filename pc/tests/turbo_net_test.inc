/* Reuse the real transport and its existing deterministic two-peer fixtures.
 * No socket is opened: the selected cases supply the simulated transport.
 * Include units here because native_test.sh's source list is read-only. */
#include "../platform/gw_net.c"
#include "../platform/gw_net_tests.c"
#include <stdarg.h>

void gw_test_fail(const char* fmt, ...)
{
    va_list args;
    va_start(args, fmt);
    vfprintf(stderr, fmt, args);
    va_end(args);
    fputc('\n', stderr);
}

void gw_test_register(const char* name, gw_test_fn fn)
{
    (void) name;
    (void) fn;
}

static void turbo_net_native_test(void)
{
    assert(!test_handshake());
    assert(!test_rules_agree());
    assert(!test_refuse_rules());
    assert(!test_refuse_rules2());
    puts("turbo handshake: standard, host adoption, agreement, OFF mismatch, near-miss refusal PASS");
}
