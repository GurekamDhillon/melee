/* Suite-pattern tests. No archive/disc or live alias assignment required. */
static int test_geno_define_registry(void)
{
    static gn_registry r;
    int i;
    char text[512];
    gn_registry_clear(&r);
    for (i = 0; i < 257; ++i) {
        snprintf(text, sizeof text,
            "{\"geno\":6,\"fighters\":[{\"define\":{\"key\":\"hero-%d\",\"name\":\"Hero %d\",\"base\":\"mario\",\"common\":\"melee.common.v1\",\"resources\":\"retail:mario\"},\"attributes\":{\"walk_max_vel\":1.8},\"common_states\":[{\"motion\":44,\"subaction\":46}]}]}", i, i);
        if (gn_parse_text(&r, text, "define-test") != 1 || r.n != i+1 ||
            !r.p[i].defined || r.p[i].nattr != 1 || r.p[i].ncommon != 1 ||
            r.p[i].common_row[0][2] != 46) {
            gw_test_fail("native definition parse/growth failed at profile %d", i);
            gn_registry_clear(&r); return 1;
        }
    }
    if (!r.p[31].id || !r.p[32].id || !r.p[255].id || r.p[31].id == r.p[32].id) {
        gw_test_fail("definition content identities lost across old 32-profile boundary");
        gn_registry_clear(&r); return 1;
    }
    gn_registry_clear(&r); return 0;
}
static int test_geno_define_repeated_install(void)
{
    int i, before = gw_Geno_ProfileCount(), rc = 0;
    /* A preceding test may have panicked before TestRestore. Preserve that
       caller's active/saved registries rather than compare it with its boot. */
    gn_registry entry = gn_boot, entry_saved = gn_saved;
    int entry_loaded = gn_boot_loaded, entry_saved_loaded = gn_saved_loaded;
    int entry_swapped = gn_swapped;
    memset(&gn_boot, 0, sizeof gn_boot);
    memset(&gn_saved, 0, sizeof gn_saved);
    gn_boot_loaded = 1; gn_saved_loaded = 0; gn_swapped = 0;
    for (i = 0; i < 8; ++i) {
        if (gw_Geno_TestInstall("{\"geno\":6,\"fighters\":[{\"attach\":\"mario\"}]}") != 1 ||
            gw_Geno_ProfileCount() != 1) {
            gw_test_fail("repeated temporary registry installation failed"); rc = 1; break;
        }
    }
    gw_Geno_TestRestore();
    if (gw_Geno_ProfileCount() != 0) {
        gw_test_fail("temporary registry did not restore original ownership"); rc = 1;
    }
    gn_registry_clear(&gn_boot);
    gn_registry_clear(&gn_saved);
    gn_boot = entry; gn_saved = entry_saved;
    gn_boot_loaded = entry_loaded; gn_saved_loaded = entry_saved_loaded;
    gn_swapped = entry_swapped; ++gn_pool_gen;
    if (gw_Geno_ProfileCount() != before) {
        gw_test_fail("repeated-install test changed caller registry"); rc = 1;
    }
    return rc;
}

/* A move tag may be declared per Geno state, per subaction overlay and per define common_states row.
 * The hit context reads them (script_hit_context.inc); here only the registry half is checked. */
