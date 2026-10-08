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

/* Slice 3 (geno 8): a define may declare articles and the named-sound table. The resolver is strict for a define:
 * an unknown name, a duplicate name, an id or volume out of range, or more than GENO_MAX_SOUNDS refuses the entry. */
#define GN_T_ART(sound_refs) ",\"sounds\":[{\"name\":\"Zap\",\"retail_sfx\":234,\"volume\":100},{\"name\":\"Fade\",\"retail_sfx\":228}]," \
    "\"articles\":[{\"name\":\"Bolt\",\"lifetime\":70,\"velocity\":[2.2,0]" sound_refs ",\"hitboxes\":[{\"damage\":5,\"size\":3.5}]}]"
static int test_geno_define_v8_parse(void)
{
    static gn_registry r;
    int rc = 0;
    gn_registry_clear(&r);
    if (gn_parse_text(&r, GN_T_HEAD(8, GN_T_ART(",\"spawn_sound\":\"Zap\",\"end_sound\":\"Fade\"")), "v8") != 1 || !r.p[0].defined ||
        r.p[0].nart != 1 || r.p[0].nsnd != 2 ||
        r.p[0].art_param[0][GENO_AP_SPAWN_SFX] != 234 || r.p[0].art_param[0][GENO_AP_SPAWN_VOL] != 100 ||
        r.p[0].art_param[0][GENO_AP_END_SFX] != 228 || r.p[0].art_param[0][GENO_AP_END_VOL] != 127) {
        gw_test_fail("v8 define with articles and sounds: n=%d nart=%d nsnd=%d", r.n, r.n ? r.p[0].nart : -1, r.n ? r.p[0].nsnd : -1); rc = 1;
    }
    if (r.n == 1 && (r.p[0].art_nhit[0] != 1 || gw_Geno_ArticleCount(0) < 0)) { gw_test_fail("v8 article lost its hitbox"); rc = 1; }
    gn_registry_clear(&r);
    if (gn_parse_text(&r, GN_T_HEAD(7, GN_T_ART("")), "v7art") != 0 || r.n != 0) { gw_test_fail("the same entry as geno 7 was accepted"); rc = 1; }
    gn_registry_clear(&r);
    if (gn_parse_text(&r, GN_T_HEAD(7, ",\"sounds\":[{\"name\":\"Zap\",\"retail_sfx\":234}]"), "v7snd") != 0) { gw_test_fail("v7 sounds accepted"); rc = 1; }
    gn_registry_clear(&r);
    /* a v8 define without articles or sounds is the same entry as before: the id is the entry's hash */
    if (gn_parse_text(&r, GN_T_HEAD(8, ",\"attributes\":{\"walk_max_vel\":1.8}"), "id8") != 1 || r.p[0].id != 0xa2759ebe1530b465ull) {
        gw_test_fail("v8 define id changed: %016llx", (unsigned long long)(r.n ? r.p[0].id : 0)); rc = 1;
    }
    gn_registry_clear(&r);
    /* two defines with articles never share an item kind (the kind is profile arithmetic) */
    if (GENO_ART_KIND(0, 0) == GENO_ART_KIND(1, 0) || GENO_ART_KIND(33, 0) == GENO_ART_KIND(34, 0) ||
        GENO_ART_KIND(33, 15) >= GENO_ART_KIND(34, 0) || !GENO_ART_IS_KIND(GENO_ART_KIND(40, 3))) {
        gw_test_fail("article kinds collide across profiles"); rc = 1;
    }
    return rc;
}

