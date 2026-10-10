/* Original synthetic FigaTree with one joint and one constant TRAX track.
 * No disc data: covers layout, Locate's relocation rules and fobj.c decoding. */
static void gn_anim_fixture(GenoPlan* plan, unsigned char bank[160])
{
    memset(plan, 0, sizeof *plan);
    memset(bank, 0, 160);
    plan->ok = 1; plan->joints = 1; plan->nclips = 1;
    strcpy(plan->clip_name[0], "Wait");
    strcpy(plan->clip_symbol[0], "test_figatree");
    plan->clip_bytes[0] = 105; plan->clip_frames[0] = 2;
    gw_w32(bank, 105); gw_w32(bank + 4, 40);
    gw_w32(bank + 8, 3); gw_w32(bank + 12, 1);
    /* data: tree [0,20), nodes [20,22), track [24,36), keys [36,38). */
    gw_w32(bank + 32, 1); gw_w32(bank + 40, 0x3f800000);
    gw_w32(bank + 44, 20); gw_w32(bank + 48, 24);
    bank[52] = 1; bank[53] = 255;
    bank[57] = 2; bank[60] = 5; bank[61] = 0x60;
    gw_w32(bank + 64, 36); bank[68] = 1; bank[69] = 7;
    gw_w32(bank + 72, 12); gw_w32(bank + 76, 16); gw_w32(bank + 80, 32);
    memcpy(bank + 92, "test_figatree", 13);
}
static int test_geno_anim_bank_layout(void)
{
    static GenoPlan plan;
    unsigned char bank[160];
    char why[200];
    int i;
    struct { int at; unsigned value; } bad[] = {
        {0, 104}, {4, 0xffffffffu}, {8, 0xffffffffu}, {12, 0xffffffffu},
        {16, 1}, {84, 40}, {88, 13}
    };
    gn_anim_fixture(&plan, bank);
    if (!gn_anim_bank_validate(&plan, bank, 128, 0x20000, why, sizeof why)) {
        gw_test_fail("valid synthetic bank refused: %s", why); return 1;
    }
    bank[69] = 17; /* same-layout pose bytes may change */
    if (!gn_anim_bank_validate(&plan, bank, 128, 0x20000, why, sizeof why)) {
        gw_test_fail("same-layout content edit refused: %s", why); return 1;
    }
    for (i = 0; i < (int)(sizeof bad / sizeof bad[0]); ++i) {
        gn_anim_fixture(&plan, bank); gw_w32(bank + bad[i].at, bad[i].value);
        if (gn_anim_bank_validate(&plan, bank, 128, 0x20000, why, sizeof why) || !why[0]) {
            gw_test_fail("corrupt archive field %d accepted or no reason", bad[i].at); return 1;
        }
    }
    gn_anim_fixture(&plan, bank); bank[92] = 'x';
    if (gn_anim_bank_validate(&plan, bank, 128, 0x20000, why, sizeof why)) {
        gw_test_fail("missing plan symbol accepted"); return 1;
    }
    gn_anim_fixture(&plan, bank); bank[104] = 'x';
    if (gn_anim_bank_validate(&plan, bank, 128, 0x20000, why, sizeof why)) {
        gw_test_fail("unterminated symbol accepted"); return 1;
    }
    gn_anim_fixture(&plan, bank);
    if (gn_anim_bank_validate(&plan, bank, 104, 0x20000, why, sizeof why) ||
        gn_anim_bank_validate(&plan, bank, 160, 0x20000, why, sizeof why) ||
        gn_anim_bank_validate(&plan, bank, 128, 104, why, sizeof why)) {
        gw_test_fail("truncation, changed bank size or oversized clip accepted"); return 1;
    }
    plan.clip_offset[0] = 0xffffffe0u;
    if (gn_anim_bank_validate(&plan, bank, 128, 0x20000, why, sizeof why)) {
        gw_test_fail("overflowing clip offset accepted"); return 1;
    }
    gn_anim_fixture(&plan, bank); plan.ok = 0;
    if (gn_anim_bank_validate(&plan, bank, 128, 0x20000, why, sizeof why)) {
        gw_test_fail("unusable plan accepted"); return 1;
    }
    return 0;
}
static int test_geno_anim_bank_relocations(void)
{
    static GenoPlan plan;
    unsigned char bank[160];
    char why[200];
    int i;
    struct { int at; unsigned value; } bad[] = {
        {72, 38}, /* misaligned/out-of-bounds pointer field */
        {44, 40}, /* target beyond data */
        {76, 12}, /* double relocation: Locate would add the base twice */
        {80, 4}   /* scalar relocation instead of required track.ad */
    };
    gn_anim_fixture(&plan, bank);
    gw_w32(bank + 72, 32); gw_w32(bank + 80, 12);
    if (!gn_anim_bank_validate(&plan, bank, 128, 0x20000, why, sizeof why)) {
        gw_test_fail("valid unordered relocations refused: %s", why); return 1;
    }
    for (i = 0; i < (int)(sizeof bad / sizeof bad[0]); ++i) {
        gn_anim_fixture(&plan, bank); gw_w32(bank + bad[i].at, bad[i].value);
        if (gn_anim_bank_validate(&plan, bank, 128, 0x20000, why, sizeof why)) {
            gw_test_fail("bad relocation case %d accepted", i); return 1;
        }
    }
    return 0;
}
static int test_geno_anim_bank_figatree(void)
{
    static GenoPlan plan;
    unsigned char bank[160];
    char why[200];
    int i;
    struct { int at; unsigned value; } bad_words[] = {
        {84, 36}, /* incomplete root */
        {44, 39}, /* unterminated nodes */
        {48, 36}, /* incomplete track */
        {64, 39}, /* stream overruns data */
        {64, 32}, /* stream aliases a relocated pointer */
        {40, 0x7f800000} /* infinite end frame */
    };
    struct { int at; unsigned char value; } bad_bytes[] = {
        {52, 2}, {53, 0}, {52, 128}, /* track count / missing terminator / negative count */
        {57, 5}, {60, 0}, {61, 0xa0}, /* length / channel / fraction type */
        {68, 0}, {68, 0x11}, {68, 0x81} /* opcode / too many keys / truncated pack */
    };
    for (i = 0; i < (int)(sizeof bad_words / sizeof bad_words[0]); ++i) {
        gn_anim_fixture(&plan, bank); gw_w32(bank + bad_words[i].at, bad_words[i].value);
        if (gn_anim_bank_validate(&plan, bank, 128, 0x20000, why, sizeof why)) {
            gw_test_fail("unsafe figatree word %d accepted", i); return 1;
        }
    }
    for (i = 0; i < (int)(sizeof bad_bytes / sizeof bad_bytes[0]); ++i) {
        gn_anim_fixture(&plan, bank); bank[bad_bytes[i].at] = bad_bytes[i].value;
        if (gn_anim_bank_validate(&plan, bank, 128, 0x20000, why, sizeof why)) {
            gw_test_fail("unsafe figatree byte %d accepted", i); return 1;
        }
    }
    gn_anim_fixture(&plan, bank); plan.joints = 2;
    if (gn_anim_bank_validate(&plan, bank, 128, 0x20000, why, sizeof why)) {
        gw_test_fail("changed skeleton layout accepted"); return 1;
    }
    return 0;
}
static int test_geno_anim_bank_request(void)
{
    char why[200];
    int phase = gn_anim_reload.phase;
    uint64_t digest = gn_anim_digest;
    if (gw_Geno_ReloadAnimBank("__missing_define__", why, sizeof why) || !why[0] ||
        gn_anim_reload.phase != phase || gn_anim_digest != digest) {
        gw_test_fail("unknown define changed reload state or lacked an error"); return 1;
    }
    if (gn_anim_reload_policy(1, 0, 0) || gn_anim_reload_policy(0, 1, 0) ||
        gn_anim_reload_policy(0, 0, 1) || !gn_anim_reload_policy(0, 0, 0)) {
        gw_test_fail("animation reload admitted netplay, rollback or resimulation"); return 1;
    }
    return 0;
}

