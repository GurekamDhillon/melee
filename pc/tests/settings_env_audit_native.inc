/* Live source guard for settings/env aliases, included only by gw_settings.c's
 * native tests. A frozen list misses new getters and new tooling variables.
 * Read the selected game checkout and workspace tools; never write either.
 * This is a conservative token audit, not a C/Python/shell interpreter. */
#ifndef _WIN32
#include <dirent.h>
#include <sys/stat.h>
#endif
#include <ctype.h>

#define SA_MAX 1024
#define SA_NAME 96
#define SA_PATH 1024
typedef struct {
    char names[SA_MAX][SA_NAME];
    unsigned char prefixes[SA_MAX];
    char keys[SA_MAX][ST_KEY];
    int nn, nk, files, failed;
} SettingsAudit;

static void sa_add(SettingsAudit *a, const char *s, int key) {
    int i, n = key ? a->nk : a->nn;
    size_t cap = key ? ST_KEY : SA_NAME;
    if (!s[0] || strlen(s) >= cap || n >= SA_MAX) {
        gw_test_fail("settings env audit: token capacity exceeded: %s", s);
        a->failed = 1;
        return;
    }
    for (i = 0; i < n; ++i)
        if (!strcmp(key ? a->keys[i] : a->names[i], s)) return;
    strcpy(key ? a->keys[a->nk++] : a->names[a->nn++], s);
}

/* Preserve strings and newlines while removing comments, so documentation and
 * include guards do not masquerade as independent environment readers. */
static void sa_comments(char *s, int script, int lua) {
    char quote = 0;
    size_t i;
    for (i = 0; s[i]; ++i) {
        if (quote) {
            if (s[i] == '\\' && s[i + 1]) ++i;
            else if (s[i] == quote) quote = 0;
        } else if (s[i] == '"' || s[i] == '\'') quote = s[i];
        else if ((!script && s[i] == '/' && s[i + 1] == '/') ||
                 (script && s[i] == '#') || (lua && s[i] == '-' && s[i + 1] == '-')) {
            while (s[i] && s[i] != '\n') s[i++] = ' ';
            if (!s[i]) break;
        } else if (!script && s[i] == '/' && s[i + 1] == '*') {
            s[i++] = ' '; s[i++] = ' ';
            while (s[i] && !(s[i] == '*' && s[i + 1] == '/')) {
                if (s[i] != '\n') s[i] = ' ';
                ++i;
            }
            if (!s[i]) break;
            s[i] = ' '; s[i + 1] = ' '; ++i;
        }
    }
}

static const char *sa_space(const char *s) {
    while (isspace((unsigned char) *s)) ++s;
    return s;
}

static int sa_literal_text(const char *s, char *out, size_t cap, const char **tail) {
    size_t n = 0;
    s = sa_space(s);
    if (*s != '"' && *s != '\'') return 0;
    /* Adjacent string literals are one C value ("vol" "ume" is "volume"). */
    do {
        char quote = *s++;
        while (*s && *s != quote) {
            if (*s == '\\' || n + 1 >= cap) return 0;
            out[n++] = *s++;
        }
        if (!*s) return 0;
        s = sa_space(s + 1);
    } while (*s == '"' || *s == '\'');
    out[n] = 0;
    if (tail) *tail = s;
    return n != 0;
}

static int sa_literal(const char *s, char *out, size_t cap) {
    const char *end;
    if (!sa_literal_text(s, out, cap, &end)) return 0;
    /* Reject partial expressions: prefix + variable, casts, indexing, etc.
     * Each resolver checks this is a complete call/table/initializer value. */
    return *end == ',' || *end == ')' || *end == '}' || *end == ';';
}

static void sa_env_literal(SettingsAudit *a, const char *s) {
    size_t n;
    char name[SA_NAME];
    if (strncmp(s, "MELEE_", 6)) return;
    for (n = 6; (s[n] >= 'A' && s[n] <= 'Z') || isdigit((unsigned char) s[n]) || s[n] == '_'; ++n) {
        if (n + 1 >= sizeof name) { a->failed = 1; return; }
    }
    if (n == 6) return;
    memcpy(name, s, n); name[n] = 0;
    sa_add(a, name, 0);
}