static int test_geno_define_resolver(void)
{
    static gn_registry r;
    static char many[1200], text[1600];
    int rc = 0, i, n = 0;
    gn_registry_clear(&r);
    if (gn_parse_text(&r, GN_T_HEAD(8, GN_T_ART(",\"spawn_sound\":\"Nope\"")), "unknown") != 0 || r.n != 0) { gw_test_fail("unknown sound name accepted (donor fallback?)"); rc = 1; }
    gn_registry_clear(&r);
    if (gn_parse_text(&r, GN_T_HEAD(8, GN_T_ART(",\"end_sound\":\"Nope\"")), "unknown-end") != 0) { gw_test_fail("unknown end sound name accepted"); rc = 1; }
    gn_registry_clear(&r);
    if (gn_parse_text(&r, GN_T_HEAD(8, ",\"sounds\":[{\"name\":\"A\",\"retail_sfx\":5},{\"name\":\"A\",\"retail_sfx\":6}]"), "dup") != 0) { gw_test_fail("duplicate sound name accepted"); rc = 1; }
    gn_registry_clear(&r);
    if (gn_parse_text(&r, GN_T_HEAD(8, ",\"sounds\":[{\"name\":\"A\",\"retail_sfx\":0}]"), "id0") != 0) { gw_test_fail("sound id 0 accepted"); rc = 1; }
    gn_registry_clear(&r);
    if (gn_parse_text(&r, GN_T_HEAD(8, ",\"sounds\":[{\"name\":\"A\",\"retail_sfx\":1000000}]"), "idbig") != 0) { gw_test_fail("sound id 1000000 accepted"); rc = 1; }
    gn_registry_clear(&r);
    if (gn_parse_text(&r, GN_T_HEAD(8, ",\"sounds\":[{\"name\":\"A\",\"retail_sfx\":5,\"volume\":128}]"), "vol") != 0) { gw_test_fail("volume 128 accepted"); rc = 1; }
    gn_registry_clear(&r);
    if (gn_parse_text(&r, GN_T_HEAD(8, ",\"sounds\":[{\"retail_sfx\":5}]"), "noname") != 0) { gw_test_fail("a sound without a name accepted"); rc = 1; }
    gn_registry_clear(&r);
    if (gn_parse_text(&r, GN_T_HEAD(8, ",\"sounds\":{}"), "notarray") != 0) { gw_test_fail("sounds as an object accepted"); rc = 1; }
    gn_registry_clear(&r);
    n = snprintf(many, sizeof many, ",\"sounds\":[");
    for (i = 0; i < GENO_MAX_SOUNDS + 1; ++i) n += snprintf(many + n, sizeof many - n, "%s{\"name\":\"S%d\",\"retail_sfx\":%d}", i ? "," : "", i, 100 + i);
    snprintf(many + n, sizeof many - n, "]");
    snprintf(text, sizeof text, "{\"geno\":8,\"fighters\":[{\"define\":{\"key\":\"t8\",\"name\":\"T8\",\"base\":\"mario\",\"common\":\"melee.common.v1\",\"resources\":\"retail:mario\"}%s}]}", many);
    if (gn_parse_text(&r, text, "toomany") != 0) { gw_test_fail("%d sounds accepted", GENO_MAX_SOUNDS + 1); rc = 1; }
    gn_registry_clear(&r);
    n = snprintf(many, sizeof many, ",\"sounds\":[");
    for (i = 0; i < GENO_MAX_SOUNDS; ++i) n += snprintf(many + n, sizeof many - n, "%s{\"name\":\"S%d\",\"retail_sfx\":%d}", i ? "," : "", i, 100 + i);
    snprintf(many + n, sizeof many - n, "]");
    snprintf(text, sizeof text, "{\"geno\":8,\"fighters\":[{\"define\":{\"key\":\"t8\",\"name\":\"T8\",\"base\":\"mario\",\"common\":\"melee.common.v1\",\"resources\":\"retail:mario\"}%s}]}", many);
    if (gn_parse_text(&r, text, "max") != 1 || r.p[0].nsnd != GENO_MAX_SOUNDS || r.p[0].snd_id[GENO_MAX_SOUNDS - 1] != 100 + GENO_MAX_SOUNDS - 1) {
        gw_test_fail("%d sounds were not all kept", GENO_MAX_SOUNDS); rc = 1;
    }
    gn_registry_clear(&r);
    /* resolution by name: the table maps the name to the engine id and volume the article was given */
    if (gn_parse_text(&r, GN_T_HEAD(8, GN_T_ART(",\"spawn_sound\":\"Fade\"")), "byname") != 1 ||
        r.p[0].art_param[0][GENO_AP_SPAWN_SFX] != 228 || r.p[0].art_param[0][GENO_AP_END_SFX] != 0) { gw_test_fail("name -> id resolution"); rc = 1; }
    gn_registry_clear(&r);
    return rc;
}