static int test_geno_anim_bank_caches(void)
{
    unsigned char bank[16] = {0}, scratch_saved[16], saved[sizeof gn_anim_reload];
    void* scratch;
    uint64_t saved_digest = gn_anim_digest;
    int ck, mode, rc = 0;
    extern int gw_ftData_AnimBankCacheTest(int kind, int mode, unsigned address);
    extern void* gw_guest_scratch(uint32_t size);
    if (gn_swapped || !gn_anim_offline() || !gw_mem1) {
        gw_test_fail("animation cache fixture requires the offline test registry"); return 1;
    }
    /* Retargeted static game arrays live in exe BSS, not MEM1. Borrow a real
     * guest scratch slot; this path invokes no nested animation callbacks. */
    scratch = gw_guest_scratch(sizeof bank);
    memcpy(scratch_saved, scratch, sizeof scratch_saved);
    memcpy(saved, &gn_anim_reload, sizeof saved);
    memset(&gn_anim_reload, 0, sizeof gn_anim_reload);
    if (gw_Geno_TestInstall("{\"geno\":9,\"fighters\":[{\"define\":{\"key\":\"anim-test\",\"name\":\"Anim Test\","
        "\"base\":\"none\",\"common\":\"melee.common.v1\",\"resources\":\"mod:files\"},"
        "\"fighter\":{\"plan\":\"test.json\",\"animation\":\"__synthetic_anim_bank__.dat\","
        "\"costumes\":[{\"file\":\"test.dat\",\"joint\":\"test_joint\"}]}}]}") != 1 ||
        (ck = gw_Geno_DefineResolve("anim-test")) <= 0) { rc = 1; goto done; }
    bank[9] = 17;
    /* Live user, stale length, rejected guest copy, success + staged load. */
    for (mode = 0; mode < 4; ++mode) {
        gn_anim_reload.bytes = (unsigned char*)malloc(sizeof bank);
        if (!gn_anim_reload.bytes) { rc = 1; break; }
        memcpy(gn_anim_reload.bytes, bank, sizeof bank);
        gn_anim_reload.kind = ck - 1; gn_anim_reload.size = sizeof bank;
        gn_anim_reload.phase = 1; gn_anim_digest = saved_digest;
        if (gw_ftData_AnimBankCacheTest(ck - 1, mode, (unsigned)(uintptr_t)scratch) ||
            (mode == 3 ? gn_anim_digest == saved_digest : gn_anim_digest != saved_digest) ||
            (mode == 0 ? gn_anim_reload.bytes == NULL : gn_anim_reload.bytes != NULL) ||
            (mode == 1 && !strstr(gw_Geno_AnimBankStatus(), "length")) ||
            (mode == 2 && !strstr(gw_Geno_AnimBankStatus(), "copy"))) {
            gw_test_fail("animation bank lifecycle case %d", mode); rc = 1;
        }
        free(gn_anim_reload.bytes); gn_anim_reload.bytes = NULL;
    }
done:
    gw_Geno_TestRestore();
    free(gn_anim_reload.bytes);
    memcpy(&gn_anim_reload, saved, sizeof saved); gn_anim_digest = saved_digest;
    memcpy(scratch, scratch_saved, sizeof scratch_saved);
    return rc;
}
