/* Native tests for online Envoy (gw_script_netbuild.inc, gw_matchbuild.h, pc/gameworld/script_build.inc). The Lua parser of the staging call is
 * exercised by the two-client loopback runs; everything below the parse is tested here with hand-made ops against the real game BSS. */
extern int gw_ScriptGame_BuildTest(void);
extern unsigned gw_ScriptGame_BuildHashWord(void);
extern int gw_ScriptGame_FighterModRead(int slot, int field);
extern int gw_ScriptGame_FighterModOwner(int slot);
extern int gw_ScriptGame_HitRulesCount(int slot);
extern int gw_ScriptGame_HitRuleRead(int slot, int at, int field);

static const char *const nb_rec1 = "EB1|31337|1|0|1|ea4291532a5df145|gambler=1,keen=1||gambler|a0a765cf140457a9";
static const char *const nb_rec2 = "EB1|31337|1|0|2|ea4291532a5df145|aerial_doctrine=1,finishing=1||aerial_doctrine|f1241f42d17295e8";

/* a record with a correct digest over `body` (so a refusal below is for the form, not for the digest) */
static void nb_sign(const char *body, char *out, size_t cap) {
    uint32_t hi, lo;
    gw_mb_digest64(body, strlen(body), "build:", &hi, &lo);
    snprintf(out, cap, "%s|%08x%08x", body, (unsigned) hi, (unsigned) lo);
}
static int nb_refused(const char *text, const char *why_part) {
    GwMbRecord r;
    char why[160];
    if (gw_mb_record_parse(text, &r, why, sizeof why)) { gw_test_fail("netbuild: accepted '%.60s' (wanted a refusal about %s)", text, why_part); return 1; }
    if (strstr(why, why_part) == NULL) { gw_test_fail("netbuild: refused '%.50s' for '%s', wanted '%s'", text, why, why_part); return 1; }
    return 0;
}
static void nb_mod_op(GsSimOp *o, int port0, int field, float v) {
    int i;
    memset(o, 0, sizeof *o);
    o->kind = 1; o->slot = port0; o->enabled = 1;
    for (i = 0; i < 11; ++i) o->bits[i] = gs_fbits(1.0f);
    o->bits[field] = gs_fbits(v);
}
static void nb_rule_op(GsSimOp *o, int port0, float percent) {
    int *b;
    memset(o, 0, sizeof *o);
    o->kind = 3; o->slot = port0; o->enabled = -1;
    o->hit_rules.count = 1;
    b = o->hit_rules.rules[0];
    b[0] = 1001; b[1] = b[2] = b[3] = b[5] = -1;
    b[6] = b[7] = b[8] = b[11] = b[15] = gs_fbits(1.0f);
    b[14] = gs_fbits(percent);
}

