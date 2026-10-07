#include "gw_ui_registry.h"

#include <stdio.h>
#include <string.h>

static void rlog(AtRegistry *r, const char *fmt, const char *a, const char *b)
{
    if (r->nlog < 4) snprintf(r->log[r->nlog++], sizeof r->log[0], fmt, a, b);
}

void at_reg_init(AtRegistry *r) { memset(r, 0, sizeof *r); }

int at_reg_is_parent(const char *id)
{
    static const char *const P[] = { "main", "solo", "versus", "online", "mods", "settings", "more", "lab.pause", "mods.self" };
    size_t i;
    for (i = 0; i < sizeof P / sizeof P[0]; i++) if (strcmp(id, P[i]) == 0) return 1;
    if (strncmp(id, "mods.", 5) == 0 && id[5] != '\0') return 1;      /* mods.<mod id>: that mod's own settings entry, in its detail screen */
    return strncmp(id, "settings.", 9) == 0 && id[9] != '\0';
}

/* Who may read a parent's entries and activate them through gd.ui.entries and gd.ui.activate: the script of the mod that draws it. "" = nobody
 * (the native menus' own parents are the adapter's). lab.pause is the LAB's pause menu; mods.<id> is the detail screen of mod <id>. */
const char *at_reg_owner(const char *parent)
{
    if (strcmp(parent, "lab.pause") == 0) return "geno-lab";
    if (strncmp(parent, "mods.", 5) == 0 && parent[5] != '\0' && strcmp(parent, "mods.self") != 0) return parent + 5;
    return "";
}

/* The parents a menu draws today: the main menu, Solo, Versus and the Settings list. The others (online, mods, more, settings.<page>) are
 * valid names for later steps, but nothing renders their children yet, so an entry under one is refused at validation with a log line
 * rather than accepted and never shown. */
int at_reg_parent_rendered(const char *id)
{
    return strcmp(id, "main") == 0 || strcmp(id, "solo") == 0 || strcmp(id, "versus") == 0 || strcmp(id, "settings") == 0 || strcmp(id, "lab.pause") == 0 ||
           (strncmp(id, "mods.", 5) == 0 && id[5] != '\0');
}

int at_reg_add(AtRegistry *r, const AtEntry *in)
{
    int i, same = 0;
    size_t ml = strlen(in->mod);
    AtEntry copy = *in;
    const AtEntry *e = &copy;
    if (strcmp(in->parent, "pause") == 0) { rlog(r, "entry \"%s\": parent \"%s\" is not available until a later step", in->id, in->parent); return 0; }
    if (ml > 0 && strcmp(in->parent, "mods.self") == 0) snprintf(copy.parent, sizeof copy.parent, "mods.%s", in->mod);    /* a mod's own settings entry: filed under its own id */
    if (!at_reg_is_parent(e->parent)) { rlog(r, "entry \"%s\": unknown parent \"%s\"", e->id, e->parent); return 0; }
    if (ml > 0 && strncmp(e->parent, "mods.", 5) == 0 && strcmp(e->parent + 5, e->mod) != 0) { rlog(r, "entry \"%s\": a mod's settings entry goes under mods.self, not under \"%s\"", e->id, e->parent); return 0; }
    if (ml == 0 && strcmp(e->parent, "mods.self") == 0) { rlog(r, "entry \"%s\": mods.self belongs to a mod%s", e->id, ""); return 0; }
    if (ml > 0 && !at_reg_parent_rendered(e->parent)) { rlog(r, "entry \"%s\": no menu shows parent \"%s\" yet (main, solo, versus, settings, mods.self and lab.pause only)", e->id, e->parent); return 0; }
    if (e->id[0] == '\0') { rlog(r, "entry in parent \"%s\"%s: it has no id", e->parent, ""); return 0; }
    if (ml > 0 && !(strcmp(e->id, e->mod) == 0 || (strncmp(e->id, e->mod, ml) == 0 && e->id[ml] == '.' && e->id[ml + 1] != '\0'))) {
        rlog(r, "entry \"%s\": the id of a mod entry is \"%s\" or starts with \"%s.\"", e->id, e->mod); return 0;
    }
    for (i = 0; i < r->n; i++) {
        if (strcmp(r->e[i].id, e->id) == 0 && strcmp(r->e[i].parent, e->parent) == 0) { rlog(r, "entry \"%s\": the id is already registered%s", e->id, ""); return 0; }
        if (ml > 0 && strcmp(r->e[i].mod, e->mod) == 0 && strcmp(r->e[i].parent, e->parent) == 0) same++;
    }
    if (ml > 0 && same >= AT_REG_PER_MOD_PARENT) { rlog(r, "entry \"%s\": a mod may add 6 entries under one parent (\"%s\")", e->id, e->parent); return 0; }
    if (r->n >= AT_REG_MAX) { rlog(r, "entry \"%s\": the registry is full%s", e->id, ""); return 0; }
    r->e[r->n] = copy;
    r->e[r->n].builtin = ml == 0;
    r->n++;
    return 1;
}

