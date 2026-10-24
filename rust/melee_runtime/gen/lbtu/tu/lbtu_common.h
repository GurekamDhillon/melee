/* Shared by gen/lbtu/tu/<stem>.c (included after mtu_common.h). Mirrored by tests/lbtu_harness/mod.rs. */
static void TVEC(const void* p)
{
    const float* v = (const float*) p;
    if (v) { T(B(v[0])); T(B(v[1])); T(B(v[2])); }
}

/* sprintf of the TU (the ucrt one is header-inline only): forwards to vsprintf */
extern int __stdio_common_vsprintf(unsigned long long, char*, unsigned long long, const char*, void*, __builtin_va_list);
int sprintf(char* s, const char* f, ...)
{
    __builtin_va_list ap;
    int r;
    __builtin_va_start(ap, f);
    r = __stdio_common_vsprintf(1ull, s, (unsigned long long) -1, f, 0, ap);
    __builtin_va_end(ap);
    return r;
}