static int test_geno_move_tags(void)
{
    static gn_registry r;
    int rc = 0;
    gn_registry_clear(&r);
    if (gn_parse_text(&r,
            "{\"geno\":6,\"fighters\":[{\"attach\":\"kirby\","
            "\"states\":[{\"name\":\"A\",\"behavior\":\"geno.air\",\"move_tag\":\"special\"},"
            "{\"name\":\"B\",\"behavior\":\"geno.air\"},"
            "{\"name\":\"C\",\"behavior\":\"geno.air\",\"move_tag\":\"aerial\"},"
            "{\"name\":\"D\",\"behavior\":\"geno.air\",\"move_tag\":\"nonsense\"}],"
            "\"subactions\":[{\"index\":46,\"words\":[\"0x04000002\"],\"move_tag\":\"smash\"},"
            "{\"index\":47,\"words\":[\"0x04000002\"]}]}]}", "tags") != 1) {
        gw_test_fail("move tag entry not parsed"); gn_registry_clear(&r); return 1;
    }
    if (r.p[0].st_tag[0] != 8 || r.p[0].st_tag[1] != 0 || r.p[0].st_tag[2] != 5 || r.p[0].st_tag[3] != 0 ||
        r.p[0].ov_tag[0] != 4 || r.p[0].ov_tag[1] != 0) {
        gw_test_fail("move tags: state/overlay declarations %d %d %d %d / %d %d",
            r.p[0].st_tag[0], r.p[0].st_tag[1], r.p[0].st_tag[2], r.p[0].st_tag[3],
            r.p[0].ov_tag[0], r.p[0].ov_tag[1]);
        rc = 1;
    }
    gn_registry_clear(&r);
    if (gn_parse_text(&r,
            "{\"geno\":6,\"fighters\":[{\"define\":{\"key\":\"tag-hero\",\"name\":\"Tag Hero\",\"base\":\"mario\","
            "\"common\":\"melee.common.v1\",\"resources\":\"retail:mario\"},"
            "\"common_states\":[{\"motion\":44,\"subaction\":46,\"move_tag\":\"jab\"},{\"motion\":60}]}]}", "tags-define") != 1 ||
        r.p[0].ncommon != 2 || r.p[0].cs_tag[0] != 1 || r.p[0].cs_tag[1] != 0 || r.p[0].common_row[0][2] != 46) {
        gw_test_fail("move tags: define common_states row declaration");
        rc = 1;
    }
    gn_registry_clear(&r);
    /* slice 2: a tilt row (motion 53) declared smash reads back smash through the registry accessor */
    if (gn_parse_text(&r,
            "{\"geno\":7,\"fighters\":[{\"define\":{\"key\":\"tag-tilt\",\"name\":\"Tag Tilt\",\"base\":\"mario\","
            "\"common\":\"melee.common.v1\",\"resources\":\"retail:mario\"},"
            "\"common_states\":[{\"motion\":53,\"move_tag\":\"smash\"},{\"motion\":57}]}]}", "tags-tilt") != 1 ||
        r.n != 1 || r.p[0].cs_tag[0] != 4 || r.p[0].common_row[0][0] != 53 || r.p[0].cs_tag[1] != 0) {
        gw_test_fail("move tags: a tilt row declared smash did not store smash");
        rc = 1;
    }
    gn_registry_clear(&r);
    return rc;
}

/* ---- slice 2 (geno 7) ------------------------------------------------------------------------ */
#define GN_T_HEAD(ver, extra) "{\"geno\":" #ver ",\"fighters\":[{\"define\":{\"key\":\"t7\",\"name\":\"T7\",\"base\":\"mario\"," \
    "\"common\":\"melee.common.v1\",\"resources\":\"retail:mario\"}" extra "}]}"

/* The widened attribute list: 60 names parse in a v7 define; one unknown name, or a name past the
 * first 40 in a v6 file, refuses the define (a define is strict; attach stays lenient). */