static int sa_ident(char c) {
    return isalnum((unsigned char) c) || c == '_';
}

/* Match identifiers outside strings, so declarations and calls can be audited
 * without being confused by log messages or ordinary C whitespace changes. */
static const char *sa_token(const char *s, const char *token) {
    const char *start = s;
    size_t n = strlen(token);
    for (; *s; ++s) {
        if (*s == '"' || *s == '\'') {
            char quote = *s++;
            while (*s && *s != quote) {
                if (*s == '\\' && s[1]) ++s;
                ++s;
            }
            if (!*s) return NULL;
        } else if ((s == start || !sa_ident(s[-1])) && !strncmp(s, token, n) && !sa_ident(s[n])) return s;
    }
    return NULL;
}

static const char *sa_close(const char *s, char open, char close) {
    int depth = 0;
    for (; *s; ++s) {
        if (*s == '"' || *s == '\'') {
            char quote = *s++;
            while (*s && *s != quote) {
                if (*s == '\\' && s[1]) ++s;
                ++s;
            }
            if (!*s) return NULL;
        } else if (*s == open) ++depth;
        else if (*s == close && --depth == 0) return s;
    }
    return NULL;
}

/* Return one call argument with insignificant whitespace removed. */
static const char *sa_arg(const char *s, char *out, size_t cap) {
    int parens = 0, brackets = 0;
    size_t n = 0;
    for (s = sa_space(s); *s; ++s) {
        if (!parens && !brackets && (*s == ',' || *s == ')')) break;
        if (*s == '(') ++parens;
        if (*s == ')') --parens;
        if (*s == '[') ++brackets;
        if (*s == ']') --brackets;
        if (isspace((unsigned char) *s)) continue;
        if (*s == '"' || *s == '\'' || n + 1 >= cap) return NULL;
        out[n++] = *s;
    }
    out[n] = 0;
    return *s && n ? s : NULL;
}

static int sa_in_function(const char *s, const char *call, const char *function) {
    const char *p = s;
    while ((p = sa_token(p, function)) != NULL) {
        const char *q = sa_space(p + strlen(function)), *end;
        ++p;
        if (*q != '(' || !(end = sa_close(q, '(', ')'))) continue;
        q = sa_space(end + 1);
        if (*q == '{' && (end = sa_close(q, '{', '}')) && q < call && call < end) return 1;
    }
    return 0;
}

static void sa_unknown(SettingsAudit *a, const char *s, const char *p, const char *path) {
    int line = 1;
    const char *q;
    for (q = s; q < p; ++q) if (*q == '\n') ++line;
    a->failed = 1;
    gw_test_fail("settings env audit: unresolved settings key at %s:%d; update the resolver", path, line);
}

static int sa_basename(const char *path, const char *name) {
    const char *p = strrchr(path, '/'), *back = strrchr(path, '\\');
    if (back && (!p || back > p)) p = back;
    return !strcmp(p ? p + 1 : path, name);
}