/* Geno slice 6: a define's costumes as declared colour sets, and its own menu and HUD art (registry half). A base "none" define fixture
 * is built in text; the fixture is parsed, never installed. */
#define GN_T_NONE(ver, fighter_extra, entry_extra) "{\"geno\":" #ver ",\"fighters\":[{\"define\":{\"key\":\"t9\",\"name\":\"T9\",\"base\":\"none\"," \
    "\"common\":\"melee.common.v1\",\"resources\":\"mod:files\"},\"fighter\":{\"plan\":\"plan.json\",\"animation\":\"bank.dat\",\"costumes\":[" fighter_extra "]}" entry_extra "}]}"
#define GN_T_COS(file, extra) "{\"file\":\"" file "\",\"joint\":\"j\"" extra "}"

/* Slice 8: fighter.rows. A base "none" define declares animation rows past Mario's 303 (one clip name each); a state, an overlay
 * or a common_states row may then name 303 + i. Without the key the bound stays 302, and a donor define never gets it. */
#define GN_T_NONE_R(rows, entry_extra) "{\"geno\":9,\"fighters\":[{\"define\":{\"key\":\"t9\",\"name\":\"T9\",\"base\":\"none\"," \
    "\"common\":\"melee.common.v1\",\"resources\":\"mod:files\"},\"fighter\":{\"plan\":\"plan.json\",\"animation\":\"bank.dat\",\"costumes\":[" GN_T_COS("a.dat", "") "]" rows "}" entry_extra "}]}"
static int test_geno_define_own_rows(void)
{
    static gn_registry r;
    int rc = 0;
    gn_registry_clear(&r);
    if (gn_parse_text(&r, GN_T_NONE_R(",\"rows\":[\"SpecialN1Start\",\"SpecialN1\"]",
                      ",\"states\":[{\"name\":\"A\",\"behavior\":\"geno.ground\",\"subaction\":304}],\"subactions\":[{\"index\":303,\"words\":[\"0x04000002\"]}]"), "rows") != 1 ||
        r.n != 1 || !r.p[0].fm || r.p[0].fm->nown != 2 || strcmp(r.p[0].fm->own_clip[1], "SpecialN1")) {
        gw_test_fail("a none define with two own rows and a state and an overlay on them was refused or misread"); rc = 1;
    }
    gn_registry_clear(&r);
    if (gn_parse_text(&r, GN_T_NONE_R(",\"rows\":[\"X\",\"Y\"]", ",\"states\":[{\"name\":\"A\",\"behavior\":\"geno.ground\",\"subaction\":305}]"), "past") != 0 || r.n != 0) {
        gw_test_fail("a state on row 305 was accepted with two own rows"); rc = 1;
    }
    gn_registry_clear(&r);
    if (gn_parse_text(&r, GN_T_NONE_R("", ",\"subactions\":[{\"index\":303,\"words\":[\"0x04000002\"]}]"), "norows") != 0 || r.n != 0) {
        gw_test_fail("an overlay on row 303 was accepted without own rows"); rc = 1;
    }
    gn_registry_clear(&r);
    if (gn_parse_text(&r, GN_T_NONE_R(",\"rows\":[5]", ""), "badrow") != 0 || r.n != 0) { gw_test_fail("a non-string row was accepted"); rc = 1; }
    gn_registry_clear(&r);
    /* a donor (mario) define does not get the key */
    if (gn_parse_text(&r, GN_T_HEAD(9, ",\"subactions\":[{\"index\":303,\"words\":[\"0x04000002\"]}]"), "donor") != 0 || r.n != 0) { gw_test_fail("a donor define reached row 303"); rc = 1; }
    gn_registry_clear(&r);
    return rc;
}