static int test_geno_define_attrs_v7(void)
{
    static gn_registry r;
    static const char* names[60] = {
        "walk_accel_mul","walk_accel_base","walk_max_vel","slow_walk_max","mid_walk_point","fast_walk_min","ground_friction",
        "dash_initial_velocity","dash_accel_mul","dash_accel_base","dash_max_velocity","run_animation_scaling","max_run_brake_frames",
        "ground_max_horizontal_velocity","jump_startup_time","jump_h_initial_velocity","jump_v_initial_velocity",
        "ground_to_air_jump_momentum_multiplier","jump_h_max_velocity","hop_v_initial_velocity","air_jump_v_multiplier",
        "air_jump_h_multiplier","max_jumps","gravity","terminal_velocity","air_drift_stick_mul","aerial_drift_base","air_drift_max",
        "aerial_friction","fast_fall_velocity","air_max_horizontal_velocity","jab_2_input_window","jab_3_input_window",
        "standing_turn_frames","weight","model_scaling","initial_shield_size","shield_break_initial_velocity","rapid_jab_window",
        "clank_animation_length","hit_spark_variant","ledge_jump_horizontal_velocity","ledge_jump_vertical_velocity",
        "item_throw_velocity_multiplier","heavy_throw_velocity_multiplier","specials_ground_speed_retention","kirby_b_star_damage",
        "normal_landing_lag","landingairn_lag","landingairf_lag","landingairb_lag","landingairhi_lag","landingairlw_lag",
        "name_tag_height","passivewall_vel_x","wall_jump_horizontal_velocity","wall_jump_vertical_velocity","passiveceil_vel_x",
        "trophy_scale","screw_attack_launch_velocity"};
    char text[4096], attrs[3072];
    int i, n = 0, rc = 0;
    attrs[0] = 0;
    for (i = 0; i < 60; ++i)
        n += snprintf(attrs + n, sizeof attrs - n, "%s\"%s\":%d.5", i ? "," : "", names[i], i == 22 ? 2 : 1);
    snprintf(text, sizeof text, GN_T_HEAD(7, ",\"attributes\":{%s}"), attrs);
    gn_registry_clear(&r);
    if (gn_parse_text(&r, text, "attrs-v7") != 1 || r.n != 1 || !r.p[0].defined || r.p[0].nattr != 60) {
        gw_test_fail("v7 define with 60 attributes: n=%d nattr=%d", r.n, r.n ? r.p[0].nattr : -1); rc = 1;
    }
    gn_registry_clear(&r);
    /* the same text as v6: the widened names need "geno": 7 */
    snprintf(text, sizeof text, GN_T_HEAD(6, ",\"attributes\":{%s}"), attrs);
    if (gn_parse_text(&r, text, "attrs-v6") != 0 || r.n != 0) { gw_test_fail("v6 define with attributes past the first 40 was accepted"); rc = 1; }
    gn_registry_clear(&r);
    /* v6 with only the original 40 still parses (the Hero's shape) */
    if (gn_parse_text(&r, GN_T_HEAD(6, ",\"attributes\":{\"walk_max_vel\":1.8,\"clank_animation_length\":3}"), "attrs-v6-old") != 1 ||
        r.p[0].nattr != 2) { gw_test_fail("v6 define with original attributes no longer parses"); rc = 1; }
    gn_registry_clear(&r);
    if (gn_parse_text(&r, GN_T_HEAD(7, ",\"attributes\":{\"walk_max_vel\":1.8,\"no_such_attribute\":3}"), "attrs-unknown") != 0 || r.n != 0) {
        gw_test_fail("a define naming an unknown attribute was accepted"); rc = 1;
    }
    gn_registry_clear(&r);
    return rc;
}

/* v7 lets a define carry special_attributes and fx_bindings; v6 may not; articles stay refused;
 * a v6 define's content id is the one the engine computed before slice 2 (captured literal). */
static int test_geno_define_v7_parse(void)
{
    static gn_registry r;
    int rc = 0;
    #define GN_T_BOTH ",\"special_attributes\":[{\"index\":13,\"float\":16.0}],\"fx_bindings\":\"fx/none.json\""
    gn_registry_clear(&r);
    if (gn_parse_text(&r, GN_T_HEAD(7, GN_T_BOTH), "v7") != 1 || !r.p[0].defined || r.p[0].nspec != 1 || r.p[0].spec_index[0] != 13) {
        gw_test_fail("v7 define with special_attributes and fx_bindings: n=%d", r.n); rc = 1;
    }
    gn_registry_clear(&r);
    if (gn_parse_text(&r, GN_T_HEAD(6, GN_T_BOTH), "v6") != 0 || r.n != 0) { gw_test_fail("the same entry as geno 6 was accepted"); rc = 1; }
    gn_registry_clear(&r);
    if (gn_parse_text(&r, GN_T_HEAD(6, ",\"special_attributes\":[{\"index\":13,\"float\":16.0}]"), "v6s") != 0) { gw_test_fail("v6 special_attributes accepted"); rc = 1; }
    gn_registry_clear(&r);
    if (gn_parse_text(&r, GN_T_HEAD(7, ",\"articles\":[]"), "v7a") != 0 || r.n != 0) { gw_test_fail("a define with articles was accepted"); rc = 1; }
    gn_registry_clear(&r);
    /* a v6 entry keeps its id: the entry is hashed as written, the file's version is not part of it */
    if (gn_parse_text(&r, GN_T_HEAD(6, ",\"attributes\":{\"walk_max_vel\":1.8}"), "id6") != 1 ||
        gn_parse_text(&r, GN_T_HEAD(7, ",\"attributes\":{\"walk_max_vel\":1.8}"), "id7") != 1 || r.n != 2 ||
        r.p[0].id != r.p[1].id || r.p[0].id != 0xa2759ebe1530b465ull /* the hash code is untouched by slice 2; the Hero id 47e0fa8287c3c75f of slice 1 is re-read in game */) {
        gw_test_fail("v6 define id: %016llx (v7 same entry %016llx)", (unsigned long long)(r.n > 0 ? r.p[0].id : 0),
                     (unsigned long long)(r.n > 1 ? r.p[1].id : 0)); rc = 1;
    }
    gn_registry_clear(&r);
    return rc;
}

