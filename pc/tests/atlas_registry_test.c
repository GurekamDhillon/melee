#define _CRT_SECURE_NO_WARNINGS
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
    { AtEntry s = E("envoy", "envoy", "settings.video", "V", "", 0); CHECK(at_reg_add(&r, &s) == 0 && strstr(r.log[2], "no menu shows parent") != NULL); }   /* a valid name nothing renders yet: refused, logged */
    { AtEntry s = E("envoy", "envoy", "mods", "M", "", 0), o = E("envoy", "envoy", "online", "O", "", 0); CHECK(at_reg_add(&r, &s) == 0 && at_reg_add(&r, &o) == 0); }
    { AtEntry s = E("envoy", "envoy", "settings", "S", "", 0); CHECK(at_reg_add(&r, &s) == 1); }
    CHECK(at_reg_parent_rendered("solo") && at_reg_parent_rendered("main") && at_reg_parent_rendered("versus") && !at_reg_parent_rendered("online"));
}
/* Atlas step 7: lab.pause is the LAB's pause menu (offline, geno-lab owns it); mods.self is a mod's own settings entry, filed under mods.<its id> */
static void step7_parents(void)
{
    AtRegistry r; const AtEntry *c[32]; int i, added = 0;
    AtEntry lab = E("tools", "tools.dummy", "lab.pause", "Dummy tool", "", 0), self = E("envoy", "envoy.settings", "mods.self", "Settings", "", 0);
    AtEntry other = E("envoy", "envoy.other", "mods.sora", "X", "", 0), builtin = E("", "mods.self.b", "mods.self", "B", "", 0);
    AtEntry pause = E("tools", "tools.p", "pause", "P", "", 0);
    at_reg_init(&r);
    CHECK(at_reg_add(&r, &lab) == 1 && at_reg_children(&r, "lab.pause", 0, c, 32) == 1 && strcmp(c[0]->id, "tools.dummy") == 0);
    CHECK(at_reg_children(&r, "lab.pause", 1, c, 32) == 1);               /* the LAB is offline only, so the netplay rule has nothing to hide here: the LAB never opens online */
    CHECK(at_reg_add(&r, &self) == 1);                                    /* mods.self is rewritten to mods.envoy */
    CHECK(at_reg_children(&r, "mods.envoy", 0, c, 32) == 1 && strcmp(c[0]->id, "envoy.settings") == 0 && strcmp(c[0]->parent, "mods.envoy") == 0);
    CHECK(at_reg_children(&r, "mods.self", 0, c, 32) == 0);
    CHECK(at_reg_add(&r, &other) == 0 && strstr(r.log[r.nlog - 1], "mods.self") != NULL);   /* another mod's detail screen is not yours to fill */
    CHECK(at_reg_add(&r, &builtin) == 0);                                 /* nor may a built-in entry use it */
    CHECK(at_reg_add(&r, &pause) == 0);                                   /* the retail pause is still a later step */
    CHECK_STR(at_reg_owner("lab.pause"), "geno-lab");
    CHECK_STR(at_reg_owner("mods.envoy"), "envoy");
    CHECK_STR(at_reg_owner("mods.self"), "");
    CHECK_STR(at_reg_owner("solo"), "");
    CHECK_STR(at_reg_owner("mods"), "");
    CHECK(at_reg_is_parent("lab.pause") && at_reg_is_parent("mods.self") && at_reg_is_parent("mods.envoy") && !at_reg_is_parent("mods.") && !at_reg_is_parent("lab"));
    CHECK(at_reg_parent_rendered("lab.pause") && at_reg_parent_rendered("mods.envoy") && !at_reg_parent_rendered("mods") && !at_reg_parent_rendered("pause"));
    /* the caps count here as everywhere: 6 per mod per parent, 12 visible */
    at_reg_init(&r);
    for (i = 0; i < 9; i++) { char id[24]; AtEntry e; snprintf(id, sizeof id, "tools.t%d", i); e = E("tools", id, "lab.pause", "T", "", 0); added += at_reg_add(&r, &e); }
    CHECK(added == AT_REG_PER_MOD_PARENT);
    for (i = 0; i < 9; i++) { char id[24], mod[16]; AtEntry e; snprintf(mod, sizeof mod, "m%d", i); snprintf(id, sizeof id, "m%d.x", i); e = E(mod, id, "lab.pause", "T", "", 0); at_reg_add(&r, &e); }
    CHECK(at_reg_children(&r, "lab.pause", 0, c, 32) == AT_REG_VISIBLE_PER_PARENT);
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
    /* Atlas proof D4: the refusal's log line has three conversions; it once had two arguments and crashed the main menu */
    {
        AtRegistry q; AtEntry row = E("lab-tool", "lab-tool-row", "lab.pause", "ROW", "", 0);
        at_reg_init(&q);
        CHECK(at_reg_add(&q, &row) == 0 && q.nlog == 1);
        CHECK(strcmp(q.log[0], "entry \"lab-tool-row\": the id of a mod entry is \"lab-tool\" or starts with \"lab-tool.\"") == 0);
    }
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
static void same_id_in_two_parents(void)             /* built-in rows share names across menus (ONLINE under main and under settings) */
{
    AtRegistry r; const AtEntry *c[8];
    AtEntry a = E("", "online", "main", "ONLINE", "", 0), b = E("", "online", "settings", "ONLINE", "", 0);
    at_reg_init(&r);
    CHECK(at_reg_add(&r, &a) == 1 && at_reg_add(&r, &b) == 1 && at_reg_add(&r, &a) == 0);
    CHECK(at_reg_children(&r, "main", 0, c, 8) == 1 && at_reg_children(&r, "settings", 0, c, 8) == 1);
}
static void duplicate_id(void)
{
    AtRegistry r; AtEntry e = E("envoy", "envoy", "solo", "ENVOY", "", 0);
    at_reg_init(&r);
    CHECK(at_reg_add(&r, &e) == 1);
    CHECK(at_reg_add(&r, &e) == 0);
}
#include "../platform/gw_ui_menus_json.h"
static void menus_parse_ok(void)
{
    AtEntry e[4]; char err[96];
    const char *j = "{ \"id\": \"envoy\", \"kind\": \"script\", \"menus\": [ { \"id\": \"envoy\", \"parent\": \"solo\", \"label\": \"ENVOY\","
                    " \"blurb\": \"Explore, fight and evolve your build.\", \"after\": \"training\", \"action\": \"script\", \"online\": false } ],"
                    " \"gameplay\": true }";
    CHECK(at_menus_parse(j, "envoy", e, 4, err, sizeof err) == 1);
    CHECK_STR(e[0].id, "envoy"); CHECK_STR(e[0].parent, "solo"); CHECK_STR(e[0].mod, "envoy"); CHECK_STR(e[0].after, "training");
    CHECK_STR(e[0].blurb, "Explore, fight and evolve your build.");
    CHECK(e[0].action == AT_ENTRY_SCRIPT); CHECK(e[0].online == 0); CHECK(e[0].visible == 1);
    j = "{ \"menus\": [ { \"id\": \"m.a\", \"parent\": \"versus\", \"label\": \"A\", \"action\": \"script\", \"online\": true },"
        " { \"id\": \"m.b\", \"parent\": \"versus\", \"label\": \"B\", \"action\": \"script\", \"online\": \"true\" } ] }";
    CHECK(at_menus_parse(j, "m", e, 4, err, sizeof err) == 2 && e[0].online == 1 && e[1].online == 1);   /* a boolean or the string */
    CHECK(at_menus_parse(j, "m", e, 1, err, sizeof err) == 1);                                            /* the cap */
}
static void menus_parse_rejects(void)
{
    AtEntry e[4]; char err[96];
    CHECK(at_menus_parse("{ \"menus\": [ { \"id\": \"x\", \"parent\": \"solo\" } ] }", "m", e, 4, err, sizeof err) == 0);    /* no label: skipped */
    CHECK(err[0] != '\0');
    CHECK(at_menus_parse("{ \"menus\": [ { \"id\": \"m.a\", \"parent\": \"solo\", \"label\": \"A\", \"opens\": \"m.s\" } ] }", "m", e, 4, err, sizeof err) == 1);
    CHECK(e[0].action == AT_ENTRY_OPENS);
    CHECK(at_menus_parse("{ \"menus\": [ { \"id\": \"m.a\", \"parent\": \"solo\", \"label\": \"A\" } ] }", "m", e, 4, err, sizeof err) == 0);   /* neither opens nor action */
    CHECK(at_menus_parse("{ \"menus\": [ { \"id\": ", "m", e, 4, err, sizeof err) == -1);
    CHECK(at_menus_parse("{ \"name\": \"no menus\" }", "m", e, 4, err, sizeof err) == 0);
    CHECK(at_menus_parse("{ \"menus\": [ {\"id\":\"m.a\",\"parent\":\"solo\",\"label\":\"A\",\"action\":\"script\",\"nested\":{\"x\":[1,2]}} ] }", "m", e, 4, err, sizeof err) == 1);
    CHECK(at_menus_parse("[1,2]", "m", e, 4, err, sizeof err) == -1);
    CHECK(at_menus_parse("{ \"menus\": [ 7, \"x\", { \"id\":\"m.a\",\"parent\":\"solo\",\"label\":\"A\",\"action\":\"script\" } ] }", "m", e, 4, err, sizeof err) == 1);   /* non-objects are skipped */
    CHECK(at_menus_parse("{ \"menus\": [ { \"id\":\"m.a\",\"parent\":\"solo\",\"label\":\"A very long label that goes past the cap\",\"action\":\"script\" } ] }", "m", e, 4, err, sizeof err) == 1
          && strlen(e[0].label) == AT_REG_LABEL_MAX);
    CHECK(at_menus_parse("\xEF\xBB\xBF{ \"menus\": [] }", "m", e, 4, err, sizeof err) == 0);                  /* a BOM */
}
/* the registry takes what the reader gives, and refuses by its own rules */
static void menus_into_registry(void)
{
    AtEntry e[8]; char err[96]; AtRegistry r; const AtEntry *c[8];
    int n = at_menus_parse("{ \"menus\": [ { \"id\":\"m.a\",\"parent\":\"nowhere\",\"label\":\"A\",\"action\":\"script\" },"
                           " { \"id\":\"m.b\",\"parent\":\"solo\",\"label\":\"B\",\"action\":\"script\" } ] }", "m", e, 8, err, sizeof err), i, added = 0;
    at_reg_init(&r);
    for (i = 0; i < n; i++) added += at_reg_add(&r, &e[i]);
    CHECK(n == 2 && added == 1 && at_reg_children(&r, "solo", 0, c, 8) == 1);
}
#ifndef ENVOY_MENUS_EXPECTED
#define ENVOY_MENUS_EXPECTED 1
#endif
static void envoy_manifest(void)
{
    FILE *f = fopen("pc/scripts/examples/envoy/mod.json", "rb"); char buf[4096]; size_t n; AtEntry e[4]; char err[96];
    CHECK(f != NULL);
    if (f == NULL) return;
    n = fread(buf, 1, sizeof buf - 1, f); fclose(f); buf[n] = 0;
    CHECK(at_menus_parse(buf, "envoy", e, 4, err, sizeof err) == ENVOY_MENUS_EXPECTED);
}
static void roguelite_manifest(void)
{
    FILE *f = fopen("pc/scripts/examples/roguelite/mod.json", "rb"); char buf[4096]; size_t n; AtEntry e[4]; char err[96];
    CHECK(f != NULL);
    if (f == NULL) return;
    n = fread(buf, 1, sizeof buf - 1, f); fclose(f); buf[n] = 0;
    CHECK(at_menus_parse(buf, "roguelite", e, 4, err, sizeof err) == 1 && strcmp(e[0].parent, "solo") == 0 && e[0].action == AT_ENTRY_SCRIPT && strstr(buf, "\"api_version\": 2") != NULL);
    {   AtRegistry r; at_reg_init(&r); CHECK(at_reg_add(&r, &e[0]) == 1); }
}
int main(void)
{
    order(); order_ties_and_missing_after(); caps(); unknown_parent(); step7_parents(); namespace_rule(); label_cap(); disabled_mod_adds_nothing(); online_hidden(); set_visibility(); duplicate_id(); same_id_in_two_parents();
    menus_parse_ok(); menus_parse_rejects(); menus_into_registry(); envoy_manifest(); roguelite_manifest();
    ATLAS_DONE("atlas-registry");
}