int at_reg_set(AtRegistry *r, const char *mod, const char *id, int visible, const char *badge)
{
    int i;
    for (i = 0; i < r->n; i++) {
        if (strcmp(r->e[i].id, id) != 0) continue;
        if (mod == NULL || mod[0] == '\0' || strcmp(r->e[i].mod, mod) != 0) return 0;      /* only the owner may change it */
        if (visible >= 0) r->e[i].visible = visible != 0;
        if (badge != NULL) snprintf(r->e[i].badge, sizeof r->e[i].badge, "%s", badge);
        return 1;
    }
    return 0;
}

void at_reg_drop_mod(AtRegistry *r, const char *mod)
{
    int i, k = 0;
    if (mod == NULL || mod[0] == '\0') return;
    for (i = 0; i < r->n; i++) if (strcmp(r->e[i].mod, mod) != 0) r->e[k++] = r->e[i];
    r->n = k;
}

int at_reg_children(const AtRegistry *r, const char *parent, int netplay, const AtEntry **out, int cap)
{
    const AtEntry *list[AT_REG_MAX];
    int n = 0, i, j, m, online_parent = strcmp(parent, "versus") == 0 || strcmp(parent, "online") == 0, cnt = 0;
    int mods[AT_REG_MAX], nm = 0;
    for (i = 0; i < r->n; i++) if (strcmp(r->e[i].parent, parent) == 0 && r->e[i].builtin) list[n++] = &r->e[i];
    for (i = 0; i < r->n; i++) if (strcmp(r->e[i].parent, parent) == 0 && !r->e[i].builtin) mods[nm++] = i;
    for (i = 1; i < nm; i++) {                                    /* insertion sort by id: ties are ordered by id */
        int key = mods[i];
        for (j = i - 1; j >= 0 && strcmp(r->e[mods[j]].id, r->e[key].id) > 0; j--) mods[j + 1] = mods[j];
        mods[j + 1] = key;
    }
    for (m = 0; m < nm; m++) {
        const AtEntry *e = &r->e[mods[m]];
        int at = n;
        if (e->after[0] != '\0') {
            for (j = 0; j < n; j++) if (strcmp(list[j]->id, e->after) == 0) {
                at = j + 1;
                while (at < n && !list[at]->builtin && strcmp(list[at]->after, e->after) == 0) at++;   /* behind earlier mod entries on the same sibling */
                break;
            }
        }
        for (j = n; j > at; j--) list[j] = list[j - 1];
        list[at] = e; n++;
    }
    for (i = 0; i < n && cnt < cap && cnt < AT_REG_VISIBLE_PER_PARENT; i++) {
        if (!list[i]->visible) continue;
        if (netplay && online_parent && !list[i]->online) continue;
        out[cnt++] = list[i];
    }
    return cnt;
}