static int test_geno_define_presentation(void)
{
    static gn_registry r;
    const gn_profile* p;
    char buf[64];
    int rc = 0;
    gn_registry_clear(&r);
    /* declared team colours and names */
    if (gn_parse_text(&r, GN_T_NONE(9, GN_T_COS("a.dat", ",\"name\":\"Plain\"") "," GN_T_COS("b.dat", ",\"name\":\"Ember\",\"team\":\"red\"") ","
                                       GN_T_COS("c.dat", ",\"team\":\"blue\"") "," GN_T_COS("d.dat", ",\"team\":\"green\""), ""), "none") != 1 || r.n != 1 || !r.p[0].fm) {
        gw_test_fail("a none define with declared costume colours was refused"); gn_registry_clear(&r); return 1;
    }
    p = &r.p[0];
    if (p->fm->ncostume != 4 || gn_fm_team_costume(p->fm, 0) != 1 || gn_fm_team_costume(p->fm, 1) != 2 || gn_fm_team_costume(p->fm, 2) != 3 ||
        strcmp(p->fm->cos_name[0], "Plain") || strcmp(p->fm->cos_name[1], "Ember") || p->fm->cos_name[2][0] || p->fm->cos_team[0] != -1) {
        gw_test_fail("declared costume colours: red %d blue %d green %d, names '%s' '%s'", gn_fm_team_costume(p->fm, 0), gn_fm_team_costume(p->fm, 1),
                     gn_fm_team_costume(p->fm, 2), p->fm->cos_name[0], p->fm->cos_name[1]);
        rc = 1;
    }
    if (gn_fm_team_costume(p->fm, 3) != -1 || gn_fm_team_costume(NULL, 0) != -1) { gw_test_fail("a team past green, or no fighter block, answered a costume"); rc = 1; }
    gn_registry_clear(&r);
    /* undeclared: red first, blue second, green third, wrapping to the first when there are fewer */
    if (gn_parse_text(&r, GN_T_NONE(9, GN_T_COS("a.dat", "") "," GN_T_COS("b.dat", ""), ""), "two") != 1 || !r.p[0].fm ||
        gn_fm_team_costume(r.p[0].fm, 0) != 0 || gn_fm_team_costume(r.p[0].fm, 1) != 1 || gn_fm_team_costume(r.p[0].fm, 2) != 0) {
        gw_test_fail("undeclared team colours of a two-costume define"); rc = 1;
    }
    gn_registry_clear(&r);
    if (gn_parse_text(&r, GN_T_NONE(9, GN_T_COS("a.dat", ""), ""), "one") != 1 || !r.p[0].fm ||
        gn_fm_team_costume(r.p[0].fm, 1) != 0 || gn_fm_team_costume(r.p[0].fm, 2) != 0) { gw_test_fail("undeclared team colours of a one-costume define"); rc = 1; }
    gn_registry_clear(&r);
    /* refusals */
    if (gn_parse_text(&r, GN_T_NONE(9, GN_T_COS("a.dat", ",\"team\":\"purple\""), ""), "team") != 0) { gw_test_fail("team purple accepted"); rc = 1; }
    gn_registry_clear(&r);
    if (gn_parse_text(&r, GN_T_NONE(9, GN_T_COS("a.dat", ",\"name\":\"A\\tB\""), ""), "ctl") != 0) { gw_test_fail("a control character in a costume name accepted"); rc = 1; }
    gn_registry_clear(&r);
    if (gn_parse_text(&r, GN_T_NONE(9, GN_T_COS("a.dat", ",\"name\":\"0123456789012345678901234\""), ""), "long") != 0) { gw_test_fail("a 25-character costume name accepted"); rc = 1; }
    gn_registry_clear(&r);

    /* the art: one icon, per-costume portraits and stock icons (a costume past the list uses the first) */
    if (gn_parse_text(&r, GN_T_NONE(9, GN_T_COS("a.dat", "") "," GN_T_COS("b.dat", ""),
            ",\"presentation\":{\"icon\":\"t9_icon.gxtex\",\"portrait\":[\"t9_p0.gxtex\",\"t9_p1.gxtex\"],\"stock\":\"t9_stock.gxtex\"}"), "art") != 1 || !r.p[0].pres) {
        gw_test_fail("a define with its package art was refused"); gn_registry_clear(&r); return 1;
    }
    p = &r.p[0];
    if (strcmp(gn_pres_file(p, GN_PRES_ICON, 0), "t9_icon.gxtex") || strcmp(gn_pres_file(p, GN_PRES_ICON, 3), "t9_icon.gxtex") ||
        strcmp(gn_pres_file(p, GN_PRES_PORTRAIT, 0), "t9_p0.gxtex") || strcmp(gn_pres_file(p, GN_PRES_PORTRAIT, 1), "t9_p1.gxtex") ||
        strcmp(gn_pres_file(p, GN_PRES_PORTRAIT, 7), "t9_p0.gxtex") || strcmp(gn_pres_file(p, GN_PRES_STOCK, 1), "t9_stock.gxtex") ||
        gn_pres_file(p, 3, 0) != NULL || gn_pres_file(&r.p[0], -1, 0) != NULL) {
        gw_test_fail("presentation files do not resolve: icon '%s' portrait1 '%s' stock1 '%s'", gn_pres_file(p, 0, 0), gn_pres_file(p, 1, 1), gn_pres_file(p, 2, 1)); rc = 1;
    }
    gn_registry_clear(&r);
    /* nothing declared: nothing resolves, and the entry is the same entry as before (its id is the hash of the entry) */
    if (gn_parse_text(&r, GN_T_NONE(9, GN_T_COS("a.dat", ""), ""), "none-art") != 1 || r.p[0].pres != NULL || gn_pres_file(&r.p[0], 0, 0) != NULL) {
        gw_test_fail("a define without presentation answered art"); rc = 1;
    }
    gn_registry_clear(&r);
    /* a donor-based define may declare art too (its icon, its portrait), at format 9 */
    if (gn_parse_text(&r, GN_T_HEAD(9, ",\"presentation\":{\"icon\":\"t7_icon.gxtex\"}"), "mario-art") != 1 || !r.p[0].pres || strcmp(gn_pres_file(&r.p[0], 0, 0), "t7_icon.gxtex")) {
        gw_test_fail("a donor-based define with an icon was refused"); rc = 1;
    }
    gn_registry_clear(&r);
    if (gn_parse_text(&r, GN_T_HEAD(9, ",\"attributes\":{\"walk_max_vel\":1.8}"), "id9") != 1 || r.p[0].id != 0xa2759ebe1530b465ull || r.p[0].pres != NULL) {
        gw_test_fail("a define without presentation changed id: %016llx", (unsigned long long)(r.n ? r.p[0].id : 0)); rc = 1;
    }
    gn_registry_clear(&r);
    /* refusals: format 8, an unknown key, a name that is not a plain .gxtex, an icon list, too many entries, a non-object */
    if (gn_parse_text(&r, GN_T_HEAD(8, ",\"presentation\":{\"icon\":\"a.gxtex\"}"), "v8") != 0) { gw_test_fail("presentation accepted at geno 8"); rc = 1; }
    gn_registry_clear(&r);
    if (gn_parse_text(&r, GN_T_HEAD(9, ",\"presentation\":{\"banner\":\"a.gxtex\"}"), "key") != 0) { gw_test_fail("an unknown presentation key accepted"); rc = 1; }
    gn_registry_clear(&r);
    if (gn_parse_text(&r, GN_T_HEAD(9, ",\"presentation\":{\"icon\":\"a.png\"}"), "ext") != 0) { gw_test_fail("a .png accepted as art"); rc = 1; }
    gn_registry_clear(&r);
    if (gn_parse_text(&r, GN_T_HEAD(9, ",\"presentation\":{\"icon\":\"sub/a.gxtex\"}"), "slash") != 0) { gw_test_fail("a path accepted as art"); rc = 1; }
    gn_registry_clear(&r);
    if (gn_parse_text(&r, GN_T_HEAD(9, ",\"presentation\":{\"icon\":\"..a.gxtex\"}"), "dots") != 0) { gw_test_fail("a .. name accepted as art"); rc = 1; }
    gn_registry_clear(&r);
    if (gn_parse_text(&r, GN_T_HEAD(9, ",\"presentation\":{\"icon\":[\"a.gxtex\"]}"), "iconlist") != 0) { gw_test_fail("an icon list accepted"); rc = 1; }
    gn_registry_clear(&r);
    if (gn_parse_text(&r, GN_T_HEAD(9, ",\"presentation\":\"a.gxtex\""), "notobj") != 0) { gw_test_fail("a non-object presentation accepted"); rc = 1; }
    gn_registry_clear(&r);
    {
        static char text[1600];
        int i, n = snprintf(text, sizeof text, "{\"geno\":9,\"fighters\":[{\"define\":{\"key\":\"t9\",\"name\":\"T9\",\"base\":\"mario\",\"common\":\"melee.common.v1\",\"resources\":\"retail:mario\"},\"presentation\":{\"stock\":[");
        for (i = 0; i < 17; ++i) n += snprintf(text + n, sizeof text - n, "%s\"s%d.gxtex\"", i ? "," : "", i);
        snprintf(text + n, sizeof text - n, "]}}]}");
        if (gn_parse_text(&r, text, "17") != 0) { gw_test_fail("17 stock icons accepted"); rc = 1; }
        gn_registry_clear(&r);
    }
    (void)buf;
    return rc;
}