static int sa_forwarded(const char *s, const char *call, const char *arg, const char *path) {
    static const char *controls[] = { "ctl_device", "ctl_load", "ctl_save",
        "gw_Controls_SetProfile", "gw_Controls_ProfileName", "gw_Controls_ProfileAction" };
    size_t i;
    if (!strcmp(arg, "key")) {
        if (sa_basename(path, "gw_settings.c") && sa_in_function(s, call, "gw_Settings_SetInt")) return 1;
        if (sa_basename(path, "shim_pad.c") && sa_in_function(s, call, "gw_pad_setting")) return 1;
        if (sa_basename(path, "gmfrontend_settings.inc") &&
            (sa_in_function(s, call, "fte_begin") || sa_in_function(s, call, "fte_fmt"))) return 1;
        if (sa_basename(path, "gw_controls_runtime.inc"))
            for (i = 0; i < sizeof controls / sizeof controls[0]; ++i)
                if (sa_in_function(s, call, controls[i])) return 1;
    }
    if (!strcmp(arg, "fte.key") && sa_basename(path, "gmfrontend_settings.inc") && sa_in_function(s, call, "fte_frame")) return 1;
    if (sa_basename(path, "gw_netplay.c") &&
        (!strcmp(arg, "rn_kv_key[slot][0]") || !strcmp(arg, "rn_kv_key[slot][1]") || !strcmp(arg, "rn_kv_key[slot][2]")) &&
        (sa_in_function(s, call, "rn_kv_get") || sa_in_function(s, call, "rn_kv_set"))) return 1;
    return 0;
}

static const char *sa_before(const char *s, const char *p, const char *token) {
    size_t n = strlen(token);
    while (p > s && isspace((unsigned char) p[-1])) --p;
    return (size_t) (p - s) >= n && !strncmp(p - n, token, n) &&
           (p - n == s || !sa_ident(p[-(int) n - 1])) ? p - n : NULL;
}

static int sa_global(const char *s, const char *p) {
    int depth = 0;
    for (; s < p; ++s) {
        if (*s == '"' || *s == '\'') {
            char quote = *s++;
            while (s < p && *s != quote) {
                if (*s == '\\' && s + 1 < p) ++s;
                ++s;
            }
        } else if (*s == '{') ++depth;
        else if (*s == '}') --depth;
    }
    return depth == 0;
}

static int sa_const_key(const char *s, const char *arg, char *key) {
    const char *p = s;
    int found = 0;
    size_t i;
    if (!isalpha((unsigned char) arg[0]) && arg[0] != '_') return 0;
    for (i = 0; arg[i]; ++i) if (!sa_ident(arg[i])) return 0;
    while ((p = sa_token(p, arg)) != NULL) {
        const char *decl = p, *before, *bracket = NULL;
        const char *q = sa_space(p + strlen(arg));
        ++p;
        if (*q == '[') {
            bracket = q;
            q = sa_close(bracket, '[', ']');
            if (!q) return 0;
            q = sa_space(q + 1);
        }
        if (*q != '=' || q[1] == '=') {
            const char *left = decl;
            while (left > s && isspace((unsigned char) left[-1])) --left;
            /* Modeled globals are used as call arguments. A same-spelled
             * parameter/local declaration (including one without '=') is
             * ambiguous; do not borrow the global value across that scope. */
            if (left > s && (left[-1] == '(' || left[-1] == ',' ||
                (left - s >= 2 && left[-1] == '=' && left[-2] == '='))) continue;
            return 0;
        }
        /* Resolve only unique immutable globals: current fso_key_name/server
         * arrays. Local pointers, shadowing and reassignment must fail rather
         * than borrowing a same-spelled initializer from another scope. */
        before = sa_before(s, decl, "char");
        if (before) before = sa_before(s, before, "const");
        if (before) before = sa_before(s, before, "static");
        if (!bracket || *sa_space(bracket + 1) != ']' || !before || !sa_global(s, decl) ||
            ++found != 1 || !sa_literal(q + 1, key, ST_KEY)) return 0;
    }
    return found == 1;
}

/* All callers of modeled forwarding wrappers must have resolvable keys too. */
static void sa_wrapper(SettingsAudit *a, const char *s, const char *name, int index, const char *path) {
    const char *p = s;
    while ((p = sa_token(p, name)) != NULL) {
        const char *call = p, *q = sa_space(p + strlen(name));
        char key[ST_KEY], arg[SA_NAME];
        int i;
        ++p;
        if (*q != '(') continue;
        q = sa_space(q + 1);
        if (!strncmp(q, "const", 5) && !sa_ident(q[5])) continue; /* declaration */
        for (i = 0; i < index && q; ++i) {
            if (*q == '"') {
                const char *end;
                q = sa_literal_text(q, arg, sizeof arg, &end) ? end : NULL;
            } else q = sa_arg(q, arg, sizeof arg);
            if (q && *q == ',') q = sa_space(q + 1); else q = NULL;
        }
        if (q && sa_literal(q, key, sizeof key)) sa_add(a, key, 1);
        else if (q && sa_arg(q, arg, sizeof arg) && sa_const_key(s, arg, key)) sa_add(a, key, 1);
        else sa_unknown(a, s, call, path);
    }
}

