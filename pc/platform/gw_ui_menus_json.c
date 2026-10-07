/* gw_ui_menus_json.c - the mod.json "menus" reader. gw_mods.c's own reader is strings-only and skips an array of objects, so this
 * is a small dedicated one: it finds the top-level key "menus", reads each element as a flat object (strings and booleans; a nested
 * value inside an element is skipped by a balanced skipper) and leaves every other top-level value alone. Pure C. */
#include "gw_ui_menus_json.h"

#include <stdio.h>
#include <string.h>

static const char *ws(const char *p)
{
    while (*p == ' ' || *p == '\t' || *p == '\r' || *p == '\n') p++;
    return p;
}

/* one JSON string at p (p[0] == '"') into out (cut to cap); returns the position after it, NULL when it is unterminated */
static const char *str_at(const char *p, char *out, int cap)
{
    int n = 0;
    p++;
    while (*p != '\0' && *p != '"') {
        char c = *p;
        if (c == '\\' && p[1] != '\0') {
            p++;
            c = *p == 'n' ? '\n' : *p == 't' ? '\t' : *p == 'r' ? '\r' : *p;      /* \", \\ and \/ are themselves; \uXXXX keeps its u */
        }
        if (out != NULL && n + 1 < cap) out[n++] = c;
        p++;
    }
    if (*p != '"') return NULL;
    if (out != NULL && cap > 0) out[n] = '\0';
    return p + 1;
}

/* skips one value of any kind (strings, numbers, literals, balanced arrays and objects); NULL on an unterminated string or bracket */
static const char *skip_value(const char *p)
{
    int depth = 0;
    p = ws(p);
    for (;;) {
        char c = *p;
        if (c == '\0') return depth == 0 ? p : NULL;
        if (c == '"') {
            p = str_at(p, NULL, 0);
            if (p == NULL) return NULL;
            if (depth == 0) return p;
            continue;
        }
        if (c == '{' || c == '[') depth++;
        else if (c == '}' || c == ']') {
            if (depth == 0) return p;
            if (--depth == 0) return p + 1;
        } else if (depth == 0 && (c == ',' || c == ' ' || c == '\t' || c == '\r' || c == '\n')) {
            return p;
        }
        p++;
    }
}

static void note(char *err, int errcap, const char *what, const char *id)
{
    if (err != NULL && errcap > 0 && err[0] == '\0') snprintf(err, (size_t) errcap, "menus: %s%s%s", what, id[0] ? ": " : "", id);
}

/* A flat object into e. Returns the position after it, NULL on a syntax error. *keep is set when the element is usable. */
static const char *read_entry(const char *p, const char *mod, AtEntry *e, int *keep, char *err, int errcap)
{
    char action[16] = "", online[8] = "";
    int has_online = 0;
    memset(e, 0, sizeof *e);
    snprintf(e->mod, sizeof e->mod, "%s", mod);
    e->visible = 1;
    *keep = 0;
    p = ws(p + 1);
    while (*p != '\0' && *p != '}') {
        char key[24], val[AT_TEXT];
        p = ws(p);
        if (*p != '"') return NULL;
        p = str_at(p, key, sizeof key);
        if (p == NULL) return NULL;
        p = ws(p);
        if (*p != ':') return NULL;
        p = ws(p + 1);
        val[0] = '\0';
        if (*p == '"') {
            p = str_at(p, val, sizeof val);
            if (p == NULL) return NULL;
        } else if (strncmp(p, "true", 4) == 0 || strncmp(p, "false", 5) == 0) {
            snprintf(val, sizeof val, "%s", *p == 't' ? "true" : "false");
            p = skip_value(p);
        } else {
            p = skip_value(p);                                           /* a number, null or a nested value: not a field */
            if (p == NULL) return NULL;
            key[0] = '\0';
        }
        if (strcmp(key, "id") == 0) snprintf(e->id, sizeof e->id, "%s", val);
        else if (strcmp(key, "parent") == 0) snprintf(e->parent, sizeof e->parent, "%s", val);
        else if (strcmp(key, "label") == 0) snprintf(e->label, sizeof e->label, "%s", val);
        else if (strcmp(key, "blurb") == 0) snprintf(e->blurb, sizeof e->blurb, "%s", val);
        else if (strcmp(key, "icon") == 0) snprintf(e->icon, sizeof e->icon, "%s", val);
        else if (strcmp(key, "after") == 0) snprintf(e->after, sizeof e->after, "%s", val);
        else if (strcmp(key, "opens") == 0) snprintf(e->opens, sizeof e->opens, "%s", val);
        else if (strcmp(key, "action") == 0) snprintf(action, sizeof action, "%s", val);
        else if (strcmp(key, "online") == 0) { snprintf(online, sizeof online, "%s", val); has_online = 1; }
        p = ws(p);
        if (*p == ',') p = ws(p + 1);
    }
    if (*p != '}') return NULL;
    p++;
    if (has_online) e->online = strcmp(online, "true") == 0;
    if (e->id[0] == '\0' || e->parent[0] == '\0' || e->label[0] == '\0') { note(err, errcap, "an entry needs id, parent and label", e->id); return p; }
    if (strcmp(action, "script") == 0) e->action = AT_ENTRY_SCRIPT;
    else if (e->opens[0] != '\0') e->action = AT_ENTRY_OPENS;
    else { note(err, errcap, "an entry needs opens or action \"script\"", e->id); return p; }
    *keep = 1;
    return p;
}

int at_menus_parse(const char *json, const char *mod, AtEntry *out, int cap, char *err, int errcap)
{
    const char *p = ws(json);
    int n = 0;
    if (err != NULL && errcap > 0) err[0] = '\0';
    if ((unsigned char) p[0] == 0xEF && (unsigned char) p[1] == 0xBB && (unsigned char) p[2] == 0xBF) p = ws(p + 3);
    if (*p != '{') { note(err, errcap, "not a JSON object", ""); return -1; }
    p = ws(p + 1);
    while (*p != '\0' && *p != '}') {
        char key[24];
        p = ws(p);
        if (*p != '"') goto syntax;
        p = str_at(p, key, sizeof key);
        if (p == NULL) goto syntax;
        p = ws(p);
        if (*p != ':') goto syntax;
        p = ws(p + 1);
        if (strcmp(key, "menus") == 0 && *p == '[') {
            p = ws(p + 1);
            while (*p != '\0' && *p != ']') {
                if (*p == '{') {
                    AtEntry e;
                    int keep;
                    p = read_entry(p, mod, &e, &keep, err, errcap);
                    if (p == NULL) goto syntax;
                    if (keep && n < cap) out[n++] = e;
                    else if (keep) note(err, errcap, "too many entries", e.id);
                } else {
                    p = skip_value(p);                                   /* not an object: skipped */
                    if (p == NULL) goto syntax;
                    note(err, errcap, "an entry is an object", "");
                }
                p = ws(p);
                if (*p == ',') p = ws(p + 1);
            }
            if (*p != ']') goto syntax;
            p++;
        } else {
            p = skip_value(p);
            if (p == NULL) goto syntax;
        }
        p = ws(p);
        if (*p == ',') p = ws(p + 1);
    }
    if (*p != '}') goto syntax;
    return n;
syntax:
    note(err, errcap, "syntax error", "");
    return -1;
}
