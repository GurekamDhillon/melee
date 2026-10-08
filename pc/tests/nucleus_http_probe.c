/* nucleus_http_probe.c - manual probe of gw_nucleus_http.inc: nucleus_http_probe <url> [max_bytes]. Prints the status, the body length and its first line.
 * Not a test (it needs the network); the build for it is in the comment of tools/port/native_test.sh (nucleus-http-probe). */
#include "../platform/gw_nucleus_http.inc"
int main(int argc, char **argv) {
    nch_result r;
    int rc;
    if (argc < 2) { printf("usage: nucleus_http_probe <url> [max_bytes]\n"); return 2; }
    if (getenv("MELEE_NUCLEUS_API")) nch_set_override(getenv("MELEE_NUCLEUS_API"));
    rc = nch_get(argv[1], argc > 2 ? (size_t) atol(argv[2]) : 1u << 20, &r);
    printf("rc=%d status=%d len=%u retry_after=%d err=%s\n", rc, r.status, (unsigned) r.len, r.retry_after, r.err);
    if (r.data) { printf("%.100s\n", r.data); free(r.data); }
    return rc;
}