static int test_netbuild(void) {
    GwMbRecord r;
    char why[160], body[400], bad[420];
    GsSimOp ops[2];
    GsNbSlot saved[GW_NB_SLOTS];
    int saved_applied = gs_nb_applied, rc = 0, i;
    unsigned w1, w2, w3, h0, h1;

    /* ---- the mode word and the record codec ---- */
    if (!gw_envoy_mode_valid(0) || !gw_envoy_mode_valid(GW_ENVOY_MODE_V1) || gw_envoy_mode_valid(1) || !gw_envoy_mode_valid(GW_ENVOY_MODE_COOP) || gw_envoy_mode_valid(GW_ENVOY_MODE_V1 + 3)) { gw_test_fail("netbuild: mode word validity wrong"); return 1; }
    if (!gw_mb_record_parse(nb_rec1, &r, why, sizeof why)) { gw_test_fail("netbuild: the Lua-written record is refused: %s", why); return 1; }
    if (r.seed != 31337 || r.game != 1 || r.loop != 0 || r.port != 1 || r.nmods != 2 || r.nkeystones != 1 || strcmp(r.digest, "a0a765cf140457a9") != 0 ||
        strcmp(r.pool, "ea4291532a5df145") != 0) { gw_test_fail("netbuild: record fields misread (seed %ld port %ld mods %d ks %d)", r.seed, r.port, r.nmods, r.nkeystones); return 1; }
    if (!gw_mb_record_parse(nb_rec2, &r, why, sizeof why) || r.port != 2 || r.nmods != 2) { gw_test_fail("netbuild: second record: %s", why); return 1; }
    /* the same record, one byte changed, a bad version, truncated, with a space, too long, not sorted, a zero tier, a trailing comma, a fake keystone count */
    snprintf(bad, sizeof bad, "%s", nb_rec1); bad[20] ^= 1;
    rc |= nb_refused(bad, "digest");
    snprintf(bad, sizeof bad, "%s", nb_rec1); bad[2] = '2';
    rc |= nb_refused(bad, "version");
    snprintf(bad, sizeof bad, "%.40s", nb_rec1);
    rc |= nb_refused(bad, "fields");
    snprintf(bad, sizeof bad, "%s", nb_rec1); bad[10] = ' ';
    rc |= nb_refused(bad, "compact");
    memset(bad, 'a', 405); bad[405] = 0;
    rc |= nb_refused(bad, "compact");
    rc |= nb_refused("", "compact");
    nb_sign("EB1|31337|1|0|1|ea4291532a5df145|keen=1,gambler=1||gambler", bad, sizeof bad);
    rc |= nb_refused(bad, "sorted");
    nb_sign("EB1|31337|1|0|1|ea4291532a5df145|gambler=1,gambler=1||gambler", bad, sizeof bad);
    rc |= nb_refused(bad, "sorted");
    nb_sign("EB1|31337|1|0|1|ea4291532a5df145|gambler=0||gambler", bad, sizeof bad);
    rc |= nb_refused(bad, "sorted");
    nb_sign("EB1|31337|1|0|1|ea4291532a5df145|gambler=1,||gambler", bad, sizeof bad);
    rc |= nb_refused(bad, "sorted");
    nb_sign("EB1|31337|1|0|1|ea4291532a5df145|gambler=1||gambler,keen", bad, sizeof bad);
    rc |= nb_refused(bad, "keystone");
    nb_sign("EB1|31337|1|0|1|ea4291532a5df145|gambler=1||gambler,zzz", bad, sizeof bad);
    rc |= nb_refused(bad, "more keystones");
    nb_sign("EB1|-5|1|0|1|ea4291532a5df145|gambler=1||gambler", bad, sizeof bad);
    rc |= nb_refused(bad, "seed");
    nb_sign("EB1|31337|01|0|1|ea4291532a5df145|gambler=1||gambler", bad, sizeof bad);
    rc |= nb_refused(bad, "seed");
    nb_sign("EB1|31337|1|0|1|EA4291532A5DF145|gambler=1||gambler", bad, sizeof bad);
    rc |= nb_refused(bad, "pool");
    snprintf(body, sizeof body, "EB1|31337|1|0|1|ea4291532a5df145|");
    for (i = 0; i < 30; ++i) snprintf(body + strlen(body), sizeof body - strlen(body), "%sm%02d=1", i ? "," : "", i);
    strcat(body, "||");
    nb_sign(body, bad, sizeof bad);
    rc |= nb_refused(bad, "sorted"); /* more than GW_MB_MODS_MAX items */
    if (rc) return 1;

    /* ---- the staged table, the agreement word, the apply ---- */
    memcpy(saved, gs_nb, sizeof saved);
    memset(gs_nb, 0, sizeof gs_nb);
    gw_Script_NetBuildSceneBegin();
    if (gw_Script_NetBuildWord() != 0 || gw_Script_NetBuildStaged() != 0 || gw_ScriptGame_BuildHashWord() != 0) { gw_test_fail("netbuild: not empty at the start"); goto done; }
    gw_mb_record_parse(nb_rec1, &r, why, sizeof why);
    nb_mod_op(&ops[0], 0, 0, 0.9f);       /* damage_dealt */
    nb_rule_op(&ops[1], 0, 1.1f);
    gs_nb_install(0, nb_rec1, &r, ops, 2);
    w1 = gw_Script_NetBuildWord();
    if (w1 == 0 || gw_Script_NetBuildStaged() != 1 || strcmp(gw_Script_NetBuildRecord(0), nb_rec1) != 0 || gw_Script_NetBuildRecord(1)[0] != 0) { gw_test_fail("netbuild: slot 1 not staged"); goto done; }
    gw_mb_record_parse(nb_rec2, &r, why, sizeof why);
    nb_mod_op(&ops[0], 1, 5, 0.8f);       /* jump_height */
    gs_nb_install(1, nb_rec2, &r, ops, 1);
    w2 = gw_Script_NetBuildWord();
    nb_mod_op(&ops[0], 1, 5, 0.81f);
    gs_nb_install(1, nb_rec2, &r, ops, 1);
    w3 = gw_Script_NetBuildWord();
    nb_mod_op(&ops[0], 1, 5, 0.8f);
    gs_nb_install(1, nb_rec2, &r, ops, 1);
    if (w2 == w1 || w3 == w2 || gw_Script_NetBuildWord() != w2) { gw_test_fail("netbuild: the word does not follow the staged ops (%08x %08x %08x)", w1, w2, w3); goto done; }
    gw_mb_record_parse(nb_rec2, &r, why, sizeof why);
    gs_nb_install(0, nb_rec2, &r, ops, 1); /* a different record (and ops) in slot 1 */
    if (gw_Script_NetBuildWord() == w2) { gw_test_fail("netbuild: the word ignores the record"); goto done; }
    gw_mb_record_parse(nb_rec1, &r, why, sizeof why);
    nb_mod_op(&ops[0], 0, 0, 0.9f); nb_rule_op(&ops[1], 0, 1.1f);
    gs_nb_install(0, nb_rec1, &r, ops, 2);
    if (gw_Script_NetBuildWord() != w2) { gw_test_fail("netbuild: the word is not a pure function of what is staged"); goto done; }

    gw_Script_NetBuildApply();
    if (!gw_Script_NetBuildApplied() || gw_ScriptGame_FighterModOwner(0) != GW_NB_OWNER || gw_ScriptGame_FighterModOwner(1) != GW_NB_OWNER ||
        gw_ScriptGame_FighterModRead(0, 0) != gs_fbits(0.9f) || gw_ScriptGame_FighterModRead(1, 5) != gs_fbits(0.8f) || gw_ScriptGame_FighterModRead(1, 0) != gs_fbits(1.0f)) {
        gw_test_fail("netbuild: fighter values not applied (owner %d/%d, dealt %08x, jump %08x)", gw_ScriptGame_FighterModOwner(0), gw_ScriptGame_FighterModOwner(1),
                     (unsigned) gw_ScriptGame_FighterModRead(0, 0), (unsigned) gw_ScriptGame_FighterModRead(1, 5));
        goto done;
    }
    if (gw_ScriptGame_HitRulesCount(0) != 1 || gw_ScriptGame_HitRulesCount(1) != 1 || gw_ScriptGame_HitRulesCount(2) != 0 ||
        gw_ScriptGame_HitRuleRead(0, 0, 14) != gs_fbits(1.1f) || gw_ScriptGame_HitRuleRead(0, 0, 0) != 1001) {
        gw_test_fail("netbuild: hit rules not applied (counts %d %d %d)", gw_ScriptGame_HitRulesCount(0), gw_ScriptGame_HitRulesCount(1), gw_ScriptGame_HitRulesCount(2));
        goto done;
    }
    if (gw_ScriptGame_BuildRead(0, 0) != 1 || gw_ScriptGame_BuildRead(1, 0) != 1 || gw_ScriptGame_BuildRead(2, 0) != 0 || gw_ScriptGame_BuildRead(0, 4) != 2 ||
        gw_ScriptGame_BuildRead(0, 1) != (int) 0xa0a765cf || gw_ScriptGame_BuildRead(0, 2) != (int) 0x140457a9 || gw_ScriptGame_BuildRead(0, 5) != 2) {
        gw_test_fail("netbuild: the applied table reads wrong (active %d %d %d, nops %d)", gw_ScriptGame_BuildRead(0, 0), gw_ScriptGame_BuildRead(1, 0), gw_ScriptGame_BuildRead(2, 0), gw_ScriptGame_BuildRead(0, 4));
        goto done;
    }
    /* the rollback word: nonzero, stable, and it sees a value edited after the apply (what a diverged peer would hold) */
    h0 = gw_ScriptGame_BuildHashWord();
    if (h0 == 0 || gw_ScriptGame_BuildHashWord() != h0 || gw_ScriptGame_BuildRead(0, 6) == 0) { gw_test_fail("netbuild: hash word %08x not stable", h0); goto done; }
    gw_ScriptGame_FighterMod(0, GW_NB_OWNER, 1, gs_fbits(0.91f), gs_fbits(1.0f), gs_fbits(1.0f), gs_fbits(1.0f), gs_fbits(1.0f), gs_fbits(1.0f), gs_fbits(1.0f), gs_fbits(1.0f));
    h1 = gw_ScriptGame_BuildHashWord();
    if (h1 == h0) { gw_test_fail("netbuild: the hash word does not see a changed fighter value"); goto done; }
    gw_ScriptGame_FighterMod(0, GW_NB_OWNER, 1, gs_fbits(0.9f), gs_fbits(1.0f), gs_fbits(1.0f), gs_fbits(1.0f), gs_fbits(1.0f), gs_fbits(1.0f), gs_fbits(1.0f), gs_fbits(1.0f));
    if (gw_ScriptGame_BuildHashWord() != h0) { gw_test_fail("netbuild: the hash word is not a function of the values"); goto done; }
    nb_rule_op(&ops[1], 0, 1.2f);
    gs_hr_apply_targets(0, -1, GW_NB_OWNER, &ops[1].hit_rules);
    if (gw_ScriptGame_BuildHashWord() == h0) { gw_test_fail("netbuild: the hash word does not see a changed hit rule"); goto done; }
    nb_rule_op(&ops[1], 0, 1.1f);
    gs_hr_apply_targets(0, -1, GW_NB_OWNER, &ops[1].hit_rules);
    if (gw_ScriptGame_BuildHashWord() != h0) { gw_test_fail("netbuild: the hash word keeps a rule handle (allocation order must not matter)"); goto done; }
    /* a restage and re-apply starts clean and gives the same table */
    gw_Script_NetBuildApply();
    if (gw_ScriptGame_BuildHashWord() != h0 || gw_ScriptGame_HitRulesCount(0) != 1) { gw_test_fail("netbuild: a second apply is not the same table"); goto done; }
    /* the scene ends: everything the build owned goes */
    gw_Script_NetBuildSceneBegin();
    if (gw_ScriptGame_FighterModOwner(0) != 0 || gw_ScriptGame_HitRulesCount(0) != 0 || gw_ScriptGame_BuildHashWord() != 0 || gw_Script_NetBuildApplied()) { gw_test_fail("netbuild: the build outlived its scene"); goto done; }
    if (gw_Script_NetBuildWord() != w2) { gw_test_fail("netbuild: the staged build must survive a scene (it is the next game's input)"); goto done; }
    /* nothing staged: apply does nothing and the match hashes as before */
    memset(gs_nb, 0, sizeof gs_nb);
    gw_Script_NetBuildApply();
    if (gw_ScriptGame_BuildHashWord() != 0 || gw_Script_NetBuildApplied()) { gw_test_fail("netbuild: apply with nothing staged changed something"); goto done; }
    /* the game-side table fixture: bounds, every field in the word, snapshot exactness */
    {
        int g = gw_ScriptGame_BuildTest();
        if (g != 0) { gw_test_fail("netbuild: game-side table fixture failed (0x%x)", g); goto done; }
    }
    {   /* stage 4: the triggered-rule evaluator's game half (program load, hooks, tick, hash word, snapshot exactness) */
        extern int gw_ScriptMods_Test(void);
        int g = gw_ScriptMods_Test();
        if (g != 0) { gw_test_fail("netbuild: triggered-rule evaluator fixture failed (0x%x)", g); goto done; }
    }
    rc = 0;
    goto restore;
done:
    rc = 1;
restore:
    gw_Script_NetBuildSceneBegin();
    memcpy(gs_nb, saved, sizeof saved);
    gs_nb_applied = saved_applied;
    return rc;
}

