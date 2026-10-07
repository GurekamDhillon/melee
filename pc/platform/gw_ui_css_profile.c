/* gw_ui_css_profile.c - what each mode's character select is allowed to do. Pure C, no game types.
 * Columns: match_type, group, max_humans, cpu_cards, teams_from_rules, min_to_start, entering_port_only, dummy_cpu, has_sss, online, roster_filter.
 * tools/port/test_css_profiles.py keeps the first column equal to the retail CSSMatchType enum (src/melee/mn/types.h). */
#include "gw_ui_css_profile.h"

static const AtCssProfile T[] = {
    /* VS family: gmVsMelee_CssData; Melee, Camera, Stamina, Sudden Death, Giant, Tiny, Invisible, Fixed Camera, Single Button, Lightning, Slo-Mo */
    { 0x0, 1, 4, 1, 1, 2, 0, 0, 1, 0, 0 },
    { 0x1, 1, 4, 1, 1, 2, 0, 0, 1, 0, 0 },
    { 0x2, 1, 4, 1, 1, 2, 0, 0, 1, 0, 0 },
    { 0x3, 1, 4, 1, 1, 2, 0, 0, 1, 0, 0 },
    { 0x4, 1, 4, 1, 1, 2, 0, 0, 1, 0, 0 },
    { 0x5, 1, 4, 1, 1, 2, 0, 0, 1, 0, 0 },
    { 0x6, 1, 4, 1, 1, 2, 0, 0, 1, 0, 0 },
    { 0x7, 1, 4, 1, 1, 2, 0, 0, 1, 0, 0 },
    { 0x8, 1, 4, 1, 1, 2, 0, 0, 1, 0, 0 },
    { 0x9, 1, 4, 1, 1, 2, 0, 0, 1, 0, 0 },
    { 0xA, 1, 4, 1, 1, 2, 0, 0, 1, 0, 0 },
    /* one-player: Classic, Adventure, All-Star */
    { 0xB, 2, 1, 0, 0, 1, 1, 0, 0, 0, 0 },
    { 0xC, 2, 1, 0, 0, 1, 1, 0, 0, 0, 0 },
    { 0xD, 2, 1, 0, 0, 1, 1, 0, 0, 0, 0 },
    /* Event (the retail Event setup only presets the opponents in the preload cache: it filters nothing the player picks) */
    { 0xE, 3, 1, 0, 0, 1, 1, 0, 0, 0, 0 },
    /* Stadium: Target Test, Home-Run, Multi-Man 10 and 100, 3-Minute, 15-Minute, Endless, Cruel */
    { 0xF, 4, 1, 0, 0, 1, 1, 0, 0, 0, 0 },
    { 0x10, 4, 1, 0, 0, 1, 1, 0, 0, 0, 0 },
    { 0x11, 4, 1, 0, 0, 1, 1, 0, 0, 0, 0 },
    { 0x12, 4, 1, 0, 0, 1, 1, 0, 0, 0, 0 },
    { 0x13, 4, 1, 0, 0, 1, 1, 0, 0, 0, 0 },
    { 0x14, 4, 1, 0, 0, 1, 1, 0, 0, 0, 0 },
    { 0x15, 4, 1, 0, 0, 1, 1, 0, 0, 0, 0 },
    { 0x16, 4, 1, 0, 0, 1, 1, 0, 0, 0, 0 },
    /* Training: the human plus the CPU dummy (its states: 0 CSS, 1 SSS, 2 the match) */
    { 0x17, 1, 1, 0, 0, 2, 0, 1, 1, 0, 0 },
    /* the online lobby's pick (the 'lobby' profile; not a retail value) */
    { 0x40, 5, 1, 0, 0, 1, 0, 0, 0, 1, 0 },
};

#define T_COUNT ((int) (sizeof T / sizeof T[0]))

const AtCssProfile *at_css_profile(int match_type)
{
    int i;
    for (i = 0; i < T_COUNT; i++) if (T[i].match_type == match_type) return &T[i];
    return 0;
}
int at_css_profile_count(void) { return T_COUNT; }
const AtCssProfile *at_css_profile_at(int i) { return i >= 0 && i < T_COUNT ? &T[i] : 0; }