/* The .gxtex container the art travels in: the kit decodes a mod's file into the same pool the disc's art uses, and refuses anything else. */
static int test_geno_gxtex_art(void)
{
    extern int gw_Kit_TexAddGxtex(const char* key, const uint8_t* blob, size_t len);
    extern int gw_Kit_TexHsdFind(const char* key);
    extern void gw_Kit_TexDropHsd(void);
    extern const uint8_t* gw_Kit_TexPixels(int tex);
    extern int gw_Kit_TexInfo(int tex, int* w, int* h, float* w1x, float* h1x, int* mask, const char** tint);
    uint8_t blob[64 + 32];
    int tex, w = 0, h = 0, rc = 0;
    const uint8_t* px;
    memset(blob, 0, sizeof blob);
    /* "GXTX", version 1, RGB5A3 (5), 4x4, no TLUT, image 32 bytes at 64 */
    blob[0] = 'G'; blob[1] = 'X'; blob[2] = 'T'; blob[3] = 'X'; blob[7] = 1; blob[11] = 5; blob[15] = 4; blob[19] = 4;
    blob[20] = blob[21] = blob[22] = blob[23] = 0xFF; blob[31] = 32; blob[39] = 64;
    blob[64] = 0xFC; blob[65] = 0x00; /* texel 0: opaque RGB555 1,31,0,0 -> red */
    tex = gw_Kit_TexAddGxtex("gn:test:gxtex", blob, sizeof blob);
    if (tex < 0 || gw_Kit_TexHsdFind("gn:test:gxtex") != tex || gw_Kit_TexAddGxtex("gn:test:gxtex", blob, sizeof blob) != tex) {
        gw_test_fail("a 4x4 RGB5A3 .gxtex was not decoded into the pool (or the key was not the same slot)"); return 1;
    }
    px = gw_Kit_TexPixels(tex);
    if (!gw_Kit_TexInfo(tex, &w, &h, NULL, NULL, NULL, NULL) || w != 4 || h != 4 || !px || px[0] != 255 || px[1] != 0 || px[2] != 0 || px[3] != 255) {
        gw_test_fail("the decoded texel is not opaque red: %dx%d", w, h); rc = 1;
    }
    if (gw_Kit_TexAddGxtex("gn:test:short", blob, 70) != -1) { gw_test_fail("a .gxtex shorter than its image was accepted"); rc = 1; }
    blob[0] = 'X';
    if (gw_Kit_TexAddGxtex("gn:test:magic", blob, sizeof blob) != -1) { gw_test_fail("a file without the GXTX magic was accepted"); rc = 1; }
    blob[0] = 'G'; blob[7] = 2;
    if (gw_Kit_TexAddGxtex("gn:test:ver", blob, sizeof blob) != -1) { gw_test_fail("a .gxtex version 2 was accepted"); rc = 1; }
    blob[7] = 1; blob[11] = 12;
    if (gw_Kit_TexAddGxtex("gn:test:fmt", blob, sizeof blob) != -1) { gw_test_fail("an unknown GX format was accepted"); rc = 1; }
    if (gw_Kit_TexAddGxtex(NULL, blob, sizeof blob) != -1 || gw_Kit_TexAddGxtex("gn:test:null", NULL, 0) != -1) { gw_test_fail("NULL arguments were accepted"); rc = 1; }
    /* the same bytes through the game-side handle API (the HUD stock icon and the results screen copy from it) */
    {
        extern int gw_GxTex_OpenBlob(const unsigned char* bytes, uint32_t len, const char* label);
        extern int gw_GxTex_Width(int h); extern int gw_GxTex_Height(int h); extern int gw_GxTex_Format(int h);
        extern int gw_GxTex_ImageSize(int h); extern int gw_GxTex_TlutFormat(int h);
        extern void gw_GxTex_CopyImage(int h, void* dst); extern void gw_GxTex_Close(int h);
        uint8_t copy[32];
        int g;
        blob[11] = 5; blob[7] = 1;
        g = gw_GxTex_OpenBlob(blob, sizeof blob, "gn:test");
        if (g < 0 || gw_GxTex_Width(g) != 4 || gw_GxTex_Height(g) != 4 || gw_GxTex_Format(g) != 5 || gw_GxTex_ImageSize(g) != 32 || gw_GxTex_TlutFormat(g) != -1) {
            gw_test_fail("gw_GxTex_OpenBlob: handle %d", g); rc = 1;
        } else {
            memset(copy, 0, sizeof copy);
            gw_GxTex_CopyImage(g, copy);
            if (copy[0] != 0xFC || copy[1] != 0x00) { gw_test_fail("gw_GxTex_CopyImage did not copy the payload bytes"); rc = 1; }
            gw_GxTex_Close(g);
            if (gw_GxTex_Width(g) != 0) { gw_test_fail("a closed handle still answers"); rc = 1; }
        }
        if (gw_GxTex_OpenBlob(blob, 20, "gn:short") != -1 || gw_GxTex_OpenBlob(NULL, 0, "gn:null") != -1) { gw_test_fail("a short blob was opened"); rc = 1; }
        blob[0] = 'X';
        if (gw_GxTex_OpenBlob(blob, sizeof blob, "gn:magic") != -1) { gw_test_fail("a blob without the magic was opened"); rc = 1; }
    }
    gw_Kit_TexDropHsd();
    return rc;
}

