/* gw_ui_retail.c - the retail element mask and the retail pause, as Atlas sees them (spec 6.8). Pure C. */
#include "gw_ui_retail.h"
#include <stdio.h>
#include <string.h>

static const char *const NAMES[AT_RE_COUNT] = { "hud.damage", "hud.stock", "hud.timer", "hud.nametag", "hud.magnify",
                                                "hud.coin", "hud.prize", "hud.hazard", "pause.panel",
                                                "toy.panel", "toy.info", "toy.text" };
const char *at_retail_name(int id) { return (id >= 0 && id < AT_RE_COUNT) ? NAMES[id] : NULL; }

int at_retail_parse(const char *list, unsigned *mask, char *err, int cap)
{
    char word[32]; int n = 0, i; unsigned m = 0; const char *p = list ? list : "";
    for (;;) {
        char c = *p;
        if (c == ',' || c == ' ' || c == '\0') {
            if (n > 0) {
                word[n] = '\0';
                if (strcmp(word, "all") == 0) m = (1u << AT_RE_COUNT) - 1;
                else {
                    for (i = 0; i < AT_RE_COUNT && strcmp(word, NAMES[i]) != 0; i++) {}
                    if (i == AT_RE_COUNT) { snprintf(err, (size_t) cap, "unknown retail element \"%s\"", word); return 0; }
                    m |= 1u << i;
                }
                n = 0;
            }
            if (c == '\0') break;
        } else if (n < (int) sizeof word - 1) word[n++] = c;
        p++;
    }
    *mask = m;
    return 1;
}
#define AT_RE_TOY_BITS ((1u << AT_RE_TOY_PANEL) | (1u << AT_RE_TOY_INFO) | (1u << AT_RE_TOY_TEXT))
unsigned at_retail_script_allowed(void) { return ((1u << AT_RE_COUNT) - 1) & ~(1u << AT_RE_HUD_TIMER) & ~AT_RE_TOY_BITS; }   /* the clock and the trophy scenes' pieces are policy-only */
void at_retail_set(AtRetail *r, int source, unsigned mask) { if (source >= 0 && source < AT_RS_COUNT) r->src[source] = mask & ((1u << AT_RE_COUNT) - 1); }
unsigned at_retail_effective(const AtRetail *r, int online)
{
    unsigned m = 0; int i;
    if (online) return 0;
    for (i = 0; i < AT_RS_COUNT; i++) m |= r->src[i];
    return m;
}
int at_retail_hidden(const AtRetail *r, int id, int online) { return id >= 0 && id < AT_RE_COUNT && (at_retail_effective(r, online) >> id & 1u); }
void at_retail_release_script(AtRetail *r, int owner) { if (owner > 0 && r->script_owner == owner) { r->src[AT_RS_SCRIPT] = 0; r->script_owner = 0; } }

void at_pause_on(AtPause *p, int pauser, int takeover_wanted, int online)
{ p->paused = 1; p->pauser = pauser; p->takeover = takeover_wanted && !online; p->unpause_req = 0; }
void at_pause_off(AtPause *p) { p->paused = 0; p->pauser = -1; p->takeover = 0; p->unpause_req = 0; }
int at_pause_request_unpause(AtPause *p, int online)
{ if (!p->paused || online || p->unpause_req) return 0; p->unpause_req = 1; return 1; }
int at_pause_take_unpause(AtPause *p)
{ if (!p->paused || !p->unpause_req) return -1; p->unpause_req = 0; return p->pauser; }
