#include "atlas_check.h"
#include "../platform/gw_ui_retail.h"

static void names_and_parse(void)
{
    unsigned m = 0; char err[96];
    CHECK_STR(at_retail_name(AT_RE_HUD_DAMAGE), "hud.damage");
    CHECK_STR(at_retail_name(AT_RE_PAUSE_PANEL), "pause.panel");
    CHECK(at_retail_name(AT_RE_COUNT) == NULL && at_retail_name(-1) == NULL);
    CHECK(at_retail_parse("hud.damage, hud.stock", &m, err, sizeof err) && m == ((1u << AT_RE_HUD_DAMAGE) | (1u << AT_RE_HUD_STOCK)));
    CHECK(at_retail_parse("", &m, err, sizeof err) && m == 0);
    CHECK(at_retail_parse(NULL, &m, err, sizeof err) && m == 0);
    CHECK(at_retail_parse("all", &m, err, sizeof err) && m == (1u << AT_RE_COUNT) - 1);
    CHECK(!at_retail_parse("hud.damage,hud.bogus", &m, err, sizeof err) && strstr(err, "hud.bogus") != NULL);
    CHECK((at_retail_script_allowed() & (1u << AT_RE_HUD_TIMER)) == 0);   /* a mod never hides the match clock */
    CHECK((at_retail_script_allowed() & (1u << AT_RE_HUD_DAMAGE)) != 0);
}
static void sources_and_online(void)
{
    AtRetail r; memset(&r, 0, sizeof r);
    CHECK(at_retail_effective(&r, 0) == 0);                                 /* empty by default */
    at_retail_set(&r, AT_RS_ENV, 1u << AT_RE_HUD_DAMAGE);
    at_retail_set(&r, AT_RS_SCRIPT, 1u << AT_RE_HUD_STOCK); r.script_owner = 3;
    CHECK(at_retail_hidden(&r, AT_RE_HUD_DAMAGE, 0) && at_retail_hidden(&r, AT_RE_HUD_STOCK, 0));
    CHECK(!at_retail_hidden(&r, AT_RE_HUD_TIMER, 0));
    CHECK(at_retail_effective(&r, 1) == 0 && !at_retail_hidden(&r, AT_RE_HUD_DAMAGE, 1));   /* online: nothing hidden */
    at_retail_release_script(&r, 2);                                        /* not the owner: no change */
    CHECK(at_retail_hidden(&r, AT_RE_HUD_STOCK, 0));
    at_retail_release_script(&r, 3);
    CHECK(!at_retail_hidden(&r, AT_RE_HUD_STOCK, 0) && r.script_owner == 0 && at_retail_hidden(&r, AT_RE_HUD_DAMAGE, 0));
    CHECK(!at_retail_hidden(&r, -1, 0) && !at_retail_hidden(&r, AT_RE_COUNT, 0));             /* out of range: never hidden */
    at_retail_set(&r, AT_RS_POLICY, 1u << AT_RE_PAUSE_PANEL);                /* the scene policy source */
    at_retail_set(&r, AT_RS_CONSOLE, 0xFFFFFFFFu);                           /* bits past the last element are dropped */
    CHECK(r.src[AT_RS_CONSOLE] == (1u << AT_RE_COUNT) - 1);
    at_retail_set(&r, 99, 1u);                                               /* an unknown source changes nothing */
    CHECK(at_retail_effective(&r, 0) == (1u << AT_RE_COUNT) - 1);
}
static void pause_machine(void)
{
    AtPause p; memset(&p, 0, sizeof p); p.pauser = -1;
    CHECK(!at_pause_request_unpause(&p, 0));                               /* not paused: refused */
    at_pause_on(&p, 2, 1, 0);
    CHECK(p.paused && p.pauser == 2 && p.takeover);
    CHECK(at_pause_take_unpause(&p) == -1);                                /* nothing requested */
    CHECK(at_pause_request_unpause(&p, 0) && !at_pause_request_unpause(&p, 0));   /* one pending at most */
    CHECK(at_pause_take_unpause(&p) == 2 && at_pause_take_unpause(&p) == -1);     /* one-shot */
    at_pause_off(&p);
    CHECK(!p.paused && p.pauser == -1 && !p.takeover && !p.unpause_req);
    at_pause_on(&p, 1, 1, 1);                                              /* online: never a takeover, never an unpause request */
    CHECK(p.paused && !p.takeover && !at_pause_request_unpause(&p, 1));
    at_pause_on(&p, 0, 0, 0);                                              /* takeover not wanted (the default) */
    CHECK(!p.takeover);
    at_pause_on(&p, 1, 1, 0); CHECK(at_pause_request_unpause(&p, 0)); at_pause_off(&p);
    at_pause_on(&p, 1, 1, 0); CHECK(at_pause_take_unpause(&p) == -1);     /* a request never leaks into the next pause */
    CHECK(at_pause_request_unpause(&p, 0)); at_pause_on(&p, 3, 1, 0);       /* a new pause starts without the old request */
    CHECK(at_pause_take_unpause(&p) == -1);
}
int main(void) { names_and_parse(); sources_and_online(); pause_machine(); ATLAS_DONE("atlas-retail"); }
