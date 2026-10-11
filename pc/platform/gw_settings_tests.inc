/* Native settings regressions: real environment, file I/O and accessors. Restore
 * process settings and environment so these tests cannot steer later tests. */
typedef struct {
    const char *name;
    const char *value;
    char *before;
} SettingsTestEnv;

typedef struct {
    void *state;
    int save_ok;
    char path[MAX_PATH];
    SettingsTestEnv *env;
    int env_count;
} SettingsTestFixture;

static void st_test_end(SettingsTestFixture *f) {
    int i;
    for (i = 0; i < f->env_count; ++i) {
        _putenv_s(f->env[i].name, f->env[i].before != NULL ? f->env[i].before : "");
        free(f->env[i].before);
    }
    if (f->path[0] != '\0') DeleteFileA(f->path);
    if (f->state != NULL) {
        memcpy(&st, f->state, sizeof st);
        free(f->state);
    }
    st_save_ok = f->save_ok;
}

static int st_test_begin(SettingsTestFixture *f, SettingsTestEnv *env, int count) {
    char temp[MAX_PATH];
    DWORD n;
    int i, len;
    memset(f, 0, sizeof *f);
    f->save_ok = st_save_ok;
    f->env = env;
    f->state = malloc(sizeof st);
    if (f->state == NULL) goto fail;
    memcpy(f->state, &st, sizeof st);
    /* Snapshot every variable before changing any, including inherited overrides. */
    for (i = 0; i < count; ++i) {
        const char *v = getenv(env[i].name);
        env[i].before = v != NULL ? _strdup(v) : NULL;
        if (v != NULL && env[i].before == NULL) goto fail;
        f->env_count++;
    }
    for (i = 0; i < count; ++i) {
        if (_putenv_s(env[i].name, env[i].value != NULL ? env[i].value : "") != 0) goto fail;
    }
    n = GetTempPathA(sizeof temp, temp);
    if (n == 0 || n >= sizeof temp) goto fail;
    len = snprintf(f->path, sizeof f->path, "%sgw_settings_%lu.cfg", temp,
                   (unsigned long) GetCurrentProcessId());
    if (len < 0 || len >= (int) sizeof f->path) { f->path[0] = '\0'; goto fail; }
    DeleteFileA(f->path);
    memset(&st, 0, sizeof st);
    snprintf(st.path, sizeof st.path, "%s", f->path);
    st_save_ok = 1;
    return 1;
fail:
    gw_test_fail("settings test fixture could not initialize");
    st_test_end(f);
    return 0;
}

/* Catches cfg-only reads, missing-key overrides, cached overrides, lost zero /
 * negative values and a truncated environment-name lookup. */
static int test_settings_env_int(void) {
    SettingsTestEnv env[] = {
        { "MELEE_ATLAS_SELECT_SOLO", "1", NULL },
        { "MELEE_SETTINGS_TEST_MISSING", "9", NULL },
        { "MELEE_SETTINGS_TEST_NUMBER", " -12tail", NULL },
        { "MELEE_ENV_TEST_1234567890123456789012", "31", NULL },
    };
    SettingsTestFixture f;
    int rc = 0;
    if (!st_test_begin(&f, env, sizeof env / sizeof env[0])) return 1;
    gw_Settings_SetInt("atlas_select_solo", 0);
    gw_Settings_SetInt("settings_test_number", 4);
    if (gw_Settings_Int("atlas_select_solo", 7) != 1 ||
        gw_Settings_Int("settings_test_missing", 7) != 9 ||
        gw_Settings_Int("Settings_Test_Number", 7) != -12) {
        gw_test_fail("MELEE_<UPPERCASE_KEY> did not override integer reads"); rc = 1;
    }
    _putenv_s("MELEE_ATLAS_SELECT_SOLO", "0");
    if (gw_Settings_Int("atlas_select_solo", 7) != 0) {
        gw_test_fail("zero override or live environment update was lost"); rc = 1;
    }
    _putenv_s("MELEE_ATLAS_SELECT_SOLO", "");
    _putenv_s("MELEE_SETTINGS_TEST_MISSING", "");
    if (gw_Settings_Int("atlas_select_solo", 7) != 0 ||
        gw_Settings_Int("settings_test_missing", 7) != 7) {
        gw_test_fail("empty environment did not fall back to cfg/default"); rc = 1;
    }
    _putenv_s("MELEE_SETTINGS_TEST_NUMBER", "invalid");
    if (gw_Settings_Int("settings_test_number", 7) != 0) {
        gw_test_fail("environment integers did not retain atoi semantics"); rc = 1;
    }
    if (gw_Settings_Int("env_test_1234567890123456789012", 7) != 31 ||
        gw_Settings_Int("env_test_1234567890123456789012_extra", 7) != 7) {
        gw_test_fail("maximum-length key or overlong-key isolation failed"); rc = 1;
    }
    st_test_end(&f);
    return rc;
}