static int sa_bound(const char *s, const char *name) {
    const char *p = s;
    while ((p = sa_token(p, name)) != NULL) {
        const char *q = sa_space(p + strlen(name));
        ++p;
        if (*q == '=' && isdigit((unsigned char) *(q = sa_space(q + 1)))) {
            char *end;
            long value = strtol(q, &end, 0);
            end = (char *) sa_space(end);
            if (value > 0 && value <= 100 && (*end == ',' || *end == '}' || *end == ';')) return (int) value;
            return 0; /* expressions/suffixes need a resolver, never atoi's prefix */
        }
    }
    return 0;
}

static void sa_controls(SettingsAudit *a, const char *s, const char *path) {
    int devices = sa_bound(s, "CTL_DEVICES"), profiles = sa_bound(s, "CTL_PROFILES"), d, pr;
    const char *p = s;
    char key[ST_KEY], arg[SA_NAME];
    if (devices <= 0 || devices > 100 || profiles <= 0 || profiles > 10) {
        a->failed = 1; gw_test_fail("settings env audit: controller bounds not understood"); return;
    }
    /* Validate actual key builders, including new fields, rather than assuming
     * that every indirect use in this file still produces the old 160 keys. */
    while ((p = sa_token(p, "snprintf")) != NULL) {
        const char *call = p, *q = sa_space(p + strlen("snprintf"));
        ++p;
        if (*q != '(' || !(q = sa_arg(q + 1, arg, sizeof arg)) || strcmp(arg, "key")) continue;
        if (*q != ',' || !(q = sa_arg(q + 1, arg, sizeof arg)) || *q != ',' || !sa_literal(q + 1, key, sizeof key)) {
            sa_unknown(a, s, call, path); continue;
        }
        if (!strcmp(key, "ctl%02d_id") || !strcmp(key, "ctl%02d_active")) {
            char expanded[ST_KEY];
            for (d = 0; d < devices; ++d) {
                snprintf(expanded, sizeof expanded, key, d); sa_add(a, expanded, 1);
            }
        } else if (!strcmp(key, "ctl%02d_p%d_%s")) { /* ctl_key's fields below */ }
        else if (!strchr(key, '%')) sa_add(a, key, 1);
        else sa_unknown(a, s, call, path);
    }
    p = s;
    while ((p = sa_token(p, "ctl_key")) != NULL) {
        const char *call = p, *q = sa_space(p + strlen("ctl_key"));
        int i;
        ++p;
        if (*q != '(') continue;
        q = sa_space(q + 1);
        if (!strncmp(q, "char", 4) && !sa_ident(q[4])) continue; /* declaration */
        for (i = 0; i < 3 && q; ++i) {
            q = sa_arg(q, arg, sizeof arg);
            if (q && *q == ',') q = sa_space(q + 1); else q = NULL;
        }
        if (!q || !sa_literal(q, key, sizeof key)) { sa_unknown(a, s, call, path); continue; }
        for (d = 0; d < devices; ++d) for (pr = 0; pr < profiles; ++pr) {
            char expanded[ST_KEY];
            int n = snprintf(expanded, sizeof expanded, "ctl%02d_p%d_%s", d, pr, key);
            if (n >= (int) sizeof expanded) { sa_unknown(a, s, call, path); return; }
            sa_add(a, expanded, 1);
        }
    }
}