/* Slice 2 budgets: a define with 40 overlays and a 1500-word pool parses and keeps its slots; the 65th
 * overlay is refused (the limit is GENO_MAX_OVERLAYS); a second define loaded afterwards still gets
 * its overlays. The overlays are 1500 words in all, spread so no slot is empty. */
static int test_geno_define_overlay_budget(void)
{
    static gn_registry r;
    static char text[26000], a[24000];
    int i, n, rc = 0, k;
    gn_registry_clear(&r);
    for (k = 0; k < 2; ++k) {
        n = 0;
        for (i = 0; i < 40; ++i) {
            int w;
            n += snprintf(a + n, sizeof a - n, "%s{\"index\":%d,\"words\":[", i ? "," : "", i);
            for (w = 0; w < 37; ++w) n += snprintf(a + n, sizeof a - n, "%s\"0x04000002\"", w ? "," : "");
            n += snprintf(a + n, sizeof a - n, "]}");
        }
        snprintf(text, sizeof text,
            "{\"geno\":7,\"fighters\":[{\"define\":{\"key\":\"budget%d\",\"name\":\"Budget %d\",\"base\":\"mario\","
            "\"common\":\"melee.common.v1\",\"resources\":\"retail:mario\"},\"subactions\":[%s]}]}", k, k, a);
        if (gn_parse_text(&r, text, "budget") != 1 || r.n != k + 1 || r.p[k].nov != 40) {
            gw_test_fail("define %d with 40 overlays: n=%d nov=%d", k, r.n, r.n > k ? r.p[k].nov : -1); rc = 1;
        }
    }
    if (r.n == 2 && (r.npool < 2 * 40 * 38 || r.npool > GENO_POOL_WORDS)) { gw_test_fail("pool words %d", r.npool); rc = 1; }
    gn_registry_clear(&r);
    /* 65 overlays: refused */
    n = 0;
    for (i = 0; i < GENO_MAX_OVERLAYS + 1; ++i) n += snprintf(a + n, sizeof a - n, "%s{\"index\":%d,\"words\":[\"0x04000002\"]}", i ? "," : "", i);
    snprintf(text, sizeof text,
        "{\"geno\":7,\"fighters\":[{\"define\":{\"key\":\"over\",\"name\":\"Over\",\"base\":\"mario\","
        "\"common\":\"melee.common.v1\",\"resources\":\"retail:mario\"},\"subactions\":[%s]}]}", a);
    if (gn_parse_text(&r, text, "budget-over") != 0 || r.n != 0) { gw_test_fail("65 overlays were accepted"); rc = 1; }
    gn_registry_clear(&r);
    /* the refusal left room: a following define still gets its overlay */
    if (gn_parse_text(&r, text, "budget-over") != 0 ||
        gn_parse_text(&r, GN_T_HEAD(7, ",\"subactions\":[{\"index\":3,\"words\":[\"0x04000002\"]}]"), "after") != 1 ||
        r.n != 1 || r.p[0].nov != 1) { gw_test_fail("a define after a refused one lost its overlay"); rc = 1; }
    gn_registry_clear(&r);
    return rc;
}
