#include "gw_ui_policy.h"

#include <stdlib.h>
#include <string.h>

#include "gw_ui_retail_ids.h"

#define AT_SCENE_TITLE 0   /* GS_TITLE: the first value of GameSceneKind (src/melee/gm/forward.h); gmfrontend_atlas.inc asserts it */

static const AtPolicyRow ROWS[] = { { AT_SCENE_TITLE, AT_POLICY_OVERLAY, "title" } };

/* "<kind>:<word>" pairs separated by commas; the last one that names this kind wins */
static int override_for(int scene_kind, const char *env, int *found)
{
    const char *p = env;
    int result = AT_POLICY_RETAIL;
    *found = 0;
    while (p != NULL && *p != '\0') {
        char *end = NULL;
        long k = strtol(p, &end, 10);
        if (end != p && *end == ':') {
            const char *w = end + 1;
            size_t n = 0;
            int pol = -1;
            while (w[n] != '\0' && w[n] != ',') n++;
            if (n == 6 && strncmp(w, "retail", 6) == 0) pol = AT_POLICY_RETAIL;
            else if (n == 7 && strncmp(w, "overlay", 7) == 0) pol = AT_POLICY_OVERLAY;
            else if (n == 7 && strncmp(w, "replace", 7) == 0) pol = AT_POLICY_REPLACE;
            if (pol >= 0 && k == (long) scene_kind) { result = pol; *found = 1; }
            p = w + n;
        }
        while (*p != '\0' && *p != ',') p++;
        if (*p == ',') p++;
    }
    return result;
}

int at_policy_for(int scene_kind, int atlas_on, const char *env_override)
{
    size_t i;
    int found, pol;
    if (!atlas_on) return AT_POLICY_RETAIL;
    pol = override_for(scene_kind, env_override, &found);
    if (found) return pol;
    for (i = 0; i < sizeof ROWS / sizeof ROWS[0]; i++) if (ROWS[i].scene_kind == scene_kind) return ROWS[i].policy;
    return AT_POLICY_RETAIL;
}

const char *at_policy_screen(int scene_kind)
{
    size_t i;
    for (i = 0; i < sizeof ROWS / sizeof ROWS[0]; i++) if (ROWS[i].scene_kind == scene_kind) return ROWS[i].screen;
    return NULL;
}

/* the bespoke scenes: what each hides under an OVERLAY, which of those bits are decoded text, and the chrome screen's id */
static const struct { int kind; unsigned mask; unsigned text_bits; const char *screen; } BESPOKE[] = {
    { 11, (1u << AT_RE_TOY_PANEL) | (1u << AT_RE_TOY_INFO) | (1u << AT_RE_TOY_TEXT), 1u << AT_RE_TOY_TEXT, "toy.gallery" },   /* GS_TOY_GALLERY */
    { 12, 0, 0, "toy.lottery" },                                                                                              /* GS_TOY_LOTTERY */
    { 13, 0, 0, "toy.collection" },                                                                                           /* GS_TOY_COLLECTION */
};

unsigned at_policy_mask(int scene_kind, int atlas_on, const char *env_override, int text_ok)
{
    size_t i;
    if (at_policy_for(scene_kind, atlas_on, env_override) != AT_POLICY_OVERLAY) return 0;
    for (i = 0; i < sizeof BESPOKE / sizeof BESPOKE[0]; i++)
        if (BESPOKE[i].kind == scene_kind) return text_ok ? BESPOKE[i].mask : (BESPOKE[i].mask & ~BESPOKE[i].text_bits);
    return 0;
}

const char *at_policy_screen_for(int scene_kind)
{
    size_t i;
    for (i = 0; i < sizeof BESPOKE / sizeof BESPOKE[0]; i++) if (BESPOKE[i].kind == scene_kind) return BESPOKE[i].screen;
    return at_policy_screen(scene_kind);
}