/* Stage 4: staging a triggered program and applying a derived-table variant (what the evaluator asks for when a fighter's status mask changes). */
static int test_netmods(void) {
    static GsNbProg saved[GW_NB_SLOTS];
    GsNbProg *g = &gs_nb_prog[0];
    float base = 1.0f, curse = 0.05f;
    int rc = 1, i;
    memcpy(saved, gs_nb_prog, sizeof saved);
    gw_Script_NetBuildSceneBegin();
    memset(gs_nb_prog, 0, sizeof gs_nb_prog);
    g->have = 1; g->nvariants = 2;
    g->words[0] = 0x534D5031; g->words[1] = 0; g->words[2] = 2;
    /* variant 0: a fighter value, an incoming launch rule (id 1002) the Marked amount lands on; variant 1: nothing */
    nb_mod_op(&g->var[0].ops[0], 0, 7, 0.75f);          /* knockback_taken field 7 */
    nb_rule_op(&g->var[0].ops[1], 0, 1.0f);
    g->var[0].ops[1].hit_rules.rules[0][0] = 1002; g->var[0].ops[1].hit_rules.rules[0][13] = 1; g->var[0].ops[1].hit_rules.rules[0][15] = gs_fbits(base);
    g->var[0].nops = 2; g->var[0].curse_rule = 1;
    g->var[1].nops = 0;
    gw_Script_NetModsVariant(0, 0, 4 | 32, gs_fbits(curse));
    if (gw_ScriptGame_FighterModOwner(0) != GW_NB_OWNER || gw_ScriptGame_FighterModRead(0, 7) != gs_fbits(0.75f)) { gw_test_fail("netmods: variant 0 values not applied"); goto done; }
    if (gw_ScriptGame_HitRulesCount(0) != 1 || gw_ScriptGame_HitRuleRead(0, 0, 0) != 1002 || gw_ScriptGame_HitRuleRead(0, 0, 15) != gs_fbits(base + curse) || gw_ScriptGame_FighterStatus(0) != 36) {
        gw_test_fail("netmods: variant 0 rules/status wrong (count %d launch %08x status %d)", gw_ScriptGame_HitRulesCount(0), (unsigned) gw_ScriptGame_HitRuleRead(0, 0, 15), gw_ScriptGame_FighterStatus(0));
        goto done;
    }
    gw_Script_NetModsVariant(0, 0, 4 | 32, 0);          /* the Marked status ended: the launch is the variant's own again */
    if (gw_ScriptGame_HitRuleRead(0, 0, 15) != gs_fbits(base)) { gw_test_fail("netmods: the curse amount was not removed again"); goto done; }
    gw_Script_NetModsVariant(0, 1, 8, 0);               /* variant 1: no values, no rules - only the status bits remain */
    if (gw_ScriptGame_FighterModOwner(0) != 0 || gw_ScriptGame_HitRulesCount(0) != 0 || gw_ScriptGame_FighterStatus(0) != 8) {
        gw_test_fail("netmods: variant 1 left something behind (owner %d rules %d status %d)", gw_ScriptGame_FighterModOwner(0), gw_ScriptGame_HitRulesCount(0), gw_ScriptGame_FighterStatus(0));
        goto done;
    }
    gw_Script_NetModsVariant(0, 5, 0, 0);               /* out of range: ignored */
    gw_Script_NetModsVariant(1, 0, 0, 0);               /* a slot with nothing staged: ignored */
    /* the digest follows the program and the variants */
    { unsigned d0 = gs_nb_prog_digest(g); g->var[0].curse_rule = 2; if (gs_nb_prog_digest(g) == d0) { gw_test_fail("netmods: the program digest ignores a variant field"); goto done; }
      g->var[0].curse_rule = 1; g->words[17] ^= 1; if (gs_nb_prog_digest(g) == d0) { gw_test_fail("netmods: the program digest ignores a word"); goto done; } g->words[17] ^= 1; }
    (void) i;
    rc = 0;
done:
    gw_Script_NetBuildSceneBegin();
    memcpy(gs_nb_prog, saved, sizeof saved);
    return rc;
}