static void sa_envs(SettingsAudit *a, const char *s, int script) {
    const char *start = s, *p;
    char quote = 0;
    for (; *s; ++s) {
        if (!strncmp(s, "MELEE_", 6) && (script || quote) &&
            (s == start || (!isalnum((unsigned char) s[-1]) && s[-1] != '_'))) {
            char name[SA_NAME];
            size_t n = 6;
            while ((s[n] >= 'A' && s[n] <= 'Z') || (s[n] >= '0' && s[n] <= '9') || s[n] == '_') ++n;
            if (n == 6) continue; /* MELEE_<UPPERCASE_KEY> is a rule, not a name. */
            if (n >= sizeof name) { a->failed = 1; gw_test_fail("settings env audit: long env token"); return; }
            memcpy(name, s, n); name[n] = 0;
            sa_add(a, name, 0);
            s += n - 1;
        } else if (quote) {
            if (*s == '\\' && s[1]) ++s;
            else if (*s == quote) quote = 0;
        } else if (*s == '"' || *s == '\'') {
            char joined[SA_NAME];
            if (sa_literal_text(s, joined, sizeof joined, NULL)) sa_env_literal(a, joined);
            quote = *s;
        }
    }
    /* l_lab_env builds MELEE_LAB_<name>; the concrete Lua callsites may have no
     * literal MELEE spelling anywhere. Also retain the open prefix itself. */
    p = start;
    while ((p = sa_token(p, "lab_env")) != NULL) {
        char suffix[SA_NAME], name[SA_NAME];
        const char *q = sa_space(p + strlen("lab_env"));
        if (*q == '(' && sa_literal(q + 1, suffix, sizeof suffix)) {
            int n = snprintf(name, sizeof name, "MELEE_LAB_%s", suffix);
            if (n >= (int) sizeof name) { a->failed = 1; gw_test_fail("settings env audit: long LAB name"); return; }
            sa_add(a, name, 0);
        }
        ++p;
    }
    /* Qt/.NET/Python prefix operations reserve namespaces even without a final
     * underscore (launcher_core clears MELEE_NET*, MELEE_SLIPPI*, MELEE_SCENE*). */
    { static const char *methods[] = { "startsWith", "StartsWith", "startswith" };
      size_t m;
      for (m = 0; m < sizeof methods / sizeof methods[0]; ++m) {
          p = start;
          while ((p = sa_token(p, methods[m])) != NULL) {
              char name[SA_NAME];
              const char *q = sa_space(p + strlen(methods[m]));
              ++p;
              if (*q == '(' && sa_literal(q + 1, name, sizeof name) && !strncmp(name, "MELEE_", 6) && name[6]) {
                  int i;
                  /* Whole-environment snapshots/clean boots use MELEE_ itself;
                   * command-line checks can include '='. Neither names a
                   * feature namespace whose meaning could alias a setting. */
                  for (i = 0; name[i]; ++i)
                      if (!(name[i] >= 'A' && name[i] <= 'Z') && !isdigit((unsigned char) name[i]) && name[i] != '_') break;
                  if (name[i]) continue;
                  sa_add(a, name, 0);
                  for (i = 0; i < a->nn; ++i) if (!strcmp(a->names[i], name)) a->prefixes[i] = 1;
              }
          }
      }
    }
}

