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

/* Slice 2 (D6): a define's declared move tag wins over the tag its motion id derives; a retail
 * fighter and an attach fighter keep the old order (the motion-id tag first). Motion 53 is the
 * forward tilt (tag 3 = tilt, 4 = smash). */
extern int ScriptGame_HitFamilyMove(Fighter* fp);
extern int Geno_DefineCount(void);
extern int Geno_DefineCKAt(int index);
static int test_geno_define_hit_tag(void)
{
    int rc = 0, ck, kind;
    t_setup();
    /* retail Mario: the forward tilt is a tilt */
    Geno_TestInstall("{\"geno\":6,\"fighters\":[]}");
    t_fp.kind = Ft_Kind_Mario; t_fp.motion_id = 53; t_fp.anim_id = 46;
    if (ScriptGame_HitFamilyMove(&t_fp) != 3) { TestFail("retail Mario tilt not reported as tilt"); rc = 1; }
    Geno_TestRestore();
    /* an attach fighter with an overlay declared smash: the motion-id tag still wins in a tilt state,
       the declaration answers in a state the motion id does not classify */
    t_setup();
    Geno_TestInstall("{\"geno\":6,\"fighters\":[{\"attach\":\"kirby\",\"subactions\":[{\"index\":46,"
        "\"words\":[\"0x04000002\"],\"move_tag\":\"smash\"}]}]}");
    t_fp.kind = Ft_Kind_Kirby; t_fp.motion_id = 53; t_fp.anim_id = 46;
    if (ScriptGame_HitFamilyMove(&t_fp) != 3) { TestFail("attach fighter: motion tag must still win"); rc = 1; }
    t_fp.motion_id = 10;
    if (ScriptGame_HitFamilyMove(&t_fp) != 4) { TestFail("attach fighter: declared overlay tag lost"); rc = 1; }
    Geno_TestRestore();
    /* a define: row 53 declared smash wins; an undeclared tilt row keeps the motion tag; the donor's
       special range is still a special */
    t_setup();
    Geno_TestInstall("{\"geno\":7,\"fighters\":[{\"define\":{\"key\":\"hit-tag\",\"name\":\"Hit Tag\",\"base\":\"mario\","
        "\"common\":\"melee.common.v1\",\"resources\":\"retail:mario\"},"
        "\"common_states\":[{\"motion\":53,\"move_tag\":\"smash\"},{\"motion\":44,\"move_tag\":\"special\"},{\"motion\":57}]}]}");
    if (Geno_DefineCount() != 1) { TestFail("define not bound for the hit-tag test"); Geno_TestRestore(); return 1; }
    ck = Geno_DefineCKAt(0); kind = ck - 1;
    t_fp.kind = kind; t_fp.anim_id = 46;
    t_fp.motion_id = 53;
    if (ScriptGame_HitFamilyMove(&t_fp) != 4) { TestFail("define: declared smash on the tilt row not reported"); rc = 1; }
    t_fp.motion_id = 44;
    if (ScriptGame_HitFamilyMove(&t_fp) != 8) { TestFail("define: declared special on the jab row not reported"); rc = 1; }
    t_fp.motion_id = 57;
    if (ScriptGame_HitFamilyMove(&t_fp) != 3) { TestFail("define: undeclared row must keep the motion tag"); rc = 1; }
    t_fp.motion_id = 345;
    if (ScriptGame_HitFamilyMove(&t_fp) != 8) { TestFail("define: donor special range must stay special"); rc = 1; }
    Geno_TestRestore();
    return rc;
}

/* Slice 2 (D8): the digest of a defined fighter's GenoState, the one word RB_GameHash adds.
 * 0 for a non-define; moves with simulation state, not with diagnostics; restored by a snapshot. */
extern u32 GenoDefine_StateDigest(Fighter* fp);
extern u32 RB_GameHash(void);
extern u32* HSD_RandSeedPtr;
static int test_geno_define_state_hash(void)
{
    int rc = 0, kind;
    TestGenoState* s;
    u32 d0, d1, old_seed;
    /* no fighter in a headless suite: RB_GameHash is the seed word alone; this literal is what the
       pre-slice-2 code produced for the same seed (FNV mix of 0x811C9DC5 with 0x12345678) */
    old_seed = *HSD_RandSeedPtr;
    *HSD_RandSeedPtr = 0x12345678u;
    if (RB_GameHash() != 0x663876f6u) { TestFail("RB_GameHash with no define changed"); rc = 1; }
    *HSD_RandSeedPtr = old_seed;
    /* non-define: Kirby attach fighter */
    t_setup();
    if (GenoDefine_StateDigest(&t_fp) != 0) { TestFail("digest of a non-define fighter is not 0"); rc = 1; }
    Geno_TestInstall("{\"geno\":7,\"fighters\":[{\"define\":{\"key\":\"hash-def\",\"name\":\"Hash Def\",\"base\":\"mario\","
        "\"common\":\"melee.common.v1\",\"resources\":\"retail:mario\"}}]}");
    if (Geno_DefineCount() != 1) { TestFail("define not bound for the hash test"); Geno_TestRestore(); return 1; }
    kind = Geno_DefineCKAt(0) - 1;
    t_fp.kind = kind;
    Geno_FighterReset(&t_fp);
    s = GenoGame_StateOf(&t_fp);
    d0 = GenoDefine_StateDigest(&t_fp);
    if (d0 == 0) { TestFail("digest of a define is 0"); rc = 1; }
    /* simulation state moves the digest */
    s->move_i[0] = 7;
    d1 = GenoDefine_StateDigest(&t_fp);
    if (d1 == d0) { TestFail("digest ignores move_i[0]"); rc = 1; }
    s->la_i[3] = 5; if (GenoDefine_StateDigest(&t_fp) == d1) { TestFail("digest ignores a variable"); rc = 1; }
    s->la_i[3] = 0;
    /* diagnostics do not */
    d1 = GenoDefine_StateDigest(&t_fp);
    s->hook_calls += 3; s->resets += 2; s->changes += 4; s->state_entries += 5; s->art_spawned += 6; s->counters += 7; s->motion_land += 8;
    if (GenoDefine_StateDigest(&t_fp) != d1) { TestFail("digest moved with a diagnostics-only change"); rc = 1; }
    /* a snapshot restores it */
    if (snap_open(1) != 0) { TestFail("snapshot could not open for the hash test"); rc = 1; }
    else {
        snap_save(0);
        s->move_i[0] = 99; s->move_f[1] = 2.5f;
        if (GenoDefine_StateDigest(&t_fp) == d1) { TestFail("mutation did not change the digest"); rc = 1; }
        if (snap_load(0) != 0 || GenoDefine_StateDigest(&t_fp) != d1) { TestFail("digest not equal after snap_load"); rc = 1; }
    }
    Geno_TestRestore();
    return rc;
}
