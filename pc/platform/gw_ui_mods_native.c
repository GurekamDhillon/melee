#include "gw_ui_mods_native.h"
#include "gw_mods.h"
#include <stdio.h>
#include <string.h>

/* Every accessor reads the mods API at the moment it is asked: the screen holds no copy, so a toggle (which can move other mods) is seen by the next row. */

static const AtModsHooks *hooks_of(void *u) { return (const AtModsHooks *) u; }

static int r_count(void *u) { (void) u; return gw_Mods_Count(); }
#define R_STR(name, call) static const char *r_##name(void *u, int i) { (void) u; return call(i); }
R_STR(id, gw_Mods_Id) R_STR(name, gw_Mods_Name) R_STR(version, gw_Mods_Version) R_STR(kind, gw_Mods_Kind) R_STR(pack, gw_Mods_Pack)
R_STR(desc, gw_Mods_Description) R_STR(requires, gw_Mods_Requires) R_STR(conflicts, gw_Mods_Conflicts) R_STR(status_text, gw_Mods_StatusText)

static int r_status(void *u, int i)
{
    (void) u;
    switch (gw_Mods_Status(i)) {
    case GW_MOD_ACTIVE: return AT_MOD_ACTIVE;
    case GW_MOD_MISSING_DEP: return AT_MOD_MISSING_DEP;
    case GW_MOD_CONFLICT: return AT_MOD_CONFLICT;
    default: return AT_MOD_OFF;
    }
}
static int r_enabled(void *u, int i) { (void) u; return gw_Mods_IsEnabled(i); }
static int r_active(void *u, int i) { (void) u; return gw_Mods_IsActive(i); }
static int r_set(void *u, int i, int on) { (void) u; return gw_Mods_SetEnabled(i, on); }
static int r_save(void *u) { (void) u; return gw_Mods_Save(); }
static int r_restart(void *u) { (void) u; return gw_Mods_RestartNeeded(); }
static int r_locked(void *u) { const AtModsHooks *h = hooks_of(u); return h != NULL && h->locked != NULL && h->locked() != 0; }

/* what the mod adds to the menus: its mod.json "menus" entries (an active mod only; an inactive one adds nothing). Its own settings entry (parent mods.self)
 * is not an addition to a menu. */
static int r_adds(void *u, int i, char out[][AT_STR], int cap)
{
    int n = gw_Mods_MenuCount(i), k, w = 0;
    (void) u;
    for (k = 0; k < n && w < cap; k++) {
        const char *parent = gw_Mods_MenuField(i, k, "parent");
        if (strncmp(parent, "mods.", 5) == 0) continue;
        at_mods_add_line(parent, gw_Mods_MenuField(i, k, "label"), out[w++], AT_STR);
    }
    return w;
}

/* the mod's own settings entry: a "menus" entry under mods.self whose screen the mod registered. This screen only names it; the registry opens it. */
static int r_settings(void *u, int i, char *sid, int cap, char *label, int lcap)
{
    const AtModsHooks *h = hooks_of(u);
    int n = gw_Mods_MenuCount(i), k;
    for (k = 0; k < n; k++) {
        const char *parent = gw_Mods_MenuField(i, k, "parent"), *id = gw_Mods_MenuField(i, k, "id");
        if (strncmp(parent, "mods.", 5) != 0 || id[0] == '\0') continue;
        if (h != NULL && h->entry_ok != NULL && !h->entry_ok(id)) continue;
        snprintf(sid, (size_t) cap, "%s", id);
        snprintf(label, (size_t) lcap, "%s settings", gw_Mods_Name(i));
        return 1;
    }
    return 0;
}

void at_mods_native_src(AtModsSrc *out, const AtModsHooks *hooks)
{
    memset(out, 0, sizeof *out);
    out->user = (void *) hooks;
    out->count = r_count; out->id = r_id; out->name = r_name; out->version = r_version; out->kind = r_kind; out->pack = r_pack; out->desc = r_desc;
    out->requires = r_requires; out->conflicts = r_conflicts; out->status = r_status; out->status_text = r_status_text; out->enabled = r_enabled;
    out->active = r_active; out->set_enabled = r_set; out->save = r_save; out->restart_needed = r_restart; out->adds = r_adds; out->settings = r_settings;
    out->locked = r_locked;
}