static void sa_keys(SettingsAudit *a, const char *s, const char *path) {
    const char *p = s;
    char key[ST_KEY];
    static const char *calls[] = { "Settings_Int", "Settings_Str", "Settings_SetInt", "Settings_SetStr",
        "gw_Settings_Int", "gw_Settings_Str", "gw_Settings_SetInt", "gw_Settings_SetStr" };
    size_t c;
    for (c = 0; c < sizeof calls / sizeof calls[0]; ++c) {
        p = s;
        while ((p = sa_token(p, calls[c])) != NULL) {
            const char *call = p, *q = sa_space(p + strlen(calls[c]));
            char arg[SA_NAME];
            ++p;
            if (*q != '(') { sa_unknown(a, s, call, path); continue; }
            q = sa_space(q + 1);
            if (!strncmp(q, "const", 5) && !sa_ident(q[5])) continue; /* declaration */
            if (*q == '"' && sa_literal(q, key, sizeof key)) sa_add(a, key, 1);
            else if (!sa_arg(q, arg, sizeof arg) || !sa_forwarded(s, call, arg, path)) sa_unknown(a, s, call, path);
        }
    }
    sa_wrapper(a, s, "gw_pad_setting", 1, path);
    sa_wrapper(a, s, "fte_begin", 0, path);
    sa_wrapper(a, s, "fte_fmt", 0, path);
    if (sa_basename(path, "gw_netplay.c")) {
        int found = 0;
        p = s;
        while ((p = sa_token(p, "rn_kv_key")) != NULL) {
            const char *end = strchr(p, ';'), *eq = strchr(p, '=');
            ++p;
            if (end && eq && eq < end && *sa_space(eq + 1) == '{') {
                for (p = eq + 1; p < end; ++p) {
                    if (isspace((unsigned char) *p) || *p == '{' || *p == '}' || *p == ',') continue;
                    if (*p != '"' || !sa_literal(p, key, sizeof key)) { sa_unknown(a, s, p, path); break; }
                    sa_add(a, key, 1); ++found;
                    { const char *tail;
                      sa_literal_text(p, key, sizeof key, &tail);
                      p = tail - 1; }
                }
                break;
            }
        }
        if (!found) sa_unknown(a, s, s, path);
    }
    if (sa_basename(path, "gw_controls_runtime.inc")) sa_controls(a, s, path);
}

static int sa_extension(const char *path, int *script, int *lua) {
    const char *ext = strrchr(path, '.');
    char lower[16];
    size_t n;
    *lua = 0;
    if (sa_basename(path, "launch-melee")) { *script = 1; return 1; }
    if (!ext || strlen(ext) >= sizeof lower) return 0;
    for (n = 0; ext[n]; ++n) lower[n] = (char) tolower((unsigned char) ext[n]);
    lower[n] = 0;
    ext = lower;
    *lua = !strcmp(ext, ".lua");
    *script = *lua || !strcmp(ext, ".py") || !strcmp(ext, ".sh") ||
              !strcmp(ext, ".ps1") || !strcmp(ext, ".bat") || !strcmp(ext, ".cmd") ||
              !strcmp(ext, ".yml") || !strcmp(ext, ".yaml");
    return *script || !strcmp(ext, ".c") || !strcmp(ext, ".h") ||
           !strcmp(ext, ".cpp") || !strcmp(ext, ".hpp") || !strcmp(ext, ".inc") || !strcmp(ext, ".cs");
}

static void sa_file(SettingsAudit *a, const char *path, int settings) {
    FILE *f;
    char *s, *tests;
    long size;
    int script, lua;
    if (!sa_extension(path, &script, &lua) || strstr(path, "gw_settings_tests.inc") ||
        strstr(path, "settings_env_audit_native.inc")) return;
    f = fopen(path, "rb");
    if (!f) goto fail;
    if (fseek(f, 0, SEEK_END) || (size = ftell(f)) < 0 || fseek(f, 0, SEEK_SET)) { fclose(f); goto fail; }
    s = (char *) malloc((size_t) size + 1);
    if (!s) { fclose(f); goto fail; }
    if (fread(s, 1, (size_t) size, f) != (size_t) size) { free(s); fclose(f); goto fail; }
    fclose(f); s[size] = 0;
    /* Some existing source files contain literal NULs. They must not hide their
     * trailing environment readers from this audit. */
    { long i; for (i = 0; i < size; ++i) if (!s[i]) s[i] = ' '; }
    if (strstr(path, "gw_settings.c") && (tests = strstr(s, "/* ---- tests")) != NULL) *tests = 0;
    sa_comments(s, script, lua);
    sa_envs(a, s, script);
    if (settings && !script) sa_keys(a, s, path);
    free(s); ++a->files;
    return;
fail:
    a->failed = 1;
    gw_test_fail("settings env audit: cannot read %s", path);
}