/* The same, through the live registry by CK (the way the menus and the match setup ask): install a base "none" define for the test, resolve its CK,
 * read its costume count, team colours, names and art flags; then restore the caller's registry. A missing art file answers -1 and does not crash. */
static int test_geno_define_presentation_ck(void)
{
    extern int gw_Geno_DefineArtTex(int ck, int what, int costume);
    extern int gw_Geno_DefineArtOpen(int ck, int what, int costume);
    gn_registry entry = gn_boot, entry_saved = gn_saved;
    int entry_loaded = gn_boot_loaded, entry_saved_loaded = gn_saved_loaded;
    int entry_swapped = gn_swapped;
    int rc = 0, ck;
    char name[32];
    memset(&gn_boot, 0, sizeof gn_boot);
    memset(&gn_saved, 0, sizeof gn_saved);
    gn_boot_loaded = 1; gn_saved_loaded = 0; gn_swapped = 0;
    if (gw_Geno_TestInstall(GN_T_NONE(9, GN_T_COS("a.dat", ",\"name\":\"Plain\"") "," GN_T_COS("b.dat", ",\"name\":\"Ember\",\"team\":\"red\"") ","
                                         GN_T_COS("c.dat", ",\"team\":\"blue\"") "," GN_T_COS("d.dat", ",\"team\":\"green\""),
                            ",\"presentation\":{\"icon\":\"gn_absent_icon.gxtex\",\"stock\":[\"gn_absent_s0.gxtex\"]}")) != 1) {
        gw_test_fail("install of a none define with art"); rc = 1;
    } else if ((ck = gw_Geno_DefineResolve("t9")) < 34) {
        gw_test_fail("the none define has no resident CK (%d)", ck); rc = 1;
    } else {
        if (gw_Geno_DefineCostumeCountCK(ck) != 4 || gw_Geno_DefineTeamCostume(ck, 0) != 1 || gw_Geno_DefineTeamCostume(ck, 1) != 2 ||
            gw_Geno_DefineTeamCostume(ck, 2) != 3) {
            gw_test_fail("by CK: costumes %d, team colours %d %d %d", gw_Geno_DefineCostumeCountCK(ck), gw_Geno_DefineTeamCostume(ck, 0),
                         gw_Geno_DefineTeamCostume(ck, 1), gw_Geno_DefineTeamCostume(ck, 2)); rc = 1;
        }
        if (!gw_Geno_DefineCostumeName(ck, 1, name, sizeof name) || strcmp(name, "Ember") || gw_Geno_DefineCostumeName(ck, 2, name, sizeof name) ||
            gw_Geno_DefineCostumeName(ck, 4, name, sizeof name)) {
            gw_test_fail("by CK: costume names"); rc = 1;
        }
        if (!gw_Geno_DefineHasArt(ck, 0, 0) || gw_Geno_DefineHasArt(ck, 1, 0) || !gw_Geno_DefineHasArt(ck, 2, 3)) { gw_test_fail("by CK: art flags"); rc = 1; }
        if (gw_Geno_DefineArtTex(ck, 0, 0) != -1 || gw_Geno_DefineArtOpen(ck, 2, 0) != -1 || gw_Geno_DefineArtTex(ck, 1, 0) != -1) {
            gw_test_fail("a missing or undeclared art file did not answer -1"); rc = 1;
        }
        /* a retail CK, CK 0 and a CK no define holds are not none defines: the callers keep their own tables */
        if (gw_Geno_DefineCostumeCountCK(8) != 0 || gw_Geno_DefineCostumeCountCK(0) != 0 || gw_Geno_DefineCostumeCountCK(255) != 0 ||
            gw_Geno_DefineTeamCostume(8, 0) != -1 || gw_Geno_DefineHasArt(8, 0, 0)) {
            gw_test_fail("a retail CK answered a define's costumes or art"); rc = 1;
        }
    }
    gw_Geno_TestRestore();
    gn_registry_clear(&gn_boot);
    gn_registry_clear(&gn_saved);
    gn_boot = entry; gn_saved = entry_saved;
    gn_boot_loaded = entry_loaded; gn_saved_loaded = entry_saved_loaded;
    gn_swapped = entry_swapped; ++gn_pool_gen;
    return rc;
}
