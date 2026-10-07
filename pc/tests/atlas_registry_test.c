/* atlas-registry: the entry registry and the mod.json "menus" reader. No game. */
#include "atlas_check.h"
#include "../platform/gw_ui_registry.h"

static AtEntry E(const char *mod, const char *id, const char *parent, const char *label, const char *after, int online)
{
    AtEntry e; memset(&e, 0, sizeof e);
    snprintf(e.mod, sizeof e.mod, "%s", mod); snprintf(e.id, sizeof e.id, "%s", id); snprintf(e.parent, sizeof e.parent, "%s", parent);
    snprintf(e.label, sizeof e.label, "%s", label); snprintf(e.after, sizeof e.after, "%s", after);
    e.action = AT_ENTRY_OPENS; e.online = online; e.visible = 1; e.builtin = mod[0] == '\0';
    return e;
}
static void order(void)
{
    AtRegistry r; const AtEntry *c[16]; int n;
    AtEntry b1 = E("", "training", "solo", "TRAINING", "", 0), b2 = E("", "lab", "solo", "LAB", "", 0);
    AtEntry m1 = E("envoy", "envoy", "solo", "ENVOY", "training", 0);
    at_reg_init(&r);
    CHECK(at_reg_add(&r, &b1) && at_reg_add(&r, &b2) && at_reg_add(&r, &m1));
    n = at_reg_children(&r, "solo", 0, c, 16);
    CHECK(n == 3);
    CHECK_STR(c[0]->id, "training"); CHECK_STR(c[1]->id, "envoy"); CHECK_STR(c[2]->id, "lab");   /* after = training */
}
static void order_ties_and_missing_after(void)
{
    AtRegistry r; const AtEntry *c[16]; int n;
    AtEntry b1 = E("", "training", "solo", "TRAINING", "", 0), b2 = E("", "lab", "solo", "LAB", "", 0);
    AtEntry zed = E("zed", "zed", "solo", "ZED", "training", 0), amy = E("amy", "amy", "solo", "AMY", "training", 0), lost = E("lost", "lost", "solo", "LOST", "nothing", 0);
    at_reg_init(&r);
    at_reg_add(&r, &b1); at_reg_add(&r, &b2); at_reg_add(&r, &zed); at_reg_add(&r, &lost); at_reg_add(&r, &amy);
    n = at_reg_children(&r, "solo", 0, c, 16);
    CHECK(n == 5);
    CHECK_STR(c[0]->id, "training"); CHECK_STR(c[1]->id, "amy"); CHECK_STR(c[2]->id, "zed"); CHECK_STR(c[3]->id, "lab"); CHECK_STR(c[4]->id, "lost");
}
static void caps(void)
{
    AtRegistry r; char id[32]; int i, added = 0; const AtEntry *c[32];
    at_reg_init(&r);
    for (i = 0; i < 9; i++) { AtEntry e; snprintf(id, sizeof id, "big.e%d", i); e = E("big", id, "solo", "X", "", 0); added += at_reg_add(&r, &e); }
    CHECK(added == AT_REG_PER_MOD_PARENT);                              /* 6 per mod per parent */
    CHECK(r.nlog >= 1);
    for (i = 0; i < 10; i++) { AtEntry e; char mod[8]; snprintf(id, sizeof id, "m%d.e", i); snprintf(mod, sizeof mod, "m%d", i); e = E(mod, id, "solo", "Y", "", 0); at_reg_add(&r, &e); }
    CHECK(at_reg_children(&r, "solo", 0, c, 32) == AT_REG_VISIBLE_PER_PARENT);   /* 12 visible: the rest are not listed */
    CHECK(at_reg_children(&r, "solo", 0, c, 5) == 5);
    CHECK(r.nlog <= 4);
}
static void unknown_parent(void)
{
    AtRegistry r; AtEntry e = E("envoy", "envoy", "nowhere", "ENVOY", "", 0), p = E("envoy", "envoy", "pause", "ENVOY", "", 0);
    at_reg_init(&r);
    CHECK(at_reg_add(&r, &e) == 0);
    CHECK(r.nlog == 1 && strstr(r.log[0], "nowhere") != NULL);
    CHECK(at_reg_add(&r, &p) == 0 && strstr(r.log[1], "later step") != NULL);
    { AtEntry s = E("envoy", "envoy", "settings.video", "V", "", 0); CHECK(at_reg_add(&r, &s) == 1); }
}
static void namespace_rule(void)
{
    AtRegistry r; AtEntry ok = E("envoy", "envoy.daily", "solo", "DAILY", "", 0), bad = E("envoy", "lab", "solo", "LAB", "", 0),
                          own = E("envoy", "envoy", "solo", "ENVOY", "", 0), pre = E("envoy", "envoyx", "solo", "X", "", 0);
    at_reg_init(&r);
    CHECK(at_reg_add(&r, &ok) == 1);
    CHECK(at_reg_add(&r, &own) == 1);                                   /* the mod's own id is allowed */
    CHECK(at_reg_add(&r, &bad) == 0);                                   /* another name is not */
    CHECK(at_reg_add(&r, &pre) == 0);                                   /* a longer name that merely starts with the id is not */
}
static void label_cap(void)
{
    AtRegistry r; AtEntry e = E("envoy", "envoy", "solo", "", "", 0);
    at_reg_init(&r);
    snprintf(e.label, sizeof e.label, "%s", "ABCDEFGHIJKLMNOPQRSTUV");      /* snprintf already cuts at 18 */
    CHECK(strlen(e.label) == AT_REG_LABEL_MAX);
    CHECK(at_reg_add(&r, &e) == 1);
}
static void disabled_mod_adds_nothing(void)
{
    AtRegistry r; const AtEntry *c[8]; AtEntry e = E("envoy", "envoy", "solo", "ENVOY", "", 0);
    at_reg_init(&r); at_reg_add(&r, &e);
    at_reg_drop_mod(&r, "envoy");
    CHECK(at_reg_children(&r, "solo", 0, c, 8) == 0);
    CHECK(r.n == 0);
    { AtEntry b = E("", "lab", "solo", "LAB", "", 0); at_reg_add(&r, &b); at_reg_drop_mod(&r, "envoy"); at_reg_drop_mod(&r, ""); CHECK(r.n == 1); }   /* built-ins are never dropped */
}
static void online_hidden(void)
{
    AtRegistry r; const AtEntry *c[8];
    AtEntry a = E("m", "m.safe", "versus", "SAFE", "", 1), b = E("m", "m.local", "versus", "LOCAL", "", 0), s = E("m", "m.solo", "solo", "SOLO", "", 0);
    at_reg_init(&r); at_reg_add(&r, &a); at_reg_add(&r, &b); at_reg_add(&r, &s);
    CHECK(at_reg_children(&r, "versus", 1, c, 8) == 1 && strcmp(c[0]->id, "m.safe") == 0);
    CHECK(at_reg_children(&r, "versus", 0, c, 8) == 2);
    CHECK(at_reg_children(&r, "solo", 1, c, 8) == 1);                    /* the rule is versus and online only */
    { AtEntry o = E("m", "m.on", "online", "ON", "", 0); at_reg_add(&r, &o); CHECK(at_reg_children(&r, "online", 1, c, 8) == 0); }
}
static void set_visibility(void)
{
    AtRegistry r; const AtEntry *c[8]; AtEntry e = E("envoy", "envoy", "solo", "ENVOY", "", 0);
    at_reg_init(&r); at_reg_add(&r, &e);
    CHECK(at_reg_set(&r, "envoy", "envoy", 0, NULL) == 1);
    CHECK(at_reg_children(&r, "solo", 0, c, 8) == 0);
    CHECK(at_reg_set(&r, "other", "envoy", 1, NULL) == 0);              /* only the owner may change it */
    CHECK(at_reg_set(&r, "", "envoy", 1, NULL) == 0);
    CHECK(at_reg_children(&r, "solo", 0, c, 8) == 0);
    CHECK(at_reg_set(&r, "envoy", "envoy", 1, "NEW") == 1);
    CHECK(at_reg_children(&r, "solo", 0, c, 8) == 1 && strcmp(c[0]->badge, "NEW") == 0);
    CHECK(at_reg_set(&r, "envoy", "envoy", -1, "") == 1 && at_reg_children(&r, "solo", 0, c, 8) == 1);   /* visible unchanged */
    CHECK(at_reg_set(&r, "envoy", "nothere", 1, NULL) == 0);
}
static void duplicate_id(void)
{
    AtRegistry r; AtEntry e = E("envoy", "envoy", "solo", "ENVOY", "", 0);
    at_reg_init(&r);
    CHECK(at_reg_add(&r, &e) == 1);
    CHECK(at_reg_add(&r, &e) == 0);
}
int main(void)
{
    order(); order_ties_and_missing_after(); caps(); unknown_parent(); namespace_rule(); label_cap(); disabled_mod_adds_nothing(); online_hidden(); set_visibility(); duplicate_id();
    ATLAS_DONE("atlas-registry");
}