/* Catches inconsistent Int/Str precedence and the found flag/buffer contract. */
static int test_settings_env_str(void) {
    SettingsTestEnv env[] = {
        { "MELEE_NAME", "Preview", NULL },
        { "MELEE_CTL15_P3_NAME", "Tournament", NULL },
        { "MELEE_SETTINGS_TEST_MISSING", NULL, NULL },
    };
    SettingsTestFixture f;
    char b[32], short_b[4], one = 'x';
    int rc = 0;
    if (!st_test_begin(&f, env, sizeof env / sizeof env[0])) return 1;
    gw_Settings_SetStr("name", "Saved player");
    if (!gw_Settings_Str("name", b, sizeof b, "default") || strcmp(b, "Preview") ||
        !gw_Settings_Str("ctl15_p3_name", b, sizeof b, "default") || strcmp(b, "Tournament")) {
        gw_test_fail("MELEE_<UPPERCASE_KEY> did not override string reads"); rc = 1;
    }
    if (!gw_Settings_Str("name", short_b, sizeof short_b, "") || strcmp(short_b, "Pre") ||
        !gw_Settings_Str("name", &one, 1, "") || one != '\0' ||
        !gw_Settings_Str("ctl15_p3_name", NULL, 0, "")) {
        gw_test_fail("environment string truncation/found flag failed"); rc = 1;
    }
    _putenv_s("MELEE_NAME", "");
    if (!gw_Settings_Str("name", b, sizeof b, "default") || strcmp(b, "Saved player") ||
        gw_Settings_Str("settings_test_missing", b, sizeof b, "default") || strcmp(b, "default")) {
        gw_test_fail("empty string environment did not preserve cfg/default contract"); rc = 1;
    }
    st_test_end(&f);
    return rc;
}

/* Catches accidentally caching an effective value in st, adding an env-only key
 * to the file, or serializing overrides when another setting is saved. */
static int test_settings_env_persistence(void) {
    SettingsTestEnv env[] = {
        { "MELEE_ATLAS_SELECT_SOLO", "1", NULL },
        { "MELEE_NAME", "Preview", NULL },
        { "MELEE_SETTINGS_TEST_MISSING", "9", NULL },
        { "MELEE_SETTINGS_TEST_UNKNOWN", NULL, NULL },
    };
    SettingsTestFixture f;
    char b[256], summary[512];
    FILE *file;
    int rc = 0, solo = 0, name = 0, unknown = 0;
    if (!st_test_begin(&f, env, sizeof env / sizeof env[0])) return 1;
    gw_Settings_SetInt("atlas_select_solo", 0);
    gw_Settings_SetStr("name", "Saved player");
    (void) gw_Settings_Int("atlas_select_solo", 7);
    (void) gw_Settings_Int("settings_test_missing", 7);
    gw_Settings_Str("name", b, sizeof b, "");
    gw_Settings_SetStr("settings_test_unknown", "keep"); /* unrelated save */
    file = fopen(f.path, "r");
    if (file == NULL) { gw_test_fail("settings persistence file missing"); rc = 1; }
    else {
        while (fgets(b, sizeof b, file) != NULL) {
            st_trim(b);
            if (!strcmp(b, "atlas_select_solo=0")) solo++;
            else if (!strcmp(b, "name=Saved player")) name++;
            else if (!strcmp(b, "settings_test_unknown=keep")) unknown++;
            else if (b[0] != '#') { gw_test_fail("environment value leaked into settings file"); rc = 1; }
        }
        fclose(file);
        if (solo != 1 || name != 1 || unknown != 1) {
            gw_test_fail("saving another key replaced persisted values"); rc = 1;
        }
    }
    gw_Settings_SetInt("atlas_select_solo", 2);
    gw_Settings_SetStr("name", "Edited player");
    memset(&st, 0, sizeof st);
    snprintf(st.path, sizeof st.path, "%s", f.path);
    if (gw_Settings_Int("atlas_select_solo", 7) != 1 ||
        !gw_Settings_Str("name", b, sizeof b, "") || strcmp(b, "Preview")) {
        gw_test_fail("reload/setter bypassed the environment override"); rc = 1;
    }
    gw_Settings_Summary(summary, sizeof summary);
    if (strstr(summary, "atlas_select_solo=2") == NULL || strstr(summary, "Preview") != NULL ||
        strstr(summary, "settings_test_missing") != NULL) {
        gw_test_fail("settings summary reported overrides rather than stored values"); rc = 1;
    }
    _putenv_s("MELEE_ATLAS_SELECT_SOLO", "");
    _putenv_s("MELEE_NAME", "");
    _putenv_s("MELEE_SETTINGS_TEST_MISSING", "");
    if (gw_Settings_Int("atlas_select_solo", 7) != 2 ||
        !gw_Settings_Str("name", b, sizeof b, "") || strcmp(b, "Edited player") ||
        gw_Settings_Int("settings_test_missing", 7) != 7 ||
        !gw_Settings_Str("settings_test_unknown", b, sizeof b, "") || strcmp(b, "keep")) {
        gw_test_fail("clearing overrides did not reveal the saved values"); rc = 1;
    }
    st_test_end(&f);
    return rc;
}
