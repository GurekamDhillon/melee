/* Shared by gen/lbtu/tu/<stem>.c (included after mtu_common.h). Mirrored by tests/lbtu_harness/mod.rs. */
static void TVEC(const void* p)
{
    const float* v = (const float*) p;
    if (v) { T(B(v[0])); T(B(v[1])); T(B(v[2])); }
}
