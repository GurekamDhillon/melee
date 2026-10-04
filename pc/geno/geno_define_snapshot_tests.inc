/* Snapshot-visible native-definition diagnostics and identity. Runtime suite
 * execution is still required after the integrator rebuild. */
extern u32 GenoDefine_ContentIdentity[Ft_Kind_Max][2];
extern u32 GenoDefine_InterpreterAttempts[Ft_Kind_Max];
extern void GenoGame_DefinePPCViolation(int kind);
static int test_geno_define_snapshot(void)
{
    int kind = Ft_Kind_Mex0, rc = 0;
    u32 old0 = GenoDefine_ContentIdentity[kind][0];
    u32 old1 = GenoDefine_ContentIdentity[kind][1];
    u32 attempts = GenoDefine_InterpreterAttempts[kind];
    GenoDefine_ContentIdentity[kind][0] = 0x47454e4f;
    GenoDefine_ContentIdentity[kind][1] = 0x44454631;
    if (snap_open(1) != 0) {
        TestFail("native definition snapshot could not open"); rc = 1;
    } else {
        snap_save(0);
        GenoDefine_ContentIdentity[kind][0] = 0;
        GenoDefine_ContentIdentity[kind][1] = 0;
        GenoGame_DefinePPCViolation(kind);
        if (GenoDefine_InterpreterAttempts[kind] != attempts+1) {
            TestFail("native definition interpreter-attempt counter did not increment"); rc = 1;
        }
        if (snap_load(0) != 0 || GenoDefine_ContentIdentity[kind][0] != 0x47454e4f ||
            GenoDefine_ContentIdentity[kind][1] != 0x44454631 ||
            GenoDefine_InterpreterAttempts[kind] != attempts) {
            TestFail("native definition identity/counter not restored by snapshot"); rc = 1;
        }
    }
    GenoDefine_ContentIdentity[kind][0] = old0;
    GenoDefine_ContentIdentity[kind][1] = old1;
    GenoDefine_InterpreterAttempts[kind] = attempts;
    return rc;
}