static int sa_skip_dir(const char *s) {
    return !strcmp(s, ".git") || !strcmp(s, ".omo") || !strcmp(s, "__pycache__") ||
           !strcmp(s, "node_modules") || !strcmp(s, "_deps") || !strcmp(s, "CMakeFiles");
}

static void sa_tree(SettingsAudit *a, const char *root, int settings, int recurse) {
    char path[SA_PATH];
#ifdef _WIN32
    WIN32_FIND_DATAA entry;
    HANDLE h;
    snprintf(path, sizeof path, "%s/*", root);
    h = FindFirstFileA(path, &entry);
    if (h == INVALID_HANDLE_VALUE) {
        a->failed = 1; gw_test_fail("settings env audit: cannot enumerate %s", root); return;
    }
    do {
        if (entry.cFileName[0] == '.' || sa_skip_dir(entry.cFileName) ||
            (entry.dwFileAttributes & FILE_ATTRIBUTE_REPARSE_POINT)) continue;
        if (snprintf(path, sizeof path, "%s/%s", root, entry.cFileName) >= (int) sizeof path) {
            a->failed = 1; gw_test_fail("settings env audit: source path too long"); break;
        }
        if (entry.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) {
            if (recurse) sa_tree(a, path, settings, recurse);
        } else sa_file(a, path, settings);
    } while (FindNextFileA(h, &entry));
    FindClose(h);
#else
    DIR *dir = opendir(root);
    struct dirent *entry;
    if (!dir) { a->failed = 1; gw_test_fail("settings env audit: cannot enumerate %s", root); return; }
    while ((entry = readdir(dir)) != NULL) {
        struct stat info;
        if (entry->d_name[0] == '.' || sa_skip_dir(entry->d_name)) continue;
        if (snprintf(path, sizeof path, "%s/%s", root, entry->d_name) >= (int) sizeof path || lstat(path, &info)) {
            a->failed = 1; gw_test_fail("settings env audit: invalid source path"); break;
        }
        if (S_ISDIR(info.st_mode)) {
            if (recurse) sa_tree(a, path, settings, recurse);
        } else if (S_ISREG(info.st_mode)) sa_file(a, path, settings);
    }
    closedir(dir);
#endif
}

static int sa_collisions(SettingsAudit *a) {
    /* These aliases already select the same feature. Keep approval keyed by BOTH
     * spellings: an unrelated new key or variable is never covered by a prefix.
     * gw_Ucf_OfflineMode and rec_gdlp_wanted keep their explicit string parsers
     * ahead of Settings_Int (0.84/off and recording paths are not atoi values). */
    static const struct { const char *key, *env; } agreed[] = {
        { "atlas_pause", "MELEE_ATLAS_PAUSE" }, /* Atlas pause takeover */
        { "gdlp_record", "MELEE_GDLP_RECORD" }, /* recording flag / optional destination */
        { "input", "MELEE_INPUT" }, /* physical device selection */
        { "log", "MELEE_LOG" }, /* category list, not a log file path */
        { "pad_diag", "MELEE_PAD_DIAG" }, /* diagnostics level 0/1/2 */
        { "ucf", "MELEE_UCF" }, /* offline Controller Fix */
        { "unlock_all", "MELEE_UNLOCK_ALL" }, /* report unlocked content */
        { "widescreen", "MELEE_WIDESCREEN" }, /* wide game canvas */
    };
    int i, j, collisions = 0;
    for (i = 0; i < a->nk; ++i) {
        char env[6 + ST_KEY];
        size_t n;
        memcpy(env, "MELEE_", 6);
        for (n = 0; a->keys[i][n]; ++n) {
            char c = a->keys[i][n];
            env[n + 6] = c >= 'a' && c <= 'z' ? (char) (c - 'a' + 'A') : c;
        }
        env[n + 6] = 0;
        for (j = 0; j < a->nn; ++j) if (!strcmp(env, a->names[j]) ||
            ((a->prefixes[j] || a->names[j][strlen(a->names[j]) - 1] == '_') &&
             !strncmp(env, a->names[j], strlen(a->names[j])))) {
            size_t k;
            for (k = 0; k < sizeof agreed / sizeof agreed[0]; ++k)
                if (!strcmp(a->keys[i], agreed[k].key) && !strcmp(env, agreed[k].env)) break;
            if (k < sizeof agreed / sizeof agreed[0]) continue;
            ++collisions;
            gw_log("settings env audit: unapproved collision %s -> %s", a->keys[i], env);
            gw_test_fail("settings env audit: unapproved collision %s -> %s", a->keys[i], env);
        }
    }
    return collisions;
}

