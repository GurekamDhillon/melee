#include "gw_ui_policy.h"

#include <stdlib.h>
#include <string.h>

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

/* the scene kinds that end a run: values of GameSceneKind (src/melee/gm/forward.h:89..): TITLE 0, MENU 1, DEBUG_MENU 7, the toy rooms 0xB-0xD, REGEND_TOYFALL 0xF,
 * REGEND_CONGRATS 0x10, MOVIE_OPENING 0x1C, MOVIE_END 0x1D, MOVIE_HOWTO 0x1E, MOVIE_OMAKE15 0x1F, GAMEOVER 0x22, COMING_SOON 0x23, MEMCARD 0x2A, STAFFROLL 0x2B */
int at_policy_ends_run(int scene_kind)
{
    static const int ENDS[] = { 0, 1, 7, 0xB, 0xC, 0xD, 0xF, 0x10, 0x1C, 0x1D, 0x1E, 0x1F, 0x22, 0x23, 0x2A, 0x2B };
    size_t i;
    for (i = 0; i < sizeof ENDS / sizeof ENDS[0]; i++) if (ENDS[i] == scene_kind) return 1;
    return 0;
}

const char *at_policy_screen(int scene_kind)
{
    size_t i;
    for (i = 0; i < sizeof ROWS / sizeof ROWS[0]; i++) if (ROWS[i].scene_kind == scene_kind) return ROWS[i].screen;
    return NULL;
}
