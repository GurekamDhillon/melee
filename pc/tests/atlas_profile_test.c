#include "atlas_check.h"
#include "../platform/gw_ui_css_profile.h"

int main(void)
{
    int mt, i;
    /* every retail match type 0x0 to 0x17 has a profile; nothing else does except the lobby */
    for (mt = 0; mt <= 0x17; mt++) CHECK(at_css_profile(mt) != NULL);
    CHECK(at_css_profile(0x18) == NULL && at_css_profile(-1) == NULL && at_css_profile(0xFF) == NULL);  /* unknown: retail screen stays */
    CHECK(at_css_profile(AT_MT_LOBBY) != NULL && at_css_profile(AT_MT_LOBBY)->online == 1);
    CHECK(at_css_profile_count() == 25 && at_css_profile_at(-1) == NULL && at_css_profile_at(25) == NULL && at_css_profile_at(0) == at_css_profile(0));
    for (i = 0; i < at_css_profile_count(); i++) CHECK(at_css_profile_at(i)->roster_filter == 0);        /* Event does not filter the roster (Task 0) */

    /* VS family (0x0 to 0xA, shares gmVsMelee_CssData): four humans, CPU cards, teams from the rules, two fighters, a stage select */
    for (mt = 0; mt <= 0xA; mt++) {
        const AtCssProfile *p = at_css_profile(mt);
        CHECK(p->group == 1 && p->max_humans == 4 && p->cpu_cards == 1 && p->teams_from_rules == 1);
        CHECK(p->min_to_start == 2 && p->has_sss == 1 && p->entering_port_only == 0 && p->online == 0 && p->dummy_cpu == 0);
    }
    /* Classic, Adventure, All-Star (0xB to 0xD): one player, no CPU cards, one fighter, no stage select */
    for (mt = 0xB; mt <= 0xD; mt++) {
        const AtCssProfile *p = at_css_profile(mt);
        CHECK(p->group == 2 && p->max_humans == 1 && p->cpu_cards == 0 && p->min_to_start == 1);
        CHECK(p->entering_port_only == 1 && p->has_sss == 0 && p->teams_from_rules == 0);
    }
    CHECK(at_css_profile(0xE)->group == 3 && at_css_profile(0xE)->entering_port_only == 1 && at_css_profile(0xE)->max_humans == 1);   /* Event */
    for (mt = 0xF; mt <= 0x16; mt++) {                                                                      /* Stadium */
        const AtCssProfile *p = at_css_profile(mt);
        CHECK(p->group == 4 && p->max_humans == 1 && p->has_sss == 0 && p->entering_port_only == 1 && p->cpu_cards == 0 && p->min_to_start == 1);
    }
    /* Training: the human plus the CPU dummy, with a stage select; it is in group 1 because it already runs on the kit */
    { const AtCssProfile *p = at_css_profile(0x17);
      CHECK(p->group == 1 && p->dummy_cpu == 1 && p->max_humans == 1 && p->has_sss == 1 && p->cpu_cards == 0 && p->min_to_start == 2); }
    /* the lobby: one card, one fighter is enough, no CPU, no teams */
    { const AtCssProfile *p = at_css_profile(AT_MT_LOBBY);
      CHECK(p->group == 5 && p->max_humans == 1 && p->min_to_start == 1 && p->cpu_cards == 0 && p->teams_from_rules == 0 && p->online == 1); }
    /* the table has no duplicate key */
    for (i = 0; i < at_css_profile_count(); i++) { int j; for (j = i + 1; j < at_css_profile_count(); j++) CHECK(at_css_profile_at(i)->match_type != at_css_profile_at(j)->match_type); }
    ATLAS_DONE("atlas profile");
}
