/* Included beside the runtime's other native headless tests. The PPC below is
 * a synthetic callback, not disc code: record the incoming GObj, flag and r2,
 * then return a fixture word. This catches a lost r4, a void/bool return, or
 * dispatch under the wrong fighter's ftFunction context without any archive. */
static int test_mex_onfloat_interpreter(void) {
    const uint32_t code = 0x802E0000u, data = 0x802E1000u;
    static const uint32_t callback[] = {
        0x90630000u, /* stw r3,0(r3) */
        0x90830004u, /* stw r4,4(r3) */
        0x90430008u, /* stw r2,8(r3) */
        0x8063000Cu, /* lwz r3,12(r3) */
        0x4E800020u, /* blr */
    };
    const int slots[2] = { GW_MEX_SLOTS - 2, GW_MEX_SLOTS - 1 };
    gw_mex_kind saved[2];
    gw_mex_kind *saved_kind = gw_mex_k;
    uint32_t saved_r2 = gw_mex_r2, saved_stack = gw_mex_stack_top;
    int failed = 0, i;
    unsigned j;

    for (i = 0; i < 2; ++i) {
        gw_mex_kind *k = &gw_mex_kinds[slots[i]];
        saved[i] = *k;
        memset(k, 0, sizeof(*k));
        k->port_kind = GW_PORT_FT_MEX0 + slots[i];
        k->ff.override_count = 1;
        k->ff.overrides[0].slot = 31u; /* Arch_FighterFunc +0x7C */
        k->ff.overrides[0].target = code + (uint32_t) i * sizeof(callback);
        for (j = 0; j < sizeof(callback) / sizeof(callback[0]); ++j) {
            gw_w32((void *) (uintptr_t) (k->ff.overrides[0].target + j * 4u), callback[j]);
        }
        gw_Mex_PredicateRegister(GW_MEX_EVENT_ON_FLOAT, k->port_kind, gw_mex_interp_onfloat);
    }
    /* The second kind returns a different word, so selecting the first blob by
     * accident cannot pass simply because both kinds use the same native shim. */
    gw_w32((void *) (uintptr_t) (code + sizeof(callback) + 12u), 0x80630010u);
    gw_w32((void *) (uintptr_t) (data + 12u), 7u);
    gw_w32((void *) (uintptr_t) (data + 16u), 9u);
    gw_mex_r2 = 0x803E0000u;
    gw_mex_stack_top = 0x803EF000u;
    gw_mex_k = &gw_mex_kinds[0];
    gw_ppc_add_code_range(code, code + 2u * sizeof(callback));

#define ONFLOAT_CHECK(cond, msg) \
    do { if (!(cond)) { gw_test_fail("%s", msg); failed = 1; goto done; } } while (0)
    for (i = 0; i < 2; ++i) {
        int kind = GW_PORT_FT_MEX0 + slots[i];
        uint32_t want = i == 0 ? 7u : 9u;
        ONFLOAT_CHECK((uint32_t) gw_Mex_OnFloatDispatch(kind, (void *) (uintptr_t) data) == want,
                      "onFloat lost the guest r3 result or selected another kind's blob");
        ONFLOAT_CHECK(gw_r32((void *) (uintptr_t) data) == data &&
                      gw_r32((void *) (uintptr_t) (data + 4u)) == 1u &&
                      gw_r32((void *) (uintptr_t) (data + 8u)) == gw_mex_r2,
                      "onFloat must receive r3=GObj, r4=1, r2=mexData");
        ONFLOAT_CHECK(gw_mex_k == &gw_mex_kinds[0],
                      "onFloat left another kind selected after dispatch");
    }
    gw_w32((void *) (uintptr_t) (data + 12u), 0u);
    ONFLOAT_CHECK(gw_Mex_OnFloatDispatch(GW_PORT_FT_MEX0 + slots[0],
                                       (void *) (uintptr_t) data) == 0,
                  "onFloat treated a guest false return as successful float");
    gw_mex_kinds[slots[0]].ff.override_count = 0;
    ONFLOAT_CHECK(gw_Mex_OnFloatDispatch(GW_PORT_FT_MEX0 + slots[0],
                                       (void *) (uintptr_t) data) == 0,
                  "onFloat without an override must return false");
    ONFLOAT_CHECK(gw_mex_slot_is_wired(31u), "onFloat still reported as an unwired slot");
done:
    for (i = 0; i < 2; ++i) {
        gw_Mex_PredicateRegister(GW_MEX_EVENT_ON_FLOAT, GW_PORT_FT_MEX0 + slots[i], NULL);
        gw_mex_kinds[slots[i]] = saved[i];
    }
    gw_mex_k = saved_kind;
    gw_mex_r2 = saved_r2;
    gw_mex_stack_top = saved_stack;
    gw_ppc_remove_code_range(code, code + 2u * sizeof(callback));
#undef ONFLOAT_CHECK
    return failed;
}