static int test_settings_env_collisions(void) {
    SettingsAudit *a = (SettingsAudit *) calloc(1, sizeof *a);
    const char *melee = getenv("GW_MELEE"), *workspace = getenv("GW_ROOT");
    char compiled[SA_PATH], parent[SA_PATH], path[SA_PATH];
    int rc;
    if (!a) { gw_test_fail("settings env audit: allocation failed"); return 1; }
    /* build.sh gives clang absolute source paths; retain a fallback for a direct
     * developer run, but fail visibly if neither source root can be located. */
    snprintf(compiled, sizeof compiled, "%s", __FILE__);
    { char *p; for (p = compiled; *p; ++p) if (*p == '\\') *p = '/';
      p = strstr(compiled, "/pc/"); if (p) *p = 0; else compiled[0] = 0; }
    if (!melee || !melee[0]) melee = compiled;
    if ((!workspace || !workspace[0]) && melee[0]) {
        snprintf(parent, sizeof parent, "%s", melee);
        for (;;) {
            FILE *f;
            char *slash;
            snprintf(path, sizeof path, "%s/tools/port/run.sh", parent);
            f = fopen(path, "r");
            if (f) { fclose(f); workspace = parent; break; }
            slash = strrchr(parent, '/');
            { char *back = strrchr(parent, '\\'); if (back && (!slash || back > slash)) slash = back; }
            if (!slash) break;
            *slash = 0;
        }
    }
    if (!melee[0] || !workspace || !workspace[0]) {
        free(a); gw_test_fail("settings env audit: GW_MELEE/GW_ROOT source roots required"); return 1;
    }
    snprintf(path, sizeof path, "%s/src", melee); sa_tree(a, path, 1, 1);
    snprintf(path, sizeof path, "%s/pc", melee); sa_tree(a, path, 1, 1);
    snprintf(path, sizeof path, "%s/tools", melee); sa_tree(a, path, 0, 1);
    snprintf(path, sizeof path, "%s/libs", melee); sa_tree(a, path, 0, 1);
    snprintf(path, sizeof path, "%s/extern", melee); sa_tree(a, path, 0, 1);
    snprintf(path, sizeof path, "%s/.github", melee); sa_tree(a, path, 0, 1);
    sa_tree(a, melee, 0, 0);
    snprintf(path, sizeof path, "%s/tools", workspace); sa_tree(a, path, 0, 1);
    snprintf(path, sizeof path, "%s/.github", workspace); sa_tree(a, path, 0, 1);
    sa_tree(a, workspace, 0, 0); /* launch scripts beside the tools */
    snprintf(path, sizeof path, "%s/_build", workspace); sa_tree(a, path, 0, 0); /* legacy launch scripts, never ax86m */
    if (!a->nk || !a->nn) { a->failed = 1; gw_test_fail("settings env audit: empty source inventory"); }
    rc = sa_collisions(a) != 0 || a->failed;
    gw_log("settings env audit: %d source files, %d keys, %d env names, %s", a->files, a->nk, a->nn, rc ? "FAIL" : "PASS");
    free(a);
    return rc;
}
